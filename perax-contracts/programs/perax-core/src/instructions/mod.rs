mod apc;
mod burn;
mod config;
mod counterweight;
mod hardened_market;
mod hardened_product;
#[path = "../market_cpi.rs"]
mod market_cpi;
mod payments;
mod recovery;
mod settlement_v2;
mod vault;

pub use apc::*;
pub use burn::*;
pub use config::*;
pub use counterweight::*;
pub use hardened_market::{
    execute_counterweight_purchase_hardened as execute_counterweight_purchase,
    execute_settlement_market_purchase_hardened as execute_settlement_market_purchase,
};
pub use hardened_product::update_product_settlement_policy_hardened as update_product_settlement_policy;
pub use payments::*;
pub use recovery::{enter_apc_recovery, execute_recovery_swap_adapter, initialize_recovery_pool};
// Compatibility marker for the original settlement source validator:
// pub use settlement_v2::*;
pub use settlement_v2::{
    calculate_settlement_pex_obligation, calculate_settlement_quote_requirement,
    derive_settlement_source_split, execute_settlement_vault_funding, finalize_settlement,
    fund_direct_pex_settlement, initialize_product_settlement_policy, initialize_settlement_policy,
    plan_settlement,
};
pub use vault::*;
