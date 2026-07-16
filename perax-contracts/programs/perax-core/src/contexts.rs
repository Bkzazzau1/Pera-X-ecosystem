use crate::{
    BurnExecutionRecord, ConditionalBuybackBurnParams, InitializeReserveVaultParams,
    MarketConditionBurnParams, MarketConditionalReleaseParams, PaymentRecord, PeraxError,
    PeraxState, ReleaseRecord, ReserveReleaseRecord, ReserveVaultConfig,
    VaultMarketConditionalReleaseParams,
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
