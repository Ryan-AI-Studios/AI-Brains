# T247 — Nightly status residual (latency + Last Result honesty)

- **Track ID:** T247-NightlyStatusResidual
- **Status:** ✅ **Completed** 2026-08-13 PR #157 `43191ff`
- **Category:** OPS / BUGFIX / PERF / UX
- **Owner:** Grok
- **Source:** CLI audit 2026-08-11 P1–P2 — `nightly --status` scored E9/Q8 but **4–6s**; Last Result **101**. Live re-scan 2026-08-13: 101 **cleared**; Last Result is now **1**; vault last-run stale; action target missing.
- **Depends on:** T229 status/probe/Last Result + UTF-8 F5 ✅; T233 multi-root ✅; T239 multi-import status block ✅; T205 global dotenv ✅
- **Blocks / feeds:** Operators can trust `nightly --status` as an interactive ops surface; T255 keeps doctor-port / JSON / embed-sleep softs
- **Absorbs:** deferred.md “Nightly status latency + Last Result 101”; placeholder F1–F3 (quick / parallel-or-short-timeout / live 101 disposition); T229 docs residual that Last Result stays 101 until next successful run
- **Not absorbed (DoD):** T255 F8 doctor model-port matrix; F9 persist probe in sync_state; F10 `nightly --schedule` registers Router; F11 Router ONLOGON codes; F12 JSON nightly status; F14 50ms embed sleep; clap 5; MSI; auto-reschedule of the live task; shipping a new `nightly-run.cmd` product
- **Research date:** 2026-08-13 (live dogfood + T229 code truth + official llama.cpp `/health` + schtasks / Task Scheduler codes + crate pins)
- **AI fold-in:** 2026-08-13 `C:\dev\AI-review.md` **T247** AI1 + AI2. No Highs. **Agree hard:** AI2 M1 string-literal `probe=skipped` (no `ProbeStatus::Skipped`); AI2 M2 `join!` + Windows 750ms Timeout. Disposition **§14**.
- **Ledger:** plan-only until go (`ledgerful ledger start T247-nightly-status-residual --category FEATURE`)
- **Isolation:** Do **not** rewrite T229 F5 truncate or T239 multi-import. Do **not** mutate the live `AI-Brains-Nightly` task or write `%USERPROFILE%\.ai-brains\nightly-run.cmd` until the user says go **and** confirms the remediating action.

---

## 1. Objective

1. **Make status fast enough to use.** Default `nightly --status` must not pay sequential 2s+2s HTTP when the router is down. Target **&lt;1.5s** default even when both probes fail; **&lt;1s** with `--quick` (no HTTP).
2. **Tell the truth about Last Result.** Distinguish **Rust panic 101** from **process exit 1** from **Task Scheduler Event ID 101**. Print scheduler last-run time separately from the vault `last_nightly_run`.
3. **Surface the live failure class on this machine:** the scheduled action points at a **missing** `nightly-run.cmd`. Status must name that path and give a copy-paste next step. Do **not** auto-replace the task.
4. **Keep T229 contracts:** status exit **0** when probes are down; no CLI `reqwest`; no CSV col-5 Last Result myth; pre-summarize probe timeout stays **2s**.
5. **Capture independence.** Status/docs/parse only. No new events, no contracts DTO, no new crates, no pin bumps.

---

## 2. Live baseline (re-scan 2026-08-13)

### 2.1 Operator dogfood (this machine)

