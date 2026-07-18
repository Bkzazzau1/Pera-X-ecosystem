from pathlib import Path
import json
import re

ROOT = Path.cwd()
CONTRACTS = ROOT / "perax-contracts"
ENGINE = ROOT / "perax-market-engine"
POLICY_PATH = CONTRACTS / "config/apc-policy-v1.json"
TOKENOMICS_PATH = CONTRACTS / "config/pex-tokenomics.json"
POLICY = json.loads(POLICY_PATH.read_text())
P = POLICY["parameters"]
HASH = POLICY["policyHashSha256"]


def replace_once(path: Path, old: str, new: str) -> None:
    text = path.read_text()
    count = text.count(old)
    if count != 1:
        raise SystemExit(f"{path}: expected one match, found {count}: {old[:120]!r}")
    path.write_text(text.replace(old, new))


def append_once(path: Path, marker: str, content: str) -> None:
    text = path.read_text()
    if marker in text:
        raise SystemExit(f"{path}: marker already present: {marker}")
    path.write_text(text.rstrip() + "\n\n" + content.rstrip() + "\n")


# ---------------------------------------------------------------------------
# Canonical tokenomics policy.
# ---------------------------------------------------------------------------
tokenomics = json.loads(TOKENOMICS_PATH.read_text())
tokenomics["adaptivePriceControl"] = {
    "model": "deterministic_adaptive_price_control",
    "policyStatus": "approved",
    "policyVersion": POLICY["policyVersion"],
    "policyHashSha256": HASH,
    "approvedParameters": P,
    "launchPriceUsd": "0.000012",
    "priceScale": str(P["priceScale"]),
    "launchPriceScaled": str(P["launchPriceScaled"]),
    "firstActivationMultiplier": 3,
    "firstActivationPriceUsd": "0.000036",
    "firstActivationPriceScaled": str(P["firstActivationPriceScaled"]),
    "fixedMultiplicationAfterFirstActivation": False,
    "bandPolicy": {
        "sequentialActivation": True,
        "multiBandPerFreshObservation": True,
        "immutableBandRecords": True,
        "usedBandCapacityNeverRestores": True,
        "minimumIntervalBps": P["minimumBandIntervalBps"],
        "maximumIntervalBps": P["maximumBandIntervalBps"],
        "riskTierThresholds": {
            "velocityBps": P["riskVelocityThresholdsBps"],
            "volatilityBps": P["riskVolatilityThresholdsBps"],
            "priceImpactBps": P["riskPriceImpactThresholdsBps"],
        },
        "intervalBpsByRisk": P["bandIntervalBpsByRisk"],
        "releaseBpsByRisk": P["bandReleaseBpsByRisk"],
        "baseBandReleaseCapAmount": P["baseBandReleaseCapPex"],
        "higherRiskMustNotIncreaseIntervalOrRelease": True,
        "intervalFormula": "exact_policy_v1_risk_table",
        "releaseCapFormula": "base_cap_times_risk_bps_times_cascade_bps",
        "cascadeReductionBps": P["cascadeReductionBps"],
    },
    "observationPolicy": {
        "permanentObservationPda": True,
        "strictlyIncreasingSequence": True,
        "freshObservationPerRelease": True,
        "trustedClockForWindows": True,
        "maximumObservationAgeSeconds": P["maximumObservationAgeSeconds"],
        "maximumFutureClockSkewSeconds": P["maximumFutureClockSkewSeconds"],
        "minimumTwapMinutes": P["minimumTwapMinutes"],
        "minimumLiquidityUsd": P["minimumLiquidityUsd"],
        "minimumQuoteLiquidityUsd": P["minimumQuoteLiquidityUsd"],
        "minimumVolumeUsd": P["minimumVolumeUsd"],
        "minimumBuyPressureBps": P["minimumBuyPressureBps"],
        "approvedPoolOnly": True,
    },
    "releaseCaps": {
        "perBandHardCapRequired": True,
        "hourlyCapAmount": P["hourlyReleaseCapPex"],
        "pumpWindowCapAmount": P["pumpWindowReleaseCapPex"],
        "pumpWindowSeconds": P["pumpWindowSeconds"],
        "dailyCapAmount": "10000000",
        "monthlyCapAmount": "150000000",
        "unconfirmedExposureCapRequired": True,
    },
    "counterweightPolicy": {
        "quoteMint": "USDC",
        "realSplTransferRequired": True,
        "pdaControlledVault": True,
        "minimumCoverageBps": P["minimumCounterweightCoverageBps"],
        "proceedsAllocationBps": P["proceedsAllocationBps"],
        "missingCoverageStopsLaterReleases": True,
    },
    "burnDeferralPolicy": {
        "enabledDuringPumpControl": True,
        "pexEscrowRequired": True,
        "permanentDeferredBurnRecord": True,
        "resumptionRateBps": P["deferredBurnResumptionRateBps"],
        "executionWindowCapAmount": P["deferredBurnWindowCapPex"],
        "executionWindowSeconds": P["deferredBurnWindowSeconds"],
        "executionCooldownSeconds": P["deferredBurnCooldownSeconds"],
    },
    "recoveryPolicy": {
        "atomicSwapRequired": True,
        "approvedAdapterOnly": True,
        "lockedRecoveryVault": True,
        "hardSpendingCapRequired": True,
        "hardSpendingCapAmount": P["recoveryTotalSpendingCapUsdc"],
        "maximumPurchaseBps": P["maximumRecoveryPurchaseBps"],
        "minimumReserveBps": P["minimumCounterweightReserveBps"],
        "windowCapAmount": P["recoveryWindowCapUsdc"],
        "windowSeconds": P["recoveryWindowSeconds"],
        "cooldownSeconds": P["recoveryCooldownSeconds"],
        "supportDrawdownBps": P["recoverySupportDrawdownBps"],
        "purchaseBpsBySupport": P["recoveryPurchaseBpsBySupport"],
        "recoveredPexAutomaticallyBurned": False,
        "recoveredPexAutomaticallyRecirculated": False,
    },
    "authorityPolicy": {
        "observationAuthority": "autonomous_oracle_signer",
        "routineReleaseApproval": "none",
        "requiresManualOrMultisigApproval": False,
        "safetyAuthority": "pause_and_recovery_safety_only",
    },
    "unresolvedNumericalPolicies": [],
}
TOKENOMICS_PATH.write_text(json.dumps(tokenomics, indent=2) + "\n")


