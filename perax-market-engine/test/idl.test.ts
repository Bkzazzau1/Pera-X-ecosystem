import assert from "node:assert/strict";
import test from "node:test";

import { assertSettlementIdlCompatible } from "../src/idl.js";

const programId = "FqEiSx5vujh2vi3yk12NaZMXhjMSaKovGUuzcKiAgshn";

function validIdl() {
  return {
    address: programId,
    instructions: [
      { name: "initializeSettlementPolicy" },
      { name: "initializeProductSettlementPolicy" },
      { name: "updateProductSettlementPolicy" },
      { name: "planSettlement" },
      { name: "fundDirectPexSettlement" },
      { name: "executeSettlementMarketPurchase" },
      { name: "executeSettlementVaultFunding" },
      { name: "finalizeSettlement" },
    ],
    accounts: [
      { name: "settlementPolicy" },
      { name: "productSettlementPolicy" },
      { name: "settlementRecord" },
      { name: "settlementCustody" },
    ],
  };
}

test("accepts a settlement IDL with the configured program and required interface", () => {
  const idl = assertSettlementIdlCompatible(validIdl(), programId);
  assert.equal(idl.address, programId);
});

test("rejects an IDL for another deployed program", () => {
  assert.throws(
    () => assertSettlementIdlCompatible(validIdl(), "another-program"),
    /does not match configured program/,
  );
});

test("rejects a stale IDL missing isolated settlement custody", () => {
  const idl = validIdl();
  idl.accounts = idl.accounts.filter(
    (account) => account.name !== "settlementCustody",
  );
  assert.throws(
    () => assertSettlementIdlCompatible(idl, programId),
    /settlementCustody/,
  );
});

test("rejects a stale IDL missing finalization", () => {
  const idl = validIdl();
  idl.instructions = idl.instructions.filter(
    (instruction) => instruction.name !== "finalizeSettlement",
  );
  assert.throws(
    () => assertSettlementIdlCompatible(idl, programId),
    /finalizeSettlement/,
  );
});
