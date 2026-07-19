import assert from "node:assert/strict";
import test from "node:test";

import anchor, { AnchorProvider, type Idl } from "@coral-xyz/anchor";
const { BN } = anchor;
type AnchorBn = InstanceType<typeof BN>;
import { Keypair, PublicKey } from "@solana/web3.js";

import { AnchorSettlementProgramClient } from "../src/anchor-client.js";
import type { SettlementIdl } from "../src/idl.js";
import type { SettlementPlanInput } from "../src/types.js";

const REQUIRED_INSTRUCTIONS = [
  "initializeSettlementPolicy",
  "initializeProductSettlementPolicy",
  "updateProductSettlementPolicy",
  "planSettlement",
  "fundDirectPexSettlement",
  "executeSettlementMarketPurchase",
  "executeSettlementVaultFunding",
  "finalizeSettlement",
];
const REQUIRED_ACCOUNTS = [
  "settlementPolicy",
  "productSettlementPolicy",
  "settlementRecord",
  "settlementCustody",
];

type Call = {
  name: string;
  params: unknown;
  accounts?: Record<string, PublicKey>;
  remaining?: Array<{ pubkey: PublicKey; isWritable: boolean; isSigner: boolean }>;
};

class MockBuilder {
  constructor(
    private readonly call: Call,
    private readonly onRpc: (call: Call) => void,
  ) {}
  accountsStrict(accounts: Record<string, PublicKey>): this {
    this.call.accounts = accounts;
    return this;
  }
  remainingAccounts(accounts: Array<{ pubkey: PublicKey; isWritable: boolean; isSigner: boolean }>): this {
    this.call.remaining = accounts;
    return this;
  }
  signers(): this {
    return this;
  }
  async rpc(): Promise<string> {
    this.onRpc(this.call);
    return `${this.call.name}-signature`;
  }
}

function makeIdl(programId: PublicKey): SettlementIdl {
  return {
    address: programId.toBase58(),
    instructions: REQUIRED_INSTRUCTIONS.map((name) => ({ name })),
    accounts: REQUIRED_ACCOUNTS.map((name) => ({ name })),
  };
}

function createHarness() {
  const programId = Keypair.generate().publicKey;
  const pexMint = Keypair.generate().publicKey;
  const initiator = Keypair.generate().publicKey;
  const beneficiary = Keypair.generate().publicKey;
  const fixedDestination = Keypair.generate().publicKey;
  const lockVault = Keypair.generate().publicKey;
  const quoteMint = Keypair.generate().publicKey;
  const marketPool = Keypair.generate().publicKey;
  const marketProgram = Keypair.generate().publicKey;
  const calls: Call[] = [];
  const records = new Map<string, Record<string, unknown>>();
  let finalSignatureLookupCount = 0;

  const provider = {
    publicKey: initiator,
    connection: {
      async confirmTransaction() {
        return { value: { err: null } };
      },
      async getSignaturesForAddress() {
        finalSignatureLookupCount += 1;
        return [{ signature: "historic-final-signature" }];
      },
    },
  } as unknown as AnchorProvider;

  const policy = {
    quoteMint,
    pexMint,
    approvedMarketPool: marketPool,
    approvedMarketProgram: marketProgram,
    approvedPolicyVaultConfig: Keypair.generate().publicKey,
    lockVault,
  };

  const onRpc = (call: Call) => {
    if (call.name === "planSettlement") {
      const params = call.params as {
        settlementId: number[];
        productId: number[];
        observationId: number[];
        fundingMethod: Record<string, unknown>;
        quantity: AnchorBn;
        beneficiary: PublicKey;
      };
      const recordAddress = account(call, "settlementRecord");
      records.set(recordAddress.toBase58(), {
        settlementId: params.settlementId,
        productId: params.productId,
        observationId: params.observationId,
        beneficiary: params.beneficiary,
        fundingMethod: params.fundingMethod,
        marketMode: { policyVault: {} },
        disposition: { utilityPayment: {} },
        status: { planned: {} },
        quantity: params.quantity,
        effectivePrice: new BN(12_000),
        pexObligation: new BN(1_000),
        marketPexRequired: new BN(0),
        policyVaultPexRequired: new BN(1_000),
        marketPexReceived: new BN(0),
        policyVaultPexReceived: new BN(0),
        directPexReceived: new BN(0),
        destinationTokenAccount: fixedDestination,
      });
    }
    if (call.name === "finalizeSettlement") {
      const recordAddress = account(call, "settlementRecord");
      const record = records.get(recordAddress.toBase58())!;
      record.status = { finalized: {} };
    }
  };

  const methods = Object.fromEntries(
    REQUIRED_INSTRUCTIONS.map((name) => [
      name,
      (params: unknown) => {
        const call: Call = { name, params };
        calls.push(call);
        return new MockBuilder(call, onRpc);
      },
    ]),
  );

  const settlementRecordClient = {
    async fetch(address: PublicKey) {
      const record = records.get(address.toBase58());
      if (!record) throw new Error("Account does not exist");
      return record;
    },
    async fetchNullable(address: PublicKey) {
      return records.get(address.toBase58()) ?? null;
    },
  };

  const program = {
    programId,
    methods,
    account: {
      settlementRecord: settlementRecordClient,
      settlementPolicy: { async fetch() { return policy; } },
      reserveVaultConfig: { async fetch() { throw new Error("not used"); } },
    },
  };

  const client = new AnchorSettlementProgramClient({
    provider,
    idl: makeIdl(programId),
    expectedProgramId: programId.toBase58(),
    pexMint,
    program: program as unknown as import("@coral-xyz/anchor").Program<Idl>,
  });

  return {
    client,
    calls,
    records,
    programId,
    beneficiary,
    fixedDestination,
    lockVault,
    get finalSignatureLookupCount() {
      return finalSignatureLookupCount;
    },
  };
}

