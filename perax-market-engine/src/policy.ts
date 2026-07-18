export const APC_POLICY_V1_HASH_SHA256 = "17f93bacb0cfa5346a466258117908068f1f0cd67054f8b61c7d40818dfe84bb" as const;
export const APC_POLICY_V1 = {
  policyVersion: 1,
  riskVelocityThresholdsBps: [500, 1500, 3000],
  riskVolatilityThresholdsBps: [400, 1200, 2500],
  riskPriceImpactThresholdsBps: [250, 750, 1500],
  bandIntervalBpsByRisk: [2000, 1500, 1000, 750],
  bandReleaseBpsByRisk: [10000, 7500, 5000, 2500],
  cascadeReductionBps: [10000, 7500, 5000, 2500],
  maximumObservationAgeSeconds: 90,
  maximumFutureClockSkewSeconds: 5,
  minimumTwapMinutes: 60,
  minimumLiquidityUsd: 27360n,
  minimumQuoteLiquidityUsd: 13680n,
  minimumVolumeUsd: 6840n,
  minimumBuyPressureBps: 5500,
  proceedsAllocationBps: { counterweightVault: 7000, liquidityReinforcement: 2000, burnReserve: 500, operations: 500 },
  recoverySupportDrawdownBps: [1000, 2500, 5000, 7500],
  recoveryPurchaseBpsBySupport: [500, 750, 1000, 1500],
} as const;

export type PolicyProceedsAllocation = {
  counterweightVault: bigint;
  liquidityReinforcement: bigint;
  burnReserve: bigint;
  operations: bigint;
  total: bigint;
};

export function allocatePolicyProceeds(total: bigint): PolicyProceedsAllocation {
  if (total < 0n) throw new Error("sale proceeds cannot be negative");
  const counterweightVault = total * 7000n / 10000n;
  const liquidityReinforcement = total * 2000n / 10000n;
  const burnReserve = total * 500n / 10000n;
  const operations = total - counterweightVault - liquidityReinforcement - burnReserve;
  return { counterweightVault, liquidityReinforcement, burnReserve, operations, total };
}
