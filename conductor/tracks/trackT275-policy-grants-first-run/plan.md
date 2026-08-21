# T275 Plan — Discovery grants first-run (grant-wall + CLI bootstrap lock)

**Status:** **Pending** (Planned; plan-only until **go**)
**Spec:** [spec.md](./spec.md) F0–F34 / AC1–AC15
**Category:** FEATURE / UX / GOVERNED
**Ledger TX (planning):** `e13a4e01-3dd6-4adc-ae57-be75e7e98ba9` (DOCS)
**Ledger TX (implement):** start **FEATURE** on **go**

---

## Preflight (plan time — 2026-08-21)

| Check | Result |
|-------|--------|
| HEAD / tree | `8cb1ce0` T274 `#189`. CLEAN. In sync with `origin/main` |
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
| Pins | clap lock **4.6.1** (crates.io 4.6.6; **no clap 5**); serde_json **1.0.150**; chrono **0.4.44** (crates.io 0.4.45); rusqlite **0.39.0** (crates.io 0.40.2) — **no bumps** |
| rustc / nextest / workspace | 1.95.0 / 0.9.140 / **0.1.1** |
| Hotspots | `project.rs` **#1** (3.990) — do not grow. `governed_common.rs` #5 — do not grow (hint freeze). CLI `preflight.rs` #7 **2027** — do not grow. `doctor.rs` **1738** — do not grow. `renderer.rs` **497** — **DoD touch**. `policy_cmd.rs` **356** — call only |
| Ledger | 0 pending at scan; planning TX `e13a4e01` |
| `ISSUES.md` | **Does not exist** (F27) |
| ledgerful search | `run_bootstrap` `policy_cmd.rs:234`; `empty_denied` `project.rs:217`; `issue_grant` callers include T263 seed + `run_bootstrap` |
| Online | clig.dev first-run + dry-run; Entra/Orca/OSO default-deny + least privilege; clap 4.6.6; rusqlite 0.40.2 **not** bumped |

---

## Phase 0 — on go (re-verify)

- [ ] `ledgerful doctor` ; `ledgerful ledger status --compact` ; `ledgerful scan --impact`
- [ ] Re-read `renderer.rs` denied `_None_` (`:93–130`), `run_bootstrap`, `cli_principal`, `project.rs` empty_denied
- [ ] Confirm T210 `policy_bootstrap.rs` still lacks briefing/evidence; T221 AC3 progressive-after-bootstrap; T263 granted-empty
- [ ] Rescan `conductor/deferred.md` for new open rows that overlap
- [ ] Confirm #189 still empty / #188 still T284 / no new Cursor leftover that needs a mint
- [ ] Re-dogfood `policy bootstrap --dry-run` + `briefing project --format human` (expect `_None_` until green). **Do not** live bootstrap unless owner confirmed
- [ ] Re-check clap/rusqlite/chrono lock vs crates.io (**no bump** unless execute proves otherwise)
- [ ] FEATURE TX start
- [ ] Do **not** `cargo install`; do **not** edit `POLICY_DENIED_HINT`; do **not** grow doctor/preflight/project.rs

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

- [ ] Renderer unit `render_project_markdown__denied__no_none_placeholder` (AC1) — **required red**
- [ ] Renderer unit grant-wall ≤140 / order before Decisions (AC2)
- [ ] Hermetic `policy_bootstrap__after_system__briefing_project_denied_false` (AC4) — lock if already green
- [ ] Hermetic `policy_bootstrap__after_system__evidence_list_exit_0` (AC5) — lock if already green
- [ ] Allowed-empty still `_None_` or empty_authority (AC6) — write as green-guard if needed

---

## Phase 2 — Green

- [ ] `BRIEFING_DENIED_GRANT_WALL` + hidden placeholder const in `renderer.rs`
- [ ] Denied branch: no `_None_`; grant-wall after bootstrap next, before Decisions
- [ ] Allowed-empty path unchanged
- [ ] No production `unwrap`/`expect`/`panic`
- [ ] Do not edit `POLICY_DENIED_HINT` / daemon / `query.rs` twins
- [ ] Do not add clap flags

---

## Phase 3 — Docs + registry

- [ ] CAPABILITIES: Denied = grant wall; bootstrap then briefing; pins via `recall`
- [ ] OPERATIONS: grant-wall sentence on cold-start
- [ ] CHANGELOG minor
- [ ] `conductor.md` stays **Pending** until implement closeout; this planning pass sets Planned in spec only
- [ ] `deferred.md` absorb notes (planning pass already)

---

## Phase 4 — Verify (implement, not plan)

- [ ] Targeted: `cargo nextest run -p ai-brains-control-plane --lib` renderer tests
- [ ] `cargo nextest run -p ai-brains-cli --test policy_bootstrap --test governed_first_run_deny_exit --test governed_vault_pin_honesty`
- [ ] `cargo clippy -p ai-brains-control-plane -p ai-brains-cli --all-targets -- -D warnings`
- [ ] Manual hermetic: bootstrap → briefing not Denied; evidence list exit 0
- [ ] Live vault: `--dry-run` only unless owner confirmed (AC15)
- [ ] Review log `review.md`; cross-model F26
- [ ] Full gate at closeout only

---

## DoD

- [ ] AC1–AC14 green (AC15 recorded)
- [ ] No live grant append without owner confirm
- [ ] T280 hint strings unchanged
- [ ] T263 H1/H2 isolation held
- [ ] Conductor T275 **Completed** only after go + gate + review + publish
- [ ] Pin decisions after implement
