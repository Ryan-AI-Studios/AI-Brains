# T174 — Desktop Tests & Offline Beta Gate (P10.3)

- **Track ID:** T174-DesktopTests
- **Phase:** P10 Task 10.3
- **Status:** **Completed** (Codex R2 PASS WITH DEFERRED P3; live L5 residual)
- **Depends on:** **T171–T173 Completed** (`apps/desktop` scaffold + 9 screens + Isolation/opener/typed WIPE/a11y); T161 loopback contracts for live smoke only
- **Blocks:** P10 phase close (desktop beta acceptance); informs T179 platform matrix later
- **Category:** FEATURE / INFRA / SECURITY (test evidence)
- **ADRs:** [ADR-0012](../../../Docs/DECISIONS/ADR-0012-local-first-control-plane-and-public-protocol.md) (adapter only; no UI authority); [ADR-0017](../../../Docs/DECISIONS/ADR-0017-desktop-frontend-stack.md) (Vite 8 + TS 7 + React 19 + npm; Playwright owned here)
- **Stop-before:** Live vault mutation without operator consent; real third-party network in CI; AGPL/GPL test tooling as required gate; shipping test-only Tauri plugins into production builds; weakening prod CSP for tests

## 1. Objective

Prove the desktop thin client is **correct, safe, and offline-honest** with a **layered test pyramid** that runs without cloud deps and without live vault mutation by default. Close **P10.3** and the **desktop beta acceptance** criteria:

1. Primary operations work **offline** (honest offline/denied UI; no hang; no fabricated authority).
2. UI **never grants authority** unavailable through service contracts.
3. T173 security handoff cases are automated where feasible and human-proven where not (Isolation live WebView2 + full keyboard GUI smoke).

## 2. Live baseline (re-scan 2026-07-30 post-T173)

| Area | Live state |
|------|------------|
| App | `apps/desktop` — Tauri v2.11.5 + Isolation; React 19.2.8; Vite 8.1.5; TS 7.0.2; npm lock |
| Screens | 9 **HashRouter** screens; invoke → Rust reqwest → T161; `retry: false` |
| Security | Dual-layer opener; typed WIPE; native `<dialog showModal()>`; `:focus-visible`; no JS opener package |
| ConfirmDialog | Calls `el.showModal()` / `el.close()` in `useEffect` when `open` — **jsdom lacks showModal** (AI3) |
| Rust tests | `csp_tests`, `capability_tests`, open validators, httpmock adapter HTTP (~46 lib tests) |
| FE unit / E2E | **None** — no Vitest, no Playwright, no Testing Library in `package.json` |
| `license:check` | **`--production` only** (`scripts/license-check.mjs`) — does **not** scan devDependencies |
| Scripts | `dev`, `build`, `typecheck`, `license:check`, `tauri` only |
| Vite config | No path aliases today; Tauri host/HMR server settings only |
| CI | Workspace cargo gate covers `ai-brains-desktop` lib tests; **no** desktop npm test job |
| Deferred | #46 Playwright/visual/offline beta; #48 live Isolation + keyboard GUI smoke |

**Unblocked:** T173 Complete. This track owns FE automation + beta gate + residual live smoke.

## 3. Research summary (online + registry + review fold-in, 2026-07-30)

### 3.1 Official Tauri testing model

