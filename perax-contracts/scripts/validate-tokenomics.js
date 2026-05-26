const fs = require('fs');
const path = require('path');

const CONFIG_PATH = path.resolve(__dirname, '../config/pex-tokenomics.json');
const EXPECTED_TOTAL_PERCENTAGE = 100;

function fail(message) {
  console.error(`❌ ${message}`);
  process.exit(1);
}

function assert(condition, message) {
  if (!condition) fail(message);
}

function readConfig() {
  assert(fs.existsSync(CONFIG_PATH), `Tokenomics config not found at ${CONFIG_PATH}`);

  const raw = fs.readFileSync(CONFIG_PATH, 'utf8');

  try {
    return JSON.parse(raw);
  } catch (error) {
    fail(`Invalid JSON in tokenomics config: ${error.message}`);
  }
}

function toBigIntAmount(value, label) {
  assert(typeof value === 'string', `${label} must be a string amount to avoid floating point errors.`);
  assert(/^\d+$/.test(value), `${label} must be a positive integer string.`);
  return BigInt(value);
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
  assert(liquidity.dex === 'Meteora', 'Initial DEX must be Meteora.');
  assert(liquidity.pair === 'PEX/USDC', 'Initial pair must be PEX/USDC.');
  assert(liquidity.targetUsd === '3000', 'Initial liquidity target must be $3,000.');
  assert(liquidity.pexAmount === '250000000', 'Initial liquidity PEX amount must be 250,000,000 PEX.');
  assert(liquidity.remainingLiquidityReserve === '130000000', 'Remaining liquidity reserve must be 130,000,000 PEX.');
}

function validateUnlocking(config) {
  const unlocking = config.unlocking;
  assert(unlocking, 'Missing unlocking section.');
  assert(
    unlocking.model === 'reactive_market_conditional_unlocking_with_twap_protection',
    'Unlocking model must be reactive_market_conditional_unlocking_with_twap_protection.'
  );
  assert(unlocking.monitoringIntervalMinutes === 10, 'Monitoring interval must be 10 minutes.');
  assert(unlocking.twapConfirmationMinutesMin === 30, 'Minimum TWAP confirmation must be 30 minutes.');
  assert(unlocking.twapConfirmationMinutesMax === 60, 'Maximum TWAP confirmation must be 60 minutes.');
  assert(unlocking.cooldownHoursMin === 2, 'Minimum cooldown must be 2 hours.');
  assert(unlocking.cooldownHoursMax === 6, 'Maximum cooldown must be 6 hours.');
  assert(unlocking.maxDailyUnlockPercentageOfTotalSupply === 1, 'Daily unlock cap must be 1% of total supply.');
  assert(unlocking.maxDailyUnlockAmount === '10000000', 'Daily unlock cap amount must be 10,000,000 PEX.');
  assert(unlocking.requiresManualOrMultisigApproval === true, 'Manual or multisig approval must be enabled.');
  assert(unlocking.emergencyPauseEnabled === true, 'Emergency pause must be enabled.');
  assert(Array.isArray(unlocking.stages) && unlocking.stages.length >= 3, 'At least 3 unlocking stages are required.');
  assert(Array.isArray(unlocking.healthChecks) && unlocking.healthChecks.length > 0, 'Unlocking health checks are required.');
}

function main() {
  const config = readConfig();

  validateTokenomics(config);
  validateInitialLiquidity(config);
  validateUnlocking(config);

  console.log('✅ PEX tokenomics config is valid.');
  console.log(`✅ Total supply: ${config.token.totalSupply} ${config.token.symbol}`);
  console.log(`✅ Initial price: $${config.token.initialPriceUsd}`);
  console.log(`✅ Allocations: ${EXPECTED_TOTAL_PERCENTAGE}%`);
}

main();
