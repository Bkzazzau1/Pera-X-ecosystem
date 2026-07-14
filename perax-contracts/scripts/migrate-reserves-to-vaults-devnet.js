const {
  PublicKey,
  TOKEN_PROGRAM_ID,
  LOCAL_MIGRATION_CONFIG_PATH,
  VAULT_REGISTRY_PATH,
  readJson,
  loadKeypair,
  allocationId,
  expectedBaseUnits,
  buildContext,
  bnFromBigInt,
} = require('./reserve-vault-utils');

const EXECUTE = process.argv.includes('--execute');
const ONLY_INDEX = process.argv.indexOf('--only');
const ONLY_ALLOCATION = ONLY_INDEX >= 0 ? process.argv[ONLY_INDEX + 1] : null;
const AMOUNT_INDEX = process.argv.indexOf('--amount-pex');
const MAX_AMOUNT_PEX = AMOUNT_INDEX >= 0 ? BigInt(process.argv[AMOUNT_INDEX + 1]) : null;

async function main() {
  const { deployment, connection, payer, program, mint, statePda } = buildContext();
  const registry = readJson(VAULT_REGISTRY_PATH, 'Public reserve-vault registry');
  const localConfig = readJson(
    LOCAL_MIGRATION_CONFIG_PATH,
    'Local reserve-vault migration signer configuration'
  );
  const signerPaths = localConfig.allocationSigners || {};

  console.log('================================================');
  console.log('Pera-X Devnet Reserve Migration');
  console.log('================================================');
  console.log(`Mode: ${EXECUTE ? 'EXECUTE' : 'DRY RUN / PLAN ONLY'}`);
  console.log('This script never mints PEX. It only transfers existing PEX into PDA vaults.');
  console.log('');

  const selectedAllocations = ONLY_ALLOCATION
    ? deployment.allocations.filter((item) => item.allocationKey === ONLY_ALLOCATION)
    : deployment.allocations;
  if (selectedAllocations.length === 0) {
    throw new Error(`Allocation not found: ${ONLY_ALLOCATION}`);
  }
  if (MAX_AMOUNT_PEX !== null && !ONLY_ALLOCATION) {
    throw new Error('--amount-pex may only be used together with --only.');
  }

  for (const allocation of selectedAllocations) {
    const allocationKey = allocation.allocationKey;
    const registryEntry = registry.vaults.find((item) => item.allocationKey === allocationKey);
    if (!registryEntry) throw new Error(`Vault registry entry missing for ${allocationKey}.`);

    const signerPath = signerPaths[allocationKey];
    if (!signerPath) {
      throw new Error(
        `Missing local signer path for ${allocationKey} in ${LOCAL_MIGRATION_CONFIG_PATH}.`
      );
    }
    const sourceOwner = loadKeypair(signerPath, `${allocationKey} allocation owner keypair`);
    if (sourceOwner.publicKey.toBase58() !== allocation.ownerWallet) {
      throw new Error(
        `${allocationKey}: loaded signer ${sourceOwner.publicKey.toBase58()} does not match recorded owner ${allocation.ownerWallet}.`
      );
    }

    const sourceTokenAccount = new PublicKey(allocation.tokenAccount);
    const vaultConfig = new PublicKey(registryEntry.configPda);
    const vaultAuthority = new PublicKey(registryEntry.authorityPda);
    const vaultTokenAccount = new PublicKey(registryEntry.tokenAccount);
    const config = await program.account.reserveVaultConfig.fetch(vaultConfig);
    const expected = expectedBaseUnits(allocation);
    const alreadyDeposited = BigInt(config.totalDeposited.toString());
    if (alreadyDeposited > expected) {
      throw new Error(`${allocationKey}: totalDeposited exceeds the approved allocation.`);
    }
    const remaining = expected - alreadyDeposited;
    const requestedMigration = MAX_AMOUNT_PEX === null
      ? remaining
      : (MAX_AMOUNT_PEX * 1_000_000n < remaining ? MAX_AMOUNT_PEX * 1_000_000n : remaining);
    const sourceBalance = await connection.getTokenAccountBalance(sourceTokenAccount, 'confirmed');
    const sourceAmount = BigInt(sourceBalance.value.amount);

    console.log(`${allocationKey}`);
    console.log(`  expected base units: ${expected}`);
    console.log(`  already deposited: ${alreadyDeposited}`);
    console.log(`  remaining migration: ${remaining}`);
    console.log(`  amount selected now: ${requestedMigration}`);
    console.log(`  source balance: ${sourceAmount}`);

    if (remaining === 0n || requestedMigration === 0n) {
      console.log('  status: already fully migrated');
      console.log('');
      continue;
    }
    if (sourceAmount < requestedMigration) {
      throw new Error(`${allocationKey}: source account does not hold the remaining amount.`);
    }

    if (EXECUTE) {
      const signerList = sourceOwner.publicKey.equals(payer.publicKey) ? [] : [sourceOwner];
      const signature = await program.methods
        .depositIntoReserveVault(Array.from(allocationId(allocationKey)), bnFromBigInt(requestedMigration))
        .accounts({
          state: statePda,
          reserveVaultConfig: vaultConfig,
          vaultAuthority,
          sourceOwner: sourceOwner.publicKey,
          sourceTokenAccount,
          vaultTokenAccount,
          tokenMint: mint,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .signers(signerList)
        .rpc();
      console.log(`  signature: ${signature}`);
    } else {
      console.log('  status: planned only');
    }
    console.log('');
  }

  if (!EXECUTE) {
    console.log('Dry run complete. No PEX was transferred.');
    console.log('Re-run with --execute only after the small-vault trial passes.');
  } else {
    console.log('Migration transactions completed. Run verify-reserve-vaults-devnet.js next.');
  }
}

main().catch((error) => {
  console.error(error);
  process.exit(1);
});
