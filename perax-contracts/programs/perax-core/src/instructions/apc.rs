use crate::{
    calculate_apc_risk_tier, calculate_band_interval_bps, calculate_band_release_cap,
    calculate_counterweight_requirement, calculate_effective_apc_price, calculate_next_band_price,
    calculate_vault_available_amount, is_apc_releasable_vault_class,
    is_program_derived_destination, reset_apc_windows_if_needed, reset_release_windows_if_needed,
    validate_apc_absorption_confirmation, validate_apc_market_gates,
    validate_apc_observation_fresh, validate_apc_observation_submission, validate_apc_policy,
    validate_apc_reference_support, validate_apc_release_caps, validate_reference,
    validate_sequential_band_index, ActivateApcBandParams, ActivateNextApcBand,
    ApcAbsorptionConfirmed, ApcBandActivated, ApcInitialized, ApcObservationSubmitted, ApcPaused,
    ApcPumpControlEntered, ApcReleaseExecuted, ApcStatus, ApcStatusChanged, ConfirmApcAbsorption,
    ExecuteApcRelease, ExecuteApcReleaseParams, InitializeApc, InitializeApcParams, PauseApc,
    PeraxError, SubmitApcObservation, SubmitApcObservationParams, PEX_MINT_DECIMALS,
};
use anchor_lang::prelude::*;
use anchor_spl::token::{self, TransferChecked};

