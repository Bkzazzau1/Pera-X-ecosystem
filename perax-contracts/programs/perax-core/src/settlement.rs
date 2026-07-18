pub use crate::PeraxError as SettlementError;
use crate::{PeraxError, PeraxState};
use anchor_lang::prelude::*;

pub const SETTLEMENT_FUNDING_PEX: u8 = 1;
pub const SETTLEMENT_FUNDING_STABLECOIN: u8 = 1 << 1;
pub const SETTLEMENT_FUNDING_FIAT: u8 = 1 << 2;
pub const SETTLEMENT_FUNDING_VIRTUAL_ACCOUNT: u8 = 1 << 3;
pub const SETTLEMENT_ALL_FUNDING_METHODS: u8 = SETTLEMENT_FUNDING_PEX
    | SETTLEMENT_FUNDING_STABLECOIN
    | SETTLEMENT_FUNDING_FIAT
    | SETTLEMENT_FUNDING_VIRTUAL_ACCOUNT;

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq, Eq, InitSpace)]
pub enum SettlementFundingMethod {
    Pex,
    Stablecoin,
    Fiat,
    VirtualAccount,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq, Eq, InitSpace)]
pub enum SettlementMarketMode {
    DirectPex,
    MarketPurchase,
    PolicyVault,
    Hybrid,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq, Eq, InitSpace)]
pub enum SettlementDisposition {
    UtilityPayment,
    CustomerDelivery,
    Burn,
    Lock,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq, Eq, InitSpace)]
pub enum SettlementStatus {
    Planned,
    Funding,
    Ready,
    Finalized,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct InitializeSettlementPolicyParams {
    pub market_share_bps_by_risk: [u16; 4],
    pub maximum_market_slippage_bps: u16,
    pub maximum_quantity_per_settlement: u64,
    pub daily_market_quote_cap: u64,
    pub daily_market_pex_cap: u64,
    pub daily_policy_vault_pex_cap: u64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct InitializeProductSettlementPolicyParams {
    pub product_id: [u8; 32],
    /// Quote-token base units per product unit. The configured quote mint uses six decimals.
    pub unit_quote_value: u64,
    pub maximum_quantity: u64,
    pub accepted_funding_mask: u8,
    pub disposition: SettlementDisposition,
    /// Required for UtilityPayment; ignored for customer delivery, burn, and lock.
    pub fixed_destination_token_account: Pubkey,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct UpdateProductSettlementPolicyParams {
    pub product_id: [u8; 32],
    pub unit_quote_value: Option<u64>,
    pub maximum_quantity: Option<u64>,
    pub accepted_funding_mask: Option<u8>,
    pub disposition: Option<SettlementDisposition>,
    pub fixed_destination_token_account: Option<Pubkey>,
    pub is_active: Option<bool>,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct PlanSettlementParams {
    pub settlement_id: [u8; 32],
    pub product_id: [u8; 32],
    pub observation_id: [u8; 32],
    pub funding_method: SettlementFundingMethod,
    pub quantity: u64,
    pub beneficiary: Pubkey,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct FundDirectPexSettlementParams {
    pub settlement_id: [u8; 32],
    pub amount: u64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct ExecuteSettlementMarketPurchaseParams {
    pub settlement_id: [u8; 32],
    pub maximum_quote_amount: u64,
    pub minimum_pex_out: u64,
    pub swap_instruction_data: Vec<u8>,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct ExecuteSettlementVaultFundingParams {
    pub settlement_id: [u8; 32],
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct FinalizeSettlementParams {
    pub settlement_id: [u8; 32],
}

#[account]
#[derive(InitSpace)]
pub struct SettlementPolicy {
    pub state: Pubkey,
    pub apc_config: Pubkey,
    pub counterweight_config: Pubkey,
    pub quote_mint: Pubkey,
    pub pex_mint: Pubkey,
    pub approved_market_program: Pubkey,
    pub approved_market_pool: Pubkey,
    pub approved_policy_vault_config: Pubkey,
    /// Reserved compatibility field. Active custody is isolated in SettlementCustody.
    pub settlement_authority: Pubkey,
    /// Reserved compatibility field. Active custody is isolated per settlement.
    pub settlement_pex_vault: Pubkey,
    pub lock_vault: Pubkey,
    pub market_share_bps_by_risk: [u16; 4],
    pub maximum_market_slippage_bps: u16,
    pub maximum_quantity_per_settlement: u64,
    pub daily_market_quote_cap: u64,
    pub daily_market_pex_cap: u64,
    pub daily_policy_vault_pex_cap: u64,
    pub daily_window_started_at: i64,
    pub daily_market_quote_spent: u64,
    pub daily_market_pex_received: u64,
    pub daily_policy_vault_pex_released: u64,
    pub is_active: bool,
    pub bump: u8,
    /// Reserved compatibility field. Active custody uses SettlementCustody::authority_bump.
    pub settlement_authority_bump: u8,
}

#[account]
#[derive(InitSpace)]
pub struct ProductSettlementPolicy {
    pub settlement_policy: Pubkey,
    pub product_id: [u8; 32],
    pub unit_quote_value: u64,
    pub maximum_quantity: u64,
    pub accepted_funding_mask: u8,
    pub disposition: SettlementDisposition,
    pub fixed_destination_token_account: Pubkey,
    pub is_active: bool,
    pub bump: u8,
}

#[account]
#[derive(InitSpace)]
pub struct SettlementRecord {
    pub settlement_id: [u8; 32],
    pub settlement_policy: Pubkey,
    pub product_policy: Pubkey,
    pub product_id: [u8; 32],
    pub initiator: Pubkey,
    pub beneficiary: Pubkey,
    pub funding_method: SettlementFundingMethod,
    pub market_mode: SettlementMarketMode,
    pub disposition: SettlementDisposition,
    pub status: SettlementStatus,
    pub observation_id: [u8; 32],
    pub effective_price: u64,
    pub risk_tier: u8,
    pub quantity: u64,
    pub quote_value: u64,
    pub pex_obligation: u64,
    pub market_pex_required: u64,
    pub policy_vault_pex_required: u64,
    pub direct_pex_received: u64,
    pub market_quote_spent: u64,
    pub market_pex_received: u64,
    pub policy_vault_pex_received: u64,
    pub destination_token_account: Pubkey,
    pub funding_source_token_account: Pubkey,
    pub created_at: i64,
    pub finalized_at: i64,
    pub final_pex_amount: u64,
    pub surplus_locked: u64,
    pub bump: u8,
}

#[derive(Accounts)]
#[instruction(params: InitializeProductSettlementPolicyParams)]
pub struct InitializeProductSettlementPolicy<'info> {
    #[account(
        seeds = [b"perax-state"],
        bump = state.bump,
        has_one = authority @ PeraxError::Unauthorized
    )]
    pub state: Box<Account<'info, PeraxState>>,
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(
        seeds = [b"settlement-policy", state.key().as_ref()],
        bump = settlement_policy.bump,
        constraint = settlement_policy.state == state.key() @ SettlementError::InvalidPolicy
    )]
    pub settlement_policy: Box<Account<'info, SettlementPolicy>>,
    #[account(
        init,
        payer = authority,
        space = 8 + ProductSettlementPolicy::INIT_SPACE,
        seeds = [b"product-settlement", params.product_id.as_ref()],
        bump
    )]
    pub product_policy: Box<Account<'info, ProductSettlementPolicy>>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(params: UpdateProductSettlementPolicyParams)]
pub struct UpdateProductSettlementPolicy<'info> {
    #[account(
        seeds = [b"perax-state"],
        bump = state.bump,
        has_one = authority @ PeraxError::Unauthorized
    )]
    pub state: Box<Account<'info, PeraxState>>,
    pub authority: Signer<'info>,
    #[account(
        seeds = [b"settlement-policy", state.key().as_ref()],
        bump = settlement_policy.bump
    )]
    pub settlement_policy: Box<Account<'info, SettlementPolicy>>,
    #[account(
        mut,
        seeds = [b"product-settlement", params.product_id.as_ref()],
        bump = product_policy.bump,
        constraint = product_policy.settlement_policy == settlement_policy.key() @ SettlementError::InvalidPolicy,
        constraint = product_policy.product_id == params.product_id @ SettlementError::InvalidPolicy
    )]
    pub product_policy: Box<Account<'info, ProductSettlementPolicy>>,
}