| Signal | Observation |
|--------|-------------|
| `ai-brains nightly --status` | **1068 ms** with router **up** (Completion/Embedding `probe=ok`) |
| Scheduled | Yes; next **8/14/2026 3:00:00 AM**; Ready |
| **Last task result** | **1** (not 101) |
| Task Scheduler `LastRunTime` | **8/13/2026 3:00:01 AM** (task **did** fire this morning) |
| Vault `Last nightly run` | **2026-08-02T07:03:58Z** (stale vs scheduler) |
| Sessions summarized / errors | `0` / `[]` |
| Multi-import | `never` |
| Task To Run | `"C:\Users\RyanB\.ai-brains\nightly-run.cmd"` |
| **`nightly-run.cmd` exists** | **False** |
| Start In | N/A |
| Logon | Interactive only; Run As `RyanB` (not SYSTEM) |
| `nightly-run.log` | Missing |
| `AI-Brains-Router` LastTaskResult | **267009** (`SCHED_S_TASK_RUNNING`) — **out of scope** (T255 F11 / ops) |
| Product `nightly --schedule` (user) | Registers `'\<exe\>' nightly` — **not** the missing `.cmd` |

### 2.2 Why the audit said 4–6s (still true when router is down)

| Cost | Measured / code |
|------|-----------------|
| Completion probe | Sequential `probe_health(2s)` — worst case **2000 ms** |
| Embedding probe | Sequential second `probe_health(2s)` — worst case **+2000 ms** |
| `powershell -NoProfile -Command Get-ScheduledTaskInfo` | **401 ms** cold |
| `schtasks /FO CSV` next-run | **30 ms** |
| `schtasks /FO LIST /V` | **33 ms** (has Next Run + Last Run + Last Result + Task To Run) |
| Loopback `/health` when up | **6–57 ms** |
| Vault open + print | Remainder of the 1068 ms happy path |

Audit 4–6s = **2s + 2s sequential timeouts** + PowerShell + vault. Happy-path 1.1s is already near the interactive bar; the residual that still hurts is **router-down** plus the **401 ms PS spawn**.

### 2.3 Why Last Result is 1 (not 101)

T229 F5 fixed the UTF-8 `content[..4000]` panic that produced **exit 101**. That residual is **cleared** on this machine.

Live **1** is the process/scheduler failure class: Task Scheduler launched a **missing** action (`nightly-run.cmd`). Windows documents `0x1` / `ERROR_INVALID_FUNCTION` as “Incorrect function”; operators see it when the action path is wrong, Start In is unset, or the process exits 1. The vault never advanced because `ai-brains nightly` never ran.

**Do not** tell the operator to “wait for the next schedule to clear 101.” The next schedule will fire the same missing `.cmd` and stay at **1**.

### 2.4 Code truth (T229 as shipped)