# ---------------------------------------------------------------------------
# Strict tokenomics validator and exact cross-file checker.
# ---------------------------------------------------------------------------
validator = CONTRACTS / "scripts/validate-tokenomics.js"
text = validator.read_text()
if "const crypto = require('crypto');" not in text:
    text = text.replace("const path = require('path');\n", "const path = require('path');\nconst crypto = require('crypto');\n")
if "const APC_POLICY_PATH" not in text:
    text = text.replace(
        "const CONFIG_PATH = path.resolve(__dirname, '../config/pex-tokenomics.json');\n",
        "const CONFIG_PATH = path.resolve(__dirname, '../config/pex-tokenomics.json');\nconst APC_POLICY_PATH = path.resolve(__dirname, '../config/apc-policy-v1.json');\n",
    )
start = text.index("function validateAdaptivePriceControl(config) {")
end = text.index("\nfunction validateWalletTemplate(config) {", start)
new_validator = r'''function canonicalJson(value) {
  if (Array.isArray(value)) return `[${value.map(canonicalJson).join(',')}]`;
  if (value && typeof value === 'object') {
    return `{${Object.keys(value).sort().map((key) => `${JSON.stringify(key)}:${canonicalJson(value[key])}`).join(',')}}`;
  }
  return JSON.stringify(value);
}

function assertNoNullProductionFields(value, label) {
  assert(value !== null && value !== undefined, `${label} cannot be null or undefined in an approved APC policy.`);
  if (Array.isArray(value)) {
    assert(value.length > 0, `${label} cannot be an empty array in an approved APC policy.`);
    value.forEach((entry, index) => assertNoNullProductionFields(entry, `${label}[${index}]`));
  } else if (typeof value === 'object') {
    const entries = Object.entries(value);
    assert(entries.length > 0, `${label} cannot be an empty object in an approved APC policy.`);
    entries.forEach(([key, entry]) => assertNoNullProductionFields(entry, `${label}.${key}`));
  }
}

function validateAdaptivePriceControl(config) {
  const apc = config.adaptivePriceControl;
  const official = readJson(APC_POLICY_PATH, 'APC Policy V1');
  assert(apc, 'Missing adaptivePriceControl section.');
  assert(!config.unlocking, 'Legacy unlocking section must be removed.');
  assert(apc.model === 'deterministic_adaptive_price_control', 'APC model is invalid.');
  assert(apc.launchPriceUsd === config.token.initialPriceUsd, 'APC launch price must match token launch price.');

  const launchScaled = toBigIntAmount(apc.launchPriceScaled, 'adaptivePriceControl.launchPriceScaled');
  const firstScaled = toBigIntAmount(apc.firstActivationPriceScaled, 'adaptivePriceControl.firstActivationPriceScaled');
  assert(apc.firstActivationMultiplier === 3, 'First activation multiplier must be exactly 3.');
  assert(firstScaled === launchScaled * 3n, 'First activation must equal exactly three times launch price.');
  assert(apc.firstActivationPriceUsd === '0.000036', 'First activation price must be $0.000036.');
  assert(apc.fixedMultiplicationAfterFirstActivation === false, 'Fixed multiplication must be disabled after first activation.');

  if (apc.policyStatus === 'approved') {
    assert(official.status === 'approved' && official.policyVersion === 1, 'Canonical APC Policy V1 must be approved.');
    assert(apc.policyVersion === official.policyVersion, 'Tokenomics APC policy version mismatch.');
    const calculatedHash = crypto.createHash('sha256').update(canonicalJson(official.parameters)).digest('hex');
    assert(calculatedHash === official.policyHashSha256, 'Canonical APC Policy V1 hash is invalid.');
    assert(apc.policyHashSha256 === official.policyHashSha256, 'Tokenomics APC policy hash mismatch.');
    assertNoNullProductionFields(apc.approvedParameters, 'adaptivePriceControl.approvedParameters');
    assert(canonicalJson(apc.approvedParameters) === canonicalJson(official.parameters), 'Tokenomics approved parameters differ from canonical APC Policy V1.');
    assert(Array.isArray(apc.unresolvedNumericalPolicies) && apc.unresolvedNumericalPolicies.length === 0, 'Approved APC policy cannot contain unresolved parameters.');
  } else {
    assert(apc.policyStatus === 'pending_formal_numerical_approval', 'Unknown APC policy status.');
    assert(Array.isArray(apc.unresolvedNumericalPolicies) && apc.unresolvedNumericalPolicies.length === 10, 'Pending APC policy must list every unresolved parameter.');
    return;
  }

  const p = official.parameters;
  const bands = apc.bandPolicy;
  assert(bands.sequentialActivation === true && bands.multiBandPerFreshObservation === true, 'Sequential multi-band activation is required.');
  assert(bands.immutableBandRecords === true && bands.usedBandCapacityNeverRestores === true, 'Permanent non-restoring band records are required.');
  assert(bands.minimumIntervalBps === p.minimumBandIntervalBps && bands.maximumIntervalBps === p.maximumBandIntervalBps, 'Band interval bounds differ from Policy V1.');
  assert(canonicalJson(bands.riskTierThresholds) === canonicalJson({ velocityBps: p.riskVelocityThresholdsBps, volatilityBps: p.riskVolatilityThresholdsBps, priceImpactBps: p.riskPriceImpactThresholdsBps }), 'Risk thresholds differ from Policy V1.');
  assert(canonicalJson(bands.intervalBpsByRisk) === canonicalJson(p.bandIntervalBpsByRisk), 'Risk interval table differs from Policy V1.');
  assert(canonicalJson(bands.releaseBpsByRisk) === canonicalJson(p.bandReleaseBpsByRisk), 'Risk release table differs from Policy V1.');
  assert(bands.baseBandReleaseCapAmount === p.baseBandReleaseCapPex, 'Base band release cap differs from Policy V1.');
  assert(canonicalJson(bands.cascadeReductionBps) === canonicalJson(p.cascadeReductionBps), 'Cascade table differs from Policy V1.');
  for (let index = 1; index < 4; index += 1) {
    assert(bands.intervalBpsByRisk[index - 1] >= bands.intervalBpsByRisk[index], 'Higher risk must not widen APC bands.');
    assert(bands.releaseBpsByRisk[index - 1] >= bands.releaseBpsByRisk[index], 'Higher risk must not increase APC releases.');
  }

  const observations = apc.observationPolicy;
  assert(observations.maximumObservationAgeSeconds === p.maximumObservationAgeSeconds, 'Observation maximum age mismatch.');
  assert(observations.maximumFutureClockSkewSeconds === p.maximumFutureClockSkewSeconds, 'Observation future skew mismatch.');
  assert(observations.minimumTwapMinutes === p.minimumTwapMinutes, 'Minimum TWAP mismatch.');
  assert(observations.minimumLiquidityUsd === p.minimumLiquidityUsd, 'Minimum total liquidity mismatch.');
  assert(observations.minimumQuoteLiquidityUsd === p.minimumQuoteLiquidityUsd, 'Minimum quote liquidity mismatch.');
  assert(observations.minimumVolumeUsd === p.minimumVolumeUsd, 'Minimum volume mismatch.');
  assert(observations.minimumBuyPressureBps === p.minimumBuyPressureBps, 'Minimum buy pressure mismatch.');
  assert(observations.permanentObservationPda && observations.strictlyIncreasingSequence && observations.freshObservationPerRelease && observations.trustedClockForWindows && observations.approvedPoolOnly, 'Observation safety flags are incomplete.');

  const caps = apc.releaseCaps;
  assert(caps.hourlyCapAmount === p.hourlyReleaseCapPex, 'Hourly release cap mismatch.');
  assert(caps.pumpWindowCapAmount === p.pumpWindowReleaseCapPex, 'Pump-window release cap mismatch.');
  assert(caps.pumpWindowSeconds === p.pumpWindowSeconds, 'Pump-window duration mismatch.');
  assert(caps.dailyCapAmount === config.marketConditionalReleasePolicy.dailyReleaseCapAmount, 'APC daily cap must match the global cap.');
  assert(caps.monthlyCapAmount === config.marketConditionalReleasePolicy.monthlyReleaseCapAmount, 'APC monthly cap must match the global cap.');
  assert(toBigIntAmount(caps.hourlyCapAmount, 'hourlyCapAmount') <= toBigIntAmount(caps.dailyCapAmount, 'dailyCapAmount'), 'Hourly cap exceeds daily cap.');
  assert(toBigIntAmount(caps.pumpWindowCapAmount, 'pumpWindowCapAmount') <= toBigIntAmount(caps.monthlyCapAmount, 'monthlyCapAmount'), 'Pump cap exceeds monthly cap.');

  const counterweight = apc.counterweightPolicy;
  assert(counterweight.minimumCoverageBps === p.minimumCounterweightCoverageBps, 'Counterweight coverage mismatch.');
  assert(canonicalJson(counterweight.proceedsAllocationBps) === canonicalJson(p.proceedsAllocationBps), 'Proceeds allocation mismatch.');
  assert(Object.values(counterweight.proceedsAllocationBps).reduce((sum, value) => sum + value, 0) === 10000, 'Proceeds allocations must total 10,000 bps.');
  assert(counterweight.realSplTransferRequired && counterweight.pdaControlledVault && counterweight.missingCoverageStopsLaterReleases, 'Counterweight custody flags are incomplete.');

  const burn = apc.burnDeferralPolicy;
  assert(burn.resumptionRateBps === p.deferredBurnResumptionRateBps, 'Deferred-burn resumption rate mismatch.');
  assert(burn.executionWindowCapAmount === p.deferredBurnWindowCapPex, 'Deferred-burn window cap mismatch.');
  assert(burn.executionWindowSeconds === p.deferredBurnWindowSeconds, 'Deferred-burn window duration mismatch.');
  assert(burn.executionCooldownSeconds === p.deferredBurnCooldownSeconds, 'Deferred-burn cooldown mismatch.');
  assert(burn.enabledDuringPumpControl && burn.pexEscrowRequired && burn.permanentDeferredBurnRecord, 'Deferred-burn custody flags are incomplete.');

  const recovery = apc.recoveryPolicy;
  assert(recovery.hardSpendingCapAmount === p.recoveryTotalSpendingCapUsdc, 'Recovery total cap mismatch.');
  assert(recovery.maximumPurchaseBps === p.maximumRecoveryPurchaseBps, 'Recovery purchase cap mismatch.');
  assert(recovery.minimumReserveBps === p.minimumCounterweightReserveBps, 'Recovery reserve floor mismatch.');
  assert(recovery.windowCapAmount === p.recoveryWindowCapUsdc, 'Recovery window cap mismatch.');
  assert(recovery.windowSeconds === p.recoveryWindowSeconds, 'Recovery window duration mismatch.');
  assert(recovery.cooldownSeconds === p.recoveryCooldownSeconds, 'Recovery cooldown mismatch.');
  assert(canonicalJson(recovery.supportDrawdownBps) === canonicalJson(p.recoverySupportDrawdownBps), 'Recovery support bands mismatch.');
  assert(canonicalJson(recovery.purchaseBpsBySupport) === canonicalJson(p.recoveryPurchaseBpsBySupport), 'Recovery purchase bands mismatch.');
  assert(recovery.atomicSwapRequired && recovery.approvedAdapterOnly && recovery.lockedRecoveryVault && recovery.hardSpendingCapRequired, 'Recovery custody flags are incomplete.');
  assert(recovery.recoveredPexAutomaticallyBurned === false && recovery.recoveredPexAutomaticallyRecirculated === false, 'Recovered PEX must remain locked.');

  assert(apc.authorityPolicy.requiresManualOrMultisigApproval === false, 'Manual release approval must remain disabled.');
  assert(apc.authorityPolicy.routineReleaseApproval === 'none', 'Routine APC releases must remain autonomous.');
}
'''
text = text[:start] + new_validator + text[end:]
validator.write_text(text)

