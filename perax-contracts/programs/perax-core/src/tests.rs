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
        policy_version: APC_POLICY_VERSION,
        policy_hash: APC_POLICY_HASH_SHA256,
        quote_mint: Pubkey::new_unique(),
        approved_pool: Pubkey::new_unique(),
        approved_proceeds_owner: Pubkey::new_unique(),
        approved_proceeds_token_account: Pubkey::new_unique(),
        approved_recovery_program: Pubkey::new_unique(),
        price_scale: APC_PRICE_SCALE,
        first_activation_price: APC_FIRST_ACTIVATION_PRICE_SCALED,
        minimum_band_interval_bps: APC_MINIMUM_BAND_INTERVAL_BPS,
        maximum_band_interval_bps: APC_MAXIMUM_BAND_INTERVAL_BPS,
        maximum_observation_age_seconds: APC_MAXIMUM_OBSERVATION_AGE_SECONDS,
        maximum_future_clock_skew_seconds: APC_MAXIMUM_FUTURE_CLOCK_SKEW_SECONDS,
        hourly_release_cap: APC_HOURLY_RELEASE_CAP,
        pump_window_release_cap: APC_PUMP_WINDOW_RELEASE_CAP,
        pump_window_seconds: APC_PUMP_WINDOW_SECONDS,
        minimum_counterweight_coverage_bps: APC_MINIMUM_COUNTERWEIGHT_COVERAGE_BPS,
        counterweight_proceeds_allocation_bps: APC_COUNTERWEIGHT_PROCEEDS_ALLOCATION_BPS,
        liquidity_reinforcement_allocation_bps: APC_LIQUIDITY_REINFORCEMENT_ALLOCATION_BPS,
        burn_reserve_allocation_bps: APC_BURN_RESERVE_ALLOCATION_BPS,
        operations_allocation_bps: APC_OPERATIONS_ALLOCATION_BPS,
        base_band_release_cap: APC_BASE_BAND_RELEASE_CAP,
        minimum_twap_minutes: APC_MINIMUM_TWAP_MINUTES,
        minimum_liquidity_usd: APC_MINIMUM_LIQUIDITY_USD,
        minimum_quote_liquidity_usd: APC_MINIMUM_QUOTE_LIQUIDITY_USD,
        minimum_volume_usd: APC_MINIMUM_VOLUME_USD,
        minimum_buy_pressure_bps: APC_MINIMUM_BUY_PRESSURE_BPS,
        risk_velocity_thresholds_bps: APC_RISK_VELOCITY_THRESHOLDS_BPS,
        risk_volatility_thresholds_bps: APC_RISK_VOLATILITY_THRESHOLDS_BPS,
        risk_price_impact_thresholds_bps: APC_RISK_PRICE_IMPACT_THRESHOLDS_BPS,
        band_interval_bps_by_risk: APC_INTERVAL_BPS_BY_RISK,
        band_release_bps_by_risk: APC_RELEASE_BPS_BY_RISK,
        cascade_reduction_bps: APC_CASCADE_REDUCTION_BPS,
        recovery_spending_cap: APC_RECOVERY_TOTAL_SPENDING_CAP,
        deferred_burn_window_cap: APC_DEFERRED_BURN_WINDOW_CAP,
        deferred_burn_window_seconds: APC_DEFERRED_BURN_WINDOW_SECONDS,
        deferred_burn_cooldown_seconds: APC_DEFERRED_BURN_COOLDOWN_SECONDS,
        deferred_burn_resumption_rate_bps: APC_DEFERRED_BURN_RESUMPTION_RATE_BPS,
        maximum_recovery_purchase_bps: APC_MAXIMUM_RECOVERY_PURCHASE_BPS,
        minimum_counterweight_reserve_bps: APC_MINIMUM_COUNTERWEIGHT_RESERVE_BPS,
        recovery_window_cap: APC_RECOVERY_WINDOW_CAP,
        recovery_window_seconds: APC_RECOVERY_WINDOW_SECONDS,
        recovery_cooldown_seconds: APC_RECOVERY_COOLDOWN_SECONDS,
        recovery_support_drawdown_bps: APC_RECOVERY_SUPPORT_DRAWDOWN_BPS,
        recovery_purchase_bps_by_support: APC_RECOVERY_PURCHASE_BPS_BY_SUPPORT,
    }
}

