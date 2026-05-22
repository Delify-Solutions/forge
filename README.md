<div align="center">

# Delify Forge

**A native local web development environment for modern stacks.**

[![License: AGPL v3](https://img.shields.io/badge/License-AGPL_v3-blue.svg)](https://www.gnu.org/licenses/agpl-3.0)
[![Platform: macOS](https://img.shields.io/badge/Platform-macOS_14%2B-lightgrey.svg)](#)
[![Status: Alpha](https://img.shields.io/badge/Status-Alpha-orange.svg)](#)

[English](README.md) · [Tiếng Việt](README.vi.md)

</div>

---

## What is Delify Forge

Delify Forge is an open-source desktop application that turns your local machine into a fast, reliable web development environment. It manages the engines (web servers, language runtimes, databases) and the wiring between them (DNS, sockets, configs) so you can focus on shipping projects.

It's built for developers who want **the polish of Laravel Herd, the breadth of XAMPP, and the openness of a community-owned tool** — all in a single native app.

## Why another local dev tool

| Tool | Open source | Multi-language | Multi-webserver | Native | Pricing |
|------|:-:|:-:|:-:|:-:|:-:|
| **Delify Forge** | ✅ AGPLv3 | ✅ | ✅ | ✅ Tauri | Free |
| Laravel Herd | ❌ | ❌ PHP-only | Limited | ✅ | Free + Pro |
| XAMPP | Mixed | ❌ PHP-only | ❌ | Bundled | Free |
| Laravel Valet | ✅ MIT | ❌ PHP-only | ❌ Nginx-only | ✅ CLI | Free |
| MAMP | ❌ | ❌ | ❌ | ✅ | Free + Pro |

Delify Forge fills the gap: an open-source, native, multi-language, multi-webserver dev environment.

## Status

**Pre-MVP.** This repo is being scaffolded. The first releasable build (`v0.0.1-mvp`) targets macOS 14+ with Nginx and a single PHP version. See the [roadmap](#roadmap) below.

## Planned features

### MVP
- macOS 14+ support
- Add a project from a folder
- Auto-routed `*.test` domains via dnsmasq + `/etc/resolver/test`
- Nginx + PHP-FPM lifecycle managed by the app
- Sidebar UI inspired by Laravel Herd, dark theme by default

### Roadmap

| Phase | Highlights |
|-------|------------|
| **MVP** | Nginx, single PHP version, `.test` domains, add/list/remove sites |
| **V0.2** | Multi PHP versions via mise, alias domains, project scaffolding (PHP/Laravel) |
| **V0.3** | Apache, OpenLiteSpeed, PHP extensions manager, bundled binary downloads |
| **V0.4** | Node.js, framework templates (Next, Vite), Nginx-as-gateway proxy mode |
| **V0.5** | Database manager (DBngin-style spawn for MySQL, MariaDB, PostgreSQL, Redis...) |
| **V0.6** | Database GUI (clean-room TablePro-style implementation) |
| **V0.7** | API tester (Bruno-based), cron tab |
| **V1.0** | Polish, Linux support |
| **V2.0** | Windows support, AI features, plugin system |

## Tech stack

- **Tauri 2** with a Rust core and a WebView frontend per OS
- **React 18 + TypeScript + Tailwind CSS 4** with **shadcn/ui**
- **SQLite** as source of truth, **Tera** for config generation
- **mise** as the language version manager
- **Bruno** (planned) for the API tester

## Installation

> Builds are not yet published. This section will be updated when `v0.0.1-mvp` is tagged.

When MVP ships, you will be able to:

```bash
# via Homebrew Cask (planned)
brew install --cask delify-forge

# or download a DMG from the releases page
```

For now, run from source:

```bash
git clone https://github.com/Delify-Solutions/forge
cd forge
pnpm install
pnpm tauri dev
```

You will need: Rust (1.78+), Node.js (20+), pnpm (8+), and Homebrew with `nginx` and `php` available on `PATH` for the MVP.

## Architecture overview

```
┌──────────────────────────────────────────────────────┐
│ Tauri WebView (React + TypeScript + Tailwind)        │
│  Sidebar │ Sites │ PHP │ Services │ About            │
└────────────┬─────────────────────────────────────────┘
             │ Tauri IPC
┌────────────▼─────────────────────────────────────────┐
│ Rust core (tokio async runtime)                      │
│                                                      │
│  ┌────────────┐  ┌──────────┐  ┌─────────────────┐  │
│  │  SQLite    │  │  Tera    │  │  Platform trait │  │
│  │  store     │──▶ templates│──▶ DnsManager      │  │
│  │  (truth)   │  │          │  │ ProcessSupervisor│ │
│  └────────────┘  └──────────┘  │ PathProvider    │  │
│                                 └────────┬────────┘  │
│                                          │           │
│                            ┌─────────────┼───────────┴─┐
│                            │  macos.rs (impl)         │
│                            │  windows.rs (stub)       │
│                            │  linux.rs (stub)         │
│                            └─────────────┬─────────────┘
└──────────────────────────────────────────┼─────────────
                                            │
                              spawn / supervise / signal
                                            ▼
                       ┌────────────────────────────────┐
                       │ Nginx │ Apache │ OLS │ PHP-FPM │
                       │ dnsmasq @ :5353               │
                       └────────────────────────────────┘
```

The cross-platform structure is in place from day one, even though MVP only ships the macOS implementation.

## Privilege model

Delify Forge needs administrative access exactly once during first-run setup, to write `/etc/resolver/test`. The MVP uses `osascript` to trigger the native macOS password prompt — the same pattern as Laravel Herd, just without an Apple Developer-signed helper. Sudo is not retained, and the app itself runs entirely in user space.

A signed `LaunchDaemon` privileged helper is on the roadmap for a later phase, once the project has an Apple Developer certificate.

## Contributing

Issues, ideas, and pull requests are welcome. Please read [CONTRIBUTING.md](CONTRIBUTING.md) before submitting changes — Delify Forge is licensed under **AGPLv3**, which has implications for derivative works.

## License

Copyright (C) 2026 Delify Solutions.

Delify Forge is free software: you can redistribute it and/or modify it under the terms of the **GNU Affero General Public License**, version 3 or (at your option) any later version, as published by the Free Software Foundation. See [LICENSE](LICENSE) for the full text.

This program is distributed in the hope that it will be useful, but **WITHOUT ANY WARRANTY**; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.

## Acknowledgements

Delify Forge stands on the shoulders of prior art. We learned from — but did not copy code from — these projects:

- **Laravel Herd** for UX and the privileged-helper pattern
- **Laravel Valet** for the catch-all nginx + driver model
- **XAMPP** for the bundled-engine concept
- **TablePro** for source-of-truth and plugin architecture inspiration
- **DBngin** for on-demand database engine spawning
- **aaPanel** for the multi-webserver gateway pattern

— **Delify Solutions**
