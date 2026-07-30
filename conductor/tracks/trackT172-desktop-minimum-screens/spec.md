# T172 — Desktop Minimum Screens (P10.1)

- **Track ID:** T172-DesktopMinimumScreens
- **Phase:** P10 Task 10.1
- **Status:** **Completed** (2026-07-30) — Codex R3 PASS WITH DEFERRED P3
- **Depends on:** **T171 Completed** (`apps/desktop` Tauri v2 scaffold, ADR-0017, invoke `ping` + `get_daemon_connection_info`); **T158–T161** governed contracts + loopback `/v1` + bearer + CORS deny-by-absence; **T160** CLI parity for shared ops
- **Blocks:** T173 security/UX polish across screens; T174 deep desktop tests / beta gate
- **Category:** FEATURE / ARCHITECTURE
- **ADRs:** [ADR-0012](../../../Docs/DECISIONS/ADR-0012-local-first-control-plane-and-public-protocol.md) (adapter only); [ADR-0013](../../../Docs/DECISIONS/ADR-0013-distinct-briefings-and-scope-hierarchy.md) (Project vs Personal briefings); [ADR-0017](../../../Docs/DECISIONS/ADR-0017-desktop-frontend-stack.md) (Vite 8 + TS 7 + React 19 + npm); ADR-0011 (evidence ≠ conclusions ≠ decisions)
- **Stop-before:** No TS domain authority; no webview `fetch` to T161 (CORS deny); no SYSTEM token; no new `/v1` routes unless a separate track; no live vault mutation without operator consent in manual tests

## 1. Objective

Ship the **minimum memory-operations screens** in `apps/desktop` so governance and inspection are usable without rebuilding Obsidian. All reads/mutations flow through **versioned contracts** via **Tauri invoke → Rust adapters → loopback HTTP (or honest offline)**. The WebView remains an **untrusted presentation layer**.

Closes P10.1: eight surfaces reachable; Home + Review fully wired where APIs exist; contract-shaped empty/loading/error/denied; no TypeScript policy/freshness/erasure semantics.

## 2. Live baseline (re-scan 2026-07-30)

| Area | Live state |
|------|------------|
| `apps/desktop` | **Present** (T171): Vite 8.1.5, TS 7.0.2, React 19.2.8, `@tauri-apps/api` 2.11.1, tauri **2.11.5** / tauri-build **2.6.3**, npm lock |
| **Shipped CSP (prod)** | `default-src 'self' customprotocol: asset:; connect-src ipc: http://ipc.localhost; img-src 'self' asset: http://asset.localhost blob: data:; style-src 'self'` — **no** `'unsafe-inline'`, **no** Vite HMR websocket (dev friction → **M24**) |
| Invoke commands | `ping`, `get_daemon_connection_info` only (AppManifest allowlist) |
| T161 HTTP | `routes.rs` exactly: `POST` scope/resolve, briefings/project+personal, knowledge/query, evidence/inspect, sources/inspect, conclusions/propose, decisions/propose; `GET` review/items; `POST` review resolve, erasure/request+wipe; `/health` + `/v1/health` |
| **Missing HTTP** | **No** connector health/list; **no** retention plan/apply; **no** grant-list-only (scope **resolve** only); **no** `/v1/ping` — honest-unavailable screens are real gaps, not invented |
| CORS | Deny-by-absence — webview `fetch` to loopback will not receive `Access-Control-Allow-Origin` |
| Token | `%USERPROFILE%\.ai-brains\http.token` owner-only ACL (user-session) |
| Retention | CLI exists; `RetentionPlanReport` in contracts **without** HTTP route |
| Connectors | P6 connectors in Rust crates; **no** desktop-facing health API |
| reqwest | Workspace **0.13.x** — loopback **plaintext HTTP** only; rustls-by-default change irrelevant for T172 traffic |

**Unblocked:** T171 Complete. Security polish remains T173; Playwright T174.

## 3. Research summary (online + standards, 2026-07-30)

