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

fn test_apc_params() -> InitializeApcParams {
    InitializeApcParams {
        quote_mint: Pubkey::new_unique(),
        approved_pool: Pubkey::new_unique(),
        approved_proceeds_owner: Pubkey::new_unique(),
        approved_proceeds_token_account: Pubkey::new_unique(),
        approved_recovery_program: Pubkey::new_unique(),
        price_scale: 100_000_000,
        first_activation_price: 3_600,
        minimum_band_interval_bps: 1_000,
        maximum_band_interval_bps: 4_000,
        maximum_observation_age_seconds: 300,
        maximum_future_clock_skew_seconds: 15,
        hourly_release_cap: 2_000_000 * PEX_DECIMALS,
        pump_window_release_cap: 8_000_000 * PEX_DECIMALS,
        pump_window_seconds: 21_600,
        minimum_counterweight_coverage_bps: 2_500,
        base_band_release_cap: 2_000_000 * PEX_DECIMALS,
        minimum_twap_minutes: 15,
        minimum_liquidity_usd: 13_680,
        minimum_volume_usd: 50_000,
        minimum_buy_pressure_bps: 5_000,
        risk_velocity_thresholds_bps: [2_000, 5_000, 10_000],
        risk_volatility_thresholds_bps: [1_000, 2_500, 5_000],
        risk_price_impact_thresholds_bps: [100, 300, 800],
        band_interval_bps_by_risk: [1_000, 1_500, 2_500, 4_000],
        band_release_bps_by_risk: [10_000, 8_000, 6_000, 4_000],
        cascade_reduction_bps: [10_000, 7_000, 4_500, 2_500],
        recovery_spending_cap: 1_000_000_000,
    }
}

fn test_apc_config() -> ApcConfig {
    let params = test_apc_params();
    ApcConfig {
        state: Pubkey::new_unique(),
        oracle_feed: Pubkey::new_unique(),
        quote_mint: params.quote_mint,
        approved_pool: params.approved_pool,
        approved_proceeds_owner: params.approved_proceeds_owner,
        approved_proceeds_token_account: params.approved_proceeds_token_account,
        approved_recovery_program: params.approved_recovery_program,
        price_scale: params.price_scale,
        first_activation_price: params.first_activation_price,
        minimum_band_interval_bps: params.minimum_band_interval_bps,
        maximum_band_interval_bps: params.maximum_band_interval_bps,
        maximum_observation_age_seconds: params.maximum_observation_age_seconds,
        maximum_future_clock_skew_seconds: params.maximum_future_clock_skew_seconds,
        hourly_release_cap: params.hourly_release_cap,
        pump_window_release_cap: params.pump_window_release_cap,
        pump_window_seconds: params.pump_window_seconds,
        minimum_counterweight_coverage_bps: params.minimum_counterweight_coverage_bps,
        base_band_release_cap: params.base_band_release_cap,
        minimum_twap_minutes: params.minimum_twap_minutes,
        minimum_liquidity_usd: params.minimum_liquidity_usd,
        minimum_volume_usd: params.minimum_volume_usd,
        minimum_buy_pressure_bps: params.minimum_buy_pressure_bps,
        risk_velocity_thresholds_bps: params.risk_velocity_thresholds_bps,
        risk_volatility_thresholds_bps: params.risk_volatility_thresholds_bps,
        risk_price_impact_thresholds_bps: params.risk_price_impact_thresholds_bps,
        band_interval_bps_by_risk: params.band_interval_bps_by_risk,
        band_release_bps_by_risk: params.band_release_bps_by_risk,
        cascade_reduction_bps: params.cascade_reduction_bps,
        recovery_spending_cap: params.recovery_spending_cap,
        is_active: true,
        is_paused: false,
        bump: 254,
    }
}

fn test_apc_state(config: Pubkey) -> ApcState {
    ApcState {
        config,
        status: ApcStatus::Active,
        status_before_pause: ApcStatus::Active,
        current_reference_price: 3_600,
        next_band_price: 3_960,
        current_band_index: 1,
        highest_crossed_band_index: 1,
        pump_window_started_at: 1_000,
        pump_window_released: 0,
        hourly_window_started_at: 1_000,
        hourly_released: 0,
        total_apc_released: 0,
        total_counterweight_credited: 0,
        total_counterweight_spent: 0,
        last_observation_sequence: 0,
        last_release_timestamp: 0,
        deferred_burn_amount: 0,
        unconfirmed_release_amount: 0,
        last_release_observation_id: [0u8; 32],
        recovery_entry_observation_id: [0u8; 32],
        cascade_observation_id: [0u8; 32],
        cascade_band_count: 0,
        active_risk_tier: 0,
        bump: 253,
    }
}

