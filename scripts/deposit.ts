import * as anchor from "@coral-xyz/anchor";
import { getAssociatedTokenAddressSync, TOKEN_PROGRAM_ID } from "@solana/spl-token";

const provider = anchor.AnchorProvider.env();
anchor.setProvider(provider);

const program = anchor.workspace.CollateralVault;

(async () => {
  const mint = new anchor.web3.PublicKey(process.env.MINT!);

  const [vault] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("vault"), provider.wallet.publicKey.toBuffer()],
    program.programId
  );

  const userTokenAccount = getAssociatedTokenAddressSync(
    mint,
    provider.wallet.publicKey
  );

  const vaultTokenAccount = getAssociatedTokenAddressSync(
    mint,
    vault,
    true
  );

  await program.methods
    .deposit(new anchor.BN(200))
    .accounts({
      user: provider.wallet.publicKey,
      vault,
      userTokenAccount,
      vaultTokenAccount,
      tokenProgram: TOKEN_PROGRAM_ID,
    })
    .rpc();

  console.log("Deposited 200 tokens");
})();
