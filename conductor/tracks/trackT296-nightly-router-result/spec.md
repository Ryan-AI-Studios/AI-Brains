# T296 — Nightly Router last-result must not look like Nightly success/failure

- **Track ID:** T296-NightlyRouterResult
- **Status:** **Planned** (Pending until **go**; not Placeholder)
- **Category:** UX / HONESTY
- **Owner:** Grok
- **Source:** Audit 2026-08-22 — `nightly --status --quick` **8/8** split exists; friction: `Router: Ready last result: 267014` + `task terminated (SCHED_S_TASK_TERMINATED)` next to Nightly `Last task result: 0`. Placeholder minted with T285–T300 (`76c4db9`).
- **Depends on:** T269 ✅ `Nightly:` heading; T255 ✅ read-only Router + JSON; T281 ✅ timeout contrast (do **not** raise 750); T247 ✅ `explain_last_task_result` decode
- **Blocks / feeds:** Operators can tell Nightly process exit **0** from Router scheduler HRESULT **267014**. Daemon Stopped vs llama Open **T297**. Device/replicate empty **T298**. Forget-list **T299**. Graph sparse **T300**.
- **Absorbs:** Placeholder problem text + Manual DoD; deferred.md “nightly Router 267014 / TASK_TERMINATED”; T269 closeout “do not restyle Router” **as a T269 freeze that this track explicitly supersedes on the human Router line only**; T255 AC6/AC15 **human** half (numeric on first line)
- **Not absorbed (DoD):** Raise 750 ms; unify daemon TCP with HTTP; JSON keys / `last_result` raw string / `last_result_hint` SCHED_S text; doctor 16th; persist probe; product `.cmd` / schedule-Router; `--quick --no-vault`; mutate `AI-Brains-Router`; T297 daemon contrast; Nightly `Last task result:` chrome; `explain_last_task_result` strings; clap 5 / rusqlite 0.40; T298–T300; T240 F2
- **Research date:** 2026-08-24 (plan dogfood HEAD `8b95181` T295 `#211`. Fold-in HEAD `c7d6e3e`. Product `src/` = T295; nightly last product chrome T281. PATH **0.1.2** 2026-08-22 19:41 **has T269/T281**, not T285–T295. Live hole is **human Router HRESULT**, not missing heading.)
- **AI fold-in:** 2026-08-24 `agy-review.md` (`c7d6e3e`) + `opencode-review.md` (`c7d6e3e`). **Agy B 0 / M 0.** **OpenCode B 0 / M 0.** **Agree:** Agy m1 hex rstest F33/AC3; Agy m2 whitespace-blank F34/AC3; Agy O1 CLI-EXIT-CODES both SCHED_S AC10. **Already:** Agy O2 AC6; OpenCode O1 AC1+AC8; OpenCode O2 F1/AC1; OpenCode m3 F11. **Snapshot:** OpenCode m1/m2 HEAD/word/pin. **Decline:** OpenCode “no help change” (F7/AC6); OpenCode `nightly.rs` production edit (F9). Disposition **§13**.
- **Ledger:** planning DOCS TX `3b6532dc-54eb-4313-bdf8-477f4348a694`. Fold-in DOCS TX `314aa590-c779-4c0a-9889-81681319e950`. Implement starts a **BUGFIX** TX on **go**.
- **Isolation:** Do **not** `cargo install`. Do **not** rewrite `.env` (T240 F2). Do **not** mutate `AI-Brains-Nightly` or `AI-Brains-Router`. Do **not** raise `NIGHTLY_STATUS_PROBE_TIMEOUT`. Do **not** grow hotspot `project.rs` / `sync.rs` / `governed_common.rs` / `forget.rs` / `doctor.rs` / `daemon.rs`. Do **not** edit `explain_last_task_result` (Nightly + JSON hints). Do **not** print or commit `AI_BRAINS_KEY`. Do **not** live `retention apply --confirm`, `graph rebuild`, leftover `rebind-path --write --yes`, or `safety sync` without `--dry-run`.

---

## 1. Objective

1. **Human Router last-result is not a Nightly failure code.** Live `nightly --status --quick` prints `Last task result: 0` under `Nightly: AI-Brains-Nightly`, then `Router: Ready  last result: 267014` + `task terminated (SCHED_S_TASK_TERMINATED)`. Microsoft Learn: **267014** = `SCHED_S_TASK_TERMINATED` = `0x00041306` — a **SCHED_S_ success** constant (“The last run of the task was terminated by the user.”), not a process exit. Operators mix it with Nightly **0**.
2. **Human omits the decimal HRESULT.** When Status is present, first line is `Router: {status}` only. Known scheduler-success codes map to a short phrase **without** `267014` / `267009` / `SCHED_S_*`. JSON stays frozen (`router.last_result` still `"267014"`).
3. **Keep T247/T255/T269/T281 contracts.** Status exit **0**. JSON `schema_version` **1** + `FROZEN_KEYS`. `--quick` still `probe=skipped`. 750 ms not raised. Heading `Nightly: AI-Brains-Nightly` stays. Nightly `Last task result:` still prints the Nightly code. `explain_last_task_result` strings stay for Nightly human + JSON hints. No live task mutate. No doctor 16th.
4. **North star.** Capture independence: status/docs/parse only. No events. No models on `--quick`. No contracts DTO. No new crates. No pin bumps.

