-- Default php_version changes from 'system' to '8.3' (the bundle we ship as default).
-- Existing rows that say 'system' map to whatever the highest installed version is at runtime,
-- so we don't backfill — the resolver layer handles the 'system' fallback.
-- This migration is a no-op on the table; it exists so the schema version moves forward
-- and we can add an index that future queries will benefit from.
CREATE INDEX IF NOT EXISTS sites_php_version_idx ON sites(php_version);
