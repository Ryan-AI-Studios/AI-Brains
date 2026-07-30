# Track Completion Audit - T173

## Verdict: FAIL

## Scope Reviewed

- `origin/main...HEAD` on `feat/t173-desktop-security-ux`
- Commits `529ec35..96dd4e4`
- Clean working tree; no uncommitted changes
- Complete [spec.md](C:/dev/AI-Brains-wt-t173/conductor/tracks/trackT173-desktop-security-ux/spec.md), [plan.md](C:/dev/AI-Brains-wt-t173/conductor/tracks/trackT173-desktop-security-ux/plan.md), review log, smoke evidence, implementation, tests, capabilities, and docs

## Requirement and DoD Matrix

### U1–U21

| ID | Result | Notes |
|---|---|---|
| U1 | PASS | Adapter-only frontend behavior preserved. |
| U2 | PASS | Isolation is enabled; pass-through limitation is honestly documented. |
| U3 | FAIL — F-01 | Scoped capability objects do not gate the custom Rust commands that directly call `OpenerExt`. |
| U4 | PASS | No API `dangerouslySetInnerHTML`; previews remain inert. |
| U5 | PASS | Production CSP is strict and includes Isolation `frame-src`. |
| U6 | PASS | Typed `WIPE`, separate dry-run checkbox, Escape, aria-live, and non-submit Enter behavior are implemented. |
| U7 | PARTIAL — F-02/F-03 | Code is keyboard-oriented, but required live keyboard evidence is absent; stale status is not wired. |
| U8 | PASS | Scope indicator is always rendered in Layout chrome. |
| U9 | PASS | No analytics or crash phone-home dependencies found. |
| U10 | PASS | Bearer remains Rust-side; no web storage use found. |
| U11 | PARTIAL — F-04 | License/deny evidence exists; required `cargo audit` evidence is absent. |
| U12 | FAIL — F-01 | AppManifest and static capabilities exist, but effective custom-command scope is unscoped. |
| U13 | PASS | Production host glue has no explicit unwrap/expect/panic; hits are test-only. |
| U14 | PASS | Desktop remains an optional adapter. |
| U15 | FAIL — F-02 | Windows WebView2 Isolation/opener smoke was deferred. |
| U16 | PASS | Typecheck, build, and license gate are reported passing. |
| U17 | PASS | No unrelated dependency-bump campaign. |
| U18 | PASS | Playwright correctly remains T174 scope. |
| U19 | PASS | Classic single-file Isolation app; no modules or npm dependencies. |
| U20 | PARTIAL — F-01 | No JS opener package and Rust-only wrappers are present, but the claimed dual-layer enforcement is incomplete. |
| U21 | PASS | Focus-visible styling and scroll padding are implemented. |

### SU1–SU18

| ID | Result |
|---|---|
| SU1 | PASS |
| SU2 | PASS |
| SU3 | PASS |
| SU4 | PARTIAL — F-01 |
| SU5 | PASS |
| SU6 | PASS |
| SU7 | PASS by code inspection |
| SU7a | PASS by code inspection |
| SU7b | PASS |
| SU8 | PARTIAL — F-02 |
| SU8a | PASS |
| SU8b | PASS |
| SU9 | FAIL — F-03 |
| SU10 | PASS |
| SU11 | PASS |
| SU12 | PARTIAL — F-04 |
| SU13 | FAIL — F-01 |
| SU14 | PASS |
| SU15 | PARTIAL — F-01 documentation overstates effective dual-layer enforcement |
| SU16 | PASS |
| SU17 | PASS |
| SU18 | PARTIAL — F-01; static shape passes, runtime enforcement does not |

### Definition of Done

| Item | Result |
|---|---|
| U1–U21 satisfied | FAIL |
| SU1–SU18 green | FAIL |
| Dual-layer opener and no npm opener | FAIL |
| Typed wipe and dialog accessibility | PASS by code inspection |
| Focus-visible and scroll padding | PASS |
| Documentation and T174 handoff | PARTIAL |
| Gates green | PARTIAL; `cargo audit` is not evidenced |
| Conductor marked Completed | Pending orchestrator action |

## Findings

### F-01 — P1: Scoped opener capabilities are not an effective second gate

