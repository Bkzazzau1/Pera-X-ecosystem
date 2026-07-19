use crate::{
    calculate_effective_apc_price, calculate_recovery_pex_out, reset_recovery_window_if_needed,
    validate_apc_observation_fresh, validate_recovery_purchase_limits, validate_reference,
    ApcRecoveryEntered, ApcStatus, ApcStatusChanged, CounterweightPurchaseExecuted,
    EnterApcRecovery, ExecuteCounterweightPurchase, ExecuteCounterweightPurchaseParams,
    InitializeRecoveryPool, InitializeRecoveryPoolParams, PeraxError, RecoveryPoolInitialized,
    RecoverySwapAdapter, RecoverySwapAdapterExecuted, RecoverySwapAdapterParams,
};
use anchor_lang::prelude::*;
use anchor_lang::solana_program::{
    instruction::{AccountMeta, Instruction},
    program::invoke_signed,
};
use anchor_spl::token::{self, TransferChecked};

pub fn initialize_recovery_pool(
    ctx: Context<InitializeRecoveryPool>,
    params: InitializeRecoveryPoolParams,
) -> Result<()> {
    validate_reference(params.pool_id)?;
    require_keys_eq!(
        ctx.accounts.pex_mint.key(),
        ctx.accounts.state.token_mint,
        PeraxError::InvalidTokenMint
    );
    require!(params.fee_bps <= 1_000, PeraxError::InvalidRecoveryPool);
    require!(
        ctx.accounts.quote_mint.key() != ctx.accounts.pex_mint.key(),
        PeraxError::InvalidRecoveryPool
    );

    let pool = &mut ctx.accounts.recovery_pool;
    pool.state = ctx.accounts.state.key();
    pool.pool_id = params.pool_id;
    pool.quote_mint = ctx.accounts.quote_mint.key();
    pool.pex_mint = ctx.accounts.pex_mint.key();
    pool.pool_authority = ctx.accounts.pool_authority.key();
    pool.pool_quote_vault = ctx.accounts.pool_quote_vault.key();
    pool.pool_pex_vault = ctx.accounts.pool_pex_vault.key();
    pool.fee_bps = params.fee_bps;
    pool.is_active = true;
    pool.bump = ctx.bumps.recovery_pool;
    pool.authority_bump = ctx.bumps.pool_authority;

    emit!(RecoveryPoolInitialized {
        recovery_pool: pool.key(),
        pool_id: pool.pool_id,
        quote_mint: pool.quote_mint,
        pex_mint: pool.pex_mint,
        pool_quote_vault: pool.pool_quote_vault,
        pool_pex_vault: pool.pool_pex_vault,
        fee_bps: pool.fee_bps,
        initialized_at: Clock::get()?.unix_timestamp,
    });
    Ok(())
}

pub fn execute_recovery_swap_adapter(
    ctx: Context<RecoverySwapAdapter>,
    params: RecoverySwapAdapterParams,
) -> Result<()> {
    require!(params.quote_amount > 0, PeraxError::InvalidAmount);
    require!(
        ctx.accounts.pool_quote_vault.amount > 0 && ctx.accounts.pool_pex_vault.amount > 0,
        PeraxError::InvalidRecoveryPool
    );

    let pex_out = calculate_recovery_pex_out(
        ctx.accounts.pool_quote_vault.amount,
        ctx.accounts.pool_pex_vault.amount,
        params.quote_amount,
        ctx.accounts.recovery_pool.fee_bps,
    )?;
    require!(
        pex_out > 0 && pex_out >= params.minimum_pex_out,
        PeraxError::InvalidRecoverySettlement
    );

    token::transfer_checked(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            TransferChecked {
                mint: ctx.accounts.quote_mint.to_account_info(),
                from: ctx.accounts.counterweight_vault.to_account_info(),
                to: ctx.accounts.pool_quote_vault.to_account_info(),
                authority: ctx.accounts.counterweight_authority.to_account_info(),
            },
        ),
        params.quote_amount,
        ctx.accounts.quote_mint.decimals,
    )?;

    let bump = [ctx.accounts.recovery_pool.authority_bump];
    let pool_key = ctx.accounts.recovery_pool.key();
    let seeds: &[&[u8]] = &[b"recovery-pool-authority", pool_key.as_ref(), &bump];
    token::transfer_checked(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            TransferChecked {
                mint: ctx.accounts.pex_mint.to_account_info(),
                from: ctx.accounts.pool_pex_vault.to_account_info(),
                to: ctx.accounts.recovery_vault.to_account_info(),
                authority: ctx.accounts.pool_authority.to_account_info(),
            },
        )
        .with_signer(&[seeds]),
        pex_out,
        ctx.accounts.pex_mint.decimals,
    )?;

    emit!(RecoverySwapAdapterExecuted {
        recovery_pool: ctx.accounts.recovery_pool.key(),
        quote_source: ctx.accounts.counterweight_vault.key(),
        pex_destination: ctx.accounts.recovery_vault.key(),
        quote_amount: params.quote_amount,
        pex_amount: pex_out,
        executed_at: Clock::get()?.unix_timestamp,
    });
    Ok(())
}

