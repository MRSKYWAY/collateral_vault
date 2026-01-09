// scripts/pda.ts
import { PublicKey } from "@solana/web3.js";

export function getStatePda(programId: PublicKey) {
  return PublicKey.findProgramAddressSync(
    [Buffer.from("state")],
    programId
  );
}

export function getVaultPda(programId: PublicKey) {
  return PublicKey.findProgramAddressSync(
    [Buffer.from("vault")],
    programId
  );
}
