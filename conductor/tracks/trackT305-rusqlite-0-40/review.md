# T305 Review Log — rusqlite 0.40.2

**Track:** T305-Rusqlite040  
**Category:** CHORE / DEPS / SECURITY / STORE  
**CHORE TX:** `d80afcd7-55fa-4c6d-8026-3701f6b90924`  
**Branch:** `track/T305-rusqlite-0-40`  
**Date:** 2026-08-26

## Scope

Dependabot `#61` rusqlite **0.39.0 → 0.40.2**. Workspace exact pin + `--precise 0.40.2` (F12). Same four features. Re-probe `PRAGMA cipher_version`; update COMPATIBILITY F8 **observed** string. Do not merge Dependabot remote `dependabot/cargo/rusqlite-0.40.1`. Do not adopt `Connection::table_exists` (F5). Stop-Before if encrypt/open/sqlcipher_export KATs fail or cipher_version empty (F9).

## Pin resolution (execute 2026-08-26)

| Pin | Before | After | crates.io | Notes |
|-----|--------|-------|-----------|-------|
| workspace rusqlite | exact `0.39.0` + 4 features | exact **`0.40.2`** same features | **0.40.2** | F1 |
| lock rusqlite | 0.39.0 | **0.40.2** | — | F12 `--precise` |
| libsqlite3-sys | 0.37.0 | **0.38.2** | — | F13 expected |
| hashlink | 0.11.0 | **0.12.1** | — | F13 expected |
| tokio | 1.53.1 | **1.53.1** | — | F6 |
| tower-http | 0.7.0 + 0.6.11 | same dual | — | F6 |
| clap | 4.6.1 | **4.6.1** | — | F6 |
| thiserror | 1.0.69 + 2.0.20 | same | — | F6 |
| windows-* extras | possible per `#61` | **absent** this bump | — | only hashlink + libsqlite3-sys |

`cargo pkgid rusqlite` → unique `rusqlite@0.40.2`.  
`rg vtab` in `crates/**/*.rs` → **zero** (AC6).  
`git diff -- crates/` → comment-only on T187-V-01 observed version string.

## DoD / AC matrix

| AC | Status | Evidence |
|----|--------|----------|
| AC1 workspace + lock 0.40.2; features unchanged | **Met** | `Cargo.toml:57`; lock `rusqlite@0.40.2` |
| AC2 T187-V-01 + COMPATIBILITY F8 observed | **Met** | probe `4.14.0 community`; COMPATIBILITY F8 + `cipher_version.txt` |
| AC3 encrypt/open/wrong-key + sqlcipher_export | **Met** | store lib nextest **31 passed** (encrypt + rotate export paths) |
| AC4 backup hermetic | **Met** | brain backup filter **28 passed** incl. `backup_create__encrypted_vault__produces_valid_backup` |
| AC5 full gate | **Met** | `dev-check.ps1` + `ledgerful verify --scope full` **exit 0** (3529 passed / 1 skipped) |
| AC6 vtab unused or updated | **Met** | `rg vtab` empty |
| AC7 CHANGELOG + COMPATIBILITY | **Met** | Unreleased Changed + F8 observed |
| AC8 live doctor vault_open / cipher_page | **Met** (new binary) | `target\debug\ai-brains.exe doctor` — see Manual |

## Manual evidence

- Hermetic: `cargo nextest run -p ai-brains-store --lib` → **31 passed**; probe `T305_OBSERVED_CIPHER_VERSION=4.14.0 community`.
- Hermetic backup: `cargo nextest run -p ai-brains-brain --lib backup` → **28 passed**.
- Live PATH `ai-brains doctor` still linked old 0.39.0 (`cipher_version=4.10.0 community`) — not the gate binary (R3).
- Live **new** binary (`cargo build -p ai-brains-cli --features graph` then `.\target\debug\ai-brains.exe doctor --summary`): status=degraded, **fail=0**; `vault_open` ok (`opened read-only`); `cipher_page` ok (`cipher_version=4.14.0 community`); soft warns only (`recovery_kit_event`, `graph_density`). **no_key_leak** in `--json`.
- Did not `vault encrypt` the live vault. Did not print `AI_BRAINS_KEY`.

## Internal findings

| ID | Severity | Description | Status |
|----|----------|-------------|--------|
| R1 | low-info | Dependabot `#61` still OPEN — close as superseded after squash; do not delete remote | **deferred** — F8 / standing hygiene |
| R2 | low-info | T213 L4 `rusqlite::Connection::table_exists` not adopted (F5 optional) | **deferred** — not easy product churn; local `fn table_exists` helpers unrelated |
| R3 | low-info | PATH-installed `ai-brains` remains pre-0.40.2 until operator `cargo install` / build.ps1 | **deferred** — F10 uses track-built binary; install not this track |
| R4 | low-info | `#61` windows-sys/socket2 extras absent on live HEAD after T303/T304 | **deferred** — F13 variance; do not hand-edit |
| R5 | low-info | clap 5 still declined (not in this Dependabot batch) | **deferred** — standing decline |

No critical / high / medium. Easy lows closed by pin + KATs + COMPATIBILITY update.

## Cross-model

**Ran** `codex exec` → `review.codex.md` (gpt-5.6-luna, read-only, 2026-08-25).

| Codex ID | Severity | Disposition |
|----------|----------|-------------|
| P1-01 full gate pending | process | **verified_fixed** — after `cargo clean` (~400 GiB), full gate **exit 0** |
| P1-02 missing post-impl Codex | process | **verified_fixed** — `review.codex.md` artifact |
| P1-03 closeout incomplete | process | **fixed_pending_verification** — plan/conductor/deferred done; commit + Phase 6 publish |
| P2-01 date 2026-08-26 vs env | low | **verified_fixed** — corrected to **2026-08-25 EDT** |
| P2-02 `cipher_version` `unwrap_or_default` | medium | **verified_fixed** — `pragmas.rs` propagates query errors (`?`) |

Internal explore + Codex: **PASS WITH DEFERRED P3** (R1–R5). No open critical/high/medium.

## Gates

- Targeted store/brain KATs + clippy `-D warnings`: **exit 0** (2026-08-25).
- Full `dev-check.ps1` + `ledgerful verify --scope full`: **exit 0** (2026-08-25; nextest **3529 passed / 1 skipped**; deny + audit green with allowed warnings only).
- Publish GHA: Phase 6.
