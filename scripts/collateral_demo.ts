import * as anchor from "@coral-xyz/anchor";
import fetch from "node-fetch";
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

/* --------------------------------------------- */
/* ZKCG CLIENT                                   */
/* --------------------------------------------- */


async function proveWithZKCG() {
  const res = await fetch("http://127.0.0.1:8080/v1/prove", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({
      secret_value: 700,   // private input
      threshold: 750       // public policy
    }),
  });

  const text = await res.text();
  console.log("ZKCG /prove response:", text);

  if (!res.ok) {
    throw new Error(`ZKCG prover failed: ${text}`);
  }

  return JSON.parse(text);
}

async function submitProofToZKCG(
  proof: string,
  threshold: number,
  commitment: number[],
) {
  const ZERO_32 = new Array(32).fill(0);

  const res = await fetch("http://127.0.0.1:8080/v1/submit-proof", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({
      proof,

      public_inputs: {
        threshold,
        old_state_root: ZERO_32, // demo: genesis
        nonce: 1,               // demo: first transition
      },

      new_state_commitment: commitment,
    }),
  });

  const text = await res.text();
  console.log("ZKCG /submit-proof response:", text);

  if (!res.ok) {
    throw new Error(`ZKCG rejected proof: ${text}`);
  }

  const json = JSON.parse(text);
  if (json.status !== "accepted") {
    throw new Error("ZK proof not accepted");
  }
}




(async () => {
  console.log("🚀 Starting ZK-Gated Collateral Vault Demo");

  /* --------------------------------------------- */
  /* ZK VERIFICATION GATE                          */
  /* --------------------------------------------- */
console.log("🔐 Generating zkVM proof via ZKCG...");
const proofBundle = await proveWithZKCG();

console.log("🔐 Submitting proof to ZKCG verifier...");
await submitProofToZKCG(
  proofBundle.proof,
  proofBundle.public_inputs.threshold,
  proofBundle.commitment
);

console.log("✅ ZK verification passed");


  /* --------------------------------------------- */
  /* Anchor Setup                                  */
  /* --------------------------------------------- */
 const provider = anchor.AnchorProvider.env();
   anchor.setProvider(provider);
 
   
   const program = anchor.workspace.CollateralVault as any;
 
   /* -------------------------------------------------- */
   /* 1. Create collateral mint                          */
   /* -------------------------------------------------- */
   const mint = await createMint(
     provider.connection,
     provider.wallet.payer,
     provider.wallet.publicKey,
     null,
     6
   );
   console.log("Mint:", mint.toBase58());
 
   /* -------------------------------------------------- */
   /* 2. Create user                                     */
   /* -------------------------------------------------- */
   const user = Keypair.generate();
   await provider.connection.confirmTransaction(
     await provider.connection.requestAirdrop(
       user.publicKey,
       5 * LAMPORTS_PER_SOL
     )
   );
   console.log("User:", user.publicKey.toBase58());
 
   /* -------------------------------------------------- */
   /* 3. User ATA                                        */
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
   console.log("VaultAuthority PDA:", vaultAuthorityPda.toBase58());
 
   /* -------------------------------------------------- */
   /* 5. Derive vault ATA (DO NOT CREATE)                */
   /* -------------------------------------------------- */
   const [vaultTokenAccount] = PublicKey.findProgramAddressSync(
     [
       vaultPda.toBuffer(),
       TOKEN_PROGRAM_ID.toBuffer(),
       mint.toBuffer(),
     ],
     ASSOCIATED_TOKEN_PROGRAM_ID
   );
 
   /* -------------------------------------------------- */
   /* 6. Initialize vault                                */
   /* -------------------------------------------------- */
   await program.methods
     .initializeVault()
     .accounts({
       user: user.publicKey,
       vault: vaultPda,
       vaultTokenAccount,
       tokenMint: mint,
       tokenProgram: TOKEN_PROGRAM_ID,
       associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
       systemProgram: SystemProgram.programId,
     })
     .signers([user])
     .rpc();
 
   console.log(" Vault initialized");
 
   /* -------------------------------------------------- */
   /* 7. Mint + deposit                                  */
   /* -------------------------------------------------- */
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
       userTokenAccount: userTokenAccount.address,
       vaultTokenAccount,
       mint,
       tokenProgram: TOKEN_PROGRAM_ID,
     })
     .signers([user])
     .rpc();
 
   console.log(" Deposit successful");
 
 
   /* -------------------------------------------------- */
   /* 9. Initialize vault authority (once)               */
   /* -------------------------------------------------- */
   const existing = await program.account.vaultAuthority.fetchNullable(
     vaultAuthorityPda
   );
  
   if (!existing) {
     await program.methods
       .initializeVaultAuthority([program.programId])
       .accounts({
         admin: provider.wallet.publicKey,
         vaultAuthority: vaultAuthorityPda,
         systemProgram: SystemProgram.programId,
       })
       .rpc();
 
     console.log(" Vault authority initialized");
   } else {
     console.log(" Vault authority already exists");
   }
 
   /* -------------------------------------------------- */
   /* 10. Lock / Unlock (CPI-simulated)                  */
   /* -------------------------------------------------- */
   await program.methods
   .lockCollateral(new anchor.BN(500))
   .accounts({
     callerProgram: program.programId,
     vaultAuthority: vaultAuthorityPda,
     vault: vaultPda,
   })
   .rpc();
 
 console.log("Locked");
 
   await program.methods
     .unlockCollateral(new anchor.BN(500))
     .accounts({
       callerProgram: program.programId,
       vaultAuthority: vaultAuthorityPda,
       vault: vaultPda,
     })
     .rpc();
 
   console.log(" Unlocked");
 
   /* -------------------------------------------------- */
   /* 11. Withdraw                                      */
   /* -------------------------------------------------- */
   await program.methods
     .withdraw(new anchor.BN(1_000))
     .accounts({
       user: user.publicKey,
       vault: vaultPda,
       vaultTokenAccount,
       userTokenAccount: userTokenAccount.address,
       mint,
       tokenProgram: TOKEN_PROGRAM_ID,
     })
     .signers([user])
     .rpc();
 
   console.log(" Withdraw successful");
  console.log("🎉 DEMO COMPLETE — ZK-GATED VAULT SUCCESS");
})().catch((e) => {
  console.error("❌ Demo failed:", e);
});
