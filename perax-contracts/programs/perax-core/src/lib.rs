use anchor_lang::prelude::*;

declare_id!("11111111111111111111111111111111");

#[program]
pub mod perax_core {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        let state = &mut ctx.accounts.state;
        state.authority = ctx.accounts.authority.key();
        state.bump = ctx.bumps.state;
        Ok(())
    }
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

#[account]
#[derive(InitSpace)]
pub struct PeraxState {
    pub authority: Pubkey,
    pub bump: u8,
}
