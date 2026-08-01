# SMOKE — macOS (`macos-15`)

**Track:** T179  
**Runner label (GHA):** `macos-15` (**soft** — `continue-on-error: true`)  
**Tier claim:** **T2 Best-effort residual** until smoke is green and explicitly promoted  
**Date:** 2026-07-31  
**Toolchain pin:** 1.95.0  

## Label honesty (F3 / F25)

- Soft pin is **`macos-15`**, not `macos-latest` (which may resolve to macOS 26 as of mid-2026).
- Do **not** publish “macOS 15 supported (T1)” evidence from a different runner label.
- Smoke OS string in this file must match the runner used.

## Status

**T2 residual** — soft CI job only; no T1 claim. Checklist below is a template for when/if soft job is green.

## Checklist (spec §6.3)

| # | Check | Result | Notes |
|---|-------|--------|-------|
| 1 | rustc/cargo 1.95.0 | Pending soft GHA | — |
| 2 | `cargo check --workspace` | Pending soft GHA | — |
| 3 | nextest (core or full) | Pending soft GHA | — |
| 4 | Capture independence | Invariant if T1 later | — |
| 5 | Path unit tests | Pending | — |
| 6 | Daemon transport | UDS `/tmp/ledgerful-bridge.sock` | Live Unix path |
| 7 | HTTP portable | Optional | — |
| 8 | Sync hermetic | Pending | — |
| 9 | Non-claims | See below | — |
| 10 | Device seed | DataKey-only | F29 |

## Non-claims (this OS)

- T1 equality with Windows
- Windows Service / pipe / DPAPI / schtasks
- WebView2 Isolation (engine is **WKWebView** — no Isolation claim)
- SQLCipher page-level CE
- arm64 promotion beyond soft job without evidence

## Desktop

- Engine: **WKWebView**
- Isolation: **not claimed**
- Desktop multi-OS is T2, not T1 DoD
