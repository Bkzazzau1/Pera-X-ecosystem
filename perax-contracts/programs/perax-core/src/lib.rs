use anchor_lang::prelude::*;
use anchor_spl::token::{self, Burn, Mint, Token, TokenAccount, Transfer};

declare_id!("11111111111111111111111111111111");

const MAX_BPS: u16 = 10_000;

#[program]
pub mod perax_core {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, params: InitializeParams) -> Result<()> {
        require!(params.burn_bps <= MAX_BPS, PeraxError::InvalidBasisPoints);
        require!(params.treasury_bps <= MAX_BPS, PeraxError::InvalidBasisPoints);
        require!(
            params.burn_bps.saturating_add(params.treasury_bps) <= MAX_BPS,
            PeraxError::InvalidBasisPoints
        );

        let state = &mut ctx.accounts.state;
        state.authority = ctx.accounts.authority.key();
        state.token_mint = params.token_mint;
        state.treasury = params.treasury;
        state.utility_vault = params.utility_vault;
        state.burn_bps = params.burn_bps;
        state.treasury_bps = params.treasury_bps;
        state.is_paused = false;
        state.bump = ctx.bumps.state;

        emit!(ConfigInitialized {
            authority: state.authority,
            token_mint: state.token_mint,
            treasury: state.treasury,
            utility_vault: state.utility_vault,
            burn_bps: state.burn_bps,
            treasury_bps: state.treasury_bps,
        });

        Ok(())
    }

    pub fn update_config(ctx: Context<UpdateConfig>, params: UpdateConfigParams) -> Result<()> {
        let state = &mut ctx.accounts.state;

        if let Some(treasury) = params.treasury {
            state.treasury = treasury;
        }

        if let Some(utility_vault) = params.utility_vault {
            state.utility_vault = utility_vault;
        }

        if let Some(burn_bps) = params.burn_bps {
            require!(burn_bps <= MAX_BPS, PeraxError::InvalidBasisPoints);
            state.burn_bps = burn_bps;
        }

        if let Some(treasury_bps) = params.treasury_bps {
            require!(treasury_bps <= MAX_BPS, PeraxError::InvalidBasisPoints);
            state.treasury_bps = treasury_bps;
        }

        require!(
            state.burn_bps.saturating_add(state.treasury_bps) <= MAX_BPS,
            PeraxError::InvalidBasisPoints
        );

        emit!(ConfigUpdated {
            authority: state.authority,
            treasury: state.treasury,
            utility_vault: state.utility_vault,
            burn_bps: state.burn_bps,
            treasury_bps: state.treasury_bps,
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

    pub fn record_utility_payment(
        ctx: Context<RecordUtilityPayment>,
        amount: u64,
        burn_amount: u64,
        treasury_amount: u64,
        reference: [u8; 32],
    ) -> Result<()> {
        let state = &ctx.accounts.state;
        require!(!state.is_paused, PeraxError::ProgramPaused);
        require!(amount > 0, PeraxError::InvalidAmount);
        require!(
            burn_amount.saturating_add(treasury_amount) <= amount,
            PeraxError::InvalidPaymentSplit
        );

        emit!(UtilityPaymentRecorded {
            payer: ctx.accounts.payer.key(),
            token_mint: state.token_mint,
            amount,
            burn_amount,
            treasury_amount,
            reference,
        });

        Ok(())
    }

    pub fn pay_with_split(ctx: Context<PayWithSplit>, amount: u64, reference: [u8; 32]) -> Result<()> {
        let state = &ctx.accounts.state;
        require!(!state.is_paused, PeraxError::ProgramPaused);
        require!(amount > 0, PeraxError::InvalidAmount);

        let burn_amount = amount
            .checked_mul(state.burn_bps as u64)
            .ok_or(PeraxError::MathOverflow)?
            .checked_div(MAX_BPS as u64)
            .ok_or(PeraxError::MathOverflow)?;

        let treasury_amount = amount
            .checked_mul(state.treasury_bps as u64)
            .ok_or(PeraxError::MathOverflow)?
            .checked_div(MAX_BPS as u64)
            .ok_or(PeraxError::MathOverflow)?;

        let utility_amount = amount
            .checked_sub(burn_amount)
            .ok_or(PeraxError::MathOverflow)?
            .checked_sub(treasury_amount)
            .ok_or(PeraxError::MathOverflow)?;

        if burn_amount > 0 {
            token::burn(ctx.accounts.burn_ctx(), burn_amount)?;
        }

        if treasury_amount > 0 {
            token::transfer(ctx.accounts.treasury_transfer_ctx(), treasury_amount)?;
        }

        if utility_amount > 0 {
            token::transfer(ctx.accounts.utility_transfer_ctx(), utility_amount)?;
        }

        emit!(UtilityPaymentExecuted {
            payer: ctx.accounts.payer.key(),
            token_mint: state.token_mint,
            amount,
            burn_amount,
            treasury_amount,
            utility_amount,
            reference,
        });

        Ok(())
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct InitializeParams {
    pub token_mint: Pubkey,
    pub treasury: Pubkey,
    pub utility_vault: Pubkey,
    pub burn_bps: u16,
    pub treasury_bps: u16,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct UpdateConfigParams {
    pub treasury: Option<Pubkey>,
    pub utility_vault: Option<Pubkey>,
    pub burn_bps: Option<u16>,
    pub treasury_bps: Option<u16>,
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
pub struct RecordUtilityPayment<'info> {
    #[account(
        seeds = [b"perax-state"],
        bump = state.bump
    )]
    pub state: Account<'info, PeraxState>,

    pub payer: Signer<'info>,
}

#[derive(Accounts)]
pub struct PayWithSplit<'info> {
    #[account(
        seeds = [b"perax-state"],
        bump = state.bump,
        constraint = token_mint.key() == state.token_mint @ PeraxError::InvalidTokenMint,
        constraint = treasury_token_account.key() == state.treasury @ PeraxError::InvalidTreasuryAccount,
        constraint = utility_vault_token_account.key() == state.utility_vault @ PeraxError::InvalidUtilityVault
    )]
    pub state: Account<'info, PeraxState>,

    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(mut, constraint = payer_token_account.owner == payer.key() @ PeraxError::Unauthorized)]
    pub payer_token_account: Account<'info, TokenAccount>,

    #[account(mut)]
    pub treasury_token_account: Account<'info, TokenAccount>,

    #[account(mut)]
    pub utility_vault_token_account: Account<'info, TokenAccount>,

    #[account(mut)]
    pub token_mint: Account<'info, Mint>,

    pub token_program: Program<'info, Token>,
}

