# T311 Plan — Decision in-force resolver

**Status:** **Pending** (plan-only until go). Spec [spec.md](./spec.md).
**Category:** FEATURE
**Ledger (planning):** DOCS `67c2081c-5040-464e-9214-4022556e7f25`
**Ledger (fold-in):** DOCS `e5f9e657-83e8-4402-9fdf-1f7089c151d7`
**Implement:** FEATURE TX on **go** (not this pass)

---

## Preflight (plan-write — 2026-08-27)

| Check | Result |
|-------|--------|
| HEAD / tree | Fold-in `b7ca150` CLEAN; plan-write was `bc74098`. Branch `track/T311-decision-in-force` |
| PATH CLI | **0.1.3** graph-on; **26,842,112** B; mtime **2026-08-27 05:52:13** (owner **elevated** install) |
| PATH daemon | **22,377,984** B; mtime **2026-08-27 05:51:37** |
| `daemon status` | **Running** PID **15200** |
| SCM `AI-Brains-Daemon` | Not re-queried this pass; T310 F12 stands (do not `sc start`) |
| `decision` CLI | Propose only — **this hole** |
| Archive | tag `archive/track-t95-in-force` @ `7812b61` (retrieval + inverted `Superseded` test) |
| rustc | **1.95.0** (T310 snapshot; re-check on go) |
| Pins | clap lock **4.6.1**; rusqlite **0.40.2**; time **0.3.47**; serde_json **1.0.150** |
| Last PR Cursor | `#228` comments **`[]`** — N/A; no T312 |
| Open PRs | **none** |
| Ledger | 0 pending / 0 drift at scan; this DOCS TX `67c2081c` |
| `ISSUES.md` | **Does not exist** |
| Planning install / daemon stop | **Not run** |

---

## Absorbed deferred

| Item | Plan action |
|------|-------------|
| Archived T95 in-force WIP | **DoD** F1–F18 / AC1–AC11 |
| T310 elevated PATH install | **Evidence** §2.1; not DoD |
| T307 / T308 floors / recovery kit / H2 / clap 5 / `--version` | **Declined** spec §9 |

---

## Phase 0 (on go)

- [ ] Re-read `main.rs` `DecisionCommands`, `adapters.rs` `list_decisions`, `briefings/project.rs` `decision_valid_at` (make `pub(crate)` on go)
- [ ] Confirm lock clap **4.6.1**, rusqlite **0.40.2**; cwd `C:\dev\AI-Brains`
- [ ] Confirm interactive daemon Running; do **not** `cargo install`
- [ ] `ledgerful hotspots` — keep resolver out of `governed_common.rs` / `project.rs`
- [ ] Rescan deferred + last-PR Cursor (`#228` was empty; re-check latest merged)
- [ ] FEATURE TX (new)
- [ ] Do **not** cherry-pick `archive/track-t95-in-force`

## Tasks

- [ ] `pub(crate) fn decision_valid_at` in `briefings/project.rs` (visibility only)
- [ ] **Red:** AC1 CP test fails (no `in_force` module)
- [ ] **Green:** `control-plane/src/in_force.rs` + `lib.rs` export; AC1–AC7
- [ ] CLI `DecisionCommands::InForce` + `value_parser` `--format` + `run_in_force` (`ReadDecisions`, F12 helper)
- [ ] Help `after_help` example; default `--format json`
- [ ] Hermetic AC8–AC10 (clap help + deny exit 3 + `ruling` key)
- [ ] CHANGELOG + CAPABILITIES/OPERATIONS one-liners
- [ ] clippy `-D warnings` `-p ai-brains-control-plane` `-p ai-brains-cli`; nextest those packages
- [ ] PR → CI → squash (never `git push origin main`)

## DoD

- [ ] AC1–AC11
- [ ] T307 / T308 floors / H2 / daemon wire / retrieval crate **not stolen**
- [ ] Manual help + JSON unknown-term recorded in plan or closeout
