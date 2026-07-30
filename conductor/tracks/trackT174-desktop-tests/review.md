# T174 Review Log — Desktop Tests & Offline Beta Gate

**Track:** T174  
**Implementer:** Grok Build  
**Date:** 2026-07-30  
**Ledger TX:** `e76e1293-59a5-4297-ba1a-45cceb3df86a`

## Implementer summary

Offline-first test pyramid landed for `apps/desktop`:

- **L1:** Existing host tests retained; D25 golden fixture sync for **all** `e2e/fixtures/*.json` via `include_str!` + serde_json field asserts.
- **L2:** Vitest 4.1.0 + jsdom 30 + RTL 16.3 + user-event 14.6; polyfills for crypto + HTMLDialogElement; unit tests green (incl. openUrl/revealPath invoke + http locator honesty).
- **L3:** Playwright 1.62.0 Chromium; build+preview webServer only; `addInitScript` Tauri invoke mock only (no `patchMockInvoke`); HashRouter `gotoRoute`; e2e green (incl. wipe execute happy path, stale badge, http locator).
- **L4:** ARIA snapshots primary; optional wipe-dialog PNG golden under pinned 1280×720.
- **L5:** Human SMOKE documented; **live WebView2 not run** in agent session (residual).

No product dependency bumps. No `@tauri-apps/plugin-opener` npm. No AGPL. CSP/Isolation/opener not weakened.

**Product fix (R1-05):** `classifyLocator` treats non-https URI schemes (`http:`, `file:`, `javascript:`, …) as display-only `text` (never path / never open).

## Dep license table (devDependencies added)

| Package | Version | License |
|---------|---------|---------|
| vitest | 4.1.0 | MIT |
| jsdom | 30.0.0 | MIT |
| @testing-library/react | 16.3.0 | MIT |
| @testing-library/jest-dom | 7.0.0 | MIT |
| @testing-library/user-event | 14.6.1 | MIT |
| @testing-library/dom | 10.4.0 | MIT |
| @playwright/test | 1.62.0 | Apache-2.0 |
| axe-core / @axe-core/playwright | — | **not added** |

Production tree unchanged for license:check (`--production` only — B1).

## Gates (implementer — after R1)

| Gate | Result |
|------|--------|
| `npm run license:check` | PASS |
| `npm run typecheck` | PASS |
| `npm run typecheck:tests` | PASS |
| `npm run test:unit` | PASS (38) |
| `npm run test:e2e` | PASS (17) |
| `npm run test:visual` | PASS (5) |
| `cargo test -p ai-brains-desktop --lib` | PASS (58; 12 D25 fixtures) |
| `cargo clippy -p ai-brains-desktop --all-targets -- -D warnings` | PASS |
| `cargo fmt --check -p ai-brains-desktop` | PASS |

## Findings — Internal Review R1 (2026-07-30)

| ID | Severity | Status | Disposition |
|----|----------|--------|-------------|
| T174-R1-01 | medium | verified_fixed | Restored full `spec.md`/`plan.md`; Status → Implementing (R1 fixes); plan checkboxes marked for done work; this review log updated. |
| T174-R1-02 | medium | verified_fixed | D25 `fixture_sync_tests` expanded to all golden fixtures under `e2e/fixtures/` (arrays not null, required keys). |
| T174-R1-03 | medium | verified_fixed | L3 wipe execute happy path: type WIPE → Execute wipe → dialog closes + success/honesty UI; fixture `wipe-execute.json`. Escape cancel remains separate test. |
| T174-R1-04 | medium | verified_fixed | L3 `stale.spec.ts` mocks `project_briefing` with `freshness.worst_state=stale`; asserts `[data-status="stale"]` + label. BETA_CHECKLIST case 10 updated. |
| T174-R1-05 | medium | verified_fixed | Product: `classifyLocator` non-https schemes → `text`; L2 + L3 source-http prove no Open/Reveal. |
| T174-R1-06 | medium | verified_fixed | Removed `patchMockInvoke`, `__AIB_MOCK_TABLE__`, and unused `Page` import; keep `installMockInvoke` + `addInitScript` only. |
| T174-R1-07 | low | verified_fixed | L2 `openUrl`/`revealPath` invoke call capture via `setupTauriMocks` + `afterEach cleanupTauriMocks`. |
| T174-R1-08 | low | verified_fixed | `tsconfig.vitest.json` + `npm run typecheck:tests`; README Testing section documents script. |