sync_script = CONTRACTS / "scripts/validate-apc-policy-sync.js"
sync_script.write_text(r'''const fs = require('fs');
const path = require('path');
const crypto = require('crypto');

const root = path.resolve(__dirname, '..');
const policy = JSON.parse(fs.readFileSync(path.join(root, 'config/apc-policy-v1.json'), 'utf8'));
const tokenomics = JSON.parse(fs.readFileSync(path.join(root, 'config/pex-tokenomics.json'), 'utf8'));
const constants = fs.readFileSync(path.join(root, 'programs/perax-core/src/constants.rs'), 'utf8');

function assert(condition, message) { if (!condition) throw new Error(message); }
function canonical(value) {
  if (Array.isArray(value)) return `[${value.map(canonical).join(',')}]`;
  if (value && typeof value === 'object') return `{${Object.keys(value).sort().map((key) => `${JSON.stringify(key)}:${canonical(value[key])}`).join(',')}}`;
  return JSON.stringify(value);
}
function rustNumber(name) {
  const match = constants.match(new RegExp(`pub const ${name}: [^=]+ = ([^;]+);`));
  assert(match, `Missing Rust constant ${name}`);
  const expression = match[1].replaceAll('_', '').trim();
  if (expression.includes('* PEX_DECIMALS')) return BigInt(expression.split('*')[0].trim());
  if (expression.includes('* APC_USDC_BASE_UNITS')) return BigInt(expression.split('*')[0].trim());
  return BigInt(expression);
}
function rustArray(name) {
  const match = constants.match(new RegExp(`pub const ${name}: \\[[^\\]]+\\] = \\[([^\\]]+)\\];`));
  assert(match, `Missing Rust array ${name}`);
  return match[1].split(',').map((value) => Number(value.replaceAll('_', '').trim())).filter(Number.isFinite);
}
const p = policy.parameters;
const calculatedHash = crypto.createHash('sha256').update(canonical(p)).digest('hex');
assert(calculatedHash === policy.policyHashSha256, 'Canonical policy hash mismatch.');
assert(tokenomics.adaptivePriceControl.policyStatus === 'approved', 'Tokenomics policy is not approved.');
assert(tokenomics.adaptivePriceControl.unresolvedNumericalPolicies.length === 0, 'Approved tokenomics still has unresolved parameters.');
assert(canonical(tokenomics.adaptivePriceControl.approvedParameters) === canonical(p), 'Tokenomics parameters differ from canonical policy.');

const scalarMap = {
  APC_POLICY_VERSION: p.policyVersion,
  APC_PRICE_SCALE: p.priceScale,
  APC_FIRST_ACTIVATION_PRICE_SCALED: p.firstActivationPriceScaled,
  APC_MINIMUM_BAND_INTERVAL_BPS: p.minimumBandIntervalBps,
  APC_MAXIMUM_BAND_INTERVAL_BPS: p.maximumBandIntervalBps,
  APC_MAXIMUM_OBSERVATION_AGE_SECONDS: p.maximumObservationAgeSeconds,
  APC_MAXIMUM_FUTURE_CLOCK_SKEW_SECONDS: p.maximumFutureClockSkewSeconds,
  APC_MINIMUM_TWAP_MINUTES: p.minimumTwapMinutes,
  APC_MINIMUM_LIQUIDITY_USD: p.minimumLiquidityUsd,
  APC_MINIMUM_QUOTE_LIQUIDITY_USD: p.minimumQuoteLiquidityUsd,
  APC_MINIMUM_VOLUME_USD: p.minimumVolumeUsd,
  APC_MINIMUM_BUY_PRESSURE_BPS: p.minimumBuyPressureBps,
  APC_BASE_BAND_RELEASE_CAP: p.baseBandReleaseCapPex,
  APC_HOURLY_RELEASE_CAP: p.hourlyReleaseCapPex,
  APC_PUMP_WINDOW_RELEASE_CAP: p.pumpWindowReleaseCapPex,
  APC_PUMP_WINDOW_SECONDS: p.pumpWindowSeconds,
  APC_MINIMUM_COUNTERWEIGHT_COVERAGE_BPS: p.minimumCounterweightCoverageBps,
  APC_COUNTERWEIGHT_PROCEEDS_ALLOCATION_BPS: p.proceedsAllocationBps.counterweightVault,
  APC_LIQUIDITY_REINFORCEMENT_ALLOCATION_BPS: p.proceedsAllocationBps.liquidityReinforcement,
  APC_BURN_RESERVE_ALLOCATION_BPS: p.proceedsAllocationBps.burnReserve,
  APC_OPERATIONS_ALLOCATION_BPS: p.proceedsAllocationBps.operations,
  APC_DEFERRED_BURN_RESUMPTION_RATE_BPS: p.deferredBurnResumptionRateBps,
  APC_DEFERRED_BURN_WINDOW_CAP: p.deferredBurnWindowCapPex,
  APC_DEFERRED_BURN_WINDOW_SECONDS: p.deferredBurnWindowSeconds,
  APC_DEFERRED_BURN_COOLDOWN_SECONDS: p.deferredBurnCooldownSeconds,
  APC_RECOVERY_TOTAL_SPENDING_CAP: p.recoveryTotalSpendingCapUsdc,
  APC_MAXIMUM_RECOVERY_PURCHASE_BPS: p.maximumRecoveryPurchaseBps,
  APC_MINIMUM_COUNTERWEIGHT_RESERVE_BPS: p.minimumCounterweightReserveBps,
  APC_RECOVERY_WINDOW_CAP: p.recoveryWindowCapUsdc,
  APC_RECOVERY_WINDOW_SECONDS: p.recoveryWindowSeconds,
  APC_RECOVERY_COOLDOWN_SECONDS: p.recoveryCooldownSeconds,
};
for (const [name, expected] of Object.entries(scalarMap)) {
  assert(rustNumber(name) === BigInt(expected), `${name} differs from APC Policy V1.`);
}
const arrayMap = {
  APC_RISK_VELOCITY_THRESHOLDS_BPS: p.riskVelocityThresholdsBps,
  APC_RISK_VOLATILITY_THRESHOLDS_BPS: p.riskVolatilityThresholdsBps,
  APC_RISK_PRICE_IMPACT_THRESHOLDS_BPS: p.riskPriceImpactThresholdsBps,
  APC_INTERVAL_BPS_BY_RISK: p.bandIntervalBpsByRisk,
  APC_RELEASE_BPS_BY_RISK: p.bandReleaseBpsByRisk,
  APC_CASCADE_REDUCTION_BPS: p.cascadeReductionBps,
  APC_RECOVERY_SUPPORT_DRAWDOWN_BPS: p.recoverySupportDrawdownBps,
  APC_RECOVERY_PURCHASE_BPS_BY_SUPPORT: p.recoveryPurchaseBpsBySupport,
};
for (const [name, expected] of Object.entries(arrayMap)) {
  assert(canonical(rustArray(name)) === canonical(expected), `${name} differs from APC Policy V1.`);
}
const hashBytes = [...constants.matchAll(/0x([0-9a-f]{2})/g)].slice(0, 32).map((match) => match[1]).join('');
assert(hashBytes === policy.policyHashSha256, 'Rust APC policy hash differs from canonical policy.');
console.log('✅ APC Policy V1 is identical across canonical JSON, tokenomics and Rust constants.');
''')

