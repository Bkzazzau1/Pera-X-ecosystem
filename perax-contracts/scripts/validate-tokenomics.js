const fs = require('fs');
const path = require('path');

const CONFIG_PATH = path.resolve(__dirname, '../config/pex-tokenomics.json');
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

function validateAdaptivePriceControl(config) {
  const apc = config.adaptivePriceControl;
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

  const bands = apc.bandPolicy;
  assert(bands.sequentialActivation === true, 'Bands must activate sequentially.');
  assert(bands.multiBandPerFreshObservation === true, 'One fresh pump observation must be able to activate several sequential bands.');
  assert(bands.immutableBandRecords === true, 'Band records must be immutable PDAs.');
  assert(bands.usedBandCapacityNeverRestores === true, 'Used band capacity must never restore.');
  if (apc.policyStatus === 'approved') {
    assert(Number.isInteger(bands.minimumIntervalBps) && bands.minimumIntervalBps > 0, 'Approved minimum band interval is invalid.');
    assert(Number.isInteger(bands.maximumIntervalBps) && bands.maximumIntervalBps > bands.minimumIntervalBps, 'Approved maximum band interval is invalid.');
    assert(Array.isArray(bands.cascadeReductionBps) && bands.cascadeReductionBps.length > 0, 'Approved cascade policy is required.');
    let previous = 10001;
    for (const value of bands.cascadeReductionBps) {
      assert(Number.isInteger(value) && value > 0 && value <= 10000 && value <= previous, 'Cascade reductions must be positive and monotonic.');
      previous = value;
    }
  } else {
    assert(apc.policyStatus === 'pending_formal_numerical_approval', 'Unknown APC policy status.');
    assert(bands.minimumIntervalBps === null && bands.maximumIntervalBps === null, 'Pending band interval values must remain null.');
    assert(bands.riskTierThresholds === null && bands.cascadeReductionBps === null, 'Pending risk and cascade values must remain null.');
  }

  const observations = apc.observationPolicy;
  assert(observations.permanentObservationPda === true, 'Permanent observation PDAs are required.');
  assert(observations.strictlyIncreasingSequence === true, 'Observation sequence must be strictly increasing.');
  assert(observations.freshObservationPerRelease === true, 'Every release must use a fresh observation.');
  assert(observations.trustedClockForWindows === true, 'On-chain windows must use the Solana clock.');
  assert(observations.approvedPoolOnly === true, 'Only the approved market pool may be observed.');

  const caps = apc.releaseCaps;
  assert(caps.perBandHardCapRequired === true, 'Every band must have a hard cap.');
  assert(caps.unconfirmedExposureCapRequired === true, 'Unconfirmed release exposure must have a hard cap.');
  assert(caps.dailyCapAmount === config.marketConditionalReleasePolicy.dailyReleaseCapAmount, 'APC daily cap must match the global daily cap.');
  assert(caps.monthlyCapAmount === config.marketConditionalReleasePolicy.monthlyReleaseCapAmount, 'APC monthly cap must match the global monthly cap.');
  if (caps.hourlyCapAmount !== null) {
    assert(toBigIntAmount(caps.hourlyCapAmount, 'adaptivePriceControl.releaseCaps.hourlyCapAmount') <= toBigIntAmount(caps.dailyCapAmount, 'adaptivePriceControl.releaseCaps.dailyCapAmount'), 'Hourly cap cannot exceed daily cap.');
  }
  if (caps.pumpWindowCapAmount !== null) {
    assert(toBigIntAmount(caps.pumpWindowCapAmount, 'adaptivePriceControl.releaseCaps.pumpWindowCapAmount') <= toBigIntAmount(caps.monthlyCapAmount, 'adaptivePriceControl.releaseCaps.monthlyCapAmount'), 'Pump-window cap cannot exceed monthly cap.');
  }

  const counterweight = apc.counterweightPolicy;
  assert(counterweight.realSplTransferRequired === true && counterweight.pdaControlledVault === true, 'Counterweight credit must come from real SPL custody.');
  assert(counterweight.missingCoverageStopsLaterReleases === true, 'Missing counterweight coverage must stop later releases.');
  if (counterweight.proceedsAllocationBps !== null) {
    const total = Object.values(counterweight.proceedsAllocationBps).reduce((sum, value) => sum + value, 0);
    assert(total === 10000, 'Counterweight proceeds percentages must total 10,000 bps.');
  }

  assert(apc.burnDeferralPolicy.enabledDuringPumpControl === true, 'Burn deferral must be enabled during pump control.');
  assert(apc.burnDeferralPolicy.pexEscrowRequired === true, 'Deferred burn PEX must be escrowed.');
  assert(apc.recoveryPolicy.atomicSwapRequired === true, 'Recovery must use an atomic swap.');
  assert(apc.recoveryPolicy.lockedRecoveryVault === true, 'Recovered PEX must enter a locked vault.');
  assert(apc.recoveryPolicy.hardSpendingCapRequired === true, 'Recovery spending must have a hard cap.');
  assert(apc.authorityPolicy.requiresManualOrMultisigApproval === false, 'Manual or multisig release approval must remain disabled.');
  assert(apc.authorityPolicy.routineReleaseApproval === 'none', 'Routine APC release must not require human approval.');
  assert(Array.isArray(apc.unresolvedNumericalPolicies) && apc.unresolvedNumericalPolicies.length === 10, 'All ten unresolved numerical policies must be listed.');
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