pub fn initialize_apc(ctx: Context<InitializeApc>, params: InitializeApcParams) -> Result<()> {
    validate_apc_policy(&ctx.accounts.state, &params)?;
    require!(
        ctx.accounts.token_mint.decimals == PEX_MINT_DECIMALS,
        PeraxError::InvalidTokenMint
    );
    require!(
        ctx.accounts.quote_mint.decimals == crate::APC_QUOTE_DECIMALS,
        PeraxError::InvalidCounterweightMint
    );
    require!(
        params.quote_mint != ctx.accounts.token_mint.key(),
        PeraxError::InvalidCounterweightMint
    );

    let config = &mut ctx.accounts.apc_config;
    config.state = ctx.accounts.state.key();
    config.oracle_feed = ctx.accounts.state.oracle_feed;
    config.quote_mint = params.quote_mint;
    config.approved_pool = params.approved_pool;
    config.approved_proceeds_owner = params.approved_proceeds_owner;
    config.approved_proceeds_token_account = params.approved_proceeds_token_account;
    config.approved_recovery_program = params.approved_recovery_program;
    config.price_scale = params.price_scale;
    config.first_activation_price = params.first_activation_price;
    config.minimum_band_interval_bps = params.minimum_band_interval_bps;
    config.maximum_band_interval_bps = params.maximum_band_interval_bps;
    config.maximum_observation_age_seconds = params.maximum_observation_age_seconds;
    config.maximum_future_clock_skew_seconds = params.maximum_future_clock_skew_seconds;
    config.hourly_release_cap = params.hourly_release_cap;
    config.pump_window_release_cap = params.pump_window_release_cap;
    config.pump_window_seconds = params.pump_window_seconds;
    config.minimum_counterweight_coverage_bps = params.minimum_counterweight_coverage_bps;
    config.base_band_release_cap = params.base_band_release_cap;
    config.minimum_twap_minutes = params.minimum_twap_minutes;
    config.minimum_liquidity_usd = params.minimum_liquidity_usd;
    config.minimum_volume_usd = params.minimum_volume_usd;
    config.minimum_buy_pressure_bps = params.minimum_buy_pressure_bps;
    config.risk_velocity_thresholds_bps = params.risk_velocity_thresholds_bps;
    config.risk_volatility_thresholds_bps = params.risk_volatility_thresholds_bps;
    config.risk_price_impact_thresholds_bps = params.risk_price_impact_thresholds_bps;
    config.band_interval_bps_by_risk = params.band_interval_bps_by_risk;
    config.band_release_bps_by_risk = params.band_release_bps_by_risk;
    config.cascade_reduction_bps = params.cascade_reduction_bps;
    config.recovery_spending_cap = params.recovery_spending_cap;
    config.deferred_burn_window_cap = params.deferred_burn_window_cap;
    config.deferred_burn_window_seconds = params.deferred_burn_window_seconds;
    config.deferred_burn_cooldown_seconds = params.deferred_burn_cooldown_seconds;
    config.maximum_recovery_purchase_bps = params.maximum_recovery_purchase_bps;
    config.minimum_counterweight_reserve_bps = params.minimum_counterweight_reserve_bps;
    config.recovery_window_cap = params.recovery_window_cap;
    config.recovery_window_seconds = params.recovery_window_seconds;
    config.recovery_cooldown_seconds = params.recovery_cooldown_seconds;
    config.is_active = true;
    config.is_paused = false;
    config.bump = ctx.bumps.apc_config;

    let apc_state = &mut ctx.accounts.apc_state;
    apc_state.config = config.key();
    apc_state.status = ApcStatus::Armed;
    apc_state.status_before_pause = ApcStatus::Armed;
    apc_state.current_reference_price = ctx.accounts.state.launch_price;
    apc_state.next_band_price = config.first_activation_price;
    apc_state.current_band_index = 0;
    apc_state.highest_crossed_band_index = 0;
    apc_state.pump_window_started_at = 0;
    apc_state.pump_window_released = 0;
    apc_state.hourly_window_started_at = 0;
    apc_state.hourly_released = 0;
    apc_state.total_apc_released = 0;
    apc_state.total_counterweight_credited = 0;
    apc_state.total_counterweight_spent = 0;
    apc_state.last_observation_sequence = 0;
    apc_state.last_release_timestamp = 0;
    apc_state.deferred_burn_amount = 0;
    apc_state.unconfirmed_release_amount = 0;
    apc_state.last_release_observation_id = [0u8; 32];
    apc_state.recovery_entry_observation_id = [0u8; 32];
    apc_state.cascade_observation_id = [0u8; 32];
    apc_state.cascade_band_count = 0;
    apc_state.active_risk_tier = 0;
    apc_state.deferred_burn_window_started_at = 0;
    apc_state.deferred_burn_window_executed = 0;
    apc_state.last_deferred_burn_timestamp = 0;
    apc_state.recovery_window_started_at = 0;
    apc_state.recovery_window_spent = 0;
    apc_state.last_recovery_purchase_timestamp = 0;
    apc_state.bump = ctx.bumps.apc_state;

    let counterweight = &mut ctx.accounts.counterweight_config;
    counterweight.apc_config = config.key();
    counterweight.state = ctx.accounts.state.key();
    counterweight.quote_mint = ctx.accounts.quote_mint.key();
    counterweight.pex_mint = ctx.accounts.token_mint.key();
    counterweight.counterweight_authority = ctx.accounts.counterweight_authority.key();
    counterweight.counterweight_vault = ctx.accounts.counterweight_vault.key();
    counterweight.deferred_burn_authority = ctx.accounts.deferred_burn_authority.key();
    counterweight.deferred_burn_vault = ctx.accounts.deferred_burn_vault.key();
    counterweight.recovery_authority = ctx.accounts.recovery_authority.key();
    counterweight.recovery_vault = ctx.accounts.recovery_vault.key();
    counterweight.approved_proceeds_owner = params.approved_proceeds_owner;
    counterweight.approved_proceeds_token_account = params.approved_proceeds_token_account;
    counterweight.approved_recovery_program = params.approved_recovery_program;
    counterweight.approved_pool = params.approved_pool;
    counterweight.bump = ctx.bumps.counterweight_config;
    counterweight.counterweight_authority_bump = ctx.bumps.counterweight_authority;
    counterweight.deferred_burn_authority_bump = ctx.bumps.deferred_burn_authority;
    counterweight.recovery_authority_bump = ctx.bumps.recovery_authority;

    let initialized_at = Clock::get()?.unix_timestamp;
    emit!(ApcInitialized {
        state: ctx.accounts.state.key(),
        apc_config: config.key(),
        apc_state: apc_state.key(),
        oracle_feed: config.oracle_feed,
        quote_mint: config.quote_mint,
        approved_pool: config.approved_pool,
        first_activation_price: config.first_activation_price,
        initialized_at,
    });

    Ok(())
}

