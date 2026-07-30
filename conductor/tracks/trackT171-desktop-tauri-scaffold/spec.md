# T171 — Desktop Tauri Scaffold (P10.0)

- **Track ID:** T171-DesktopTauriScaffold
- **Phase:** P10 Task 10.0
- **Status:** 📋 **Proposed / Expanded** (planning only — implement on go-ahead; **AI1–AI3 review fold-in 2026-07-30**)
- **Depends on:** **T161 Complete** (loopback `/v1` + bearer + CORS deny-by-absence); **T158/T159/T160** governed contracts; ADR-0010 / ADR-0012
- **Blocks:** T172 screens; T173 security/UX polish; T174 desktop tests
- **Category:** FEATURE / INFRA / SECURITY
- **ADRs:** [ADR-0010](../../../Docs/DECISIONS/ADR-0010-evolve-ai-brains-into-successor.md); [ADR-0012](../../../Docs/DECISIONS/ADR-0012-local-first-control-plane-and-public-protocol.md); **ADR-0017** (frontend stack — create at implement)
- **Stop-before:** Do not add `apps/desktop/src-tauri` to workspace until scaffold **smoke-builds on Windows**; no UI-only authority; no analytics by default

## 1. Objective

Scaffold **`apps/desktop`** as a **Tauri v2** + TypeScript thin client that:

1. Runs **offline-first** on **Windows** (WebView2) as the primary target.  
2. Talks to the **same versioned contracts** as CLI / IPC / HTTP — **no business logic in TypeScript**.  
3. Uses a **minimal capability allowlist** + **working Tauri v2 CSP** (T173 deepens).  
4. Passes **cargo deny** + **maintained npm license audit** (no AGPL/GPL).  
5. Joins the Rust workspace **only after** a clean verified scaffold.

Closes P10.0: desktop shell exists, local ping works, stack frozen (Vite+TS+React), licenses green, adapter-only architecture documented.

## 2. Live baseline (re-scan 2026-07-30 + AI2/AI3 verify)

| Area | Live state |
|------|------------|
| `apps/` | **Missing** — no desktop tree, no root `package.json` |
| Workspace members | `crates/*` only |
| T161 HTTP routes | `/health` + `/v1/health` (liveness); **no `/v1/ping`**; data under `/v1/*` bearer-protected |
| CORS | No permissive CORS layer (deny-by-absence) — webview `fetch` to loopback will not get `Access-Control-Allow-Origin` |
| Token path | `%USERPROFILE%\.ai-brains\http.token` owner-only ACL (user-session) |
| SYSTEM service token | Residual — **not** for interactive desktop |
| Toolchain | `rust-toolchain.toml` **1.95.0** ≫ Tauri MSRV ~1.77.2 |
| ADR-0017 | **Does not exist yet** (create this track) |
| WebView2 | Pre-installed on **Windows 10 1803+** and **Windows 11**; Bootstrapper only for stripped/older SKUs |

**Unblocked:** T158–T161 Complete. Screens remain T172.

## 3. Research summary (online + standards, 2026-07-30)

### 3.1 Why Tauri v2 (not Electron)

| Factor | Practice | T171 |
|--------|----------|------|
| Footprint | Small vs Electron | Local-first fit |
| Security | Capabilities default-deny; CSP; Isolation Pattern recommended for large FE trees | Baseline now; Isolation → T173 |
| Host | Rust ↔ `ai-brains-*` | Invoke adapters |
| License | Apache-2.0 OR MIT | deny-compatible |
| Windows | WebView2 Evergreen | S21 diagnostics + README |

### 3.2 Dependency pins (research 2026-07-30 — pin exact at implement)

| Component | Research band | License | Notes |
|-----------|---------------|---------|--------|
| **tauri** | **2.x** (observed ~2.11.x class) | Apache-2.0 OR MIT | Pin latest 2.x that builds |
| **tauri-build** | **2.6.x** (e.g. 2.6.3) | Apache-2.0 OR MIT | Pair with tauri |
| **@tauri-apps/api** | **2.11.x** class | MIT | Match shell major |
| **@tauri-apps/cli** | **2.11.x** class | Apache-2.0 OR MIT | devDependency |
| **Vite** | **8.x** (e.g. 8.1.5) | MIT | SPA only — no SSR in Tauri |
| **TypeScript** | **7.x** (e.g. 7.0.2), strict | Apache-2.0 | Not 5.x |
| **React / react-dom** | **19.x** | MIT | Frozen stack |
| **@vitejs/plugin-react** | match Vite major | MIT | |
| Tailwind | optional 4.x | MIT | Not required for smoke |
| **license-checker-rseidelsohn** (or **license-checker-evergreen**) | current MIT fork | MIT | **Not** abandoned `license-checker` (dlibjs, 2019) |

