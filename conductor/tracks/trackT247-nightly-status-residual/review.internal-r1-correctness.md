# T247 internal R1 — correctness / regression

| Field | Value |
|-------|--------|
| **Track** | T247-NightlyStatusResidual |
| **Reviewer** | Internal correctness (read-only) |
| **Spec** | `conductor/tracks/trackT247-nightly-status-residual/spec.md` |
| **Code** | `crates/ai-brains-cli/src/commands/nightly.rs`, `crates/ai-brains-cli/src/main.rs` |
| **Verdict** | **CLEAN** |
| **Findings** | None |

Production files were not modified. This log is the only write.

---

## Verdict

Implementation matches F1–F7 / F9 / F17–F19 and AC1–AC7 / AC12–AC14 on the assigned surfaces. No unwrap/expect/panic in T247 production paths. T229 F5 truncate, models AC6 probe fixtures, CLI `reqwest` absence, and tokio **1.52.3** / reqwest **0.13.4** pins are untouched. Evaluated edges (quoted Last Result; quoted Task To Run vs first-token extract) are consistent with the spec, not defects.

---

## Focus 1 — `parse_schtasks_list_v`

**Status:** Correct.

- **First colon:** `line.split_once(':')` (`nightly.rs` 808–810). `Next Run Time: 8/14/2026 3:00:00 AM` and `Task To Run: "C:\…"` keep the drive-letter / clock colons in the value.
- **N/A skip:** `list_v_usable_value` (`790–799`) drops empty, `N/A` (case-insensitive), and `"N/A"` via `trim_matches('"')` **only for the skip check**, then returns the original trimmed token. Unit: `parse_schtasks_list_v__missing_english_labels__fields_none` (`1433–1438`).
- **Task To Run quotes kept:** English fixture expects `Some(r#""C:\Users\RyanB\.ai-brains\nightly-run.cmd""#)` (`1414–1417`). Required so `first_quoted_action_target` can see a quoted token (Focus 10).
- **Locale miss:** German-style labels → all four fields `None` (`1421–1431`). PS fallback is not in the unit (AC2).
- Single struct `SchtasksListV` (`782–787`). T229 `parse_last_result_list_v` remains (`895–908`, test `1612–1624`).

---

## Focus 2 — `explain_last_task_result` + AC4

**Status:** Correct.

- Trim; `0x`/`0X` → `u32::from_str_radix(..., 16)`; else `parse::<u32>()`; fail → `None` (`827–833`).
- Closed set: `0` none; `1` fail/missing; `101` panic/abort; `267009` / `0x41301` running + `SCHED_S_TASK_RUNNING`; `267014` / `0x41306` terminated + `SCHED_S_TASK_TERMINATED`; `"99"` / garbage none (`834–841`, tests `1442–1532`).
- Hint is a **following line**, not a suffix: `format_schedule_status_lines` pushes `Last task result: {label}` then optionally `hint` (`748–756`). Order lock: `format_status_schedule_block__order__result_hint_then_last_scheduled` asserts `lines[1] == "Last task result: 101"` and panic text on `lines[2]` (`1535–1549`).
- **AC4 exact:** `format_schedule_status_lines__last_result_101__contains_101` still asserts `Last task result: 101` (`1354–1360`).

---

## Focus 3 — `first_quoted_action_target` + `exists()`

**Status:** Correct.

- First `"` or `'`; matching closer; empty → `None`; only `.cmd` / `.bat` / `.exe` (ASCII-lower) (`847–861`).
- Missing path → `Action target missing:` + `next: ai-brains nightly --schedule --dry-run` (`865–878`, test `1553–1569`).
- Product `"exe" nightly` with existing exe → no lines (`1573–1587`). Function also accepts `'exe' nightly` (same quote matcher). Unquoted path / `.txt` / `None` → no lines (`1591–1594`).
- Status block extends these lines after last-scheduled (`776`). No schtasks `/create`/`/delete` on the status path (F10).

---

## Focus 4 — Fetch order

**Status:** Correct.

`fetch_schedule_snapshot` (`929–946`):

1. `schtasks /query /tn … /fo LIST /v`
2. Non-success (including spawn fail) → `SchtasksListV::default()`, **no** PS
3. Success → `parse_schtasks_list_v`
4. PS `Get-ScheduledTaskInfo` **only if** `last_result.is_none()`
5. CSV next-run **only if** `next_run.is_none()` (`fetch_schedule_next_run`, 3-col helper unchanged)

CSV is never used for Last Result. Old PS-primary `fetch_last_task_result` is gone; only `fetch_last_task_result_ps` remains as the locale fallback.

