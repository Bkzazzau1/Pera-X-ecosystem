use crate::{
    reset_burn_window_if_needed, reset_deferred_burn_window_if_needed,
    validate_deferred_burn_limits, validate_reference, ApcStatus, CounterweightProceedsDeposited,
    DeferredBurnExecuted, DeferredBurnRecorded, DepositCounterweightParams,
    DepositCounterweightProceeds, ExecuteDeferredBurn, ExecuteDeferredBurnParams, PeraxError,
    RecordDeferredBurn, RecordDeferredBurnParams,
};
use anchor_lang::prelude::*;
use anchor_spl::token::{self, Burn, TransferChecked};

pub fn deposit_counterweight_proceeds(
    ctx: Context<DepositCounterweightProceeds>,
    params: DepositCounterweightParams,
) -> Result<()> {
    validate_reference(params.deposit_id)?;
    require!(params.amount > 0, PeraxError::InvalidAmount);
    require!(!ctx.accounts.state.is_paused, PeraxError::ProgramPaused);
    require!(ctx.accounts.apc_config.is_active, PeraxError::ApcInactive);
    require!(!ctx.accounts.apc_config.is_paused, PeraxError::ApcPaused);

    let credited_after = ctx
        .accounts
        .apc_state
        .total_counterweight_credited
        .checked_add(params.amount)
        .ok_or(PeraxError::InvalidCounterweightVault)?;

    let transfer_accounts = TransferChecked {
        mint: ctx.accounts.quote_mint.to_account_info(),
        from: ctx.accounts.source_token_account.to_account_info(),
        to: ctx.accounts.counterweight_vault.to_account_info(),
        authority: ctx.accounts.source_owner.to_account_info(),
    };
    token::transfer_checked(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            transfer_accounts,
        ),
        params.amount,
        ctx.accounts.quote_mint.decimals,
    )?;

    ctx.accounts.apc_state.total_counterweight_credited = credited_after;
    let deposited_at = Clock::get()?.unix_timestamp;
    let record = &mut ctx.accounts.deposit_record;
    record.deposit_id = params.deposit_id;
    record.source_owner = ctx.accounts.source_owner.key();
    record.source_token_account = ctx.accounts.source_token_account.key();
    record.amount = params.amount;
    record.credited_after = credited_after;
    record.deposited_at = deposited_at;
    record.bump = ctx.bumps.deposit_record;

    emit!(CounterweightProceedsDeposited {
        deposit_record: record.key(),
        deposit_id: record.deposit_id,
        source_owner: record.source_owner,
        source_token_account: record.source_token_account,
        counterweight_vault: ctx.accounts.counterweight_vault.key(),
        amount: record.amount,
        credited_after,
        deposited_at,
    });

    Ok(())
}

pub fn record_deferred_burn(
    ctx: Context<RecordDeferredBurn>,
    params: RecordDeferredBurnParams,
) -> Result<()> {
    validate_reference(params.decision_id)?;
    require!(params.amount > 0, PeraxError::InvalidAmount);
    require!(!ctx.accounts.state.is_paused, PeraxError::ProgramPaused);
    require!(!ctx.accounts.apc_config.is_paused, PeraxError::ApcPaused);
    require!(
        matches!(
            ctx.accounts.apc_state.status,
            ApcStatus::PumpControl | ApcStatus::AwaitingAbsorption | ApcStatus::Recovery
        ),
        PeraxError::InvalidApcStatus
    );

    let now = Clock::get()?.unix_timestamp;
    require!(params.observed_at > 0, PeraxError::InvalidMarketParameter);
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

    let total_deferred_after = ctx
        .accounts
        .apc_state
        .deferred_burn_amount
        .checked_add(params.amount)
        .ok_or(PeraxError::InvalidAmount)?;

    let transfer_accounts = TransferChecked {
        mint: ctx.accounts.token_mint.to_account_info(),
        from: ctx.accounts.source_token_account.to_account_info(),
        to: ctx.accounts.deferred_burn_vault.to_account_info(),
        authority: ctx.accounts.source_authority.to_account_info(),
    };
    token::transfer_checked(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            transfer_accounts,
        ),
        params.amount,
        ctx.accounts.token_mint.decimals,
    )?;

    ctx.accounts.apc_state.deferred_burn_amount = total_deferred_after;
    let record = &mut ctx.accounts.deferred_burn_record;
    record.decision_id = params.decision_id;
    record.state = ctx.accounts.state.key();
    record.apc_state = ctx.accounts.apc_state.key();
    record.source_token_account = ctx.accounts.source_token_account.key();
    record.amount = params.amount;
    record.amount_executed = 0;
    record.observed_at = params.observed_at;
    record.recorded_at = now;
    record.last_executed_at = 0;
    record.is_complete = false;
    record.bump = ctx.bumps.deferred_burn_record;

    emit!(DeferredBurnRecorded {
        deferred_burn_record: record.key(),
        decision_id: record.decision_id,
        source_token_account: record.source_token_account,
        deferred_burn_vault: ctx.accounts.deferred_burn_vault.key(),
        amount: record.amount,
        total_deferred_after,
        recorded_at: now,
    });

    Ok(())
}

