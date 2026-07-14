use anchor_lang::prelude::*;
use anchor_spl::token::{self, TransferChecked};
use crate::{
    approved_allocation, calculate_vault_available_amount, reset_release_windows_if_needed,
    validate_emergency_release_fields, validate_growth_release_fields, validate_oracle_snapshot,
    validate_reference, validate_vault_class_for_release, DepositIntoReserveVault,
    ExecuteMarketConditionalRelease, InitializeReserveVault, MarketConditionalReleaseParams,
    PeraxError, ReconcileReserveVault, RecordMarketConditionalRelease,
    ReleaseType, ReserveVaultDepositReceived, ReserveVaultInitialized, ReserveVaultPaused,
    ReserveVaultReconciled, ReserveVaultReleaseExecuted, SetReserveVaultPause,
    VaultClass, VaultMarketConditionalReleaseParams, PEX_MINT_DECIMALS,
};

pub fn initialize_reserve_vault(
    ctx: Context<InitializeReserveVault>,
    allocation_id: [u8; 32],
    vault_class: VaultClass,
    allocation_cap: u64,
) -> Result<()> {
    validate_reference(allocation_id)?;
    require!(allocation_cap > 0, PeraxError::InvalidAllocationCap);
    require!(
        ctx.accounts.token_mint.decimals == PEX_MINT_DECIMALS,
        PeraxError::InvalidTokenMint
    );

    let (approved_class, approved_cap) = approved_allocation(allocation_id)?;
    require!(
        vault_class == approved_class,
        PeraxError::UnsupportedVaultClass
    );
    require!(
        allocation_cap <= approved_cap,
        PeraxError::AllocationCapExceeded
    );

    let config = &mut ctx.accounts.reserve_vault_config;
    config.state = ctx.accounts.state.key();
    config.allocation_id = allocation_id;
    config.vault_class = vault_class;
    config.token_mint = ctx.accounts.token_mint.key();
    config.vault_authority = ctx.accounts.vault_authority.key();
    config.vault_token_account = ctx.accounts.vault_token_account.key();
    config.allocation_cap = allocation_cap;
    config.total_deposited = 0;
    config.total_released = 0;
    config.is_active = true;
    config.is_paused = false;
    config.authority_bump = ctx.bumps.vault_authority;
    config.config_bump = ctx.bumps.reserve_vault_config;

    emit!(ReserveVaultInitialized {
        state: config.state,
        allocation_id,
        vault_class,
        token_mint: config.token_mint,
        vault_authority: config.vault_authority,
        vault_token_account: config.vault_token_account,
        allocation_cap,
        initialized_by: ctx.accounts.authority.key(),
        initialized_at: Clock::get()?.unix_timestamp,
    });

    Ok(())
}

pub fn deposit_into_reserve_vault(
    ctx: Context<DepositIntoReserveVault>,
    allocation_id: [u8; 32],
    amount: u64,
) -> Result<()> {
    require!(amount > 0, PeraxError::InvalidAmount);
    let config = &ctx.accounts.reserve_vault_config;
    require!(config.is_active, PeraxError::VaultInactive);

    let observed_lifetime_deposits = ctx
        .accounts
        .vault_token_account
        .amount
        .checked_add(config.total_released)
        .ok_or(PeraxError::VaultAccountingOverflow)?;
    let accounted_deposits = config.total_deposited.max(observed_lifetime_deposits);
    let new_total_deposited = accounted_deposits
        .checked_add(amount)
        .ok_or(PeraxError::VaultAccountingOverflow)?;
    require!(
        new_total_deposited <= config.allocation_cap,
        PeraxError::AllocationCapExceeded
    );
    let vault_balance_after = ctx
        .accounts
        .vault_token_account
        .amount
        .checked_add(amount)
        .ok_or(PeraxError::VaultAccountingOverflow)?;

    token::transfer_checked(
        ctx.accounts.deposit_transfer_ctx(),
        amount,
        ctx.accounts.token_mint.decimals,
    )?;

    let config = &mut ctx.accounts.reserve_vault_config;
    config.total_deposited = new_total_deposited;

    emit!(ReserveVaultDepositReceived {
        state: config.state,
        allocation_id,
        vault_class: config.vault_class,
        source_owner: ctx.accounts.source_owner.key(),
        source_token_account: ctx.accounts.source_token_account.key(),
        vault_token_account: config.vault_token_account,
        amount,
        total_deposited: config.total_deposited,
        vault_balance_after,
        deposited_at: Clock::get()?.unix_timestamp,
    });

    Ok(())
}

