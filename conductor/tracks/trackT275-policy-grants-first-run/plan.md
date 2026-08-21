# T275 Plan — Discovery grants first-run (grant-wall + CLI bootstrap lock)

**Status:** **Completed** 2026-08-21
**Spec:** [spec.md](./spec.md) F0–F37 / AC1–AC16 + §13 AI fold-in
**Category:** FEATURE / UX / GOVERNED
**Ledger TX (planning):** `e13a4e01-3dd6-4adc-ae57-be75e7e98ba9` (DOCS)
**Ledger TX (fold-in):** `79f6e233-42ba-4660-b7c5-5560579ddece` (DOCS)
**Ledger TX (implement):** start **FEATURE** on **go**

---

## AI fold-in (2026-08-21) — `agy-review.md` + `opencode-review.md`

No Blockers / Majors. Disposition in spec **§13**.

### Pins locked by fold-in

1. **F2 / AC2:** `BRIEFING_DENIED_GRANT_WALL` exact 88-char string; `.chars().count() <= 140`; no `\n`.
2. **F35 / AC16:** personal denied must not contain project grant-wall / bootstrap consts (Agy m2).
3. **F36:** AC4/AC5 comment — omit `--principal-id` = `cli_principal()` System (Agy O1).
4. **F37:** GRANT_WALL + HIDDEN immediately after `BRIEFING_DENIED_DENIAL_HINT` (Agy O2).
5. **F16:** strings in CP `renderer.rs`, not CLI domain (OpenCode m6).
6. **F29:** AC2 is renderer order only; no preflight.rs budget hermetic (OpenCode m5).
7. **§2.1 / §2.3:** HEAD `c576b58`; Bootstrap `:2211`; `empty_denied` `:218`.

---

## Preflight (plan time — 2026-08-21)

| Check | Result |
|-------|--------|
| HEAD / tree | Plan dogfood `8cb1ce0`; fold-in `c576b58` (docs-only; product tree identical). CLEAN; `main` ahead of `origin/main` by planning docs |
| PATH `ai-brains` | **0.1.1** mtime 2026-08-21 05:55. T270 on PATH. **Do not `cargo install`.** |
| Source debug | 2026-08-21 07:46. Tests/manual use `cargo run` / hermetic |
| `preflight --summary` | Pinned **3325**; in-context 0/0/0; **grants 0 of 3** + short SOOT |
| `policy bootstrap --dry-run` | `would_issue` ×3; `registered: already`; System `a1b2a1b2`; Scope `3581317d` |
| `policy show` | `grants: []` + `next_step` short SOOT |
| `briefing project` human | **Denied** + bootstrap `--scope …` + Decisions/Conclusions **`_None_`** (8/3 hole) |
| `briefing project` JSON | `denied: true`; `decisions: []`; `denial_hint` short SOOT; exit 0 |
| `evidence`/`source`/`review` list | Exit **3** POLICY_DENIED |
| `query progressive` | Exit **3** + bootstrap + T243 recall fallback |
| `doctor --summary` | `policy_grants` warn 0 of 3 + long SOOT omit-`--scope` |
| Last PR comments | #189 T274 — **empty** (N/A). #188 Mediums stay **T284**. No T285 |
| Open PR on HEAD | none (Dependabot remotes only) |
| Pins | clap lock **4.6.1** (crates.io 4.6.6; **no clap 5**); serde_json **1.0.150** (crates.io **1.0.151**); chrono **0.4.44** (crates.io 0.4.45); rusqlite **0.39.0** (crates.io 0.40.2) — **no bumps** |
| rustc / nextest / workspace | 1.95.0 / 0.9.140 / **0.1.1** |
| Hotspots | `project.rs` **#1** (3.990) — do not grow. `governed_common.rs` #5 — do not grow (hint freeze). CLI `preflight.rs` #7 **2027** — do not grow. `doctor.rs` **1738** — do not grow. `renderer.rs` **497** — **DoD touch**. `policy_cmd.rs` **356** — call only |
| Ledger | 0 pending at scan; planning TX `e13a4e01`; fold-in TX `79f6e233` |
| `ISSUES.md` | **Does not exist** (F27) |
| ledgerful search | `run_bootstrap` `policy_cmd.rs:234`; `empty_denied` `project.rs:218` (OpenCode m3); `issue_grant` callers include T263 seed + `run_bootstrap` |
| Online | clig.dev first-run + dry-run; Entra/Orca/OSO default-deny + least privilege; clap 4.6.6; rusqlite 0.40.2 **not** bumped |

---

## Phase 0 — on go (re-verify)