pub fn enter_apc_recovery(ctx: Context<EnterApcRecovery>) -> Result<()> {
    require!(!ctx.accounts.state.is_paused, PeraxError::ProgramPaused);
    require!(ctx.accounts.apc_config.is_active, PeraxError::ApcInactive);
    require!(!ctx.accounts.apc_config.is_paused, PeraxError::ApcPaused);
    require!(
        ctx.accounts.apc_state.current_band_index > 0,
        PeraxError::InvalidApcStatus
    );

    let now = Clock::get()?.unix_timestamp;
    validate_apc_observation_fresh(&ctx.accounts.apc_config, &ctx.accounts.observation, now)?;
    require!(
        !ctx.accounts.observation.is_consumed_for_release
            && !ctx.accounts.observation.is_consumed_for_confirmation
            && !ctx.accounts.observation.is_consumed_for_recovery,
        PeraxError::ObservationAlreadyUsed
    );
    let observed_price = calculate_effective_apc_price(
        ctx.accounts.observation.spot_price,
        ctx.accounts.observation.twap_price,
    )?;
    require!(
        observed_price < ctx.accounts.apc_state.current_reference_price,
        PeraxError::InvalidApcStatus
    );

    let previous_status = ctx.accounts.apc_state.status;
    ctx.accounts.apc_state.status = ApcStatus::Recovery;
    ctx.accounts.apc_state.recovery_entry_observation_id = ctx.accounts.observation.observation_id;
    ctx.accounts.observation.is_consumed_for_recovery = true;
    ctx.accounts.observation.consumed_by_recovery = ctx.accounts.apc_state.key();

    emit!(ApcRecoveryEntered {
        apc_state: ctx.accounts.apc_state.key(),
        observation_id: ctx.accounts.observation.observation_id,
        reference_price: ctx.accounts.apc_state.current_reference_price,
        observed_price,
        entered_at: now,
    });
    if previous_status != ApcStatus::Recovery {
        emit!(ApcStatusChanged {
            apc_state: ctx.accounts.apc_state.key(),
            previous_status,
            new_status: ApcStatus::Recovery,
            actor: ctx.accounts.oracle_feed.key(),
            changed_at: now,
        });
    }
    Ok(())
}

