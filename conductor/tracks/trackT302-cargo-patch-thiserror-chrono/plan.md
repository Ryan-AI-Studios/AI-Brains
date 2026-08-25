# T302 Plan — thiserror + chrono patches

**Status:** **Pending**. Spec [spec.md](./spec.md).
**Ledger:** series DOCS `30b7ca9d`. Implement **CHORE** on go.

## Phase 0

- [ ] Branch `track/T302-cargo-patch-thiserror-chrono`
- [ ] Re-read crates.io thiserror 2.0.x + chrono 0.4.x latest
- [ ] `cargo tree -i thiserror@2.0.18` ; `cargo tree -i chrono@0.4.44`
- [ ] Do **not** merge Dependabot remotes
- [ ] CHORE TX

## Tasks

- [ ] `cargo update -p thiserror` ; `cargo update -p chrono`
- [ ] Confirm lock versions AC1/AC2; no rusqlite/tokio/tower-http/clap bump in the same lock diff
- [ ] clippy + nextest + deny + audit
- [ ] CHANGELOG
- [ ] PR → watch CI → squash (never `git push origin main`)

## DoD

- [ ] Lock pins thiserror 2.0.20 + chrono 0.4.45 (or execute-current same lines)
- [ ] Full gate green; no unrelated crate bumps