pub fn submit_apc_observation(
    ctx: Context<SubmitApcObservation>,
    params: SubmitApcObservationParams,
) -> Result<()> {
    require!(!ctx.accounts.state.is_paused, PeraxError::ProgramPaused);
    require!(ctx.accounts.apc_config.is_active, PeraxError::ApcInactive);
    require!(!ctx.accounts.apc_config.is_paused, PeraxError::ApcPaused);
    require!(
        ctx.accounts.apc_state.status != ApcStatus::Paused,
        PeraxError::ApcPaused
    );

    let now = Clock::get()?.unix_timestamp;
    validate_apc_observation_submission(
        &ctx.accounts.apc_config,
        &ctx.accounts.apc_state,
        &params,
        now,
    )?;

    let observation = &mut ctx.accounts.observation;
    observation.observation_id = params.observation_id;
    observation.sequence = params.sequence;
    observation.oracle_feed = ctx.accounts.oracle_feed.key();
    observation.pool = params.pool;
    observation.spot_price = params.spot_price;
    observation.twap_price = params.twap_price;
    observation.twap_minutes = params.twap_minutes;
    observation.liquidity_usd = params.liquidity_usd;
    observation.quote_liquidity_usd = params.quote_liquidity_usd;
    observation.volume_usd = params.volume_usd;
    observation.net_buy_pressure_bps = params.net_buy_pressure_bps;
    observation.price_velocity_bps = params.price_velocity_bps;
    observation.volatility_bps = params.volatility_bps;
    observation.estimated_price_impact_bps = params.estimated_price_impact_bps;
    observation.observed_at = params.observed_at;
    observation.submitted_at = now;
    observation.is_consumed_for_release = false;
    observation.is_consumed_for_confirmation = false;
    observation.is_consumed_for_recovery = false;
    observation.consumed_by_release = Pubkey::default();
    observation.consumed_by_recovery = Pubkey::default();
    observation.bump = ctx.bumps.observation;

    ctx.accounts.apc_state.last_observation_sequence = params.sequence;

    emit!(ApcObservationSubmitted {
        observation: observation.key(),
        observation_id: observation.observation_id,
        sequence: observation.sequence,
        spot_price: observation.spot_price,
        twap_price: observation.twap_price,
        pool: observation.pool,
        observed_at: observation.observed_at,
        submitted_at: now,
    });

    Ok(())
}