fn test_apc_config() -> ApcConfig {
    let params = test_apc_params();
    ApcConfig {
        policy_version: params.policy_version,
        policy_hash: params.policy_hash,
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
        counterweight_proceeds_allocation_bps: params.counterweight_proceeds_allocation_bps,
        liquidity_reinforcement_allocation_bps: params.liquidity_reinforcement_allocation_bps,
        burn_reserve_allocation_bps: params.burn_reserve_allocation_bps,
        operations_allocation_bps: params.operations_allocation_bps,
        base_band_release_cap: params.base_band_release_cap,
        minimum_twap_minutes: params.minimum_twap_minutes,
        minimum_liquidity_usd: params.minimum_liquidity_usd,
        minimum_quote_liquidity_usd: params.minimum_quote_liquidity_usd,
        minimum_volume_usd: params.minimum_volume_usd,
        minimum_buy_pressure_bps: params.minimum_buy_pressure_bps,
        risk_velocity_thresholds_bps: params.risk_velocity_thresholds_bps,
        risk_volatility_thresholds_bps: params.risk_volatility_thresholds_bps,
        risk_price_impact_thresholds_bps: params.risk_price_impact_thresholds_bps,
        band_interval_bps_by_risk: params.band_interval_bps_by_risk,
        band_release_bps_by_risk: params.band_release_bps_by_risk,
        cascade_reduction_bps: params.cascade_reduction_bps,
        recovery_spending_cap: params.recovery_spending_cap,
        deferred_burn_window_cap: params.deferred_burn_window_cap,
        deferred_burn_window_seconds: params.deferred_burn_window_seconds,
        deferred_burn_cooldown_seconds: params.deferred_burn_cooldown_seconds,
        deferred_burn_resumption_rate_bps: params.deferred_burn_resumption_rate_bps,
        maximum_recovery_purchase_bps: params.maximum_recovery_purchase_bps,
        minimum_counterweight_reserve_bps: params.minimum_counterweight_reserve_bps,
        recovery_window_cap: params.recovery_window_cap,
        recovery_window_seconds: params.recovery_window_seconds,
        recovery_cooldown_seconds: params.recovery_cooldown_seconds,
        recovery_support_drawdown_bps: params.recovery_support_drawdown_bps,
        recovery_purchase_bps_by_support: params.recovery_purchase_bps_by_support,
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
        deferred_burn_window_started_at: 0,
        deferred_burn_window_executed: 0,
        last_deferred_burn_timestamp: 0,
        recovery_window_started_at: 0,
        recovery_window_spent: 0,
        last_recovery_purchase_timestamp: 0,
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
    assert_eq!(tier, 3);
    assert_eq!(calculate_band_interval_bps(&config, tier).unwrap(), 750);
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

#[test]
fn apc_policy_rejects_more_aggressive_higher_risk_settings() {
    let state = test_state(0);

    let mut widening = test_apc_params();
    widening.band_interval_bps_by_risk = [1_000, 1_500, 2_500, 4_000];
    assert!(validate_apc_policy(&state, &widening).is_err());

    let mut increasing_release = test_apc_params();
    increasing_release.band_release_bps_by_risk = [4_000, 6_000, 8_000, 10_000];
    assert!(validate_apc_policy(&state, &increasing_release).is_err());

    assert!(validate_apc_policy(&state, &test_apc_params()).is_ok());
}

#[test]
fn old_band_release_requires_highest_reference_support() {
    let config = test_apc_config();
    let mut state = test_apc_state(config.state);
    state.current_reference_price = 10_200;

    assert!(validate_apc_reference_support(&state, 10_200).is_ok());
    assert!(validate_apc_reference_support(&state, 5_500).is_err());
}

#[test]
fn deferred_burn_limits_are_windowed_capped_and_cooled_down() {
    let config = test_apc_config();
    let core = test_state(0);
    let mut apc = test_apc_state(config.state);
    apc.deferred_burn_amount = 1_000 * PEX_DECIMALS;
    let amount = 100 * PEX_DECIMALS;

    assert!(
        validate_deferred_burn_limits(&config, &apc, &core, amount, PEX_TOTAL_SUPPLY, 10_000,)
            .is_ok()
    );

    apc.deferred_burn_window_executed = config.deferred_burn_window_cap;
    assert!(
        validate_deferred_burn_limits(&config, &apc, &core, amount, PEX_TOTAL_SUPPLY, 10_000,)
            .is_err()
    );

    apc.deferred_burn_window_executed = 0;
    apc.last_deferred_burn_timestamp = 10_000;
    assert!(validate_deferred_burn_limits(
        &config,
        &apc,
        &core,
        amount,
        PEX_TOTAL_SUPPLY,
        10_000 + config.deferred_burn_cooldown_seconds - 1,
    )
    .is_err());
}

#[test]
fn recovery_limits_preserve_reserve_and_prevent_single_price_depletion() {
    let config = test_apc_config();
    let mut apc = test_apc_state(config.state);
    apc.total_counterweight_credited = 1_000_000;

    assert!(validate_recovery_purchase_limits(
        &config, &apc, 100_000, 1_000_000, 1_000_000, 2_000, 10_000, 10_000,
    )
    .is_ok());
    assert!(validate_recovery_purchase_limits(
        &config, &apc, 300_000, 1_000_000, 1_000_000, 2_000, 10_000, 10_000,
    )
    .is_err());

    apc.recovery_window_spent = config.recovery_window_cap - 50_000;
    assert!(validate_recovery_purchase_limits(
        &config, &apc, 100_000, 1_000_000, 1_000_000, 2_000, 10_000, 10_000,
    )
    .is_err());

    apc.recovery_window_spent = 0;
    apc.last_recovery_purchase_timestamp = 10_000;
    assert!(validate_recovery_purchase_limits(
        &config,
        &apc,
        100_000,
        1_000_000,
        1_000_000,
        2_000,
        10_000,
        10_000 + config.recovery_cooldown_seconds - 1,
    )
    .is_err());
}

fn assert_apc_policy_mutation_rejected(mutator: impl FnOnce(&mut InitializeApcParams)) {
    let state = test_state(0);
    let mut params = test_apc_params();
    mutator(&mut params);
    assert!(validate_apc_policy(&state, &params).is_err());
}

#[test]
fn every_apc_policy_v1_parameter_is_exact_and_immutable() {
    assert!(validate_apc_policy(&test_state(0), &test_apc_params()).is_ok());
    assert_apc_policy_mutation_rejected(|p| p.policy_version += 1);
    assert_apc_policy_mutation_rejected(|p| p.policy_hash[0] ^= 1);
    assert_apc_policy_mutation_rejected(|p| p.price_scale += 1);
    assert_apc_policy_mutation_rejected(|p| p.first_activation_price += 1);
    assert_apc_policy_mutation_rejected(|p| p.minimum_band_interval_bps += 1);
    assert_apc_policy_mutation_rejected(|p| p.maximum_band_interval_bps += 1);
    assert_apc_policy_mutation_rejected(|p| p.maximum_observation_age_seconds += 1);
    assert_apc_policy_mutation_rejected(|p| p.maximum_future_clock_skew_seconds += 1);
    assert_apc_policy_mutation_rejected(|p| p.hourly_release_cap += 1);
    assert_apc_policy_mutation_rejected(|p| p.pump_window_release_cap += 1);
    assert_apc_policy_mutation_rejected(|p| p.pump_window_seconds += 1);
    assert_apc_policy_mutation_rejected(|p| p.minimum_counterweight_coverage_bps += 1);
    assert_apc_policy_mutation_rejected(|p| p.counterweight_proceeds_allocation_bps += 1);
    assert_apc_policy_mutation_rejected(|p| p.liquidity_reinforcement_allocation_bps += 1);
    assert_apc_policy_mutation_rejected(|p| p.burn_reserve_allocation_bps += 1);
    assert_apc_policy_mutation_rejected(|p| p.operations_allocation_bps += 1);
    assert_apc_policy_mutation_rejected(|p| p.base_band_release_cap += 1);
    assert_apc_policy_mutation_rejected(|p| p.minimum_twap_minutes += 1);
    assert_apc_policy_mutation_rejected(|p| p.minimum_liquidity_usd += 1);
    assert_apc_policy_mutation_rejected(|p| p.minimum_quote_liquidity_usd += 1);
    assert_apc_policy_mutation_rejected(|p| p.minimum_volume_usd += 1);
    assert_apc_policy_mutation_rejected(|p| p.minimum_buy_pressure_bps += 1);
    assert_apc_policy_mutation_rejected(|p| p.risk_velocity_thresholds_bps[0] += 1);
    assert_apc_policy_mutation_rejected(|p| p.risk_volatility_thresholds_bps[1] += 1);
    assert_apc_policy_mutation_rejected(|p| p.risk_price_impact_thresholds_bps[2] += 1);
    assert_apc_policy_mutation_rejected(|p| p.band_interval_bps_by_risk[0] += 1);
    assert_apc_policy_mutation_rejected(|p| p.band_release_bps_by_risk[1] += 1);
    assert_apc_policy_mutation_rejected(|p| p.cascade_reduction_bps[2] += 1);
    assert_apc_policy_mutation_rejected(|p| p.recovery_spending_cap += 1);
    assert_apc_policy_mutation_rejected(|p| p.deferred_burn_window_cap += 1);
    assert_apc_policy_mutation_rejected(|p| p.deferred_burn_window_seconds += 1);
    assert_apc_policy_mutation_rejected(|p| p.deferred_burn_cooldown_seconds += 1);
    assert_apc_policy_mutation_rejected(|p| p.deferred_burn_resumption_rate_bps += 1);
    assert_apc_policy_mutation_rejected(|p| p.maximum_recovery_purchase_bps += 1);
    assert_apc_policy_mutation_rejected(|p| p.minimum_counterweight_reserve_bps += 1);
    assert_apc_policy_mutation_rejected(|p| p.recovery_window_cap += 1);
    assert_apc_policy_mutation_rejected(|p| p.recovery_window_seconds += 1);
    assert_apc_policy_mutation_rejected(|p| p.recovery_cooldown_seconds += 1);
    assert_apc_policy_mutation_rejected(|p| p.recovery_support_drawdown_bps[0] += 1);
    assert_apc_policy_mutation_rejected(|p| p.recovery_purchase_bps_by_support[3] -= 1);
}

#[test]
fn apc_policy_v1_property_invariants_hold_for_thousands_of_inputs() {
    let config = test_apc_config();
    let mut seed = 0x9e37_79b9_7f4a_7c15u64;
    for _ in 0..25_000 {
        seed = seed.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
        let risk = (seed % 4) as u8;
        let cascade = ((seed >> 8) % 64 + 1) as u32;
        let cap = calculate_band_release_cap(&config, risk, cascade).unwrap();
        assert!(cap > 0);
        assert!(cap <= config.base_band_release_cap);
        assert!(cap <= config.hourly_release_cap);
        if risk > 0 {
            let safer = calculate_band_release_cap(&config, risk - 1, cascade).unwrap();
            assert!(cap <= safer);
            assert!(
                config.band_interval_bps_by_risk[risk as usize]
                    <= config.band_interval_bps_by_risk[(risk - 1) as usize]
            );
        }

        let reference = 10_000 + (seed % 1_000_000);
        let drawdown = config.recovery_support_drawdown_bps[(seed as usize) % 4] as u64;
        let effective = reference
            .saturating_mul(10_000u64.saturating_sub(drawdown))
            .checked_div(10_000)
            .unwrap()
            .max(1);
        let support =
            recovery_purchase_bps_for_price_support(&config, effective, reference).unwrap();
        assert!(support <= config.maximum_recovery_purchase_bps);

        let deferred = 1_000_000 * PEX_DECIMALS + (seed % 10_000) * PEX_DECIMALS;
        let resumption = amount_bps(deferred, config.deferred_burn_resumption_rate_bps).unwrap();
        assert!(resumption <= deferred);
        let executable = resumption.min(config.deferred_burn_window_cap);
        assert!(executable <= resumption);
        assert!(executable <= config.deferred_burn_window_cap);

        assert!(calculate_next_band_price(u64::MAX, config.maximum_band_interval_bps).is_err());
    }
}

// every_apc_policy_v1_field_rejects_plus_and_minus_one
#[test]
fn every_apc_policy_v1_field_rejects_plus_and_minus_one() {
    macro_rules! reject_scalar {
        ($field:ident) => {{
            assert_apc_policy_mutation_rejected(|p| p.$field -= 1);
            assert_apc_policy_mutation_rejected(|p| p.$field += 1);
        }};
    }
    macro_rules! reject_array {
        ($field:ident, $length:expr) => {{
            for index in 0..$length {
                assert_apc_policy_mutation_rejected(|p| p.$field[index] -= 1);
                assert_apc_policy_mutation_rejected(|p| p.$field[index] += 1);
            }
        }};
    }
    reject_scalar!(policy_version);
    assert_apc_policy_mutation_rejected(|p| p.policy_hash[31] ^= 1);
    reject_scalar!(price_scale);
    reject_scalar!(first_activation_price);
    reject_scalar!(minimum_band_interval_bps);
    reject_scalar!(maximum_band_interval_bps);
    reject_scalar!(maximum_observation_age_seconds);
    reject_scalar!(maximum_future_clock_skew_seconds);
    reject_scalar!(hourly_release_cap);
    reject_scalar!(pump_window_release_cap);
    reject_scalar!(pump_window_seconds);
    reject_scalar!(minimum_counterweight_coverage_bps);
    reject_scalar!(counterweight_proceeds_allocation_bps);
    reject_scalar!(liquidity_reinforcement_allocation_bps);
    reject_scalar!(burn_reserve_allocation_bps);
    reject_scalar!(operations_allocation_bps);
    reject_scalar!(base_band_release_cap);
    reject_scalar!(minimum_twap_minutes);
    reject_scalar!(minimum_liquidity_usd);
    reject_scalar!(minimum_quote_liquidity_usd);
    reject_scalar!(minimum_volume_usd);
    reject_scalar!(minimum_buy_pressure_bps);
    reject_array!(risk_velocity_thresholds_bps, 3);
    reject_array!(risk_volatility_thresholds_bps, 3);
    reject_array!(risk_price_impact_thresholds_bps, 3);
    reject_array!(band_interval_bps_by_risk, 4);
    reject_array!(band_release_bps_by_risk, 4);
    reject_array!(cascade_reduction_bps, 4);
    reject_scalar!(recovery_spending_cap);
    reject_scalar!(deferred_burn_window_cap);
    reject_scalar!(deferred_burn_window_seconds);
    reject_scalar!(deferred_burn_cooldown_seconds);
    reject_scalar!(deferred_burn_resumption_rate_bps);
    reject_scalar!(maximum_recovery_purchase_bps);
    reject_scalar!(minimum_counterweight_reserve_bps);
    reject_scalar!(recovery_window_cap);
    reject_scalar!(recovery_window_seconds);
    reject_scalar!(recovery_cooldown_seconds);
    reject_array!(recovery_support_drawdown_bps, 4);
    reject_array!(recovery_purchase_bps_by_support, 4);
}
