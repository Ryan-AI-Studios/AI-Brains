# T173 — Desktop Security & UX Requirements (P10.2)

- **Track ID:** T173-DesktopSecurityUx
- **Phase:** P10 Task 10.2
- **Status:** 🔄 **In Progress** (implementation complete — pending review; research **2026-07-30** + **AI1–AI3 review fold-in**)
- **Depends on:** **T172 Completed** (nine screens, invoke→reqwest→T161, M1–M24, ConfirmDialog baseline, ScopeIndicator, prod CSP + `devCsp`); **T171** scaffold locks (S1–S24, ADR-0017)
- **Blocks:** T174 deep desktop tests / offline beta gate (security cases + a11y fixtures)
- **Category:** SECURITY / FEATURE
- **ADRs:** [ADR-0012](../../../Docs/DECISIONS/ADR-0012-local-first-control-plane-and-public-protocol.md) (adapter only; no UI-only privileged path); [ADR-0017](../../../Docs/DECISIONS/ADR-0017-desktop-frontend-stack.md); erasure honesty via T165 contract warnings (no new crypto ADR)
- **Stop-before:** Do not ship relaxed production CSP; do not grant `opener:default` / `allow-default-urls` / bare unscoped open-path; no analytics/crash phone-home; no TS domain authority; do not expose JS `@tauri-apps/plugin-opener` to webview

## 1. Objective

Enforce **security and accessibility UX** so the Tauri desktop client cannot become a privileged or unsafe bypass of the control plane. Apply requirements **across T172 screens**: hardened CSP posture, safe external open (dual-layer), inert source/evidence preview, destructive confirmations with **API-sourced** impact, keyboard-operable review (WCAG 2.2 AA **intent**), Isolation Pattern for FE supply-chain defense, and **no analytics by default**.

Closes P10.2: security checklist green on primary screens; Isolation mandated for initial desktop release (residual only if blocked); opener dual-layer allowlist; a11y review path; privacy defaults locked.

## 2. Live baseline (re-scan 2026-07-30 post-T172 + AI2/AI3 verify)

| Area | Live state |
|------|------------|
| Screens | 9 under `apps/desktop/src/screens/` (Home, Query, Evidence, Source, Review, ClaimDetail, Scope, Erasure, Connectors) |
| Transport | Invoke-first + structured `kind`; react-query `retry: false` |
| Confirm | `ConfirmDialog`: role=dialog, aria-modal, aria-labelledby — **no** focus trap, Escape, restore-focus, typed confirm, aria-describedby |
| Erasure wipe | `confirmWipe` **checkbox** (not typed phrase) + dry_run; API honesty warnings rendered on result |
| Scope chrome | `ScopeIndicator` in Layout — text badges, **no** icons |
| StatePanel | Text + color classes only — **no** icons (not color-only yet for status) |
| Focus CSS | **No** `:focus-visible` / focus outline in `App.css` |
| CSP **prod** | S7 baseline; `csp_tests` already assert non-null, ipc, no unsafe-inline/eval, no localhost/ws |
| CSP **dev** | `devCsp` Vite HMR — never ship as prod |
| Plugins / Isolation | **None**; `apps/desktop/isolation/` **missing** |
| Opener npm | **Not** present (must stay absent — **U20**) |
| Preview | JSON `<pre>` dumps; no open buttons; no markdown |
| Analytics / token | Absent analytics; Rust bearer + Zeroizing ✓ |

**Unblocked:** T172 Complete. T174 owns Playwright/visual automation.

## 3. Research summary (online + standards, 2026-07-30 + review fold-in)

### 3.1 Threat model (desktop UI)

| Threat | Practice | T173 |
|--------|----------|------|
| XSS / malicious Markdown or source text | Inert rendering; no raw HTML; CSP | **U4**, **U5** |
| Compromised frontend deps | Isolation AES-GCM IPC sandbox | **U2**, **U19** |
| Unrestricted open (CVE class: shell open protocol bypass) | Dual-layer: **scoped capabilities** + **Rust validators** | **U3**, **U20** |
| Single-layer Rust-only open checks | Tauri-core capability scope is second gate (CVE-2025-31477 class lesson) | **U3** |
| UI invents authority | ADR-0012; API impact only | **U1**, **U6** |
| Accidental wipe | Typed confirm + danger + dialog a11y | **U6** |
| Focus obscured by sticky chrome | `scroll-padding-top` on scroll containers (WCAG 2.2 SC 2.4.11) | **U21** |
| Telemetry / secrets | No analytics; no web storage bearer | **U9**, **U10** |

