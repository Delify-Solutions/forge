# Contributing to Delify Forge

Thanks for your interest in helping build Delify Forge. This document explains how the project handles licensing, what the workflow looks like, and how to submit changes that we can merge.

If anything below feels unclear, open an issue or a draft PR with your question — that's a contribution too.

---

## Code of conduct

Be kind. Assume good intent. Critique ideas, not people. We follow a relaxed version of the Contributor Covenant; egregious behavior gets a warning then a ban.

---

## License & contributor agreement

Delify Forge is licensed under the **GNU Affero General Public License v3.0 or later (AGPLv3+)**. By submitting a contribution, you agree that:

1. **Your contribution will be released under AGPLv3+.** You can keep using your code anywhere else under whatever terms you like — the contribution itself just needs to be AGPL-compatible inside this repo.
2. **You have the right to submit it.** Either you wrote it yourself, or you're submitting code that was already AGPL-compatible (MIT, BSD, Apache 2.0, MPL, LGPL with proper handling, AGPL itself). Don't paste in code from sources you don't have rights to.
3. **You sign off your commits.** We use the [Developer Certificate of Origin (DCO) 1.1](https://developercertificate.org/). Append `Signed-off-by: Your Name <you@example.com>` to commit messages — `git commit -s` does this automatically. PRs without sign-off will be asked to amend.

We do **not** use a CLA. The DCO is enough. We do not relicense contributions away from AGPL.

### What you must not contribute

- **Code copied from AGPL projects we explicitly avoid** — notably TablePro for the future database GUI work. We deliberately do clean-room implementations there, and your contribution should match.
- **Code under GPLv2-only**, BUSL, SSPL, Commons Clause, or similar non-AGPL-compatible licenses. AGPLv3 is one-way compatible with GPLv3+ but not GPLv2-only.
- **Proprietary or vendored binaries without source.** Any binary we bundle (Nginx, PHP, etc.) must be reproducibly buildable from a public source repo.

When in doubt, ask before opening the PR.

---

## How to set up your environment

Prerequisites:

- macOS 14+ (the only supported development platform for now)
- **Rust** 1.78+ (`rustup install stable`)
- **Node.js** 20+ and **pnpm** 8+ (`corepack enable` works)
- **Homebrew** with `nginx` and `php` available on `PATH` for end-to-end testing

```bash
git clone https://github.com/Delify-Solutions/forge
cd forge
pnpm install
pnpm tauri dev
```

The first build is slow — Tauri compiles the Rust core fresh. Subsequent builds incremental.

---

## Workflow

### Branches

- `main` is protected. All changes land via pull request.
- Feature branches: `feat/<scope>`
- Bug fixes: `fix/<scope>`
- Refactors: `refactor/<scope>`
- Chores: `chore/<scope>`

Common scopes: `ui`, `sites`, `nginx`, `php`, `dns`, `wizard`, `system`, `store`, `platform`.

### Commits

We use [Conventional Commits 1.0.0](https://www.conventionalcommits.org/en/v1.0.0/).

```
feat(sites): add multi-domain alias support
fix(dns): handle dnsmasq port conflict on first run
chore: bump tauri to 2.1.0
refactor(platform): extract DnsManager trait
docs: clarify privilege model in CLAUDE.md
```

Sign-offs are mandatory:

```bash
git commit -s -m "feat(sites): add multi-domain alias support"
```

### Pull requests

1. **Open a PR against `main`.**
2. **Title** uses the same Conventional Commits format as commits.
3. **Description** answers: what changed, why, how to test, what's NOT changed.
4. **Keep PRs focused.** One concern per PR. Refactors that ride along with a feature should be split.
5. **Add or update tests** for behavior changes.
6. **Update the changelog.** Add an entry under `[Unreleased]` in `CHANGELOG.md` describing user-visible changes.
7. **Wait for CI** to go green before requesting review.

---

## Code style

### Rust

- Format: `cargo fmt --all`
- Lint: `cargo clippy --all-targets --all-features -- -D warnings`
- Indentation: 4 spaces
- Comments: only when the *why* is non-obvious. The code already says *what*.
- AGPL header: every new `.rs` file gets a short SPDX header (script will be added; for now copy from existing files once they exist).

### TypeScript / React

- Lint: `pnpm lint` (ESLint + Prettier)
- Indentation: 4 spaces
- Line endings: LF
- No trailing commas
- Components: functional, hooks-only. No class components.
- Imports: absolute from `@/` alias for `src/`.

### General

- Don't refactor unrelated code in the same PR.
- Don't add abstractions for "future flexibility" — wait for the second use case.
- Don't catch errors just to log and rethrow — let them bubble unless you're adding context.

---

## Testing

- **Rust:** `cargo test --all`
- **Frontend:** `pnpm test`
- **End-to-end:** documented separately in `docs/testing.md` once the test harness lands.

For features that touch DNS, web server lifecycle, or process supervision, add integration tests that exercise the real OS interfaces (with a non-default port + sandboxed paths).

---

## What we're working on

The current focus is the **MVP** (`v0.0.1-mvp`): macOS-only, Nginx + single PHP version, `.test` domains, add/list/remove sites. See the roadmap in [README.md](README.md#roadmap) for the longer arc.

Good first issues will be tagged `good-first-issue` once the MVP is stable. For now, the most useful contributions are:

- **Reviewing `CLAUDE.md` and surfacing gaps or contradictions**
- **Trying the dev build on different macOS versions and reporting compatibility issues**
- **Documentation clarity** — the README and CLAUDE.md are still rough

---

## Reporting bugs and security issues

### Bugs

Use [GitHub Issues](https://github.com/Delify-Solutions/forge/issues). Include:

- macOS version
- Forge version (`Forge → About`)
- Steps to reproduce
- What you expected vs. what happened
- Logs from `~/Library/Logs/DelifyForge/` if the issue is runtime-related

### Security

**Do not open a public issue for security vulnerabilities.** Email the maintainers directly (contact details to be added when the maintainer team is finalized) with:

- Description of the vulnerability
- Steps to reproduce
- Potential impact

We'll acknowledge within 7 days and coordinate disclosure timing with you.

---

## Questions?

Open a [discussion](https://github.com/Delify-Solutions/forge/discussions) if it's open-ended. Open an [issue](https://github.com/Delify-Solutions/forge/issues) if it's actionable. Both are welcome.

— **Delify Solutions**