### 3.1 Architecture (unchanged locks from T171)

| Practice | T172 application |
|----------|------------------|
| **Invoke-first (S9)** | Every screen data path: React → `invoke` → Rust command → `reqwest` to `127.0.0.1:<port>/v1/*` with bearer from user token file. **Never** default webview `fetch` to T161. |
| **Adapter only (ADR-0012)** | TS: layout, routes, form state, render DTOs. Rust: HTTP, token read, serde, error mapping. Domain stays in control-plane / daemon. |
| **E1 empty-state honesty** | `items: []` not null; `authoritative: false` for low scope confidence; `denied` / offline shown as first-class UI states. |
| **Capture independence** | Desktop optional; capture/CLI work without it. |
| **Capabilities + AppManifest** | New commands added to `build.rs` allowlist + permissions; no wildcards. |
| **Supply chain (S6)** | New `tauri-*` crates only from **tauri-apps**; first-time deps diff-reviewed; npm license gate continues. |

### 3.2 Frontend dependency pins (research 2026-07-30 — pin exact at implement)

| Package | Research band | License | Role in T172 |
|---------|---------------|---------|--------------|
| **react-router** | **8.x** (observed **8.3.0**) | MIT | SPA routes; **HashRouter** (not BrowserRouter) for packaged webview. **Import path:** confirm against installed package type declarations at implement — v8 dropped the separate `react-router-dom` package; HashRouter may be root export or `react-router/dom` subpath depending on exact pin. Do not copy v6/v7 tutorial snippets blindly (**M14a**). |
| **@tanstack/react-query** | **5.x** (observed **5.101.x**) | MIT | Cache/loading/error for invoke queries; **must not use default retry** for offline/denied (**M23**) |
| **lucide-react** | **1.x** (observed **1.27.x**) | **ISC** | Icons (allowlist already has ISC in `deny.toml` / npm gate) |
| **@xyflow/react** | **12.x** (observed **12.11.2**) | MIT | **Optional soft** for conclusion dependency graph — default is simple list/SVG; add only if DoD graph needs interactivity |
| **reqwest** (Rust) | workspace **0.13** (~0.13.4 class) | MIT/Apache | Loopback HTTP from Tauri commands (plaintext; no TLS needed) |
| **serde_json** | workspace | MIT/Apache | Wire JSON |
| Existing | Vite 8 / TS 7 / React 19 / tauri 2.x | frozen ADR-0017 | Do not re-litigate stack |

**Avoid:** AGPL/GPL UI kits; Electron; unknown-git; analytics SDKs; GPL icon fonts; webview CORS “fixes”; abandoned `license-checker` (keep **license-checker-rseidelsohn**).

**Not in T172 DoD:** Tailwind/shadcn (optional MIT later); specta/tauri-specta full codegen (hand-sync DTO types preferred; optional soft gen); DOMPurify (T173 if HTML sanitization required — prefer inert text/markdown subset); Isolation Pattern (T173); Playwright (T174).

### 3.2a TanStack Query retry policy (**M23** — review fold-in)

TanStack Query v5 defaults: **3 retries** with exponential backoff (~1s → 2s → 4s ≈ **7s** hang) on **all** failures. That fights **M15** / prompt offline UX: connection-refused and **401 denied** are **not** transient and will look identical on every retry.

| Error class (from Rust structured code) | Query `retry` | Rationale |
|-----------------------------------------|---------------|-----------|
| `offline` (connect fail / unreachable) | **false** (never) | Not transient; show offline immediately |
| `denied` (401 / auth) | **false** | Not transient; show denied immediately |
| `unavailable` (honest missing surface) | **false** | Static |
| `error` with **5xx** or explicit **timeout** / `transient` | Optional: `retry: (n, err) => err.kind === 'transient' && n < 2` | Only genuine transient classes |
| Default if unsure | Prefer **no retry** over multi-second hang | Desktop dogfood UX |

