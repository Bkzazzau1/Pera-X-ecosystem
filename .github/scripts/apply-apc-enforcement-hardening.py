from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]


def replace_exact(relative, old, new):
    path = ROOT / relative
    text = path.read_text()
    count = text.count(old)
    if count != 1:
        raise SystemExit(f"{relative}: expected exactly one match, found {count}: {old[:80]!r}")
    path.write_text(text.replace(old, new))


def append_once(relative, marker, addition):
    path = ROOT / relative
    text = path.read_text()
    if addition.strip() in text:
        raise SystemExit(f"{relative}: addition already present")
    if marker not in text:
        raise SystemExit(f"{relative}: insertion marker not found")
    path.write_text(text.replace(marker, addition + marker, 1))


# ---------------------------------------------------------------------------
# State/config fields: immutable policy inputs and mutable trusted windows.
# ---------------------------------------------------------------------------
replace_exact(
    "perax-contracts/programs/perax-core/src/state.rs",
    """    pub cascade_reduction_bps: [u16; 4],
    pub recovery_spending_cap: u64,
}
""",
    """    pub cascade_reduction_bps: [u16; 4],
    pub recovery_spending_cap: u64,
    pub deferred_burn_window_cap: u64,
    pub deferred_burn_window_seconds: i64,
    pub deferred_burn_cooldown_seconds: i64,
    pub maximum_recovery_purchase_bps: u16,
    pub minimum_counterweight_reserve_bps: u16,
    pub recovery_window_cap: u64,
    pub recovery_window_seconds: i64,
    pub recovery_cooldown_seconds: i64,
}
""",
)
replace_exact(
    "perax-contracts/programs/perax-core/src/state.rs",
    """    pub cascade_reduction_bps: [u16; 4],
    pub recovery_spending_cap: u64,
    pub is_active: bool,
""",
    """    pub cascade_reduction_bps: [u16; 4],
    pub recovery_spending_cap: u64,
    pub deferred_burn_window_cap: u64,
    pub deferred_burn_window_seconds: i64,
    pub deferred_burn_cooldown_seconds: i64,
    pub maximum_recovery_purchase_bps: u16,
    pub minimum_counterweight_reserve_bps: u16,
    pub recovery_window_cap: u64,
    pub recovery_window_seconds: i64,
    pub recovery_cooldown_seconds: i64,
    pub is_active: bool,
""",
)
replace_exact(
    "perax-contracts/programs/perax-core/src/state.rs",
    """    pub cascade_band_count: u32,
    pub active_risk_tier: u8,
    pub bump: u8,
""",
    """    pub cascade_band_count: u32,
    pub active_risk_tier: u8,
    pub deferred_burn_window_started_at: i64,
    pub deferred_burn_window_executed: u64,
    pub last_deferred_burn_timestamp: i64,
    pub recovery_window_started_at: i64,
    pub recovery_window_spent: u64,
    pub last_recovery_purchase_timestamp: i64,
    pub bump: u8,
""",
)

# ---------------------------------------------------------------------------
# Initialization: copy immutable limits and initialize trusted counters.
# ---------------------------------------------------------------------------
replace_exact(
    "perax-contracts/programs/perax-core/src/instructions/apc.rs",
    """    config.cascade_reduction_bps = params.cascade_reduction_bps;
    config.recovery_spending_cap = params.recovery_spending_cap;
    config.is_active = true;
""",
    """    config.cascade_reduction_bps = params.cascade_reduction_bps;
    config.recovery_spending_cap = params.recovery_spending_cap;
    config.deferred_burn_window_cap = params.deferred_burn_window_cap;
    config.deferred_burn_window_seconds = params.deferred_burn_window_seconds;
    config.deferred_burn_cooldown_seconds = params.deferred_burn_cooldown_seconds;
    config.maximum_recovery_purchase_bps = params.maximum_recovery_purchase_bps;
    config.minimum_counterweight_reserve_bps = params.minimum_counterweight_reserve_bps;
    config.recovery_window_cap = params.recovery_window_cap;
    config.recovery_window_seconds = params.recovery_window_seconds;
    config.recovery_cooldown_seconds = params.recovery_cooldown_seconds;
    config.is_active = true;
""",
)
replace_exact(
    "perax-contracts/programs/perax-core/src/instructions/apc.rs",
    """    apc_state.cascade_band_count = 0;
    apc_state.active_risk_tier = 0;
    apc_state.bump = ctx.bumps.apc_state;
""",
    """    apc_state.cascade_band_count = 0;
    apc_state.active_risk_tier = 0;
    apc_state.deferred_burn_window_started_at = 0;
    apc_state.deferred_burn_window_executed = 0;
    apc_state.last_deferred_burn_timestamp = 0;
    apc_state.recovery_window_started_at = 0;
    apc_state.recovery_window_spent = 0;
    apc_state.last_recovery_purchase_timestamp = 0;
    apc_state.bump = ctx.bumps.apc_state;
""",
)

