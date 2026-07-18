from pathlib import Path

ROOT = Path.cwd()
SRC = ROOT / "perax-contracts/programs/perax-core/src"


def replace_once(path: Path, old: str, new: str) -> None:
    text = path.read_text()
    count = text.count(old)
    if count != 1:
        raise SystemExit(f"{path}: expected one match, found {count}: {old[:100]!r}")
    path.write_text(text.replace(old, new))


def insert_before(path: Path, marker: str, addition: str) -> None:
    text = path.read_text()
    if addition.strip() in text:
        raise SystemExit(f"{path}: addition already present")
    count = text.count(marker)
    if count != 1:
        raise SystemExit(f"{path}: expected one insertion marker, found {count}: {marker!r}")
    path.write_text(text.replace(marker, addition + marker))


constants = SRC / "constants.rs"
insert_before(
    constants,
    "pub const MIN_GROWTH_TWAP_MINUTES: u64 = 60;\n",
    """pub const APC_POLICY_VERSION: u16 = 1;
pub const APC_POLICY_HASH_SHA256: [u8; 32] = [
    0x17, 0xf9, 0x3b, 0xac, 0xb0, 0xcf, 0xa5, 0x34,
    0x6a, 0x46, 0x62, 0x58, 0x11, 0x79, 0x08, 0x06,
    0x8f, 0x1f, 0x0c, 0xd6, 0x70, 0x54, 0xf8, 0xb6,
    0x1c, 0x7d, 0x40, 0x81, 0x8d, 0xfe, 0x84, 0xbb,
];
pub const APC_PRICE_SCALE: u64 = 100_000_000;
pub const APC_FIRST_ACTIVATION_PRICE_SCALED: u64 = 3_600;
pub const APC_MINIMUM_BAND_INTERVAL_BPS: u16 = 750;
pub const APC_MAXIMUM_BAND_INTERVAL_BPS: u16 = 2_000;
pub const APC_RISK_VELOCITY_THRESHOLDS_BPS: [u32; 3] = [500, 1_500, 3_000];
pub const APC_RISK_VOLATILITY_THRESHOLDS_BPS: [u32; 3] = [400, 1_200, 2_500];
pub const APC_RISK_PRICE_IMPACT_THRESHOLDS_BPS: [u32; 3] = [250, 750, 1_500];
pub const APC_INTERVAL_BPS_BY_RISK: [u16; 4] = [2_000, 1_500, 1_000, 750];
pub const APC_RELEASE_BPS_BY_RISK: [u16; 4] = [10_000, 7_500, 5_000, 2_500];
pub const APC_CASCADE_REDUCTION_BPS: [u16; 4] = [10_000, 7_500, 5_000, 2_500];
pub const APC_MAXIMUM_OBSERVATION_AGE_SECONDS: i64 = 90;
pub const APC_MAXIMUM_FUTURE_CLOCK_SKEW_SECONDS: i64 = 5;
pub const APC_MINIMUM_TWAP_MINUTES: u64 = 60;
pub const APC_MINIMUM_LIQUIDITY_USD: u64 = 27_360;
pub const APC_MINIMUM_QUOTE_LIQUIDITY_USD: u64 = 13_680;
pub const APC_MINIMUM_VOLUME_USD: u64 = 6_840;
pub const APC_MINIMUM_BUY_PRESSURE_BPS: u16 = 5_500;
pub const APC_BASE_BAND_RELEASE_CAP: u64 = 2_000_000 * PEX_DECIMALS;
pub const APC_HOURLY_RELEASE_CAP: u64 = 2_500_000 * PEX_DECIMALS;
pub const APC_PUMP_WINDOW_RELEASE_CAP: u64 = 6_000_000 * PEX_DECIMALS;
pub const APC_PUMP_WINDOW_SECONDS: i64 = 21_600;
pub const APC_MINIMUM_COUNTERWEIGHT_COVERAGE_BPS: u16 = 5_000;
pub const APC_COUNTERWEIGHT_PROCEEDS_ALLOCATION_BPS: u16 = 7_000;
pub const APC_LIQUIDITY_REINFORCEMENT_ALLOCATION_BPS: u16 = 2_000;
pub const APC_BURN_RESERVE_ALLOCATION_BPS: u16 = 500;
pub const APC_OPERATIONS_ALLOCATION_BPS: u16 = 500;
pub const APC_DEFERRED_BURN_RESUMPTION_RATE_BPS: u16 = 1_000;
pub const APC_DEFERRED_BURN_WINDOW_CAP: u64 = 400_000 * PEX_DECIMALS;
pub const APC_DEFERRED_BURN_WINDOW_SECONDS: i64 = 3_600;
pub const APC_DEFERRED_BURN_COOLDOWN_SECONDS: i64 = 900;
pub const APC_USDC_BASE_UNITS: u64 = 1_000_000;
pub const APC_RECOVERY_TOTAL_SPENDING_CAP: u64 = 3_000 * APC_USDC_BASE_UNITS;
pub const APC_MAXIMUM_RECOVERY_PURCHASE_BPS: u16 = 1_500;
pub const APC_MINIMUM_COUNTERWEIGHT_RESERVE_BPS: u16 = 3_000;
pub const APC_RECOVERY_WINDOW_CAP: u64 = 500 * APC_USDC_BASE_UNITS;
pub const APC_RECOVERY_WINDOW_SECONDS: i64 = 21_600;
pub const APC_RECOVERY_COOLDOWN_SECONDS: i64 = 1_800;
pub const APC_RECOVERY_SUPPORT_DRAWDOWN_BPS: [u16; 4] = [1_000, 2_500, 5_000, 7_500];
pub const APC_RECOVERY_PURCHASE_BPS_BY_SUPPORT: [u16; 4] = [500, 750, 1_000, 1_500];

""",
)

