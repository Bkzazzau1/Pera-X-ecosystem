from pathlib import Path
import json
import re

ROOT = Path.cwd()


def replace_exact(path: Path, old: str, new: str, expected: int = 1) -> None:
    text = path.read_text()
    count = text.count(old)
    if count != expected:
        raise SystemExit(f"{path}: expected {expected} matches for {old!r}, found {count}")
    path.write_text(text.replace(old, new))


lib = ROOT / "perax-contracts/programs/perax-core/src/lib.rs"
replace_exact(lib, "mod instructions;\nmod settlement;", "mod instructions;\nmod market_cpi;\nmod settlement;")
replace_exact(lib, "pub use events::*;\npub use settlement::*;", "pub use events::*;\npub(crate) use market_cpi::*;\npub use settlement::*;")

market_cpi = ROOT / "perax-contracts/programs/perax-core/src/market_cpi.rs"
replace_exact(
    market_cpi,
    "use anchor_lang::prelude::*;\nuse anchor_lang::solana_program::instruction::AccountMeta;",
    "use crate::{APC_BPS_DENOMINATOR, APC_QUOTE_DECIMALS, PEX_DECIMALS};\nuse anchor_lang::prelude::*;\nuse anchor_lang::solana_program::instruction::AccountMeta;",
)
minimum_output_helper = r'''pub(crate) fn minimum_pex_out_for_quote(
    maximum_quote_amount: u64,
    effective_price: u64,
    price_scale: u64,
    maximum_slippage_bps: u16,
) -> Option<u64> {
    if maximum_quote_amount == 0
        || effective_price == 0
        || price_scale == 0
        || maximum_slippage_bps == 0
        || u128::from(maximum_slippage_bps) >= APC_BPS_DENOMINATOR
    {
        return None;
    }
    let quote_scale = 10u128.checked_pow(u32::from(APC_QUOTE_DECIMALS))?;
    let numerator = u128::from(maximum_quote_amount)
        .checked_mul(u128::from(PEX_DECIMALS))?
        .checked_mul(u128::from(price_scale))?;
    let denominator = quote_scale.checked_mul(u128::from(effective_price))?;
    let fair_output = numerator.checked_add(denominator.checked_sub(1)?)?.checked_div(denominator)?;
    let retained_bps = APC_BPS_DENOMINATOR.checked_sub(u128::from(maximum_slippage_bps))?;
    let minimum = fair_output.checked_mul(retained_bps)?.checked_div(APC_BPS_DENOMINATOR)?;
    let minimum = u64::try_from(minimum).ok()?;
    (minimum > 0).then_some(minimum)
}

'''
replace_exact(market_cpi, "fn require_account(\n", minimum_output_helper + "fn require_account(\n")
replace_exact(
    market_cpi,
    "    #[test]\n    fn rejects_host_fee_wrong_destination_and_extra_signer() {",
    '''    #[test]
    fn derives_policy_bounded_recovery_output() {
        assert_eq!(minimum_pex_out_for_quote(1_000_000, 100_000_000, 100_000_000, 500), Some(950_000));
        assert_eq!(minimum_pex_out_for_quote(1_000_000, 100_000_000, 100_000_000, 0), None);
        assert_eq!(minimum_pex_out_for_quote(1_000_000, 100_000_000, 100_000_000, 10_000), None);
    }

    #[test]
    fn rejects_host_fee_wrong_destination_and_extra_signer() {''',
)

