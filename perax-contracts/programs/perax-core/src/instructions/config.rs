use anchor_lang::prelude::*;
use crate::{
    AcceptAuthority, AuthorityTransferAccepted, AuthorityTransferCancelled,
    AuthorityTransferNominated, ConfigInitialized, ConfigUpdated, EmergencyPauseStatusChanged,
    Initialize, InitializeParams, MarketEngineConfigUpdated, PauseStatusChanged, PeraxError,
    SafetyAdminAction, UpdateConfig, UpdateConfigParams, UpdateMarketEngineConfigParams,
};

pub fn initialize(ctx: Context<Initialize>, params: InitializeParams) -> Result<()> {
    require!(
        params.trading_company_token_account != Pubkey::default(),
        PeraxError::InvalidTradingCompanyAccount
    );
    require!(
        params.trading_company_revenue_token_account != Pubkey::default(),
        PeraxError::InvalidTradingCompanyRevenueAccount
    );
    require!(
        params.trading_company_token_account != params.trading_company_revenue_token_account,
        PeraxError::TradingCompanyAccountsMustDiffer
    );
    require!(
        params.safety_admin != Pubkey::default(),
        PeraxError::InvalidSafetyAdmin
    );
    require!(
        params.oracle_feed != Pubkey::default(),
        PeraxError::InvalidOracleFeed
    );
    require!(params.launch_price > 0, PeraxError::InvalidMarketParameter);
    require!(
        params.daily_release_cap > 0,
        PeraxError::InvalidMarketParameter
    );
    require!(
        params.monthly_release_cap >= params.daily_release_cap,
        PeraxError::InvalidMarketParameter
    );

    let state = &mut ctx.accounts.state;
    state.authority = ctx.accounts.authority.key();
    state.pending_authority = Pubkey::default();
    state.has_pending_authority = false;
    state.token_mint = params.token_mint;
    state.trading_company_token_account = params.trading_company_token_account;
    state.trading_company_revenue_token_account = params.trading_company_revenue_token_account;
    state.max_payment_amount = params.max_payment_amount;
    state.safety_admin = params.safety_admin;
    state.oracle_feed = params.oracle_feed;
    state.launch_price = params.launch_price;
    state.current_stepped_floor = params.current_stepped_floor.max(params.launch_price);
    state.last_release_timestamp = 0;
    state.daily_unlocked_accumulator = 0;
    state.monthly_unlocked_accumulator = 0;
    state.daily_window_start = 0;
    state.monthly_window_start = 0;
    state.daily_release_cap = params.daily_release_cap;
    state.monthly_release_cap = params.monthly_release_cap;
    state.emergency_hourly_release_bps = params.emergency_hourly_release_bps;
    state.daily_burn_accumulator = 0;
    state.daily_burn_window_start = 0;
    state.is_paused = false;
    state.emergency_pause = false;
    state.bump = ctx.bumps.state;

    emit!(ConfigInitialized {
        authority: state.authority,
        token_mint: state.token_mint,
        trading_company_token_account: state.trading_company_token_account,
        trading_company_revenue_token_account: state.trading_company_revenue_token_account,
        safety_admin: state.safety_admin,
        oracle_feed: state.oracle_feed,
        launch_price: state.launch_price,
        daily_release_cap: state.daily_release_cap,
        monthly_release_cap: state.monthly_release_cap,
        max_payment_amount: state.max_payment_amount,
    });

    Ok(())
}

pub fn update_config(ctx: Context<UpdateConfig>, params: UpdateConfigParams) -> Result<()> {
    let state = &mut ctx.accounts.state;

    if let Some(trading_company_token_account) = params.trading_company_token_account {
        require!(
            trading_company_token_account != Pubkey::default(),
            PeraxError::InvalidTradingCompanyAccount
        );
        require!(
            trading_company_token_account != state.trading_company_revenue_token_account,
            PeraxError::TradingCompanyAccountsMustDiffer
        );
        state.trading_company_token_account = trading_company_token_account;
    }

    if let Some(trading_company_revenue_token_account) =
        params.trading_company_revenue_token_account
    {
        require!(
            trading_company_revenue_token_account != Pubkey::default(),
            PeraxError::InvalidTradingCompanyRevenueAccount
        );
        require!(
            trading_company_revenue_token_account != state.trading_company_token_account,
            PeraxError::TradingCompanyAccountsMustDiffer
        );
        state.trading_company_revenue_token_account = trading_company_revenue_token_account;
    }

    if let Some(max_payment_amount) = params.max_payment_amount {
        state.max_payment_amount = max_payment_amount;
    }

    emit!(ConfigUpdated {
        authority: state.authority,
        trading_company_token_account: state.trading_company_token_account,
        trading_company_revenue_token_account: state.trading_company_revenue_token_account,
        max_payment_amount: state.max_payment_amount,
    });

    Ok(())
}

