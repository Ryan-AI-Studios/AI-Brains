# T320 Plan — unified `ai-brains status` glance

**Status:** **Planned** (Pending until **go**). Spec [spec.md](./spec.md).
**Category:** FEATURE / UX
**Ledger (planning):** DOCS `dcb67912-8fb7-4bbd-a354-68ba41857744`
**Ledger (fold-in):** DOCS `a92f9b07-1894-42a1-8526-9f66fa9ed02d`

---

## Preflight (plan time — 2026-08-29)

| Check | Result |
|-------|--------|
| HEAD / tree | Fold-in `e15188e` plan commit CLEAN; `origin/main` = `464edc2` (ahead **1**). Plan-write was `464edc2` / ahead **0** (Agy m1). Branch `track/T320-unified-status`. Product `src/` = T319 `#236`. |
| PATH `ai-brains` | **0.1.3** graph-on; **26,897,408** B; mtime **2026-08-27 8:21:55 PM**. T320 hole **is** (`status` unrecognized, exit 2). T312–T319 **not** on PATH. |
| `preflight --summary` (PATH) | Pinned **4563**; in-context **0/0/0**; plan-write words **699**; OpenCode **705** (PATH-behind T315) |
| `doctor --summary` | degraded; **only** graph_density warn E/N **0.416**; exit 0. Stopped daemon would **not** show (daemon_reachable always ok). |
| `nightly --status --quick` | last **2026-08-28T07:08:30Z**; scheduled Yes; last_result **0**; probe=skipped |
| `daemon status` | Running; PID 3592; TCP Open; exit 0 |
| `graph update --format human` | sparse; E/N **0.4155**; no remediator line |
| `ai-brains status --help` | clap unrecognized subcommand exit **2** |
| `help_ia.rs` Daily | `recall, preflight, doctor, project, pin, memory, context, stop-session, daemon` — **no status** |
| rustc | **1.95.0** |
| Pins | clap `"4.5"` / lock **4.6.1** / crates.io **4.6.6**; rusqlite **0.40.2**; serde_json **1.0.150**; uuid ws `"1.13"` / lock **1.23.1**; tokio **1.53.1**; workspace **0.1.3** — no bump |
| Last PR Cursor | `#236` `mergedAt` **2026-08-29T01:32:14Z**; comments/reviews **[]** — **N/A empty**. `#235` empty. `#230` → **T325** already. |
| Open PRs | **none** |
| Ledger | 0 pending / 0 drift at scan; this TX `dcb67912` |
| Hotspots | `project.rs` #1 **3.698** — do not touch. `sync.rs` #2. `governed_common.rs` #3. `daemon.rs` #8 — do not grow `run_status`. |
| Line counts | Nonblank (plan-write): doctor **1738**; nightly **1968**; daemon **1232**; `commands/graph.rs` **1606**; main **5402**. Physical (OpenCode m1): **1855 / 2128 / 1341 / 1731 / 5578**. F32 = go-HEAD diff. |
| `ISSUES.md` | **Does not exist** |
| Planning install / live pin / live rebuild / daemon mutate | **Not run** |

---

## Absorbed deferred

| Item | Plan action |
|------|-------------|
| Audit unified-status opportunity | **DoD** F1–F15 / AC3–AC12 |
| Compose vs subprocess | **F2** in-process |
| Name vs `daemon status` | **F1** top-level `status`; nested unchanged |
| Fail-open / 750 ms | **F4** / **F7** no HTTP |
| T192/T249 doctor summary | **Reuse** F6/F10/F38 |
| T199 daemon status | **F8/F9/F39** IPC Status policy; do not call `run_status` |
| T255 nightly | **F12/F37** last-run + schtasks only |
| T308 floors / Sparse None | **F36** |
| T204 Daily lock | **F17** additive `status` |
| T310 F15 `--version` | **Decline** |
| last-PR `#236` Cursor | **N/A empty** F43 |
| last-PR `#230` F8 recency | **T325** — not stolen |
| T316 / T318 / T321–T324 / clap 5 | **Not stolen** / **Decline** |
| minikube bitwise exit | **F31** decline |
| OpenCode m1 line counts | **Partial** §2.3 dual-count; F32 go-HEAD; decline `src/graph.rs` |
| OpenCode O1 AC9 host daemon | **F45 / AC9** |
| OpenCode O2 `status_next_line` | **F27 / AC7** |
| OpenCode O3 scheduled mapper | **F12 / AC6** `next_run.is_some()` |
| Agy m1 HEAD | **§2.1** `e15188e` / ahead **1** |
| Agy m2 `graph_density` path | **§2.3** `src/graph_density.rs` |

---

## Phase 0 — on go (re-verify + deferred rescan)

- [ ] `ledgerful doctor` ; `ledgerful ledger status --compact`
- [ ] Confirm cwd `C:\dev\AI-Brains`
- [ ] Re-read doctor `build_report` `:77` + `format_doctor_summary` `:932` + 15-check unit `:1065` + `daemon_reachable` always-ok `:187–192`
- [ ] Re-read `main.rs` doctor early dispatch `:4454` + `Commands::Doctor` unreachable `:4551`
- [ ] Re-read `daemon.rs` `run_status` `:739` (do **not** call) + `status_next_line` `:697–701` (F27 reuse) + `daemon_probe.rs` Status vs Safety
- [ ] Re-read `nightly.rs` status branch `:40–222` + `scheduled = snap.next_run.is_some()` `:104` (F12) + `fetch_schedule_snapshot` `:1067` (still private?)
- [ ] Re-read `commands/graph.rs` `graph_health_report` `:429` (do **not** import) + `crates/ai-brains-cli/src/graph_density.rs` gather/assess (`main.rs:9`)
- [ ] Re-read `help_ia.rs:11` Daily string + unit `:57–61` + `memory_list_inventory.rs:612`
- [ ] Re-dogfood `doctor --summary` / `nightly --status --quick` / `daemon status` / `graph update --format human` / `ai-brains status --help` (must still be missing until green)
- [ ] Confirm clap lock still **4.6.1**; floors still `0.50`; `DoctorReport` still 15
- [ ] Rescan `deferred.md` open overlapping rows
- [ ] Confirm T325 placeholder still Pending (do not steal F8 recency)
- [ ] `ledgerful ledger start T320-unified-status --category FEATURE`
- [ ] **Do not** `cargo install` / `daemon start|stop` / `graph rebuild` / schtasks mutate / `.env` rewrite / clap 5