settlement = ROOT / "perax-contracts/programs/perax-core/src/instructions/settlement_v2.rs"
replace_exact(
    settlement,
    "    calculate_apc_risk_tier, calculate_effective_apc_price, calculate_vault_available_amount,\n",
    "    calculate_apc_risk_tier, calculate_effective_apc_price, calculate_vault_available_amount,\n    validated_exact_out_market_metas, ExactOutMarketValidation,\n",
)
replace_exact(
    settlement,
    "    instruction::{AccountMeta, Instruction},\n",
    "    instruction::Instruction,\n",
)
replace_exact(
    settlement,
    "        params.minimum_pex_out >= market_remaining,\n",
    "        params.minimum_pex_out == market_remaining,\n",
)
old_settlement_cpi = '''    let quote_before = ctx.accounts.quote_source_token_account.amount;
    let pex_before = ctx.accounts.settlement_pex_vault.amount;
    let mut metas = vec![
        AccountMeta::new(ctx.accounts.quote_source_token_account.key(), false),
        AccountMeta::new(ctx.accounts.settlement_pex_vault.key(), false),
        AccountMeta::new_readonly(ctx.accounts.quote_source_authority.key(), true),
        AccountMeta::new(ctx.accounts.approved_market_pool.key(), false),
        AccountMeta::new_readonly(ctx.accounts.token_program.key(), false),
    ];
    let mut infos = vec![
        ctx.accounts.quote_source_token_account.to_account_info(),
        ctx.accounts.settlement_pex_vault.to_account_info(),
        ctx.accounts.quote_source_authority.to_account_info(),
        ctx.accounts.approved_market_pool.to_account_info(),
        ctx.accounts.token_program.to_account_info(),
    ];
    for account in ctx.remaining_accounts {
        metas.push(if account.is_writable {
            AccountMeta::new(account.key(), account.is_signer)
        } else {
            AccountMeta::new_readonly(account.key(), account.is_signer)
        });
        infos.push(account.clone());
    }
    infos.push(ctx.accounts.market_program.to_account_info());
    invoke(
        &Instruction {
            program_id: ctx.accounts.market_program.key(),
            accounts: metas,
            data: params.swap_instruction_data,
        },
        &infos,
    )?;
'''
new_settlement_cpi = '''    let quote_before = ctx.accounts.quote_source_token_account.amount;
    let pex_before = ctx.accounts.settlement_pex_vault.amount;
    let metas = validated_exact_out_market_metas(
        ctx.remaining_accounts,
        &params.swap_instruction_data,
        ExactOutMarketValidation {
            market_program: ctx.accounts.market_program.key(),
            approved_pool: ctx.accounts.approved_market_pool.key(),
            quote_source: ctx.accounts.quote_source_token_account.key(),
            pex_destination: ctx.accounts.settlement_pex_vault.key(),
            authority: ctx.accounts.quote_source_authority.key(),
            quote_mint: ctx.accounts.quote_mint.key(),
            pex_mint: ctx.accounts.pex_mint.key(),
            token_program: ctx.accounts.token_program.key(),
            maximum_quote_amount: params.maximum_quote_amount,
            exact_pex_out: market_remaining,
            authority_is_pda: false,
        },
    )
    .ok_or_else(|| error!(SettlementError::InvalidMarketSettlement))?;
    let mut infos: Vec<AccountInfo<'info>> = ctx.remaining_accounts.to_vec();
    infos.push(ctx.accounts.market_program.to_account_info());
    invoke(
        &Instruction {
            program_id: ctx.accounts.market_program.key(),
            accounts: metas,
            data: params.swap_instruction_data,
        },
        &infos,
    )?;
'''
replace_exact(settlement, old_settlement_cpi, new_settlement_cpi)

recovery = ROOT / "perax-contracts/programs/perax-core/src/instructions/recovery.rs"
replace_exact(
    recovery,
    "    calculate_effective_apc_price, calculate_recovery_pex_out, reset_recovery_window_if_needed,\n",
    "    calculate_effective_apc_price, calculate_recovery_pex_out, minimum_pex_out_for_quote,\n    reset_recovery_window_if_needed, validated_exact_out_market_metas, ExactOutMarketValidation,\n",
)
replace_exact(
    recovery,
    "    instruction::{AccountMeta, Instruction},\n",
    "    instruction::Instruction,\n",
)
recovery_price_guard = '''    require!(
        observed_price < ctx.accounts.apc_state.current_reference_price,
        PeraxError::RecoveryNotActive
    );

    reset_recovery_window_if_needed(&ctx.accounts.apc_config, &mut ctx.accounts.apc_state, now);
'''
recovery_price_guard_new = '''    require!(
        observed_price < ctx.accounts.apc_state.current_reference_price,
        PeraxError::RecoveryNotActive
    );
    let policy_minimum_pex_out = minimum_pex_out_for_quote(
        params.maximum_quote_amount,
        observed_price,
        ctx.accounts.apc_config.price_scale,
        ctx.accounts.apc_config.maximum_recovery_slippage_bps,
    )
    .ok_or(PeraxError::InvalidRecoverySettlement)?;
    require!(
        params.minimum_pex_out >= policy_minimum_pex_out,
        PeraxError::InvalidRecoverySettlement
    );

    reset_recovery_window_if_needed(&ctx.accounts.apc_config, &mut ctx.accounts.apc_state, now);
'''
replace_exact(recovery, recovery_price_guard, recovery_price_guard_new)
old_recovery_cpi = '''    let mut metas = vec![
        AccountMeta::new(ctx.accounts.counterweight_vault.key(), false),
        AccountMeta::new(ctx.accounts.recovery_vault.key(), false),
        AccountMeta::new_readonly(ctx.accounts.counterweight_authority.key(), true),
        AccountMeta::new(ctx.accounts.approved_pool.key(), false),
        AccountMeta::new_readonly(ctx.accounts.token_program.key(), false),
    ];
    let mut infos = vec![
        ctx.accounts.counterweight_vault.to_account_info(),
        ctx.accounts.recovery_vault.to_account_info(),
        ctx.accounts.counterweight_authority.to_account_info(),
        ctx.accounts.approved_pool.to_account_info(),
        ctx.accounts.token_program.to_account_info(),
    ];
    for account in ctx.remaining_accounts {
        let meta = if account.is_writable {
            AccountMeta::new(account.key(), account.is_signer)
        } else {
            AccountMeta::new_readonly(account.key(), account.is_signer)
        };
        metas.push(meta);
        infos.push(account.clone());
    }
    infos.push(ctx.accounts.recovery_program.to_account_info());

    let instruction = Instruction {
        program_id: ctx.accounts.recovery_program.key(),
        accounts: metas,
        data: params.swap_instruction_data,
    };
'''
new_recovery_cpi = '''    let metas = validated_exact_out_market_metas(
        ctx.remaining_accounts,
        &params.swap_instruction_data,
        ExactOutMarketValidation {
            market_program: ctx.accounts.recovery_program.key(),
            approved_pool: ctx.accounts.approved_pool.key(),
            quote_source: ctx.accounts.counterweight_vault.key(),
            pex_destination: ctx.accounts.recovery_vault.key(),
            authority: ctx.accounts.counterweight_authority.key(),
            quote_mint: ctx.accounts.quote_mint.key(),
            pex_mint: ctx.accounts.pex_mint.key(),
            token_program: ctx.accounts.token_program.key(),
            maximum_quote_amount: params.maximum_quote_amount,
            exact_pex_out: params.minimum_pex_out,
            authority_is_pda: true,
        },
    )
    .ok_or_else(|| error!(PeraxError::InvalidRecoverySettlement))?;
    let mut infos: Vec<AccountInfo<'info>> = ctx.remaining_accounts.to_vec();
    infos.push(ctx.accounts.recovery_program.to_account_info());

    let instruction = Instruction {
        program_id: ctx.accounts.recovery_program.key(),
        accounts: metas,
        data: params.swap_instruction_data,
    };
'''
replace_exact(recovery, old_recovery_cpi, new_recovery_cpi)