### 3.2 Dependency pins (research 2026-07-30 — pin exact at implement)

| Package / crate | Research band | License | Role |
|-----------------|---------------|---------|------|
| **tauri-plugin-opener** | **2.5.x** (e.g. 2.5.3–2.5.4) | Apache-2.0 OR MIT | Desktop-only Cargo dep — **not** workspace pin |
| **@tauri-apps/plugin-opener** | — | — | **Do NOT add** to `package.json` (**U20** / C8) |
| **tauri-plugin-shell** | 2.3.x | Apache-2.0 OR MIT | **Avoid** for open |
| **tauri-plugin-single-instance** | **2.4.3** class | Apache-2.0 OR MIT | Soft; register **first** before opener |
| Isolation app | first-party static | product | Classic script; zero npm deps |
| **react-markdown** | **10.1.0** | MIT (+ ~10 MIT transitive) | Optional; license:check full tree if added |
| **dompurify** | only if HTML forced | **MPL-2.0 OR Apache-2.0** (dual — not Apache-only) | Prefer avoid |
| **focus-trap-react** | — | MIT | **Last resort** — prefer native `<dialog>` |
| **axe-core** | **4.12.1** | MPL-2.0 | Optional **devDep** only |
| lucide-react | **0.468.0** (T172 pin) | ISC | **Do not bump** for StatusBadge icons |

**Avoid:** AGPL tools; analytics SDKs; unknown-git; `opener:default` / `allow-default-urls`; bare unscoped `opener:allow-open-path`; JS opener package; `rehype-raw` with react-markdown; shipping `devCsp` as prod.

### 3.3 Tauri Isolation Pattern (**U2** + **U19**)

**Rollout decision (AI1 open question):** **Mandate Isolation for initial desktop release** — not feature-flagged off by default. Residual escape only if Windows WebView2 packaging is blocked with documented evidence (not “defer for beta convenience”).

Per Tauri v2 Isolation docs:

- Sandboxed iframe + AES-GCM per-run keys intercept all IPC.
- **Windows limitation:** external ES modules do **not** load in sandboxed iframe — use **classic** `<script src="index.js">` or **single-file inline** script. **No** `type="module"`, **no** `import`/`export`.
- **U19:** Prefer **zero-dependency single-file** `index.html` with inline JS **or** classic external `index.js` that Tauri inlines at build — minimize protocol hang risk on WebView2.
- `options.dir`: path **relative to `tauri.conf.json`** (e.g. `../isolation`) pointing at static built/source files (no Vite pipeline for isolation app).
- **frame-src:** verify iframe under current CSP; if WebView2 blocks, add explicit `frame-src 'self' customprotocol: asset:` and assert in tests.
- **Hook capability honesty (C13):** isolation hook inspects/logs payloads; **may not be able to deny** commands (API returns modified payload, not a deny result). Do **not** claim denylist enforcement if denial is impossible — residual = audit/hygiene only.

**Smoke:** app starts; `ping` + one briefing + review list; optional soft: ping latency overhead sanity (not hard &lt;10ms gate if flaky).

### 3.4 Safe open (**U3** + **U20**) — dual-layer, opener not shell

| Concern | Rule |
|---------|------|
| Plugin | **`tauri-plugin-opener`** only in **desktop** `Cargo.toml` (not workspace) |
| JS package | **Forbidden** — no `@tauri-apps/plugin-opener` in npm |
| Transport | **Only** Rust commands `open_url` / `reveal_path` (names as implemented) accepting `AppHandle`; use `tauri_plugin_opener::OpenerExt` after `plugin(init())` in Builder |
| URL policy | **`https:` only** by default; refuse `http:`, `mailto:`, `tel:`, `javascript:`, `data:`, `file:`, `vbscript:`, SMB/NFS-style, empty |
| Path policy | Only locators from **API response fields** (`SourceDto.locator` etc.); refuse `..`; honest “no locator available” if absent — **never fabricate** |
| **Capabilities (critical)** | **Do NOT** use `opener:default` or `opener:allow-default-urls` (those allow **http/mailto/tel** — violates https-only). Use **scoped objects**: |
| | `{"identifier": "opener:allow-open-url", "allow": [{"url": "https://*"}]}` |
| | path: **object** with `"allow": [{"path": "<glob>"}]` — **bare string** `opener:allow-open-path` enables **unscoped** open (**forbidden**) |
| Defense-in-depth | Capability scope (Tauri core) **+** Rust re-validation (CVE-2025-31477 class: do not rely on one layer) |
| Register order | `tauri_plugin_opener::init()` in Builder **before** commands that use it; if single-instance present, **single-instance first** |

