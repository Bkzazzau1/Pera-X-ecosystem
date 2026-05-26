use anchor_lang::prelude::*;
use anchor_spl::token::{self, Burn, Mint, Token, TokenAccount, Transfer};

declare_id!("11111111111111111111111111111111");

#[program]
pub mod perax_core {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, params: InitializeParams) -> Result<()> {
        require!(
            params.trading_company_token_account != Pubkey::default(),
            PeraxError::InvalidTradingCompanyAccount
        );
        require!(
            params.trading_company_revenue_token_account != Pubkey::default(),
            PeraxError::InvalidTradingCompanyRevenueAccount
        );
        require!(
            params.trading_company_token_account != params.trading_company_revenue_token_account,
            PeraxError::TradingCompanyAccountsMustDiffer
        );

        let state = &mut ctx.accounts.state;
        state.authority = ctx.accounts.authority.key();
        state.pending_authority = Pubkey::default();
        state.has_pending_authority = false;
        state.token_mint = params.token_mint;
        state.trading_company_token_account = params.trading_company_token_account;
        state.trading_company_revenue_token_account = params.trading_company_revenue_token_account;
        state.max_payment_amount = params.max_payment_amount;
        state.is_paused = false;
        state.bump = ctx.bumps.state;

        emit!(ConfigInitialized {
            authority: state.authority,
            token_mint: state.token_mint,
            trading_company_token_account: state.trading_company_token_account,
            trading_company_revenue_token_account: state.trading_company_revenue_token_account,
            max_payment_amount: state.max_payment_amount,
        });

        Ok(())
    }

    pub fn update_config(ctx: Context<UpdateConfig>, params: UpdateConfigParams) -> Result<()> {
        let state = &mut ctx.accounts.state;

        if let Some(trading_company_token_account) = params.trading_company_token_account {
            require!(
                trading_company_token_account != Pubkey::default(),
                PeraxError::InvalidTradingCompanyAccount
            );
            require!(
                trading_company_token_account != state.trading_company_revenue_token_account,
                PeraxError::TradingCompanyAccountsMustDiffer
            );
            state.trading_company_token_account = trading_company_token_account;
        }

        if let Some(trading_company_revenue_token_account) = params.trading_company_revenue_token_account {
            require!(
                trading_company_revenue_token_account != Pubkey::default(),
                PeraxError::InvalidTradingCompanyRevenueAccount
            );
            require!(
                trading_company_revenue_token_account != state.trading_company_token_account,
                PeraxError::TradingCompanyAccountsMustDiffer
            );
            state.trading_company_revenue_token_account = trading_company_revenue_token_account;
        }

        if let Some(max_payment_amount) = params.max_payment_amount {
            state.max_payment_amount = max_payment_amount;
        }

        emit!(ConfigUpdated {
            authority: state.authority,
            trading_company_token_account: state.trading_company_token_account,
            trading_company_revenue_token_account: state.trading_company_revenue_token_account,
            max_payment_amount: state.max_payment_amount,
        });

        Ok(())
    }

    pub fn set_pause(ctx: Context<UpdateConfig>, is_paused: bool) -> Result<()> {
        let state = &mut ctx.accounts.state;
        state.is_paused = is_paused;

        emit!(PauseStatusChanged {
            authority: state.authority,
            is_paused,
        });

        Ok(())
    }

    pub fn nominate_authority(ctx: Context<UpdateConfig>, new_authority: Pubkey) -> Result<()> {
        require!(new_authority != Pubkey::default(), PeraxError::InvalidAuthority);

        let state = &mut ctx.accounts.state;
        require!(new_authority != state.authority, PeraxError::InvalidAuthority);

        state.pending_authority = new_authority;
        state.has_pending_authority = true;

        emit!(AuthorityTransferNominated {
            current_authority: state.authority,
            pending_authority: state.pending_authority,
        });

        Ok(())
    }

    pub fn cancel_authority_transfer(ctx: Context<UpdateConfig>) -> Result<()> {
        let state = &mut ctx.accounts.state;
        require!(state.has_pending_authority, PeraxError::NoPendingAuthority);

        let cancelled_authority = state.pending_authority;
        state.pending_authority = Pubkey::default();
        state.has_pending_authority = false;

        emit!(AuthorityTransferCancelled {
            authority: state.authority,
            cancelled_authority,
        });

        Ok(())
    }

    pub fn accept_authority(ctx: Context<AcceptAuthority>) -> Result<()> {
        let state = &mut ctx.accounts.state;
        let new_authority = ctx.accounts.pending_authority.key();

        require!(state.has_pending_authority, PeraxError::NoPendingAuthority);
        require!(state.pending_authority == new_authority, PeraxError::Unauthorized);

        let previous_authority = state.authority;
        state.authority = new_authority;
        state.pending_authority = Pubkey::default();
        state.has_pending_authority = false;

        emit!(AuthorityTransferAccepted {
            previous_authority,
            new_authority,
        });

        Ok(())
    }

    pub fn pay_to_trading_company(
        ctx: Context<PayToTradingCompany>,
        amount: u64,
        reference: [u8; 32],
    ) -> Result<()> {
        let state = &ctx.accounts.state;
        require!(!state.is_paused, PeraxError::ProgramPaused);
        validate_payment_amount(state, amount)?;
        validate_reference(reference)?;

        let payment_record = &mut ctx.accounts.payment_record;
        payment_record.reference = reference;
        payment_record.payer = ctx.accounts.payer.key();
        payment_record.amount = amount;
        payment_record.token_mint = state.token_mint;
        payment_record.trading_company_token_account = state.trading_company_token_account;
        payment_record.trading_company_revenue_token_account = state.trading_company_revenue_token_account;
        payment_record.created_at = Clock::get()?.unix_timestamp;
        payment_record.bump = ctx.bumps.payment_record;

        token::transfer(ctx.accounts.payment_transfer_ctx(), amount)?;

        emit!(UtilityPaymentReceived {
            payer: ctx.accounts.payer.key(),
            token_mint: state.token_mint,
            trading_company_token_account: state.trading_company_token_account,
            trading_company_revenue_token_account: state.trading_company_revenue_token_account,
            amount,
            reference,
        });

        Ok(())
    }

    pub fn record_external_utility_payment(
        ctx: Context<RecordExternalUtilityPayment>,
        amount: u64,
        reference: [u8; 32],
        payment_source: [u8; 16],
    ) -> Result<()> {
        let state = &ctx.accounts.state;
        require!(!state.is_paused, PeraxError::ProgramPaused);
        validate_payment_amount(state, amount)?;
        validate_reference(reference)?;

        emit!(ExternalUtilityPaymentRecorded {
            authority: ctx.accounts.authority.key(),
            token_mint: state.token_mint,
            amount,
            reference,
            payment_source,
        });

        Ok(())
    }

    pub fn burn_from_trading_company(
        ctx: Context<BurnFromTradingCompany>,
        amount: u64,
        decision_id: [u8; 32],
    ) -> Result<()> {
        let state = &ctx.accounts.state;
        require!(!state.is_paused, PeraxError::ProgramPaused);
        require!(amount > 0, PeraxError::InvalidAmount);
        validate_reference(decision_id)?;

        token::burn(ctx.accounts.burn_ctx(), amount)?;

        emit!(TradingCompanyBurnExecuted {
            authority: ctx.accounts.authority.key(),
            trading_company_authority: ctx.accounts.trading_company_authority.key(),
            token_mint: state.token_mint,
            trading_company_token_account: state.trading_company_token_account,
            trading_company_revenue_token_account: state.trading_company_revenue_token_account,
            amount,
            decision_id,
        });

        Ok(())
    }
}