pub fn set_reserve_vault_pause(
    ctx: Context<SetReserveVaultPause>,
    allocation_id: [u8; 32],
    is_paused: bool,
) -> Result<()> {
    let actor = ctx.accounts.actor.key();
    let state = &ctx.accounts.state;
    require!(
        actor == state.authority || actor == state.safety_admin,
        PeraxError::Unauthorized
    );

    let config = &mut ctx.accounts.reserve_vault_config;
    config.is_paused = is_paused;

    emit!(ReserveVaultPaused {
        state: state.key(),
        allocation_id,
        vault_token_account: config.vault_token_account,
        is_paused,
        actor,
        changed_at: Clock::get()?.unix_timestamp,
    });

    Ok(())
}

pub fn reconcile_reserve_vault(
    ctx: Context<ReconcileReserveVault>,
    allocation_id: [u8; 32],
) -> Result<()> {
    let config = &mut ctx.accounts.reserve_vault_config;
    let observed_total_deposited = ctx
        .accounts
        .vault_token_account
        .amount
        .checked_add(config.total_released)
        .ok_or(PeraxError::VaultAccountingOverflow)?;

    require!(
        observed_total_deposited >= config.total_deposited,
        PeraxError::VaultAccountingMismatch
    );
    require!(
        observed_total_deposited <= config.allocation_cap,
        PeraxError::AllocationCapExceeded
    );

    let previous_total_deposited = config.total_deposited;
    config.total_deposited = observed_total_deposited;

    emit!(ReserveVaultReconciled {
        state: ctx.accounts.state.key(),
        allocation_id,
        vault_token_account: config.vault_token_account,
        previous_total_deposited,
        reconciled_total_deposited: observed_total_deposited,
        actual_vault_balance: ctx.accounts.vault_token_account.amount,
        total_released: config.total_released,
        reconciled_by: ctx.accounts.authority.key(),
        reconciled_at: Clock::get()?.unix_timestamp,
    });

    Ok(())
}

