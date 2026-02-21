-- ─── Membres de l'église ──────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS members (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    card_number TEXT    NOT NULL UNIQUE,          -- Numéro de carte unique
    full_name   TEXT    NOT NULL,
    address     TEXT,
    phone       TEXT,
    job         TEXT,
    gender      TEXT    NOT NULL DEFAULT 'M',     -- 'M' | 'F'
    member_type TEXT    NOT NULL DEFAULT 'Communiant', -- 'Communiant' | 'Cathekomen'
    created_at  TEXT    NOT NULL
);

-- ─── Contributions / cotisations ──────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS contributions (
    id            INTEGER PRIMARY KEY AUTOINCREMENT,
    member_id     INTEGER NOT NULL,
    payment_date  TEXT    NOT NULL,               -- 'YYYY-MM-DD'
    period        TEXT    NOT NULL,               -- ex. '2024-01', 'T1-2025'
    amount        TEXT    NOT NULL DEFAULT '0',   -- Decimal stocké en TEXT
    recorded_year INTEGER NOT NULL,               -- extrait automatiquement de payment_date
    FOREIGN KEY (member_id) REFERENCES members(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_contributions_member_id
    ON contributions(member_id);

CREATE INDEX IF NOT EXISTS idx_contributions_recorded_year
    ON contributions(recorded_year);

-- ─── Résumés annuels ──────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS year_summaries (
    year      INTEGER PRIMARY KEY,
    total     TEXT    NOT NULL DEFAULT '0',       -- Decimal stocké en TEXT
    closed_at TEXT,                               -- NULL = année ouverte
    note      TEXT
);