# Collapse protection: an old band cannot release below the highest live reference.
replace_exact(
    "perax-contracts/programs/perax-core/src/instructions/apc.rs",
    """    calculate_vault_available_amount, is_apc_releasable_vault_class,
    is_program_derived_destination, reset_apc_windows_if_needed, reset_release_windows_if_needed,
""",
    """    calculate_vault_available_amount, is_apc_releasable_vault_class,
    is_program_derived_destination, reset_apc_windows_if_needed, reset_release_windows_if_needed,
    validate_apc_reference_support,
""",
)
replace_exact(
    "perax-contracts/programs/perax-core/src/instructions/apc.rs",
    """    require!(
        effective_price >= ctx.accounts.band_record.trigger_price,
        PeraxError::ApcPriceGateNotMet
    );

    let vault = &ctx.accounts.reserve_vault_config;
""",
    """    require!(
        effective_price >= ctx.accounts.band_record.trigger_price,
        PeraxError::ApcPriceGateNotMet
    );
    validate_apc_reference_support(&ctx.accounts.apc_state, effective_price)?;

    let vault = &ctx.accounts.reserve_vault_config;
""",
)

# ---------------------------------------------------------------------------
# Policy validation and pure limit helpers.
# ---------------------------------------------------------------------------
replace_exact(
    "perax-contracts/programs/perax-core/src/validation.rs",
    """    require!(
        params.base_band_release_cap > 0
            && params.base_band_release_cap <= state.daily_release_cap
            && params.recovery_spending_cap > 0,
        PeraxError::InvalidApcPolicy
    );
""",
    """    require!(
        params.base_band_release_cap > 0
            && params.base_band_release_cap <= state.daily_release_cap
            && params.recovery_spending_cap > 0,
        PeraxError::InvalidApcPolicy
    );
    require!(
        params.deferred_burn_window_cap > 0
            && params.deferred_burn_window_seconds > 0
            && params.deferred_burn_cooldown_seconds >= 0
            && params.deferred_burn_cooldown_seconds <= params.deferred_burn_window_seconds,
        PeraxError::InvalidApcPolicy
    );
    require!(
        params.maximum_recovery_purchase_bps > 0
            && params.maximum_recovery_purchase_bps < 10_000
            && params.minimum_counterweight_reserve_bps > 0
            && params.minimum_counterweight_reserve_bps < 10_000
            && u32::from(params.maximum_recovery_purchase_bps)
                + u32::from(params.minimum_counterweight_reserve_bps)
                <= 10_000
            && params.recovery_window_cap > 0
            && params.recovery_window_cap <= params.recovery_spending_cap
            && params.recovery_window_seconds > 0
            && params.recovery_cooldown_seconds >= 0
            && params.recovery_cooldown_seconds <= params.recovery_window_seconds,
        PeraxError::InvalidApcPolicy
    );
""",
)
replace_exact(
    "perax-contracts/programs/perax-core/src/validation.rs",
    """    for release_bps in params.band_release_bps_by_risk {
        require!(
            release_bps > 0 && release_bps <= 10_000,
            PeraxError::InvalidApcPolicy
        );
    }
    let mut previous = 10_001u16;
""",
    """    for release_bps in params.band_release_bps_by_risk {
        require!(
            release_bps > 0 && release_bps <= 10_000,
            PeraxError::InvalidApcPolicy
        );
    }
    validate_non_increasing_u16(params.band_interval_bps_by_risk)?;
    validate_non_increasing_u16(params.band_release_bps_by_risk)?;
    let mut previous = 10_001u16;
""",
)
insert_marker = """pub(crate) fn validate_apc_observation_submission(
"""
append_once(
    "perax-contracts/programs/perax-core/src/validation.rs",
    insert_marker,
    """fn validate_non_increasing_u16(values: [u16; 4]) -> Result<()> {
    for pair in values.windows(2) {
        require!(pair[0] >= pair[1], PeraxError::InvalidApcPolicy);
    }
    Ok(())
}

""",
)
replace_exact(
    "perax-contracts/programs/perax-core/src/validation.rs",
    """pub fn calculate_effective_apc_price(spot_price: u64, twap_price: u64) -> Result<u64> {
    require!(
        spot_price > 0 && twap_price > 0,
        PeraxError::InvalidMarketParameter
    );
    Ok(spot_price.min(twap_price))
}

""",
    """pub fn calculate_effective_apc_price(spot_price: u64, twap_price: u64) -> Result<u64> {
    require!(
        spot_price > 0 && twap_price > 0,
        PeraxError::InvalidMarketParameter
    );
    Ok(spot_price.min(twap_price))
}

pub(crate) fn validate_apc_reference_support(
    apc_state: &ApcState,
    effective_price: u64,
) -> Result<()> {
    require!(
        effective_price >= apc_state.current_reference_price,
        PeraxError::ApcReferencePriceNotSupported
    );
    Ok(())
}

""",
)
replace_exact(
    "perax-contracts/programs/perax-core/src/validation.rs",
    """pub(crate) fn reset_apc_windows_if_needed(config: &ApcConfig, apc_state: &mut ApcState, now: i64) {
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

""",
    """pub(crate) fn reset_apc_windows_if_needed(config: &ApcConfig, apc_state: &mut ApcState, now: i64) {
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

pub(crate) fn reset_deferred_burn_window_if_needed(
    config: &ApcConfig,
    apc_state: &mut ApcState,
    now: i64,
) {
    if apc_state.deferred_burn_window_started_at == 0
        || now
            >= apc_state
                .deferred_burn_window_started_at
                .saturating_add(config.deferred_burn_window_seconds)
    {
        apc_state.deferred_burn_window_started_at = now;
        apc_state.deferred_burn_window_executed = 0;
    }
}

pub(crate) fn reset_recovery_window_if_needed(
    config: &ApcConfig,
    apc_state: &mut ApcState,
    now: i64,
) {
    if apc_state.recovery_window_started_at == 0
        || now
            >= apc_state
                .recovery_window_started_at
                .saturating_add(config.recovery_window_seconds)
    {
        apc_state.recovery_window_started_at = now;
        apc_state.recovery_window_spent = 0;
    }
}

pub(crate) fn validate_deferred_burn_limits(
    config: &ApcConfig,
    apc_state: &ApcState,
    core_state: &PeraxState,
    requested_amount: u64,
    current_mint_supply: u64,
    now: i64,
) -> Result<()> {
    if apc_state.last_deferred_burn_timestamp > 0 {
        require!(
            now >= apc_state
                .last_deferred_burn_timestamp
                .saturating_add(config.deferred_burn_cooldown_seconds),
            PeraxError::DeferredBurnCooldownActive
        );
    }
    require_checked_cap(
        apc_state.deferred_burn_window_executed,
        requested_amount,
        config.deferred_burn_window_cap,
        PeraxError::DeferredBurnWindowCapExceeded,
    )?;
    require_checked_cap(
        core_state.daily_burn_accumulator,
        requested_amount,
        calculate_daily_burn_cap_amount(current_mint_supply)?,
        PeraxError::DailyBurnCapExceeded,
    )?;
    Ok(())
}

pub(crate) fn validate_recovery_purchase_limits(
    config: &ApcConfig,
    apc_state: &ApcState,
    requested_maximum: u64,
    tracked_available: u64,
    actual_vault_balance: u64,
    now: i64,
) -> Result<()> {
    if apc_state.last_recovery_purchase_timestamp > 0 {
        require!(
            now >= apc_state
                .last_recovery_purchase_timestamp
                .saturating_add(config.recovery_cooldown_seconds),
            PeraxError::RecoveryCooldownActive
        );
    }
    require!(
        requested_maximum <= tracked_available && requested_maximum <= actual_vault_balance,
        PeraxError::InvalidCounterweightVault
    );
    let single_purchase_cap = amount_bps(tracked_available, config.maximum_recovery_purchase_bps)?;
    require!(
        requested_maximum <= single_purchase_cap,
        PeraxError::RecoveryPurchaseCapExceeded
    );
    require_checked_cap(
        apc_state.recovery_window_spent,
        requested_maximum,
        config.recovery_window_cap,
        PeraxError::RecoveryWindowCapExceeded,
    )?;
    require_checked_cap(
        apc_state.total_counterweight_spent,
        requested_maximum,
        config.recovery_spending_cap,
        PeraxError::RecoveryCapExceeded,
    )?;
    let protected_reserve = amount_bps(
        apc_state.total_counterweight_credited,
        config.minimum_counterweight_reserve_bps,
    )?;
    let tracked_after = tracked_available
        .checked_sub(requested_maximum)
        .ok_or(PeraxError::RecoveryReserveFloorViolated)?;
    let vault_after = actual_vault_balance
        .checked_sub(requested_maximum)
        .ok_or(PeraxError::RecoveryReserveFloorViolated)?;
    require!(
        tracked_after >= protected_reserve && vault_after >= protected_reserve,
        PeraxError::RecoveryReserveFloorViolated
    );
    Ok(())
}

""",
)
replace_exact(
    "perax-contracts/programs/perax-core/src/validation.rs",
    """    let daily_cap_bps = if conservation_phase {
        CONSERVATION_DAILY_BURN_CAP_BPS
    } else {
        EARLY_DAILY_BURN_CAP_BPS
    };
    let daily_cap_amount = amount_bps(PEX_TOTAL_SUPPLY, daily_cap_bps)?;

    require_checked_cap(
""",
    """    let daily_cap_amount = calculate_daily_burn_cap_amount(current_mint_supply)?;

    require_checked_cap(
""",
)
append_once(
    "perax-contracts/programs/perax-core/src/validation.rs",
    """pub(crate) fn amount_bps(amount: u64, bps: u16) -> Result<u64> {
""",
    """pub(crate) fn calculate_daily_burn_cap_amount(current_mint_supply: u64) -> Result<u64> {
    let conservation_threshold = amount_bps(PEX_TOTAL_SUPPLY, CONSERVATION_SUPPLY_THRESHOLD_BPS)?;
    let daily_cap_bps = if current_mint_supply <= conservation_threshold {
        CONSERVATION_DAILY_BURN_CAP_BPS
    } else {
        EARLY_DAILY_BURN_CAP_BPS
    };
    amount_bps(PEX_TOTAL_SUPPLY, daily_cap_bps)
}

""",
)

