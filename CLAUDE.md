# CLAUDE.md

This file gives AI assistants (Claude, Cursor, etc.) the architectural context they need to work productively in this repo without re-deriving every decision from scratch.

If you're a human reader, this is also a high-density "why we chose this" document. The detailed planning record lives in `docs/PLAN.md`, which is gitignored — keep CLAUDE.md as the public, version-controlled summary.

---

## What this project is

Delify Forge is a **Tauri 2 desktop application** that manages a local web development environment on macOS (Linux and Windows planned). It owns the lifecycle of web servers (Nginx, Apache, OpenLiteSpeed), language runtimes (PHP-FPM, Node.js), DNS routing for `*.test` domains, and — in later phases — databases and an API tester.

Think Laravel Herd, but open-source, multi-language, and multi-webserver.

---

## Stack

| Layer | Technology |
|-------|------------|
| Shell | Tauri 2 |
| Frontend | React 18 + TypeScript + Tailwind CSS 4 + shadcn/ui |
| Backend | Rust (tokio async, sqlx, tera, tracing, anyhow/thiserror) |
| Storage | SQLite (source of truth) + JSON (transient UI state) + macOS Keychain (secrets) |
| Package manager | pnpm |
| Version manager (target apps) | mise (shell-out) |

Bundle ID: `vn.delify.forge`. License: **AGPL-3.0-or-later**.

---

## Core architectural decisions

### 1. SQLite is the source of truth

All canonical state — sites, projects, settings, DB connections, cron jobs — lives in SQLite. Web server config files (nginx.conf, vhost files, OLS XML) are **generated artifacts** rendered by Tera templates. Never parse user-edited config files back into state. If the user wants to override config, the app exposes that override in the UI and stores it in SQLite, then re-renders.

This avoids the fragile comment-marker pattern aaPanel uses. The trade-off: if a user hand-edits a generated file, their edit is lost on next reload — by design.

### 2. Platform trait abstraction

Cross-platform is a day-one concern even though MVP only ships macOS. The Rust core defines traits for OS-coupled behavior:

```rust
src-tauri/src/platform/
├── mod.rs         // trait DnsManager, ProcessSupervisor, PathProvider
├── macos.rs       // impl for MVP
├── linux.rs       // stub: unimplemented!() — V1.0
└── windows.rs     // stub: unimplemented!() — V2.0
```

Domain logic stays OS-agnostic. The traits are the only place that knows about `osascript`, `launchd`, or `/etc/resolver/`.

### 3. Privilege model: osascript for MVP

Forge runs entirely as the user. The only operation requiring root is creating `/etc/resolver/test` during first-run setup. We trigger the native macOS password dialog via `osascript -e 'do shell script "..." with administrator privileges'`. The user types their password once and we never retain credentials.

A signed `SMAppService` privileged helper is a Phase 2 concern, blocked on getting an Apple Developer certificate. Until then: no `NOPASSWD` sudoers entries, no app-as-root, no privileged daemons we don't own.

### 4. Domain & DNS

We use `.test` (RFC 6761 reserved) — never `.local` (mDNS/Bonjour conflict) and never `.dev` (HSTS-preloaded by Google).

dnsmasq listens on **port 5353**, not 53. This avoids the root requirement for binding privileged ports. `/etc/resolver/test` contains:

```
nameserver 127.0.0.1
port 5353
```

macOS's per-TLD resolver mechanism makes this work without touching system DNS settings.

### 5. Web server strategy: aaPanel-inspired but cleaner

Mode B is the engine routing model from V0.3 onward:

- **Nginx** is always the gateway at port 80/443. It terminates every request and proxies to the appropriate backend.
- **Apache** binds plain HTTP at `127.0.0.1:8288`. Per-site engine selection routes through Nginx upstream config — a site with `web_server=apache` gets an Nginx `proxy_pass http://127.0.0.1:8288` block.
- **OpenLiteSpeed** is still deferred to V0.4+ pending the macOS build issue documented in `domain/bundle.rs:151`.

PHP-FPM is shared across all web servers — one pool per PHP version, exposed via Unix socket at `~/Library/Application Support/Forge/runtime/php/<version>.sock`.

### 6. Binary distribution

MVP requires the user to `brew install nginx php`. We detect existing binaries on `PATH` and Homebrew prefixes. From V0.3, Forge will download prebuilt binaries on demand to `~/Library/Application Support/Forge/engines/<engine>/<version>/`, following the DBngin pattern.

We do not build engines from source on the user's machine. CI handles all binary production.

### 7. First-run wizard is non-dismissible and idempotent

On every launch we run a silent health check (< 500ms). If it fails — engines missing, ports conflicting, resolver gone — the wizard appears and cannot be closed. The wizard is safe to run any number of times: every step is idempotent, so re-running after a partial failure is a recovery action, not a corruption risk.

### 8. AGPLv3 implications for the codebase

