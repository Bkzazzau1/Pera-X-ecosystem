const fs = require('fs');
const path = require('path');
const anchor = require('@coral-xyz/anchor');
const {
  Connection,
  Keypair,
  PublicKey,
  SystemProgram,
} = require('@solana/web3.js');
const {
  ASSOCIATED_TOKEN_PROGRAM_ID,
  TOKEN_PROGRAM_ID,
  getAssociatedTokenAddressSync,
} = require('@solana/spl-token');

const ROOT = path.resolve(__dirname, '..');
const DEPLOYMENT_RECORD_PATH = path.join(ROOT, 'config/deployment-record.devnet.public.json');
const VAULT_REGISTRY_PATH = path.join(ROOT, 'config/reserve-vaults.devnet.public.json');
const LOCAL_MIGRATION_CONFIG_PATH = path.join(
  ROOT,
  'config/reserve-vault-migration.devnet.local.json'
);
const IDL_PATH = path.join(ROOT, 'target/idl/perax_core.json');
const DEFAULT_RPC_URL = 'https://api.devnet.solana.com';
const DEFAULT_KEYPAIR_PATH = '.local/devnet-deployer.json';
const DEFAULT_PUBLIC_KEY = new PublicKey('11111111111111111111111111111111');
const PEX_DECIMALS = 6;
const BASE_UNITS = 10n ** BigInt(PEX_DECIMALS);

const VAULT_CLASS_BY_ALLOCATION = Object.freeze({
  liquidity_pool: 'liquidity',
  community_utility_rewards: 'communityRewards',
  treasury: 'marketReserve',
  ecosystem_marketing: 'marketReserve',
  trading_company_operations: 'operations',
  development_team: 'vesting',
  founder: 'vesting',
  future_team_incentives: 'marketReserve',
  team_emergency_reserve: 'emergencyReserve',
  private_strategic_investors: 'vesting',
  advisor_wallet_1: 'vesting',
  advisor_wallet_2: 'vesting',
  advisor_wallet_3: 'vesting',
});

const MARKET_RELEASABLE_ALLOCATIONS = new Set([
  'community_utility_rewards',
  'treasury',
  'ecosystem_marketing',
  'trading_company_operations',
  'future_team_incentives',
  'team_emergency_reserve',
]);

function readJson(filePath, label) {
  if (!fs.existsSync(filePath)) {
    throw new Error(`${label} not found at ${filePath}`);
  }
  return JSON.parse(fs.readFileSync(filePath, 'utf8'));
}

function writeJson(filePath, value) {
  fs.writeFileSync(filePath, `${JSON.stringify(value, null, 2)}\n`);
}

function resolveFromRoot(filePath) {
  return path.isAbsolute(filePath) ? filePath : path.resolve(ROOT, filePath);
}

function loadKeypair(filePath, label = 'Solana keypair') {
  const resolvedPath = resolveFromRoot(filePath);
  const raw = readJson(resolvedPath, label);
  if (!Array.isArray(raw) || raw.length !== 64) {
    throw new Error(`${label} must be a JSON array containing 64 integers.`);
  }
  return Keypair.fromSecretKey(Uint8Array.from(raw));
}

function loadLocalMigrationConfig(required = false) {
  if (!fs.existsSync(LOCAL_MIGRATION_CONFIG_PATH)) {
    if (required) {
      throw new Error(
        `Local migration configuration not found at ${LOCAL_MIGRATION_CONFIG_PATH}.`
      );
    }
    return { allocationSigners: {}, approvedDestinations: {} };
  }
  const value = readJson(
    LOCAL_MIGRATION_CONFIG_PATH,
    'Local reserve-vault migration configuration'
  );
  return {
    allocationSigners: value.allocationSigners || {},
    approvedDestinations: value.approvedDestinations || {},
  };
}

function isMarketReleasableAllocation(allocationKey) {
  return MARKET_RELEASABLE_ALLOCATIONS.has(allocationKey);
}

