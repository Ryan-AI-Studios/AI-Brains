# T173 Smoke / Gate Evidence

Date: 2026-07-30  
Worktree: `C:\dev\AI-Brains-wt-t173`  
Branch: `feat/t173-desktop-security-ux`  
Ledger TX: `a57911f7-6fc6-4bcf-9958-57e0e3bb9a05`

## Automated gates (Codex F-01..F-04 fix pass)

| Command | Result |
|---------|--------|
| `npm run typecheck` (apps/desktop) | **PASS** |
| `npm run build` (apps/desktop) | **PASS** (vite production build) |
| `npm run license:check` (apps/desktop) | **PASS** — no GPL/AGPL; no `@tauri-apps/plugin-opener` |
| `cargo test -p ai-brains-desktop --lib` | **PASS** — 46 tests (was 38; +capability-mirror dual-layer) |
| `cargo clippy -p ai-brains-desktop --all-targets -- -D warnings` | **PASS** |
| `cargo fmt -p ai-brains-desktop -- --check` | **PASS** |
| `cargo deny check` | **PASS** — advisories/bans/licenses/sources ok |
| `cargo audit` | **PASS** (exit 0) — 0 vulnerabilities; **19 allowed warnings** (unmaintained gtk/async-std/unic/proc-macro-error; unsound anyhow/glib — transitive/allowed) |
| `cargo check -p ai-brains-desktop` | **PASS** — isolation feature + plugins compile |

### cargo audit detail (F-04 / SU12)

```
Scanning Cargo.lock for vulnerabilities (688 crate dependencies)
warning: 19 allowed warnings found
```

Exit code **0**. No yanked/critical advisories blocking. Warnings are unmaintained (gtk-rs 0.18, async-std, unic-*, proc-macro-error) and known unsound (anyhow downcast_mut, glib VariantStrIter) — allowed by audit config / non-blocking for this track.

## Unit coverage added/extended

- `csp_tests`: prod CSP frame-src for Isolation iframe; isolation pattern mandated in conf
- `capability_tests`: no `opener:default` / `allow-default-urls` / bare `opener:allow-open-path`; https-only url scopes; no JS opener package in package.json
- `commands::open::tests`: https-only URL validators; path `..` / empty refusal
- **F-01 dual-layer (effective):**
  - `url_capability_allows` / `path_capability_allows` independent Layer-2 gates on custom command path
  - `default_json_url_allows_match_rust_mirror` keeps `CAPABILITY_URL_ALLOWS` / `CAPABILITY_PATH_ALLOWS` in sync with `capabilities/default.json`
  - dual_layer https passes both gates; http fails both

## F-01 dual-layer opener (effective on custom path)

| Layer | Where | Behavior |
|-------|--------|----------|
| Layer 1 validators | `open.rs` `validate_https_url` / `validate_reveal_path` | https-only; empty/`..`/device refuse |
| Layer 2 capability-mirror | `open.rs` `url_capability_allows` / `path_capability_allows` | Independent match of `https://*` / `**` (mirrors default.json) |
| Plugin IPC scopes | `capabilities/default.json` object form | Constrain plugin handlers if invoked; do **not** alone gate `OpenerExt` from custom commands |
| Frontend | `openExternal.ts` invoke only | No JS opener package |

Honest residual: path allow `"**"` is broad by design; Layer 1 still blocks empty/`..`/device forms.

## F-03 StatusBadge stale wiring (SU9)

- `HomeScreen.tsx` `FreshnessSummaryView`: when `stale_count > 0` or `worst_state === "stale"`, renders `<StatusBadge kind="stale" …>` (icon + text).
- Offline/denied/unavailable/error remain in `StatePanel`.

## F-02 Windows / keyboard smoke (max feasible)

### Host / Isolation compile proof

| Check | Status | Evidence |
|-------|--------|----------|
| Isolation pattern in conf | **Automated** | `tauri.conf.json` L27 `"use": "isolation"`, dir `../isolation` |
| Isolation classic single-file | **Present** | `apps/desktop/isolation/index.html` (503 B; inline hook pass-through) |
| Isolation + plugins compile | **PASS** | `cargo check -p ai-brains-desktop`; `cargo test` csp/isolation tests |
| WebView2 detect unit path | **PASS** | `webview2::tests::*` on Windows |

### Keyboard / dialog structural walkthrough (code + unit; live GUI deferred)

