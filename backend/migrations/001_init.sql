-- Vaults table
CREATE TABLE IF NOT EXISTS vaults (
    id TEXT PRIMARY KEY,
    owner_pubkey TEXT NOT NULL UNIQUE,
    vault_pda TEXT NOT NULL,
    token_account TEXT NOT NULL,
    total_balance INTEGER NOT NULL,
    locked_balance INTEGER NOT NULL,
    available_balance INTEGER NOT NULL,
    created_at TEXT NOT NULL
);

-- Vault transaction events (derived from on-chain events)
CREATE TABLE IF NOT EXISTS vault_transactions (
    id TEXT PRIMARY KEY,
    vault_pda TEXT NOT NULL,
    tx_signature TEXT NOT NULL,
    event_type TEXT NOT NULL,
    amount INTEGER NOT NULL,
    timestamp TEXT NOT NULL
);
