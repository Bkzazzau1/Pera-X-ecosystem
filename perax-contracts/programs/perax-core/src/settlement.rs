use crate::{
    ApcConfig, ApcObservation, ApcState, CounterweightConfig, PeraxError, PeraxState,
    ReserveVaultConfig, VaultClass, APC_QUOTE_DECIMALS, PEX_MINT_DECIMALS,
};
use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Mint, Token, TokenAccount},
};

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
    /// Quote-token base units per product unit. The configured quote mint must use six decimals.
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
    pub settlement_authority: Pubkey,
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

#[error_code]
pub enum SettlementError {
    #[msg("The settlement policy is invalid.")]
    InvalidPolicy,
    #[msg("The settlement policy is inactive.")]
    PolicyInactive,
    #[msg("The product settlement policy is inactive.")]
    ProductInactive,
    #[msg("The selected funding method is not accepted for this product.")]
    FundingMethodNotAccepted,
    #[msg("The requested product quantity is invalid or exceeds policy.")]
    InvalidQuantity,
    #[msg("Market-funded settlement is disabled during APC pump protection.")]
    MarketActionPaused,
    #[msg("The settlement is not in the required state for this action.")]
    InvalidSettlementStatus,
    #[msg("The settlement source does not match the contract-derived market mode.")]
    InvalidSettlementMode,
    #[msg("The settlement destination is invalid.")]
    InvalidSettlementDestination,
    #[msg("The approved atomic market adapter or pool is invalid.")]
    InvalidMarketAdapter,
    #[msg("The market adapter did not spend and receive the required assets atomically.")]
    InvalidMarketSettlement,
    #[msg("The policy vault cannot satisfy the contract-derived settlement amount.")]
    PolicyVaultUnavailable,
    #[msg("A settlement daily cap would be exceeded.")]
    SettlementDailyCapExceeded,
    #[msg("The settlement has not received enough PEX to finalize.")]
    SettlementNotFunded,
    #[msg("Settlement arithmetic overflowed or produced an invalid amount.")]
    SettlementArithmeticError,
}

#[event]
pub struct SettlementPolicyInitialized {
    pub settlement_policy: Pubkey,
    pub approved_market_program: Pubkey,
    pub approved_market_pool: Pubkey,
    pub approved_policy_vault_config: Pubkey,
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

#[derive(Accounts)]
#[instruction(params: InitializeSettlementPolicyParams)]
pub struct InitializeSettlementPolicy<'info> {
    #[account(
        seeds = [b"perax-state"],
        bump = state.bump,
        has_one = authority @ PeraxError::Unauthorized,
        constraint = pex_mint.key() == state.token_mint @ PeraxError::InvalidTokenMint
    )]
    pub state: Box<Account<'info, PeraxState>>,
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(
        seeds = [b"apc-config", state.key().as_ref()],
        bump = apc_config.bump,
        has_one = state @ PeraxError::ApcNotInitialized
    )]
    pub apc_config: Box<Account<'info, ApcConfig>>,
    #[account(
        seeds = [b"counterweight-config", apc_config.key().as_ref()],
        bump = counterweight_config.bump,
        constraint = counterweight_config.state == state.key() @ SettlementError::InvalidPolicy
    )]
    pub counterweight_config: Box<Account<'info, CounterweightConfig>>,
    #[account(
        constraint = approved_policy_vault_config.state == state.key() @ SettlementError::InvalidPolicy,
        constraint = approved_policy_vault_config.token_mint == pex_mint.key() @ PeraxError::InvalidTokenMint,
        constraint = approved_policy_vault_config.vault_class == VaultClass::MarketReserve @ SettlementError::InvalidPolicy,
        constraint = approved_policy_vault_config.is_active @ SettlementError::InvalidPolicy
    )]
    pub approved_policy_vault_config: Box<Account<'info, ReserveVaultConfig>>,
    #[account(
        init,
        payer = authority,
        space = 8 + SettlementPolicy::INIT_SPACE,
        seeds = [b"settlement-policy", state.key().as_ref()],
        bump
    )]
    pub settlement_policy: Box<Account<'info, SettlementPolicy>>,
    /// CHECK: PDA-only authority for settlement PEX custody.
    #[account(seeds = [b"settlement-authority", settlement_policy.key().as_ref()], bump)]
    pub settlement_authority: UncheckedAccount<'info>,
    #[account(
        init,
        payer = authority,
        associated_token::mint = pex_mint,
        associated_token::authority = settlement_authority
    )]
    pub settlement_pex_vault: Box<Account<'info, TokenAccount>>,
    #[account(
        address = counterweight_config.recovery_vault @ SettlementError::InvalidSettlementDestination,
        constraint = lock_vault.mint == pex_mint.key() @ PeraxError::InvalidTokenMint,
        constraint = lock_vault.owner == counterweight_config.recovery_authority @ SettlementError::InvalidSettlementDestination
    )]
    pub lock_vault: Box<Account<'info, TokenAccount>>,
    #[account(
        address = apc_config.quote_mint @ PeraxError::InvalidCounterweightMint,
        constraint = quote_mint.decimals == APC_QUOTE_DECIMALS @ SettlementError::InvalidPolicy
    )]
    pub quote_mint: Box<Account<'info, Mint>>,
    #[account(constraint = pex_mint.decimals == PEX_MINT_DECIMALS @ SettlementError::InvalidPolicy)]
    pub pex_mint: Box<Account<'info, Mint>>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
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