# ---------------------------------------------------------------------------
# Deferred burn: trusted daily/window caps, accumulator and cooldown.
# ---------------------------------------------------------------------------
replace_exact(
    "perax-contracts/programs/perax-core/src/instructions/counterweight.rs",
    """use crate::{
    validate_reference, ApcStatus, CounterweightProceedsDeposited, DeferredBurnExecuted,
""",
    """use crate::{
    reset_burn_window_if_needed, reset_deferred_burn_window_if_needed,
    validate_deferred_burn_limits, validate_reference, ApcStatus, CounterweightProceedsDeposited, DeferredBurnExecuted,
""",
)
replace_exact(
    "perax-contracts/programs/perax-core/src/instructions/counterweight.rs",
    """    require!(
        !ctx.accounts.deferred_burn_record.is_complete,
        PeraxError::DeferredBurnNotExecutable
    );

    let remaining_record_amount = ctx
""",
    """    require!(
        !ctx.accounts.deferred_burn_record.is_complete,
        PeraxError::DeferredBurnNotExecutable
    );

    let now = Clock::get()?.unix_timestamp;
    reset_burn_window_if_needed(&mut ctx.accounts.state, now);
    reset_deferred_burn_window_if_needed(
        &ctx.accounts.apc_config,
        &mut ctx.accounts.apc_state,
        now,
    );
    validate_deferred_burn_limits(
        &ctx.accounts.apc_config,
        &ctx.accounts.apc_state,
        &ctx.accounts.state,
        params.amount,
        ctx.accounts.token_mint.supply,
        now,
    )?;

    let remaining_record_amount = ctx
""",
)
replace_exact(
    "perax-contracts/programs/perax-core/src/instructions/counterweight.rs",
    """    let remaining_deferred = ctx
        .accounts
        .apc_state
        .deferred_burn_amount
        .checked_sub(params.amount)
        .ok_or(PeraxError::DeferredBurnNotExecutable)?;
    let now = Clock::get()?.unix_timestamp;

    ctx.accounts.deferred_burn_record.amount_executed = amount_executed_after;
""",
    """    let remaining_deferred = ctx
        .accounts
        .apc_state
        .deferred_burn_amount
        .checked_sub(params.amount)
        .ok_or(PeraxError::DeferredBurnNotExecutable)?;
    let daily_burn_after = ctx
        .accounts
        .state
        .daily_burn_accumulator
        .checked_add(params.amount)
        .ok_or(PeraxError::DailyBurnCapExceeded)?;
    let deferred_window_after = ctx
        .accounts
        .apc_state
        .deferred_burn_window_executed
        .checked_add(params.amount)
        .ok_or(PeraxError::DeferredBurnWindowCapExceeded)?;

    ctx.accounts.deferred_burn_record.amount_executed = amount_executed_after;
""",
)
replace_exact(
    "perax-contracts/programs/perax-core/src/instructions/counterweight.rs",
    """    ctx.accounts.deferred_burn_record.is_complete =
        amount_executed_after == ctx.accounts.deferred_burn_record.amount;
    ctx.accounts.apc_state.deferred_burn_amount = remaining_deferred;

    emit!(DeferredBurnExecuted {
""",
    """    ctx.accounts.deferred_burn_record.is_complete =
        amount_executed_after == ctx.accounts.deferred_burn_record.amount;
    ctx.accounts.state.daily_burn_accumulator = daily_burn_after;
    ctx.accounts.apc_state.deferred_burn_amount = remaining_deferred;
    ctx.accounts.apc_state.deferred_burn_window_executed = deferred_window_after;
    ctx.accounts.apc_state.last_deferred_burn_timestamp = now;

    emit!(DeferredBurnExecuted {
""",
)
replace_exact(
    "perax-contracts/programs/perax-core/src/contexts.rs",
    """pub struct ExecuteDeferredBurn<'info> {
    #[account(seeds = [b\"perax-state\"], bump = state.bump, constraint = token_mint.key() == state.token_mint @ PeraxError::InvalidTokenMint)]
""",
    """pub struct ExecuteDeferredBurn<'info> {
    #[account(mut, seeds = [b\"perax-state\"], bump = state.bump, constraint = token_mint.key() == state.token_mint @ PeraxError::InvalidTokenMint)]
""",
)

