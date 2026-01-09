# Collateral Vault – Architecture

## High-Level Overview

The system is intentionally split into **two layers**:

1. **On-chain custody layer (Anchor program)**
2. **Off-chain coordination & observation layer (Rust backend)**

This separation mirrors real-world DeFi architectures and minimizes trust assumptions.

---

## 1. On-Chain Layer (Anchor Program)

### Responsibilities
- Custody of user collateral
- Enforcing balance invariants
- Authorization of sensitive operations
- Emitting verifiable events

### Vault PDA
- One vault per user
- Seeds: `["vault", user_pubkey]`
- Program-controlled, no private key

### Vault Token Account
- SPL token account owned by the vault PDA
- Holds real collateral tokens

### Balance Model
- `total_balance`
- `available_balance`
- `locked_balance`

Invariant:
```
available_balance + locked_balance == total_balance
```

---

## 2. Authority Model

### Vault Authority PDA
- Global PDA: `["vault_authority"]`
- Whitelist of authorized program IDs

### Purpose
- Restricts `lock`, `unlock`, and `transfer` to CPI calls only
- Prevents user-level bypass of protocol rules

---

## 3. Security Guarantees

- Only vault owner can withdraw
- Only authorized programs can lock/unlock/transfer
- PDA signer enforcement
- Checked arithmetic
- Atomic state updates

---

## 4. Events

- DepositEvent
- WithdrawEvent
- TransferEvent

Used by the backend for indexing and monitoring.

---

## 5. Trust Model

- On-chain program is the sole custodian
- Backend is non-custodial
- Backend can be replaced without risk

---

## Summary

The design prioritizes security, explicit authority, and clean separation of concerns.
