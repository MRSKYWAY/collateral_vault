import * as anchor from "@coral-xyz/anchor";

const provider = anchor.AnchorProvider.env();
anchor.setProvider(provider);

const program = anchor.workspace.CollateralVault;

(async () => {
  const [vaultAuthorityPda] =
    anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("vault_authority")],
      program.programId
    );

  await program.methods
    .initializeVaultAuthority([
      program.programId, // authorize THIS program
    ])
    .accounts({
      admin: provider.wallet.publicKey,
      vaultAuthority: vaultAuthorityPda,
      systemProgram: anchor.web3.SystemProgram.programId,
    })
    .rpc();

  console.log("✅ VaultAuthority initialized");
})();
