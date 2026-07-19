use crate::{
    calculate_apc_risk_tier, calculate_effective_apc_price, calculate_vault_available_amount,
    validate_apc_observation_fresh, validate_reference, ApcStatus, DirectPexSettlementFunded,
    ExecuteSettlementMarketPurchaseParams, ExecuteSettlementMarketPurchaseV2,
    ExecuteSettlementVaultFundingParams, ExecuteSettlementVaultFundingV2, FinalizeSettlementParams,
    FinalizeSettlementV2, FundDirectPexSettlementParams, FundDirectPexSettlementV2,
    InitializeProductSettlementPolicy, InitializeProductSettlementPolicyParams,
    InitializeSettlementPolicyParams, InitializeSettlementPolicyV2, PlanSettlementParams,
    PlanSettlementV2, ProductSettlementPolicyInitialized, ProductSettlementPolicyUpdated,
    SettlementDisposition, SettlementError, SettlementFinalized, SettlementFundingMethod,
    SettlementMarketMode, SettlementMarketPurchaseExecuted, SettlementPlanned,
    SettlementPolicyInitialized, SettlementPolicyVaultFunded, SettlementRecord, SettlementStatus,
    UpdateProductSettlementPolicy, UpdateProductSettlementPolicyParams, APC_BPS_DENOMINATOR,
    APC_QUOTE_DECIMALS, PEX_DECIMALS, SETTLEMENT_ALL_FUNDING_METHODS, SETTLEMENT_FUNDING_FIAT,
    SETTLEMENT_FUNDING_PEX, SETTLEMENT_FUNDING_STABLECOIN, SETTLEMENT_FUNDING_VIRTUAL_ACCOUNT,
};
use anchor_lang::prelude::*;
use anchor_lang::solana_program::{
    instruction::{AccountMeta, Instruction},
    program::invoke,
};
use anchor_spl::token::{self, Burn, TransferChecked};

pub fn initialize_settlement_policy(
    ctx: Context<InitializeSettlementPolicyV2>,
    params: InitializeSettlementPolicyParams,
) -> Result<()> {
    validate_settlement_policy_params(&params)?;
    require!(
        !ctx.accounts.state.is_paused,
        crate::PeraxError::ProgramPaused
    );
    require!(
        ctx.accounts.apc_config.is_active && !ctx.accounts.apc_config.is_paused,
        SettlementError::InvalidPolicy
    );
    require!(
        ctx.accounts.counterweight_config.apc_config == ctx.accounts.apc_config.key(),
        SettlementError::InvalidPolicy
    );
    require!(
        ctx.accounts.approved_policy_vault_config.is_active
            && !ctx.accounts.approved_policy_vault_config.is_paused,
        SettlementError::InvalidPolicy
    );

    let now = Clock::get()?.unix_timestamp;
    let policy_key = ctx.accounts.settlement_policy.key();
    let policy = &mut ctx.accounts.settlement_policy;
    policy.state = ctx.accounts.state.key();
    policy.apc_config = ctx.accounts.apc_config.key();
    policy.counterweight_config = ctx.accounts.counterweight_config.key();
    policy.quote_mint = ctx.accounts.quote_mint.key();
    policy.pex_mint = ctx.accounts.pex_mint.key();
    policy.approved_market_program = ctx.accounts.apc_config.approved_recovery_program;
    policy.approved_market_pool = ctx.accounts.apc_config.approved_pool;
    policy.approved_policy_vault_config = ctx.accounts.approved_policy_vault_config.key();
    policy.settlement_authority = Pubkey::default();
    policy.settlement_pex_vault = Pubkey::default();
    policy.lock_vault = ctx.accounts.lock_vault.key();
    policy.market_share_bps_by_risk = params.market_share_bps_by_risk;
    policy.maximum_market_slippage_bps = params.maximum_market_slippage_bps;
    policy.maximum_quantity_per_settlement = params.maximum_quantity_per_settlement;
    policy.daily_market_quote_cap = params.daily_market_quote_cap;
    policy.daily_market_pex_cap = params.daily_market_pex_cap;
    policy.daily_policy_vault_pex_cap = params.daily_policy_vault_pex_cap;
    policy.daily_window_started_at = now;
    policy.daily_market_quote_spent = 0;
    policy.daily_market_pex_received = 0;
    policy.daily_policy_vault_pex_released = 0;
    policy.is_active = true;
    policy.bump = ctx.bumps.settlement_policy;
    policy.settlement_authority_bump = 0;

    emit!(SettlementPolicyInitialized {
        settlement_policy: policy_key,
        approved_market_program: policy.approved_market_program,
        approved_market_pool: policy.approved_market_pool,
        approved_policy_vault_config: policy.approved_policy_vault_config,
        settlement_pex_vault: Pubkey::default(),
        lock_vault: policy.lock_vault,
        initialized_at: now,
    });
    Ok(())
}