**Avoid:** AGPL/GPL; Electron; unknown-git; default analytics; GPL icon packs; **typosquat `tauri-*` crates** not published by **tauri-apps** org (see **S6**).

### 3.3 Frontend stack freeze (**confirmed**)

| Decision | Value |
|----------|--------|
| UI | **Vite 8.x + TypeScript 7.x + React 19.x** |
| Package manager | **npm ≥10** with committed **`package-lock.json`** (not pnpm default) |
| Node | `engines.node` **`>=22`** (current LTS band) |
| ADR | **`Docs/DECISIONS/ADR-0017-desktop-frontend-stack.md`** at implement — **Accepted** when shipping scaffold |
| Tailwind / shadcn-style | Optional; MIT-only if added |

**Open question (AI1): aligned on React stack?** **Yes** — freeze stands; ADR-0017 records it.

### 3.4 Security / architecture

| Practice | T171 |
|----------|------|
| Capabilities | Strip create-tauri-app defaults; no menu/tray/resources unless needed |
| **AppManifest::commands** | `build.rs` allowlists only registered smoke commands (e.g. `ping`) — capabilities alone do **not** limit which invoke commands exist |
| **CSP (exact baseline)** | See **S7** — must include `connect-src` **ipc** or invoke is blocked |
| Isolation Pattern | **Not required for T171 smoke**; **T173 candidate** (Tauri “highly recommend” for FE-heavy apps) |
| ADR-0012 | UI untrusted presentation; Rust owns system/daemon/contract calls |
| T161 CORS | **Invoke-first**; no TS `fetch` to `/v1` as scaffold default |
| Token | User-session `http.token` only; never log full bearer |

### 3.5 IPC architecture (scaffold)

```text
[React UI] --invoke--> [src-tauri commands]
                         |-- ping (static JSON)          [required smoke]
                         |-- get_daemon_connection_info  [optional soft S22]
                         |-- reqwest -> 127.0.0.1/health [optional soft]
```

| Layer | May | Must not |
|-------|-----|----------|
| TypeScript | Presentation, invoke wrappers | Policy, freshness, authority rules |
| Tauri commands | Serde, read token path, reqwest loopback | Second domain implementation |
| HTTP | Existing T161 `/health`, `/v1/*` | New `/v1/ping` unless separate T161 track |

**T171 smoke (required):** static `ping` / local health JSON from Rust — **no HTTP required**.

**Optional soft (after smoke):**

1. Rust `reqwest` to `http://127.0.0.1:<port>/health` (unauthenticated liveness) — **not** `/v1/ping` (does not exist).  
2. Or `GET /v1/health` with bearer from user token file.  
3. **`get_daemon_connection_info`** (**S22**): returns loopback base URL + whether token file is present (or passes token only over **invoke**, never into `localStorage`/logs). Frontend must not use this to justify webview `fetch` until CORS policy is redesigned.

**Contract types in TS:** T171 smoke may use **hand-written** thin types for ping. **Optional:** `specta` / `ts-rs` for generated `bindings.ts` from Rust — evaluate for **T172** contract surface (avoid mandatory heavy gen pipeline for scaffold DoD). If added in T171, dev-only path; no AGPL tools.

## 4. Non-negotiable locks (S1–S24)