| Site | Role |
|------|------|
| `nightly.rs` `run` status branch | Sequential 2s probes **before** print; then CSV next-run + PS Last Result |
| `NIGHTLY_PROBE_TIMEOUT` | `Duration::from_secs(2)` — used for **status and** pre-summarize |
| `fetch_last_task_result` | Primary: spawn `powershell` `Get-ScheduledTaskInfo`; fallback English LIST /V |
| `fetch_schedule_next_run` | Separate `schtasks /FO CSV` spawn |
| `format_schedule_status_lines` | `Scheduled:` + `Last task result:` only — **no** scheduler last-run time, **no** action path |
| `LlamaCppProvider::probe_health` | `GET /health` → 404 → `GET /v1/models`; 2s connect+request timeout; **no CLI reqwest** |
| clap `Nightly` | `--status` exists; **no** `--quick` |
| Product schedule (non-SYSTEM) | `'\<exe\>' nightly` |
| SYSTEM schedule | `%ProgramData%\AI-Brains\` wrapper (T145) — do not retouch |

### 2.5 Event ID 101 ≠ Last Result 101

| Token | Meaning |
|-------|---------|
| **Last Result 101** | Child process **exit code 101**. Rust’s default panic/abort path (and rustc ICE) uses **101**. This was the T229 F5 panic. |
| **Last Result 1** | Child **exit 1** or scheduler “Incorrect function” — missing/bad action, CLI `fail_api`, `.cmd` not found. **Live residual.** |
| **Task Scheduler Event ID 101** | Operational log: “task **failed to start**” (permissions / principal). **Different namespace.** Do not document Last Result 101 as Event ID 101. |

---

## 3. Research (2026-08-13)

| Topic | Finding | Use in T247 |
|-------|---------|-------------|
| **llama.cpp server** | Official `GET /health` is public, does **not** reset idle timer; fallback `GET /models` / `/v1/models` still valid ([server README](https://github.com/ggml-org/llama.cpp/blob/master/tools/server/README.md), fetched 2026-08-13) | Keep T229 probe sequence; do not invent new paths |
| **schtasks `/FO LIST /V`** | Official verbose list includes Last Run Time, Last Result, Next Run Time, Task To Run ([schtasks query](https://learn.microsoft.com/en-us/windows-server/administration/windows-commands/schtasks-query)) | **F3** one-spawn primary |
| **schtasks `/FO CSV`** | Still **3 columns** on this Windows (TaskName, Next Run Time, Status) | Next-run **fallback only**; never Last Result |
| **Get-ScheduledTaskInfo** | Authoritative numeric `LastTaskResult`; **401 ms** PowerShell cold start | **Fallback** if LIST /V locale parse fails |
| **Last Result 0x1** | Win32 `ERROR_INVALID_FUNCTION` / process exit 1; common for missing action or unset Start In | **F4/F6** live class |
| **Last Result 101** | Process exit 101 = Rust panic/abort — **not** Event ID 101 | **F4** decode; T229 F5 already shipped |
| **0x41301 = 267009** | `SCHED_S_TASK_RUNNING` | Decode table only; Router out of scope |
| **0x41306 = 267014** | `SCHED_S_TASK_TERMINATED` | Decode table only (T229 F11 / T255) |
| **clap** | workspace **4.5** / lock **4.6.1** / crates.io **4.6.6** (5.x not released) | **No bump**; clap 5 forbidden when it ships |
| **reqwest** | workspace **0.13** / lock **0.13.4** (latest 0.13 train) | **No bump**; still models-only |
| **tokio** | workspace **1.52** / lock **1.52.3** / crates.io **1.53.1** | `join!` available; **do not** `cargo update -p tokio` |
| **serde_json** | lock **1.0.150** | **No bump**; no new DTO |
| **T205** | Global dotenv already merged before subcommands | Status still sees `:8081`/`:8083` |

---

## 4. Findings (DoD)

| ID | Severity | Requirement |
|----|----------|-------------|
| **F1** | Hard | clap `--quick` **requires** `--status`, conflicts with `--schedule` / `--unschedule`. Skips HTTP probes. Pass the string literal **`"skipped"`** to `format_endpoint_line` — **no** `ProbeStatus::Skipped` / no `ai-brains-models` change (AI2 **M1**). Still opens the vault (last-run / unsummarized / multi-import). Still calls `resolve_nightly_model_endpoints` so host:port + model print. |
| **F2** | Hard | Default status probes **in parallel** via **`tokio::join!` (not `try_join!`)** with **`NIGHTLY_STATUS_PROBE_TIMEOUT = Duration::from_millis(750)`** next to the existing 2s const. Probes return `ProbeStatus`, not `Result` — both must complete for display. Pre-summarize (nightly **run**) keeps `NIGHTLY_PROBE_TIMEOUT = 2s`. Do not change models AC6 2s fixtures. Windows closed loopback may wait the **full 750 ms** as `Timeout` (T229 AC6) — default-down ≈750 ms, still &lt;1.5s (AI2 **M2**). |
| **F3** | Hard | New **`parse_schtasks_list_v(stdout) -> SchtasksListV`** `{ next_run, last_run_time, last_result, task_to_run }` — one struct, not four helpers. Existing `parse_last_result_list_v` **stays** (PS-miss / LIST fallback path; its unit stays green). CSV next-run = fallback. **PS `Get-ScheduledTaskInfo` = Last Result fallback only when LIST /V succeeded but last_result parse missed (locale).** LIST /V **non-zero exit** (task missing) → all `None` → `Scheduled: No` — **no** PS fallback. Never CSV col 5. Non-Windows: omit scheduler lines (T229). |
| **F4** | Hard | Pure `explain_last_task_result(&str) -> Option<&'static str>`. Parse: trim; if starts with `0x`/`0X` → `u32::from_str_radix(rest, 16)`; else `s.parse::<u32>()`; parse fail → `None`. Known: `0` → none; `1` → process failed / missing action / CLI error; `101` → Rust panic/abort; `267009` (`0x41301`) → `task still running (SCHED_S_TASK_RUNNING)`; `267014` (`0x41306`) → `task terminated (SCHED_S_TASK_TERMINATED)` — those two are **success-class**, not errors. Decode is a **separate line after** `Last task result: N`, not a suffix on the same line. **AC4** substring `Last task result: 101` **must** remain. |
| **F5** | Hard | Print **`Last scheduled run:`** from Task Scheduler separately from vault **`Last nightly run:`**. Order: `Scheduled:` → `Last task result:` → hint line? → **`Last scheduled run:`** → **`Last nightly run:`** → unsummarized → endpoints. When both present and they disagree, keep both. Do not overwrite vault timestamps. |
| **F6** | Hard | Parse the **first quoted token** from Task To Run; strip surrounding `"`. Only candidates ending in `.cmd` / `.bat` / `.exe`. If that path `exists()` is false → `Action target missing: <path>` + `next: ai-brains nightly --schedule --dry-run`. Product `'…\ai-brains.exe' nightly` (quoted exe + args): check the exe token only — if it exists, **no** missing line. Unparseable / no quoted token → no missing line. `--schedule --dry-run` is **non-mutating** (safe under F10). **Do not** unschedule/reschedule/write the `.cmd` from status. |
| **F7** | Hard | Latency targets on this class of machine: `--quick` **&lt;1s**; default **&lt;1.5s** with both probes failing (parallel 750ms, not 4s). Record live timings in plan.md on go. Status **exit 0** for down / timeout / missing action / nonzero Last Result. |
| **F8** | Hard docs | OPERATIONS: Last Result **1 vs 101 vs Event ID 101**; missing-wrapper next step; `--quick`; LIST /V primary. CAPABILITIES T229 honesty bullets updated. Repo-root `CHANGELOG.md` T247 row only. |
| **F9** | Hard | Zero new crates; **no version pin bumps** (do **not** `cargo update -p tokio` — lock stays **1.52.3** even though crates.io has 1.53.1); **no CLI reqwest**; **no contracts DTO**; capture-independent (no events). clap 5 is **not released** (max 4.6.6); “forbidden” is a future-bump guard. |
| **F10** | Hard stop-before | Do not mutate `AI-Brains-Nightly`, do not create `nightly-run.cmd`, do not retouch SYSTEM ProgramData wrappers, unless the user explicitly confirms that remediating action after go. |
| **F11** | Soft residual | JSON `--format json` nightly status → **T255 F2** |
| **F12** | Soft residual | Doctor model-port matrix → **T255 F1** |
| **F13** | Soft residual | Persist probe in sync_state → **T255 F9** |
| **F14** | Soft residual | 50ms embed sleep → **T255 F14** (run latency, not status) |
| **F15** | Soft residual | Router ONLOGON 267009/267014 → ops / T255 F11 |
| **F16** | Soft residual | Product-owned user wrapper that logs to `nightly-run.log` (T229 F10 class) |
| **F17** | Hard clap | `--quick` without `--status` → clap usage **exit 2**. `requires = "status"` generates this automatically — **no** manual `if !status && quick` in `nightly::run`. |
| **F18** | Hard verify-only | T229 F5 `truncate_for_embed` stays. Re-open only if a **real** nightly process (not the missing `.cmd`) exits 101. |
| **F19** | Hard | `--quick` must not construct `LlamaCppProvider` / must not call `probe_health`. **Does** call `resolve_nightly_model_endpoints` (endpoint lines still need URL + model). Pass `"skipped"` as `probe_label`. |
| **F20** | Soft | help_ia after_long_help one-liner for `--quick` — add only if a nightly example line already exists or is one additive Operator bullet; do not restack T204. |