fn validate_payment_amount(state: &PeraxState, amount: u64) -> Result<()> {
    require!(amount > 0, PeraxError::InvalidAmount);

    if state.max_payment_amount > 0 {
        require!(amount <= state.max_payment_amount, PeraxError::PaymentAmountTooLarge);
    }

    Ok(())
}

fn validate_reference(reference: [u8; 32]) -> Result<()> {
    require!(reference != [0u8; 32], PeraxError::InvalidReference);
    Ok(())
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct InitializeParams {
    pub token_mint: Pubkey,
    pub trading_company_token_account: Pubkey,
    pub trading_company_revenue_token_account: Pubkey,
    pub max_payment_amount: u64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct UpdateConfigParams {
    pub trading_company_token_account: Option<Pubkey>,
    pub trading_company_revenue_token_account: Option<Pubkey>,
    pub max_payment_amount: Option<u64>,
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + PeraxState::INIT_SPACE,
        seeds = [b"perax-state"],
        bump
    )]
    pub state: Account<'info, PeraxState>,

    #[account(mut)]
    pub authority: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct UpdateConfig<'info> {
    #[account(
        mut,
        seeds = [b"perax-state"],
        bump = state.bump,
        has_one = authority @ PeraxError::Unauthorized
    )]
    pub state: Account<'info, PeraxState>,

    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct AcceptAuthority<'info> {
    #[account(
        mut,
        seeds = [b"perax-state"],
        bump = state.bump
    )]
    pub state: Account<'info, PeraxState>,

    pub pending_authority: Signer<'info>,
}

