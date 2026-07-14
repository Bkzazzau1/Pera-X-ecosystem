use anchor_lang::prelude::*;

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
    #[msg("The payment, allocation, observation, or decision reference is invalid.")]
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
    #[msg("Legacy approval-only release is disabled. Use execute_market_conditional_release with a program-controlled vault.")]
    UseVaultControlledRelease,
    #[msg("The allocation ID is not one of the approved Pera-X allocations.")]
    UnknownAllocationId,
    #[msg("The selected vault class does not match the approved allocation.")]
    UnsupportedVaultClass,
    #[msg("The vault allocation cap is invalid.")]
    InvalidAllocationCap,
    #[msg("The requested cap or deposit exceeds the approved allocation cap.")]
    AllocationCapExceeded,
    #[msg("The reserve vault configuration is invalid.")]
    InvalidVaultConfiguration,
    #[msg("The reserve vault authority PDA is invalid.")]
    InvalidVaultAuthority,
    #[msg("The reserve vault token account is invalid.")]
    InvalidVaultTokenAccount,
    #[msg("The reserve vault is inactive.")]
    VaultInactive,
    #[msg("The reserve vault is paused.")]
    VaultPaused,
    #[msg("This vault class cannot use the market-conditional release route.")]
    VaultClassNotMarketReleasable,
    #[msg("The reserve vault does not have enough authoritative available PEX.")]
    InsufficientVaultBalance,
    #[msg("The release destination does not match the destination signed by the oracle bot.")]
    InvalidReleaseDestination,
    #[msg("Vault accounting is inconsistent with deposited and released totals.")]
    VaultAccountingMismatch,
    #[msg("Vault accounting arithmetic overflowed.")]
    VaultAccountingOverflow,
    #[msg("The bot-reported emergency reserve balance does not match the authoritative vault balance.")]
    VaultBalanceObservationMismatch,
    #[msg("Legacy burn is disabled. Use execute_market_condition_burn or execute_conditional_buyback_burn instead.")]
    UseMarketConditionBurn,
    #[msg("Burn rate is outside the approved market-condition policy.")]
    InvalidBurnRate,
    #[msg("Burn amount does not match the approved market-condition burn rate.")]
    BurnAmountMismatch,
    #[msg("Daily burn cap exceeded.")]
    DailyBurnCapExceeded,
    #[msg("Market health score must be between 0 and 100.")]
    InvalidMarketHealthScore,
    #[msg("The selected burn source account is not approved for this burn source.")]
    InvalidBurnSourceAccount,
}
