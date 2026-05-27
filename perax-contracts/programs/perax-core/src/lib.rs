use anchor_lang::prelude::*;
use anchor_spl::token::{self, Burn, Mint, Token, TokenAccount, Transfer};

declare_id!("11111111111111111111111111111111");

pub const PEX_DECIMALS: u64 = 1_000_000;
pub const PEX_TOTAL_SUPPLY: u64 = 1_000_000_000 * PEX_DECIMALS;
pub const PEX_LAUNCH_PRICE_SCALED: u64 = 1_200;
pub const GROWTH_PRICE_MULTIPLIER: u64 = 3;
pub const MIN_GROWTH_TWAP_MINUTES: u64 = 60;
pub const INITIAL_LIQUIDITY_USD: u64 = 4_560;
pub const MIN_GROWTH_LIQUIDITY_USD: u64 = INITIAL_LIQUIDITY_USD * 5;
pub const MIN_NET_BUY_VOLUME_BPS: u16 = 5_000;
pub const DAILY_RELEASE_CAP: u64 = 10_000_000 * PEX_DECIMALS;
pub const MONTHLY_RELEASE_CAP: u64 = 50_000_000 * PEX_DECIMALS;
pub const RELEASE_COOLDOWN_SECONDS: i64 = 86_400;
pub const EMERGENCY_DOWNSIDE_TRIGGER_BPS: u16 = 3_000;
pub const EMERGENCY_LIQUIDITY_DRAIN_TRIGGER_BPS: u16 = 6_000;
pub const EMERGENCY_HOURLY_RESERVE_RELEASE_BPS: u16 = 50;

#[program]
pub mod perax_core {
    use super::*;

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
        require!(params.safety_admin != Pubkey::default(), PeraxError::InvalidSafetyAdmin);
        require!(params.oracle_feed != Pubkey::default(), PeraxError::InvalidOracleFeed);
        require!(params.launch_price > 0, PeraxError::InvalidMarketParameter);
        require!(params.daily_release_cap > 0, PeraxError::InvalidMarketParameter);
        require!(params.monthly_release_cap >= params.daily_release_cap, PeraxError::InvalidMarketParameter);

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

        if let Some(trading_company_revenue_token_account) = params.trading_company_revenue_token_account {
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
            require!(safety_admin != Pubkey::default(), PeraxError::InvalidSafetyAdmin);
            state.safety_admin = safety_admin;
        }

        if let Some(oracle_feed) = params.oracle_feed {
            require!(oracle_feed != Pubkey::default(), PeraxError::InvalidOracleFeed);
            state.oracle_feed = oracle_feed;
        }

        if let Some(current_stepped_floor) = params.current_stepped_floor {
            require!(current_stepped_floor > 0, PeraxError::InvalidMarketParameter);
            state.current_stepped_floor = current_stepped_floor;
        }

        if let Some(daily_release_cap) = params.daily_release_cap {
            require!(daily_release_cap > 0, PeraxError::InvalidMarketParameter);
            state.daily_release_cap = daily_release_cap;
        }

        if let Some(monthly_release_cap) = params.monthly_release_cap {
            require!(monthly_release_cap >= state.daily_release_cap, PeraxError::InvalidMarketParameter);
            state.monthly_release_cap = monthly_release_cap;
        }