pub fn activate_next_apc_band(
    ctx: Context<ActivateNextApcBand>,
    params: ActivateApcBandParams,
) -> Result<()> {
    require!(!ctx.accounts.state.is_paused, PeraxError::ProgramPaused);
    let config = &ctx.accounts.apc_config;
    require!(config.is_active, PeraxError::ApcInactive);
    require!(!config.is_paused, PeraxError::ApcPaused);
    require!(
        !matches!(
            ctx.accounts.apc_state.status,
            ApcStatus::Inactive | ApcStatus::Recovery | ApcStatus::Paused
        ),
        PeraxError::InvalidApcStatus
    );

    let now = Clock::get()?.unix_timestamp;
    validate_apc_observation_fresh(config, &ctx.accounts.observation, now)?;
    validate_apc_market_gates(config, &ctx.accounts.observation)?;
    require!(
        !ctx.accounts.observation.is_consumed_for_release
            && !ctx.accounts.observation.is_consumed_for_confirmation
            && !ctx.accounts.observation.is_consumed_for_recovery,
        PeraxError::ObservationAlreadyUsed
    );

    let apc_state = &mut ctx.accounts.apc_state;
    validate_sequential_band_index(apc_state.current_band_index, params.band_index)?;

    let effective_price = calculate_effective_apc_price(
        ctx.accounts.observation.spot_price,
        ctx.accounts.observation.twap_price,
    )?;
    let trigger_price = if params.band_index == 1 {
        config.first_activation_price
    } else {
        apc_state.next_band_price
    };
    require!(
        effective_price >= trigger_price,
        PeraxError::ApcPriceGateNotMet
    );

    let risk_tier = calculate_apc_risk_tier(
        ctx.accounts.observation.price_velocity_bps,
        ctx.accounts.observation.volatility_bps,
        ctx.accounts.observation.estimated_price_impact_bps,
        config.risk_velocity_thresholds_bps,
        config.risk_volatility_thresholds_bps,
        config.risk_price_impact_thresholds_bps,
    );
    let interval_bps = calculate_band_interval_bps(config, risk_tier)?;

    let cascade_position =
        if apc_state.cascade_observation_id == ctx.accounts.observation.observation_id {
            apc_state
                .cascade_band_count
                .checked_add(1)
                .ok_or(PeraxError::InvalidBandIndex)?
        } else {
            1
        };
    let maximum_release_amount = calculate_band_release_cap(config, risk_tier, cascade_position)?;

    let band = &mut ctx.accounts.band_record;
    band.apc_state = apc_state.key();
    band.band_index = params.band_index;
    band.trigger_price = trigger_price;
    band.interval_bps = interval_bps;
    band.risk_tier = risk_tier;
    band.maximum_release_amount = maximum_release_amount;
    band.amount_released = 0;
    band.activation_observation_id = ctx.accounts.observation.observation_id;
    band.first_observed_at = ctx.accounts.observation.observed_at;
    band.last_release_at = 0;
    band.is_crossed = true;
    band.is_exhausted = false;
    band.bump = ctx.bumps.band_record;

    let previous_status = apc_state.status;
    apc_state.current_band_index = params.band_index;
    apc_state.highest_crossed_band_index =
        apc_state.highest_crossed_band_index.max(params.band_index);
    apc_state.current_reference_price = trigger_price;
    apc_state.next_band_price = calculate_next_band_price(trigger_price, interval_bps)?;
    apc_state.cascade_observation_id = ctx.accounts.observation.observation_id;
    apc_state.cascade_band_count = cascade_position;
    apc_state.active_risk_tier = risk_tier;
    apc_state.status =
        if previous_status == ApcStatus::PumpControl || cascade_position > 1 || risk_tier >= 2 {
            ApcStatus::PumpControl
        } else if previous_status == ApcStatus::AwaitingAbsorption {
            ApcStatus::AwaitingAbsorption
        } else {
            ApcStatus::Active
        };

    emit!(ApcBandActivated {
        apc_state: apc_state.key(),
        band_record: band.key(),
        band_index: band.band_index,
        trigger_price: band.trigger_price,
        interval_bps: band.interval_bps,
        risk_tier: band.risk_tier,
        maximum_release_amount: band.maximum_release_amount,
        observation_id: band.activation_observation_id,
        activated_at: now,
    });

    if apc_state.status == ApcStatus::PumpControl {
        emit!(ApcPumpControlEntered {
            apc_state: apc_state.key(),
            band_index: band.band_index,
            risk_tier,
            observation_id: band.activation_observation_id,
            entered_at: now,
        });
    }
    if previous_status != apc_state.status {
        emit!(ApcStatusChanged {
            apc_state: apc_state.key(),
            previous_status,
            new_status: apc_state.status,
            actor: ctx.accounts.oracle_feed.key(),
            changed_at: now,
        });
    }

    Ok(())
}