Placeholder “shell open” is **superseded**. Shell spawn remains non-goal.

### 3.5 CSP posture (**U5**)

| Build | Policy |
|-------|--------|
| **Production `csp`** | Keep S7 baseline. **Existing `csp_tests` already satisfy SU1** — Phase A is extend-if-needed, not rebuild. |
| Explicit `script-src 'self'` | **Optional:** test carefully — may drop `customprotocol:`/`asset:` from script sources or interact with Tauri nonce append. If breaks, **keep default-src fallback** and document. |
| **`devCsp`** | Unchanged HMR allowances; never ship as prod |
| Isolation | If needed, add `frame-src` and assert |

### 3.6 Inert preview (**U4**)

| Content | Render |
|---------|--------|
| Source locator / path | Plain text + open button only if locator present |
| Evidence / claim body | Plain text **or** optional `react-markdown` (no `rehype-raw`) |
| HTML from vault | Do not render as HTML app surface |
| `dangerouslySetInnerHTML` | **Forbidden** for API content |
| DOMPurify | Only if HTML forced — license **MPL-2.0 OR Apache-2.0** |

### 3.7 Destructive confirmation (**U6**)

| Action | Requirements |
|--------|----------------|
| Review resolve | Confirm dialog; id/subject + resolution |
| Erasure ticket | Confirm; ticket ≠ wipe honesty |
| Erasure **wipe** | **Replace** `confirmWipe` checkbox with **typed phrase** `WIPE` inside ConfirmDialog; Confirm disabled until exact match; send `confirm: true` only on match; **dry_run checkbox stays separate** |
| Honesty | Prefer API `warnings[]` (already returned on wipe success); if dry-run lacks warnings, show known contract honesty constants as static text |

**Dialog implementation preference (C / AI2):**

1. **Preferred:** native HTML **`<dialog>` + `showModal()`** (WebView2/Chromium) — built-in focus trap, Escape, top-layer, aria-modal semantics; **zero new deps**.  
2. Fallback: hand-rolled trap.  
3. Last resort: `focus-trap-react` MIT.

**Typed confirm UX:**

- Enter in phrase input → **moves focus to Confirm button** (does **not** auto-submit wipe).  
- Escape → cancel immediately.  
- **aria-live** polite region: “Type WIPE to enable confirm” / clear when matched (screen-reader honesty).  
- `aria-describedby` for body; restore focus on close.

### 3.8 Accessibility (**U7** + **U21**) — WCAG 2.2 AA intent

| Requirement | T173 |
|-------------|------|
| Keyboard review | List → resolve dialog → complete without pointer |
| **`:focus-visible`** | Add to `.btn`, `.nav-link`, `input`, `textarea`, `select`, `a` — **currently zero focus styles in App.css** |
| Focus not obscured (SC 2.4.11) | **`scroll-padding-top: 4rem`** (or match sticky header height) on main scroll containers (**U21**) |
| Focus appearance (SC 2.4.13 intent) | Visible high-contrast outline (e.g. 2px accent + offset) |
| Not color-only | StatusBadge: lucide icon + text (StatePanel + ScopeIndicator) |
| Dialog | Native dialog or equivalent trap; Escape; restore focus |
| axe-core | Optional devDep — not Playwright |

Not a legal conformance claim — **intent + evidence**.

### 3.9 Privacy / analytics (**U9**)

No Sentry/PostHog/crash phone-home. Opt-in needs ADR + track. No bearer in logs/storage.

### 3.10 Single-instance (**soft**)

