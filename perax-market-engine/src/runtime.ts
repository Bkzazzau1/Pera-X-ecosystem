import type { AnchorProvider } from "@coral-xyz/anchor";
import type { PublicKey } from "@solana/web3.js";

import type {
  AnchorSettlementProgramClientConfig,
  SettlementTokenSource,
} from "./anchor-client.js";
import type { SettlementIdl } from "./idl.js";
import type {
  AtomicMarketPurchase,
  SettlementExecutionVenue,
  SettlementObservationProvider,
  SettlementRecordView,
} from "./types.js";

export type SettlementRuntimeContext = {
  provider: AnchorProvider;
  programId: PublicKey;
  statePda: PublicKey;
  pexMint: PublicKey;
  idl: SettlementIdl;
};

export type SettlementRuntimeBindings = {
  venue: SettlementExecutionVenue;
  observations: SettlementObservationProvider;
  resolveDirectPexSource?: (
    settlement: SettlementRecordView,
    amount: bigint,
  ) => Promise<SettlementTokenSource> | SettlementTokenSource;
  resolveQuoteSource: (
    settlement: SettlementRecordView,
    purchase: AtomicMarketPurchase,
  ) => Promise<SettlementTokenSource> | SettlementTokenSource;
  resolveCustomerDestination?: AnchorSettlementProgramClientConfig["resolveCustomerDestination"];
  isTerminalError?: (error: unknown) => boolean;
};

export type SettlementRuntimeModule = {
  createSettlementRuntime(
    context: SettlementRuntimeContext,
  ): Promise<SettlementRuntimeBindings> | SettlementRuntimeBindings;
};

export function assertSettlementRuntimeBindings(
  value: unknown,
): SettlementRuntimeBindings {
  if (!isObject(value)) {
    throw new Error("Settlement runtime module returned an invalid bindings object");
  }
  if (
    !isObject(value.venue) ||
    typeof value.venue.buildAtomicPexPurchase !== "function"
  ) {
    throw new Error("Settlement runtime must provide an atomic execution venue");
  }
  if (
    !isObject(value.observations) ||
    typeof value.observations.getFreshObservationId !== "function"
  ) {
    throw new Error("Settlement runtime must provide a fresh observation provider");
  }
  if (typeof value.resolveQuoteSource !== "function") {
    throw new Error("Settlement runtime must provide a quote-token source resolver");
  }
  return value as unknown as SettlementRuntimeBindings;
}

function isObject(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}
