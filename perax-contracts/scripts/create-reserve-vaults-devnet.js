const {
  SystemProgram,
  TOKEN_PROGRAM_ID,
  ASSOCIATED_TOKEN_PROGRAM_ID,
  VAULT_REGISTRY_PATH,
  VAULT_CLASS_BY_ALLOCATION,
  writeJson,
  vaultClassArg,
  expectedBaseUnits,
  deriveVaultAddresses,
  buildContext,
  bnFromBigInt,
} = require('./reserve-vault-utils');

const EXECUTE = process.argv.includes('--execute');
const ONLY_INDEX = process.argv.indexOf('--only');
const ONLY_ALLOCATION = ONLY_INDEX >= 0 ? process.argv[ONLY_INDEX + 1] : null;

async function main() {
  const { deployment, connection, payer, program, programId, mint, statePda } = buildContext();
  const state = await program.account.peraxState.fetch(statePda);

  if (!state.tokenMint.equals(mint)) {
    throw new Error('On-chain PeraxState token mint does not match the public deployment record.');
  }
  if (!state.authority.equals(payer.publicKey)) {
    throw new Error(
      `Fee-payer ${payer.publicKey.toBase58()} is not the current program authority ${state.authority.toBase58()}.`
    );
  }

  console.log('================================================');
  console.log('Pera-X Devnet Reserve Vault Creation');
  console.log('================================================');
  console.log(`Mode: ${EXECUTE ? 'EXECUTE' : 'DRY RUN / PLAN ONLY'}`);
  console.log(`Program: ${programId.toBase58()}`);
  console.log(`State: ${statePda.toBase58()}`);
  console.log(`PEX mint: ${mint.toBase58()}`);
  console.log('No PEX will be transferred by this script.');
  console.log('');

  const registry = {
    project: 'Pera-X',
    cluster: 'devnet',
    programId: programId.toBase58(),
    statePda: statePda.toBase58(),
    tokenMint: mint.toBase58(),
    generatedAt: new Date().toISOString(),
    vaults: [],
  };

  const selectedAllocations = ONLY_ALLOCATION
    ? deployment.allocations.filter((item) => item.allocationKey === ONLY_ALLOCATION)
    : deployment.allocations;
  if (selectedAllocations.length === 0) {
    throw new Error(`Allocation not found: ${ONLY_ALLOCATION}`);
  }

  for (const allocation of selectedAllocations) {
    const allocationKey = allocation.allocationKey;
    const vaultClass = VAULT_CLASS_BY_ALLOCATION[allocationKey];
    if (!vaultClass) throw new Error(`No vault class configured for ${allocationKey}.`);

    const cap = expectedBaseUnits(allocation);
    const { id, configPda, authorityPda, tokenAccount } = deriveVaultAddresses(
      programId,
      mint,
      allocationKey
    );
    const existing = await connection.getAccountInfo(configPda, 'confirmed');
    let signature = null;
    let status = existing ? 'already_initialized' : 'planned';

    console.log(`${allocationKey}`);
    console.log(`  class: ${vaultClass}`);
    console.log(`  cap base units: ${cap}`);
    console.log(`  config PDA: ${configPda.toBase58()}`);
    console.log(`  authority PDA: ${authorityPda.toBase58()}`);
    console.log(`  vault token account: ${tokenAccount.toBase58()}`);

    if (!existing && EXECUTE) {
      signature = await program.methods
        .initializeReserveVault(Array.from(id), vaultClassArg(allocationKey), bnFromBigInt(cap))
        .accounts({
          state: statePda,
          authority: payer.publicKey,
          reserveVaultConfig: configPda,
          vaultAuthority: authorityPda,
          vaultTokenAccount: tokenAccount,
          tokenMint: mint,
          tokenProgram: TOKEN_PROGRAM_ID,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
        })
        .rpc();
      status = 'initialized';
      console.log(`  signature: ${signature}`);
    } else if (existing) {
      const config = await program.account.reserveVaultConfig.fetch(configPda);
      if (!config.tokenMint.equals(mint)) throw new Error(`${allocationKey}: wrong mint.`);
      if (!config.vaultAuthority.equals(authorityPda)) {
        throw new Error(`${allocationKey}: wrong vault authority PDA.`);
      }
      if (!config.vaultTokenAccount.equals(tokenAccount)) {
        throw new Error(`${allocationKey}: wrong vault token account.`);
      }
      if (BigInt(config.allocationCap.toString()) !== cap) {
        throw new Error(`${allocationKey}: existing cap differs from approved allocation.`);
      }
      console.log('  existing configuration verified');
    }

    registry.vaults.push({
      allocationKey,
      vaultClass,
      allocationCapBaseUnits: cap.toString(),
      allocationCapUiAmount: allocation.expectedAmount,
      configPda: configPda.toBase58(),
      authorityPda: authorityPda.toBase58(),
      tokenAccount: tokenAccount.toBase58(),
      sourceOwnerWallet: allocation.ownerWallet,
      sourceTokenAccount: allocation.tokenAccount,
      status,
      initializeSignature: signature,
    });
    console.log('');
  }

  if (EXECUTE) {
    writeJson(VAULT_REGISTRY_PATH, registry);
    console.log(`Public vault registry written to ${VAULT_REGISTRY_PATH}`);
  } else {
    console.log('Dry run complete. Re-run with --execute only after the upgraded program is deployed.');
  }
}

main().catch((error) => {
  console.error(error);
  process.exit(1);
});