---

## 5. Acceptance criteria

| AC | Proof |
|----|-------|
| **AC1** | Unit: `parse_schtasks_list_v` English fixture extracts next-run, last-run-time, last-result `"1"`, Task To Run path |
| **AC2** | Unit: locale-garbled LIST /V (no `Last Result:` / `Last Run Time:`) → those fields `None` (PS fallback is integration; unit only proves parse miss) |
| **AC3** | Unit: `explain_last_task_result` — `0` no hint; `1` mentions fail/missing; `101` mentions panic; `0x65` ≡ 101; `267009` running + `SCHED_S_TASK_RUNNING`; `267014` terminated; unknown `"99"` none |
| **AC4** | Existing `format_schedule_status_lines__last_result_101__contains_101` still passes; the **result line** still contains `Last task result: 101` (hint is a **following** line) |
| **AC5** | Unit: missing `.cmd` path → status lines contain `Action target missing:` + `nightly --schedule --dry-run`; product `'exe' nightly` with existing exe → **no** missing line |
| **AC6** | Unit: `format_endpoint_line(..., "skipped")` contains `probe=skipped` — string literal, not a `ProbeStatus` variant |
| **AC7** | Unit or compile-visible: `--quick` requires `--status` (clap `requires`); conflicts schedule/unschedule |
| **AC8** | Manual: `nightly --status --quick` **&lt;1s**, `probe=skipped`, still shows Last task result + scheduled + vault last-run, exit 0 |
| **AC9** | Manual: `nightly --status` (default) still shows Completion/Embedding probe ok/down; **&lt;1.5s** even if router stopped (or document if probe classify is Down &lt;750ms) |
| **AC10** | Manual: live status names **missing** `nightly-run.cmd` and prints `Last scheduled run:` **8/13…** (or whatever Task Scheduler now has) **and** vault `Last nightly run:` 2026-08-02 (until a real nightly writes) |
| **AC11** | Docs: OPERATIONS 1 vs 101 vs Event ID 101; CAPABILITIES; root CHANGELOG T247 |
| **AC12** | No contracts DTO; no pin bumps; no new CLI dep; T229 models `probe_health` AC6 unchanged; full gate green |
| **AC13** | `--quick` without `--status` → exit **2**, no vault work required |
| **AC14** | CSV 3-col next-run fallback unit still green (T229 F6b) |