        if let Some(emergency_hourly_release_bps) = params.emergency_hourly_release_bps {
            require!(emergency_hourly_release_bps <= 10_000, PeraxError::InvalidMarketParameter);
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

    pub fn record_market_conditional_release(
        ctx: Context<RecordMarketConditionalRelease>,
        params: MarketConditionalReleaseParams,
    ) -> Result<()> {
        let state = &mut ctx.accounts.state;
        require!(!state.is_paused, PeraxError::ProgramPaused);
        require!(!state.emergency_pause, PeraxError::EmergencyPaused);
        require!(params.requested_amount > 0, PeraxError::InvalidAmount);
        validate_reference(params.release_id)?;
        validate_oracle_snapshot(state, &params.snapshot)?;
        reset_release_windows_if_needed(state, params.snapshot.observed_at);

        match params.release_type {
            ReleaseType::Growth => validate_growth_release(state, &params)?,
            ReleaseType::Emergency => validate_emergency_release(state, &params)?,
        }

        state.daily_unlocked_accumulator = state
            .daily_unlocked_accumulator
            .checked_add(params.requested_amount)
            .ok_or(PeraxError::ReleaseCapExceeded)?;
        state.monthly_unlocked_accumulator = state
            .monthly_unlocked_accumulator
            .checked_add(params.requested_amount)
            .ok_or(PeraxError::ReleaseCapExceeded)?;
        state.last_release_timestamp = params.snapshot.observed_at;

        emit!(MarketConditionalReleaseApproved {
            oracle_feed: ctx.accounts.oracle_feed.key(),
            release_type: params.release_type,
            requested_amount: params.requested_amount,
            release_id: params.release_id,
            observed_price: params.snapshot.observed_price,
            twap_minutes: params.snapshot.twap_minutes,
            liquidity_usd: params.snapshot.liquidity_usd,
            net_buy_volume_bps: params.snapshot.net_buy_volume_bps,
            daily_unlocked_accumulator: state.daily_unlocked_accumulator,
            monthly_unlocked_accumulator: state.monthly_unlocked_accumulator,
            observed_at: params.snapshot.observed_at,
        });

        Ok(())
    }

    pub fn nominate_authority(ctx: Context<UpdateConfig>, new_authority: Pubkey) -> Result<()> {
        require!(new_authority != Pubkey::default(), PeraxError::InvalidAuthority);

        let state = &mut ctx.accounts.state;
        require!(new_authority != state.authority, PeraxError::InvalidAuthority);

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
        require!(state.pending_authority == new_authority, PeraxError::Unauthorized);

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
        payment_record.trading_company_revenue_token_account = state.trading_company_revenue_token_account;
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

    pub fn burn_from_trading_company(
        ctx: Context<BurnFromTradingCompany>,
        amount: u64,
        decision_id: [u8; 32],
    ) -> Result<()> {
        let state = &ctx.accounts.state;
        require!(!state.is_paused, PeraxError::ProgramPaused);
        require!(amount > 0, PeraxError::InvalidAmount);
        validate_reference(decision_id)?;

        token::burn(ctx.accounts.burn_ctx(), amount)?;

        emit!(TradingCompanyBurnExecuted {
            authority: ctx.accounts.authority.key(),
            trading_company_authority: ctx.accounts.trading_company_authority.key(),
            token_mint: state.token_mint,
            trading_company_token_account: state.trading_company_token_account,
            trading_company_revenue_token_account: state.trading_company_revenue_token_account,
            amount,
            decision_id,
        });

        Ok(())
    }
}

fn validate_payment_amount(state: &PeraxState, amount: u64) -> Result<()> {
    require!(amount > 0, PeraxError::InvalidAmount);

    if state.max_payment_amount > 0 {
        require!(amount <= state.max_payment_amount, PeraxError::PaymentAmountTooLarge);
    }

    Ok(())
}

fn validate_reference(reference: [u8; 32]) -> Result<()> {
    require!(reference != [0u8; 32], PeraxError::InvalidReference);
    Ok(())
}

fn validate_oracle_snapshot(state: &PeraxState, snapshot: &MarketConditionSnapshot) -> Result<()> {
    require!(snapshot.observed_at > 0, PeraxError::InvalidMarketParameter);
    require!(snapshot.observed_price > 0, PeraxError::InvalidMarketParameter);
    require!(snapshot.net_buy_volume_bps <= 10_000, PeraxError::InvalidMarketParameter);
    require!(state.oracle_feed != Pubkey::default(), PeraxError::InvalidOracleFeed);
    Ok(())
}

fn validate_growth_release(state: &PeraxState, params: &MarketConditionalReleaseParams) -> Result<()> {
    let snapshot = &params.snapshot;
    let growth_price_trigger = state
        .launch_price
        .checked_mul(GROWTH_PRICE_MULTIPLIER)
        .ok_or(PeraxError::InvalidMarketParameter)?;

    require!(snapshot.observed_price >= growth_price_trigger, PeraxError::GrowthPriceGateNotMet);
    require!(snapshot.twap_minutes >= MIN_GROWTH_TWAP_MINUTES, PeraxError::TwapGateNotMet);
    require!(snapshot.liquidity_usd >= MIN_GROWTH_LIQUIDITY_USD, PeraxError::LiquidityGateNotMet);
    require!(snapshot.net_buy_volume_bps >= MIN_NET_BUY_VOLUME_BPS, PeraxError::BuyPressureGateNotMet);
    require!(
        snapshot.observed_at >= state.last_release_timestamp + RELEASE_COOLDOWN_SECONDS || state.last_release_timestamp == 0,
        PeraxError::ReleaseCooldownActive
    );
    require!(
        state.daily_unlocked_accumulator.saturating_add(params.requested_amount) <= state.daily_release_cap,
        PeraxError::DailyReleaseCapExceeded
    );
    require!(
        state.monthly_unlocked_accumulator.saturating_add(params.requested_amount) <= state.monthly_release_cap,
        PeraxError::MonthlyReleaseCapExceeded
    );

    Ok(())
}

fn validate_emergency_release(state: &PeraxState, params: &MarketConditionalReleaseParams) -> Result<()> {
    let snapshot = &params.snapshot;
    require!(
        snapshot.downside_move_bps >= EMERGENCY_DOWNSIDE_TRIGGER_BPS,
        PeraxError::EmergencyDownsideGateNotMet
    );
    require!(
        snapshot.liquidity_drain_bps >= EMERGENCY_LIQUIDITY_DRAIN_TRIGGER_BPS,
        PeraxError::EmergencyLiquidityGateNotMet
    );
    require!(snapshot.emergency_reserve_available_amount > 0, PeraxError::InvalidMarketParameter);

    let hourly_cap = amount_bps(
        snapshot.emergency_reserve_available_amount,
        state.emergency_hourly_release_bps,
    )?;
    require!(params.requested_amount <= hourly_cap, PeraxError::EmergencyHourlyCapExceeded);
    require!(
        state.daily_unlocked_accumulator.saturating_add(params.requested_amount) <= state.daily_release_cap,
        PeraxError::DailyReleaseCapExceeded
    );
    require!(
        state.monthly_unlocked_accumulator.saturating_add(params.requested_amount) <= state.monthly_release_cap,
        PeraxError::MonthlyReleaseCapExceeded
    );

    Ok(())
}

fn reset_release_windows_if_needed(state: &mut PeraxState, observed_at: i64) {
    if state.daily_window_start == 0 || observed_at >= state.daily_window_start + 86_400 {
        state.daily_window_start = observed_at;
        state.daily_unlocked_accumulator = 0;
    }

    if state.monthly_window_start == 0 || observed_at >= state.monthly_window_start + 2_592_000 {
        state.monthly_window_start = observed_at;
        state.monthly_unlocked_accumulator = 0;
    }
}

fn amount_bps(amount: u64, bps: u16) -> Result<u64> {
    let result = (amount as u128)
        .checked_mul(bps as u128)
        .ok_or(PeraxError::InvalidMarketParameter)?
        .checked_div(10_000)
        .ok_or(PeraxError::InvalidMarketParameter)?;
    u64::try_from(result).map_err(|_| PeraxError::InvalidMarketParameter.into())
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct InitializeParams {
    pub token_mint: Pubkey,
    pub trading_company_token_account: Pubkey,
    pub trading_company_revenue_token_account: Pubkey,
    pub max_payment_amount: u64,
    pub safety_admin: Pubkey,
    pub oracle_feed: Pubkey,
    pub launch_price: u64,
    pub current_stepped_floor: u64,
    pub daily_release_cap: u64,
    pub monthly_release_cap: u64,
    pub emergency_hourly_release_bps: u16,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct UpdateConfigParams {
    pub trading_company_token_account: Option<Pubkey>,
    pub trading_company_revenue_token_account: Option<Pubkey>,
    pub max_payment_amount: Option<u64>,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct UpdateMarketEngineConfigParams {
    pub safety_admin: Option<Pubkey>,
    pub oracle_feed: Option<Pubkey>,
    pub current_stepped_floor: Option<u64>,
    pub daily_release_cap: Option<u64>,
    pub monthly_release_cap: Option<u64>,
    pub emergency_hourly_release_bps: Option<u16>,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug)]
pub enum ReleaseType {
    Growth,
    Emergency,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct MarketConditionSnapshot {
    pub observed_price: u64,
    pub twap_minutes: u64,
    pub liquidity_usd: u64,
    pub net_buy_volume_bps: u16,
    pub downside_move_bps: u16,
    pub liquidity_drain_bps: u16,
    pub emergency_reserve_available_amount: u64,
    pub observed_at: i64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct MarketConditionalReleaseParams {
    pub release_type: ReleaseType,
    pub requested_amount: u64,
    pub release_id: [u8; 32],
    pub snapshot: MarketConditionSnapshot,
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

#[derive(Accounts)]
pub struct UpdateConfig<'info> {
    #[account(
        mut,
        seeds = [b"perax-state"],
        bump = state.bump,
        has_one = authority @ PeraxError::Unauthorized
    )]
    pub state: Account<'info, PeraxState>,

    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct SafetyAdminAction<'info> {
    #[account(
        mut,
        seeds = [b"perax-state"],
        bump = state.bump,
        has_one = safety_admin @ PeraxError::Unauthorized
    )]
    pub state: Account<'info, PeraxState>,

    pub safety_admin: Signer<'info>,
}

#[derive(Accounts)]
pub struct RecordMarketConditionalRelease<'info> {
    #[account(
        mut,
        seeds = [b"perax-state"],
        bump = state.bump,
        has_one = oracle_feed @ PeraxError::Unauthorized
    )]
    pub state: Account<'info, PeraxState>,

    pub oracle_feed: Signer<'info>,
}