#[derive(Accounts)]
#[instruction(amount: u64, reference: [u8; 32])]
pub struct PayToTradingCompany<'info> {
    #[account(
        seeds = [b"perax-state"],
        bump = state.bump,
        constraint = token_mint.key() == state.token_mint @ PeraxError::InvalidTokenMint,
        constraint = trading_company_revenue_token_account.key() == state.trading_company_revenue_token_account @ PeraxError::InvalidTradingCompanyRevenueAccount
    )]
    pub state: Account<'info, PeraxState>,

    #[account(
        init,
        payer = payer,
        space = 8 + PaymentRecord::SPACE,
        seeds = [b"payment", reference.as_ref()],
        bump
    )]
    pub payment_record: Account<'info, PaymentRecord>,

    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        mut,
        constraint = payer_token_account.owner == payer.key() @ PeraxError::Unauthorized,
        constraint = payer_token_account.mint == token_mint.key() @ PeraxError::InvalidTokenMint
    )]
    pub payer_token_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        constraint = trading_company_revenue_token_account.mint == token_mint.key() @ PeraxError::InvalidTokenMint
    )]
    pub trading_company_revenue_token_account: Account<'info, TokenAccount>,

    pub token_mint: Account<'info, Mint>,

    pub token_program: Program<'info, Token>,

    pub system_program: Program<'info, System>,
}

impl<'info> PayToTradingCompany<'info> {
    fn payment_transfer_ctx(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
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
    #[account(
        seeds = [b"perax-state"],
        bump = state.bump,
        has_one = authority @ PeraxError::Unauthorized
    )]
    pub state: Account<'info, PeraxState>,

    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct BurnFromTradingCompany<'info> {
    #[account(
        seeds = [b"perax-state"],
        bump = state.bump,
        has_one = authority @ PeraxError::Unauthorized,
        constraint = token_mint.key() == state.token_mint @ PeraxError::InvalidTokenMint,
        constraint = trading_company_revenue_token_account.key() == state.trading_company_revenue_token_account @ PeraxError::InvalidTradingCompanyRevenueAccount
    )]
    pub state: Account<'info, PeraxState>,

    pub authority: Signer<'info>,

    pub trading_company_authority: Signer<'info>,

    #[account(
        mut,
        constraint = trading_company_revenue_token_account.owner == trading_company_authority.key() @ PeraxError::Unauthorized,
        constraint = trading_company_revenue_token_account.mint == token_mint.key() @ PeraxError::InvalidTokenMint
    )]
    pub trading_company_revenue_token_account: Account<'info, TokenAccount>,

    #[account(mut)]
    pub token_mint: Account<'info, Mint>,

    pub token_program: Program<'info, Token>,
}

impl<'info> BurnFromTradingCompany<'info> {
    fn burn_ctx(&self) -> CpiContext<'_, '_, '_, 'info, Burn<'info>> {
        let accounts = Burn {
            mint: self.token_mint.to_account_info(),
            from: self.trading_company_revenue_token_account.to_account_info(),
            authority: self.trading_company_authority.to_account_info(),
        };
        CpiContext::new(self.token_program.to_account_info(), accounts)
    }
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
    pub is_paused: bool,
    pub bump: u8,
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

#[event]
pub struct ConfigInitialized {
    pub authority: Pubkey,
    pub token_mint: Pubkey,
    pub trading_company_token_account: Pubkey,
    pub trading_company_revenue_token_account: Pubkey,
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
pub struct PauseStatusChanged {
    pub authority: Pubkey,
    pub is_paused: bool,
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

#[error_code]
pub enum PeraxError {
    #[msg("The caller is not authorized to perform this action.")]
    Unauthorized,
    #[msg("The program is currently paused.")]
    ProgramPaused,
    #[msg("Amount must be greater than zero.")]
    InvalidAmount,
    #[msg("The token mint does not match the configured Pera-X mint.")]
    InvalidTokenMint,
    #[msg("The trading company locked token account does not match the configured account.")]
    InvalidTradingCompanyAccount,
    #[msg("The trading company revenue token account does not match the configured account.")]
    InvalidTradingCompanyRevenueAccount,
    #[msg("Trading company locked and revenue token accounts must be different.")]
    TradingCompanyAccountsMustDiffer,
    #[msg("The payment amount is above the configured maximum payment amount.")]
    PaymentAmountTooLarge,
    #[msg("The new authority is invalid.")]
    InvalidAuthority,
    #[msg("There is no pending authority transfer.")]
    NoPendingAuthority,
    #[msg("The payment or decision reference is invalid.")]
    InvalidReference,
}

#[cfg(test)]
mod tests;