impl<'info> PayWithSplit<'info> {
    fn burn_ctx(&self) -> CpiContext<'_, '_, '_, 'info, Burn<'info>> {
        let accounts = Burn {
            mint: self.token_mint.to_account_info(),
            from: self.payer_token_account.to_account_info(),
            authority: self.payer.to_account_info(),
        };
        CpiContext::new(self.token_program.to_account_info(), accounts)
    }

    fn treasury_transfer_ctx(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        let accounts = Transfer {
            from: self.payer_token_account.to_account_info(),
            to: self.treasury_token_account.to_account_info(),
            authority: self.payer.to_account_info(),
        };
        CpiContext::new(self.token_program.to_account_info(), accounts)
    }

    fn utility_transfer_ctx(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        let accounts = Transfer {
            from: self.payer_token_account.to_account_info(),
            to: self.utility_vault_token_account.to_account_info(),
            authority: self.payer.to_account_info(),
        };
        CpiContext::new(self.token_program.to_account_info(), accounts)
    }
}

#[account]
#[derive(InitSpace)]
pub struct PeraxState {
    pub authority: Pubkey,
    pub token_mint: Pubkey,
    pub treasury: Pubkey,
    pub utility_vault: Pubkey,
    pub burn_bps: u16,
    pub treasury_bps: u16,
    pub is_paused: bool,
    pub bump: u8,
}

#[event]
pub struct ConfigInitialized {
    pub authority: Pubkey,
    pub token_mint: Pubkey,
    pub treasury: Pubkey,
    pub utility_vault: Pubkey,
    pub burn_bps: u16,
    pub treasury_bps: u16,
}

#[event]
pub struct ConfigUpdated {
    pub authority: Pubkey,
    pub treasury: Pubkey,
    pub utility_vault: Pubkey,
    pub burn_bps: u16,
    pub treasury_bps: u16,
}

#[event]
pub struct PauseStatusChanged {
    pub authority: Pubkey,
    pub is_paused: bool,
}

#[event]
pub struct UtilityPaymentRecorded {
    pub payer: Pubkey,
    pub token_mint: Pubkey,
    pub amount: u64,
    pub burn_amount: u64,
    pub treasury_amount: u64,
    pub reference: [u8; 32],
}

#[event]
pub struct UtilityPaymentExecuted {
    pub payer: Pubkey,
    pub token_mint: Pubkey,
    pub amount: u64,
    pub burn_amount: u64,
    pub treasury_amount: u64,
    pub utility_amount: u64,
    pub reference: [u8; 32],
}

#[error_code]
pub enum PeraxError {
    #[msg("The caller is not authorized to perform this action.")]
    Unauthorized,
    #[msg("Basis points must be between 0 and 10,000, and total split cannot exceed 10,000.")]
    InvalidBasisPoints,
    #[msg("The program is currently paused.")]
    ProgramPaused,
    #[msg("Amount must be greater than zero.")]
    InvalidAmount,
    #[msg("Burn and treasury amounts cannot exceed the total payment amount.")]
    InvalidPaymentSplit,
    #[msg("The payment split calculation overflowed.")]
    MathOverflow,
    #[msg("The token mint does not match the configured Pera-X mint.")]
    InvalidTokenMint,
    #[msg("The treasury token account does not match the configured treasury.")]
    InvalidTreasuryAccount,
    #[msg("The utility vault token account does not match the configured utility vault.")]
    InvalidUtilityVault,
}
