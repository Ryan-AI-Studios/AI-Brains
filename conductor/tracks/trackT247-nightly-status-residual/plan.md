# T247 Plan — Nightly status residual

**Status:** ✅ **Completed** 2026-08-13 PR #157 `43191ff` (ledger `5211d86f`)  
**Spec:** [spec.md](./spec.md) F1–F20 / AC1–AC14 + §14 AI fold-in  
**Category:** OPS / BUGFIX / PERF / UX  
**Ledger TX (on go):** `ledgerful ledger start T247-nightly-status-residual --category FEATURE --message "Status --quick + parallel 750ms probes + LIST/V Last Result honesty; no live reschedule"`

---

## AI fold-in (2026-08-13) — `C:\dev\AI-review.md` AI1 + AI2

No Highs. AI1 restates planned work with a concrete parser/decode sketch. AI2 two mediums are **must-pin** before go (string-literal `skipped`; `join!` + Windows 750 ms Timeout).

### Disposition

| ID | Source | Verdict | Action |
|----|--------|---------|--------|
| **AI1 M1** | AI1 | **Agree** | F3 `SchtasksListV` + `parse_schtasks_list_v` |
| **AI1 M2** | AI1 | **Agree, refined** | F6 first-quoted-token (AI2 L5/L6) |
| **AI1 M3** | AI1 | **Agree, refined** | F2 `join!`; **decline** `ProbeOutcome::Skipped` |
| **AI1 M4** | AI1 | **Agree, refined** | F1/F4; hex via radix (AI2 L2) |
| **AI1 L1–L2 / O1** | AI1 | **Agree** | Already F8 / Phase 1 names |
| **AI2 M1** | AI2 | **Agree hard** | `"skipped"` string; no models change |
| **AI2 M2** | AI2 | **Agree hard** | `join!` not `try_join!`; Win Timeout note |
| **AI2 L1–L12 / L15** | AI2 | **Agree** | Pinned into F3–F6 / F9 / F17 / F19 |
| **AI2 L13–L14** | AI2 | **Affirm** | clap 5 future-guard; T255 boundary |
| **AI2 O12 `--no-vault`** | AI2 | **Decline as DoD** | T255 class; `--quick` still opens vault |
| **AI1 ~50 ms `--quick`** | AI1 | **Decline** | Keep &lt;1s; vault still opens |

### Pins locked by fold-in

1. **F1/F19:** `"skipped"` to `format_endpoint_line`; still `resolve_nightly_model_endpoints`.  
2. **F2:** `tokio::join!`; 750 ms const; Windows may wait full 750 ms.  
3. **F3:** struct parser; LIST /V non-zero → no PS.  
4. **F4:** hex/decimal `u32`; hint on the **next** line.  
5. **F5:** last-scheduled after result, before vault last-run.  
6. **F6:** first quoted `.cmd`/`.bat`/`.exe`; product `'exe' nightly` not missing if exe exists.

---

## Preflight (plan time — 2026-08-13)

| Check | Result |
|-------|--------|
| `nightly --status` | 1068 ms; probes **ok**; Last task result **1**; vault last-run **2026-08-02**; multi-import **never** |
| `Get-ScheduledTaskInfo` | LastRunTime **8/13/2026 3:00:01 AM**; LastTaskResult **1** |
| Task To Run | `"C:\Users\RyanB\.ai-brains\nightly-run.cmd"` — **file missing** |
| `nightly-run.log` | Missing |
| Timing | PS Get-ScheduledTaskInfo **401 ms**; schtasks CSV **30 ms**; LIST /V **33 ms**; `/health` 6–57 ms |
| T229 F5 panic 101 | **Cleared** as Last Result; residual is **exit 1** + missing wrapper |
| clap / reqwest / tokio | 4.5 / lock 4.6.1; reqwest 0.13.4 models-only; tokio 1.52.3 — **no bumps** |
| llama.cpp `/health` | Still official public probe (fetched 2026-08-13) |
| Ledger | 0 pending, 0 unaudited drift; tree clean at plan time |
| T255 softs | F8–F12/F14 **not** absorbed |

---

## Absorbed deferred

| Item | Source | Plan action |
|------|--------|-------------|
| Status 4–6s | audit + deferred.md T247 | **DoD F1/F2/F7** — `--quick` + parallel 750 ms |
| Last Result 101 residual | T229 closeout / placeholder | **Disposition F4/F18**: 101 class still decoded; **live** result is **1** |
| Operator Last Result stays 101 until next run | T229 §9 | **Superseded** — next run will stay **1** until action target exists |
| `--quick` skip HTTP | placeholder F1 | **DoD F1/F17/F19** |
| Parallel or short timeout | placeholder F2 | **DoD F2** |
| JSON status | placeholder F4 / T229 F12 | **Soft F11 → T255** |
| Doctor model ports / embed sleep / Router codes | T229 F8/F11/F14 | **Not absorbed → T255** |

---

## Phase 0 — Ledger + impact (on go)

- [x] `ledgerful ledger status --compact` — 0 pending, 0 unaudited drift (pre-go)
- [x] `ledgerful ledger start T247-nightly-status-residual --category FEATURE` — tx `5211d86f-cad6-45b8-9c9d-01d1a24a6cea`
- [x] `ledgerful scan --impact` — MEDIUM (planning dirt only); nightly.rs/main.rs not yet in working tree
- [x] Confirm no other agent is editing `nightly.rs` — implementers isolated in worktrees

## Phase 1 — Red → Green: LIST /V parse + decode (F3 / F4 / F5 / AC1–AC4 / AC14)