`tauri-plugin-single-instance` **2.4.3** class: **first** plugin in Builder (`#[cfg(desktop)]`), before opener; focus `main` on second launch.

## 4. Non-negotiable locks (U1–U21)

| ID | Lock |
|----|------|
| **U1** | **Adapter only.** No invented grants/freshness/erasure semantics. |
| **U2** | **Isolation mandated for initial release** (not feature-flagged off). Minimal isolation app **or** explicit residual with evidence. |
| **U3** | **Dual-layer safe open:** scoped opener capabilities (**no** `allow-default-urls` / bare unscoped open-path) **+** Rust validators; https-only URLs. |
| **U4** | **Inert preview.** No untrusted HTML; no `dangerouslySetInnerHTML` for API content. |
| **U5** | **Production CSP strict.** Existing tests cover SU1 baseline; never ship `devCsp`. |
| **U6** | **Wipe typed confirm** replaces checkbox; Enter focuses Confirm (no auto-submit); Escape cancels; API honesty only. |
| **U7** | Keyboard review path; dialog a11y; **:focus-visible** on interactive controls; non-color-only status. |
| **U8** | Scope indicator always visible. |
| **U9** | No analytics / crash phone-home by default. |
| **U10** | No secrets in web storage; bearer Rust-only. |
| **U11** | tauri-apps provenance; deny + npm license gate. |
| **U12** | AppManifest + capabilities for new commands; minimal scopes. |
| **U13** | No unwrap/expect/panic in production host glue. |
| **U14** | Capture independence. |
| **U15** | Windows-first verify (Isolation + opener). |
| **U16** | `tsc --noEmit`, `vite build`, `license:check`. |
| **U17** | **#40** out unless forced. |
| **U18** | Playwright deferred to **T174** (handoff case list). |
| **U19** | Isolation app = **classic script / single-file inline**, zero npm deps; no ES modules. |
| **U20** | **All opens via Rust commands only** — **no** `@tauri-apps/plugin-opener` npm package. |
| **U21** | **Focus not obscured:** scroll-padding (or equivalent) so focus is not hidden under sticky header. |

## 5. Repository layout (target delta)

```text
apps/desktop/
  isolation/                 # static; options.dir = ../isolation
    index.html               # classic script or single-file inline JS
    index.js                 # optional classic; window.__TAURI_ISOLATION_HOOK__
  src/
    components/
      ConfirmDialog.tsx      # prefer <dialog showModal>; typed confirm; aria-live
      StatusBadge.tsx        # icon + text
      InertMarkdown.tsx      # optional
    lib/
      openExternal.ts        # invoke wrappers ONLY (no JS opener plugin)
    App.css                  # :focus-visible + scroll-padding-top
  src-tauri/
    src/commands/open.rs     # AppHandle + OpenerExt + validators
    capabilities/default.json  # scoped opener objects — NOT opener:default
    Cargo.toml               # tauri-plugin-opener desktop-only
    tauri.conf.json          # isolation pattern
```

## 6. Acceptance criteria (SU1–SU18)

| ID | Behavior |
|----|----------|
| SU1 | Prod CSP tests green (existing suite); any script-src change documented |
| SU2 | Isolation enabled **or** documented residual |
| SU3 | Open URL: https works; evil schemes + **http** refused |
| SU4 | Open path: validated locators only; traversal refused |
| SU5 | No locator → honest unavailable (no crash, no fabricate) |
| SU6 | Wipe: typed `WIPE` only; checkbox removed; honesty visible |
| SU7 | Dialog: focus trap (native dialog preferred) + Escape + restore focus |
| SU7a | Enter in typed field focuses Confirm; does not auto-submit |
| SU7b | aria-live announces typed-confirm gate state |
| SU8 | Full review keyboard path without pointer |
| SU8a | `:focus-visible` styles present on interactive controls |
| SU8b | scroll-padding prevents focus under sticky header |
| SU9 | Offline/denied/stale = icon + text |
| SU10 | Scope indicator always visible |
| SU11 | No analytics in production deps |
| SU12 | license:check + deny + audit; opener tauri-apps provenance |
| SU13 | AppManifest + **scoped** opener capabilities (no allow-default-urls; no bare unscoped open-path) |
| SU14 | Inert preview policy; no raw HTML injection |
| SU15 | README/OPERATIONS: dual-layer open, isolation, CSP split, typed wipe, no analytics |
| SU16 | T174 handoff security cases listed |
| SU17 | **No** `@tauri-apps/plugin-opener` in package.json |
| SU18 | Capability fixture/test or review evidence: permissions are scoped objects, not opener defaults |