package_path = CONTRACTS / "package.json"
package_json = json.loads(package_path.read_text())
package_json["scripts"]["validate:tokenomics"] = "node scripts/validate-tokenomics.js && node scripts/validate-apc-policy-sync.js"
package_json["scripts"]["simulate:apc-policy-v1"] = "node scripts/simulate-apc-policy-v1.js"
package_path.write_text(json.dumps(package_json, indent=2) + "\n")


# ---------------------------------------------------------------------------
# Safe planning and initialization gates.
# ---------------------------------------------------------------------------
(CONTRACTS / "scripts/plan-apc-initialize.js").write_text(r'''const fs = require('fs');
const path = require('path');
const crypto = require('crypto');
const tokenomics = JSON.parse(fs.readFileSync(path.resolve(__dirname, '../config/pex-tokenomics.json'), 'utf8'));
const policy = JSON.parse(fs.readFileSync(path.resolve(__dirname, '../config/apc-policy-v1.json'), 'utf8'));
function canonical(value) {
  if (Array.isArray(value)) return `[${value.map(canonical).join(',')}]`;
  if (value && typeof value === 'object') return `{${Object.keys(value).sort().map((key) => `${JSON.stringify(key)}:${canonical(value[key])}`).join(',')}}`;
  return JSON.stringify(value);
}
const apc = tokenomics.adaptivePriceControl;
const hash = crypto.createHash('sha256').update(canonical(policy.parameters)).digest('hex');
const policyReady = apc?.policyStatus === 'approved'
  && apc.policyVersion === policy.policyVersion
  && apc.policyHashSha256 === policy.policyHashSha256
  && hash === policy.policyHashSha256
  && canonical(apc.approvedParameters) === canonical(policy.parameters)
  && Array.isArray(apc.unresolvedNumericalPolicies)
  && apc.unresolvedNumericalPolicies.length === 0;
console.log(JSON.stringify({
  action: 'initialize_apc',
  policyReady,
  executionReady: false,
  policyVersion: policy.policyVersion,
  policyHashSha256: policy.policyHashSha256,
  parameters: policy.parameters,
  blockedBy: policyReady ? ['reviewed production addresses', 'successful full validation pipeline', 'independent security approval'] : ['exact APC Policy V1 approval and synchronization'],
}, null, 2));
if (!policyReady) process.exitCode = 1;
''')