# ---------------------------------------------------------------------------
# Recovery: per-purchase percentage, reserve floor, window cap and cooldown.
# ---------------------------------------------------------------------------
replace_exact(
    "perax-contracts/programs/perax-core/src/instructions/recovery.rs",
    """    calculate_effective_apc_price, calculate_recovery_pex_out, validate_apc_observation_fresh,
    validate_reference, ApcRecoveryEntered, ApcStatus, ApcStatusChanged,
""",
    """    calculate_effective_apc_price, calculate_recovery_pex_out, reset_recovery_window_if_needed,
    validate_apc_observation_fresh, validate_recovery_purchase_limits, validate_reference,
    ApcRecoveryEntered, ApcStatus, ApcStatusChanged,
""",
)
replace_exact(
    "perax-contracts/programs/perax-core/src/instructions/recovery.rs",
    """    let tracked_available = ctx
        .accounts
        .apc_state
        .total_counterweight_credited
        .checked_sub(ctx.accounts.apc_state.total_counterweight_spent)
        .ok_or(PeraxError::InvalidCounterweightVault)?;
    require!(
        params.maximum_quote_amount <= tracked_available
            && params.maximum_quote_amount <= ctx.accounts.counterweight_vault.amount,
        PeraxError::InvalidCounterweightVault
    );
    let maximum_total_spend = ctx
        .accounts
        .apc_state
        .total_counterweight_spent
        .checked_add(params.maximum_quote_amount)
        .ok_or(PeraxError::RecoveryCapExceeded)?;
    require!(
        maximum_total_spend <= ctx.accounts.apc_config.recovery_spending_cap,
        PeraxError::RecoveryCapExceeded
    );

    let quote_before = ctx.accounts.counterweight_vault.amount;
""",
    """    reset_recovery_window_if_needed(
        &ctx.accounts.apc_config,
        &mut ctx.accounts.apc_state,
        now,
    );
    let tracked_available = ctx
        .accounts
        .apc_state
        .total_counterweight_credited
        .checked_sub(ctx.accounts.apc_state.total_counterweight_spent)
        .ok_or(PeraxError::InvalidCounterweightVault)?;
    validate_recovery_purchase_limits(
        &ctx.accounts.apc_config,
        &ctx.accounts.apc_state,
        params.maximum_quote_amount,
        tracked_available,
        ctx.accounts.counterweight_vault.amount,
        now,
    )?;

    let quote_before = ctx.accounts.counterweight_vault.amount;
""",
)
replace_exact(
    "perax-contracts/programs/perax-core/src/instructions/recovery.rs",
    """    require!(
        ctx.accounts.apc_state.total_counterweight_spent
            <= ctx.accounts.apc_config.recovery_spending_cap,
        PeraxError::RecoveryCapExceeded
    );

    let record = &mut ctx.accounts.recovery_record;
""",
    """    require!(
        ctx.accounts.apc_state.total_counterweight_spent
            <= ctx.accounts.apc_config.recovery_spending_cap,
        PeraxError::RecoveryCapExceeded
    );
    ctx.accounts.apc_state.recovery_window_spent = ctx
        .accounts
        .apc_state
        .recovery_window_spent
        .checked_add(quote_spent)
        .ok_or(PeraxError::RecoveryWindowCapExceeded)?;
    require!(
        ctx.accounts.apc_state.recovery_window_spent <= ctx.accounts.apc_config.recovery_window_cap,
        PeraxError::RecoveryWindowCapExceeded
    );
    ctx.accounts.apc_state.last_recovery_purchase_timestamp = now;

    let record = &mut ctx.accounts.recovery_record;
""",
)