state = SRC / "state.rs"
replace_once(
    state,
    "pub struct InitializeApcParams {\n    pub quote_mint: Pubkey,",
    "pub struct InitializeApcParams {\n    pub policy_version: u16,\n    pub policy_hash: [u8; 32],\n    pub quote_mint: Pubkey,",
)
replace_once(
    state,
    "    pub minimum_liquidity_usd: u64,\n    pub minimum_volume_usd: u64,",
    "    pub minimum_liquidity_usd: u64,\n    pub minimum_quote_liquidity_usd: u64,\n    pub minimum_volume_usd: u64,",
)
replace_once(
    state,
    "    pub minimum_counterweight_coverage_bps: u16,\n    pub base_band_release_cap: u64,",
    "    pub minimum_counterweight_coverage_bps: u16,\n    pub counterweight_proceeds_allocation_bps: u16,\n    pub liquidity_reinforcement_allocation_bps: u16,\n    pub burn_reserve_allocation_bps: u16,\n    pub operations_allocation_bps: u16,\n    pub base_band_release_cap: u64,",
)
replace_once(
    state,
    "    pub deferred_burn_cooldown_seconds: i64,\n    pub maximum_recovery_purchase_bps: u16,",
    "    pub deferred_burn_cooldown_seconds: i64,\n    pub deferred_burn_resumption_rate_bps: u16,\n    pub maximum_recovery_purchase_bps: u16,",
)
replace_once(
    state,
    "    pub recovery_cooldown_seconds: i64,\n}\n\n#[derive(AnchorSerialize, AnchorDeserialize, Clone)]\npub struct SubmitApcObservationParams",
    "    pub recovery_cooldown_seconds: i64,\n    pub recovery_support_drawdown_bps: [u16; 4],\n    pub recovery_purchase_bps_by_support: [u16; 4],\n}\n\n#[derive(AnchorSerialize, AnchorDeserialize, Clone)]\npub struct SubmitApcObservationParams",
)
replace_once(
    state,
    "pub struct ApcConfig {\n    pub state: Pubkey,",
    "pub struct ApcConfig {\n    pub policy_version: u16,\n    pub policy_hash: [u8; 32],\n    pub state: Pubkey,",
)
replace_once(
    state,
    "    pub minimum_liquidity_usd: u64,\n    pub minimum_volume_usd: u64,",
    "    pub minimum_liquidity_usd: u64,\n    pub minimum_quote_liquidity_usd: u64,\n    pub minimum_volume_usd: u64,",
)
replace_once(
    state,
    "    pub minimum_counterweight_coverage_bps: u16,\n    pub base_band_release_cap: u64,",
    "    pub minimum_counterweight_coverage_bps: u16,\n    pub counterweight_proceeds_allocation_bps: u16,\n    pub liquidity_reinforcement_allocation_bps: u16,\n    pub burn_reserve_allocation_bps: u16,\n    pub operations_allocation_bps: u16,\n    pub base_band_release_cap: u64,",
)
replace_once(
    state,
    "    pub deferred_burn_cooldown_seconds: i64,\n    pub maximum_recovery_purchase_bps: u16,",
    "    pub deferred_burn_cooldown_seconds: i64,\n    pub deferred_burn_resumption_rate_bps: u16,\n    pub maximum_recovery_purchase_bps: u16,",
)
replace_once(
    state,
    "    pub recovery_cooldown_seconds: i64,\n    pub is_active: bool,",
    "    pub recovery_cooldown_seconds: i64,\n    pub recovery_support_drawdown_bps: [u16; 4],\n    pub recovery_purchase_bps_by_support: [u16; 4],\n    pub is_active: bool,",
)

