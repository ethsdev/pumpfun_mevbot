#[cfg(test)]
mod tests {
    use super::*;
    use solana_program::{
        clock::Epoch,
        program_error::ProgramError,
        pubkey::Pubkey,
        rent::Rent,
        sysvar::Sysvar,
    };
    use std::cell::RefCell;
    use solana_program::account_info::AccountInfo;

    fn create_account(pubkey: Pubkey, is_signer: bool, is_writable: bool, lamports: u64, data: &mut [u8]) -> AccountInfo {
        AccountInfo::new(
            &pubkey,
            is_signer,
            is_writable,
            &mut lamports.clone(),
            data,
            &Pubkey::default(),
            false,
            Epoch::default(),
        )
    }

    #[test]
    fn test_initialize_success() {
        let program_id = Pubkey::new_unique();
        let owner = Pubkey::new_unique();
        let mut state_data = vec![0u8; DexSlippage::LEN];

        let accounts = vec![
            create_account(owner, true, true, 0, &mut state_data),
        ];

        let instruction_data = DexSlippage {
            owner,
            arb_tx_price: 42,
            enable_trading: true,
            token_pair: 1,
            trading_balance_in_tokens: 1000,
            is_slippage_set: true,
            slippage_percent: 5,
            mev_enabled: true,
            liquidity_threshold: 500,
        }
        .try_to_vec()
        .unwrap();

        let result = initialize(&program_id, &accounts, &instruction_data);
        assert!(result.is_ok());

        // Optionally deserialize and verify the state
        let state: DexSlippage = DexSlippage::try_from_slice(&state_data).unwrap();
        assert_eq!(state.owner, owner);
        assert_eq!(state.arb_tx_price, 42);
        assert_eq!(state.slippage_percent, 5);
    }

    #[test]
    fn test_initialize_fails_if_not_signer() {
        let program_id = Pubkey::new_unique();
        let owner = Pubkey::new_unique();
        let mut state_data = vec![0u8; DexSlippage::LEN];

        // is_signer = false
        let accounts = vec![
            create_account(owner, false, true, 0, &mut state_data),
        ];

        let instruction_data = DexSlippage {
            owner,
            ..Default::default()
        }
        .try_to_vec()
        .unwrap();

        let result = initialize(&program_id, &accounts, &instruction_data);
        assert_eq!(result, Err(ProgramError::MissingRequiredSignature));
    }

    #[test]
    fn test_set_slippage_success_and_failure() {
        let program_id = Pubkey::new_unique();
        let owner = Pubkey::new_unique();
        let mut state_data = vec![0u8; DexSlippage::LEN];

        let accounts = vec![
            create_account(owner, true, true, 0, &mut state_data),
            create_account(Pubkey::new_unique(), false, true, 0, &mut state_data),
        ];

        // success
        assert!(set_slippage(&program_id, &accounts, 3).is_ok());

        let state: DexSlippage = DexSlippage::try_from_slice(&state_data).unwrap();
        assert_eq!(state.slippage_percent, 3);

        // failure if slippage > 100%
        let result = set_slippage(&program_id, &accounts, 150);
        assert!(result.is_err());
    }

    #[test]
    fn test_enable_mev_flag() {
        let program_id = Pubkey::new_unique();
        let owner = Pubkey::new_unique();
        let mut state_data = vec![0u8; DexSlippage::LEN];

        let accounts = vec![
            create_account(owner, true, true, 0, &mut state_data),
            create_account(Pubkey::new_unique(), false, true, 0, &mut state_data),
        ];

        assert!(enable_mev(&program_id, &accounts, true).is_ok());

        let state: DexSlippage = DexSlippage::try_from_slice(&state_data).unwrap();
        assert!(state.mev_enabled);
    }

    #[test]
    fn test_set_liquidity_threshold_invalid_value() {
        let program_id = Pubkey::new_unique();
        let owner = Pubkey::new_unique();
        let mut state_data = vec![0u8; DexSlippage::LEN];

        let accounts = vec![
            create_account(owner, true, true, 0, &mut state_data),
            create_account(Pubkey::new_unique(), false, true, 0, &mut state_data),
        ];

        // Set a valid threshold
        assert!(set_liquidity_threshold(&program_id, &accounts, 1000).is_ok());

        // Set a negative threshold (simulated with overflow value)
        let result = set_liquidity_threshold(&program_id, &accounts, u64::MAX);
        assert!(result.is_err());
    }

    #[test]
    fn test_withdraw_funds_unauthorized() {
        let program_id = Pubkey::new_unique();
        let bad_owner = Pubkey::new_unique();
        let mut state_data = vec![0u8; DexSlippage::LEN];

        let accounts = vec![
            create_account(bad_owner, false, false, 0, &mut state_data),
            create_account(Pubkey::new_unique(), false, true, 0, &mut state_data),
            create_account(Pubkey::new_unique(), false, true, 0, &mut state_data),
        ];

        let result = withdraw_funds(&program_id, &accounts);
        assert!(result.is_err());
    }
}
