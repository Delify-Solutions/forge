# Changelog

All notable changes to **Delify Forge** will be documented in this file.

The format is based on [Keep a Changelog 1.1.0](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning 2.0.0](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Alias domains per site: each site can have additional `.test` hostnames stored in a new `site_domains` table (migration `0004_site_domains`) and rendered into the same Nginx `server_name` directive. New Tauri commands `add_site_alias` / `remove_site_alias` and a Sites page "Aliases" column with a manage dialog (EN + VI). Cascade-deletes when the parent site is removed.

### Changed
- Clippy-clean codebase: `process::disclaim_trampoline` uses `Iterator::find`, and `forge-disclaim` uses a `c"..."` literal for the dlsym name.

## [0.0.2-mvp] — 2026-05-26

### Added
- Multi-PHP-version backend with one PHP-FPM pool per detected `<major>.<minor>` line, sockets at `runtime/php/<line>.sock`.
- Per-site PHP version picker in the Sites page.
- Per-site web_server picker (Nginx default; Apache/OLS catalog entries reserved for V0.3+).
- On-demand engine bundle installer with pinned arm64 sha256 hashes for dnsmasq, nginx, and PHP, downloaded from the `forge-engines` release artifacts.
- English + Vietnamese i18n bundles for the sidebar, wizard, and Services page.
- DNS port setting persisted in SQLite (migration `0003_dns_port`), editable from the wizard, and validated against the macOS resolver file.
- Default landing page (`index.php`) generated for new empty sites — white/blue Delify Forge intro with a one-click `phpinfo()` and a link back to `https://github.com/Delify-Solutions/forge`.
- Auto document-root detection: when a site has `public/index.php` (or `.html`/`.htm`), Nginx serves from `public/` so Laravel-style projects work out of the box.
- "Open wizard" entry point on the Services page so the first-run flow can be re-triggered after the initial setup.
- `forge-disclaim` trampoline binary for spawning supervised engines without inheriting the Tauri parent's TCC permissions.

### Changed
- First-run wizard blocks the **Continue** button on the DNS step until DNS setup actually succeeds (`setupDnsResolver` + `startDnsmasq` + `startPhpFpm` + `startNginx`), preventing users from finishing the wizard while DNS/resolver is unconfigured.
- Nginx supervisor cleans up orphan PID files and any stray `nginx` listener on port 80 before spawning, fixing the "old master kept port 80" failure when the previous app session did not exit cleanly.
- Site config rendering uses quoted paths (`root "..."`, `fastcgi_pass "unix:..."`, `include "..."`) so document roots and runtime paths with spaces render correctly.

### Fixed
- "No input file specified" on Laravel-style projects: Nginx now points at `public/` automatically when an entrypoint is present there, instead of forcing the project root.
- Wizard skip-DNS regression that left the wizard closed even though no resolver had been written.

## [0.0.1-mvp] — 2026-05-23

### Added
- Repository foundation documents: `README.md`, `README.vi.md`, `CLAUDE.md`, `CONTRIBUTING.md`, `CHANGELOG.md`.
- Tauri 2 + React 18 + TypeScript + Tailwind CSS 4 + shadcn-style UI primitives (Button, Dialog, Input).
- Sidebar layout with five navigation surfaces (General, Sites, PHP, Services, About) inspired by Laravel Herd.
- Non-dismissible six-step first-run wizard: Welcome → System scan → Choose source → Resolve conflicts → Setup DNS → Done.
- Real `scan_system` Tauri command on macOS that detects Homebrew, Nginx, PHP, PHP-FPM, port 80/443/5353 status, and `/etc/resolver/test` presence.
- SQLite-backed site persistence (sqlx migrations) with kebab-case validation and Tauri commands `list_sites`, `add_site`, `remove_site`.
- Sites page with empty state, table view, and a Tauri folder-picker Add Site dialog.
- DNS resolver setup via osascript (native macOS admin prompt) — idempotent: skipped when `/etc/resolver/test` already matches.
- dnsmasq lifecycle on the unprivileged port 5353, configured to route `*.test` to `127.0.0.1`.
- Nginx config generation from SQLite via Tera templates, with stale per-site config cleanup and `nginx -s reload` after add/remove site.
- PHP-FPM single-pool lifecycle sharing a Unix socket with Nginx.
- Services page with live state polling and Start/Stop controls for dnsmasq, Nginx, and PHP-FPM.
- Process supervisor (`tokio::process::Child`) with start/stop/status/shutdown_all, hooked into Tauri's `RunEvent::ExitRequested` so child processes die with the app.
- Cross-platform-ready architecture: `platform::macos` is implemented; `platform::linux` and `platform::windows` are stubs returning `ForgeError::NotImplemented`.
- Rust unit tests for kebab-case validation, dnsmasq constants, and Nginx template rendering.

### Notes
- macOS 14+ only. Linux is on the V1.0 roadmap, Windows on V2.0.
- PHP versioning, alias domains, project scaffolding, database management, API tester, and the cron tab arrive in V0.2 onward.

[Unreleased]: https://github.com/Delify-Solutions/forge/compare/v0.0.2-mvp...HEAD
[0.0.2-mvp]: https://github.com/Delify-Solutions/forge/compare/v0.0.1-mvp...v0.0.2-mvp
[0.0.1-mvp]: https://github.com/Delify-Solutions/forge/releases/tag/v0.0.1-mvp
