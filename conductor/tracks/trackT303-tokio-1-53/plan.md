# T303 Plan — tokio 1.53.1

**Status:** **Pending**. Spec [spec.md](./spec.md).
**Ledger:** series DOCS `30b7ca9d`. Implement **CHORE** on go.

## Phase 0

- [ ] Branch `track/T303-tokio-1-53`
- [ ] Re-read https://github.com/tokio-rs/tokio/blob/master/tokio/CHANGELOG.md 1.52.3…1.53.1
- [ ] `cargo tree -i tokio@1.52.3`
- [ ] CHORE TX; do not merge Dependabot remote

## Tasks

- [ ] Workspace tokio `1.53` + `cargo update -p tokio`
- [ ] Clippy/nextest/deny/audit
- [ ] Confirm lock 1.53.1; no rusqlite/tower-http/clap in diff
- [ ] CHANGELOG
- [ ] PR → CI watch → squash (never `git push origin main`)

## DoD

- [ ] tokio lock 1.53.1; daemon/CLI tests green
- [ ] Full gate green
