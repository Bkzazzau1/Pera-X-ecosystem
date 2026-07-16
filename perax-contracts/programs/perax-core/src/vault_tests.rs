use super::*;

fn test_config(vault_class: VaultClass) -> ReserveVaultConfig {
    ReserveVaultConfig {
        state: Pubkey::new_unique(),
        allocation_id: ALLOCATION_COMMUNITY_REWARDS,
        vault_class,
        token_mint: Pubkey::new_unique(),
        vault_authority: Pubkey::new_unique(),
        vault_token_account: Pubkey::new_unique(),
        authorized_source_owner: Pubkey::new_unique(),
        authorized_source_token_account: Pubkey::new_unique(),
        approved_destination_owner: Pubkey::new_unique(),
        approved_destination_token_account: Pubkey::new_unique(),
        allocation_cap: 1_000,
        authorized_deposited: 1_000,
        unsolicited_balance: 0,
        total_released: 100,
        is_active: true,
        is_paused: false,
        authority_bump: 255,
        config_bump: 254,
    }
}

#[test]
fn reserve_vault_space_matches_hardened_fields() {
    assert_eq!(ReserveVaultConfig::SPACE, (9 * 32) + 1 + (4 * 8) + 4);
}

#[test]
fn all_approved_allocations_resolve_to_expected_classes() {
    let cases = [
        (ALLOCATION_LIQUIDITY_POOL, VaultClass::Liquidity),
        (ALLOCATION_COMMUNITY_REWARDS, VaultClass::CommunityRewards),
        (ALLOCATION_TREASURY, VaultClass::MarketReserve),
        (ALLOCATION_ECOSYSTEM_MARKETING, VaultClass::MarketReserve),
        (ALLOCATION_TRADING_OPERATIONS, VaultClass::Operations),
        (ALLOCATION_DEVELOPMENT_TEAM, VaultClass::Vesting),
        (ALLOCATION_FOUNDER, VaultClass::Vesting),
        (ALLOCATION_FUTURE_TEAM_INCENTIVES, VaultClass::MarketReserve),
        (
            ALLOCATION_TEAM_EMERGENCY_RESERVE,
            VaultClass::EmergencyReserve,
        ),
        (ALLOCATION_PRIVATE_STRATEGIC, VaultClass::Vesting),
        (ALLOCATION_ADVISOR_1, VaultClass::Vesting),
        (ALLOCATION_ADVISOR_2, VaultClass::Vesting),
        (ALLOCATION_ADVISOR_3, VaultClass::Vesting),
    ];

    for (allocation_id, expected_class) in cases {
        let (actual_class, cap) = approved_allocation(allocation_id).unwrap();
        assert_eq!(actual_class, expected_class);
        assert!(cap > 0);
    }
}

#[test]
fn unknown_allocation_is_rejected() {
    assert!(approved_allocation([255u8; 32]).is_err());
}

#[test]
fn authorized_accounting_caps_available_balance() {
    let config = test_config(VaultClass::CommunityRewards);

    assert_eq!(calculate_vault_available_amount(&config, 900).unwrap(), 900);
    assert_eq!(calculate_vault_available_amount(&config, 950).unwrap(), 900);
}

#[test]
fn tracked_unsolicited_balance_is_never_releasable() {
    let mut config = test_config(VaultClass::CommunityRewards);
    config.unsolicited_balance = 50;

    assert_eq!(calculate_vault_available_amount(&config, 950).unwrap(), 900);
    assert_eq!(calculate_vault_available_amount(&config, 925).unwrap(), 875);
}

#[test]
fn accounting_rejects_released_amount_above_authorized_deposits() {
    let mut config = test_config(VaultClass::CommunityRewards);
    config.total_released = 1_001;

    assert!(calculate_vault_available_amount(&config, 1_000).is_err());
}

#[test]
fn growth_release_classes_are_strictly_limited() {
    assert!(
        validate_vault_class_for_release(VaultClass::MarketReserve, ReleaseType::Growth).is_ok()
    );
    assert!(validate_vault_class_for_release(VaultClass::Operations, ReleaseType::Growth).is_ok());
    assert!(
        validate_vault_class_for_release(VaultClass::CommunityRewards, ReleaseType::Growth).is_ok()
    );
    assert!(validate_vault_class_for_release(VaultClass::Liquidity, ReleaseType::Growth).is_err());
    assert!(validate_vault_class_for_release(VaultClass::Vesting, ReleaseType::Growth).is_err());
    assert!(
        validate_vault_class_for_release(VaultClass::EmergencyReserve, ReleaseType::Growth)
            .is_err()
    );
}

#[test]
fn emergency_release_is_limited_to_emergency_reserve() {
    assert!(
        validate_vault_class_for_release(VaultClass::EmergencyReserve, ReleaseType::Emergency)
            .is_ok()
    );
    assert!(
        validate_vault_class_for_release(VaultClass::MarketReserve, ReleaseType::Emergency)
            .is_err()
    );
}

#[test]
fn program_derived_destination_is_rejected() {
    let program_id = Pubkey::new_unique();
    let (pda, _) = Pubkey::find_program_address(&[b"destination-test"], &program_id);
    assert!(is_program_derived_destination(pda));
}
