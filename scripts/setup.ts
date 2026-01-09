import * as anchor from "@coral-xyz/anchor";
import { PublicKey, Keypair } from "@solana/web3.js";
import fs from "fs";
import {
  getAssociatedTokenAddressSync,
  createAssociatedTokenAccountInstruction,
} from "@solana/spl-token";
import { COLLATERAL_MINT } from "./config";

// 🔑 LOAD WALLET EXPLICITLY
const walletPath = process.env.ANCHOR_WALLET!;
const keypair = Keypair.fromSecretKey(
  Uint8Array.from(JSON.parse(fs.readFileSync(walletPath, "utf-8")))
);

const connection = new anchor.web3.Connection(
  "http://127.0.0.1:8899",
  "confirmed"
);

const wallet = new anchor.Wallet(keypair);
const provider = new anchor.AnchorProvider(connection, wallet, {
  commitment: "confirmed",
});

anchor.setProvider(provider);

const program = anchor.workspace.CollateralVault;

(async () => {
  const user = provider.wallet.publicKey;
  console.log("✅ USING WALLET:", user.toBase58());

  const mint = new PublicKey(COLLATERAL_MINT);

  const [vaultPda] = PublicKey.findProgramAddressSync(
    [Buffer.from("vault"), user.toBuffer()],
    program.programId
  );

  const vaultAta = getAssociatedTokenAddressSync(
    mint,
    vaultPda,
    true
  );

  const ataInfo = await connection.getAccountInfo(vaultAta);
  if (!ataInfo) {
    const ix = createAssociatedTokenAccountInstruction(
      user,
      vaultAta,
      vaultPda,
      mint
    );

    await provider.sendAndConfirm(
      new anchor.web3.Transaction().add(ix)
    );
  }

  await program.methods
    .initializeVault()
    .accounts({
      user,
      vault: vaultPda,
      vaultTokenAccount: vaultAta,
      systemProgram: anchor.web3.SystemProgram.programId,
    })
    .rpc();

  console.log("✅ Vault + ATA initialized");
})();