state = ROOT / "perax-contracts/programs/perax-core/src/state.rs"
replace_exact(
    state,
    "    pub maximum_recovery_purchase_bps: u16,\n",
    "    pub maximum_recovery_slippage_bps: u16,\n    pub maximum_recovery_purchase_bps: u16,\n",
    expected=2,
)

apc = ROOT / "perax-contracts/programs/perax-core/src/instructions/apc.rs"
replace_exact(
    apc,
    "    config.maximum_recovery_purchase_bps = params.maximum_recovery_purchase_bps;\n",
    "    config.maximum_recovery_slippage_bps = params.maximum_recovery_slippage_bps;\n    config.maximum_recovery_purchase_bps = params.maximum_recovery_purchase_bps;\n",
)

validation = ROOT / "perax-contracts/programs/perax-core/src/validation.rs"
replace_exact(
    validation,
    "        params.maximum_recovery_purchase_bps > 0\n",
    "        params.maximum_recovery_slippage_bps > 0\n            && params.maximum_recovery_slippage_bps < 10_000\n            && params.maximum_recovery_purchase_bps > 0\n",
)

# Add the new field to all Rust APC policy fixtures without choosing a production value.
for path in (ROOT / "perax-contracts").rglob("*.rs"):
    if path in {state, apc, validation}:
        continue
    text = path.read_text()
    pattern = re.compile(r"^(?P<indent>\s*)maximum_recovery_purchase_bps:\s*", re.MULTILINE)
    matches = list(pattern.finditer(text))
    if not matches:
        continue
    offset = 0
    for match in matches:
        position = match.start() + offset
        prefix = text[max(0, position - 160):position]
        if "maximum_recovery_slippage_bps" in prefix:
            continue
        indent = match.group("indent")
        insertion = f"{indent}maximum_recovery_slippage_bps: 500,\n"
        text = text[:position] + insertion + text[position:]
        offset += len(insertion)
    path.write_text(text)

# Add camelCase test/planning fixtures if any exist.
for suffix in ("*.ts", "*.js"):
    for path in (ROOT / "perax-contracts").rglob(suffix):
        text = path.read_text()
        pattern = re.compile(r"^(?P<indent>\s*)maximumRecoveryPurchaseBps:\s*(?P<value>[^,\n]+),", re.MULTILINE)
        matches = list(pattern.finditer(text))
        if not matches:
            continue
        offset = 0
        for match in matches:
            position = match.start() + offset
            prefix = text[max(0, position - 180):position]
            if "maximumRecoverySlippageBps" in prefix:
                continue
            value = match.group("value")
            if "maximumPurchaseBps" in value:
                slippage_value = value.replace("maximumPurchaseBps", "maximumSlippageBps")
            else:
                slippage_value = "500"
            insertion = f"{match.group('indent')}maximumRecoverySlippageBps: {slippage_value},\n"
            text = text[:position] + insertion + text[position:]
            offset += len(insertion)
        path.write_text(text)

