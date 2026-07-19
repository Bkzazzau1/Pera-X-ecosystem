use crate::{
    ApcConfig, ApcObservation, ApcState, CounterweightConfig,
    ExecuteSettlementMarketPurchaseParams, ExecuteSettlementVaultFundingParams,
    FinalizeSettlementParams, FundDirectPexSettlementParams, InitializeSettlementPolicyParams,
    PeraxError, PeraxState, PlanSettlementParams, ProductSettlementPolicy, ReserveVaultConfig,
    SettlementError, SettlementPolicy, SettlementRecord, VaultClass, APC_QUOTE_DECIMALS,
    PEX_MINT_DECIMALS,
};
use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Mint, Token, TokenAccount},
};

#[account]
#[derive(InitSpace)]
pub struct SettlementCustody {
    pub settlement_record: Pubkey,
    pub settlement_authority: Pubkey,
    pub settlement_pex_vault: Pubkey,
    pub authority_bump: u8,
    pub bump: u8,
}

#[derive(Accounts)]
#[instruction(_params: InitializeSettlementPolicyParams)]
pub struct InitializeSettlementPolicyV2<'info> {
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
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(params: PlanSettlementParams)]
pub struct PlanSettlementV2<'info> {
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
    #[account(
        init,
        payer = initiator,
        space = 8 + SettlementCustody::INIT_SPACE,
        seeds = [b"settlement-custody", params.settlement_id.as_ref()],
        bump
    )]
    pub settlement_custody: Box<Account<'info, SettlementCustody>>,
    /// CHECK: PDA-only authority dedicated to this settlement.
    #[account(
        seeds = [b"settlement-custody-authority", settlement_record.key().as_ref()],
        bump
    )]
    pub settlement_authority: UncheckedAccount<'info>,
    #[account(
        init,
        payer = initiator,
        associated_token::mint = pex_mint,
        associated_token::authority = settlement_authority
    )]
    pub settlement_pex_vault: Box<Account<'info, TokenAccount>>,
    #[account(address = settlement_policy.pex_mint @ PeraxError::InvalidTokenMint)]
    pub pex_mint: Box<Account<'info, Mint>>,
    #[account(mut)]
    pub initiator: Signer<'info>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(params: FundDirectPexSettlementParams)]
pub struct FundDirectPexSettlementV2<'info> {
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
    #[account(
        seeds = [b"settlement-custody", params.settlement_id.as_ref()],
        bump = settlement_custody.bump,
        constraint = settlement_custody.settlement_record == settlement_record.key() @ SettlementError::InvalidPolicy
    )]
    pub settlement_custody: Box<Account<'info, SettlementCustody>>,
    pub source_authority: Signer<'info>,
    #[account(
        mut,
        constraint = source_token_account.owner == source_authority.key() @ PeraxError::Unauthorized,
        constraint = source_token_account.mint == pex_mint.key() @ PeraxError::InvalidTokenMint
    )]
    pub source_token_account: Box<Account<'info, TokenAccount>>,
    #[account(
        mut,
        address = settlement_custody.settlement_pex_vault @ SettlementError::InvalidSettlementDestination,
        constraint = settlement_pex_vault.owner == settlement_custody.settlement_authority @ SettlementError::InvalidSettlementDestination,
        constraint = settlement_pex_vault.mint == pex_mint.key() @ PeraxError::InvalidTokenMint
    )]
    pub settlement_pex_vault: Box<Account<'info, TokenAccount>>,
    #[account(address = settlement_policy.pex_mint @ PeraxError::InvalidTokenMint)]
    pub pex_mint: Box<Account<'info, Mint>>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
#[instruction(params: ExecuteSettlementMarketPurchaseParams)]
pub struct ExecuteSettlementMarketPurchaseV2<'info> {
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
        seeds = [b"settlement-custody", params.settlement_id.as_ref()],
        bump = settlement_custody.bump,
        constraint = settlement_custody.settlement_record == settlement_record.key() @ SettlementError::InvalidPolicy
    )]
    pub settlement_custody: Box<Account<'info, SettlementCustody>>,
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
        address = settlement_custody.settlement_pex_vault @ SettlementError::InvalidSettlementDestination,
        constraint = settlement_pex_vault.owner == settlement_custody.settlement_authority @ SettlementError::InvalidSettlementDestination,
        constraint = settlement_pex_vault.mint == pex_mint.key() @ PeraxError::InvalidTokenMint
    )]
    pub settlement_pex_vault: Box<Account<'info, TokenAccount>>,
    #[account(address = settlement_policy.quote_mint @ PeraxError::InvalidCounterweightMint)]
    pub quote_mint: Box<Account<'info, Mint>>,
    #[account(address = settlement_policy.pex_mint @ PeraxError::InvalidTokenMint)]
    pub pex_mint: Box<Account<'info, Mint>>,
    /// CHECK: Immutable approved pool, passed writable to the atomic adapter.
    #[account(mut, address = settlement_policy.approved_market_pool @ SettlementError::InvalidMarketAdapter)]
    pub approved_market_pool: UncheckedAccount<'info>,
    /// CHECK: Immutable executable adapter; token deltas are checked after CPI.
    #[account(address = settlement_policy.approved_market_program @ SettlementError::InvalidMarketAdapter, executable)]
    pub market_program: UncheckedAccount<'info>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
#[instruction(params: ExecuteSettlementVaultFundingParams)]
pub struct ExecuteSettlementVaultFundingV2<'info> {
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
        seeds = [b"settlement-custody", params.settlement_id.as_ref()],
        bump = settlement_custody.bump,
        constraint = settlement_custody.settlement_record == settlement_record.key() @ SettlementError::InvalidPolicy
    )]
    pub settlement_custody: Box<Account<'info, SettlementCustody>>,
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
        address = settlement_custody.settlement_pex_vault @ SettlementError::InvalidSettlementDestination,
        constraint = settlement_pex_vault.owner == settlement_custody.settlement_authority @ SettlementError::InvalidSettlementDestination,
        constraint = settlement_pex_vault.mint == pex_mint.key() @ PeraxError::InvalidTokenMint
    )]
    pub settlement_pex_vault: Box<Account<'info, TokenAccount>>,
    #[account(address = settlement_policy.pex_mint @ PeraxError::InvalidTokenMint)]
    pub pex_mint: Box<Account<'info, Mint>>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
#[instruction(params: FinalizeSettlementParams)]
pub struct FinalizeSettlementV2<'info> {
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
    #[account(
        seeds = [b"settlement-custody", params.settlement_id.as_ref()],
        bump = settlement_custody.bump,
        constraint = settlement_custody.settlement_record == settlement_record.key() @ SettlementError::InvalidPolicy
    )]
    pub settlement_custody: Box<Account<'info, SettlementCustody>>,
    /// CHECK: PDA-only authority dedicated to this settlement.
    #[account(
        seeds = [b"settlement-custody-authority", settlement_record.key().as_ref()],
        bump = settlement_custody.authority_bump,
        constraint = settlement_authority.key() == settlement_custody.settlement_authority @ SettlementError::InvalidPolicy
    )]
    pub settlement_authority: UncheckedAccount<'info>,
    #[account(
        mut,
        address = settlement_custody.settlement_pex_vault @ SettlementError::InvalidSettlementDestination,
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
