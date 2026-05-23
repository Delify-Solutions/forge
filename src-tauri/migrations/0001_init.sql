CREATE TABLE IF NOT EXISTS sites (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    name        TEXT    NOT NULL UNIQUE,
    path        TEXT    NOT NULL,
    php_version TEXT    NOT NULL DEFAULT 'system',
    web_server  TEXT    NOT NULL DEFAULT 'nginx',
    created_at  TEXT    NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS sites_name_idx ON sites(name);

CREATE TABLE IF NOT EXISTS settings (
    key   TEXT PRIMARY KEY,
    value TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS engines_detected (
    engine     TEXT PRIMARY KEY,
    binary     TEXT NOT NULL,
    version    TEXT NOT NULL,
    source     TEXT NOT NULL,
    last_seen  TEXT NOT NULL DEFAULT (datetime('now'))
);
