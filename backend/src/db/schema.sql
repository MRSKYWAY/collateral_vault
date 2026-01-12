CREATE TABLE IF NOT EXISTS vaults (
    owner TEXT PRIMARY KEY,
    vault_pda TEXT NOT NULL,
    total_balance BIGINT NOT NULL,
    locked_balance BIGINT NOT NULL,
    available_balance BIGINT NOT NULL,
    last_updated TIMESTAMP WITH TIME ZONE NOT NULL
);

CREATE TABLE IF NOT EXISTS vault_transactions (
    id SERIAL PRIMARY KEY,
    owner TEXT NOT NULL,
    tx_type TEXT NOT NULL, -- deposit / withdraw / lock / unlock / transfer
    amount BIGINT NOT NULL,
    signature TEXT,
    timestamp TIMESTAMP WITH TIME ZONE NOT NULL
);

-- Bonus tables for snapshots and logs
CREATE TABLE IF NOT EXISTS balance_snapshots (
    id SERIAL PRIMARY KEY,
    vault_owner TEXT NOT NULL,
    total_balance BIGINT NOT NULL,
    locked_balance BIGINT NOT NULL,
    available_balance BIGINT NOT NULL,
    snapshot_at TIMESTAMP WITH TIME ZONE NOT NULL
);

CREATE TABLE IF NOT EXISTS reconciliation_logs (
    id SERIAL PRIMARY KEY,
    vault_owner TEXT NOT NULL,
    discrepancy TEXT,
    resolved BOOLEAN DEFAULT FALSE,
    logged_at TIMESTAMP WITH TIME ZONE NOT NULL
);


CREATE INDEX IF NOT EXISTS idx_vaults_owner ON vaults(owner);
CREATE INDEX IF NOT EXISTS idx_transactions_owner ON vault_transactions(owner);

-- Audit trail 
CREATE TABLE IF NOT EXISTS audit_trail (
    id SERIAL PRIMARY KEY,
    action TEXT NOT NULL,       
    actor TEXT,                  
    details JSONB,
    timestamp TIMESTAMPTZ NOT NULL
);
