const { execFileSync } = require('child_process');
const fs = require('fs');
const path = require('path');

const EXECUTE = process.argv.includes('--execute');
const WALLETS_PATH = path.resolve(__dirname, '../config/pex-allocation-wallets.devnet.json');
const DEPLOYMENT_RECORD_PATH = path.resolve(__dirname, '../config/deployment-record.devnet.public.json');

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

function run(command, args) {
  console.log(`$ ${command} ${args.join(' ')}`);
  if (!EXECUTE) return '';
  return execFileSync(command, args, { encoding: 'utf8', stdio: ['ignore', 'pipe', 'inherit'] }).trim();
}

function parseAta(output) {
  const candidates = output.split(/\s+/).filter((part) => /^[1-9A-HJ-NP-Za-km-z]{32,44}$/.test(part));
  if (candidates.length === 0) throw new Error(`Could not parse token account from output: ${output}`);
  return candidates[candidates.length - 1];
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
    console.log(`Amount: ${entry.amount} PEX`);

    const ataOutput = run('spl-token', ['create-account', mint, '--owner', entry.address, '--fee-payer', keypairPath]);
    const destinationTokenAccount = parseAta(ataOutput);
    run('spl-token', ['transfer', mint, entry.amount, destinationTokenAccount, '--from', sourceTokenAccount, '--owner', keypairPath, '--fee-payer', keypairPath, '--allow-unfunded-recipient']);

    results.push({
      allocationKey: entry.allocationKey,
      percentage: entry.percentage,
      amount: entry.amount,
      ownerWallet: entry.address,
      tokenAccount: destinationTokenAccount,
    });
  }

  console.log('');
  console.log('Allocation transfers completed.');
  console.log('ALLOCATION_TRANSFER_RESULTS_JSON=' + JSON.stringify(results));
}

main();
