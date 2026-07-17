const fs = require('fs');
const path = require('path');

const TOKENOMICS_PATH = path.resolve(__dirname, '../config/pex-tokenomics.json');
const WALLETS_TEMPLATE_PATH = path.resolve(__dirname, '../config/pex-allocation-wallets.example.json');
const DEVNET_WALLETS_PATH = path.resolve(__dirname, '../config/pex-allocation-wallets.devnet.json');
const PRODUCTION_WALLETS_PATH = path.resolve(__dirname, '../config/pex-allocation-wallets.json');

function readJson(filePath, label) {
  if (!fs.existsSync(filePath)) {
    throw new Error(`${label} not found at ${filePath}`);
  }

  return JSON.parse(fs.readFileSync(filePath, 'utf8'));
}

function formatNumber(value) {
  return new Intl.NumberFormat('en-US').format(Number(value));
}

function getWalletSource() {
  if (fs.existsSync(PRODUCTION_WALLETS_PATH)) {
    return {
      label: 'production wallet config',
      path: PRODUCTION_WALLETS_PATH,
      data: readJson(PRODUCTION_WALLETS_PATH, 'Production allocation wallet config'),
      isTemplate: false,
    };
  }

  if (fs.existsSync(DEVNET_WALLETS_PATH)) {
    return {
      label: 'devnet wallet config',
      path: DEVNET_WALLETS_PATH,
      data: readJson(DEVNET_WALLETS_PATH, 'Devnet allocation wallet config'),
      isTemplate: false,
    };
  }

  return {
    label: 'example wallet template',
    path: WALLETS_TEMPLATE_PATH,
    data: readJson(WALLETS_TEMPLATE_PATH, 'Allocation wallet template'),
    isTemplate: true,
  };
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

function main() {
  const tokenomics = readJson(TOKENOMICS_PATH, 'PEX tokenomics config');
  const walletSource = getWalletSource();
  const walletEntries = flattenWalletEntries(walletSource.data.wallets);

  console.log('==============================================');
  console.log('Pera-X (PEX) Allocation Deployment Plan');
  console.log('==============================================');
  console.log(`Token: ${tokenomics.token.name} (${tokenomics.token.symbol})`);
  console.log(`Network: ${tokenomics.token.network}`);
  console.log(`Total supply: ${formatNumber(tokenomics.token.totalSupply)} ${tokenomics.token.symbol}`);
  console.log(`Decimals: ${tokenomics.token.decimals}`);
  console.log(`Initial price: $${tokenomics.token.initialPriceUsd}`);
  console.log(`Initial valuation: $${formatNumber(tokenomics.token.initialValuationUsd)}`);
  console.log('');
  console.log(`Wallet source: ${walletSource.label}`);
  console.log(`Wallet file: ${walletSource.path}`);

  if (walletSource.isTemplate) {
    console.log('Mode: DRY RUN / TEMPLATE ONLY');
    console.log('No real wallet addresses are loaded. No transfer should be executed from this template.');
  } else {
    console.log('Mode: PUBLIC WALLET CONFIG LOADED');
    console.log('Review all public addresses carefully before any deployment action.');
  }

  console.log('');
  console.log('Allocation plan:');
  console.log('----------------------------------------------');

  for (const entry of walletEntries) {
    console.log(`${entry.allocationKey}`);
    console.log(`  Percentage: ${entry.percentage}%`);
    console.log(`  Amount: ${formatNumber(entry.amount)} ${tokenomics.token.symbol}`);
    console.log(`  Address: ${entry.address}`);
    console.log('');
  }

  console.log('Initial liquidity guidance:');
  console.log('----------------------------------------------');
  console.log(`Policy: ${tokenomics.initialLiquidity.policy}`);
  console.log(`DEX: ${tokenomics.initialLiquidity.dex}`);
  console.log(`Pair: ${tokenomics.initialLiquidity.pair}`);
  console.log(`PEX side: ${formatNumber(tokenomics.initialLiquidity.pexAmount)} PEX`);
  console.log(`Quote side: $${formatNumber(tokenomics.initialLiquidity.quoteAmountUsd)}`);
  console.log(`Remaining liquidity reserve: ${formatNumber(tokenomics.initialLiquidity.remainingLiquidityReserve)} PEX`);
  console.log('');

  console.log('Adaptive Price Control policy summary:');
  console.log('----------------------------------------------');
  const apc = tokenomics.adaptivePriceControl;
  console.log(`Policy status: ${apc.policyStatus}`);
  console.log(`First activation: $${apc.firstActivationPriceUsd} (${apc.firstActivationMultiplier}x launch)`);
  console.log(`Fixed multiplication after first activation: ${apc.fixedMultiplicationAfterFirstActivation}`);
  console.log(`Observation authority: ${apc.authorityPolicy.observationAuthority}`);
  console.log(`Human approval after contract gates pass: ${apc.authorityPolicy.requiresManualOrMultisigApproval}`);
  console.log(`Daily cap: ${apc.releaseCaps.dailyCapAmount} PEX`);
  console.log(`Monthly cap: ${apc.releaseCaps.monthlyCapAmount} PEX`);
  console.log(`Pending numerical policies: ${apc.unresolvedNumericalPolicies.length}`);
  console.log('');

  console.log('Next safe step: run npm run validate:tokenomics before any real deployment.');
}

main();