(CONTRACTS / "scripts/initialize-apc-program.js").write_text(r'''const fs = require('fs');
const path = require('path');
const crypto = require('crypto');
const tokenomics = JSON.parse(fs.readFileSync(path.resolve(__dirname, '../config/pex-tokenomics.json'), 'utf8'));
const policy = JSON.parse(fs.readFileSync(path.resolve(__dirname, '../config/apc-policy-v1.json'), 'utf8'));
function canonical(value) {
  if (Array.isArray(value)) return `[${value.map(canonical).join(',')}]`;
  if (value && typeof value === 'object') return `{${Object.keys(value).sort().map((key) => `${JSON.stringify(key)}:${canonical(value[key])}`).join(',')}}`;
  return JSON.stringify(value);
}
const hash = crypto.createHash('sha256').update(canonical(policy.parameters)).digest('hex');
const apc = tokenomics.adaptivePriceControl;
const exact = apc?.policyStatus === 'approved'
  && apc.policyVersion === policy.policyVersion
  && apc.policyHashSha256 === hash
  && hash === policy.policyHashSha256
  && canonical(apc.approvedParameters) === canonical(policy.parameters)
  && apc.unresolvedNumericalPolicies?.length === 0;
if (!exact) throw new Error('APC initialization blocked: Policy V1 is not exact across canonical JSON and tokenomics.');
if (!process.argv.includes('--execute')) {
  console.log(`Dry run only. APC Policy V${policy.policyVersion} (${policy.policyHashSha256}) is exact, but execution remains blocked.`);
  process.exit(0);
}
throw new Error('APC initialization intentionally blocked until reviewed production addresses, full CI/local-validator proof, and independent security approval are supplied.');
''')


