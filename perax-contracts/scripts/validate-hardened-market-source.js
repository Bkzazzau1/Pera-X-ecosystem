import fs from "node:fs";
import path from "node:path";
import process from "node:process";

const root = process.cwd();
const read = (relativePath) =>
  fs.readFileSync(path.join(root, relativePath), "utf8");
const assertContains = (text, expected, label) => {
  if (!text.includes(expected)) {
    throw new Error(`${label}: missing hardened market requirement: ${expected}`);
  }
};
const assertNotContains = (text, forbidden, label) => {
  if (text.includes(forbidden)) {
    throw new Error(`${label}: forbidden hardened market pattern: ${forbidden}`);
  }
};

const modules = read("programs/perax-core/src/instructions/mod.rs");
const hardened = read("programs/perax-core/src/instructions/hardened_market.rs");
const product = read("programs/perax-core/src/instructions/hardened_product.rs");
const validator = read("programs/perax-core/src/market_cpi.rs");

assertContains(modules, "mod hardened_market;", "instructions/mod.rs");
assertContains(modules, "mod hardened_product;", "instructions/mod.rs");
assertContains(modules, 'path = "../market_cpi.rs"', "instructions/mod.rs");
assertContains(
  modules,
  "execute_counterweight_purchase_hardened as execute_counterweight_purchase",
  "instructions/mod.rs",
);
assertContains(
  modules,
  "execute_settlement_market_purchase_hardened as execute_settlement_market_purchase",
  "instructions/mod.rs",
);
assertContains(
  modules,
  "update_product_settlement_policy_hardened as update_product_settlement_policy",
  "instructions/mod.rs",
);
assertNotContains(modules, "pub use recovery::*;", "instructions/mod.rs");

assertContains(
  hardened,
  "params.minimum_pex_out == market_remaining",
  "hardened settlement",
);
assertContains(
  hardened,
  "validated_exact_out_market_metas",
  "hardened market handlers",
);
assertContains(
  hardened,
  "load_recovery_market_policy",
  "hardened recovery",
);
assertContains(
  hardened,
  "minimum_pex_out_for_quote",
  "hardened recovery",
);
assertContains(
  hardened,
  "policy.maximum_market_slippage_bps",
  "hardened recovery",
);
assertContains(
  hardened,
  "authority_is_pda: true",
  "hardened recovery",
);
assertNotContains(hardened, "let mut metas = vec![", "hardened handlers");

assertContains(
  product,
  "requested_disposition == current.disposition",
  "hardened product policy",
);
assertContains(
  product,
  "policy.disposition",
  "hardened product policy",
);
assertNotContains(
  product,
  "policy.disposition =",
  "hardened product policy",
);

assertContains(
  validator,
  "METEORA_SWAP_EXACT_OUT2_DISCRIMINATOR",
  "market CPI validator",
);
assertContains(
  validator,
  "transfer_hook_slice_count != 0",
  "market CPI validator",
);
assertContains(
  validator,
  "Host fees are deliberately forbidden",
  "market CPI validator",
);
assertContains(
  validator,
  "index != 10 && account.is_signer",
  "market CPI validator",
);
assertContains(
  validator,
  "expected.quote_source",
  "market CPI validator",
);
assertContains(
  validator,
  "expected.pex_destination",
  "market CPI validator",
);

console.log("Hardened market and immutable product policy source guards passed.");
