import * as anchor from "@anchor-lang/core";
import { Program } from "@anchor-lang/core";
import { CollateralVault } from "../target/types/collateral_vault";

describe("collateral_vault", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.collateralVault as Program<CollateralVault>;

  it("Is initialized!", async () => {
    // Add your test here.
    const tx = await program.methods.initialize().rpc();
    console.log("Your transaction signature", tx);
  });
});