pub fn execute_counterweight_purchase<'info>(
    ctx: Context<'_, '_, '_, 'info, ExecuteCounterweightPurchase<'info>>,
    params: ExecuteCounterweightPurchaseParams,
) -> Result<()> {
    validate_reference(params.recovery_id)?;
    validate_reference(params.observation_id)?;
    require!(
        params.maximum_quote_amount > 0 && params.minimum_pex_out > 0,
        PeraxError::InvalidAmount
    );
    require!(
        !params.swap_instruction_data.is_empty(),
        PeraxError::InvalidRecoverySettlement
    );
    require!(!ctx.accounts.state.is_paused, PeraxError::ProgramPaused);
    require!(!ctx.accounts.apc_config.is_paused, PeraxError::ApcPaused);
    require!(
        ctx.accounts.apc_state.status == ApcStatus::Recovery,
        PeraxError::RecoveryNotActive
    );
    require!(
        ctx.accounts.counterweight_config.approved_recovery_program
            == ctx.accounts.recovery_program.key(),
        PeraxError::InvalidRecoveryProgram
    );
    require!(
        ctx.accounts.counterweight_config.approved_pool == ctx.accounts.approved_pool.key(),
        PeraxError::InvalidApcPool
    );

    let now = Clock::get()?.unix_timestamp;
    validate_apc_observation_fresh(&ctx.accounts.apc_config, &ctx.accounts.observation, now)?;
    require!(
        ctx.accounts.observation.observation_id
            != ctx.accounts.apc_state.recovery_entry_observation_id
            && !ctx.accounts.observation.is_consumed_for_release
            && !ctx.accounts.observation.is_consumed_for_confirmation
            && !ctx.accounts.observation.is_consumed_for_recovery,
        PeraxError::ObservationAlreadyUsed
    );
    let observed_price = calculate_effective_apc_price(
        ctx.accounts.observation.spot_price,
        ctx.accounts.observation.twap_price,
    )?;
    require!(
        observed_price < ctx.accounts.apc_state.current_reference_price,
        PeraxError::RecoveryNotActive
    );

    reset_recovery_window_if_needed(&ctx.accounts.apc_config, &mut ctx.accounts.apc_state, now);
    let tracked_available = ctx
        .accounts
        .apc_state
        .total_counterweight_credited
        .checked_sub(ctx.accounts.apc_state.total_counterweight_spent)
        .ok_or(PeraxError::InvalidCounterweightVault)?;
    validate_recovery_purchase_limits(
        &ctx.accounts.apc_config,
        &ctx.accounts.apc_state,
        params.maximum_quote_amount,
        tracked_available,
        ctx.accounts.counterweight_vault.amount,
        now,
    )?;

    let quote_before = ctx.accounts.counterweight_vault.amount;
    let pex_before = ctx.accounts.recovery_vault.amount;

    let mut metas = vec![
        AccountMeta::new(ctx.accounts.counterweight_vault.key(), false),
        AccountMeta::new(ctx.accounts.recovery_vault.key(), false),
        AccountMeta::new_readonly(ctx.accounts.counterweight_authority.key(), true),
        AccountMeta::new(ctx.accounts.approved_pool.key(), false),
        AccountMeta::new_readonly(ctx.accounts.token_program.key(), false),
    ];
    let mut infos = vec![
        ctx.accounts.counterweight_vault.to_account_info(),
        ctx.accounts.recovery_vault.to_account_info(),
        ctx.accounts.counterweight_authority.to_account_info(),
        ctx.accounts.approved_pool.to_account_info(),
        ctx.accounts.token_program.to_account_info(),
    ];
    for account in ctx.remaining_accounts {
        let meta = if account.is_writable {
            AccountMeta::new(account.key(), account.is_signer)
        } else {
            AccountMeta::new_readonly(account.key(), account.is_signer)
        };
        metas.push(meta);
        infos.push(account.clone());
    }
    infos.push(ctx.accounts.recovery_program.to_account_info());

    let instruction = Instruction {
        program_id: ctx.accounts.recovery_program.key(),
        accounts: metas,
        data: params.swap_instruction_data,
    };
    let bump = [ctx
        .accounts
        .counterweight_config
        .counterweight_authority_bump];
    let apc_config_key = ctx.accounts.apc_config.key();
    let signer_seeds: &[&[u8]] = &[b"counterweight-authority", apc_config_key.as_ref(), &bump];
    invoke_signed(&instruction, &infos, &[signer_seeds])?;

    ctx.accounts.counterweight_vault.reload()?;
    ctx.accounts.recovery_vault.reload()?;
    let quote_spent = quote_before
        .checked_sub(ctx.accounts.counterweight_vault.amount)
        .ok_or(PeraxError::InvalidRecoverySettlement)?;
    let pex_received = ctx
        .accounts
        .recovery_vault
        .amount
        .checked_sub(pex_before)
        .ok_or(PeraxError::InvalidRecoverySettlement)?;
    require!(
        quote_spent > 0
            && quote_spent <= params.maximum_quote_amount
            && pex_received >= params.minimum_pex_out,
        PeraxError::InvalidRecoverySettlement
    );

    ctx.accounts.apc_state.total_counterweight_spent = ctx
        .accounts
        .apc_state
        .total_counterweight_spent
        .checked_add(quote_spent)
        .ok_or(PeraxError::RecoveryCapExceeded)?;
    require!(
        ctx.accounts.apc_state.total_counterweight_spent
            <= ctx.accounts.apc_config.recovery_spending_cap,
        PeraxError::RecoveryCapExceeded
    );
    ctx.accounts.apc_state.recovery_window_spent = ctx
        .accounts
        .apc_state
        .recovery_window_spent
        .checked_add(quote_spent)
        .ok_or(PeraxError::RecoveryWindowCapExceeded)?;
    require!(
        ctx.accounts.apc_state.recovery_window_spent <= ctx.accounts.apc_config.recovery_window_cap,
        PeraxError::RecoveryWindowCapExceeded
    );
    ctx.accounts.apc_state.last_recovery_purchase_timestamp = now;

    let record = &mut ctx.accounts.recovery_record;
    record.recovery_id = params.recovery_id;
    record.observation_id = params.observation_id;
    record.apc_state = ctx.accounts.apc_state.key();
    record.counterweight_config = ctx.accounts.counterweight_config.key();
    record.quote_spent = quote_spent;
    record.pex_received = pex_received;
    record.executed_at = now;
    record.bump = ctx.bumps.recovery_record;
    ctx.accounts.observation.is_consumed_for_recovery = true;
    ctx.accounts.observation.consumed_by_recovery = record.key();

    emit!(CounterweightPurchaseExecuted {
        recovery_record: record.key(),
        recovery_id: record.recovery_id,
        observation_id: record.observation_id,
        quote_spent,
        pex_received,
        recovery_vault: ctx.accounts.recovery_vault.key(),
        executed_at: now,
    });

    Ok(())
}
