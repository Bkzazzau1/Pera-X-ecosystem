use super::*;

fn test_state(max_payment_amount: u64) -> PeraxState {
    PeraxState {
        authority: Pubkey::new_unique(),
        pending_authority: Pubkey::default(),
        has_pending_authority: false,
        token_mint: Pubkey::new_unique(),
        trading_company_token_account: Pubkey::new_unique(),
        max_payment_amount,
        is_paused: false,
        bump: 255,
    }
}

#[test]
fn accepts_valid_payment_amount_without_limit() {
    let state = test_state(0);
    assert!(validate_payment_amount(&state, 1).is_ok());
    assert!(validate_payment_amount(&state, u64::MAX).is_ok());
}

#[test]
fn rejects_zero_payment_amount() {
    let state = test_state(0);
    let result = validate_payment_amount(&state, 0);
    assert!(result.is_err());
}

#[test]
fn enforces_max_payment_amount_when_configured() {
    let state = test_state(1_000);
    assert!(validate_payment_amount(&state, 1_000).is_ok());
    assert!(validate_payment_amount(&state, 1_001).is_err());
}

#[test]
fn accepts_non_zero_references() {
    let mut reference = [0u8; 32];
    reference[31] = 1;
    assert!(validate_reference(reference).is_ok());
}

#[test]
fn rejects_zero_references() {
    assert!(validate_reference([0u8; 32]).is_err());
}

#[test]
fn payment_record_space_matches_account_fields() {
    let expected_space = 32 + 32 + 8 + 32 + 32 + 8 + 1;
    assert_eq!(PaymentRecord::SPACE, expected_space);
}