config_path = ROOT / "perax-contracts/config/pex-tokenomics.json"
config = json.loads(config_path.read_text())
recovery_policy = config["adaptivePriceControl"]["recoveryPolicy"]
if "maximumSlippageBps" in recovery_policy:
    raise SystemExit("maximumSlippageBps already exists")
ordered = {}
for key, value in recovery_policy.items():
    if key == "maximumPurchaseBps":
        ordered["maximumSlippageBps"] = None
    ordered[key] = value
config["adaptivePriceControl"]["recoveryPolicy"] = ordered
unresolved = config["adaptivePriceControl"]["unresolvedNumericalPolicies"]
if "recovery_maximum_slippage" not in unresolved:
    unresolved.append("recovery_maximum_slippage")
config_path.write_text(json.dumps(config, indent=2) + "\n")

validator = ROOT / "perax-contracts/scripts/validate-tokenomics.js"
replace_exact(
    validator,
    "    assert(Number.isInteger(apc.recoveryPolicy.maximumPurchaseBps) && apc.recoveryPolicy.maximumPurchaseBps > 0 && apc.recoveryPolicy.maximumPurchaseBps < 10000, 'Approved recovery purchase percentage is invalid.');\n",
    "    assert(Number.isInteger(apc.recoveryPolicy.maximumSlippageBps) && apc.recoveryPolicy.maximumSlippageBps > 0 && apc.recoveryPolicy.maximumSlippageBps < 10000, 'Approved recovery slippage limit is invalid.');\n    assert(Number.isInteger(apc.recoveryPolicy.maximumPurchaseBps) && apc.recoveryPolicy.maximumPurchaseBps > 0 && apc.recoveryPolicy.maximumPurchaseBps < 10000, 'Approved recovery purchase percentage is invalid.');\n",
)
replace_exact(
    validator,
    "    assert(apc.recoveryPolicy.maximumPurchaseBps === null && apc.recoveryPolicy.minimumReserveBps === null && apc.recoveryPolicy.windowCapAmount === null && apc.recoveryPolicy.windowSeconds === null && apc.recoveryPolicy.cooldownSeconds === null, 'Pending recovery limits must remain null.');\n",
    "    assert(apc.recoveryPolicy.maximumSlippageBps === null && apc.recoveryPolicy.maximumPurchaseBps === null && apc.recoveryPolicy.minimumReserveBps === null && apc.recoveryPolicy.windowCapAmount === null && apc.recoveryPolicy.windowSeconds === null && apc.recoveryPolicy.cooldownSeconds === null, 'Pending recovery limits must remain null.');\n",
)
replace_exact(
    validator,
    "  assert(Array.isArray(apc.unresolvedNumericalPolicies) && apc.unresolvedNumericalPolicies.length === 10, 'All ten unresolved numerical policies must be listed.');\n",
    "  if (apc.policyStatus === 'approved') {\n    assert(Array.isArray(apc.unresolvedNumericalPolicies) && apc.unresolvedNumericalPolicies.length === 0, 'Approved APC policy cannot retain unresolved numerical policies.');\n  } else {\n    assert(Array.isArray(apc.unresolvedNumericalPolicies) && apc.unresolvedNumericalPolicies.length === 11, 'All eleven unresolved numerical policies must be listed.');\n  }\n",
)

guard = ROOT / "perax-contracts/scripts/validate-settlement-source.js"
replace_exact(
    guard,
    'const handlers = read("programs/perax-core/src/instructions/settlement_v2.rs");\n',
    'const handlers = read("programs/perax-core/src/instructions/settlement_v2.rs");\nconst recoveryHandler = read("programs/perax-core/src/instructions/recovery.rs");\nconst marketCpi = read("programs/perax-core/src/market_cpi.rs");\n',
)
replace_exact(
    guard,
    'assertContains(handlers, "program::invoke", "settlement handler");\n',
    'assertContains(handlers, "program::invoke", "settlement handler");\nassertContains(handlers, "validated_exact_out_market_metas", "settlement handler");\nassertNotContains(handlers, "let mut metas = vec![", "settlement handler");\nassertContains(recoveryHandler, "minimum_pex_out_for_quote", "recovery handler");\nassertContains(recoveryHandler, "validated_exact_out_market_metas", "recovery handler");\nassertNotContains(recoveryHandler, "let mut metas = vec![", "recovery handler");\nassertContains(marketCpi, "METEORA_SWAP_EXACT_OUT2_DISCRIMINATOR", "market CPI validation");\nassertContains(marketCpi, "transfer_hook_slice_count != 0", "market CPI validation");\nassertContains(marketCpi, "Host fees are deliberately forbidden", "market CPI validation");\n',
)

print("Applied direct approved-market CPI validation and recovery slippage policy hardening.")