fn observation_params(
    config: &ApcConfig,
    sequence: u64,
    observed_at: i64,
) -> SubmitApcObservationParams {
    let mut observation_id = [0u8; 32];
    observation_id[31] = sequence as u8 + 1;
    SubmitApcObservationParams {
        observation_id,
        sequence,
        pool: config.approved_pool,
        spot_price: 4_000,
        twap_price: 3_900,
        twap_minutes: config.minimum_twap_minutes,
        liquidity_usd: config.minimum_liquidity_usd,
        quote_liquidity_usd: config.minimum_liquidity_usd / 2,
        volume_usd: config.minimum_volume_usd,
        net_buy_pressure_bps: config.minimum_buy_pressure_bps,
        price_velocity_bps: 2_000,
        volatility_bps: 1_000,
        estimated_price_impact_bps: 100,
        observed_at,
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
) -> ConditionalBuybackBurnParams {
    let mut decision_id = [0u8; 32];
    decision_id[31] = 3;
    ConditionalBuybackBurnParams {
        amount: amount_bps(eligible_revenue_amount, burn_rate_bps).unwrap(),
        eligible_revenue_amount,
        burn_rate_bps,
        market_health_score,
        observed_at: 1_000_000,
        decision_id,
        burn_source: BurnFulfillmentSource::OpenMarketPurchase,
    }
}

fn test_observation(
    config: &ApcConfig,
    observation_id: [u8; 32],
    observed_at: i64,
) -> ApcObservation {
    ApcObservation {
        observation_id,
        sequence: 2,
        oracle_feed: config.oracle_feed,
        pool: config.approved_pool,
        spot_price: 4_000,
        twap_price: 3_900,
        twap_minutes: config.minimum_twap_minutes,
        liquidity_usd: config.minimum_liquidity_usd,
        quote_liquidity_usd: config.minimum_liquidity_usd / 2,
        volume_usd: config.minimum_volume_usd,
        net_buy_pressure_bps: config.minimum_buy_pressure_bps,
        price_velocity_bps: 2_000,
        volatility_bps: 1_000,
        estimated_price_impact_bps: 100,
        observed_at,
        submitted_at: observed_at,
        is_consumed_for_release: false,
        is_consumed_for_confirmation: false,
        is_consumed_for_recovery: false,
        consumed_by_release: Pubkey::default(),
        consumed_by_recovery: Pubkey::default(),
        bump: 252,
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
    assert!(validate_payment_amount(&test_state(0), 0).is_err());
}

#[test]
fn enforces_max_payment_amount_when_configured() {
    let state = test_state(1_000);
    assert!(validate_payment_amount(&state, 1_000).is_ok());
    assert!(validate_payment_amount(&state, 1_001).is_err());
}

#[test]
fn reference_validation_rejects_zero_and_accepts_nonzero() {
    assert!(validate_reference([0u8; 32]).is_err());
    let mut reference = [0u8; 32];
    reference[31] = 1;
    assert!(validate_reference(reference).is_ok());
}

#[test]
fn legacy_growth_release_is_always_rejected() {
    let params = MarketConditionalReleaseParams {
        release_type: ReleaseType::Growth,
        requested_amount: 1,
        release_id: [1u8; 32],
        snapshot: MarketConditionSnapshot {
            observed_price: 3_600,
            twap_minutes: 60,
            liquidity_usd: 13_680,
            net_buy_volume_bps: 5_000,
            downside_move_bps: 0,
            liquidity_drain_bps: 0,
            emergency_reserve_available_amount: 0,
            observed_at: 1,
        },
    };
    assert!(validate_growth_release(&test_state(0), &params).is_err());
}

#[test]
fn first_activation_must_equal_exactly_three_times_launch() {
    let state = test_state(0);
    let params = test_apc_params();
    assert_eq!(params.first_activation_price, 3_600);
    assert!(validate_apc_policy(&state, &params).is_ok());

    let mut below = test_apc_params();
    below.first_activation_price = 3_599;
    assert!(validate_apc_policy(&state, &below).is_err());
}

#[test]
fn adaptive_interval_must_stay_inside_policy_bounds() {
    let mut config = test_apc_config();
    config.band_interval_bps_by_risk[0] = config.minimum_band_interval_bps - 1;
    assert!(calculate_band_interval_bps(&config, 0).is_err());

    let mut config = test_apc_config();
    config.band_interval_bps_by_risk[3] = config.maximum_band_interval_bps + 1;
    assert!(calculate_band_interval_bps(&config, 3).is_err());
}

#[test]
fn risk_and_interval_calculation_is_deterministic() {
    let config = test_apc_config();
    let tier = calculate_apc_risk_tier(
        5_000,
        1_100,
        120,
        config.risk_velocity_thresholds_bps,
        config.risk_volatility_thresholds_bps,
        config.risk_price_impact_thresholds_bps,
    );
    assert_eq!(tier, 2);
    assert_eq!(calculate_band_interval_bps(&config, tier).unwrap(), 2_500);
}

#[test]
fn band_price_math_uses_checked_ceil_rounding() {
    assert_eq!(calculate_next_band_price(3_600, 1_000).unwrap(), 3_960);
    assert_eq!(calculate_next_band_price(1, 1).unwrap(), 2);
    assert!(calculate_next_band_price(u64::MAX, 10_000).is_err());
}

#[test]
fn high_price_can_cross_multiple_sequential_bands_without_skipping() {
    let config = test_apc_config();
    let effective_price = 100_000;
    let mut trigger = config.first_activation_price;
    for index in 1..=6 {
        assert!(effective_price >= trigger);
        assert!(validate_sequential_band_index(index - 1, index).is_ok());
        trigger = calculate_next_band_price(trigger, config.band_interval_bps_by_risk[0]).unwrap();
    }
    assert!(validate_sequential_band_index(1, 3).is_err());
    assert!(validate_sequential_band_index(1, 1).is_err());
}

#[test]
fn cascade_tranches_decrease_and_never_restore() {
    let config = test_apc_config();
    let first = calculate_band_release_cap(&config, 0, 1).unwrap();
    let second = calculate_band_release_cap(&config, 0, 2).unwrap();
    let third = calculate_band_release_cap(&config, 0, 3).unwrap();
    let fourth = calculate_band_release_cap(&config, 0, 4).unwrap();
    let later = calculate_band_release_cap(&config, 0, 20).unwrap();
    assert!(first > second && second > third && third > fourth);
    assert_eq!(later, fourth);
}

#[test]
fn apc_caps_use_checked_addition() {
    let config = test_apc_config();
    let mut apc_state = test_apc_state(config.state);
    let mut core_state = test_state(0);
    let amount = 1_000;
    assert!(validate_apc_release_caps(&config, &apc_state, &core_state, 0, 2_000, amount).is_ok());

    apc_state.hourly_released = config.hourly_release_cap;
    assert!(validate_apc_release_caps(&config, &apc_state, &core_state, 0, 2_000, amount).is_err());

    apc_state.hourly_released = 0;
    apc_state.pump_window_released = config.pump_window_release_cap;
    assert!(validate_apc_release_caps(&config, &apc_state, &core_state, 0, 2_000, amount).is_err());

    apc_state.pump_window_released = 0;
    core_state.daily_unlocked_accumulator = core_state.daily_release_cap;
    assert!(validate_apc_release_caps(&config, &apc_state, &core_state, 0, 2_000, amount).is_err());
}

#[test]
fn unconfirmed_release_exposure_cannot_reset_with_time_windows() {
    let config = test_apc_config();
    let mut apc_state = test_apc_state(config.state);
    let core_state = test_state(0);
    apc_state.unconfirmed_release_amount = config.pump_window_release_cap - 500;
    apc_state.hourly_released = 0;
    apc_state.pump_window_released = 0;
    assert!(validate_apc_release_caps(
        &config,
        &apc_state,
        &core_state,
        0,
        config.base_band_release_cap,
        500
    )
    .is_ok());
    assert!(validate_apc_release_caps(
        &config,
        &apc_state,
        &core_state,
        0,
        config.base_band_release_cap,
        501
    )
    .is_err());
}

#[test]
fn recovery_swap_output_is_deterministic_checked_and_fee_aware() {
    let no_fee = calculate_recovery_pex_out(1_000_000, 2_000_000, 100_000, 0).unwrap();
    let with_fee = calculate_recovery_pex_out(1_000_000, 2_000_000, 100_000, 300).unwrap();
    assert_eq!(no_fee, 181_818);
    assert_eq!(with_fee, 176_845);
    assert!(with_fee < no_fee);
    assert!(calculate_recovery_pex_out(0, 2_000_000, 100_000, 300).is_err());
    assert!(calculate_recovery_pex_out(1_000_000, 2_000_000, 100_000, 1_001).is_err());
}

#[test]
fn observation_clock_and_sequence_security_is_enforced() {
    let config = test_apc_config();
    let mut state = test_apc_state(config.state);
    let now = 10_000;
    let valid = observation_params(&config, 1, now - 10);
    assert!(validate_apc_observation_submission(&config, &state, &valid, now).is_ok());

    let stale = observation_params(&config, 1, now - config.maximum_observation_age_seconds - 1);
    assert!(validate_apc_observation_submission(&config, &state, &stale, now).is_err());

    let future = observation_params(
        &config,
        1,
        now + config.maximum_future_clock_skew_seconds + 1,
    );
    assert!(validate_apc_observation_submission(&config, &state, &future, now).is_err());

    state.last_observation_sequence = 1;
    let repeated = observation_params(&config, 1, now);
    assert!(validate_apc_observation_submission(&config, &state, &repeated, now).is_err());
}

#[test]
fn counterweight_requirement_uses_real_quote_units() {
    let required =
        calculate_counterweight_requirement(1_000_000 * PEX_DECIMALS, 3_600, 100_000_000, 2_500)
            .unwrap();
    assert_eq!(required, 9_000_000); // $9.00 in six-decimal USDC units.
}

#[test]
fn absorption_confirmation_requires_a_distinct_fresh_supported_observation() {
    let config = test_apc_config();
    let mut state = test_apc_state(config.state);
    state.status = ApcStatus::AwaitingAbsorption;
    state.current_reference_price = 3_600;
    state.last_release_observation_id = [7u8; 32];
    let now = 10_000;
    let valid = test_observation(&config, [8u8; 32], now - 5);
    assert_eq!(
        validate_apc_absorption_confirmation(&config, &state, &valid, now).unwrap(),
        3_900
    );
    let reused = test_observation(&config, [7u8; 32], now - 5);
    assert!(validate_apc_absorption_confirmation(&config, &state, &reused, now).is_err());
    let mut below_support = test_observation(&config, [9u8; 32], now - 5);
    below_support.spot_price = 3_599;
    below_support.twap_price = 3_599;
    assert!(validate_apc_absorption_confirmation(&config, &state, &below_support, now).is_err());
    let stale = test_observation(
        &config,
        [10u8; 32],
        now - config.maximum_observation_age_seconds - 1,
    );
    assert!(validate_apc_absorption_confirmation(&config, &state, &stale, now).is_err());
}

#[test]
fn burn_is_deferred_during_pump_absorption_and_recovery() {
    let config = Pubkey::new_unique();
    let mut state = test_apc_state(config);
    for status in [
        ApcStatus::PumpControl,
        ApcStatus::AwaitingAbsorption,
        ApcStatus::Recovery,
    ] {
        state.status = status;
        assert!(validate_apc_burn_allowed(&state).is_err());
    }
    state.status = ApcStatus::Active;
    assert!(validate_apc_burn_allowed(&state).is_ok());
}

#[test]
fn emergency_release_still_succeeds_independently() {
    let state = test_state(0);
    let reserve_amount = 10_000_000 * PEX_DECIMALS;
    let requested_amount =
        amount_bps(reserve_amount, EMERGENCY_HOURLY_RESERVE_RELEASE_BPS).unwrap();
    let params = emergency_release_params(requested_amount, reserve_amount, 1_000_000);
    assert!(validate_emergency_release(&state, &params).is_ok());
}

#[test]
fn emergency_release_rejects_bad_stress_or_excess_amount() {
    let state = test_state(0);
    let reserve_amount = 10_000_000 * PEX_DECIMALS;
    let cap = amount_bps(reserve_amount, EMERGENCY_HOURLY_RESERVE_RELEASE_BPS).unwrap();
    let mut bad = emergency_release_params(cap, reserve_amount, 1_000_000);
    bad.snapshot.downside_move_bps = EMERGENCY_DOWNSIDE_TRIGGER_BPS - 1;
    assert!(validate_emergency_release(&state, &bad).is_err());
    assert!(validate_emergency_release(
        &state,
        &emergency_release_params(cap + 1, reserve_amount, 1_000_000)
    )
    .is_err());
}

#[test]
fn release_windows_reset_from_trusted_clock_input() {
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
fn market_burn_policy_remains_enforced() {
    let state = test_state(0);
    let params = market_burn_params(1_000_000 * PEX_DECIMALS, DEFAULT_BURN_RATE_BPS, 50);
    assert!(validate_market_condition_burn(&state, &params, PEX_TOTAL_SUPPLY).is_ok());

    let conservation_threshold =
        amount_bps(PEX_TOTAL_SUPPLY, CONSERVATION_SUPPLY_THRESHOLD_BPS).unwrap();
    let conservation = market_burn_params(1_000_000 * PEX_DECIMALS, CONSERVATION_BURN_RATE_BPS, 50);
    assert!(validate_market_condition_burn(&state, &conservation, conservation_threshold).is_ok());
}
