const fs = require('fs');
const path = require('path');
const { execFileSync } = require('child_process');

const ENV_PATH = path.resolve(__dirname, '../.env');
const TOKENOMICS_PATH = path.resolve(__dirname, '../config/pex-tokenomics.json');
const EXECUTE = process.argv.includes('--execute');

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

function readJson(filePath, label) {
  if (!fs.existsSync(filePath)) throw new Error(`${label} not found at ${filePath}`);
  return JSON.parse(fs.readFileSync(filePath, 'utf8'));
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

function main() {
  const env = loadRuntimeEnv();
  const tokenomics = readJson(TOKENOMICS_PATH, 'PEX tokenomics config');

  console.log('==============================================');
  console.log('Pera-X Fixed Supply Mint Execution');
  console.log('==============================================');
  console.log(`Mode: ${EXECUTE ? 'EXECUTE' : 'DRY RUN / PLAN ONLY'}`);
  console.log('');
  console.log(`Token: ${tokenomics.token.name} (${tokenomics.token.symbol})`);
  console.log(`Decimals: ${tokenomics.token.decimals}`);
  console.log(`Fixed supply: ${Number(tokenomics.token.totalSupply).toLocaleString()} PEX`);
  console.log('');

  if (!EXECUTE) {
    console.log('This script will do the following when run with --execute:');
    console.log('1. Configure Solana CLI for the selected RPC.');
    console.log('2. Create the PEX mint with 6 decimals.');
    console.log('3. Create the deployer token account for the PEX mint.');
    console.log('4. Mint exactly 1,000,000,000 PEX once.');
    console.log('5. Print the mint address and source token account for .env update.');
    console.log('');
    console.log('Required env values for execution:');
    console.log('- SOLANA_RPC_URL');
    console.log('- SOLANA_KEYPAIR_PATH');
    console.log('');
    console.log('Run only after manual approval: node scripts/execute-fixed-supply-mint.js --execute');
    return;
  }

  requireEnv(env, ['SOLANA_RPC_URL', 'SOLANA_KEYPAIR_PATH']);

  run('solana', ['config', 'set', '--url', env.SOLANA_RPC_URL, '--keypair', env.SOLANA_KEYPAIR_PATH]);

  const createMintOutput = run('spl-token', ['create-token', '--decimals', String(tokenomics.token.decimals)]);
  const mintMatch = createMintOutput.match(/Creating token\s+([1-9A-HJ-NP-Za-km-z]+)/i);
  const mintAddress = mintMatch ? mintMatch[1] : createMintOutput.split(/\s+/).find((part) => /^[1-9A-HJ-NP-Za-km-z]{32,44}$/.test(part));

  if (!mintAddress) {
    throw new Error(`Could not parse mint address from spl-token output: ${createMintOutput}`);
  }

  const createAccountOutput = run('spl-token', ['create-account', mintAddress]);
  const accountMatch = createAccountOutput.match(/Creating account\s+([1-9A-HJ-NP-Za-km-z]+)/i);
  const sourceTokenAccount = accountMatch ? accountMatch[1] : createAccountOutput.split(/\s+/).find((part) => /^[1-9A-HJ-NP-Za-km-z]{32,44}$/.test(part));

  if (!sourceTokenAccount) {
    throw new Error(`Could not parse token account from spl-token output: ${createAccountOutput}`);
  }

  run('spl-token', ['mint', mintAddress, tokenomics.token.totalSupply, sourceTokenAccount]);

  console.log('');
  console.log('Fixed supply mint completed. Update .env with:');
  console.log(`PEX_MINT_ADDRESS=${mintAddress}`);
  console.log(`PEX_MINT_AUTHORITY=${env.SOLANA_KEYPAIR_PATH}`);
  console.log(`PEX_SOURCE_TOKEN_ACCOUNT=${sourceTokenAccount}`);
  console.log('');
  console.log('Do NOT revoke mint authority until allocations and liquidity actions are complete and supply is verified.');
}

main();