validation = SRC / "validation.rs"
replace_once(
    validation,
    "pub(crate) fn validate_apc_policy(state: &PeraxState, params: &InitializeApcParams) -> Result<()> {\n",
    """fn validate_exact_apc_policy_v1(params: &InitializeApcParams) -> Result<()> {
    require!(
        params.policy_version == crate::APC_POLICY_VERSION
            && params.policy_hash == crate::APC_POLICY_HASH_SHA256
            && params.price_scale == crate::APC_PRICE_SCALE
            && params.first_activation_price == crate::APC_FIRST_ACTIVATION_PRICE_SCALED
            && params.minimum_band_interval_bps == crate::APC_MINIMUM_BAND_INTERVAL_BPS
            && params.maximum_band_interval_bps == crate::APC_MAXIMUM_BAND_INTERVAL_BPS
            && params.maximum_observation_age_seconds == crate::APC_MAXIMUM_OBSERVATION_AGE_SECONDS
            && params.maximum_future_clock_skew_seconds == crate::APC_MAXIMUM_FUTURE_CLOCK_SKEW_SECONDS
            && params.hourly_release_cap == crate::APC_HOURLY_RELEASE_CAP
            && params.pump_window_release_cap == crate::APC_PUMP_WINDOW_RELEASE_CAP
            && params.pump_window_seconds == crate::APC_PUMP_WINDOW_SECONDS
            && params.minimum_counterweight_coverage_bps == crate::APC_MINIMUM_COUNTERWEIGHT_COVERAGE_BPS
            && params.counterweight_proceeds_allocation_bps == crate::APC_COUNTERWEIGHT_PROCEEDS_ALLOCATION_BPS
            && params.liquidity_reinforcement_allocation_bps == crate::APC_LIQUIDITY_REINFORCEMENT_ALLOCATION_BPS
            && params.burn_reserve_allocation_bps == crate::APC_BURN_RESERVE_ALLOCATION_BPS
            && params.operations_allocation_bps == crate::APC_OPERATIONS_ALLOCATION_BPS
            && params.base_band_release_cap == crate::APC_BASE_BAND_RELEASE_CAP
            && params.minimum_twap_minutes == crate::APC_MINIMUM_TWAP_MINUTES
            && params.minimum_liquidity_usd == crate::APC_MINIMUM_LIQUIDITY_USD
            && params.minimum_quote_liquidity_usd == crate::APC_MINIMUM_QUOTE_LIQUIDITY_USD
            && params.minimum_volume_usd == crate::APC_MINIMUM_VOLUME_USD
            && params.minimum_buy_pressure_bps == crate::APC_MINIMUM_BUY_PRESSURE_BPS
            && params.risk_velocity_thresholds_bps == crate::APC_RISK_VELOCITY_THRESHOLDS_BPS
            && params.risk_volatility_thresholds_bps == crate::APC_RISK_VOLATILITY_THRESHOLDS_BPS
            && params.risk_price_impact_thresholds_bps == crate::APC_RISK_PRICE_IMPACT_THRESHOLDS_BPS
            && params.band_interval_bps_by_risk == crate::APC_INTERVAL_BPS_BY_RISK
            && params.band_release_bps_by_risk == crate::APC_RELEASE_BPS_BY_RISK
            && params.cascade_reduction_bps == crate::APC_CASCADE_REDUCTION_BPS
            && params.recovery_spending_cap == crate::APC_RECOVERY_TOTAL_SPENDING_CAP
            && params.deferred_burn_window_cap == crate::APC_DEFERRED_BURN_WINDOW_CAP
            && params.deferred_burn_window_seconds == crate::APC_DEFERRED_BURN_WINDOW_SECONDS
            && params.deferred_burn_cooldown_seconds == crate::APC_DEFERRED_BURN_COOLDOWN_SECONDS
            && params.deferred_burn_resumption_rate_bps == crate::APC_DEFERRED_BURN_RESUMPTION_RATE_BPS
            && params.maximum_recovery_purchase_bps == crate::APC_MAXIMUM_RECOVERY_PURCHASE_BPS
            && params.minimum_counterweight_reserve_bps == crate::APC_MINIMUM_COUNTERWEIGHT_RESERVE_BPS
            && params.recovery_window_cap == crate::APC_RECOVERY_WINDOW_CAP
            && params.recovery_window_seconds == crate::APC_RECOVERY_WINDOW_SECONDS
            && params.recovery_cooldown_seconds == crate::APC_RECOVERY_COOLDOWN_SECONDS
            && params.recovery_support_drawdown_bps == crate::APC_RECOVERY_SUPPORT_DRAWDOWN_BPS
            && params.recovery_purchase_bps_by_support == crate::APC_RECOVERY_PURCHASE_BPS_BY_SUPPORT,
        PeraxError::InvalidApcPolicy
    );
    Ok(())
}

pub(crate) fn validate_apc_policy(state: &PeraxState, params: &InitializeApcParams) -> Result<()> {
    validate_exact_apc_policy_v1(params)?;
""",
)
replace_once(
    validation,
    "        observation.liquidity_usd >= config.minimum_liquidity_usd,\n        PeraxError::LiquidityGateNotMet\n    );\n    require!(\n        observation.volume_usd >= config.minimum_volume_usd,",
    "        observation.liquidity_usd >= config.minimum_liquidity_usd,\n        PeraxError::LiquidityGateNotMet\n    );\n    require!(\n        observation.quote_liquidity_usd >= config.minimum_quote_liquidity_usd,\n        PeraxError::LiquidityGateNotMet\n    );\n    require!(\n        observation.volume_usd >= config.minimum_volume_usd,",
)
replace_once(
    validation,
    "    require_checked_cap(\n        apc_state.deferred_burn_window_executed,\n        requested_amount,",
    "    let resumption_cap = amount_bps(\n        apc_state.deferred_burn_amount,\n        config.deferred_burn_resumption_rate_bps,\n    )?;\n    require!(\n        resumption_cap > 0 && requested_amount <= resumption_cap,\n        PeraxError::DeferredBurnWindowCapExceeded\n    );\n    require_checked_cap(\n        apc_state.deferred_burn_window_executed,\n        requested_amount,",
)
replace_once(
    validation,
    "pub(crate) fn validate_recovery_purchase_limits(\n    config: &ApcConfig,\n    apc_state: &ApcState,\n    requested_maximum: u64,\n    tracked_available: u64,\n    actual_vault_balance: u64,\n    now: i64,\n) -> Result<()> {",
    """pub fn recovery_purchase_bps_for_price_support(
    config: &ApcConfig,
    effective_price: u64,
    reference_price: u64,
) -> Result<u16> {
    require!(
        effective_price > 0 && reference_price > 0 && effective_price < reference_price,
        PeraxError::RecoveryNotActive
    );
    let drawdown = (u128::from(reference_price - effective_price))
        .checked_mul(APC_BPS_DENOMINATOR)
        .ok_or(PeraxError::InvalidMarketParameter)?
        .checked_div(u128::from(reference_price))
        .ok_or(PeraxError::InvalidMarketParameter)?;
    let drawdown = u16::try_from(drawdown.min(u128::from(u16::MAX)))
        .map_err(|_| PeraxError::InvalidMarketParameter)?;
    let mut selected = None;
    for (index, threshold) in config.recovery_support_drawdown_bps.iter().enumerate() {
        if drawdown >= *threshold {
            selected = Some(config.recovery_purchase_bps_by_support[index]);
        }
    }
    selected.ok_or(PeraxError::RecoverySupportBandNotMet.into())
}

pub(crate) fn validate_recovery_purchase_limits(
    config: &ApcConfig,
    apc_state: &ApcState,
    requested_maximum: u64,
    tracked_available: u64,
    actual_vault_balance: u64,
    effective_price: u64,
    reference_price: u64,
    now: i64,
) -> Result<()> {""",
)
replace_once(
    validation,
    "    let single_purchase_cap = amount_bps(tracked_available, config.maximum_recovery_purchase_bps)?;",
    "    let support_bps = recovery_purchase_bps_for_price_support(\n        config,\n        effective_price,\n        reference_price,\n    )?;\n    require!(\n        support_bps <= config.maximum_recovery_purchase_bps,\n        PeraxError::InvalidApcPolicy\n    );\n    let single_purchase_cap = amount_bps(tracked_available, support_bps)?;",
)

