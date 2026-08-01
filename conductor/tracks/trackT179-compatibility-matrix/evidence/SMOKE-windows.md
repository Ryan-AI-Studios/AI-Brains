# SMOKE — Windows (`windows-2025`)

**Track:** T179  
**Runner label (GHA):** `windows-2025`  
**Local host:** Windows 11 x64 (developer machine)  
**Date:** 2026-07-31  
**Toolchain pin:** 1.95.0  

## Checklist (spec §6.3)

| # | Check | Result | Notes |
|---|-------|--------|-------|
| 1 | `rustc -V` / `cargo -V` match pin 1.95.0 | **PASS (local)** | `rustc 1.95.0 (59807616e 2026-04-14)`; `cargo 1.95.0 (f2d3ce0bd 2026-03-21)` |
| 2 | `cargo clippy --workspace --all-targets -- -D warnings` | **PASS (local)** | Full workspace including desktop |
| 3 | `cargo nextest run --workspace` | **PASS (local)** | **1653** passed, 1 skipped (2026-07-31) |
| 4 | Capture independence (no capture→sync) | **PASS** | Mandate + `cargo tree -p ai-brains-capture` on Linux confirmed no sync edge; capture `Cargo.toml` has no sync/models/graph deps |
| 5 | Path unit tests (WSL string forms) | **PASS** | Via nextest `ai-brains-path` |
| 6 | Daemon transport smoked | **PASS** | Live: named pipe `\\.\pipe\ledgerful-bridge`; unit `daemon_client__new__uses_os_native_transport_path` |
| 7 | HTTP health/bearer | **PASS (hermetic)** | `ai-brainsd::http_enable_smoke` + `http_dispatch__ping__returns_pong` in nextest |
| 8 | Sync hermetic tests | **PASS** | Wire crypto + `device_replicate_cli` 14/14 including DPAPI private-key export |
| 9 | Explicit non-claims for this OS | See below | — |
| 10 | Device seed seal | **PASS** | Windows seals with `PROTECTION_DATAKEY_DPAPI`; junk open fails with DPAPI message |

## Local vs GHA

| Source | Status |
|--------|--------|
| Local Windows 11 developer machine | Full gate subset recorded 2026-07-31: clippy workspace, nextest 1653, deny, audit exit 0 |
| `cargo deny check` | **PASS** local |
| `cargo audit` | **PASS** exit 0 (allowed warnings only; F27 exit-code gate) |
| GitHub Actions `windows-2025` | **Pending first green** after workflow lands on PR |

## Non-claims (this OS)

- SQLCipher page-level CE: **honesty only** — vault uses bundled SQLite + application-level CE (F8).  
- Service / pipe / DPAPI / Isolation / schtasks are **T1 on Windows** (not non-claims).

## Transport smoked

- **Pipe:** `\\.\pipe\ledgerful-bridge` (live CLI `DaemonClient`)  
- **HTTP:** loopback HTTP + bearer (T161) — hermetic smoke via nextest  
