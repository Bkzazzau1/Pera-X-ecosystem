use anchor_lang::prelude::*;

mod constants;
mod contexts;
mod errors;
mod events;
mod instructions;
mod settlement;
mod settlement_v2;
mod state;
mod validation;

pub use constants::*;
pub use contexts::*;
pub use errors::*;
pub use events::*;
pub use settlement::*;
pub use settlement_v2::*;
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

    pub fn set_emergency_pause(ctx: Context<SafetyAdminAction>, is_paused: bool) -> Result<()> {
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

    pub fn initialize_recovery_pool(
        ctx: Context<InitializeRecoveryPool>,
        params: InitializeRecoveryPoolParams,
    ) -> Result<()> {
        instructions::initialize_recovery_pool(ctx, params)
    }

    pub fn execute_recovery_swap_adapter(
        ctx: Context<RecoverySwapAdapter>,
        params: RecoverySwapAdapterParams,
    ) -> Result<()> {
        instructions::execute_recovery_swap_adapter(ctx, params)
    }

    pub fn initialize_apc(ctx: Context<InitializeApc>, params: InitializeApcParams) -> Result<()> {
        instructions::initialize_apc(ctx, params)
    }

    pub fn submit_apc_observation(
        ctx: Context<SubmitApcObservation>,
        params: SubmitApcObservationParams,
    ) -> Result<()> {
        instructions::submit_apc_observation(ctx, params)
    }

    pub fn activate_next_apc_band(
        ctx: Context<ActivateNextApcBand>,
        params: ActivateApcBandParams,
    ) -> Result<()> {
        instructions::activate_next_apc_band(ctx, params)
    }

    pub fn execute_apc_release(
        ctx: Context<ExecuteApcRelease>,
        params: ExecuteApcReleaseParams,
    ) -> Result<()> {
        instructions::execute_apc_release(ctx, params)
    }

    pub fn deposit_counterweight_proceeds(
        ctx: Context<DepositCounterweightProceeds>,
        params: DepositCounterweightParams,
    ) -> Result<()> {
        instructions::deposit_counterweight_proceeds(ctx, params)
    }

    pub fn record_deferred_burn(
        ctx: Context<RecordDeferredBurn>,
        params: RecordDeferredBurnParams,
    ) -> Result<()> {
        instructions::record_deferred_burn(ctx, params)
    }

    pub fn execute_deferred_burn(
        ctx: Context<ExecuteDeferredBurn>,
        params: ExecuteDeferredBurnParams,
    ) -> Result<()> {
        instructions::execute_deferred_burn(ctx, params)
    }

    pub fn confirm_apc_absorption(ctx: Context<ConfirmApcAbsorption>) -> Result<()> {
        instructions::confirm_apc_absorption(ctx)
    }

    pub fn enter_apc_recovery(ctx: Context<EnterApcRecovery>) -> Result<()> {
        instructions::enter_apc_recovery(ctx)
    }

    pub fn execute_counterweight_purchase<'info>(
        ctx: Context<'_, '_, '_, 'info, ExecuteCounterweightPurchase<'info>>,
        params: ExecuteCounterweightPurchaseParams,
    ) -> Result<()> {
        instructions::execute_counterweight_purchase(ctx, params)
    }

    pub fn pause_apc(ctx: Context<PauseApc>, is_paused: bool) -> Result<()> {
        instructions::pause_apc(ctx, is_paused)
    }

    pub fn initialize_settlement_policy(
        ctx: Context<InitializeSettlementPolicyV2>,
        params: InitializeSettlementPolicyParams,
    ) -> Result<()> {
        instructions::initialize_settlement_policy(ctx, params)
    }

    pub fn initialize_product_settlement_policy(
        ctx: Context<InitializeProductSettlementPolicy>,
        params: InitializeProductSettlementPolicyParams,
    ) -> Result<()> {
        instructions::initialize_product_settlement_policy(ctx, params)
    }

    pub fn update_product_settlement_policy(
        ctx: Context<UpdateProductSettlementPolicy>,
        params: UpdateProductSettlementPolicyParams,
    ) -> Result<()> {
        instructions::update_product_settlement_policy(ctx, params)
    }

    pub fn plan_settlement(
        ctx: Context<PlanSettlementV2>,
        params: PlanSettlementParams,
    ) -> Result<()> {
        instructions::plan_settlement(ctx, params)
    }

    pub fn fund_direct_pex_settlement(
        ctx: Context<FundDirectPexSettlementV2>,
        params: FundDirectPexSettlementParams,
    ) -> Result<()> {
        instructions::fund_direct_pex_settlement(ctx, params)
    }

    pub fn execute_settlement_market_purchase<'info>(
        ctx: Context<'_, '_, '_, 'info, ExecuteSettlementMarketPurchaseV2<'info>>,
        params: ExecuteSettlementMarketPurchaseParams,
    ) -> Result<()> {
        instructions::execute_settlement_market_purchase(ctx, params)
    }

    pub fn execute_settlement_vault_funding(
        ctx: Context<ExecuteSettlementVaultFundingV2>,
        params: ExecuteSettlementVaultFundingParams,
    ) -> Result<()> {
        instructions::execute_settlement_vault_funding(ctx, params)
    }

    pub fn finalize_settlement(
        ctx: Context<FinalizeSettlementV2>,
        params: FinalizeSettlementParams,
    ) -> Result<()> {
        instructions::finalize_settlement(ctx, params)
    }
}

#[cfg(test)]
mod settlement_tests;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod vault_tests;
