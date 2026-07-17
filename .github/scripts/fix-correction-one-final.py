from pathlib import Path

vault_path = Path("perax-contracts/programs/perax-core/src/instructions/vault.rs")
vault = vault_path.read_text()
old_cap = """    require!(
        params.allocation_cap <= approved_cap,
        PeraxError::AllocationCapExceeded
    );"""
new_cap = """    require!(
        params.allocation_cap == approved_cap,
        PeraxError::InvalidAllocationCap
    );"""
if vault.count(old_cap) != 1:
    raise SystemExit("Expected one permissive allocation-cap check")
vault_path.write_text(vault.replace(old_cap, new_cap))

test_path = Path("perax-contracts/tests/perax-core.ts")
test = test_path.read_text()

old_type = """type AllocationDefinition = {
  key: string;
  vaultClass: Record<string, Record<string, never>>;
  releasable: boolean;
};"""
new_type = """type AllocationDefinition = {
  key: string;
  vaultClass: Record<string, Record<string, never>>;
  releasable: boolean;
  approvedCapPex: number;
};"""
if test.count(old_type) != 1:
    raise SystemExit("AllocationDefinition shape changed unexpectedly")
test = test.replace(old_type, new_type)

start = test.index("const ALLOCATIONS: AllocationDefinition[] = [")
end = test.index("\n];", start) + len("\n];")
allocation_block = """const ALLOCATIONS: AllocationDefinition[] = [
  {
    key: "liquidity_pool",
    vaultClass: { liquidity: {} },
    releasable: false,
    approvedCapPex: 380_000_000,
  },
  {
    key: "community_utility_rewards",
    vaultClass: { communityRewards: {} },
    releasable: true,
    approvedCapPex: 170_000_000,
  },
  {
    key: "treasury",
    vaultClass: { marketReserve: {} },
    releasable: true,
    approvedCapPex: 120_000_000,
  },
  {
    key: "ecosystem_marketing",
    vaultClass: { marketReserve: {} },
    releasable: true,
    approvedCapPex: 120_000_000,
  },
  {
    key: "trading_company_operations",
    vaultClass: { operations: {} },
    releasable: true,
    approvedCapPex: 70_000_000,
  },
  {
    key: "development_team",
    vaultClass: { vesting: {} },
    releasable: false,
    approvedCapPex: 20_000_000,
  },
  {
    key: "founder",
    vaultClass: { vesting: {} },
    releasable: false,
    approvedCapPex: 20_000_000,
  },
  {
    key: "future_team_incentives",
    vaultClass: { marketReserve: {} },
    releasable: true,
    approvedCapPex: 10_000_000,
  },
  {
    key: "team_emergency_reserve",
    vaultClass: { emergencyReserve: {} },
    releasable: true,
    approvedCapPex: 10_000_000,
  },
  {
    key: "private_strategic_investors",
    vaultClass: { vesting: {} },
    releasable: false,
    approvedCapPex: 50_000_000,
  },
  {
    key: "advisor_wallet_1",
    vaultClass: { vesting: {} },
    releasable: false,
    approvedCapPex: 10_000_000,
  },
  {
    key: "advisor_wallet_2",
    vaultClass: { vesting: {} },
    releasable: false,
    approvedCapPex: 10_000_000,
  },
  {
    key: "advisor_wallet_3",
    vaultClass: { vesting: {} },
    releasable: false,
    approvedCapPex: 10_000_000,
  },
];"""
test = test[:start] + allocation_block + test[end:]

old_config_type = """type ReserveVaultConfigAccount = {
  authorizedDeposited: anchor.BN;
  unsolicitedBalance: anchor.BN;
  totalReleased: anchor.BN;
};"""
new_config_type = """type ReserveVaultConfigAccount = {
  allocationCap: anchor.BN;
  authorizedDeposited: anchor.BN;
  unsolicitedBalance: anchor.BN;
  totalReleased: anchor.BN;
};"""
if test.count(old_config_type) != 1:
    raise SystemExit("ReserveVaultConfigAccount shape changed unexpectedly")
test = test.replace(old_config_type, new_config_type)

