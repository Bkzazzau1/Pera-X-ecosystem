const fs = require('fs');
const path = require('path');

const TOKENOMICS_PATH = path.resolve(__dirname, '../config/pex-tokenomics.json');
const ENV_PATH = path.resolve(__dirname, '../.env');

function readJson(filePath, label) {
  if (!fs.existsSync(filePath)) {
    throw new Error(`${label} not found at ${filePath}`);
  }

  return JSON.parse(fs.readFileSync(filePath, 'utf8'));
}

function readEnv(filePath) {
  if (!fs.existsSync(filePath)) {
    return {};
  }

  const raw = fs.readFileSync(filePath, 'utf8');
  const env = {};

  for (const line of raw.split(/\r?\n/)) {
    const trimmed = line.trim();

    if (!trimmed || trimmed.startsWith('#')) continue;

    const separatorIndex = trimmed.indexOf('=');
    if (separatorIndex === -1) continue;

    const key = trimmed.slice(0, separatorIndex).trim();
    const value = trimmed.slice(separatorIndex + 1).trim();
    env[key] = value;
  }

  return env;
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

  console.log('==============================================');
  console.log('Pera-X (PEX) Mint and Core Initialization Plan');
  console.log('==============================================');
  console.log('Mode: PLAN ONLY / NO TOKEN WILL BE CREATED');
  console.log('');

  console.log('Token configuration:');
  console.log('----------------------------------------------');
  console.log(`Name: ${tokenomics.token.name}`);
  console.log(`Symbol: ${tokenomics.token.symbol}`);
  console.log(`Network: ${tokenomics.token.network}`);
  console.log(`Decimals: ${tokenomics.token.decimals}`);
  console.log(`Total supply: ${formatNumber(tokenomics.token.totalSupply)} ${tokenomics.token.symbol}`);
  console.log(`Supply model: fixed supply`);
  console.log(`Initial price: $${tokenomics.token.initialPriceUsd}`);
  console.log(`Initial valuation: $${formatNumber(tokenomics.token.initialValuationUsd)}`);
  console.log('');

  console.log('Environment status:');
  console.log('----------------------------------------------');
  console.log(`.env file: ${fs.existsSync(ENV_PATH) ? 'FOUND' : 'NOT FOUND - copy .env.example to .env locally'}`);
  console.log(`SOLANA_CLUSTER: ${valueOrMissing(env.SOLANA_CLUSTER)}`);
  console.log(`SOLANA_RPC_URL: ${valueOrMissing(env.SOLANA_RPC_URL)}`);
  console.log(`SOLANA_KEYPAIR_PATH: ${valueOrMissing(env.SOLANA_KEYPAIR_PATH)}`);
  console.log(`PERAX_CORE_PROGRAM_ID: ${valueOrMissing(env.PERAX_CORE_PROGRAM_ID)}`);
  console.log(`PEX_MINT_ADDRESS: ${valueOrMissing(env.PEX_MINT_ADDRESS)}`);
  console.log(`PEX_MINT_AUTHORITY: ${valueOrMissing(env.PEX_MINT_AUTHORITY)}`);
  console.log(`PEX_FREEZE_AUTHORITY: ${valueOrMissing(env.PEX_FREEZE_AUTHORITY)}`);
  console.log(`TRADING_COMPANY_TOKEN_ACCOUNT: ${valueOrMissing(env.TRADING_COMPANY_TOKEN_ACCOUNT)}`);
  console.log(`TRADING_COMPANY_REVENUE_TOKEN_ACCOUNT: ${valueOrMissing(env.TRADING_COMPANY_REVENUE_TOKEN_ACCOUNT)}`);
  console.log('');

  console.log('Trading Company wallet model:');
  console.log('----------------------------------------------');
  console.log('1. TRADING_COMPANY_TOKEN_ACCOUNT = locked/strategic account.');
  console.log('2. TRADING_COMPANY_REVENUE_TOKEN_ACCOUNT = second/revenue account for PEX-for-Credits payments.');
  console.log('3. PEX payments and burns must use the revenue token account, not the locked account.');
  console.log('');

  console.log('Future real mint and initialization requirements:');
  console.log('----------------------------------------------');
  console.log('1. Solana CLI installed and configured.');
  console.log('2. SPL Token CLI installed.');
  console.log('3. Deployer wallet funded with SOL for rent and transaction fees.');
  console.log('4. Mint authority public key confirmed.');
  console.log('5. Freeze authority decision confirmed.');
  console.log('6. Token metadata plan confirmed.');
  console.log('7. Tokenomics validation passed.');
  console.log('8. Locked Trading Company token account created.');
  console.log('9. Revenue Trading Company token account created.');
  console.log('10. Core program initialized with both Trading Company token accounts.');
  console.log('11. Mint authority revoked after fixed supply is fully minted and verified.');
  console.log('');

  console.log('Safety note:');
  console.log('----------------------------------------------');
  console.log('This script intentionally does not create a mint, mint supply, initialize the program, or move tokens.');
  console.log('Execution scripts should be added only after authorities and wallet addresses are approved.');
}

main();
