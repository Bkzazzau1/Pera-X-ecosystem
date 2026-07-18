import assert from "node:assert/strict";
import type { AddressInfo } from "node:net";
import test from "node:test";

import { createSettlementExecutorServer } from "../src/executor.js";
import type {
  AtomicMarketPurchase,
  SettlementExecutionVenue,
  SettlementObservationProvider,
  SettlementPlanInput,
  SettlementProgramClient,
  SettlementRecordView,
} from "../src/types.js";

const token = "test-settlement-executor-token-123456";
const settlementIdHex = "11".repeat(32);
const productIdHex = "22".repeat(32);

class ProgramClient implements SettlementProgramClient {
  planInput?: SettlementPlanInput;

  async planSettlement(input: SettlementPlanInput): Promise<SettlementRecordView> {
    this.planInput = input;
    return record({ status: "planned" });
  }

  async fundDirectPex(): Promise<SettlementRecordView> {
    throw new Error("direct PEX was not expected");
  }

  async executeMarketPurchase(
    _settlementId: Uint8Array,
    purchase: AtomicMarketPurchase,
  ): Promise<SettlementRecordView> {
    assert.equal(purchase.minimumPexOut, 10n);
    return record({ status: "ready", marketPexReceived: 10n });
  }

  async executePolicyVaultFunding(): Promise<SettlementRecordView> {
    throw new Error("policy vault was not expected");
  }

  async finalizeSettlement(): Promise<SettlementRecordView> {
    return record({
      status: "finalized",
      marketPexReceived: 10n,
      settlementRecordAddress: "settlement-record-address",
      transactionSignature: "settlement-transaction-signature",
    });
  }
}

class Venue implements SettlementExecutionVenue {
  async buildAtomicPexPurchase(): Promise<AtomicMarketPurchase> {
    return {
      maximumQuoteAmount: 5n,
      minimumPexOut: 10n,
      instructionData: new Uint8Array([1, 2, 3]),
    };
  }
}

class Observations implements SettlementObservationProvider {
  async getFreshObservationId(): Promise<Uint8Array> {
    return new Uint8Array(32).fill(9);
  }
}

function record(
  overrides: Partial<SettlementRecordView> = {},
): SettlementRecordView {
  return {
    settlementId: Uint8Array.from(Buffer.from(settlementIdHex, "hex")),
    marketMode: "marketPurchase",
    disposition: "utilityPayment",
    status: "planned",
    pexObligation: 10n,
    marketPexRequired: 10n,
    policyVaultPexRequired: 0n,
    marketPexReceived: 0n,
    policyVaultPexReceived: 0n,
    directPexReceived: 0n,
    ...overrides,
  };
}

function requestBody() {
  return {
    solanaRpcUrl: "https://api.devnet.solana.com",
    programId: "program-id",
    statePda: "state-pda",
    pexMintAddress: "pex-mint",
    orderReference: "checkout-test",
    settlementIdHex,
    productIdHex,
    fundingMethod: "stablecoin",
    quantity: 1,
    beneficiaryWallet: "beneficiary-wallet",
    previousStatus: "pending",
    attempt: 1,
  };
}

test("executor authenticates and returns finalized program results", async () => {
  const program = new ProgramClient();
  const server = createSettlementExecutorServer({
    program,
    venue: new Venue(),
    observations: new Observations(),
    bearerToken: token,
    expected: {
      programId: "program-id",
      statePda: "state-pda",
      pexMintAddress: "pex-mint",
      solanaRpcUrl: "https://api.devnet.solana.com",
    },
  });

  await new Promise<void>((resolve) => server.listen(0, "127.0.0.1", resolve));
  const address = server.address() as AddressInfo;
  try {
    const response = await fetch(
      `http://127.0.0.1:${address.port}/execute/settlement`,
      {
        method: "POST",
        headers: {
          authorization: `Bearer ${token}`,
          "content-type": "application/json",
        },
        body: JSON.stringify(requestBody()),
      },
    );
    const body = (await response.json()) as Record<string, unknown>;

    assert.equal(response.status, 200);
    assert.equal(body.status, "finalized");
    assert.equal(body.terminalFailure, false);
    assert.equal(body.settlementRecordAddress, "settlement-record-address");
    assert.equal(body.transactionSignature, "settlement-transaction-signature");
    assert.equal(program.planInput?.fundingMethod, "stablecoin");
    assert.equal(program.planInput?.quantity, 1n);
    assert.deepEqual(program.planInput?.observationId, new Uint8Array(32).fill(9));
  } finally {
    await new Promise<void>((resolve, reject) =>
      server.close((error) => (error ? reject(error) : resolve())),
    );
  }
});

test("executor rejects missing bearer authentication", async () => {
  const server = createSettlementExecutorServer({
    program: new ProgramClient(),
    venue: new Venue(),
    observations: new Observations(),
    bearerToken: token,
    expected: {
      programId: "program-id",
      statePda: "state-pda",
      pexMintAddress: "pex-mint",
    },
  });

  await new Promise<void>((resolve) => server.listen(0, "127.0.0.1", resolve));
  const address = server.address() as AddressInfo;
  try {
    const response = await fetch(
      `http://127.0.0.1:${address.port}/execute/settlement`,
      {
        method: "POST",
        headers: { "content-type": "application/json" },
        body: JSON.stringify(requestBody()),
      },
    );
    assert.equal(response.status, 401);
  } finally {
    await new Promise<void>((resolve, reject) =>
      server.close((error) => (error ? reject(error) : resolve())),
    );
  }
});