old_record_type = """type ReserveReleaseRecordAccount = {
  destinationTokenAccount: anchor.web3.PublicKey;
};"""
new_record_type = """type ReserveReleaseRecordAccount = {
  destinationTokenAccount: anchor.web3.PublicKey;
  requestedAmount: anchor.BN;
};"""
if test.count(old_record_type) != 1:
    raise SystemExit("ReserveReleaseRecordAccount shape changed unexpectedly")
test = test.replace(old_record_type, new_record_type)

old_default = """      allocationCap:
        overrides.allocationCap ?? new anchor.BN(1_000 * BASE_UNITS),"""
new_default = """      allocationCap:
        overrides.allocationCap ??
        new anchor.BN(allocation.approvedCapPex).mul(new anchor.BN(BASE_UNITS)),"""
if test.count(old_default) != 1:
    raise SystemExit("Default test allocation cap changed unexpectedly")
test = test.replace(old_default, new_default)

unknown = """    const definition: AllocationDefinition = {
      key: "unknown_allocation",
      vaultClass: { marketReserve: {} },
      releasable: true,
    };"""
unknown_new = """    const definition: AllocationDefinition = {
      key: "unknown_allocation",
      vaultClass: { marketReserve: {} },
      releasable: true,
      approvedCapPex: 1,
    };"""
if test.count(unknown) != 1:
    raise SystemExit("Unknown allocation fixture changed unexpectedly")
test = test.replace(unknown, unknown_new)

marker = '  it("rejects a configured destination owned by any reserve-authority PDA", async () => {'
below_cap_test = """  it("rejects an allocation cap below the approved amount", async () => {
    const treasury = ALLOCATIONS.find((item) => item.key === "treasury")!;
    const built = await buildInitialization(treasury, {
      allocationCap: new anchor.BN(119_000_000 * BASE_UNITS),
    });
    await expectFailure(() =>
      program.methods
        .initializeReserveVault(built.params)
        .accounts({
          state,
          authority,
          reserveVaultConfig: built.config,
          vaultAuthority: built.vaultAuthority,
          vaultTokenAccount: built.tokenAccount,
          tokenMint: mint,
          tokenProgram: TOKEN_PROGRAM_ID,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .rpc()
    );
  });

"""
if test.count(marker) != 1:
    raise SystemExit("Destination-test insertion marker changed")
test = test.replace(marker, below_cap_test + marker)

old_loop = """    for (const allocation of ALLOCATIONS) {
      await initializeVault(allocation);
    }"""
new_loop = """    for (const allocation of ALLOCATIONS) {
      const initialized = await initializeVault(allocation);
      const config = await programAccounts.reserveVaultConfig.fetch(
        initialized.config
      );
      const approvedCap = new anchor.BN(allocation.approvedCapPex).mul(
        new anchor.BN(BASE_UNITS)
      );
      expect(config.allocationCap.toString()).to.equal(approvedCap.toString());
    }"""
if test.count(old_loop) != 1:
    raise SystemExit("13-vault initialization loop changed unexpectedly")
test = test.replace(old_loop, new_loop)

old_deposit_test = """  it("rejects an authorized deposit above the configured cap", async () => {
    const community = vaults.get("community_utility_rewards")!;
    await expectFailure(() =>
      program.methods
        .depositIntoReserveVault(community.allocationId, new anchor.BN(1))
        .accounts({
          state,
          reserveVaultConfig: community.config,
          vaultAuthority: community.authority,
          sourceOwner: community.sourceOwner.publicKey,
          sourceTokenAccount: community.sourceTokenAccount,
          vaultTokenAccount: community.tokenAccount,
          tokenMint: mint,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .signers([community.sourceOwner])
        .rpc()
    );
  });"""
