# T171 Review Log — Desktop Tauri Scaffold

## Status

**Implement complete in worktree** — R1 easy findings + Codex R1 FAIL remediations addressed; pending cross-model / orchestrator re-review if still required.

## Scope shipped

- ADR-0017 Accepted (Vite 8 + TS 7 + React 19 + npm, Node ≥22)
- `apps/desktop` Tauri v2 shell: invoke-first, CSP non-null + ipc, stripped capabilities
- Commands: `ping`, `get_daemon_connection_info` (E1 honest; no bearer)
- S21 WebView2 registry diagnostic + MessageBox + exit 1 (pure helpers + unit tests)
- license:check (rseidelsohn) + cargo deny (LLVM-exception allow for Tauri GTK transitive)
- Workspace member: `apps/desktop/src-tauri` (`ai-brains-desktop`)
- Windows smoke: `npm run tauri -- build --debug` exit 0; binary brief start without panic; WebView2 Available

## Findings

| ID | Severity | Status | Notes |
|----|----------|--------|-------|
| R1-01 | medium | fixed_pending_verification | `cargo audit` re-run exit 0; evidence in `evidence/SMOKE.md`. Warnings only (GTK3 unmaintained transitive, unic-*, etc.) — no vulns affecting Windows desktop. |
| R1-02 | low | fixed_pending_verification | Deleted `icons/android/` and `icons/ios/` (mobile trees unused by desktop targets). Kept tauri.conf.json icon list + icon.png / 64x64 / Square* Windows assets. |
| R1-03 | low | fixed_pending_verification | Removed unused `thiserror`; `serde_json` dev-deps only (CSP test). |
| R1-04 | low | fixed_pending_verification | Production CSP `style-src 'self'` only (dropped `'unsafe-inline'`). Unit test asserts no `unsafe-inline` / no `unsafe-eval`. README updated. |
| R1-05 | low | verified_fixed | Codex R1 P1: full `npm run tauri -- build --debug` exit 0 on Windows host; exe + MSI + NSIS; binary started 4s without panic (killed); WebView2 Available recorded in SMOKE.md. No interactive GUI claim. |

## Codex R1 (cross-model FAIL → remediations)

| ID | Severity | Status | Disposition |
|----|----------|--------|-------------|
| Codex-R1-P1 | high (P1) | fixed_pending_verification | Required Windows Tauri E2E smoke incomplete → ran `npm run tauri -- build --debug` (exit 0), `cargo build -p ai-brains-desktop` (exit 0), `detect_webview2() = Available`, brief `ai-brains-desktop.exe` start (alive 4s → kill). Updated `evidence/SMOKE.md` SC2/SC3 with real commands, exit codes, OS, WebView2 status. Did **not** claim interactive ping UI. |
| Codex-R1-P2-fmt | medium (P2) | fixed_pending_verification | `cargo fmt -p ai-brains-desktop` applied; `cargo fmt --check -p ai-brains-desktop` exit 0 (Windows CRLF per rustfmt.toml). |
| Codex-R1-P2-S21 | medium (P2) | fixed_pending_verification | Extracted pure helpers `pv_indicates_installed`, `webview2_missing_message`; unit tests for empty / 0.0.0.0 / real version + Bootstrapper URL in message; Windows `detect_webview2__windows__returns_known_variant` (Available\|Missing, no panic). Process-exit Missing path documented, not process-tested. No unwrap/expect in production. |

## Deferred (track policy)

| Item | Owner |
|------|--------|
| Isolation Pattern | T173 |
| single-instance | T172/T173 |
| Product screens | T172 |
| Playwright | T174 |
| Manual interactive GUI `invoke('ping')` visual confirm | Operator optional |

## Gates (implementer — Codex R1 remediations)

- typecheck / vite build: pass
- `npm run tauri -- build --debug`: pass (exit 0)
- cargo test -p ai-brains-desktop: pass (9 tests)
- cargo clippy -p ai-brains-desktop -D warnings: pass
- cargo fmt --check -p ai-brains-desktop: pass
- cargo deny check: pass (prior)
- cargo audit: exit 0 (prior R1)

## Codex R2 (fresh re-review — PASS)

Date: 2026-07-30  
Artifact: `review.codex.r2.md`  
Verdict: **PASS**

Prior R1 FAIL findings verified fixed (build artifacts, fmt, S21 helpers/tests).  
No new P0/P1/P2. Residuals: interactive GUI visual confirm; Missing-WebView2 process-exit not process-tested (honest).

Gate clear authorized on engineering DoD + Codex R2 PASS.