# ---------------------------------------------------------------------------
# Market-engine policy and proceeds allocation.
# ---------------------------------------------------------------------------
(ENGINE / "src/policy.ts").write_text(f'''export const APC_POLICY_V1_HASH_SHA256 = "{HASH}" as const;
export const APC_POLICY_V1 = {{
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
  proceedsAllocationBps: {{ counterweightVault: 7000, liquidityReinforcement: 2000, burnReserve: 500, operations: 500 }},
  recoverySupportDrawdownBps: [1000, 2500, 5000, 7500],
  recoveryPurchaseBpsBySupport: [500, 750, 1000, 1500],
}} as const;

export type PolicyProceedsAllocation = {{
  counterweightVault: bigint;
  liquidityReinforcement: bigint;
  burnReserve: bigint;
  operations: bigint;
  total: bigint;
}};

export function allocatePolicyProceeds(total: bigint): PolicyProceedsAllocation {{
  if (total < 0n) throw new Error("sale proceeds cannot be negative");
  const counterweightVault = total * 7000n / 10000n;
  const liquidityReinforcement = total * 2000n / 10000n;
  const burnReserve = total * 500n / 10000n;
  const operations = total - counterweightVault - liquidityReinforcement - burnReserve;
  return {{ counterweightVault, liquidityReinforcement, burnReserve, operations, total }};
}}
''')

(ENGINE / "src/types.ts").write_text('''import type { PolicyProceedsAllocation } from "./policy.js";
export type PriceSample = { price: bigint; liquidityUsd: bigint; quoteLiquidityUsd: bigint; volumeUsd: bigint; netBuyPressureBps: number; observedAt: number };
export type ApcSnapshot = { observationId: Uint8Array; sequence: bigint; pool: string; spotPrice: bigint; twapPrice: bigint; twapMinutes: bigint; liquidityUsd: bigint; quoteLiquidityUsd: bigint; volumeUsd: bigint; netBuyPressureBps: number; priceVelocityBps: number; volatilityBps: number; estimatedPriceImpactBps: number; observedAt: bigint };
export type ApcStateView = { status: "inactive"|"armed"|"active"|"pumpControl"|"awaitingAbsorption"|"recovery"|"paused"; currentBandIndex: number; nextBandPrice: bigint };
export interface MarketDataSource { readApprovedPool(): Promise<{ pool: string; samples: PriceSample[]; estimatedPriceImpactBps: number }>; }
export interface ApcProgramClient { readState(): Promise<ApcStateView>; submitObservation(snapshot: ApcSnapshot): Promise<void>; activateNextBand(observationId: Uint8Array, bandIndex: number): Promise<void>; readPermittedReleaseAmount(bandIndex: number, observationId: Uint8Array): Promise<bigint>; executePermittedRelease(bandIndex: number, observationId: Uint8Array, amount: bigint): Promise<void>; depositCounterweightProceeds(amount: bigint, settlementReference: Uint8Array): Promise<void>; enterRecovery(observationId: Uint8Array): Promise<void>; executeRecoveryPurchase(observationId: Uint8Array): Promise<void>; }
export interface ExecutionVenue { placeControlledSell(amount: bigint): Promise<{ actualUsdcProceeds: bigint; settlementReference: Uint8Array }>; recordPolicyAllocation?(allocation: PolicyProceedsAllocation, settlementReference: Uint8Array): Promise<void>; }
''')

