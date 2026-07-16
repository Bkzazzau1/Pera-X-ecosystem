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
