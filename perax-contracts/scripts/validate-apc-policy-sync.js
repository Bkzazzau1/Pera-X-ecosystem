const fs = require('fs');
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
  const rawExpression = match[1].trim();
  if (rawExpression.includes('* PEX_DECIMALS')) return BigInt(rawExpression.split('*')[0].replaceAll('_', '').trim());
  if (rawExpression.includes('* APC_USDC_BASE_UNITS')) return BigInt(rawExpression.split('*')[0].replaceAll('_', '').trim());
  return BigInt(rawExpression.replaceAll('_', ''));
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
