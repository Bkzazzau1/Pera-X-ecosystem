import fs from "node:fs";
import path from "node:path";
import process from "node:process";

const root = process.cwd();
const read = (relativePath) =>
  fs.readFileSync(path.join(root, relativePath), "utf8");
const assertContains = (text, expected, label) => {
  if (!text.includes(expected)) {
    throw new Error(`${label}: missing required source guard: ${expected}`);
  }
};
const assertNotContains = (text, forbidden, label) => {
  if (text.includes(forbidden)) {
    throw new Error(`${label}: forbidden source pattern remains: ${forbidden}`);
  }
};

const lib = read("programs/perax-core/src/lib.rs");
const instructionModules = read("programs/perax-core/src/instructions/mod.rs");
const definitions = read("programs/perax-core/src/settlement.rs");
const contexts = read("programs/perax-core/src/settlement_v2.rs");
const handlers = read("programs/perax-core/src/instructions/settlement_v2.rs");
const errors = read("programs/perax-core/src/errors.rs");
const marketTypes = read("../perax-market-engine/src/types.ts");
const coordinator = read("../perax-market-engine/src/settlement.ts");
const executor = read("../perax-market-engine/src/executor.ts");
const idlGate = read("../perax-market-engine/src/idl.ts");
const anchorClient = read("../perax-market-engine/src/anchor-client.ts");
const runtimeBindings = read("../perax-market-engine/src/runtime.ts");
const runtimeBootstrap = read("../perax-market-engine/src/start-executor.ts");
const marketIndex = read("../perax-market-engine/src/index.ts");
const marketPackage = read("../perax-market-engine/package.json");
const marketLock = read("../perax-market-engine/package-lock.json");
const legacyHandler = path.join(
  root,
  "programs/perax-core/src/instructions/settlement.rs",
);

assertContains(lib, "mod settlement_v2;", "lib.rs");
assertContains(lib, "Context<PlanSettlementV2>", "lib.rs");
assertContains(
  lib,
  "Context<'_, '_, '_, 'info, ExecuteSettlementMarketPurchaseV2<'info>>",
  "lib.rs",
);
assertContains(lib, "Context<FinalizeSettlementV2>", "lib.rs");
assertNotContains(lib, "Context<PlanSettlement>", "lib.rs");

assertContains(instructionModules, "mod settlement_v2;", "instructions/mod.rs");
assertContains(instructionModules, "pub use settlement_v2::*;", "instructions/mod.rs");
assertNotContains(instructionModules, "mod settlement;", "instructions/mod.rs");
if (fs.existsSync(legacyHandler)) {
  throw new Error("superseded shared-custody settlement handler still exists");
}

assertContains(
  contexts,
  'seeds = [b"settlement-custody", params.settlement_id.as_ref()]',
  "settlement_v2.rs",
);
assertContains(
  contexts,
  'seeds = [b"settlement-custody-authority", settlement_record.key().as_ref()]',
  "settlement_v2.rs",
);
assertContains(
  contexts,
  "associated_token::authority = settlement_authority",
  "settlement_v2.rs",
);
assertContains(
  contexts,
  "#[account(mut, address = settlement_policy.approved_market_pool",
  "settlement_v2.rs",
);
assertContains(
  contexts,
  "executable)]\n    pub market_program",
  "settlement_v2.rs",
);
assertContains(
  contexts,
  "reserve_vault_config.vault_class == VaultClass::MarketReserve",
  "settlement_v2.rs",
);

assertContains(handlers, "derive_settlement_source_split", "settlement handler");
assertContains(handlers, "ApcStatus::PumpControl", "settlement handler");
assertContains(handlers, "ApcStatus::Recovery => 10_000", "settlement handler");
assertContains(handlers, "program::invoke", "settlement handler");
assertContains(handlers, "let quote_before", "settlement handler");
assertContains(handlers, "let pex_before", "settlement handler");
assertContains(
  handlers,
  ".checked_sub(ctx.accounts.quote_source_token_account.amount)",
  "settlement handler",
);
assertContains(
  handlers,
  ".checked_sub(pex_before)",
  "settlement handler",
);
assertContains(
  handlers,
  "calculate_vault_available_amount",
  "settlement handler",
);
assertContains(handlers, "SettlementDisposition::Burn", "settlement handler");
assertContains(handlers, "surplus_locked", "settlement handler");
assertNotContains(handlers, "reported_pex_received", "settlement handler");
assertNotContains(handlers, "reported_quote_spent", "settlement handler");

assertContains(
  definitions,
  "pub use crate::PeraxError as SettlementError;",
  "settlement definitions",
);
assertNotContains(definitions, "#[error_code]", "settlement definitions");
assertContains(errors, "InvalidMarketSettlement", "errors.rs");
assertContains(errors, "SettlementNotFunded", "errors.rs");

