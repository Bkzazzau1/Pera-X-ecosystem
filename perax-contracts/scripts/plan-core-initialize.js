const fs = require('fs');
const path = require('path');

const ENV_PATH = path.resolve(__dirname, '../.env');
const TOKENOMICS_PATH = path.resolve(__dirname, '../config/pex-tokenomics.json');

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

function readJson(filePath, label) {
  if (!fs.existsSync(filePath)) throw new Error(`${label} not found at ${filePath}`);
  return JSON.parse(fs.readFileSync(filePath, 'utf8'));
}

function valueOrMissing(value) {
  if (!value || value.startsWith('REPLACE_')) return 'MISSING';
  return value;
}

function isMissing(value) {
  return valueOrMissing(value) === 'MISSING';
}

function main() {
  const env = readEnv(ENV_PATH);
  const tokenomics = readJson(TOKENOMICS_PATH, 'PEX tokenomics config');

  const required = [
    ['PERAX_CORE_PROGRAM_ID', env.PERAX_CORE_PROGRAM_ID],
    ['PEX_MINT_ADDRESS', env.PEX_MINT_ADDRESS],
    ['TRADING_COMPANY_TOKEN_ACCOUNT', env.TRADING_COMPANY_TOKEN_ACCOUNT],
    ['TRADING_COMPANY_REVENUE_TOKEN_ACCOUNT', env.TRADING_COMPANY_REVENUE_TOKEN_ACCOUNT],
  ];

  console.log('==============================================');
  console.log('Pera-X Core Program Initialization Plan');
  console.log('==============================================');
  console.log('Mode: PLAN ONLY / NO ON-CHAIN INITIALIZATION');
  console.log('');

  console.log('Program inputs:');
  console.log('----------------------------------------------');
  console.log(`Program ID: ${valueOrMissing(env.PERAX_CORE_PROGRAM_ID)}`);
  console.log(`PEX mint: ${valueOrMissing(env.PEX_MINT_ADDRESS)}`);
  console.log(`Locked Trading Company token account: ${valueOrMissing(env.TRADING_COMPANY_TOKEN_ACCOUNT)}`);
  console.log(`Revenue Trading Company token account: ${valueOrMissing(env.TRADING_COMPANY_REVENUE_TOKEN_ACCOUNT)}`);
  console.log(`Max payment amount: ${env.MAX_PAYMENT_AMOUNT || '0'}`);
  console.log('');

  console.log('Business rules confirmed by contract:');
  console.log('----------------------------------------------');
  console.log('1. Locked and revenue token accounts must be different.');
  console.log('2. User PEX payments route to the revenue token account.');
  console.log('3. Burns execute from the revenue token account.');
  console.log('4. Locked/strategic Trading Company account is not used for daily PEX-for-Credits revenue.');
  console.log('');

  console.log('Token policy:');
  console.log('----------------------------------------------');
  console.log(`Token: ${tokenomics.token.name} (${tokenomics.token.symbol})`);
  console.log(`Total supply: ${Number(tokenomics.token.totalSupply).toLocaleString()} PEX`);
  console.log(`Supply model: fixed`);
  console.log(`Initial price: $${tokenomics.token.initialPriceUsd}`);
  console.log(`Liquidity venue: ${tokenomics.initialLiquidity.dex}`);
  console.log('');

  const missing = required.filter(([, value]) => isMissing(value)).map(([key]) => key);

  if (missing.length > 0) {
    console.log('Missing required values:');
    console.log('----------------------------------------------');
    for (const key of missing) console.log(`- ${key}`);
    console.log('');
    console.log('Status: NOT READY for real initialization.');
  } else if (env.TRADING_COMPANY_TOKEN_ACCOUNT === env.TRADING_COMPANY_REVENUE_TOKEN_ACCOUNT) {
    console.log('Status: NOT READY - locked and revenue token accounts must not be the same.');
  } else {
    console.log('Status: READY for real initialization script after manual approval.');
  }

  console.log('');
  console.log('Future initialize params:');
  console.log('----------------------------------------------');
  console.log('InitializeParams {');
  console.log(`  token_mint: ${valueOrMissing(env.PEX_MINT_ADDRESS)},`);
  console.log(`  trading_company_token_account: ${valueOrMissing(env.TRADING_COMPANY_TOKEN_ACCOUNT)},`);
  console.log(`  trading_company_revenue_token_account: ${valueOrMissing(env.TRADING_COMPANY_REVENUE_TOKEN_ACCOUNT)},`);
  console.log(`  max_payment_amount: ${env.MAX_PAYMENT_AMOUNT || '0'}`);
  console.log('}');
}

main();
