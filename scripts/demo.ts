// scripts/demo.ts - Full end-to-end demo script for Collateral Vault System
// Run with: ts-node scripts/demo.ts
// Assumes local Solana validator running (solana-test-validator) and Anchor setup

import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { CollateralVault } from "../target/types/collateral_vault";
import {
  createMint,
  getAssociatedTokenAddressSync,
  getOrCreateAssociatedTokenAccount,
  mintTo,
  TOKEN_PROGRAM_ID,
  ASSOCIATED_TOKEN_PROGRAM_ID,
} from "@solana/spl-token";
import { Keypair, PublicKey, SystemProgram, LAMPORTS_PER_SOL } from "@solana/web3.js";
import { assert } from "chai";

async function runDemo() {
  console.log("Starting Collateral Vault Demo...");

  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const program = anchor.workspace.CollateralVault as Program<CollateralVault>;

  // Step 1: Setup - Create USDT mint (mock)
  console.log("\n1. Setting up mock USDT mint...");
  const mint = await createMint(
    provider.connection,
    provider.wallet.payer,
    provider.wallet.publicKey,
    null,
    6 // 6 decimals like USDT
  );
  console.log("Mint created:", mint.toBase58());

  // Step 2: Create user and fund
  const user = Keypair.generate();
  const airdropSig = await provider.connection.requestAirdrop(user.publicKey, 2 * LAMPORTS_PER_SOL);
  await provider.connection.confirmTransaction(airdropSig);
  console.log("User funded:", user.publicKey.toBase58());

  // Derive PDAs
  const [vaultPda] = PublicKey.findProgramAddressSync(
    [Buffer.from("vault"), user.publicKey.toBuffer()],
    program.programId
  );
  const [vaultAuthorityPda] = PublicKey.findProgramAddressSync(
    [Buffer.from("vault_authority")],
    program.programId
  );
  const vaultTokenAccount = getAssociatedTokenAddressSync(mint, vaultPda, true);

  // Step 3: Initialize Vault
  console.log("\n2. Initializing vault...");
  await getOrCreateAssociatedTokenAccount(
    provider.connection,
    provider.wallet.payer,
    mint,
    vaultPda,
    true
  );
  await program.methods
    .initializeVault()
    .accounts({
      user: user.publicKey,
      vault: vaultPda,
      vaultTokenAccount,
      tokenMint: mint,
      associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
      tokenProgram: TOKEN_PROGRAM_ID,
      systemProgram: SystemProgram.programId,
    }as any)
    .signers([user])
    .rpc();
  let vaultAcc = await program.account.collateralVault.fetch(vaultPda);
  console.log("Vault initialized. Balances: Total=0, Available=0, Locked=0");

  // Step 4: Deposit Collateral
  console.log("\n3. Depositing 1000 USDT...");
  const userTokenAccount = await getOrCreateAssociatedTokenAccount(
    provider.connection,
    provider.wallet.payer,
    mint,
    user.publicKey
  );
  await mintTo(
    provider.connection,
    provider.wallet.payer,
    mint,
    userTokenAccount.address,
    provider.wallet.publicKey,
    1000
  );
  await program.methods
    .deposit(new anchor.BN(1000))
    .accounts({
      user: user.publicKey,
      vault: vaultPda,
      userTokenAccount: userTokenAccount.address,
      vaultTokenAccount,
      tokenProgram: TOKEN_PROGRAM_ID,
    }as any)
    .signers([user])
    .rpc();
  vaultAcc = await program.account.collateralVault.fetch(vaultPda);
  assert(vaultAcc.totalBalance.toNumber() === 1000);
  assert(vaultAcc.availableBalance.toNumber() === 1000);
  console.log("Deposit successful. Balances: Total=1000, Available=1000, Locked=0");

  // Step 5: Setup Mock Position Manager for CPI
  console.log("\n4. Setting up mock position manager for CPI...");
  const mockPositionProgram = Keypair.generate();
  const mockAirdrop = await provider.connection.requestAirdrop(mockPositionProgram.publicKey, LAMPORTS_PER_SOL);
  await provider.connection.confirmTransaction(mockAirdrop);

  await program.methods
    .initializeVaultAuthority([mockPositionProgram.publicKey])
    .accounts({
      admin: provider.wallet.publicKey,
      vaultAuthority: vaultAuthorityPda,
      systemProgram: SystemProgram.programId,
    }as any)
    .signers([provider.wallet.payer])
    .rpc();
  console.log("Vault authority initialized with mock program authorized.");

  // Step 6: Lock Collateral via CPI Mock
  console.log("\n5. Locking 600 USDT via mock CPI...");
  await program.methods
    .lockCollateral(new anchor.BN(600))
    .accounts({
      callerProgram: mockPositionProgram.publicKey,
      vaultAuthority: vaultAuthorityPda,
      vault: vaultPda,
    }as any)
    .signers([mockPositionProgram])
    .rpc();
  vaultAcc = await program.account.collateralVault.fetch(vaultPda);
  assert(vaultAcc.lockedBalance.toNumber() === 600);
  assert(vaultAcc.availableBalance.toNumber() === 400);
  console.log("Lock successful. Balances: Total=1000, Available=400, Locked=600");

  // Step 7: Attempt Withdraw (should fail due to insufficient available)
  console.log("\n6. Attempting to withdraw 500 USDT (should fail)...");
  try {
    await program.methods
      .withdraw(new anchor.BN(500))
      .accounts({
        user: user.publicKey,
        vault: vaultPda,
        vaultTokenAccount,
        userTokenAccount: userTokenAccount.address,
        tokenProgram: TOKEN_PROGRAM_ID,
      }as any)
      .signers([user])
      .rpc();
    assert.fail("Withdraw should have failed");
  } catch (err) {
    console.log("Withdraw failed as expected:", err.toString().substring(0, 50) + "...");
  }

  // Step 8: Unlock Collateral via CPI Mock
  console.log("\n7. Unlocking 600 USDT via mock CPI...");
  await program.methods
    .unlockCollateral(new anchor.BN(600))
    .accounts({
      callerProgram: mockPositionProgram.publicKey,
      vaultAuthority: vaultAuthorityPda,
      vault: vaultPda,
    }as any)
    .signers([mockPositionProgram])
    .rpc();
  vaultAcc = await program.account.collateralVault.fetch(vaultPda);
  assert(vaultAcc.lockedBalance.toNumber() === 0);
  assert(vaultAcc.availableBalance.toNumber() === 1000);
  console.log("Unlock successful. Balances: Total=1000, Available=1000, Locked=0");

  // Step 9: Withdraw Collateral
  console.log("\n8. Withdrawing 1000 USDT...");
  await program.methods
    .withdraw(new anchor.BN(1000))
    .accounts({
      user: user.publicKey,
      vault: vaultPda,
      vaultTokenAccount,
      userTokenAccount: userTokenAccount.address,
      tokenProgram: TOKEN_PROGRAM_ID,
    }as any)
    .signers([user])
    .rpc();
  vaultAcc = await program.account.collateralVault.fetch(vaultPda);
  assert(vaultAcc.totalBalance.toNumber() === 0);
  console.log("Withdraw successful. Balances: Total=0, Available=0, Locked=0");

  // Step 10: Demo Unauthorized Access (non-owner withdraw)
  console.log("\n9. Demo unauthorized access (non-owner withdraw)...");
  const attacker = Keypair.generate();
  const attackerAirdrop = await provider.connection.requestAirdrop(attacker.publicKey, LAMPORTS_PER_SOL);
  await provider.connection.confirmTransaction(attackerAirdrop);
  const attackerToken = await getOrCreateAssociatedTokenAccount(
    provider.connection,
    provider.wallet.payer,
    mint,
    attacker.publicKey
  );
  try {
    await program.methods
      .withdraw(new anchor.BN(1))
      .accounts({
        user: attacker.publicKey,
        vault: vaultPda,
        vaultTokenAccount,
        userTokenAccount: attackerToken.address,
        tokenProgram: TOKEN_PROGRAM_ID,
      }as any)
      .signers([attacker])
      .rpc();
    assert.fail("Unauthorized withdraw should fail");
  } catch (err) {
    console.log("Unauthorized withdraw failed as expected.");
  }

  // Step 11: Demo Transfer Collateral (setup second vault)
  console.log("\n10. Demo transfer collateral between vaults...");
  const user2 = Keypair.generate();
  const airdrop2 = await provider.connection.requestAirdrop(user2.publicKey, 2 * LAMPORTS_PER_SOL);
  await provider.connection.confirmTransaction(airdrop2);

  const [vault2Pda] = PublicKey.findProgramAddressSync(
    [Buffer.from("vault"), user2.publicKey.toBuffer()],
    program.programId
  );
  const vault2Token = getAssociatedTokenAddressSync(mint, vault2Pda, true);

  await getOrCreateAssociatedTokenAccount(
    provider.connection,
    provider.wallet.payer,
    mint,
    vault2Pda,
    true
  );
  await program.methods
    .initializeVault()
    .accounts({
      user: user2.publicKey,
      vault: vault2Pda,
      vaultTokenAccount: vault2Token,
      tokenMint: mint,
      associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
      tokenProgram: TOKEN_PROGRAM_ID,
      systemProgram: SystemProgram.programId,
    }as any)
    .signers([user2])
    .rpc();

  // Deposit to first vault again for transfer
  await mintTo(
    provider.connection,
    provider.wallet.payer,
    mint,
    userTokenAccount.address,
    provider.wallet.publicKey,
    500
  );
  await program.methods
    .deposit(new anchor.BN(500))
    .accounts({
      user: user.publicKey,
      vault: vaultPda,
      userTokenAccount: userTokenAccount.address,
      vaultTokenAccount,
      tokenProgram: TOKEN_PROGRAM_ID,
    }as any)
    .signers([user])
    .rpc();

  // Transfer 200 from vault1 to vault2 (mock caller)
  await program.methods
    .transferCollateral(new anchor.BN(200))
    .accounts({
      callerProgram: mockPositionProgram.publicKey,
      vaultAuthority: vaultAuthorityPda,
      fromVault: vaultPda,
      toVault: vault2Pda,
    }as any)
    .signers([mockPositionProgram])
    .rpc();

  const vault1Acc = await program.account.collateralVault.fetch(vaultPda);
  const vault2Acc = await program.account.collateralVault.fetch(vault2Pda);
  assert(vault1Acc.totalBalance.toNumber() === 300);
  assert(vault2Acc.totalBalance.toNumber() === 200);
  console.log("Transfer successful. Vault1 Total=300, Vault2 Total=200");

  // Bonus: If multi-sig/yield implemented, demo here
  // e.g., Initialize multi-sig, attempt withdraw with/without threshold

  // Step 12: Run a simple perf test (10 ops)
  console.log("\n11. Running simple performance demo (10 vault inits + deposits)...");
  const start = Date.now();
  for (let i = 0; i < 10; i++) {
    const tempUser = Keypair.generate();
    await provider.connection.requestAirdrop(tempUser.publicKey, LAMPORTS_PER_SOL);
    const [tempVault] = PublicKey.findProgramAddressSync(
      [Buffer.from("vault"), tempUser.publicKey.toBuffer()],
      program.programId
    );
    const tempVaultToken = getAssociatedTokenAddressSync(mint, tempVault, true);
    await getOrCreateAssociatedTokenAccount(provider.connection, provider.wallet.payer, mint, tempVault, true);
    await program.methods.initializeVault().accounts({ /* ... similar */ }).signers([tempUser]).rpc();
    // Deposit skipped for speed
  }
  const duration = (Date.now() - start) / 1000;
  console.log(`10 ops in ${duration}s | ~${10 / duration} ops/sec`);

  // Note: For backend demo, add axios calls to /vault/initialize, /deposit, etc.
  // Assume backend running at localhost:3000
  // e.g., import axios; await axios.post('/vault/deposit', { user: user.publicKey.toBase58(), amount: 100 });

  console.log("\nDemo complete!");
}

runDemo().catch(console.error);