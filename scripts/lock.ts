import * as anchor from "@coral-xyz/anchor";

const provider = anchor.AnchorProvider.env();
anchor.setProvider(provider);

const program = anchor.workspace.CollateralVault;

(async () => {
  const user = provider.wallet.publicKey;

  const [vaultPda] =
    anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("vault"), user.toBuffer()],
      program.programId
    );

  const [vaultAuthorityPda] =
    anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("vault_authority")],
      program.programId
    );

  await program.methods
    .demoLock(new anchor.BN(500))
    .accounts({
      callerProgram: program.programId,
      vaultAuthority: vaultAuthorityPda,
      vault: vaultPda,
    })
    .rpc();

  console.log("🔒 Collateral locked");
})();
