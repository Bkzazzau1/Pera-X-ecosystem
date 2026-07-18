const fs = require('fs');
const path = require('path');
const crypto = require('crypto');

const CONFIG_PATH = path.resolve(__dirname, '../config/pex-tokenomics.json');
const APC_POLICY_PATH = path.resolve(__dirname, '../config/apc-policy-v1.json');
const WALLETS_TEMPLATE_PATH = path.resolve(__dirname, '../config/pex-allocation-wallets.example.json');
const PRODUCTION_WALLETS_PATH = path.resolve(__dirname, '../config/pex-allocation-wallets.json');
const EXPECTED_TOTAL_PERCENTAGE = 100;
const PLACEHOLDER_PREFIX = 'REPLACE_WITH_';

function fail(message) {
  console.error(`❌ ${message}`);
  process.exit(1);
}

function assert(condition, message) {
  if (!condition) fail(message);
}

function readJson(filePath, label) {
  assert(fs.existsSync(filePath), `${label} not found at ${filePath}`);

  const raw = fs.readFileSync(filePath, 'utf8');

  try {
    return JSON.parse(raw);
  } catch (error) {
    fail(`Invalid JSON in ${label}: ${error.message}`);
  }
}

function readConfig() {
  return readJson(CONFIG_PATH, 'Tokenomics config');
}

function toBigIntAmount(value, label) {
  assert(typeof value === 'string', `${label} must be a string amount to avoid floating point errors.`);
  assert(/^\d+$/.test(value), `${label} must be a positive integer string.`);
  return BigInt(value);
}

function flattenAllocations(config) {
  const map = new Map();

  for (const allocation of config.allocations) {
    map.set(allocation.key, allocation);

    if (Array.isArray(allocation.children)) {
      for (const child of allocation.children) {
        map.set(child.key, child);
      }
    }
  }

  return map;
}

function flattenWalletEntries(wallets) {
  const entries = [];

  function walk(node, pathParts) {
    if (!node || typeof node !== 'object') return;

    if (node.allocationKey) {
      entries.push({
        path: pathParts.join('.'),
        ...node,
      });
      return;
    }

    for (const [key, value] of Object.entries(node)) {
      walk(value, [...pathParts, key]);
    }
  }

  walk(wallets, ['wallets']);
  return entries;
}

function validateChildren(parent) {
  if (!parent.children || parent.children.length === 0) return;

  const childPercentageTotal = parent.children.reduce((sum, child) => sum + child.percentage, 0);
  const childAmountTotal = parent.children.reduce(
    (sum, child) => sum + toBigIntAmount(child.amount, `${child.key}.amount`),
    0n
  );

  assert(
    childPercentageTotal === parent.percentage,
    `${parent.key} child percentages total ${childPercentageTotal}%, expected ${parent.percentage}%.`
  );

  assert(
    childAmountTotal === toBigIntAmount(parent.amount, `${parent.key}.amount`),
    `${parent.key} child amounts total ${childAmountTotal.toString()}, expected ${parent.amount}.`
  );
}

function validateTokenomics(config) {
  assert(config.token, 'Missing token section.');
  assert(config.token.symbol === 'PEX', 'Token symbol must be PEX.');
  assert(config.token.name === 'Pera-X', 'Token name must be Pera-X.');
  assert(config.token.decimals === 6, 'Token decimals must be 6.');

  const totalSupply = toBigIntAmount(config.token.totalSupply, 'token.totalSupply');
  assert(totalSupply === 1000000000n, 'Total supply must be exactly 1,000,000,000 PEX.');
  assert(config.token.initialPriceUsd === '0.000012', 'Initial price must be $0.000012.');
  assert(config.token.initialValuationUsd === '12000', 'Initial valuation must be $12,000.');

  assert(Array.isArray(config.allocations), 'allocations must be an array.');
  assert(config.allocations.length > 0, 'allocations cannot be empty.');

  const totalPercentage = config.allocations.reduce((sum, allocation) => sum + allocation.percentage, 0);
  const totalAmount = config.allocations.reduce(
    (sum, allocation) => sum + toBigIntAmount(allocation.amount, `${allocation.key}.amount`),
    0n
  );

  assert(
    totalPercentage === EXPECTED_TOTAL_PERCENTAGE,
    `Allocation percentages total ${totalPercentage}%, expected ${EXPECTED_TOTAL_PERCENTAGE}%.`
  );

  assert(
    totalAmount === totalSupply,
    `Allocation amounts total ${totalAmount.toString()}, expected ${totalSupply.toString()}.`
  );

  for (const allocation of config.allocations) {
    assert(typeof allocation.key === 'string' && allocation.key.length > 0, 'Every allocation must have a key.');
    assert(typeof allocation.name === 'string' && allocation.name.length > 0, `${allocation.key} must have a name.`);
    assert(typeof allocation.percentage === 'number', `${allocation.key}.percentage must be a number.`);
    toBigIntAmount(allocation.amount, `${allocation.key}.amount`);
    validateChildren(allocation);
  }
}