# ---------------------------------------------------------------------------
# Explicit errors.
# ---------------------------------------------------------------------------
replace_exact(
    "perax-contracts/programs/perax-core/src/errors.rs",
    """    #[msg(\"The effective APC price has not reached the next band trigger.\")]
    ApcPriceGateNotMet,
""",
    """    #[msg(\"The effective APC price has not reached the next band trigger.\")]
    ApcPriceGateNotMet,
    #[msg(\"The market no longer supports the highest crossed APC reference price.\")]
    ApcReferencePriceNotSupported,
""",
)
replace_exact(
    "perax-contracts/programs/perax-core/src/errors.rs",
    """    #[msg(\"The deferred burn cannot execute in the current APC state.\")]
    DeferredBurnNotExecutable,
""",
    """    #[msg(\"The deferred burn cannot execute in the current APC state.\")]
    DeferredBurnNotExecutable,
    #[msg(\"The deferred-burn execution window cap would be exceeded.\")]
    DeferredBurnWindowCapExceeded,
    #[msg(\"The deferred-burn execution cooldown is still active.\")]
    DeferredBurnCooldownActive,
""",
)
replace_exact(
    "perax-contracts/programs/perax-core/src/errors.rs",
    """    #[msg(\"The APC recovery spending cap would be exceeded.\")]
    RecoveryCapExceeded,
""",
    """    #[msg(\"The APC recovery spending cap would be exceeded.\")]
    RecoveryCapExceeded,
    #[msg(\"The requested recovery purchase exceeds the immutable per-purchase percentage cap.\")]
    RecoveryPurchaseCapExceeded,
    #[msg(\"The recovery purchase would violate the protected Counterweight Vault reserve floor.\")]
    RecoveryReserveFloorViolated,
    #[msg(\"The APC recovery spending window cap would be exceeded.\")]
    RecoveryWindowCapExceeded,
    #[msg(\"The APC recovery purchase cooldown is still active.\")]
    RecoveryCooldownActive,
""",
)

