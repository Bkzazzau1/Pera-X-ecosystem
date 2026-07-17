use crate::{
    ActivateApcBandParams, ApcBandRecord, ApcConfig, ApcObservation, ApcRecoveryRecord,
    ApcReleaseRecord, ApcState, BurnExecutionRecord, ConditionalBuybackBurnParams,
    CounterweightConfig, CounterweightDepositRecord, DeferredBurnRecord,
    DepositCounterweightParams, ExecuteApcReleaseParams, ExecuteCounterweightPurchaseParams,
    InitializeApcParams, InitializeRecoveryPoolParams, InitializeReserveVaultParams,
    MarketConditionBurnParams, MarketConditionalReleaseParams, PaymentRecord, PeraxError,
    PeraxState, RecordDeferredBurnParams, RecoveryPoolConfig, ReleaseRecord, ReserveReleaseRecord,
    ReserveVaultConfig, SubmitApcObservationParams, VaultMarketConditionalReleaseParams,
};
use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Mint, Token, TokenAccount, Transfer, TransferChecked},
};

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(init, payer = authority, space = 8 + PeraxState::INIT_SPACE, seeds = [b"perax-state"], bump)]
    pub state: Account<'info, PeraxState>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct UpdateConfig<'info> {
    #[account(mut, seeds = [b"perax-state"], bump = state.bump, has_one = authority @ PeraxError::Unauthorized)]
    pub state: Account<'info, PeraxState>,
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct SafetyAdminAction<'info> {
    #[account(mut, seeds = [b"perax-state"], bump = state.bump, has_one = safety_admin @ PeraxError::Unauthorized)]
    pub state: Account<'info, PeraxState>,
    pub safety_admin: Signer<'info>,
}

#[derive(Accounts)]
#[instruction(params: InitializeReserveVaultParams)]
pub struct InitializeReserveVault<'info> {
    #[account(
        seeds = [b"perax-state"],
        bump = state.bump,
        has_one = authority @ PeraxError::Unauthorized,
        constraint = token_mint.key() == state.token_mint @ PeraxError::InvalidTokenMint
    )]
    pub state: Account<'info, PeraxState>,
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(
        init,
        payer = authority,
        space = 8 + ReserveVaultConfig::SPACE,
        seeds = [b"reserve-config", params.allocation_id.as_ref()],
        bump
    )]
    pub reserve_vault_config: Account<'info, ReserveVaultConfig>,
    /// CHECK: PDA authority constrained by seeds; it has no private key.
    #[account(seeds = [b"reserve-authority", params.allocation_id.as_ref()], bump)]
    pub vault_authority: UncheckedAccount<'info>,
    #[account(
        init,
        payer = authority,
        associated_token::mint = token_mint,
        associated_token::authority = vault_authority
    )]
    pub vault_token_account: Account<'info, TokenAccount>,
    pub token_mint: Account<'info, Mint>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(allocation_id: [u8; 32], amount: u64)]
pub struct DepositIntoReserveVault<'info> {
    #[account(
        seeds = [b"perax-state"],
        bump = state.bump,
        constraint = token_mint.key() == state.token_mint @ PeraxError::InvalidTokenMint
    )]
    pub state: Account<'info, PeraxState>,
    #[account(
        mut,
        seeds = [b"reserve-config", allocation_id.as_ref()],
        bump = reserve_vault_config.config_bump,
        has_one = state @ PeraxError::InvalidVaultConfiguration,
        constraint = reserve_vault_config.allocation_id == allocation_id @ PeraxError::InvalidVaultConfiguration,
        constraint = reserve_vault_config.token_mint == token_mint.key() @ PeraxError::InvalidTokenMint
    )]
    pub reserve_vault_config: Account<'info, ReserveVaultConfig>,
    /// CHECK: PDA authority constrained by seeds and stored configuration.
    #[account(
        seeds = [b"reserve-authority", allocation_id.as_ref()],
        bump = reserve_vault_config.authority_bump,
        constraint = vault_authority.key() == reserve_vault_config.vault_authority @ PeraxError::InvalidVaultAuthority
    )]
    pub vault_authority: UncheckedAccount<'info>,
    #[account(
        constraint = source_owner.key() == reserve_vault_config.authorized_source_owner
            @ PeraxError::InvalidAuthorizedSourceOwner
    )]
    pub source_owner: Signer<'info>,
    #[account(
        mut,
        address = reserve_vault_config.authorized_source_token_account
            @ PeraxError::InvalidAuthorizedSourceTokenAccount,
        constraint = source_token_account.owner == source_owner.key()
            @ PeraxError::InvalidAuthorizedSourceOwner,
        constraint = source_token_account.mint == token_mint.key()
            @ PeraxError::InvalidTokenMint
    )]
    pub source_token_account: Account<'info, TokenAccount>,
    #[account(
        mut,
        address = reserve_vault_config.vault_token_account @ PeraxError::InvalidVaultTokenAccount,
        constraint = vault_token_account.owner == vault_authority.key() @ PeraxError::InvalidVaultAuthority,
        constraint = vault_token_account.mint == token_mint.key() @ PeraxError::InvalidTokenMint
    )]
    pub vault_token_account: Account<'info, TokenAccount>,
    pub token_mint: Account<'info, Mint>,
    pub token_program: Program<'info, Token>,
}

