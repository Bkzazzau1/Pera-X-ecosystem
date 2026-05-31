use super::*;

fn test_state(max_payment_amount: u64) -> PeraxState {
    PeraxState {
        authority: Pubkey::new_unique(),
        pending_authority: Pubkey::default(),
        has_pending_authority: false,
        token_mint: Pubkey::new_unique(),
        trading_company_token_account: Pubkey::new_unique(),
        trading_company_revenue_token_account: Pubkey::new_unique(),
        max_payment_amount,
        safety_admin: Pubkey::new_unique(),
        oracle_feed: Pubkey::new_unique(),
        launch_price: PEX_LAUNCH_PRICE_SCALED,
        current_stepped_floor: PEX_LAUNCH_PRICE_SCALED,
        last_release_timestamp: 0,
        daily_unlocked_accumulator: 0,
        monthly_unlocked_accumulator: 0,
        daily_window_start: 0,
        monthly_window_start: 0,
        daily_release_cap: DAILY_RELEASE_CAP,
        monthly_release_cap: MONTHLY_RELEASE_CAP,
        emergency_hourly_release_bps: EMERGENCY_HOURLY_RESERVE_RELEASE_BPS,
        daily_burn_accumulator: 0,
        daily_burn_window_start: 0,
        is_paused: false,
        emergency_pause: false,
        bump: 255,
    }
}

fn growth_release_params(
    requested_amount: u64,
    observed_at: i64,
) -> MarketConditionalReleaseParams {
    let mut release_id = [0u8; 32];
    release_id[31] = 1;

    MarketConditionalReleaseParams {
        release_type: ReleaseType::Growth,
        requested_amount,
        release_id,
        snapshot: MarketConditionSnapshot {
            observed_price: PEX_LAUNCH_PRICE_SCALED * GROWTH_PRICE_MULTIPLIER,
            twap_minutes: MIN_GROWTH_TWAP_MINUTES,
            liquidity_usd: MIN_GROWTH_LIQUIDITY_USD,
            net_buy_volume_bps: MIN_NET_BUY_VOLUME_BPS,
            downside_move_bps: 0,
            liquidity_drain_bps: 0,
            emergency_reserve_available_amount: 0,
            observed_at,
        },
    }
}

fn emergency_release_params(
    requested_amount: u64,
    reserve_amount: u64,
    observed_at: i64,
) -> MarketConditionalReleaseParams {
    let mut release_id = [0u8; 32];
    release_id[31] = 2;

    MarketConditionalReleaseParams {
        release_type: ReleaseType::Emergency,
        requested_amount,
        release_id,
        snapshot: MarketConditionSnapshot {
            observed_price: PEX_LAUNCH_PRICE_SCALED,
            twap_minutes: 0,
            liquidity_usd: 0,
            net_buy_volume_bps: 0,
            downside_move_bps: EMERGENCY_DOWNSIDE_TRIGGER_BPS,
            liquidity_drain_bps: EMERGENCY_LIQUIDITY_DRAIN_TRIGGER_BPS,
            emergency_reserve_available_amount: reserve_amount,
            observed_at,
        },
    }
}

