const {
  PublicKey,
  TOKEN_PROGRAM_ID,
  VAULT_REGISTRY_PATH,
  VAULT_CLASS_BY_ALLOCATION,
  DEFAULT_PUBLIC_KEY,
  readJson,
  allocationKeyFromId,
  expectedBaseUnits,
  deriveVaultAddresses,
  buildContext,
  enumVariantName,
} = require('./reserve-vault-utils');
const { getAccount, getMint } = require('@solana/spl-token');

async function main() {
  const { deployment, connection, program, programId, mint, statePda } = buildContext();
  const registry = readJson(VAULT_REGISTRY_PATH, 'Public reserve-vault registry');
  const failures = [];
  const expectedKeys = new Set(deployment.allocations.map((item) => item.allocationKey));
  const mintInfo = await getMint(connection, mint, 'confirmed', TOKEN_PROGRAM_ID);
  const expectedSupply = 1_000_000_000n * 1_000_000n;

  if (mintInfo.supply !== expectedSupply) {
    failures.push(`Mint supply is ${mintInfo.supply}, expected ${expectedSupply}.`);
  }
  if (mintInfo.mintAuthority !== null) failures.push('Mint authority is not disabled.');
  if (mintInfo.freezeAuthority !== null) failures.push('Freeze authority is not disabled.');

  const allConfigs = await program.account.reserveVaultConfig.all();
  const observedKeys = new Set();
  let combinedVaultBalances = 0n;
  let combinedAuthorizedRemaining = 0n;
  let combinedLegacyBalances = 0n;

  console.log('================================================');
  console.log('Pera-X Devnet Reserve Vault Verification');
  console.log('================================================');
  console.log(`Program: ${programId.toBase58()}`);
  console.log(`State: ${statePda.toBase58()}`);
  console.log(`PEX mint: ${mint.toBase58()}`);
  console.log(`Mint supply base units: ${mintInfo.supply}`);
  console.log('');

  for (const allocation of deployment.allocations) {
    const allocationKey = allocation.allocationKey;
    const registryEntry = registry.vaults.find((item) => item.allocationKey === allocationKey);
    if (!registryEntry) {
      failures.push(`${allocationKey}: missing public registry entry.`);
      continue;
    }

    const expected = expectedBaseUnits(allocation);
    const derived = deriveVaultAddresses(programId, mint, allocationKey);
    const configPda = new PublicKey(registryEntry.configPda);
    if (!configPda.equals(derived.configPda)) {
      failures.push(`${allocationKey}: registry config PDA is incorrect.`);
    }
    if (registryEntry.authorityPda !== derived.authorityPda.toBase58()) {
      failures.push(`${allocationKey}: registry authority PDA is incorrect.`);
    }
    if (registryEntry.tokenAccount !== derived.tokenAccount.toBase58()) {
      failures.push(`${allocationKey}: registry token account is incorrect.`);
    }

    const config = await program.account.reserveVaultConfig.fetch(derived.configPda);
    const decodedKey = allocationKeyFromId(config.allocationId);
    observedKeys.add(decodedKey);
    if (decodedKey !== allocationKey) failures.push(`${allocationKey}: allocation ID mismatch.`);
    if (!config.state.equals(statePda)) failures.push(`${allocationKey}: state link mismatch.`);
    if (!config.tokenMint.equals(mint)) failures.push(`${allocationKey}: wrong mint in config.`);
    if (!config.vaultAuthority.equals(derived.authorityPda)) {
      failures.push(`${allocationKey}: wrong PDA authority in config.`);
    }
    if (!config.vaultTokenAccount.equals(derived.tokenAccount)) {
      failures.push(`${allocationKey}: wrong token account in config.`);
    }
    if (!config.authorizedSourceOwner.equals(new PublicKey(allocation.ownerWallet))) {
      failures.push(`${allocationKey}: authorized source owner mismatch.`);
    }
    if (!config.authorizedSourceTokenAccount.equals(new PublicKey(allocation.tokenAccount))) {
      failures.push(`${allocationKey}: authorized source token account mismatch.`);
    }
    if (BigInt(config.allocationCap.toString()) !== expected) {
      failures.push(`${allocationKey}: allocation cap mismatch.`);
    }
    if (enumVariantName(config.vaultClass) !== VAULT_CLASS_BY_ALLOCATION[allocationKey]) {
      failures.push(`${allocationKey}: vault class mismatch.`);
    }

    const registryDestinationOwner = registryEntry.approvedDestinationOwner
      ? new PublicKey(registryEntry.approvedDestinationOwner)
      : DEFAULT_PUBLIC_KEY;
    const registryDestinationTokenAccount = registryEntry.approvedDestinationTokenAccount
      ? new PublicKey(registryEntry.approvedDestinationTokenAccount)
      : DEFAULT_PUBLIC_KEY;
    if (!config.approvedDestinationOwner.equals(registryDestinationOwner)) {
      failures.push(`${allocationKey}: approved destination owner mismatch.`);
    }
    if (!config.approvedDestinationTokenAccount.equals(registryDestinationTokenAccount)) {
      failures.push(`${allocationKey}: approved destination token account mismatch.`);
    }

    if (!registryDestinationTokenAccount.equals(DEFAULT_PUBLIC_KEY)) {
      const destinationAccount = await getAccount(
        connection,
        registryDestinationTokenAccount,
        'confirmed',
        TOKEN_PROGRAM_ID
      );
      if (!destinationAccount.mint.equals(mint)) {
        failures.push(`${allocationKey}: approved destination uses wrong mint.`);
      }
      if (!destinationAccount.owner.equals(registryDestinationOwner)) {
        failures.push(`${allocationKey}: approved destination owner does not control its token account.`);
      }
      if (destinationAccount.owner.equals(derived.authorityPda)) {
        failures.push(`${allocationKey}: approved destination is the same reserve authority.`);
      }
    }

    const vaultAccount = await getAccount(connection, derived.tokenAccount, 'confirmed', TOKEN_PROGRAM_ID);
    if (!vaultAccount.mint.equals(mint)) failures.push(`${allocationKey}: vault uses wrong mint.`);
    if (!vaultAccount.owner.equals(derived.authorityPda)) {
      failures.push(`${allocationKey}: vault is not owned by its PDA authority.`);
    }
    const vaultBalance = vaultAccount.amount;
    const legacyAccount = await getAccount(
      connection,
      new PublicKey(allocation.tokenAccount),
      'confirmed',
      TOKEN_PROGRAM_ID
    );
    const legacyBalance = legacyAccount.amount;
    const authorizedDeposited = BigInt(config.authorizedDeposited.toString());
    const unsolicitedBalance = BigInt(config.unsolicitedBalance.toString());
    const released = BigInt(config.totalReleased.toString());

    if (released > authorizedDeposited) {
      failures.push(`${allocationKey}: released amount exceeds authorized deposits.`);
    } else {
      const expectedVaultBalance = authorizedDeposited - released + unsolicitedBalance;
      if (vaultBalance !== expectedVaultBalance) {
        failures.push(
          `${allocationKey}: vault balance does not match authorized remaining plus unsolicited balance.`
        );
      }
    }
    if (authorizedDeposited > expected) {
      failures.push(`${allocationKey}: authorized deposits exceed the approved allocation.`);
    }
    if (legacyBalance + authorizedDeposited !== expected) {
      failures.push(
        `${allocationKey}: legacy balance plus authorized deposits does not equal the allocation.`
      );
    }

    combinedVaultBalances += vaultBalance;
    combinedAuthorizedRemaining += authorizedDeposited - released;
    combinedLegacyBalances += legacyBalance;

    console.log(`${allocationKey}`);
    console.log(`  config: ${derived.configPda.toBase58()}`);
    console.log(`  authority: ${derived.authorityPda.toBase58()}`);
    console.log(`  vault balance: ${vaultBalance}`);
    console.log(`  authorized deposited: ${authorizedDeposited}`);
    console.log(`  unsolicited balance: ${unsolicitedBalance}`);
    console.log(`  released: ${released}`);
    console.log(`  old allocation balance: ${legacyBalance}`);
    console.log('');
  }

  if (allConfigs.length !== expectedKeys.size) {
    failures.push(
      `Program has ${allConfigs.length} reserve configs; expected exactly ${expectedKeys.size}.`
    );
  }
  for (const config of allConfigs) {
    const key = allocationKeyFromId(config.account.allocationId);
    if (!expectedKeys.has(key)) failures.push(`Unauthorized or unknown vault config found: ${key}.`);
  }
  for (const key of expectedKeys) {
    if (!observedKeys.has(key)) failures.push(`Approved vault config missing: ${key}.`);
  }

  const releasedTotal = allConfigs.reduce(
    (sum, item) => sum + BigInt(item.account.totalReleased.toString()),
    0n
  );
  if (combinedAuthorizedRemaining + combinedLegacyBalances + releasedTotal !== expectedSupply) {
    failures.push('Authorized remaining, old-account, and released accounting does not equal 1 billion PEX.');
  }

  console.log(`Combined physical vault balances: ${combinedVaultBalances}`);
  console.log(`Combined authorized remaining: ${combinedAuthorizedRemaining}`);
  console.log(`Combined old allocation balances: ${combinedLegacyBalances}`);
  console.log(`Combined released amount: ${releasedTotal}`);
  console.log(`Mint supply: ${mintInfo.supply}`);
  console.log('');

  if (failures.length > 0) {
    console.error('VERIFICATION FAILED');
    for (const failure of failures) console.error(`- ${failure}`);
    process.exit(1);
  }

  console.log('VERIFICATION PASSED');
  console.log('Authorized migration sources, destinations, accounting, and PDA custody are correct.');
  console.log('No minting occurred and total PEX supply remains exactly 1 billion.');
}

main().catch((error) => {
  console.error(error);
  process.exit(1);
});
