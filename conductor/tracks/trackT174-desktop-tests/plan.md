# T174 Plan — Desktop Tests & Offline Beta Gate (P10.3)

Status: **Implementing** (internal review R1 fixes, 2026-07-30).

Authority: `spec.md` locks **D1–D27**, **DT1–DT20**. Offline-first mocks; Playwright renderer primary; ARIA structural primary; live WebView2 human/soft.

## Phase 0 — Preconditions

- [x] Confirm T171–T173 Complete on main (Isolation, opener dual-layer, typed WIPE, nine screens, ConfirmDialog `showModal`).
- [x] **Node ≥22** hard floor (jest-dom 7 + package engines); npm ≥10; WebView2 for L5.
- [x] Re-pin at implement: `vitest@4.1.x`, `@playwright/test@1.62.x`, `@testing-library/react@16.3.x`, jsdom 30.x.
- [x] Confirm `license-check.mjs` remains **`--production`** (B1).
- [x] `ledgerful doctor`; ledger start when implementing.
- [x] Read T173 `evidence/SMOKE.md` handoff cases 1–10.
- [x] Product pins frozen (React/Vite/TS/lucide/tauri-api/httpmock **0.7**).

## Phase A — Tooling freeze

- [x] Add **devDependencies** only:
  - `vitest`, `jsdom`
  - `@testing-library/react`, `@testing-library/jest-dom`, `@testing-library/user-event` (+ `@testing-library/dom` if needed)
  - `@playwright/test`
  - Soft: **`@axe-core/playwright` only** (do **not** also add bare `axe-core`) (B15) — not added
- [x] Prefer **`test:` block in `vite.config.ts`** (import from `vitest/config` if needed) over separate `vitest.config.ts` (B12). Live app has **no path aliases**.
- [x] `src/test/setup.ts` (setupFiles) — **both** polyfills:
  - [x] Crypto: idempotent `window.crypto.getRandomValues` via Node `randomFillSync` (B2; Tauri docs)
  - [x] Dialog: stub `HTMLDialogElement.prototype.showModal` / `close` if missing (AI3)
- [x] `src/test/setupTauriMocks.ts`:
  - [x] `mockIPC` install helper
  - [x] afterEach: **`clearMocks()` + `vi.restoreAllMocks()`** (B9)
  - [ ] Soft: regression that invoke without crypto polyfill throws
- [x] `playwright.config.ts` locks:
  - [x] Chromium only; `testDir: e2e`
  - [x] **webServer:** chain **build then preview** (PowerShell-friendly script if needed); `url` wait on preview port; `reuseExistingServer: !CI` (B7)
  - [x] **No vite dev server** for e2e/CI
  - [x] `retries: process.env.CI ? 2 : 0`; `trace: 'on-first-retry'`; `video: 'retain-on-failure'` (B14)
  - [x] Visual project: viewport `{ width: 1280, height: 720 }`; Desktop Chrome; `headless: true`; launch `--font-render-hinting=none` (B4)
- [x] Scripts:
  - [x] `test` / `test:unit` → vitest run
  - [x] `test:e2e` → playwright test
  - [x] `test:visual` → ARIA (+ optional pixel) project/tag
  - [x] `test:e2e:update` / snapshot update — document only, never silent CI auto-update
  - [x] `typecheck:tests` → `tsc --noEmit -p tsconfig.vitest.json` (R1-08)
- [x] Document `npx playwright install chromium` + **Node 22** in README.
- [x] `.gitattributes`: `apps/desktop/e2e/**/*-snapshots/**/*.png binary -diff` (B16); no LFS.
- [x] `npm run license:check` green (prod only).
- [x] Do **not** add `@tauri-apps/plugin-opener` or WDIO release plugins.

## Phase B — L1 Rust host gap fill

- [x] Inventory `csp_tests`, `capability_tests`, open validators, httpmock (stay **0.7** — no 0.8 bump) (B13).
- [x] Fill missing handoff cases 1–3, 9 only if holes.
- [x] **D25:** deserialize **all** critical shared fixture JSON under `e2e/fixtures/` with `serde_json` field asserts (R1-02 expanded).
- [ ] Soft: progressive command shape coverage (D8).
- [x] No production `unwrap`/`expect`.
- [x] `cargo test -p ai-brains-desktop --lib` green.