errors = SRC / "errors.rs"
replace_once(
    errors,
    "    #[msg(\"The requested recovery purchase exceeds the immutable per-purchase percentage cap.\")]\n    RecoveryPurchaseCapExceeded,",
    "    #[msg(\"The market drawdown has not reached an approved recovery support band.\")]\n    RecoverySupportBandNotMet,\n    #[msg(\"The requested recovery purchase exceeds the immutable per-purchase percentage cap.\")]\n    RecoveryPurchaseCapExceeded,",
)

apc_instruction = SRC / "instructions/apc.rs"
replace_once(
    apc_instruction,
    "    let config = &mut ctx.accounts.apc_config;\n    config.state = ctx.accounts.state.key();",
    "    let config = &mut ctx.accounts.apc_config;\n    config.policy_version = params.policy_version;\n    config.policy_hash = params.policy_hash;\n    config.state = ctx.accounts.state.key();",
)
replace_once(
    apc_instruction,
    "    config.minimum_counterweight_coverage_bps = params.minimum_counterweight_coverage_bps;\n    config.base_band_release_cap = params.base_band_release_cap;",
    "    config.minimum_counterweight_coverage_bps = params.minimum_counterweight_coverage_bps;\n    config.counterweight_proceeds_allocation_bps = params.counterweight_proceeds_allocation_bps;\n    config.liquidity_reinforcement_allocation_bps = params.liquidity_reinforcement_allocation_bps;\n    config.burn_reserve_allocation_bps = params.burn_reserve_allocation_bps;\n    config.operations_allocation_bps = params.operations_allocation_bps;\n    config.base_band_release_cap = params.base_band_release_cap;",
)
replace_once(
    apc_instruction,
    "    config.minimum_liquidity_usd = params.minimum_liquidity_usd;\n    config.minimum_volume_usd = params.minimum_volume_usd;",
    "    config.minimum_liquidity_usd = params.minimum_liquidity_usd;\n    config.minimum_quote_liquidity_usd = params.minimum_quote_liquidity_usd;\n    config.minimum_volume_usd = params.minimum_volume_usd;",
)
replace_once(
    apc_instruction,
    "    config.deferred_burn_cooldown_seconds = params.deferred_burn_cooldown_seconds;\n    config.maximum_recovery_purchase_bps = params.maximum_recovery_purchase_bps;",
    "    config.deferred_burn_cooldown_seconds = params.deferred_burn_cooldown_seconds;\n    config.deferred_burn_resumption_rate_bps = params.deferred_burn_resumption_rate_bps;\n    config.maximum_recovery_purchase_bps = params.maximum_recovery_purchase_bps;",
)
replace_once(
    apc_instruction,
    "    config.recovery_cooldown_seconds = params.recovery_cooldown_seconds;\n    config.is_active = true;",
    "    config.recovery_cooldown_seconds = params.recovery_cooldown_seconds;\n    config.recovery_support_drawdown_bps = params.recovery_support_drawdown_bps;\n    config.recovery_purchase_bps_by_support = params.recovery_purchase_bps_by_support;\n    config.is_active = true;",
)