Per [Tauri v2 Tests](https://v2.tauri.app/develop/tests/) / [Mocking](https://v2.tauri.app/develop/tests/mocking/) (docs updated 2026-06-29):

| Layer | Official path | Notes for AI-Brains |
|-------|---------------|---------------------|
| Unit / integration (Rust host) | Mock runtime; nextest | **Already used** — extend, do not replace |
| Frontend unit | `@tauri-apps/api/mocks` **`mockIPC` / `mockWindows` / `clearMocks`** — docs use **Vitest** + **mandatory jsdom crypto polyfill** | **Primary FE unit path** |
| Desktop E2E (full binary) | WebDriver via **WebdriverIO + `@wdio/tauri-service`** (MIT); embedded provider now covers macOS too | **Soft / optional** — `tauri-plugin-wdio*` must not ship in release builds (cfg/dev only if ever added); T179 may revisit |
| Renderer-only | **Playwright against Vite preview + init-script mock** | **Primary offline CI E2E** |

**Decision freeze (T174):**

| Choice | Decision | Why |
|--------|----------|-----|
| Unit runner | **Vitest 4.x** (jsdom) | Vite-native; Tauri mockIPC docs use Vitest |
| **Vitest browser-mode (`@vitest/browser-playwright`)** | **Rejected for L3** | Unifies runner but lacks first-class `toHaveScreenshot` + `webServer` + Trace Viewer parity; keep standalone Playwright for L3/L4 (**B6**) |
| Component helpers | **@testing-library/react** + **user-event** + **jest-dom** | React 19 peers; user-event for keyboard honesty |
| DOM env | **jsdom** + **setupFiles polyfills** (crypto + HTMLDialogElement) | Completeness; polyfills unblock ConfirmDialog (**AI3**) |
| Renderer E2E | **@playwright/test** (Apache-2.0) standalone | ADR-0017 + #46; screenshots + ARIA snapshots + webServer |
| Full WebView2 binary E2E | **Human checklist + soft automation** | #48 residual; WDIO soft (embedded provider exists; still soft for T174) |
| **Structural regression (L4 primary)** | Playwright **`toMatchAriaSnapshot`** | Cross-machine structural gate; avoids pixel flakiness from scaling/GPU (**AI2**) |
| **Pixel visual (L4 secondary)** | **`toHaveScreenshot`** for **critical dialog/chrome only** | Offline/denied/empty **prefer ARIA**; wipe dialog + optional chrome may use pixels with pinned render (**B4**) |
| A11y automation | Soft **`@axe-core/playwright` only** (not separate `axe-core` dep) | MPL-2.0 devDep; not DoD (**B15**); not mandatory CI (**reject AI2 D26 hard gate**) |

**Avoid:**

| Item | Why |
|------|-----|
| Jest as primary runner | Duplicates Vite config |
| Cypress as required gate | Extra stack |
| AGPL visual recorders / mandatory proprietary screenshot cloud | Commercial + privacy |
| Real third-party network | Project test rules |
| Live user vault mutation in CI | Stop-before |
| `waitForTimeout` / sleep-for-async | Auto-wait + web-first expect |
| `page.evaluate` for mock install after load | Wrong ordering — use **`context.addInitScript`** (**B3**) |
| `fireEvent` for keyboard paths | False greens on focus — **user-event only** (**B11**) |
| Shipping `tauri-plugin-wdio*` in production | Test-only; cfg-gate if ever |
| Separate `axe-core` + `@axe-core/playwright` | Version drift risk (**B15**) |
| Coverage % gates | Soft only |

### 3.2 Dependency pins (research 2026-07-30 — pin exact at implement)

All **devDependencies** only (except already-frozen product deps).

| Package | Research latest | License | Role |
|---------|-----------------|---------|------|
| **vitest** | **4.1.10** | MIT | Unit/component runner; peers Vite 6–8 ✓ |
| **jsdom** | **30.0.1** | MIT | Vitest environment (**no** native `showModal`) |
| **@testing-library/react** | **16.3.2** | MIT | React 18/19 peers ✓ |
| **@testing-library/dom** | peer of RTL | MIT | Explicit if peer requires |
| **@testing-library/jest-dom** | **7.0.0** | MIT | Matchers; **engines node ≥22** (**B5**) |
| **@testing-library/user-event** | **14.6.1** | MIT | Keyboard/typed WIPE — **per-test `userEvent.setup()`** |
| **@playwright/test** | **1.62.1** | **Apache-2.0** | Renderer E2E + ARIA + screenshots |
| **playwright** browsers | same band | Apache-2.0 | `npx playwright install chromium` |
| **@axe-core/playwright** | **4.12.1** | MPL-2.0 | Soft only; **do not** also add bare `axe-core` |
| **happy-dom** | **20.11.1** | MIT | Soft alt — not default |
| **@vitest/coverage-v8** | **4.1.10** | MIT | Soft only |
| **@vitest/browser-playwright** | — | MIT | **Not adopted** for L3 (**B6**) |

**Frozen product / host stack — do not bump in T174:**

| Package | Pin | Note |
|---------|-----|------|
| react / react-dom | 19.2.8 | ADR-0017 |
| vite | 8.1.5 | ADR-0017 |
| typescript | 7.0.2 | ADR-0017 |
| @tauri-apps/api | 2.11.1 | provides `mocks` |
| lucide-react | 0.468.0 | T172/T173 freeze |
| httpmock (Rust) | **0.7** (desktop pin) | **Not workspace.** **0.7 → 0.8.x is breaking** (API rework); stay on 0.7 for T174; upgrade = separate INFRA chore (**B13**) |

#### License gate design (**B1**)

| Tree | Gate | Policy |
|------|------|--------|
| **Production** | `npm run license:check` | **By design production-scoped** (`--production` in `license-check.mjs`). Fail on GPL/AGPL. |
| **DevDependencies** | **Manual review at add-time** | Record pins + licenses in implement `review.md`. Soft axe MPL-2.0 is allowed **only** as devDep and must not enter production bundle. |
| **Not required** | `license:check:dev` | Deferred unless MPL/dev-tree pain appears; option (b) residual if needed later |

### 3.3 Playwright practices + determinism (**B4**, **B7**, **B8**, **B14**)

- Prefer **role locators**: `getByRole` → `getByLabel` → `getByText` → `getByTestId` last.
- **Web-first assertions** only; **no hard sleeps**.
- **Navigation:** all routes via **`gotoRoute(page, 'review')` → `baseURL + '#/' + route`**. Direct `page.goto('/path')` is **forbidden** (HashRouter; Vite preview 404s path URLs) (**B8**).
- **Boot strategy (locked):** Playwright `webServer`:
  - `command`: build then preview (e.g. `npm run build ; npm run preview` on PowerShell, or a small `scripts/e2e-serve.mjs` that chains them)
  - `url`: `http://127.0.0.1:4173` (or preview port)
  - `reuseExistingServer: !process.env.CI`
  - Build failure must surface as **webServer start/timeout failure**, not “element not found” (**B7**).
  - **No Vite dev server for CI/e2e** (HMR inject + different module graph → flaky).
- **Retries / traces (**B14**):** `retries: process.env.CI ? 2 : 0`; `trace: 'on-first-retry'`; `video: 'retain-on-failure'`.
- **L4 structural (primary):** `toMatchAriaSnapshot` for offline / denied / empty review / connectors honesty / wipe dialog structure (**AI2**).
- **L4 pixel (secondary, critical only):** `toHaveScreenshot` with:
  - Fixed viewport **`{ width: 1280, height: 720 }`**
  - `devices['Desktop Chrome']` + **`headless: true`** for baseline generation **and** CI (same mode always)
  - Launch arg **`--font-render-hinting=none`** (and/or disable subpixel positioning) for text raster stability (**B4**)
  - Windows-first baselines; regenerator-machine-specific — regeneration PR from same Windows class or full re-baseline
  - README: pixel visual is **advisory-gate** on non-generating machines (structural ARIA remains strict)
- Snapshot PNGs: **binary** in `.gitattributes` (`-diff`); **not** Git LFS for small UI crops (**B16**).

### 3.4 Mock strategy (offline-first) — locks **B2**, **B3**, **B9**, **AI3**

| Layer | Mock | Real I/O |
|-------|------|----------|
| **L1 Rust unit** | httpmock loopback; pure validators | tempfile tokens; no live daemon |
| **L2 Vitest** | See **setupTauriMocks()** below | No daemon |
| **L3 Playwright** | **`context.addInitScript` only** (not `page.evaluate` after load) installs mockIPC / `__TAURI_INTERNALS__` **before** any app module eval (**B3**) | `build` + `preview` only; no T161 |
| **L4** | Same as L3 fixtures | Chromium headless pinned |
| **L5 Live WebView2** | Optional live daemon + user token | Human/soft; operator vault + consent |

#### L2 helper lock — `setupTauriMocks()` (**B2**, **B9**, **AI3**)

Canonical pattern (Tauri mocking docs + review):

1. **Idempotent crypto polyfill** (once, e.g. `beforeAll` / setupFiles):  
   `Object.defineProperty(window, 'crypto', { value: { getRandomValues: (buf) => randomFillSync(buf) } })`  
   Import `randomFillSync` from Node `crypto`. Guard if already present.
2. **HTMLDialogElement polyfill** in setupFiles (**AI3**): if `!HTMLDialogElement.prototype.showModal`, stub `showModal` (set `open` attr) and `close` (remove attr + dispatch `close`). **Does not** restore real Escape/focus-trap — those stay **L3 only**.
3. Per suite/test: `mockIPC(handler)`.
4. **`afterEach`:** `clearMocks()` (**Tauri**) **and** `vi.restoreAllMocks()` (**Vitest**) (**B9**).
5. Soft regression: assert invoke without crypto polyfill fails (proves polyfill is load-bearing).

#### L3 harness lock (**B3**)

- Use **`browser.newContext()` + `context.addInitScript(...)`** (or project-level `contextOptions` / fixture) so mock runs **before** the Vite production bundle evaluates `@tauri-apps/api/core` invoke binding.
- Single init-script helper file under `e2e/helpers/` — no HMR path.
- Smoke: after first `gotoRoute`, assert `window.__TAURI_INTERNALS__` is defined (via `page.evaluate`).

#### Fixture honesty + sync (**D25** / AI2)

- Mock JSON fixtures are **contract-shaped** (E1: `items: []` not null; warnings arrays).
- Shared under `e2e/fixtures/` (and/or `src/test/fixtures/`).
- **D25:** At least the critical fixtures used by L2/L3 for review list, wipe, offline/denied error shapes are either (a) deserialized in a Rust unit test via `serde_json` against expected fields, **or** (b) validated by a shared schema comment + FE shape assert. Prefer (a) for 1–2 golden fixtures without inventing a full codegen pipeline.

### 3.5 Layer map (test pyramid)

```text
        ┌─────────────────────────────┐
        │ L5 Live WebView2 smoke      │  human / soft (Isolation + keyboard)
        │ (binary + real IPC)         │
        ├─────────────────────────────┤
        │ L4 Structural ARIA + pixel  │  toMatchAriaSnapshot primary;
        │    critical screenshots     │  toHaveScreenshot dialog-only
        │ L3 Renderer E2E             │  Playwright + addInitScript mock
        ├─────────────────────────────┤
        │ L2 Component / unit FE      │  Vitest + RTL + mockIPC + polyfills
        ├─────────────────────────────┤
        │ L1 Rust host contracts      │  nextest — csp/open/http (+ fixture sync)
        └─────────────────────────────┘
```

| Layer | Owner scripts | Gate expectation |
|-------|---------------|------------------|
| L1 | `cargo test -p ai-brains-desktop --lib` | **Required** (workspace gate) |
| L2 | `npm run test:unit` | **Required** for DoD; **Node ≥22** |
| L3 | `npm run test:e2e` | **Required** offline flows |
| L4 | `npm run test:visual` (ARIA + optional pixel) | **Required** ARIA for critical states; pixel advisory off generator host |
| L5 | Manual SMOKE + soft script | **Required once** for beta; residual OK if documented |

## 4. Requirements (D1–D27)

### 4.1 Architecture & safety

| ID | Requirement |
|----|-------------|
| **D1** | Tests must **not** weaken production CSP, Isolation, capabilities, or dual-layer opener. |
| **D2** | Adapter-only honesty: FE tests assert UI **renders** DTOs; no domain policy only in tests. |
| **D3** | No webview `fetch` to T161 in “prod path” harnesses — invoke/mockIPC only. |
| **D4** | No live vault mutation in automated suites. |
| **D5** | No real third-party network. Loopback + mocks only. |
| **D6** | Capture independence: desktop test gate is additive to CLI/capture. |

### 4.2 Rust host (L1)

| ID | Requirement |
|----|-------------|
| **D7** | Keep/extend `csp_tests` / `capability_tests` / open validators / httpmock tests. |
| **D8** | Progressive: each AppManifest command has ≥1 shape or error-map test (soft full matrix). |
| **D9** | Dual-layer opener cases remain green; no `allow-default-urls`; no bare unscoped open-path; no npm opener package. |
| **D25** | Critical FE mock fixtures stay in sync with Rust/DTO field expectations (serde or shape assert). |

### 4.3 Frontend unit (L2)

| ID | Requirement |
|----|-------------|
| **D10** | Vitest + jsdom; `test` / `test:unit` non-watch. **Node ≥22 hard floor** (package engines + jest-dom 7); document in README/CI runbook (**B5**). Prefer **`test:` block in `vite.config.ts`** over separate vitest.config unless conflict (**B12**). |
| **D11** | `setupTauriMocks()`: crypto polyfill + dialog polyfill + `mockIPC`; afterEach **`clearMocks` + `vi.restoreAllMocks`**. |
| **D12** | Component coverage: `StatePanel` (+ StatusBadge), `ConfirmDialog` **with open=true under dialog polyfill** (typed WIPE match/mismatch, Enter focuses Confirm via **user-event**, aria-live). **Escape / focus-trap: L3 only** (polyfill does not restore native Escape) (**AI3**). `ScopeIndicator`. Source locator honesty (missing → no invent). Soft: Review/Erasure screens. |
| **D13** | `lib/errors.ts` kind mapping tests. Keyboard interactions use **`userEvent.setup()` per test** + `await user.type` / `user.keyboard`; **forbid `fireEvent` for keyboard** (**B11**). |
| **D27** | setupFiles polyfills: crypto `getRandomValues` + `HTMLDialogElement.showModal/close` stubs (**B2**, **AI3**). |

### 4.4 Playwright renderer E2E (L3)

| ID | Requirement |
|----|-------------|
| **D14** | Playwright Chromium; `playwright.config.ts`; retries/trace/video per §3.3; webServer build+preview only. |
| **D15** | Offline Home: StatusBadge offline promptly (`retry: false`). |
| **D16** | Review: empty list; resolve dialog open/cancel (**Escape in real Chromium**)/confirm; warnings render. |
| **D17** | Erasure: dry_run no WIPE; execute requires exact `WIPE`; Enter does not auto-submit; Escape cancels. |
| **D18** | Connectors/retention honest unavailable. |
| **D19** | T173 handoff cases 1–10 mapped to evidence. |
| **D26** | **Source locator honesty (case 8):** fixtures for https → Open; path → Reveal; null/missing → “No locator available” and **no** fabricated open button (**B10**). |

### 4.5 Visual / structural (L4)

| ID | Requirement |
|----|-------------|
| **D20** | **ARIA snapshots** for critical states: offline, denied, empty review, wipe dialog structure, connectors unavailable. |
| **D21** | Optional **pixel** baselines for wipe dialog (and optionally chrome) under §3.3 pin rules; committed under `apps/desktop/e2e/`; Windows-first; update command documented. |
| **D22** | Mask volatile fields; `.gitattributes` binary for snapshot PNGs; no LFS required for small crops. |

### 4.6 Live smoke & beta (L5 + gate)

| ID | Requirement |
|----|-------------|
| **D23** | Live WebView2 Isolation smoke (#48): start → ping → briefing or review list. Human evidence OK. |
| **D24** | Offline beta checklist: primary ops offline-honest; no UI-only authority; security cases closed or residual-listed. |

## 5. T173 handoff security cases → ownership

| # | Case | Layer | Automate? |
|---|------|-------|-----------|
| 1 | Prod CSP no HMR/unsafe; frame-src | L1 | Keep |
| 2 | open_url refuse http/file/js | L1 | Keep |
| 3 | capabilities not default-urls; CAPABILITY sync | L1 | Keep |
| 4 | Wipe typed WIPE; Enter focuses Confirm; Escape | L2 (phrase/Enter/aria-live) + **L3 Escape** | **New** |
| 5 | Keyboard review without pointer | L3 + L5 | Partial L3; full L5 |
| 6 | Focus-visible on nav + dialog | L3 CSS/ARIA or pixel | **New** |
| 7 | Isolation boots + ping | L1 conf + **L5 live** | Live #48 |
| 8 | Source locator open/reveal/missing | L2 + **L3 D26** + L1 | **New** explicit |
| 9 | No `@tauri-apps/plugin-opener` | L1 / package assert | Keep |
| 10 | Offline/denied StatusBadge not color-only | L2 + L4 ARIA | **New** |

## 6. Acceptance criteria (DT1–DT20)

| ID | Behavior |
|----|----------|
| **DT1** | `npm run test:unit` green offline (Node ≥22) |
| **DT2** | `npm run test:e2e` green: offline Home + Review + Erasure WIPE + Source locator (D26) |
| **DT3** | ARIA snapshots present for offline / denied / empty / wipe structure; pixel optional under pins |
| **DT4** | Rust desktop lib tests green |
| **DT5** | T173 cases 1–10 each have evidence |
| **DT6** | Live Isolation smoke or residual with owner |
| **DT7** | Keyboard review evidence (L3 and/or human) |
| **DT8** | Offline beta checklist signed |
| **DT9** | `license:check` (prod) + deny + audit green; axe MPL recorded in review if added |
| **DT10** | README: unit/e2e/visual/update snaps; Node ≥22; HashRouter `gotoRoute`; visual advisory off-host |
| **DT11** | No unjustified production-only test deps |
| **DT12** | No sleep-only waits |
| **DT13** | afterEach: `clearMocks` + `vi.restoreAllMocks`; init-script order smoke green |
| **DT14** | Connectors honest unavailable |
| **DT15** | offline prompt / retry:false regression-locked |
| **DT16** | CI path docs: Node **22**, Playwright chromium install, unit + e2e |
| **DT17** | deferred #46/#48 updated |
| **DT18** | Conductor Completed only after DoD + review |
| **DT19** | Dialog polyfill present; ConfirmDialog L2 tests open without throw |
| **DT20** | webServer is build+preview only; no dev-server CI path |

## 7. Desktop beta acceptance (phase rollup)

| Criterion | Evidence owner |
|-----------|----------------|
| Scaffold (T171) | ✅ Complete |
| Minimum screens (T172) | ✅ Complete |
| Security/UX (T173) | ✅ Complete (P3 live smoke → T174) |
| Tests (T174) | **This track** |
| Primary ops offline | DT2 + DT8 |
| No UI-only authority | D2 + review/erasure/source tests |
| Isolation live | D23 / #48 |
| Keyboard operable review | DT7 |

## 8. File layout (expected)

```text
apps/desktop/
  package.json                 # test:unit, test:e2e, test:visual
  vite.config.ts               # + test: { environment: 'jsdom', setupFiles: [...] }  (preferred)
  src/test/setup.ts            # crypto + dialog polyfills
  src/test/setupTauriMocks.ts  # mockIPC helper
  src/**/*.test.tsx
  playwright.config.ts         # webServer build+preview; retries; viewport; font args
  e2e/
    fixtures/                  # contract-shaped JSON (D25)
    helpers/
      mockInvoke.ts            # addInitScript payload
      gotoRoute.ts             # baseURL + '#/' + path
    offline.spec.ts
    review.spec.ts
    erasure.spec.ts
    source.spec.ts             # D26 locator honesty
    honesty.spec.ts            # connectors etc.
    visual.spec.ts             # ARIA (+ optional pixel) @visual
  .gitattributes               # or repo root: e2e/**/*-snapshots/*.png binary -diff
  README.md
src-tauri/                     # L1 only; no WDIO in release
```

## 9. Deferred.md absorption

| Deferred | Disposition |
|----------|-------------|
| **#46** Playwright / visual / offline beta | **Absorb** |
| **#48** Live Isolation + keyboard GUI | **Absorb** D23/DT6/DT7 |
| Isolation cannot deny IPC (C13) | Document only |
| Path capability `"**"` breadth | Document residual |
| T173 soft axe | Soft `@axe-core/playwright` only; manual MPL review |
| Multi-OS visual / WDIO matrix | **T179** — note embedded WDIO provider exists |
| `license:check:dev` positive gate | Residual if needed later (B1 option b) |

## 10. Non-goals

| Out of scope | Owner |
|--------------|--------|
| macOS/Linux CI matrix | **T179** |
| Mandatory axe CI gate | Soft only (reject hard D26) |
| Vitest browser-mode as L3 | Rejected B6 |
| Load testing | Future |
| Electron / prod CSP weaken | Forbidden |
| httpmock 0.8 upgrade | Separate chore |
| Required WDIO release plugins | Forbidden |

## 11. Definition of Done

- [ ] D1–D27 satisfied or residual-listed  
- [ ] DT1–DT20 green  
- [ ] Offline beta checklist in `evidence/`  
- [ ] T173 cases 1–10 evidenced  
- [ ] Live Isolation + keyboard done **or** residual  
- [ ] README + Node ≥22 + visual regen policy  
- [ ] deferred #46 absorbed; #48 closed or residual  
- [ ] license:check (prod) + deny + audit + unit + e2e green  
- [ ] Conductor → Completed after review  

## 12. Risks

| Risk | Mitigation |
|------|------------|
| Playwright ≠ WebView2 | L5 human Isolation smoke |
| Pixel flakiness | ARIA primary; pixel rare + pinned viewport/headless/font (**B4**) |
| mockIPC order flakes | **addInitScript only** (**B3**); internals smoke |
| jsdom no showModal | setupFiles polyfill; Escape only L3 (**AI3**) |
| mockIPC + vitest spy leak | clearMocks + restoreAllMocks (**B9**) |
| license script scope confusion | Document prod-only (**B1**) |
| WDIO plugins in release | Soft only; cfg-gate; T179 may use embedded provider |
| CI Node 20 | engines + runbook Node 22 (**B5**) |
| Fixture drift | D25 golden fixture sync |

## 13. Implementation priority

1. Phase A tooling + setupFiles polyfills + playwright webServer lock  
2. L2: errors + StatePanel + ConfirmDialog (polyfill) + user-event WIPE  
3. L3: offline + review + erasure + **source locator**  
4. L4: ARIA snapshots; optional pixel wipe dialog  
5. L1 gaps + D25 fixture sync  
6. L5 human Isolation + keyboard  
7. Beta checklist + README + closeout  

## 14. Open questions (resolved defaults)

| Question | Default |
|----------|---------|
| Vitest vs Jest? | **Vitest 4.x** |
| Vitest browser-mode vs standalone Playwright for L3? | **Standalone Playwright** (screenshots, webServer, traces) (**B6**) |
| Playwright vs WDIO primary? | **Playwright** renderer; WDIO soft (embedded provider exists for later) |
| DOM env? | **jsdom** + polyfills |
| Visual primary? | **`toMatchAriaSnapshot`**; pixel secondary for critical dialogs (**AI2** + **B4**) |
| license:check scope? | **Production only by design**; axe MPL manual at add-time (**B1a**) |
| ConfirmDialog Escape under L2? | **No** — L3 only (**AI3**) |
| axe required CI? | **Soft** — not hard DoD |
| Live Isolation every PR? | **No** — once for beta |
| httpmock bump? | **Stay 0.7** |
| Bump product deps? | **No** |

## 15. License / commercial gate

```text
# Rust
cargo deny check
cargo audit
cargo test -p ai-brains-desktop --lib

# Frontend (apps/desktop) — Node ≥22
npm run license:check          # production tree ONLY (by design)
npm run test:unit
npm run test:e2e
npm run test:visual            # ARIA required; pixel advisory off-host
```

| Check | Scope |
|-------|--------|
| `license:check` | Production deps; fail GPL/AGPL |
| DevDep licenses | Manual at add-time; record in `review.md` (e.g. `@axe-core/playwright` MPL-2.0) |
| Playwright | Apache-2.0 |
| Vitest / RTL / jsdom | MIT |

Fail if GPL/AGPL enters production tree. Do not flip license script off `--production` without a documented allowlist for axe.
