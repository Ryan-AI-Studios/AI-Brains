# T173 Smoke / Gate Evidence

Date: 2026-07-30  
Worktree: `C:\dev\AI-Brains-wt-t173`  
Branch: `feat/t173-desktop-security-ux`  
Ledger TX: `a57911f7-6fc6-4bcf-9958-57e0e3bb9a05`

## Automated gates

| Command | Result |
|---------|--------|
| `npm run typecheck` (apps/desktop) | **PASS** |
| `npm run build` (apps/desktop) | **PASS** (vite production build) |
| `npm run license:check` (apps/desktop) | **PASS** — no GPL/AGPL; no `@tauri-apps/plugin-opener` |
| `cargo test -p ai-brains-desktop --lib` | **PASS** — 38 tests |
| `cargo clippy -p ai-brains-desktop --all-targets -- -D warnings` | **PASS** |
| `cargo fmt -p ai-brains-desktop -- --check` | **PASS** |
| `cargo deny check` | **PASS** — advisories/bans/licenses/sources ok |
| `cargo check -p ai-brains-desktop` | **PASS** — isolation feature + plugins compile |

## Unit coverage added/extended

- `csp_tests`: prod CSP frame-src for Isolation iframe; isolation pattern mandated in conf
- `capability_tests`: no `opener:default` / `allow-default-urls` / bare `opener:allow-open-path`; https-only url scopes; no JS opener package in package.json
- `commands::open::tests`: https-only URL validators; path `..` / empty refusal

## Manual checklist (host-level proven; full UI smoke optional until T174)

| Item | Status | Notes |
|------|--------|-------|
| Prod CSP non-null + ipc + no unsafe-inline/eval + no HMR hosts | **Automated** | `csp_tests` |
| frame-src for Isolation | **Automated** | prod CSP string |
| Isolation pattern in conf + `isolation/index.html` classic single-file | **Landed** | hook pass-through only |
| Dual-layer opener (Rust validators + scoped capabilities) | **Landed** | unit tests + capability parse tests |
| No JS `@tauri-apps/plugin-opener` | **Automated** | package.json test + license:check tree |
| Typed WIPE on execute; dry-run skips typed phrase | **Code complete** | ConfirmDialog + ErasureScreen |
| Native `<dialog showModal>` Escape / restore-focus / Enter→Confirm | **Code complete** | ConfirmDialog.tsx |
| :focus-visible + scroll-padding / sticky topbar | **Code complete** | App.css |
| StatusBadge icon+text | **Code complete** | StatePanel + ScopeIndicator |
| Inert preview (no dangerouslySetInnerHTML) | **Verified** | repo grep empty |
| Single-instance plugin first | **Code complete** | lib.rs Builder order |
| Live `tauri dev` ping + briefing + review list | **Deferred to T174 / human** | requires WebView2 session + daemon token |

## Isolation residual (C13 honesty)

- Isolation hook returns payload unmodified.
- Does **not** claim denylist enforcement (API returns modified payload, not a deny result).
- Residual risk: FE supply-chain defense is AES-GCM sandbox + hygiene hook only; true deny lists would need a different mechanism if Tauri ever exposes one.

## Path capability residual

- `opener:allow-open-path` uses object form with `"path": "**"` so arbitrary vault locators can reveal after Rust validation.
- Rust still refuses empty / `..` / device traversal forms.
- Documented breadth residual for reviewers (prefer not bare string unscoped).

## T174 handoff — security cases

1. CSP fixture: prod string must not include localhost:1420 / unsafe-inline / unsafe-eval; must include frame-src.
2. Open refuse: `open_url("http://…")`, `file:`, `javascript:` → structured error; no OS open.
3. Capability not default-urls: parse capabilities JSON in CI.
4. Wipe typed confirm: Confirm disabled until `WIPE`; Enter focuses Confirm without submit; Escape closes.
5. Keyboard review path: list → resolve dialog → complete without pointer.
6. Focus-visible: tab through nav + dialog buttons shows outline.
7. Isolation app boots: main window + ping invoke under isolation pattern.
8. Source locator: https → Open URL; path → Reveal; missing → “No locator available”; never invent locator.
9. No `@tauri-apps/plugin-opener` in package-lock production tree.
10. Offline/denied StatePanel show StatusBadge icons (not color-only).
