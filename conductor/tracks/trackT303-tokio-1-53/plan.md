# T303 Plan — tokio 1.53.1

**Status:** ✅ **Completed**. Spec [spec.md](./spec.md).
**Ledger:** series DOCS `30b7ca9d`. Fold-in DOCS `6014c95c`. Implement **CHORE** `46c31a21-dc44-4f2d-b037-0290bb792bb4`.

## Phase 0

- [x] Branch `track/T303-tokio-1-53`
- [x] Re-read https://github.com/tokio-rs/tokio/blob/master/tokio/CHANGELOG.md 1.52.3…1.53.1 (target still **1.53.1**, not 1.53.0; skip 1.52.4)
- [x] Confirm `cargo pkgid tokio` is still unique; `cargo info tokio` latest 1.53.x → **1.53.1**
- [x] `cargo tree -i tokio@1.52.3` (pre-update)
- [x] CHORE TX; do **not** merge Dependabot remote `#59`

## Tasks

- [x] Workspace floor `tokio = { version = "1.53", features = ["full"] }` (F1 — min version, not caret-unblock)
- [x] `cargo update -p tokio --precise 1.53.1` (F8)
- [x] Confirm lock **1.53.1**; allow F9 `windows-sys` edge flips; abort if rusqlite / tower-http / clap / thiserror versions move (AC4) — **AC4 met**
- [x] Targeted: `cargo nextest run -p ai-brainsd` (**87 passed**); `cargo nextest run -p ai-brains-cli daemon_status` (**9 passed**) (AC3). Then full clippy/nextest/deny/audit (AC2) — **exit 0**
- [x] CHANGELOG Unreleased
- [x] PR body: `#8300` Windows signal; `#8095` daemon-lifetime mpsc; F9 windows-sys extras expected
- [ ] PR → CI watch → squash (never `git push origin main`) — Phase 6

## DoD

- [x] workspace tokio **1.53**; lock **1.53.1** (AC1)
- [x] F9 extras only; no rusqlite/tower-http/clap/thiserror bump (AC4)
- [x] Daemon/CLI tests green (AC3); full gate green (AC2); CHANGELOG (AC5)
- [x] No live `daemon stop` (F7)

## Evidence (2026-08-25)

```text
cargo info tokio → version: 1.52.3 (latest 1.53.1)
cargo update -p tokio --precise 1.53.1 → Updating tokio v1.52.3 -> v1.53.1
cargo pkgid tokio → tokio@1.53.1
cargo nextest run -p ai-brainsd → 87 passed
cargo nextest run -p ai-brains-cli daemon_status → 9 passed
.\scripts\dev-check.ps1 → [SUCCESS] CI Gate passed! (3529 passed / 1 skipped)
ledgerful verify --scope full → Verification passed
git diff -- crates/ → empty
```
