import { readFile } from "node:fs/promises";

const REQUIRED_INSTRUCTIONS = [
  "initializeSettlementPolicy",
  "initializeProductSettlementPolicy",
  "updateProductSettlementPolicy",
  "planSettlement",
  "fundDirectPexSettlement",
  "executeSettlementMarketPurchase",
  "executeSettlementVaultFunding",
  "finalizeSettlement",
] as const;

const REQUIRED_ACCOUNTS = [
  "settlementPolicy",
  "productSettlementPolicy",
  "settlementRecord",
  "settlementCustody",
] as const;

export type SettlementIdl = {
  address?: string;
  metadata?: { address?: string };
  instructions?: Array<{ name?: string }>;
  accounts?: Array<{ name?: string }>;
};

export async function loadSettlementIdl(
  filePath: string,
  expectedProgramId: string,
): Promise<SettlementIdl> {
  const content = await readFile(filePath, "utf8");
  const parsed = JSON.parse(content) as unknown;
  return assertSettlementIdlCompatible(parsed, expectedProgramId);
}

export function assertSettlementIdlCompatible(
  value: unknown,
  expectedProgramId: string,
): SettlementIdl {
  if (!isObject(value)) {
    throw new Error("Settlement IDL must be a JSON object");
  }

  const idl = value as SettlementIdl;
  const expected = expectedProgramId.trim();
  if (expected.length === 0) {
    throw new Error("Expected Pera-X program ID is required");
  }

  const address = readIdlAddress(idl);
  if (!address) {
    throw new Error("Settlement IDL does not contain a program address");
  }
  if (address !== expected) {
    throw new Error(
      `Settlement IDL program address ${address} does not match configured program ${expected}`,
    );
  }

  requireNames(idl.instructions, REQUIRED_INSTRUCTIONS, "instructions");
  requireNames(idl.accounts, REQUIRED_ACCOUNTS, "accounts");
  return idl;
}

export function readIdlAddress(idl: SettlementIdl): string | undefined {
  const address = idl.address ?? idl.metadata?.address;
  return typeof address === "string" && address.trim().length > 0
    ? address.trim()
    : undefined;
}

function requireNames(
  items: Array<{ name?: string }> | undefined,
  expected: readonly string[],
  label: string,
): void {
  const available = new Set(
    (items ?? []).map((item) => normalize(item.name)).filter(Boolean),
  );
  const missing = expected.filter((name) => !available.has(normalize(name)));
  if (missing.length > 0) {
    throw new Error(
      `Settlement IDL is missing required ${label}: ${missing.join(", ")}`,
    );
  }
}

function normalize(value: unknown): string {
  return String(value ?? "")
    .replace(/[^a-zA-Z0-9]/g, "")
    .toLowerCase();
}

function isObject(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}
