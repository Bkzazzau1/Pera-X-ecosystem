use super::market_cpi::{validated_exact_out_market_metas, ExactOutMarketValidation};
use super::settlement_v2::calculate_settlement_quote_requirement;
use crate::{
    calculate_effective_apc_price, reset_recovery_window_if_needed, validate_apc_observation_fresh,
    validate_recovery_purchase_limits, validate_reference, ApcStatus,
    CounterweightPurchaseExecuted, ExecuteCounterweightPurchase,
    ExecuteCounterweightPurchaseParams, ExecuteSettlementMarketPurchaseParams,
    ExecuteSettlementMarketPurchaseV2, PeraxError, SettlementError, SettlementMarketMode,
    SettlementMarketPurchaseExecuted, SettlementPolicy, SettlementRecord, SettlementStatus,
    APC_BPS_DENOMINATOR, APC_QUOTE_DECIMALS, PEX_DECIMALS,
};
use anchor_lang::prelude::*;
use anchor_lang::solana_program::{
    instruction::Instruction,
    program::{invoke, invoke_signed},
};

pub fn execute_settlement_market_purchase_hardened<'info>(
    ctx: Context<'_, '_, '_, 'info, ExecuteSettlementMarketPurchaseV2<'info>>,
    params: ExecuteSettlementMarketPurchaseParams,
) -> Result<()> {
    require!(!ctx.accounts.state.is_paused, PeraxError::ProgramPaused);
    require!(
        ctx.accounts.settlement_policy.is_active,
        SettlementError::PolicyInactive
    );
    require!(
        matches!(
            ctx.accounts.settlement_record.market_mode,
            SettlementMarketMode::MarketPurchase | SettlementMarketMode::Hybrid
        ),
        SettlementError::InvalidSettlementMode
    );
    require!(
        matches!(
            ctx.accounts.settlement_record.status,
            SettlementStatus::Planned | SettlementStatus::Funding
        ),
        SettlementError::InvalidSettlementStatus
    );
    require!(
        params.maximum_quote_amount > 0
            && params.minimum_pex_out > 0
            && !params.swap_instruction_data.is_empty(),
        SettlementError::InvalidMarketSettlement
    );

    let now = Clock::get()?.unix_timestamp;
    validate_apc_observation_fresh(&ctx.accounts.apc_config, &ctx.accounts.observation, now)?;
    let effective_price = calculate_effective_apc_price(
        ctx.accounts.observation.spot_price,
        ctx.accounts.observation.twap_price,
    )?;
    require!(
        effective_price == ctx.accounts.settlement_record.effective_price,
        SettlementError::InvalidMarketSettlement
    );

    let market_remaining = ctx
        .accounts
        .settlement_record
        .market_pex_required
        .checked_sub(ctx.accounts.settlement_record.market_pex_received)
        .ok_or(SettlementError::SettlementArithmeticError)?;
    require!(
        market_remaining > 0,
        SettlementError::InvalidSettlementStatus
    );
    // The market instruction is exact-out. The caller may not request excess
    // output and use the settlement vault as an unbounded market accumulator.
    require!(
        params.minimum_pex_out == market_remaining,
        SettlementError::InvalidMarketSettlement
    );

    let expected_quote = calculate_settlement_quote_requirement(
        market_remaining,
        effective_price,
        ctx.accounts.apc_config.price_scale,
    )?;
    let maximum_allowed_quote = amount_with_bps_ceiling(
        expected_quote,
        ctx.accounts.settlement_policy.maximum_market_slippage_bps,
    )?;
    require!(
        params.maximum_quote_amount <= maximum_allowed_quote,
        SettlementError::InvalidMarketSettlement
    );

    reset_settlement_daily_window(&mut ctx.accounts.settlement_policy, now);
    require_checked_daily_cap(
        ctx.accounts.settlement_policy.daily_market_quote_spent,
        params.maximum_quote_amount,
        ctx.accounts.settlement_policy.daily_market_quote_cap,
    )?;

    let metas = validated_exact_out_market_metas(
        ctx.remaining_accounts,
        &params.swap_instruction_data,
        ExactOutMarketValidation {
            market_program: ctx.accounts.market_program.key(),
            approved_pool: ctx.accounts.approved_market_pool.key(),
            quote_source: ctx.accounts.quote_source_token_account.key(),
            pex_destination: ctx.accounts.settlement_pex_vault.key(),
            authority: ctx.accounts.quote_source_authority.key(),
            quote_mint: ctx.accounts.quote_mint.key(),
            pex_mint: ctx.accounts.pex_mint.key(),
            token_program: ctx.accounts.token_program.key(),
            maximum_quote_amount: params.maximum_quote_amount,
            exact_pex_out: market_remaining,
            authority_is_pda: false,
        },
    )
    .ok_or_else(|| error!(SettlementError::InvalidMarketSettlement))?;

    let quote_before = ctx.accounts.quote_source_token_account.amount;
    let pex_before = ctx.accounts.settlement_pex_vault.amount;
    let mut infos: Vec<AccountInfo<'info>> = ctx.remaining_accounts.to_vec();
    infos.push(ctx.accounts.market_program.to_account_info());
    invoke(
        &Instruction {
            program_id: ctx.accounts.market_program.key(),
            accounts: metas,
            data: params.swap_instruction_data,
        },
        &infos,
    )?;

    ctx.accounts.quote_source_token_account.reload()?;
    ctx.accounts.settlement_pex_vault.reload()?;
    let quote_spent = quote_before
        .checked_sub(ctx.accounts.quote_source_token_account.amount)
        .ok_or(SettlementError::InvalidMarketSettlement)?;
    let pex_received = ctx
        .accounts
        .settlement_pex_vault
        .amount
        .checked_sub(pex_before)
        .ok_or(SettlementError::InvalidMarketSettlement)?;
    require!(
        quote_spent > 0
            && quote_spent <= params.maximum_quote_amount
            && pex_received >= market_remaining,
        SettlementError::InvalidMarketSettlement
    );
    require_checked_daily_cap(
        ctx.accounts.settlement_policy.daily_market_quote_spent,
        quote_spent,
        ctx.accounts.settlement_policy.daily_market_quote_cap,
    )?;
    require_checked_daily_cap(
        ctx.accounts.settlement_policy.daily_market_pex_received,
        pex_received,
        ctx.accounts.settlement_policy.daily_market_pex_cap,
    )?;

    ctx.accounts.settlement_policy.daily_market_quote_spent = ctx
        .accounts
        .settlement_policy
        .daily_market_quote_spent
        .checked_add(quote_spent)
        .ok_or(SettlementError::SettlementArithmeticError)?;
    ctx.accounts.settlement_policy.daily_market_pex_received = ctx
        .accounts
        .settlement_policy
        .daily_market_pex_received
        .checked_add(pex_received)
        .ok_or(SettlementError::SettlementArithmeticError)?;

    let record_key = ctx.accounts.settlement_record.key();
    let source_key = ctx.accounts.quote_source_token_account.key();
    let record = &mut ctx.accounts.settlement_record;
    record.market_quote_spent = record
        .market_quote_spent
        .checked_add(quote_spent)
        .ok_or(SettlementError::SettlementArithmeticError)?;
    record.market_pex_received = record
        .market_pex_received
        .checked_add(pex_received)
        .ok_or(SettlementError::SettlementArithmeticError)?;
    record.funding_source_token_account =
        set_or_validate_funding_source(record.funding_source_token_account, source_key)?;
    refresh_settlement_status(record)?;

    emit!(SettlementMarketPurchaseExecuted {
        settlement_record: record_key,
        quote_source_token_account: source_key,
        quote_spent,
        pex_received,
        executed_at: now,
    });
    Ok(())
}