function approvedDestination(localConfig, allocationKey, required = false) {
  if (!isMarketReleasableAllocation(allocationKey)) {
    return {
      owner: DEFAULT_PUBLIC_KEY,
      tokenAccount: DEFAULT_PUBLIC_KEY,
      configured: true,
    };
  }

  const entry = localConfig.approvedDestinations?.[allocationKey];
  if (!entry || !entry.ownerWallet || !entry.tokenAccount) {
    if (required) {
      throw new Error(
        `Approved destination for ${allocationKey} is missing from ${LOCAL_MIGRATION_CONFIG_PATH}.`
      );
    }
    return {
      owner: DEFAULT_PUBLIC_KEY,
      tokenAccount: DEFAULT_PUBLIC_KEY,
      configured: false,
    };
  }

  const owner = new PublicKey(entry.ownerWallet);
  const tokenAccount = new PublicKey(entry.tokenAccount);
  if (owner.equals(DEFAULT_PUBLIC_KEY) || tokenAccount.equals(DEFAULT_PUBLIC_KEY)) {
    throw new Error(`Approved destination for ${allocationKey} cannot be the default public key.`);
  }
  return { owner, tokenAccount, configured: true };
}

function allocationId(allocationKey) {
  const label = Buffer.from(allocationKey, 'utf8');
  if (label.length > 32) {
    throw new Error(`Allocation key exceeds 32 bytes: ${allocationKey}`);
  }
  const id = Buffer.alloc(32);
  label.copy(id);
  return id;
}

function allocationKeyFromId(id) {
  return Buffer.from(id).toString('utf8').replace(/\0+$/g, '');
}

function vaultClassArg(allocationKey) {
  const variant = VAULT_CLASS_BY_ALLOCATION[allocationKey];
  if (!variant) throw new Error(`Unsupported allocation key: ${allocationKey}`);
  return { [variant]: {} };
}

function expectedBaseUnits(allocation) {
  return BigInt(allocation.expectedAmount) * BASE_UNITS;
}

function deriveVaultAddresses(programId, mint, allocationKey) {
  const id = allocationId(allocationKey);
  const [configPda] = PublicKey.findProgramAddressSync(
    [Buffer.from('reserve-config'), id],
    programId
  );
  const [authorityPda] = PublicKey.findProgramAddressSync(
    [Buffer.from('reserve-authority'), id],
    programId
  );
  const tokenAccount = getAssociatedTokenAddressSync(
    mint,
    authorityPda,
    true,
    TOKEN_PROGRAM_ID,
    ASSOCIATED_TOKEN_PROGRAM_ID
  );
  return { id, configPda, authorityPda, tokenAccount };
}

function buildContext() {
  const deployment = readJson(DEPLOYMENT_RECORD_PATH, 'Devnet deployment record');
  const rpcUrl = process.env.SOLANA_RPC_URL || DEFAULT_RPC_URL;
  const keypairPath = process.env.SOLANA_KEYPAIR_PATH || DEFAULT_KEYPAIR_PATH;
  const payer = loadKeypair(keypairPath, 'Devnet fee-payer keypair');
  const connection = new Connection(rpcUrl, 'confirmed');
  const wallet = new anchor.Wallet(payer);
  const provider = new anchor.AnchorProvider(connection, wallet, {
    commitment: 'confirmed',
    preflightCommitment: 'confirmed',
  });

  const idl = readJson(IDL_PATH, 'Built perax_core IDL');
  const programId = new PublicKey(deployment.program.programId);
  idl.address = programId.toBase58();
  const program = new anchor.Program(idl, provider);
  const mint = new PublicKey(deployment.token.mintAddress);
  const [statePda] = PublicKey.findProgramAddressSync(
    [Buffer.from('perax-state')],
    programId
  );

  return {
    deployment,
    connection,
    payer,
    provider,
    program,
    programId,
    mint,
    statePda,
  };
}

function bnFromBigInt(value) {
  return new anchor.BN(value.toString());
}

function enumVariantName(value) {
  if (!value || typeof value !== 'object') return null;
  return Object.keys(value)[0] || null;
}

module.exports = {
  anchor,
  PublicKey,
  SystemProgram,
  TOKEN_PROGRAM_ID,
  ASSOCIATED_TOKEN_PROGRAM_ID,
  DEPLOYMENT_RECORD_PATH,
  VAULT_REGISTRY_PATH,
  LOCAL_MIGRATION_CONFIG_PATH,
  DEFAULT_PUBLIC_KEY,
  PEX_DECIMALS,
  BASE_UNITS,
  VAULT_CLASS_BY_ALLOCATION,
  MARKET_RELEASABLE_ALLOCATIONS,
  readJson,
  writeJson,
  loadKeypair,
  loadLocalMigrationConfig,
  isMarketReleasableAllocation,
  approvedDestination,
  allocationId,
  allocationKeyFromId,
  vaultClassArg,
  expectedBaseUnits,
  deriveVaultAddresses,
  buildContext,
  bnFromBigInt,
  enumVariantName,
};
