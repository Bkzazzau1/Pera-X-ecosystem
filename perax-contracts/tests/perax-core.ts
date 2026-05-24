import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { expect } from "chai";

describe("perax-core", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.PeraxCore as Program;

  it("initializes the Perax trading company payment configuration", async () => {
    const [state] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("perax-state")],
      program.programId
    );

    const tokenMint = anchor.web3.Keypair.generate().publicKey;
    const tradingCompanyTokenAccount = anchor.web3.Keypair.generate().publicKey;
    const maxPaymentAmount = new anchor.BN(1_000_000);

    await program.methods
      .initialize({
        tokenMint,
        tradingCompanyTokenAccount,
        maxPaymentAmount,
      })
      .accounts({
        state,
        authority: provider.wallet.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();

    const account = await (program.account as any).peraxState.fetch(state);

    expect(account.authority.toBase58()).to.equal(provider.wallet.publicKey.toBase58());
    expect(account.tokenMint.toBase58()).to.equal(tokenMint.toBase58());
    expect(account.tradingCompanyTokenAccount.toBase58()).to.equal(
      tradingCompanyTokenAccount.toBase58()
    );
    expect(account.maxPaymentAmount.toString()).to.equal(maxPaymentAmount.toString());
    expect(account.isPaused).to.equal(false);
  });
});
