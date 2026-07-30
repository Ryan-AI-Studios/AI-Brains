# T173 Plan — Desktop Security & UX (P10.2)

Status: **Implementation complete — pending review** (2026-07-30)

Authority: `spec.md` locks **U1–U21**. Depends on **T172 Completed**. Isolation **mandated** for initial release (not feature-flagged off).

## Phase 0 — Preconditions

- [x] Confirm T172 Complete: 9 screens, ConfirmDialog baseline, ScopeIndicator, prod CSP + devCsp, csp_tests, retry:false.
- [x] Re-read ADR-0012 / ADR-0017; T172 M23/M24.
- [x] Pins: `tauri-plugin-opener` **2.5.x** desktop-only; single-instance **2.4.3** soft; **no** `@tauri-apps/plugin-opener` npm; lucide stay **0.468.0**.
- [x] Ledger TX already started: `a57911f7-6fc6-4bcf-9958-57e0e3bb9a05`.
- [x] #40 out unless forced.

## Phase A — CSP (U5) — extend, not rebuild

- [x] **Existing `csp_tests` already satisfy SU1 baseline** (non-null, default-src self, connect-src ipc, no unsafe-eval/inline, no localhost:1420/ws in prod).
- [x] Optional: try explicit `script-src 'self'` — **skipped** (keep default-src fallback; Isolation needs customprotocol/asset).
- [x] If Isolation needs it: add `frame-src 'self' customprotocol: asset:` + test assert.
- [x] Document prod vs devCsp in README (reaffirm M24).

## Phase B — ConfirmDialog + wipe typed confirm (U6/U7)

- [x] Prefer **native `<dialog>` + `showModal()`** for focus trap / Escape / top-layer (zero new deps). Fallback hand-roll; last resort focus-trap-react.
- [x] `aria-describedby`, restore focus on close.
- [x] **Replace** ErasureScreen `confirmWipe` **checkbox** with typed-phrase input inside ConfirmDialog.
  - Confirm disabled until exact match **`WIPE`**.
  - Send `confirm: true` only on match; **dry_run checkbox remains separate**.
  - Remove `confirmWipe` state.
- [x] Enter in phrase field → **focus Confirm button** (do **not** auto-submit).
- [x] Escape → cancel immediately.
- [x] **aria-live** polite: “Type WIPE to enable confirm” / clear when matched.
- [x] Wipe honesty: prefer API `warnings[]` (already on wipe success). If dry-run lacks warnings, show known contract honesty constants as static text in dialog.
- [x] Review resolve uses upgraded dialog (no typed phrase).
- [ ] Manual: keyboard-only review resolve. *(handoff T174 / human smoke)*

## Phase C — Focus CSS + StatusBadge (U7/U21)

- [x] Add **`:focus-visible`** outline to `.btn`, `.nav-link`, `input`, `textarea`, `select`, `a` (and dialog buttons). No `outline: none` without replacement.
- [x] **`scroll-padding-top: 4rem`** (or match sticky header) on main scroll containers — focus not obscured under sticky chrome.
- [x] `StatusBadge`: lucide icon + text for offline/denied/stale/warn (use existing lucide pin — **no bump**).
- [x] Apply to StatePanel / ScopeIndicator as needed.

## Phase D — Safe open dual-layer (U3/U20)

- [x] Add `tauri-plugin-opener = "2.5"` to **`apps/desktop/src-tauri/Cargo.toml` only** (not workspace). Verify cargo deny + tauri-apps provenance.
- [x] **Do NOT** add `@tauri-apps/plugin-opener` to package.json.
- [x] Register `tauri_plugin_opener::init()` in Builder (after single-instance if present).
- [x] Commands `open_url` / `reveal_path`: take `app: tauri::AppHandle`; use `OpenerExt` (`app.opener().open_url` / `open_path`).
- [x] Rust validators: https-only URLs; refuse http/mailto/tel/file/javascript/data; path no `..`; unit table tests.
- [x] **Capabilities — critical shape (not plugin defaults):** scoped https url object; path object with allow globs — never bare unscoped string; never default-urls.
- [x] AppManifest allowlist new commands.
- [x] Test/review: capabilities file must not contain allow-default-urls or bare unscoped open-path (SU18).
- [x] Source/Evidence: extract **real** `locator` / path fields from contract types; Open button only if present; else honest “no locator available” — **do not fabricate**. (Evidence has no locator field — open skipped.)

## Phase E — Inert preview (U4)

- [x] No `dangerouslySetInnerHTML` for API content.
- [x] Plain text default; optional `react-markdown@10.1.x` without `rehype-raw` — **skipped**.
- [x] DOMPurify only if HTML forced — **skipped**.

## Phase F — Isolation Pattern (U2/U19)

- [x] Create `apps/desktop/isolation/` static files.
- [x] **Classic script only:** no `type="module"`, no import/export, zero npm deps. Prefer single-file inline JS in `index.html`.
- [x] Hook: `window.__TAURI_ISOLATION_HOOK__ = (payload) => { ...; return payload; }` — inspect/log only.
- [x] **Honesty:** if hook cannot deny commands, document residual (hygiene only — do not claim denylist).
- [x] `tauri.conf.json`: `app.security.pattern.use = "isolation"`, `options.dir` relative to conf (`../isolation`).
- [x] Verify iframe under CSP; add frame-src if needed.
- [ ] Windows smoke: start; ping; briefing; review list. *(handoff T174 / human — host compiles with isolation)*
- [x] Residual documented if denylist impossible (not silent skip).

## Phase G — Soft

- [x] `tauri-plugin-single-instance`: **first** plugin in Builder, **before** opener; focus main window.
- [x] Optional axe-core **devDep** only — **skipped**.
- [x] Optional skip-to-main link.

## Phase H — Docs, gates, T174 handoff

- [x] README + OPERATIONS: dual-layer open, no JS opener, isolation mandate, CSP split, typed WIPE, focus/a11y, no analytics.
- [x] `tsc` / `vite build` / `license:check` / `cargo test -p ai-brains-desktop` / deny.
- [x] SU1–SU18 checklist (see evidence/SMOKE.md).
- [x] T174 security cases listed in evidence/SMOKE.md.
- [x] Residuals: Isolation cannot deny; path `**` object breadth.
- [ ] Conductor → Completed after review. *(leave In Progress for orchestrator)*

## Out of scope

- [x] Playwright (T174)
- [x] Feature-flag Isolation off for convenience
- [x] opener:default / allow-default-urls / JS opener package
- [x] New T161 routes / CORS weaken
- [x] Analytics SDKs
- [x] Unrestricted shell execute
- [x] Full WCAG certification claim
- [x] lucide major bump

## Definition of Done

Mirror spec §10. Isolation mandated (or residual); dual-layer opener; typed wipe; dialog + focus a11y; gates green.
