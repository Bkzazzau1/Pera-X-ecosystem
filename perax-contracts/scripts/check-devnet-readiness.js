const fs = require('fs');
const path = require('path');

const TOKENOMICS_PATH = path.resolve(__dirname, '../config/pex-tokenomics.json');
const DEVNET_WALLETS_PATH = path.resolve(__dirname, '../config/pex-allocation-wallets.devnet.json');
const ENV_PATH = path.resolve(__dirname, '../.env');

const ROLE_WALLETS = {
  tradingCompanyRevenueWallet: 'DaCFT5EZ6heLj2kSQjoAmz5gN7hHo3RdS5teuB2qupUX',
  safetyAdminWallet: '2FuGxUt1EaewriZ9QpSkqz8Lbj2M69q2F4vnr1NJHPHX',
  oracleBotWallet: '4hx4YvYdyQMziqxqTv5gjRbLcXZFwX1kaL9vdPBYdU9D',
  deployerWallet: '35LjkfJhyxP5GeCRDMHrMWHdzHdrznrNnq1aJ5fr1ZPQ',
};

function readJson(filePath, label) {
  if (!fs.existsSync(filePath)) throw new Error(`${label} not found at ${filePath}`);
  return JSON.parse(fs.readFileSync(filePath, 'utf8'));
}

