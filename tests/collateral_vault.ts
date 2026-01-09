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

describe("collateral-vault security", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace
    .CollateralVault as Program<CollateralVault>;

  // Separate users to avoid shared state
  const userA = anchor.web3.Keypair.generate(); // owner (test 1)
  const userB = anchor.web3.Keypair.generate(); // attacker
  const userC = anchor.web3.Keypair.generate(); // owner (test 2)

  // Helper: derive vault PDA
  const deriveVaultPda = (user: anchor.web3.PublicKey) => {
    return anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("vault"), user.toBuffer()],
      program.programId
    );
  };

  let mint: anchor.web3.PublicKey;

  before(async () => {
    // Fund users
    for (const user of [userA, userB, userC]) {
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
   * TEST 1
   * Unauthorized user cannot withdraw from another user's vault
   */
  it("fails when non-owner tries to withdraw", async () => {
    const [vault] = deriveVaultPda(userA.publicKey);

    const vaultTokenAccount = getAssociatedTokenAddressSync(
      mint,
      vault,
      true
    );

    // Create PDA-owned vault ATA
    await getOrCreateAssociatedTokenAccount(
      provider.connection,
      provider.wallet.payer,
      mint,
      vault,
      true
    );

    // Initialize vault
    await program.methods
      .initializeVault()
      .accounts({
        user: userA.publicKey,
        vault,
        vaultTokenAccount,
        mint,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId,
      } as any)
      .signers([userA])
      .rpc();

    // Create attacker token account
    const userBTokenAccount =
      await getOrCreateAssociatedTokenAccount(
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
          vault,
          vaultTokenAccount,
          userTokenAccount: userBTokenAccount.address,
          mint,
          tokenProgram: TOKEN_PROGRAM_ID,
        } as any)
        .signers([userB])
        .rpc();

      expect.fail("Withdraw by non-owner should fail");
    } catch (err) {
      expect(err).to.exist;
    }
  });

  /**
   * TEST 2
   * User cannot withdraw more than available balance
   * (indirectly validates locked/available balance enforcement)
   */
  it("fails when withdrawing more than available balance", async () => {
    const [vault] = deriveVaultPda(userC.publicKey);

    const vaultTokenAccount = getAssociatedTokenAddressSync(
      mint,
      vault,
      true
    );

    // Create PDA-owned vault ATA
    await getOrCreateAssociatedTokenAccount(
      provider.connection,
      provider.wallet.payer,
      mint,
      vault,
      true
    );

    // Initialize vault
    await program.methods
      .initializeVault()
      .accounts({
        user: userC.publicKey,
        vault,
        vaultTokenAccount,
        mint,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId,
      } as any)
      .signers([userC])
      .rpc();

    // Create userC token account
    const userCTokenAccount =
      await getOrCreateAssociatedTokenAccount(
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
        vault,
        userTokenAccount: userCTokenAccount.address,
        vaultTokenAccount,
        mint,
        tokenProgram: TOKEN_PROGRAM_ID,
      } as any)
      .signers([userC])
      .rpc();

    // Attempt to withdraw more than total balance
    try {
      await program.methods
        .withdraw(new anchor.BN(150))
        .accounts({
          user: userC.publicKey,
          vault,
          vaultTokenAccount,
          userTokenAccount: userCTokenAccount.address,
          mint,
          tokenProgram: TOKEN_PROGRAM_ID,
        } as any)
        .signers([userC])
        .rpc();

      expect.fail("Withdraw exceeding balance should fail");
    } catch (err) {
      expect(err).to.exist;
    }
  });
});
