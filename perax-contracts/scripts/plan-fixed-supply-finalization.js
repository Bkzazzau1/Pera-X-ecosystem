const fs = require('fs');
const path = require('path');

const TOKENOMICS_PATH = path.resolve(__dirname, '../config/pex-tokenomics.json');
const WALLETS_TEMPLATE_PATH = path.resolve(__dirname, '../config/pex-allocation-wallets.example.json');
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

function valueOrMissing(value) {
  if (!value || value.startsWith('REPLACE_')) return 'MISSING';
  return value;
}

function flattenWalletEntries(wallets) {
  const entries = [];

  function walk(node, pathParts) {
    if (!node || typeof node !== 'object') return;

    if (node.allocationKey) {
      entries.push({ path: pathParts.join('.'), ...node });
      return;
    }

    for (const [key, value] of Object.entries(node)) {
      walk(value, [...pathParts, key]);
    }
  }

  walk(wallets, ['wallets']);
  return entries;
}

function formatNumber(value) {
  return new Intl.NumberFormat('en-US').format(Number(value));
}

function main() {
  const tokenomics = readJson(TOKENOMICS_PATH, 'PEX tokenomics config');
  const env = readEnv(ENV_PATH);
  const walletFile = fs.existsSync(WALLETS_PATH) ? WALLETS_PATH : WALLETS_TEMPLATE_PATH;
  const walletSource = readJson(walletFile, 'PEX allocation wallet config');
  const walletEntries = flattenWalletEntries(walletSource.wallets);

  const totalSupply = BigInt(tokenomics.token.totalSupply);
  const allocationTotal = walletEntries.reduce((sum, entry) => sum + BigInt(entry.amount), 0n);
  const usingTemplate = walletFile === WALLETS_TEMPLATE_PATH;
  const placeholderWallets = walletEntries.filter((entry) => valueOrMissing(entry.address) === 'MISSING');

  console.log('==============================================');
  console.log('Pera-X Fixed-Supply Finalization Plan');
  console.log('==============================================');
  console.log('Mode: PLAN ONLY / NO MINT OR AUTHORITY CHANGE');
  console.log('');

  console.log('Fixed supply policy:');
  console.log('----------------------------------------------');
  console.log(`Token: ${tokenomics.token.name} (${tokenomics.token.symbol})`);
  console.log(`Total supply: ${formatNumber(tokenomics.token.totalSupply)} PEX`);
  console.log(`Decimals: ${tokenomics.token.decimals}`);
  console.log(`Initial price: $${tokenomics.token.initialPriceUsd}`);
  console.log('Rule: Full fixed supply is minted once, then mint authority is revoked.');
  console.log('Rule: No future PEX minting after revocation.');
  console.log('');

  console.log('Authority status:');
  console.log('----------------------------------------------');
  console.log(`PEX_MINT_ADDRESS: ${valueOrMissing(env.PEX_MINT_ADDRESS)}`);
  console.log(`PEX_MINT_AUTHORITY: ${valueOrMissing(env.PEX_MINT_AUTHORITY)}`);
  console.log(`PEX_FREEZE_AUTHORITY: ${valueOrMissing(env.PEX_FREEZE_AUTHORITY)}`);
  console.log('');

  console.log('Allocation readiness:');
  console.log('----------------------------------------------');
  console.log(`Wallet config source: ${usingTemplate ? 'example template' : 'local production config'}`);
  console.log(`Allocation total: ${formatNumber(allocationTotal.toString())} PEX`);
  console.log(`Expected total supply: ${formatNumber(totalSupply.toString())} PEX`);
  console.log(`Allocation total matches supply: ${allocationTotal === totalSupply ? 'YES' : 'NO'}`);
  console.log(`Placeholder wallets remaining: ${placeholderWallets.length}`);
  console.log('');

  console.log('Finalization sequence:');
  console.log('----------------------------------------------');
  console.log('1. Create PEX mint with 6 decimals.');
  console.log('2. Mint exactly 1,000,000,000 PEX once.');
  console.log('3. Verify on-chain mint supply equals 1,000,000,000 PEX.');
  console.log('4. Create/verify all allocation token accounts.');
  console.log('5. Transfer allocations according to pex-tokenomics.json.');
  console.log('6. Create Meteora DLMM liquidity using 380,000,000 PEX + $4,560 USDC.');
  console.log('7. Initialize core program with locked and revenue Trading Company token accounts.');
  console.log('8. Revoke mint authority permanently.');
  console.log('9. Confirm mint authority is None/null on-chain.');
  console.log('10. Store final public deployment records.');
  console.log('');

  console.log('Safety gates before real execution:');
  console.log('----------------------------------------------');
  const blockers = [];

  if (valueOrMissing(env.PEX_MINT_ADDRESS) === 'MISSING') blockers.push('PEX_MINT_ADDRESS missing');
  if (valueOrMissing(env.PEX_MINT_AUTHORITY) === 'MISSING') blockers.push('PEX_MINT_AUTHORITY missing');
  if (allocationTotal !== totalSupply) blockers.push('allocation total does not match total supply');
  if (usingTemplate) blockers.push('using example wallet template, not local production wallet config');
  if (placeholderWallets.length > 0) blockers.push(`${placeholderWallets.length} placeholder allocation wallets remain`);

  if (blockers.length === 0) {
    console.log('Status: READY for manual review before real fixed-supply finalization.');
  } else {
    console.log('Status: NOT READY for real fixed-supply finalization.');
    for (const blocker of blockers) console.log(`- ${blocker}`);
  }
}

main();