(ENGINE / "src/engine.ts").write_text('''import { randomBytes } from "node:crypto";
import { calculateTwap, calculateVelocityBps, calculateVolatilityBps, effectivePrice } from "./metrics.js";
import { allocatePolicyProceeds, APC_POLICY_V1 } from "./policy.js";
import type { ApcProgramClient, ApcSnapshot, ExecutionVenue, MarketDataSource } from "./types.js";

export class ApcMarketEngine {
  constructor(
    private readonly market: MarketDataSource,
    private readonly program: ApcProgramClient,
    private readonly venue: ExecutionVenue,
    private readonly maxBandsPerCycle = 32,
  ) {}

  async runCycle(sequence: bigint): Promise<void> {
    const market = await this.market.readApprovedPool();
    if (!market.samples.length) throw new Error("approved pool returned no samples");
    const latest = market.samples[market.samples.length - 1]!;
    const first = market.samples[0]!;
    const twap = calculateTwap(market.samples);
    const twapMinutes = Math.max(1, Math.floor((latest.observedAt - first.observedAt) / 60));
    if (twapMinutes < APC_POLICY_V1.minimumTwapMinutes) throw new Error("approved pool TWAP window is below APC Policy V1");
    if (latest.liquidityUsd < APC_POLICY_V1.minimumLiquidityUsd) throw new Error("approved pool liquidity is below APC Policy V1");
    if (latest.quoteLiquidityUsd < APC_POLICY_V1.minimumQuoteLiquidityUsd) throw new Error("approved quote liquidity is below APC Policy V1");
    if (latest.volumeUsd < APC_POLICY_V1.minimumVolumeUsd) throw new Error("approved pool volume is below APC Policy V1");
    if (latest.netBuyPressureBps < APC_POLICY_V1.minimumBuyPressureBps) throw new Error("approved buy pressure is below APC Policy V1");

    const snapshot: ApcSnapshot = {
      observationId: randomBytes(32), sequence, pool: market.pool,
      spotPrice: latest.price, twapPrice: twap, twapMinutes: BigInt(twapMinutes),
      liquidityUsd: latest.liquidityUsd, quoteLiquidityUsd: latest.quoteLiquidityUsd,
      volumeUsd: latest.volumeUsd, netBuyPressureBps: latest.netBuyPressureBps,
      priceVelocityBps: calculateVelocityBps(market.samples),
      volatilityBps: calculateVolatilityBps(market.samples, twap),
      estimatedPriceImpactBps: market.estimatedPriceImpactBps,
      observedAt: BigInt(latest.observedAt),
    };
    await this.program.submitObservation(snapshot);
    let state = await this.program.readState();
    for (let activated = 0; activated < this.maxBandsPerCycle && effectivePrice(snapshot.spotPrice, snapshot.twapPrice) >= state.nextBandPrice; activated += 1) {
      await this.program.activateNextBand(snapshot.observationId, state.currentBandIndex + 1);
      state = await this.program.readState();
    }
    if (state.status === "recovery") {
      await this.program.executeRecoveryPurchase(snapshot.observationId);
      return;
    }
    const amount = await this.program.readPermittedReleaseAmount(state.currentBandIndex, snapshot.observationId);
    if (amount <= 0n) return;
    await this.program.executePermittedRelease(state.currentBandIndex, snapshot.observationId, amount);
    const settlement = await this.venue.placeControlledSell(amount);
    const allocation = allocatePolicyProceeds(settlement.actualUsdcProceeds);
    if (allocation.counterweightVault > 0n) {
      await this.program.depositCounterweightProceeds(allocation.counterweightVault, settlement.settlementReference);
    }
    await this.venue.recordPolicyAllocation?.(allocation, settlement.settlementReference);
  }
}
''')

(ENGINE / "test").mkdir(exist_ok=True)
(ENGINE / "test/policy.test.ts").write_text(r'''import test from "node:test";
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
''')


# ---------------------------------------------------------------------------
# Anchor transaction initialization uses the exact policy and rejects mismatch.
# ---------------------------------------------------------------------------
tx = CONTRACTS / "tests/perax-core.ts"
tx_text = tx.read_text()
old_start = tx_text.index("    await program.methods\n      .initializeApc({")
old_end_marker = "      .rpc();\n\n    const currentChainTime"
old_end = tx_text.index(old_end_marker, old_start)
new_init = f'''    const officialApcPolicy = {{
      policyVersion: 1,
      policyHash: Array.from(Buffer.from("{HASH}", "hex")),
      quoteMint,
      approvedPool: recoveryPool,
      approvedProceedsOwner: proceedsOwner.publicKey,
      approvedProceedsTokenAccount: proceedsTokenAccount,
      approvedRecoveryProgram: program.programId,
      priceScale: new anchor.BN(100_000_000),
      firstActivationPrice: new anchor.BN(3_600),
      minimumBandIntervalBps: 750,
      maximumBandIntervalBps: 2_000,
      maximumObservationAgeSeconds: new anchor.BN(90),
      maximumFutureClockSkewSeconds: new anchor.BN(5),
      hourlyReleaseCap: new anchor.BN(2_500_000 * BASE_UNITS),
      pumpWindowReleaseCap: new anchor.BN(6_000_000 * BASE_UNITS),
      pumpWindowSeconds: new anchor.BN(21_600),
      minimumCounterweightCoverageBps: 5_000,
      counterweightProceedsAllocationBps: 7_000,
      liquidityReinforcementAllocationBps: 2_000,
      burnReserveAllocationBps: 500,
      operationsAllocationBps: 500,
      baseBandReleaseCap: new anchor.BN(2_000_000 * BASE_UNITS),
      minimumTwapMinutes: new anchor.BN(60),
      minimumLiquidityUsd: new anchor.BN(27_360),
      minimumQuoteLiquidityUsd: new anchor.BN(13_680),
      minimumVolumeUsd: new anchor.BN(6_840),
      minimumBuyPressureBps: 5_500,
      riskVelocityThresholdsBps: [500, 1_500, 3_000],
      riskVolatilityThresholdsBps: [400, 1_200, 2_500],
      riskPriceImpactThresholdsBps: [250, 750, 1_500],
      bandIntervalBpsByRisk: [2_000, 1_500, 1_000, 750],
      bandReleaseBpsByRisk: [10_000, 7_500, 5_000, 2_500],
      cascadeReductionBps: [10_000, 7_500, 5_000, 2_500],
      recoverySpendingCap: new anchor.BN(3_000_000_000),
      deferredBurnWindowCap: new anchor.BN(400_000 * BASE_UNITS),
      deferredBurnWindowSeconds: new anchor.BN(3_600),
      deferredBurnCooldownSeconds: new anchor.BN(900),
      deferredBurnResumptionRateBps: 1_000,
      maximumRecoveryPurchaseBps: 1_500,
      minimumCounterweightReserveBps: 3_000,
      recoveryWindowCap: new anchor.BN(500_000_000),
      recoveryWindowSeconds: new anchor.BN(21_600),
      recoveryCooldownSeconds: new anchor.BN(1_800),
      recoverySupportDrawdownBps: [1_000, 2_500, 5_000, 7_500],
      recoveryPurchaseBpsBySupport: [500, 750, 1_000, 1_500],
    }};
    const apcInitializationAccounts = {{
      state, authority, apcConfig, apcState, counterweightConfig,
      counterweightAuthority, counterweightVault, deferredBurnAuthority,
      deferredBurnVault, recoveryAuthority, recoveryVault, quoteMint,
      tokenMint: mint, tokenProgram: TOKEN_PROGRAM_ID,
      associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
      systemProgram: anchor.web3.SystemProgram.programId,
    }};
    await expectFailure(() =>
      program.methods
        .initializeApc({{ ...officialApcPolicy, hourlyReleaseCap: officialApcPolicy.hourlyReleaseCap.sub(new anchor.BN(1)) }})
        .accounts(apcInitializationAccounts)
        .rpc()
    );
    await program.methods
      .initializeApc(officialApcPolicy)
      .accounts(apcInitializationAccounts)
      .rpc();

    const currentChainTime'''
