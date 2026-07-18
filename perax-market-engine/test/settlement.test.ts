import assert from "node:assert/strict";
import test from "node:test";

import { SettlementCoordinator } from "../src/settlement.js";
import type {
  AtomicMarketPurchase,
  SettlementExecutionVenue,
  SettlementPlanInput,
  SettlementProgramClient,
  SettlementRecordView,
} from "../src/types.js";

const id = new Uint8Array(32).fill(7);

function record(
  overrides: Partial<SettlementRecordView> = {},
): SettlementRecordView {
  return {
    settlementId: id,
    marketMode: "hybrid",
    disposition: "utilityPayment",
    status: "planned",
    pexObligation: 1_000n,
    marketPexRequired: 400n,
    policyVaultPexRequired: 600n,
    marketPexReceived: 0n,
    policyVaultPexReceived: 0n,
    directPexReceived: 0n,
    ...overrides,
  };
}

const input: SettlementPlanInput = {
  settlementId: id,
  productId: new Uint8Array(32).fill(8),
  observationId: new Uint8Array(32).fill(9),
  fundingMethod: "stablecoin",
  quantity: 1n,
  beneficiary: "beneficiary-wallet",
};

class MockVenue implements SettlementExecutionVenue {
  calls: bigint[] = [];

  async buildAtomicPexPurchase({
    pexAmount,
  }: {
    settlement: SettlementRecordView;
    pexAmount: bigint;
  }): Promise<AtomicMarketPurchase> {
    this.calls.push(pexAmount);
    return {
      maximumQuoteAmount: 100n,
      minimumPexOut: pexAmount,
      instructionData: new Uint8Array([1, 2, 3]),
    };
  }
}

class MockProgram implements SettlementProgramClient {
  calls: string[] = [];
  constructor(private planned: SettlementRecordView) {}

  async planSettlement(): Promise<SettlementRecordView> {
    this.calls.push("plan");
    return this.planned;
  }

  async fundDirectPex(
    _settlementId: Uint8Array,
    amount: bigint,
  ): Promise<SettlementRecordView> {
    this.calls.push(`direct:${amount}`);
    return record({
      ...this.planned,
      directPexReceived: this.planned.pexObligation,
      status: "ready",
    });
  }

  async executeMarketPurchase(): Promise<SettlementRecordView> {
    this.calls.push("market");
    return record({
      ...this.planned,
      marketPexReceived: this.planned.marketPexRequired,
      status:
        this.planned.policyVaultPexRequired === 0n ? "ready" : "funding",
    });
  }

  async executePolicyVaultFunding(): Promise<SettlementRecordView> {
    this.calls.push("vault");
    return record({
      ...this.planned,
      marketPexReceived: this.planned.marketPexRequired,
      policyVaultPexReceived: this.planned.policyVaultPexRequired,
      status: "ready",
    });
  }

  async finalizeSettlement(): Promise<SettlementRecordView> {
    this.calls.push("finalize");
    return record({ ...this.planned, status: "finalized" });
  }
}

test("hybrid settlement follows the contract-derived market then vault sequence", async () => {
  const program = new MockProgram(record());
  const venue = new MockVenue();
  const coordinator = new SettlementCoordinator(program, venue);

  const result = await coordinator.execute(input);

  assert.equal(result.status, "finalized");
  assert.deepEqual(program.calls, ["plan", "market", "vault", "finalize"]);
  assert.deepEqual(venue.calls, [400n]);
});

test("direct PEX settlement never calls a market venue", async () => {
  const program = new MockProgram(
    record({
      marketMode: "directPex",
      marketPexRequired: 0n,
      policyVaultPexRequired: 0n,
      pexObligation: 900n,
    }),
  );
  const venue = new MockVenue();
  const coordinator = new SettlementCoordinator(program, venue);

  await coordinator.execute({ ...input, fundingMethod: "pex" });

  assert.deepEqual(program.calls, ["plan", "direct:900", "finalize"]);
  assert.deepEqual(venue.calls, []);
});

test("coordinator rejects an adapter that weakens minimum output", async () => {
  const program = new MockProgram(
    record({
      marketMode: "marketPurchase",
      marketPexRequired: 500n,
      policyVaultPexRequired: 0n,
    }),
  );
  const venue: SettlementExecutionVenue = {
    async buildAtomicPexPurchase() {
      return {
        maximumQuoteAmount: 10n,
        minimumPexOut: 499n,
        instructionData: new Uint8Array([1]),
      };
    },
  };
  const coordinator = new SettlementCoordinator(program, venue);

  await assert.rejects(
    coordinator.execute(input),
    /minimum PEX output is below the contract-derived requirement/,
  );
  assert.deepEqual(program.calls, ["plan"]);
});