---

## 6. Design notes

### 6.1 Fetch order (Windows)

```text
schtasks /query /tn AI-Brains-Nightly /fo LIST /v
  → non-zero exit → SchtasksListV all None (Scheduled: No). Stop. No PS.
  → parse_schtasks_list_v (English labels)
  → if last_result missing after a successful LIST /V: powershell Get-ScheduledTaskInfo
  → if next_run missing: schtasks /fo CSV (existing helper)
```

One LIST /V spawn covers the common English machine (this dogfood). Locale-sensitive labels stay a **soft miss → fallback**, not a hard English-only DoD.

### 6.2 Status probe vs run probe

| Caller | Timeout | Parallel? |
|--------|---------|-----------|
| `nightly --status` (default) | **750 ms** | **Yes** |
| `nightly --status --quick` | none | n/a |
| Nightly **run** pre-summarize | **2 s** (T229 F2) | unchanged (sequential OK; not interactive) |

### 6.3 Last Result decode (closed set)

Do **not** ship a 200-row Win32 table. Four operator-relevant codes + unknown. Hint is a **separate line** so AC4’s `Last task result: 101` substring stays exact.

### 6.4 Remediation honesty (no mutate)

Product user-principal schedule is `ai-brains.exe nightly` (probes live in Rust since T229). That is the documented next step **after dry-run**. Recreating the historical ops `.cmd` is **F16 soft** and **F10 stop-before**.

### 6.5 Capture independence

No `MemoryPinned`, no nightly events, no probe persistence (F13/T255). Status is a query.

### 6.6 clap