pub fn execute_apc_release(
    ctx: Context<ExecuteApcRelease>,
    params: ExecuteApcReleaseParams,
) -> Result<()> {
    validate_reference(params.release_id)?;
    validate_reference(params.observation_id)?;
    require!(params.amount > 0, PeraxError::InvalidAmount);
    require!(!ctx.accounts.state.is_paused, PeraxError::ProgramPaused);
    require!(
        !ctx.accounts.state.emergency_pause,
        PeraxError::EmergencyPaused
    );

    let config = &ctx.accounts.apc_config;
    require!(config.is_active, PeraxError::ApcInactive);
    require!(!config.is_paused, PeraxError::ApcPaused);
    require!(
        !matches!(
            ctx.accounts.apc_state.status,
            ApcStatus::Inactive | ApcStatus::Recovery | ApcStatus::Paused
        ),
        PeraxError::InvalidApcStatus
    );

    let now = Clock::get()?.unix_timestamp;
    validate_apc_observation_fresh(config, &ctx.accounts.observation, now)?;
    validate_apc_market_gates(config, &ctx.accounts.observation)?;
    require!(
        !ctx.accounts.observation.is_consumed_for_release
            && !ctx.accounts.observation.is_consumed_for_confirmation
            && !ctx.accounts.observation.is_consumed_for_recovery,
        PeraxError::ObservationAlreadyUsed
    );
    require!(
        ctx.accounts.band_record.is_crossed,
        PeraxError::BandNotActivated
    );
    require!(
        !ctx.accounts.band_record.is_exhausted,
        PeraxError::BandAlreadyExhausted
    );
    let effective_price = calculate_effective_apc_price(
        ctx.accounts.observation.spot_price,
        ctx.accounts.observation.twap_price,
    )?;
    require!(
        effective_price >= ctx.accounts.band_record.trigger_price,
        PeraxError::ApcPriceGateNotMet
    );
    validate_apc_reference_support(&ctx.accounts.apc_state, effective_price)?;

    let vault = &ctx.accounts.reserve_vault_config;
    require!(vault.is_active, PeraxError::VaultInactive);
    require!(!vault.is_paused, PeraxError::VaultPaused);
    require!(
        is_apc_releasable_vault_class(vault.vault_class),
        PeraxError::VaultClassNotMarketReleasable
    );
    require!(
        ctx.accounts.destination_token_account.key() == vault.approved_destination_token_account
            && ctx.accounts.destination_token_account.owner == vault.approved_destination_owner,
        PeraxError::InvalidApprovedDestination
    );
    require!(
        !is_program_derived_destination(ctx.accounts.destination_token_account.owner),
        PeraxError::DestinationIsReserveVault
    );

    let available_amount =
        calculate_vault_available_amount(vault, ctx.accounts.vault_token_account.amount)?;
    require!(
        params.amount <= available_amount,
        PeraxError::InsufficientVaultBalance
    );

    reset_release_windows_if_needed(&mut ctx.accounts.state, now);
    reset_apc_windows_if_needed(config, &mut ctx.accounts.apc_state, now);
    validate_apc_release_caps(
        config,
        &ctx.accounts.apc_state,
        &ctx.accounts.state,
        ctx.accounts.band_record.amount_released,
        ctx.accounts.band_record.maximum_release_amount,
        params.amount,
    )?;

    let tracked_counterweight_available = ctx
        .accounts
        .apc_state
        .total_counterweight_credited
        .checked_sub(ctx.accounts.apc_state.total_counterweight_spent)
        .ok_or(PeraxError::InvalidCounterweightVault)?;
    require!(
        ctx.accounts.counterweight_vault.amount >= tracked_counterweight_available,
        PeraxError::InvalidCounterweightVault
    );
    let counterweight_required_before = calculate_counterweight_requirement(
        ctx.accounts.apc_state.total_apc_released,
        ctx.accounts.observation.twap_price,
        config.price_scale,
        config.minimum_counterweight_coverage_bps,
    )?;
    require!(
        tracked_counterweight_available >= counterweight_required_before,
        PeraxError::CounterweightCoverageNotMet
    );

    let remaining_vault_balance = ctx
        .accounts
        .vault_token_account
        .amount
        .checked_sub(params.amount)
        .ok_or(PeraxError::InsufficientVaultBalance)?;
    require!(
        remaining_vault_balance >= vault.unsolicited_balance,
        PeraxError::VaultAccountingMismatch
    );

    let allocation_id = vault.allocation_id;
    let authority_bump = [vault.authority_bump];
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
    token::transfer_checked(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            transfer_accounts,
        )
        .with_signer(signer_seeds),
        params.amount,
        ctx.accounts.token_mint.decimals,
    )?;

    let band_released_after = ctx
        .accounts
        .band_record
        .amount_released
        .checked_add(params.amount)
        .ok_or(PeraxError::BandReleaseCapExceeded)?;
    let pump_window_released_after = ctx
        .accounts
        .apc_state
        .pump_window_released
        .checked_add(params.amount)
        .ok_or(PeraxError::PumpWindowCapExceeded)?;
    let hourly_released_after = ctx
        .accounts
        .apc_state
        .hourly_released
        .checked_add(params.amount)
        .ok_or(PeraxError::HourlyApcCapExceeded)?;
    let total_apc_released_after = ctx
        .accounts
        .apc_state
        .total_apc_released
        .checked_add(params.amount)
        .ok_or(PeraxError::ReleaseCapExceeded)?;
    let unconfirmed_release_after = ctx
        .accounts
        .apc_state
        .unconfirmed_release_amount
        .checked_add(params.amount)
        .ok_or(PeraxError::PumpWindowCapExceeded)?;
    let counterweight_required_after = calculate_counterweight_requirement(
        total_apc_released_after,
        ctx.accounts.observation.twap_price,
        config.price_scale,
        config.minimum_counterweight_coverage_bps,
    )?;

    ctx.accounts.state.daily_unlocked_accumulator = ctx
        .accounts
        .state
        .daily_unlocked_accumulator
        .checked_add(params.amount)
        .ok_or(PeraxError::DailyReleaseCapExceeded)?;
    ctx.accounts.state.monthly_unlocked_accumulator = ctx
        .accounts
        .state
        .monthly_unlocked_accumulator
        .checked_add(params.amount)
        .ok_or(PeraxError::MonthlyReleaseCapExceeded)?;
    ctx.accounts.state.last_release_timestamp = now;

    ctx.accounts.apc_state.hourly_released = hourly_released_after;
    ctx.accounts.apc_state.pump_window_released = pump_window_released_after;
    ctx.accounts.apc_state.total_apc_released = total_apc_released_after;
    ctx.accounts.apc_state.unconfirmed_release_amount = unconfirmed_release_after;
    ctx.accounts.apc_state.last_release_timestamp = now;
    ctx.accounts.apc_state.last_release_observation_id = params.observation_id;
    let previous_status = ctx.accounts.apc_state.status;
    ctx.accounts.apc_state.status = if previous_status == ApcStatus::PumpControl
        || ctx.accounts.apc_state.active_risk_tier >= 2
        || ctx.accounts.apc_state.cascade_band_count > 1
    {
        ApcStatus::PumpControl
    } else {
        ApcStatus::AwaitingAbsorption
    };

    ctx.accounts.band_record.amount_released = band_released_after;
    ctx.accounts.band_record.last_release_at = now;
    ctx.accounts.band_record.is_exhausted =
        band_released_after == ctx.accounts.band_record.maximum_release_amount;

    ctx.accounts.reserve_vault_config.total_released = ctx
        .accounts
        .reserve_vault_config
        .total_released
        .checked_add(params.amount)
        .ok_or(PeraxError::VaultAccountingOverflow)?;
    require!(
        ctx.accounts.reserve_vault_config.total_released
            <= ctx.accounts.reserve_vault_config.authorized_deposited,
        PeraxError::VaultAccountingMismatch
    );

    let record = &mut ctx.accounts.release_record;
    record.release_id = params.release_id;
    record.band_index = params.band_index;
    record.band_record = ctx.accounts.band_record.key();
    record.allocation_id = params.allocation_id;
    record.vault_config = ctx.accounts.reserve_vault_config.key();
    record.destination_token_account = ctx.accounts.destination_token_account.key();
    record.observation_id = params.observation_id;
    record.amount = params.amount;
    record.band_released_after = band_released_after;
    record.pump_window_released_after = pump_window_released_after;
    record.unconfirmed_release_after = unconfirmed_release_after;
    record.counterweight_required_after = counterweight_required_after;
    record.executed_at = now;
    record.bump = ctx.bumps.release_record;

    ctx.accounts.observation.is_consumed_for_release = true;
    ctx.accounts.observation.consumed_by_release = record.key();

    emit!(ApcReleaseExecuted {
        release_record: record.key(),
        release_id: record.release_id,
        band_index: record.band_index,
        allocation_id: record.allocation_id,
        vault_config: record.vault_config,
        destination_token_account: record.destination_token_account,
        observation_id: record.observation_id,
        amount: record.amount,
        band_released_after,
        pump_window_released_after,
        unconfirmed_release_after,
        counterweight_required_after,
        executed_at: now,
    });

    if previous_status != ctx.accounts.apc_state.status {
        emit!(ApcStatusChanged {
            apc_state: ctx.accounts.apc_state.key(),
            previous_status,
            new_status: ctx.accounts.apc_state.status,
            actor: ctx.accounts.oracle_feed.key(),
            changed_at: now,
        });
    }

    Ok(())
}

