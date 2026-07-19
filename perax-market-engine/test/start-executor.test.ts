import assert from "node:assert/strict";
import { mkdtemp, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import test from "node:test";

import { web3 } from "@coral-xyz/anchor";

import { assertSettlementRuntimeBindings } from "../src/runtime.js";
import {
  loadSettlementKeypair,
  requiredEnvironment,
  runtimeModuleUrl,
} from "../src/start-executor.js";

test("executor environment rejects missing values and trims configured values", () => {
  assert.throws(
    () => requiredEnvironment({}, "PERAX_PROGRAM_ID"),
    /PERAX_PROGRAM_ID is required/,
  );
  assert.equal(
    requiredEnvironment(
      { PERAX_PROGRAM_ID: "  program-id  " },
      "PERAX_PROGRAM_ID",
    ),
    "program-id",
  );
});

test("runtime module paths resolve to local file URLs", () => {
  const url = runtimeModuleUrl("./runtime-module.mjs");
  assert.match(url, /^file:/);
  assert.match(url, /runtime-module\.mjs$/);
});

test("settlement keypair loader accepts exactly 64 bytes and rejects malformed files", async () => {
  const directory = await mkdtemp(join(tmpdir(), "perax-settlement-"));
  try {
    const keypair = web3.Keypair.generate();
    const validPath = join(directory, "valid.json");
    const invalidPath = join(directory, "invalid.json");
    await writeFile(validPath, JSON.stringify(Array.from(keypair.secretKey)));
    await writeFile(invalidPath, JSON.stringify([1, 2, 3]));

    const loaded = await loadSettlementKeypair(validPath);
    assert.equal(loaded.publicKey.toBase58(), keypair.publicKey.toBase58());
    await assert.rejects(
      loadSettlementKeypair(invalidPath),
      /64-byte keypair array/,
    );
  } finally {
    await rm(directory, { recursive: true, force: true });
  }
});

test("runtime bindings fail closed without a venue, observations, or quote source", () => {
  assert.throws(
    () => assertSettlementRuntimeBindings({}),
    /atomic execution venue/,
  );
  assert.throws(
    () =>
      assertSettlementRuntimeBindings({
        venue: { buildAtomicPexPurchase() {} },
      }),
    /fresh observation provider/,
  );
  assert.throws(
    () =>
      assertSettlementRuntimeBindings({
        venue: { buildAtomicPexPurchase() {} },
        observations: { getFreshObservationId() {} },
      }),
    /quote-token source resolver/,
  );
});
