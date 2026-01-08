# Collateral Vault – Current Progress & Testing Guide

This document captures **exactly what has been implemented so far** in the Collateral Vault assignment and provides **step‑by‑step instructions to reproduce and test the working functionality on localnet**.

---

## 1. What Is Implemented (Current State)

### 1.1 Smart Contract (Anchor Program)

The following on-chain components are implemented and verified:

#### ✅ Vault PDA
- Deterministic PDA derived as:
  ```text
  seeds = ["vault", user_pubkey]
  ```
- One vault per user
- Rent-exempt, owned by the program

#### ✅ Vault State (`CollateralVault`)
- Owner (user pubkey)
- Total balance
- Locked balance
- Available balance (derived)
- Vault token account stored

#### ✅ Instructions Implemented

| Instruction | Status | Notes |
|------------|-------|-------|
| `initialize_vault` | ✅ Working | Initializes vault PDA and links vault token account |
| `deposit` | ✅ Working | SPL token transfer user → vault |
| `withdraw` | ✅ Implemented | Not yet demoed via script |
| `lock_collateral` | ✅ Implemented | CPI-only, invariant tested |
| `unlock_collateral` | ✅ Implemented | CPI-only, invariant tested |
| `transfer_collateral` | ✅ Implemented | Vault → vault internal transfer |

#### ✅ Authority Model
- `VaultAuthority` PDA
- Authorized caller programs list
- CPI-only enforcement for lock/unlock/transfer

#### ✅ Safety & Correctness
- Overflow/underflow safe arithmetic
- Balance invariants tested
- Unauthorized access prevented

---

### 1.2 Tests

#### On-chain Unit Tests (Rust)

The following tests pass:

- `lock_unlock_preserves_invariant`
- `transfer_conserves_collateral`
- Program ID sanity test

These validate:
- Locked + available = total
- No collateral is created/destroyed

---

### 1.3 TypeScript Scripts (Working)

Located in the root-level `scripts/` directory:

| Script | Purpose | Status |
|------|--------|-------|
| `derive.ts` | Derive vault PDA | ✅ |
| `createVaultAta.ts` | Create vault token account | ✅ |
| `initVault.ts` | Initialize vault | ✅ |
| `deposit.ts` | Deposit SPL tokens | ✅ |

All scripts run successfully against **localnet**.

---

## 2. What Is NOT Implemented Yet

This is intentional and will be completed next.

- ❌ Rust backend service
- ❌ Database schema
- ❌ REST / WebSocket APIs
- ❌ Event emission
- ❌ CPI demo caller program
- ❌ Transaction history tracking
- ❌ Production hardening (`init_if_needed`, retries, etc.)

---

## 3. How to Run & Test (From Scratch)

### 3.1 Prerequisites

Ensure the following are installed:

- Solana CLI
- Anchor CLI
- Node.js (>= 18)
- npm / yarn

---

### 3.2 Start Local Validator

```bash
solana-test-validator
```

In a separate terminal:

```bash
solana config set --url localhost
```

Airdrop SOL:

```bash
solana airdrop 5
```

---

### 3.3 Build & Deploy Program

```bash
anchor build
anchor deploy
```

Confirm program ID matches `Anchor.toml`.

---

### 3.4 Derive Vault PDA

```bash
anchor run derive
```

Output:
- Wallet pubkey
- Vault PDA

---

### 3.5 Create SPL Token & User Token Account

```bash
spl-token create-token
```

Save the **mint address**.

Create user token account:

```bash
spl-token create-account <MINT>
```

Mint tokens:

```bash
spl-token mint <MINT> 1000
```

Verify:

```bash
spl-token accounts
```

---

### 3.6 Create Vault Token Account

```bash
anchor run createVaultAta
```

This creates the SPL token account owned by the vault PDA.

---

### 3.7 Initialize Vault

Update `scripts/initVault.ts` with the **vault token account address**:

```ts
const vaultTokenAccount = new PublicKey("<VAULT_TOKEN_ACCOUNT>");
```

Run:

```bash
anchor run initVault
```

Expected output:

```text
Vault initialized
```

---

### 3.8 Deposit Tokens

Update `scripts/deposit.ts` with:

- User token account
- Vault token account

Run:

```bash
anchor run deposit
```

Expected output:

```text
Deposited 200 tokens
```

Verify balances:

```bash
spl-token accounts
```

You should see user balance reduced accordingly.

---

## 4. Known Gotchas (Important)

- **Vault token account must exist before deposit**
- **User token account must be initialized**
- `initialize_vault` can only be run once per PDA
- Re-running requires fresh validator or new wallet

---

## 5. Current State Summary

✅ Core custody smart contract works end-to-end

✅ Real SPL tokens transferred on-chain

✅ Invariants enforced and tested

🟡 Assignment partially complete (backend + APIs pending)

---

## 6. Next Steps

1. Add event emission to all instructions
2. Demo withdraw + lock/unlock via scripts
3. Build minimal Rust backend
4. Add database schema
5. Expose REST APIs
6. Write architecture & security docs

---

**Status:** Core on-chain system is correct, secure, and working.