pub fn confirm_apc_absorption(ctx: Context<ConfirmApcAbsorption>) -> Result<()> {
    require!(!ctx.accounts.state.is_paused, PeraxError::ProgramPaused);
    require!(ctx.accounts.apc_config.is_active, PeraxError::ApcInactive);
    require!(!ctx.accounts.apc_config.is_paused, PeraxError::ApcPaused);
    let now = Clock::get()?.unix_timestamp;
    let confirmed_price = validate_apc_absorption_confirmation(
        &ctx.accounts.apc_config,
        &ctx.accounts.apc_state,
        &ctx.accounts.observation,
        now,
    )?;

    let previous_status = ctx.accounts.apc_state.status;
    ctx.accounts.apc_state.status = ApcStatus::Active;
    ctx.accounts.apc_state.unconfirmed_release_amount = 0;
    ctx.accounts.observation.is_consumed_for_confirmation = true;

    emit!(ApcAbsorptionConfirmed {
        apc_state: ctx.accounts.apc_state.key(),
        band_index: ctx.accounts.apc_state.current_band_index,
        observation_id: ctx.accounts.observation.observation_id,
        reference_price: ctx.accounts.apc_state.current_reference_price,
        confirmed_price,
        confirmed_at: now,
    });
    emit!(ApcStatusChanged {
        apc_state: ctx.accounts.apc_state.key(),
        previous_status,
        new_status: ApcStatus::Active,
        actor: ctx.accounts.oracle_feed.key(),
        changed_at: now,
    });

    Ok(())
}

