use crate::{
    reset_burn_window_if_needed, validate_apc_burn_allowed, validate_market_condition_burn,
    validate_reference, BurnExecutionRecord, BurnFromTradingCompany, BurnFulfillmentSource,
    ConditionalBuybackBurnExecuted, ConditionalBuybackBurnParams, ExecuteConditionalBuybackBurn,
    ExecuteMarketConditionBurn, MarketConditionBurnExecuted, MarketConditionBurnParams, PeraxError,
    PeraxState,
};
use anchor_lang::prelude::*;
use anchor_spl::token::{self, Burn};

pub fn burn_from_trading_company(
    _ctx: Context<BurnFromTradingCompany>,
    _amount: u64,
    _decision_id: [u8; 32],
) -> Result<()> {
    err!(PeraxError::UseMarketConditionBurn)
}

pub fn execute_market_condition_burn(
    ctx: Context<ExecuteMarketConditionBurn>,
    params: MarketConditionBurnParams,
) -> Result<()> {
    validate_apc_burn_allowed(&ctx.accounts.apc_state)?;
    let now = Clock::get()?.unix_timestamp;
    require!(
        params.observed_at
            <= now.saturating_add(ctx.accounts.apc_config.maximum_future_clock_skew_seconds),
        PeraxError::ObservationFromFuture
    );
    require!(
        now.saturating_sub(params.observed_at)
            <= ctx.accounts.apc_config.maximum_observation_age_seconds,
        PeraxError::ObservationStale
    );
    let burn_params = ConditionalBuybackBurnParams {
        amount: params.amount,
        eligible_revenue_amount: params.eligible_revenue_amount,
        burn_rate_bps: params.burn_rate_bps,
        market_health_score: params.market_health_score,
        observed_at: params.observed_at,
        decision_id: params.decision_id,
        burn_source: BurnFulfillmentSource::OpenMarketPurchase,
    };
    let burn_ctx = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        Burn {
            mint: ctx.accounts.token_mint.to_account_info(),
            from: ctx
                .accounts
                .trading_company_revenue_token_account
                .to_account_info(),
            authority: ctx.accounts.trading_company_authority.to_account_info(),
        },
    );

    execute_validated_burn(
        &mut ctx.accounts.state,
        ctx.accounts.authority.key(),
        ctx.accounts.trading_company_authority.key(),
        ctx.accounts.trading_company_revenue_token_account.key(),
        &mut ctx.accounts.burn_record,
        burn_ctx,
        burn_params,
        ctx.accounts.token_mint.supply,
        ctx.bumps.burn_record,
    )
}

pub fn execute_conditional_buyback_burn(
    ctx: Context<ExecuteConditionalBuybackBurn>,
    params: ConditionalBuybackBurnParams,
) -> Result<()> {
    validate_apc_burn_allowed(&ctx.accounts.apc_state)?;
    let now = Clock::get()?.unix_timestamp;
    require!(
        params.observed_at
            <= now.saturating_add(ctx.accounts.apc_config.maximum_future_clock_skew_seconds),
        PeraxError::ObservationFromFuture
    );
    require!(
        now.saturating_sub(params.observed_at)
            <= ctx.accounts.apc_config.maximum_observation_age_seconds,
        PeraxError::ObservationStale
    );
    let state = &ctx.accounts.state;

    match params.burn_source {
        BurnFulfillmentSource::OpenMarketPurchase => {
            require!(
                ctx.accounts.source_token_account.key()
                    == state.trading_company_revenue_token_account,
                PeraxError::InvalidBurnSourceAccount
            );
        }
        BurnFulfillmentSource::TradingTreasury => {
            require!(
                ctx.accounts.source_token_account.key() == state.trading_company_token_account,
                PeraxError::InvalidBurnSourceAccount
            );
        }
    }
    let burn_ctx = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        Burn {
            mint: ctx.accounts.token_mint.to_account_info(),
            from: ctx.accounts.source_token_account.to_account_info(),
            authority: ctx.accounts.source_authority.to_account_info(),
        },
    );

    execute_validated_burn(
        &mut ctx.accounts.state,
        ctx.accounts.authority.key(),
        ctx.accounts.source_authority.key(),
        ctx.accounts.source_token_account.key(),
        &mut ctx.accounts.burn_record,
        burn_ctx,
        params,
        ctx.accounts.token_mint.supply,
        ctx.bumps.burn_record,
    )
}

fn execute_validated_burn<'info>(
    state: &mut Account<'info, PeraxState>,
    authority: Pubkey,
    source_authority: Pubkey,
    source_token_account: Pubkey,
    burn_record: &mut Account<'info, BurnExecutionRecord>,
    burn_ctx: CpiContext<'_, '_, '_, 'info, Burn<'info>>,
    params: ConditionalBuybackBurnParams,
    current_mint_supply: u64,
    bump: u8,
) -> Result<()> {
    require!(!state.is_paused, PeraxError::ProgramPaused);
    require!(!state.emergency_pause, PeraxError::EmergencyPaused);
    require!(params.amount > 0, PeraxError::InvalidAmount);
    require!(
        params.eligible_revenue_amount > 0,
        PeraxError::InvalidAmount
    );
    require!(params.observed_at > 0, PeraxError::InvalidMarketParameter);
    validate_reference(params.decision_id)?;

    let executed_at = Clock::get()?.unix_timestamp;
    reset_burn_window_if_needed(state, executed_at);
    validate_market_condition_burn(state, &params, current_mint_supply)?;

    token::burn(burn_ctx, params.amount)?;

    state.daily_burn_accumulator = state
        .daily_burn_accumulator
        .checked_add(params.amount)
        .ok_or(PeraxError::DailyBurnCapExceeded)?;

    burn_record.decision_id = params.decision_id;
    burn_record.authority = authority;
    burn_record.trading_company_authority = source_authority;
    burn_record.token_mint = state.token_mint;
    burn_record.trading_company_revenue_token_account = state.trading_company_revenue_token_account;
    burn_record.source_token_account = source_token_account;
    burn_record.burn_source = params.burn_source;
    burn_record.amount = params.amount;
    burn_record.eligible_revenue_amount = params.eligible_revenue_amount;
    burn_record.burn_rate_bps = params.burn_rate_bps;
    burn_record.market_health_score = params.market_health_score;
    burn_record.observed_at = params.observed_at;
    burn_record.executed_at = executed_at;
    burn_record.bump = bump;

    emit!(ConditionalBuybackBurnExecuted {
        burn_record: burn_record.key(),
        authority,
        source_authority,
        token_mint: state.token_mint,
        source_token_account,
        burn_source: params.burn_source,
        amount: params.amount,
        eligible_revenue_amount: params.eligible_revenue_amount,
        burn_rate_bps: params.burn_rate_bps,
        market_health_score: params.market_health_score,
        decision_id: params.decision_id,
        observed_at: params.observed_at,
        executed_at,
    });

    emit!(MarketConditionBurnExecuted {
        burn_record: burn_record.key(),
        authority,
        trading_company_authority: source_authority,
        token_mint: state.token_mint,
        trading_company_revenue_token_account: state.trading_company_revenue_token_account,
        amount: params.amount,
        eligible_revenue_amount: params.eligible_revenue_amount,
        burn_rate_bps: params.burn_rate_bps,
        market_health_score: params.market_health_score,
        decision_id: params.decision_id,
        observed_at: params.observed_at,
        executed_at,
    });

    Ok(())
}