## Phase 1 — Red

- [ ] `status_envelope__fixture__frozen_keys_schema_1` (AC3)
- [ ] `status_envelope__doctor_err__error_keeps_daemon` (AC4)
- [ ] `status_envelope__graph_err__error_keeps_others` (AC5)
- [ ] `status_nightly_human__never_and_not_scheduled` (AC6)
- [ ] `format_status_human__stopped__reuses_daemon_next` / `format_status_human__running__no_next` (AC7) — human **eq** `daemon::status_next_line(false)`; JSON prefix-less; do **not** reuse daemon.rs test names
- [ ] `format_status_human__includes_doctor_summary_no_nightly_banner` (AC8)
- [ ] `format_status_graph_line__three_decimal_en` (AC16)
- [ ] Clap AC2 `status --format xml` / `JSON` InvalidValue exit 2 (hermetic or clap parse unit)
- [ ] Hermetic AC9 `status__format_json__parses_envelope` — keys only; **do not** assert `daemon.state` (host IPC / F45)
- [ ] Confirm red tests **fail** on current tree (no `commands/status.rs`; clap has no `Status`)

## Phase 2 — Green

- [ ] `commands/status.rs` + `mod status` (F29)
- [ ] `Commands::Status { format }` display_order 12; tokens F14; `after_help` F26
- [ ] Early dispatch before AppContext (F30) — same class as doctor
- [ ] Probe `DaemonProbePolicy::Status`; `build_report`; slim doctor JSON (F9/F10)
- [ ] Graph gather+assess; human one-liner (F11)
- [ ] Nightly `sync_state` SQL + `pub(crate)` `fetch_schedule_snapshot`; `scheduled = snap.next_run.is_some()` (F12/F34/AC6)
- [ ] Human `next:` = `daemon::status_next_line(false)`; JSON prefix-less const (F27)
- [ ] Fail-open per section (F4); exit 0 (F5)
- [ ] `help_ia.rs` Daily + Start-here; update both Daily string tests (F17)
- [ ] Production: no `unwrap`/`expect`/`panic`
- [ ] Isolation: doctor.rs / `commands/graph.rs` / daemon.rs `run_status` / project.rs / sync.rs / governed_common.rs behavior-empty (AC13); import `status_next_line` only

## Phase 3 — Stay-green

- [ ] AC1 format_resolve auto TTY/pipe
- [ ] AC14 doctor 15-check; T255 nightly heading; T249 daemon `next:`; T308 sparse omits remediation
- [ ] AC15 graph-off `status --format json` exit 0 (not FEATURE_UNAVAILABLE)
- [ ] `daemon status` still Running/Stopped + TCP (F39)
- [ ] `nightly --status` still `=== Nightly Status ===` (F37)
- [ ] `graph update` JSON keys unchanged (F40)

## Phase 4 — Docs

- [ ] CAPABILITIES Daily + format-matrix row + honesty bullet
- [ ] PROTOCOL-COMPAT P-CLI **add** `status` row (pretty JSON; keys §5.1)
- [ ] OPERATIONS glance example; does not replace doctor
- [ ] CLI-EXIT-CODES `status` exit 0 footnote
- [ ] CHANGELOG Unreleased
- [ ] conductor.md Completed only after go + merge

## Phase 5 — Manual + isolation

- [ ] AC11 `cargo run -q -p ai-brains-cli -- status` + `--format json` (PATH-behind not a fail; observed-data for E/N / last_run)
- [ ] AC13 in-repo `crates/` name-only isolation
- [ ] Confirm no daemon mutate / no rebuild / no schtasks /change
- [ ] `status --help` F26 examples

## Phase 6 — Review + gate + Complete (only after go)

- [ ] Phase-1 review log `review.md` until clean
- [ ] Cross-model `codex-review` (F23 FEATURE)
- [ ] Targeted nextest + clippy `-p ai-brains-cli --all-targets -- -D warnings`
- [ ] Full gate: fmt / clippy workspace / nextest / deny / audit / `ledgerful verify --scope full`
- [ ] FEATURE TX commit; push `track/T320-*`; PR; watch GHA `CI` green; squash-merge; prune (never `git push origin main`)

---

## DoD (checkable)

- [ ] `ai-brains status` exists; four named sections; fail-open
- [ ] JSON envelope §5.1; pretty; no contracts DTO
- [ ] Exit 0 for degraded/Stopped/sparse/never; clap 2 on bad `--format`
- [ ] No HTTP probes; no daemon TCP on glance; no doctor.rs growth
- [ ] Graph-off glance works (AC15)
- [ ] T204 Daily includes `status`; doctor 15 / nightly chrome / daemon status stay-green
- [ ] Docs AC12; CHANGELOG; no clap 5; no T325 steal
- [ ] Conductor **Completed** only after merge + hygiene

## Isolation (repeat)

Do not grow `doctor.rs`. Do not call `nightly::run` or `daemon::run_status`. Do not import `commands/graph.rs`. Reuse `status_next_line`. No `cargo install`. Never `git push origin main`.