**Normative:** `QueryClient` default `queries: { retry: false }` **or** a shared `retry` predicate that returns true **only** for `kind === 'transient'`. Mutations: same rule (no retry on denied/offline). Document in `main.tsx` / `lib/queryClient.ts`.

### 3.2b Dev CSP vs production CSP (**M24** — review fold-in)

T171 production CSP is correct for **shipped** builds and still required (S7/SC4 intent). T172 is the first track with **real multi-screen CSS** and prolonged `tauri dev` use. Vite dev:

- Injects styles via runtime `<style>` (often needs `style-src 'unsafe-inline'` **or** nonces in **dev only**)
- HMR needs `connect-src` for the dev websocket (e.g. `ws://localhost:1420` / `http://localhost:1420` matching `devUrl`)

| Build | CSP policy |
|-------|------------|
| **`tauri build` / production** | **Unchanged S7 baseline** — no `'unsafe-inline'`; `connect-src` ipc + asset only as today |
| **`tauri dev` only** | **May** relax: allow `style-src` `'self' 'unsafe-inline'` and `connect-src` additions for Vite HMR host/port — gated by Tauri dev config / `cfg!(dev)` / separate config merge — **never** ship relaxed policy in release artifacts |

**Verify:** After first CSS-bearing screen lands, `npm run tauri dev` must not CSP-block HMR/styles; `tauri build` must still parse CSP **non-null** without dev-only directives.

### 3.3 Tauri / host plugins

| Item | Band | Disposition |
|------|------|-------------|
| **tauri-plugin-single-instance** | **2.x** (observed **2.4.x** class, May 2026; MIT/Apache, **tauri-apps**) | **Recommended soft** for dogfood — focus existing window on second launch; first plugin registered. Not a hard DoD if Windows smoke is blocked by plugin friction. |
| **tauri-plugin-shell** | 2.x | **Defer to T173** for path open / deep links (allowlisted). T172 may show path as text only. |
| **tauri-plugin-http** | — | **Not required** — use workspace **reqwest** in Rust. |
| **Isolation Pattern** | Tauri v2 recommended for heavy FE trees | **T173** (document only here). |
| **AppManifest::commands** | — | Expand allowlist for every new invoke command. |

### 3.4 Transport design (normative)

```text
[React screens] --invoke(dto)--> [src-tauri commands]
                                    |-- read user http.token (never return full token to JS)
                                    |-- reqwest POST/GET 127.0.0.1:port/v1/*
                                    |-- map status + JSON → contract-shaped result or typed error
                                    \-- offline / no-token / denied → structured error codes
```

| Layer | May | Must not |
|-------|-----|----------|
| TypeScript | Presentation, routing, local UI state, invoke wrappers | Grants, freshness, erasure semantics, invent empty as “no data” vs denied |
| Tauri commands | Serde, token file read, loopback reqwest, timeout, redact logs | Second control-plane implementation |
| HTTP | Existing T161 routes only | New product routes without a track; SYSTEM token |

**Bearer handling:** Read token only in Rust; send `Authorization: Bearer …` on `/v1/*` (except public `/health` if used for connectivity). Prefer not to put bearer in JS heap; never `localStorage`; never log full bearer.

**Command_id:** Mutations that need spool/idempotency generate UUID in Rust (or accept optional client id) and set body / `X-Command-Id` — mirror CLI/T160 behavior.

**Structured invoke errors (normative for M15/M23):** Rust maps failures into a stable JSON/error shape with at least:

| `kind` | Sources |
|--------|---------|
| `offline` | TCP refused, DNS/loopback unreachable, client connect error |
| `denied` | HTTP 401; missing token file when required |
| `error` | Other 4xx (except 401), parse failures, unexpected |
| `transient` | HTTP 5xx and/or explicit request timeout (optional retry) |

TypeScript maps `kind` → StatePanel; QueryClient uses `kind` for retry (**M23**). Never invent domain empty success when `kind` is offline/denied.

