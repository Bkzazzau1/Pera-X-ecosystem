use crate::instructions::{
    calculate_settlement_pex_obligation, calculate_settlement_quote_requirement,
    derive_settlement_source_split,
};
use crate::{ApcStatus, SettlementFundingMethod, SettlementMarketMode};

#[test]
fn direct_pex_never_uses_market_or_policy_vault() {
    let (mode, market, vault) = derive_settlement_source_split(
        SettlementFundingMethod::Pex,
        ApcStatus::PumpControl,
        3_600,
        3_600,
        3,
        [0, 2_500, 5_000, 10_000],
        1_000_000,
    )
    .expect("direct PEX settlement should remain available during pump control");

    assert_eq!(mode, SettlementMarketMode::DirectPex);
    assert_eq!(market, 0);
    assert_eq!(vault, 0);
}

#[test]
fn external_funding_is_blocked_during_pump_control() {
    let result = derive_settlement_source_split(
        SettlementFundingMethod::Stablecoin,
        ApcStatus::PumpControl,
        3_600,
        3_600,
        1,
        [0, 2_500, 5_000, 10_000],
        1_000_000,
    );

    assert!(result.is_err());
}

#[test]
fn risk_policy_derives_hybrid_split_without_bot_choice() {
    let (mode, market, vault) = derive_settlement_source_split(
        SettlementFundingMethod::Fiat,
        ApcStatus::Active,
        4_000,
        3_600,
        2,
        [0, 2_500, 5_000, 10_000],
        1_000_000,
    )
    .expect("risk tier two should use the immutable fifty-fifty test policy");

    assert_eq!(mode, SettlementMarketMode::Hybrid);
    assert_eq!(market, 500_000);
    assert_eq!(vault, 500_000);
}

#[test]
fn price_below_reference_forces_market_purchase() {
    let (mode, market, vault) = derive_settlement_source_split(
        SettlementFundingMethod::VirtualAccount,
        ApcStatus::Active,
        3_000,
        3_600,
        0,
        [0, 2_500, 5_000, 10_000],
        1_000_000,
    )
    .expect("a price below the protected reference should force market sourcing");

    assert_eq!(mode, SettlementMarketMode::MarketPurchase);
    assert_eq!(market, 1_000_000);
    assert_eq!(vault, 0);
}

#[test]
fn quote_to_pex_math_uses_quote_and_pex_decimals() {
    let pex = calculate_settlement_pex_obligation(1_000_000, 1_200, 100_000_000)
        .expect("one dollar quote should convert at the launch price");
    assert_eq!(pex, 83_333_333_334);

    let quote = calculate_settlement_quote_requirement(pex, 1_200, 100_000_000)
        .expect("inverse pricing should recover approximately one quote token");
    assert!(quote >= 1_000_000);
    assert!(quote <= 1_000_001);
}

#[test]
fn recovery_always_uses_market_purchase() {
    let (mode, market, vault) = derive_settlement_source_split(
        SettlementFundingMethod::Stablecoin,
        ApcStatus::Recovery,
        2_500,
        3_600,
        3,
        [0, 2_500, 5_000, 7_500],
        9_000_000,
    )
    .expect("recovery settlement should source PEX from the market");

    assert_eq!(mode, SettlementMarketMode::MarketPurchase);
    assert_eq!(market, 9_000_000);
    assert_eq!(vault, 0);
}
