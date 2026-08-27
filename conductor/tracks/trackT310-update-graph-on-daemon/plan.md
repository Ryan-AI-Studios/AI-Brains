# T310 Plan — `update` graph-on + PATH daemon 4.14

**Status:** ✅ **Completed**. Spec [spec.md](./spec.md).
**Category:** CHORE / FEATURE (light)
**Ledger (planning):** DOCS `4e15b2eb-cc78-40e0-aaf2-0dd362814c7e`
**Ledger (fold-in):** DOCS `20060ded-80be-4a78-b10b-a7dd69e4f817`
**Implement:** **FEATURE** TX `65008805-2230-485d-84d3-580659b519b8`

---

## Preflight (fold-in — 2026-08-26)

| Check | Result |
|-------|--------|
| HEAD / tree | Fold-in `87919dd` CLEAN; `origin/main...HEAD` **ahead 1**. Plan-write was `e577c8c` / 0/0 (Agy m1). |
| PATH CLI | **0.1.3** graph-on; mtime **2026-08-26 6:54:32 AM**; `cipher_page` **4.14.0 community** |
| PATH daemon | **21,045,248** B; mtime **2026-08-22 2:48:10 PM** |
| `daemon status` | **Running** PID **48960** (plan-time; re-check on go) |
| SCM `AI-Brains-Daemon` | **Stopped**; ImagePath PATH `ai-brainsd.exe --service` |
| `run_update` | `daemon.rs:1034–1100`; CLI argv `:1070–1071` **no** `--features graph` — **this hole** |
| `run_start` | `:5–20` |
| `GRAPH_REINSTALL_SOOT` | `governed_common.rs:45–46` unchanged |
| rustc / Perl | **1.95.0** / **v5.42.2** |
| Pins | rusqlite **0.40.2**; clap **4.5** / lock **4.6.1**; clap 5 declined |
| Last PR Cursor | `#227` `mergedAt` **22:22:01Z**; comments **`[]`** — N/A; no T311 |
| Open PRs | **none** |
| Ledger | 0 pending / 0 drift at scan; plan TX `4e15b2eb`; this fold-in TX `20060ded` |
| `ISSUES.md` | **Does not exist** |
| Planning install / daemon stop | **Not run** |

---

## Absorbed deferred

| Item | Plan action |
|------|-------------|
| T306 F9 `run_update` graph-off | **DoD** F1 / F9 / AC1 |
| T306 F8 PATH `ai-brainsd` 4.10 | **DoD** F4 / F10 / F11 / AC2 (or AC9) |
| T309 R3 T310 placeholder | **This plan** |
| T307 / T308 floors / clap 5 / T197 | **Declined** spec §9 |

---

## Phase 0 (on go)

- [x] Re-read `daemon.rs` `run_update` `:1070–1084` vs `GRAPH_REINSTALL_SOOT`
- [x] Confirm lock rusqlite **0.40.2** (`23f2a97d…`); `perl` v5.42.2; cwd `C:\dev\AI-Brains`
- [x] Confirm interactive daemon Running PID 48960; SCM Stopped
- [x] `ledgerful hotspots` — `governed_common.rs` #3; F1 keep argv in `daemon.rs`
- [x] Rescan deferred + last-PR Cursor (`#227` comments `[]`; T307 not stolen)
- [x] FEATURE TX `65008805-2230-485d-84d3-580659b519b8`
- [x] Do **not** `cargo install` / `daemon stop` / `sc start` / PATH `ai-brains update` until F10 (after green)

## Tasks

- [x] **Red:** `run_update_cli_args__reconstruct_graph_reinstall_soot` fails on current CLI argv (source-level: join lacked `--features graph`)
- [x] **Green:** `UPDATE_CLI_CARGO_ARGS` reconstructs SOOT; `run_update` uses it
- [x] **AC4 lock:** `UPDATE_DAEMON_CARGO_ARGS` + `run_update_daemon_args__no_graph_feature` (stay-green on today’s daemon argv)
- [x] Do not edit `GRAPH_REINSTALL_SOOT`; do not grow `governed_common.rs`
- [x] CHANGELOG
- [x] clippy `-D warnings` `-p ai-brains-cli`; nextest that package (1602 passed)
- [x] Owner-confirm F10: CLI SOOT first, then F10 **OR** path (`cargo install --path crates/ai-brainsd --locked` + `daemon start`) after PATH `daemon update` hit os error 5 self-replace
- [ ] PR → CI → squash (never `git push origin main`)

## DoD

- [x] AC1, AC4–AC8
- [x] AC2 + AC3 **or** AC9 written
- [x] T307 / T308 floors / SCM start not stolen