`--quick` is a flag on `Commands::Nightly`, `requires = "status"`, `conflicts_with_all = ["schedule", "unschedule"]`. Thread a `quick: bool` into `nightly::run`. `--quick` still resolves endpoint URLs; it only skips `LlamaCppProvider::new` + `probe_health` and passes `"skipped"`.

---

## 7. Non-goals

- Shipping or starting `router.bat` / llama-server
- Registering `AI-Brains-Router` from `nightly --schedule` (T255 F10)
- Doctor model-port matrix (T255 F1)
- JSON nightly status contract (T255 F12)
- Embed inter-call sleep (T255 F14)
- Changing SYSTEM `--skip-import` or ProgramData ACL wrappers
- Auto-unschedule / auto-reschedule / writing `nightly-run.cmd`
- clap 5 / forced dep bumps
- Multi-root / T233 reopen
- Treating Event ID 101 as Last Result 101

---

## 8. Verification plan

```powershell
# Red → green
cargo nextest run -p ai-brains-cli --lib nightly
cargo clippy -p ai-brains-cli --all-targets -- -D warnings

# Manual (do not reschedule)
ai-brains nightly --status --quick
ai-brains nightly --status
# Optional: stop router, confirm default still <1.5s and probe=down, exit 0

# Full gate
cargo fmt --check ; cargo clippy --workspace --all-targets -- -D warnings
cargo nextest run --workspace ; cargo deny check ; cargo audit
ledgerful verify --scope full
```

---

## 9. Risk register

| Risk | Mitigation |
|------|------------|
| LIST /V locale | Parse miss after **successful** LIST /V → PS fallback; unit AC2 |
| LIST /V task missing | Non-zero exit → all None; **no** PS |
| 750 ms too short on slow loopback | Localhost health is &lt;60 ms here; timeout still maps to `timeout`; run-path stays 2s |
| Windows closed loopback waits **full 750 ms** as Timeout | Documented T229 AC6; default-down ≈750 ms still &lt;1.5s (F7) |
| Operator thinks status will fix the task | F6 next-step is `--dry-run` only; F10 stop-before |
| AC4 broken by decode suffix | Hint is a **following line**, not a suffix |
| Parallel probes change classify | Same `probe_health`; `join!` not `try_join!` |
| `ProbeStatus::Skipped` creep | Forbidden — string literal `"skipped"` only (F1) |
| Confuse Event ID 101 | F8 docs table |
| Another agent owns live schedule | Isolation: no schtasks `/create`/`/delete` in this track unless user confirms |
| Accidental tokio 1.53 bump | F9: do not `cargo update -p tokio` |

---

## 10. Implement order (on go)

1. Red→Green F3/F4/F5 parse + decode + format units  
2. Red→Green F6 missing-action lines  
3. Red→Green F1/F17/F19 `--quick` clap + skip probe  
4. Green F2 parallel 750 ms status probes  
5. Wire fetch order (LIST /V first)  
6. Docs F8  
7. Manual AC8–AC10 (no live mutate) → review → full gate → Complete  

---

## 11. Soft residuals (post-close → T255 / ops)

| Residual | Note |
|----------|------|
| F11 JSON status | T255 F2 |
| F12 doctor ports | T255 F1 |
| F13 persist probe | T255 F9 |
| F14 embed 50ms | T255 F14 |
| F15 Router 267009 | ops / T255 F11 |
| F16 user `.cmd` wrapper | T229 F10 class |
| Live reschedule | Operator confirm after go — not automatic Close |

---

## 12. Touch map (expected)

| Site | Change |
|------|--------|
| `crates/ai-brains-cli/src/commands/nightly.rs` | parse LIST /V; decode; format; `--quick`; parallel probes; missing action |
| `crates/ai-brains-cli/src/main.rs` | clap `--quick` |
| `Docs/OPERATIONS.md` | 1 vs 101 vs Event ID 101; `--quick`; LIST /V |
| `Docs/CAPABILITIES.md` | status honesty bullets |
| `CHANGELOG.md` (repo root) | T247 row |
| `ai-brains-models` | **No** — caller already passes `Duration`; **no** `ProbeStatus::Skipped` |
| contracts / events / store | **None** |