#[derive(Accounts)]
#[instruction(params: PlanSettlementParams)]
pub struct PlanSettlement<'info> {
    #[account(seeds = [b"perax-state"], bump = state.bump)]
    pub state: Box<Account<'info, PeraxState>>,
    #[account(
        seeds = [b"settlement-policy", state.key().as_ref()],
        bump = settlement_policy.bump,
        constraint = settlement_policy.state == state.key() @ SettlementError::InvalidPolicy
    )]
    pub settlement_policy: Box<Account<'info, SettlementPolicy>>,
    #[account(
        seeds = [b"product-settlement", params.product_id.as_ref()],
        bump = product_policy.bump,
        constraint = product_policy.settlement_policy == settlement_policy.key() @ SettlementError::InvalidPolicy,
        constraint = product_policy.product_id == params.product_id @ SettlementError::InvalidPolicy
    )]
    pub product_policy: Box<Account<'info, ProductSettlementPolicy>>,
    #[account(
        seeds = [b"apc-config", state.key().as_ref()],
        bump = apc_config.bump,
        constraint = apc_config.key() == settlement_policy.apc_config @ SettlementError::InvalidPolicy
    )]
    pub apc_config: Box<Account<'info, ApcConfig>>,
    #[account(
        seeds = [b"apc-state", apc_config.key().as_ref()],
        bump = apc_state.bump,
        constraint = apc_state.config == apc_config.key() @ PeraxError::ApcNotInitialized
    )]
    pub apc_state: Box<Account<'info, ApcState>>,
    #[account(
        seeds = [b"apc-observation", params.observation_id.as_ref()],
        bump = observation.bump,
        constraint = observation.observation_id == params.observation_id @ PeraxError::InvalidReference,
        constraint = observation.oracle_feed == apc_config.oracle_feed @ PeraxError::Unauthorized
    )]
    pub observation: Box<Account<'info, ApcObservation>>,
    #[account(
        init,
        payer = initiator,
        space = 8 + SettlementRecord::INIT_SPACE,
        seeds = [b"settlement", params.settlement_id.as_ref()],
        bump
    )]
    pub settlement_record: Box<Account<'info, SettlementRecord>>,
    #[account(mut)]
    pub initiator: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(params: FundDirectPexSettlementParams)]
pub struct FundDirectPexSettlement<'info> {
    #[account(seeds = [b"perax-state"], bump = state.bump)]
    pub state: Box<Account<'info, PeraxState>>,
    #[account(
        seeds = [b"settlement-policy", state.key().as_ref()],
        bump = settlement_policy.bump
    )]
    pub settlement_policy: Box<Account<'info, SettlementPolicy>>,
    #[account(
        mut,
        seeds = [b"settlement", params.settlement_id.as_ref()],
        bump = settlement_record.bump,
        constraint = settlement_record.settlement_policy == settlement_policy.key() @ SettlementError::InvalidPolicy
    )]
    pub settlement_record: Box<Account<'info, SettlementRecord>>,
    pub source_authority: Signer<'info>,
    #[account(
        mut,
        constraint = source_token_account.owner == source_authority.key() @ PeraxError::Unauthorized,
        constraint = source_token_account.mint == pex_mint.key() @ PeraxError::InvalidTokenMint
    )]
    pub source_token_account: Box<Account<'info, TokenAccount>>,
    #[account(
        mut,
        address = settlement_policy.settlement_pex_vault @ SettlementError::InvalidSettlementDestination,
        constraint = settlement_pex_vault.mint == pex_mint.key() @ PeraxError::InvalidTokenMint
    )]
    pub settlement_pex_vault: Box<Account<'info, TokenAccount>>,
    #[account(address = settlement_policy.pex_mint @ PeraxError::InvalidTokenMint)]
    pub pex_mint: Box<Account<'info, Mint>>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