#[derive(Accounts)]
pub struct AcceptAuthority<'info> {
    #[account(
        mut,
        seeds = [b"perax-state"],
        bump = state.bump
    )]
    pub state: Account<'info, PeraxState>,

    pub pending_authority: Signer<'info>,
}

#[derive(Accounts)]
#[instruction(amount: u64, reference: [u8; 32])]
pub struct PayToTradingCompany<'info> {
    #[account(
        seeds = [b"perax-state"],
        bump = state.bump,
        constraint = token_mint.key() == state.token_mint @ PeraxError::InvalidTokenMint,
        constraint = trading_company_revenue_token_account.key() == state.trading_company_revenue_token_account @ PeraxError::InvalidTradingCompanyRevenueAccount
    )]
    pub state: Account<'info, PeraxState>,

    #[account(
        init,
        payer = payer,
        space = 8 + PaymentRecord::SPACE,
        seeds = [b"payment", reference.as_ref()],
        bump
    )]
    pub payment_record: Account<'info, PaymentRecord>,

    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        mut,
        constraint = payer_token_account.owner == payer.key() @ PeraxError::Unauthorized,
        constraint = payer_token_account.mint == token_mint.key() @ PeraxError::InvalidTokenMint
    )]
    pub payer_token_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        constraint = trading_company_revenue_token_account.mint == token_mint.key() @ PeraxError::InvalidTokenMint
    )]
    pub trading_company_revenue_token_account: Account<'info, TokenAccount>,

    pub token_mint: Account<'info, Mint>,

    pub token_program: Program<'info, Token>,

    pub system_program: Program<'info, System>,
}