impl<'info> DepositIntoReserveVault<'info> {
    pub fn deposit_transfer_ctx(&self) -> CpiContext<'_, '_, '_, 'info, TransferChecked<'info>> {
        let accounts = TransferChecked {
            mint: self.token_mint.to_account_info(),
            from: self.source_token_account.to_account_info(),
            to: self.vault_token_account.to_account_info(),
            authority: self.source_owner.to_account_info(),
        };
        CpiContext::new(self.token_program.to_account_info(), accounts)
    }
}

#[derive(Accounts)]
#[instruction(allocation_id: [u8; 32], is_paused: bool)]
pub struct SetReserveVaultPause<'info> {
    #[account(seeds = [b"perax-state"], bump = state.bump)]
    pub state: Account<'info, PeraxState>,
    #[account(
        mut,
        seeds = [b"reserve-config", allocation_id.as_ref()],
        bump = reserve_vault_config.config_bump,
        has_one = state @ PeraxError::InvalidVaultConfiguration,
        constraint = reserve_vault_config.allocation_id == allocation_id @ PeraxError::InvalidVaultConfiguration
    )]
    pub reserve_vault_config: Account<'info, ReserveVaultConfig>,
    pub actor: Signer<'info>,
}

#[derive(Accounts)]
#[instruction(allocation_id: [u8; 32])]
pub struct ReconcileReserveVault<'info> {
    #[account(
        seeds = [b"perax-state"],
        bump = state.bump,
        has_one = authority @ PeraxError::Unauthorized
    )]
    pub state: Account<'info, PeraxState>,
    pub authority: Signer<'info>,
    #[account(
        mut,
        seeds = [b"reserve-config", allocation_id.as_ref()],
        bump = reserve_vault_config.config_bump,
        has_one = state @ PeraxError::InvalidVaultConfiguration,
        constraint = reserve_vault_config.allocation_id == allocation_id @ PeraxError::InvalidVaultConfiguration
    )]
    pub reserve_vault_config: Account<'info, ReserveVaultConfig>,
    #[account(
        address = reserve_vault_config.vault_token_account @ PeraxError::InvalidVaultTokenAccount,
        constraint = vault_token_account.owner == reserve_vault_config.vault_authority @ PeraxError::InvalidVaultAuthority,
        constraint = vault_token_account.mint == reserve_vault_config.token_mint @ PeraxError::InvalidTokenMint
    )]
    pub vault_token_account: Account<'info, TokenAccount>,
}

#[derive(Accounts)]
#[instruction(params: VaultMarketConditionalReleaseParams)]
pub struct ExecuteMarketConditionalRelease<'info> {
    #[account(
        mut,
        seeds = [b"perax-state"],
        bump = state.bump,
        has_one = oracle_feed @ PeraxError::Unauthorized,
        constraint = token_mint.key() == state.token_mint @ PeraxError::InvalidTokenMint
    )]
    pub state: Account<'info, PeraxState>,
    #[account(
        mut,
        seeds = [b"reserve-config", params.allocation_id.as_ref()],
        bump = reserve_vault_config.config_bump,
        has_one = state @ PeraxError::InvalidVaultConfiguration,
        constraint = reserve_vault_config.allocation_id == params.allocation_id @ PeraxError::InvalidVaultConfiguration,
        constraint = reserve_vault_config.token_mint == token_mint.key() @ PeraxError::InvalidTokenMint
    )]
    pub reserve_vault_config: Account<'info, ReserveVaultConfig>,
    /// CHECK: PDA authority constrained by seeds and stored configuration.
    #[account(
        seeds = [b"reserve-authority", params.allocation_id.as_ref()],
        bump = reserve_vault_config.authority_bump,
        constraint = vault_authority.key() == reserve_vault_config.vault_authority @ PeraxError::InvalidVaultAuthority
    )]
    pub vault_authority: UncheckedAccount<'info>,
    #[account(
        mut,
        address = reserve_vault_config.vault_token_account @ PeraxError::InvalidVaultTokenAccount,
        constraint = vault_token_account.owner == vault_authority.key() @ PeraxError::InvalidVaultAuthority,
        constraint = vault_token_account.mint == token_mint.key() @ PeraxError::InvalidTokenMint
    )]
    pub vault_token_account: Account<'info, TokenAccount>,
    #[account(
        mut,
        address = reserve_vault_config.approved_destination_token_account
            @ PeraxError::InvalidApprovedDestination,
        constraint = destination_token_account.key() == params.destination_token_account
            @ PeraxError::InvalidReleaseDestination,
        constraint = destination_token_account.owner == reserve_vault_config.approved_destination_owner
            @ PeraxError::InvalidApprovedDestination,
        constraint = destination_token_account.mint == token_mint.key()
            @ PeraxError::InvalidTokenMint
    )]
    pub destination_token_account: Account<'info, TokenAccount>,
    #[account(
        init,
        payer = oracle_feed,
        space = 8 + ReserveReleaseRecord::SPACE,
        seeds = [b"vault-release", params.release_id.as_ref()],
        bump
    )]
    pub release_record: Account<'info, ReserveReleaseRecord>,
    #[account(mut)]
    pub oracle_feed: Signer<'info>,
    pub token_mint: Account<'info, Mint>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(params: MarketConditionalReleaseParams)]