pub fn initialize_product_settlement_policy(
    ctx: Context<InitializeProductSettlementPolicy>,
    params: InitializeProductSettlementPolicyParams,
) -> Result<()> {
    validate_reference(params.product_id)?;
    validate_product_policy_values(
        &ctx.accounts.settlement_policy,
        params.unit_quote_value,
        params.maximum_quantity,
        params.accepted_funding_mask,
        params.disposition,
        params.fixed_destination_token_account,
    )?;

    let now = Clock::get()?.unix_timestamp;
    let product_key = ctx.accounts.product_policy.key();
    let policy = &mut ctx.accounts.product_policy;
    policy.settlement_policy = ctx.accounts.settlement_policy.key();
    policy.product_id = params.product_id;
    policy.unit_quote_value = params.unit_quote_value;
    policy.maximum_quantity = params.maximum_quantity;
    policy.accepted_funding_mask = params.accepted_funding_mask;
    policy.disposition = params.disposition;
    policy.fixed_destination_token_account = params.fixed_destination_token_account;
    policy.is_active = true;
    policy.bump = ctx.bumps.product_policy;

    emit!(ProductSettlementPolicyInitialized {
        product_policy: product_key,
        product_id: policy.product_id,
        unit_quote_value: policy.unit_quote_value,
        maximum_quantity: policy.maximum_quantity,
        accepted_funding_mask: policy.accepted_funding_mask,
        disposition: policy.disposition,
        fixed_destination_token_account: policy.fixed_destination_token_account,
        initialized_at: now,
    });
    Ok(())
}

pub fn update_product_settlement_policy(
    ctx: Context<UpdateProductSettlementPolicy>,
    params: UpdateProductSettlementPolicyParams,
) -> Result<()> {
    let current = &ctx.accounts.product_policy;
    let unit_quote_value = params.unit_quote_value.unwrap_or(current.unit_quote_value);
    let maximum_quantity = params.maximum_quantity.unwrap_or(current.maximum_quantity);
    let accepted_funding_mask = params
        .accepted_funding_mask
        .unwrap_or(current.accepted_funding_mask);
    let disposition = params.disposition.unwrap_or(current.disposition);
    let fixed_destination = params
        .fixed_destination_token_account
        .unwrap_or(current.fixed_destination_token_account);

    validate_product_policy_values(
        &ctx.accounts.settlement_policy,
        unit_quote_value,
        maximum_quantity,
        accepted_funding_mask,
        disposition,
        fixed_destination,
    )?;

    let product_key = ctx.accounts.product_policy.key();
    let policy = &mut ctx.accounts.product_policy;
    policy.unit_quote_value = unit_quote_value;
    policy.maximum_quantity = maximum_quantity;
    policy.accepted_funding_mask = accepted_funding_mask;
    policy.disposition = disposition;
    policy.fixed_destination_token_account = fixed_destination;
    if let Some(is_active) = params.is_active {
        policy.is_active = is_active;
    }

    emit!(ProductSettlementPolicyUpdated {
        product_policy: product_key,
        product_id: policy.product_id,
        unit_quote_value: policy.unit_quote_value,
        maximum_quantity: policy.maximum_quantity,
        accepted_funding_mask: policy.accepted_funding_mask,
        disposition: policy.disposition,
        fixed_destination_token_account: policy.fixed_destination_token_account,
        is_active: policy.is_active,
        updated_at: Clock::get()?.unix_timestamp,
    });
    Ok(())
}

