use crate::{
    ApcConfig, ApcObservation, ApcState, ApcStatus, ConditionalBuybackBurnParams,
    InitializeApcParams, MarketConditionSnapshot, MarketConditionalReleaseParams, PeraxError,
    PeraxState, ReleaseType, ReserveVaultConfig, SubmitApcObservationParams, VaultClass,
    ALLOCATION_ADVISOR_1, ALLOCATION_ADVISOR_2, ALLOCATION_ADVISOR_3, ALLOCATION_COMMUNITY_REWARDS,
    ALLOCATION_DEVELOPMENT_TEAM, ALLOCATION_ECOSYSTEM_MARKETING, ALLOCATION_FOUNDER,
    ALLOCATION_FUTURE_TEAM_INCENTIVES, ALLOCATION_LIQUIDITY_POOL, ALLOCATION_PRIVATE_STRATEGIC,
    ALLOCATION_TEAM_EMERGENCY_RESERVE, ALLOCATION_TRADING_OPERATIONS, ALLOCATION_TREASURY,
    APC_BPS_DENOMINATOR, APC_FIRST_ACTIVATION_MULTIPLIER, APC_MAX_METRIC_BPS, APC_QUOTE_DECIMALS,
    CONSERVATION_BURN_RATE_BPS, CONSERVATION_DAILY_BURN_CAP_BPS, CONSERVATION_SUPPLY_THRESHOLD_BPS,
    DEFAULT_BURN_RATE_BPS, EARLY_DAILY_BURN_CAP_BPS, EMERGENCY_DOWNSIDE_TRIGGER_BPS,
    EMERGENCY_LIQUIDITY_DRAIN_TRIGGER_BPS, MAX_BURN_RATE_BPS, MIN_BURN_RATE_BPS, PEX_DECIMALS,
    PEX_TOTAL_SUPPLY,
};
use anchor_lang::prelude::*;

pub(crate) fn validate_payment_amount(state: &PeraxState, amount: u64) -> Result<()> {
    require!(amount > 0, PeraxError::InvalidAmount);
    if state.max_payment_amount > 0 {
        require!(
            amount <= state.max_payment_amount,
            PeraxError::PaymentAmountTooLarge
        );
    }
    Ok(())
}

pub(crate) fn validate_reference(reference: [u8; 32]) -> Result<()> {
    require!(reference != [0u8; 32], PeraxError::InvalidReference);
    Ok(())
}

pub(crate) fn approved_allocation(allocation_id: [u8; 32]) -> Result<(VaultClass, u64)> {
    let approved = if allocation_id == ALLOCATION_LIQUIDITY_POOL {
        (VaultClass::Liquidity, 380_000_000 * PEX_DECIMALS)
    } else if allocation_id == ALLOCATION_COMMUNITY_REWARDS {
        (VaultClass::CommunityRewards, 170_000_000 * PEX_DECIMALS)
    } else if allocation_id == ALLOCATION_TREASURY {
        (VaultClass::MarketReserve, 120_000_000 * PEX_DECIMALS)
    } else if allocation_id == ALLOCATION_ECOSYSTEM_MARKETING {
        (VaultClass::MarketReserve, 120_000_000 * PEX_DECIMALS)
    } else if allocation_id == ALLOCATION_TRADING_OPERATIONS {
        (VaultClass::Operations, 70_000_000 * PEX_DECIMALS)
    } else if allocation_id == ALLOCATION_DEVELOPMENT_TEAM {
        (VaultClass::Vesting, 20_000_000 * PEX_DECIMALS)
    } else if allocation_id == ALLOCATION_FOUNDER {
        (VaultClass::Vesting, 20_000_000 * PEX_DECIMALS)
    } else if allocation_id == ALLOCATION_FUTURE_TEAM_INCENTIVES {
        (VaultClass::MarketReserve, 10_000_000 * PEX_DECIMALS)
    } else if allocation_id == ALLOCATION_TEAM_EMERGENCY_RESERVE {
        (VaultClass::EmergencyReserve, 10_000_000 * PEX_DECIMALS)
    } else if allocation_id == ALLOCATION_PRIVATE_STRATEGIC {
        (VaultClass::Vesting, 50_000_000 * PEX_DECIMALS)
    } else if allocation_id == ALLOCATION_ADVISOR_1
        || allocation_id == ALLOCATION_ADVISOR_2
        || allocation_id == ALLOCATION_ADVISOR_3
    {
        (VaultClass::Vesting, 10_000_000 * PEX_DECIMALS)
    } else {
        return err!(PeraxError::UnknownAllocationId);
    };

    Ok(approved)
}