impl<'info> PayToTradingCompany<'info> {
    fn payment_transfer_ctx(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        let accounts = Transfer {
            from: self.payer_token_account.to_account_info(),
            to: self.trading_company_revenue_token_account.to_account_info(),
            authority: self.payer.to_account_info(),
        };
        CpiContext::new(self.token_program.to_account_info(), accounts)
    }
}

#[derive(Accounts)]
pub struct RecordExternalUtilityPayment<'info> {
    #[account(
        seeds = [b"perax-state"],
        bump = state.bump,
        has_one = authority @ PeraxError::Unauthorized
    )]
    pub state: Account<'info, PeraxState>,

    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct BurnFromTradingCompany<'info> {
    #[account(
        seeds = [b"perax-state"],
        bump = state.bump,
        has_one = authority @ PeraxError::Unauthorized,
        constraint = token_mint.key() == state.token_mint @ PeraxError::InvalidTokenMint,
        constraint = trading_company_revenue_token_account.key() == state.trading_company_revenue_token_account @ PeraxError::InvalidTradingCompanyRevenueAccount
    )]
    pub state: Account<'info, PeraxState>,

    pub authority: Signer<'info>,

    pub trading_company_authority: Signer<'info>,

    #[account(
        mut,
        constraint = trading_company_revenue_token_account.owner == trading_company_authority.key() @ PeraxError::Unauthorized,
        constraint = trading_company_revenue_token_account.mint == token_mint.key() @ PeraxError::InvalidTokenMint
    )]
    pub trading_company_revenue_token_account: Account<'info, TokenAccount>,

    #[account(mut)]
    pub token_mint: Account<'info, Mint>,

    pub token_program: Program<'info, Token>,
}