tx_text = tx_text[:old_start] + new_init + tx_text[old_end + len(old_end_marker):]
tx.write_text(tx_text)


# ---------------------------------------------------------------------------
# Complete plus/minus exact-policy Rust test.
# ---------------------------------------------------------------------------
rust_tests = CONTRACTS / "programs/perax-core/src/tests.rs"
append_once(
    rust_tests,
    "every_apc_policy_v1_field_rejects_plus_and_minus_one",
    r'''// every_apc_policy_v1_field_rejects_plus_and_minus_one
#[test]
fn every_apc_policy_v1_field_rejects_plus_and_minus_one() {
    macro_rules! reject_scalar {
        ($field:ident) => {{
            assert_apc_policy_mutation_rejected(|p| p.$field -= 1);
            assert_apc_policy_mutation_rejected(|p| p.$field += 1);
        }};
    }
    macro_rules! reject_array {
        ($field:ident, $length:expr) => {{
            for index in 0..$length {
                assert_apc_policy_mutation_rejected(|p| p.$field[index] -= 1);
                assert_apc_policy_mutation_rejected(|p| p.$field[index] += 1);
            }
        }};
    }
    reject_scalar!(policy_version);
    assert_apc_policy_mutation_rejected(|p| p.policy_hash[31] ^= 1);
    reject_scalar!(price_scale);
    reject_scalar!(first_activation_price);
    reject_scalar!(minimum_band_interval_bps);
    reject_scalar!(maximum_band_interval_bps);
    reject_scalar!(maximum_observation_age_seconds);
    reject_scalar!(maximum_future_clock_skew_seconds);
    reject_scalar!(hourly_release_cap);
    reject_scalar!(pump_window_release_cap);
    reject_scalar!(pump_window_seconds);
    reject_scalar!(minimum_counterweight_coverage_bps);
    reject_scalar!(counterweight_proceeds_allocation_bps);
    reject_scalar!(liquidity_reinforcement_allocation_bps);
    reject_scalar!(burn_reserve_allocation_bps);
    reject_scalar!(operations_allocation_bps);
    reject_scalar!(base_band_release_cap);
    reject_scalar!(minimum_twap_minutes);
    reject_scalar!(minimum_liquidity_usd);
    reject_scalar!(minimum_quote_liquidity_usd);
    reject_scalar!(minimum_volume_usd);
    reject_scalar!(minimum_buy_pressure_bps);
    reject_array!(risk_velocity_thresholds_bps, 3);
    reject_array!(risk_volatility_thresholds_bps, 3);
    reject_array!(risk_price_impact_thresholds_bps, 3);
    reject_array!(band_interval_bps_by_risk, 4);
    reject_array!(band_release_bps_by_risk, 4);
    reject_array!(cascade_reduction_bps, 4);
    reject_scalar!(recovery_spending_cap);
    reject_scalar!(deferred_burn_window_cap);
    reject_scalar!(deferred_burn_window_seconds);
    reject_scalar!(deferred_burn_cooldown_seconds);
    reject_scalar!(deferred_burn_resumption_rate_bps);
    reject_scalar!(maximum_recovery_purchase_bps);
    reject_scalar!(minimum_counterweight_reserve_bps);
    reject_scalar!(recovery_window_cap);
    reject_scalar!(recovery_window_seconds);
    reject_scalar!(recovery_cooldown_seconds);
    reject_array!(recovery_support_drawdown_bps, 4);
    reject_array!(recovery_purchase_bps_by_support, 4);
}
''',
)


# ---------------------------------------------------------------------------
# Documentation synchronization.
# ---------------------------------------------------------------------------
policy_table = f'''<!-- APC_POLICY_V1_SYNC -->
## APC Numerical Policy Version 1

Correction 2 uses immutable APC Policy V1, policy hash `{HASH}`. The canonical machine-readable source is `perax-contracts/config/apc-policy-v1.json`; contract initialization rejects every numerical or hash difference. The deterministic selection evaluated 2,916 candidates, 1,920 market scenarios and 25,000 randomized invariant cases. The selected policy produced a 498-bps worst modeled APC-added impact and 1.365× minimum counterweight coverage.

Key controls: 750–2,000 bps adaptive bands; 2,000,000 PEX base band cap; 2,500,000 PEX hourly cap; 6,000,000 PEX six-hour pump cap; 70% counterweight allocation; 10% deferred-burn resumption; 30% protected recovery reserve; and four drawdown support bands at 10%, 25%, 50% and 75%.

No deployment, initialization, reserve movement or migration is authorized by this policy approval. Runtime freeze still requires the complete validation pipeline and independent security approval.
'''
for relative in ["README.md", "docs/APC_LOGIC_SPECIFICATION.md", "docs/APC_VALIDATION_STATUS.md"]:
    append_once(ROOT / relative, "APC_POLICY_V1_SYNC", policy_table)

print("APC Policy V1 synchronized across tokenomics, validator, scripts, market engine, tests and documentation")