pub struct RecordMarketConditionalRelease<'info> {
    #[account(mut, seeds = [b"perax-state"], bump = state.bump, has_one = oracle_feed @ PeraxError::Unauthorized)]
    pub state: Account<'info, PeraxState>,
    #[account(
        init,
        payer = oracle_feed,
        space = 8 + ReleaseRecord::SPACE,
        seeds = [b"release", params.release_id.as_ref()],
        bump
    )]
    pub release_record: Account<'info, ReleaseRecord>,
    #[account(mut)]
    pub oracle_feed: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct AcceptAuthority<'info> {
    #[account(mut, seeds = [b"perax-state"], bump = state.bump)]
    pub state: Account<'info, PeraxState>,
    pub pending_authority: Signer<'info>,
}

#[derive(Accounts)]
#[instruction(amount: u64, reference: [u8; 32])]
pub struct PayToTradingCompany<'info> {
    #[account(seeds = [b"perax-state"], bump = state.bump, constraint = token_mint.key() == state.token_mint @ PeraxError::InvalidTokenMint, constraint = trading_company_revenue_token_account.key() == state.trading_company_revenue_token_account @ PeraxError::InvalidTradingCompanyRevenueAccount)]
    pub state: Account<'info, PeraxState>,
    #[account(init, payer = payer, space = 8 + PaymentRecord::SPACE, seeds = [b"payment", reference.as_ref()], bump)]
    pub payment_record: Account<'info, PaymentRecord>,
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(mut, constraint = payer_token_account.owner == payer.key() @ PeraxError::Unauthorized, constraint = payer_token_account.mint == token_mint.key() @ PeraxError::InvalidTokenMint)]
    pub payer_token_account: Account<'info, TokenAccount>,
    #[account(mut, constraint = trading_company_revenue_token_account.mint == token_mint.key() @ PeraxError::InvalidTokenMint)]
    pub trading_company_revenue_token_account: Account<'info, TokenAccount>,
    pub token_mint: Account<'info, Mint>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

impl<'info> PayToTradingCompany<'info> {
    pub fn payment_transfer_ctx(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        let accounts = Transfer {
            from: self.payer_token_account.to_account_info(),
            to: self.trading_company_revenue_token_account.to_account_info(),
            authority: self.payer.to_account_info(),
        };
        CpiContext::new(self.token_program.to_account_info(), accounts)
    }
}

#[derive(Accounts)]
pub struct RecordExternalUtilityPayment<'info> {
    #[account(seeds = [b"perax-state"], bump = state.bump, has_one = authority @ PeraxError::Unauthorized)]
    pub state: Account<'info, PeraxState>,
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct BurnFromTradingCompany<'info> {
    #[account(seeds = [b"perax-state"], bump = state.bump, has_one = authority @ PeraxError::Unauthorized, constraint = token_mint.key() == state.token_mint @ PeraxError::InvalidTokenMint, constraint = trading_company_revenue_token_account.key() == state.trading_company_revenue_token_account @ PeraxError::InvalidTradingCompanyRevenueAccount)]
    pub state: Account<'info, PeraxState>,
    pub authority: Signer<'info>,
    pub trading_company_authority: Signer<'info>,
    #[account(mut, constraint = trading_company_revenue_token_account.owner == trading_company_authority.key() @ PeraxError::Unauthorized, constraint = trading_company_revenue_token_account.mint == token_mint.key() @ PeraxError::InvalidTokenMint)]
    pub trading_company_revenue_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub token_mint: Account<'info, Mint>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
#[instruction(params: MarketConditionBurnParams)]
pub struct ExecuteMarketConditionBurn<'info> {
    #[account(mut, seeds = [b"perax-state"], bump = state.bump, has_one = authority @ PeraxError::Unauthorized, constraint = token_mint.key() == state.token_mint @ PeraxError::InvalidTokenMint, constraint = trading_company_revenue_token_account.key() == state.trading_company_revenue_token_account @ PeraxError::InvalidTradingCompanyRevenueAccount)]
    pub state: Account<'info, PeraxState>,
    #[account(seeds = [b"apc-config", state.key().as_ref()], bump = apc_config.bump, has_one = state @ PeraxError::ApcNotInitialized)]
    pub apc_config: Account<'info, ApcConfig>,
    #[account(seeds = [b"apc-state", apc_config.key().as_ref()], bump = apc_state.bump, constraint = apc_state.config == apc_config.key() @ PeraxError::ApcNotInitialized)]
    pub apc_state: Account<'info, ApcState>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub trading_company_authority: Signer<'info>,
    #[account(
        init,
        payer = authority,
        space = 8 + BurnExecutionRecord::SPACE,
        seeds = [b"burn", params.decision_id.as_ref()],
        bump
    )]
    pub burn_record: Account<'info, BurnExecutionRecord>,
    #[account(mut, constraint = trading_company_revenue_token_account.owner == trading_company_authority.key() @ PeraxError::Unauthorized, constraint = trading_company_revenue_token_account.mint == token_mint.key() @ PeraxError::InvalidTokenMint)]
    pub trading_company_revenue_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub token_mint: Account<'info, Mint>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(params: ConditionalBuybackBurnParams)]