pub(crate) fn is_market_releasable_vault_class(vault_class: VaultClass) -> bool {
    matches!(
        vault_class,
        VaultClass::MarketReserve
            | VaultClass::Operations
            | VaultClass::CommunityRewards
            | VaultClass::EmergencyReserve
    )
}

pub(crate) fn is_apc_releasable_vault_class(vault_class: VaultClass) -> bool {
    matches!(
        vault_class,
        VaultClass::MarketReserve | VaultClass::Operations | VaultClass::CommunityRewards
    )
}

pub(crate) fn is_program_derived_destination(owner: Pubkey) -> bool {
    !owner.is_on_curve()
}

pub(crate) fn validate_vault_class_for_release(
    vault_class: VaultClass,
    release_type: ReleaseType,
) -> Result<()> {
    match release_type {
        ReleaseType::Growth => require!(
            is_apc_releasable_vault_class(vault_class),
            PeraxError::VaultClassNotMarketReleasable
        ),
        ReleaseType::Emergency => require!(
            vault_class == VaultClass::EmergencyReserve,
            PeraxError::VaultClassNotMarketReleasable
        ),
    }
    Ok(())
}

pub(crate) fn calculate_vault_available_amount(
    config: &ReserveVaultConfig,
    actual_vault_balance: u64,
) -> Result<u64> {
    require!(
        config.authorized_deposited <= config.allocation_cap,
        PeraxError::VaultAccountingMismatch
    );
    require!(
        config.total_released <= config.authorized_deposited,
        PeraxError::VaultAccountingMismatch
    );

    let internally_available = config
        .authorized_deposited
        .checked_sub(config.total_released)
        .ok_or(PeraxError::VaultAccountingMismatch)?;
    let actual_authorized_balance = actual_vault_balance
        .checked_sub(config.unsolicited_balance)
        .ok_or(PeraxError::VaultAccountingMismatch)?;

    Ok(internally_available.min(actual_authorized_balance))
}

pub(crate) fn validate_oracle_snapshot(
    state: &PeraxState,
    snapshot: &MarketConditionSnapshot,
) -> Result<()> {
    require!(snapshot.observed_at > 0, PeraxError::InvalidMarketParameter);
    require!(
        snapshot.observed_price > 0,
        PeraxError::InvalidMarketParameter
    );
    require!(
        snapshot.net_buy_volume_bps <= 10_000,
        PeraxError::InvalidMarketParameter
    );
    require!(
        state.oracle_feed != Pubkey::default(),
        PeraxError::InvalidOracleFeed
    );
    Ok(())
}

pub(crate) fn validate_growth_release(
    _state: &PeraxState,
    _params: &MarketConditionalReleaseParams,
) -> Result<()> {
    err!(PeraxError::UseApcRelease)
}

pub(crate) fn validate_growth_release_fields(
    _state: &PeraxState,
    _requested_amount: u64,
    _snapshot: &MarketConditionSnapshot,
) -> Result<()> {
    err!(PeraxError::UseApcRelease)
}

pub(crate) fn validate_emergency_release(
    state: &PeraxState,
    params: &MarketConditionalReleaseParams,
) -> Result<()> {
    validate_emergency_release_fields(
        state,
        params.requested_amount,
        &params.snapshot,
        params.snapshot.emergency_reserve_available_amount,
    )
}

