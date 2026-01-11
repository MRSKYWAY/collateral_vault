import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { CollateralVault } from "../target/types/collateral_vault";
import { expect } from "chai";
import {
  createMint,
  getAssociatedTokenAddressSync,
  getOrCreateAssociatedTokenAccount,
  mintTo,
  TOKEN_PROGRAM_ID,
  ASSOCIATED_TOKEN_PROGRAM_ID,
} from "@solana/spl-token";
import { Keypair, PublicKey, SystemProgram } from "@solana/web3.js";

describe("collateral-vault security and integration", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace
    .CollateralVault as Program<CollateralVault>;

  // Separate users to avoid shared state
  const userA = anchor.web3.Keypair.generate(); // owner (test 1: non-owner withdraw)
  const userB = anchor.web3.Keypair.generate(); // attacker
  const userC = anchor.web3.Keypair.generate(); // owner (test 2: over-withdraw)
  const userD = anchor.web3.Keypair.generate(); // owner (full flow test)

  // Helper: derive vault PDA
  const deriveVaultPda = (user: anchor.web3.PublicKey) => {
    return anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("vault"), user.toBuffer()],
      program.programId
    );
  };

  // Helper: derive vault authority PDA
  const deriveVaultAuthorityPda = () => {
    return anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("vault_authority")],
      program.programId
    );
  };

  let mint: anchor.web3.PublicKey;

  before(async () => {
    // Fund users
    for (const user of [userA, userB, userC, userD]) {
      const sig = await provider.connection.requestAirdrop(
        user.publicKey,
        2 * anchor.web3.LAMPORTS_PER_SOL
      );
      await provider.connection.confirmTransaction(sig);
    }

    // Create test USDT mint
    mint = await createMint(
      provider.connection,
      provider.wallet.payer,
      provider.wallet.publicKey,
      null,
      6
    );
  });

  /**
   * TEST 1: Unauthorized user cannot withdraw from another user's vault
   */
  it("fails when non-owner tries to withdraw", async () => {
    const [vaultPda, vaultBump] = deriveVaultPda(userA.publicKey);

    const vaultTokenAccount = getAssociatedTokenAddressSync(
      mint,
      vaultPda,
      true
    );

    // Create PDA-owned vault ATA
    await getOrCreateAssociatedTokenAccount(
      provider.connection,
      provider.wallet.payer,
      mint,
      vaultPda,
      true
    );

    // Initialize vault
    await program.methods
      .initializeVault()
      .accounts({
        user: userA.publicKey,
        vault: vaultPda,
        vaultTokenAccount,
        tokenMint: mint,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId,
      }as any)
      .signers([userA])
      .rpc();

    // Create attacker token account
    const userBTokenAccount = await getOrCreateAssociatedTokenAccount(
      provider.connection,
      provider.wallet.payer,
      mint,
      userB.publicKey
    );

    // Unauthorized withdraw attempt
    try {
      await program.methods
        .withdraw(new anchor.BN(1))
        .accounts({
          user: userB.publicKey, // ❌ not vault owner
          vault: vaultPda,
          vaultTokenAccount,
          userTokenAccount: userBTokenAccount.address,
          tokenProgram: TOKEN_PROGRAM_ID,
        }as any)
        .signers([userB])
        .rpc();

      expect.fail("Withdraw by non-owner should fail");
    } catch (err) {
      expect(err).to.exist;
      expect(err.toString()).to.include("ConstraintSeeds"); // Or specific error like "Unauthorized"
    }
  });

  /**
   * TEST 2: User cannot withdraw more than available balance (without lock)
   */
  it("fails when withdrawing more than total balance", async () => {
    const [vaultPda, vaultBump] = deriveVaultPda(userC.publicKey);

    const vaultTokenAccount = getAssociatedTokenAddressSync(
      mint,
      vaultPda,
      true
    );

    // Create PDA-owned vault ATA
    await getOrCreateAssociatedTokenAccount(
      provider.connection,
      provider.wallet.payer,
      mint,
      vaultPda,
      true
    );

    // Initialize vault
    await program.methods
      .initializeVault()
      .accounts({
        user: userC.publicKey,
        vault: vaultPda,
        vaultTokenAccount,
        tokenMint: mint,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId,
      }as any)
      .signers([userC])
      .rpc();

    // Create userC token account
    const userCTokenAccount = await getOrCreateAssociatedTokenAccount(
      provider.connection,
      provider.wallet.payer,
      mint,
      userC.publicKey
    );

    // Mint 100 tokens to userC
    await mintTo(
      provider.connection,
      provider.wallet.payer,
      mint,
      userCTokenAccount.address,
      provider.wallet.publicKey,
      100
    );

    // Deposit 100
    await program.methods
      .deposit(new anchor.BN(100))
      .accounts({
        user: userC.publicKey,
        vault: vaultPda,
        userTokenAccount: userCTokenAccount.address,
        vaultTokenAccount,
        tokenProgram: TOKEN_PROGRAM_ID,
      }as any)
      .signers([userC])
      .rpc();

    // Attempt to withdraw more than total balance
    try {
      await program.methods
        .withdraw(new anchor.BN(150))
        .accounts({
          user: userC.publicKey,
          vault: vaultPda,
          vaultTokenAccount,
          userTokenAccount: userCTokenAccount.address,
          tokenProgram: TOKEN_PROGRAM_ID,
        }as any)
        .signers([userC])
        .rpc();

      expect.fail("Withdraw exceeding balance should fail");
    } catch (err) {
      expect(err).to.exist;
      expect(err.toString()).to.include("InsufficientAvailableBalance"); // Assume custom error
    }
  });

  /**
   * TEST 3: Full end-to-end flow: deposit → lock → fail withdraw → unlock → withdraw
   */
  it("full flow: deposit, lock, fail withdraw, unlock, withdraw", async () => {
    const [vaultPda, vaultBump] = deriveVaultPda(userD.publicKey);

    const vaultTokenAccount = getAssociatedTokenAddressSync(
      mint,
      vaultPda,
      true
    );

    // Create PDA-owned vault ATA
    await getOrCreateAssociatedTokenAccount(
      provider.connection,
      provider.wallet.payer,
      mint,
      vaultPda,
      true
    );

    // Initialize vault
    await program.methods
      .initializeVault()
      .accounts({
        user: userD.publicKey,
        vault: vaultPda,
        vaultTokenAccount,
        tokenMint: mint,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId,
      }as any)
      .signers([userD])
      .rpc();

    const userDTokenAccount = await getOrCreateAssociatedTokenAccount(
      provider.connection,
      provider.wallet.payer,
      mint,
      userD.publicKey
    );

    // Mint 1000 tokens to userD
    await mintTo(
      provider.connection,
      provider.wallet.payer,
      mint,
      userDTokenAccount.address,
      provider.wallet.publicKey,
      1000
    );

    // Deposit 1000
    await program.methods
      .deposit(new anchor.BN(1000))
      .accounts({
        user: userD.publicKey,
        vault: vaultPda,
        userTokenAccount: userDTokenAccount.address,
        vaultTokenAccount,
        tokenProgram: TOKEN_PROGRAM_ID,
      }as any)
      .signers([userD])
      .rpc();

    let vaultAcc = await program.account.collateralVault.fetch(vaultPda);
    expect(vaultAcc.totalBalance.toNumber()).to.equal(1000);
    expect(vaultAcc.availableBalance.toNumber()).to.equal(1000);
    expect(vaultAcc.lockedBalance.toNumber()).to.equal(0);

    // Mock CPI setup
    const mockCaller = Keypair.generate();
    const mockSig = await provider.connection.requestAirdrop(
      mockCaller.publicKey,
      anchor.web3.LAMPORTS_PER_SOL
    );
    await provider.connection.confirmTransaction(mockSig);

    const [vaultAuthorityPda] = deriveVaultAuthorityPda();

    // Initialize vault authority with mock caller authorized
    await program.methods
      .initializeVaultAuthority([mockCaller.publicKey])
      .accounts({
        admin: provider.wallet.publicKey,
        vaultAuthority: vaultAuthorityPda,
        systemProgram: SystemProgram.programId,
      }as any)
      .signers([provider.wallet.payer])
      .rpc();

    // Lock 600 via mock CPI
    await program.methods
      .lockCollateral(new anchor.BN(600))
      .accounts({
        callerProgram: mockCaller.publicKey,
        vaultAuthority: vaultAuthorityPda,
        vault: vaultPda,
      }as any)
      .signers([mockCaller])
      .rpc();

    vaultAcc = await program.account.collateralVault.fetch(vaultPda);
    expect(vaultAcc.lockedBalance.toNumber()).to.equal(600);
    expect(vaultAcc.availableBalance.toNumber()).to.equal(400);

    // Fail withdraw 500 (more than available 400)
    try {
      await program.methods
        .withdraw(new anchor.BN(500))
        .accounts({
          user: userD.publicKey,
          vault: vaultPda,
          vaultTokenAccount,
          userTokenAccount: userDTokenAccount.address,
          tokenProgram: TOKEN_PROGRAM_ID,
        }as any)
        .signers([userD])
        .rpc();
      expect.fail("Withdraw exceeding available should fail");
    } catch (err) {
      expect(err).to.exist;
      expect(err.toString()).to.include("InsufficientAvailableBalance");
    }

    // Unlock 600 via mock CPI
    await program.methods
      .unlockCollateral(new anchor.BN(600))
      .accounts({
        callerProgram: mockCaller.publicKey,
        vaultAuthority: vaultAuthorityPda,
        vault: vaultPda,
      }as any)
      .signers([mockCaller])
      .rpc();

    vaultAcc = await program.account.collateralVault.fetch(vaultPda);
    expect(vaultAcc.lockedBalance.toNumber()).to.equal(0);
    expect(vaultAcc.availableBalance.toNumber()).to.equal(1000);

    // Withdraw 1000 success
    await program.methods
      .withdraw(new anchor.BN(1000))
      .accounts({
        user: userD.publicKey,
        vault: vaultPda,
        vaultTokenAccount,
        userTokenAccount: userDTokenAccount.address,
        tokenProgram: TOKEN_PROGRAM_ID,
      }as any)
      .signers([userD])
      .rpc();

    vaultAcc = await program.account.collateralVault.fetch(vaultPda);
    expect(vaultAcc.totalBalance.toNumber()).to.equal(0);
    expect(vaultAcc.availableBalance.toNumber()).to.equal(0);
    expect(vaultAcc.lockedBalance.toNumber()).to.equal(0);
  });

  /**
   * Additional Test: Initializes vault (standalone)
   */
  it("Initializes vault", async () => {
    const testUser = Keypair.generate();
    const sig = await provider.connection.requestAirdrop(
      testUser.publicKey,
      2 * anchor.web3.LAMPORTS_PER_SOL
    );
    await provider.connection.confirmTransaction(sig);

    const [vaultPda] = deriveVaultPda(testUser.publicKey);
    const vaultTokenAccount = getAssociatedTokenAddressSync(
      mint,
      vaultPda,
      true
    );

    // Create PDA-owned vault ATA
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
        user: testUser.publicKey,
        vault: vaultPda,
        vaultTokenAccount,
        tokenMint: mint,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      }as any)
      .signers([testUser])
      .rpc();

    const vaultAcc = await program.account.collateralVault.fetch(vaultPda);
    expect(vaultAcc.totalBalance.toNumber()).to.equal(0);
    expect(vaultAcc.lockedBalance.toNumber()).to.equal(0);
    expect(vaultAcc.availableBalance.toNumber()).to.equal(0);
  });

  /**
   * Additional Test: Deposits and locks via CPI mock (standalone)
   */
  it("Deposits and locks via CPI mock", async () => {
    const testUser = Keypair.generate();
    const sig = await provider.connection.requestAirdrop(
      testUser.publicKey,
      2 * anchor.web3.LAMPORTS_PER_SOL
    );
    await provider.connection.confirmTransaction(sig);

    const [vaultPda] = deriveVaultPda(testUser.publicKey);
    const vaultTokenAccount = getAssociatedTokenAddressSync(
      mint,
      vaultPda,
      true
    );

    // Create PDA-owned vault ATA
    await getOrCreateAssociatedTokenAccount(
      provider.connection,
      provider.wallet.payer,
      mint,
      vaultPda,
      true
    );

    // Initialize vault
    await program.methods
      .initializeVault()
      .accounts({
        user: testUser.publicKey,
        vault: vaultPda,
        vaultTokenAccount,
        tokenMint: mint,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      }as any)
      .signers([testUser])
      .rpc();

    const userTokenAccount = await getOrCreateAssociatedTokenAccount(
      provider.connection,
      provider.wallet.payer,
      mint,
      testUser.publicKey
    );

    // Mint 1000 tokens to user
    await mintTo(
      provider.connection,
      provider.wallet.payer,
      mint,
      userTokenAccount.address,
      provider.wallet.publicKey,
      1000
    );

    // Deposit 1000
    await program.methods
      .deposit(new anchor.BN(1000))
      .accounts({
        user: testUser.publicKey,
        vault: vaultPda,
        userTokenAccount: userTokenAccount.address,
        vaultTokenAccount,
        tokenProgram: TOKEN_PROGRAM_ID,
      }as any)
      .signers([testUser])
      .rpc();

    // Mock CPI: Simulate position manager program calling lock_collateral
    const mockCaller = Keypair.generate(); // Mock authorized program

    // Fund mock caller if needed
    const mockSig = await provider.connection.requestAirdrop(
      mockCaller.publicKey,
      anchor.web3.LAMPORTS_PER_SOL
    );
    await provider.connection.confirmTransaction(mockSig);

    const [vaultAuthorityPda] = deriveVaultAuthorityPda();

    // Initialize vault authority with mock caller authorized
    await program.methods
      .initializeVaultAuthority([mockCaller.publicKey])
      .accounts({
        admin: provider.wallet.publicKey,
        vaultAuthority: vaultAuthorityPda,
        systemProgram: SystemProgram.programId,
      }as any)
      .signers([provider.wallet.payer])
      .rpc();

    // Call lock as if from mockCaller
    await program.methods
      .lockCollateral(new anchor.BN(500))
      .accounts({
        callerProgram: mockCaller.publicKey,
        vaultAuthority: vaultAuthorityPda,
        vault: vaultPda,
      }as any)
      .signers([mockCaller])
      .rpc();

    // Assert: Fetch vault, check locked_balance=500, available=500
    const vaultAcc = await program.account.collateralVault.fetch(vaultPda);
    expect(vaultAcc.lockedBalance.toNumber()).to.equal(500);
    expect(vaultAcc.availableBalance.toNumber()).to.equal(500);
  });

  /**
   * Additional Test: Unauthorized CPI lock
   */
  it("Tests unauthorized CPI", async () => {
    const testUser = Keypair.generate();
    const sig = await provider.connection.requestAirdrop(
      testUser.publicKey,
      2 * anchor.web3.LAMPORTS_PER_SOL
    );
    await provider.connection.confirmTransaction(sig);

    const [vaultPda] = deriveVaultPda(testUser.publicKey);
    const vaultTokenAccount = getAssociatedTokenAddressSync(
      mint,
      vaultPda,
      true
    );

    // Create PDA-owned vault ATA
    await getOrCreateAssociatedTokenAccount(
      provider.connection,
      provider.wallet.payer,
      mint,
      vaultPda,
      true
    );

    // Initialize vault
    await program.methods
      .initializeVault()
      .accounts({
        user: testUser.publicKey,
        vault: vaultPda,
        vaultTokenAccount,
        tokenMint: mint,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      }as any)
      .signers([testUser])
      .rpc();

    const [vaultAuthorityPda] = deriveVaultAuthorityPda();

    // Try lock with unauthorized caller → expect error
    const unauthorizedCaller = Keypair.generate();
    const mockSig = await provider.connection.requestAirdrop(
      unauthorizedCaller.publicKey,
      anchor.web3.LAMPORTS_PER_SOL
    );
    await provider.connection.confirmTransaction(mockSig);

    try {
      await program.methods
        .lockCollateral(new anchor.BN(100))
        .accounts({
          callerProgram: unauthorizedCaller.publicKey,
          vaultAuthority: vaultAuthorityPda,
          vault: vaultPda,
        }as any)
        .signers([unauthorizedCaller])
        .rpc();
      expect.fail("Should have failed");
    } catch (err) {
      expect(err.toString()).to.include("Unauthorized");
    }
  });
});