recovery_instruction = SRC / "instructions/recovery.rs"
replace_once(
    recovery_instruction,
    "        tracked_available,\n        ctx.accounts.counterweight_vault.amount,\n        now,",
    "        tracked_available,\n        ctx.accounts.counterweight_vault.amount,\n        observed_price,\n        ctx.accounts.apc_state.current_reference_price,\n        now,",
)

tests = SRC / "tests.rs"
text = tests.read_text()
start = text.index("fn test_apc_params() -> InitializeApcParams {")
end = text.index("\nfn test_apc_config() -> ApcConfig {", start)
new_params = """fn test_apc_params() -> InitializeApcParams {
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
"""
text = text[:start] + new_params + text[end:]
tests.write_text(text)

replace_once(
    tests,
    "    ApcConfig {\n        state: Pubkey::new_unique(),",
    "    ApcConfig {\n        policy_version: params.policy_version,\n        policy_hash: params.policy_hash,\n        state: Pubkey::new_unique(),",
)
replace_once(
    tests,
    "        minimum_counterweight_coverage_bps: params.minimum_counterweight_coverage_bps,\n        base_band_release_cap: params.base_band_release_cap,",
    "        minimum_counterweight_coverage_bps: params.minimum_counterweight_coverage_bps,\n        counterweight_proceeds_allocation_bps: params.counterweight_proceeds_allocation_bps,\n        liquidity_reinforcement_allocation_bps: params.liquidity_reinforcement_allocation_bps,\n        burn_reserve_allocation_bps: params.burn_reserve_allocation_bps,\n        operations_allocation_bps: params.operations_allocation_bps,\n        base_band_release_cap: params.base_band_release_cap,",
)
replace_once(
    tests,
    "        minimum_liquidity_usd: params.minimum_liquidity_usd,\n        minimum_volume_usd: params.minimum_volume_usd,",
    "        minimum_liquidity_usd: params.minimum_liquidity_usd,\n        minimum_quote_liquidity_usd: params.minimum_quote_liquidity_usd,\n        minimum_volume_usd: params.minimum_volume_usd,",
)
replace_once(
    tests,
    "        deferred_burn_cooldown_seconds: params.deferred_burn_cooldown_seconds,\n        maximum_recovery_purchase_bps: params.maximum_recovery_purchase_bps,",
    "        deferred_burn_cooldown_seconds: params.deferred_burn_cooldown_seconds,\n        deferred_burn_resumption_rate_bps: params.deferred_burn_resumption_rate_bps,\n        maximum_recovery_purchase_bps: params.maximum_recovery_purchase_bps,",
)
replace_once(
    tests,
    "        recovery_cooldown_seconds: params.recovery_cooldown_seconds,\n        is_active: true,",
    "        recovery_cooldown_seconds: params.recovery_cooldown_seconds,\n        recovery_support_drawdown_bps: params.recovery_support_drawdown_bps,\n        recovery_purchase_bps_by_support: params.recovery_purchase_bps_by_support,\n        is_active: true,",
)

