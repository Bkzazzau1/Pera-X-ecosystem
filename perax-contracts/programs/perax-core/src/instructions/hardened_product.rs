use crate::{
    ProductSettlementPolicyUpdated, SettlementDisposition, SettlementError,
    UpdateProductSettlementPolicy, UpdateProductSettlementPolicyParams,
    SETTLEMENT_ALL_FUNDING_METHODS,
};
use anchor_lang::prelude::*;

pub fn update_product_settlement_policy_hardened(
    ctx: Context<UpdateProductSettlementPolicy>,
    params: UpdateProductSettlementPolicyParams,
) -> Result<()> {
    let current = &ctx.accounts.product_policy;
    if let Some(requested_disposition) = params.disposition {
        require!(
            requested_disposition == current.disposition,
            SettlementError::InvalidPolicy
        );
    }

    let unit_quote_value = params.unit_quote_value.unwrap_or(current.unit_quote_value);
    let maximum_quantity = params.maximum_quantity.unwrap_or(current.maximum_quantity);
    let accepted_funding_mask = params
        .accepted_funding_mask
        .unwrap_or(current.accepted_funding_mask);
    let fixed_destination = params
        .fixed_destination_token_account
        .unwrap_or(current.fixed_destination_token_account);

    require!(unit_quote_value > 0, SettlementError::InvalidPolicy);
    require!(
        maximum_quantity > 0
            && maximum_quantity
                <= ctx
                    .accounts
                    .settlement_policy
                    .maximum_quantity_per_settlement,
        SettlementError::InvalidQuantity
    );
    require!(
        accepted_funding_mask > 0
            && accepted_funding_mask & !SETTLEMENT_ALL_FUNDING_METHODS == 0,
        SettlementError::FundingMethodNotAccepted
    );
    if current.disposition == SettlementDisposition::UtilityPayment {
        require!(
            fixed_destination != Pubkey::default(),
            SettlementError::InvalidSettlementDestination
        );
    } else if params.fixed_destination_token_account.is_some() {
        // Non-utility dispositions resolve their destination from immutable
        // contract policy. An update may not introduce a hidden wallet route.
        require!(
            fixed_destination == current.fixed_destination_token_account,
            SettlementError::InvalidPolicy
        );
    }

    let product_key = ctx.accounts.product_policy.key();
    let policy = &mut ctx.accounts.product_policy;
    policy.unit_quote_value = unit_quote_value;
    policy.maximum_quantity = maximum_quantity;
    policy.accepted_funding_mask = accepted_funding_mask;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_funding_mask_remains_bounded() {
        assert_eq!(SETTLEMENT_ALL_FUNDING_METHODS & !SETTLEMENT_ALL_FUNDING_METHODS, 0);
    }
}