pub(crate) fn validate_emergency_release_fields(
    state: &PeraxState,
    requested_amount: u64,
    snapshot: &MarketConditionSnapshot,
    authoritative_available_amount: u64,
) -> Result<()> {
    require!(
        snapshot.downside_move_bps >= EMERGENCY_DOWNSIDE_TRIGGER_BPS,
        PeraxError::EmergencyDownsideGateNotMet
    );
    require!(
        snapshot.liquidity_drain_bps >= EMERGENCY_LIQUIDITY_DRAIN_TRIGGER_BPS,
        PeraxError::EmergencyLiquidityGateNotMet
    );
    require!(
        authoritative_available_amount > 0,
        PeraxError::InsufficientVaultBalance
    );
    require!(
        snapshot.emergency_reserve_available_amount == authoritative_available_amount,
        PeraxError::VaultBalanceObservationMismatch
    );
    let hourly_cap = amount_bps(
        authoritative_available_amount,
        state.emergency_hourly_release_bps,
    )?;
    require!(
        requested_amount <= hourly_cap,
        PeraxError::EmergencyHourlyCapExceeded
    );
    require_checked_cap(
        state.daily_unlocked_accumulator,
        requested_amount,
        state.daily_release_cap,
        PeraxError::DailyReleaseCapExceeded,
    )?;
    require_checked_cap(
        state.monthly_unlocked_accumulator,
        requested_amount,
        state.monthly_release_cap,
        PeraxError::MonthlyReleaseCapExceeded,
    )?;
    Ok(())
}

pub(crate) fn reset_release_windows_if_needed(state: &mut PeraxState, now: i64) {
    if state.daily_window_start == 0 || now >= state.daily_window_start.saturating_add(86_400) {
        state.daily_window_start = now;
        state.daily_unlocked_accumulator = 0;
    }
    if state.monthly_window_start == 0
        || now >= state.monthly_window_start.saturating_add(2_592_000)
    {
        state.monthly_window_start = now;
        state.monthly_unlocked_accumulator = 0;
    }
}

pub(crate) fn reset_burn_window_if_needed(state: &mut PeraxState, now: i64) {
    if state.daily_burn_window_start == 0
        || now >= state.daily_burn_window_start.saturating_add(86_400)
    {
        state.daily_burn_window_start = now;
        state.daily_burn_accumulator = 0;
    }
}

pub(crate) fn validate_apc_policy(state: &PeraxState, params: &InitializeApcParams) -> Result<()> {
    require!(
        params.quote_mint != Pubkey::default(),
        PeraxError::InvalidCounterweightMint
    );
    require!(
        params.approved_pool != Pubkey::default(),
        PeraxError::InvalidApcPool
    );
    require!(
        params.approved_proceeds_owner != Pubkey::default()
            && params.approved_proceeds_token_account != Pubkey::default(),
        PeraxError::InvalidCounterweightVault
    );
    require!(
        params.approved_recovery_program != Pubkey::default(),
        PeraxError::InvalidRecoveryProgram
    );
    require!(params.price_scale > 0, PeraxError::InvalidApcPolicy);
    let expected_first_activation = state
        .launch_price
        .checked_mul(APC_FIRST_ACTIVATION_MULTIPLIER)
        .ok_or(PeraxError::InvalidApcPolicy)?;
    require!(
        params.first_activation_price == expected_first_activation,
        PeraxError::InvalidApcPolicy
    );
    require!(
        params.minimum_band_interval_bps > 0
            && params.minimum_band_interval_bps < params.maximum_band_interval_bps,
        PeraxError::InvalidBandInterval
    );
    require!(
        params.maximum_band_interval_bps <= 10_000,
        PeraxError::InvalidBandInterval
    );
    require!(
        params.maximum_observation_age_seconds > 0 && params.maximum_future_clock_skew_seconds >= 0,
        PeraxError::InvalidApcPolicy
    );
    require!(
        params.hourly_release_cap > 0
            && params.hourly_release_cap <= state.daily_release_cap
            && params.pump_window_release_cap > 0
            && params.pump_window_release_cap <= state.monthly_release_cap
            && params.pump_window_seconds > 0,
        PeraxError::InvalidApcPolicy
    );
    require!(
        params.minimum_counterweight_coverage_bps > 0
            && params.minimum_counterweight_coverage_bps <= 10_000
            && params.minimum_buy_pressure_bps <= 10_000,
        PeraxError::InvalidApcPolicy
    );
    require!(
        params.base_band_release_cap > 0
            && params.base_band_release_cap <= state.daily_release_cap
            && params.recovery_spending_cap > 0,
        PeraxError::InvalidApcPolicy
    );
    require!(
        params.minimum_twap_minutes > 0
            && params.minimum_liquidity_usd > 0
            && params.minimum_volume_usd > 0,
        PeraxError::InvalidApcPolicy
    );
    validate_strictly_increasing_u32(params.risk_velocity_thresholds_bps)?;
    validate_strictly_increasing_u32(params.risk_volatility_thresholds_bps)?;
    validate_strictly_increasing_u32(params.risk_price_impact_thresholds_bps)?;

    for interval in params.band_interval_bps_by_risk {
        require!(
            interval >= params.minimum_band_interval_bps
                && interval <= params.maximum_band_interval_bps,
            PeraxError::InvalidBandInterval
        );
    }
    for release_bps in params.band_release_bps_by_risk {
        require!(
            release_bps > 0 && release_bps <= 10_000,
            PeraxError::InvalidApcPolicy
        );
    }
    let mut previous = 10_001u16;
    for reduction in params.cascade_reduction_bps {
        require!(
            reduction > 0 && reduction <= 10_000 && reduction <= previous,
            PeraxError::InvalidApcPolicy
        );
        previous = reduction;
    }
    Ok(())
}