pub fn plan_settlement(ctx: Context<PlanSettlementV2>, params: PlanSettlementParams) -> Result<()> {
    validate_reference(params.settlement_id)?;
    validate_reference(params.product_id)?;
    validate_reference(params.observation_id)?;
    require_keys_eq!(
        ctx.accounts.settlement_policy.state,
        ctx.accounts.state.key(),
        SettlementError::InvalidPolicy
    );
    require!(
        ctx.accounts.product_policy.settlement_policy == ctx.accounts.settlement_policy.key()
            && ctx.accounts.product_policy.product_id == params.product_id,
        SettlementError::InvalidPolicy
    );
    require_keys_eq!(
        ctx.accounts.apc_config.key(),
        ctx.accounts.settlement_policy.apc_config,
        SettlementError::InvalidPolicy
    );
    require_keys_eq!(
        ctx.accounts.apc_state.config,
        ctx.accounts.apc_config.key(),
        crate::PeraxError::ApcNotInitialized
    );
    require!(
        ctx.accounts.observation.observation_id == params.observation_id
            && ctx.accounts.observation.oracle_feed == ctx.accounts.apc_config.oracle_feed,
        crate::PeraxError::InvalidReference
    );
    require_keys_eq!(
        ctx.accounts.pex_mint.key(),
        ctx.accounts.settlement_policy.pex_mint,
        crate::PeraxError::InvalidTokenMint
    );
    require!(
        !ctx.accounts.state.is_paused,
        crate::PeraxError::ProgramPaused
    );
    require!(
        ctx.accounts.settlement_policy.is_active,
        SettlementError::PolicyInactive
    );
    require!(
        ctx.accounts.product_policy.is_active,
        SettlementError::ProductInactive
    );
    require!(
        params.quantity > 0
            && params.quantity <= ctx.accounts.product_policy.maximum_quantity
            && params.quantity
                <= ctx
                    .accounts
                    .settlement_policy
                    .maximum_quantity_per_settlement,
        SettlementError::InvalidQuantity
    );
    require!(
        funding_method_allowed(
            ctx.accounts.product_policy.accepted_funding_mask,
            params.funding_method
        ),
        SettlementError::FundingMethodNotAccepted
    );
    require!(
        !ctx.accounts.apc_config.is_paused,
        crate::PeraxError::ApcPaused
    );

    let now = Clock::get()?.unix_timestamp;
    validate_apc_observation_fresh(&ctx.accounts.apc_config, &ctx.accounts.observation, now)?;
    require!(
        !ctx.accounts.observation.is_consumed_for_release
            && !ctx.accounts.observation.is_consumed_for_confirmation
            && !ctx.accounts.observation.is_consumed_for_recovery,
        crate::PeraxError::ObservationAlreadyUsed
    );

    let quote_value = ctx
        .accounts
        .product_policy
        .unit_quote_value
        .checked_mul(params.quantity)
        .ok_or(SettlementError::SettlementArithmeticError)?;
    let effective_price = calculate_effective_apc_price(
        ctx.accounts.observation.spot_price,
        ctx.accounts.observation.twap_price,
    )?;
    let pex_obligation = calculate_settlement_pex_obligation(
        quote_value,
        effective_price,
        ctx.accounts.apc_config.price_scale,
    )?;
    let risk_tier = calculate_apc_risk_tier(
        ctx.accounts.observation.price_velocity_bps,
        ctx.accounts.observation.volatility_bps,
        ctx.accounts.observation.estimated_price_impact_bps,
        ctx.accounts.apc_config.risk_velocity_thresholds_bps,
        ctx.accounts.apc_config.risk_volatility_thresholds_bps,
        ctx.accounts.apc_config.risk_price_impact_thresholds_bps,
    );
    let (market_mode, market_pex_required, policy_vault_pex_required) =
        derive_settlement_source_split(
            params.funding_method,
            ctx.accounts.apc_state.status,
            effective_price,
            ctx.accounts.apc_state.current_reference_price,
            risk_tier,
            ctx.accounts.settlement_policy.market_share_bps_by_risk,
            pex_obligation,
        )?;

    let destination_token_account = match ctx.accounts.product_policy.disposition {
        SettlementDisposition::UtilityPayment => {
            require!(
                ctx.accounts.product_policy.fixed_destination_token_account != Pubkey::default(),
                SettlementError::InvalidSettlementDestination
            );
            ctx.accounts.product_policy.fixed_destination_token_account
        }
        SettlementDisposition::Lock => ctx.accounts.settlement_policy.lock_vault,
        SettlementDisposition::CustomerDelivery | SettlementDisposition::Burn => Pubkey::default(),
    };

    let record_key = ctx.accounts.settlement_record.key();
    let vault_key = ctx.accounts.settlement_pex_vault.key();
    let authority_key = ctx.accounts.settlement_authority.key();

    let record = &mut ctx.accounts.settlement_record;
    record.settlement_id = params.settlement_id;
    record.settlement_policy = ctx.accounts.settlement_policy.key();
    record.product_policy = ctx.accounts.product_policy.key();
    record.product_id = params.product_id;
    record.initiator = ctx.accounts.initiator.key();
    record.beneficiary = params.beneficiary;
    record.funding_method = params.funding_method;
    record.market_mode = market_mode;
    record.disposition = ctx.accounts.product_policy.disposition;
    record.status = SettlementStatus::Planned;
    record.observation_id = params.observation_id;
    record.effective_price = effective_price;
    record.risk_tier = risk_tier;
    record.quantity = params.quantity;
    record.quote_value = quote_value;
    record.pex_obligation = pex_obligation;
    record.market_pex_required = market_pex_required;
    record.policy_vault_pex_required = policy_vault_pex_required;
    record.direct_pex_received = 0;
    record.market_quote_spent = 0;
    record.market_pex_received = 0;
    record.policy_vault_pex_received = 0;
    record.destination_token_account = destination_token_account;
    record.funding_source_token_account = Pubkey::default();
    record.created_at = now;
    record.finalized_at = 0;
    record.final_pex_amount = 0;
    record.surplus_locked = 0;
    record.bump = ctx.bumps.settlement_record;

    let custody = &mut ctx.accounts.settlement_custody;
    custody.settlement_record = record_key;
    custody.settlement_authority = authority_key;
    custody.settlement_pex_vault = vault_key;
    custody.authority_bump = ctx.bumps.settlement_authority;
    custody.bump = ctx.bumps.settlement_custody;

    emit!(SettlementPlanned {
        settlement_record: record_key,
        settlement_id: params.settlement_id,
        product_id: params.product_id,
        observation_id: params.observation_id,
        funding_method: params.funding_method,
        market_mode,
        disposition: ctx.accounts.product_policy.disposition,
        quote_value,
        pex_obligation,
        market_pex_required,
        policy_vault_pex_required,
        planned_at: now,
    });
    Ok(())
}

