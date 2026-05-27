const crypto = require('crypto');
const fs = require('fs');
const path = require('path');
const {
  Connection,
  Keypair,
  PublicKey,
  SystemProgram,
  Transaction,
  TransactionInstruction,
  sendAndConfirmTransaction,
} = require('@solana/web3.js');

const DEPLOYMENT_RECORD_PATH = path.resolve(__dirname, '../config/deployment-record.devnet.public.json');
const EXECUTE = process.argv.includes('--execute');

const DEFAULTS = {
  rpcUrl: 'https://api.devnet.solana.com',
  keypairPath: '.local/devnet-deployer.json',
  programId: 'FqEiSx5vujh2vi3yk12NaZMXhjMSaKovGUuzcKiAgshn',
  pexMint: 'DnkAW3B1ckzW6eimgSBNPK3XTt83wMiZRETy8iF3gdsn',
  lockedTradingCompanyTokenAccount: 'Wcx1HaNWZQmtfn1yfS3FWXv7AKSXV8ftciezTX3ufs8',
  revenueTradingCompanyTokenAccount: 'E4JJoNkKhFq8Ev7XtJhuWrvdyNyWRzkwjsukgF9VbegA',
  safetyAdminWallet: '2FuGxUt1EaewriZ9QpSkqz8Lbj2M69q2F4vnr1NJHPHX',
  oracleBotWallet: '4hx4YvYdyQMziqxqTv5gjRbLcXZFwX1kaL9vdPBYdU9D',
  launchPrice: 1200n,
  currentSteppedFloor: 1200n,
  dailyReleaseCap: 10_000_000n * 1_000_000n,
  monthlyReleaseCap: 150_000_000n * 1_000_000n,
  emergencyHourlyReleaseBps: 50,
  maxPaymentAmount: 0n,
};

function readJson(filePath, label) {
  if (!fs.existsSync(filePath)) throw new Error(`${label} not found at ${filePath}`);
  return JSON.parse(fs.readFileSync(filePath, 'utf8'));
}

function loadKeypair(filePath) {
  const resolvedPath = path.resolve(process.cwd(), filePath);
  const raw = readJson(resolvedPath, 'Deployer keypair');
  if (!Array.isArray(raw) || raw.length !== 64) {
    throw new Error('Deployer keypair must be a Solana JSON array with 64 integers.');
  }
  return Keypair.fromSecretKey(Uint8Array.from(raw));
}

function anchorDiscriminator(name) {
  return crypto.createHash('sha256').update(`global:${name}`).digest().subarray(0, 8);
}

function writePubkey(value) {
  return Buffer.from(new PublicKey(value).toBytes());
}

function writeU64(value) {
  const buffer = Buffer.alloc(8);
  buffer.writeBigUInt64LE(BigInt(value));
  return buffer;
}

function writeU16(value) {
  const buffer = Buffer.alloc(2);
  buffer.writeUInt16LE(Number(value));
  return buffer;
}

function buildInitializeData(params) {
  return Buffer.concat([
    anchorDiscriminator('initialize'),
    writePubkey(params.tokenMint),
    writePubkey(params.tradingCompanyTokenAccount),
    writePubkey(params.tradingCompanyRevenueTokenAccount),
    writeU64(params.maxPaymentAmount),
    writePubkey(params.safetyAdmin),
    writePubkey(params.oracleFeed),
    writeU64(params.launchPrice),
    writeU64(params.currentSteppedFloor),
    writeU64(params.dailyReleaseCap),
    writeU64(params.monthlyReleaseCap),
    writeU16(params.emergencyHourlyReleaseBps),
  ]);
}

