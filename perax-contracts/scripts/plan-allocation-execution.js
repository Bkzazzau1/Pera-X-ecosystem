const fs = require('fs');
const path = require('path');

const TOKENOMICS_PATH = path.resolve(__dirname, '../config/pex-tokenomics.json');
const WALLETS_TEMPLATE_PATH = path.resolve(__dirname, '../config/pex-allocation-wallets.example.json');
const DEVNET_WALLETS_PATH = path.resolve(__dirname, '../config/pex-allocation-wallets.devnet.json');
const WALLETS_PATH = path.resolve(__dirname, '../config/pex-allocation-wallets.json');
const ENV_PATH = path.resolve(__dirname, '../.env');

function readJson(filePath, label) {
  if (!fs.existsSync(filePath)) throw new Error(`${label} not found at ${filePath}`);
  return JSON.parse(fs.readFileSync(filePath, 'utf8'));
}

function readEnv(filePath) {
  if (!fs.existsSync(filePath)) return {};
  const raw = fs.readFileSync(filePath, 'utf8');
  const env = {};

  for (const line of raw.split(/\r?\n/)) {
    const trimmed = line.trim();
    if (!trimmed || trimmed.startsWith('#')) continue;
    const idx = trimmed.indexOf('=');
    if (idx === -1) continue;
    env[trimmed.slice(0, idx).trim()] = trimmed.slice(idx + 1).trim();
  }

  return env;
}

function getWalletSource() {
  if (fs.existsSync(WALLETS_PATH)) {
    return {
      label: 'production wallet config',
      path: WALLETS_PATH,
      data: readJson(WALLETS_PATH, 'Production allocation wallet config'),
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
      entries.push({ path: pathParts.join('.'), ...node });
      return;
    }

    for (const [key, value] of Object.entries(node)) walk(value, [...pathParts, key]);
  }

  walk(wallets, ['wallets']);
  return entries;
}

function formatNumber(value) {
  return new Intl.NumberFormat('en-US').format(Number(value));
}

function valueOrMissing(value) {
  if (!value || value.startsWith('REPLACE_')) return 'MISSING';
  return value;
}

function main() {
  const tokenomics = readJson(TOKENOMICS_PATH, 'PEX tokenomics config');
  const env = readEnv(ENV_PATH);
  const walletSource = getWalletSource();
  const walletEntries = flattenWalletEntries(walletSource.data.wallets);
  const totalSupply = BigInt(tokenomics.token.totalSupply);
  const allocationTotal = walletEntries.reduce((sum, entry) => sum + BigInt(entry.amount), 0n);
  const placeholderWallets = walletEntries.filter((entry) => valueOrMissing(entry.address) === 'MISSING');

  console.log('==============================================');
  console.log('Pera-X Allocation Execution Plan');
  console.log('==============================================');
  console.log('Mode: PLAN ONLY / NO TOKEN TRANSFER');
  console.log('');

  console.log('Mint and source status:');
  console.log('----------------------------------------------');
  console.log(`PEX mint: ${valueOrMissing(env.PEX_MINT_ADDRESS)}`);
  console.log(`Mint authority/source account: ${valueOrMissing(env.PEX_MINT_AUTHORITY)}`);
  console.log(`Wallet config source: ${walletSource.label}`);
  console.log(`Wallet config file: ${walletSource.path}`);
  console.log('');

  console.log('Allocation summary:');
  console.log('----------------------------------------------');
  console.log(`Expected supply: ${formatNumber(totalSupply.toString())} PEX`);
  console.log(`Allocation total: ${formatNumber(allocationTotal.toString())} PEX`);
  console.log(`Matches supply: ${allocationTotal === totalSupply ? 'YES' : 'NO'}`);
  console.log(`Placeholder wallets: ${placeholderWallets.length}`);
  console.log('');

  console.log('Transfer sequence:');
  console.log('----------------------------------------------');
  walletEntries.forEach((entry, index) => {
    console.log(`${index + 1}. ${entry.allocationKey}`);
    console.log(`   Percentage: ${entry.percentage}%`);
    console.log(`   Amount: ${formatNumber(entry.amount)} PEX`);
    console.log(`   Destination: ${valueOrMissing(entry.address)}`);
    console.log('');
  });

  console.log('Special liquidity action:');
  console.log('----------------------------------------------');
  console.log(`Liquidity policy: ${tokenomics.initialLiquidity.policy}`);
  console.log(`Liquidity venue: ${tokenomics.initialLiquidity.dex}`);
  console.log(`Pair: ${tokenomics.initialLiquidity.pair}`);
  console.log(`PEX side: ${formatNumber(tokenomics.initialLiquidity.pexAmount)} PEX`);
  console.log(`USDC side: $${formatNumber(tokenomics.initialLiquidity.quoteAmountUsd)}`);
  console.log(`Remaining liquidity reserve: ${formatNumber(tokenomics.initialLiquidity.remainingLiquidityReserve)} PEX`);
  console.log('The liquidity allocation must be used to create Meteora DLMM liquidity, not transferred as ordinary operating funds.');
  console.log('');

  const blockers = [];
  if (valueOrMissing(env.PEX_MINT_ADDRESS) === 'MISSING') blockers.push('PEX_MINT_ADDRESS missing');
  if (walletSource.isTemplate) blockers.push('using example wallet template');
  if (placeholderWallets.length > 0) blockers.push(`${placeholderWallets.length} placeholder wallet addresses remain`);
  if (allocationTotal !== totalSupply) blockers.push('allocation total does not match fixed supply');

  console.log('Readiness:');
  console.log('----------------------------------------------');
  if (blockers.length === 0) {
    console.log('Status: READY for devnet deployment planning. Real token transfers remain manual/secure deployment actions.');
  } else {
    console.log('Status: NOT READY for real allocation transfers.');
    blockers.forEach((blocker) => console.log(`- ${blocker}`));
  }
}

main();