assertContains(marketTypes, "SettlementProgramClient", "market-engine types");
assertContains(marketTypes, "SettlementObservationProvider", "market-engine types");
assertContains(marketTypes, "SettlementExecutorRequest", "market-engine types");
assertContains(marketTypes, "SettlementRemainingAccount", "market-engine types");
assertContains(marketTypes, "isWritable: boolean", "market-engine types");
assertContains(marketTypes, "settlementRecordAddress?: string", "market-engine types");
assertContains(
  coordinator,
  "switch (settlement.marketMode)",
  "market-engine coordinator",
);
assertContains(
  coordinator,
  'settlement.status === "finalized"',
  "market-engine coordinator",
);
assertContains(
  coordinator,
  'settlement.status === "ready"',
  "market-engine coordinator",
);
assertContains(
  coordinator,
  "purchase.minimumPexOut < remaining",
  "market-engine coordinator",
);
assertNotContains(coordinator, "requestedMarketMode", "market-engine coordinator");
assertNotContains(coordinator, "overrideMarketMode", "market-engine coordinator");

assertContains(executor, "timingSafeEqual", "settlement executor");
assertContains(executor, "getFreshObservationId", "settlement executor");
assertContains(executor, "positiveSafeInteger", "settlement executor");
assertContains(executor, "settlementRecordAddress", "settlement executor");
assertContains(executor, "transactionSignature", "settlement executor");
assertNotContains(executor, "requestedMarketMode", "settlement executor");
assertNotContains(executor, "overrideMarketMode", "settlement executor");

assertContains(idlGate, "assertSettlementIdlCompatible", "settlement IDL gate");
assertContains(idlGate, "settlementCustody", "settlement IDL gate");
assertContains(idlGate, "does not match configured program", "settlement IDL gate");

assertContains(anchorClient, "AnchorSettlementProgramClient", "Anchor settlement client");
assertContains(anchorClient, "assertExistingPlanMatches", "Anchor settlement client");
assertContains(anchorClient, '"settlement-custody-authority"', "Anchor settlement client");
assertContains(anchorClient, "getAssociatedTokenAddressSync", "Anchor settlement client");
assertContains(
  anchorClient,
  "createAssociatedTokenAccountIdempotentInstruction",
  "Anchor settlement client",
);
assertContains(anchorClient, "preInstructions", "Anchor settlement client");
assertContains(anchorClient, "confirmTransaction", "Anchor settlement client");
assertContains(anchorClient, "resolveQuoteSource", "Anchor settlement client");
assertContains(anchorClient, "assertRemainingSignersAvailable", "Anchor settlement client");
assertContains(anchorClient, "latestSignature", "Anchor settlement client");
assertNotContains(anchorClient, "requestedMarketMode", "Anchor settlement client");
assertNotContains(anchorClient, "overrideMarketMode", "Anchor settlement client");

assertContains(
  runtimeBindings,
  "assertSettlementRuntimeBindings",
  "settlement runtime bindings",
);
assertContains(
  runtimeBindings,
  "buildAtomicPexPurchase",
  "settlement runtime bindings",
);
assertContains(
  runtimeBindings,
  "getFreshObservationId",
  "settlement runtime bindings",
);
assertContains(
  runtimeBindings,
  "resolveQuoteSource",
  "settlement runtime bindings",
);

assertContains(
  runtimeBootstrap,
  "PERAX_SETTLEMENT_IDL_PATH",
  "settlement runtime bootstrap",
);
assertContains(
  runtimeBootstrap,
  "PERAX_SETTLEMENT_SIGNER_PATH",
  "settlement runtime bootstrap",
);
assertContains(
  runtimeBootstrap,
  "PERAX_SETTLEMENT_RUNTIME_MODULE",
  "settlement runtime bootstrap",
);
assertContains(
  runtimeBootstrap,
  "loadSettlementKeypair",
  "settlement runtime bootstrap",
);
assertContains(
  runtimeBootstrap,
  "loadSettlementRuntimeModule",
  "settlement runtime bootstrap",
);
assertContains(
  runtimeBootstrap,
  "expectedStatePda",
  "settlement runtime bootstrap",
);
assertContains(
  runtimeBootstrap,
  "createSettlementExecutorServer",
  "settlement runtime bootstrap",
);
assertContains(
  runtimeBootstrap,
  "process.exitCode = 1",
  "settlement runtime bootstrap",
);

assertContains(marketPackage, '"@coral-xyz/anchor": "0.30.1"', "market-engine package");
assertContains(marketPackage, '"@solana/spl-token": "^0.4.8"', "market-engine package");
assertContains(marketPackage, '"build": "tsc"', "market-engine package");
assertContains(
  marketPackage,
  '"start:executor": "node dist/src/start-executor.js"',
  "market-engine package",
);
assertContains(marketLock, '"node_modules/@coral-xyz/anchor"', "market-engine lock");
assertContains(marketLock, '"node_modules/@solana/spl-token"', "market-engine lock");

assertContains(marketIndex, 'export * from "./executor.js";', "market-engine index");
assertContains(marketIndex, 'export * from "./idl.js";', "market-engine index");
assertContains(marketIndex, 'export * from "./anchor-client.js";', "market-engine index");
assertContains(marketIndex, 'export * from "./runtime.js";', "market-engine index");
assertContains(marketIndex, 'export * from "./start-executor.js";', "market-engine index");

console.log("Settlement source guards passed.");
