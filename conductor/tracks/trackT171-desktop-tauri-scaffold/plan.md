# T171 Plan — Desktop Tauri Scaffold (P10.0)

Status: **In Progress / Implemented in worktree** (awaiting orchestrator Completed).

Authority: `spec.md` locks **S1–S24**. Adapter-only; Windows-first; workspace membership **after** smoke.

## Phase 0 — Preconditions

- [x] T161/T158–T160 Complete; confirm routes: `/health`, `/v1/health` (no `/v1/ping`).
- [x] Toolchain: Rust 1.95+, Node **≥22**, npm **≥10**, WebView2 (usually preinstalled).
- [x] Package manager locked: **npm** + `package-lock.json`.
- [x] Pin tauri / tauri-build / @tauri-apps/* / vite 8 / typescript 7 / react 19 at implement day.
- [x] Confirm ADR-0017 number free (0016 last).
- [x] #40 out of scope unless forced.
- [ ] `ledgerful doctor` before code edits. *(ledger TX already started in primary: 3e7d2571-bd14-45c8-8f15-c83bd4d0151e)*

## Phase A — ADR + docs

- [x] Write **ADR-0017** — Vite 8 + TS 7 + React 19 + npm; Accept on ship.
- [x] README outline: WebView2 preinstall vs Bootstrapper; user-session `http.token`; not SYSTEM token; capture independence.
- [x] Note Isolation Pattern + single-instance as **T173/T172** candidates.

## Phase B — Scaffold off-workspace

- [x] Manual scaffold under `apps/desktop` (create-tauri-app not used interactively).
- [x] **Do not** add to workspace members until smoke (then added).
- [x] `package.json`: `engines.node: ">=22"`; private name.
- [x] **`.gitignore`:** `node_modules/`, `dist/`, `src-tauri/target/`, `src-tauri/gen/`.
- [x] **CSP non-null** per **S7** (include `connect-src ipc: http://ipc.localhost`).
- [x] **Strip capabilities:** remove unused menu/tray/resources; keep window/event as needed.
- [x] **`build.rs`:** `AppManifest::commands(&["ping", "get_daemon_connection_info"])` allowlist.
- [x] No analytics SDKs; no remote CDN scripts in `index.html`.

## Phase C — Rust invoke smoke

- [x] **Required:** `ping` → static structured JSON (no HTTP).
- [x] **S21:** WebView2 missing → clear dialog + clean exit (Windows).
- [x] React UI: invoke ping; show loading/error/ok.
- [x] No unwrap/expect in production glue (macro-only allow on `generate_context!`).
- [x] Unit test `ping__returns_ok_shape`.
- [ ] **Optional soft:** Rust reqwest `GET http://127.0.0.1:<port>/health` (unauth) — skipped.
- [x] **Optional soft:** `get_daemon_connection_info` (**S22**) — loopback base + token-present flag; no bearer.
- [x] Hand-written TS types for ping / connection info.

## Phase D — License + supply-chain gates

- [x] `cargo deny check` green (added `Apache-2.0 WITH LLVM-exception` for Tauri GTK transitive).
- [ ] `cargo audit` (run at PR if needed).
- [x] **S6 provenance:** tauri / tauri-build / @tauri-apps/* from tauri-apps.
- [x] npm: **`license:check`** via license-checker-rseidelsohn + GPL/AGPL fail.
- [x] Commit `package-lock.json` + root Cargo.lock.
- [x] Document command in README.

## Phase E — Frontend gate + Windows smoke + workspace

- [x] `npx tsc --noEmit` / `npm run typecheck`
- [x] `npx vite build` / `npm run build`
- [x] `cargo check` / `cargo test` for `ai-brains-desktop`
- [ ] Manual: `npm run tauri dev` window open (operator optional; host checks pass)
- [x] Record evidence (versions, no secrets).
- [x] Add `apps/desktop/src-tauri` to workspace members.
- [x] `cargo check -p ai-brains-desktop`; capture crates independent (desktop not a dep).

## Phase F — Closeout

- [x] SC1–SC9 checklist complete (see evidence/SMOKE.md).
- [x] ADR-0017 Accepted.
- [x] Review log skeleton created.
- [ ] Conductor → Completed (orchestrator finalizes).
- [x] Pin: React stack; invoke-first; CSP+ipc; npm lock; tauri-apps provenance.

## Out of scope

- [x] T172–T174 product UI/tests
- [x] Isolation Pattern enablement (document only)
- [x] single-instance plugin
- [x] `/v1/ping` route addition
- [x] Weakening T161 CORS
- [x] SYSTEM token path

## Definition of Done

Mirror spec §10.
