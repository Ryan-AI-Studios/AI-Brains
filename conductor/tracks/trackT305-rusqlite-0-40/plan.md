# T305 Plan — rusqlite 0.40.2

**Status:** **Pending**. Spec [spec.md](./spec.md).
**Ledger:** series DOCS `30b7ca9d`. Fold-in DOCS `db5c6f11`. Implement **DEPS** (or SECURITY) on go.

## Phase 0

- [ ] Branch `track/T305-rusqlite-0-40`
- [ ] Re-read rusqlite [0.40.0](https://github.com/rusqlite/rusqlite/releases/tag/v0.40.0)–0.40.2 + `Docs/COMPATIBILITY.md` F8
- [ ] `rg vtab` in `crates/` ; confirm local `fn table_exists` only; `cargo tree -i rusqlite@0.39.0` ; `cargo pkgid rusqlite` unique
- [ ] Note: `#61` extras (libsqlite3-sys 0.38.2, hashlink 0.12.x, windows-*) may **not** match live HEAD after T303/T304 — accept resolver (F13)
- [ ] DEPS TX; do **not** merge `dependabot/cargo/rusqlite-0.40.1`
- [ ] Stop-Before if encrypt/open/rotate-export KATs red or `cipher_version` empty (F9)

## Tasks

- [ ] Workspace rusqlite exact `0.40.2` + **same four features**
- [ ] `cargo update -p rusqlite --precise 0.40.2` (F12)
- [ ] Confirm lock 0.40.2 + libsqlite3-sys 0.38.x; allow F13 extras; abort if clap moves or tokio/tower-http revert (F6)
- [ ] Fix compile (VTab only if needed). Do **not** adopt `Connection::table_exists` (F5)
- [ ] T187-V-01 green; record **observed** `PRAGMA cipher_version` in COMPATIBILITY F8 (do not pre-write `4.14.0 community`)
- [ ] Targeted: `cargo nextest run -p ai-brains-store` (encrypt/open/wrong-key + cipher_version) ; backup hermetic T277
- [ ] Full clippy/nextest/deny/audit
- [ ] CHANGELOG Unreleased; refresh T187-V-01 comment if it still cites 4.10.0
- [ ] Manual `ai-brains doctor --summary` vault_open / cipher_page (no key in output)
- [ ] Codex review (SECURITY/DEPS) after Phase-1 clean
- [ ] PR → CI watch → squash (never `git push origin main`)

## DoD

- [ ] rusqlite **0.40.2**; same features; cipher_version recorded (AC1/AC2)
- [ ] Encrypt/open/sqlcipher_export KATs green (AC3); backup green (AC4); F9 not tripped
- [ ] Full gate green (AC5); CHANGELOG + COMPATIBILITY (AC7); live doctor AC8
