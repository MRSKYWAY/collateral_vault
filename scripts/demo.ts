import * as anchor from "@coral-xyz/anchor";
import {
  createMint,
  getOrCreateAssociatedTokenAccount,
  mintTo,
  TOKEN_PROGRAM_ID,
  ASSOCIATED_TOKEN_PROGRAM_ID,
} from "@solana/spl-token";
import {
  Keypair,
  PublicKey,
  SystemProgram,
  LAMPORTS_PER_SOL,
} from "@solana/web3.js";

(async () => {
  console.log("🚀 Starting Collateral Vault Demo");

  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  // IMPORTANT: kill TS infinite generics
  const program = anchor.workspace.CollateralVault as any;

  /* -------------------------------------------------- */
  /* 1. Create mock USDT mint                            */
  /* -------------------------------------------------- */
  console.log("\n1️⃣ Creating mock USDT mint...");
  const mint = await createMint(
    provider.connection,
    provider.wallet.payer,
    provider.wallet.publicKey,
    null,
    6
  );
  console.log("Mint:", mint.toBase58());

  /* -------------------------------------------------- */
  /* 2. Create user & fund                              */
  /* -------------------------------------------------- */
  const user = Keypair.generate();
  await provider.connection.confirmTransaction(
    await provider.connection.requestAirdrop(
      user.publicKey,
      2 * LAMPORTS_PER_SOL
    )
  );
  console.log("User:", user.publicKey.toBase58());

  /* -------------------------------------------------- */
  /* 3. Create USER token account                       */
  /* -------------------------------------------------- */
  const userTokenAccount = await getOrCreateAssociatedTokenAccount(
    provider.connection,
    provider.wallet.payer,
    mint,
    user.publicKey
  );

  /* -------------------------------------------------- */
  /* 4. Derive PDAs                                     */
  /* -------------------------------------------------- */
  const [vaultPda] = PublicKey.findProgramAddressSync(
    [Buffer.from("vault"), user.publicKey.toBuffer()],
    program.programId
  );

  const [vaultAuthorityPda] = PublicKey.findProgramAddressSync(
    [Buffer.from("vault_authority")],
    program.programId
  );

  console.log("Vault PDA:", vaultPda.toBase58());
  console.log("Vault Authority PDA:", vaultAuthorityPda.toBase58());

  /* -------------------------------------------------- */
  /* 5. Create VAULT token account (PDA ATA)            */
  /* -------------------------------------------------- */
  const vaultTokenAccount = await getOrCreateAssociatedTokenAccount(
    provider.connection,
    provider.wallet.payer,
    mint,
    vaultPda,
    true // allowOwnerOffCurve — CRITICAL
  );

  console.log("Vault ATA:", vaultTokenAccount.address.toBase58());

  /* -------------------------------------------------- */
  /* 6. Initialize vault                                */
  /* -------------------------------------------------- */
  console.log("\n2️⃣ Initializing vault...");
  await program.methods
    .initializeVault()
    .accounts({
      user: user.publicKey,
      vault: vaultPda,
      vaultTokenAccount: vaultTokenAccount.address,
      tokenMint: mint,
      tokenProgram: TOKEN_PROGRAM_ID,
      associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
      systemProgram: SystemProgram.programId,
    } as any)
    .signers([user])
    .rpc();

  console.log("✅ Vault initialized");

  /* -------------------------------------------------- */
  /* 7. Mint + Deposit                                  */
  /* -------------------------------------------------- */
  console.log("\n3️⃣ Minting & depositing collateral...");

  await mintTo(
    provider.connection,
    provider.wallet.payer,
    mint,
    userTokenAccount.address,
    provider.wallet.publicKey,
    1_000
  );

  await program.methods
    .deposit(new anchor.BN(1_000))
    .accounts({
      user: user.publicKey,
      vault: vaultPda,
      mint: mint,
      userTokenAccount: userTokenAccount.address,
      vaultTokenAccount: vaultTokenAccount.address,
      tokenProgram: TOKEN_PROGRAM_ID,
    } as any)
    .signers([user])
    .rpc();

  console.log("✅ Deposit successful");

  /* -------------------------------------------------- */
  /* 8. Initialize Vault Authority                      */
  /* -------------------------------------------------- */
  console.log("\n4️⃣ Initializing vault authority...");

  const existing = await program.account.vaultAuthority.fetchNullable(
    vaultAuthorityPda
  );

  if (!existing) {
    await program.methods
      .initializeVaultAuthority([program.programId]) // authorize self
      .accounts({
        admin: provider.wallet.publicKey,
        vaultAuthority: vaultAuthorityPda,
        systemProgram: SystemProgram.programId,
      } as any)
      .rpc();

    console.log("✅ Vault authority initialized");
  } else {
    console.log("⚠️ Vault authority already exists");
  }

  /* -------------------------------------------------- */
  /* 9. Lock                                           */
  /* -------------------------------------------------- */
  console.log("\n5️⃣ Locking collateral...");
  await program.methods
    .lockCollateral(new anchor.BN(600))
    .accounts({
      callerProgram: program.programId,
      vaultAuthority: vaultAuthorityPda,
      vault: vaultPda,
    } as any)
    .rpc();

  console.log("✅ Locked");

  /* -------------------------------------------------- */
  /* 10. Unlock                                        */
  /* -------------------------------------------------- */
  console.log("\n6️⃣ Unlocking collateral...");
  await program.methods
    .unlockCollateral(new anchor.BN(600))
    .accounts({
      callerProgram: program.programId,
      vaultAuthority: vaultAuthorityPda,
      vault: vaultPda,
    } as any)
    .rpc();

  console.log("✅ Unlocked");

  /* -------------------------------------------------- */
  /* 11. Withdraw                                      */
  /* -------------------------------------------------- */
  console.log("\n7️⃣ Withdrawing collateral...");
  await program.methods
    .withdraw(new anchor.BN(1_000))
    .accounts({
      user: user.publicKey,
      vault: vaultPda,
      mint: mint,
      vaultTokenAccount: vaultTokenAccount.address,
      userTokenAccount: userTokenAccount.address,
      tokenProgram: TOKEN_PROGRAM_ID,
    } as any)
    .signers([user])
    .rpc();

  console.log("✅ Withdraw successful");
  console.log("\n🎉 DEMO COMPLETE");
})().catch((e) => {
  console.error("❌ Demo failed");
  console.error(e);
});