pub struct ExecuteConditionalBuybackBurn<'info> {
    #[account(
        mut,
        seeds = [b"perax-state"],
        bump = state.bump,
        has_one = authority @ PeraxError::Unauthorized,
        constraint = token_mint.key() == state.token_mint @ PeraxError::InvalidTokenMint
    )]
    pub state: Account<'info, PeraxState>,
    #[account(seeds = [b"apc-config", state.key().as_ref()], bump = apc_config.bump, has_one = state @ PeraxError::ApcNotInitialized)]
    pub apc_config: Account<'info, ApcConfig>,
    #[account(seeds = [b"apc-state", apc_config.key().as_ref()], bump = apc_state.bump, constraint = apc_state.config == apc_config.key() @ PeraxError::ApcNotInitialized)]
    pub apc_state: Account<'info, ApcState>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub source_authority: Signer<'info>,
    #[account(
        init,
        payer = authority,
        space = 8 + BurnExecutionRecord::SPACE,
        seeds = [b"burn", params.decision_id.as_ref()],
        bump
    )]
    pub burn_record: Account<'info, BurnExecutionRecord>,
    #[account(
        mut,
        constraint = source_token_account.owner == source_authority.key() @ PeraxError::Unauthorized,
        constraint = source_token_account.mint == token_mint.key() @ PeraxError::InvalidTokenMint
    )]
    pub source_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub token_mint: Account<'info, Mint>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(params: InitializeRecoveryPoolParams)]
pub struct InitializeRecoveryPool<'info> {
    #[account(
        seeds = [b"perax-state"],
        bump = state.bump,
        has_one = authority @ PeraxError::Unauthorized,
        constraint = pex_mint.key() == state.token_mint @ PeraxError::InvalidTokenMint
    )]
    pub state: Account<'info, PeraxState>,
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(
        init,
        payer = authority,
        space = 8 + RecoveryPoolConfig::INIT_SPACE,
        seeds = [b"recovery-pool", params.pool_id.as_ref()],
        bump
    )]
    pub recovery_pool: Account<'info, RecoveryPoolConfig>,
    /// CHECK: PDA authority for the recovery pool token vaults.
    #[account(seeds = [b"recovery-pool-authority", recovery_pool.key().as_ref()], bump)]
    pub pool_authority: UncheckedAccount<'info>,
    #[account(
        init,
        payer = authority,
        associated_token::mint = quote_mint,
        associated_token::authority = pool_authority
    )]
    pub pool_quote_vault: Account<'info, TokenAccount>,
    #[account(
        init,
        payer = authority,
        associated_token::mint = pex_mint,
        associated_token::authority = pool_authority
    )]
    pub pool_pex_vault: Account<'info, TokenAccount>,
    pub quote_mint: Account<'info, Mint>,
    pub pex_mint: Account<'info, Mint>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(params: InitializeApcParams)]
pub struct InitializeApc<'info> {
    #[account(
        seeds = [b"perax-state"],
        bump = state.bump,
        has_one = authority @ PeraxError::Unauthorized,
        constraint = token_mint.key() == state.token_mint @ PeraxError::InvalidTokenMint
    )]
    pub state: Account<'info, PeraxState>,
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(
        init,
        payer = authority,
        space = 8 + ApcConfig::INIT_SPACE,
        seeds = [b"apc-config", state.key().as_ref()],
        bump
    )]
    pub apc_config: Account<'info, ApcConfig>,
    #[account(
        init,
        payer = authority,
        space = 8 + ApcState::INIT_SPACE,
        seeds = [b"apc-state", apc_config.key().as_ref()],
        bump
    )]
    pub apc_state: Account<'info, ApcState>,
    #[account(
        init,
        payer = authority,
        space = 8 + CounterweightConfig::INIT_SPACE,
        seeds = [b"counterweight-config", apc_config.key().as_ref()],
        bump
    )]
    pub counterweight_config: Account<'info, CounterweightConfig>,
    /// CHECK: PDA-only authority for the quote counterweight vault.
    #[account(seeds = [b"counterweight-authority", apc_config.key().as_ref()], bump)]
    pub counterweight_authority: UncheckedAccount<'info>,
    #[account(
        init,
        payer = authority,
        associated_token::mint = quote_mint,
        associated_token::authority = counterweight_authority
    )]
    pub counterweight_vault: Account<'info, TokenAccount>,
    /// CHECK: PDA-only authority for deferred burn custody.
    #[account(seeds = [b"deferred-burn-authority", apc_config.key().as_ref()], bump)]
    pub deferred_burn_authority: UncheckedAccount<'info>,
    #[account(
        init,
        payer = authority,
        associated_token::mint = token_mint,
        associated_token::authority = deferred_burn_authority
    )]
    pub deferred_burn_vault: Account<'info, TokenAccount>,
    /// CHECK: PDA-only authority for locked recovery inventory.
    #[account(seeds = [b"recovery-authority", apc_config.key().as_ref()], bump)]
    pub recovery_authority: UncheckedAccount<'info>,
    #[account(
        init,
        payer = authority,
        associated_token::mint = token_mint,
        associated_token::authority = recovery_authority
    )]
    pub recovery_vault: Account<'info, TokenAccount>,
    #[account(address = params.quote_mint @ PeraxError::InvalidCounterweightMint)]
    pub quote_mint: Account<'info, Mint>,
    pub token_mint: Account<'info, Mint>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(params: SubmitApcObservationParams)]