# ---------------------------------------------------------------------------
# Rust unit fixtures and regression tests.
# ---------------------------------------------------------------------------
replace_exact(
    "perax-contracts/programs/perax-core/src/tests.rs",
    """        band_interval_bps_by_risk: [1_000, 1_500, 2_500, 4_000],
        band_release_bps_by_risk: [10_000, 8_000, 6_000, 4_000],
        cascade_reduction_bps: [10_000, 7_000, 4_500, 2_500],
        recovery_spending_cap: 1_000_000_000,
""",
    """        band_interval_bps_by_risk: [4_000, 2_500, 1_500, 1_000],
        band_release_bps_by_risk: [10_000, 8_000, 6_000, 4_000],
        cascade_reduction_bps: [10_000, 7_000, 4_500, 2_500],
        recovery_spending_cap: 1_000_000_000,
        deferred_burn_window_cap: 1_000_000 * PEX_DECIMALS,
        deferred_burn_window_seconds: 3_600,
        deferred_burn_cooldown_seconds: 60,
        maximum_recovery_purchase_bps: 2_000,
        minimum_counterweight_reserve_bps: 5_000,
        recovery_window_cap: 200_000_000,
        recovery_window_seconds: 3_600,
        recovery_cooldown_seconds: 60,
""",
)
replace_exact(
    "perax-contracts/programs/perax-core/src/tests.rs",
    """        cascade_reduction_bps: params.cascade_reduction_bps,
        recovery_spending_cap: params.recovery_spending_cap,
        is_active: true,
""",
    """        cascade_reduction_bps: params.cascade_reduction_bps,
        recovery_spending_cap: params.recovery_spending_cap,
        deferred_burn_window_cap: params.deferred_burn_window_cap,
        deferred_burn_window_seconds: params.deferred_burn_window_seconds,
        deferred_burn_cooldown_seconds: params.deferred_burn_cooldown_seconds,
        maximum_recovery_purchase_bps: params.maximum_recovery_purchase_bps,
        minimum_counterweight_reserve_bps: params.minimum_counterweight_reserve_bps,
        recovery_window_cap: params.recovery_window_cap,
        recovery_window_seconds: params.recovery_window_seconds,
        recovery_cooldown_seconds: params.recovery_cooldown_seconds,
        is_active: true,
""",
)
replace_exact(
    "perax-contracts/programs/perax-core/src/tests.rs",
    """        cascade_band_count: 0,
        active_risk_tier: 0,
        bump: 253,
""",
    """        cascade_band_count: 0,
        active_risk_tier: 0,
        deferred_burn_window_started_at: 0,
        deferred_burn_window_executed: 0,
        last_deferred_burn_timestamp: 0,
        recovery_window_started_at: 0,
        recovery_window_spent: 0,
        last_recovery_purchase_timestamp: 0,
        bump: 253,
""",
)
replace_exact(
    "perax-contracts/programs/perax-core/src/tests.rs",
    """    assert_eq!(calculate_band_interval_bps(&config, tier).unwrap(), 2_500);
""",
    """    assert_eq!(calculate_band_interval_bps(&config, tier).unwrap(), 1_500);
""",
)
append_once(
    "perax-contracts/programs/perax-core/src/tests.rs",
    "",
    """
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
    let amount = 100 * PEX_DECIMALS;

    assert!(validate_deferred_burn_limits(
        &config,
        &apc,
        &core,
        amount,
        PEX_TOTAL_SUPPLY,
        10_000,
    )
    .is_ok());

    apc.deferred_burn_window_executed = config.deferred_burn_window_cap;
    assert!(validate_deferred_burn_limits(
        &config,
        &apc,
        &core,
        amount,
        PEX_TOTAL_SUPPLY,
        10_000,
    )
    .is_err());

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
        &config,
        &apc,
        100_000,
        1_000_000,
        1_000_000,
        10_000,
    )
    .is_ok());
    assert!(validate_recovery_purchase_limits(
        &config,
        &apc,
        300_000,
        1_000_000,
        1_000_000,
        10_000,
    )
    .is_err());

    apc.recovery_window_spent = config.recovery_window_cap - 50_000;
    assert!(validate_recovery_purchase_limits(
        &config,
        &apc,
        100_000,
        1_000_000,
        1_000_000,
        10_000,
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
        10_000 + config.recovery_cooldown_seconds - 1,
    )
    .is_err());
}
""",
)

# ---------------------------------------------------------------------------
# Anchor transaction fixture and direct regression demonstrations.
# ---------------------------------------------------------------------------
replace_exact(
    "perax-contracts/tests/perax-core.ts",
    """        bandIntervalBpsByRisk: [1_000, 1_500, 2_500, 4_000],
        bandReleaseBpsByRisk: [10_000, 8_000, 6_000, 4_000],
        cascadeReductionBps: [10_000, 7_000, 4_500, 2_500],
        recoverySpendingCap: new anchor.BN(1_000_000),
""",
    """        bandIntervalBpsByRisk: [4_000, 2_500, 1_500, 1_000],
        bandReleaseBpsByRisk: [10_000, 8_000, 6_000, 4_000],
        cascadeReductionBps: [10_000, 7_000, 4_500, 2_500],
        recoverySpendingCap: new anchor.BN(1_000_000),
        deferredBurnWindowCap: new anchor.BN(BASE_UNITS),
        deferredBurnWindowSeconds: new anchor.BN(3_600),
        deferredBurnCooldownSeconds: new anchor.BN(60),
        maximumRecoveryPurchaseBps: 2_000,
        minimumCounterweightReserveBps: 5_000,
        recoveryWindowCap: new anchor.BN(200_000),
        recoveryWindowSeconds: new anchor.BN(3_600),
        recoveryCooldownSeconds: new anchor.BN(60),
""",
)
replace_exact(
    "perax-contracts/tests/perax-core.ts",
    """    const firstReleaseObservation = await submitObservation(50_011, 10_000);
""",
    """    const collapsedObservation = await submitObservation(50_009, 4_000);
    await expectFailure(() =>
      releaseFromBand(
        50_100,
        1,
        firstBand,
        collapsedObservation.observationId,
        collapsedObservation.observation,
        1
      )
    );

    const firstReleaseObservation = await submitObservation(50_011, 10_000);
""",
)
replace_exact(
    "perax-contracts/tests/perax-core.ts",
    """        amount: new anchor.BN(BASE_UNITS),
        observedAt: new anchor.BN(await currentChainTime()),
""",
    """        amount: new anchor.BN(2 * BASE_UNITS),
        observedAt: new anchor.BN(await currentChainTime()),
""",
)
replace_exact(
    "perax-contracts/tests/perax-core.ts",
    """    expect((await getAccount(provider.connection, deferredBurnVault)).amount).to.equal(0n);

    const rollbackObservation = await submitObservation(50_015, 10_000);
""",
    """    expect((await getAccount(provider.connection, deferredBurnVault)).amount).to.equal(
      BigInt(BASE_UNITS)
    );
    await expectFailure(() =>
      program.methods
        .executeDeferredBurn({ amount: new anchor.BN(BASE_UNITS) })
        .accounts({
          state,
          apcConfig,
          apcState,
          counterweightConfig,
          deferredBurnAuthority,
          deferredBurnVault,
          deferredBurnRecord,
          tokenMint: mint,
          oracleFeed: oracle.publicKey,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .signers([oracle])
        .rpc()
    );

    const rollbackObservation = await submitObservation(50_015, 10_000);
""",
)
replace_exact(
    "perax-contracts/tests/perax-core.ts",
    """    const recoveryId = uniqueId(50_501);
""",
    """    const oversizedRecoveryId = uniqueId(50_500);
    const [oversizedRecoveryRecord] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from(\"apc-recovery\"), Buffer.from(oversizedRecoveryId)],
      program.programId
    );
    await expectFailure(() =>
      program.methods
        .executeCounterweightPurchase({
          recoveryId: oversizedRecoveryId,
          observationId: recoveryPurchaseObservation.observationId,
          maximumQuoteAmount: new anchor.BN(900_000),
          minimumPexOut: new anchor.BN(1),
          swapInstructionData: adapterInstruction.data,
        })
        .accounts({
          state,
          apcConfig,
          apcState,
          observation: recoveryPurchaseObservation.observation,
          counterweightConfig,
          counterweightAuthority,
          counterweightVault,
          recoveryVault,
          quoteMint,
          pexMint: mint,
          approvedPool: recoveryPool,
          recoveryProgram: program.programId,
          recoveryRecord: oversizedRecoveryRecord,
          oracleFeed: oracle.publicKey,
          tokenProgram: TOKEN_PROGRAM_ID,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .remainingAccounts([
          { pubkey: quoteMint, isWritable: false, isSigner: false },
          { pubkey: mint, isWritable: false, isSigner: false },
          { pubkey: poolAuthority, isWritable: false, isSigner: false },
          { pubkey: poolQuoteVault, isWritable: true, isSigner: false },
          { pubkey: poolPexVault, isWritable: true, isSigner: false },
        ])
        .signers([oracle])
        .rpc()
    );

    const recoveryId = uniqueId(50_501);
""",
)

