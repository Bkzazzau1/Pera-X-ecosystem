import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
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

describe("perax-core reserve vault custody", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const program = anchor.workspace.PeraxCore as Program<any>;
  const payer = (provider.wallet as anchor.Wallet).payer;

  const allocationId = fixedId("community_utility_rewards");
  const [state] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("perax-state")],
    program.programId
  );
  const [vaultConfig] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("reserve-config"), Buffer.from(allocationId)],
    program.programId
  );
  const [vaultAuthority] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("reserve-authority"), Buffer.from(allocationId)],
    program.programId
  );

  let mint: anchor.web3.PublicKey;
  let wrongMint: anchor.web3.PublicKey;
  let sourceTokenAccount: anchor.web3.PublicKey;
  let vaultTokenAccount: anchor.web3.PublicKey;
  let destinationTokenAccount: anchor.web3.PublicKey;
  let otherDestinationTokenAccount: anchor.web3.PublicKey;
  let wrongMintDestination: anchor.web3.PublicKey;

  before(async () => {
    mint = await createMint(provider.connection, payer, payer.publicKey, null, 6);
    wrongMint = await createMint(provider.connection, payer, payer.publicKey, null, 6);

    sourceTokenAccount = (
      await getOrCreateAssociatedTokenAccount(
        provider.connection,
        payer,
        mint,
        payer.publicKey
      )
    ).address;
    destinationTokenAccount = (
      await getOrCreateAssociatedTokenAccount(
        provider.connection,
        payer,
        mint,
        anchor.web3.Keypair.generate().publicKey
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
      sourceTokenAccount,
      payer,
      1_000n * BigInt(BASE_UNITS)
    );

    await program.methods
      .initialize({
        tokenMint: mint,
        tradingCompanyTokenAccount: anchor.web3.Keypair.generate().publicKey,
        tradingCompanyRevenueTokenAccount: anchor.web3.Keypair.generate().publicKey,
        maxPaymentAmount: new anchor.BN(0),
        safetyAdmin: provider.wallet.publicKey,
        oracleFeed: provider.wallet.publicKey,
        launchPrice: new anchor.BN(1_200),
        currentSteppedFloor: new anchor.BN(1_200),
        dailyReleaseCap: new anchor.BN("10000000000000"),
        monthlyReleaseCap: new anchor.BN("150000000000000"),
        emergencyHourlyReleaseBps: 50,
      })
      .accounts({
        state,
        authority: provider.wallet.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();

    vaultTokenAccount = getAssociatedTokenAddressSync(
      mint,
      vaultAuthority,
      true,
      TOKEN_PROGRAM_ID,
      ASSOCIATED_TOKEN_PROGRAM_ID
    );

    await program.methods
      .initializeReserveVault(
        allocationId,
        { communityRewards: {} },
        new anchor.BN(1_000 * BASE_UNITS)
      )
      .accounts({
        state,
        authority: provider.wallet.publicKey,
        reserveVaultConfig: vaultConfig,
        vaultAuthority,
        vaultTokenAccount,
        tokenMint: mint,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();
  });

  function growthRelease(
    releaseNumber: number,
    destination: anchor.web3.PublicKey,
    amount = 1
  ) {
    return {
      allocationId,
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
        observedAt: new anchor.BN(Math.floor(Date.now() / 1000)),
      },
    };
  }

  async function expectFailure(action: () => Promise<unknown>) {
    let failed = false;
    try {
      await action();
    } catch {
      failed = true;
    }
    expect(failed).to.equal(true);
  }

  async function executeRelease(params: ReturnType<typeof growthRelease>) {
    const [releaseRecordV2] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("vault-release"), Buffer.from(params.releaseId)],
      program.programId
    );
    return program.methods
      .executeMarketConditionalRelease(params)
      .accounts({
        state,
        reserveVaultConfig: vaultConfig,
        vaultAuthority,
        vaultTokenAccount,
        destinationTokenAccount: params.destinationTokenAccount,
        releaseRecordV2,
        oracleFeed: provider.wallet.publicKey,
        tokenMint: mint,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();
  }

  it("deposits 1,000 PEX into a PDA-controlled vault", async () => {
    await program.methods
      .depositIntoReserveVault(allocationId, new anchor.BN(1_000 * BASE_UNITS))
      .accounts({
        state,
        reserveVaultConfig: vaultConfig,
        vaultAuthority,
        sourceOwner: provider.wallet.publicKey,
        sourceTokenAccount,
        vaultTokenAccount,
        tokenMint: mint,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .rpc();

    const vault = await getAccount(provider.connection, vaultTokenAccount);
    expect(vault.owner.toBase58()).to.equal(vaultAuthority.toBase58());
    expect(vault.amount).to.equal(1_000n * BigInt(BASE_UNITS));
  });

  it("rejects an ordinary-wallet withdrawal", async () => {
    await expectFailure(() =>
      transfer(
        provider.connection,
        payer,
        vaultTokenAccount,
        destinationTokenAccount,
        payer,
        1n
      )
    );
  });

  it("atomically releases 100 PEX and stores a permanent record", async () => {
    const params = growthRelease(1, destinationTokenAccount, 100 * BASE_UNITS);
    await executeRelease(params);

    const vault = await getAccount(provider.connection, vaultTokenAccount);
    const destination = await getAccount(provider.connection, destinationTokenAccount);
    const [recordPda] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("vault-release"), Buffer.from(params.releaseId)],
      program.programId
    );
    const record = await program.account.reserveReleaseRecord.fetch(recordPda);

    expect(vault.amount).to.equal(900n * BigInt(BASE_UNITS));
    expect(destination.amount).to.equal(100n * BigInt(BASE_UNITS));
    expect(record.destinationTokenAccount.toBase58()).to.equal(
      destinationTokenAccount.toBase58()
    );
  });

  it("rejects a replayed release ID", async () => {
    await expectFailure(() => executeRelease(growthRelease(1, destinationTokenAccount)));
  });

  it("rejects a destination different from the bot-signed destination", async () => {
    const params = growthRelease(2, destinationTokenAccount);
    const [releaseRecordV2] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("vault-release"), Buffer.from(params.releaseId)],
      program.programId
    );
    await expectFailure(() =>
      program.methods
        .executeMarketConditionalRelease(params)
        .accounts({
          state,
          reserveVaultConfig: vaultConfig,
          vaultAuthority,
          vaultTokenAccount,
          destinationTokenAccount: otherDestinationTokenAccount,
          releaseRecordV2,
          oracleFeed: provider.wallet.publicKey,
          tokenMint: mint,
          tokenProgram: TOKEN_PROGRAM_ID,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .rpc()
    );
  });

  it("rejects a destination using the wrong mint", async () => {
    await expectFailure(() => executeRelease(growthRelease(3, wrongMintDestination)));
  });

  it("rejects release while the vault is paused", async () => {
    await program.methods
      .setReserveVaultPause(allocationId, true)
      .accounts({ state, reserveVaultConfig: vaultConfig, actor: provider.wallet.publicKey })
      .rpc();

    await expectFailure(() => executeRelease(growthRelease(4, destinationTokenAccount)));

    await program.methods
      .setReserveVaultPause(allocationId, false)
      .accounts({ state, reserveVaultConfig: vaultConfig, actor: provider.wallet.publicKey })
      .rpc();
  });

  it("rejects release above the authoritative vault balance", async () => {
    await expectFailure(() =>
      executeRelease(growthRelease(5, destinationTokenAccount, 901 * BASE_UNITS))
    );
  });

  it("disables the old approval-only release route", async () => {
    const releaseId = uniqueId(6);
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
          snapshot: growthRelease(6, destinationTokenAccount).snapshot,
        })
        .accounts({
          state,
          releaseRecord,
          oracleFeed: provider.wallet.publicKey,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .rpc()
    );
  });
});