| ID | Lock |
|----|------|
| **S1** | **Adapter only (ADR-0012).** No TS authority/grants/erasure semantics. |
| **S2** | **Tauri v2 only** (not v1/Electron) without new ADR. |
| **S3** | **Stack freeze:** Vite **8.x** + TypeScript **7.x** + **React 19.x**. ADR-0017 at implement. |
| **S3a** | **Package manager:** **npm** + committed **`package-lock.json`**. `engines.node: ">=22"`. |
| **S4** | **Workspace membership after** Windows smoke (`tauri dev`/`build` green). |
| **S5** | **Zero AGPL/GPL** npm/Rust product deps. |
| **S6** | **cargo deny + audit** green; crates.io only; no unknown-git. **Additionally:** any new `tauri*` / plugin crate must be verified as published by **tauri-apps** (or documented first-party/workspace crate) — crates.io license pass alone is insufficient (2026 typosquat/malicious-crate campaigns). Diff-review first-time deps. |
| **S7** | **CSP baseline (non-null, exact intent):** production CSP **must not** be `null` (null disables CSP). Minimum working Tauri v2 baseline: `default-src 'self' customprotocol: asset:`; `connect-src ipc: http://ipc.localhost`; `img-src 'self' asset: http://asset.localhost blob: data:`. No remote script CDN. No `'unsafe-eval'`. Prefer no `'unsafe-inline'` in production (dev tooling may differ; document). T173 may tighten. |
| **S8** | **Capabilities minimal + command allowlist.** After scaffold: strip unused `core:menu` / `core:tray` / `core:resources` if unused; keep what window/event need. **`build.rs`:** `AppManifest::commands` allowlists smoke commands only. |
| **S9** | **Primary transport = invoke.** No scaffold default webview `fetch` to T161. |
| **S10** | **User-session credentials only** (`%USERPROFILE%\.ai-brains\http.token`). Not SYSTEM service token. |
| **S11** | **No analytics / crash-phone-home by default.** |
| **S12** | **Capture independence.** Desktop not required for capture/CLI. |
| **S13** | **No unwrap/expect/panic** in production `src-tauri` glue. |
| **S14** | Workspace edition **2024**; Tauri MSRV headroom OK (1.95 ≫ 1.77). |
| **S15** | **Windows-first.** Linux/macOS optional. |
| **S16** | PolyForm product license; MIT/Apache deps OK; do not use Tauri logo as product brand. |
| **S17** | E1 empty-state honesty for any stub DTOs. |
| **S18** | Lockfiles committed (`package-lock.json` + Cargo.lock for src-tauri). |
| **S19** | Dev runbook: WebView2 (usually preinstalled on Win10 1803+/Win11), Node ≥22, rustup, `npm run tauri dev`. Bootstrapper fallback only for stripped systems. |
| **S20** | **#40** opportunistic bumps out of scope unless Tauri forces documented bump. |
| **S21** | **WebView2 pre-launch diagnostic.** On Windows startup failure path: detect missing WebView2 (registry/API as appropriate) and show a **clear dialog** with Evergreen Bootstrapper guidance; exit cleanly — no opaque panic. |
| **S22** | **Daemon connection info only via invoke (optional soft).** If UI needs loopback port/token presence, Rust command reads config/token path and returns structured info over IPC. Never commit tokens; never log full bearer; prefer not to put bearer in JS memory longer than needed. Does **not** authorize webview CORS bypass design. |
| **S23** | **npm license tool:** use **`license-checker-rseidelsohn`** or **`license-checker-evergreen`** (maintained MIT forks) — **not** abandoned `license-checker` (dlibjs). Fail on GPL*/AGPL*. |
| **S24** | **`.gitignore`:** `node_modules/`, `dist/`, `src-tauri/target/`, `src-tauri/gen/` (and schemas under gen as applicable). |

## 5. Repository layout (target)

```text
apps/desktop/
  package.json                 # private; engines.node >=22
  package-lock.json            # committed
  .gitignore                   # S24
  vite.config.ts
  tsconfig.json
  index.html                   # no remote CDN scripts
  src/
    main.tsx
    App.tsx                    # shows ping result
    lib/api.ts                 # invoke wrappers only
  src-tauri/
    Cargo.toml
    build.rs                   # AppManifest::commands allowlist
    tauri.conf.json            # CSP non-null baseline S7
    capabilities/default.json  # stripped
    src/lib.rs | main.rs       # S21 WebView2 diagnostic path
    src/commands/mod.rs        # ping (+ optional get_daemon_connection_info)
  README.md
Docs/DECISIONS/ADR-0017-desktop-frontend-stack.md
```

## 6. Smoke acceptance (SC1–SC9)

| ID | Behavior |
|----|----------|
| SC1 | `apps/desktop` exists; `tsc --noEmit` succeeds |
| SC2 | `vite build` succeeds; `src-tauri` builds; Windows `tauri` smoke runnable |
| SC3 | Window opens; UI shows `invoke('ping')` structured JSON |
| SC4 | CSP **non-null** and includes **`default-src` with `'self'`** and **`connect-src` with `ipc:`** (or equivalent documented Tauri ipc allow) — not merely “key present” |
| SC5 | Capabilities stripped; no unrestricted fs/shell; AppManifest commands allowlist present |
| SC6 | `cargo deny check` + audit green; tauri-* provenance reviewed (**S6**) |
| SC7 | Maintained license-checker fork: no AGPL/GPL in production tree |
| SC8 | README: WebView2 note (preinstalled vs Bootstrapper), user-session token, run without live vault mutation |
| SC9 | `.gitignore` covers node_modules/dist/target/gen |

**Optional soft:** Rust probes **`/health`** (not `/v1/ping`); or authenticated `/v1/health` with user token.

## 7. Testing strategy (scaffold)

