const { execFileSync } = require('child_process');
const fs = require('fs');
const path = require('path');

const EXECUTE = process.argv.includes('--execute');
const WALLETS_PATH = path.resolve(__dirname, '../config/pex-allocation-wallets.devnet.json');
const DEPLOYMENT_RECORD_PATH = path.resolve(__dirname, '../config/deployment-record.devnet.public.json');
const TOKEN_DECIMALS = 6n;
const TOKEN_SCALE = 10n ** TOKEN_DECIMALS;

function readJson(filePath, label) {
  if (!fs.existsSync(filePath)) throw new Error(`${label} not found at ${filePath}`);
  return JSON.parse(fs.readFileSync(filePath, 'utf8'));
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

function run(command, args, options = {}) {
  console.log(`$ ${command} ${args.join(' ')}`);
  if (!EXECUTE) return '';
  try {
    return execFileSync(command, args, { encoding: 'utf8', stdio: ['ignore', 'pipe', 'pipe'] }).trim();
  } catch (error) {
    const stdout = error.stdout ? String(error.stdout) : '';
    const stderr = error.stderr ? String(error.stderr) : '';
    const combined = `${stdout}\n${stderr}`.trim();
    if (options.allowAccountAlreadyExists && combined.includes('Account already exists')) {
      console.log(combined);
      return combined;
    }
    if (combined) console.error(combined);
    throw error;
  }
}

function parseAta(output) {
  const accountExistsMatch = output.match(/Account already exists:\s*([1-9A-HJ-NP-Za-km-z]{32,44})/);
  if (accountExistsMatch) return accountExistsMatch[1];

  const creatingAccountMatch = output.match(/Creating account\s+([1-9A-HJ-NP-Za-km-z]{32,44})/);
  if (creatingAccountMatch) return creatingAccountMatch[1];

  const candidates = output.split(/\s+/).filter((part) => /^[1-9A-HJ-NP-Za-km-z]{32,44}$/.test(part));
  if (candidates.length === 0) throw new Error(`Could not parse token account from output: ${output}`);
  return candidates[candidates.length - 1];
}

function parseUiAmountToBaseUnits(value) {
  const cleaned = String(value || '0').trim();
  if (!cleaned) return 0n;
  const [wholePart, fractionPart = ''] = cleaned.split('.');
  const whole = BigInt(wholePart || '0') * TOKEN_SCALE;
  const paddedFraction = (fractionPart + '000000').slice(0, 6);
  return whole + BigInt(paddedFraction || '0');
}

function baseUnitsToUiAmount(baseUnits) {
  const amount = BigInt(baseUnits);
  const whole = amount / TOKEN_SCALE;
  const fraction = amount % TOKEN_SCALE;
  if (fraction === 0n) return whole.toString();
  return `${whole.toString()}.${fraction.toString().padStart(6, '0').replace(/0+$/, '')}`;
}

function getTokenBalanceBaseUnits(tokenAccount) {
  const output = run('spl-token', ['balance', '--address', tokenAccount]);
  return parseUiAmountToBaseUnits(output);
}

function main() {
  const wallets = readJson(WALLETS_PATH, 'Devnet allocation wallet config');
  const record = readJson(DEPLOYMENT_RECORD_PATH, 'Devnet deployment record');
  const entries = flattenWalletEntries(wallets.wallets);

  const rpcUrl = process.env.SOLANA_RPC_URL || 'https://api.devnet.solana.com';
  const keypairPath = process.env.SOLANA_KEYPAIR_PATH || '.local/devnet-deployer.json';
  const mint = record.token.mintAddress;
  const sourceTokenAccount = record.token.sourceTokenAccount;
  const total = entries.reduce((sum, entry) => sum + BigInt(entry.amount), 0n);

  console.log('==============================================');
  console.log('Pera-X Devnet Allocation Transfer Execution');
  console.log('==============================================');
  console.log(`Mode: ${EXECUTE ? 'EXECUTE' : 'DRY RUN / PLAN ONLY'}`);
  console.log('');
  console.log(`Mint: ${mint}`);
  console.log(`Source token account: ${sourceTokenAccount}`);
  console.log(`Allocation entries: ${entries.length}`);
  console.log(`Total transfer amount: ${total.toString()} PEX`);
  console.log('');

  if (record.status && record.status.allocationTransfersExecuted) {
    console.log('Deployment record already marks allocationTransfersExecuted=true. Refusing duplicate execution.');
    return;
  }

  if (!EXECUTE) {
    console.log('Dry run transfer plan:');
    for (const entry of entries) {
      console.log(`- ${entry.allocationKey}: ${entry.amount} PEX -> ${entry.address}`);
    }
    console.log('');
    console.log('Run with --execute only after reviewing all destinations.');
    return;
  }

  run('solana', ['config', 'set', '--url', rpcUrl, '--keypair', keypairPath]);

  const results = [];
  for (const entry of entries) {
    console.log('');
    console.log(`Allocation: ${entry.allocationKey}`);
    console.log(`Owner wallet: ${entry.address}`);
    console.log(`Expected amount: ${entry.amount} PEX`);

    const ataOutput = run('spl-token', ['create-account', mint, '--owner', entry.address, '--fee-payer', keypairPath], { allowAccountAlreadyExists: true });
    const destinationTokenAccount = parseAta(ataOutput);
    const expectedBaseUnits = BigInt(entry.amount) * TOKEN_SCALE;
    const currentBaseUnits = getTokenBalanceBaseUnits(destinationTokenAccount);

    console.log(`Destination token account: ${destinationTokenAccount}`);
    console.log(`Current balance: ${baseUnitsToUiAmount(currentBaseUnits)} PEX`);

    let transferredAmount = '0';
    let status = 'skipped_already_funded';
    if (currentBaseUnits < expectedBaseUnits) {
      const missingBaseUnits = expectedBaseUnits - currentBaseUnits;
      const missingUiAmount = baseUnitsToUiAmount(missingBaseUnits);
      console.log(`Missing amount: ${missingUiAmount} PEX`);
      run('spl-token', ['transfer', mint, missingUiAmount, destinationTokenAccount, '--from', sourceTokenAccount, '--owner', keypairPath, '--fee-payer', keypairPath, '--allow-unfunded-recipient']);
      transferredAmount = missingUiAmount;
      status = 'transferred_missing_amount';
    } else if (currentBaseUnits > expectedBaseUnits) {
      status = 'overfunded_manual_review_required';
      console.log(`WARNING: destination already has more than expected. Expected ${entry.amount}, current ${baseUnitsToUiAmount(currentBaseUnits)}.`);
    } else {
      console.log('Expected allocation already funded. Skipping transfer.');
    }

    const finalBaseUnits = getTokenBalanceBaseUnits(destinationTokenAccount);
    results.push({
      allocationKey: entry.allocationKey,
      percentage: entry.percentage,
      expectedAmount: entry.amount,
      ownerWallet: entry.address,
      tokenAccount: destinationTokenAccount,
      transferredAmount,
      finalBalance: baseUnitsToUiAmount(finalBaseUnits),
      status,
    });
  }

  console.log('');
  console.log('Allocation transfer run completed.');
  console.log('ALLOCATION_TRANSFER_RESULTS_JSON=' + JSON.stringify(results));
}

main();
