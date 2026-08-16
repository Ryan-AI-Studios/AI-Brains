# T255 — Nightly / router soft residuals (T229+ / T247+)

- **Track ID:** T255-NightlyRouterSoftResiduals
- **Status:** ✅ **Completed** (2026-08-16; FEATURE TX `646bd578`)
- **Category:** OPS / POLISH / UX
- **Owner:** Grok
- **Source:** T229 soft F8–F12/F14 + T247 soft F11–F16 / O12. Series closer for T240–T255.
- **Depends on:** T229 ✅ PR #140 `1ec9142`; T247 ✅ PR #157 `43191ff`; T248/T249 format SOOT ✅
- **Blocks / feeds:** Operators can script `nightly --status` and see the Router task without mutating it. Closes the T240–T255 placeholder series.
- **Absorbs:** T229 F12 / T247 F11 JSON status; T229 F11 / T247 F15 **read-only** Router Last Result line
- **Not absorbed (DoD):** T229 F8 / T247 F12 doctor 16th model-port check; T229 F9 / T247 F13 persist probe in `sync_state`; T229 F10 / T247 F16 product `nightly-run.cmd` / register Router from `--schedule`; T229 F14 / T247 F14 50ms embed sleep; T247 O12 `--quick --no-vault`; T253 F34 Claude/Codex nightly; T167; T240 F13/F14; clap 5 / pin bumps; contracts DTO; daemon; live task mutate
- **Research date:** 2026-08-15 (source HEAD `1f7b014` at plan; fold-in against `12cb898`)
- **AI fold-in:** 2026-08-16 `C:\dev\AI-review.md` **T255** AI1 + AI2. No Highs. **Agree hard:** AI1 BS1 AC3/§5.1 key reconcile; AI1 BS2 thread raw `Option` sync_state (do not coalesce `"0"`/`"[]"` before the builder); AI1 BS3+O1 snapshot `found` flag + Router `scheduled` ≠ `next_run`; AI1 BS5/AC9 host-tolerant scheduler keys; AI1 BS7 Nightly `after_help` required. **Agree:** AI1 BS6 blank-status Router unit (AC15); AI1 O2 same host/model/probe tuple; AI1 O3 MultiImport parse-or-warn idiom for F20; AI1 O4 clap template + CHANGELOG default-human; AI1 O5 additive docs not a new section; AI1 O6 non-Windows JSON unit (AC16). **Decline:** AI1 BS4 “make `first_quoted_action_target` pub(crate)” — already `pub(crate)` at `nightly.rs:872`; pass precomputed action fields into the builder instead. AI2 “1968 lines” — file is **1819** lines. AI2 JSON example omitting `errors_last_run_unreadable` / `router.task_to_run` — keep both (BS1). Disposition **§12**.
- **Ledger:** planning DOCS TX `5d3d182d-f689-4673-9a03-733f5a178f3c`. Fold-in DOCS TX `6a1380e9-3c04-4d0f-b8d9-69f2dcb1a265`. Implement starts a FEATURE TX on **go**.
- **Isolation:** Do **not** reopen T229 F5 truncate, T247 `--quick` / 750 ms / LIST /V / missing-action, T239 multi-import, T253 Claude/Codex nightly. Do **not** mutate `AI-Brains-Nightly` or `AI-Brains-Router`, do **not** write `%USERPROFILE%\.ai-brains\nightly-run.cmd`, do **not** print or commit `AI_BRAINS_KEY`.

---

## 1. Objective

1. **Scripts can read nightly status.** `nightly --status --format json` emits a frozen CLI-local JSON object. Human chrome stays the T229/T247 contract unless `--format auto` is requested.
2. **Operators can see the Router task.** `nightly --status` prints a read-only `Router:` line (Last Result + T247 decode). It does **not** register, start, or repair `AI-Brains-Router`.
3. **Keep T229/T247 contracts.** Status exit **0** when probes are down / timeout / missing action / nonzero Last Result. `--quick` still skips HTTP with the string literal `probe=skipped`. Run-path probe stays **2s**. No CLI `reqwest`. No CSV col-5 Last Result.
4. **Decline the rest of the parked bag with evidence.** Doctor stays the frozen **15**-check matrix. Status stays a query (no `sync_state` probe write). 50ms embed sleep stays. Product wrapper / `--schedule` Router registration stay ops.
5. **Capture independence.** Status/docs/parse only. No new events, no contracts DTO, no new crates, no pin bumps.

---

## 2. Live baseline (re-scan 2026-08-15)

