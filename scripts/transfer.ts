import * as anchor from "@coral-xyz/anchor";

const provider = anchor.AnchorProvider.env();
anchor.setProvider(provider);

const program = anchor.workspace.CollateralVault;

(async () => {
  const fromUser = provider.wallet.publicKey;

  // ⚠️ CHANGE THIS to another wallet pubkey
  const toUser = new anchor.web3.PublicKey(
    "zUjYEaLxc1S9bp3jU47vyc3amTqnB1s6pjZfjNKMkYa"
  );

  const [fromVault] =
    anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("vault"), fromUser.toBuffer()],
      program.programId
    );

  const [toVault] =
    anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("vault"), toUser.toBuffer()],
      program.programId
    );

  const [vaultAuthority] =
    anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("vault_authority")],
      program.programId
    );

  await program.methods
    .demoTransferCollateral(new anchor.BN(300))
    .accounts({
      callerProgram: program.programId,
      vaultAuthority,
      fromVault,
      toVault,
    })
    .rpc();

  console.log("🔁 Collateral transferred");
})();
