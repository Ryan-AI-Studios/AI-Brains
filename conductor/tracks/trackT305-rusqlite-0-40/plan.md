# T305 Plan — rusqlite 0.40.2

**Status:** **Completed** 2026-08-25. Spec [spec.md](./spec.md).
**Ledger:** series DOCS `30b7ca9d`. Fold-in DOCS `db5c6f11`. Implement CHORE `d80afcd7-55fa-4c6d-8026-3701f6b90924`.

## Phase 0

- [x] Branch `track/T305-rusqlite-0-40`
- [x] Re-read rusqlite [0.40.0](https://github.com/rusqlite/rusqlite/releases/tag/v0.40.0)–0.40.2 + `Docs/COMPATIBILITY.md` F8
- [x] `rg vtab` in `crates/` ; confirm local `fn table_exists` only; `cargo tree -i rusqlite@0.39.0` ; `cargo pkgid rusqlite` unique
- [x] Note: `#61` extras (libsqlite3-sys 0.38.2, hashlink 0.12.x, windows-*) may **not** match live HEAD after T303/T304 — accept resolver (F13)
- [x] CHORE TX; do **not** merge `dependabot/cargo/rusqlite-0.40.1`
- [x] Stop-Before if encrypt/open/rotate-export KATs red or `cipher_version` empty (F9) — **not tripped**

## Tasks

- [x] Workspace rusqlite exact `0.40.2` + **same four features**
- [x] `cargo update -p rusqlite --precise 0.40.2` (F12)
- [x] Confirm lock 0.40.2 + libsqlite3-sys 0.38.x; allow F13 extras; abort if clap moves or tokio/tower-http revert (F6)
- [x] Fix compile (VTab only if needed). Do **not** adopt `Connection::table_exists` (F5). Bonus: propagate `cipher_version` query errors (Codex P2-02)
- [x] T187-V-01 green; record **observed** `PRAGMA cipher_version` = `4.14.0 community` in COMPATIBILITY F8
- [x] Targeted: `cargo nextest run -p ai-brains-store` (encrypt/open/wrong-key + cipher_version) ; backup hermetic T277
- [x] Full clippy/nextest/deny/audit
- [x] CHANGELOG Unreleased; refresh T187-V-01 comment if it still cites 4.10.0
- [x] Manual `ai-brains doctor --summary` vault_open / cipher_page (no key in output) — via `target\debug\ai-brains.exe`
- [x] Codex review (SECURITY/DEPS) after Phase-1 clean → `review.codex.md`
- [ ] PR → CI watch → squash (never `git push origin main`) — Phase 6

## DoD

- [x] rusqlite **0.40.2**; same features; cipher_version recorded (AC1/AC2)
- [x] Encrypt/open/sqlcipher_export KATs green (AC3); backup green (AC4); F9 not tripped
- [x] Full gate green (AC5); CHANGELOG + COMPATIBILITY (AC7); live doctor AC8
