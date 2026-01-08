import * as anchor from "@coral-xyz/anchor";
import { PublicKey } from "@solana/web3.js";
import fs from "fs";

const state = JSON.parse(fs.readFileSync("scripts/state.json", "utf8"));
const provider = anchor.AnchorProvider.env();
anchor.setProvider(provider);

const program = anchor.workspace.CollateralVault;

const [vaultPda] = PublicKey.findProgramAddressSync(
  [Buffer.from("vault"), provider.wallet.publicKey.toBuffer()],
  program.programId
);
state.vaultPda = vaultPda.toBase58();

fs.writeFileSync("scripts/state.json", JSON.stringify(state, null, 2));
console.log("Wallet:", provider.wallet.publicKey.toBase58());
console.log("Vault PDA:", vaultPda.toBase58());