pub(crate) fn validate_apc_observation_submission(
    config: &ApcConfig,
    apc_state: &ApcState,
    params: &SubmitApcObservationParams,
    now: i64,
) -> Result<()> {
    validate_reference(params.observation_id)?;
    require!(
        params.sequence > apc_state.last_observation_sequence,
        PeraxError::ObservationSequenceInvalid
    );
    require!(
        params.pool == config.approved_pool,
        PeraxError::InvalidApcPool
    );
    require!(
        params.observed_at <= now.saturating_add(config.maximum_future_clock_skew_seconds),
        PeraxError::ObservationFromFuture
    );
    require!(
        now.saturating_sub(params.observed_at) <= config.maximum_observation_age_seconds,
        PeraxError::ObservationStale
    );
    require!(
        params.spot_price > 0
            && params.twap_price > 0
            && params.twap_minutes > 0
            && params.liquidity_usd > 0
            && params.quote_liquidity_usd > 0
            && params.volume_usd > 0
            && params.quote_liquidity_usd <= params.liquidity_usd,
        PeraxError::InvalidMarketParameter
    );
    require!(
        params.net_buy_pressure_bps <= 10_000
            && params.price_velocity_bps <= APC_MAX_METRIC_BPS
            && params.volatility_bps <= APC_MAX_METRIC_BPS
            && params.estimated_price_impact_bps <= APC_MAX_METRIC_BPS,
        PeraxError::InvalidMarketParameter
    );
    Ok(())
}

pub(crate) fn validate_apc_observation_fresh(
    config: &ApcConfig,
    observation: &ApcObservation,
    now: i64,
) -> Result<()> {
    require!(
        observation.pool == config.approved_pool,
        PeraxError::InvalidApcPool
    );
    require!(
        observation.observed_at <= now.saturating_add(config.maximum_future_clock_skew_seconds),
        PeraxError::ObservationFromFuture
    );
    require!(
        now.saturating_sub(observation.observed_at) <= config.maximum_observation_age_seconds,
        PeraxError::ObservationStale
    );
    Ok(())
}

pub(crate) fn validate_apc_market_gates(
    config: &ApcConfig,
    observation: &ApcObservation,
) -> Result<()> {
    require!(
        observation.twap_minutes >= config.minimum_twap_minutes,
        PeraxError::TwapGateNotMet
    );
    require!(
        observation.liquidity_usd >= config.minimum_liquidity_usd,
        PeraxError::LiquidityGateNotMet
    );
    require!(
        observation.volume_usd >= config.minimum_volume_usd,
        PeraxError::LiquidityGateNotMet
    );
    require!(
        observation.net_buy_pressure_bps >= config.minimum_buy_pressure_bps,
        PeraxError::BuyPressureGateNotMet
    );
    Ok(())
}

pub fn validate_sequential_band_index(
    current_band_index: u32,
    requested_band_index: u32,
) -> Result<()> {
    let expected = current_band_index
        .checked_add(1)
        .ok_or(PeraxError::InvalidBandIndex)?;
    require!(
        requested_band_index == expected,
        PeraxError::NonSequentialBand
    );
    Ok(())
}

pub fn calculate_apc_risk_tier(
    velocity_bps: u32,
    volatility_bps: u32,
    impact_bps: u32,
    velocity_thresholds: [u32; 3],
    volatility_thresholds: [u32; 3],
    impact_thresholds: [u32; 3],
) -> u8 {
    let velocity_tier = threshold_tier(velocity_bps, velocity_thresholds);
    let volatility_tier = threshold_tier(volatility_bps, volatility_thresholds);
    let impact_tier = threshold_tier(impact_bps, impact_thresholds);
    velocity_tier.max(volatility_tier).max(impact_tier)
}