#[event]
pub struct SettlementPolicyInitialized {
    pub settlement_policy: Pubkey,
    pub approved_market_program: Pubkey,
    pub approved_market_pool: Pubkey,
    pub approved_policy_vault_config: Pubkey,
    /// Default when per-settlement isolated custody is active.
    pub settlement_pex_vault: Pubkey,
    pub lock_vault: Pubkey,
    pub initialized_at: i64,
}

#[event]
pub struct ProductSettlementPolicyInitialized {
    pub product_policy: Pubkey,
    pub product_id: [u8; 32],
    pub unit_quote_value: u64,
    pub maximum_quantity: u64,
    pub accepted_funding_mask: u8,
    pub disposition: SettlementDisposition,
    pub fixed_destination_token_account: Pubkey,
    pub initialized_at: i64,
}

#[event]
pub struct ProductSettlementPolicyUpdated {
    pub product_policy: Pubkey,
    pub product_id: [u8; 32],
    pub unit_quote_value: u64,
    pub maximum_quantity: u64,
    pub accepted_funding_mask: u8,
    pub disposition: SettlementDisposition,
    pub fixed_destination_token_account: Pubkey,
    pub is_active: bool,
    pub updated_at: i64,
}

#[event]
pub struct SettlementPlanned {
    pub settlement_record: Pubkey,
    pub settlement_id: [u8; 32],
    pub product_id: [u8; 32],
    pub observation_id: [u8; 32],
    pub funding_method: SettlementFundingMethod,
    pub market_mode: SettlementMarketMode,
    pub disposition: SettlementDisposition,
    pub quote_value: u64,
    pub pex_obligation: u64,
    pub market_pex_required: u64,
    pub policy_vault_pex_required: u64,
    pub planned_at: i64,
}

#[event]
pub struct DirectPexSettlementFunded {
    pub settlement_record: Pubkey,
    pub source_token_account: Pubkey,
    pub amount: u64,
    pub funded_at: i64,
}

#[event]
pub struct SettlementMarketPurchaseExecuted {
    pub settlement_record: Pubkey,
    pub quote_source_token_account: Pubkey,
    pub quote_spent: u64,
    pub pex_received: u64,
    pub executed_at: i64,
}

#[event]
pub struct SettlementPolicyVaultFunded {
    pub settlement_record: Pubkey,
    pub reserve_vault_config: Pubkey,
    pub pex_received: u64,
    pub funded_at: i64,
}

#[event]
pub struct SettlementFinalized {
    pub settlement_record: Pubkey,
    pub settlement_id: [u8; 32],
    pub disposition: SettlementDisposition,
    pub destination_token_account: Pubkey,
    pub final_pex_amount: u64,
    pub surplus_locked: u64,
    pub finalized_at: i64,
}
