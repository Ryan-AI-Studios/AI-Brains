# T291 — `query trace` missing must not be a bare `null`

- **Track ID:** T291-QueryTraceNext
- **Status:** **Planned** (Pending until **go**)
- **Category:** FEATURE / UX / HONESTY
- **Owner:** Grok
- **Source:** Audit 2026-08-22 — `query trace` **3/8**; friction “null with no next.” Placeholder minted with T285–T300 (`76c4db9`). T274–T284 declined the surface as honest-empty E=8; **reopened** because U&lt;8. T263 ✅ F6 / F26 **froze** scalar `null` (P-CLI) — **this track lifts that freeze** with a documented missing-only envelope (not `{trace:null}` wrapping of `QueryTraceDto`). T202 ✅ F31 project-id gate still excluded. T152 ✅ progressive `dry_run` default **true** (no `QueryTraceRecorded`). T290 ✅ lists/progressive next (do **not** steal).
- **Depends on:** T152 ✅ `get_query_trace` / `QueryTraceRecorded`; T202 ✅ F31 no project-id; T221 ✅ exit 0 vs 3; T257 ✅ `print_json_stdout`; T263 ✅ F6 freeze (lifted here); T180 ✅ P-CLI type-change checklist
- **Blocks / feeds:** Operators who run `query trace <id>` (including `missing-id`) see a **copy-paste** `query progressive … --dry-run false` instead of a JSON token that looks like a bug. Policy-check human remains **T292**. Neighbors **T293**. Forget-list **T299**.
- **Absorbs:** Placeholder problem text + Manual DoD two commands; deferred.md “`query trace` bare `null`”; T263 F6/F26 wrap-decline **reopened**; last-PR Cursor **#206** Bugbot Low `sanitize_recall_query` interpolator collapse
- **Not absorbed (DoD):** Inventing traces; `{trace: QueryTraceDto}` wrap of **found** path; progressive `--dry-run` default flip; invent `--trace` on progressive; QueryTraceDto new fields; daemon GetQueryTrace RPC; T292/T293/T299; T240 F2; T263 H2; clap 5 / rusqlite 0.40
- **Research date:** 2026-08-23 (plan dogfood HEAD `37012fe` T290 `#206`; fold-in HEAD `ff61399` on `track-t95-in-force`; `origin/main` still `37012fe`. Product `src/` = T263 `println!("null")` + T152 dry-run skip persist; T95 did **not** touch `run_trace`. PATH **0.1.2** 2026-08-22 19:41 **without** T285–T290 — hole is in **source and PATH**)
- **AI fold-in:** 2026-08-23 `agy-review.md` (`e81a1a2`) + `opencode-review.md` (`560f3b0`). **Agy B 0 / M 0.** **OpenCode B 0.** OpenCode M-1 hotspot rank **folded**. M-2/M-3 volatile snapshot refresh (not DoD). **Agree:** Agy m1 + OpenCode o-1 clap `value_parser` for AC7 (not `OutputFormat::parse`); Agy m2 sanitizer space-boundary (already F16, tightened); Agy O1 parent+Trace after_help (already F14, line pointers); Agy O2 / OpenCode o-2 AC5 bootstrap+persist; OpenCode o-6 OPERATIONS null phrases in AC10. **Decline:** OpenCode o-5 extra `cli_help_ia.rs` T204 lock (AC10 covers `query trace --help`). **No declines of B.** Disposition **§13**.
- **Ledger:** planning DOCS TX `c59e5bb6-adf1-40c5-9288-66403d208aca`. Fold-in DOCS TX `627d3871-b5c6-4e03-8b11-9588a61777d1`. Implement starts a **FEATURE** TX on **go**.
- **Isolation:** Do **not** `cargo install`. Do **not** pin production decisions to the live vault as implement. Do **not** rewrite `.env`. Do **not** live `policy bootstrap`. Do **not** flip progressive `dry_run` default. Do **not** grow hotspot `project.rs` / `sync.rs` / `forget.rs` / CLI `preflight.rs` / `personal.rs` / `briefing.rs`. Grow `governed_common.rs` **only** for sanitizer collapse (hotspot **#2** at 3.806 — Bugbot #206). Missing envelope lives in `governed_query.rs` (not top-10). Do **not** print or commit `AI_BRAINS_KEY`.

---

## 1. Objective

1. **Missing `query trace` is a remediator, not a bug token.** Default JSON for `get_query_trace` `None` (missing **or** unauthorized) is a CLI-local envelope with `found: false` + copy-paste `next_step`. Exit **0** stays. Agents must not stop at the JSON token `null`.
2. **Human empty is two lines.** `--format human` (and `auto` on TTY) prints `No trace for <id>.` plus `next: ai-brains query progressive "what did we decide" --dry-run false`. Found traces stay `QueryTraceDto` JSON (no human DTO dump).
3. **Do not invent traces. Do not leak existence.** Unauthorized and missing stay the **same** envelope (`found: false`; no `denied` / no distinct reason). CP `Ok(None)` stands.
4. **North star.** Capture independence: CLI overlay on a projection read. No new events. No hidden CoT. Progressive JSON `query_trace_id` unchanged. T152 dry-run default **true** stays (write-nothing); next_step tells the operator how to persist.

This unblocks the unused surface: T152 records traces only when `--dry-run false`; T263 documented `null` as honest empty-success; the 2026-08-22 audit still scores **3/8** because a scalar `null` is not a command and does not say traces come from progressive persist.

---

## 2. Live baseline (re-scan 2026-08-23)

### 2.1 Operator dogfood (this machine)

| Signal | Observation |
|--------|-------------|
| HEAD | Fold-in `ff61399` (`track-t95-in-force`; T95 in-force CLI). T291 plan `e81a1a2`. `origin/main` = `37012fe` T290 `#206`. Tree **CLEAN**. T95 did **not** change `run_trace`. |
| PATH `ai-brains` | `C:\Users\RyanB\.cargo\bin\ai-brains.exe` mtime **2026-08-22 19:41**, 25 139 712 bytes, **0.1.2**. Has T274. **Does not have T285–T290.** Trace `null` is in **source + PATH**. **Do not `cargo install`.** |
| `preflight --summary` | Scope `C:\dev\ai-brains` (`3581317d`). Pinned **4019** (volatile; plan-time 3976 / OpenCode 367 words). In-context **0/0/0**. Word **689**. Grants omitted (live **3 of 3**). Phase 0 re-verify. |
| `query trace missing-id` | stdout exact token `null` + newline. Exit **0**. **This is the 3/8 hole.** |
| `query trace missing-id --format human` | clap **unexpected argument** `--format`; tip suggests global `--log-format`. **No `--format` on Trace today.** |
| `query trace --help` | “null when missing or unauthorized”; after_help “JSON token null and exit 0 (not an object).” |
| `query progressive` clap | `--dry-run` default **true** (`ArgAction::Set`). Default progressive **does not** append `QueryTraceRecorded`. |
| Last GitHub PR | [#206](https://github.com/Ryan-AI-Studios/AI-Brains/pull/206) T290 (2026-08-23). Cursor Bugbot **1 Low**: `sanitize_recall_query` dropping `$`/backtick sets `prev_space = false` so `a $ b` → `a  b`; interpolator+spaces can yield a whitespace-only needle instead of `LIST_RECALL_QUERY`. **Absorb** F16 / AC8. Open PRs: Dependabot remotes only (`#61` rusqlite 0.40.2, `#62` chrono 0.4.45, `#59` tokio, `#60` thiserror, `#58` tower-http, actions `#68–#72`). **No T301.** |
| Identity / doctor | ledgerful doctor **4** warn at fold-in (legacy `.changeguard` / sig-pin / sig-version / timings; OpenCode saw 6 including impact-stale — **volatile**, Phase 0). Optional :8081 unreachable. **0 pending / 0 drift.** Hotspot **#1** `project.rs` (3.941) — **do not touch.** `governed_common.rs` **#2** (3.806) — sanitizer collapse only (OpenCode M-1). `governed_query.rs` **not** top-10 — envelope lives here. CLI `preflight.rs` #8 (2.159). |
| `ISSUES.md` | **Does not exist.** |

### 2.2 Why `null` still trains “broken / unused”

| Layer | Truth |
|-------|--------|
| Dual-shape already | Found → pretty `QueryTraceDto`. Missing → compact token `null`. Scripts already branch on null vs object. |
| T263 F6 shipped | Docs + clap after_help + hermetic `assert_eq!(trimmed, "null")`. **Honest and useless.** |
| T263 declined `{trace:null}` | Wrapping **found** as `{trace: …}` would break `.query_trace_id` at top level. This track does **not** wrap found. Missing-only envelope is a different type. |
| T152 dry-run default | `persist_trace` returns immediately when `dry_run` (`query.rs:783–785`). Default CLI progressive therefore **never** fills `query_trace_projection`. Tracing the `query_trace_id` from a default progressive packet is still `None`. Next **must** name `--dry-run false`. |
| No `--trace` flag | Placeholder guessed `query progressive "…" --trace`. **Does not exist.** Do not invent it. |
| No project-id gate | T202 F31: missing `AI_BRAINS_PROJECT_ID` still exit **0**. **Keep.** |
| No daemon RPC | `ai-brainsd` has no GetQueryTrace op. CLI-local only. Do not mint a contracts DTO for daemon. |
| Unauthorized = missing | CP returns `Ok(None)` for no-row, cross-principal, and no-read-grant (`query.rs:443`, `:463–488`). Envelope must not distinguish. |
| PROTOCOL-COMPAT §5 | **No row** for `query trace` today. Adding a row is the T180 honesty for this type change. |
| T274–T284 decline | Series called U=4 “the surface, not a defect.” T285–T300 reopened U&lt;8. |

### 2.3 Code truth (opened)

| Item | Location | Notes |
|------|----------|-------|
| CLI None arm | `governed_query.rs` `run_trace` `:205–233` | `Some(t)` → `print_json_stdout`; `None` → `note_machine_stdout(); println!("null")`. **Replace None arm.** |
| Options | `TraceOptions { trace_id: String }` `:39–41` | Add `format: String`. |
| clap | `main.rs` `Trace { trace_id: String }` `:1922–1926` | No `--format`. after_help documents null. Dispatch `:3961–3966`. Add T266 `value_parser` (precedent `:1195` / `:1943` / `:2383`) — **not** `OutputFormat::parse`. |
| Query after_help (null sentences) | `main.rs` **`:1589`** (Commands::Query) + **`:1892`** (enum) + **`:1924`** (Trace) | All three still say “JSON token null”. F14/AC10 rewrite **all three**. |
| CP | `get_query_trace` `query.rs:413` → `Result<Option<QueryTraceDto>>` | **Do not change.** |
| Persist | `persist_trace` `:769` | `dry_run` writes nothing. |
| DTO | `QueryTraceDto` `briefings.rs:469–486` | Required `api_version`, `query_trace_id`, `scope`, `principal`, `query`, `applied_policy`, … **Do not add `found` / `next_step`.** |
| Progressive DTO | `query_trace_id: String` required | **Unchanged.** Dry-run still emits an id that is **not** in the projection. |
| Hermetic null | `governed_vault_pin_honesty.rs` `query_trace__unknown__stdout_null_exit_0` `:206–232` | Exact `"null"`. **Replace** with envelope AC. |
| Hermetic no-project | `recall_briefing_clarity.rs` `query_trace__missing_project__still_exit_0_null` `:96–122` | Exit 0 + contains null. **Keep exit 0; envelope not token.** |
| Sanitizer | `governed_common.rs` `sanitize_recall_query` `:60–85` | `$`/`` ` `` drop sets `prev_space = false`. Tests `:868` `echo $(hi)` → `echo (hi)`; **no** `a $ b` case. |
| Format enum | `OutputFormat::parse` `:352–357` | **Case-insensitive** + unknown → **Json**. **Cannot** implement AC7. Clap `value_parser` rejects `JSON`/`Pretty`. After clap accepts a token, map json→envelope; human/pretty/text/markdown/md→F2; `auto`→`stdout().is_terminal()` (briefing.rs:275 analog). |
| Emit | `emit_json` = `print_json_stdout` (pretty) | Envelope uses this. Human uses `emit_human` / two `println!`. |

### 2.4 Dependency / standards research (2026-08-23)

| Pin | Workspace / lock | Action |
|-----|------------------|--------|
| `clap` | workspace **4.5** / lock **4.6.1** / crates.io **4.6.6** (GitHub **v4.6.6** 2026-08-06) | **No bump.** clap **5** not current. Snapshot — re-verify at execute. |
| `serde_json` | workspace **1.0** / lock **1.0.150** / crates.io **1.0.151** (2026-07-20) | **No bump.** |
| `chrono` | lock **0.4.44** / crates.io **0.4.45** (`#62`) | **No bump.** |
| `rusqlite` | lock **0.39.0** / crates.io **0.40.2** (`#61`) | **No bump.** |
| `thiserror` | lock **2.0.18** / crates.io **2.0.20** (`#60`) | **No bump.** |
| `tokio` | lock **1.52.3** (`#59`) | **No bump.** |
| rustc / edition / nextest | **1.95.0** / **2024** / **0.9.140** | Unchanged. |
| workspace version | **0.1.2** | **No bump.** |
| New crates | — | **Zero.** |
| [clig.dev](https://clig.dev/) (fetched 2026-08-23) | Human-first; suggest the next command; stdout for primary output; JSON when `--json` / scripts; keep changes additive; changing human output is usually OK; machine JSON is an interface — type change needs a note. | Missing default JSON **must** carry `next_step` or agents stay at U=3. Human two-line + next. Found JSON keys frozen. |
| T180 P-CLI §8 | Prefer additive keys; type change → dual fixtures **or** documented lift + tests. Compact↔pretty switch is breaking unless flagged. | Missing `null` → pretty envelope is a **documented type change**. Found path pretty `QueryTraceDto` **unchanged**. Add §5 row. |
| T263 F6 | Scalar null frozen | **Lift** with track note. Not `{trace:null}`. |
| JSON null vs object | Industry: empty collection → `[]` not `null`; scalar null means “no document.” A CLI that always prints JSON for scripts should return an object agents can key. | Envelope `found: false` is the object form of “no document” **with** a remediator. |

Training data is not a pin. Re-verify clap/serde_json at execute.

---

## 3. Frozen decisions

| ID | Decision |
|----|----------|
| **F0 — Go gate** | Plan-only until user **go**. Planning is DOCS. Implement starts a FEATURE TX. |
| **F1 — Lift T263 F6 for missing only** | `None` stdout is **not** the JSON token `null`. Default/`--format json` is a pretty CLI-local envelope (F7). Found `Some(QueryTraceDto)` stays top-level DTO JSON via `print_json_stdout`. Do **not** wrap found as `{trace: …}`. |
| **F2 — Human missing** | `--format human` / `pretty` / `text` / `markdown` / `md`: exactly two stdout lines. Line 1 `No trace for {id}.` Line 2 `next: ai-brains query progressive "what did we decide" --dry-run false`. `{id}` is sanitized (F15). No JSON object. |
| **F3 — `--format` default json** | clap `default_value = "json"` + **`value_parser = ["auto", "pretty", "human", "text", "json", "markdown", "md"]`** (T266 set; **case-sensitive**; `JSON`/`Pretty` clap InvalidValue exit **2**). Do **not** route AC7 through `OutputFormat::parse` (`:352` lowercases and maps unknown → Json). After clap accepts a token: `json` → envelope/DTO; `human`/`pretty`/`text`/`markdown`/`md` → F2; `auto`: TTY human / pipe json via `std::io::stdout().is_terminal()`. Found path **ignores** format (always QueryTraceDto JSON) — F10. |
| **F4 — Exit 0** | Missing/unauthorized stays exit **0**. Not exit 4 `NOT_FOUND`. Not exit 3 (no existence leak via deny). Progressive/expand Denied stays 3. |
| **F5 — T202 F31 project-id** | `query trace` still does **not** require `--project-id` / env. Missing project still envelope + exit 0. |
| **F6 — Existence freeze** | Missing row, cross-principal, and no-read-grant are indistinguishable. Envelope has **no** `denied` / `denial_reason` / `unauthorized`. CP `Ok(None)` unchanged. |
| **F7 — Envelope shape (SOOT)** | Pretty JSON object, keys **exactly**: `api_version` (briefings `"1"`), `found` (boolean `false`), `trace_id` (sanitized requested id), `next_step` (F8). No extra required keys. Not a `QueryTraceDto`. Type lives in `governed_query.rs` (not contracts). |
| **F8 — `next_step` exact** | Const `TRACE_MISSING_NEXT_STEP` in `governed_query.rs`: `No persisted trace. Run: ai-brains query progressive "what did we decide" --dry-run false` (one line, no U+2026, no newline). Human line 2 is `next: ` + the same command substring `ai-brains query progressive "what did we decide" --dry-run false`. Must contain `query progressive` and `--dry-run false`. Must **not** invent `--trace`. Must **not** lead with `recall` (T290 lists already do vault search). |
| **F9 — Progressive dry-run default frozen** | clap `default_value_t = true` stays. Do **not** persist traces by default. DoD is the remediator, not a write-path flip. |
| **F10 — Found path frozen** | `QueryTraceDto` fields unchanged. `--format human` on a **found** id still prints QueryTraceDto JSON (no human renderer this track). Soft residual §11. |
| **F11 — No contracts / daemon** | Do not add GetQueryTrace to daemon protocol. Do not add envelope fields to `QueryTraceDto`. PROTOCOL-COMPAT documents **P-CLI** stdout only. |
| **F12 — No invented traces** | Do not insert `QueryTraceRecorded`. Do not fabricate ranking/handles. Projection read only. |
| **F13 — Tests** | Naming `function_or_feature__condition__expected_result`. No `unwrap`/`expect`/`panic` in production. Hermetic `tempfile::tempdir`. Replace null ACs; do not leave a green `assert_eq!(…, "null")`. |
| **F14 — Docs** | CAPABILITIES Trace row; OPERATIONS empty-vs-deny sentence; CLI-EXIT-CODES (exit 0 + optional `--format`); clap after_help (query parent + Trace); PROTOCOL-COMPAT §5 new row + §3.1 briefings note (CLI-local; DTO unaugmented). Skill one-liner if a query-trace sentence exists. |
| **F15 — Displayed `trace_id` sanitize** | Trim, collapse ASCII whitespace **including** around dropped `$` / backtick (interpolators count as whitespace — F16), `"` → `'`, cap 80, no newline. Empty after sanitize → `<empty>`. Used in envelope `trace_id` and human line 1. |
| **F16 — #206 Bugbot (absorb)** | `sanitize_recall_query`: dropping `$` / backtick **counts as a space boundary** (do **not** set `prev_space = false`). No double-space runs (`a $ b` → `a b`, not `a  b`). Final `trim` after collapse. Whitespace-only after drop → `LIST_RECALL_QUERY`. rstest `a $ b` → `a b`; `$ $` → default needle. Share collapse helper if that avoids duplicating the bug in F15. Hotspot `governed_common.rs` **#2** — **only** this change. |
| **F17 — PATH** | Soft. Source/hermetic SoT. Do not `cargo install` as implement. |
| **F18 — Capture independence** | No models, embeddings, graph, or new events. |
| **F19 — Isolation hotspots** | Do not grow `project.rs` / `sync.rs` / `forget.rs` / `preflight.rs` / `personal.rs` / `briefing.rs` / CP `query.rs` persist path. |
| **F20 — `run_trace` errors** | CP `Err` may stay `?` (pre-existing). Do not expand into fail_cp unless a red test requires it. None-arm is DoD. |
| **F21 — Identity stdout** | JSON envelope calls `print_json_stdout` / `note_machine_stdout` (T257). Human path does **not** mark machine JSON. |
| **F22 — ISSUES.md** | Does not exist. Debt is `deferred.md`. |
| **F23 — Decline peers** | T292 policy-check human; T293 neighbors; T294 leftover; T298 device; T299 forget-list. |
| **F24 — Standing declines** | T240 F2; T263 H2; 750 ms; clap 5; rusqlite 0.40; DTO new required keys. |
| **F25 — No T301** | #206 leftover **fits this track** (F16). Dependabot remotes are not tracks. |
| **F26 — Const freeze for T290 lists** | `PROGRESSIVE_RECALL_FALLBACK` ellipsis **unchanged** (T290 F8 deny stderr). F16 must not retune list next_step shape except interpolator collapse on operator needles. |

---

## 4. Acceptance criteria

| ID | Criterion |
|----|-----------|
| **AC1** | Unit: `TRACE_MISSING_NEXT_STEP` **exact** `assert_eq!` F8 string; `!contains('\n')`; `!contains('…')`; contains `query progressive` and `--dry-run false`; does **not** contain `--trace`. **Required red.** |
| **AC2** | Hermetic: `query trace missing-id` (default json) exit **0**; stdout trim is **not** `null`; `serde_json` object `found == false`, `api_version == "1"`, `trace_id` contains sanitized `missing-id`, `next_step` equals F8 const. |
| **AC3** | Hermetic: `query trace missing-id --format human` exit **0**; stdout is **not** a JSON object (no leading `{`); contains `No trace`; contains `next:`; contains `query progressive` and `--dry-run false`; two lines (one `\n` separating, no extra blank). |
| **AC4** | Hermetic: `query_trace__missing_project__still_exit_0_*` still exit **0** without project id; stdout is envelope (AC2 keys), not token `null`. |
| **AC5** | Hermetic found: `policy bootstrap --scope Repository:<uuid> --format json` **omit `--principal-id`** (System = `cli_principal()`, precedent `governed_first_run_deny_exit.rs:193–212`) then `query progressive "what did we decide" --dry-run false` (do **not** reuse `progressive_cmd` — it hardcodes `"x"` and default dry-run). Parse `query_trace_id`; `query trace <id>` → object with that `query_trace_id` and **no** `found: false`. Bootstrap’s ReadEvidence/ReadConclusions/ReadDecisions satisfy `get_query_trace` (any of the three). `--format human` on that id still JSON DTO (F10). |
| **AC6** | Stay-green: T221/T290 progressive/list tests still pass. `PROGRESSIVE_RECALL_FALLBACK` exact ellipsis (F26). |
| **AC7** | clap `value_parser`: `query trace x --format JSON` InvalidValue exit **2** (stderr clap, not `OutputFormat::parse`). `--format json` works. Unknown flag other than format unchanged. |
| **AC8** | rstest sanitizer: `a $ b` → `a b`; interpolator-only/whitespace → `LIST_RECALL_QUERY`; existing `echo $(hi)` / tab / newline / quotes / empty / 80-cap stay green. **#206.** |
| **AC9** | Unit: displayed id with newline/`$` is single-line and has no `$` / backtick / `"`. |
| **AC10** | Docs + help **must not** retain the scalar-null contract. Rewrite: CAPABILITIES Trace row; OPERATIONS phrases currently “empty-success `null` when missing” and “stdout JSON token `null`”; CLI-EXIT-CODES “query trace is excluded”; PROTOCOL-COMPAT §5 new row; clap after_help at `main.rs` **`:1589` / `:1892` / `:1924`**. Hermetic: `query trace --help` stdout does **not** contain `JSON token null`. Describe envelope + human next. Do **not** require a new `cli_help_ia.rs` T204 lock (OpenCode o-5 declined). |
| **AC11** | Manual (on go, `cargo run -p ai-brains-cli --`, no `--daemon`): `query trace missing-id` and `query trace missing-id --format human`. Pass = AC2/AC3 on this vault. Unique canary pin **not** required. **Do not** `cargo install`. |
| **AC12** | `QueryTraceDto` serde: no new fields in `briefings.rs`. Contracts golden if any still parse. |
| **AC13** | Unauthorized indistinguishability: hermetic vault with a persisted trace + different principal (or no grants) still AC2 envelope (not DTO, not deny exit 3). May reuse CP fixture pattern via CLI `--no-project-context` temp vault without grants: unknown id is enough if AC5 covers found. If CLI cannot easily mint cross-principal, document CP test stays SoT for `Ok(None)` and CLI only locks `None` → envelope. |

---

## 5. Design notes

### 5.1 Why not `{trace:null}`

T263 declined wrapping because scripts that read `.query_trace_id` on found would break if found became `{trace: {query_trace_id, …}}`. Missing-only envelope keeps found as `QueryTraceDto`. Discriminator: `found == false` vs presence of `ranking_json` / `query_trace_id` without `found`.

### 5.2 Why next is progressive `--dry-run false`

Default progressive already prints a `query_trace_id` that is **not** stored. Telling operators to `query trace <that id>` without `--dry-run false` would bounce. F8 is the copy-paste that actually fills the projection. After that, they use the packet’s `query_trace_id`.

### 5.3 Envelope is CLI-local

No daemon GetQueryTrace. Putting the type in `ai-brains-contracts` would imply a wire DTO. Keep a private serde struct next to `run_trace`. PROTOCOL-COMPAT §5 describes stdout, not P-DAEMON.

### 5.4 Sanitizer share (F16)

```text
// governed_common.rs — treat $ / ` as whitespace; trim end
pub fn sanitize_recall_query(raw: &str) -> String  // empty → LIST_RECALL_QUERY

// governed_query.rs
fn sanitize_trace_id(raw: &str) -> String          // empty → "<empty>"
```

Prefer a tiny shared collapse so F15 cannot reintroduce the Bugbot. Do not import `QueryStore` into `governed_common.rs`.

### 5.5 clap `value_parser` vs `OutputFormat::parse` (Agy m1 / OpenCode o-1)

`OutputFormat::parse` (`governed_common.rs:352–357`) lowercases and maps anything unknown to **Json**. Wiring Trace `--format` through it would make `--format JSON` succeed as json and fail AC7. Trace clap must use the T266 `value_parser` list (same as nightly `:1195`, scope `:1943`, retention `:2383`). Mapping after clap is a local match on the accepted token plus `is_terminal` for `auto`.

### 5.6 AC5 persist + read grants (OpenCode o-2)

`get_query_trace` returns `Ok(None)` without ReadEvidence **or** ReadConclusions **or** ReadDecisions on the stored scope. T210 bootstrap issues exactly those three (LocalOnly). Hermetic found path: bootstrap System (omit `--principal-id`) then `--dry-run false` so `persist_trace` appends `QueryTraceRecorded`. Do not reuse `progressive_cmd` (`:83` query `"x"`, default dry-run).

---

## 6. Non-goals

- Invent / persist traces on default progressive
- Flip `--dry-run` default
- `--trace` flag on progressive
- Wrap found `QueryTraceDto`
- Human renderer for found traces (soft)
- QueryTraceDto / ProgressiveQueryResponse new keys
- Daemon GetQueryTrace
- T292 `policy check` human
- T293 graph neighbors
- T294 leftover upsert
- T299 forget-list
- T240 F2 / T263 H2 / clap 5 / rusqlite 0.40
- `cargo install` / `.env` write / live extra bootstrap
- Growing `project.rs` / `preflight.rs` / `briefing.rs` / `personal.rs` / CP persist

---

## 7. Verification plan (TDD)

**Red first:**

1. `trace_missing_next_step__frozen__exact_string` (AC1)
2. `query_trace__unknown__stdout_envelope_exit_0` replacing `…_null_…` (AC2)
3. `sanitize_recall_query__interpolator_spaces__collapses` (AC8)

Then human AC3; no-project AC4; found AC5; clap AC7; id sanitize AC9; docs AC10; Manual AC11; DTO AC12.

---

## 8. Risk

| Risk | Mitigation |
|------|------------|
| Scripts parse exact `null` | Documented P-CLI lift; AC2 forbids token; CHANGELOG + PROTOCOL-COMPAT §5; after_help |
| Existence leak via `denied` | F6 / no deny keys / exit 0 |
| Operators still dry-run then trace | F8 names `--dry-run false`; AC1/AC3 |
| Invented `--trace` from placeholder | F8 / AC1 forbid |
| Hotspot `governed_common.rs` | F16 sanitizer only; envelope in `governed_query.rs` |
| Found human surprise | F10 documented; soft §11 |
| `$` in displayed id | F15 / AC9 |
| T290 list next_step drift | F26 ellipsis const + AC6 |

---

## 9. Deferred absorb / decline

| Item | Disposition |
|------|-------------|
| Audit `query trace` bare `null` U=3 | **Absorb** F1–F8 / AC1–AC4 / AC11 |
| Placeholder Manual two commands | **Absorb** AC11 |
| Placeholder JSON `null` **or** wrap | **Absorb envelope** F1/F7 — not `{trace:null}` |
| Placeholder `--trace` on progressive | **Decline** — flag does not exist (F8) |
| T263 F6 / F26 scalar freeze | **Lift** F1 with track note |
| T202 F31 no project-id | **Affirm** F5 / AC4 |
| T152 dry-run default true | **Affirm** F9 |
| T290 lists/progressive next | **Decline** — Completed `#206` |
| T292 policy-check human | **Decline → T292** |
| T293 neighbors dump sessions | **Decline → T293** |
| T294 leftover dest-missing | **Decline → T294** |
| T298 device/replicate empty | **Decline → T298** |
| T299 forget-list empty | **Decline → T299** |
| T240 F2 / T263 H2 / clap 5 / rusqlite 0.40 | **Decline** F24 |
| last-PR Cursor **#206** Bugbot Low sanitizer collapse | **Absorb** F16 / AC8 |
| Identity leftover `7d97a456` | **Not this track** — T258 / T294 |
| Open T292–T300 | **Not related** except named declines |
| Closed T274–T290 | **Stay closed** (T263 F6 is the intentional lift) |
| Dependabot `#58–#72` | **Not this track** |
| T142 #4 archive specs / audit allowlist / connector cursor / … | **Not related** (no query-trace overlap) |

---

## 10. Implement order (on go)

1. Phase 0 re-verify HEAD / deferred / #206 Bugbot still true / live `query trace missing-id` still `null`
2. FEATURE TX
3. Red AC1 / AC8 / AC2
4. Green: F8 const + None-arm envelope + `--format` clap; sanitizer collapse
5. Red/green AC3 human; AC4 no-project; AC7 clap `value_parser` (not `OutputFormat::parse`)
6. AC5 found persist (System bootstrap + `--dry-run false`; not `progressive_cmd`)
7. Stay-green AC6 / AC12
8. Docs AC10
9. Clippy + nextest + deny/audit
10. Manual AC11
11. Phase-1 review → codex-review
12. Publish: push `track/T291-*` → PR → watch GHA `CI` green → squash-merge → prune

---

## 11. Soft residuals

| Residual | Notes |
|----------|-------|
| PATH until `cargo install` | F17 |
| Found `--format human` still JSON | F10 — human DTO dump is a future track if wanted |
| Default progressive still does not persist | F9 by design |
| CP `Err` still `?` | F20 |
| No daemon GetQueryTrace | F11 |
| T292–T300 | Not stolen |
| `cli_help_ia.rs` query-trace after_help lock | OpenCode o-5 — declined; AC10 hermetic `--help` is enough |

---

## 12. Touch map

| Path | Change |
|------|--------|
| `crates/ai-brains-cli/src/commands/governed_query.rs` | Envelope + human + `run_trace` format + F8 const + id sanitize; units AC1/AC9 |
| `crates/ai-brains-cli/src/main.rs` | Trace `--format` with T266 `value_parser` (not `OutputFormat::parse`); after_help **`:1589` / `:1892` / `:1924`**; dispatch |
| `crates/ai-brains-cli/src/commands/governed_common.rs` | F16 sanitizer collapse **only** |
| `crates/ai-brains-cli/tests/governed_vault_pin_honesty.rs` | Replace null AC → envelope AC2 |
| `crates/ai-brains-cli/tests/recall_briefing_clarity.rs` | AC4 envelope |
| New hermetic found test (same crate tests/) | AC5 |
| `Docs/CAPABILITIES.md` | Trace row |
| `Docs/OPERATIONS.md` | both null phrases (excluded empty-success + empty-vs-deny token) |
| `Docs/CLI-EXIT-CODES.md` | optional `--format`; still exit 0 |
| `Docs/PROTOCOL-COMPAT.md` | §5 row + §3.1 note |
| `CHANGELOG.md` | on implement |
| `conductor/conductor.md` | Completed on implement only |
| `conductor/deferred.md` | this planning table (now); closeout later |

**Do not touch:** `briefings.rs` `QueryTraceDto` fields; CP `get_query_trace` / `persist_trace`; `query_store.rs`; `briefing.rs`; `personal.rs`; CLI `preflight.rs`; `project.rs`; `.github/workflows/ci.yml`.

---

## 13. AI fold-in

Inputs (not edited): `agy-review.md` (HEAD `e81a1a2`) + `opencode-review.md` (HEAD `560f3b0`). Fold-in HEAD `ff61399` (`track-t95-in-force`). Live verify: `run_trace` None arm `println!("null")` `:229`; `OutputFormat::parse` `:352–357` case-insensitive unknown→Json; clap `value_parser` precedent `:1195` / `:1943` / `:2383`; `sanitize_recall_query` `:73–75` `prev_space = false`; `QueryTraceDto` no `found`; `API_VERSION` `"1"` `briefings.rs:13`; `persist_trace` dry_run skip `:783`; bootstrap grants cover `get_query_trace` OR-of-three; `progressive_cmd` `:83` query `"x"`; after_help null at `:1589` / `:1892` / `:1924`; OPERATIONS empty-success null + token null; `cli_help_ia.rs` locks progressive/expand only `:112–116`. Hotspot `governed_common.rs` **#2** (3.806). Pins **snapshot — re-verify at execute** (clap lock 4.6.1 / crates.io 4.6.6; rusqlite 0.39.0; no clap 5). Last merged PR still **#206**. **No T301.**

### Pins locked by fold-in

1. **F3 / AC7 (Agy m1 + OpenCode o-1):** Trace `--format` uses clap `value_parser` T266 set. **Forbidden:** `OutputFormat::parse` for InvalidValue.
2. **AC5 (Agy O2 + OpenCode o-2):** System bootstrap omit `--principal-id`; `--dry-run false`; do **not** reuse `progressive_cmd`. Bootstrap trio covers trace read.
3. **AC10 (Agy O1 + OpenCode o-6):** Rewrite all three after_help sites **and** both OPERATIONS null phrases; hermetic `query trace --help` forbids `JSON token null`.
4. **F16 (Agy m2):** interpolators are space boundaries; no double-space; final trim. Already DoD; wording tightened.
5. **Hotspot (OpenCode M-1):** `governed_common.rs` **#2** (3.806) — sanitizer only.

### Per-AI disposition

| Source | Item | Disposition |
|--------|------|-------------|
| Agy | B / M | None filed |
| Agy | **m1** `--format JSON` via `OutputFormat::parse` | **Folded** F3 / AC7 / §5.5 / touch map |
| Agy | **m2** sanitizer space boundaries / no double-space | **Already** F16 / AC8; **tightened** F16 wording |
| Agy | **O1** parent + Trace after_help | **Already** F14; **tightened** AC10 line pointers `:1589` / `:1892` / `:1924` |
| Agy | **O2** e2e persist then trace | **Already** AC5; **tightened** bootstrap + `--dry-run false` + not `progressive_cmd` |
| OpenCode | B | None filed |
| OpenCode | **M-1** hotspot #3 → **#2** (3.806) | **Folded** §2.1 / isolation / plan preflight |
| OpenCode | **M-2** word count 381 → volatile | **Folded snapshot** (fold-in **689** / pinned **4019**); Phase 0 re-verify; **not DoD** |
| OpenCode | **M-3** doctor 5 → volatile | **Folded snapshot** (fold-in **4** warn); Phase 0; **not DoD** |
| OpenCode | **o-1** name clap `value_parser` | **Folded** F3 / AC7 / §5.5 (same as Agy m1) |
| OpenCode | **o-2** bootstrap covers `get_query_trace` | **Folded** AC5 / §5.6 |
| OpenCode | **o-3** `api_version` `"1"` | **Already** F7 / `briefings.rs:13` |
| OpenCode | **o-4** `auto` TTY `is_terminal` | **Already** F3 |
| OpenCode | **o-5** `cli_help_ia.rs` lock | **Decline** — T204 suite locks progressive/expand project-id only; AC10 hermetic `--help` is the lock |
| OpenCode | **o-6** OPERATIONS two null sentences | **Folded** AC10 / touch map |
| both | last-PR #206 Cursor | **Affirm F16** — no T301 |
| both | deferred T292–T300 / H2 / clap 5 | **Affirm** |

No Blockers. OpenCode Majors are snapshot drift (M-1 folded; M-2/M-3 not product DoD). No new placeholder minted. Do **not** edit `*-review.md`.

