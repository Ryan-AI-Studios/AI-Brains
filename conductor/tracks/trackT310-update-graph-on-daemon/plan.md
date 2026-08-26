# T310 Plan — `update` graph-on + PATH daemon 4.14

**Status:** **Pending**. Spec [spec.md](./spec.md).
**Category:** CHORE / FEATURE (light)
**Ledger (planning):** DOCS `4e15b2eb-cc78-40e0-aaf2-0dd362814c7e`
**Implement:** **FEATURE** TX on go (src in `run_update`)

---

## Preflight (plan time — 2026-08-26)

| Check | Result |
|-------|--------|
| HEAD / tree | `e577c8c` CLEAN; `main...origin/main` |
| PATH CLI | **0.1.3** graph-on; mtime **2026-08-26 6:54:32 AM**; `cipher_page` **4.14.0 community** |
| PATH daemon | **21,045,248** B; mtime **2026-08-22 2:48:10 PM** |
| `daemon status` | **Running** PID **48960** |
| SCM `AI-Brains-Daemon` | **Stopped**; ImagePath PATH `ai-brainsd.exe --service` |
| `run_update` CLI argv | `:1070–1071` **no** `--features graph` — **this hole** |
| `GRAPH_REINSTALL_SOOT` | `governed_common.rs:45–46` unchanged |
| rustc / Perl | **1.95.0** / **v5.42.2** |
| Pins | rusqlite **0.40.2**; clap **4.5** / lock **4.6.1**; clap 5 declined |
| Last PR Cursor | `#227` comments/reviews/issues **`[]`** — N/A; no T311 |
| Open PRs | **none** |
| Ledger | 0 pending / 0 drift at scan; this TX `4e15b2eb` |
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

- [ ] Re-read `daemon.rs` `run_update` `:1070–1084` vs `GRAPH_REINSTALL_SOOT`
- [ ] Confirm lock rusqlite **0.40.2**; `perl -v`; cwd `C:\dev\AI-Brains`
- [ ] Confirm interactive daemon Running; SCM Stopped
- [ ] Rescan deferred + last-PR Cursor
- [ ] FEATURE TX
- [ ] Do **not** `cargo install` / `daemon stop` / `sc start` until F10 (after green)

## Tasks

- [ ] **Red:** `run_update_cli_args__reconstruct_graph_reinstall_soot` fails on current argv
- [ ] **Green:** `pub(crate)` argv slice reconstructs SOOT; `run_update` uses it (`--features graph`)
- [ ] Daemon argv stays `--path crates/ai-brainsd --locked` (AC4)
- [ ] Do not edit `GRAPH_REINSTALL_SOOT`; do not grow `governed_common.rs`
- [ ] CHANGELOG
- [ ] clippy `-D warnings` `-p ai-brains-cli`; nextest that package
- [ ] Owner-confirm F10: CLI SOOT first, then `ai-brains update` **or** stop + `cargo install --path crates/ai-brainsd --locked` + start — **or** record AC9
- [ ] PR → CI → squash (never `git push origin main`)

## DoD

- [ ] AC1, AC4–AC8
- [ ] AC2 + AC3 **or** AC9 written
- [ ] T307 / T308 floors / SCM start not stolen