- [x] `ledgerful doctor` ; `ledgerful ledger status --compact` ; `ledgerful scan --impact` — work root `C:\dev\AI-Brains`; 0 pending / 0 drift (before FEATURE TX)
- [x] Re-read `renderer.rs` denied `_None_` (`:93–130`), `run_bootstrap` `:234`, `cli_principal`, `project.rs` `empty_denied` `:218`, clap Bootstrap `:2211`
- [x] Confirm T210 `policy_bootstrap.rs` lacked briefing/evidence; T221 AC3 progressive-after-bootstrap present; T263 granted-empty present
- [x] Rescan `conductor/deferred.md` — T275 rows already absorbed; no new overlapping open rows
- [x] Confirm #189 comments/reviews still empty (N/A); #188 Mediums stay T284; no mint
- [x] Re-dogfood `policy bootstrap --dry-run` (`would_issue` ×3) + `briefing project --format human` (`_None_` on PATH). **Did not** live bootstrap (owner did not confirm)
- [x] Re-check clap lock **4.6.1** (crates.io 4.6.6), rusqlite **0.39.0** (0.40.2), chrono **0.4.44** (0.4.45) — **no bump**
- [x] FEATURE TX `1f2c1ddb-5657-4af9-9a30-8285efca8895`
- [x] Did **not** `cargo install`; did **not** edit `POLICY_DENIED_HINT`; did **not** grow doctor/preflight/project.rs

---

## Absorbed deferred

| Item | Plan action |
|------|-------------|
| Audit briefing/progressive/lists POLICY_DENIED 0 of 3 | **DoD** F1–F6 / AC1–AC5 |
| Denied looks like empty vault | **DoD** F1/F2 / AC1 — grant-wall, not `_None_` |
| CLI bootstrap → briefing/evidence untested | **DoD** F5/F6 / AC4/AC5 |
| T241 F21 skill one-liner | **Docs** F23 |

---

## Declined (written)

| Item | Why |
|------|-----|
| Auto-grant on init/preflight | F8 — T210 F13 |
| `preflight --install-grants` / doctor `--fix` | F9 — T241 F4/F20 |
| T280 omit-`--scope` on deny hint | F11 — peer |
| T263 H2 pin→Approved | F12 |
| Live vault bootstrap without confirm | F10 |
| last-PR #189 Cursor | N/A empty |
| #188 Work/samples | T284 |
| leftover `7d97a456` | T276 |
| clap 5 / rusqlite 0.40 / DTO / `cargo install` | F17 / F18 |

---

## Phase 1 — Red (TDD)

- [x] Renderer unit `render_project_markdown__denied__no_none_placeholder` (AC1) — **required red** (failed on `_None_`)
- [x] Renderer unit grant-wall 88 chars / ≤140 / order before Decisions (AC2) — red (grant-wall pos missing)
- [x] Hermetic `policy_bootstrap__after_system__briefing_project_denied_false` (AC4) — lock; **F36 comment** omit `--principal-id`
- [x] Hermetic `policy_bootstrap__after_system__evidence_list_exit_0` (AC5) — lock; same F36 comment
- [x] Allowed-empty still `_None_` or empty_authority (AC6) — green-guard `render_project_markdown__allowed_empty__keeps_none_not_grant_wall`
- [x] Personal deny regression `render_personal_markdown__denied__names_recall_not_personal_bootstrap` + new consts (AC16 / F35) — extend T263 unit

---

## Phase 2 — Green

- [x] F37: `BRIEFING_DENIED_GRANT_WALL` + `BRIEFING_DENIED_HIDDEN` immediately after `BRIEFING_DENIED_DENIAL_HINT`
- [x] Denied branch: no `_None_`; grant-wall after bootstrap next, before Decisions
- [x] Allowed-empty path unchanged
- [x] F35: personal denied path does not import/print project grant-wall consts
- [x] No production `unwrap`/`expect`/`panic`
- [x] Do not edit `POLICY_DENIED_HINT` / daemon / `query.rs` twins
- [x] Do not add clap flags

---

## Phase 3 — Docs + registry

- [x] CAPABILITIES: Denied = grant wall; bootstrap then briefing; pins via `recall`
- [x] OPERATIONS: grant-wall sentence on cold-start
- [x] CHANGELOG minor
- [x] `conductor.md` **In Progress** during implement; Completed only after gate + review + publish
- [x] `deferred.md` absorb notes (planning pass already)

---

## Phase 4 — Verify (implement, not plan)

- [x] Targeted: `cargo nextest run -p ai-brains-control-plane --lib` renderer tests — 8 passed
- [x] `cargo nextest run -p ai-brains-cli --test policy_bootstrap --test governed_first_run_deny_exit --test governed_vault_pin_honesty` — 33 passed; AC3 lock +1 passed
- [x] `cargo clippy -p ai-brains-control-plane -p ai-brains-cli --all-targets -- -D warnings` — exit 0
- [x] Manual hermetic: AC4/AC5 cargo nextest (bootstrap → briefing not Denied; evidence list exit 0)
- [x] Live vault: `--dry-run` only (AC15; owner did not confirm)
- [x] Review log `review.md`; R1 + R1b **PASS**; Codex CX1 product **PASS** (P1-1/P2-1 process)
- [x] Full gate: `.\scripts\dev-check.ps1` **SUCCESS** nextest **3253** passed / 1 skipped; `ledgerful verify --scope full` exit 0

---

## DoD

- [x] AC1–AC16 green (AC15 recorded)
- [x] No live grant append without owner confirm
- [x] T280 hint strings unchanged
- [x] T263 H1/H2 isolation held
- [x] Conductor T275 **Completed** after go + gate + review (publish Phase 6 next)
- [x] Pin decisions after implement