---

## 14. AI fold-in disposition (2026-08-13)

Source: `C:\dev\AI-review.md` — **AI1** (API sketches) + **AI2** (code-truth pins). No Highs. No blockers.

### AI1

| ID | Verdict | Action |
|----|---------|--------|
| **M1** `SchtasksListV` + `parse_schtasks_list_v` | **Agree** | Pin F3 struct API; N/A vs `"N/A"` skip |
| **M2** missing-action + last-run line | **Agree, refined** | F6 first-quoted-token (AI2 L5/L6); do not treat `'exe' nightly` as one path |
| **M3** `join!` + 750 ms | **Agree, refined** | F2; **decline** AI1 `ProbeOutcome::Skipped` type — use string literal (AI2 M1) |
| **M4** clap `--quick` + decode match | **Agree, refined** | F1/F4/F17; hex via `from_str_radix` not only string match (AI2 L2) |
| **L1** Event ID 101 ≠ Last Result 101 | **Agree** | Already F8 |
| **L2** OPERATIONS / CAPABILITIES / CHANGELOG | **Agree** | Already F8 / Phase 6 |
| **O1** unit names | **Agree** | Plan Phase 1 names |
| `--quick` typically ~50 ms | **Decline** | Vault still opens (AI2 O12); keep **&lt;1s** target, not 50 ms |
| `/v1/health` extra probe | **Decline** | T229 `/health` → `/v1/models` stays |

### AI2

| ID | Verdict | Action |
|----|---------|--------|
| **M1** `probe=skipped` = string literal | **Agree hard** | F1 / F19 / AC6 / §12 |
| **M2** `join!` not `try_join!`; Win Timeout 750 ms | **Agree hard** | F2 / §9 |
| **L1** struct parser; keep `parse_last_result_list_v` | **Agree** | F3 |
| **L2** hex parse | **Agree** | F4 |
| **L3** 267009/267014 success-class | **Agree** | F4 hints |
| **L4** line order | **Agree** | F5 |
| **L5/L6** quote strip + first token + product exe | **Agree** | F6 / AC5 |
| **L7** `--dry-run` safe under F10 | **Agree** | F6/F10 |
| **L8** clap `requires` auto exit 2 | **Agree** | F17 |
| **L9** still `resolve_nightly_model_endpoints` | **Agree** | F19 |
| **L10** const next to 2s timeout | **Agree** | F2 |
| **L11** decode = following line | **Agree hard** | F4 / AC4 |
| **L12** LIST /V non-zero → no PS | **Agree** | F3 / §6.1 |
| **L13** clap 5 not released | **Agree** | F9 / §3 |
| **L14** T255 boundary | **Affirm** | F11–F16 unchanged |
| **L15** no `cargo update -p tokio` | **Agree** | F9 |
| **O10/O11** F3/F1 value | **Affirm** | already DoD |
| **O12** `--quick --no-vault` | **Decline as DoD** | Soft residual → T255 class; `--quick` still opens vault |

### Pins locked by fold-in

1. **F1/F19:** `"skipped"` string to `format_endpoint_line`; no models enum change; still resolve endpoints + open vault.  
2. **F2:** `tokio::join!`; 750 ms const beside 2s; Windows closed-loopback may be full 750 ms Timeout.  
3. **F3:** `SchtasksListV` struct; LIST /V fail-closed on non-zero; PS only after successful LIST /V with missing last_result.  
4. **F4:** hex/decimal `u32`; hint on the **next** line; SCHED_S codes are not errors.  
5. **F5:** `Last scheduled run:` after result block, before vault last-run.  
6. **F6:** first quoted `.cmd`/`.bat`/`.exe` token; product `'exe' nightly` is not missing if exe exists.

---

**Completed 2026-08-13.** Soft residuals F11–F16 → T255.