function validateInitialLiquidity(config) {
  const liquidity = config.initialLiquidity;
  assert(liquidity, 'Missing initialLiquidity section.');
  assert(liquidity.policy === 'OPTION_B_FULL_38_PERCENT_LIQUIDITY_ALLOCATION', 'Liquidity policy must be Option B full 38% allocation.');
  assert(liquidity.dex === 'Meteora DLMM', 'Initial DEX must be Meteora DLMM.');
  assert(liquidity.pair === 'PEX/USDC', 'Initial pair must be PEX/USDC.');
  assert(liquidity.targetUsd === '4560', 'Initial liquidity target must be $4,560.');
  assert(liquidity.pexAmount === '380000000', 'Initial liquidity PEX amount must be 380,000,000 PEX.');
  assert(liquidity.quoteAmountUsd === '4560', 'Initial quote amount must be $4,560.');
  assert(liquidity.remainingLiquidityReserve === '0', 'Remaining liquidity reserve must be 0 PEX when full 38% is used initially.');
}

function validateMarketConditionalReleasePolicy(config) {
  const policy = config.marketConditionalReleasePolicy;
  assert(policy, 'Missing marketConditionalReleasePolicy section.');
  assert(policy.minimumLiquidityMultipleOfInitial === 3, 'Minimum liquidity gate must be 3x initial liquidity.');
  assert(policy.minimumLiquidityUsd === '13680', 'Minimum liquidity gate must be $13,680.');
  assert(policy.minimumNetBuyPressureBps === 5000, 'Minimum net buy pressure must be 5,000 bps.');
  assert(policy.minimumNetBuyPressurePercentage === 50, 'Minimum net buy pressure must be 50%.');
  assert(policy.dailyReleaseCapPercentageOfTotalSupply === 1, 'Daily release cap must be 1% of total supply.');
  assert(policy.dailyReleaseCapAmount === '10000000', 'Daily release cap amount must be 10,000,000 PEX.');
  assert(policy.monthlyReleaseCapPercentageOfTotalSupply === 15, 'Monthly release cap must be 15% of total supply.');
  assert(policy.monthlyReleaseCapAmount === '150000000', 'Monthly release cap amount must be 150,000,000 PEX.');
}

function canonicalJson(value) {
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

function validateWalletTemplate(config) {
  const template = readJson(WALLETS_TEMPLATE_PATH, 'Allocation wallet template');
  const allocations = flattenAllocations(config);
  const entries = flattenWalletEntries(template.wallets);

  assert(template.network === config.token.network, 'Wallet template network must match token network.');
  assert(template.tokenSymbol === config.token.symbol, 'Wallet template symbol must match token symbol.');
  assert(entries.length > 0, 'Wallet template must include wallet entries.');

  const productionWalletExists = fs.existsSync(PRODUCTION_WALLETS_PATH);
  assert(
    !productionWalletExists,
    'Production wallet config must not be committed at perax-contracts/config/pex-allocation-wallets.json. Use local or secret-managed config only.'
  );

  const seenKeys = new Set();

  for (const entry of entries) {
    const allocation = allocations.get(entry.allocationKey);

    assert(allocation, `${entry.path} references unknown allocation key ${entry.allocationKey}.`);
    assert(entry.percentage === allocation.percentage, `${entry.path} percentage must match ${entry.allocationKey}.`);
    assert(entry.amount === allocation.amount, `${entry.path} amount must match ${entry.allocationKey}.`);
    assert(typeof entry.address === 'string' && entry.address.length > 0, `${entry.path} must include an address placeholder.`);
    assert(entry.address.startsWith(PLACEHOLDER_PREFIX), `${entry.path} must use a placeholder address in the example template.`);
    assert(!seenKeys.has(entry.allocationKey), `Duplicate wallet entry for allocation key ${entry.allocationKey}.`);

    seenKeys.add(entry.allocationKey);
  }

  const requiredWalletKeys = [
    'liquidity_pool',
    'community_utility_rewards',
    'treasury',
    'ecosystem_marketing',
    'trading_company_operations',
    'development_team',
    'founder',
    'future_team_incentives',
    'team_emergency_reserve',
    'private_strategic_investors',
    'advisor_wallet_1',
    'advisor_wallet_2',
    'advisor_wallet_3',
  ];

  for (const key of requiredWalletKeys) {
    assert(seenKeys.has(key), `Wallet template missing required allocation key ${key}.`);
  }
}

function main() {
  const config = readConfig();

  validateTokenomics(config);
  validateInitialLiquidity(config);
  validateMarketConditionalReleasePolicy(config);
  validateAdaptivePriceControl(config);
  validateWalletTemplate(config);

  console.log('✅ PEX tokenomics config is valid.');
  console.log(`✅ Total supply: ${config.token.totalSupply} ${config.token.symbol}`);
  console.log(`✅ Initial price: $${config.token.initialPriceUsd}`);
  console.log(`✅ Allocations: ${EXPECTED_TOTAL_PERCENTAGE}%`);
  console.log('✅ Initial liquidity uses full 38% allocation on Meteora DLMM.');
  console.log('✅ Adaptive Price Control structure is valid.');
  console.log(`✅ APC numerical policy status: ${config.adaptivePriceControl.policyStatus}`);
  console.log('✅ Allocation wallet template is valid.');
}

main();
