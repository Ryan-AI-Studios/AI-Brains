# T172 — Desktop Minimum Screens — Plan

Status: **In Progress** (implementer). Orchestrator owns final Completed + gate clear.

Ledger TX: `6c9af29d-21e4-486e-aa0a-b6dadbe51a5b` (FEATURE)

## Phases

### Phase A — Rust adapter spine — **Done**

- [x] Deps: workspace `reqwest`, `serde_json`, `uuid`; dev `httpmock`, `tokio`
- [x] `commands/http_client.rs` — loopback base, token read, timeouts, error map, never log bearer
- [x] Commands for all T161 routes + `probe_health` soft optional
- [x] `command_id` UUID generated in Rust when omitted (review resolve, erasure request/wipe)
- [x] AppManifest + capabilities + `generate_handler!`
- [x] Unit tests: status map, command_id, httpmock empty briefing/review, unreachable offline/transient
- [x] CSP regression: prod no unsafe-inline / no :1420; devCsp may relax for HMR

### Phase B — Frontend shell — **Done**

- [x] `react-router@8` HashRouter (confirmed export from package types — M14a)
- [x] `@tanstack/react-query@5` with `retry: false` (M23)
- [x] `lucide-react` icons
- [x] Layout nav (8 destinations) + ScopeIndicator
- [x] StatePanel + ConfirmDialog
- [x] External CSS (prod `style-src 'self'`)

### Phase C — Screens — **Done**

- [x] HomeScreen (Project/Personal separate)
- [x] ReviewScreen (list + resolve confirm)
- [x] ScopeScreen (resolve + grants unavailable honesty)
- [x] QueryScreen
- [x] EvidenceScreen / SourceScreen
- [x] ClaimDetailScreen (no xyflow)
- [x] ErasureScreen (ticket ≠ wipe + wipe honesty)
- [x] ConnectorsScreen (static unavailable)

### Phase D — Docs — **Done**

- [x] `apps/desktop/README.md` expanded
- [x] `Docs/OPERATIONS.md` pointer
- [x] `conductor.md` → In Progress

### Phase E — Gates — **Done** (package-scoped)

```text
npm run typecheck / build / license:check  — PASS
cargo fmt -p ai-brains-desktop             — PASS
cargo clippy -p ai-brains-desktop --all-targets -- -D warnings — PASS
cargo test -p ai-brains-desktop            — PASS (25 lib tests)
cargo deny check                           — PASS (warnings only, pre-existing wildcards)
```

## Soft optionals

| Item | Result |
|------|--------|
| tauri-plugin-single-instance | **Skipped** → T173 |
| GET /health probe | **Done** (`probe_health`) |
| xyflow / specta / propose forms | **Skipped** |

## Residual risks

- Live daemon happy-path not E2E'd in this worktree (offline/denied proven via unit tests + StatePanel).
- Claim detail is route-handle only (no packet re-fetch by id).
- Invoke error JSON parse depends on Tauri serializing `InvokeApiError` as the error payload.
