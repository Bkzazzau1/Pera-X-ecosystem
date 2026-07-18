import fs from "node:fs";
import path from "node:path";
import process from "node:process";

const root = process.cwd();
const idlPath = path.join(root, "target", "idl", "perax_core.json");

if (!fs.existsSync(idlPath)) {
  throw new Error(
    `Generated Anchor IDL is missing at ${idlPath}. Run anchor build before validating it.`,
  );
}

const idl = JSON.parse(fs.readFileSync(idlPath, "utf8"));

const normalize = (value) =>
  String(value ?? "")
    .replace(/[^a-zA-Z0-9]/g, "")
    .toLowerCase();

const names = (items) => new Set((items ?? []).map((item) => normalize(item.name)));

const requireNames = (available, expected, label) => {
  const missing = expected.filter((name) => !available.has(normalize(name)));
  if (missing.length > 0) {
    throw new Error(`${label} is missing: ${missing.join(", ")}`);
  }
};

const instructionNames = names(idl.instructions);
requireNames(
  instructionNames,
  [
    "initializeSettlementPolicy",
    "initializeProductSettlementPolicy",
    "updateProductSettlementPolicy",
    "planSettlement",
    "fundDirectPexSettlement",
    "executeSettlementMarketPurchase",
    "executeSettlementVaultFunding",
    "finalizeSettlement",
  ],
  "Settlement IDL instructions",
);

const accountNames = names(idl.accounts);
requireNames(
  accountNames,
  [
    "settlementPolicy",
    "productSettlementPolicy",
    "settlementRecord",
    "settlementCustody",
  ],
  "Settlement IDL accounts",
);

const eventNames = names(idl.events);
requireNames(
  eventNames,
  [
    "settlementPolicyInitialized",
    "productSettlementPolicyInitialized",
    "productSettlementPolicyUpdated",
    "settlementPlanned",
    "directPexSettlementFunded",
    "settlementMarketPurchaseExecuted",
    "settlementPolicyVaultFunded",
    "settlementFinalized",
  ],
  "Settlement IDL events",
);

const errorNames = names(idl.errors);
requireNames(
  errorNames,
  [
    "invalidPolicy",
    "policyInactive",
    "marketActionPaused",
    "invalidSettlementStatus",
    "invalidSettlementMode",
    "invalidMarketAdapter",
    "invalidMarketSettlement",
    "settlementDailyCapExceeded",
    "settlementNotFunded",
    "settlementArithmeticError",
  ],
  "Settlement IDL errors",
);

const flattenAccounts = (accounts, output = []) => {
  for (const account of accounts ?? []) {
    if (Array.isArray(account.accounts)) {
      flattenAccounts(account.accounts, output);
    } else {
      output.push(account);
    }
  }
  return output;
};

const findInstruction = (name) => {
  const normalized = normalize(name);
  return (idl.instructions ?? []).find(
    (instruction) => normalize(instruction.name) === normalized,
  );
};

const requireInstructionAccounts = (instructionName, expectedAccounts) => {
  const instruction = findInstruction(instructionName);
  if (!instruction) {
    throw new Error(`IDL instruction is missing: ${instructionName}`);
  }
  const available = names(flattenAccounts(instruction.accounts));
  requireNames(
    available,
    expectedAccounts,
    `${instructionName} account list`,
  );
};

requireInstructionAccounts("planSettlement", [
  "settlementRecord",
  "settlementCustody",
  "settlementAuthority",
  "settlementPexVault",
  "observation",
]);
requireInstructionAccounts("fundDirectPexSettlement", [
  "settlementRecord",
  "settlementCustody",
  "settlementPexVault",
  "sourceAuthority",
  "sourceTokenAccount",
]);
requireInstructionAccounts("executeSettlementMarketPurchase", [
  "settlementRecord",
  "settlementCustody",
  "settlementPexVault",
  "approvedMarketPool",
  "marketProgram",
  "quoteSourceTokenAccount",
]);
requireInstructionAccounts("executeSettlementVaultFunding", [
  "settlementRecord",
  "settlementCustody",
  "settlementPexVault",
  "reserveVaultConfig",
  "vaultTokenAccount",
]);
requireInstructionAccounts("finalizeSettlement", [
  "settlementRecord",
  "settlementCustody",
  "settlementAuthority",
  "settlementPexVault",
  "destinationTokenAccount",
  "lockVault",
]);

const address = idl.address ?? idl.metadata?.address;
if (typeof address !== "string" || address.trim().length === 0) {
  throw new Error("Generated IDL does not contain a program address.");
}

console.log(
  `Validated settlement IDL for program ${address}: instructions, isolated custody, events and errors are present.`,
);
