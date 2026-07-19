import assert from "node:assert/strict";
import test from "node:test";

import { Keypair, PublicKey, SystemProgram } from "@solana/web3.js";
import {
  ASSOCIATED_TOKEN_PROGRAM_ID,
  createAssociatedTokenAccountIdempotentInstruction,
  getAssociatedTokenAddressSync,
  TOKEN_PROGRAM_ID,
} from "../src/token.js";

test("local ATA derivation matches the canonical PDA formula", () => {
  const mint = new PublicKey("So11111111111111111111111111111111111111112");
  const owner = new PublicKey("11111111111111111111111111111111");
  const expected = PublicKey.findProgramAddressSync(
    [owner.toBuffer(), TOKEN_PROGRAM_ID.toBuffer(), mint.toBuffer()],
    ASSOCIATED_TOKEN_PROGRAM_ID,
  )[0];
  assert.equal(
    getAssociatedTokenAddressSync(mint, owner, true).toBase58(),
    expected.toBase58(),
  );
});

test("local ATA derivation rejects off-curve owners unless explicitly permitted", () => {
  const mint = Keypair.generate().publicKey;
  const owner = PublicKey.findProgramAddressSync(
    [Buffer.from("owner")],
    Keypair.generate().publicKey,
  )[0];
  assert.throws(
    () => getAssociatedTokenAddressSync(mint, owner, false),
    /off curve/,
  );
  assert.doesNotThrow(() => getAssociatedTokenAddressSync(mint, owner, true));
});

test("idempotent ATA instruction uses the canonical account order and opcode", () => {
  const payer = Keypair.generate().publicKey;
  const owner = Keypair.generate().publicKey;
  const mint = Keypair.generate().publicKey;
  const associatedToken = getAssociatedTokenAddressSync(mint, owner);
  const instruction = createAssociatedTokenAccountIdempotentInstruction(
    payer,
    associatedToken,
    owner,
    mint,
  );

  assert.equal(
    instruction.programId.toBase58(),
    ASSOCIATED_TOKEN_PROGRAM_ID.toBase58(),
  );
  assert.deepEqual(Array.from(instruction.data), [1]);
  assert.deepEqual(
    instruction.keys.map((key) => key.pubkey.toBase58()),
    [
      payer,
      associatedToken,
      owner,
      mint,
      SystemProgram.programId,
      TOKEN_PROGRAM_ID,
    ].map((key) => key.toBase58()),
  );
  assert.deepEqual(
    instruction.keys.map((key) => [key.isSigner, key.isWritable]),
    [
      [true, true],
      [false, true],
      [false, false],
      [false, false],
      [false, false],
      [false, false],
    ],
  );
});
