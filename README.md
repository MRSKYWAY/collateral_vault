
# Collateral Vault Management System

This repository contains the implementation of a **Collateral Vault Management System** for a decentralized perpetual futures exchange on Solana. It handles secure, non-custodial management of user collateral (USDT) using Anchor for the smart contract and a Rust backend for off-chain services. The system supports deposits, withdrawals, locking/unlocking for trading, and internal transfers, with PostgreSQL for transaction history.


## Table of Contents

- [Objective](#objective)
- [System Architecture](#system-architecture)
- [Prerequisites](#prerequisites)
- [Setup Instructions](#setup-instructions)
- [Smart Contract (On-Chain)](#smart-contract-on-chain)
- [Backend Service (Off-Chain)](#backend-service-off-chain)
- [Database Schema](#database-schema)
- [API Endpoints](#api-endpoints)
- [Testing](#testing)
- [Deployment](#deployment)
- [Security Analysis](#security-analysis)
- [Real-World Integration Example](#real-world-integration-example)
- [Contact](#contact)

## Objective

Build a secure custody layer for a DEX that:
- Manages USDT collateral in PDA vaults.
- Supports deposits/withdrawals with balance checks.
- Locks/unlocks collateral for trading via CPIs.
- Tracks history in PostgreSQL.
- Ensures atomicity, security, and scalability for 1000+ users.

## System Architecture

- **On-Chain (Solana Program)**: Anchor-based program for vault initialization, deposit, withdraw, lock, unlock, and transfer. Uses SPL Token for transfers and PDAs for ownership.
- **Off-Chain (Rust Backend)**: Axum server for APIs, transaction building (unsigned tx base64), real-time monitoring, and DB integration.
- **Database**: PostgreSQL for vaults, transactions, snapshots, and logs.
- **Integration**: WebSockets for updates, reconciliation tasks for on-chain/DB sync.

Flow Diagram (text-based):
```
User -> Backend API (/tx/deposit) -> Build Unsigned Tx -> Client Signs & Submits
On-Chain Program -> CPI to SPL Token -> Update Vault
Backend Confirm (/tx/confirm) -> Log to DB
Periodic Reconciliation -> Sync DB with On-Chain
```

## Prerequisites

- Rust 1.75+ (with async/await).
- Anchor 0.32.1.
- Solana CLI 2.3.0.
- PostgreSQL (local instance with DB `vault_db`, user `postgres`, password `password`).
- Node.js/Yarn for tests (if using TS tests).

## Setup Instructions

1. Clone the repo:
   ```
   git clone https://github.com/MRSKYWAY/collateral_vault.git
   cd collateral_vault
   ```

2. Install dependencies:
   ```
   cargo build
   anchor build
   ```

3. Set up PostgreSQL:
   - Create DB and user as per `backend/src/config.rs`.
   - Run the backend to init schema: `cd backend && cargo run`.

4. Start local Solana validator:
   ```
   solana-test-validator
   ```

5. Deploy the program:
   ```
   anchor deploy
   ```

6. Run backend:
   ```
   cd backend
   cargo run
   ```

## Smart Contract (On-Chain)

Located in `programs/collateral_vault/src/lib.rs`.

- **Instructions**:
  - `initialize_vault`: Creates PDA vault for user.
  - `deposit`: Transfers USDT to vault, updates balances, emits event.
  - `withdraw`: Transfers from vault if no locked balance, emits event.
  - `lock_collateral`: Locks amount for trading (CPI-authorized).
  - `unlock_collateral`: Unlocks after trade settlement.
  - `transfer_collateral`: Internal transfer between vaults.

- Build & Deploy: `anchor build && anchor deploy`.

## Backend Service (Off-Chain)

Located in `backend/src/`.

- Axum server with routes for vault queries, tx building (returns base64 unsigned tx for client signing), TVL, transactions.
- WebSockets at `/ws` for real-time TVL updates.
- Periodic reconciliation to sync DB with on-chain.

Run: `cargo run` (listens on 0.0.0.0:3000).

## Database Schema

Defined in `backend/src/db/schema.sql` (executed on init):

- `vaults`: Owner, PDA, balances, last_updated.
- `vault_transactions`: ID, owner, type, amount, signature, timestamp.
- `balance_snapshots`: Snapshots for auditing.
- `reconciliation_logs`: Discrepancy logs.

Migrations: Empty dir; schema loaded on startup.

## API Endpoints

- GET `/health`: Service status.
- GET `/vault/:owner`: Vault details (fetches from chain, upserts DB).
- GET `/vault/:owner/balance`: Balance info.
- GET `/vault/:owner/transactions`: Transaction history.
- GET `/tvl`: Total Value Locked.
- POST `/tx/deposit`: Build unsigned deposit tx (body: {owner, amount}).
- POST `/tx/withdraw`: Build unsigned withdraw tx.
- POST `/tx/lock`: Intent for lock (or build tx).
- POST `/tx/unlock`: Intent for unlock.
- POST `/tx/transfer`: Intent for transfer (body: {from, to, amount}).
- POST `/tx/confirm`: Log confirmed tx (body: {owner, event_type, amount, sig}).
- GET `/ws`: WebSocket for real-time updates.

Example: `curl -X POST http://localhost:3000/tx/deposit -H "Content-Type: application/json" -d '{"owner": "pubkey", "amount": 1000}'`

## Testing

- Anchor tests: `anchor test` (in root; covers instructions, security).
- Backend tests: `cd backend && cargo test`.
- Integration: See `tests/collateral_vault.ts` for TS-based tests.
- Demo Script: `scripts/collateral_demo.ts` (run with `ts-node` after setup).

## Deployment

- Deploy program to devnet/mainnet: Update `Anchor.toml` cluster, `anchor deploy --program-name collateral_vault`.
- Backend: Deploy to server (e.g., Heroku/Railway), set env for DATABASE_URL, RPC_URL.
- Monitoring: Use reconciliation logs for alerts.

## Security Analysis

- **Threat Model**: Unauthorized access, overflows, race conditions, CPI abuse.
- **Mitigations**: PDA ownership, authority checks, checked math, atomic instructions.
- **Attack Surface**: CPIs limited to authorized programs; backend rate-limited.
- **Best Practices**: Rent-exempt accounts, event emissions for auditing.

Test coverage: >80% (run `cargo tarpaulin` for report).

## Real-World Integration Example
ZKCG can be integrated into DeFi protocols for privacy-preserving verifications (e.g., credit score checks without revealing scores). See this demo in the [collateral_vault repository](https://github.com/MRSKYWAY/collateral_vault/blob/master/scripts/collateral_demo.ts), which shows the full on-chain + off-chain pipeline:
- **Off-Chain Proof Generation**: Generate a ZK proof using ZKCG's prover (Halo2 or zkVM) for conditions like "credit score > threshold".
- **Off-Chain Verification**: Call ZKCG's API (/v1/submit-proof) to verify the proof trustlessly.
- **On-Chain Settlement**: If verified, anchor the new state commitment on-chain (Solana program in collateral_vault) to approve loans or unlock collateral.
Run the demo: `ts-node collateral_demo.ts` (requires ZKCG API running locally).
This pipeline ensures fast off-chain processing (~340ms E2E for Halo2) with on-chain immutability.

## Contact
For questions, collaborations, or sponsorships, reach out:
- X (Twitter): [@sujyot](https://x.com/Sujyot10)
- GitHub Issues: Open in this repo for verifier discussions, or in [ZKCG private repo](https://github.com/MRSKYWAY/ZKCG) for prover/circuits.
