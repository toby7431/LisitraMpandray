-- ─── Paramètres de l'application ─────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS settings (
    key   TEXT PRIMARY KEY,
    value TEXT NOT NULL
);

-- ─── Journal d'audit des modifications de contributions ────────────────────────
-- Chaque ligne = un champ modifié sur une contribution.
-- `field` : 'amount' | 'period' | 'payment_date'
CREATE TABLE IF NOT EXISTS contribution_audits (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    contribution_id INTEGER NOT NULL,
    field           TEXT    NOT NULL,
    old_value       TEXT    NOT NULL,
    new_value       TEXT    NOT NULL,
    changed_at      TEXT    NOT NULL,
    reason          TEXT,
    FOREIGN KEY (contribution_id) REFERENCES contributions(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_audits_contribution_id
    ON contribution_audits(contribution_id);
