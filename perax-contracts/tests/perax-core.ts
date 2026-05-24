import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { expect } from "chai";

describe("perax-core", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.PeraxCore as Program;

  it("initializes the Perax state account", async () => {
    const [state] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("perax-state")],
      program.programId
    );

    await program.methods.initialize().accounts({ state }).rpc();

    const account = await program.account.peraxState.fetch(state);
    expect(account.authority.toBase58()).to.equal(provider.wallet.publicKey.toBase58());
  });
});