#[instruction(params: ExecuteSettlementMarketPurchaseParams)]
pub struct ExecuteSettlementMarketPurchase<'info> {
    #[account(seeds = [b"perax-state"], bump = state.bump)]
    pub state: Box<Account<'info, PeraxState>>,
    #[account(
        mut,
        seeds = [b"settlement-policy", state.key().as_ref()],
        bump = settlement_policy.bump
    )]
    pub settlement_policy: Box<Account<'info, SettlementPolicy>>,
    #[account(
        mut,
        seeds = [b"settlement", params.settlement_id.as_ref()],
        bump = settlement_record.bump,
        constraint = settlement_record.settlement_policy == settlement_policy.key() @ SettlementError::InvalidPolicy
    )]
    pub settlement_record: Box<Account<'info, SettlementRecord>>,
    #[account(
        seeds = [b"apc-config", state.key().as_ref()],
        bump = apc_config.bump,
        constraint = apc_config.key() == settlement_policy.apc_config @ SettlementError::InvalidPolicy
    )]
    pub apc_config: Box<Account<'info, ApcConfig>>,
    #[account(
        seeds = [b"apc-observation", settlement_record.observation_id.as_ref()],
        bump = observation.bump,
        constraint = observation.observation_id == settlement_record.observation_id @ PeraxError::InvalidReference
    )]
    pub observation: Box<Account<'info, ApcObservation>>,
    pub quote_source_authority: Signer<'info>,
    #[account(
        mut,
        constraint = quote_source_token_account.owner == quote_source_authority.key() @ PeraxError::Unauthorized,
        constraint = quote_source_token_account.mint == quote_mint.key() @ PeraxError::InvalidCounterweightMint
    )]
    pub quote_source_token_account: Box<Account<'info, TokenAccount>>,
    #[account(
        mut,
        address = settlement_policy.settlement_pex_vault @ SettlementError::InvalidSettlementDestination,
        constraint = settlement_pex_vault.mint == pex_mint.key() @ PeraxError::InvalidTokenMint
    )]
    pub settlement_pex_vault: Box<Account<'info, TokenAccount>>,
    #[account(address = settlement_policy.quote_mint @ PeraxError::InvalidCounterweightMint)]
    pub quote_mint: Box<Account<'info, Mint>>,
    #[account(address = settlement_policy.pex_mint @ PeraxError::InvalidTokenMint)]
    pub pex_mint: Box<Account<'info, Mint>>,
    /// CHECK: Immutable approved pool, passed to the atomic adapter.
    #[account(address = settlement_policy.approved_market_pool @ SettlementError::InvalidMarketAdapter)]
    pub approved_market_pool: UncheckedAccount<'info>,
    /// CHECK: Immutable executable adapter; token deltas are checked after CPI.
    #[account(address = settlement_policy.approved_market_program @ SettlementError::InvalidMarketAdapter, executable)]
    pub market_program: UncheckedAccount<'info>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
