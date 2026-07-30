# T173 Internal Review R1
Date: 2026-07-30  
Reviewer: Internal Reviewer (read-only)  
Branch: `feat/t173-desktop-security-ux`  
Workspace: `C:\dev\AI-Brains-wt-t173`  
Diff base: `origin/main...HEAD` + implementer smoke claims  
Verdict: **NEEDS_FIX**

## Requirement matrix (summary)

### U1–U21

| ID | Status | Evidence |
|----|--------|----------|
| **U1** Adapter only | **Met** | Invoke wrappers in `api.ts` / `openExternal.ts`; wipe honesty from API + static contract bullets; no TS grants/freshness/erasure authority |
| **U2** Isolation mandated | **Met** | `tauri.conf.json` `pattern.use=isolation`, `options.dir=../isolation`; `isolation/index.html` present; residual honesty documented (cannot deny) |
| **U3** Dual-layer safe open | **Met** | Rust validators in `open.rs` + scoped capabilities objects; https-only URL; path `..` refused |
| **U4** Inert preview | **Met** | No `dangerouslySetInnerHTML` in `apps/desktop/src`; Evidence/Source use `<pre>` / plain text |
| **U5** Prod CSP strict | **Met** | Prod CSP non-null, ipc, no unsafe-inline/eval, no HMR hosts; `frame-src` for Isolation; `csp_tests` |
| **U6** Typed wipe + dialog a11y | **Met** | Checkbox removed; `typedConfirmPhrase="WIPE"` on execute only; Enter focuses Confirm; Escape → cancel; honesty bullets |
| **U7** Keyboard / focus / non-color status | **Met** | `:focus-visible` on interactive controls; native `<dialog showModal>`; StatusBadge icon+text in StatePanel + ScopeIndicator |
| **U8** Scope indicator always visible | **Met** | `Layout.tsx` topbar always renders `ScopeIndicator` |
| **U9** No analytics | **Met** | No analytics deps in `package.json` / production tree; README privacy lock |
| **U10** Secrets Rust-only | **Met** | Bearer via `Zeroizing` in `http_client.rs`; no web storage of token |
| **U11** Provenance + license | **Met** | opener/single-instance in desktop Cargo only (tauri-apps class); `license:check` + deny claimed PASS in SMOKE |
| **U12** AppManifest + capabilities | **Met** | `allow-open-url` / `allow-reveal-path` permissions + scoped opener objects in `default.json` |
| **U13** No unwrap/expect/panic in prod host glue | **Met** | Production host paths avoid unwrap/expect/panic; only `#[cfg(test)]` modules use them |
| **U14** Capture independence | **Met** | Desktop remains optional adapter (README + architecture) |
| **U15** Windows-first verify | **Partial** | Host compiles + unit tests; live WebView Isolation smoke deferred to T174/human (documented) |
| **U16** tsc / vite build / license:check | **Met** | SMOKE evidence PASS (not re-run by reviewer) |
| **U17** #40 out | **Met** | No opportunistic dep-bump campaign; pins stay in ADR/README band |
| **U18** Playwright → T174 | **Met** | No Playwright added; handoff list in `evidence/SMOKE.md` |
| **U19** Isolation classic single-file | **Met** | `isolation/index.html` inline classic script; no `type=module` / import / npm deps |
| **U20** No JS opener package | **Met** | Absent from `package.json` / lockfile; capability test asserts absence |
| **U21** scroll-padding focus not obscured | **Met** | `scroll-padding-top: 4rem` on `.main-column` / `.content`; sticky topbar |

### SU1–SU18

