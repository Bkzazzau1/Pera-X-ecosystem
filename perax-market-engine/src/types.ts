export type PriceSample = {
  price: bigint;
  liquidityUsd: bigint;
  quoteLiquidityUsd: bigint;
  volumeUsd: bigint;
  netBuyPressureBps: number;
  observedAt: number;
};

export type ApcSnapshot = {
  observationId: Uint8Array;
  sequence: bigint;
  pool: string;
  spotPrice: bigint;
  twapPrice: bigint;
  twapMinutes: bigint;
  liquidityUsd: bigint;
  quoteLiquidityUsd: bigint;
  volumeUsd: bigint;
  netBuyPressureBps: number;
  priceVelocityBps: number;
  volatilityBps: number;
  estimatedPriceImpactBps: number;
  observedAt: bigint;
};

export type ApcStateView = {
  status:
    | "inactive"
    | "armed"
    | "active"
    | "pumpControl"
    | "awaitingAbsorption"
    | "recovery"
    | "paused";
  currentBandIndex: number;
  nextBandPrice: bigint;
};

export interface MarketDataSource {
  readApprovedPool(): Promise<{
    pool: string;
    samples: PriceSample[];
    estimatedPriceImpactBps: number;
  }>;
}

export interface ApcProgramClient {
  readState(): Promise<ApcStateView>;
  submitObservation(snapshot: ApcSnapshot): Promise<void>;
  activateNextBand(observationId: Uint8Array, bandIndex: number): Promise<void>;
  readPermittedReleaseAmount(
    bandIndex: number,
    observationId: Uint8Array,
  ): Promise<bigint>;
  executePermittedRelease(
    bandIndex: number,
    observationId: Uint8Array,
    amount: bigint,
  ): Promise<void>;
  depositCounterweightProceeds(
    amount: bigint,
    settlementReference: Uint8Array,
  ): Promise<void>;
  enterRecovery(observationId: Uint8Array): Promise<void>;
  executeRecoveryPurchase(observationId: Uint8Array): Promise<void>;
}

export interface ExecutionVenue {
  placeControlledSell(amount: bigint): Promise<{
    actualUsdcProceeds: bigint;
    settlementReference: Uint8Array;
  }>;
}

export type SettlementFundingMethod =
  | "pex"
  | "stablecoin"
  | "fiat"
  | "virtualAccount";

export type SettlementMarketMode =
  | "directPex"
  | "marketPurchase"
  | "policyVault"
  | "hybrid";

export type SettlementDisposition =
  | "utilityPayment"
  | "customerDelivery"
  | "burn"
  | "lock";

export type SettlementStatus = "planned" | "funding" | "ready" | "finalized";

export type SettlementPlanInput = {
  settlementId: Uint8Array;
  productId: Uint8Array;
  observationId: Uint8Array;
  fundingMethod: SettlementFundingMethod;
  quantity: bigint;
  beneficiary: string;
};

export type SettlementRecordView = {
  settlementId: Uint8Array;
  marketMode: SettlementMarketMode;
  disposition: SettlementDisposition;
  status: SettlementStatus;
  pexObligation: bigint;
  marketPexRequired: bigint;
  policyVaultPexRequired: bigint;
  marketPexReceived: bigint;
  policyVaultPexReceived: bigint;
  directPexReceived: bigint;
  productId?: Uint8Array;
  observationId?: Uint8Array;
  beneficiary?: string;
  destinationTokenAccount?: string;
  effectivePrice?: bigint;
  settlementRecordAddress?: string;
  transactionSignature?: string;
};

export type SettlementRemainingAccount = {
  publicKey: string;
  isWritable: boolean;
  isSigner?: boolean;
};

export type AtomicMarketPurchase = {
  maximumQuoteAmount: bigint;
  minimumPexOut: bigint;
  instructionData: Uint8Array;
  remainingAccounts?: SettlementRemainingAccount[];
};

export interface SettlementExecutionVenue {
  buildAtomicPexPurchase(input: {
    settlement: SettlementRecordView;
    pexAmount: bigint;
  }): Promise<AtomicMarketPurchase>;
}

export interface SettlementProgramClient {
  planSettlement(input: SettlementPlanInput): Promise<SettlementRecordView>;
  fundDirectPex(
    settlementId: Uint8Array,
    amount: bigint,
  ): Promise<SettlementRecordView>;
  executeMarketPurchase(
    settlementId: Uint8Array,
    purchase: AtomicMarketPurchase,
  ): Promise<SettlementRecordView>;
  executePolicyVaultFunding(
    settlementId: Uint8Array,
  ): Promise<SettlementRecordView>;
  finalizeSettlement(
    settlementId: Uint8Array,
  ): Promise<SettlementRecordView>;
}

export interface SettlementObservationProvider {
  getFreshObservationId(): Promise<Uint8Array>;
}

export type SettlementExecutorRequest = {
  solanaRpcUrl: string;
  programId: string;
  statePda: string;
  pexMintAddress: string;
  orderReference: string;
  settlementIdHex: string;
  productIdHex: string;
  fundingMethod: SettlementFundingMethod;
  quantity: number;
  beneficiaryWallet: string;
  previousStatus: string;
  attempt: number;
};

export type SettlementExecutorResponse = {
  status: "finalized" | "failed";
  terminalFailure: boolean;
  settlementRecordAddress?: string;
  transactionSignature?: string;
  error?: string;
};
