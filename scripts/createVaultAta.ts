import * as anchor from "@coral-xyz/anchor";
import { getAssociatedTokenAddressSync, createAssociatedTokenAccountInstruction } from "@solana/spl-token";

const provider = anchor.AnchorProvider.env();
anchor.setProvider(provider);

const program = anchor.workspace.CollateralVault;

(async () => {
  const mint = new anchor.web3.PublicKey(process.env.MINT!);

  const [vault] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("vault"), provider.wallet.publicKey.toBuffer()],
    program.programId
  );

  const vaultAta = getAssociatedTokenAddressSync(
    mint,
    vault,
    true // PDA
  );

  const ix = createAssociatedTokenAccountInstruction(
    provider.wallet.publicKey,
    vaultAta,
    vault,
    mint
  );

  const tx = new anchor.web3.Transaction().add(ix);
  await provider.sendAndConfirm(tx);

  console.log("Vault ATA:", vaultAta.toBase58());
})();