## Phase C — L2 Vitest component suite

- [x] Use `setupTauriMocks()` + setupFiles polyfills in all invoke/dialog tests.
- [x] Fixture map: command → DTO (shared with e2e where practical).
- [x] `lib/errors.test.ts` — offline/denied/transient/error.
- [x] `StatePanel` / StatusBadge — offline, denied, error, empty, loading; icon+text.
- [x] **`ConfirmDialog` with `open={true}`** under dialog polyfill (must not throw):
  - [x] Typed WIPE: Confirm disabled until exact match
  - [x] `userEvent.setup()` **per test**; `await user.type` / `user.keyboard` — **no fireEvent keyboard** (B11)
  - [x] Enter focuses Confirm (does not auto-submit)
  - [x] aria-live polite match/mismatch
  - [x] **Do not** assert Escape/focus-trap here — **L3 only** (AI3)
- [x] `ScopeIndicator` text badges.
- [x] **Source locator honesty (L2):** missing/null → “No locator available”; no fabricated Open/Reveal (B10).
- [x] http/file/javascript schemes → `kind: "text"` (R1-05); openUrl/revealPath invoke wrappers (R1-07).
- [ ] Soft: Review empty; Erasure dry_run vs execute entry.
- [x] `npm run test:unit` green offline on Node ≥22.

## Phase D — L3 Playwright renderer E2E

- [x] `e2e/helpers/gotoRoute.ts` — `page.goto(baseURL + '#/' + route)`; **forbid** `page.goto('/path')` (B8).
- [x] `e2e/helpers/mockInvoke.ts` — install via **`context.addInitScript` only** (not `page.evaluate` after load) (B3); `patchMockInvoke` removed (R1-06).
- [x] After first navigation: assert `window.__TAURI_INTERNALS__` defined.
- [x] Boot = webServer build+preview only (Phase A); document build-fail = webServer error (B7).
- [x] `e2e/offline.spec.ts` — Home offline StatusBadge promptly.
- [x] `e2e/stale.spec.ts` — Home stale StatusBadge from briefing freshness (R1-04).
- [x] `e2e/review.spec.ts` — empty list; resolve dialog; **Escape cancel** (real Chromium); confirm + warnings.
- [x] `e2e/erasure.spec.ts` — dry_run no WIPE; execute needs WIPE; Enter no auto-submit; Escape cancels; **execute happy path** (R1-03).
- [x] **`e2e/source.spec.ts` (D26 / case 8):**  
  - (a) `locator: "https://…"` → Open  
  - (b) file path → Reveal  
  - (c) null/missing → “No locator available”, no open button  
  - (d) `http://…` → display only, no Open/Reveal (R1-05)
- [x] `e2e/honesty.spec.ts` — Connectors (and retention if shown) unavailable copy.
- [x] Locators: prefer `getByRole` / `getByLabel`.
- [ ] Soft: Tab keyboard path review resolve; soft axe on Home+Review if dep added.
- [x] `npm run test:e2e` green offline.

## Phase E — L4 Structural ARIA + optional pixel

- [x] `e2e/visual.spec.ts` (`@visual`):
  - [x] **`toMatchAriaSnapshot`** for: offline Home, denied, empty Review, wipe dialog structure, connectors unavailable (AI2 primary)
  - [x] Optional **`toHaveScreenshot`** for wipe dialog only (or chrome) under pinned viewport/headless/font (B4)
- [x] Generate on **Windows** same headless mode as CI; commit snapshot dirs.
- [x] README: pixel advisory on non-generating hosts; ARIA strict; regen procedure; never silent CI update.
- [x] `npm run test:visual` green.

## Phase F — L5 Live WebView2 + keyboard (absorb #48)

- [x] Human checklist → `evidence/SMOKE.md` (documented; live not run in agent):
  1. App starts under Isolation  
  2. ping / connection info  
  3. briefing **or** review list (live daemon + user token; operator vault + consent)  
  4. Keyboard-only Review → resolve → Escape / complete  
  5. Focus-visible when tabbing  
  6. Typed WIPE in real WebView  
- [ ] Soft: CDP attach Playwright — not DoD if flaky.
- [ ] Soft: WDIO embedded provider — **not required**; residual for T179 (macOS embedded now exists; still cfg-gate plugins).
- [ ] Record PASS/FAIL + date + env. **Residual: live WebView2 not executed in agent session.**

