import assert from "node:assert/strict";
import test from "node:test";
import { createRequire } from "node:module";

const require = createRequire(import.meta.url);
const bigintBuffer = require("bigint-buffer") as {
  toBigIntLE(value: Uint8Array): bigint;
  toBigIntBE(value: Uint8Array): bigint;
  toBufferLE(value: bigint, width: number): Buffer;
  toBufferBE(value: bigint, width: number): Buffer;
};

test("safe bigint-buffer replacement preserves canonical endian conversions", () => {
  assert.equal(bigintBuffer.toBigIntLE(Uint8Array.from([0x34, 0x12])), 0x1234n);
  assert.equal(bigintBuffer.toBigIntBE(Uint8Array.from([0x12, 0x34])), 0x1234n);
  assert.deepEqual([...bigintBuffer.toBufferLE(0x1234n, 2)], [0x34, 0x12]);
  assert.deepEqual([...bigintBuffer.toBufferBE(0x1234n, 2)], [0x12, 0x34]);
  assert.throws(() => bigintBuffer.toBufferLE(0x10000n, 2), /does not fit/);
});
