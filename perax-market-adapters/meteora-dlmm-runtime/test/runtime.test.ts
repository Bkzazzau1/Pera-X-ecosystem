import assert from "node:assert/strict";
import test from "node:test";

import { Keypair, TransactionInstruction } from "@solana/web3.js";

import {
  calculateFlowMetrics,
  isTerminalError,
  normalizeMeteoraExactOutInstruction,
  orientPrice,
} from "../src/index.js";

const discriminator = Buffer.from([43, 215, 247, 132, 137, 60, 243, 81]);

function key() {
  return Keypair.generate().publicKey;
}

function exactOutData(maximum: bigint, output: bigint): Buffer {
  const data = Buffer.alloc(28);
  discriminator.copy(data, 0);
  data.writeBigUInt64LE(maximum, 8);
  data.writeBigUInt64LE(output, 16);
  data.writeUInt32LE(0, 24);
  return data;
}

test("orients Meteora token-Y-per-token-X prices into quote per PEX", () => {
  assert.equal(orientPrice("0.000012", false, 100_000_000n), 1_200n);
  assert.equal(orientPrice("83333.3333333333", true, 100_000_000n), 1_200n);
  assert.equal(orientPrice("1.2e-5", false, 100_000_000n), 1_200n);
});

test("calculates deterministic flow, pressure, velocity and volatility", () => {
  const metrics = calculateFlowMetrics(
    [
      { observedAt: 1, quoteReserve: "100000000", pexReserve: "1", spotPriceScaled: "1000" },
      { observedAt: 2, quoteReserve: "103000000", pexReserve: "1", spotPriceScaled: "1100" },
      { observedAt: 3, quoteReserve: "101000000", pexReserve: "1", spotPriceScaled: "1050" },
    ],
    1_050n,
  );
  assert.equal(metrics.volumeUsd, 5n);
  assert.equal(metrics.netBuyPressureBps, 6_000);
  assert.equal(metrics.priceVelocityBps, 500);
  assert.equal(metrics.volatilityBps, 476);
});

test("normalizes only an exact-out instruction and preserves ordered market accounts", () => {
  const program = key();
  const pool = key();
  const source = key();
  const destination = key();
  const authority = key();
  const originalSource = key();
  const originalDestination = key();
  const keys = Array.from({ length: 17 }, (_, index) => ({
    pubkey: index === 0 ? pool : index === 4 ? originalSource : index === 5 ? originalDestination : index === 10 ? authority : key(),
    isSigner: index === 10,
    isWritable: [0, 2, 3, 4, 5, 8, 16].includes(index),
  }));
  const instruction = new TransactionInstruction({
    programId: program,
    keys,
    data: exactOutData(500n, 1_000n),
  });
  const normalized = normalizeMeteoraExactOutInstruction(
    instruction,
    pool,
    source,
    destination,
    authority,
    500n,
    1_000n,
  );
  assert.equal(normalized.keys[4]!.pubkey.toBase58(), source.toBase58());
  assert.equal(normalized.keys[5]!.pubkey.toBase58(), destination.toBase58());
  assert.equal(normalized.keys[10]!.pubkey.toBase58(), authority.toBase58());
  assert.equal(normalized.data.toString("hex"), instruction.data.toString("hex"));
});

test("rejects alternate market instructions, hook slices and changed amounts", () => {
  const program = key();
  const pool = key();
  const authority = key();
  const base = new TransactionInstruction({
    programId: program,
    keys: Array.from({ length: 17 }, (_, index) => ({
      pubkey: index === 0 ? pool : index === 10 ? authority : key(),
      isSigner: index === 10,
      isWritable: [0, 2, 3, 4, 5, 8, 16].includes(index),
    })),
    data: exactOutData(500n, 1_000n),
  });
  const source = key();
  const destination = key();
  const alternate = new TransactionInstruction({ ...base, data: Buffer.from(base.data) });
  alternate.data[0] = (alternate.data[0] ?? 0) ^ 1;
  assert.throws(
    () => normalizeMeteoraExactOutInstruction(alternate, pool, source, destination, authority, 500n, 1_000n),
    /Only Meteora/,
  );
  const hooks = new TransactionInstruction({ ...base, data: Buffer.from(base.data) });
  hooks.data.writeUInt32LE(1, 24);
  assert.throws(
    () => normalizeMeteoraExactOutInstruction(hooks, pool, source, destination, authority, 500n, 1_000n),
    /transfer-hook/,
  );
  assert.throws(
    () => normalizeMeteoraExactOutInstruction(base, pool, key(), key(), authority, 501n, 1_000n),
    /amounts do not match/,
  );
});

test("classifies configuration failures as terminal but warmup/liquidity as retryable", () => {
  assert.equal(isTerminalError(new Error("Configured Meteora pool is not the APC-approved pool")), true);
  assert.equal(isTerminalError(new Error("Meteora on-chain oracle does not yet cover the configured TWAP window")), false);
  assert.equal(isTerminalError(new Error("Quote source balance is below the contract-bounded purchase ceiling")), false);
});
