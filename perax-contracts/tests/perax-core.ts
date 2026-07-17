import * as anchor from "@coral-xyz/anchor";
import { expect } from "chai";
import {
  ASSOCIATED_TOKEN_PROGRAM_ID,
  TOKEN_PROGRAM_ID,
  createMint,
  getAccount,
  getAssociatedTokenAddressSync,
  getOrCreateAssociatedTokenAccount,
  mintTo,
  transfer,
} from "@solana/spl-token";

const BASE_UNITS = 1_000_000;
const DEFAULT_PUBLIC_KEY = new anchor.web3.PublicKey(
  "11111111111111111111111111111111"
);

function fixedId(label: string): number[] {
  const id = Buffer.alloc(32);
  Buffer.from(label, "utf8").copy(id);
  return Array.from(id);
}

function uniqueId(value: number): number[] {
  const id = Buffer.alloc(32);
  id.writeUInt32LE(value, 28);
  return Array.from(id);
}

type AllocationDefinition = {
  key: string;
  vaultClass: Record<string, Record<string, never>>;
  releasable: boolean;
  approvedCapPex: number;
};

type InitializationOverrides = Partial<{
  allocationId: number[];
  vaultClass: Record<string, Record<string, never>>;
  allocationCap: anchor.BN;
  authorizedSourceOwner: anchor.web3.PublicKey;
  authorizedSourceTokenAccount: anchor.web3.PublicKey;
  approvedDestinationOwner: anchor.web3.PublicKey;
  approvedDestinationTokenAccount: anchor.web3.PublicKey;
}>;

type ReserveVaultConfigAccount = {
  allocationCap: anchor.BN;
  authorizedDeposited: anchor.BN;
  unsolicitedBalance: anchor.BN;
  totalReleased: anchor.BN;
};

type ReserveReleaseRecordAccount = {
  destinationTokenAccount: anchor.web3.PublicKey;
  requestedAmount: anchor.BN;
};

const ALLOCATIONS: AllocationDefinition[] = [
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
];