pub fn pause_apc(ctx: Context<PauseApc>, is_paused: bool) -> Result<()> {
    let actor = ctx.accounts.actor.key();
    require!(
        actor == ctx.accounts.state.authority || actor == ctx.accounts.state.safety_admin,
        PeraxError::Unauthorized
    );
    let previous_status = ctx.accounts.apc_state.status;
    if is_paused {
        require!(previous_status != ApcStatus::Paused, PeraxError::ApcPaused);
        ctx.accounts.apc_state.status_before_pause = previous_status;
        ctx.accounts.apc_state.status = ApcStatus::Paused;
    } else {
        require!(
            previous_status == ApcStatus::Paused,
            PeraxError::InvalidApcStatus
        );
        require!(
            ctx.accounts.apc_state.status_before_pause != ApcStatus::Paused,
            PeraxError::InvalidApcStatus
        );
        ctx.accounts.apc_state.status = ctx.accounts.apc_state.status_before_pause;
    }
    ctx.accounts.apc_config.is_paused = is_paused;
    let now = Clock::get()?.unix_timestamp;
    emit!(ApcPaused {
        apc_state: ctx.accounts.apc_state.key(),
        actor,
        is_paused,
        changed_at: now,
    });
    if previous_status != ctx.accounts.apc_state.status {
        emit!(ApcStatusChanged {
            apc_state: ctx.accounts.apc_state.key(),
            previous_status,
            new_status: ctx.accounts.apc_state.status,
            actor,
            changed_at: now,
        });
    }
    Ok(())
}