## Phase G — Offline beta gate + docs

- [x] `evidence/BETA_CHECKLIST.md` (all primary honesty items + security cases 1–10 table).
- [x] Expand `apps/desktop/README.md` Testing:
  - Node ≥22  
  - unit / e2e / visual  
  - HashRouter `gotoRoute`  
  - build+preview webServer  
  - snapshot update + pixel advisory  
  - live smoke  
  - `typecheck:tests`
- [ ] Soft: OPERATIONS.md pointer.
- [x] Map DT1–DT20 → evidence rows.
- [ ] If axe added: record MPL-2.0 pin in `review.md` (B1a).

## Phase H — Gates, deferred, closeout

- [x] `cargo fmt --check` (if rust touched)
- [x] `cargo clippy -p ai-brains-desktop --all-targets -- -D warnings`
- [x] `cargo test -p ai-brains-desktop --lib`
- [x] `npm run typecheck` / `build` / `license:check` (prod)
- [x] `npm run test:unit` / `test:e2e` / `test:visual`
- [ ] Workspace gate or documented subset
- [ ] `ledgerful verify`
- [ ] deferred.md: absorb #46; close/residual #48; note #49 amendments
- [ ] conductor.md T174 → Completed **(held at In Progress until orchestrator clears R1)**
- [x] `review.md` + pin decisions (Playwright primary; ARIA>pixel; dialog polyfill; license prod-only)

## License gate

- [x] Playwright Apache-2.0 pin recorded
- [x] Vitest / RTL / jsdom MIT
- [x] **`license:check` = production only by design** (B1)
- [ ] axe MPL: manual review at add-time if present; only `@axe-core/playwright`
- [x] No AGPL e2e tools
- [ ] `cargo deny` green

## Priority order (implement)

1. Phase A (tooling + polyfills + webServer lock)  
2. Phase C (unit: errors, StatePanel, ConfirmDialog, source honesty)  
3. Phase D (e2e: offline, review, erasure, **source**)  
4. Phase E (ARIA + optional pixel)  
5. Phase B (rust gaps + D25)  
6. Phase F (live smoke)  
7. Phase G/H (beta + closeout)  

## Out of scope (do not pull in)

- T179 multi-OS matrix (note WDIO embedded for later)  
- Mandatory axe CI gate  
- Vitest browser-mode as L3  
- httpmock 0.8  
- T175+ sync tests  
- Electron / WDIO release plugins  
- Product dep major bumps  

## Review fold-in disposition (2026-07-30)

| ID | Sev | Disposition |
|----|-----|-------------|
| B1 | HIGH | **Fold** — license:check prod-only; axe MPL manual (option a) |
| B2 | HIGH | **Fold** — crypto polyfill lock + load-bearing soft test |
| B3 | HIGH | **Fold** — `context.addInitScript` only |
| B4 | HIGH | **Fold** — viewport/headless/font pins; pixel advisory off-host |
| B5 | MED | **Fold** — Node ≥22 CI/runbook |
| B6 | MED | **Fold** — reject Vitest browser-mode for L3 |
| B7 | MED | **Fold** — webServer build+preview only |
| B8 | MED | **Fold** — `gotoRoute` hash helper |
| B9 | MED | **Fold** — clearMocks + restoreAllMocks |
| B10 | MED | **Fold** — explicit source.spec + L2 honesty |
| B11 | MED | **Fold** — userEvent only for keyboard |
| B12 | MED | **Fold** — prefer vite.config `test:` block |
| B13 | LOW | **Fold** — httpmock 0.7 stay note |
| B14 | LOW | **Fold** — CI retries + trace + video |
| B15 | LOW | **Fold** — only @axe-core/playwright |
| B16 | LOW | **Fold** — gitattributes binary snapshots |
| AI2 ARIA primary | — | **Fold** — toMatchAriaSnapshot primary structural |
| AI2 D25 fixtures | — | **Fold** — light Rust/FE fixture sync |
| AI2 D26 mandatory axe | — | **Reject hard gate** — keep soft |
| AI3 showModal | HIGH | **Fold** — setupFiles polyfill; Escape L3-only |
| AI3 WDIO macOS embedded | — | **Fold** — T179 residual note only |