N/A values become `None` and therefore take the §6.1 “missing after successful LIST /V” PS branch. That matches the design note, not a locale-only special case in the struct.

---

## Focus 5 — `--quick` / F19

**Status:** Correct.

- clap: `requires = "status"`, `conflicts_with_all = ["schedule", "unschedule"]` (`main.rs` 287–289). Tests: missing status → `MissingRequiredArgument` (`88–95`); with `--schedule` → `ArgumentConflict` (`100–113`); `--status --quick` parses (`118–131`). No runtime `if !status && quick`.
- Dispatch threads `*quick` into `nightly::run` (`main.rs` 3356–3382).
- `resolve_nightly_model_endpoints()` always runs on the status path (`nightly.rs` 49–50).
- `if quick { ("skipped", "skipped") }` — string literals, **no** `LlamaCppProvider::new`, **no** `probe_health` (`54–55`). `ProbeStatus` has no `Skipped` variant (`llama_cpp.rs` 39–44).
- AC6: `format_endpoint_line(..., "skipped")` → `probe=skipped` (`1598–1608`).

---

## Focus 6 — `join!` vs `try_join!`; timeouts

**Status:** Correct.

- Status: `tokio::join!` (`57–66`), **not** `try_join!` (no matches in CLI nightly).
- `NIGHTLY_STATUS_PROBE_TIMEOUT = 750ms` (`12`); run-path `NIGHTLY_PROBE_TIMEOUT = 2s` (`10`) still used sequentially at `380–397`.
- Models AC6 fixtures still pass `Duration::from_secs(2)` / `from_millis(500)` (`crates/ai-brains-models/tests/llama_cpp_probe_health.rs`).

---

## Focus 7 — unwrap / expect / panic

**Status:** Correct for production T247 code.

`unwrap(`/`expect(`/`panic!` in `nightly.rs` are confined to `#[cfg(test)]` (`1451+`). Production uses `unwrap_or` / `unwrap_or_else` / `?` only. Status returns `Ok(())` for down / timeout / missing action (`122–123`).

---

## Focus 8 — `format_schedule_status_lines` missing last

**Status:** Correct.

`format_schedule_status_lines__missing_data__unknown_and_not_scheduled` (`1364–1368`): `None`/`None`/`true` → **2** lines, `Scheduled: No…` + exact `Last task result: unknown`. `"unknown"` does not parse as `u32`, so no third hint line.

---

## Focus 9 — Regressions

| Guard | Evidence |
|-------|----------|
| T229 F5 truncate | Still `floor_char_boundary`; no `&content[..4000]` (`crates/ai-brains-brain/src/embeddings.rs` 192–194, 297–307). |
| Models probe AC6 | Unchanged 2s `/health` → `/v1/models` tests; no `ProbeStatus::Skipped`. |
| No CLI reqwest | `crates/ai-brains-cli/Cargo.toml` has tokio, not reqwest; `reqwest` unused under `crates/ai-brains-cli/src`. |
| No lock/tokio bump | `Cargo.lock` tokio **1.52.3** (`6008–6009`); reqwest **0.13.4** (`4591–4592`). |
| CSV 3-col (AC14) | `nightly_status__schedule_state_parse__extracts_next_run_from_csv` + quote-aware split still present (`1310–1381`). |
| Capture / contracts | Status remains a query; no new events/DTOs. Docs + CHANGELOG T247 row present (F8). |

---

## Focus 10 — Quoted last_result vs quoted Task To Run

**Status:** Correct by design; not a defect.

- `list_v_usable_value` **must** return `t` (quotes intact) so live `Task To Run: "C:\Users\RyanB\.ai-brains\nightly-run.cmd"` stays a quoted token. Stripping quotes at parse time would make `first_quoted_action_target` return `None` and hide the missing `.cmd` (F6 / AC10).
- English Last Result is unquoted `1` (fixture `1403`, `1413`; spec live baseline). `explain_last_task_result` therefore sees `1`, not `"1"`.
- Hypothetical `"1"` (quotes stored on last_result) would skip the hint; LIST /V does not quote that field. Not filed.

---

## Findings

None.

---

## Notes (not findings)

- F20 help_ia one-liner is **soft**; not required for this verdict.
- `Path::exists()` is what F6 specifies. Unelevated `exists()` on an ACL-locked SYSTEM `%ProgramData%` wrapper can false-negative; live residual is the user-profile missing `.cmd`, and F10 forbids remediating wrappers from status.
- Manual AC8–AC10 timings / live naming were not re-run in this review (code-only).