pub struct SubmitApcObservation<'info> {
    #[account(seeds = [b"perax-state"], bump = state.bump)]
    pub state: Account<'info, PeraxState>,
    #[account(
        seeds = [b"apc-config", state.key().as_ref()],
        bump = apc_config.bump,
        has_one = state @ PeraxError::ApcNotInitialized,
        has_one = oracle_feed @ PeraxError::Unauthorized
    )]
    pub apc_config: Account<'info, ApcConfig>,
    #[account(
        mut,
        seeds = [b"apc-state", apc_config.key().as_ref()],
        bump = apc_state.bump,
        constraint = apc_state.config == apc_config.key() @ PeraxError::ApcNotInitialized
    )]
    pub apc_state: Account<'info, ApcState>,
    #[account(
        init,
        payer = oracle_feed,
        space = 8 + ApcObservation::INIT_SPACE,
        seeds = [b"apc-observation", params.observation_id.as_ref()],
        bump
    )]
    pub observation: Account<'info, ApcObservation>,
    #[account(mut)]
    pub oracle_feed: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(params: ActivateApcBandParams)]
pub struct ActivateNextApcBand<'info> {
    #[account(seeds = [b"perax-state"], bump = state.bump)]
    pub state: Account<'info, PeraxState>,
    #[account(
        seeds = [b"apc-config", state.key().as_ref()],
        bump = apc_config.bump,
        has_one = oracle_feed @ PeraxError::Unauthorized
    )]
    pub apc_config: Account<'info, ApcConfig>,
    #[account(
        mut,
        seeds = [b"apc-state", apc_config.key().as_ref()],
        bump = apc_state.bump,
        constraint = apc_state.config == apc_config.key() @ PeraxError::ApcNotInitialized
    )]
    pub apc_state: Account<'info, ApcState>,
    #[account(
        seeds = [b"apc-observation", observation.observation_id.as_ref()],
        bump = observation.bump,
        constraint = observation.oracle_feed == oracle_feed.key() @ PeraxError::Unauthorized
    )]
    pub observation: Account<'info, ApcObservation>,
    #[account(
        init,
        payer = oracle_feed,
        space = 8 + ApcBandRecord::INIT_SPACE,
        seeds = [b"apc-band", apc_state.key().as_ref(), params.band_index.to_le_bytes().as_ref()],
        bump
    )]
    pub band_record: Account<'info, ApcBandRecord>,
    #[account(mut)]
    pub oracle_feed: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(params: ExecuteApcReleaseParams)]
pub struct ExecuteApcRelease<'info> {
    #[account(
        mut,
        seeds = [b"perax-state"],
        bump = state.bump,
        constraint = token_mint.key() == state.token_mint @ PeraxError::InvalidTokenMint
    )]
    pub state: Account<'info, PeraxState>,
    #[account(
        seeds = [b"apc-config", state.key().as_ref()],
        bump = apc_config.bump,
        has_one = oracle_feed @ PeraxError::Unauthorized
    )]
    pub apc_config: Account<'info, ApcConfig>,
    #[account(
        mut,
        seeds = [b"apc-state", apc_config.key().as_ref()],
        bump = apc_state.bump,
        constraint = apc_state.config == apc_config.key() @ PeraxError::ApcNotInitialized
    )]
    pub apc_state: Account<'info, ApcState>,
    #[account(
        mut,
        seeds = [b"apc-observation", params.observation_id.as_ref()],
        bump = observation.bump,
        constraint = observation.observation_id == params.observation_id @ PeraxError::InvalidReference,
        constraint = observation.oracle_feed == oracle_feed.key() @ PeraxError::Unauthorized
    )]
    pub observation: Account<'info, ApcObservation>,
    #[account(
        mut,
        seeds = [b"apc-band", apc_state.key().as_ref(), params.band_index.to_le_bytes().as_ref()],
        bump = band_record.bump,
        constraint = band_record.band_index == params.band_index @ PeraxError::InvalidBandIndex,
        constraint = band_record.apc_state == apc_state.key() @ PeraxError::InvalidBandIndex
    )]
    pub band_record: Account<'info, ApcBandRecord>,
    #[account(
        mut,
        seeds = [b"reserve-config", params.allocation_id.as_ref()],
        bump = reserve_vault_config.config_bump,
        has_one = state @ PeraxError::InvalidVaultConfiguration,
        constraint = reserve_vault_config.allocation_id == params.allocation_id @ PeraxError::InvalidVaultConfiguration,
        constraint = reserve_vault_config.token_mint == token_mint.key() @ PeraxError::InvalidTokenMint
    )]
    pub reserve_vault_config: Account<'info, ReserveVaultConfig>,
    /// CHECK: PDA authority constrained by the Correction 1 reserve configuration.
    #[account(
        seeds = [b"reserve-authority", params.allocation_id.as_ref()],
        bump = reserve_vault_config.authority_bump,
        constraint = vault_authority.key() == reserve_vault_config.vault_authority @ PeraxError::InvalidVaultAuthority
    )]
    pub vault_authority: UncheckedAccount<'info>,
    #[account(
        mut,
        address = reserve_vault_config.vault_token_account @ PeraxError::InvalidVaultTokenAccount,
        constraint = vault_token_account.owner == vault_authority.key() @ PeraxError::InvalidVaultAuthority,
        constraint = vault_token_account.mint == token_mint.key() @ PeraxError::InvalidTokenMint
    )]
    pub vault_token_account: Account<'info, TokenAccount>,
    #[account(
        mut,
        address = reserve_vault_config.approved_destination_token_account @ PeraxError::InvalidApprovedDestination,
        constraint = destination_token_account.key() == params.destination_token_account @ PeraxError::InvalidReleaseDestination,
        constraint = destination_token_account.owner == reserve_vault_config.approved_destination_owner @ PeraxError::InvalidApprovedDestination,
        constraint = destination_token_account.mint == token_mint.key() @ PeraxError::InvalidTokenMint
    )]
    pub destination_token_account: Account<'info, TokenAccount>,
    #[account(
        seeds = [b"counterweight-config", apc_config.key().as_ref()],
        bump = counterweight_config.bump,
        constraint = counterweight_config.apc_config == apc_config.key() @ PeraxError::InvalidCounterweightVault
    )]
    pub counterweight_config: Account<'info, CounterweightConfig>,
    #[account(
        address = counterweight_config.counterweight_vault @ PeraxError::InvalidCounterweightVault,
        constraint = counterweight_vault.owner == counterweight_config.counterweight_authority @ PeraxError::InvalidCounterweightVault,
        constraint = counterweight_vault.mint == apc_config.quote_mint @ PeraxError::InvalidCounterweightMint
    )]
    pub counterweight_vault: Account<'info, TokenAccount>,
    #[account(
        init,
        payer = oracle_feed,
        space = 8 + ApcReleaseRecord::INIT_SPACE,
        seeds = [b"apc-release", params.release_id.as_ref()],
        bump
    )]
    pub release_record: Account<'info, ApcReleaseRecord>,
    #[account(mut)]
    pub oracle_feed: Signer<'info>,
    pub token_mint: Account<'info, Mint>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(params: DepositCounterweightParams)]
