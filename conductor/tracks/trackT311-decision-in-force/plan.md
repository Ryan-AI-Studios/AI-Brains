# T311 Plan — Decision in-force resolver

**Status:** **Completed**. Spec [spec.md](./spec.md).
**Category:** FEATURE
**Ledger (planning):** DOCS `67c2081c-5040-464e-9214-4022556e7f25`
**Ledger (fold-in):** DOCS `e5f9e657-83e8-4402-9fdf-1f7089c151d7`
**Ledger (implement):** FEATURE `e88743aa-e92c-407a-8093-6c6e4e6d9b53`

---

## Preflight (execute — 2026-08-27)

| Check | Result |
|-------|--------|
| HEAD / tree | Plan-fold `3c5d49c`; FEATURE TX started; dirty with product + docs |
| clap / rusqlite / time / serde_json | **4.6.1** / **0.40.2** / **0.3.47** / **1.0.150** — no bump |
| rustc | **1.95.0** |
| daemon | Running PID 15200; **not** stopped |
| hotspots | `governed_common.rs` #3 — not grown |
| last-PR `#228` | comments/reviews/issue **0**; open PRs **none** |
| FEATURE TX | `e88743aa-e92c-407a-8093-6c6e4e6d9b53` |
| Cherry-pick archive | **Not run** |

---

## Absorbed deferred

| Item | Plan action |
|------|-------------|
| Archived T95 in-force WIP | **DoD** F1–F18 / AC1–AC11 |
| T310 elevated PATH install | **Evidence** §2.1; not DoD |
| T307 / T308 floors / recovery kit / H2 / clap 5 / `--version` | **Declined** spec §9 |

---

## Phase 0 (on go)

- [x] Re-read `main.rs` `DecisionCommands`, `adapters.rs` `list_decisions`, `briefings/project.rs` `decision_valid_at` (make `pub(crate)` on go)
- [x] Confirm lock clap **4.6.1**, rusqlite **0.40.2**; cwd `C:\dev\AI-Brains`
- [x] Confirm interactive daemon Running; do **not** `cargo install`
- [x] `ledgerful hotspots` — keep resolver out of `governed_common.rs` / `project.rs`
- [x] Rescan deferred + last-PR Cursor (`#228` was empty; re-check latest merged)
- [x] FEATURE TX (new) `e88743aa`
- [x] Do **not** cherry-pick `archive/track-t95-in-force`

## Tasks

- [x] `pub(crate) fn decision_valid_at` in `briefings/project.rs` (visibility only)
- [x] **Red:** AC1 CP test fails (no `in_force` module)
- [x] **Green:** `control-plane/src/in_force.rs` + `lib.rs` export; AC1–AC7
- [x] CLI `DecisionCommands::InForce` + `value_parser` `--format` + `run_in_force` (`ReadDecisions`, F12 helper)
- [x] Help `after_help` example; default `--format json`
- [x] Hermetic AC8–AC10 (clap help + deny exit 3 + `ruling` key)
- [x] CHANGELOG + CAPABILITIES/OPERATIONS one-liners
- [x] clippy `-D warnings` `-p ai-brains-control-plane` `-p ai-brains-cli`; nextest those packages
- [ ] PR → CI → squash (never `git push origin main`)

## DoD

- [x] AC1–AC11 (targeted)
- [x] T307 / T308 floors / H2 / daemon wire / retrieval crate **not stolen**
- [x] Manual help + JSON unknown-term recorded in review.md
- [x] Full `dev-check.ps1` + `ledgerful verify --scope full`

## Manual evidence (execute)

```
cargo run -q -p ai-brains-cli -- decision in-force --help
# <TERM>; --format [default: json] possible values auto,pretty,human,text,json,markdown,md

cargo run -q -p ai-brains-cli -- decision in-force workspace_id --format json
# {"term":"workspace_id","scope":"Repository:3581317d-…","ruling":null,"chain":[]}  exit 0

cargo run -q -p ai-brains-cli -- decision in-force "   " --format json
# term must be non-empty  exit 2
```
