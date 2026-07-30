# T174 Offline Beta + Security Cases Checklist

**Track:** T174 Desktop Tests (P10.3)  
**Date:** 2026-07-30  
**Product pins:** react 19.2.8 · vite 8.1.5 · typescript 7.0.2 · @tauri-apps/api 2.11.1 · lucide-react 0.468.0 · httpmock 0.7  
**R1/Codex fixes:** full D25 · wipe execute · stale badge · http locator · retention honesty e2e · missing locator property · deny/audit recorded  

## Sign-off (DT8)

| Role | Name / agent | Date | Decision |
|------|--------------|------|----------|
| Implementer / orchestrator | Grok Build (T174) | 2026-07-30 | **APPROVED for offline beta engineering gate** — automated L1–L4 green; L5 live Isolation residual documented in SMOKE.md |
| Live WebView2 operator | — | pending | Required once before release packaging (not PR merge blocker) |

## Gate commands (all PASS)

```powershell
cd C:\dev\AI-Brains-wt-t174\apps\desktop
npm run license:check          # PASS — no GPL/AGPL production
npm run typecheck              # PASS (product sources; tests excluded)
npm run typecheck:tests        # PASS (vitest unit sources)
npm run test:unit              # PASS
npx playwright install chromium
npm run test:e2e               # PASS
npm run test:visual            # PASS
cd C:\dev\AI-Brains-wt-t174
cargo test -p ai-brains-desktop --lib   # PASS (incl. full D25 fixtures)
cargo clippy -p ai-brains-desktop --all-targets -- -D warnings  # PASS
cargo fmt --check -p ai-brains-desktop  # PASS
cargo deny check               # PASS — advisories ok, bans ok, licenses ok, sources ok
cargo audit                    # PASS exit 0 — 19 allowed warnings (unmaintained/unsound transitive; no blocking vulns)
```

## Offline beta cases

| Case | Expected | Evidence |
|------|----------|----------|
| Home offline StatusBadge promptly | Offline badge + icon + text; no retry delay | L3 `offline.spec.ts`; L2 StatePanel |
| Home stale StatusBadge | `[data-status="stale"]` + label + icon when `freshness.worst_state=stale` | L3 `stale.spec.ts` + fixture `briefing-stale.json` |
| Denied StatusBadge | Denied icon + text | L4 visual denied; L2 StatePanel |
| Empty review | Empty message; items `[]` not null | L3 review empty; D25 fixture Rust |
| Connectors unavailable | Honest unavailable copy | L3 `honesty.spec.ts`; L4 ARIA |
| Retention plan unavailable | Honest unavailable copy on Erasure | L3 `honesty.spec.ts` retention case (D18) |
| Wipe dry-run | No typed WIPE required | L3 `erasure.spec.ts` dry_run |
| Wipe execute | WIPE required; Enter no auto-submit; Escape cancels; **Execute wipe closes dialog + success/honesty** | L2 ConfirmDialog + L3 erasure (escape + happy path) |
| Source locator honesty (D26) | https→Open; path→Reveal; null→no button; **missing key→no button**; **http→display only** | L3 `source.spec.ts`; L2 classifyLocator |
| CSP/Isolation not weakened | prod CSP + isolation still locked | L1 `csp_tests` + `capability_tests` |
| No JS opener package | package.json free of `@tauri-apps/plugin-opener` | L1 `package_json__must_not_include_js_opener_plugin` |
| license:check production only | B1 | `scripts/license-check.mjs --production` |
| Full D25 fixture sync | All `e2e/fixtures/*.json` contract-shaped | L1 `fixture_sync_tests` in `lib.rs` |

## T173 security cases 1–10

| # | Case | Evidence |
|---|------|----------|
| 1 | CSP fixture: no localhost:1420 / unsafe-inline / unsafe-eval; has frame-src | L1 `csp_tests` |
| 2 | Open refuse http/file/javascript | L1 open validators dual-layer unit tests; L2 classifyLocator text for non-https schemes |
| 3 | Capability not default-urls; mirror constants | L1 `capability_tests` + open tests |
| 4 | Wipe typed WIPE; Enter focus; Escape; **execute success path** | L2 ConfirmDialog + L3 erasure (escape + execute happy path) |
| 5 | Keyboard review path | L3 Escape + residual full GUI L5 |
| 6 | Focus-visible | CSS T173; residual live tab-through L5 |
| 7 | Isolation app boots | Structural conf + residual live L5 |
| 8 | Source locator honesty | L3 source.spec (https/path/null/**http**) + L2 classifyLocator |
| 9 | No JS plugin-opener in production tree | L1 + license:check |
| 10 | Offline/denied/**stale** StatusBadge icons | L2 StatePanel offline/denied + **L3 `stale.spec.ts`** (`[data-status="stale"]`) + L3 offline + L4 ARIA |

## Residuals (P3)

| Item | Owner |
|------|-------|
| Live WebView2 Isolation + daemon smoke | T174 operator / human (SMOKE.md) |
| Full keyboard-only GUI traversal | T174 / human |
| Pixel screenshot cross-host drift | Advisory; ARIA primary |
| Soft D8 progressive AppManifest command-shape matrix | Soft residual — httpmock covers key commands; full matrix not required for DoD |

## Dep license table (new test tooling)

| Package | License | Role |
|---------|---------|------|
| vitest | MIT | L2 runner |
| jsdom | MIT | L2 DOM |
| @testing-library/* | MIT | L2 RTL |
| @playwright/test | Apache-2.0 | L3/L4 |
| (no bare axe-core) | — | not added |
