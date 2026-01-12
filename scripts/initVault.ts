import * as anchor from "@coral-xyz/anchor";
import { getAssociatedTokenAddressSync } from "@solana/spl-token";

const provider = anchor.AnchorProvider.env();
anchor.setProvider(provider);

const program = anchor.workspace.CollateralVault;

(async () => {
  const mint = new anchor.web3.PublicKey(process.env.MINT!);
  console.log("Initializing Vault for mint:", mint.toBase58());
  const [vault] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("vault"), provider.wallet.publicKey.toBuffer()],
    program.programId
  );

  const vaultTokenAccount = getAssociatedTokenAddressSync(
    mint,
    vault,
    true
  );

  await program.methods
    .initializeVault()
    .accounts({
      user: provider.wallet.publicKey,
      vault,
      vaultTokenAccount,
    })
    .rpc();

  console.log("Vault initialized");
})();