| Test | Expect |
|------|--------|
| Rust `ping__returns_ok_shape` | stable fields |
| Manual Windows smoke | window + ping |
| Frontend gate | `tsc --noEmit`, `vite build` (when `apps/desktop` exists) |
| License | deny + S23 npm command |
| CSP regression | config parse / fixture assert non-null + ipc connect-src |

No Playwright (T174). No live vault required.

## 8. Deferred.md absorption / related

| Deferred | Disposition |
|----------|-------------|
| **T161 SYSTEM HTTP token** | **S10** honesty — desktop = user-session only |
| **T161 Host rebinding residual** | Invoke path reduces; bearer remains |
| **#40** | Out unless Tauri forces |
| Capture independence | **S12** |
| No analytics (T173) | **S11** |
| CORS for TS fetch | Still deferred; do not weaken T161 in T171 |
| **Tauri Isolation Pattern** | **T173 candidate** (document; not T171 DoD) |
| **tauri-plugin-single-instance** | **T172/T173 candidate** (multi-window confusion) |
| **AppManifest + capability deepening** | Scaffold does minimum; T173 expands |
| **specta/ts-rs full contract gen** | Optional T171; prefer **T172** for full DTO surface |
| Soft-canonicalize | Out of scope |

## 9. Non-goals

| Out of scope | Owner |
|--------------|--------|
| Full screens | **T172** |
| Isolation Pattern enablement, deep a11y, shell open | **T173** |
| Playwright | **T174** |
| Adding `/v1/ping` to HTTP server | T161 follow-up if ever needed |
| Electron | New ADR |
| Auto-update / single-instance plugins in smoke | Later |
| Making desktop required for capture | Forbidden |

## 10. Definition of Done

- [ ] ADR-0017 Accepted (Vite+TS+React+npm)  
- [ ] Windows scaffold builds; SC1–SC9  
- [ ] Ping invoke works; CSP non-null with ipc connect-src  
- [ ] AppManifest commands + stripped capabilities  
- [ ] S6 provenance + deny + S23 npm license  
- [ ] S21 WebView2 fail path documented/implemented  
- [ ] README + .gitignore  
- [ ] Workspace membership only after smoke  
- [ ] No TS domain logic  
- [ ] Conductor → Completed after review  

## 11. Risks

| Risk | Mitigation |
|------|------------|
| CSP blocks invoke | **S7** exact connect-src ipc |
| CSP null disables protection | **SC4** value assert |
| Malicious crates.io tauri-* | **S6** org provenance |
| Abandoned license-checker | **S23** maintained fork |
| WebView2 missing (rare) | **S21** + Bootstrapper note |
| Multi-instance UX | Defer single-instance plugin |
| TS/contract drift | Hand types smoke; specta later |
| CORS confusion | **S9** invoke-first |

## 12. Review fold-in (AI1–AI3, 2026-07-30)

| Source | Item | Disposition |
|--------|------|-------------|
| AI1 | WebView2 diagnostic | **Accepted** → **S21** |
| AI1 | specta/ts-rs | **Partial** — optional; T172 preferred for full gen |
| AI1 | get_daemon_connection_info | **Accepted** optional soft → **S22** |
| AI1 | npm + package-lock | **Accepted** → **S3a** |
| AI1 | React freeze Q | **Yes** — **S3** / ADR-0017 |
| AI2 C1 | No /v1/ping | **Accepted** → probe `/health` or `/v1/health` |
| AI2 C2/C3 | TS 7 / Vite 8 | **Accepted** |
| AI2 C4 | CSP ipc connect-src | **Accepted** → **S7** (highest severity) |
| AI2 C5 / AI3 | Isolation Pattern | **Accepted** → T173 deferred table |
| AI2 C6 | AppManifest::commands | **Accepted** → **S8** |
| AI2 C7 / AI3 | license tool abandoned | **Accepted** → **S23** |
| AI2 C8 / AI3 | WebView2 preinstalled | **Accepted** → **S19** |
| AI2 C9 | single-instance | **Accepted** → deferred T172/T173 |
| AI2 C10 | static ping; optional /health | **Accepted** |
| AI2 C11 | frontend CI gate | **Accepted** plan |
| AI2 C12 | .gitignore | **Accepted** → **S24** |
| AI2 C13 | engines node | **Accepted** → **S3a** |
| AI2 C15 | strip template capabilities | **Accepted** plan Phase B |
| AI3 | SC4 csp null | **Accepted** → SC4/S7 non-null |
| AI3 | crates.io malicious / TrapDoor | **Accepted** → **S6** provenance |

## 13. Expand-ready checklist

- [x] Research Tauri v2 / licenses / CSP  
- [x] Stack freeze React+Vite+TS+npm  
- [x] AI1–AI3 fold-in  
- [ ] Implement on user go-ahead  