The frontend invokes custom `open_url` and `reveal_path` commands through [openExternal.ts](C:/dev/AI-Brains-wt-t173/apps/desktop/src/lib/openExternal.ts:8). Those commands directly call `app.opener().open_url` and `app.opener().open_path` in [open.rs](C:/dev/AI-Brains-wt-t173/apps/desktop/src-tauri/src/commands/open.rs:108).

The scoped capability objects in [default.json](C:/dev/AI-Brains-wt-t173/apps/desktop/src-tauri/capabilities/default.json:18) apply to the opener plugin’s IPC handlers, whose scope checks occur before their own opener calls. They do not scope these custom host-side commands. Consequently, the actual custom-command path is protected only by the Rust validators; the path capability `"**"` is not a second runtime gate.

This violates the behavioral dual-layer requirement U3 and the effective scope requirements U12/SU13/SU18. The static capability-shape tests would not catch this.

Required: make the scoped authorization apply to the exact Rust command path, or otherwise add an effective second authorization layer and regression test it end to end.

### F-02 — P2: Required Windows and keyboard smoke remains unverified

The plan explicitly leaves Windows smoke and keyboard-only review unchecked. [SMOKE.md](C:/dev/AI-Brains-wt-t173/conductor/tracks/trackT173-desktop-security-ux/evidence/SMOKE.md:42) records the live Isolation/WebView2 smoke as deferred.

This leaves U15 and SU8 unproven. T174’s Playwright ownership does not replace the required Windows/WebView2 and manual keyboard verification.

Required: run and record Isolation startup, ping, briefing, review-list, keyboard review resolution, typed-WIPE, Escape, focus restoration, and Enter-focus behavior.

### F-03 — P3: Stale status is not wired to production UI

`StatusBadge` defines a `stale` kind, but no production component renders it. `StatePanel` has no stale status route, while Home renders freshness/stale counts as plain text in [HomeScreen.tsx](C:/dev/AI-Brains-wt-t173/apps/desktop/src/screens/HomeScreen.tsx:198).

This leaves SU9 incomplete. It is an easy fix and should not be deferred.

### F-04 — P2: Required `cargo audit` result is missing

SU12 requires `license:check + deny + audit`. The smoke evidence records license checking and `cargo deny check`, but no `cargo audit` result in [SMOKE.md](C:/dev/AI-Brains-wt-t173/conductor/tracks/trackT173-desktop-security-ux/evidence/SMOKE.md:12).

The README documents how to run it, but documentation is not gate evidence.

## Completeness Sweep

- No T173 placeholders or fake security paths found.
- Isolation pass-through is intentional and honestly documented as hygiene-only.
- No JS opener package found.
- No raw HTML injection found.
- Production host unwrap/expect/panic scan found only test-module usages.
- Path capability breadth (`"**"`) and Isolation’s inability to deny IPC are documented residuals.
- No uncommitted work exists.
- Internal review R2’s “CLEAN” conclusion is superseded by F-01 through F-04.

## Wiring and Regression Review

Implemented and reachable:

- Single-instance plugin is registered before opener.
- Rust opener commands are registered in the invoke handler and AppManifest.
- Source locators are API-derived; absent locators display honestly.
- Native dialog uses `showModal()`, Escape handling, restore-focus, aria-live, and typed `WIPE`.
- Production CSP and Isolation configuration are wired.
- Focus-visible and scroll-padding styles are present.
- Offline/denied/unavailable/error badges are wired.

The principal wiring failure is that the static scoped opener permissions are not applied to the custom Rust command execution path.

## Verification Evidence

Reported passing:

- npm typecheck
- Vite build
- npm license check
- Desktop Rust tests
- Clippy
- Formatting
- Cargo deny
- Desktop cargo check

Read-only checks:

- Git range and working-tree inspection completed.
- Static capability, Isolation, CSP, opener, dialog, frontend storage, analytics, and host-panic scans completed.
- `ledgerful doctor` and `ledgerful verify` could not complete because the restricted environment could not open the ledger database or write verification reports.

## Deferred Candidates

Only these documented residuals are reasonable difficult, non-blocking P3 candidates:

- Isolation hook cannot deny IPC; hygiene/audit-only behavior is documented.
- Broad path capability `"path": "**"`; documented residual.

F-01, F-02, F-03, and F-04 should not be deferred for track completion.

## Completion Decision

Do not mark T173 complete. Resolve the effective opener authorization boundary, perform the required Windows/keyboard smoke, wire stale status presentation, and provide `cargo audit` evidence before re-review.