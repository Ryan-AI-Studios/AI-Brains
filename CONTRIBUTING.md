# Contributing to AI-Brains

Thanks for helping improve AI-Brains. This guide is **contributor hygiene** (build, gate, process) — not a second product handbook. Operator install and ops live under `Docs/`.

## License

AI-Brains is licensed under the **PolyForm Noncommercial License 1.0.0** (`LICENSE`), with additional permissions for qualified small entities in `COMMERCIAL-EXCEPTION.md`. Contact: legal@ledgerful.dev.

By contributing, you agree that your contributions are provided under the same project license terms unless a separate agreement says otherwise.

## Prerequisites

| Tool | Notes |
|------|--------|
| **Rust 1.95.0** | Workspace pin (`rust-toolchain.toml` / `clippy.toml` msrv). Edition **2024**. |
| **MSVC + Perl (Windows)** | SQLCipher vendored OpenSSL build on Win MSVC needs a working Perl on `PATH` (see `Docs/INSTALL.md`, `Docs/ci-tooling.md`). |
| **cargo-nextest** | Preferred test runner (`cargo nextest run --workspace`). |
| **cargo-deny** | License / advisory policy (`deny.toml`). |
| **cargo-audit** | Advisory DB check. |

Optional: PowerShell 7+ on Windows; POSIX shell for `scripts/dev-check.sh` on Unix.

## Full CI gate (local)

Run the full gate before opening a PR or pushing (PowerShell uses `;` as statement separator — not `&&`):

```powershell
./scripts/dev-check.ps1
```

Unix mirror:

```bash
./scripts/dev-check.sh
```

The gate covers fmt, clippy (`-D warnings`), nextest, deny, and audit (aligned with `AGENTS.md` / CI). See [Docs/ci-tooling.md](Docs/ci-tooling.md) for pins and job shapes.

Soft packaging check for reference units (T196):

```bash
./scripts/check-reference-units.sh
```

## Engineering rules (summary)

Normative project rules: **[AGENTS.md](AGENTS.md)** (and `Claude.md` pointer). Highlights:

- **Capture independence** — CLI → daemon → event log works without models/graph.
- **Event sourcing / CQRS** — commands append; queries read projections.
- **Rust safety** — no `unwrap()` / `expect()` / `panic!` in production code; explicit errors; `zeroize` for key material.
- **No repo pollution** — default storage under user profile (`~/.ai-brains` / `$env:USERPROFILE\.ai-brains`), not project-local files unless the user opts in.
- **TDD** — failing tests before implementation for behavioral changes where required by track discipline.

## Conductor + ledgerful workflow

Implementation follows **track-by-track** discipline under `conductor/` (see `conductor/conductor.md`).

Typical loop for a track change:

1. Read the track **spec** and **plan** under `conductor/tracks/…`.
2. Ensure toolchain health: `ledgerful doctor`.
3. For meaningful code/config/policy edits: `ledgerful scan --impact` (inspect hotspots / high temporal coupling).
4. Start ledger provenance for the work:  
   `ledgerful ledger start <TrackId> --category <CATEGORY>`  
   (or the track’s documented start command). **Do not** edit `.ledgerful/` state files by hand.
5. Implement (Red → Green when TDD applies).
6. After edits: `ledgerful verify` — report outcomes, pending transactions, risk, drift.
7. Commit the ledger transaction when the track says so (`ledgerful ledger commit` / `atomic` per local practice).

AI-assisted sessions (optional self-usage):

- `ai-brains preflight --summary`
- `ai-brains recall "<query>" --semantic`
- `ai-brains pin "<DECISION/CONSTRAINT: message>"`

## Git

- **Do not** push to `main` / `master` without explicit project approval.
- **Do not** force-push or run destructive git ops without explicit approval.
- **Never** commit secrets, `.env` files, or real vault keys.
- Inspect the diff before commit; stage only intentional files; keep unrelated fixes separate where practical.
- Pre-push hooks may run `ledgerful verify --scope fast` and `ledgerful ledger status` — treat them as the publish gate.

## Changelog policy

Root [CHANGELOG.md](CHANGELOG.md) follows **[Keep a Changelog](https://keepachangelog.com/en/1.1.0/)** and Semantic Versioning.

> **Note on Common Changelog:** a stricter “Common Changelog” style is intentionally **not** used here yet — it is incompatible with Keep a Changelog’s separate **Security** / **Deprecated** categories and top-level **Unreleased** section for our release process.

Add user-visible changes under `## [Unreleased]` when your track ships them. Do **not** convert the file to Common Changelog format.

## Docs map (start here)

| Doc | Role |
|-----|------|
| [Docs/INSTALL.md](Docs/INSTALL.md) | Install & first vault |
| [AGENTS.md](AGENTS.md) | Engineering mandates |
| [Docs/ci-tooling.md](Docs/ci-tooling.md) | CI tool pins |
| [Docs/OPERATIONS.md](Docs/OPERATIONS.md) | Day-to-day ops |
| [Docs/COMPATIBILITY.md](Docs/COMPATIBILITY.md) | OS tiers + honesty |
| [Docs/SECURITY-LIMITS.md](Docs/SECURITY-LIMITS.md) | Security non-claims |
| [Docs/README.md](Docs/README.md) | Documentation index |
| [packaging/reference/README.md](packaging/reference/README.md) | Reference systemd / launchd units (not product Unix install) |
| [CHANGELOG.md](CHANGELOG.md) | Keep a Changelog |

Soft onboarding skill (agents / deep setup): [`.agents/skills/onboarding/`](.agents/skills/onboarding/).

## Questions / stop-before

Halt and ask maintainers before:

- Destructive git, force-push, push to `main`/`master`
- Missing secrets with no documented mock
- Ambiguous specs not resolvable from code + plan
- Broad unrelated failure cleanup
- Unsafe dependency upgrades
- Scope beyond the current conductor track