pub fn execute_deferred_burn(
    ctx: Context<ExecuteDeferredBurn>,
    params: ExecuteDeferredBurnParams,
) -> Result<()> {
    require!(params.amount > 0, PeraxError::InvalidAmount);
    require!(!ctx.accounts.state.is_paused, PeraxError::ProgramPaused);
    require!(!ctx.accounts.apc_config.is_paused, PeraxError::ApcPaused);
    require!(
        matches!(
            ctx.accounts.apc_state.status,
            ApcStatus::Armed | ApcStatus::Active
        ),
        PeraxError::DeferredBurnNotExecutable
    );
    require!(
        ctx.accounts.deferred_burn_record.apc_state == ctx.accounts.apc_state.key(),
        PeraxError::DeferredBurnNotExecutable
    );
    require!(
        !ctx.accounts.deferred_burn_record.is_complete,
        PeraxError::DeferredBurnNotExecutable
    );

    let now = Clock::get()?.unix_timestamp;
    reset_burn_window_if_needed(&mut ctx.accounts.state, now);
    reset_deferred_burn_window_if_needed(
        &ctx.accounts.apc_config,
        &mut ctx.accounts.apc_state,
        now,
    );
    validate_deferred_burn_limits(
        &ctx.accounts.apc_config,
        &ctx.accounts.apc_state,
        &ctx.accounts.state,
        params.amount,
        ctx.accounts.token_mint.supply,
        now,
    )?;

    let remaining_record_amount = ctx
        .accounts
        .deferred_burn_record
        .amount
        .checked_sub(ctx.accounts.deferred_burn_record.amount_executed)
        .ok_or(PeraxError::DeferredBurnNotExecutable)?;
    require!(
        params.amount <= remaining_record_amount
            && params.amount <= ctx.accounts.apc_state.deferred_burn_amount
            && params.amount <= ctx.accounts.deferred_burn_vault.amount,
        PeraxError::DeferredBurnNotExecutable
    );

    let bump = [ctx
        .accounts
        .counterweight_config
        .deferred_burn_authority_bump];
    let apc_config_key = ctx.accounts.apc_config.key();
    let seeds: &[&[u8]] = &[b"deferred-burn-authority", apc_config_key.as_ref(), &bump];
    let signer: &[&[&[u8]]] = &[seeds];
    let burn_accounts = Burn {
        mint: ctx.accounts.token_mint.to_account_info(),
        from: ctx.accounts.deferred_burn_vault.to_account_info(),
        authority: ctx.accounts.deferred_burn_authority.to_account_info(),
    };
    token::burn(
        CpiContext::new(ctx.accounts.token_program.to_account_info(), burn_accounts)
            .with_signer(signer),
        params.amount,
    )?;

    let amount_executed_after = ctx
        .accounts
        .deferred_burn_record
        .amount_executed
        .checked_add(params.amount)
        .ok_or(PeraxError::DeferredBurnNotExecutable)?;
    let remaining_deferred = ctx
        .accounts
        .apc_state
        .deferred_burn_amount
        .checked_sub(params.amount)
        .ok_or(PeraxError::DeferredBurnNotExecutable)?;
    let daily_burn_after = ctx
        .accounts
        .state
        .daily_burn_accumulator
        .checked_add(params.amount)
        .ok_or(PeraxError::DailyBurnCapExceeded)?;
    let deferred_window_after = ctx
        .accounts
        .apc_state
        .deferred_burn_window_executed
        .checked_add(params.amount)
        .ok_or(PeraxError::DeferredBurnWindowCapExceeded)?;

    ctx.accounts.deferred_burn_record.amount_executed = amount_executed_after;
    ctx.accounts.deferred_burn_record.last_executed_at = now;
    ctx.accounts.deferred_burn_record.is_complete =
        amount_executed_after == ctx.accounts.deferred_burn_record.amount;
    ctx.accounts.state.daily_burn_accumulator = daily_burn_after;
    ctx.accounts.apc_state.deferred_burn_amount = remaining_deferred;
    ctx.accounts.apc_state.deferred_burn_window_executed = deferred_window_after;
    ctx.accounts.apc_state.last_deferred_burn_timestamp = now;

    emit!(DeferredBurnExecuted {
        deferred_burn_record: ctx.accounts.deferred_burn_record.key(),
        decision_id: ctx.accounts.deferred_burn_record.decision_id,
        deferred_burn_vault: ctx.accounts.deferred_burn_vault.key(),
        amount: params.amount,
        amount_executed_after,
        remaining_deferred,
        executed_at: now,
    });

    Ok(())
}
