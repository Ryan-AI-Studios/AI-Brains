# T305 Plan — rusqlite 0.40.2

**Status:** **Pending**. Spec [spec.md](./spec.md).
**Ledger:** series DOCS `30b7ca9d`. Implement **DEPS** (or SECURITY) on go.

## Phase 0

- [ ] Branch `track/T305-rusqlite-0-40`
- [ ] Re-read rusqlite 0.40.0–0.40.2 changelog + COMPATIBILITY F8
- [ ] `rg vtab` store/cli; `cargo tree -i rusqlite@0.39.0`
- [ ] DEPS TX; do not merge Dependabot remote
- [ ] Stop-Before if encrypt/open red

## Tasks

- [ ] Workspace rusqlite `0.40.2` + same features; `cargo update -p rusqlite`
- [ ] Fix compile (VTab only if needed)
- [ ] cipher_version smoke + COMPATIBILITY F8 string
- [ ] clippy/nextest/deny/audit
- [ ] CHANGELOG
- [ ] Manual doctor vault_open/cipher_page (no key in output)
- [ ] Codex review (SECURITY/DEPS)
- [ ] PR → CI watch → squash (never `git push origin main`)

## DoD

- [ ] rusqlite 0.40.2; cipher_version recorded; vault still opens
- [ ] Full gate green