pub fn fund_direct_pex_settlement(
    ctx: Context<FundDirectPexSettlementV2>,
    params: FundDirectPexSettlementParams,
) -> Result<()> {
    require!(
        !ctx.accounts.state.is_paused,
        crate::PeraxError::ProgramPaused
    );
    require!(
        ctx.accounts.settlement_policy.is_active,
        SettlementError::PolicyInactive
    );
    require!(
        ctx.accounts.settlement_record.market_mode == SettlementMarketMode::DirectPex,
        SettlementError::InvalidSettlementMode
    );
    require!(
        matches!(
            ctx.accounts.settlement_record.status,
            SettlementStatus::Planned | SettlementStatus::Funding
        ),
        SettlementError::InvalidSettlementStatus
    );
    require!(params.amount > 0, crate::PeraxError::InvalidAmount);
    let remaining = ctx
        .accounts
        .settlement_record
        .pex_obligation
        .checked_sub(ctx.accounts.settlement_record.direct_pex_received)
        .ok_or(SettlementError::SettlementArithmeticError)?;
    require!(
        params.amount <= remaining,
        SettlementError::InvalidMarketSettlement
    );

    token::transfer_checked(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            TransferChecked {
                mint: ctx.accounts.pex_mint.to_account_info(),
                from: ctx.accounts.source_token_account.to_account_info(),
                to: ctx.accounts.settlement_pex_vault.to_account_info(),
                authority: ctx.accounts.source_authority.to_account_info(),
            },
        ),
        params.amount,
        ctx.accounts.pex_mint.decimals,
    )?;

    let record_key = ctx.accounts.settlement_record.key();
    let source_key = ctx.accounts.source_token_account.key();
    let record = &mut ctx.accounts.settlement_record;
    record.direct_pex_received = record
        .direct_pex_received
        .checked_add(params.amount)
        .ok_or(SettlementError::SettlementArithmeticError)?;
    record.funding_source_token_account =
        set_or_validate_funding_source(record.funding_source_token_account, source_key)?;
    refresh_settlement_status(record)?;

    emit!(DirectPexSettlementFunded {
        settlement_record: record_key,
        source_token_account: source_key,
        amount: params.amount,
        funded_at: Clock::get()?.unix_timestamp,
    });
    Ok(())
}