# ---------------------------------------------------------------------------
# Machine-readable policy and validator/documentation.
# ---------------------------------------------------------------------------
replace_exact(
    "perax-contracts/config/pex-tokenomics.json",
    '"riskTierThresholds": null,\n      "intervalFormula": "contract_derived_from_risk_tier",',
    '"riskTierThresholds": null,\n      "intervalBpsByRisk": null,\n      "releaseBpsByRisk": null,\n      "higherRiskMustNotIncreaseIntervalOrRelease": true,\n      "intervalFormula": "contract_derived_from_risk_tier",',
)
replace_exact(
    "perax-contracts/config/pex-tokenomics.json",
    '"permanentDeferredBurnRecord": true,\n      "resumptionRateBps": null',
    '"permanentDeferredBurnRecord": true,\n      "resumptionRateBps": null,\n      "executionWindowCapAmount": null,\n      "executionWindowSeconds": null,\n      "executionCooldownSeconds": null',
)
replace_exact(
    "perax-contracts/config/pex-tokenomics.json",
    '"hardSpendingCapAmount": null,\n      "recoveredPexAutomaticallyBurned": false,',
    '"hardSpendingCapAmount": null,\n      "maximumPurchaseBps": null,\n      "minimumReserveBps": null,\n      "windowCapAmount": null,\n      "windowSeconds": null,\n      "cooldownSeconds": null,\n      "recoveredPexAutomaticallyBurned": false,',
)
replace_exact(
    "perax-contracts/scripts/validate-tokenomics.js",
    """    assert(Array.isArray(bands.cascadeReductionBps) && bands.cascadeReductionBps.length > 0, 'Approved cascade policy is required.');
    let previous = 10001;
""",
    """    assert(Array.isArray(bands.intervalBpsByRisk) && bands.intervalBpsByRisk.length === 4, 'Approved risk interval policy is required.');
    assert(Array.isArray(bands.releaseBpsByRisk) && bands.releaseBpsByRisk.length === 4, 'Approved risk release policy is required.');
    for (let index = 1; index < 4; index += 1) {
      assert(bands.intervalBpsByRisk[index - 1] >= bands.intervalBpsByRisk[index], 'Higher risk must not widen APC bands.');
      assert(bands.releaseBpsByRisk[index - 1] >= bands.releaseBpsByRisk[index], 'Higher risk must not increase APC release capacity.');
    }
    assert(Array.isArray(bands.cascadeReductionBps) && bands.cascadeReductionBps.length > 0, 'Approved cascade policy is required.');
    let previous = 10001;
""",
)
replace_exact(
    "perax-contracts/scripts/validate-tokenomics.js",
    """    assert(bands.riskTierThresholds === null && bands.cascadeReductionBps === null, 'Pending risk and cascade values must remain null.');
""",
    """    assert(bands.riskTierThresholds === null && bands.cascadeReductionBps === null, 'Pending risk and cascade values must remain null.');
    assert(bands.intervalBpsByRisk === null && bands.releaseBpsByRisk === null, 'Pending risk response tables must remain null.');
""",
)
replace_exact(
    "perax-contracts/scripts/validate-tokenomics.js",
    """  assert(apc.burnDeferralPolicy.pexEscrowRequired === true, 'Deferred burn PEX must be escrowed.');
  assert(apc.recoveryPolicy.atomicSwapRequired === true, 'Recovery must use an atomic swap.');
""",
    """  assert(apc.burnDeferralPolicy.pexEscrowRequired === true, 'Deferred burn PEX must be escrowed.');
  if (apc.policyStatus === 'approved') {
    assert(apc.burnDeferralPolicy.executionWindowCapAmount !== null, 'Approved deferred-burn window cap is required.');
    assert(Number.isInteger(apc.burnDeferralPolicy.executionWindowSeconds) && apc.burnDeferralPolicy.executionWindowSeconds > 0, 'Approved deferred-burn window is required.');
    assert(Number.isInteger(apc.burnDeferralPolicy.executionCooldownSeconds) && apc.burnDeferralPolicy.executionCooldownSeconds >= 0, 'Approved deferred-burn cooldown is required.');
  } else {
    assert(apc.burnDeferralPolicy.executionWindowCapAmount === null && apc.burnDeferralPolicy.executionWindowSeconds === null && apc.burnDeferralPolicy.executionCooldownSeconds === null, 'Pending deferred-burn limits must remain null.');
  }
  assert(apc.recoveryPolicy.atomicSwapRequired === true, 'Recovery must use an atomic swap.');
""",
)
replace_exact(
    "perax-contracts/scripts/validate-tokenomics.js",
    """  assert(apc.recoveryPolicy.hardSpendingCapRequired === true, 'Recovery spending must have a hard cap.');
  assert(apc.authorityPolicy.requiresManualOrMultisigApproval === false, 'Manual or multisig release approval must remain disabled.');
""",
    """  assert(apc.recoveryPolicy.hardSpendingCapRequired === true, 'Recovery spending must have a hard cap.');
  if (apc.policyStatus === 'approved') {
    assert(Number.isInteger(apc.recoveryPolicy.maximumPurchaseBps) && apc.recoveryPolicy.maximumPurchaseBps > 0 && apc.recoveryPolicy.maximumPurchaseBps < 10000, 'Approved recovery purchase percentage is invalid.');
    assert(Number.isInteger(apc.recoveryPolicy.minimumReserveBps) && apc.recoveryPolicy.minimumReserveBps > 0 && apc.recoveryPolicy.minimumReserveBps < 10000, 'Approved recovery reserve percentage is invalid.');
    assert(apc.recoveryPolicy.maximumPurchaseBps + apc.recoveryPolicy.minimumReserveBps <= 10000, 'Recovery purchase and reserve percentages are inconsistent.');
    assert(apc.recoveryPolicy.windowCapAmount !== null && Number.isInteger(apc.recoveryPolicy.windowSeconds) && apc.recoveryPolicy.windowSeconds > 0, 'Approved recovery window limits are required.');
    assert(Number.isInteger(apc.recoveryPolicy.cooldownSeconds) && apc.recoveryPolicy.cooldownSeconds >= 0, 'Approved recovery cooldown is required.');
  } else {
    assert(apc.recoveryPolicy.maximumPurchaseBps === null && apc.recoveryPolicy.minimumReserveBps === null && apc.recoveryPolicy.windowCapAmount === null && apc.recoveryPolicy.windowSeconds === null && apc.recoveryPolicy.cooldownSeconds === null, 'Pending recovery limits must remain null.');
  }
  assert(apc.authorityPolicy.requiresManualOrMultisigApproval === false, 'Manual or multisig release approval must remain disabled.');
""",
)
replace_exact(
    "docs/APC_LOGIC_SPECIFICATION.md",
    """Risk tier is the maximum tier reached by velocity, volatility, or estimated impact. The contract selects the interval and base release percentage from immutable arrays and applies the monotonic cascade reduction. All multiplication and division uses checked `u128` arithmetic and explicit rounding.
""",
    """Risk tier is the maximum tier reached by velocity, volatility, or estimated impact. The contract requires both the interval and release arrays to be non-increasing as risk rises, selects the response from those immutable arrays, and applies the monotonic cascade reduction. All multiplication and division uses checked `u128` arithmetic and explicit rounding.
""",
)
replace_exact(
    "docs/APC_LOGIC_SPECIFICATION.md",
    """2. Reset hourly, pump, daily, and monthly windows from `Clock::get()`.
3. Calculate all caps and counterweight coverage.
""",
    """2. Require the effective price to support both the selected band and the highest crossed APC reference, then reset hourly, pump, daily, and monthly windows from `Clock::get()`.
3. Calculate all caps and counterweight coverage.
""",
)
replace_exact(
    "docs/APC_LOGIC_SPECIFICATION.md",
    """USDC credit follows a real SPL transfer. Recovery invokes only the immutable approved executable adapter. Before and after balances are reloaded; a recovery record is created only when USDC actually decreased and the locked PEX vault actually increased within the permitted limits.
""",
    """USDC credit follows a real SPL transfer. Recovery invokes only the immutable approved executable adapter. Before and after balances are reloaded; a recovery record is created only when USDC actually decreased and the locked PEX vault actually increased within the permitted limits. Every recovery purchase is also constrained by an immutable percentage cap, protected reserve floor, trusted-clock spending window, cooldown, and the cumulative recovery cap. Deferred burns share the global daily burn cap and additionally use an immutable execution window and cooldown.
""",
)

print("APC enforcement hardening applied successfully")
