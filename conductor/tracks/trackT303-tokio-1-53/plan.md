# T303 Plan — tokio 1.53.1

**Status:** **Pending**. Spec [spec.md](./spec.md).
**Ledger:** series DOCS `30b7ca9d`. Fold-in DOCS `6014c95c`. Implement **CHORE** on go.

## Phase 0

- [ ] Branch `track/T303-tokio-1-53`
- [ ] Re-read https://github.com/tokio-rs/tokio/blob/master/tokio/CHANGELOG.md 1.52.3…1.53.1 (target still **1.53.1**, not 1.53.0; skip 1.52.4)
- [ ] Confirm `cargo pkgid tokio` is still unique; `cargo info tokio` latest 1.53.x
- [ ] `cargo tree -i tokio@1.52.3`
- [ ] CHORE TX; do **not** merge Dependabot remote `#59`

## Tasks

- [ ] Workspace floor `tokio = { version = "1.53", features = ["full"] }` (F1 — min version, not caret-unblock)
- [ ] `cargo update -p tokio --precise 1.53.1` (F8)
- [ ] Confirm lock **1.53.1**; allow F9 `windows-sys` edge flips; abort if rusqlite / tower-http / clap / thiserror versions move (AC4)
- [ ] Targeted: `cargo nextest run -p ai-brainsd` ; `cargo nextest run -p ai-brains-cli daemon_status` (AC3). Then full clippy/nextest/deny/audit (AC2)
- [ ] CHANGELOG Unreleased
- [ ] PR body: `#8300` Windows signal; `#8095` daemon-lifetime mpsc; F9 windows-sys extras expected
- [ ] PR → CI watch → squash (never `git push origin main`)

## DoD

- [ ] workspace tokio **1.53**; lock **1.53.1** (AC1)
- [ ] F9 extras only; no rusqlite/tower-http/clap/thiserror bump (AC4)
- [ ] Daemon/CLI tests green (AC3); full gate green (AC2); CHANGELOG (AC5)
- [ ] No live `daemon stop` (F7)