### 3.5 Contract ↔ screen map

| Screen | Route (Hash) | Primary API | Contract types (contracts crate) | Live? |
|--------|--------------|-------------|----------------------------------|-------|
| **Home** | `#/` | `POST /v1/briefings/project`, `POST /v1/briefings/personal` | `ProjectBriefingRequest/Response`, `PersonalBriefingRequest/Response` | Yes |
| **Search / query** | `#/query` | `POST /v1/knowledge/query` | `QueryKnowledgeRequest`, progressive/query response DTOs | Yes |
| **Evidence** | `#/evidence/:id?` | `POST /v1/evidence/inspect` | `InspectEvidenceRequest` + response | Yes |
| **Source** | `#/source/:id?` | `POST /v1/sources/inspect` | `InspectSourceRequest` + source DTO | Yes |
| **Review inbox** | `#/review` | `GET /v1/review/items`, `POST /v1/review/items/{id}/resolve` | `ReviewQueueResponse`, `ResolveReviewItemRequest`, `ReviewResolvedResponse` | Yes |
| **Claim detail** | `#/claims/:kind/:id` | Inspect/query results only (read model) | Knowledge / briefing claim DTOs | Yes (read); **no** graph engine |
| **Scope inspector** | `#/scope` | `POST /v1/scope/resolve` | `ResolveScopeRequest`, `ScopeResolvedResponse` | Yes (resolve only; grants list = honest unavailable if no route) |
| **Erasure center** | `#/erasure` | `POST /v1/erasure/request`, `POST /v1/erasure/wipe` | `RequestErasureRequest`, `ErasureAcceptedResponse`, wipe DTOs + honesty warnings | Yes |
| **Connector health** | `#/connectors` | — | — | **Honest unavailable** (no `/v1` surface) |
| **Retention plan** | under `#/erasure` or sub-tab | CLI-only today | `RetentionPlanReport` exists in contracts but **no HTTP** | **Honest unavailable** for plan/apply; erasure ticket+wipe stay live |

**Out of minimum green path:** `POST /v1/conclusions/propose`, `POST /v1/decisions/propose` as full authoring UX — **optional soft** (simple form OK if free; not DoD). Graph viz with `@xyflow/react` — optional soft.

### 3.6 UI / UX standards (T172 minimum; T173 deepens)

| Standard | T172 |
|----------|------|
| **IA** | Persistent nav + **scope indicator** chrome (show last resolve: scope key, confidence, authoritative flag) |
| **States** | loading / ok / empty / offline / denied / error for every live screen |
| **Prompt offline/denied** | Offline and denied must surface **without** multi-second retry hangs (**M15** + **M23**). Target: first failure paints offline/denied promptly (sub-second path after invoke returns). |
| **Destructive** | Resolve + erasure: confirm dialog with **API-returned** impact/warnings only (typed confirm / impact preview polish → T173) |
| **Markdown / preview** | Inert plain text or constrained markdown renderer — **no** `dangerouslySetInnerHTML` of untrusted HTML |
| **A11y** | Semantic headings, focusable nav, non-color-only status where cheap; full keyboard review map → T173 |
| **No analytics** | Continue S11 |
| **Dev styling** | Real CSS allowed; if CSP blocks Vite HMR, apply **M24** dev-only relaxation — never weaken production CSP |

### 3.7 TypeScript contract types

| Approach | Disposition |
|----------|-------------|
| **Hand-written** mirrors of `ai-brains-contracts` JSON fields for screens in scope | **Default DoD** — small surface, no new Rust codegen deps |
| **specta / tauri-specta / ts-rs** generate `bindings.ts` | **Optional soft** — only if low friction; must be MIT/Apache; commit generated output or document regen script |
| Share OpenAPI | Not required; HTTP is thin DaemonRequest adapter |

## 4. Non-negotiable locks (M1–M24)

