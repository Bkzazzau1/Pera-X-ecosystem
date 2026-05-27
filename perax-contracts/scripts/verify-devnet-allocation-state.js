const { execFileSync } = require('child_process');
const fs = require('fs');
const path = require('path');

const DEPLOYMENT_RECORD_PATH = path.resolve(__dirname, '../config/deployment-record.devnet.public.json');
const TOKEN_DECIMALS = 6n;
const TOKEN_SCALE = 10n ** TOKEN_DECIMALS;

function readJson(filePath, label) {
  if (!fs.existsSync(filePath)) throw new Error(`${label} not found at ${filePath}`);
  return JSON.parse(fs.readFileSync(filePath, 'utf8'));
}

function run(command, args) {
  console.log(`$ ${command} ${args.join(' ')}`);
  return execFileSync(command, args, { encoding: 'utf8', stdio: ['ignore', 'pipe', 'pipe'] }).trim();
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
  const record = readJson(DEPLOYMENT_RECORD_PATH, 'Devnet deployment record');
  const rpcUrl = process.env.SOLANA_RPC_URL || 'https://api.devnet.solana.com';
  const keypairPath = process.env.SOLANA_KEYPAIR_PATH || '.local/devnet-deployer.json';

  const mint = record.token.mintAddress;
  const sourceTokenAccount = record.token.sourceTokenAccount;
  const allocations = record.allocations || [];

  console.log('==============================================');
  console.log('Pera-X Devnet Allocation State Verification');
  console.log('==============================================');
  console.log('');

  run('solana', ['config', 'set', '--url', rpcUrl, '--keypair', keypairPath]);

  console.log(`Mint: ${mint}`);
  console.log(`Source token account: ${sourceTokenAccount}`);
  console.log(`Allocation entries: ${allocations.length}`);
  console.log('');

  const mintSupplyOutput = run('spl-token', ['supply', mint]);
  const mintSupplyBaseUnits = parseUiAmountToBaseUnits(mintSupplyOutput);
  const expectedMintSupplyBaseUnits = BigInt(record.token.fixedSupply) * TOKEN_SCALE;

  const sourceBalanceBaseUnits = getTokenBalanceBaseUnits(sourceTokenAccount);
  let allocationTotalBaseUnits = 0n;
  const failures = [];
  const results = [];

  for (const allocation of allocations) {
    const expectedBaseUnits = BigInt(allocation.expectedAmount) * TOKEN_SCALE;
    const actualBaseUnits = getTokenBalanceBaseUnits(allocation.tokenAccount);
    allocationTotalBaseUnits += actualBaseUnits;

    const ok = actualBaseUnits === expectedBaseUnits;
    if (!ok) {
      failures.push(`${allocation.allocationKey} expected ${allocation.expectedAmount}, actual ${baseUnitsToUiAmount(actualBaseUnits)}`);
    }

    results.push({
      allocationKey: allocation.allocationKey,
      tokenAccount: allocation.tokenAccount,
      expectedAmount: allocation.expectedAmount,
      actualAmount: baseUnitsToUiAmount(actualBaseUnits),
      ok,
    });
  }

  if (mintSupplyBaseUnits !== expectedMintSupplyBaseUnits) {
    failures.push(`Mint supply expected ${record.token.fixedSupply}, actual ${baseUnitsToUiAmount(mintSupplyBaseUnits)}`);
  }

  if (sourceBalanceBaseUnits !== 0n) {
    failures.push(`Source token account expected 0, actual ${baseUnitsToUiAmount(sourceBalanceBaseUnits)}`);
  }

  if (allocationTotalBaseUnits !== expectedMintSupplyBaseUnits) {
    failures.push(`Allocation total expected ${record.token.fixedSupply}, actual ${baseUnitsToUiAmount(allocationTotalBaseUnits)}`);
  }

  console.log('Verification summary:');
  console.log('----------------------------------------------');
  console.log(`Mint supply: ${baseUnitsToUiAmount(mintSupplyBaseUnits)} PEX`);
  console.log(`Source balance: ${baseUnitsToUiAmount(sourceBalanceBaseUnits)} PEX`);
  console.log(`Allocation total: ${baseUnitsToUiAmount(allocationTotalBaseUnits)} PEX`);
  console.log(`All allocation balances correct: ${failures.length === 0 ? 'YES' : 'NO'}`);
  console.log('');

  console.log('ALLOCATION_VERIFICATION_RESULTS_JSON=' + JSON.stringify(results));

  if (failures.length > 0) {
    console.log('');
    console.log('Verification failures:');
    failures.forEach((failure) => console.log(`- ${failure}`));
    process.exit(1);
  }

  console.log('');
  console.log('Status: VERIFIED. Allocation state is correct.');
}

main();