| ID | Status | Evidence |
|----|--------|----------|
| SU1 | **Met** | `csp_tests` cover baseline + frame-src |
| SU2 | **Met** | Isolation enabled + residual honesty |
| SU3 | **Met** | URL validator unit table (https allow; http/file/js/… refuse) |
| SU4 | **Met** | Path validator unit table (`..` / empty refuse) |
| SU5 | **Met** | SourceScreen “No locator available”; never fabricates |
| SU6 | **Met** | Typed WIPE; dry_run checkbox separate; honesty in dialog + result |
| SU7 | **Met** | Native dialog focus trap / Escape / restore-focus |
| SU7a | **Met** | Enter on phrase input → `confirmBtnRef.focus()`; no auto-submit |
| SU7b | **Met** | `aria-live="polite"` gate message |
| SU8 | **Partial** | Code path keyboard-operable; live keyboard smoke deferred (T174) |
| SU8a | **Met** | `:focus-visible` rules in `App.css` |
| SU8b | **Met** | scroll-padding + sticky topbar |
| SU9 | **Met** | StatusBadge offline/denied/unavailable/error in StatePanel |
| SU10 | **Met** | ScopeIndicator always in chrome |
| SU11 | **Met** | No analytics production deps |
| SU12 | **Met** | SMOKE: license:check + deny PASS |
| SU13 | **Met** | Scoped opener objects; no default-urls / bare open-path |
| SU14 | **Met** | Inert previews only |
| SU15 | **Met** | README complete; OPERATIONS expanded for T173 dual-layer / Isolation / CSP / typed WIPE / a11y / no-analytics (R1-01 `verified_fixed`) |
| SU16 | **Met** | T174 handoff cases 1–10 in SMOKE |
| SU17 | **Met** | No `@tauri-apps/plugin-opener` in package.json |
| SU18 | **Met** | `capability_tests` + review of `default.json` object scopes |

## Findings