| ID | Lock |
|----|------|
| **M1** | **Adapter only.** No TS grants, freshness, circularity, or erasure semantics. Render API truth. |
| **M2** | **Invoke-first.** No production webview `fetch` to loopback `/v1`. |
| **M3** | **User-session token only** in Rust. Never SYSTEM service token; never return full bearer to JS. |
| **M4** | **Existing T161 routes only** for live data. Missing surfaces → **honest unavailable**, not fake success. |
| **M5** | **E1 shapes.** Empty arrays not null; scope non-authoritative never shown as full grant. |
| **M6** | **AppManifest + capabilities** updated for every new command; minimal permissions. |
| **M7** | **No unwrap/expect/panic** in production `src-tauri` glue. |
| **M8** | **Capture independence** preserved. |
| **M9** | **No analytics** by default. |
| **M10** | **Zero AGPL/GPL** product deps (npm license gate + cargo deny). |
| **M11** | **tauri-apps provenance** for any new tauri plugin crate. |
| **M12** | **ADR-0013:** distinct Project vs Personal briefing UI; never silently merge. |
| **M13** | **Erasure honesty:** ticket accept ≠ CE wipe; show contract warnings (`ERASURE_TICKET_NO_WIPE_WARNING`, wipe honesty bullets). |
| **M14** | **Windows-first** Hash-based routing for packaged webview (not BrowserRouter). |
| **M14a** | **HashRouter import:** resolve exact export from installed `react-router` **8.x** type declarations (`react-router` root vs `react-router/dom`); do not assume v6 `react-router-dom` package. |
| **M15** | **Offline-first presentation (prompt):** when daemon/HTTP unreachable **or** auth denied, screens show offline/denied **promptly** — do not invent domain defaults; do not leave a multi-second “loading” spin caused by client retries (**M23**). |
| **M16** | **command_id** on mutating invokes (review resolve, erasure request/wipe) for spool/idempotency parity with CLI. |
| **M17** | **No new domain crates.** Desktop host may depend on `reqwest`, serde, contracts types (or duplicated serde structs matching wire). Prefer calling HTTP only (not embedding control-plane in the UI process) unless a later ADR allows in-process GovernedServices. |
| **M18** | **Fixture-ready states:** empty, offline, denied, stale warning (from packet fields), conflict badge from API only. |
| **M19** | **Shell open / Isolation / deep a11y** deferred to **T173** unless free. |
| **M20** | **Playwright / visual regression** deferred to **T174**. |
| **M21** | Frontend gate when changing `apps/desktop`: `tsc --noEmit`, `vite build`, `npm run license:check`. |
| **M22** | **#40** opportunistic dep bumps out of scope unless a new dep forces a documented bump. |
| **M23** | **QueryClient retry:** default TanStack v5 3× backoff is **forbidden** for `offline` / `denied`. Global default `retry: false` **or** predicate only for `kind === 'transient'` (5xx/timeout). Same for mutations. |
| **M24** | **Dev-only CSP relaxation allowed; production CSP must stay S7-strict.** Never ship `'unsafe-inline'` or Vite HMR `connect-src` in release `tauri.conf` / build output. Verify `tauri dev` after first styled screen. |

## 5. Repository layout (target delta)

```text
apps/desktop/
  src/
    main.tsx                 # HashRouter + QueryClientProvider (M23 retry policy)
    App.tsx                  # layout shell: nav + scope chrome + Outlet
    routes.tsx               # route table
    lib/
      api.ts                 # invoke wrappers (expand)
      queryClient.ts         # QueryClient defaults: retry policy M23
      types/                 # hand-synced wire DTOs (or generated)
      queryKeys.ts
      errors.ts              # map invoke error.kind → UI state
    screens/
      HomeScreen.tsx
      QueryScreen.tsx
      EvidenceScreen.tsx
      SourceScreen.tsx
      ReviewScreen.tsx
      ClaimDetailScreen.tsx
      ScopeScreen.tsx
      ErasureScreen.tsx
      ConnectorsScreen.tsx   # honest unavailable
    components/
      ScopeIndicator.tsx
      StatePanel.tsx         # loading/empty/error/denied/offline
      ConfirmDialog.tsx      # basic confirm (T173 hardens)
  src-tauri/
    src/commands/
      mod.rs                 # re-exports
      http_client.rs         # shared reqwest + token + error map
      briefings.rs
      knowledge.rs
      review.rs
      scope.rs
      erasure.rs
      health.rs              # optional soft: GET /health
    build.rs                 # AppManifest allowlist expanded
    capabilities/default.json
  package.json               # + react-router, @tanstack/react-query, lucide-react
```