function readEnv(filePath) {
  if (!fs.existsSync(filePath)) return {};
  const env = {};
  for (const line of fs.readFileSync(filePath, 'utf8').split(/\r?\n/)) {
    const trimmed = line.trim();
    if (!trimmed || trimmed.startsWith('#')) continue;
    const idx = trimmed.indexOf('=');
    if (idx === -1) continue;
    env[trimmed.slice(0, idx).trim()] = trimmed.slice(idx + 1).trim();
  }
  return env;
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

function isMissing(value) {
  return !value || value.startsWith('REPLACE_');
}

function main() {
  const tokenomics = readJson(TOKENOMICS_PATH, 'PEX tokenomics config');
  const wallets = readJson(DEVNET_WALLETS_PATH, 'PEX devnet wallet config');
  const env = readEnv(ENV_PATH);
  const entries = flattenWalletEntries(wallets.wallets);
  const totalSupply = BigInt(tokenomics.token.totalSupply);
  const allocationTotal = entries.reduce((sum, entry) => sum + BigInt(entry.amount), 0n);
  const uniqueAddresses = new Set(entries.map((entry) => entry.address));
  const missingAddresses = entries.filter((entry) => isMissing(entry.address));

  console.log('==============================================');
  console.log('Pera-X Devnet Readiness Check');
  console.log('==============================================');
  console.log('');

  console.log('Token policy:');
  console.log('----------------------------------------------');
  console.log(`Token: ${tokenomics.token.name} (${tokenomics.token.symbol})`);
  console.log(`Cluster: ${wallets.cluster}`);
  console.log(`Total supply: ${formatNumber(tokenomics.token.totalSupply)} PEX`);
  console.log(`Initial price: $${tokenomics.token.initialPriceUsd}`);
  console.log('');

  console.log('Option B liquidity policy:');
  console.log('----------------------------------------------');
  console.log(`Policy: ${tokenomics.initialLiquidity.policy}`);
  console.log(`DEX: ${tokenomics.initialLiquidity.dex}`);
  console.log(`Pair: ${tokenomics.initialLiquidity.pair}`);
  console.log(`PEX side: ${formatNumber(tokenomics.initialLiquidity.pexAmount)} PEX`);
  console.log(`USDC side: $${formatNumber(tokenomics.initialLiquidity.quoteAmountUsd)}`);
  console.log(`Remaining liquidity reserve: ${formatNumber(tokenomics.initialLiquidity.remainingLiquidityReserve)} PEX`);
  console.log('');

  console.log('Allocation wallets:');
  console.log('----------------------------------------------');
  console.log(`Wallet entries: ${entries.length}`);
  console.log(`Unique allocation addresses: ${uniqueAddresses.size}`);
  console.log(`Missing/placeholder allocation addresses: ${missingAddresses.length}`);
  console.log(`Allocation total: ${formatNumber(allocationTotal.toString())} PEX`);
  console.log(`Matches total supply: ${allocationTotal === totalSupply ? 'YES' : 'NO'}`);
  console.log('');

  console.log('Extra role wallets:');
  console.log('----------------------------------------------');
  for (const [role, address] of Object.entries(ROLE_WALLETS)) {
    console.log(`${role}: ${address}`);
  }
  console.log('');

  console.log('Adaptive Price Control policy:');
  console.log('----------------------------------------------');
  const apc = tokenomics.adaptivePriceControl;
  console.log(`Policy status: ${apc.policyStatus}`);
  console.log(`Observation authority: ${apc.authorityPolicy.observationAuthority}`);
  console.log(`Human approval when contract gates pass: ${apc.authorityPolicy.requiresManualOrMultisigApproval}`);
  console.log(`First activation: $${apc.firstActivationPriceUsd}`);
  console.log(`Daily cap: ${formatNumber(apc.releaseCaps.dailyCapAmount)} PEX`);
  console.log(`Monthly cap: ${formatNumber(apc.releaseCaps.monthlyCapAmount)} PEX`);
  console.log('');

  const blockers = [];
  if (wallets.cluster !== 'devnet') blockers.push('devnet wallet config must have cluster=devnet');
  if (entries.length !== 13) blockers.push('expected 13 allocation wallet entries');
  if (uniqueAddresses.size !== entries.length) blockers.push('allocation wallet addresses must be unique');
  if (missingAddresses.length > 0) blockers.push('allocation wallet placeholders remain');
  if (allocationTotal !== totalSupply) blockers.push('allocation total does not match total supply');
  if (tokenomics.initialLiquidity.policy !== 'OPTION_B_FULL_38_PERCENT_LIQUIDITY_ALLOCATION') blockers.push('Option B liquidity policy is not locked');
  if (tokenomics.initialLiquidity.pexAmount !== '380000000') blockers.push('liquidity PEX side must be 380,000,000 PEX');
  if (tokenomics.initialLiquidity.quoteAmountUsd !== '4560') blockers.push('liquidity quote side must be $4,560');
  if (apc.authorityPolicy.requiresManualOrMultisigApproval !== false) blockers.push('manual/multisig approval must remain disabled');
  if (apc.fixedMultiplicationAfterFirstActivation !== false) blockers.push('fixed multiplication must be disabled after first activation');
  if (apc.policyStatus !== 'approved') blockers.push('APC numerical policy is not formally approved');

  const missingOnchain = [];
  if (isMissing(env.PEX_MINT_ADDRESS)) missingOnchain.push('PEX_MINT_ADDRESS');
  if (isMissing(env.PERAX_CORE_PROGRAM_ID)) missingOnchain.push('PERAX_CORE_PROGRAM_ID');
  if (isMissing(env.TRADING_COMPANY_TOKEN_ACCOUNT)) missingOnchain.push('TRADING_COMPANY_TOKEN_ACCOUNT');
  if (isMissing(env.TRADING_COMPANY_REVENUE_TOKEN_ACCOUNT)) missingOnchain.push('TRADING_COMPANY_REVENUE_TOKEN_ACCOUNT');

  console.log('Readiness:');
  console.log('----------------------------------------------');
  if (blockers.length === 0) {
    console.log('Status: DEVNET CONFIG READY.');
  } else {
    console.log('Status: BLOCKED.');
    blockers.forEach((blocker) => console.log(`- ${blocker}`));
  }

  if (missingOnchain.length > 0) {
    console.log('');
    console.log('On-chain values still expected after devnet deployment:');
    missingOnchain.forEach((item) => console.log(`- ${item}`));
  }
}

main();