pub fn execute_settlement_market_purchase<'info>(
    ctx: Context<'_, '_, '_, 'info, ExecuteSettlementMarketPurchaseV2<'info>>,
    params: ExecuteSettlementMarketPurchaseParams,
) -> Result<()> {
    require!(
        !ctx.accounts.state.is_paused,
        crate::PeraxError::ProgramPaused
    );
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
    require!(
        params.minimum_pex_out >= market_remaining,
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

    let quote_before = ctx.accounts.quote_source_token_account.amount;
    let pex_before = ctx.accounts.settlement_pex_vault.amount;
    let mut metas = vec![
        AccountMeta::new(ctx.accounts.quote_source_token_account.key(), false),
        AccountMeta::new(ctx.accounts.settlement_pex_vault.key(), false),
        AccountMeta::new_readonly(ctx.accounts.quote_source_authority.key(), true),
        AccountMeta::new(ctx.accounts.approved_market_pool.key(), false),
        AccountMeta::new_readonly(ctx.accounts.token_program.key(), false),
    ];
    let mut infos = vec![
        ctx.accounts.quote_source_token_account.to_account_info(),
        ctx.accounts.settlement_pex_vault.to_account_info(),
        ctx.accounts.quote_source_authority.to_account_info(),
        ctx.accounts.approved_market_pool.to_account_info(),
        ctx.accounts.token_program.to_account_info(),
    ];
    for account in ctx.remaining_accounts {
        metas.push(if account.is_writable {
            AccountMeta::new(account.key(), account.is_signer)
        } else {
            AccountMeta::new_readonly(account.key(), account.is_signer)
        });
        infos.push(account.clone());
    }
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
            && pex_received >= params.minimum_pex_out,
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

pub fn execute_settlement_vault_funding(
    ctx: Context<ExecuteSettlementVaultFundingV2>,
    _params: ExecuteSettlementVaultFundingParams,
) -> Result<()> {
    require!(
        !ctx.accounts.state.is_paused,
        crate::PeraxError::ProgramPaused
    );
    require!(
        ctx.accounts.settlement_policy.is_active,
        SettlementError::PolicyInactive
    );
    require!(
        matches!(
            ctx.accounts.settlement_record.market_mode,
            SettlementMarketMode::PolicyVault | SettlementMarketMode::Hybrid
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
        ctx.accounts.reserve_vault_config.is_active && !ctx.accounts.reserve_vault_config.is_paused,
        SettlementError::PolicyVaultUnavailable
    );

    let remaining = ctx
        .accounts
        .settlement_record
        .policy_vault_pex_required
        .checked_sub(ctx.accounts.settlement_record.policy_vault_pex_received)
        .ok_or(SettlementError::SettlementArithmeticError)?;
    require!(remaining > 0, SettlementError::InvalidSettlementStatus);
    let available = calculate_vault_available_amount(
        &ctx.accounts.reserve_vault_config,
        ctx.accounts.vault_token_account.amount,
    )?;
    require!(
        available >= remaining,
        SettlementError::PolicyVaultUnavailable
    );

    let now = Clock::get()?.unix_timestamp;
    reset_settlement_daily_window(&mut ctx.accounts.settlement_policy, now);
    require_checked_daily_cap(
        ctx.accounts
            .settlement_policy
            .daily_policy_vault_pex_released,
        remaining,
        ctx.accounts.settlement_policy.daily_policy_vault_pex_cap,
    )?;

    let bump = [ctx.accounts.reserve_vault_config.authority_bump];
    let signer_seeds: &[&[u8]] = &[
        b"reserve-authority",
        ctx.accounts.reserve_vault_config.allocation_id.as_ref(),
        &bump,
    ];
    token::transfer_checked(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            TransferChecked {
                mint: ctx.accounts.pex_mint.to_account_info(),
                from: ctx.accounts.vault_token_account.to_account_info(),
                to: ctx.accounts.settlement_pex_vault.to_account_info(),
                authority: ctx.accounts.vault_authority.to_account_info(),
            },
        )
        .with_signer(&[signer_seeds]),
        remaining,
        ctx.accounts.pex_mint.decimals,
    )?;

    ctx.accounts.reserve_vault_config.total_released = ctx
        .accounts
        .reserve_vault_config
        .total_released
        .checked_add(remaining)
        .ok_or(SettlementError::SettlementArithmeticError)?;
    ctx.accounts
        .settlement_policy
        .daily_policy_vault_pex_released = ctx
        .accounts
        .settlement_policy
        .daily_policy_vault_pex_released
        .checked_add(remaining)
        .ok_or(SettlementError::SettlementArithmeticError)?;

    let record_key = ctx.accounts.settlement_record.key();
    let reserve_key = ctx.accounts.reserve_vault_config.key();
    let record = &mut ctx.accounts.settlement_record;
    record.policy_vault_pex_received = record
        .policy_vault_pex_received
        .checked_add(remaining)
        .ok_or(SettlementError::SettlementArithmeticError)?;
    refresh_settlement_status(record)?;

    emit!(SettlementPolicyVaultFunded {
        settlement_record: record_key,
        reserve_vault_config: reserve_key,
        pex_received: remaining,
        funded_at: now,
    });
    Ok(())
}

pub fn finalize_settlement(
    ctx: Context<FinalizeSettlementV2>,
    _params: FinalizeSettlementParams,
) -> Result<()> {
    require!(
        !ctx.accounts.state.is_paused,
        crate::PeraxError::ProgramPaused
    );
    require!(
        ctx.accounts.settlement_policy.is_active,
        SettlementError::PolicyInactive
    );
    require!(
        ctx.accounts.settlement_record.status == SettlementStatus::Ready,
        SettlementError::SettlementNotFunded
    );

    let acquired = total_settlement_pex_received(&ctx.accounts.settlement_record)?;
    require!(
        acquired >= ctx.accounts.settlement_record.pex_obligation
            && ctx.accounts.settlement_pex_vault.amount >= acquired,
        SettlementError::SettlementNotFunded
    );
    let obligation = ctx.accounts.settlement_record.pex_obligation;
    let surplus = acquired
        .checked_sub(obligation)
        .ok_or(SettlementError::SettlementArithmeticError)?;

    match ctx.accounts.settlement_record.disposition {
        SettlementDisposition::UtilityPayment => require!(
            ctx.accounts.destination_token_account.key()
                == ctx.accounts.settlement_record.destination_token_account,
            SettlementError::InvalidSettlementDestination
        ),
        SettlementDisposition::CustomerDelivery => require!(
            ctx.accounts.destination_token_account.owner
                == ctx.accounts.settlement_record.beneficiary,
            SettlementError::InvalidSettlementDestination
        ),
        SettlementDisposition::Lock | SettlementDisposition::Burn => require!(
            ctx.accounts.destination_token_account.key() == ctx.accounts.lock_vault.key(),
            SettlementError::InvalidSettlementDestination
        ),
    }

    let bump = [ctx.accounts.settlement_custody.authority_bump];
    let record_key = ctx.accounts.settlement_record.key();
    let signer_seeds: &[&[u8]] = &[b"settlement-custody-authority", record_key.as_ref(), &bump];
    match ctx.accounts.settlement_record.disposition {
        SettlementDisposition::Burn => {
            token::burn(
                CpiContext::new(
                    ctx.accounts.token_program.to_account_info(),
                    Burn {
                        mint: ctx.accounts.pex_mint.to_account_info(),
                        from: ctx.accounts.settlement_pex_vault.to_account_info(),
                        authority: ctx.accounts.settlement_authority.to_account_info(),
                    },
                )
                .with_signer(&[signer_seeds]),
                obligation,
            )?;
            if surplus > 0 {
                transfer_from_custody(
                    ctx.accounts.token_program.to_account_info(),
                    ctx.accounts.pex_mint.to_account_info(),
                    ctx.accounts.settlement_pex_vault.to_account_info(),
                    ctx.accounts.lock_vault.to_account_info(),
                    ctx.accounts.settlement_authority.to_account_info(),
                    surplus,
                    ctx.accounts.pex_mint.decimals,
                    signer_seeds,
                )?;
            }
        }
        SettlementDisposition::Lock => transfer_from_custody(
            ctx.accounts.token_program.to_account_info(),
            ctx.accounts.pex_mint.to_account_info(),
            ctx.accounts.settlement_pex_vault.to_account_info(),
            ctx.accounts.lock_vault.to_account_info(),
            ctx.accounts.settlement_authority.to_account_info(),
            acquired,
            ctx.accounts.pex_mint.decimals,
            signer_seeds,
        )?,
        SettlementDisposition::UtilityPayment | SettlementDisposition::CustomerDelivery => {
            transfer_from_custody(
                ctx.accounts.token_program.to_account_info(),
                ctx.accounts.pex_mint.to_account_info(),
                ctx.accounts.settlement_pex_vault.to_account_info(),
                ctx.accounts.destination_token_account.to_account_info(),
                ctx.accounts.settlement_authority.to_account_info(),
                obligation,
                ctx.accounts.pex_mint.decimals,
                signer_seeds,
            )?;
            if surplus > 0 {
                transfer_from_custody(
                    ctx.accounts.token_program.to_account_info(),
                    ctx.accounts.pex_mint.to_account_info(),
                    ctx.accounts.settlement_pex_vault.to_account_info(),
                    ctx.accounts.lock_vault.to_account_info(),
                    ctx.accounts.settlement_authority.to_account_info(),
                    surplus,
                    ctx.accounts.pex_mint.decimals,
                    signer_seeds,
                )?;
            }
        }
    }

    let now = Clock::get()?.unix_timestamp;
    let destination_key = ctx.accounts.destination_token_account.key();
    let record = &mut ctx.accounts.settlement_record;
    if record.disposition == SettlementDisposition::CustomerDelivery {
        record.destination_token_account = destination_key;
    }
    record.status = SettlementStatus::Finalized;
    record.finalized_at = now;
    record.final_pex_amount = obligation;
    record.surplus_locked = surplus;

    emit!(SettlementFinalized {
        settlement_record: record_key,
        settlement_id: record.settlement_id,
        disposition: record.disposition,
        destination_token_account: record.destination_token_account,
        final_pex_amount: obligation,
        surplus_locked: surplus,
        finalized_at: now,
    });
    Ok(())
}

fn transfer_from_custody<'info>(
    token_program: AccountInfo<'info>,
    mint: AccountInfo<'info>,
    source: AccountInfo<'info>,
    destination: AccountInfo<'info>,
    authority: AccountInfo<'info>,
    amount: u64,
    decimals: u8,
    signer_seeds: &[&[u8]],
) -> Result<()> {
    token::transfer_checked(
        CpiContext::new(
            token_program,
            TransferChecked {
                mint,
                from: source,
                to: destination,
                authority,
            },
        )
        .with_signer(&[signer_seeds]),
        amount,
        decimals,
    )
}

fn validate_settlement_policy_params(params: &InitializeSettlementPolicyParams) -> Result<()> {
    require!(
        params.maximum_market_slippage_bps < 10_000
            && params.maximum_quantity_per_settlement > 0
            && params.daily_market_quote_cap > 0
            && params.daily_market_pex_cap > 0
            && params.daily_policy_vault_pex_cap > 0,
        SettlementError::InvalidPolicy
    );
    let shares = params.market_share_bps_by_risk;
    require!(
        shares.iter().all(|value| *value <= 10_000)
            && shares[0] <= shares[1]
            && shares[1] <= shares[2]
            && shares[2] <= shares[3],
        SettlementError::InvalidPolicy
    );
    Ok(())
}

fn validate_product_policy_values(
    settlement_policy: &crate::SettlementPolicy,
    unit_quote_value: u64,
    maximum_quantity: u64,
    accepted_funding_mask: u8,
    disposition: SettlementDisposition,
    fixed_destination: Pubkey,
) -> Result<()> {
    require!(unit_quote_value > 0, SettlementError::InvalidPolicy);
    require!(
        maximum_quantity > 0
            && maximum_quantity <= settlement_policy.maximum_quantity_per_settlement,
        SettlementError::InvalidQuantity
    );
    require!(
        accepted_funding_mask > 0 && accepted_funding_mask & !SETTLEMENT_ALL_FUNDING_METHODS == 0,
        SettlementError::FundingMethodNotAccepted
    );
    if disposition == SettlementDisposition::UtilityPayment {
        require!(
            fixed_destination != Pubkey::default(),
            SettlementError::InvalidSettlementDestination
        );
    }
    Ok(())
}

fn funding_method_allowed(mask: u8, method: SettlementFundingMethod) -> bool {
    let bit = match method {
        SettlementFundingMethod::Pex => SETTLEMENT_FUNDING_PEX,
        SettlementFundingMethod::Stablecoin => SETTLEMENT_FUNDING_STABLECOIN,
        SettlementFundingMethod::Fiat => SETTLEMENT_FUNDING_FIAT,
        SettlementFundingMethod::VirtualAccount => SETTLEMENT_FUNDING_VIRTUAL_ACCOUNT,
    };
    mask & bit != 0
}

pub fn derive_settlement_source_split(
    funding_method: SettlementFundingMethod,
    apc_status: ApcStatus,
    effective_price: u64,
    current_reference_price: u64,
    risk_tier: u8,
    market_share_bps_by_risk: [u16; 4],
    pex_obligation: u64,
) -> Result<(SettlementMarketMode, u64, u64)> {
    require!(
        pex_obligation > 0,
        SettlementError::SettlementArithmeticError
    );
    if funding_method == SettlementFundingMethod::Pex {
        return Ok((SettlementMarketMode::DirectPex, 0, 0));
    }
    let market_share_bps = match apc_status {
        ApcStatus::PumpControl | ApcStatus::AwaitingAbsorption => {
            return err!(SettlementError::MarketActionPaused)
        }
        ApcStatus::Recovery => 10_000,
        ApcStatus::Armed | ApcStatus::Active => {
            if current_reference_price > 0 && effective_price < current_reference_price {
                10_000
            } else {
                let index = usize::from(risk_tier);
                require!(
                    index < market_share_bps_by_risk.len(),
                    SettlementError::InvalidPolicy
                );
                market_share_bps_by_risk[index]
            }
        }
        ApcStatus::Inactive | ApcStatus::Paused => return err!(SettlementError::PolicyInactive),
    };
    let market_pex = amount_bps_ceiling(pex_obligation, market_share_bps)?;
    let vault_pex = pex_obligation
        .checked_sub(market_pex)
        .ok_or(SettlementError::SettlementArithmeticError)?;
    let mode = if market_pex == 0 {
        SettlementMarketMode::PolicyVault
    } else if vault_pex == 0 {
        SettlementMarketMode::MarketPurchase
    } else {
        SettlementMarketMode::Hybrid
    };
    Ok((mode, market_pex, vault_pex))
}

pub fn calculate_settlement_pex_obligation(
    quote_value: u64,
    effective_price: u64,
    price_scale: u64,
) -> Result<u64> {
    require!(
        quote_value > 0 && effective_price > 0 && price_scale > 0,
        SettlementError::SettlementArithmeticError
    );
    let quote_scale = 10u128
        .checked_pow(u32::from(APC_QUOTE_DECIMALS))
        .ok_or(SettlementError::SettlementArithmeticError)?;
    let numerator = u128::from(quote_value)
        .checked_mul(u128::from(PEX_DECIMALS))
        .ok_or(SettlementError::SettlementArithmeticError)?
        .checked_mul(u128::from(price_scale))
        .ok_or(SettlementError::SettlementArithmeticError)?;
    let denominator = quote_scale
        .checked_mul(u128::from(effective_price))
        .ok_or(SettlementError::SettlementArithmeticError)?;
    ceil_div_u128(numerator, denominator)
}

pub fn calculate_settlement_quote_requirement(
    pex_amount: u64,
    effective_price: u64,
    price_scale: u64,
) -> Result<u64> {
    require!(
        pex_amount > 0 && effective_price > 0 && price_scale > 0,
        SettlementError::SettlementArithmeticError
    );
    let quote_scale = 10u128
        .checked_pow(u32::from(APC_QUOTE_DECIMALS))
        .ok_or(SettlementError::SettlementArithmeticError)?;
    let numerator = u128::from(pex_amount)
        .checked_mul(u128::from(effective_price))
        .ok_or(SettlementError::SettlementArithmeticError)?
        .checked_mul(quote_scale)
        .ok_or(SettlementError::SettlementArithmeticError)?;
    let denominator = u128::from(PEX_DECIMALS)
        .checked_mul(u128::from(price_scale))
        .ok_or(SettlementError::SettlementArithmeticError)?;
    ceil_div_u128(numerator, denominator)
}

fn amount_bps_ceiling(amount: u64, bps: u16) -> Result<u64> {
    if bps == 0 {
        return Ok(0);
    }
    let numerator = u128::from(amount)
        .checked_mul(u128::from(bps))
        .ok_or(SettlementError::SettlementArithmeticError)?;
    ceil_div_u128(numerator, APC_BPS_DENOMINATOR)
}

fn amount_with_bps_ceiling(amount: u64, additional_bps: u16) -> Result<u64> {
    let multiplier = APC_BPS_DENOMINATOR
        .checked_add(u128::from(additional_bps))
        .ok_or(SettlementError::SettlementArithmeticError)?;
    let numerator = u128::from(amount)
        .checked_mul(multiplier)
        .ok_or(SettlementError::SettlementArithmeticError)?;
    ceil_div_u128(numerator, APC_BPS_DENOMINATOR)
}

fn ceil_div_u128(numerator: u128, denominator: u128) -> Result<u64> {
    require!(denominator > 0, SettlementError::SettlementArithmeticError);
    let value = numerator
        .checked_add(denominator - 1)
        .ok_or(SettlementError::SettlementArithmeticError)?
        .checked_div(denominator)
        .ok_or(SettlementError::SettlementArithmeticError)?;
    let value = u64::try_from(value).map_err(|_| SettlementError::SettlementArithmeticError)?;
    require!(value > 0, SettlementError::SettlementArithmeticError);
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

fn reset_settlement_daily_window(policy: &mut crate::SettlementPolicy, now: i64) {
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