new_deposit_test = """  it("rejects an authorized deposit above the full approved allocation cap", async () => {
    const treasury = vaults.get("treasury")!;
    const excessiveAmount = 120_000_001n * BigInt(BASE_UNITS);
    await mintTo(
      provider.connection,
      payer,
      mint,
      treasury.sourceTokenAccount,
      payer,
      excessiveAmount
    );
    await expectFailure(() =>
      program.methods
        .depositIntoReserveVault(
          treasury.allocationId,
          new anchor.BN(excessiveAmount.toString())
        )
        .accounts({
          state,
          reserveVaultConfig: treasury.config,
          vaultAuthority: treasury.authority,
          sourceOwner: treasury.sourceOwner.publicKey,
          sourceTokenAccount: treasury.sourceTokenAccount,
          vaultTokenAccount: treasury.tokenAccount,
          tokenMint: mint,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .signers([treasury.sourceOwner])
        .rpc()
    );
  });"""
if test.count(old_deposit_test) != 1:
    raise SystemExit("Old cap deposit test changed unexpectedly")
test = test.replace(old_deposit_test, new_deposit_test)

old_route_marker = '  it("disables the old approval-only release route", async () => {'
emergency_test = """  it("successfully releases from the emergency vault without counting unsolicited PEX", async () => {
    const emergency = vaults.get("team_emergency_reserve")!;
    await mintTo(
      provider.connection,
      payer,
      mint,
      emergency.sourceTokenAccount,
      payer,
      1_000n * BigInt(BASE_UNITS)
    );
    await program.methods
      .depositIntoReserveVault(
        emergency.allocationId,
        new anchor.BN(1_000 * BASE_UNITS)
      )
      .accounts({
        state,
        reserveVaultConfig: emergency.config,
        vaultAuthority: emergency.authority,
        sourceOwner: emergency.sourceOwner.publicKey,
        sourceTokenAccount: emergency.sourceTokenAccount,
        vaultTokenAccount: emergency.tokenAccount,
        tokenMint: mint,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([emergency.sourceOwner])
      .rpc();

    await transfer(
      provider.connection,
      payer,
      wrongSourceTokenAccount,
      emergency.tokenAccount,
      wrongSourceOwner,
      50n * BigInt(BASE_UNITS)
    );
    await program.methods
      .reconcileReserveVault(emergency.allocationId)
      .accounts({
        state,
        authority,
        reserveVaultConfig: emergency.config,
        vaultTokenAccount: emergency.tokenAccount,
      })
      .rpc();

    const params = emergencyRelease(
      "team_emergency_reserve",
      12,
      emergency.destinationTokenAccount,
      1_000 * BASE_UNITS,
      5 * BASE_UNITS
    );
    await executeRelease("team_emergency_reserve", params);

    const vault = await getAccount(provider.connection, emergency.tokenAccount);
    const destination = await getAccount(
      provider.connection,
      emergency.destinationTokenAccount
    );
    const config = await programAccounts.reserveVaultConfig.fetch(
      emergency.config
    );
    const [recordPda] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("vault-release"), Buffer.from(params.releaseId)],
      program.programId
    );
    const record = await programAccounts.reserveReleaseRecord.fetch(recordPda);

    expect(vault.amount).to.equal(1_045n * BigInt(BASE_UNITS));
    expect(destination.amount).to.equal(5n * BigInt(BASE_UNITS));
    expect(config.authorizedDeposited.toString()).to.equal(
      String(1_000 * BASE_UNITS)
    );
    expect(config.unsolicitedBalance.toString()).to.equal(
      String(50 * BASE_UNITS)
    );
    expect(config.totalReleased.toString()).to.equal(
      String(5 * BASE_UNITS)
    );
    expect(record.destinationTokenAccount.toBase58()).to.equal(
      emergency.destinationTokenAccount.toBase58()
    );
    expect(record.requestedAmount.toString()).to.equal(
      String(5 * BASE_UNITS)
    );

    await expectFailure(() =>
      executeRelease(
        "team_emergency_reserve",
        emergencyRelease(
          "team_emergency_reserve",
          13,
          emergency.destinationTokenAccount,
          1_045 * BASE_UNITS,
          1
        )
      )
    );
  });

"""
if test.count(old_route_marker) != 1:
    raise SystemExit("Emergency-test insertion marker changed")
test = test.replace(old_route_marker, emergency_test + old_route_marker)

test_path.write_text(test)