## 6. Acceptance criteria (SC1–SC16)

| ID | Behavior |
|----|----------|
| SC1 | All eight nav destinations reachable offline (connectors = unavailable copy, not crash) |
| SC2 | Home loads Project **or** Personal briefing via invoke when daemon+token present; shows empty/denied/offline honestly |
| SC2a | With daemon **stopped**, Home (and at least one other live screen) shows **offline** promptly — no ~7s multi-retry loading hang (**M15/M23**) |
| SC2b | With missing/invalid token (or forced 401), live screen shows **denied** promptly — no multi-retry hang |
| SC3 | Query screen posts knowledge query; compact results + expand handles |
| SC4 | Evidence + Source inspect by id |
| SC5 | Review list + resolve (with confirm) round-trips contract |
| SC6 | Scope resolve shows scope, confidence, authoritative, evidence/warnings/alternatives |
| SC7 | Erasure: request ticket shows honesty warnings; wipe path requires explicit confirm + surfaces wipe honesty bullets |
| SC8 | No bearer in JS memory beyond accidental invoke error strings; no token in logs |
| SC9 | `tsc --noEmit` + `vite build` + `license:check` green |
| SC10 | `cargo test -p ai-brains-desktop` covers HTTP mapper / command pure helpers (httpmock or unit map tests); bare `cargo deny` covers httpmock if used as dep |
| SC11 | AppManifest lists all new commands; capabilities grant only those |
| SC12 | Capture/CLI still independent (desktop not required) |
| SC13 | No AGPL/GPL in npm production tree |
| SC14 | README section: screens map, invoke-first, daemon prerequisites, unavailable surfaces |
| SC15 | **QueryClient** configured per **M23** (assert in code review / unit test of retry helper if pure) |
| SC16 | **Production CSP** still S7-strict (no unsafe-inline / no HMR hosts). **Dev:** `tauri dev` works with first CSS-bearing screens; any relaxation is **dev-only** (**M24**) |

**Optional soft:** single-instance plugin; health probe command; claim graph with `@xyflow/react`; propose conclusion/decision form; specta bindings.

## 7. Testing strategy

| Layer | Expect |
|-------|--------|
| Rust unit | Token path presence; error map (401→denied, connection→offline); DTO roundtrip for request builders |
| Rust command | httpmock loopback for briefing/review happy + empty |
| TS typecheck | Strict; screens compile against types |
| Manual Windows | `npm run tauri dev` with daemon up: Home + Review happy path; daemon down: **prompt** offline (SC2a); no CSP/HMR breakage after styled screens (SC16) |
| Query retry | Code review + optional pure test: offline/denied never retry |
| License | `npm run license:check` + cargo deny |
| No Playwright | T174 |

## 8. Deferred.md absorption

| Deferred / residual | Disposition |
|---------------------|-------------|
| **#45** specta/ts-rs full gen | Optional soft; hand types default |
| **#45** single-instance | **Recommended soft** this track; else document residual for T173 |
| **#45** Isolation Pattern | **T173** |
| **#45** deep shell/fs capabilities | **T173** |
| **T161 CORS / SYSTEM token** | M2/M3 reaffirm |
| **#20 nil ProjectId / Option ScopeRef** | UI uses `authoritative` + confidence — never invent grant |
| **T165 dual-path erasure** | **M13** honesty on Erasure screen |
| **T166 retention HTTP** | Honest unavailable for plan/apply until a future HTTP track |
| **Connector list/health API** | Connectors screen unavailable; no fake green health |
| **#23 connector list cursor** | Out of scope (no list UI) |
| **#40** | Out unless forced |
| Capture independence / no analytics | **M8/M9** |

