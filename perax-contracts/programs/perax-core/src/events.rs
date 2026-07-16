use crate::{BurnFulfillmentSource, ReleaseType, VaultClass};
use anchor_lang::prelude::*;

#[event]
pub struct ConfigInitialized {
    pub authority: Pubkey,
    pub token_mint: Pubkey,
    pub trading_company_token_account: Pubkey,
    pub trading_company_revenue_token_account: Pubkey,
    pub safety_admin: Pubkey,
    pub oracle_feed: Pubkey,
    pub launch_price: u64,
    pub daily_release_cap: u64,
    pub monthly_release_cap: u64,
    pub max_payment_amount: u64,
}

#[event]
pub struct ConfigUpdated {
    pub authority: Pubkey,
    pub trading_company_token_account: Pubkey,
    pub trading_company_revenue_token_account: Pubkey,
    pub max_payment_amount: u64,
}

#[event]
pub struct MarketEngineConfigUpdated {
    pub authority: Pubkey,
    pub safety_admin: Pubkey,
    pub oracle_feed: Pubkey,
    pub current_stepped_floor: u64,
    pub daily_release_cap: u64,
    pub monthly_release_cap: u64,
    pub emergency_hourly_release_bps: u16,
}

#[event]
pub struct PauseStatusChanged {
    pub authority: Pubkey,
    pub is_paused: bool,
}

#[event]
pub struct EmergencyPauseStatusChanged {
    pub safety_admin: Pubkey,
    pub emergency_pause: bool,
}

#[event]
pub struct ReserveVaultInitialized {
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
    pub initialized_by: Pubkey,
    pub initialized_at: i64,
}

#[event]
pub struct ReserveVaultDepositReceived {
    pub state: Pubkey,
    pub allocation_id: [u8; 32],
    pub vault_class: VaultClass,
    pub source_owner: Pubkey,
    pub source_token_account: Pubkey,
    pub vault_token_account: Pubkey,
    pub amount: u64,
    pub authorized_deposited: u64,
    pub unsolicited_balance: u64,
    pub vault_balance_after: u64,
    pub deposited_at: i64,
}

#[event]
pub struct ReserveVaultReleaseExecuted {
    pub state: Pubkey,
    pub allocation_id: [u8; 32],
    pub vault_class: VaultClass,
    pub vault_config: Pubkey,
    pub vault_token_account: Pubkey,
    pub destination_token_account: Pubkey,
    pub release_record: Pubkey,
    pub release_id: [u8; 32],
    pub release_type: ReleaseType,
    pub release_amount: u64,
    pub remaining_vault_balance: u64,
    pub authorized_deposited: u64,
    pub unsolicited_balance: u64,
    pub total_released: u64,
    pub market_observation_id: [u8; 32],
    pub observed_at: i64,
    pub executed_at: i64,
}

#[event]
pub struct ReserveVaultPaused {
    pub state: Pubkey,
    pub allocation_id: [u8; 32],
    pub vault_token_account: Pubkey,
    pub is_paused: bool,
    pub actor: Pubkey,
    pub changed_at: i64,
}

#[event]
pub struct ReserveVaultReconciled {
    pub state: Pubkey,
    pub allocation_id: [u8; 32],
    pub vault_token_account: Pubkey,
    pub authorized_deposited: u64,
    pub total_released: u64,
    pub previous_unsolicited_balance: u64,
    pub reconciled_unsolicited_balance: u64,
    pub actual_vault_balance: u64,
    pub reconciled_by: Pubkey,
    pub reconciled_at: i64,
}

#[event]
pub struct MarketConditionalReleaseApproved {
    pub release_record: Pubkey,
    pub oracle_feed: Pubkey,
    pub release_type: ReleaseType,
    pub requested_amount: u64,
    pub release_id: [u8; 32],
    pub observed_price: u64,
    pub twap_minutes: u64,
    pub liquidity_usd: u64,
    pub net_buy_volume_bps: u16,
    pub daily_unlocked_accumulator: u64,
    pub monthly_unlocked_accumulator: u64,
    pub observed_at: i64,
}

#[event]
pub struct AuthorityTransferNominated {
    pub current_authority: Pubkey,
    pub pending_authority: Pubkey,
}

#[event]
pub struct AuthorityTransferCancelled {
    pub authority: Pubkey,
    pub cancelled_authority: Pubkey,
}

#[event]
pub struct AuthorityTransferAccepted {
    pub previous_authority: Pubkey,
    pub new_authority: Pubkey,
}

#[event]
pub struct UtilityPaymentReceived {
    pub payer: Pubkey,
    pub token_mint: Pubkey,
    pub trading_company_token_account: Pubkey,
    pub trading_company_revenue_token_account: Pubkey,
    pub amount: u64,
    pub reference: [u8; 32],
}

#[event]
pub struct ExternalUtilityPaymentRecorded {
    pub authority: Pubkey,
    pub token_mint: Pubkey,
    pub amount: u64,
    pub reference: [u8; 32],
    pub payment_source: [u8; 16],
}

#[event]
pub struct TradingCompanyBurnExecuted {
    pub authority: Pubkey,
    pub trading_company_authority: Pubkey,
    pub token_mint: Pubkey,
    pub trading_company_token_account: Pubkey,
    pub trading_company_revenue_token_account: Pubkey,
    pub amount: u64,
    pub decision_id: [u8; 32],
}

#[event]
pub struct MarketConditionBurnExecuted {
    pub burn_record: Pubkey,
    pub authority: Pubkey,
    pub trading_company_authority: Pubkey,
    pub token_mint: Pubkey,
    pub trading_company_revenue_token_account: Pubkey,
    pub amount: u64,
    pub eligible_revenue_amount: u64,
    pub burn_rate_bps: u16,
    pub market_health_score: u8,
    pub decision_id: [u8; 32],
    pub observed_at: i64,
    pub executed_at: i64,
}

#[event]
pub struct ConditionalBuybackBurnExecuted {
    pub burn_record: Pubkey,
    pub authority: Pubkey,
    pub source_authority: Pubkey,
    pub token_mint: Pubkey,
    pub source_token_account: Pubkey,
    pub burn_source: BurnFulfillmentSource,
    pub amount: u64,
    pub eligible_revenue_amount: u64,
    pub burn_rate_bps: u16,
    pub market_health_score: u8,
    pub decision_id: [u8; 32],
    pub observed_at: i64,
    pub executed_at: i64,
}
