use anchor_lang::prelude::*;

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct InitializeParams {
    pub token_mint: Pubkey,
    pub trading_company_token_account: Pubkey,
    pub trading_company_revenue_token_account: Pubkey,
    pub max_payment_amount: u64,
    pub safety_admin: Pubkey,
    pub oracle_feed: Pubkey,
    pub launch_price: u64,
    pub current_stepped_floor: u64,
    pub daily_release_cap: u64,
    pub monthly_release_cap: u64,
    pub emergency_hourly_release_bps: u16,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct UpdateConfigParams {
    pub trading_company_token_account: Option<Pubkey>,
    pub trading_company_revenue_token_account: Option<Pubkey>,
    pub max_payment_amount: Option<u64>,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct UpdateMarketEngineConfigParams {
    pub safety_admin: Option<Pubkey>,
    pub oracle_feed: Option<Pubkey>,
    /// Legacy compatibility field. APC progression is never read from this value.
    pub current_stepped_floor: Option<u64>,
    pub daily_release_cap: Option<u64>,
    pub monthly_release_cap: Option<u64>,
    pub emergency_hourly_release_bps: Option<u16>,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum VaultClass {
    Liquidity,
    MarketReserve,
    Operations,
    CommunityRewards,
    EmergencyReserve,
    Vesting,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct InitializeReserveVaultParams {
    pub allocation_id: [u8; 32],
    pub vault_class: VaultClass,
    pub allocation_cap: u64,
    pub authorized_source_owner: Pubkey,
    pub authorized_source_token_account: Pubkey,
    pub approved_destination_owner: Pubkey,
    pub approved_destination_token_account: Pubkey,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReleaseType {
    Growth,
    Emergency,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum BurnFulfillmentSource {
    OpenMarketPurchase,
    TradingTreasury,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct MarketConditionSnapshot {
    pub observed_price: u64,
    pub twap_minutes: u64,
    pub liquidity_usd: u64,
    pub net_buy_volume_bps: u16,
    pub downside_move_bps: u16,
    pub liquidity_drain_bps: u16,
    pub emergency_reserve_available_amount: u64,
    pub observed_at: i64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct MarketConditionalReleaseParams {
    pub release_type: ReleaseType,
    pub requested_amount: u64,
    pub release_id: [u8; 32],
    pub snapshot: MarketConditionSnapshot,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct VaultMarketConditionalReleaseParams {
    pub allocation_id: [u8; 32],
    pub release_type: ReleaseType,
    pub requested_amount: u64,
    pub release_id: [u8; 32],
    pub market_observation_id: [u8; 32],
    pub destination_token_account: Pubkey,
    pub snapshot: MarketConditionSnapshot,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct MarketConditionBurnParams {
    pub amount: u64,
    pub eligible_revenue_amount: u64,
    pub burn_rate_bps: u16,
    pub market_health_score: u8,
    pub observed_at: i64,
    pub decision_id: [u8; 32],
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct ConditionalBuybackBurnParams {
    pub amount: u64,
    pub eligible_revenue_amount: u64,
    pub burn_rate_bps: u16,
    pub market_health_score: u8,
    pub observed_at: i64,
    pub decision_id: [u8; 32],
    pub burn_source: BurnFulfillmentSource,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq, Eq, InitSpace)]
pub enum ApcStatus {
    Inactive,
    Armed,
    Active,
    PumpControl,
    AwaitingAbsorption,
    Recovery,
    Paused,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct InitializeRecoveryPoolParams {
    pub pool_id: [u8; 32],
    pub fee_bps: u16,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct RecoverySwapAdapterParams {
    pub quote_amount: u64,
    pub minimum_pex_out: u64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct InitializeApcParams {
    pub policy_version: u16,
    pub policy_hash: [u8; 32],
    pub quote_mint: Pubkey,
    pub approved_pool: Pubkey,
    pub approved_proceeds_owner: Pubkey,
    pub approved_proceeds_token_account: Pubkey,
    pub approved_recovery_program: Pubkey,
    pub price_scale: u64,
    pub first_activation_price: u64,
    pub minimum_band_interval_bps: u16,
    pub maximum_band_interval_bps: u16,
    pub maximum_observation_age_seconds: i64,
    pub maximum_future_clock_skew_seconds: i64,
    pub hourly_release_cap: u64,
    pub pump_window_release_cap: u64,
    pub pump_window_seconds: i64,
    pub minimum_counterweight_coverage_bps: u16,
    pub counterweight_proceeds_allocation_bps: u16,
    pub liquidity_reinforcement_allocation_bps: u16,
    pub burn_reserve_allocation_bps: u16,
    pub operations_allocation_bps: u16,
    pub base_band_release_cap: u64,
    pub minimum_twap_minutes: u64,
    pub minimum_liquidity_usd: u64,
    pub minimum_quote_liquidity_usd: u64,
    pub minimum_volume_usd: u64,
    pub minimum_buy_pressure_bps: u16,
    pub risk_velocity_thresholds_bps: [u32; 3],
    pub risk_volatility_thresholds_bps: [u32; 3],
    pub risk_price_impact_thresholds_bps: [u32; 3],
    pub band_interval_bps_by_risk: [u16; 4],
    pub band_release_bps_by_risk: [u16; 4],
    pub cascade_reduction_bps: [u16; 4],
    pub recovery_spending_cap: u64,
    pub deferred_burn_window_cap: u64,
    pub deferred_burn_window_seconds: i64,
    pub deferred_burn_cooldown_seconds: i64,
    pub deferred_burn_resumption_rate_bps: u16,
    pub maximum_recovery_purchase_bps: u16,
    pub minimum_counterweight_reserve_bps: u16,
    pub recovery_window_cap: u64,
    pub recovery_window_seconds: i64,
    pub recovery_cooldown_seconds: i64,
    pub recovery_support_drawdown_bps: [u16; 4],
    pub recovery_purchase_bps_by_support: [u16; 4],
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct SubmitApcObservationParams {
    pub observation_id: [u8; 32],
    pub sequence: u64,
    pub pool: Pubkey,
    pub spot_price: u64,
    pub twap_price: u64,
    pub twap_minutes: u64,
    pub liquidity_usd: u64,
    pub quote_liquidity_usd: u64,
    pub volume_usd: u64,
    pub net_buy_pressure_bps: u16,
    pub price_velocity_bps: u32,
    pub volatility_bps: u32,
    pub estimated_price_impact_bps: u32,
    pub observed_at: i64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct ActivateApcBandParams {
    pub band_index: u32,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct ExecuteApcReleaseParams {
    pub release_id: [u8; 32],
    pub allocation_id: [u8; 32],
    pub band_index: u32,
    pub observation_id: [u8; 32],
    pub amount: u64,
    pub destination_token_account: Pubkey,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct DepositCounterweightParams {
    pub deposit_id: [u8; 32],
    pub amount: u64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct RecordDeferredBurnParams {
    pub decision_id: [u8; 32],
    pub amount: u64,
    pub observed_at: i64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct ExecuteDeferredBurnParams {
    pub amount: u64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct ExecuteCounterweightPurchaseParams {
    pub recovery_id: [u8; 32],
    pub observation_id: [u8; 32],
    pub maximum_quote_amount: u64,
    pub minimum_pex_out: u64,
    pub swap_instruction_data: Vec<u8>,
}

#[account]
#[derive(InitSpace)]
pub struct PeraxState {
    pub authority: Pubkey,
    pub pending_authority: Pubkey,
    pub has_pending_authority: bool,
    pub token_mint: Pubkey,
    pub trading_company_token_account: Pubkey,
    pub trading_company_revenue_token_account: Pubkey,
    pub max_payment_amount: u64,
    pub safety_admin: Pubkey,
    pub oracle_feed: Pubkey,
    pub launch_price: u64,
    /// Legacy field retained solely for account layout compatibility. APC never reads it.
    pub current_stepped_floor: u64,
    pub last_release_timestamp: i64,
    pub daily_unlocked_accumulator: u64,
    pub monthly_unlocked_accumulator: u64,
    pub daily_window_start: i64,
    pub monthly_window_start: i64,
    pub daily_release_cap: u64,
    pub monthly_release_cap: u64,
    pub emergency_hourly_release_bps: u16,
    pub daily_burn_accumulator: u64,
    pub daily_burn_window_start: i64,
    pub is_paused: bool,
    pub emergency_pause: bool,
    pub bump: u8,
}

#[account]
#[derive(InitSpace)]
pub struct RecoveryPoolConfig {
    pub state: Pubkey,
    pub pool_id: [u8; 32],
    pub quote_mint: Pubkey,
    pub pex_mint: Pubkey,
    pub pool_authority: Pubkey,
    pub pool_quote_vault: Pubkey,
    pub pool_pex_vault: Pubkey,
    pub fee_bps: u16,
    pub is_active: bool,
    pub bump: u8,
    pub authority_bump: u8,
}

#[account]
#[derive(InitSpace)]
pub struct ApcConfig {
    pub policy_version: u16,
    pub policy_hash: [u8; 32],
    pub state: Pubkey,
    pub oracle_feed: Pubkey,
    pub quote_mint: Pubkey,
    pub approved_pool: Pubkey,
    pub approved_proceeds_owner: Pubkey,
    pub approved_proceeds_token_account: Pubkey,
    pub approved_recovery_program: Pubkey,
    pub price_scale: u64,
    pub first_activation_price: u64,
    pub minimum_band_interval_bps: u16,
    pub maximum_band_interval_bps: u16,
    pub maximum_observation_age_seconds: i64,
    pub maximum_future_clock_skew_seconds: i64,
    pub hourly_release_cap: u64,
    pub pump_window_release_cap: u64,
    pub pump_window_seconds: i64,
    pub minimum_counterweight_coverage_bps: u16,
    pub counterweight_proceeds_allocation_bps: u16,
    pub liquidity_reinforcement_allocation_bps: u16,
    pub burn_reserve_allocation_bps: u16,
    pub operations_allocation_bps: u16,
    pub base_band_release_cap: u64,
    pub minimum_twap_minutes: u64,
    pub minimum_liquidity_usd: u64,
    pub minimum_quote_liquidity_usd: u64,
    pub minimum_volume_usd: u64,
    pub minimum_buy_pressure_bps: u16,
    pub risk_velocity_thresholds_bps: [u32; 3],
    pub risk_volatility_thresholds_bps: [u32; 3],
    pub risk_price_impact_thresholds_bps: [u32; 3],
    pub band_interval_bps_by_risk: [u16; 4],
    pub band_release_bps_by_risk: [u16; 4],
    pub cascade_reduction_bps: [u16; 4],
    pub recovery_spending_cap: u64,
    pub deferred_burn_window_cap: u64,
    pub deferred_burn_window_seconds: i64,
    pub deferred_burn_cooldown_seconds: i64,
    pub deferred_burn_resumption_rate_bps: u16,
    pub maximum_recovery_purchase_bps: u16,
    pub minimum_counterweight_reserve_bps: u16,
    pub recovery_window_cap: u64,
    pub recovery_window_seconds: i64,
    pub recovery_cooldown_seconds: i64,
    pub recovery_support_drawdown_bps: [u16; 4],
    pub recovery_purchase_bps_by_support: [u16; 4],
    pub is_active: bool,
    pub is_paused: bool,
    pub bump: u8,
}

#[account]
#[derive(InitSpace)]
pub struct ApcState {
    pub config: Pubkey,
    pub status: ApcStatus,
    pub status_before_pause: ApcStatus,
    pub current_reference_price: u64,
    pub next_band_price: u64,
    pub current_band_index: u32,
    pub highest_crossed_band_index: u32,
    pub pump_window_started_at: i64,
    pub pump_window_released: u64,
    pub hourly_window_started_at: i64,
    pub hourly_released: u64,
    pub total_apc_released: u64,
    pub total_counterweight_credited: u64,
    pub total_counterweight_spent: u64,
    pub last_observation_sequence: u64,
    pub last_release_timestamp: i64,
    pub deferred_burn_amount: u64,
    pub unconfirmed_release_amount: u64,
    pub last_release_observation_id: [u8; 32],
    pub recovery_entry_observation_id: [u8; 32],
    pub cascade_observation_id: [u8; 32],
    pub cascade_band_count: u32,
    pub active_risk_tier: u8,
    pub deferred_burn_window_started_at: i64,
    pub deferred_burn_window_executed: u64,
    pub last_deferred_burn_timestamp: i64,
    pub recovery_window_started_at: i64,
    pub recovery_window_spent: u64,
    pub last_recovery_purchase_timestamp: i64,
    pub bump: u8,
}

#[account]
#[derive(InitSpace)]
pub struct ApcObservation {
    pub observation_id: [u8; 32],
    pub sequence: u64,
    pub oracle_feed: Pubkey,
    pub pool: Pubkey,
    pub spot_price: u64,
    pub twap_price: u64,
    pub twap_minutes: u64,
    pub liquidity_usd: u64,
    pub quote_liquidity_usd: u64,
    pub volume_usd: u64,
    pub net_buy_pressure_bps: u16,
    pub price_velocity_bps: u32,
    pub volatility_bps: u32,
    pub estimated_price_impact_bps: u32,
    pub observed_at: i64,
    pub submitted_at: i64,
    pub is_consumed_for_release: bool,
    pub is_consumed_for_confirmation: bool,
    pub is_consumed_for_recovery: bool,
    pub consumed_by_release: Pubkey,
    pub consumed_by_recovery: Pubkey,
    pub bump: u8,
}

#[account]
#[derive(InitSpace)]
pub struct ApcBandRecord {
    pub apc_state: Pubkey,
    pub band_index: u32,
    pub trigger_price: u64,
    pub interval_bps: u16,
    pub risk_tier: u8,
    pub maximum_release_amount: u64,
    pub amount_released: u64,
    pub activation_observation_id: [u8; 32],
    pub first_observed_at: i64,
    pub last_release_at: i64,
    pub is_crossed: bool,
    pub is_exhausted: bool,
    pub bump: u8,
}

#[account]
#[derive(InitSpace)]
pub struct ApcReleaseRecord {
    pub release_id: [u8; 32],
    pub band_index: u32,
    pub band_record: Pubkey,
    pub allocation_id: [u8; 32],
    pub vault_config: Pubkey,
    pub destination_token_account: Pubkey,
    pub observation_id: [u8; 32],
    pub amount: u64,
    pub band_released_after: u64,
    pub pump_window_released_after: u64,
    pub unconfirmed_release_after: u64,
    pub counterweight_required_after: u64,
    pub executed_at: i64,
    pub bump: u8,
}

#[account]
#[derive(InitSpace)]
pub struct CounterweightConfig {
    pub apc_config: Pubkey,
    pub state: Pubkey,
    pub quote_mint: Pubkey,
    pub pex_mint: Pubkey,
    pub counterweight_authority: Pubkey,
    pub counterweight_vault: Pubkey,
    pub deferred_burn_authority: Pubkey,
    pub deferred_burn_vault: Pubkey,
    pub recovery_authority: Pubkey,
    pub recovery_vault: Pubkey,
    pub approved_proceeds_owner: Pubkey,
    pub approved_proceeds_token_account: Pubkey,
    pub approved_recovery_program: Pubkey,
    pub approved_pool: Pubkey,
    pub bump: u8,
    pub counterweight_authority_bump: u8,
    pub deferred_burn_authority_bump: u8,
    pub recovery_authority_bump: u8,
}

#[account]
#[derive(InitSpace)]
pub struct CounterweightDepositRecord {
    pub deposit_id: [u8; 32],
    pub source_owner: Pubkey,
    pub source_token_account: Pubkey,
    pub amount: u64,
    pub credited_after: u64,
    pub deposited_at: i64,
    pub bump: u8,
}

#[account]
#[derive(InitSpace)]
pub struct DeferredBurnRecord {
    pub decision_id: [u8; 32],
    pub state: Pubkey,
    pub apc_state: Pubkey,
    pub source_token_account: Pubkey,
    pub amount: u64,
    pub amount_executed: u64,
    pub observed_at: i64,
    pub recorded_at: i64,
    pub last_executed_at: i64,
    pub is_complete: bool,
    pub bump: u8,
}

#[account]
#[derive(InitSpace)]
pub struct ApcRecoveryRecord {
    pub recovery_id: [u8; 32],
    pub observation_id: [u8; 32],
    pub apc_state: Pubkey,
    pub counterweight_config: Pubkey,
    pub quote_spent: u64,
    pub pex_received: u64,
    pub executed_at: i64,
    pub bump: u8,
}

#[account]
pub struct ReserveVaultConfig {
    pub state: Pubkey,
    pub allocation_id: [u8; 32],
    pub vault_class: VaultClass,
    pub token_mint: Pubkey,
    pub vault_authority: Pubkey,
    pub vault_token_account: Pubkey,
    pub authorized_source_owner: Pubkey,
    pub authorized_source_token_account: Pubkey,
    pub approved_destination_owner: Pubkey,
    pub approved_destination_token_account: Pubkey,
    pub allocation_cap: u64,
    pub authorized_deposited: u64,
    pub unsolicited_balance: u64,
    pub total_released: u64,
    pub is_active: bool,
    pub is_paused: bool,
    pub authority_bump: u8,
    pub config_bump: u8,
}

impl ReserveVaultConfig {
    pub const SPACE: usize = (9 * 32) + 1 + (4 * 8) + 4;
}

#[account]
pub struct PaymentRecord {
    pub reference: [u8; 32],
    pub payer: Pubkey,
    pub amount: u64,
    pub token_mint: Pubkey,
    pub trading_company_token_account: Pubkey,
    pub trading_company_revenue_token_account: Pubkey,
    pub created_at: i64,
    pub bump: u8,
}

impl PaymentRecord {
    pub const SPACE: usize = 32 + 32 + 8 + 32 + 32 + 32 + 8 + 1;
}

#[account]
pub struct ReleaseRecord {
    pub release_id: [u8; 32],
    pub oracle_feed: Pubkey,
    pub release_type: ReleaseType,
    pub requested_amount: u64,
    pub observed_price: u64,
    pub twap_minutes: u64,
    pub liquidity_usd: u64,
    pub net_buy_volume_bps: u16,
    pub observed_at: i64,
    pub recorded_at: i64,
    pub bump: u8,
}

impl ReleaseRecord {
    pub const SPACE: usize = 32 + 32 + 1 + 8 + 8 + 8 + 8 + 2 + 8 + 8 + 1;
}

#[account]
pub struct ReserveReleaseRecord {
    pub release_id: [u8; 32],
    pub state: Pubkey,
    pub allocation_id: [u8; 32],
    pub vault_config: Pubkey,
    pub vault_class: VaultClass,
    pub vault_token_account: Pubkey,
    pub destination_token_account: Pubkey,
    pub oracle_feed: Pubkey,
    pub release_type: ReleaseType,
    pub requested_amount: u64,
    pub observed_price: u64,
    pub twap_minutes: u64,
    pub liquidity_usd: u64,
    pub net_buy_volume_bps: u16,
    pub market_observation_id: [u8; 32],
    pub observed_at: i64,
    pub executed_at: i64,
    pub bump: u8,
}

impl ReserveReleaseRecord {
    pub const SPACE: usize =
        32 + 32 + 32 + 32 + 1 + 32 + 32 + 32 + 1 + 8 + 8 + 8 + 8 + 2 + 32 + 8 + 8 + 1;
}

#[account]
pub struct BurnExecutionRecord {
    pub decision_id: [u8; 32],
    pub authority: Pubkey,
    pub trading_company_authority: Pubkey,
    pub token_mint: Pubkey,
    pub trading_company_revenue_token_account: Pubkey,
    pub source_token_account: Pubkey,
    pub burn_source: BurnFulfillmentSource,
    pub amount: u64,
    pub eligible_revenue_amount: u64,
    pub burn_rate_bps: u16,
    pub market_health_score: u8,
    pub observed_at: i64,
    pub executed_at: i64,
    pub bump: u8,
}

impl BurnExecutionRecord {
    pub const SPACE: usize = 32 + 32 + 32 + 32 + 32 + 32 + 1 + 8 + 8 + 2 + 1 + 8 + 8 + 1;
}
