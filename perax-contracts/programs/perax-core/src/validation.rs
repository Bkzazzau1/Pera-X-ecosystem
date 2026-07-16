use crate::{
    ConditionalBuybackBurnParams, MarketConditionSnapshot, MarketConditionalReleaseParams,
    PeraxError, PeraxState, ReleaseType, ReserveVaultConfig, VaultClass, ALLOCATION_ADVISOR_1,
    ALLOCATION_ADVISOR_2, ALLOCATION_ADVISOR_3, ALLOCATION_COMMUNITY_REWARDS,
    ALLOCATION_DEVELOPMENT_TEAM, ALLOCATION_ECOSYSTEM_MARKETING, ALLOCATION_FOUNDER,
    ALLOCATION_FUTURE_TEAM_INCENTIVES, ALLOCATION_LIQUIDITY_POOL, ALLOCATION_PRIVATE_STRATEGIC,
    ALLOCATION_TEAM_EMERGENCY_RESERVE, ALLOCATION_TRADING_OPERATIONS, ALLOCATION_TREASURY,
    CONSERVATION_BURN_RATE_BPS, CONSERVATION_DAILY_BURN_CAP_BPS, CONSERVATION_SUPPLY_THRESHOLD_BPS,
    DEFAULT_BURN_RATE_BPS, EARLY_DAILY_BURN_CAP_BPS, EMERGENCY_DOWNSIDE_TRIGGER_BPS,
    EMERGENCY_LIQUIDITY_DRAIN_TRIGGER_BPS, GROWTH_PRICE_MULTIPLIER, MAX_BURN_RATE_BPS,
    MIN_BURN_RATE_BPS, MIN_GROWTH_LIQUIDITY_USD, MIN_GROWTH_TWAP_MINUTES, MIN_NET_BUY_VOLUME_BPS,
    PEX_DECIMALS, PEX_TOTAL_SUPPLY, RELEASE_COOLDOWN_SECONDS,
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

pub(crate) fn is_program_derived_destination(owner: Pubkey) -> bool {
    !owner.is_on_curve()
}

pub(crate) fn validate_vault_class_for_release(
    vault_class: VaultClass,
    release_type: ReleaseType,
) -> Result<()> {
    match release_type {
        ReleaseType::Growth => require!(
            matches!(
                vault_class,
                VaultClass::MarketReserve | VaultClass::Operations | VaultClass::CommunityRewards
            ),
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
    state: &PeraxState,
    params: &MarketConditionalReleaseParams,
) -> Result<()> {
    validate_growth_release_fields(state, params.requested_amount, &params.snapshot)
}

pub(crate) fn validate_growth_release_fields(
    state: &PeraxState,
    requested_amount: u64,
    snapshot: &MarketConditionSnapshot,
) -> Result<()> {
    let growth_price_trigger = state
        .launch_price
        .checked_mul(GROWTH_PRICE_MULTIPLIER)
        .ok_or(PeraxError::InvalidMarketParameter)?;
    require!(
        snapshot.observed_price >= growth_price_trigger,
        PeraxError::GrowthPriceGateNotMet
    );
    require!(
        snapshot.twap_minutes >= MIN_GROWTH_TWAP_MINUTES,
        PeraxError::TwapGateNotMet
    );
    require!(
        snapshot.liquidity_usd >= MIN_GROWTH_LIQUIDITY_USD,
        PeraxError::LiquidityGateNotMet
    );
    require!(
        snapshot.net_buy_volume_bps >= MIN_NET_BUY_VOLUME_BPS,
        PeraxError::BuyPressureGateNotMet
    );
    require!(
        snapshot.observed_at >= state.last_release_timestamp + RELEASE_COOLDOWN_SECONDS
            || state.last_release_timestamp == 0,
        PeraxError::ReleaseCooldownActive
    );
    require!(
        state
            .daily_unlocked_accumulator
            .saturating_add(requested_amount)
            <= state.daily_release_cap,
        PeraxError::DailyReleaseCapExceeded
    );
    require!(
        state
            .monthly_unlocked_accumulator
            .saturating_add(requested_amount)
            <= state.monthly_release_cap,
        PeraxError::MonthlyReleaseCapExceeded
    );
    Ok(())
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
    require!(
        state
            .daily_unlocked_accumulator
            .saturating_add(requested_amount)
            <= state.daily_release_cap,
        PeraxError::DailyReleaseCapExceeded
    );
    require!(
        state
            .monthly_unlocked_accumulator
            .saturating_add(requested_amount)
            <= state.monthly_release_cap,
        PeraxError::MonthlyReleaseCapExceeded
    );
    Ok(())
}

pub(crate) fn reset_release_windows_if_needed(state: &mut PeraxState, observed_at: i64) {
    if state.daily_window_start == 0 || observed_at >= state.daily_window_start + 86_400 {
        state.daily_window_start = observed_at;
        state.daily_unlocked_accumulator = 0;
    }
    if state.monthly_window_start == 0 || observed_at >= state.monthly_window_start + 2_592_000 {
        state.monthly_window_start = observed_at;
        state.monthly_unlocked_accumulator = 0;
    }
}

pub(crate) fn reset_burn_window_if_needed(state: &mut PeraxState, observed_at: i64) {
    if state.daily_burn_window_start == 0 || observed_at >= state.daily_burn_window_start + 86_400 {
        state.daily_burn_window_start = observed_at;
        state.daily_burn_accumulator = 0;
    }
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

    require!(
        state.daily_burn_accumulator.saturating_add(params.amount) <= daily_cap_amount,
        PeraxError::DailyBurnCapExceeded
    );

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