| Behavior | File:line proof | Status |
|----------|-----------------|--------|
| Native dialog `showModal()` | `ConfirmDialog.tsx:66` `el.showModal()` | **Code verified** |
| Escape cancel via `onCancel` | `ConfirmDialog.tsx:114–118` dialog `onCancel` → prop `onCancel` | **Code verified** |
| Restore focus on close | `ConfirmDialog.tsx:81–96` `close` listener → `previousFocusRef.focus()` | **Code verified** |
| Typed WIPE Enter focuses Confirm (not submit) | `ConfirmDialog.tsx:146–151` Enter → `preventDefault` + `confirmBtnRef.focus()` | **Code verified** |
| Confirm disabled until phrase match | `ConfirmDialog.tsx:48–51`, `175` `disabled={confirmDisabled}` | **Code verified** |
| Erasure execute uses typed `WIPE` | `ErasureScreen.tsx:314` `typedConfirmPhrase={dryRun ? undefined : "WIPE"}` | **Code verified** |
| Review resolve uses ConfirmDialog (keyboard path) | `ReviewScreen.tsx:176–196` `<ConfirmDialog …>` | **Code verified** |
| aria-live phrase gate | `ConfirmDialog.tsx:155` `aria-live="polite"` | **Code verified** |

**Framing:** host/unit + structural a11y verified. **Live WebView2 E2E** (start → ping → briefing → review list under Isolation; full keyboard-only path in GUI) remains **T174 / human** residual (U15 partial). Attempting `npm run tauri` / full GUI smoke was not run to avoid hanging the agent session without an interactive desktop operator.

## Manual checklist

| Item | Status | Notes |
|------|--------|-------|
| Prod CSP non-null + ipc + no unsafe-inline/eval + no HMR hosts | **Automated** | `csp_tests` |
| frame-src for Isolation | **Automated** | prod CSP string |
| Isolation pattern in conf + `isolation/index.html` classic single-file | **Landed** | hook pass-through only |
| Dual-layer opener (validators + capability-mirror + scoped caps) | **Landed** | effective on custom path; sync test |
| No JS `@tauri-apps/plugin-opener` | **Automated** | package.json test + license:check tree |
| Typed WIPE on execute; dry-run skips typed phrase | **Code complete** | ConfirmDialog + ErasureScreen |
| Native `<dialog showModal>` Escape / restore-focus / Enter→Confirm | **Code complete** | ConfirmDialog.tsx (table above) |
| :focus-visible + scroll-padding / sticky topbar | **Code complete** | App.css |
| StatusBadge icon+text (incl. **stale**) | **Code complete** | StatePanel + ScopeIndicator + HomeScreen freshness |
| Inert preview (no dangerouslySetInnerHTML) | **Verified** | repo grep empty |
| Single-instance plugin first | **Code complete** | lib.rs Builder order |
| Live `tauri dev` ping + briefing + review list | **Deferred to T174 / human** | requires interactive WebView2 session + daemon token |

## Isolation residual (C13 honesty)

- Isolation hook returns payload unmodified.
- Does **not** claim denylist enforcement (API returns modified payload, not a deny result).
- Residual risk: FE supply-chain defense is AES-GCM sandbox + hygiene hook only; true deny lists would need a different mechanism if Tauri ever exposes one.

## Path capability residual

- `opener:allow-open-path` uses object form with `"path": "**"` so arbitrary vault locators can reveal after Rust Layer-1 validation.
- Layer 2 capability-mirror with `**` intentionally accepts any non-empty path (residual breadth).
- Layer 1 still refuses empty / `..` / device traversal forms.
- Documented breadth residual for reviewers (prefer not bare string unscoped).

## T174 handoff — security cases

1. CSP fixture: prod string must not include localhost:1420 / unsafe-inline / unsafe-eval; must include frame-src.
2. Open refuse: `open_url("http://…")`, `file:`, `javascript:` → structured error; no OS open. Also capability-mirror refuse independent of Layer 1.
3. Capability not default-urls: parse capabilities JSON in CI; sync with Rust `CAPABILITY_*_ALLOWS`.
4. Wipe typed confirm: Confirm disabled until `WIPE`; Enter focuses Confirm without submit; Escape closes.
5. Keyboard review path: list → resolve dialog → complete without pointer.
6. Focus-visible: tab through nav + dialog buttons shows outline.
7. Isolation app boots: main window + ping invoke under isolation pattern.
8. Source locator: https → Open URL; path → Reveal; missing → “No locator available”; never invent locator.
9. No `@tauri-apps/plugin-opener` in package-lock production tree.
10. Offline/denied/stale StatePanel/Home show StatusBadge icons (not color-only).