pub struct DepositCounterweightProceeds<'info> {
    #[account(seeds = [b"perax-state"], bump = state.bump)]
    pub state: Account<'info, PeraxState>,
    #[account(seeds = [b"apc-config", state.key().as_ref()], bump = apc_config.bump)]
    pub apc_config: Account<'info, ApcConfig>,
    #[account(
        mut,
        seeds = [b"apc-state", apc_config.key().as_ref()],
        bump = apc_state.bump
    )]
    pub apc_state: Account<'info, ApcState>,
    #[account(
        seeds = [b"counterweight-config", apc_config.key().as_ref()],
        bump = counterweight_config.bump
    )]
    pub counterweight_config: Account<'info, CounterweightConfig>,
    #[account(
        mut,
        constraint = source_owner.key() == counterweight_config.approved_proceeds_owner @ PeraxError::Unauthorized
    )]
    pub source_owner: Signer<'info>,
    #[account(
        mut,
        address = counterweight_config.approved_proceeds_token_account @ PeraxError::InvalidCounterweightVault,
        constraint = source_token_account.owner == source_owner.key() @ PeraxError::Unauthorized,
        constraint = source_token_account.mint == quote_mint.key() @ PeraxError::InvalidCounterweightMint
    )]
    pub source_token_account: Account<'info, TokenAccount>,
    #[account(
        mut,
        address = counterweight_config.counterweight_vault @ PeraxError::InvalidCounterweightVault,
        constraint = counterweight_vault.mint == quote_mint.key() @ PeraxError::InvalidCounterweightMint
    )]
    pub counterweight_vault: Account<'info, TokenAccount>,
    #[account(address = counterweight_config.quote_mint @ PeraxError::InvalidCounterweightMint)]
    pub quote_mint: Account<'info, Mint>,
    #[account(
        init,
        payer = source_owner,
        space = 8 + CounterweightDepositRecord::INIT_SPACE,
        seeds = [b"counterweight-deposit", params.deposit_id.as_ref()],
        bump
    )]
    pub deposit_record: Account<'info, CounterweightDepositRecord>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(params: RecordDeferredBurnParams)]