fn market_burn_params(
    eligible_revenue_amount: u64,
    burn_rate_bps: u16,
    market_health_score: u8,
) -> MarketConditionBurnParams {
    let mut decision_id = [0u8; 32];
    decision_id[31] = 3;

    MarketConditionBurnParams {
        amount: amount_bps(eligible_revenue_amount, burn_rate_bps).unwrap(),
        eligible_revenue_amount,
        burn_rate_bps,
        market_health_score,
        observed_at: 1_000_000,
        decision_id,
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
    let expected_space = 32 + 32 + 8 + 32 + 32 + 32 + 8 + 1;
    assert_eq!(PaymentRecord::SPACE, expected_space);
}

#[test]
fn release_record_space_matches_account_fields() {
    let expected_space = 32 + 32 + 1 + 8 + 8 + 8 + 8 + 2 + 8 + 8 + 1;
    assert_eq!(ReleaseRecord::SPACE, expected_space);
}

#[test]
fn growth_release_passes_when_all_market_gates_are_met() {
    let state = test_state(0);
    let params = growth_release_params(1_000_000 * PEX_DECIMALS, 1_000_000);

    assert!(validate_growth_release(&state, &params).is_ok());
}

#[test]
fn growth_release_rejects_when_price_is_below_3x_launch() {
    let state = test_state(0);
    let mut params = growth_release_params(1_000_000 * PEX_DECIMALS, 1_000_000);
    params.snapshot.observed_price = (PEX_LAUNCH_PRICE_SCALED * GROWTH_PRICE_MULTIPLIER) - 1;

    assert!(validate_growth_release(&state, &params).is_err());
}

#[test]
fn growth_release_rejects_when_twap_is_under_60_minutes() {
    let state = test_state(0);
    let mut params = growth_release_params(1_000_000 * PEX_DECIMALS, 1_000_000);
    params.snapshot.twap_minutes = MIN_GROWTH_TWAP_MINUTES - 1;

    assert!(validate_growth_release(&state, &params).is_err());
}

#[test]
fn growth_release_rejects_when_liquidity_is_below_3x_initial() {
    let state = test_state(0);
    let mut params = growth_release_params(1_000_000 * PEX_DECIMALS, 1_000_000);
    params.snapshot.liquidity_usd = MIN_GROWTH_LIQUIDITY_USD - 1;

    assert!(validate_growth_release(&state, &params).is_err());
}

#[test]
fn growth_release_rejects_when_buy_pressure_is_below_50_percent() {
    let state = test_state(0);
    let mut params = growth_release_params(1_000_000 * PEX_DECIMALS, 1_000_000);
    params.snapshot.net_buy_volume_bps = MIN_NET_BUY_VOLUME_BPS - 1;

    assert!(validate_growth_release(&state, &params).is_err());
}

#[test]
fn growth_release_rejects_during_24_hour_cooldown() {
    let mut state = test_state(0);
    state.last_release_timestamp = 1_000_000;
    let params = growth_release_params(
        1_000_000 * PEX_DECIMALS,
        1_000_000 + RELEASE_COOLDOWN_SECONDS - 1,
    );

    assert!(validate_growth_release(&state, &params).is_err());
}

#[test]
fn growth_release_rejects_above_daily_cap() {
    let state = test_state(0);
    let params = growth_release_params(DAILY_RELEASE_CAP + 1, 1_000_000);

    assert!(validate_growth_release(&state, &params).is_err());
}

#[test]
fn emergency_release_passes_when_stress_gates_and_hourly_cap_are_met() {
    let state = test_state(0);
    let reserve_amount = 10_000_000 * PEX_DECIMALS;
    let requested_amount =
        amount_bps(reserve_amount, EMERGENCY_HOURLY_RESERVE_RELEASE_BPS).unwrap();
    let params = emergency_release_params(requested_amount, reserve_amount, 1_000_000);

    assert!(validate_emergency_release(&state, &params).is_ok());
}

#[test]
fn emergency_release_rejects_when_downside_trigger_is_not_met() {
    let state = test_state(0);
    let reserve_amount = 10_000_000 * PEX_DECIMALS;
    let requested_amount =
        amount_bps(reserve_amount, EMERGENCY_HOURLY_RESERVE_RELEASE_BPS).unwrap();
    let mut params = emergency_release_params(requested_amount, reserve_amount, 1_000_000);
    params.snapshot.downside_move_bps = EMERGENCY_DOWNSIDE_TRIGGER_BPS - 1;

    assert!(validate_emergency_release(&state, &params).is_err());
}

#[test]
fn emergency_release_rejects_above_hourly_cap() {
    let state = test_state(0);
    let reserve_amount = 10_000_000 * PEX_DECIMALS;
    let requested_amount =
        amount_bps(reserve_amount, EMERGENCY_HOURLY_RESERVE_RELEASE_BPS).unwrap() + 1;
    let params = emergency_release_params(requested_amount, reserve_amount, 1_000_000);

    assert!(validate_emergency_release(&state, &params).is_err());
}

#[test]
fn release_windows_reset_after_day_and_month_boundaries() {
    let mut state = test_state(0);
    state.daily_window_start = 1;
    state.monthly_window_start = 1;
    state.daily_unlocked_accumulator = 100;
    state.monthly_unlocked_accumulator = 100;

    reset_release_windows_if_needed(&mut state, 2_592_001);

    assert_eq!(state.daily_unlocked_accumulator, 0);
    assert_eq!(state.monthly_unlocked_accumulator, 0);
}

#[test]
fn market_burn_uses_market_health_rate_before_conservation() {
    let state = test_state(0);
    let params = market_burn_params(1_000_000 * PEX_DECIMALS, DEFAULT_BURN_RATE_BPS, 50);

    assert!(validate_market_condition_burn(&state, &params, PEX_TOTAL_SUPPLY).is_ok());
}

#[test]
fn market_burn_uses_conservation_rate_at_threshold() {
    let state = test_state(0);
    let conservation_threshold =
        amount_bps(PEX_TOTAL_SUPPLY, CONSERVATION_SUPPLY_THRESHOLD_BPS).unwrap();
    let params = market_burn_params(1_000_000 * PEX_DECIMALS, CONSERVATION_BURN_RATE_BPS, 50);

    assert!(validate_market_condition_burn(&state, &params, conservation_threshold).is_ok());
}

#[test]
fn market_burn_rejects_market_health_rate_during_conservation() {
    let state = test_state(0);
    let conservation_threshold =
        amount_bps(PEX_TOTAL_SUPPLY, CONSERVATION_SUPPLY_THRESHOLD_BPS).unwrap();
    let params = market_burn_params(1_000_000 * PEX_DECIMALS, DEFAULT_BURN_RATE_BPS, 50);

    assert!(validate_market_condition_burn(&state, &params, conservation_threshold).is_err());
}

#[test]
fn market_burn_enforces_daily_cap() {
    let mut state = test_state(0);
    state.daily_burn_accumulator = amount_bps(PEX_TOTAL_SUPPLY, EARLY_DAILY_BURN_CAP_BPS).unwrap();
    let params = market_burn_params(10, DEFAULT_BURN_RATE_BPS, 50);

    assert!(validate_market_condition_burn(&state, &params, PEX_TOTAL_SUPPLY).is_err());
}