This unblocks daily ops honesty for the Windows-first vault: the nightly job succeeded (Last Result **0**) while the ONLOGON Router keep-alive is *supposed* to show a scheduler success HRESULT. T269 split the headings. T296 stops printing the HRESULT as if it were a failed nightly.

---

## 2. Live baseline (re-scan 2026-08-24)

### 2.1 Operator dogfood (this machine)

| Signal | Observation |
|--------|-------------|
| HEAD | Plan dogfood `8b95181` T295 `#211`. Fold-in `c7d6e3e` (T296 plan; parent `8b95181`). Tree **CLEAN**. `origin/main` = `8b95181` (`left-right` `0 1` — local plan commit). Branch `main`. |
| PATH `ai-brains` | `C:\Users\RyanB\.cargo\bin\ai-brains.exe` mtime **2026-08-22 19:41**, 25 139 712 bytes, **0.1.2**. **Has T269 heading + T281 contrast + T255 Router numeric.** **Does not have T285–T295.** Nightly chrome for this hole is **on PATH** (T281-era). **Do not `cargo install`.** Tests/manual AC use `cargo run` / hermetic. |
| `preflight --summary` | Scope `C:\dev\ai-brains` (`3581317d`). Pinned **4119** (volatile; plan 4102 / OpenCode 4102). In-context **0/0/0**. Word **536** (plan 367 / OpenCode 428). Capture independence holds. |
| PATH `nightly --status --quick` | Banner + `Nightly: AI-Brains-Nightly`. `Last task result: **0**`. Last scheduled **8/24/2026 3:00:02 AM**. Last nightly run **2026-08-24T07:08:10Z**. Sessions summarized **3**. Completion/Embedding `probe=skipped`. **`Router: Ready  last result: 267014`** then **`task terminated (SCHED_S_TASK_TERMINATED)`**. Multi-import 2026-08-24 ok. Exit **0**. **Live hole confirmed.** |
| PATH `nightly --status --quick --format json` | `schema_version` 1. `last_task_result` **`"0"`** / hint `null`. `completion.probe` / `embedding.probe` `"skipped"`. `router.scheduled` true, `status` `"Ready"`, `last_result` **`"267014"`**, `last_result_hint` `"task terminated (SCHED_S_TASK_TERMINATED)"`, `task_to_run` `C:\llm\router.bat`. Frozen keys present. |
| `AI-Brains-Router` LIST /V | **Ready**; Enabled; Next Run **N/A** (ONLOGON); Last Run **8/19/2026 2:40:07 PM**; Last Result **267014**; Task To Run **unquoted** `C:\llm\router.bat`. |
| `AI-Brains-Nightly` (from status) | Ready; Last Result **0**; next **8/25/2026 3:00:00 AM**. |
| `nightly --help` after_help | Already: `Nightly Last Result is AI-Brains-Nightly. Router 267009 is SCHED_S_TASK_RUNNING (success; ONLOGON keep-alive).` **No 267014 / TERMINATED sentence.** T269 AC6 locks 267009 needles. |
| `daemon status` (`--no-project-context`) | **Stopped** + `next: ai-brains daemon start`. LLM default `:11434` Open (env-unset path). **T297**, not this track. |
| Doctor (ledgerful) | **4** warn (legacy `.changeguard` / sig-pin / sig-version / timings). Completion model **unreachable** this pass. Embedding `:8083` **ok**. **0 pending / 0 drift.** |
| Last GitHub PR | [#211](https://github.com/Ryan-AI-Studios/AI-Brains/pull/211) T295 (merged 2026-08-24). `gh pr view --comments`, `/reviews`, `/comments`, `issues/211/comments` all **empty**. **last-PR Cursor: N/A.** Open PRs: Dependabot remotes only (`#61` rusqlite 0.40.2, `#62` chrono 0.4.45, `#59` tokio, `#60` thiserror, `#58` tower-http, actions `#68–#72`). **No leftover to mint. No T301.** |
| Identity / hotspots | Hotspot **#1** `project.rs` (**3.897** fold-in; plan 3.906) — **do not touch.** `sync.rs` #2. `governed_common.rs` #3. `context.rs` #4. `forget.rs` #5. `nightly.rs` **2128** / `nightly_status.rs` **760** — not top-10. `doctor.rs` **1855**. |
| `ISSUES.md` | **Does not exist.** |

### 2.2 Why this still matters

| Residual | Why it is still a product hole / why decline |
|----------|----------------------------------------------|
| Human `last result: 267014` next to Nightly `0` | T269 labeled the Nightly task. The Router line still dumps a six-digit HRESULT that looks like a failed nightly. Microsoft: **SCHED_S_** = success. clig.dev: saying (just) enough — the Status word `Ready` is the live state; the decimal is noise that trains operators to ignore preflight. **DoD.** |
| Hint `task terminated (SCHED_S_TASK_TERMINATED)` | Accurate for JSON / Nightly decode. On human Router it still *looks* like an error (`terminated` + `SCHED_S_*`). Map to `last run: terminated` without the constant name. **DoD.** |
| after_help explains 267009 only | Live Router this machine is **267014**, not 267009. Help does not cover the audit token. Additive sentence. **DoD.** |
| Restyle Nightly `Last task result:` | Nightly uses process exits (0/1/101). Heading already names the task. **Decline.** |
| Change JSON `last_result` / hint | T255 F5 / AC3 / `FROZEN_KEYS`. Scripts already have `"267014"`. **Decline.** |
| Change `explain_last_task_result` | Nightly human + JSON hints + T247 units. **Decline.** New Router human helper. |
| Raise 750 / unify daemon HTTP | T255 F18 / T281 F2/F10. **Decline** → T297 for daemon Stopped vs Open. |
| Mutate Router / write `.cmd` | T255 F14 / F30. **Decline.** |
| Doctor 16th | T255 F11. **Decline.** |

### 2.3 Code truth (opened)

| Item | Location | Notes |
|------|----------|--------|
| Human Router print | `nightly.rs` **`:208–216`** | Calls `format_router_status_lines(found, status, last_result)`. **Keep call site.** Body change is in the helper. |
| Helper | `nightly_status.rs` `format_router_status_lines` **`:187–216`** | Today: `Router: {st}  last result: {code}` + `explain_last_task_result` following line. **This is the DoD edit.** Signature stays 3 args. |
| Nightly heading | `NIGHTLY_TASK_HEADING` **`:10`** / print `nightly.rs` **`:157`** | Frozen. |
| Nightly Last Result | `format_schedule_status_lines` `nightly.rs` **`:865–887`** | Still `Last task result: {label}` + `explain_last_task_result`. **Do not edit.** |
| Decode | `explain_last_task_result` `nightly.rs` **`:958–973`** | 0 → None; 1 / 101 / **267009** / **267014** (hex `0x41301` / `0x41306`). **Do not edit strings.** JSON `last_result_hint` still uses this (`nightly_status.rs` **`:224–228`**). |
| JSON builder | `build_nightly_status_json` + `FROZEN_KEYS` **`:289–308`** | `router.last_result` raw. **No new keys.** |
| Existing Router units | `nightly_status.rs` **`:560–627`** | AC6 Running+267009 first line **numeric**; AC15 blank status `Router: last result: 267009`. **Rewrite human asserts** (T296 supersedes T255 AC6/AC15 human). JSON half of AC6 stays (`last_result == "267009"`). |
| `--quick` hermetic | `tests/nightly_status.rs` **`:77–121`** | Heading + `probe=skipped` + no T281 contrast. **Stay-green.** Do **not** assert host Router code (T255 AC9 — live `schtasks`). Units lock 267014. |
| after_help | `main.rs` Nightly **`:1432–1434`** | T269 267009 sentence. Help test **`:952–980`**. **Additive 267014 sentence.** AC6 needles stay. |
| T281 contrast | `HTTP_VS_TCP_CONTRAST` / `completion_status_human_lines` | Untouched. |
| Doctor / embeddings / daemon | `doctor.rs` 15; `embeddings.rs` 50 ms; `daemon.rs` TCP | Untouched. |
| PROTOCOL-COMPAT | no `nightly` keys | Untouched. |

Callers (`ledgerful search`): `format_router_status_lines` = helper + `nightly.rs:210` + three units. `explain_last_task_result` = Nightly schedule block `:883` + JSON hint + Router human (remove that last use for 267009/267014).

### 2.4 Dependency / standards research (2026-08-24) — snapshot; re-verify at execute

| Pin | Workspace / lock | Ecosystem (today) | Action |
|-----|------------------|-------------------|--------|
| `clap` | workspace **4.5** / lock **4.6.1** | crates.io **4.6.6** (2026-08-06). GitHub clap-rs latest **v4.6.6**. **No clap 5.** | **No bump.** after_help additive string only. |
| `serde_json` | lock **1.0.150** | JSON keys frozen | **No bump.** |
| `chrono` | lock **0.4.44** | crates.io **0.4.45** (Dependabot `#62`) | **No bump.** |
| `rusqlite` | lock **0.39.0** + sqlcipher + backup | crates.io **0.40.2** (Dependabot `#61`) | **No bump.** |
| `thiserror` | lock **2.0.18** | Dependabot `#60` 2.0.20 | **No bump.** |
| `tokio` | workspace **1.52** / lock **1.52.3** | Dependabot `#59` 1.53.1 | **No bump.** |
| rustc / edition | **1.95.0** / **2024** | workspace toolchain | Unchanged |
| nextest | **0.9.140** | — | Unchanged |
| workspace | **0.1.2** | — | **No bump** |
| New crates | — | — | **Zero.** |

**Online / primary sources**

| Claim | Source | Takeaway |
|-------|--------|----------|
| 267014 is a **success** HRESULT | [Microsoft Learn — Task Scheduler error and success constants](https://learn.microsoft.com/en-us/windows/win32/taskschd/task-scheduler-error-and-success-constants) (updated 2024-10-29): **SCHED_S_** = success; **SCHED_S_TASK_TERMINATED** `_HRESULT_TYPEDEF_(0x00041306L)` — “The last run of the task was terminated by the user.” `0x41306` = **267014**. **SCHED_S_TASK_RUNNING** `0x00041301` = **267009**. | Do not present 267014 as a failed Nightly. Human phrase, not the decimal. |
| Operators treat 267014 as an error | [Microsoft Q&A 4257963](https://learn.microsoft.com/en-us/answers/questions/4257963/task-scheduler-error-267014-process-terminated-by) titles it “error 267014”. Same confusion this audit scored. | Omitting the decimal on human is the remediator. |
| Human-first; saying (just) enough | [clig.dev](https://clig.dev/) (current) | Status word is enough for live state. JSON keeps the raw code for scripts. |
| clap `after_help` | docs.rs/clap/4.6.6 `Command::after_help` | Keep derive string. T269 AC6 needles stay; add 267014 sentence. |
| schtasks query (read-only) | [Microsoft Learn schtasks](https://learn.microsoft.com/en-us/windows/win32/taskschd/schtasks) | Planning used `/Query /FO LIST /V` only. **Do not** `/End` / `/Change` / `/Create`. |

**N/A:** SQLCipher page encrypt, T180 DTO new keys, Windows service install, Safety GLOB, policy HINT, clap 5 API, rusqlite Online Backup (T295).

**Could not verify:** why Router Last Run is **8/19/2026** (stale vs Nightly 8/24) — not this DoD; do not `/Run` the Router to refresh. Whether `C:\llm\router.bat` llama.cpp includes #20817 — T281 residual.

**ledgerful / ai-brains:** `preflight --summary` pinned **4102**; `ledgerful ledger status --compact` 0 pending / 0 drift; `search "format_router_status_lines"` → `nightly_status.rs:187` / `nightly.rs:210` / units `:560/:584/:614`; `scan --impact` CLEAN at `8b95181`; hotspots `project.rs` #1 **3.906**. Recall of “267014 Router” returned T255 review dumps (PATH-behind rank) — live src is SoT.

---

## 3. Frozen decisions

| ID | Decision |
|----|----------|
| **F0 — Go gate** | Plan-only until user **go**. Planning is DOCS TX `3b6532dc`. Implement starts a **BUGFIX** TX. |
| **F1 — Human first line** | If `found` and Status is non-empty: **`Router: {status}` only**. Never append `  last result: {code}` when Status is present. `Router: not scheduled` when `found == false` (T255 AC7). |
| **F2 — Human 267014** | Const `pub(crate) ROUTER_LAST_RUN_TERMINATED: &str = "last run: terminated"`. Ready + 267014 (or `0x41306`) → `["Router: Ready", "last run: terminated"]`. Blank Status + 267014 → `["Router: terminated"]` (do **not** invent `Ready`). **No** `267014` and **no** `SCHED_S_TASK_TERMINATED` on human Router lines. |
| **F3 — Human 267009** | Running + 267009 (or `0x41301`) → `["Router: Running"]` only (Status already says running). Blank Status + 267009 → `["Router: running"]`. **No** `267009` and **no** `SCHED_S_TASK_RUNNING` on human Router lines. **Supersedes T255 AC6/AC15 human** (JSON half of AC6 stays). |
| **F4 — Human 0 / process failures** | Ready + `0` → `["Router: Ready"]` (no follow). Blank + `0` → `["Router:"]`. Ready + `1`/`101` → first line Status; follow line **existing** `explain_last_task_result` strings (those are process exits, Nightly-like). Blank + `1`/`101` → `Router: last result: {code}` + existing hint. Unknown codes: if Status present, Status-only; else `Router: last result: {code}` (honesty). |
| **F5 — JSON frozen** | `schema_version` **1**. `FROZEN_KEYS` unchanged. `router.last_result` still `"267014"` / `"267009"`. `last_result_hint` still `explain_last_task_result` (`"task terminated (SCHED_S_TASK_TERMINATED)"` / `"task still running (SCHED_S_TASK_RUNNING)"`). No new keys. No human phrase in JSON. |
| **F6 — `explain_last_task_result` freeze** | Do **not** edit `nightly.rs` `:958–973` or T247 Nightly units. Nightly `Last task result: 0` / `1` / `101` chrome unchanged. Router **human** stops calling it for 267009/267014. JSON + Nightly still call it. |
| **F7 — after_help additive** | Keep T269 AC6 needles (`AI-Brains-Nightly`, `267009` or `SCHED_S_TASK_RUNNING`, `750`, `TCP`, `/health`). Add one sentence: Router **267014** is `SCHED_S_TASK_TERMINATED` (success; last run ended), **not** Nightly Last Result. New help unit. Do **not** rewrite the 267009 sentence. |
| **F8 — `--quick` / 750 / heading / T281** | `--quick` still `probe=skipped` (no HTTP, no contrast). 750 ms frozen. `NIGHTLY_TASK_HEADING` frozen. `HTTP_VS_TCP_CONTRAST` frozen. |
| **F9 — Module** | Human mapper + `ROUTER_LAST_RUN_TERMINATED` + units in `nightly_status.rs`. Prefer **zero** production edits in `nightly.rs` (call site already correct). Do **not** move T229/T247 tests out of `nightly.rs`. Do **not** grow `project.rs` / `sync.rs` / `forget.rs` / `daemon.rs` / `doctor.rs`. |
| **F10 — Decline doctor 16th / persist / embed sleep / wrapper / `--no-vault`** | T255 F11–F15 stand. |
| **F11 — Decline daemon HTTP / T297 steal** | `daemon status` Stopped vs Open is **T297**. Do not import `probe_health`. Do not print T297 contrast from nightly. |
| **F12 — Pins / crates** | No clap 5, no rusqlite 0.40, no chrono 0.4.45, no tokio 1.53, no new crates, workspace **0.1.2**. No CLI `reqwest`. |
| **F13 — PATH** | Do not `cargo install` unless the user asks. Tests/manual use `cargo run` / hermetic. PATH 0.1.2 already shows the hole. |
| **F14 — Contracts** | No DTO. PROTOCOL-COMPAT untouched. |
| **F15 — Capture independence** | Status/docs only. No events. No `sync_state` probe write. |
| **F16 — Stop-before live mutate** | Even after go: do not unschedule/reschedule Nightly, do not `/End` or `/Run` Router, do not write `.cmd`. |
| **F17 — Decline peers** | T297–T300; T294 leftover `--write`; T295 engine/doctor remediator; T240 F2; T263 H2; T255 750 ms raise. |
| **F18 — last-PR Cursor** | **#211** empty → **N/A**. Dependabot `#61` rusqlite **not** this track. **No T301.** |
| **F19 — Docs** | CAPABILITIES T269/T281 bullet: additive “human Router omits 267014/267009 decimals; `Ready` + `last run: terminated`”. OPERATIONS Router bullet. Root CHANGELOG T296. CLI-EXIT-CODES: status still exit **0** for Router **267009** (`SCHED_S_TASK_RUNNING`) **and** **267014** (`SCHED_S_TASK_TERMINATED`) — both scheduler **success** constants (Agy O1). |
| **F20 — Exit 0** | Unchanged. Timeout / 267009 / **267014** / down / daemon Stopped are still success for `--status`. |
| **F21 — Tests** | Naming `function_or_feature__condition__expected_result`. Required red names in §7 (include F33 hex + F34 whitespace). Existing T247/T269/T281 units stay green. Hermetic `--quick` must **not** contain human `267014` or `SCHED_S_TASK_TERMINATED` (safe: Nightly last result is 0; JSON is a different invocation). No `unwrap`/`expect`/`panic` in production. |
| **F22 — Cross-model** | Honesty UX on the status path (easy T255/T269 regression). After Phase-1 review clean, run read-only `codex-review`. Category **BUGFIX** (not FEATURE) — still run Codex like T281. |
| **F23 — Debt file** | `conductor/ISSUES.md` does **not** exist. Deferrals → `conductor/deferred.md`. |
| **F24 — PowerShell** | `;` not `&&`. |
| **F25 — Hex parse** | Accept `267014` and `0x41306` / `0X41306` (same parser class as `explain_last_task_result`). Do not fork a third parser if a tiny private `fn` in `nightly_status.rs` can parse decimal/hex; duplicating the 6-line parse is OK to avoid editing `nightly.rs`. |
| **F26 — Do not invent Status** | Blank Status must not print `Ready` / `Running` title-case unless mapping F2/F3 lowercase phrases (`terminated` / `running`). |
| **F27 — Identity leftover** | `7d97a456` vs `fcb8a40f` is T258 / leftover data. **Not this track.** |
| **F28 — `--no-project-context`** | Manual AC uses it (same as T295). Nightly status still opens the vault (`--quick` F15). |
| **F29 — Existing tests stay green** | T229 truncate; T247 `--quick` / LIST /V / `Last task result: 101`; T255 JSON keys / default human pipes / Router not-scheduled; T269 heading / suffix / after_help AC6; T281 contrast. **Except** T255 AC6/AC15 **human** numeric asserts — those are this rewrite. |
| **F30 — Signature freeze** | `format_router_status_lines(found, status, last_result) -> Vec<String>` stays 3 args. Do not add a 4th. |
| **F31 — Non-Windows** | Omit Router lines (T255 F32). JSON `router: null`. Units stay `#[cfg_attr(not(windows), allow(dead_code))]` as today. |
| **F32 — Two-space freeze gone** | T255 §5.2 “two spaces before `last result:`” **retired for Router human** when Status is present (there is no `last result:`). Do not keep a dead two-space lock. |
| **F33 — Hex rstest (Agy m1)** | AC3 **required red** rstest: `0x41306` / `0X41306` same vec as decimal `267014`; `0x41301` same as `267009`. Live `explain_last_task_result` already parses hex (`nightly.rs` `:960–961`) — Router human mapper must too (F25). Name: `format_router_status_lines__hex_0x41306__same_as_267014`. |
| **F34 — Whitespace-only Status (Agy m2)** | Helper already `status.map(str::trim).filter(\|s\| !s.is_empty())` (`:195`). AC3: `Some("   ")` + 267014 → `["Router: terminated"]` (same as `None`). Do **not** invent `Ready`. |

---

## 4. Acceptance criteria

| AC | Proof |
|----|-------|
| **AC1** | Unit **required red:** `format_router_status_lines(true, Some("Ready"), Some("267014"))` == `["Router: Ready", "last run: terminated"]`. Joined stdout contains **neither** `267014` **nor** `SCHED_S`. Const `ROUTER_LAST_RUN_TERMINATED` `assert_eq!`. Name: `format_router_status_lines__ready_267014__status_then_terminated_no_numeric`. |
| **AC2** | Unit **required red:** `format_router_status_lines(true, Some("Running"), Some("267009"))` == `["Router: Running"]`. No `267009`, no `SCHED_S`. **Rewrites** T255 AC6 human half. JSON half of that test still `last_result == "267009"` + existing hint string. Name: `format_router_status_lines__running_267009__status_only_no_numeric`. |
| **AC3** | Unit **required red:** blank Status + `267014` → `["Router: terminated"]`. Blank + `267009` → `["Router: running"]`. Whitespace-only Status `"   "` + 267014 → same as blank (F34). Hex `0x41306` / `0X41306` same vec as `267014`; `0x41301` same as `267009` (F33). Names: `format_router_status_lines__blank_status_267014__terminated_phrase`; `format_router_status_lines__hex_0x41306__same_as_267014`. **Rewrites** T255 AC15. |
| **AC4** | Unit: Ready + `0` → `["Router: Ready"]`. Ready + `1` → first `Router: Ready`, follow existing `explain_last_task_result("1")` string. `found == false` still `Router: not scheduled` with no `next:`. |
| **AC5** | Unit: `router_json_from_input` Ready + 267014 → `last_result == "267014"`, `last_result_hint` still `"task terminated (SCHED_S_TASK_TERMINATED)"`, `scheduled == true`. `FROZEN_KEYS` still present. |
| **AC6** | Existing T269: `nightly --help` still contains `AI-Brains-Nightly`, `267009` or `SCHED_S_TASK_RUNNING`, `750`, `TCP`, `/health`. **Additive** unit: help contains `267014` **or** `SCHED_S_TASK_TERMINATED`, and names success / not Nightly Last Result. Name: `nightly__help__names_router_267014_success`. |
| **AC7** | Existing `explain_last_task_result__267014__terminated` and `__267009__running_sched_s` stay green (Nightly decode). |
| **AC8** | Hermetic `tests/nightly_status.rs` `--quick` human: heading + `probe=skipped` + no T281 contrast. **Additive:** stdout does **not** contain `267014` and does **not** contain `SCHED_S_TASK_TERMINATED`. Exit **0**. JSON `--quick` still `probe == "skipped"`; **do not** assert host `router.last_result` (T255 AC9). Helper lock is **AC1** (unit `format_router_status_lines`); this hermetic is live-`schtasks` contains-not (OpenCode O1 already covered by AC1+AC8 — do not add a third copy). Keep T255 “AC10” / T269 “AC8” **comment numbers** in that file (T281 m-2 analog). |
| **AC9** | Manual (`cargo run -p ai-brains-cli -- --no-project-context nightly --status --quick`): still `Nightly:` separate from `Router:`. `Last task result: 0` present. Router human **does not** present `267014` (Status `Ready` + `last run: terminated` on this machine today). JSON `--format json --quick` still `router.last_result == "267014"` (pass-with-observed-data if LIST /V still 267014). Exit **0**. **Did not** mutate schtasks. |
| **AC10** | Docs: CAPABILITIES + OPERATIONS + CHANGELOG T296. CLI-EXIT-CODES Nightly-status paragraph names **both** `267009` (`SCHED_S_TASK_RUNNING`) and `267014` (`SCHED_S_TASK_TERMINATED`) as scheduler success → exit **0** (Agy O1). PROTOCOL-COMPAT no new required keys. |
| **AC11** | No production `unwrap`/`expect`/`panic`; no clap/rusqlite bump; no DTO keys; `NIGHTLY_STATUS_PROBE_TIMEOUT` still 750 ms; `daemon.rs` / `llama_cpp.rs` / `project.rs` / `sync.rs` / `forget.rs` / `doctor.rs` / `explain_last_task_result` absent from the product diff (except `nightly.rs` only if compile forces — then stop). |
| **AC12** | `--quick` still does not construct `LlamaCppProvider` (T247 F19). |
| **AC13** | Full gate at closeout: `scripts/dev-check.ps1` (fmt / clippy `-D warnings` / nextest / deny / audit). |
| **AC14** | Capture independence: no new events; `--quick` no models. |

---

## 5. Design notes

### 5.1 Human layout (this machine after T296)

```text
=== Nightly Status ===
Nightly: AI-Brains-Nightly
Scheduled: Yes (next run: 8/25/2026 3:00:00 AM)
Last task result: 0
Last scheduled run: 8/24/2026 3:00:02 AM
Last nightly run: …
Unsummarized sessions remaining: 0
Sessions summarized in last run: 3
Errors in last run: []
Completion: 127.0.0.1:8081  model=…  probe=skipped
Embedding: 127.0.0.1:8083  model=…  probe=skipped
Router: Ready
last run: terminated
Multi-import: …
======================
```

JSON (unchanged keys):

```json
"router": {
  "scheduled": true,
  "status": "Ready",
  "last_result": "267014",
  "last_result_hint": "task terminated (SCHED_S_TASK_TERMINATED)",
  "task_to_run": "C:\\llm\\router.bat"
}
```

### 5.2 Helper sketch (in `nightly_status.rs`)

Keep `format_router_status_lines(found, status, last_result)`. Private parse of decimal/hex. Match on `(status, code)` per F1–F4. Do not call `explain_last_task_result` for 0 / 267009 / 267014 on the human path.

### 5.3 after_help (additive; T269 sentence stays)

Keep: `Nightly Last Result is AI-Brains-Nightly. Router 267009 is SCHED_S_TASK_RUNNING (success; ONLOGON keep-alive).`

Add: `Router 267014 is SCHED_S_TASK_TERMINATED (success; last run ended), not Nightly Last Result.`

### 5.4 Why not omit the follow line entirely?

Placeholder allowed “omit the numeric when Ready” **or** “label terminated/Ready without the raw code.” Omitting all last-run info would hide that the keep-alive last ended. `last run: terminated` without HRESULT is the clig just-enough remediator. JSON still has the code.

### 5.5 T255 AC6/AC15

Those units **are** the red tests. Rewrite in place (same functions, new expected vec). Do not leave a second copy that still asserts `last result: 267009`.

---

## 6. Non-goals

- Raise 750 ms / persist probe / doctor 16th / product `.cmd` / schedule-Router / `--quick --no-vault`
- Unify daemon TCP with HTTP `/health` (T297)
- JSON schema bump / new `router` keys / rewrite `last_result_hint`
- Edit `explain_last_task_result` or Nightly `Last task result:` line
- Mutate live tasks; `/Run` Router to clear 267014
- clap 5 / rusqlite 0.40 / workspace 0.1.3
- T298–T300 / leftover `--write` / T240 F2 / T263 H2
- `cargo install`

---

## 7. Verification plan (TDD)

**Red first (must fail on current tree):**

1. `format_router_status_lines__ready_267014__status_then_terminated_no_numeric`
2. Rewrite `format_router_status_lines__running_267009__router_and_hint_on_following_line` → `__status_only_no_numeric`
3. Rewrite `format_router_status_lines__blank_status_267009__last_result_only` → running phrase
4. `format_router_status_lines__blank_status_267014__terminated_phrase` (F34 whitespace sibling in same test or rstest)
5. `format_router_status_lines__hex_0x41306__same_as_267014` (F33)
6. `nightly__help__names_router_267014_success`

**Then green:** helper body + after_help sentence + docs.

**Stay-green:** AC4 not-scheduled, AC5 JSON, AC6 T269 needles, AC7 decode, AC8 hermetic heading/`--quick`, T281 contrast units, T247 `Last task result: 101`.

**Manual:** AC9 classify-only. Pass-with-observed-data if LIST /V last_result is no longer 267014 (still: no decimal HRESULT on human Router).

---

## 8. Risk

| Risk | Mitigation |
|------|------------|
| Operators who grepped `267014` on human lose the token | JSON + after_help + docs keep it. Human is Status + `last run: terminated`. |
| T255 AC6/AC15 rewrite looks like a regression | Spec F3/F29 + review.md cite supersede. JSON asserts stay. |
| Hermetic `--quick` spawns live `schtasks` | Do not assert host `router.last_result`. Do assert human stdout lacks `267014` / `SCHED_S_TASK_TERMINATED` (Nightly result is 0). |
| Editing `explain_last_task_result` breaks Nightly 1/101 | F6 — do not touch. |
| Growing `nightly.rs` (2128) | F9 — helper only. |
| Hotspot `project.rs` | Do not touch. |

---

## 9. Deferred absorb / decline

**Entire `conductor/deferred.md` scanned** (T142 archive through T295 closeout + T285–T300 mint). Overlapping open rows:

| Item | Disposition |
|------|-------------|
| Audit / mint “nightly Router 267014 / TASK_TERMINATED” | **Absorb** F1–F7 / AC1–AC3 / AC6 / AC9 |
| Placeholder Manual `nightly --status --quick` | **Absorb** AC9 |
| T269 “do not restyle Router” | **Supersede human only** F1–F3; JSON + heading + Nightly Last Result **affirm** |
| T255 AC6/AC15 human numeric | **Absorb / rewrite** AC2 / AC3 |
| T255 JSON keys / pipes human / `--quick` skip | **Affirm** F5 / F8 |
| T255 doctor 16th / persist / 50ms / `.cmd` / `--no-vault` | **Decline** F10 |
| T269 JSON `probe_budget_ms` | **Decline** F8 (T269 F21) |
| T281 750 raise / HTTP vs TCP | **Affirm freeze** F8; contrast already shipped |
| T255 F12 persist probe | **Decline** F10 |
| PATH until `cargo install` (T285–T295) | **Residual** F13 — not this chrome (PATH already has T281 Router numeric) |
| T297 daemon Stopped vs LLM Open | **Decline steal** F11 |
| T298–T300 | **Decline** F17 |
| T294 leftover `--write` | **Decline** F17 |
| T295 doctor remediator / keep-10 / residuals | **Decline** — Completed |
| T240 F2 / T263 H2 / clap 5 / rusqlite 0.40 | **Decline** F12 / F17 |
| last-PR Cursor **#211** | **N/A empty** — **no T301** F18 |
| Identity leftover `7d97a456` vs `fcb8a40f` | **Not this track** F27 |
| Closed T229/T247/T255/T269/T281 DoDs (heading, `--quick`, LIST /V, JSON schema) | **Stay closed** except T255 AC6/AC15 human as named |

---

## 10. Implement order (on go)

1. Phase 0 re-verify (plan.md) + BUGFIX TX.
2. Red AC1–AC3 + AC6 help unit.
3. Green `format_router_status_lines` + const.
4. after_help sentence; T269 AC6 stay-green.
5. Docs CAPABILITIES / OPERATIONS / CHANGELOG / CLI-EXIT-CODES.
6. Hermetic AC8 + stay-green T247/T269/T281.
7. Manual AC9 (read-only).
8. `scripts/dev-check.ps1`; Phase-1 review; `codex-review`.
9. conductor Completed + deferred closeout + pin.
10. Phase 6 publish (`track/T296-*` → PR → watch GHA `CI` green → squash-merge). Never `git push origin main`.

---

## 11. Soft residuals

| Residual | Notes |
|----------|--------|
| PATH until `cargo install` | F13 — T285–T295 still off PATH; this chrome is on PATH **until** go lands |
| Live Router Last Run 8/19 | Stale keep-alive; not a display bug. Do not `/Run` |
| JSON still shows `"267014"` + SCHED_S hint | Intentional F5 |
| Nightly `Last task result: 267014` if someone `/End`s Nightly | Out of scope; heading names the task |
| T297 daemon Stopped vs Open | Next placeholder |
| `--quick` still opens the vault | T255 F15 |

---

## 12. Touch map

| Path | Change |
|------|--------|
| `crates/ai-brains-cli/src/commands/nightly_status.rs` | F1–F4 helper + const + unit rewrite/add |
| `crates/ai-brains-cli/src/main.rs` | Nightly `after_help` additive + AC6 new test |
| `crates/ai-brains-cli/tests/nightly_status.rs` | AC8 additive contains-not 267014 / SCHED_S |
| `Docs/CAPABILITIES.md` | T269/T281 bullet additive |
| `Docs/OPERATIONS.md` | Router bullet |
| `Docs/CLI-EXIT-CODES.md` | 267014 next to 267009 exit 0 |
| `CHANGELOG.md` | T296 Unreleased |
| `conductor/conductor.md` / `deferred.md` / this spec+plan / README-T285-T300 | Planning now; Completed on go |

**Do not touch:** `nightly.rs` `explain_last_task_result` / schedule block; `doctor.rs`; `daemon.rs`; `project.rs`; `llama_cpp.rs`; `embeddings.rs`; `ai-brains-contracts`; `Cargo.lock`; PROTOCOL-COMPAT.

---

## 13. AI fold-in

Inputs (not edited): `agy-review.md` (HEAD `c7d6e3e`) + `opencode-review.md` (HEAD `c7d6e3e`). Fold-in on `main` (ahead of `origin/main` `8b95181` T295 `#211`). Live verify: `format_router_status_lines` **`:187–216`** still prints `last result: {code}` + `explain_last_task_result`; already **trims** status **`:195`**; `explain_last_task_result` hex `0x`/`0X` **`:960–961`**; after_help `main.rs` **`:1432–1434`** has 267009 sentence **no 267014** (F7 **does** add a sentence — OpenCode “no help change” is wrong); T269 AC6 **`:952–980`**; hermetic `tests/nightly_status.rs` **`:77–121`**. Hotspot `project.rs` **#1** (**3.897** fold-in; plan 3.906). Pins **snapshot — re-verify at execute** (clap lock 4.6.1 / crates.io 4.6.6; rusqlite 0.39.0; **no clap 5**). Last merged PR still **#211** (comments/reviews **empty**). **No T301.** Fold-in preflight: Pinned **4119** / in-context **0/0/0** / word **536** (plan 4102/367; OpenCode 4102/428 — volatile). Doctor **4** warn; :8083 **ok**; :8081 **ok** this pass (`gemma-4-E4B-it-Q5_K_M.gguf` — model tag volatile).

### Pins locked by fold-in

1. **F33 / AC3 (Agy m1):** rstest hex `0x41306` / `0X41306` / `0x41301` same vec as decimal.
2. **F34 / AC3 (Agy m2):** whitespace-only Status `"   "` ≡ blank → `Router: terminated` (live trim `:195`; do not invent `Ready`).
3. **F19 / AC10 (Agy O1):** CLI-EXIT-CODES names **both** 267009 and 267014 as `SCHED_S_*` success → exit 0.
4. **F7 / AC6:** after_help **does** add 267014 success sentence (decline OpenCode “no help change”).
5. **F9:** zero production edits in `nightly.rs` (decline OpenCode listing the print call site as a product change).

### Per-AI disposition

| Source | Item | Disposition |
|--------|------|-------------|
| Agy | B / M | None filed |
| Agy | **m1** hex parse in `format_router_status_lines` | **Folded** F33 / AC3 required red |
| Agy | **m2** blank/whitespace Status must not invent `Ready` | **Already** F2 / F26 / AC3; **tightened** F34 (`:195` trim) |
| Agy | **O1** CLI-EXIT-CODES both 267009 and 267014 success | **Already** F19 / AC10; **tightened** AC10 SCHED_S names |
| Agy | **O2** help unit `nightly__help__names_router_267014_success` | **Already** F7 / AC6 |
| OpenCode | B / M | None filed |
| OpenCode | **m1** plan HEAD `8b95181` vs `c7d6e3e` | **Snapshot** — preflight refreshed; not DoD |
| OpenCode | **m2** word 367→428 | **Snapshot** — fold-in 536 / pin 4119; not DoD |
| OpenCode | **m3** `daemon status` Stopped is T297 | **Already** F11 |
| OpenCode | **O1** hermetic/unit lock HRESULT omission | **Already** AC1 (unit helper) + AC8 (hermetic contains-not). OpenCode mixed T255 AC9/AC10/AC14 comment numbers with this track’s ACs. Do **not** add a third copy. |
| OpenCode | **O2** keep `Ready` state word | **Already** F1 / AC1 `["Router: Ready", "last run: terminated"]` |
| OpenCode | “What Looks Solid” #5 no help change | **Decline** — F7 / AC6 after_help additive **is** DoD. Today’s after_help has no 267014; that is the hole. |
| OpenCode | summary lists `nightly.rs` print site as a product change | **Decline** — F9: call site already correct; zero production `nightly.rs` |
| both | last-PR #211 Cursor | **Affirm F18** — no T301 |
| both | deferred T297–T300 / T240 F2 / 750 ms / JSON freeze | **Affirm** |

No Blockers. No Majors. No new placeholder minted. Do **not** edit `*-review.md`.

