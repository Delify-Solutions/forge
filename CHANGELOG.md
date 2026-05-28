# Changelog

All notable changes to **Delify Forge** will be documented in this file.

The format is based on [Keep a Changelog 1.1.0](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning 2.0.0](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.0.5-mvp] — 2026-05-28

### Added
- Per-site Apache backend: sites can pick `web_server=apache`. Nginx terminates every request at port 80/443 and proxies to Apache at `127.0.0.1:8288`. Apache shares the existing PHP-FPM Unix socket for PHP execution via `mod_proxy_fcgi`.
- Lazy install of the Apache 2.4.62 bundle: triggered only when the user first picks `engine=apache` in the Sites table engine select. Reuses the existing `installBundle` channel pattern.
- Services page Apache row with Start / Stop / state / PID display, mirroring the Nginx and PHP-FPM rows. When the Apache bundle is not yet installed, the row offers an inline **Install** button instead of Start so the action can never fail with a missing-bundle error.
- Sites page warning banner: when Apache is stopped and one or more sites use Apache, an amber banner with a one-click Start button surfaces the dependency.
- New Tera templates `httpd.conf.tera` (master config) and `apache.vhost.conf.tera` (per-site vhost), with `mod_proxy_fcgi` routing PHP requests via `<FilesMatch \.php$> SetHandler "proxy:unix:..."`.
- Composer 2.9.8 bundle: the Laravel template no longer requires `brew install composer`. The Add Site dialog detects the bundle, falls back to PATH, and offers an in-app install when missing. Scaffolder runs the PHAR via the bundled PHP CLI so it works on a stock macOS install.

### Changed
- CLAUDE.md decision 5 updated to describe the realised Mode B architecture: Nginx is always the gateway from V0.3 onward; Apache binds at `127.0.0.1:8288`; OLS remains deferred to V0.4+.

### Fixed
- Services page error banner now surfaces the actual backend message (e.g. "apache bundle not installed") instead of a generic "Operation failed". Tauri rejects with a raw string rather than an `Error` instance, so the UI now handles both shapes.

## [0.0.4-mvp] — 2026-05-28

### Added
- "Add Site" dialog gains a **Template** picker (None / Plain PHP / Static HTML / Laravel). Plain PHP and Static HTML write a bundled boilerplate; Laravel runs `composer create-project laravel/laravel <path> --prefer-dist` synchronously with a 120-second timeout. Folders Forge creates are rolled back on failure. Composer is detected via the existing `detect_binary` pattern; the Laravel option is disabled with an inline banner when composer is missing.
- General page is now a real screen with **Preferred tools** selects for Editor and Terminal, backed by a curated catalog of 10 editors and 8 terminals. Selection persists in the existing `settings` table. `open_site_in_editor` and `open_site_terminal` dispatch through a resolver that prefers CLI, then `.app` bundle, then falls back to the existing auto chain. iTerm2 uses its own AppleScript dialect; Warp / Tabby / Alacritty / Kitty / WezTerm / Ghostty launch via `open -na "<App>" --args ...` with each tool's documented cwd flag.
- 20 new Rust unit tests covering scaffold templates, composer detection, and the editor/terminal resolver (46 unit tests total).

### Notes
- V0.2 roadmap milestone closes here: multi-PHP (v0.0.2-mvp), alias domains (v0.0.3-mvp), and project scaffolding (this release). HTTPS+CI, per-site quick actions, search, and preferred tools shipped along the way as bonus features beyond the roadmap.

## [0.0.3-mvp] — 2026-05-27

### Added
- Per-site "Open in Terminal" quick action: each site row gets a Terminal button that opens Terminal.app with the working directory `cd`'d to the site path. Path is resolved from SQLite, never from frontend input.
- Per-site HTTPS via mkcert: each site can opt into locally trusted HTTPS with certificates covering the primary `.test` domain and aliases.
- Sites page HTTPS switch and mkcert banner for missing mkcert or local CA setup.
- GitHub Actions CI workflow on `macos-latest` for Rust formatting, clippy, tests, and frontend build.
- Per-site quick actions: Open in browser, Reveal in Finder, Open in editor (VS Code / Cursor / Sublime), and View logs. The logs modal shows the last 200 lines of Nginx error and access logs with a refresh button. All actions resolve the site path from SQLite and never trust frontend-provided paths.
- Alias domains per site: each site can have additional `.test` hostnames stored in a new `site_domains` table (migration `0004_site_domains`) and rendered into the same Nginx `server_name` directive. New Tauri commands `add_site_alias` / `remove_site_alias` and a Sites page "Aliases" column with a manage dialog (EN + VI). Cascade-deletes when the parent site is removed.

### Changed
- Clippy-clean codebase: `process::disclaim_trampoline` uses `Iterator::find`, and `forge-disclaim` uses a `c"..."` literal for the dlsym name.
- Sites page now includes a case-insensitive search field for names, domains, paths, and aliases.
- Logs dialog tabs now follow the ARIA tab pattern with keyboard navigation and a focusable panel.
- Log tailing now distinguishes missing files from permission, seek, and read errors so unreadable logs are surfaced to users.
- Sites table now keeps the row action buttons (including delete) visible on narrow viewports by truncating the path column and allowing horizontal scroll when needed.
- `Cargo.toml` declares `default-run = "delify-forge"` so `cargo run` is unambiguous now that the workspace ships a second binary (`forge-disclaim`).

### Fixed
- Editor detection no longer fails when the Tauri app bundle inherits launchd's minimal PATH: detection now augments PATH with `/opt/homebrew/bin`, `/usr/local/bin`, `/opt/local/bin`, and per-user bin directories, and falls back to `open -a "<App>"` for VS Code / Cursor / Sublime when only the `.app` bundle is present.

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

[Unreleased]: https://github.com/Delify-Solutions/forge/compare/v0.0.4-mvp...HEAD
[0.0.4-mvp]: https://github.com/Delify-Solutions/forge/compare/v0.0.3-mvp...v0.0.4-mvp
[0.0.3-mvp]: https://github.com/Delify-Solutions/forge/compare/v0.0.2-mvp...v0.0.3-mvp
[0.0.2-mvp]: https://github.com/Delify-Solutions/forge/compare/v0.0.1-mvp...v0.0.2-mvp
[0.0.1-mvp]: https://github.com/Delify-Solutions/forge/releases/tag/v0.0.1-mvp
