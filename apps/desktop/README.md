# AI-Brains Desktop (`apps/desktop`)

Windows-first **Tauri v2** thin client for the AI-Brains control plane. The UI is **adapter-only**: presentation + `invoke` wrappers. Domain policy, grants, erasure, and vault authority live in Rust crates / the daemon — not in TypeScript.

See [ADR-0017](../../Docs/DECISIONS/ADR-0017-desktop-frontend-stack.md) (stack freeze) and [ADR-0012](../../Docs/DECISIONS/ADR-0012-local-first-control-plane-and-public-protocol.md) (local-first control plane).

## Stack (frozen — ADR-0017 + T172 pins)

| Piece | Pin |
|-------|-----|
| Vite | 8.1.5 |
| TypeScript | 7.0.2 |
| React / react-dom | 19.2.8 |
| react-router | 8.3.0 (**HashRouter** from `react-router`, not `react-router-dom`) |
| @tanstack/react-query | 5.101.4 (`retry: false` default — M23) |
| lucide-react | 0.468.0 |
| @tauri-apps/api | 2.11.1 |
| @tauri-apps/cli | 2.11.4 |
| tauri (Rust) | 2.11.5 |
| tauri-build | 2.6.3 |
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

## Architecture: invoke-first + CSP

```text
[React UI] --invoke--> [src-tauri commands]
                         |-- ping / get_daemon_connection_info / probe_health
                         |-- project_briefing / personal_briefing
                         |-- query_knowledge / inspect_evidence / inspect_source
                         |-- list_review_items / resolve_review_item
                         |-- resolve_scope
                         |-- request_erasure / wipe_content_envelope
                         |-- (reqwest → loopback T161 /v1/* with Bearer)
```

- **Primary transport = Tauri `invoke`.** Never use webview `fetch` to T161 by default (M2).
- T161 CORS is deny-by-absence; do **not** weaken CORS to enable browser `fetch`.
- **Production CSP** (`app.security.csp`): T171 S7-strict — `style-src 'self'`, no `unsafe-inline`, no Vite HMR hosts (M24).
- **Dev CSP** (`app.security.devCsp`): allows Vite `http://localhost:1420` / `ws://localhost:1420` and `style-src 'unsafe-inline'` for HMR only. Production string is unchanged.
- **Capabilities** allowlist every command via `AppManifest` + `capabilities/default.json`.

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
| `#/evidence`, `#/evidence/:id` | Evidence inspect | Bounded preview |
| `#/source`, `#/source/:id` | Source inspect | By id |
| `#/claim/:kind/:id` | Claim detail | Read-only handles (no xyflow) |
| `#/erasure` | Erasure | Ticket ≠ wipe honesty (M13); retention plan **unavailable** |
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
cargo test -p ai-brains-desktop
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

## Deferred / optional

| Item | Status |
|------|--------|
| tauri-plugin-single-instance | Soft optional — **skipped** (T173 candidate) |
| probe_health command | **Landed** (GET `/health`) |
| xyflow claim graph | Skipped |
| specta / ts-rs full gen | Hand-synced DTOs instead |
| Propose conclusion/decision forms | Skipped |
| Playwright E2E | T174 |

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