pub fn calculate_effective_apc_price(spot_price: u64, twap_price: u64) -> Result<u64> {
    require!(
        spot_price > 0 && twap_price > 0,
        PeraxError::InvalidMarketParameter
    );
    Ok(spot_price.min(twap_price))
}

pub fn calculate_band_interval_bps(config: &ApcConfig, risk_tier: u8) -> Result<u16> {
    let index = usize::from(risk_tier);
    require!(
        index < config.band_interval_bps_by_risk.len(),
        PeraxError::InvalidApcPolicy
    );
    let interval = config.band_interval_bps_by_risk[index];
    require!(
        interval >= config.minimum_band_interval_bps
            && interval <= config.maximum_band_interval_bps,
        PeraxError::InvalidBandInterval
    );
    Ok(interval)
}

pub fn calculate_next_band_price(reference_price: u64, interval_bps: u16) -> Result<u64> {
    require!(reference_price > 0, PeraxError::InvalidMarketParameter);
    let numerator = (reference_price as u128)
        .checked_mul(APC_BPS_DENOMINATOR + u128::from(interval_bps))
        .ok_or(PeraxError::InvalidMarketParameter)?
        .checked_add(APC_BPS_DENOMINATOR - 1)
        .ok_or(PeraxError::InvalidMarketParameter)?;
    let next = numerator
        .checked_div(APC_BPS_DENOMINATOR)
        .ok_or(PeraxError::InvalidMarketParameter)?;
    u64::try_from(next).map_err(|_| PeraxError::InvalidMarketParameter.into())
}

pub fn calculate_cascade_reduction(config: &ApcConfig, cascade_position: u32) -> Result<u16> {
    require!(cascade_position > 0, PeraxError::InvalidBandIndex);
    let index = usize::try_from(cascade_position.saturating_sub(1))
        .map_err(|_| PeraxError::InvalidBandIndex)?
        .min(config.cascade_reduction_bps.len() - 1);
    Ok(config.cascade_reduction_bps[index])
}

pub fn calculate_band_release_cap(
    config: &ApcConfig,
    risk_tier: u8,
    cascade_position: u32,
) -> Result<u64> {
    let risk_index = usize::from(risk_tier);
    require!(
        risk_index < config.band_release_bps_by_risk.len(),
        PeraxError::InvalidApcPolicy
    );
    let risk_bps = config.band_release_bps_by_risk[risk_index];
    let cascade_bps = calculate_cascade_reduction(config, cascade_position)?;
    let amount = (config.base_band_release_cap as u128)
        .checked_mul(u128::from(risk_bps))
        .ok_or(PeraxError::InvalidMarketParameter)?
        .checked_div(APC_BPS_DENOMINATOR)
        .ok_or(PeraxError::InvalidMarketParameter)?
        .checked_mul(u128::from(cascade_bps))
        .ok_or(PeraxError::InvalidMarketParameter)?
        .checked_div(APC_BPS_DENOMINATOR)
        .ok_or(PeraxError::InvalidMarketParameter)?;
    let amount = u64::try_from(amount).map_err(|_| PeraxError::InvalidMarketParameter)?;
    require!(amount > 0, PeraxError::InvalidApcPolicy);
    Ok(amount.min(config.hourly_release_cap))
}

pub fn calculate_recovery_pex_out(
    quote_reserve: u64,
    pex_reserve: u64,
    quote_amount: u64,
    fee_bps: u16,
) -> Result<u64> {
    require!(
        quote_reserve > 0 && pex_reserve > 0 && quote_amount > 0 && fee_bps <= 1_000,
        PeraxError::InvalidRecoveryPool
    );
    let effective_quote = u128::from(quote_amount)
        .checked_mul(
            APC_BPS_DENOMINATOR
                .checked_sub(u128::from(fee_bps))
                .ok_or(PeraxError::InvalidRecoverySettlement)?,
        )
        .ok_or(PeraxError::InvalidRecoverySettlement)?
        .checked_div(APC_BPS_DENOMINATOR)
        .ok_or(PeraxError::InvalidRecoverySettlement)?;
    require!(effective_quote > 0, PeraxError::InvalidRecoverySettlement);
    let denominator = u128::from(quote_reserve)
        .checked_add(effective_quote)
        .ok_or(PeraxError::InvalidRecoverySettlement)?;
    let pex_out = u128::from(pex_reserve)
        .checked_mul(effective_quote)
        .ok_or(PeraxError::InvalidRecoverySettlement)?
        .checked_div(denominator)
        .ok_or(PeraxError::InvalidRecoverySettlement)?;
    let pex_out = u64::try_from(pex_out).map_err(|_| PeraxError::InvalidRecoverySettlement)?;
    require!(pex_out > 0, PeraxError::InvalidRecoverySettlement);
    Ok(pex_out)
}