## 9. Non-goals

| Out of scope | Owner |
|--------------|--------|
| Isolation Pattern, production CSP further hardening, shell open allowlist, full a11y keyboard map | **T173** |
| Shipping relaxed CSP in production | Forbidden (**M24**) |
| Playwright, visual regression, offline beta formal gate | **T174** |
| New T161 routes (connectors, retention, grants list, `/v1/ping`) | Separate track if needed |
| In-process GovernedServices in the UI binary | Later ADR / T172+ if dogfood demands |
| Full Obsidian editor / authoring suite | Product non-goal |
| MCP/IDE clients | Other surfaces |
| Weakening CORS for webview fetch | Forbidden |
| Electron | New ADR |

## 10. Definition of Done

- [x] Route map + contract map implemented per §3.5  
- [x] SC1–SC16 green (incl. prompt offline/denied + prod CSP intact)  
- [x] Home + Review are dogfood-usable against local daemon  
- [x] Unavailable surfaces honest (connectors, retention plan)  
- [x] No TS domain logic; invoke-first; token only in Rust  
- [x] QueryClient **M23** retry policy  
- [x] AppManifest/capabilities/license/deny gates green  
- [x] README + OPERATIONS note for desktop screens  
- [x] deferred.md updated (absorb #45 items that land)  
- [x] Conductor → Completed after review  

## 11. Risks

| Risk | Mitigation |
|------|------------|
| CORS temptation | **M2** hard lock + code review |
| Token leak to webview | **M3**; connection_info stays presence-only |
| Fake connector/retention success | **M4** unavailable UI |
| Scope chrome shows nil as authority | **M5** + ADR-0013 |
| Erasure UX implies full wipe on ticket | **M13** |
| Plugin typosquat | **M11** |
| React Router path 404 on package | **M14** HashRouter |
| HashRouter import path wrong (v8 package split) | **M14a** check installed types at implement |
| Default react-query 3× retry ~7s hang on offline/denied | **M23** + structured `kind` + SC2a/SC2b |
| Vite HMR / style inject blocked by prod CSP in `tauri dev` | **M24** dev-only CSP; SC16 |
| Scope creep (propose + xyflow + specta) | Soft only; Home+Review first |
| reqwest in desktop process | Loopback only; timeouts; no remote default |

## 12. Implementation priority

1. Shared Rust HTTP client + structured error `kind` + AppManifest expansion  
2. Layout + HashRouter (**M14a** import) + QueryClient (**M23**) + ScopeIndicator + StatePanel  
3. Verify **M24** after first CSS-bearing screen (`tauri dev` + prod CSP assert)  
4. **Home** (project/personal)  
5. **Review** list/resolve  
6. Scope resolve  
7. Query + Evidence + Source + Claim detail (read)  
8. Erasure center (ticket + wipe confirm)  
9. Connectors unavailable + retention honesty panel  
10. Optional softs (single-instance, health, xyflow, propose)  
11. Docs + gates + review  

## 13. Open questions (resolved defaults)

| Question | Default for implement |
|----------|----------------------|
| In-process CP vs HTTP? | **HTTP via reqwest** to T161 (out-of-process daemon). Document in-process as future. |
| Router mode? | **HashRouter** (confirm import path on pin) |
| Data library? | **@tanstack/react-query v5** with **retry: false** (or transient-only) |
| Graph library? | **List first**; `@xyflow/react` optional soft |
| Contract types? | **Hand-sync** default |
| Single-instance? | Soft recommend yes |
| Propose forms? | Soft; not DoD |
| Prod CSP relax for styles? | **No** — only dev (**M24**) |