### R1-01 | medium | OPERATIONS still T172-only; SU15 security operator notes missing
- status: verified_fixed
- files:
  - `Docs/OPERATIONS.md` (## Desktop thin client (T172 + T173 security))
  - `apps/desktop/README.md` (complete — contrast)
  - `conductor/tracks/trackT173-desktop-security-ux/spec.md` (SU15)
  - `conductor/tracks/trackT173-desktop-security-ux/plan.md` (Phase H claims OPERATIONS done)
- description:
  Spec **SU15** and plan Phase H require **OPERATIONS** to document dual-layer open, Isolation mandate, CSP prod/dev split, typed WIPE, and no-analytics defaults. `Docs/OPERATIONS.md` still titles the section “Desktop thin client (**T172**)” and only lists T172 operator bullets (invoke-first, token path, offline/denied, unavailable surfaces) plus a README pointer. It does **not** restate the T173 security/ops locks operators need without opening the app README.
- required_fix:
  Expand (or retitle) the desktop section to cover at least: dual-layer opener (Rust-only; no JS opener; https-only; no `opener:default` / allow-default-urls / bare unscoped open-path); Isolation mandated + cannot-deny residual; prod CSP vs `devCsp`; typed `WIPE` for execute wipe; single-instance focus behavior; no analytics by default. Keep the README deep-dive link.
- evidence:
  OPERATIONS L578–585 vs README dual-layer / Isolation / CSP / wipe / privacy sections; SU15 text; plan Phase H checklist marked complete.
- fix_notes:
  Retitled section to T172+T173; added SU15 operator bullets (dual-layer open, Isolation hygiene residual, CSP/`devCsp`, typed WIPE, focus a11y, no analytics) + kept README deep-dive pointer.
- verification_notes (R2):
  OPERATIONS L578–597 retitled “T172 + T173 security”; Runtime (T172) + Security locks (T173 / SU15) cover dual-layer open, Isolation cannot-deny residual, CSP/`devCsp`, typed WIPE, focus a11y, no analytics, single-instance; README deep-dive link retained.

### R1-02 | low_info | Live Isolation + keyboard WebView smoke deferred
- status: open
- files:
  - `conductor/tracks/trackT173-desktop-security-ux/evidence/SMOKE.md`
  - `conductor/tracks/trackT173-desktop-security-ux/plan.md` (Phase F/B manual items)
- description:
  U15 / SU8 live Windows WebView smoke (start → ping → briefing → review list under Isolation; full keyboard review path) is intentionally deferred to human/T174. Host-side conf, isolation app, and unit gates are in place. Not a code defect; residual for release confidence.
- required_fix:
  None for T173 code. Ensure T174 (or human smoke) runs the listed cases before calling desktop release complete.
- evidence:
  SMOKE “Live tauri dev… Deferred”; plan unchecked manual Windows smoke / keyboard review.

### R1-03 | low_info | Isolation hook cannot deny IPC (accepted residual)
- status: open
- files:
  - `apps/desktop/isolation/index.html`
  - `apps/desktop/README.md`
  - `evidence/SMOKE.md`
- description:
  Hook is pass-through (`return payload`). Documented as hygiene/audit only (C13 / U2 residual). Correct honesty — do not claim denylist.
- required_fix:
  None. Keep residual language in README/OPERATIONS when R1-01 is fixed.
- evidence:
  Isolation script returns payload unmodified; residual sections in README + SMOKE.

### R1-04 | low_info | Path capability breadth `"path": "**"` (accepted residual)
- status: open
- files:
  - `apps/desktop/src-tauri/capabilities/default.json`
  - `apps/desktop/src-tauri/src/commands/open.rs`
  - README residual note
- description:
  Object-form `opener:allow-open-path` with `"path": "**"` is intentionally broad so API locators on arbitrary drives can reveal. Not bare-string unscoped permission (SU18 satisfied). Rust still refuses empty / `..` / selected device forms. Residual risk if FE is compromised: any non-`..` path can be opened via invoke.
- required_fix:
  None for this track unless product wants tighter globs later.
- evidence:
  `default.json` L29–32; open.rs validators; README path residual.

### R1-05 | low_info | ADR-0017 Isolation still listed as “T173 candidate”
- status: verified_fixed
- files:
  - `Docs/DECISIONS/ADR-0017-desktop-frontend-stack.md` (deferred table)
- description:
  ADR deferred table still says Isolation is a T173 candidate. Implementation now mandates Isolation. Historical owner row is not wrong, but status language is slightly stale relative to landed U2.
- required_fix:
  Optional follow-up: mark Isolation landed under T173 (or supersede row) when track closes. Not blocking security behavior.
- evidence:
  ADR-0017 L68 vs `tauri.conf.json` isolation pattern.
- fix_notes:
  Deferred table row now says Isolation is **mandated in T173** (implemented; classic single-file isolation app).
- verification_notes (R2):
  ADR-0017 deferred table L68: “**Mandated in T173** (implemented; classic single-file isolation app)” — matches `tauri.conf.json` pattern.use=isolation.

## Completeness / wiring notes

### Dual-layer opener (checklist 3) — **pass**
- Cargo-only: `apps/desktop/src-tauri/Cargo.toml` has `tauri-plugin-opener = "2.5"`; **not** workspace; **no** `@tauri-apps/plugin-opener` in `package.json` / lockfile.
- Builder: `lib.rs` registers single-instance **first**, then opener.
- Commands: `open_url` / `reveal_path` take `AppHandle`, use `OpenerExt`, validate first.
- Capabilities: scoped objects only — `https://*` URL; object path allow — **no** `opener:default`, **no** `allow-default-urls`, **no** bare string `opener:allow-open-path`.
- FE: `openExternal.ts` invoke-only; Source uses `classifyLocator` on API `locator` only.

### Isolation (checklist 4) — **pass**
- Classic single-file inline JS; configured in `tauri.conf.json`; residual honesty present.
- `frame-src 'self' customprotocol: asset:` on prod CSP + test assert.
- `tauri` / `tauri-build` features include `isolation`.

### Typed WIPE (checklist 5) — **pass**
- No `confirmWipe` checkbox state.
- Execute path: `typedConfirmPhrase="WIPE"`; Confirm disabled until exact match; `confirm: true` only after dialog confirm.
- Enter focuses Confirm (no auto-submit); Escape cancels via `onCancel`; `aria-live` polite messages.
- Dry-run remains separate checkbox; dry-run dialog does not require phrase.

### Focus / StatusBadge (checklists 6–7) — **pass**
- Global `:focus-visible` + interactive selectors; `outline: none` only with `:focus-visible` replacement.
- `scroll-padding-top: 4rem` on scroll containers; sticky topbar.
- StatusBadge = lucide icon + text for offline/denied/unavailable/error (and scope states).

### Production host glue (checklist 8) — **pass**
- No production `unwrap`/`expect`/`panic` in host glue reviewed (`open.rs`, `lib.rs` run path, `main.rs`, `webview2.rs`). Test modules only.

### Tests (checklist 9) — **pass**
- URL/path validator unit tables in `open.rs`.
- CSP + isolation pattern + capability shape + no JS opener package tests in `lib.rs`.

### Docs (checklist 10) — **pass**
- README matches reality thoroughly.
- OPERATIONS expanded for T173 SU15 (R1-01 `verified_fixed`).

### Single-instance (checklist 11) — **pass**
- Present; first plugin; focuses/unminimizes `main`.

### No stubs / XSS sinks (checklist 2) — **pass**
- No placeholders/stubs in security path; no `dangerouslySetInnerHTML` on API content; Connectors/Retention remain honest unavailable (pre-existing T172 design).

## Gate evidence observed (if any)

From `evidence/SMOKE.md` (implementer-reported; not re-executed in this read-only review):

| Gate | Claimed |
|------|---------|
| `npm run typecheck` | PASS |
| `npm run build` | PASS |
| `npm run license:check` | PASS |
| `cargo test -p ai-brains-desktop --lib` | PASS (38 tests) |
| `cargo clippy -p ai-brains-desktop --all-targets -- -D warnings` | PASS |
| `cargo fmt -p ai-brains-desktop -- --check` | PASS |
| `cargo deny check` | PASS |
| `cargo check -p ai-brains-desktop` | PASS (isolation + plugins) |
| Live `tauri dev` Isolation smoke | Deferred T174 / human |

## Reviewer conclusion

Security-critical implementation for T173 is **substantially complete and correctly shaped**: dual-layer opener, Isolation mandate + honest residual, typed WIPE, dialog/focus a11y, CSP+tests, no JS opener package, StatusBadge, single-instance first.

**Implementer fix pass:** R1-01 and R1-05 marked `fixed_pending_verification`. Residual low_info items R1-02 (live smoke → T174/human), R1-03 (isolation cannot-deny), R1-04 (path `**`) remain open/deferred by design. Re-review OPERATIONS + ADR-0017 wording for clearance.

**Cross-model:** SECURITY category — recommend orchestrator still run codex-style cross-model review before final track closure, focused on opener capabilities + Isolation residual honesty.

---

# T173 Internal Review R2
Date: 2026-07-30  
Reviewer: Internal Re-Reviewer (read-only + review.md update)  
Branch: `feat/t173-desktop-security-ux`  
Workspace: `C:\dev\AI-Brains-wt-t173`  
Scope: Verify R1-01 + R1-05 fixes; security regression spot-check  
Verdict: **CLEAN**

## Fix verification

| Finding | Prior status | R2 status | Result |
|---------|--------------|-----------|--------|
| **R1-01** medium OPERATIONS SU15 | `fixed_pending_verification` | **`verified_fixed`** | OPERATIONS retitled + Security locks (T173/SU15) bullets present |
| **R1-05** low ADR Isolation wording | `fixed_pending_verification` | **`verified_fixed`** | ADR-0017 deferred row: “Mandated in T173 (implemented…)” |

### R1-01 evidence (re-read)

`Docs/OPERATIONS.md` L578–597:

- Title: `## Desktop thin client (T172 + T173 security)` (was T172-only).
- README deep-dive pointer retained.
- **Runtime (T172):** invoke-first, token prereqs, offline/denied, unavailable surfaces, **single-instance**.
- **Security locks (T173 / SU15):**
  - Dual-layer open: Rust-only `open_url`/`reveal_path`; https-only; empty/`..` refuse; scoped capability objects; forbids `opener:default`, `allow-default-urls`, bare unscoped open-path, JS `@tauri-apps/plugin-opener`.
  - Isolation mandated + **cannot-deny** residual honesty (C13).
  - CSP prod vs `devCsp` (never ship devCsp as prod).
  - Typed **`WIPE`**; Enter focuses Confirm; Escape cancels.
  - Focus a11y (`:focus-visible`, scroll-padding).
  - No analytics/crash phone-home by default.

Meets required_fix from R1 and SU15. No residual gap for medium severity.

### R1-05 evidence (re-read)

`Docs/DECISIONS/ADR-0017-desktop-frontend-stack.md` deferred table L68:

| Item | Owner |
|------|--------|
| **Tauri Isolation Pattern** | **Mandated in T173** (implemented; classic single-file isolation app) |

Stale “T173 candidate” language is gone; aligns with `tauri.conf.json` `pattern.use=isolation`.

## Security regression spot-check (R2)

| Surface | Result | Notes |
|---------|--------|-------|
| `open.rs` validators + commands | **OK** | https-only URL; path empty/`..` refuse; `OpenerExt` after validate |
| `capabilities/default.json` | **OK** | Object-scoped `https://*` + path `**`; no `opener:default` / `allow-default-urls` / bare open-path string |
| `isolation/index.html` | **OK** | Classic inline script; pass-through hook; no module/import |
| `tauri.conf.json` | **OK** | Isolation mandated; prod CSP + `frame-src`; separate `devCsp` |
| `package.json` / lockfile | **OK** | No `@tauri-apps/plugin-opener` |
| `openExternal.ts` | **OK** | Invoke-only wrappers |
| `ConfirmDialog.tsx` + Erasure WIPE | **OK** | `typedConfirmPhrase="WIPE"` on execute; Enter→focus Confirm; Escape cancel; dry-run checkbox separate |
| `dangerouslySetInnerHTML` in `apps/desktop/src` | **OK** | None |
| Production `unwrap`/`expect`/`panic` in host glue | **OK** | Only in `#[cfg(test)]` modules |

No security regressions observed relative to R1 checklist passes.

## Open residuals (unchanged; accepted low_info)

| ID | Severity | Status | Note |
|----|----------|--------|------|
| R1-02 | low_info | open | Live Isolation + keyboard WebView smoke → T174/human |
| R1-03 | low_info | open | Isolation hook cannot deny IPC (documented residual) |
| R1-04 | low_info | open | Path capability `"path": "**"` object breadth (accepted) |

## New findings (R2)

**None.**

## R2 conclusion

- **R1-01** and **R1-05** are **`verified_fixed`**.
- No open critical / high / medium findings.
- Residual open items are low_info only (by design / deferred smoke).
- **Verdict: CLEAN** — clearance for T173 internal review on docs + security shape (live WebView smoke still R1-02 residual for release confidence, not a code defect).

---

# T173 Codex Cross-Model R1 Findings — Fixer Disposition
Date: 2026-07-30  
Fixer: Grok Build (Codex FAIL F-01..F-04)  
Ledger TX: `a57911f7-6fc6-4bcf-9958-57e0e3bb9a05`  
Source: `review.codex.md` Verdict FAIL  

## Disposition summary

| ID | Sev | Title | Status |
|----|-----|-------|--------|
| **F-01** | P1 | Dual-layer opener not effective on custom `OpenerExt` path | `fixed_pending_verification` |
| **F-02** | P2 | Windows/keyboard smoke incomplete | `fixed_pending_verification` (host/unit + structural; live GUI residual T174) |
| **F-03** | P3 | Stale StatusBadge not wired | `fixed_pending_verification` |
| **F-04** | P2 | cargo audit evidence missing | `fixed_pending_verification` |

### F-01 | P1 | Effective dual-layer on custom open path
- status: fixed_pending_verification
- files:
  - `apps/desktop/src-tauri/src/commands/open.rs`
  - `apps/desktop/src-tauri/capabilities/default.json` (unchanged shape; sync-tested)
  - `apps/desktop/README.md`
  - `Docs/OPERATIONS.md`
- description:
  Codex correctly noted scoped capability objects gate plugin IPC handlers, not custom commands that call `OpenerExt` directly. Custom path previously had only Layer-1 validators.
- fix:
  - Layer 2 **capability-mirror**: `url_capability_allows` / `path_capability_allows` with `CAPABILITY_URL_ALLOWS = ["https://*"]` and `CAPABILITY_PATH_ALLOWS = ["**"]`.
  - Both layers run before `OpenerExt` in `open_url` / `reveal_path`.
  - Test `default_json_url_allows_match_rust_mirror` enforces sync with `default.json`.
  - Docs updated: honest dual-layer wording (validators + capability-mirror + scoped plugin caps).
  - Residual path breadth `"**"` documented; Layer 1 still refuses empty/`..`/device forms.
- residual:
  Plugin IPC scopes alone still do not gate `OpenerExt`; effective second gate is the Rust mirror. Frontend remains invoke-only (no JS opener).

### F-02 | P2 | Windows / keyboard smoke
- status: fixed_pending_verification
- files:
  - `conductor/tracks/trackT173-desktop-security-ux/evidence/SMOKE.md`
- description:
  Live WebView Isolation + keyboard-only operator path was deferred.
- fix:
  - Structural walkthrough with file:line proof (showModal, Escape, restore-focus, WIPE Enter→Confirm, ReviewScreen ConfirmDialog).
  - Isolation conf + `isolation/index.html` presence + compile/unit evidence recorded.
  - Honest framing: host/unit verified; live GUI E2E remains T174 residual (U15 partial).

### F-03 | P3 | Stale StatusBadge
- status: fixed_pending_verification
- files:
  - `apps/desktop/src/screens/HomeScreen.tsx`
- description:
  `stale` kind existed but no production render.
- fix:
  - `FreshnessSummaryView` shows `<StatusBadge kind="stale">` when `stale_count > 0` or `worst_state` is stale (SU9).

### F-04 | P2 | cargo audit evidence
- status: fixed_pending_verification
- files:
  - `conductor/tracks/trackT173-desktop-security-ux/evidence/SMOKE.md`
- description:
  SU12 requires audit evidence; prior SMOKE lacked it.
- fix:
  - Ran `cargo audit` (exit 0; 19 allowed warnings). Exact result recorded in SMOKE.md.

## Gate re-run (fixer)

| Gate | Result |
|------|--------|
| npm typecheck / build / license:check | PASS |
| cargo test -p ai-brains-desktop --lib | PASS (46) |
| cargo clippy -p ai-brains-desktop --all-targets -D warnings | PASS |
| cargo fmt --check | PASS |
| cargo deny check | PASS |
| cargo audit | PASS (exit 0; 19 allowed warnings) |

## Open residuals after fixer

| Item | Severity | Note |
|------|----------|------|
| Live WebView2 Isolation + keyboard E2E | low_info / U15 partial | T174 / human (documented) |
| Isolation cannot-deny | low_info | accepted residual |
| Path capability `"**"` breadth | low_info | accepted residual; dual-layer Layer1 still tight |

---

## Internal Review R3
Date: 2026-07-30  
Reviewer: Internal Re-Reviewer (read-only + review.md update)  
Branch: `feat/t173-desktop-security-ux`  
Workspace: `C:\dev\AI-Brains-wt-t173`  
Scope: Verify Codex FAIL fixes F-01..F-04 (post-fixer; claim commit `a627227`)  
Source: `review.codex.md` FAIL + fixer disposition block above  
Verdict: **CLEAN**

## Codex finding dispositions (R3)

| ID | Sev | Title | Prior (fixer) | R3 status | Result |
|----|-----|-------|---------------|-----------|--------|
| **F-01** | P1 | Dual-layer opener not effective on custom `OpenerExt` path | `fixed_pending_verification` | **`verified_fixed`** | Layer1 validators + Layer2 capability-mirror before `OpenerExt`; sync test vs `default.json` |
| **F-02** | P2 | Windows/keyboard smoke incomplete | `fixed_pending_verification` | **`verified_fixed`** | Structural + host/unit evidence improved; live GUI residual T174 honest |
| **F-03** | P3 | Stale StatusBadge not wired | `fixed_pending_verification` | **`verified_fixed`** | Home `FreshnessSummaryView` renders `kind="stale"` |
| **F-04** | P2 | cargo audit evidence missing | `fixed_pending_verification` | **`verified_fixed`** | SMOKE records audit exit 0 + 19 allowed warnings |

### F-01 — verified_fixed

**Codex claim:** Scoped capability objects gate plugin IPC only; custom `open_url` / `reveal_path` called `OpenerExt` after validators alone — dual-layer incomplete.

**R3 evidence (code re-read):**

1. **Layer 1** — `validate_https_url` / `validate_reveal_path` still run first in commands  
   (`apps/desktop/src-tauri/src/commands/open.rs` L256–257, L272–273).
2. **Layer 2** — independent `url_capability_allows` / `path_capability_allows` run **before** `app.opener().open_*`  
   (same file L257–261, L273–277). Constants `CAPABILITY_URL_ALLOWS = ["https://*"]`,  
   `CAPABILITY_PATH_ALLOWS = ["**"]` (L26–33).
3. **Sync test** — `default_json_url_allows_match_rust_mirror` parses  
   `capabilities/default.json` via `include_str!` and asserts allow arrays match mirror constants  
   (L403–464). `default.json` L26–32 is object-scoped `https://*` + path `**` (not  
   `opener:default` / `allow-default-urls` / bare string open-path).
4. **Honesty** — module docs + README state plugin IPC scopes alone do **not** gate `OpenerExt`;  
   mirror is the effective second gate on the custom path.

**Residual (unchanged low_info, not a re-open):** path mirror with `"**"` accepts any non-empty  
path after Layer 1 (R1-04 / path capability residual). Layer 1 still refuses empty / `..` /  
device forms. Acceptable for vault locators on arbitrary drives.

### F-02 — verified_fixed (residual T174 remains low_info)

**Codex claim:** Required Windows Isolation + keyboard live smoke deferred / incomplete.

**R3 evidence:**

- `evidence/SMOKE.md` F-02 section now has file:line structural walkthrough  
  (`ConfirmDialog` showModal / Escape / restore-focus / Enter→Confirm / WIPE; Erasure typed WIPE;  
  Review ConfirmDialog) plus Isolation conf + `isolation/index.html` + compile/unit table.
- Explicit framing: host/unit + structural a11y verified; **live WebView2 E2E deferred to  
  T174/human** (U15 partial) — honest residual, same class as R1-02.

Not re-opened: residual is documented and severity stays low_info / U15 partial, not a code defect.

### F-03 — verified_fixed

**Codex claim:** `StatusBadge` had `stale` kind but production UI never rendered it; Home used plain text.

**R3 evidence:**

- `HomeScreen.tsx` `FreshnessSummaryView` (L294–325): when `stale_count > 0` or  
  `worst_state === "stale"`, renders `<StatusBadge kind="stale" label=…>` (Clock icon + text).
- `StatusBadge.tsx` still maps `stale → Clock` + `badge-warn` (icon + text, not color-only).
- StatePanel still covers offline/denied/unavailable/error.

### F-04 — verified_fixed

**Codex claim:** SU12 requires license:check + deny + audit; SMOKE lacked `cargo audit`.

**R3 evidence:**

- `evidence/SMOKE.md` automated gates table includes `cargo audit` **PASS** (exit 0;  
  0 vulnerabilities; 19 allowed warnings) with detail subsection for F-04 / SU12.
- Implementer gate re-run block records same. Reviewer did not re-execute audit in this  
  read-only pass; evidence artifact is present and specific enough for SU12 closure.

## Regression scan (R3)

| Surface | Result | Notes |
|---------|--------|-------|
| No JS `@tauri-apps/plugin-opener` | **OK** | Absent from `package.json` deps/devDeps; **no** matches in `package-lock.json`; `openExternal.ts` invoke-only |
| Typed WIPE | **OK** | `ErasureScreen.tsx` L314 `typedConfirmPhrase={dryRun ? undefined : "WIPE"}`; Confirm disabled until exact match; Enter focuses Confirm (no auto-submit); Escape → cancel |
| Isolation mandated | **OK** | `tauri.conf.json` L27 `"use": "isolation"`, dir `../isolation`; Cargo `features = ["isolation"]`; classic single-file `isolation/index.html` pass-through hook; residual cannot-deny documented |
| Plugin order | **OK** | `lib.rs` single-instance first, then opener |
| Capabilities shape | **OK** | Object-scoped only; capability_tests still forbid defaults / bare open-path |
| Inert preview | **OK** | No `dangerouslySetInnerHTML` in `apps/desktop/src` |
| Analytics | **OK** | No analytics SDK deps |

No security regressions relative to R1/R2 checklist passes.

## Open residuals after R3 (low_info only)

| ID / item | Severity | Status | Note |
|-----------|----------|--------|------|
| R1-02 / F-02 live GUI | low_info / U15 partial | open | Live Isolation + keyboard WebView E2E → T174/human |
| R1-03 | low_info | open | Isolation hook cannot deny IPC |
| R1-04 / path `"**"` | low_info | open | Accepted residual; Layer 1 still tight |

## New findings (R3)

**None.**

## R3 conclusion

- Codex **F-01**, **F-02**, **F-03**, **F-04** are **`verified_fixed`**.
- No open critical / high / medium findings.
- Residual open items remain low_info only (live GUI smoke, Isolation cannot-deny, path `**` breadth).
- **Verdict: CLEAN** — T173 internal re-review after Codex FAIL fixes clears for track security shape; live WebView smoke stays T174 residual, not a re-open.
