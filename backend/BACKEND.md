# Collateral Vault – Backend Design

## Purpose

The backend is an **observer and coordinator**, not a custodian.

It:
- Reads on-chain vault state
- Exposes APIs
- Stores derived data
- Builds unsigned transactions (future)

---

## 1. Design Principles

- No private keys
- No fund custody
- All enforcement on-chain

---

## 2. Components

### HTTP API
- `/health`
- `/vault/:owner`
- `/vault/:owner/balance`

Returns real on-chain data.

---

### Solana Integration
- Uses `solana-client`
- Derives vault PDA off-chain
- Decodes Anchor accounts using Borsh
- Explicitly skips Anchor discriminator

---

### Database Layer
- `sqlx`
- SQLite (in-memory for development)
- Tables:
  - `vaults`
  - `vault_transactions`

Used for snapshots and history.

---

## 3. Storage Choice

In-memory SQLite is used for simplicity.
Switching to persistent storage requires only a config change.

---

## 4. Scope Boundaries

The backend does NOT:
- Sign transactions
- Move funds
- Enforce balances

---

## 5. Extensibility

Can be extended with:
- WebSockets
- Indexing
- Analytics
- Risk monitoring

Without modifying the on-chain program.

---

## Summary

The backend complements the on-chain vault while preserving strong security guarantees.
