import test from "node:test";
import assert from "node:assert/strict";
import fs from "node:fs";
import { fileURLToPath } from "node:url";
import { APC_POLICY_V1, APC_POLICY_V1_HASH_SHA256, allocatePolicyProceeds } from "../src/policy.js";

const policyPath = fileURLToPath(new URL("../../perax-contracts/config/apc-policy-v1.json", import.meta.url));
const policy = JSON.parse(fs.readFileSync(policyPath, "utf8"));

test("market engine uses the exact canonical APC Policy V1", () => {
  assert.equal(APC_POLICY_V1_HASH_SHA256, policy.policyHashSha256);
  assert.deepEqual(APC_POLICY_V1.riskVelocityThresholdsBps, policy.parameters.riskVelocityThresholdsBps);
  assert.deepEqual(APC_POLICY_V1.riskVolatilityThresholdsBps, policy.parameters.riskVolatilityThresholdsBps);
  assert.deepEqual(APC_POLICY_V1.riskPriceImpactThresholdsBps, policy.parameters.riskPriceImpactThresholdsBps);
  assert.deepEqual(APC_POLICY_V1.bandIntervalBpsByRisk, policy.parameters.bandIntervalBpsByRisk);
  assert.deepEqual(APC_POLICY_V1.bandReleaseBpsByRisk, policy.parameters.bandReleaseBpsByRisk);
  assert.deepEqual(APC_POLICY_V1.cascadeReductionBps, policy.parameters.cascadeReductionBps);
  assert.equal(APC_POLICY_V1.maximumObservationAgeSeconds, policy.parameters.maximumObservationAgeSeconds);
  assert.equal(APC_POLICY_V1.maximumFutureClockSkewSeconds, policy.parameters.maximumFutureClockSkewSeconds);
  assert.equal(APC_POLICY_V1.minimumTwapMinutes, policy.parameters.minimumTwapMinutes);
  assert.equal(APC_POLICY_V1.minimumLiquidityUsd, BigInt(policy.parameters.minimumLiquidityUsd));
  assert.equal(APC_POLICY_V1.minimumQuoteLiquidityUsd, BigInt(policy.parameters.minimumQuoteLiquidityUsd));
  assert.equal(APC_POLICY_V1.minimumVolumeUsd, BigInt(policy.parameters.minimumVolumeUsd));
  assert.equal(APC_POLICY_V1.minimumBuyPressureBps, policy.parameters.minimumBuyPressureBps);
  assert.deepEqual(APC_POLICY_V1.proceedsAllocationBps, policy.parameters.proceedsAllocationBps);
  assert.deepEqual(APC_POLICY_V1.recoverySupportDrawdownBps, policy.parameters.recoverySupportDrawdownBps);
  assert.deepEqual(APC_POLICY_V1.recoveryPurchaseBpsBySupport, policy.parameters.recoveryPurchaseBpsBySupport);
});

test("policy proceeds allocation is exact and preserves all integer dust", () => {
  const split = allocatePolicyProceeds(1_000_003n);
  assert.equal(split.counterweightVault, 700_002n);
  assert.equal(split.liquidityReinforcement, 200_000n);
  assert.equal(split.burnReserve, 50_000n);
  assert.equal(split.operations, 50_001n);
  assert.equal(split.counterweightVault + split.liquidityReinforcement + split.burnReserve + split.operations, split.total);
});