#[instruction(params: ExecuteSettlementVaultFundingParams)]
pub struct ExecuteSettlementVaultFunding<'info> {
    #[account(seeds = [b"perax-state"], bump = state.bump)]
    pub state: Box<Account<'info, PeraxState>>,
    #[account(
        mut,
        seeds = [b"settlement-policy", state.key().as_ref()],
        bump = settlement_policy.bump
    )]
    pub settlement_policy: Box<Account<'info, SettlementPolicy>>,
    #[account(
        mut,
        seeds = [b"settlement", params.settlement_id.as_ref()],
        bump = settlement_record.bump,
        constraint = settlement_record.settlement_policy == settlement_policy.key() @ SettlementError::InvalidPolicy
    )]
    pub settlement_record: Box<Account<'info, SettlementRecord>>,
    #[account(
        mut,
        address = settlement_policy.approved_policy_vault_config @ SettlementError::InvalidPolicy,
        constraint = reserve_vault_config.state == state.key() @ SettlementError::InvalidPolicy,
        constraint = reserve_vault_config.vault_class == VaultClass::MarketReserve @ SettlementError::InvalidPolicy,
        constraint = reserve_vault_config.token_mint == pex_mint.key() @ PeraxError::InvalidTokenMint
    )]
    pub reserve_vault_config: Box<Account<'info, ReserveVaultConfig>>,
    /// CHECK: PDA authority constrained by the reserve-vault configuration.
    #[account(
        seeds = [b"reserve-authority", reserve_vault_config.allocation_id.as_ref()],
        bump = reserve_vault_config.authority_bump,
        constraint = vault_authority.key() == reserve_vault_config.vault_authority @ PeraxError::InvalidVaultAuthority
    )]
    pub vault_authority: UncheckedAccount<'info>,
    #[account(
        mut,
        address = reserve_vault_config.vault_token_account @ PeraxError::InvalidVaultTokenAccount,
        constraint = vault_token_account.owner == vault_authority.key() @ PeraxError::InvalidVaultAuthority,
        constraint = vault_token_account.mint == pex_mint.key() @ PeraxError::InvalidTokenMint
    )]
    pub vault_token_account: Box<Account<'info, TokenAccount>>,
    #[account(
        mut,
        address = settlement_policy.settlement_pex_vault @ SettlementError::InvalidSettlementDestination,
        constraint = settlement_pex_vault.mint == pex_mint.key() @ PeraxError::InvalidTokenMint
    )]
    pub settlement_pex_vault: Box<Account<'info, TokenAccount>>,
    #[account(address = settlement_policy.pex_mint @ PeraxError::InvalidTokenMint)]
    pub pex_mint: Box<Account<'info, Mint>>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
#[instruction(params: FinalizeSettlementParams)]
pub struct FinalizeSettlement<'info> {
    #[account(seeds = [b"perax-state"], bump = state.bump)]
    pub state: Box<Account<'info, PeraxState>>,
    #[account(
        seeds = [b"settlement-policy", state.key().as_ref()],
        bump = settlement_policy.bump
    )]
    pub settlement_policy: Box<Account<'info, SettlementPolicy>>,
    #[account(
        seeds = [b"product-settlement", settlement_record.product_id.as_ref()],
        bump = product_policy.bump,
        constraint = product_policy.key() == settlement_record.product_policy @ SettlementError::InvalidPolicy
    )]
    pub product_policy: Box<Account<'info, ProductSettlementPolicy>>,
    #[account(
        mut,
        seeds = [b"settlement", params.settlement_id.as_ref()],
        bump = settlement_record.bump,
        constraint = settlement_record.settlement_policy == settlement_policy.key() @ SettlementError::InvalidPolicy
    )]
    pub settlement_record: Box<Account<'info, SettlementRecord>>,
    /// CHECK: PDA-only authority for the shared settlement vault.
    #[account(
        seeds = [b"settlement-authority", settlement_policy.key().as_ref()],
        bump = settlement_policy.settlement_authority_bump,
        constraint = settlement_authority.key() == settlement_policy.settlement_authority @ SettlementError::InvalidPolicy
    )]
    pub settlement_authority: UncheckedAccount<'info>,
    #[account(
        mut,
        address = settlement_policy.settlement_pex_vault @ SettlementError::InvalidSettlementDestination,
        constraint = settlement_pex_vault.owner == settlement_authority.key() @ SettlementError::InvalidSettlementDestination,
        constraint = settlement_pex_vault.mint == pex_mint.key() @ PeraxError::InvalidTokenMint
    )]
    pub settlement_pex_vault: Box<Account<'info, TokenAccount>>,
    #[account(mut, constraint = destination_token_account.mint == pex_mint.key() @ PeraxError::InvalidTokenMint)]
    pub destination_token_account: Box<Account<'info, TokenAccount>>,
    #[account(
        mut,
        address = settlement_policy.lock_vault @ SettlementError::InvalidSettlementDestination,
        constraint = lock_vault.mint == pex_mint.key() @ PeraxError::InvalidTokenMint
    )]
    pub lock_vault: Box<Account<'info, TokenAccount>>,
    #[account(mut, address = settlement_policy.pex_mint @ PeraxError::InvalidTokenMint)]
    pub pex_mint: Box<Account<'info, Mint>>,
    pub token_program: Program<'info, Token>,
}
