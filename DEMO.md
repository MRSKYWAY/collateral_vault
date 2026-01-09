# Collateral Vault – End-to-End Demo & Review Guide

## Purpose
This document explains **exactly how the collateral vault system works**, how to run the demo from a clean localnet, and how each requirement of the GoQuant assignment is satisfied.

It is written so a reviewer can:
- Reproduce the demo step by step
- Verify security and correctness
- Map implementation → requirements without guessing

---

## 1. Environment Setup

```bash
solana-test-validator --reset
```

```bash
solana config set --url localhost
solana airdrop 20
```

```bash
anchor clean
anchor build
anchor program deploy
```

---

## 2. Token Setup (Collateral Mint)

The system uses **legacy SPL tokens** (not Token-2022).

```bash
spl-token create-token
spl-token create-account <MINT>
spl-token mint <MINT> 1000000
```

Update the mint in:

```ts
scripts/config.ts
```

---

## 3. User 1 – Vault Initialization & Deposit

```bash
ANCHOR_WALLET=~/.config/solana/id.json anchor run setup
ANCHOR_WALLET=~/.config/solana/id.json anchor run deposit
```

What happens:
- Vault PDA is created using seeds: `["vault", user_pubkey]`
- Vault ATA is created and owned by the PDA
- Tokens are transferred into the vault
- Internal balances are updated atomically

---

## 4. User 2 – Independent Vault

```bash
solana-keygen new -o user2.json
solana airdrop 5 --keypair user2.json
```

```bash
ANCHOR_WALLET=$(pwd)/user2.json npx ts-node scripts/setup.ts
```

This proves:
- Each user has an isolated vault PDA
- Vaults are independent and non-custodial

---

## 5. Vault Authority Initialization (One-Time)

```bash
ANCHOR_WALLET=~/.config/solana/id.json anchor run initAuthority
```

This creates a **global authority PDA**:
- Controls CPI access to lock/unlock/transfer
- Maintains a whitelist of authorized programs

---

## 6. Collateral Transfer (CPI-Only)

```bash
ANCHOR_WALLET=~/.config/solana/id.json anchor run transfer
```

Security guarantees:
- Only authorized programs may call transfer
- Vault balances update atomically
- No token mint/burn occurs

---

## 7. Security Model Summary

- Vault PDAs prevent direct user tampering
- Token accounts are owned by PDAs
- Only vault owner can withdraw
- Only authorized CPI callers can lock/unlock/transfer
- Checked arithmetic prevents overflow
- All state transitions are atomic

---

## 8. Vault Lifecycle

```
Initialize → Deposit → [Lock ↔ Unlock] → Withdraw
                  ↓
           CPI-based Transfer
```

---

## 9. Key Design Choices

- `init` (not `init_if_needed`) to prevent re-initialization
- Explicit authority PDA for CPI control
- Legacy SPL token for simplicity and compatibility
- Separate scripts for user actions vs protocol actions

---

## 10. Conclusion

This implementation demonstrates a **production-grade custody layer** suitable for a perpetual futures DEX, with strong guarantees around security, isolation, and correctness.

All core assignment requirements are implemented and demonstrated on-chain.