- [x] `SchtasksListV` + `parse_schtasks_list_v` English fixture (next-run, last-run-time, result `1`, Task To Run; skip `N/A`)
- [x] Locale-miss unit (AC2) — keep `parse_last_result_list_v` test green
- [x] `explain_last_task_result` matrix (AC3) — separate `#[test]`s (no rstest dep); decimal + `0x65` / `0x41301`
- [x] Hint is a **following line**; AC4 `Last task result: 101` substring stays
- [x] CSV 3-col next-run fallback still green

## Phase 2 — Red → Green: missing action + format (F5 / F6 / AC5)

- [x] `Last scheduled run:` after result block, before vault last-run
- [x] First quoted `.cmd`/`.bat`/`.exe` token; strip `"`
- [x] Missing path → `Action target missing:` + `nightly --schedule --dry-run`
- [x] Product `'…\ai-brains.exe' nightly` with existing exe → **no** missing line

## Phase 3 — Red → Green: `--quick` (F1 / F17 / F19 / AC6–AC7 / AC13)

- [x] clap `--quick` requires `--status`; conflicts schedule/unschedule (auto exit 2)
- [x] Thread `quick: bool` into `nightly::run`
- [x] `format_endpoint_line(..., "skipped")` — **no** `ProbeStatus::Skipped`
- [x] Still `resolve_nightly_model_endpoints`; no `LlamaCppProvider` / no `probe_health`
- [x] `--quick` without `--status` → exit 2 (clap)

## Phase 4 — Green: parallel status probes (F2 / F7 / AC9)

- [x] `const NIGHTLY_STATUS_PROBE_TIMEOUT: Duration = Duration::from_millis(750);` next to 2s const
- [x] `tokio::join!` (not `try_join!`) completion + embedding on default `--status`
- [x] Pre-summarize path still `NIGHTLY_PROBE_TIMEOUT = 2s`
- [x] Do not edit `llama_cpp_probe_health.rs` AC6 timeouts

## Phase 5 — Wire Windows fetch order (F3)

- [x] LIST /V first (one spawn)
- [x] Non-zero LIST /V → all None; **no** PS
- [x] PS Last Result fallback **only** after successful LIST /V with missing last_result
- [x] CSV next-run fallback
- [x] No CSV col-5 Last Result

## Phase 6 — Docs (F8 / AC11)

- [x] `Docs/OPERATIONS.md` — 1 vs 101 vs Event ID 101; missing wrapper; `--quick`; LIST /V primary
- [x] `Docs/CAPABILITIES.md` — status honesty bullets
- [x] Repo-root `CHANGELOG.md` T247 row only
- [x] Optional F20 help_ia one-liner — **skipped** (no nightly after_help exists)

## Phase 7 — Manual + gate (AC8–AC12)

- [x] `ai-brains nightly --status --quick` — **167 ms**; `probe=skipped`; exit 0
- [x] `ai-brains nightly --status` — probes print; **926 ms** default (completion timeout / embedding ok)
- [x] Live missing `.cmd` named; Last scheduled run 8/13/2026 3:00:01 AM vs vault 2026-08-02
- [x] **Stop-before:** did **not** `nightly --schedule` / write `.cmd`
- [x] Focused nextest + clippy `-p ai-brains-cli` — 48 nightly tests PASS; clippy `-D warnings` PASS
- [x] Review log `review.md` — internal R1 CLEAN×2; Codex CX1 product CLEAN (P1-01 = process closeout)
- [x] Full gate + `ledgerful verify --scope full` (CI PR #157 Win/Linux/macOS green; local fmt/clippy PASS, nextest 2764; deny/audit via CI)
- [x] `ledgerful ledger commit` + pin
- [x] conductor.md → Completed; deferred.md strike T247

---

## Stop-before (hard)

- Destructive git / push to main
- `schtasks /create` `/delete` / `nightly --schedule` / `--unschedule` on the live machine
- Writing `%USERPROFILE%\.ai-brains\nightly-run.cmd`
- Editing `%ProgramData%\AI-Brains\` SYSTEM wrappers
- clap 5 / lockfile pin bumps
- T255 scope (JSON status, doctor ports, embed sleep, Router task)
- Reopening T229 F5 unless a real `ai-brains nightly` process exits 101

---

## Suggested unit names

- `parse_schtasks_list_v__english_fixture__extracts_next_last_result_and_action`
- `parse_schtasks_list_v__missing_english_labels__fields_none`
- `explain_last_task_result__known_codes__expected_hint`
- `format_schedule_status_lines__last_result_101__contains_101` (existing)
- `format_status_action_missing__absent_cmd__next_step_dry_run`
- `format_status_action_missing__product_exe_nightly__no_missing_line`
- `format_endpoint_line__quick__probe_skipped`

---

## Manual evidence template (fill on go)

```
cmd: target\debug\ai-brains.exe nightly --status --quick
elapsed_ms: 167
last_task_result: 1
hint: process failed / missing action / CLI error
action_target_line: Action target missing: C:\Users\RyanB\.ai-brains\nightly-run.cmd
next: ai-brains nightly --schedule --dry-run
probe_lines: Completion/Embedding probe=skipped
exit: 0

cmd: target\debug\ai-brains.exe nightly --status
elapsed_ms: 926
probes: Completion probe=timeout ; Embedding probe=ok
last_scheduled_run: 8/13/2026 3:00:01 AM
last_nightly_run_vault: 2026-08-02T07:03:58.159733500+00:00
exit: 0

cmd: target\debug\ai-brains.exe nightly --quick
exit: 2 (clap requires --status)
```