impl<'info> BurnFromTradingCompany<'info> {
    fn burn_ctx(&self) -> CpiContext<'_, '_, '_, 'info, Burn<'info>> {
        let accounts = Burn {
            mint: self.token_mint.to_account_info(),
            from: self.trading_company_revenue_token_account.to_account_info(),
            authority: self.trading_company_authority.to_account_info(),
        };
        CpiContext::new(self.token_program.to_account_info(), accounts)
    }
}

#[account]
#[derive(InitSpace)]
pub struct PeraxState {
    pub authority: Pubkey,
    pub pending_authority: Pubkey,
    pub has_pending_authority: bool,
    pub token_mint: Pubkey,
    pub trading_company_token_account: Pubkey,
    pub trading_company_revenue_token_account: Pubkey,
    pub max_payment_amount: u64,
    pub safety_admin: Pubkey,
    pub oracle_feed: Pubkey,
    pub launch_price: u64,
    pub current_stepped_floor: u64,
    pub last_release_timestamp: i64,
    pub daily_unlocked_accumulator: u64,
    pub monthly_unlocked_accumulator: u64,
    pub daily_window_start: i64,
    pub monthly_window_start: i64,
    pub daily_release_cap: u64,
    pub monthly_release_cap: u64,
    pub emergency_hourly_release_bps: u16,
    pub is_paused: bool,
    pub emergency_pause: bool,
    pub bump: u8,
}

#[account]
pub struct PaymentRecord {
    pub reference: [u8; 32],
    pub payer: Pubkey,
    pub amount: u64,
    pub token_mint: Pubkey,
    pub trading_company_token_account: Pubkey,
    pub trading_company_revenue_token_account: Pubkey,
    pub created_at: i64,
    pub bump: u8,
}

impl PaymentRecord {
    pub const SPACE: usize = 32 + 32 + 8 + 32 + 32 + 32 + 8 + 1;
}

#[event]
pub struct ConfigInitialized {
    pub authority: Pubkey,
    pub token_mint: Pubkey,
    pub trading_company_token_account: Pubkey,
    pub trading_company_revenue_token_account: Pubkey,
    pub safety_admin: Pubkey,
    pub oracle_feed: Pubkey,
    pub launch_price: u64,
    pub daily_release_cap: u64,
    pub monthly_release_cap: u64,
    pub max_payment_amount: u64,
}

#[event]
pub struct ConfigUpdated {
    pub authority: Pubkey,
    pub trading_company_token_account: Pubkey,
    pub trading_company_revenue_token_account: Pubkey,
    pub max_payment_amount: u64,
}

#[event]
pub struct MarketEngineConfigUpdated {
    pub authority: Pubkey,
    pub safety_admin: Pubkey,
    pub oracle_feed: Pubkey,
    pub current_stepped_floor: u64,
    pub daily_release_cap: u64,
    pub monthly_release_cap: u64,
    pub emergency_hourly_release_bps: u16,
}

#[event]
pub struct PauseStatusChanged {
    pub authority: Pubkey,
    pub is_paused: bool,
}

#[event]
pub struct EmergencyPauseStatusChanged {
    pub safety_admin: Pubkey,
    pub emergency_pause: bool,
}

