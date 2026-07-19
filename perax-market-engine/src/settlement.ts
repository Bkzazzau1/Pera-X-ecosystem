import type {
  SettlementExecutionVenue,
  SettlementPlanInput,
  SettlementProgramClient,
  SettlementRecordView,
} from "./types.js";

export class SettlementCoordinator {
  constructor(
    private readonly program: SettlementProgramClient,
    private readonly venue: SettlementExecutionVenue,
  ) {}

  async execute(input: SettlementPlanInput): Promise<SettlementRecordView> {
    let settlement = await this.program.planSettlement(input);

    if (settlement.status === "finalized") {
      return settlement;
    }
    if (settlement.status === "ready") {
      return this.program.finalizeSettlement(settlement.settlementId);
    }

    switch (settlement.marketMode) {
      case "directPex": {
        const remaining = remainingDirectPex(settlement);
        if (remaining > 0n) {
          settlement = await this.program.fundDirectPex(
            settlement.settlementId,
            remaining,
          );
        }
        break;
      }
      case "marketPurchase":
        settlement = await this.executeMarketStage(settlement);
        break;
      case "policyVault": {
        if (remainingPolicyVaultPex(settlement) > 0n) {
          settlement = await this.program.executePolicyVaultFunding(
            settlement.settlementId,
          );
        }
        break;
      }
      case "hybrid":
        settlement = await this.executeMarketStage(settlement);
        if (remainingPolicyVaultPex(settlement) > 0n) {
          settlement = await this.program.executePolicyVaultFunding(
            settlement.settlementId,
          );
        }
        break;
      default:
        return assertNever(settlement.marketMode);
    }

    if (settlement.status === "finalized") {
      return settlement;
    }
    if (settlement.status !== "ready") {
      throw new Error(
        `Settlement is not ready after contract-permitted funding: ${settlement.status}`,
      );
    }

    return this.program.finalizeSettlement(settlement.settlementId);
  }

  private async executeMarketStage(
    settlement: SettlementRecordView,
  ): Promise<SettlementRecordView> {
    const remaining = remainingMarketPex(settlement);
    if (remaining <= 0n) {
      return settlement;
    }

    const purchase = await this.venue.buildAtomicPexPurchase({
      settlement,
      pexAmount: remaining,
    });
    if (purchase.minimumPexOut < remaining) {
      throw new Error(
        "Atomic market adapter minimum PEX output is below the contract-derived requirement",
      );
    }
    if (purchase.maximumQuoteAmount <= 0n || purchase.instructionData.length === 0) {
      throw new Error("Atomic market adapter payload is incomplete");
    }

    return this.program.executeMarketPurchase(
      settlement.settlementId,
      purchase,
    );
  }
}

export function remainingMarketPex(settlement: SettlementRecordView): bigint {
  return positiveDifference(
    settlement.marketPexRequired,
    settlement.marketPexReceived,
  );
}

export function remainingPolicyVaultPex(
  settlement: SettlementRecordView,
): bigint {
  return positiveDifference(
    settlement.policyVaultPexRequired,
    settlement.policyVaultPexReceived,
  );
}

export function remainingDirectPex(settlement: SettlementRecordView): bigint {
  return positiveDifference(settlement.pexObligation, settlement.directPexReceived);
}

function positiveDifference(required: bigint, received: bigint): bigint {
  if (required <= received) {
    return 0n;
  }
  return required - received;
}

function assertNever(value: never): never {
  throw new Error(`Unsupported contract-derived settlement mode: ${String(value)}`);
}