pub struct RecordDeferredBurn<'info> {
    #[account(seeds = [b"perax-state"], bump = state.bump, constraint = token_mint.key() == state.token_mint @ PeraxError::InvalidTokenMint)]
    pub state: Account<'info, PeraxState>,
    #[account(seeds = [b"apc-config", state.key().as_ref()], bump = apc_config.bump)]
    pub apc_config: Account<'info, ApcConfig>,
    #[account(mut, seeds = [b"apc-state", apc_config.key().as_ref()], bump = apc_state.bump)]
    pub apc_state: Account<'info, ApcState>,
    #[account(seeds = [b"counterweight-config", apc_config.key().as_ref()], bump = counterweight_config.bump)]
    pub counterweight_config: Account<'info, CounterweightConfig>,
    #[account(mut)]
    pub source_authority: Signer<'info>,
    #[account(mut, constraint = source_token_account.owner == source_authority.key() @ PeraxError::Unauthorized, constraint = source_token_account.mint == token_mint.key() @ PeraxError::InvalidTokenMint)]
    pub source_token_account: Account<'info, TokenAccount>,
    #[account(mut, address = counterweight_config.deferred_burn_vault @ PeraxError::InvalidCounterweightVault, constraint = deferred_burn_vault.mint == token_mint.key() @ PeraxError::InvalidTokenMint)]
    pub deferred_burn_vault: Account<'info, TokenAccount>,
    pub token_mint: Account<'info, Mint>,
    #[account(
        init,
        payer = source_authority,
        space = 8 + DeferredBurnRecord::INIT_SPACE,
        seeds = [b"deferred-burn", params.decision_id.as_ref()],
        bump
    )]
    pub deferred_burn_record: Account<'info, DeferredBurnRecord>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ExecuteDeferredBurn<'info> {
    #[account(seeds = [b"perax-state"], bump = state.bump, constraint = token_mint.key() == state.token_mint @ PeraxError::InvalidTokenMint)]
    pub state: Account<'info, PeraxState>,
    #[account(seeds = [b"apc-config", state.key().as_ref()], bump = apc_config.bump, has_one = oracle_feed @ PeraxError::Unauthorized)]
    pub apc_config: Account<'info, ApcConfig>,
    #[account(mut, seeds = [b"apc-state", apc_config.key().as_ref()], bump = apc_state.bump)]
    pub apc_state: Account<'info, ApcState>,
    #[account(seeds = [b"counterweight-config", apc_config.key().as_ref()], bump = counterweight_config.bump)]
    pub counterweight_config: Account<'info, CounterweightConfig>,
    /// CHECK: PDA authority constrained by configured seeds.
    #[account(seeds = [b"deferred-burn-authority", apc_config.key().as_ref()], bump = counterweight_config.deferred_burn_authority_bump)]
    pub deferred_burn_authority: UncheckedAccount<'info>,
    #[account(mut, address = counterweight_config.deferred_burn_vault @ PeraxError::InvalidCounterweightVault, constraint = deferred_burn_vault.owner == deferred_burn_authority.key() @ PeraxError::InvalidCounterweightVault, constraint = deferred_burn_vault.mint == token_mint.key() @ PeraxError::InvalidTokenMint)]
    pub deferred_burn_vault: Account<'info, TokenAccount>,
    #[account(
        mut,
        seeds = [b"deferred-burn", deferred_burn_record.decision_id.as_ref()],
        bump = deferred_burn_record.bump,
        constraint = deferred_burn_record.apc_state == apc_state.key() @ PeraxError::DeferredBurnNotExecutable
    )]
    pub deferred_burn_record: Account<'info, DeferredBurnRecord>,
    #[account(mut)]
    pub token_mint: Account<'info, Mint>,
    pub oracle_feed: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct ConfirmApcAbsorption<'info> {
    #[account(seeds = [b"perax-state"], bump = state.bump)]
    pub state: Account<'info, PeraxState>,
    #[account(
        seeds = [b"apc-config", state.key().as_ref()],
        bump = apc_config.bump,
        has_one = oracle_feed @ PeraxError::Unauthorized
    )]
    pub apc_config: Account<'info, ApcConfig>,
    #[account(
        mut,
        seeds = [b"apc-state", apc_config.key().as_ref()],
        bump = apc_state.bump,
        constraint = apc_state.config == apc_config.key() @ PeraxError::ApcNotInitialized
    )]
    pub apc_state: Account<'info, ApcState>,
    #[account(
        mut,
        seeds = [b"apc-observation", observation.observation_id.as_ref()],
        bump = observation.bump,
        constraint = observation.oracle_feed == oracle_feed.key() @ PeraxError::Unauthorized
    )]
    pub observation: Account<'info, ApcObservation>,
    pub oracle_feed: Signer<'info>,
}

#[derive(Accounts)]
pub struct EnterApcRecovery<'info> {
    #[account(seeds = [b"perax-state"], bump = state.bump)]
    pub state: Account<'info, PeraxState>,
    #[account(seeds = [b"apc-config", state.key().as_ref()], bump = apc_config.bump, has_one = oracle_feed @ PeraxError::Unauthorized)]
    pub apc_config: Account<'info, ApcConfig>,
    #[account(mut, seeds = [b"apc-state", apc_config.key().as_ref()], bump = apc_state.bump)]
    pub apc_state: Account<'info, ApcState>,
    #[account(
        mut,
        seeds = [b"apc-observation", observation.observation_id.as_ref()],
        bump = observation.bump,
        constraint = observation.oracle_feed == oracle_feed.key() @ PeraxError::Unauthorized
    )]
    pub observation: Account<'info, ApcObservation>,
    pub oracle_feed: Signer<'info>,
}