#[event]
pub struct MarketConditionalReleaseApproved {
    pub oracle_feed: Pubkey,
    pub release_type: ReleaseType,
    pub requested_amount: u64,
    pub release_id: [u8; 32],
    pub observed_price: u64,
    pub twap_minutes: u64,
    pub liquidity_usd: u64,
    pub net_buy_volume_bps: u16,
    pub daily_unlocked_accumulator: u64,
    pub monthly_unlocked_accumulator: u64,
    pub observed_at: i64,
}

#[event]
pub struct AuthorityTransferNominated {
    pub current_authority: Pubkey,
    pub pending_authority: Pubkey,
}

#[event]
pub struct AuthorityTransferCancelled {
    pub authority: Pubkey,
    pub cancelled_authority: Pubkey,
}

#[event]
pub struct AuthorityTransferAccepted {
    pub previous_authority: Pubkey,
    pub new_authority: Pubkey,
}

#[event]
pub struct UtilityPaymentReceived {
    pub payer: Pubkey,
    pub token_mint: Pubkey,
    pub trading_company_token_account: Pubkey,
    pub trading_company_revenue_token_account: Pubkey,
    pub amount,
    pub reference: [u8; 32],
}

#[event]
pub struct ExternalUtilityPaymentRecorded {
    pub authority: Pubkey,
    pub token_mint: Pubkey,
    pub amount: u64,
    pub reference: [u8; 32],
    pub payment_source: [u8; 16],
}

#[event]
pub struct TradingCompanyBurnExecuted {
    pub authority: Pubkey,
    pub trading_company_authority: Pubkey,
    pub token_mint: Pubkey,
    pub trading_company_token_account: Pubkey,
    pub trading_company_revenue_token_account: Pubkey,
    pub amount: u64,
    pub decision_id: [u8; 32],
}

#[error_code]
pub enum PeraxError {
    #[msg("The caller is not authorized to perform this action.")]
    Unauthorized,
    #[msg("The program is currently paused.")]
    ProgramPaused,
    #[msg("The market-conditional engine is currently under emergency pause.")]
    EmergencyPaused,
    #[msg("Amount must be greater than zero.")]
    InvalidAmount,
    #[msg("The token mint does not match the configured Pera-X mint.")]
    InvalidTokenMint,
    #[msg("The trading company locked token account does not match the configured account.")]
    InvalidTradingCompanyAccount,
    #[msg("The trading company revenue token account does not match the configured account.")]
    InvalidTradingCompanyRevenueAccount,
    #[msg("Trading company locked and revenue token accounts must be different.")]
    TradingCompanyAccountsMustDiffer,
    #[msg("The payment amount is above the configured maximum payment amount.")]
    PaymentAmountTooLarge,
    #[msg("The new authority is invalid.")]
    InvalidAuthority,
    #[msg("The safety admin is invalid.")]
    InvalidSafetyAdmin,
    #[msg("The oracle feed is invalid.")]
    InvalidOracleFeed,
    #[msg("There is no pending authority transfer.")]
    NoPendingAuthority,
    #[msg("The payment or decision reference is invalid.")]
    InvalidReference,
    #[msg("A market engine parameter is invalid.")]
    InvalidMarketParameter,
    #[msg("Growth price gate was not met.")]
    GrowthPriceGateNotMet,
    #[msg("TWAP confirmation gate was not met.")]
    TwapGateNotMet,
    #[msg("Liquidity depth gate was not met.")]
    LiquidityGateNotMet,
    #[msg("Net buy pressure gate was not met.")]
    BuyPressureGateNotMet,
    #[msg("Release cooldown is still active.")]
    ReleaseCooldownActive,
    #[msg("Daily release cap exceeded.")]
    DailyReleaseCapExceeded,
    #[msg("Monthly release cap exceeded.")]
    MonthlyReleaseCapExceeded,
    #[msg("Release cap arithmetic overflowed or was exceeded.")]
    ReleaseCapExceeded,
    #[msg("Emergency downside trigger was not met.")]
    EmergencyDownsideGateNotMet,
    #[msg("Emergency liquidity-drain trigger was not met.")]
    EmergencyLiquidityGateNotMet,
    #[msg("Emergency hourly release cap exceeded.")]
    EmergencyHourlyCapExceeded,
}

#[cfg(test)]
mod tests;
