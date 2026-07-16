use crate::{
    validate_payment_amount, validate_reference, ExternalUtilityPaymentRecorded,
    PayToTradingCompany, PeraxError, RecordExternalUtilityPayment, UtilityPaymentReceived,
};
use anchor_lang::prelude::*;
use anchor_spl::token;

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
    payment_record.trading_company_revenue_token_account =
        state.trading_company_revenue_token_account;
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
