CREATE TABLE IF NOT EXISTS site_domains (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    site_id    INTEGER NOT NULL REFERENCES sites(id) ON DELETE CASCADE,
    domain     TEXT    NOT NULL UNIQUE,
    is_alias   INTEGER NOT NULL DEFAULT 1,
    created_at TEXT    NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_site_domains_site_id ON site_domains(site_id);