pub fn execute_counterweight_purchase_hardened<'info>(
    ctx: Context<'_, '_, '_, 'info, ExecuteCounterweightPurchase<'info>>,
    params: ExecuteCounterweightPurchaseParams,
) -> Result<()> {
    validate_reference(params.recovery_id)?;
    require_keys_eq!(
        ctx.accounts.state.token_mint,
        ctx.accounts.pex_mint.key(),
        PeraxError::InvalidTokenMint
    );
    require_keys_eq!(
        ctx.accounts.apc_config.oracle_feed,
        ctx.accounts.oracle_feed.key(),
        PeraxError::Unauthorized
    );
    require!(
        ctx.accounts.observation.observation_id == params.observation_id
            && ctx.accounts.observation.oracle_feed == ctx.accounts.oracle_feed.key(),
        PeraxError::InvalidReference
    );
    require!(
        ctx.accounts.counterweight_vault.key()
            == ctx.accounts.counterweight_config.counterweight_vault
            && ctx.accounts.counterweight_vault.owner == ctx.accounts.counterweight_authority.key()
            && ctx.accounts.counterweight_vault.mint == ctx.accounts.quote_mint.key(),
        PeraxError::InvalidCounterweightVault
    );
    require!(
        ctx.accounts.recovery_vault.key() == ctx.accounts.counterweight_config.recovery_vault
            && ctx.accounts.recovery_vault.owner
                == ctx.accounts.counterweight_config.recovery_authority
            && ctx.accounts.recovery_vault.mint == ctx.accounts.pex_mint.key(),
        PeraxError::InvalidCounterweightVault
    );
    require_keys_eq!(
        ctx.accounts.quote_mint.key(),
        ctx.accounts.counterweight_config.quote_mint,
        PeraxError::InvalidCounterweightMint
    );
    require_keys_eq!(
        ctx.accounts.approved_pool.key(),
        ctx.accounts.apc_config.approved_pool,
        PeraxError::InvalidApcPool
    );
    require_keys_eq!(
        ctx.accounts.recovery_program.key(),
        ctx.accounts.apc_config.approved_recovery_program,
        PeraxError::InvalidRecoveryProgram
    );
    require!(
        ctx.accounts.recovery_program.to_account_info().executable,
        PeraxError::InvalidRecoveryProgram
    );
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

    let (policy_info, market_accounts) = ctx
        .remaining_accounts
        .split_first()
        .ok_or(PeraxError::InvalidRecoverySettlement)?;
    let settlement_policy = load_recovery_market_policy(
        policy_info,
        ctx.accounts.state.key(),
        ctx.accounts.apc_config.key(),
        ctx.accounts.counterweight_config.key(),
        ctx.accounts.quote_mint.key(),
        ctx.accounts.pex_mint.key(),
        ctx.accounts.recovery_program.key(),
        ctx.accounts.approved_pool.key(),
    )?;

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

    let policy_minimum_pex_out = minimum_pex_out_for_quote(
        params.maximum_quote_amount,
        observed_price,
        ctx.accounts.apc_config.price_scale,
        settlement_policy.maximum_market_slippage_bps,
    )?;
    require!(
        params.minimum_pex_out >= policy_minimum_pex_out,
        PeraxError::InvalidRecoverySettlement
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

    let metas = validated_exact_out_market_metas(
        market_accounts,
        &params.swap_instruction_data,
        ExactOutMarketValidation {
            market_program: ctx.accounts.recovery_program.key(),
            approved_pool: ctx.accounts.approved_pool.key(),
            quote_source: ctx.accounts.counterweight_vault.key(),
            pex_destination: ctx.accounts.recovery_vault.key(),
            authority: ctx.accounts.counterweight_authority.key(),
            quote_mint: ctx.accounts.quote_mint.key(),
            pex_mint: ctx.accounts.pex_mint.key(),
            token_program: ctx.accounts.token_program.key(),
            maximum_quote_amount: params.maximum_quote_amount,
            exact_pex_out: params.minimum_pex_out,
            authority_is_pda: true,
        },
    )
    .ok_or_else(|| error!(PeraxError::InvalidRecoverySettlement))?;

    let quote_before = ctx.accounts.counterweight_vault.amount;
    let pex_before = ctx.accounts.recovery_vault.amount;
    let mut infos: Vec<AccountInfo<'info>> = market_accounts.to_vec();
    infos.push(ctx.accounts.recovery_program.to_account_info());
    let bump = [ctx
        .accounts
        .counterweight_config
        .counterweight_authority_bump];
    let apc_config_key = ctx.accounts.apc_config.key();
    let signer_seeds: &[&[u8]] = &[b"counterweight-authority", apc_config_key.as_ref(), &bump];
    invoke_signed(
        &Instruction {
            program_id: ctx.accounts.recovery_program.key(),
            accounts: metas,
            data: params.swap_instruction_data,
        },
        &infos,
        &[signer_seeds],
    )?;

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

#[allow(clippy::too_many_arguments)]
fn load_recovery_market_policy(
    info: &AccountInfo<'_>,
    state: Pubkey,
    apc_config: Pubkey,
    counterweight_config: Pubkey,
    quote_mint: Pubkey,
    pex_mint: Pubkey,
    market_program: Pubkey,
    market_pool: Pubkey,
) -> Result<SettlementPolicy> {
    require!(
        info.owner == &crate::ID && !info.is_signer && !info.is_writable,
        PeraxError::InvalidRecoverySettlement
    );
    let expected =
        Pubkey::find_program_address(&[b"settlement-policy", state.as_ref()], &crate::ID).0;
    require!(*info.key == expected, PeraxError::InvalidRecoverySettlement);
    let data = info.try_borrow_data()?;
    let mut data_slice: &[u8] = &data;
    let policy = SettlementPolicy::try_deserialize(&mut data_slice)
        .map_err(|_| error!(PeraxError::InvalidRecoverySettlement))?;
    require!(
        policy.is_active
            && policy.state == state
            && policy.apc_config == apc_config
            && policy.counterweight_config == counterweight_config
            && policy.quote_mint == quote_mint
            && policy.pex_mint == pex_mint
            && policy.approved_market_program == market_program
            && policy.approved_market_pool == market_pool
            && policy.maximum_market_slippage_bps > 0
            && policy.maximum_market_slippage_bps < 10_000,
        PeraxError::InvalidRecoverySettlement
    );
    Ok(policy)
}

fn minimum_pex_out_for_quote(
    maximum_quote_amount: u64,
    effective_price: u64,
    price_scale: u64,
    maximum_slippage_bps: u16,
) -> Result<u64> {
    require!(
        maximum_quote_amount > 0
            && effective_price > 0
            && price_scale > 0
            && maximum_slippage_bps > 0
            && maximum_slippage_bps < 10_000,
        PeraxError::InvalidRecoverySettlement
    );
    let quote_scale = 10u128
        .checked_pow(u32::from(APC_QUOTE_DECIMALS))
        .ok_or(PeraxError::InvalidRecoverySettlement)?;
    let numerator = u128::from(maximum_quote_amount)
        .checked_mul(u128::from(PEX_DECIMALS))
        .and_then(|value| value.checked_mul(u128::from(price_scale)))
        .ok_or(PeraxError::InvalidRecoverySettlement)?;
    let denominator = quote_scale
        .checked_mul(u128::from(effective_price))
        .ok_or(PeraxError::InvalidRecoverySettlement)?;
    let fair_output = ceil_div_u128(
        numerator,
        denominator,
        PeraxError::InvalidRecoverySettlement,
    )?;
    let retained_bps = APC_BPS_DENOMINATOR
        .checked_sub(u128::from(maximum_slippage_bps))
        .ok_or(PeraxError::InvalidRecoverySettlement)?;
    let minimum = u128::from(fair_output)
        .checked_mul(retained_bps)
        .and_then(|value| value.checked_div(APC_BPS_DENOMINATOR))
        .ok_or(PeraxError::InvalidRecoverySettlement)?;
    let minimum = u64::try_from(minimum).map_err(|_| PeraxError::InvalidRecoverySettlement)?;
    require!(minimum > 0, PeraxError::InvalidRecoverySettlement);
    Ok(minimum)
}

fn amount_with_bps_ceiling(amount: u64, additional_bps: u16) -> Result<u64> {
    let multiplier = APC_BPS_DENOMINATOR
        .checked_add(u128::from(additional_bps))
        .ok_or(SettlementError::SettlementArithmeticError)?;
    let numerator = u128::from(amount)
        .checked_mul(multiplier)
        .ok_or(SettlementError::SettlementArithmeticError)?;
    ceil_div_u128(
        numerator,
        APC_BPS_DENOMINATOR,
        SettlementError::SettlementArithmeticError,
    )
}

fn ceil_div_u128<E>(numerator: u128, denominator: u128, error: E) -> Result<u64>
where
    E: Into<anchor_lang::error::Error> + Copy,
{
    if denominator == 0 {
        return Err(error.into());
    }
    let value = numerator
        .checked_add(denominator - 1)
        .and_then(|value| value.checked_div(denominator))
        .ok_or_else(|| error.into())?;
    let value = u64::try_from(value).map_err(|_| error.into())?;
    if value == 0 {
        return Err(error.into());
    }
    Ok(value)
}

fn total_settlement_pex_received(record: &SettlementRecord) -> Result<u64> {
    record
        .direct_pex_received
        .checked_add(record.market_pex_received)
        .and_then(|value| value.checked_add(record.policy_vault_pex_received))
        .ok_or(SettlementError::SettlementArithmeticError.into())
}

fn refresh_settlement_status(record: &mut SettlementRecord) -> Result<()> {
    let received = total_settlement_pex_received(record)?;
    record.status = if received >= record.pex_obligation {
        SettlementStatus::Ready
    } else if received > 0 {
        SettlementStatus::Funding
    } else {
        SettlementStatus::Planned
    };
    Ok(())
}

fn set_or_validate_funding_source(current: Pubkey, supplied: Pubkey) -> Result<Pubkey> {
    if current == Pubkey::default() {
        Ok(supplied)
    } else {
        require!(
            current == supplied,
            SettlementError::InvalidMarketSettlement
        );
        Ok(current)
    }
}

fn reset_settlement_daily_window(policy: &mut SettlementPolicy, now: i64) {
    if policy.daily_window_started_at == 0
        || now >= policy.daily_window_started_at.saturating_add(86_400)
    {
        policy.daily_window_started_at = now;
        policy.daily_market_quote_spent = 0;
        policy.daily_market_pex_received = 0;
        policy.daily_policy_vault_pex_released = 0;
    }
}

fn require_checked_daily_cap(current: u64, requested: u64, cap: u64) -> Result<()> {
    let after = current
        .checked_add(requested)
        .ok_or(SettlementError::SettlementArithmeticError)?;
    require!(after <= cap, SettlementError::SettlementDailyCapExceeded);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recovery_minimum_output_respects_policy_slippage() {
        assert_eq!(
            minimum_pex_out_for_quote(1_000_000, 100_000_000, 100_000_000, 500).unwrap(),
            950_000
        );
    }

    #[test]
    fn recovery_minimum_output_rejects_unbounded_slippage() {
        assert!(minimum_pex_out_for_quote(1_000_000, 100_000_000, 100_000_000, 0).is_err());
        assert!(minimum_pex_out_for_quote(1_000_000, 100_000_000, 100_000_000, 10_000).is_err());
    }
}
