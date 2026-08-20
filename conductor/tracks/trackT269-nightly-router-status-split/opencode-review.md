# Track review: T269-NightlyRouterStatusSplit

**Harness:** OpenCode (`opencode`)
**Track:** `conductor/tracks/trackT269-nightly-router-status-split`
**Date:** 2026-08-20
**HEAD:** `5bfc088` (docs commit "plan T269 Nightly heading vs Router 267009; probe timeout budget" on top of `6825343` T273 #185)

## Summary

T269 is a tightly-scoped OPS/UX/BUGFIX track: add a human-only `Nightly: AI-Brains-Nightly` heading to disambiguate `Last task result: 0` from `Router: … 267009`, and relabel human `probe=timeout` as `probe=timeout (750ms)` without raising the 750 ms budget. JSON stays frozen; `--quick` stays `probe=skipped`; no new crates, pins, DTO, or live task mutation.

The plan is honest about its own scope and declines are written with reasons. Every load-bearing code claim was verified against live `src/` (status branch, banner, schedule block, endpoint line signature, probe constants, clap after_help, T247/T255 regression locks). Pins match `Cargo.lock` and current crates.io. Deferred §9 and last-PR Cursor audits are complete and match what I independently re-checked. F0 go-gate is present (plan-only until **go**; planning DOCS TX `7f7f7fd2…`; implement opens a BUGFIX TX).

Only minor cosmetic findings — stale line-count approximations and a research-HEAD note that predates the plan commit. **Verdict: Planned** (no fixes required before implement; the m-items should fold cheaply).

## Findings (B/M/m/O)

- **m — Stale line-count approximations in spec §2.3 / plan preflight.** Spec says `nightly.rs` **1964** lines / `nightly_status.rs` **554**; live `src/` is **2124** and **593**. Cosmetic — the claim that matters (neither file is top-10 hotspot; helpers belong in `nightly_status.rs`) holds. Fold in the corrected counts.
- **m/O — Spec "research HEAD `6825343`" vs actual HEAD `5bfc088`.** The spec (§1 research date, §2.1 HEAD row) predates the docs-only planning commit; the product tree at `5bfc088` is identical to `6825343` (status clean). Non-material; note it so nobody chases a ghost diff.
- **O — `after_help` doc check could not be fully re-verified via docs.rs.** docs.rs clap 4.6.6 `Command` page truncated on fetch; however `Command::after_help` is a live API already proven in `main.rs` (existing Nightly `after_help` at `main.rs:1058-1062`) with test precedents at `main.rs:610-645` (T273 AC12 / T268 AC13). Non-blocking.
- No B / M findings. No false baseline, no missing F0, no unsafe mutate, no contract miss, no silent reopen of a closed decline.

## What looks solid

- **F0 go-gate + CQRS/ledger discipline.** Plan-only until **go**; planning DOCS TX `7f7f7fd2-5ce1-4892-94d0-451699366dd0`; implement starts BUGFIX TX. `ledgerful ledger status --compact` → 0 pending / 0 unaudited drift; `ledgerful doctor` ready.
- **F1/F7 heading placement is correct and regression-safe.** Heading is a separate `println!` at the status call site *after* the banner (`=== Nightly Status ===` at `nightly.rs:156`) and *before* `format_status_schedule_block` — not inside the helper. Verified the T247 substring locks the plan cites: `format_status_schedule_block__order__result_hint_then_last_scheduled` asserts `lines[1] == "Last task result: 101"` (`nightly.rs:1696-1711`) and `:1503/:1512` assert `lines[1] == "Last task result: unknown"`. F7 is the correct non-reopen.
- **F3/F8 helper design is pure and JSON-safe.** `format_probe_label_human(label, budget_ms) -> String` wrapping only the **human** call site of the 4-arg `format_endpoint_line` (`nightly.rs:800`, call sites `:188`/`:197`, test sites `:1472`/`:1760`). `--quick` stays the string literal `"skipped"` (`nightly.rs:52-53`, T247 F19 comment at `:51`). AC5 fixture check and `format_endpoint_line__quick__probe_skipped` (`:1759-1767`) protect the JSON path.
- **F4 decline of the 750 ms raise is evidence-backed.** `NIGHTLY_STATUS_PROBE_TIMEOUT` = 750 ms (`nightly.rs:13`); run-path `NIGHTLY_PROBE_TIMEOUT` = 2 s (`:11`). Live re-verified llama.cpp #20684 (closed by #20817): `/health` was queued like any HTTP request under load, so raising the budget would not fix a busy slot and would reopen T247 `&lt;1.5s` status. Labeling is the honest remediator.
- **F6/F5 Router line + JSON frozen.** `format_router_status_lines` in `nightly_status.rs` stays T255 AC6/AC15; `Router: Running  last result: 267009` verified as the exact current output shape. `ProbeStatus::as_label` tokens `ok|down|timeout|error` (`ai-brains-models/src/llama_cpp.rs:37-55`), `"skipped"` literal on `--quick` — no suffix leaks into JSON.
- **F10 after_help additive, clap 4, no bump.** Existing Nightly `after_help` (`main.rs:1058-1062`) is format-examples only; plan augments it (keep T255 examples). Cargo.lock clap **4.6.1**, workspace `"4.5"`; crates.io clap **4.6.6** (2026-08-11) — no clap 5. serde_json **1.0.150** (crates.io 1.0.151), tokio **1.52.3** (crates.io 1.53.1), reqwest **0.13.4** (models crate only). No pin bumps; zero new crates.
- **AC10/AC8 hermetic discipline.** AC8 explicitly forbids asserting live `267009` (T255 AC9 lesson) — correct, since host schtasks is reachable from hermetic runs.
- **Non-goals / DoD declines are exhaustive and written with reasons** (daemon TCP→HTTP = T199; doctor 16th / persist probe / `.cmd` / `--no-vault` = T255 F11–F15; T270/T272/T273 F7 = peers; contracts DTO = F15). F9 forbids hotspot growth (`project.rs` #1).

## Deferred fold-in table

Source: `conductor/deferred.md` rows (audit row at `:234`; T269 planning absorption table at `:534-545`) and spec §9. Verified matching:

| deferred item | Disposition (spec §9 / plan) | Verified |
|---|---|---|
| Audit "Nightly human mixes Router 267009; completion probe timeout" | **Absorb** F1–F3 / AC1–AC2 / AC6 / AC8 / AC10 | ✅ heading + human timeout suffix in scope |
| Placeholder F1 Nightly/Router headings | **Absorb** F1 / F6 (Nightly heading is the hole) | ✅ |
| Placeholder F2 `--quick` skipped | **Absorb** F2 / AC8 / AC13 | ✅ string literal preserved |
| Placeholder F3 raise or label 750 ms | **Absorb as label** F3 / F4 — do not raise | ✅ evidence-backed decline |
| Placeholder F4 read-only Router | **Affirm** T255 F7 / F6 / F17 | ✅ |
| Placeholder F5 JSON keys frozen; human-only preferred | **Absorb** F5 / F21 | ✅ no `probe_budget_ms` |
| T255 F18 probe timeouts unchanged | **Affirm** F4 | ✅ |
| T255 closeout (doctor 16th / persist probe / embed sleep / `.cmd` / `--no-vault`) | **Decline** F19 | ✅ closed declines not reopened |
| T255 PATH `cargo install` | **Decline as DoD** F16 | ✅ |
| T255 live reschedule missing `.cmd` | **Decline** F17 | ✅ no schtasks mutate |
| T270 / T272 / T273 F7 | **Decline** — peers | ✅ |
| last-PR Cursor #185 | **N/A** — empty | ✅ re-verified |
| #184 Linux Path units | **Decline** — already `#[cfg(windows)]`, no T274 | ✅ |
| clap 5 / pin bumps / DTO | **Decline** F14 / F15 | ✅ |

## Last-PR Cursor comments

Re-checked at review time:

- **#185 (T273, merged 2026-08-20)** — `gh pr view 185 --comments --json comments,reviews` → `comments: [], reviews: []`. **N/A (empty)**. No leftover to mint.
- **#184 (T268)** — Linux Path units already `#[cfg(windows)]`; no T274 needed. Plan's decline is correct.
- **Open PR on HEAD** — none (Dependabot remotes only). Git tree at `5bfc088` is clean.

## Research / tools notes

- **Pins (verified against `Cargo.lock`, no plan drift):** clap **4.6.1** (crates.io **4.6.6**, released 2026-08-11, no clap 5; `after_help` unchanged), serde_json **1.0.150**, tokio **1.52.3**, reqwest **0.13.4**. Workspace: clap `"4.5"` (derive, env), tokio `"1.52"` (full), reqwest `"0.13"` (json), serde_json `"1.0"`. rustc 1.95.0 / edition 2024 / workspace 0.1.1 / nextest 0.9.140. No bumps proposed.
- **`SCHED_S_TASK_RUNNING` = success.** Microsoft Learn Task Scheduler constants fetched: `SCHED_S_TASK_RUNNING` = `0x00041301` = **267009** decimal is a *success* constant ("The task is currently running"), not an error. Plan's decode claim and its "do not fix 267009" posture confirmed. Human already decodes the hint (`nightly.rs:965`, unit at `:1656`).
- **llama.cpp #20684** fetched (closed, resolved by #20817): `/health` had no fast path and was queued like any HTTP request under load. Confirms raising 750 ms would not fix a busy slot; labeling is the remediator. (Could not verify whether this machine's operator llama.cpp includes #20817 — spec correctly flags this as non-DoD.)
- **Live `src/` opened** (all plan-named files re-checked): `nightly.rs` (2124 lines; status branch `:40-217`, `NIGHTLY_STATUS_PROBE_TIMEOUT` `:13`, banner `:156`, `format_endpoint_line` 4-arg `:800`, `format_status_schedule_block` `:890`, 267009 hint `:965`, T247/T255 lock units `:1503/:1512/:1696-1711/:1759-1767`), `nightly_status.rs` (593 lines; module doc names T255 JSON builder + Router helpers), `main.rs` (Nightly clap `:1058-1084`, after_help test precedents `:610-645`), `ai-brains-models/src/llama_cpp.rs` (`ProbeStatus` `:37-44`, `as_label` `:46-55`, `probe_health` `:127-138`).
- **ai-brains / ledgerful used:** `ai-brains recall "nightly status Router 267009 probe timeout 750ms" --no-bridge` surfaced T255/T267 review summaries (JSON already split; human Router shipped); `ledgerful doctor` ready (5 warnings, Windows/PowerShell); `ledgerful ledger status --compact` 0 pending / 0 drift. No `scan --impact` re-run needed for a plan audit (no code delta).
- **Tool notes:** `rg` unavailable in this environment — used `Select-String`/`Grep` equivalents. docs.rs clap 4.6.6 page fetch truncated — non-blocking (API proven live). Full CI gate not run (plan review; not an execute gate).

## Verdict: Planned

No blockers, no majors. Two m/O cosmetic findings (stale line counts in spec §2.3 / plan preflight; research-HEAD note predating the docs commit) — cheap to fold. Proceed to `/fold-in T269`.
