import * as anchor from "@coral-xyz/anchor";
import { Keypair, LAMPORTS_PER_SOL } from "@solana/web3.js";
import { CollateralVault } from "../target/types/collateral_vault";
import {
  createMint,
  getAssociatedTokenAddressSync,
  getOrCreateAssociatedTokenAccount,
  mintTo,
  TOKEN_PROGRAM_ID,
  ASSOCIATED_TOKEN_PROGRAM_ID,
} from "@solana/spl-token";

async function perfTest() {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const program = anchor.workspace.CollateralVault as anchor.Program<CollateralVault>;

  // Create test USDT mint (once)
  const mint = await createMint(
    provider.connection,
    provider.wallet.payer,
    provider.wallet.publicKey,
    null,
    6
  );

  const ops = 1000; // Start with 1000 for local; scale to 10k if performant
  const start = Date.now();

  for (let i = 0; i < ops; i++) {
    const user = Keypair.generate();

    // Fund user
    const sig = await provider.connection.requestAirdrop(user.publicKey, 2 * LAMPORTS_PER_SOL);
    await provider.connection.confirmTransaction(sig);

    const [vaultPda] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("vault"), user.publicKey.toBuffer()],
      program.programId
    );
    const vaultTokenAccount = getAssociatedTokenAddressSync(mint, vaultPda, true);

    // Create PDA-owned vault ATA (payer funds it)
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
        user: user.publicKey,
        vault: vaultPda,
        vaultTokenAccount,
        tokenMint: mint,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId,
      }as any)
      .signers([user])
      .rpc();

    const userTokenAccount = await getOrCreateAssociatedTokenAccount(
      provider.connection,
      provider.wallet.payer,
      mint,
      user.publicKey
    );

    // Mint tokens to user (payer authority)
    await mintTo(
      provider.connection,
      provider.wallet.payer,
      mint,
      userTokenAccount.address,
      provider.wallet.publicKey,
      1000
    );

    // Deposit
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
  }

  const duration = (Date.now() - start) / 1000;
  console.log(`Performed ${ops} vault inits + deposits in ${duration}s | Ops/sec: ${ops / duration}`);
}

perfTest().catch(console.error);
