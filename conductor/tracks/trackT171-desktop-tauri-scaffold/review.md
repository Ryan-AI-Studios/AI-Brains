# T171 Review Log — Desktop Tauri Scaffold

## Status

**Implement complete in worktree** — R1 easy findings addressed; pending cross-model / orchestrator review for high-risk FEATURE/SECURITY scaffold.

## Scope shipped

- ADR-0017 Accepted (Vite 8 + TS 7 + React 19 + npm, Node ≥22)
- `apps/desktop` Tauri v2 shell: invoke-first, CSP non-null + ipc, stripped capabilities
- Commands: `ping`, `get_daemon_connection_info` (E1 honest; no bearer)
- S21 WebView2 registry diagnostic + MessageBox + exit 1
- license:check (rseidelsohn) + cargo deny (LLVM-exception allow for Tauri GTK transitive)
- Workspace member: `apps/desktop/src-tauri` (`ai-brains-desktop`)

## Findings

| ID | Severity | Status | Notes |
|----|----------|--------|-------|
| R1-01 | medium | fixed_pending_verification | `cargo audit` re-run exit 0; evidence in `evidence/SMOKE.md`. Warnings only (GTK3 unmaintained transitive, unic-*, etc.) — no vulns affecting Windows desktop. |
| R1-02 | low | fixed_pending_verification | Deleted `icons/android/` and `icons/ios/` (mobile trees unused by desktop targets). Kept tauri.conf.json icon list + icon.png / 64x64 / Square* Windows assets. |
| R1-03 | low | fixed_pending_verification | Removed unused `thiserror`; `serde_json` dev-deps only (CSP test). |
| R1-04 | low | fixed_pending_verification | Production CSP `style-src 'self'` only (dropped `'unsafe-inline'`). Unit test asserts no `unsafe-inline` / no `unsafe-eval`. README updated. |
| R1-05 | low | residual | Full `npm run tauri build` / interactive GUI not required for R1 easy pass; SC3 residual left honest in SMOKE.md. |

## Deferred (track policy)

| Item | Owner |
|------|--------|
| Isolation Pattern | T173 |
| single-instance | T172/T173 |
| Product screens | T172 |
| Playwright | T174 |
| Manual GUI tauri dev recording | Operator optional |
| Full `tauri build` installer smoke (R1-05) | Operator optional |

## Gates (implementer)

- typecheck / vite build: pass (prior)
- cargo test -p ai-brains-desktop: pass (R1 re-run)
- cargo clippy -p ai-brains-desktop -D warnings: pass (R1 re-run)
- cargo deny check: pass (prior)
- cargo audit: exit 0 (R1)
