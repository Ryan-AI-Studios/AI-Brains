# SMOKE — Linux (`ubuntu-24.04` + local WSL)

**Track:** T179  
**GHA runner label:** `ubuntu-24.04` (job `gate-linux`)  
**Local host evidence:** WSL2 Ubuntu x86_64 (not GHA; do not claim label `ubuntu-24.04` for this host)  
**Date:** 2026-07-31  
**Toolchain pin:** 1.95.0  

## Hosts

| Source | OS string / label | Status |
|--------|-------------------|--------|
| Local WSL2 | `Linux … microsoft-standard-WSL2` x86_64 | **Recorded below** |
| GitHub Actions | `ubuntu-24.04` | **Pending first PR green** after workflow lands |

## Checklist (spec §6.3) — local WSL2

| # | Check | Result | Notes |
|---|-------|--------|-------|
| 1 | `rustc -V` / `cargo -V` match pin 1.95.0 | **PASS** | `rustc 1.95.0 (59807616e 2026-04-14)`; `cargo 1.95.0` |
| 2 | `cargo check --workspace --exclude ai-brains-desktop` | **PASS** | Full workspace **fails** without WebKitGTK (desktop T2); exclude is intentional F13 |
| 3 | `cargo clippy --workspace --all-targets --exclude ai-brains-desktop -- -D warnings` | **PASS** | After Phase B hygiene (cfg gates, collapsible_if, needless_return) |
| 4 | Capture independence | **PASS** | `cargo tree -p ai-brains-capture` → **NO** `ai-brains-sync` / models / graph edges |
| 5 | Path unit tests | **PASS** | Covered under nextest `ai-brains-path` (WSL string forms) |
| 6 | Daemon transport | **PASS (unit)** | Live Unix = UDS `/tmp/ledgerful-bridge.sock` (`daemon_client__new__uses_os_native_transport_path`) |
| 7 | HTTP portable IPC | **PASS (hermetic tests)** | `ai-brainsd::http_enable_smoke` + `http_dispatch__ping__returns_pong` exercise loopback HTTP path (same code on Linux) |
| 8 | Sync hermetic tests | **PASS** | Full nextest exclude desktop: **1587 passed**, 0 failed (2026-07-31) after Phase B2 Linux hygiene |
| 9 | Explicit non-claims | See below | — |
| 10 | Device seed seal | **PASS (unit)** | DataKey-only on non-Windows; DPAPI junk open fails with “DPAPI” (`device_private_blob__open_dpapi_junk__fails_with_dpapi_message`) |

## GHA `ubuntu-24.04` checklist

| Step | Status |
|------|--------|
| `cargo check|clippy|nextest --exclude ai-brains-desktop` | Pending first PR |
| `cargo deny check` + `cargo audit` (exit code) | Pending first PR |

## Non-claims (this OS)

- Windows Service / named pipe SDDL / schtasks SYSTEM ACL  
- DPAPI unlock  
- Desktop WebView2 Isolation; full desktop build without WebKitGTK  
- SQLCipher page-level CE (F8 honesty)  
- Opening Windows DPAPI-sealed device seeds  

## Transport

| Mode | Path / endpoint | Evidence |
|------|-----------------|----------|
| Live CLI | UDS `/tmp/ledgerful-bridge.sock` | Unit test + DEFAULT_DAEMON_TRANSPORT_PATH |
| Portable | Loopback HTTP + bearer (T161) | http_enable_smoke / dispatch tests |

## Notes

- Git askpass requires `/bin/true` (F32).  
- First compile breakages + clippy fixes → `UNIX-BUILD.md`.  
- Desktop remains **T2** on Linux until system packages + optional job.
