import * as anchor from "@coral-xyz/anchor";
import {
  getAssociatedTokenAddressSync,
  TOKEN_PROGRAM_ID,
} from "@solana/spl-token";

export function getContext() {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.CollateralVault;
  const wallet = provider.wallet.publicKey;

  const [vault] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("vault"), wallet.toBuffer()],
    program.programId
  );

  return {
    provider,
    program,
    wallet,
    vault,
    tokenProgram: TOKEN_PROGRAM_ID,
  };
}

export function deriveAta(
  mint: anchor.web3.PublicKey,
  owner: anchor.web3.PublicKey
) {
  return getAssociatedTokenAddressSync(
    mint,
    owner,
    true,
    TOKEN_PROGRAM_ID
  );
}