- Every source file must carry a brief AGPL header (we'll automate this).
- Dependencies must be license-compatible: MIT, BSD, Apache 2.0, LGPL (dynamic link), MPL, AGPL itself. **No GPLv2-only deps**, no proprietary deps.
- We do not copy code from other AGPL projects (notably TablePro for the future DB GUI). We learn from patterns and reimplement clean-room.

---

## Directory layout (planned)

```
forge/
├── README.md            # English, public
├── README.vi.md         # Vietnamese, public
├── LICENSE              # AGPLv3 full text
├── CHANGELOG.md         # Keep a Changelog 1.1.0
├── CLAUDE.md            # this file
├── CONTRIBUTING.md      # contribution policy
│
├── package.json
├── pnpm-lock.yaml
├── vite.config.ts
├── tsconfig.json
├── tailwind.config.ts
├── components.json      # shadcn config
│
├── src/                 # React frontend
│   ├── main.tsx
│   ├── App.tsx
│   ├── components/
│   │   ├── Sidebar.tsx
│   │   ├── Wizard/      # first-run wizard
│   │   └── ui/          # shadcn primitives
│   ├── pages/
│   │   ├── General.tsx
│   │   ├── Sites.tsx
│   │   ├── PHP.tsx
│   │   ├── Services.tsx
│   │   └── About.tsx
│   ├── lib/
│   │   ├── tauri.ts     # IPC wrappers
│   │   └── utils.ts
│   └── types/
│
├── src-tauri/           # Rust backend
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   ├── build.rs
│   ├── icons/
│   └── src/
│       ├── main.rs
│       ├── lib.rs
│       ├── commands/    # Tauri IPC handlers
│       │   ├── mod.rs
│       │   ├── sites.rs
│       │   ├── system.rs
│       │   └── wizard.rs
│       ├── domain/      # OS-agnostic business logic
│       │   ├── sites.rs
│       │   ├── nginx.rs
│       │   ├── php.rs
│       │   └── dns.rs
│       ├── platform/    # OS-specific traits + impls
│       │   ├── mod.rs
│       │   ├── macos.rs
│       │   ├── linux.rs
│       │   └── windows.rs
│       ├── store.rs     # SQLite persistence (sqlx)
│       ├── templates/   # Tera config templates
│       └── error.rs
│
├── docs/                # GITIGNORED — local-only
└── openspec/            # GITIGNORED — local-only specs
```

---

## Conventions

### Code style

- Rust: `cargo fmt` + `cargo clippy --all-targets --all-features -- -D warnings`. 4-space indent.
- TypeScript: ESLint + Prettier. 4-space indent. LF line endings. No trailing commas.
- No comments unless the **why** is non-obvious. The code says what; the comment explains why.

### Commits

Conventional Commits 1.0.0:

```
feat(scope): short description
fix(scope): short description
chore(scope): short description
refactor(scope): short description
```

Common scopes: `ui`, `sites`, `nginx`, `php`, `dns`, `wizard`, `system`, `store`, `platform`.

### Tags

- Releases: `v<major>.<minor>.<patch>`
- Pre-release: `v0.0.1-mvp`, `v0.1.0-rc.1`

### Branches

- `main` is protected.
- Feature branches: `feat/<scope>`, `fix/<scope>`.

---

## What lives outside this repo (and why)

Two directories are deliberately gitignored:

- **`docs/`** — long-form planning notes (`docs/PLAN.md`, decision logs, brainstorm transcripts). They are useful locally but would clutter the public repo and become stale.
- **`openspec/`** — OpenSpec proposals and tasks for in-flight changes. Per-developer workflow artifact.

The public repo carries summaries (this file, README, CHANGELOG) — the source of truth for *external* readers. Internal long-form context stays local.

---

## When working on this codebase

1. **Read `docs/PLAN.md`** if it exists locally — it has the full decision log with dates and rationales.
2. **Don't reinvent.** Use mise for version management, prebuilt binaries for engines, Bruno for the API tester. Forge orchestrates; it does not re-implement.
3. **Source of truth is SQLite.** Render configs from SQLite, do not parse configs back.
4. **Platform-couple code goes in `src-tauri/src/platform/`.** Domain logic stays OS-agnostic.
5. **AGPL means clean-room.** Especially for the future DB GUI — do not look at TablePro source code while implementing.
6. **Idempotency first.** Wizard steps, DNS setup, config generation — all must be safe to re-run.
7. **First-run UX matters.** Forge competes with Herd's polish. Detect, suggest, and one-click-fix wherever possible.

---

## Roadmap pointer

Phase milestones (MVP, V0.2, V0.3, …, V2.0) are listed in [README.md](README.md#roadmap). Each phase has its own OpenSpec proposal locally (gitignored).

---

**Last updated:** 2026-05-23. When architectural decisions change, update this file in the same commit.
