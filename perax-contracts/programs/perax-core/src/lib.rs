use anchor_lang::prelude::*;

mod constants;
mod contexts;
mod errors;
mod events;
mod instructions;
mod state;
mod validation;

pub use constants::*;
pub use contexts::*;
pub use errors::*;
pub use events::*;
pub use state::*;
pub(crate) use validation::*;

declare_id!("FqEiSx5vujh2vi3yk12NaZMXhjMSaKovGUuzcKiAgshn");

#[program]
pub mod perax_core {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, params: InitializeParams) -> Result<()> {
        instructions::initialize(ctx, params)
    }

    pub fn update_config(ctx: Context<UpdateConfig>, params: UpdateConfigParams) -> Result<()> {
        instructions::update_config(ctx, params)
    }

    pub fn update_market_engine_config(
        ctx: Context<UpdateConfig>,
        params: UpdateMarketEngineConfigParams,
    ) -> Result<()> {
        instructions::update_market_engine_config(ctx, params)
    }

    pub fn set_pause(ctx: Context<UpdateConfig>, is_paused: bool) -> Result<()> {
        instructions::set_pause(ctx, is_paused)
    }

    pub fn set_emergency_pause(
        ctx: Context<SafetyAdminAction>,
        is_paused: bool,
    ) -> Result<()> {
        instructions::set_emergency_pause(ctx, is_paused)
    }

    pub fn initialize_reserve_vault(
        ctx: Context<InitializeReserveVault>,
        params: InitializeReserveVaultParams,
    ) -> Result<()> {
        instructions::initialize_reserve_vault(ctx, params)
    }

    pub fn deposit_into_reserve_vault(
        ctx: Context<DepositIntoReserveVault>,
        allocation_id: [u8; 32],
        amount: u64,
    ) -> Result<()> {
        instructions::deposit_into_reserve_vault(ctx, allocation_id, amount)
    }

    pub fn set_reserve_vault_pause(
        ctx: Context<SetReserveVaultPause>,
        allocation_id: [u8; 32],
        is_paused: bool,
    ) -> Result<()> {
        instructions::set_reserve_vault_pause(ctx, allocation_id, is_paused)
    }

    pub fn reconcile_reserve_vault(
        ctx: Context<ReconcileReserveVault>,
        allocation_id: [u8; 32],
    ) -> Result<()> {
        instructions::reconcile_reserve_vault(ctx, allocation_id)
    }

    pub fn execute_market_conditional_release(
        ctx: Context<ExecuteMarketConditionalRelease>,
        params: VaultMarketConditionalReleaseParams,
    ) -> Result<()> {
        instructions::execute_market_conditional_release(ctx, params)
    }

    pub fn record_market_conditional_release(
        ctx: Context<RecordMarketConditionalRelease>,
        params: MarketConditionalReleaseParams,
    ) -> Result<()> {
        instructions::record_market_conditional_release(ctx, params)
    }

    pub fn nominate_authority(ctx: Context<UpdateConfig>, new_authority: Pubkey) -> Result<()> {
        instructions::nominate_authority(ctx, new_authority)
    }

    pub fn cancel_authority_transfer(ctx: Context<UpdateConfig>) -> Result<()> {
        instructions::cancel_authority_transfer(ctx)
    }

    pub fn accept_authority(ctx: Context<AcceptAuthority>) -> Result<()> {
        instructions::accept_authority(ctx)
    }

    pub fn pay_to_trading_company(
        ctx: Context<PayToTradingCompany>,
        amount: u64,
        reference: [u8; 32],
    ) -> Result<()> {
        instructions::pay_to_trading_company(ctx, amount, reference)
    }

    pub fn record_external_utility_payment(
        ctx: Context<RecordExternalUtilityPayment>,
        amount: u64,
        reference: [u8; 32],
        payment_source: [u8; 16],
    ) -> Result<()> {
        instructions::record_external_utility_payment(ctx, amount, reference, payment_source)
    }

    pub fn burn_from_trading_company(
        ctx: Context<BurnFromTradingCompany>,
        amount: u64,
        decision_id: [u8; 32],
    ) -> Result<()> {
        instructions::burn_from_trading_company(ctx, amount, decision_id)
    }

    pub fn execute_market_condition_burn(
        ctx: Context<ExecuteMarketConditionBurn>,
        params: MarketConditionBurnParams,
    ) -> Result<()> {
        instructions::execute_market_condition_burn(ctx, params)
    }

    pub fn execute_conditional_buyback_burn(
        ctx: Context<ExecuteConditionalBuybackBurn>,
        params: ConditionalBuybackBurnParams,
    ) -> Result<()> {
        instructions::execute_conditional_buyback_burn(ctx, params)
    }
}

#[cfg(test)]
mod tests;
