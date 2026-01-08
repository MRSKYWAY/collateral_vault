import * as anchor from "@coral-xyz/anchor";
import { PublicKey } from "@solana/web3.js";

const provider = anchor.AnchorProvider.env();
anchor.setProvider(provider);
const program = anchor.workspace.CollateralVault;

(async () => {
  const [vault] = PublicKey.findProgramAddressSync(
    [Buffer.from("vault"), provider.wallet.publicKey.toBuffer()],
    program.programId
  );

  const vaultTokenAccount = await anchor.utils.token.associatedAddress({
    mint: new PublicKey("<MINT>"),
    owner: vault,
  });

  await program.methods
    .initializeVault()
    .accounts({
      user: provider.wallet.publicKey,
      vault,
      vaultTokenAccount,
    })
    .rpc();

  console.log("Vault initialized:", vault.toBase58());
})();
