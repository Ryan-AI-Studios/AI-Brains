# AI-Brains Desktop (`apps/desktop`)

Windows-first **Tauri v2** thin client for the AI-Brains control plane. The UI is **adapter-only**: presentation + `invoke` wrappers. Domain policy, grants, erasure, and vault authority live in Rust crates / the daemon — not in TypeScript.

See [ADR-0017](../../Docs/DECISIONS/ADR-0017-desktop-frontend-stack.md) (stack freeze) and [ADR-0012](../../Docs/DECISIONS/ADR-0012-local-first-control-plane-and-public-protocol.md) (local-first control plane).

## Stack (frozen — ADR-0017 + T172/T173 pins)

| Piece | Pin |
|-------|-----|
| Vite | 8.1.5 |
| TypeScript | 7.0.2 |
| React / react-dom | 19.2.8 |
| react-router | 8.3.0 (**HashRouter** from `react-router`, not `react-router-dom`) |
| @tanstack/react-query | 5.101.4 (`retry: false` default — M23) |
| lucide-react | 0.468.0 (**do not bump** for StatusBadge) |
| @tauri-apps/api | 2.11.1 |
| @tauri-apps/cli | 2.11.4 |
| tauri (Rust) | 2.11.5 (`isolation` feature) |
| tauri-build | 2.6.3 |
| tauri-plugin-opener | 2.5.x (**Cargo only** — no JS package) |
| tauri-plugin-single-instance | 2.4.x (desktop soft) |
| Package manager | **npm** + committed `package-lock.json` |
| Node | `>=22` |

## Prerequisites

1. **Node.js ≥ 22** and **npm ≥ 10**
2. **Rust** via `rustup` (workspace `rust-toolchain.toml`)
3. **WebView2 Evergreen Runtime** (Windows 10 1803+ / Windows 11 usually preinstalled)
4. **For live data screens:** daemon listening on loopback (`AI_BRAINS_HTTP_PORT`, default **7432**) and user-session token at `%USERPROFILE%\.ai-brains\http.token`

## Capture independence

The desktop app is **not** required for capture or CLI operation. The capture path (CLI → daemon → event log) must keep working without this UI.

## Credentials (user session only)

| Path | Role |
|------|------|
| `%USERPROFILE%\.ai-brains\http.token` | **User-session** bearer for interactive loopback HTTP (owner-only ACL) |
| SYSTEM profile token | **Not** for this app — never wire desktop to SYSTEM token |

- Token is read **only in Rust**. Full bearer is **never** returned to the webview and never logged.
- Missing/empty token → structured `denied` error (UI paints promptly).

## Architecture: invoke-first + CSP + Isolation

```text
[React UI] --invoke--> [Isolation iframe hook] --AES-GCM--> [src-tauri commands]
                         |-- ping / get_daemon_connection_info / probe_health
                         |-- project_briefing / personal_briefing
                         |-- query_knowledge / inspect_evidence / inspect_source
                         |-- list_review_items / resolve_review_item
                         |-- resolve_scope
                         |-- request_erasure / wipe_content_envelope
                         |-- open_url / reveal_path  (Rust validators + opener plugin)
                         |-- (reqwest → loopback T161 /v1/* with Bearer)
```

- **Primary transport = Tauri `invoke`.** Never use webview `fetch` to T161 by default (M2).
- T161 CORS is deny-by-absence; do **not** weaken CORS to enable browser `fetch`.
- **Isolation Pattern (U2/U19) is mandated** for initial desktop release (`app.security.pattern.use = "isolation"`, dir `../isolation`). Classic single-file inline script only (no ES modules). The isolation hook is **hygiene/audit only** — it returns the payload unmodified and does **not** claim denylist enforcement (C13 residual).
- **Capabilities** allowlist every command via `AppManifest` + `capabilities/default.json`.

### CSP: production vs dev (M24 / U5)

| Build | Field | Policy |
|-------|-------|--------|
| **Production** | `app.security.csp` | Strict S7 + Isolation `frame-src 'self' customprotocol: asset:`. No `unsafe-inline`, no `unsafe-eval`, no Vite HMR hosts. |
| **Dev only** | `app.security.devCsp` | Relaxes for Vite `http://localhost:1420` / `ws://localhost:1420` and `style-src`/`script-src` `'unsafe-inline'`. **Never ship as prod.** |

Regression tests live in `src-tauri/src/lib.rs` (`csp_tests`).

### Dual-layer safe open (U3 / U20)

| Layer | Rule |
|-------|------|
| **Rust commands** | `open_url` / `reveal_path` only — validators: **https-only** URLs; paths refuse `..` / empty |
| **Capabilities** | Scoped objects only: `opener:allow-open-url` with `https://*`; `opener:allow-open-path` with object path globs |
| **Forbidden** | `opener:default`, `opener:allow-default-urls`, bare string `opener:allow-open-path`, **JS** `@tauri-apps/plugin-opener` npm package |
| **Frontend** | `lib/openExternal.ts` invoke wrappers only; Source screen uses API `locator` when present (never fabricated) |