### 2.1 Operator dogfood (this machine)

| Signal | Observation |
|--------|-------------|
| HEAD | `1f7b014` — T254 Completed. T255 still Placeholder until this plan. Ahead of `origin/main` by **10**. Tree CLEAN at plan start. |
| PATH `ai-brains` | **Pre-T247.** `nightly --status --quick` → unexpected argument. Default status **2807 ms**; Completion `probe=timeout`; Embedding `probe=ok`; Last task result **1**; **no** Last scheduled run / missing-action lines. **PATH-behind — out of T255 scope** (same class as T253 live `install_ready=false`). |
| Source `cargo run -p ai-brains-cli -- nightly --status --quick` | T247 surface: Last task result **1** + hint; Last scheduled run **8/15/2026 3:00:01 AM**; `Action target missing: C:\Users\RyanB\.ai-brains\nightly-run.cmd`; `next: ai-brains nightly --schedule --dry-run`; vault Last nightly run **2026-08-02T07:03:58Z**; Multi-import **never**; `probe=skipped`. |
| `AI-Brains-Nightly` | Ready; next **8/16/2026 3:00:00 AM**; Last Result **1**; Task To Run `"C:\Users\RyanB\.ai-brains\nightly-run.cmd"`; file **missing**; log **missing**; Interactive / Run As `RyanB`. |
| `AI-Brains-Router` | **Running**; ONLOGON; Last Result **267009** (`SCHED_S_TASK_RUNNING`); Task To Run **unquoted** `C:\llm\router.bat`; Start In N/A. |
| Completion :8081 | Unreachable (ledgerful doctor + PATH status `timeout`). |
| Embedding :8083 | **ok** (nomic). |
| `nightly.rs` `--format` | **None.** Status always prints `=== Nightly Status ===` human, including pipes. |
| Doctor checks | Frozen **15** (`health_check_order_names__fixed_matrix` + `report.checks.len() == 15`). No model-port check. |
| Embed sleep | `embeddings.rs` `tokio::time::sleep(50ms)` after each backfill/stale item. Nightly has **not run** since 2026-08-02 (missing `.cmd`) — **no live latency evidence**. |
| Last GitHub PR | [#168](https://github.com/Ryan-AI-Studios/AI-Brains/pull/168) T251 closeout (2026-08-15). **No Cursor / review comments** on #167 or #168. T252–T254 are local-only (not pushed). |
| Identity / doctor ambient | Scope `test-alias`; Claude/Codex PATH `(pending)`; ledgerful doctor legacy `.changeguard` / sig-pin / timings. Do not “fix” here. |

### 2.2 Why these residuals still matter

| Residual | Why it is still a product hole / why decline |
|----------|----------------------------------------------|
| JSON status (T229 F12 / T247 F11) | Every peer status surface (scope / retention / device / list-paths) has `--format json`. Nightly is the last operator ops report that scripts cannot parse. **DoD.** |
| Router ONLOGON 267009/267014 (T229 F11 / T247 F15) | Product status only queries `AI-Brains-Nightly`. Live Router is **Running / 267009** and is why :8083 is up. T247 already decodes those codes. A second LIST /V (~30 ms) is cheap. **DoD as read-only display.** |
| Doctor model ports (T229 F8 / T247 F12) | Would be check **16**, contracts `DoctorReport`, T192/T249 AC freeze, HTTP on the vault skim, and a duplicate of `nightly --status`. **Decline.** Status (human + JSON) **is** the model-port matrix. |
| Persist probe (T229 F9 / T247 F13) | `--status` is a query. Writing probe results into `sync_state` mixes read/write and is not capture-independent telemetry we want in the vault. **Decline.** |
| Register Router / product `.cmd` (T229 F10 / T247 F16) | T247 F10 stop-before: do not mutate live tasks or write `nightly-run.cmd`. Ops script remains. **Decline.** |
| 50ms embed sleep (T229 F14 / T247 F14) | 50 backfill × 50 ms = **2.5 s** of yield vs seconds of model inference. No live nightly since 2026-08-02. T229: retune only with evidence. **Decline.** |
| `--quick --no-vault` (T247 O12) | `--quick` is already &lt;1s in source. Vault last-run / unsummarized / multi-import are why operators run status. **Decline as DoD.** |

### 2.3 Code truth

| Item | Location | Notes |
|------|----------|-------|
| Status branch | `nightly.rs` `run` `if status` | Human `println!` only. Vault open first. Parallel 750 ms probes unless `quick`. |
| Timeouts | `NIGHTLY_PROBE_TIMEOUT` 2s; `NIGHTLY_STATUS_PROBE_TIMEOUT` 750 ms | Do not change. |
| Endpoints | `DEFAULT_MODEL_URL` `:8081`; `DEFAULT_EMBEDDING_URL` `:8083` | `resolve_nightly_model_endpoints` |
| Format helpers | `format_endpoint_line` / `format_schedule_status_lines` / `format_status_schedule_block` | Human SOOT. Keep. |
| LIST /V | `SchtasksListV` + `parse_schtasks_list_v` + `fetch_schedule_snapshot(task_name)` | Already parameterized by task name. **Reuse** for Router. No `status` / `found` today. Miss and all-N/A both return `SchtasksListV::default()` (`:970`) — **F34** must distinguish. English fixture already has `Status: Ready` (AC5 additive). |
| Decode | `explain_last_task_result` | 0 / 1 / 101 / 267009 / 267014 — reuse, do not fork. |
| `--quick` | clap `requires = "status"` | `probe=skipped` string literal (T247 F1/F19). Keep. |
| Multi-import | `multi_import.rs` `MultiImportStatusView` / `MultiImportReport` | never / unreadable / Report v1. Reuse in JSON. |
| Embed sleep | `ai-brains-brain/src/embeddings.rs` 129 / 173 | 50 ms. Do not touch. |
| Doctor matrix | `doctor.rs` 15 checks | `health_check_order_names__fixed_matrix` locks names + len 15. |
| Format SOOT | `retention.rs` / `scope.rs` `resolve_*_format` | T248/T249 local helpers; T249 pin: **do not** share. |
| clap format | T248/T249 `value_parser` + default; T249 AC16 case-sensitive | Copy parser set; **default `human`** (see F9). |
| `is-terminal` | crate `is-terminal` (T248/T249) | Reuse. Do **not** migrate to `std::io::IsTerminal` (T249 F12 soft). |
| Contracts / daemon | none | No nightly-status DTO. |
| Hotspot | `nightly.rs` **1819 lines**, **not** top-10 | New JSON/router helpers go in **`nightly_status.rs`**. Do not relocate T229/T247 units. |
| `project.rs` | hotspot **#1** (3.640) | Do not touch. |

### 2.4 Dependency / standards research (2026-08-15)

| Pin | Workspace / lock | Action |
|-----|------------------|--------|
| `clap` | workspace **4.5** / lock **4.6.1** | **No bump.** Add `--format` on `Nightly` only. |
| `serde_json` | lock **1.0.150** | **No bump.** CLI-local struct. |
| `tokio` | workspace **1.52** / lock **1.52.3** | **No bump.** Do not `cargo update -p tokio`. |
| `reqwest` | workspace **0.13** / lock **0.13.4** | **Models only.** No CLI dep. |
| `is-terminal` | lock present via CLI | Reuse. |
| `uuid` / `dirs` | lock **1.23.1** / **6.0.0** | **No bump.** |
| rustc / edition | **1.95.0** / **2024** | Unchanged. |
| nextest | **0.9.140** | Unchanged. |
| workspace version | **0.1.1** | **No bump.** |
| New crates | — | **Zero.** |
| clap 5 | not released (max 4.6.x) | Forbidden future-bump guard. |

---

## 3. Frozen decisions

| ID | Decision |
|----|----------|
| **F0 — Go gate** | Plan-only until user **go**. Planning is DOCS. Implement starts a FEATURE TX. |
| **F1 — JSON status** | Ship `ai-brains nightly --status --format json`. CLI-local serde struct. **No** `ai-brains-contracts` DTO. **No** daemon/HTTP path. |
| **F2 — Human freeze** | Default `--format human`. Piped `nightly --status` **stays human** (`=== Nightly Status ===`). Do **not** silently switch pipes to JSON (T229/T247 contract). |
| **F3 — `--format auto`** | Available. TTY → human; pipe → json. Local `resolve_nightly_status_format` copied from T248/T249 (pretty/human/text/markdown/md → human; json → json; `_` fail-closed json). **Do not** extract a shared `resolve_*_format` (T249 pin). |
| **F4 — clap** | `--format` **requires** `--status`; `conflicts_with_all = ["schedule", "unschedule"]`. `value_parser = ["auto", "pretty", "human", "text", "json", "markdown", "md"]`. Default **`human`**. Unknown / `JSON` / `Pretty` → clap `InvalidValue` exit **2** (T249 AC16). No manual `if` in `nightly::run`. |
| **F5 — JSON keys** | Frozen object (pretty-printed). `schema_version` **1**. **SoT is AC3 + §5.1 together** (AI1 BS1): include `action_target` and `errors_last_run_unreadable` in both. Sort by struct field order (do not `BTreeMap` scramble). `null` for missing optional scheduler fields. |
| **F6 — Exit 0** | JSON and human status exit **0** for down / timeout / missing action / nonzero Last Result / Router 267009. Usage (`--format` without `--status`, unknown token) → **2**. |
| **F7 — Router display** | Additive human lines after Completion/Embedding, before Multi-import. Reuse `fetch_schedule_snapshot("AI-Brains-Router")` + `explain_last_task_result`. Do **not** register, start, stop, or rewrite the Router task. |
| **F8 — SchtasksListV.status** | Additive `status: Option<String>` parsed from English `Status:`. Existing units stay green (`Default` / extra field). Missing → omit human status word, JSON `null`. |
| **F9 — Router missing** | `found == false` (LIST /V non-zero / spawn fail) → human `Router: not scheduled` (no `next:`). JSON `router.scheduled = false`. ONLOGON with `found == true` and `next_run == None` is **still scheduled** (do **not** use Nightly’s `next_run.is_some()`). Do **not** suggest `nightly --schedule` (that does not register Router). |
| **F10 — Router Task To Run** | Live value is **unquoted** `C:\llm\router.bat`. Do **not** apply T247 F6 first-quoted-token missing-action to Router (would miss unquoted paths). JSON may include raw `task_to_run`. No Router “action missing” human line. |
| **F11 — Decline doctor ports** | Do **not** add a 16th doctor check. T192/T249 lock 15 names + `DoctorReport`. Status is the model-port matrix. Docs say so. |
| **F12 — Decline persist probe** | `--status` does not write `sync_state`. Last-run counters stay as they are. |
| **F13 — Decline embed sleep** | Do not retune the 50 ms yield. No evidence. Leave `embeddings.rs` untouched. |
| **F14 — Decline wrapper / schedule Router** | Do not write `nightly-run.cmd`. Do not register `AI-Brains-Router` from `nightly --schedule`. Do not mutate the live Nightly task (still points at the missing `.cmd`). T247 F10 stands. |
| **F15 — Decline `--no-vault`** | `--quick` still opens the vault (T247 O12). Soft residual only. |
| **F16 — Decline T253 nightly** | Claude/Codex stay out of the nightly batch (T239 D16 / T253 F34). |
| **F17 — `--quick` unchanged** | Still skips `LlamaCppProvider` / `probe_health`. Still `resolve_nightly_model_endpoints`. JSON `probe` is `"skipped"`. |
| **F18 — Probe timeouts unchanged** | Status 750 ms `join!`; run 2s. No models crate change. No `ProbeStatus::Skipped`. |
| **F19 — Multi-import JSON** | Reuse `MultiImportStatusView`: `{ "status": "never" }` / `{ "status": "unreadable" }` / `{ "status": "ok", "at", "agy", "grok", "opencode" }` with existing `SourceImportReport` fields. Do not fork the report schema. |
| **F20 — errors_last_run** | Thread raw `Option<String>` from `get_sync_state` (AI1 BS2). **Do not** `unwrap_or_else(\|\| "[]")` before the builder. `None` → JSON `errors_last_run: null`, `errors_last_run_unreadable: false` (absent ≠ corrupt). `Some` valid JSON array → emit the array, flag false. `Some` non-array / parse fail → emit the raw string, flag **true**. Human path may still display `"[]"` when missing (T247 chrome freeze). Never panic. Same parse-or-warn idiom as `load_multi_import_status` (AI1 O3). |
| **F21 — Module** | New helpers + JSON types + Router formatter + format resolver + their units live in `crates/ai-brains-cli/src/commands/nightly_status.rs`. `nightly.rs` prints / dispatches only. **Do not** move T229/T247 tests out of `nightly.rs`. Builder takes **precomputed** `action_target` / `action_target_missing` / `next_step` (call `first_quoted_action_target` + `exists` in `nightly.rs`). Helper is **already** `pub(crate)` (`:872`) — do not “promote” it; do not re-stat the path in the builder. |
| **F22 — Capture independence** | Status/docs only. No events. No models on the `--quick` / JSON-build path beyond existing probe. |
| **F23 — Pins / crates** | No clap 5, no lock bumps, no new crates, workspace **0.1.1**. No CLI `reqwest` / `wiremock`. |
| **F24 — Contracts** | No DTO. PROTOCOL-COMPAT untouched. CHANGELOG + CAPABILITIES + OPERATIONS + CLI-EXIT-CODES. |
| **F25 — Help / docs** | CAPABILITIES: **one additive** T247 honesty bullet (`--format json` / default human / Router read-only) — not a new section (AI1 O5). OPERATIONS: one `--format json` example + “doctor is not the model-port matrix”. Root CHANGELOG T255 row **must** say piped default stays human. CLI-EXIT-CODES status footnote. **Required** Nightly clap `after_help` (none today): `--format json` example + “default human; pipes stay human” (AI1 BS7). |
| **F26 — Tests** | Naming `function_or_feature__condition__expected_result`. Units for format resolver, JSON keys, Router missing/running, `Status:` parse. Clap AC for requires / unknown / `JSON`. Hermetic: `--format json` parses + has no `=== Nightly Status ===`. Existing T229/T247 units stay green. No `unwrap`/`expect`/`panic` in production. |
| **F27 — Cross-model** | FEATURE (new operator JSON contract). After Phase-1 review clean, run read-only `codex-review`. |
| **F28 — Debt file** | `conductor/ISSUES.md` does **not** exist. Deferrals go to `conductor/deferred.md`. |
| **F29 — PATH-behind** | Live `ai-brains` on PATH is pre-T247. Do **not** `cargo install` unless the user asks. Tests/manual AC use `cargo run` / hermetic bin. |
| **F30 — Stop-before live mutate** | Even after go: do not unschedule/reschedule Nightly, do not write `.cmd`, do not touch Router registration, unless the user explicitly confirms that remediating action. |
| **F31 — is-terminal** | `is_terminal::IsTerminal` on stdout, same as T248/T249. Do not migrate the crate. |
| **F32 — Non-Windows** | Omit scheduler + Router lines (T229). JSON `scheduled: null` / `router: null` on non-Windows (not fake `false`). |
| **F33 — Decline extras** | T167; T240 F13/F14; T254 F12; T249 F12 shared format helper; T253 Unix wrappers; color/pager; `comfy-table`; doctor compact JSON DTO. |
| **F34 — Snapshot foundness** | `fetch_schedule_snapshot` returns `{ found: bool, snap: SchtasksListV }`. `found =` LIST /V **success** (task exists). Non-zero / spawn fail → `found: false` + default fields. Success with all-N/A → `found: true`. Nightly JSON `scheduled` stays `next_run.is_some()` (matches human `Scheduled: Yes/No`). Router JSON `scheduled` = `found` (ONLOGON has no next run). |
| **F35 — last_nightly_count Option** | Thread raw `Option<String>` (AI1 BS2). Do **not** coalesce missing to `"0"` before the builder. `None` / non-`usize` → JSON `sessions_summarized_last_run: null`. Human may still print `0` when missing. |
| **F36 — Same endpoint tuple** | JSON `completion` / `embedding` consume the same `host_port_from_url` + model + probe-label tuple the human `format_endpoint_line` uses (AI1 O2). `--quick` label is still the string `"skipped"`. |
| **F37 — after_help required** | Nightly has **no** `after_help` today (`main.rs` `:480`). Add one small block. Not optional. |

---

## 4. Acceptance criteria

| AC | Proof |
|----|-------|
| **AC1** | Unit: `resolve_nightly_status_format("auto", true) == "human"`; `("auto", false) == "json"`; pretty/human/text/markdown/md → human; `json` → json regardless of TTY |
| **AC2** | Clap: `nightly --format json` without `--status` → `InvalidValue`/`MissingRequiredArgument` class, exit **2**. `nightly --status --format xml` → `InvalidValue`. `nightly --status --format JSON` → `InvalidValue` (T249 AC16) |
| **AC3** | Unit: JSON value from a fixture snapshot contains **every** frozen key: `schema_version`, `scheduled`, `next_run`, `last_task_result`, `last_task_result_hint`, `last_scheduled_run`, `action_target`, `action_target_missing`, `next_step`, `last_nightly_run`, `unsummarized_sessions`, `sessions_summarized_last_run`, `errors_last_run`, `errors_last_run_unreadable`, `completion`, `embedding`, `multi_import`, `router` (and `router.task_to_run` when scheduled). `schema_version == 1` |
| **AC4** | Unit: completion/embedding objects are `{ host_port, model, probe }`. `--quick` fixture `probe == "skipped"` |
| **AC5** | Unit: `parse_schtasks_list_v` English fixture now also fills `status` (`Ready` / `Running`) without breaking existing next/last/result/task-to-run asserts |
| **AC6** | Unit: Router `found` + status `Running` + 267009 → human contains `Router:` and `267009` and the T247 running hint **on the following line**; JSON `router.scheduled == true`, `router.last_result == "267009"` |
| **AC7** | Unit: Router `found == false` → human `Router: not scheduled`; JSON `router.scheduled == false`; **no** `next: ai-brains nightly --schedule`. Separate unit: `found == true`, `next_run == None`, last_result present (ONLOGON) → `router.scheduled == true` |
| **AC8** | Existing T247 units stay green: `Last task result: 101` substring; `probe=skipped`; missing `.cmd` + `nightly --schedule --dry-run`; `--quick` requires `--status` |
| **AC9** | Hermetic / process: `nightly --status --format json` on a temp vault exits **0**, stdout is one JSON object (no `=== Nightly Status ===`). Assert **only** stable keys: `schema_version`, `completion`, `embedding`, `multi_import` (never on a fresh vault). **Do not** assert host `scheduled` / `router` / Last Result — those spawn live `schtasks` against this machine (AI1 BS5). |
| **AC10** | Hermetic: default `nightly --status` (no `--format`) still prints `=== Nightly Status ===` (human), even if stdout is not a TTY |
| **AC11** | Manual (source bin): `nightly --status --format json` shows Nightly last result **1**, `action_target_missing: true`, Router **267009** / Running, embedding host `:8083`. Exit 0. Do **not** mutate tasks |
| **AC12** | Docs: CAPABILITIES + OPERATIONS + root CHANGELOG T255 + CLI-EXIT-CODES status footnote. Doctor still documented as 15 checks |
| **AC13** | No contracts DTO; no pin bumps; no new CLI dep; `embeddings.rs` untouched; doctor 15-check unit still passes; full gate green |
| **AC14** | `--format json --quick` does not construct `LlamaCppProvider` (same F19). JSON `probe` is `"skipped"` |
| **AC15** | Unit: Router `found`, `status == None`, last_result `267009` → human first line is `Router: last result: 267009` (no `Running`); hint still follows (AI1 BS6) |
| **AC16** | Unit (or `#[cfg(not(windows))]`): JSON builder with no scheduler input emits `scheduled: null` and `router: null`, never `false` (AI1 O6 / F32) |

---

## 5. Design notes

### 5.1 JSON shape (frozen, `schema_version: 1`)

```json
{
  "schema_version": 1,
  "scheduled": true,
  "next_run": "8/16/2026 3:00:00 AM",
  "last_task_result": "1",
  "last_task_result_hint": "process failed / missing action / CLI error",
  "last_scheduled_run": "8/15/2026 3:00:01 AM",
  "action_target": "C:\\Users\\RyanB\\.ai-brains\\nightly-run.cmd",
  "action_target_missing": true,
  "next_step": "ai-brains nightly --schedule --dry-run",
  "last_nightly_run": "2026-08-02T07:03:58.159733500+00:00",
  "unsummarized_sessions": 0,
  "sessions_summarized_last_run": 0,
  "errors_last_run": [],
  "errors_last_run_unreadable": false,
  "completion": { "host_port": "127.0.0.1:8081", "model": "gemma-4-E4B-it-Q6_K.gguf", "probe": "skipped" },
  "embedding": { "host_port": "127.0.0.1:8083", "model": "nomic-embed-text-v1.5", "probe": "ok" },
  "multi_import": { "status": "never" },
  "router": {
    "scheduled": true,
    "status": "Running",
    "last_result": "267009",
    "last_result_hint": "task still running (SCHED_S_TASK_RUNNING)",
    "task_to_run": "C:\\llm\\router.bat"
  }
}
```

- Non-Windows: `scheduled`, scheduler strings, and `router` are JSON `null` (F32 / AC16).
- `next_step` is present **only** when `action_target_missing` is true; otherwise `null`.
- `last_nightly_run` is `null` when vault says never (human prints `never`).
- `sessions_summarized_last_run`: `null` when the sync_state key is **missing** or not a `usize`. Do **not** invent `0` from a missing key (F35). A stored `"0"` is a real zero.
- `errors_last_run`: `null` + `errors_last_run_unreadable: false` when the key is missing; array when valid JSON array; raw string + `true` when corrupt (F20).

### 5.2 Human Router lines

```text
Completion: …
Embedding: …
Router: Running  last result: 267009
task still running (SCHED_S_TASK_RUNNING)
Multi-import: never
```

When status is missing but last_result present: `Router: last result: 267009` (AC15).
When `found == false`: single line `Router: not scheduled`.
When `found == true` and ONLOGON (`next_run` N/A): still print `Router:` (status word if present) + last result — **not** `not scheduled`.

Hint is a **following** line (same T247 F4 rule). Reuse `explain_last_task_result` — do not fork. AC8’s Nightly `Last task result: 101` is a different line — do not suffix.

### 5.3 Default format is human

T248/T249 defaulted **new** dual surfaces to `auto`. Nightly `--status` has been human since T135/T229/T247, including pipes. Switching the default to `auto` would break `nightly --status | Select-String`. Default **`human`**. Operators who want T248 behavior pass `--format auto`.

### 5.4 Capture independence

No `MemoryPinned`, no nightly events, no probe persistence (F12). Status is a query. Router fetch is a second `schtasks` spawn (~30 ms), not HTTP.

### 5.5 Live missing `.cmd`

T247 already names it. T255 does **not** remediate. `next_step` in JSON is the same dry-run command. F30 stop-before.

---

## 6. Non-goals

- Doctor 16th / 17th check or contracts `DoctorReport` growth
- Writing probe results to `sync_state`
- Retuning embed 50 ms
- Product-owned `nightly-run.cmd` / registering `AI-Brains-Router` from `nightly --schedule`
- Mutating the live Nightly task (missing `.cmd`)
- `--quick --no-vault`
- Claude/Codex nightly sources (T253 F34)
- Shared `resolve_*_format` helper
- clap 5 / pin bumps / new crates / CLI `reqwest`
- T167 / T240 F13–F14 / T254 leftovers
- Starting `router.bat` / llama-server from Rust
- Reinstalling PATH `ai-brains`

---

## 7. Verification plan

```powershell
# Red → green
cargo nextest run -p ai-brains-cli --lib nightly_status
cargo nextest run -p ai-brains-cli --lib nightly
cargo nextest run -p ai-brains-cli -E "test(nightly)"
cargo clippy -p ai-brains-cli --all-targets -- -D warnings

# Manual (source bin — PATH is pre-T247)
cargo run -q -p ai-brains-cli -- nightly --status
cargo run -q -p ai-brains-cli -- nightly --status --format json
cargo run -q -p ai-brains-cli -- nightly --status --quick --format json
# Do not schtasks /change or write nightly-run.cmd

# Full gate
cargo fmt --check ; cargo clippy --workspace --all-targets -- -D warnings
cargo nextest run --workspace ; cargo deny check ; cargo audit
ledgerful verify --scope full
```

---

## 8. Risk

| Risk | Mitigation |
|------|------------|
| Pipe default JSON breaks greps | **F2** default `human` |
| Shared format helper drift | **F3** local copy (T249) |
| Doctor 16th check contracts | **F11** decline |
| Growing 1819-line `nightly.rs` | **F21** `nightly_status.rs` |
| Relocating T247 tests | **F21** leave them in `nightly.rs` |
| Router unquoted path vs F6 | **F10** do not reuse missing-action parser |
| Live task “fix” temptation | **F14/F30** stop-before |
| PATH-behind false AC fail | **F29** `cargo run` / hermetic |
| `SchtasksListV` field add breaks tests | **F8** `Default`; update explicit structs only if they fail |
| AC3/§5.1 key drift | **F5** single SoT; AC3 lists every key including `action_target` + `errors_last_run_unreadable` |
| Missing sync_state coalesced to 0/[] | **F20/F35** thread `Option` |
| Router ONLOGON `scheduled: false` | **F34** `found` flag; Router ≠ Nightly derivation |
| Hermetic AC9 flakes on host tasks | **AC9** assert only stable keys |
| Nightly help gap | **F25/F37** required `after_help` |

---

## 9. Implement order (on go)

1. Red→Green F3/F4/AC1–AC2 format resolver + clap + F37 `after_help`  
2. Red→Green F5/F19/F20/F35/F36/AC3–AC4/AC9/AC14/AC16 JSON builder (raw `Option`, same endpoint tuple)  
3. Red→Green F7–F10/F8/F34/AC5–AC7/AC15 Router line + `status` + snapshot `found`  
4. Wire `nightly.rs` status branch (human additive + JSON emit; precompute action fields)  
5. Docs F25 / AC12 (additive bullets + CHANGELOG default-human)  
6. Manual AC10–AC11 (no live mutate) → review → full gate → Complete  

---

## 10. Soft residuals (post-close)

| Residual | Note |
|----------|------|
| `--quick --no-vault` | T247 O12 — still soft |
| Persist probe | Declined F12 |
| Doctor model ports | Declined F11 |
| 50ms embed sleep | Declined F13 — reopen only with nightly-run timings |
| Product `.cmd` / schedule Router | Declined F14 — operator confirm |
| Shared `resolve_*_format` | T249 F12 |
| `std::io::IsTerminal` migrate | T249 F12 |
| PATH reinstall | Operator / F29 |
| Live reschedule of missing `.cmd` | F30 — not automatic Close |

---

## 11. Touch map (expected)

| Site | Change |
|------|--------|
| **new** `commands/nightly_status.rs` | format resolver, JSON types, Router formatter, units |
| `commands/mod.rs` | `mod nightly_status` |
| `commands/nightly.rs` | call JSON/router helpers; additive human Router lines; thread `format`; **F34** snapshot `{ found, snap }`; thread raw `Option` sync_state; precompute action fields |
| `main.rs` | clap `--format` on `Nightly`; dispatch the string; **required** `after_help` (F37) |
| `Docs/OPERATIONS.md` | `--format json`; default human; Router read-only line |
| `Docs/CAPABILITIES.md` | status honesty bullets |
| `Docs/CLI-EXIT-CODES.md` | nightly status exit **0** footnote |
| `CHANGELOG.md` (repo root) | T255 row |
| `ai-brains-models` / `embeddings.rs` / doctor / contracts | **No** |
| events / store / daemon | **None** |

---

## 12. AI fold-in disposition (2026-08-16)

Source: `C:\dev\AI-review.md` — **AI1** (code-truth + 7 blind spots) + **AI2** (affirms F2/F10/F11/F12/F21). No Highs. No blockers.

### AI1

| ID | Verdict | Action |
|----|---------|--------|
| **BS1** AC3 vs §5.1 keys | **Agree hard** | AC3 + F5 list `action_target` and `errors_last_run_unreadable` |
| **BS2** `unwrap_or "0"`/`"[]"` | **Agree hard** | F20 / F35 thread raw `Option` |
| **BS3** Router `scheduled` ≠ `next_run` | **Agree hard** | F9 / F34 `found` flag |
| **BS4** `first_quoted` private | **Decline as stated** | Already `pub(crate)` `:872`. **Agree the intent:** pass precomputed action fields (F21) |
| **BS5** AC9 host schtasks | **Agree hard** | AC9 stable keys only |
| **BS6** blank-status Router line | **Agree** | AC15 |
| **BS7** Nightly `after_help` | **Agree hard** | F25 / F37 required |
| **O1** snapshot foundness | **Agree hard** | F34 |
| **O2** same endpoint tuple | **Agree** | F36 |
| **O3** MultiImport parse idiom | **Agree** | F20 wording |
| **O4** clap template + CHANGELOG human | **Agree** | F2 / F4 / Phase 4 already; keep |
| **O5** additive docs | **Agree** | F25 |
| **O6** non-Windows JSON unit | **Agree** | AC16 |

### AI2

| ID | Verdict | Action |
|----|---------|--------|
| **BS1** default human vs auto | **Affirm** | Already F2 / §5.3 |
| **BS2** unquoted Router path | **Affirm** | Already F10 |
| **BS3** CQRS / no persist probe | **Affirm** | Already F12 |
| **BS4** doctor 15-check | **Affirm** | Already F11 |
| **BS5** non-Windows null | **Affirm** | Already F32; **plus** AC16 |
| **BS6** module split | **Affirm** | F21; line count is **1819** not 1968 |
| JSON example omitting two keys | **Decline as SoT** | Keep `errors_last_run_unreadable` + `router.task_to_run` (AI1 BS1) |
| “Ready for implementation” | **Affirm as plan-ready** | Still **plan-only until go** |

### Pins locked by fold-in

1. **F5/AC3:** one key list; include `action_target` + `errors_last_run_unreadable`.
2. **F20/F35:** raw `Option<String>` from `get_sync_state`; human may still print `0`/`[]`.
3. **F34:** snapshot `{ found, snap }`; Nightly `scheduled` = `next_run.is_some()`; Router `scheduled` = `found`.
4. **F21:** precompute action fields; do not restat in the builder.
5. **AC9:** hermetic asserts only `schema_version` / endpoints / `multi_import`.
6. **F25/F37:** Nightly `after_help` is required.
7. **AC15/AC16:** blank-status Router line; non-Windows JSON nulls.

---

**Planning + fold-in 2026-08-16.** Still **plan-only until go**.
