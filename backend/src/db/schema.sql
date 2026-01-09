CREATE TABLE IF NOT EXISTS vaults (
    owner TEXT PRIMARY KEY,
    vault_pda TEXT NOT NULL,
    total_balance INTEGER NOT NULL,
    locked_balance INTEGER NOT NULL,
    available_balance INTEGER NOT NULL,
    last_updated INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS vault_transactions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    owner TEXT NOT NULL,
    tx_type TEXT NOT NULL, -- deposit / withdraw / lock / unlock / transfer
    amount INTEGER NOT NULL,
    signature TEXT,
    timestamp INTEGER NOT NULL
);