exact_tests = r'''

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
            assert!(config.band_interval_bps_by_risk[risk as usize]
                <= config.band_interval_bps_by_risk[(risk - 1) as usize]);
        }

        let reference = 10_000 + (seed % 1_000_000);
        let drawdown = config.recovery_support_drawdown_bps[(seed as usize) % 4] as u64;
        let effective = reference
            .saturating_mul(10_000u64.saturating_sub(drawdown))
            .checked_div(10_000)
            .unwrap()
            .max(1);
        let support = recovery_purchase_bps_for_price_support(&config, effective, reference).unwrap();
        assert!(support <= config.maximum_recovery_purchase_bps);

        let deferred = 1_000_000 * PEX_DECIMALS + (seed % 10_000) * PEX_DECIMALS;
        let resumption = amount_bps(deferred, config.deferred_burn_resumption_rate_bps).unwrap();
        assert!(resumption <= deferred);
        assert!(resumption <= config.deferred_burn_window_cap || config.deferred_burn_window_cap < resumption);

        assert!(calculate_next_band_price(u64::MAX, config.maximum_band_interval_bps).is_err());
    }
}
'''
with tests.open("a") as handle:
    handle.write(exact_tests)

print("APC Policy V1 contract constants, exact initialization binding, support bands and property tests applied")