## Findings — Internal Review R2 (2026-07-30)

| ID | Severity | Status | Disposition |
|----|----------|--------|-------------|
| T174-R2-01 | low | verified_fixed | DT13/B3: `offline.spec.ts` asserts `window.__TAURI_INTERNALS__.invoke` is a function after first `gotoRoute` (e2e offline re-run PASS). |
| T174-R2-02 | low_info | deferred | Optional retention-copy e2e assert; connectors honesty already automated. |

## Findings — Codex Review R1 (2026-07-30)

| ID | Severity | Status | Disposition |
|----|----------|--------|-------------|
| Codex-R1-P1-DT9 | P1 | verified_fixed | `cargo deny check` PASS; `cargo audit` PASS (exit 0, allowed warnings only). Recorded in BETA_CHECKLIST + this log. |
| Codex-R1-P1-DT8 | P1 | verified_fixed | BETA_CHECKLIST signed: implementer APPROVED offline beta engineering gate 2026-07-30; live operator pending for release packaging only. |
| Codex-R1-P2-D18 | P2 | verified_fixed | L3 `erasure__retention_plan__honest_unavailable_copy` in `honesty.spec.ts` (e2e PASS). |
| Codex-R1-P2-D26 | P2 | verified_fixed | Fixture `source-no-locator-key.json` (locator key absent) + L3 + D25 Rust assert (PASS). |
| Codex-R1-P3-D8 | P3 soft | deferred | Soft progressive command-shape matrix residual-listed in BETA_CHECKLIST / deferred. |
| Codex-R1-P3-gitattributes | P3 | verified_fixed | Trailing blank line removed from `.gitattributes`. |

## Residuals / deferred

| Item | Severity | Owner |
|------|----------|-------|
| Live Isolation + daemon WebView2 smoke | low_info / P3 | T174 operator (SMOKE.md) |
| Full keyboard-only GUI path in live WebView | low_info / P3 | T174 / human |
| Cross-host pixel PNG drift | low_info | ARIA primary; update via `test:e2e:update` only |
| Soft D8 full AppManifest command-shape matrix | soft / P3 | residual (not DoD hard) |

## Codex Review R2 (fresh — 2026-07-30)

**Verdict: PASS WITH DEFERRED P3**

- No P0/P1/P2 remaining.
- Prior R1 findings all `verified_fixed`.
- Deferred P3 only: live WebView2 L5, full keyboard GUI, soft D8 matrix, pixel host drift.
- Raw: `review.codex.md` (R1 FAIL) + `review.codex.r2.md` (PASS WITH DEFERRED P3).

## Final gate snapshot (orchestrator)

| Gate | Result |
|------|--------|
| unit | 38 PASS |
| e2e | 19 PASS |
| visual | 5 PASS |
| typecheck / typecheck:tests | PASS |
| cargo test -p ai-brains-desktop --lib | 59 PASS |
| clippy desktop | PASS |
| cargo deny check | PASS |
| cargo audit | PASS exit 0 (allowed warnings) |
| license:check (prod) | PASS |

## Reviewer checklist

- [x] Confirm product pins untouched
- [x] Confirm no CSP/Isolation regression
- [x] Confirm license:check still production-only
- [x] Spot-check e2e mock does not claim live daemon
- [x] Mark findings `verified_fixed` after gate re-run
- [x] Fresh Codex R2 clean before gate clear
