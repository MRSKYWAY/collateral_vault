import * as anchor from "@coral-xyz/anchor";
import { PublicKey } from "@solana/web3.js";
import { getAssociatedTokenAddressSync, TOKEN_PROGRAM_ID } from "@solana/spl-token";
import { COLLATERAL_MINT } from "./config";

const provider = anchor.AnchorProvider.env();
anchor.setProvider(provider);

const program = anchor.workspace.CollateralVault;

(async () => {
  const user = provider.wallet.publicKey;
  const mint = new PublicKey(COLLATERAL_MINT);

  const [vaultPda] = PublicKey.findProgramAddressSync(
    [Buffer.from("vault"), user.toBuffer()],
    program.programId
  );

  const userAta = getAssociatedTokenAddressSync(
    mint,
    user
  );

  const vaultAta = getAssociatedTokenAddressSync(
    mint,
    vaultPda,
    true
  );

  await program.methods
    .withdraw(new anchor.BN(1_000))
    .accounts({
      user,
      vault: vaultPda,
      vaultTokenAccount: vaultAta,
      userTokenAccount: userAta,
      tokenProgram: TOKEN_PROGRAM_ID,
    })
    .rpc();

  console.log("✅ Withdraw successful");
})();
