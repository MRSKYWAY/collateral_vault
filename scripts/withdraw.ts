import * as anchor from "@coral-xyz/anchor";
import { TOKEN_PROGRAM_ID } from "@solana/spl-token";
import state from "state.json";

const provider = anchor.AnchorProvider.env();
anchor.setProvider(provider);

const program = anchor.workspace.CollateralVault;

(async () => {
  const vault = new anchor.web3.PublicKey(state.vaultPda);

  await program.methods
    .withdraw(new anchor.BN(100))
    .accounts({
      user: provider.wallet.publicKey,
      vault,
      vaultTokenAccount: new anchor.web3.PublicKey(state.vaultTokenAccount),
      userTokenAccount: new anchor.web3.PublicKey(state.userTokenAccount),
      tokenProgram: TOKEN_PROGRAM_ID,
    })
    .rpc();

  console.log("Withdrew 100 tokens");
})();