pub fn calculate_counterweight_requirement(
    pex_amount: u64,
    price_scaled: u64,
    price_scale: u64,
    coverage_bps: u16,
) -> Result<u64> {
    require!(price_scale > 0, PeraxError::InvalidApcPolicy);
    let quote_scale = 10u128
        .checked_pow(u32::from(APC_QUOTE_DECIMALS))
        .ok_or(PeraxError::InvalidApcPolicy)?;
    let quote_value = (pex_amount as u128)
        .checked_mul(price_scaled as u128)
        .ok_or(PeraxError::InvalidMarketParameter)?
        .checked_mul(quote_scale)
        .ok_or(PeraxError::InvalidMarketParameter)?
        .checked_div(PEX_DECIMALS as u128)
        .ok_or(PeraxError::InvalidMarketParameter)?
        .checked_div(price_scale as u128)
        .ok_or(PeraxError::InvalidMarketParameter)?;
    let required = quote_value
        .checked_mul(u128::from(coverage_bps))
        .ok_or(PeraxError::InvalidMarketParameter)?
        .checked_div(APC_BPS_DENOMINATOR)
        .ok_or(PeraxError::InvalidMarketParameter)?;
    u64::try_from(required).map_err(|_| PeraxError::InvalidMarketParameter.into())
}

pub(crate) fn reset_apc_windows_if_needed(config: &ApcConfig, apc_state: &mut ApcState, now: i64) {
    if apc_state.hourly_window_started_at == 0
        || now >= apc_state.hourly_window_started_at.saturating_add(3_600)
    {
        apc_state.hourly_window_started_at = now;
        apc_state.hourly_released = 0;
    }
    if apc_state.pump_window_started_at == 0
        || now
            >= apc_state
                .pump_window_started_at
                .saturating_add(config.pump_window_seconds)
    {
        apc_state.pump_window_started_at = now;
        apc_state.pump_window_released = 0;
    }
}

pub(crate) fn validate_apc_release_caps(
    config: &ApcConfig,
    apc_state: &ApcState,
    core_state: &PeraxState,
    band_released: u64,
    band_cap: u64,
    requested_amount: u64,
) -> Result<()> {
    require_checked_cap(
        band_released,
        requested_amount,
        band_cap,
        PeraxError::BandReleaseCapExceeded,
    )?;
    require_checked_cap(
        apc_state.hourly_released,
        requested_amount,
        config.hourly_release_cap,
        PeraxError::HourlyApcCapExceeded,
    )?;
    require_checked_cap(
        apc_state.pump_window_released,
        requested_amount,
        config.pump_window_release_cap,
        PeraxError::PumpWindowCapExceeded,
    )?;
    require_checked_cap(
        apc_state.unconfirmed_release_amount,
        requested_amount,
        config.pump_window_release_cap,
        PeraxError::PumpWindowCapExceeded,
    )?;
    require_checked_cap(
        core_state.daily_unlocked_accumulator,
        requested_amount,
        core_state.daily_release_cap,
        PeraxError::DailyReleaseCapExceeded,
    )?;
    require_checked_cap(
        core_state.monthly_unlocked_accumulator,
        requested_amount,
        core_state.monthly_release_cap,
        PeraxError::MonthlyReleaseCapExceeded,
    )?;
    Ok(())
}