pub fn execute_market_conditional_release(
    ctx: Context<ExecuteMarketConditionalRelease>,
    params: VaultMarketConditionalReleaseParams,
) -> Result<()> {
    require!(params.requested_amount > 0, PeraxError::InvalidAmount);
    validate_reference(params.release_id)?;
    validate_reference(params.market_observation_id)?;
    require!(
        params.destination_token_account == ctx.accounts.destination_token_account.key(),
        PeraxError::InvalidReleaseDestination
    );
    require!(
        ctx.accounts.destination_token_account.key()
            != ctx.accounts.reserve_vault_config.vault_token_account,
        PeraxError::InvalidReleaseDestination
    );

    let config_snapshot = &ctx.accounts.reserve_vault_config;
    require!(config_snapshot.is_active, PeraxError::VaultInactive);
    require!(!config_snapshot.is_paused, PeraxError::VaultPaused);
    validate_vault_class_for_release(config_snapshot.vault_class, params.release_type)?;

    let available_amount = calculate_vault_available_amount(
        config_snapshot,
        ctx.accounts.vault_token_account.amount,
    )?;
    require!(
        params.requested_amount <= available_amount,
        PeraxError::InsufficientVaultBalance
    );

    {
        let state = &mut ctx.accounts.state;
        require!(!state.is_paused, PeraxError::ProgramPaused);
        require!(!state.emergency_pause, PeraxError::EmergencyPaused);
        validate_oracle_snapshot(state, &params.snapshot)?;
        reset_release_windows_if_needed(state, params.snapshot.observed_at);

        match params.release_type {
            ReleaseType::Growth => validate_growth_release_fields(
                state,
                params.requested_amount,
                &params.snapshot,
            )?,
            ReleaseType::Emergency => validate_emergency_release_fields(
                state,
                params.requested_amount,
                &params.snapshot,
                available_amount,
            )?,
        }
    }

    let remaining_vault_balance = ctx
        .accounts
        .vault_token_account
        .amount
        .checked_sub(params.requested_amount)
        .ok_or(PeraxError::InsufficientVaultBalance)?;
    let allocation_id = ctx.accounts.reserve_vault_config.allocation_id;
    let authority_bump = [ctx.accounts.reserve_vault_config.authority_bump];
    let authority_seeds: &[&[u8]] = &[
        b"reserve-authority",
        allocation_id.as_ref(),
        &authority_bump,
    ];
    let signer_seeds: &[&[&[u8]]] = &[authority_seeds];

    let transfer_accounts = TransferChecked {
        mint: ctx.accounts.token_mint.to_account_info(),
        from: ctx.accounts.vault_token_account.to_account_info(),
        to: ctx.accounts.destination_token_account.to_account_info(),
        authority: ctx.accounts.vault_authority.to_account_info(),
    };
    let transfer_context = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        transfer_accounts,
    )
    .with_signer(signer_seeds);

    token::transfer_checked(
        transfer_context,
        params.requested_amount,
        ctx.accounts.token_mint.decimals,
    )?;

    let executed_at = Clock::get()?.unix_timestamp;
    let state = &mut ctx.accounts.state;
    state.daily_unlocked_accumulator = state
        .daily_unlocked_accumulator
        .checked_add(params.requested_amount)
        .ok_or(PeraxError::ReleaseCapExceeded)?;
    state.monthly_unlocked_accumulator = state
        .monthly_unlocked_accumulator
        .checked_add(params.requested_amount)
        .ok_or(PeraxError::ReleaseCapExceeded)?;
    state.last_release_timestamp = params.snapshot.observed_at;

    let config = &mut ctx.accounts.reserve_vault_config;
    config.total_released = config
        .total_released
        .checked_add(params.requested_amount)
        .ok_or(PeraxError::VaultAccountingOverflow)?;
    require!(
        config.total_released <= config.total_deposited,
        PeraxError::VaultAccountingMismatch
    );

    let record = &mut ctx.accounts.release_record_v2;
    record.release_id = params.release_id;
    record.state = state.key();
    record.allocation_id = allocation_id;
    record.vault_config = config.key();
    record.vault_class = config.vault_class;
    record.vault_token_account = config.vault_token_account;
    record.destination_token_account = ctx.accounts.destination_token_account.key();
    record.oracle_feed = ctx.accounts.oracle_feed.key();
    record.release_type = params.release_type;
    record.requested_amount = params.requested_amount;
    record.observed_price = params.snapshot.observed_price;
    record.twap_minutes = params.snapshot.twap_minutes;
    record.liquidity_usd = params.snapshot.liquidity_usd;
    record.net_buy_volume_bps = params.snapshot.net_buy_volume_bps;
    record.market_observation_id = params.market_observation_id;
    record.observed_at = params.snapshot.observed_at;
    record.executed_at = executed_at;
    record.bump = ctx.bumps.release_record_v2;

    emit!(ReserveVaultReleaseExecuted {
        state: state.key(),
        allocation_id,
        vault_class: config.vault_class,
        vault_config: config.key(),
        vault_token_account: config.vault_token_account,
        destination_token_account: ctx.accounts.destination_token_account.key(),
        release_record: record.key(),
        release_id: params.release_id,
        release_type: params.release_type,
        release_amount: params.requested_amount,
        remaining_vault_balance,
        total_released: config.total_released,
        market_observation_id: params.market_observation_id,
        observed_at: params.snapshot.observed_at,
        executed_at,
    });

    Ok(())
}

pub fn record_market_conditional_release(
    _ctx: Context<RecordMarketConditionalRelease>,
    _params: MarketConditionalReleaseParams,
) -> Result<()> {
    err!(PeraxError::UseVaultControlledRelease)
}