function account(call: Call, name: string): PublicKey {
  const value = call.accounts?.[name];
  if (!value) {
    throw new Error(`Missing test account ${name}`);
  }
  return value;
}

function planInput(beneficiary: PublicKey): SettlementPlanInput {
  return {
    settlementId: new Uint8Array(32).fill(1),
    productId: new Uint8Array(32).fill(2),
    observationId: new Uint8Array(32).fill(3),
    fundingMethod: "stablecoin",
    quantity: 1n,
    beneficiary: beneficiary.toBase58(),
  };
}

test("Anchor client derives plan accounts and reuses an existing immutable settlement", async () => {
  const harness = createHarness();
  const input = planInput(harness.beneficiary);

  const first = await harness.client.planSettlement(input);
  assert.equal(first.marketMode, "policyVault");
  assert.equal(first.pexObligation, 1_000n);
  assert.equal(harness.calls.filter((call) => call.name === "planSettlement").length, 1);

  const planCall = harness.calls.find((call) => call.name === "planSettlement")!;
  assert.equal(account(planCall, "state").toBase58(), harness.client.statePda.toBase58());
  assert.equal(
    account(planCall, "settlementPolicy").toBase58(),
    harness.client.settlementPolicyPda.toBase58(),
  );
  assert.equal(account(planCall, "initiator").toBase58().length > 0, true);

  const replay = await harness.client.planSettlement(input);
  assert.equal(replay.settlementRecordAddress, first.settlementRecordAddress);
  assert.equal(harness.calls.filter((call) => call.name === "planSettlement").length, 1);

  await assert.rejects(
    harness.client.planSettlement({ ...input, quantity: 2n }),
    /conflicts with the requested immutable settlement plan/,
  );
});

test("Anchor client finalizes to the contract-recorded destination and returns confirmation metadata", async () => {
  const harness = createHarness();
  const input = planInput(harness.beneficiary);
  const planned = await harness.client.planSettlement(input);
  const raw = harness.records.get(planned.settlementRecordAddress!)!;
  raw.status = { ready: {} };
  raw.policyVaultPexReceived = new BN(1_000);

  const finalized = await harness.client.finalizeSettlement(input.settlementId);
  assert.equal(finalized.status, "finalized");
  assert.equal(finalized.transactionSignature, "finalizeSettlement-signature");

  const call = harness.calls.find((item) => item.name === "finalizeSettlement")!;
  assert.equal(
    account(call, "destinationTokenAccount").toBase58(),
    harness.fixedDestination.toBase58(),
  );
  assert.equal(account(call, "lockVault").toBase58(), harness.lockVault.toBase58());

  const replay = await harness.client.finalizeSettlement(input.settlementId);
  assert.equal(replay.transactionSignature, "historic-final-signature");
  assert.equal(harness.finalSignatureLookupCount, 1);
  assert.equal(harness.calls.filter((item) => item.name === "finalizeSettlement").length, 1);
});
