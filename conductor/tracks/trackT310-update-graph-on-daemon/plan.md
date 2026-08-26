# T310 Plan — update graph-on + PATH daemon 4.14 (placeholder)

**Status:** **Pending**. Spec [spec.md](./spec.md). **Upgrade on `/plan-track 310`.**
**Ledger:** T306 planning DOCS `2b0a2dec`. Implement **CHORE** (or FEATURE if `run_update` src) on go.

## Phase 0 (on full plan / go)

- [ ] Re-read `run_update` (`daemon.rs` ~1030–1099) vs `GRAPH_REINSTALL_SOOT`
- [ ] PATH `ai-brainsd` mtime / whether daemon is Running
- [ ] Do **not** `daemon stop` or `cargo install` as planning

## Tasks (stub)

- [ ] Full F-list / ACs on `/plan-track 310`
- [ ] Wire `run_update` CLI install to `--features graph`
- [ ] Owner-confirm PATH `ai-brainsd --locked` (daemon stop if Running)
- [ ] Unit: cargo args include `--features graph`
- [ ] PR → CI → squash (never `git push origin main`)

## DoD

- [ ] `ai-brains update` cannot reinstall graph-off
- [ ] PATH daemon matches HEAD lock rusqlite 0.40.2 / SQLCipher 4.14.x (probe defined at full plan)