describe("perax-core reserve vault custody", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  // Anchor workspace clients are generated dynamically from the IDL at runtime.
  // Keeping this boundary dynamic avoids TypeScript recursively expanding the full IDL.
  const program: any = anchor.workspace.PeraxCore;
  const programAccounts = program.account as unknown as {
    reserveVaultConfig: {
      all(): Promise<Array<{ account: ReserveVaultConfigAccount }>>;
      fetch(address: anchor.web3.PublicKey): Promise<ReserveVaultConfigAccount>;
    };
    reserveReleaseRecord: {
      fetch(address: anchor.web3.PublicKey): Promise<ReserveReleaseRecordAccount>;
    };
  };
  const payer = (provider.wallet as anchor.Wallet).payer;

  const authority = provider.wallet.publicKey;
  const safetyAdmin = anchor.web3.Keypair.generate();
  const oracle = anchor.web3.Keypair.generate();
  const outsider = anchor.web3.Keypair.generate();

  const [state] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("perax-state")],
    program.programId
  );

  let mint: anchor.web3.PublicKey;
  let wrongMint: anchor.web3.PublicKey;
  const vaults = new Map<
    string,
    {
      allocationId: number[];
      config: anchor.web3.PublicKey;
      authority: anchor.web3.PublicKey;
      tokenAccount: anchor.web3.PublicKey;
      sourceOwner: anchor.web3.Keypair;
      sourceTokenAccount: anchor.web3.PublicKey;
      destinationOwner: anchor.web3.Keypair | null;
      destinationTokenAccount: anchor.web3.PublicKey;
    }
  >();

  let wrongSourceOwner: anchor.web3.Keypair;
  let wrongSourceTokenAccount: anchor.web3.PublicKey;
  let otherDestinationTokenAccount: anchor.web3.PublicKey;
  let wrongMintDestination: anchor.web3.PublicKey;

  async function expectFailure(action: () => Promise<unknown>) {
    let failed = false;
    try {
      await action();
    } catch {
      failed = true;
    }
    expect(failed).to.equal(true);
  }

  function deriveVault(allocationId: number[]) {
    const [config] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("reserve-config"), Buffer.from(allocationId)],
      program.programId
    );
    const [vaultAuthority] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("reserve-authority"), Buffer.from(allocationId)],
      program.programId
    );
    const tokenAccount = getAssociatedTokenAddressSync(
      mint,
      vaultAuthority,
      true,
      TOKEN_PROGRAM_ID,
      ASSOCIATED_TOKEN_PROGRAM_ID
    );
    return { config, vaultAuthority, tokenAccount };
  }

  async function buildInitialization(
    allocation: AllocationDefinition,
    overrides: InitializationOverrides = {}
  ) {
    const allocationId = overrides.allocationId ?? fixedId(allocation.key);
    const sourceOwner = anchor.web3.Keypair.generate();
    const sourceTokenAccount = (
      await getOrCreateAssociatedTokenAccount(
        provider.connection,
        payer,
        mint,
        sourceOwner.publicKey
      )
    ).address;

    let destinationOwner: anchor.web3.Keypair | null = null;
    let destinationTokenAccount = DEFAULT_PUBLIC_KEY;
    if (allocation.releasable) {
      destinationOwner = anchor.web3.Keypair.generate();
      destinationTokenAccount = (
        await getOrCreateAssociatedTokenAccount(
          provider.connection,
          payer,
          mint,
          destinationOwner.publicKey
        )
      ).address;
    }

    const { config, vaultAuthority, tokenAccount } = deriveVault(allocationId);
    const params = {
      allocationId,
      vaultClass: overrides.vaultClass ?? allocation.vaultClass,
      allocationCap:
        overrides.allocationCap ??
        new anchor.BN(allocation.approvedCapPex).mul(new anchor.BN(BASE_UNITS)),
      authorizedSourceOwner:
        overrides.authorizedSourceOwner ?? sourceOwner.publicKey,
      authorizedSourceTokenAccount:
        overrides.authorizedSourceTokenAccount ?? sourceTokenAccount,
      approvedDestinationOwner:
        overrides.approvedDestinationOwner ??
        destinationOwner?.publicKey ??
        DEFAULT_PUBLIC_KEY,
      approvedDestinationTokenAccount:
        overrides.approvedDestinationTokenAccount ?? destinationTokenAccount,
    };

    return {
      params,
      config,
      vaultAuthority,
      tokenAccount,
      sourceOwner,
      sourceTokenAccount,
      destinationOwner,
      destinationTokenAccount,
    };
  }

  async function initializeVault(
    allocation: AllocationDefinition,
    overrides: InitializationOverrides = {}
  ) {
    const built = await buildInitialization(allocation, overrides);
    await program.methods
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
      .rpc();

    vaults.set(allocation.key, {
      allocationId: built.params.allocationId,
      config: built.config,
      authority: built.vaultAuthority,
      tokenAccount: built.tokenAccount,
      sourceOwner: built.sourceOwner,
      sourceTokenAccount: built.sourceTokenAccount,
      destinationOwner: built.destinationOwner,
      destinationTokenAccount: built.destinationTokenAccount,
    });
    return built;
  }

  function growthRelease(
    allocationKey: string,
    releaseNumber: number,
    destination: anchor.web3.PublicKey,
    amount = 1
  ) {
    const vault = vaults.get(allocationKey);
    if (!vault) throw new Error(`Vault not initialized: ${allocationKey}`);
    return {
      allocationId: vault.allocationId,
      releaseType: { growth: {} },
      requestedAmount: new anchor.BN(amount),
      releaseId: uniqueId(releaseNumber),
      marketObservationId: uniqueId(releaseNumber + 10_000),
      destinationTokenAccount: destination,
      snapshot: {
        observedPrice: new anchor.BN(3_600),
        twapMinutes: new anchor.BN(60),
        liquidityUsd: new anchor.BN(13_680),
        netBuyVolumeBps: 5_000,
        downsideMoveBps: 0,
        liquidityDrainBps: 0,
        emergencyReserveAvailableAmount: new anchor.BN(0),
        observedAt: new anchor.BN(1_800_000_000 + releaseNumber * 86_401),
      },
    };
  }

  function emergencyRelease(
    allocationKey: string,
    releaseNumber: number,
    destination: anchor.web3.PublicKey,
    availableAmount: number,
    amount = 1
  ) {
    const params = growthRelease(
      allocationKey,
      releaseNumber,
      destination,
      amount
    );
    return {
      ...params,
      releaseType: { emergency: {} },
      snapshot: {
        ...params.snapshot,
        downsideMoveBps: 3_000,
        liquidityDrainBps: 6_000,
        emergencyReserveAvailableAmount: new anchor.BN(availableAmount),
      },
    };
  }

  async function releaseInstruction(
    allocationKey: string,
    params: any
  ) {
    const vault = vaults.get(allocationKey)!;
    const [releaseRecord] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("vault-release"), Buffer.from(params.releaseId)],
      program.programId
    );
    return program.methods
      .executeMarketConditionalRelease(params)
      .accounts({
        state,
        reserveVaultConfig: vault.config,
        vaultAuthority: vault.authority,
        vaultTokenAccount: vault.tokenAccount,
        destinationTokenAccount: params.destinationTokenAccount,
        releaseRecord,
        oracleFeed: oracle.publicKey,
        tokenMint: mint,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .instruction();
  }

  async function executeRelease(
    allocationKey: string,
    params: any
  ) {
    const instruction = await releaseInstruction(allocationKey, params);
    const transaction = new anchor.web3.Transaction().add(instruction);
    return provider.sendAndConfirm(transaction, [oracle]);
  }

  before(async () => {
    const airdrop = await provider.connection.requestAirdrop(
      oracle.publicKey,
      5 * anchor.web3.LAMPORTS_PER_SOL
    );
    await provider.connection.confirmTransaction(airdrop, "confirmed");

    mint = await createMint(provider.connection, payer, payer.publicKey, null, 6);
    wrongMint = await createMint(
      provider.connection,
      payer,
      payer.publicKey,
      null,
      6
    );

    wrongSourceOwner = anchor.web3.Keypair.generate();
    wrongSourceTokenAccount = (
      await getOrCreateAssociatedTokenAccount(
        provider.connection,
        payer,
        mint,
        wrongSourceOwner.publicKey
      )
    ).address;
    otherDestinationTokenAccount = (
      await getOrCreateAssociatedTokenAccount(
        provider.connection,
        payer,
        mint,
        anchor.web3.Keypair.generate().publicKey
      )
    ).address;
    wrongMintDestination = (
      await getOrCreateAssociatedTokenAccount(
        provider.connection,
        payer,
        wrongMint,
        anchor.web3.Keypair.generate().publicKey
      )
    ).address;

    await mintTo(
      provider.connection,
      payer,
      mint,
      wrongSourceTokenAccount,
      payer,
      500n * BigInt(BASE_UNITS)
    );

    await program.methods
      .initialize({
        tokenMint: mint,
        tradingCompanyTokenAccount: anchor.web3.Keypair.generate().publicKey,
        tradingCompanyRevenueTokenAccount:
          anchor.web3.Keypair.generate().publicKey,
        maxPaymentAmount: new anchor.BN(0),
        safetyAdmin: safetyAdmin.publicKey,
        oracleFeed: oracle.publicKey,
        launchPrice: new anchor.BN(1_200),
        currentSteppedFloor: new anchor.BN(1_200),
        dailyReleaseCap: new anchor.BN("10000000000000"),
        monthlyReleaseCap: new anchor.BN("150000000000000"),
        emergencyHourlyReleaseBps: 50,
      })
      .accounts({
        state,
        authority,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();
  });

  it("rejects an unknown allocation ID", async () => {
    const definition: AllocationDefinition = {
      key: "unknown_allocation",
      vaultClass: { marketReserve: {} },
      releasable: true,
      approvedCapPex: 1,
    };
    const built = await buildInitialization(definition);
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

  it("rejects a wrong vault class", async () => {
    const treasury = ALLOCATIONS.find((item) => item.key === "treasury")!;
    const built = await buildInitialization(treasury, {
      vaultClass: { vesting: {} },
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

  it("rejects an allocation cap above the approved maximum", async () => {
    const treasury = ALLOCATIONS.find((item) => item.key === "treasury")!;
    const built = await buildInitialization(treasury, {
      allocationCap: new anchor.BN(121_000_000 * BASE_UNITS),
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

  it("rejects an allocation cap below the approved amount", async () => {
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

  it("rejects a configured destination owned by any reserve-authority PDA", async () => {
    const treasury = ALLOCATIONS.find((item) => item.key === "treasury")!;
    const liquidityAuthority = deriveVault(fixedId("liquidity_pool")).vaultAuthority;
    const crossVaultDestination = (
      await getOrCreateAssociatedTokenAccount(
        provider.connection,
        payer,
        mint,
        liquidityAuthority,
        true
      )
    ).address;
    const built = await buildInitialization(treasury, {
      approvedDestinationOwner: liquidityAuthority,
      approvedDestinationTokenAccount: crossVaultDestination,
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

  it("initializes all 13 approved allocation vaults with separate identities", async () => {
    for (const allocation of ALLOCATIONS) {
      const initialized = await initializeVault(allocation);
      const config = await programAccounts.reserveVaultConfig.fetch(
        initialized.config
      );
      const approvedCap = new anchor.BN(allocation.approvedCapPex).mul(
        new anchor.BN(BASE_UNITS)
      );
      expect(config.allocationCap.toString()).to.equal(approvedCap.toString());
    }

    const configs = await programAccounts.reserveVaultConfig.all();
    expect(configs.length).to.equal(13);

    const community = vaults.get("community_utility_rewards")!;
    await mintTo(
      provider.connection,
      payer,
      mint,
      community.sourceTokenAccount,
      payer,
      1_000n * BigInt(BASE_UNITS)
    );
  });

  it("rejects a deposit from an unauthorized source account", async () => {
    const community = vaults.get("community_utility_rewards")!;
    await expectFailure(() =>
      program.methods
        .depositIntoReserveVault(
          community.allocationId,
          new anchor.BN(10 * BASE_UNITS)
        )
        .accounts({
          state,
          reserveVaultConfig: community.config,
          vaultAuthority: community.authority,
          sourceOwner: wrongSourceOwner.publicKey,
          sourceTokenAccount: wrongSourceTokenAccount,
          vaultTokenAccount: community.tokenAccount,
          tokenMint: mint,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .signers([wrongSourceOwner])
        .rpc()
    );
  });

  it("deposits exactly 1,000 authorized PEX into the community vault", async () => {
    const community = vaults.get("community_utility_rewards")!;
    await program.methods
      .depositIntoReserveVault(
        community.allocationId,
        new anchor.BN(1_000 * BASE_UNITS)
      )
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
      .rpc();

    const vault = await getAccount(provider.connection, community.tokenAccount);
    const config = await programAccounts.reserveVaultConfig.fetch(
      community.config
    );
    expect(vault.owner.toBase58()).to.equal(community.authority.toBase58());
    expect(vault.amount).to.equal(1_000n * BigInt(BASE_UNITS));
    expect(config.authorizedDeposited.toString()).to.equal(
      String(1_000 * BASE_UNITS)
    );
    expect(config.unsolicitedBalance.toString()).to.equal("0");
  });

  it("rejects an authorized deposit above the full approved allocation cap", async () => {
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
  });

  it("records unsolicited direct transfers separately without increasing allocation capacity", async () => {
    const community = vaults.get("community_utility_rewards")!;
    await transfer(
      provider.connection,
      payer,
      wrongSourceTokenAccount,
      community.tokenAccount,
      wrongSourceOwner,
      50n * BigInt(BASE_UNITS)
    );

    await program.methods
      .reconcileReserveVault(community.allocationId)
      .accounts({
        state,
        authority,
        reserveVaultConfig: community.config,
        vaultTokenAccount: community.tokenAccount,
      })
      .rpc();

    const config = await programAccounts.reserveVaultConfig.fetch(
      community.config
    );
    expect(config.authorizedDeposited.toString()).to.equal(
      String(1_000 * BASE_UNITS)
    );
    expect(config.unsolicitedBalance.toString()).to.equal(
      String(50 * BASE_UNITS)
    );
  });

  it("rejects an ordinary-wallet withdrawal", async () => {
    const community = vaults.get("community_utility_rewards")!;
    await expectFailure(() =>
      transfer(
        provider.connection,
        payer,
        community.tokenAccount,
        community.destinationTokenAccount,
        payer,
        1n
      )
    );
  });

  it("proves full transaction rollback when a later instruction fails", async () => {
    const community = vaults.get("community_utility_rewards")!;
    const beforeVault = await getAccount(
      provider.connection,
      community.tokenAccount
    );
    const beforeDestination = await getAccount(
      provider.connection,
      community.destinationTokenAccount
    );
    const params = growthRelease(
      "community_utility_rewards",
      40,
      community.destinationTokenAccount,
      10 * BASE_UNITS
    );
    const first = await releaseInstruction(
      "community_utility_rewards",
      params
    );
    const second = await releaseInstruction(
      "community_utility_rewards",
      params
    );
    const transaction = new anchor.web3.Transaction().add(first, second);

    await expectFailure(() => provider.sendAndConfirm(transaction, [oracle]));

    const afterVault = await getAccount(
      provider.connection,
      community.tokenAccount
    );
    const afterDestination = await getAccount(
      provider.connection,
      community.destinationTokenAccount
    );
    expect(afterVault.amount).to.equal(beforeVault.amount);
    expect(afterDestination.amount).to.equal(beforeDestination.amount);
  });

  it("atomically releases 100 PEX only to the configured destination", async () => {
    const community = vaults.get("community_utility_rewards")!;
    const params = growthRelease(
      "community_utility_rewards",
      1,
      community.destinationTokenAccount,
      100 * BASE_UNITS
    );
    await executeRelease("community_utility_rewards", params);

    const vault = await getAccount(provider.connection, community.tokenAccount);
    const destination = await getAccount(
      provider.connection,
      community.destinationTokenAccount
    );
    const [recordPda] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("vault-release"), Buffer.from(params.releaseId)],
      program.programId
    );
    const record = await programAccounts.reserveReleaseRecord.fetch(recordPda);
    const config = await programAccounts.reserveVaultConfig.fetch(
      community.config
    );

    expect(vault.amount).to.equal(950n * BigInt(BASE_UNITS));
    expect(destination.amount).to.equal(100n * BigInt(BASE_UNITS));
    expect(record.destinationTokenAccount.toBase58()).to.equal(
      community.destinationTokenAccount.toBase58()
    );
    expect(config.authorizedDeposited.toString()).to.equal(
      String(1_000 * BASE_UNITS)
    );
    expect(config.unsolicitedBalance.toString()).to.equal(
      String(50 * BASE_UNITS)
    );
    expect(config.totalReleased.toString()).to.equal(
      String(100 * BASE_UNITS)
    );
  });

  it("rejects a replayed release ID", async () => {
    const community = vaults.get("community_utility_rewards")!;
    await expectFailure(() =>
      executeRelease(
        "community_utility_rewards",
        growthRelease(
          "community_utility_rewards",
          1,
          community.destinationTokenAccount,
          1
        )
      )
    );
  });

  it("rejects an unapproved ordinary PEX destination", async () => {
    await expectFailure(() =>
      executeRelease(
        "community_utility_rewards",
        growthRelease(
          "community_utility_rewards",
          2,
          otherDestinationTokenAccount
        )
      )
    );
  });

  it("rejects a destination using the wrong mint", async () => {
    await expectFailure(() =>
      executeRelease(
        "community_utility_rewards",
        growthRelease(
          "community_utility_rewards",
          3,
          wrongMintDestination
        )
      )
    );
  });

  it("rejects a destination pointing to another reserve vault", async () => {
    const treasury = vaults.get("treasury")!;
    await expectFailure(() =>
      executeRelease(
        "community_utility_rewards",
        growthRelease(
          "community_utility_rewards",
          4,
          treasury.tokenAccount
        )
      )
    );
  });

  it("rejects unauthorized pause attempts and accepts the safety admin", async () => {
    const community = vaults.get("community_utility_rewards")!;
    await expectFailure(() =>
      program.methods
        .setReserveVaultPause(community.allocationId, true)
        .accounts({
          state,
          reserveVaultConfig: community.config,
          actor: outsider.publicKey,
        })
        .signers([outsider])
        .rpc()
    );

    await program.methods
      .setReserveVaultPause(community.allocationId, true)
      .accounts({
        state,
        reserveVaultConfig: community.config,
        actor: safetyAdmin.publicKey,
      })
      .signers([safetyAdmin])
      .rpc();

    await expectFailure(() =>
      executeRelease(
        "community_utility_rewards",
        growthRelease(
          "community_utility_rewards",
          5,
          community.destinationTokenAccount
        )
      )
    );

    await program.methods
      .setReserveVaultPause(community.allocationId, false)
      .accounts({
        state,
        reserveVaultConfig: community.config,
        actor: safetyAdmin.publicKey,
      })
      .signers([safetyAdmin])
      .rpc();
  });

  it("rejects release above authorized remaining balance", async () => {
    const community = vaults.get("community_utility_rewards")!;
    await expectFailure(() =>
      executeRelease(
        "community_utility_rewards",
        growthRelease(
          "community_utility_rewards",
          6,
          community.destinationTokenAccount,
          901 * BASE_UNITS
        )
      )
    );
  });

  it("rejects liquidity and vesting vault market releases", async () => {
    const liquidity = vaults.get("liquidity_pool")!;
    const vesting = vaults.get("development_team")!;
    await expectFailure(() =>
      executeRelease(
        "liquidity_pool",
        growthRelease(
          "liquidity_pool",
          7,
          otherDestinationTokenAccount
        )
      )
    );
    await expectFailure(() =>
      executeRelease(
        "development_team",
        growthRelease(
          "development_team",
          8,
          otherDestinationTokenAccount
        )
      )
    );
    expect(liquidity.destinationTokenAccount.equals(DEFAULT_PUBLIC_KEY)).to.equal(
      true
    );
    expect(vesting.destinationTokenAccount.equals(DEFAULT_PUBLIC_KEY)).to.equal(
      true
    );
  });

  it("rejects emergency release from a growth vault and growth release from the emergency vault", async () => {
    const treasury = vaults.get("treasury")!;
    const emergency = vaults.get("team_emergency_reserve")!;

    await expectFailure(() =>
      executeRelease(
        "treasury",
        emergencyRelease(
          "treasury",
          9,
          treasury.destinationTokenAccount,
          1_000,
          1
        )
      )
    );
    await expectFailure(() =>
      executeRelease(
        "team_emergency_reserve",
        growthRelease(
          "team_emergency_reserve",
          10,
          emergency.destinationTokenAccount,
          1
        )
      )
    );
  });

  it("successfully releases from the emergency vault without counting unsolicited PEX", async () => {
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

  it("disables the old approval-only release route", async () => {
    const community = vaults.get("community_utility_rewards")!;
    const releaseId = uniqueId(11);
    const [releaseRecord] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("release"), Buffer.from(releaseId)],
      program.programId
    );
    await expectFailure(() =>
      program.methods
        .recordMarketConditionalRelease({
          releaseType: { growth: {} },
          requestedAmount: new anchor.BN(1),
          releaseId,
          snapshot: growthRelease(
            "community_utility_rewards",
            11,
            community.destinationTokenAccount
          ).snapshot,
        })
        .accounts({
          state,
          releaseRecord,
          oracleFeed: oracle.publicKey,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .signers([oracle])
        .rpc()
    );
  });
});