Path capability residual: object form with `"path": "**"` is broad by design so vault locators on arbitrary drives can reveal; Rust still re-validates every path.

### Destructive confirm (U6)

- **Review resolve:** native `<dialog showModal()>` confirm (no typed phrase).
- **Wipe execute:** type exact phrase **`WIPE`** inside ConfirmDialog; Confirm disabled until match; Enter focuses Confirm (does **not** auto-submit); Escape cancels. Dry-run does **not** require typed WIPE.
- Honesty warnings come from the API on success; execute dialog also shows static contract honesty bullets.

### Accessibility (U7 / U21)

- `:focus-visible` outline on interactive controls (2px accent + offset).
- `scroll-padding-top: 4rem` + sticky topbar so focus is not hidden under chrome.
- `StatusBadge` (lucide icon + text) for non-color-only status in StatePanel / ScopeIndicator.
- Skip-to-main link in Layout.

### Privacy / analytics (U9)

**No** Sentry, PostHog, crash phone-home, or other analytics by default. Opt-in would need an ADR + track.

### Error kinds (adapter → UI)

| kind | Meaning |
|------|---------|
| `offline` | Connect/refused to loopback daemon |
| `denied` | Missing token / 401 / 403 |
| `transient` | Timeout / 5xx / 429 |
| `error` | Other HTTP / parse failures |

React Query defaults: **`retry: false`** for queries and mutations so offline/denied paint promptly (M15/M23).

## Screen map (T172)

| Route (HashRouter) | Screen | Notes |
|--------------------|--------|-------|
| `#/` | Home / Briefing | Project **or** Personal selector — never silently merged (M12) |
| `#/review` | Review queue | List + resolve with confirm; API warnings shown |
| `#/scope` | Scope resolve | Authoritative/confidence honest; grants list **unavailable** |
| `#/query` | Knowledge query | Compact/expand results |
| `#/evidence`, `#/evidence/:id` | Evidence inspect | Bounded preview (inert plain text / JSON) |
| `#/source`, `#/source/:id` | Source inspect | Locator open/reveal when API provides it |
| `#/claims/:kind/:id` | Claim detail | Read-only handles (no xyflow); `#/claim/...` redirects |
| `#/erasure` | Erasure | Ticket ≠ wipe; typed **WIPE** for execute |
| `#/connectors` | Connectors | Static **unavailable** (M4) |

### Honest unavailable surfaces

- **Connectors** — no T161 connector management UI
- **Retention plan** — not wired on this track
- **Grants inventory** — not surfaced; empty UI ≠ “no grants”

## Runbook

```powershell
cd apps/desktop
npm install
npm run typecheck
npm run build
npm run license:check
npm run tauri dev
```

Rust host only:

```powershell
# from repo root
cargo test -p ai-brains-desktop --lib
cargo clippy -p ai-brains-desktop --all-targets -- -D warnings
```

Without a live daemon, screens that invoke T161 show **offline** or **denied** (missing token). That path is intentional and proven by host error-map unit tests + UI StatePanel.

## License / supply-chain

```powershell
# from repo root
cargo deny check
cargo audit

cd apps/desktop
npm run license:check
```

`license:check` fails if GPL/AGPL appears in the production dependency tree. Tauri plugins must be from **tauri-apps** only (M10/M11).

## Operations notes (desktop)

| Topic | Behavior |
|-------|----------|
| Second launch | `tauri-plugin-single-instance` focuses the existing main window |
| Open external | Only via Rust `open_url` / `reveal_path`; https-only for URLs |
| Wipe | dry_run default true; execute requires typed `WIPE` in dialog |
| Isolation residual | Hook does not deny IPC commands — audit/pass-through only |
| No analytics | Default privacy lock (U9) |

## Deferred / optional

| Item | Status |
|------|--------|
| tauri-plugin-single-instance | **Landed** (T173 soft) |
| Isolation Pattern | **Mandated** (U2/U19) |
| Dual-layer opener | **Landed** (U3/U20) — no JS opener package |
| probe_health command | **Landed** (GET `/health`) |
| react-markdown | Skipped (plain text / JSON pre; license simplicity) |
| axe-core | Optional — skipped; T174 may add |
| xyflow claim graph | Skipped |
| specta / ts-rs full gen | Hand-synced DTOs instead |
| Propose conclusion/decision forms | Skipped |
| Playwright E2E | **T174** (security case handoff in track evidence) |

## Scripts

| Script | Purpose |
|--------|---------|
| `npm run dev` | Vite dev server only |
| `npm run build` | `tsc --noEmit` + Vite production build |
| `npm run preview` | Preview Vite `dist/` |
| `npm run tauri` | Tauri CLI passthrough |
| `npm run typecheck` | `tsc --noEmit` |
| `npm run license:check` | Production license summary + GPL/AGPL fail |

## Product license

AI-Brains product terms: PolyForm Noncommercial 1.0.0 + Small-Entity Commercial Exception. Do not use the Tauri logo as the product brand.