**Optional soft:** single-instance (first in chain); axe-core; skip link; DOMPurify only if HTML forced.

## 7. Testing strategy

| Layer | Expect |
|-------|--------|
| Rust unit | URL/path validators (https allow; http/file/js refuse; `..` refuse) |
| Capabilities | Assert default.json does not contain `allow-default-urls` / bare unscoped open-path |
| CSP | Existing suite; add frame-src/script-src only if changed |
| Isolation | Manual Windows: start + ping + briefing + review |
| Manual a11y | Keyboard review; focus visible; typed WIPE + Enter focuses Confirm |
| License | Full tree if react-markdown added |
| No Playwright | T174 |

## 8. Deferred.md absorption

| Deferred / residual | Disposition |
|---------------------|-------------|
| #45/#46/#47 Isolation | **U2** + **U19** — mandated for release |
| shell/fs deep | **U3** dual-layer opener |
| confirm + keyboard | **U6**/**U7**/**U21** |
| CSP harden | **U5** (tests already baseline) |
| single-instance | Soft; first plugin |
| Isolation cannot deny IPC | Document residual if true (**C13**) |
| T174 Playwright | **U18** |

## 9. Non-goals

| Out of scope | Owner |
|--------------|--------|
| Playwright / visual golden | **T174** |
| Feature-flag Isolation off for “early beta convenience” | Forbidden default |
| `opener:default` / allow-default-urls | Forbidden |
| JS opener package | Forbidden |
| Unrestricted shell execute | Forbidden |
| Analytics SDKs | Forbidden without ADR |
| Full WCAG certification | Intent only |
| Electron / CORS weaken | Forbidden |

## 10. Definition of Done

- [ ] U1–U21 satisfied (Isolation residual only if justified)  
- [ ] SU1–SU18 green  
- [ ] Dual-layer opener + no npm opener package  
- [ ] Typed wipe replaces checkbox; dialog a11y  
- [ ] :focus-visible + scroll-padding  
- [ ] Docs + T174 handoff  
- [ ] Gates green; conductor → Completed after review  

## 11. Risks

| Risk | Mitigation |
|------|------------|
| Isolation Windows friction | U19 single-file/classic script; residual U2 |
| Opener defaults too broad | **Never** allow-default-urls; scoped globs + SU18 |
| Single-layer open (CVE class) | Dual-layer U3 |
| Explicit script-src breaks nonces | Optional only; document keep default-src |
| Isolation cannot deny | Hygiene-only residual |
| focus-trap dependency | Prefer `<dialog>` |
| react-markdown transitive licenses | license:check full tree |
| Typed confirm SR silence | aria-live U6 |

## 12. Implementation priority

1. CSP: verify existing tests; optional script-src experiment  
2. ConfirmDialog → `<dialog>` + typed wipe (replace checkbox) + aria-live  
3. `:focus-visible` + scroll-padding (**U21**) + StatusBadge icons  
4. Opener dual-layer (Cargo dep, Builder, scoped caps, Rust commands, **no** npm)  
5. Source/Evidence open buttons from real locator fields only  
6. Isolation static app + config (**U19**)  
7. Single-instance soft (first)  
8. Docs + gates + T174 handoff  

## 13. Open questions (resolved defaults)

| Question | Default |
|----------|---------|
| Isolation rollout? | **Mandate for initial release**; residual only if blocked — **not** feature-flagged off for beta convenience |
| shell vs opener? | **opener** dual-layer |
| JS opener package? | **Never** |
| Opener capabilities? | Scoped `https://*` only — **not** plugin defaults |
| Dialog implementation? | **Native `<dialog showModal>`** first |
| Typed wipe phrase? | Exact `WIPE` |
| Enter in typed field? | Focus Confirm — **no** auto-submit |
| Markdown? | Plain text default; react-markdown optional + license:check |
| lucide bump? | **No** — stay on T172 pin |
