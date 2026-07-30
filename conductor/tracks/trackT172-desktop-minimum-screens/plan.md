# T172 Plan — Desktop Minimum Screens (P10.1)

Status: **In Progress** (2026-07-30 implement) — internal review CLEAN; Codex gate pending.

Authority: `spec.md` locks **M1–M24**. Invoke-first; adapter-only; existing T161 routes only; prompt offline; prod CSP strict.

## Phase 0 — Preconditions

- [x] Confirm T171 Complete: `apps/desktop` builds; CSP non-null + ipc; AppManifest `ping` + `get_daemon_connection_info`.
- [x] Confirm T161 routes (no `/v1/ping`; no connectors/retention HTTP; RetentionPlanReport contract-only).
- [x] Node ≥22, npm ≥10, Rust 1.95, WebView2 (usually preinstalled).
- [x] Pin at implement: `react-router@8.x`, `@tanstack/react-query@5.x`, `lucide-react@1.x` (ISC); optional soft `@xyflow/react@12.x` MIT.
- [x] Research re-check: licenses MIT/ISC/Apache only; no AGPL/GPL.
- [x] `ledgerful doctor` before code edits; ledger start when implementing.
- [x] #40 out of scope unless forced.

## Phase A — Rust adapter spine

- [x] Add workspace **reqwest** (+ serde_json as needed) to `ai-brains-desktop` (loopback plaintext HTTP; rustls default irrelevant).
- [x] `http_client.rs`: resolve loopback base; read user `http.token`; attach bearer; timeouts; map:
  - connect fail → `kind: offline`
  - 401 / missing token → `kind: denied`
  - 5xx / timeout → `kind: transient` (optional)
  - other → `kind: error`
- [x] Generate **command_id** (UUID) for mutations when client omits.
- [x] Commands (pure builders + invoke handlers):
  - [ ] `project_briefing` / `personal_briefing`
  - [ ] `query_knowledge`
  - [ ] `inspect_evidence` / `inspect_source`
  - [ ] `list_review_items` / `resolve_review_item`
  - [ ] `resolve_scope`
  - [ ] `request_erasure` / `wipe_content_envelope`
  - [ ] Keep `ping` + `get_daemon_connection_info`
- [x] **AppManifest::commands** allowlist all of the above.
- [x] Capabilities: grant only new allow-* permissions for main window.
- [x] Unit tests: error map; request JSON shape; httpmock happy/empty for ≥1 briefing + review list.
- [x] **No unwrap/expect** in production glue.

## Phase B — Frontend shell

- [x] `npm i react-router @tanstack/react-query lucide-react` (exact pins at implement).
- [x] **HashRouter:** confirm import against installed **react-router 8.x** type declarations (root vs `react-router/dom` — **M14a**). Not `react-router-dom` package.
- [x] **`lib/queryClient.ts`:** `QueryClient` with **`retry: false`** by default **or** `retry: (n, err) => err.kind === 'transient' && n < 2` — **never** default v5 3× backoff for offline/denied (**M23**). Mutations same.
- [x] `lib/errors.ts`: map invoke structured `kind` → StatePanel.
- [x] HashRouter + `QueryClientProvider` in `main.tsx`.
- [x] Layout: nav links to all eight destinations; **ScopeIndicator** chrome.
- [x] `StatePanel` for loading/empty/error/denied/offline.
- [x] Basic `ConfirmDialog` for resolve + wipe (T173 deepens).
- [x] Expand `lib/api.ts` invoke wrappers; `lib/types/*` hand-synced DTOs (E1: arrays default `[]`).
- [x] No webview `fetch` to loopback; no analytics SDK.
- [x] **M24 CSP check (after first real CSS):**
  - [ ] `npm run tauri dev` — styles/HMR not CSP-blocked; if blocked, add **dev-only** CSP (e.g. `style-src 'self' 'unsafe-inline'`; `connect-src` + Vite `devUrl` / `ws://localhost:1420`) via Tauri dev config / env — **not** in release config.
  - [ ] Assert production CSP remains T171 S7 baseline (no unsafe-inline, no HMR hosts) for `tauri build`.

## Phase C — Screens (priority order)

### C1 — Home (DoD critical)

- [x] Project / Personal selector (ADR-0013 — never merge).
- [x] Invoke briefing; render claims/warnings/freshness **fields from packet only**.
- [x] empty / denied / offline / loading states.
- [x] Manual: daemon stopped → offline **promptly** (SC2a); no multi-second hang.

### C2 — Review (DoD critical)

- [x] List items (`items: []` empty state).
- [x] Resolve with confirm + resolution enum string from UI → API.
- [x] Surface response warnings.

### C3 — Scope

- [x] Resolve form (cwd / explicit_project_id / force_personal).
- [x] Show authoritative, confidence, evidence, warnings, alternatives.
- [x] Grants list: **honest unavailable** (no HTTP) unless free later.

### C4 — Query + Evidence + Source + Claim detail

- [x] Query form → progressive hits; compact + expand.
- [x] Evidence/source inspect by id (route params).
- [x] Claim detail read-only from prior result or inspect; **list** dependency handles (xyflow optional soft).

### C5 — Erasure center

- [x] Ticket request path + display `ERASURE_TICKET_NO_WIPE_WARNING` style honesty from API.
- [x] Wipe path: confirm required; show wipe honesty bullets from response.
- [x] Retention plan/apply: **honest unavailable** panel (CLI-only until HTTP track).

### C6 — Connectors

- [x] Static unavailable explanation; no fake green health.
- [x] Link to docs / OPERATIONS for connector ops via CLI if useful.

## Phase D — Optional soft (not DoD)

- [ ] `tauri-plugin-single-instance` (tauri-apps only; ~2.4.x class; first registered; focus main window).
- [x] Rust `GET /health` connectivity command.
- [ ] `@xyflow/react` dependency graph for claim detail.
- [ ] Minimal propose conclusion/decision forms.
- [ ] specta/tauri-specta generated bindings (or document skip).

## Phase E — License, docs, gates

- [x] `npm run license:check` green (license-checker-rseidelsohn).
- [x] `cargo deny check` + `cargo audit` green; provenance note for any new tauri plugin.
- [x] `tsc --noEmit` / `vite build` / `cargo test -p ai-brains-desktop`.
- [x] SC15: QueryClient M23 visible in code.
- [x] SC16: prod CSP string check; dev CSP note in README if relaxed.
- [x] README: screen map, invoke-first, daemon + token prerequisites, unavailable surfaces, retry policy, CSP split.
- [x] OPERATIONS.md short desktop section (or pointer from README).
- [ ] Manual smoke notes (daemon up/down, prompt offline) in plan or evidence/.

## Phase F — Closeout

- [x] SC1–SC16 checklist.
- [ ] deferred.md: strike/absorb #45/#46 items that landed; residual single-instance/Isolation as needed.
- [ ] Review log + conductor → Completed after review.
- [ ] Pin: invoke-first screens; HashRouter; hand DTOs; erasure honesty; M23 retry; M24 prod CSP.

## Out of scope (explicit)

- [ ] T173 Isolation / shell open / production CSP further tighten / full a11y
- [ ] Shipping relaxed CSP in production
- [ ] T174 Playwright
- [ ] New T161 routes
- [ ] In-process GovernedServices default
- [ ] Weakening CORS
- [ ] Electron

## Definition of Done

Mirror spec §10. Home + Review dogfood-ready; remaining live screens wired; unavailable honest; prompt offline/denied; gates green.
