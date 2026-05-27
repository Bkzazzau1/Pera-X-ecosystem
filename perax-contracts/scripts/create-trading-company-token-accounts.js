const fs = require('fs');
const path = require('path');
const { execFileSync } = require('child_process');

const ENV_PATH = path.resolve(__dirname, '../.env');
const DEPLOYMENT_RECORD_PATH = path.resolve(__dirname, '../config/deployment-record.devnet.public.json');
const EXECUTE = process.argv.includes('--execute');

const DEVNET_MINT = 'DnkAW3B1ckzW6eimgSBNPK3XTt83wMiZRETy8iF3gdsn';
const LOCKED_WALLET = 'A9kRqJ2hcPu5EWXoK4Ti8HUG5h4n75qEpwpAjhqFNt2f';
const REVENUE_WALLET = 'DaCFT5EZ6heLj2kSQjoAmz5gN7hHo3RdS5teuB2qupUX';

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

function loadRuntimeEnv() {
  return {
    ...readEnv(ENV_PATH),
    ...process.env,
  };
}

function missing(value) {
  return !value || value.startsWith('REPLACE_');
}

function run(command, args) {
  console.log(`$ ${command} ${args.join(' ')}`);
  if (!EXECUTE) return '';
  return execFileSync(command, args, { encoding: 'utf8', stdio: ['ignore', 'pipe', 'inherit'] }).trim();
}

function requireEnv(env, keys) {
  const missingKeys = keys.filter((key) => missing(env[key]));
  if (missingKeys.length > 0) {
    throw new Error(`Missing required env values for --execute: ${missingKeys.join(', ')}`);
  }
}

function parseAddress(output, label) {
  const candidates = output.split(/\s+/).filter((part) => /^[1-9A-HJ-NP-Za-km-z]{32,44}$/.test(part));
  if (candidates.length === 0) {
    throw new Error(`Could not parse ${label} address from output: ${output}`);
  }
  return candidates[candidates.length - 1];
}

function main() {
  const env = loadRuntimeEnv();
  const mint = env.PEX_MINT_ADDRESS && !missing(env.PEX_MINT_ADDRESS) ? env.PEX_MINT_ADDRESS : DEVNET_MINT;
  const lockedWallet = env.TRADING_COMPANY_LOCKED_WALLET || LOCKED_WALLET;
  const revenueWallet = env.TRADING_COMPANY_REVENUE_WALLET || REVENUE_WALLET;

  console.log('==============================================');
  console.log('Pera-X Trading Company Token Account Creation');
  console.log('==============================================');
  console.log(`Mode: ${EXECUTE ? 'EXECUTE' : 'DRY RUN / PLAN ONLY'}`);
  console.log('');
  console.log(`PEX mint: ${mint}`);
  console.log(`Locked/strategic owner wallet: ${lockedWallet}`);
  console.log(`Revenue owner wallet: ${revenueWallet}`);
  console.log('');

  if (!EXECUTE) {
    console.log('This script will create two PEX SPL token accounts when run with --execute:');
    console.log('1. Trading Company locked/strategic token account.');
    console.log('2. Trading Company revenue token account for PEX-for-Credits and burns.');
    console.log('');
    console.log('Required env values for execution:');
    console.log('- SOLANA_RPC_URL');
    console.log('- SOLANA_KEYPAIR_PATH');
    console.log('');
    console.log('Run only after mint is created: node scripts/create-trading-company-token-accounts.js --execute');
    return;
  }

  requireEnv(env, ['SOLANA_RPC_URL', 'SOLANA_KEYPAIR_PATH']);
  run('solana', ['config', 'set', '--url', env.SOLANA_RPC_URL, '--keypair', env.SOLANA_KEYPAIR_PATH]);

  const lockedOutput = run('spl-token', ['create-account', mint, '--owner', lockedWallet]);
  const lockedTokenAccount = parseAddress(lockedOutput, 'locked Trading Company token account');

  const revenueOutput = run('spl-token', ['create-account', mint, '--owner', revenueWallet]);
  const revenueTokenAccount = parseAddress(revenueOutput, 'revenue Trading Company token account');

  console.log('');
  console.log('Trading Company PEX token accounts created. Update .env with:');
  console.log(`TRADING_COMPANY_TOKEN_ACCOUNT=${lockedTokenAccount}`);
  console.log(`TRADING_COMPANY_REVENUE_TOKEN_ACCOUNT=${revenueTokenAccount}`);
  console.log('');
  console.log('Deployment record update values:');
  console.log(`lockedStrategicTokenAccount=${lockedTokenAccount}`);
  console.log(`revenueTokenAccount=${revenueTokenAccount}`);
  console.log('');
  console.log(`Deployment record path: ${DEPLOYMENT_RECORD_PATH}`);
}

main();