#[derive(Accounts)]
#[instruction(params: ExecuteCounterweightPurchaseParams)]
pub struct ExecuteCounterweightPurchase<'info> {
    #[account(seeds = [b"perax-state"], bump = state.bump, constraint = pex_mint.key() == state.token_mint @ PeraxError::InvalidTokenMint)]
    pub state: Account<'info, PeraxState>,
    #[account(seeds = [b"apc-config", state.key().as_ref()], bump = apc_config.bump, has_one = oracle_feed @ PeraxError::Unauthorized)]
    pub apc_config: Account<'info, ApcConfig>,
    #[account(mut, seeds = [b"apc-state", apc_config.key().as_ref()], bump = apc_state.bump)]
    pub apc_state: Account<'info, ApcState>,
    #[account(
        mut,
        seeds = [b"apc-observation", params.observation_id.as_ref()],
        bump = observation.bump,
        constraint = observation.observation_id == params.observation_id @ PeraxError::InvalidReference,
        constraint = observation.oracle_feed == oracle_feed.key() @ PeraxError::Unauthorized
    )]
    pub observation: Account<'info, ApcObservation>,
    #[account(seeds = [b"counterweight-config", apc_config.key().as_ref()], bump = counterweight_config.bump)]
    pub counterweight_config: Account<'info, CounterweightConfig>,
    /// CHECK: PDA authority used by the approved atomic recovery adapter.
    #[account(seeds = [b"counterweight-authority", apc_config.key().as_ref()], bump = counterweight_config.counterweight_authority_bump)]
    pub counterweight_authority: UncheckedAccount<'info>,
    #[account(mut, address = counterweight_config.counterweight_vault @ PeraxError::InvalidCounterweightVault, constraint = counterweight_vault.owner == counterweight_authority.key() @ PeraxError::InvalidCounterweightVault, constraint = counterweight_vault.mint == quote_mint.key() @ PeraxError::InvalidCounterweightMint)]
    pub counterweight_vault: Account<'info, TokenAccount>,
    #[account(
        mut,
        address = counterweight_config.recovery_vault @ PeraxError::InvalidCounterweightVault,
        constraint = recovery_vault.owner == counterweight_config.recovery_authority @ PeraxError::InvalidCounterweightVault,
        constraint = recovery_vault.mint == pex_mint.key() @ PeraxError::InvalidTokenMint
    )]
    pub recovery_vault: Account<'info, TokenAccount>,
    #[account(address = counterweight_config.quote_mint @ PeraxError::InvalidCounterweightMint)]
    pub quote_mint: Account<'info, Mint>,
    pub pex_mint: Account<'info, Mint>,
    /// CHECK: Address is constrained to the immutable approved market pool.
    #[account(address = apc_config.approved_pool @ PeraxError::InvalidApcPool)]
    pub approved_pool: UncheckedAccount<'info>,
    /// CHECK: Executable adapter is constrained to the immutable approved recovery program.
    #[account(address = apc_config.approved_recovery_program @ PeraxError::InvalidRecoveryProgram, executable)]
    pub recovery_program: UncheckedAccount<'info>,
    #[account(
        init,
        payer = oracle_feed,
        space = 8 + ApcRecoveryRecord::INIT_SPACE,
        seeds = [b"apc-recovery", params.recovery_id.as_ref()],
        bump
    )]
    pub recovery_record: Account<'info, ApcRecoveryRecord>,
    #[account(mut)]
    pub oracle_feed: Signer<'info>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct RecoverySwapAdapter<'info> {
    #[account(mut, constraint = counterweight_vault.mint == quote_mint.key() @ PeraxError::InvalidCounterweightMint)]
    pub counterweight_vault: Account<'info, TokenAccount>,
    #[account(mut, constraint = recovery_vault.mint == pex_mint.key() @ PeraxError::InvalidTokenMint)]
    pub recovery_vault: Account<'info, TokenAccount>,
    pub counterweight_authority: Signer<'info>,
    #[account(
        mut,
        constraint = recovery_pool.is_active @ PeraxError::RecoveryPoolInactive,
        constraint = recovery_pool.quote_mint == quote_mint.key() @ PeraxError::InvalidRecoveryPool,
        constraint = recovery_pool.pex_mint == pex_mint.key() @ PeraxError::InvalidRecoveryPool
    )]
    pub recovery_pool: Account<'info, RecoveryPoolConfig>,
    pub token_program: Program<'info, Token>,
    pub quote_mint: Account<'info, Mint>,
    pub pex_mint: Account<'info, Mint>,
    /// CHECK: PDA authority constrained by the recovery pool configuration.
    #[account(
        seeds = [b"recovery-pool-authority", recovery_pool.key().as_ref()],
        bump = recovery_pool.authority_bump,
        constraint = pool_authority.key() == recovery_pool.pool_authority @ PeraxError::InvalidRecoveryPool
    )]
    pub pool_authority: UncheckedAccount<'info>,
    #[account(
        mut,
        address = recovery_pool.pool_quote_vault @ PeraxError::InvalidRecoveryPool,
        constraint = pool_quote_vault.owner == pool_authority.key() @ PeraxError::InvalidRecoveryPool,
        constraint = pool_quote_vault.mint == quote_mint.key() @ PeraxError::InvalidCounterweightMint
    )]
    pub pool_quote_vault: Account<'info, TokenAccount>,
    #[account(
        mut,
        address = recovery_pool.pool_pex_vault @ PeraxError::InvalidRecoveryPool,
        constraint = pool_pex_vault.owner == pool_authority.key() @ PeraxError::InvalidRecoveryPool,
        constraint = pool_pex_vault.mint == pex_mint.key() @ PeraxError::InvalidTokenMint
    )]
    pub pool_pex_vault: Account<'info, TokenAccount>,
}

#[derive(Accounts)]
pub struct PauseApc<'info> {
    #[account(seeds = [b"perax-state"], bump = state.bump)]
    pub state: Account<'info, PeraxState>,
    #[account(mut, seeds = [b"apc-config", state.key().as_ref()], bump = apc_config.bump)]
    pub apc_config: Account<'info, ApcConfig>,
    #[account(mut, seeds = [b"apc-state", apc_config.key().as_ref()], bump = apc_state.bump)]
    pub apc_state: Account<'info, ApcState>,
    pub actor: Signer<'info>,
}