pub fn update_market_engine_config(
    ctx: Context<UpdateConfig>,
    params: UpdateMarketEngineConfigParams,
) -> Result<()> {
    let state = &mut ctx.accounts.state;

    if let Some(safety_admin) = params.safety_admin {
        require!(
            safety_admin != Pubkey::default(),
            PeraxError::InvalidSafetyAdmin
        );
        state.safety_admin = safety_admin;
    }

    if let Some(oracle_feed) = params.oracle_feed {
        require!(
            oracle_feed != Pubkey::default(),
            PeraxError::InvalidOracleFeed
        );
        state.oracle_feed = oracle_feed;
    }

    if let Some(current_stepped_floor) = params.current_stepped_floor {
        require!(
            current_stepped_floor > 0,
            PeraxError::InvalidMarketParameter
        );
        state.current_stepped_floor = current_stepped_floor;
    }

    if let Some(daily_release_cap) = params.daily_release_cap {
        require!(daily_release_cap > 0, PeraxError::InvalidMarketParameter);
        state.daily_release_cap = daily_release_cap;
    }

    if let Some(monthly_release_cap) = params.monthly_release_cap {
        require!(
            monthly_release_cap >= state.daily_release_cap,
            PeraxError::InvalidMarketParameter
        );
        state.monthly_release_cap = monthly_release_cap;
    }

    if let Some(emergency_hourly_release_bps) = params.emergency_hourly_release_bps {
        require!(
            emergency_hourly_release_bps <= 10_000,
            PeraxError::InvalidMarketParameter
        );
        state.emergency_hourly_release_bps = emergency_hourly_release_bps;
    }

    emit!(MarketEngineConfigUpdated {
        authority: state.authority,
        safety_admin: state.safety_admin,
        oracle_feed: state.oracle_feed,
        current_stepped_floor: state.current_stepped_floor,
        daily_release_cap: state.daily_release_cap,
        monthly_release_cap: state.monthly_release_cap,
        emergency_hourly_release_bps: state.emergency_hourly_release_bps,
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

pub fn set_emergency_pause(ctx: Context<SafetyAdminAction>, is_paused: bool) -> Result<()> {
    let state = &mut ctx.accounts.state;
    state.emergency_pause = is_paused;
    emit!(EmergencyPauseStatusChanged {
        safety_admin: ctx.accounts.safety_admin.key(),
        emergency_pause: is_paused,
    });
    Ok(())
}

pub fn nominate_authority(ctx: Context<UpdateConfig>, new_authority: Pubkey) -> Result<()> {
    require!(
        new_authority != Pubkey::default(),
        PeraxError::InvalidAuthority
    );
    let state = &mut ctx.accounts.state;
    require!(
        new_authority != state.authority,
        PeraxError::InvalidAuthority
    );
    state.pending_authority = new_authority;
    state.has_pending_authority = true;
    emit!(AuthorityTransferNominated {
        current_authority: state.authority,
        pending_authority: state.pending_authority,
    });
    Ok(())
}

pub fn cancel_authority_transfer(ctx: Context<UpdateConfig>) -> Result<()> {
    let state = &mut ctx.accounts.state;
    require!(state.has_pending_authority, PeraxError::NoPendingAuthority);
    let cancelled_authority = state.pending_authority;
    state.pending_authority = Pubkey::default();
    state.has_pending_authority = false;
    emit!(AuthorityTransferCancelled {
        authority: state.authority,
        cancelled_authority,
    });
    Ok(())
}

pub fn accept_authority(ctx: Context<AcceptAuthority>) -> Result<()> {
    let state = &mut ctx.accounts.state;
    let new_authority = ctx.accounts.pending_authority.key();
    require!(state.has_pending_authority, PeraxError::NoPendingAuthority);
    require!(
        state.pending_authority == new_authority,
        PeraxError::Unauthorized
    );
    let previous_authority = state.authority;
    state.authority = new_authority;
    state.pending_authority = Pubkey::default();
    state.has_pending_authority = false;
    emit!(AuthorityTransferAccepted {
        previous_authority,
        new_authority,
    });
    Ok(())
}