pub(crate) fn validate_apc_absorption_confirmation(
    config: &ApcConfig,
    apc_state: &ApcState,
    observation: &ApcObservation,
    now: i64,
) -> Result<u64> {
    require!(
        matches!(
            apc_state.status,
            ApcStatus::AwaitingAbsorption | ApcStatus::PumpControl | ApcStatus::Recovery
        ),
        PeraxError::InvalidApcStatus
    );
    validate_apc_observation_fresh(config, observation, now)?;
    validate_apc_market_gates(config, observation)?;
    require!(
        observation.observation_id != apc_state.last_release_observation_id,
        PeraxError::ObservationAlreadyUsed
    );
    require!(
        !observation.is_consumed_for_release && !observation.is_consumed_for_confirmation,
        PeraxError::ObservationAlreadyUsed
    );
    let confirmed_price =
        calculate_effective_apc_price(observation.spot_price, observation.twap_price)?;
    require!(
        confirmed_price >= apc_state.current_reference_price,
        PeraxError::ApcPriceGateNotMet
    );
    Ok(confirmed_price)
}

pub(crate) fn validate_apc_burn_allowed(apc_state: &ApcState) -> Result<()> {
    require!(
        !matches!(
            apc_state.status,
            ApcStatus::PumpControl | ApcStatus::AwaitingAbsorption | ApcStatus::Recovery
        ),
        PeraxError::BurnDeferredDuringPump
    );
    require!(apc_state.status != ApcStatus::Paused, PeraxError::ApcPaused);
    Ok(())
}

pub(crate) fn validate_market_condition_burn(
    state: &PeraxState,
    params: &ConditionalBuybackBurnParams,
    current_mint_supply: u64,
) -> Result<()> {
    let conservation_threshold = amount_bps(PEX_TOTAL_SUPPLY, CONSERVATION_SUPPLY_THRESHOLD_BPS)?;
    let conservation_phase = current_mint_supply <= conservation_threshold;

    let expected_burn_rate_bps = if conservation_phase {
        CONSERVATION_BURN_RATE_BPS
    } else {
        burn_rate_bps_for_market_health(params.market_health_score)?
    };

    require!(
        params.burn_rate_bps == expected_burn_rate_bps,
        PeraxError::InvalidBurnRate
    );

    let expected_amount = amount_bps(params.eligible_revenue_amount, params.burn_rate_bps)?;
    require!(
        params.amount == expected_amount,
        PeraxError::BurnAmountMismatch
    );

    let daily_cap_bps = if conservation_phase {
        CONSERVATION_DAILY_BURN_CAP_BPS
    } else {
        EARLY_DAILY_BURN_CAP_BPS
    };
    let daily_cap_amount = amount_bps(PEX_TOTAL_SUPPLY, daily_cap_bps)?;

    require_checked_cap(
        state.daily_burn_accumulator,
        params.amount,
        daily_cap_amount,
        PeraxError::DailyBurnCapExceeded,
    )?;

    Ok(())
}

pub(crate) fn amount_bps(amount: u64, bps: u16) -> Result<u64> {
    let result = (amount as u128)
        .checked_mul(bps as u128)
        .ok_or(PeraxError::InvalidMarketParameter)?
        .checked_div(10_000)
        .ok_or(PeraxError::InvalidMarketParameter)?;
    u64::try_from(result).map_err(|_| PeraxError::InvalidMarketParameter.into())
}

fn burn_rate_bps_for_market_health(score: u8) -> Result<u16> {
    let rate = match score {
        0..=20 => MAX_BURN_RATE_BPS,
        21..=30 => 2_500,
        31..=45 => 2_000,
        46..=60 => DEFAULT_BURN_RATE_BPS,
        61..=75 => 800,
        76..=85 => 500,
        86..=100 => MIN_BURN_RATE_BPS,
        _ => return err!(PeraxError::InvalidMarketHealthScore),
    };
    Ok(rate)
}

fn threshold_tier(value: u32, thresholds: [u32; 3]) -> u8 {
    if value >= thresholds[2] {
        3
    } else if value >= thresholds[1] {
        2
    } else if value >= thresholds[0] {
        1
    } else {
        0
    }
}

fn validate_strictly_increasing_u32(values: [u32; 3]) -> Result<()> {
    require!(
        values[0] > 0 && values[0] < values[1] && values[1] < values[2],
        PeraxError::InvalidApcPolicy
    );
    Ok(())
}

fn require_checked_cap(current: u64, amount: u64, cap: u64, error: PeraxError) -> Result<()> {
    let next = current
        .checked_add(amount)
        .ok_or(PeraxError::ReleaseCapExceeded)?;
    if next > cap {
        return Err(error.into());
    }
    Ok(())
}