async function main() {
  const record = readJson(DEPLOYMENT_RECORD_PATH, 'Devnet deployment record');
  const rpcUrl = process.env.SOLANA_RPC_URL || DEFAULTS.rpcUrl;
  const keypairPath = process.env.SOLANA_KEYPAIR_PATH || DEFAULTS.keypairPath;
  const programId = new PublicKey(record.program.programId || DEFAULTS.programId);
  const authority = loadKeypair(keypairPath);
  const connection = new Connection(rpcUrl, 'confirmed');
  const [statePda, stateBump] = PublicKey.findProgramAddressSync([Buffer.from('perax-state')], programId);

  const params = {
    tokenMint: record.token.mintAddress || DEFAULTS.pexMint,
    tradingCompanyTokenAccount: record.tradingCompany.lockedStrategicTokenAccount || DEFAULTS.lockedTradingCompanyTokenAccount,
    tradingCompanyRevenueTokenAccount: record.tradingCompany.revenueTokenAccount || DEFAULTS.revenueTradingCompanyTokenAccount,
    maxPaymentAmount: DEFAULTS.maxPaymentAmount,
    safetyAdmin: record.program.safetyAdminWallet || DEFAULTS.safetyAdminWallet,
    oracleFeed: record.program.oracleBotWallet || DEFAULTS.oracleBotWallet,
    launchPrice: DEFAULTS.launchPrice,
    currentSteppedFloor: DEFAULTS.currentSteppedFloor,
    dailyReleaseCap: DEFAULTS.dailyReleaseCap,
    monthlyReleaseCap: DEFAULTS.monthlyReleaseCap,
    emergencyHourlyReleaseBps: DEFAULTS.emergencyHourlyReleaseBps,
  };

  console.log('==============================================');
  console.log('Pera-X Core Program Initialization');
  console.log('==============================================');
  console.log(`Mode: ${EXECUTE ? 'EXECUTE' : 'DRY RUN / PLAN ONLY'}`);
  console.log('');
  console.log(`Program ID: ${programId.toBase58()}`);
  console.log(`State PDA: ${statePda.toBase58()}`);
  console.log(`State bump: ${stateBump}`);
  console.log(`Authority: ${authority.publicKey.toBase58()}`);
  console.log(`PEX mint: ${params.tokenMint}`);
  console.log(`Locked TC token account: ${params.tradingCompanyTokenAccount}`);
  console.log(`Revenue TC token account: ${params.tradingCompanyRevenueTokenAccount}`);
  console.log(`Safety admin: ${params.safetyAdmin}`);
  console.log(`Oracle/Bot: ${params.oracleFeed}`);
  console.log(`Launch price scaled: ${params.launchPrice.toString()}`);
  console.log(`Daily release cap base units: ${params.dailyReleaseCap.toString()}`);
  console.log(`Monthly release cap base units: ${params.monthlyReleaseCap.toString()}`);
  console.log(`Emergency hourly release bps: ${params.emergencyHourlyReleaseBps}`);
  console.log('');

  const existingState = await connection.getAccountInfo(statePda);
  if (existingState) {
    console.log('State PDA already exists. No initialization transaction sent.');
    console.log(`STATE_PDA=${statePda.toBase58()}`);
    console.log(`STATE_BUMP=${stateBump}`);
    console.log(`STATE_ALREADY_INITIALIZED=true`);
    return;
  }

  if (!EXECUTE) {
    console.log('This script will create the perax-state PDA when run with --execute.');
    console.log('Run only after devnet program deployment is confirmed.');
    return;
  }

  const instruction = new TransactionInstruction({
    programId,
    keys: [
      { pubkey: statePda, isSigner: false, isWritable: true },
      { pubkey: authority.publicKey, isSigner: true, isWritable: true },
      { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
    ],
    data: buildInitializeData(params),
  });

  const transaction = new Transaction().add(instruction);
  const signature = await sendAndConfirmTransaction(connection, transaction, [authority], {
    commitment: 'confirmed',
    preflightCommitment: 'confirmed',
  });

  console.log('Core program initialized successfully.');
  console.log(`INITIALIZE_SIGNATURE=${signature}`);
  console.log(`STATE_PDA=${statePda.toBase58()}`);
  console.log(`STATE_BUMP=${stateBump}`);
}

main().catch((error) => {
  console.error(error);
  process.exit(1);
});
