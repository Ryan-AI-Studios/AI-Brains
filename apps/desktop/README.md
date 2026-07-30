# AI-Brains Desktop (`apps/desktop`)

Windows-first **Tauri v2** thin client for the AI-Brains control plane. The UI is **adapter-only**: presentation + `invoke` wrappers. Domain policy, grants, erasure, and vault authority live in Rust crates / the daemon — not in TypeScript.

See [ADR-0017](../../Docs/DECISIONS/ADR-0017-desktop-frontend-stack.md) (stack freeze) and [ADR-0012](../../Docs/DECISIONS/ADR-0012-local-first-control-plane-and-public-protocol.md) (local-first control plane).

## Stack (frozen — ADR-0017)

| Piece | Pin |
|-------|-----|
| Vite | 8.1.5 |
| TypeScript | 7.0.2 |
| React / react-dom | 19.2.8 |
| @tauri-apps/api | 2.11.1 |
| @tauri-apps/cli | 2.11.4 |
| tauri (Rust) | 2.11.5 |
| tauri-build | 2.6.3 |
| Package manager | **npm** + committed `package-lock.json` |
| Node | `>=22` |

## Prerequisites

1. **Node.js ≥ 22** and **npm ≥ 10**
2. **Rust** via `rustup` (workspace `rust-toolchain.toml`, currently 1.95.x)
3. **WebView2 Evergreen Runtime**
   - **Preinstalled** on **Windows 10 version 1803+** and **Windows 11** for normal consumer/pro SKUs.
   - Install the [Evergreen Bootstrapper](https://developer.microsoft.com/en-us/microsoft-edge/webview2/) only on stripped/older/server SKUs where the runtime is missing.
   - On startup the host runs a **WebView2 diagnostic** (registry check). If missing, it shows a clear dialog with Bootstrapper guidance and exits cleanly (no panic).

## Capture independence

The desktop app is **not** required for capture or CLI operation. The capture path (CLI → daemon → event log) must keep working without models, embeddings, graph DBs, or this UI. Do not treat the desktop shell as a hard dependency of capture crates.

## Credentials (user session only)

| Path | Role |
|------|------|
| `%USERPROFILE%\.ai-brains\http.token` | **User-session** bearer for interactive loopback HTTP (owner-only ACL) |
| SYSTEM profile token | **Not** for this app — residual service path; never wire desktop to SYSTEM token |

Never commit tokens. Never log full bearer values. Prefer keeping secrets in Rust and only crossing invoke when necessary.

Default daemon HTTP port (when unset): **7432** (`AI_BRAINS_HTTP_PORT` overrides). Loopback base shape: `http://127.0.0.1:<port>`.

## Architecture: invoke-first + CSP

```text
[React UI] --invoke--> [src-tauri commands]
                         |-- ping (static JSON smoke)
                         |-- get_daemon_connection_info (port + token-present; no bearer in response)
```

- **Primary transport = Tauri `invoke`.** Scaffold default does **not** use webview `fetch` to the T161 loopback API.
- T161 CORS is deny-by-absence; do **not** weaken CORS to enable browser `fetch` from the webview.
- **CSP** is non-null in production (`tauri.conf.json`). Baseline includes:
  - `default-src 'self' customprotocol: asset:`
  - `connect-src ipc: http://ipc.localhost` (required so invoke is not blocked)
  - no remote script CDN, no `'unsafe-eval'`
- **Capabilities** are stripped (no unused menu/tray/resources). Commands are allowlisted via `AppManifest::commands` in `build.rs`.

### Deferred (not this app’s DoD)

| Item | Track |
|------|-------|
| Tauri **Isolation Pattern** enablement | T173 candidate |
| **single-instance** plugin | T172/T173 candidate |
| Full product screens | T172 |
| Playwright | T174 |

## Runbook

```powershell
cd apps/desktop
npm install
npm run typecheck
npm run build
npm run tauri dev
```

Smoke without opening a window (Rust host only):

```powershell
cd apps/desktop/src-tauri
cargo test
cargo check
```

Workspace membership (after smoke): package name `ai-brains-desktop`.

```powershell
# from repo root
cargo check -p ai-brains-desktop
```

Dev does not require a live vault mutation. Optional daemon connection info is honest empty-state when the daemon/token is absent.

## License / supply-chain checks

Rust workspace:

```powershell
# from repo root
cargo deny check
cargo audit
```

npm production licenses (maintained fork — **not** abandoned `license-checker`):

```powershell
cd apps/desktop
npm run license:check
# equivalent:
npx license-checker-rseidelsohn --production --summary
```

`license:check` fails if GPL/AGPL appears in the production dependency tree.

### Provenance (S6)

New `tauri*` crates must be published by the **tauri-apps** org (crates.io / npm scope). License string alone is insufficient.

| Crate / package | Expected publisher |
|-----------------|-------------------|
| `tauri`, `tauri-build` | tauri-apps (crates.io) |
| `@tauri-apps/api`, `@tauri-apps/cli` | @tauri-apps (npm) |

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
