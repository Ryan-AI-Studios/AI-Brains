# T314 — Unify `--format` / `--dry-run` clap semantics

- **Track ID:** T314-ClapFlagUnify
- **Status:** **Planned** (Pending until **go**)
- **Category:** UX / CLI
- **Owner:** Grok
- **Source:** Live CLI audit 2026-08-27 — friction (5 clap errors). `--format` rejected on `query expand`. `--dry-run` requires a value on `query progressive` / `briefing project` / `briefing personal`. `project scan-roots` rejects `--dry-run` (command is already dry-run-only). Series README `README-T312-T324-CLI-DOGFOOD.md`.
- **Depends on:** T266 ✅ four-family format table; T268 ✅ scan-roots dry-run-only; T290 F10 progressive JSON-only (do **not** add `--format` to progressive); T291 ✅ `query trace` token set + `--dry-run false` persist SOOT
- **Blocks / feeds:** Every later CLI track. T291 copy-paste `query progressive … --dry-run false` must stay valid. Does **not** populate governed stores. Does **not** steal T319 handle-vs-memory UUID.
- **Absorbs:** Audit clap friction (expand `--format`; progressive/briefing optional-value `--dry-run`; scan-roots `--dry-run` no-op)
- **Not absorbed (DoD):** clap **5**; T266 auto TTY/pipe family rewrite; adding `--format` to `query progressive` (T290 F10); `--commit` / `--apply` instead of `--dry-run false`; silent `.env`; T321 `safety sync`; T324 empty TERM; rotate-datakey `require_backup` `ArgAction::Set`
- **Research date:** 2026-08-28 (plan-write product HEAD `ae6615d` T315 `#231`). Snapshot — **re-verify at execute**.
- **Ledger:** planning DOCS TX `23da7568-f134-4dde-8a9a-3842eb213cb7`. Series mint DOCS `a6d3c404-1d64-4cba-a743-d75ac16c74cd`. Implement starts a **FEATURE** TX on **go**.
- **Isolation:** Do **not** implement until **go**. Do **not** `cargo install`. Do **not** bump clap. Do **not** grow hotspot `project.rs` / `sync.rs` / `governed_common.rs`. Do **not** rewrite scan-roots to write. Do **not** print or commit `AI_BRAINS_KEY`. Touch `main.rs` clap + parse tests + a thin expand human branch in `governed_query.rs`.

---

## 1. Objective

1. **`--dry-run` is a flag that still accepts `true`/`false`.** `ai-brains query progressive "…" --dry-run` parses (dry-run **true**). `… --dry-run false` still parses (T291 persist). Same clap block on `briefing project` and `briefing personal` (live same `ArgAction::Set` trap). Omitted stays default **true**.
2. **`query expand` accepts `--format`** with the same token set as `query trace` (`auto` / `pretty` / `human` / `text` / `json` / `markdown` / `md`). Default remains **json**. `--format human` is not a JSON wall.
3. **`project scan-roots --dry-run` is accepted as a no-op alias.** The command stays dry-run-only (T268). Unknown-arg friction goes away. No events. No `.env`.
4. **North star.** Capture independence: clap parse + a thin expand stdout branch. No new events. No hidden CoT. Operators who copy `--dry-run` from `pin` / `backup` / T291 `after_help` must not hit clap.

This unblocks daily CLI: five live parse failures on PATH 0.1.3 and on HEAD `ae6615d` are the same clap structs (T315 did not touch them).

---

## 2. Live baseline (re-scan 2026-08-28)

### 2.1 Operator dogfood (this machine)

| Signal | Observation |
|--------|-------------|
| HEAD | `ae6615d` `feat(cli): T315 preflight empty-decisions next-step + Budget window words (#231)`. Tree **CLEAN**. Branch `track/T314-clap-flag-unify` (from `main`). `origin/main` = `ae6615d` (ahead **0** at plan-write). |
| PATH `ai-brains.exe` | `C:\Users\RyanB\.cargo\bin\ai-brains.exe` **26,897,408** B; LastWriteTime **2026-08-27 8:21:55 PM**; `ai-brains 0.1.3`. Owner elevated install after T311 `#229`. **T312 and T315 are not on PATH.** T314 clap holes **are** on PATH (same structs). **Do not `cargo install`.** Tests/manual AC use `cargo run` / hermetic / `Cli::try_parse_from`. |
| `preflight --summary` (PATH) | Pinned **4536**. In-context **0/0/0**. `Total Word Count: 705` (PATH-behind T315 `Budget window words:`). Grants omitted (3 of 3). **Not this DoD.** |
| `query expand <uuid> --format json` | `error: unexpected argument '--format' found` + tip `--log-format`. **Source + PATH.** |
| `query progressive "what did we decide" --dry-run` | `error: a value is required for '--dry-run <DRY_RUN>' but none was supplied` `[possible values: true, false]`. |
| `briefing project --dry-run` / `briefing personal --dry-run` | Same required-value error. |
| `project scan-roots --dry-run` | `error: unexpected argument '--dry-run' found` + tip `-- --dry-run`. |
| `query progressive --help` | `--dry-run <DRY_RUN>` default **true**; possible values `true, false`. No `--format`. |
| Last GitHub PR | [#231](https://github.com/Ryan-AI-Studios/AI-Brains/pull/231) T315. `mergedAt` **2026-08-28T04:49:26Z**. Issue comments **[]**. Review comments **[]**. Reviews **[]**. **last-PR Cursor: N/A empty.** `#230` Bugbot medium already **T325**. Open PRs: **none**. |
| Ledger | 0 pending / 0 drift at scan (before this DOCS TX). Hotspot **#1** `project.rs` (3.741) — **do not grow.** `sync.rs` #2. `governed_common.rs` **#3** (3.414) — **do not grow.** CLI `preflight.rs` #7 (T315) — **do not touch.** |
| `ISSUES.md` | **Does not exist.** |

### 2.2 Why these five clap errors still matter

| Hole | Why it is still a product hole / why decline extras |
|------|-----------------------------------------------------|
| `query expand --format` | Trace already has T291 `value_parser` (7 tokens). Expand is the sibling handle preview and always JSON (`emit_json`). Agents who pass `--format json` / `--format human` from the T266 table hit `--log-format`. **DoD.** |
| Progressive / briefing `--dry-run` `ArgAction::Set` | Default is already dry-run **true** (skip `QueryTraceRecorded` / skip BriefingGenerated). Operators type a bare flag because `pin` / `backup` / `graph rebuild` / `unregister-path` are `SetTrue`. Help even says “Skip QueryTraceRecorded event (default: true)” then requires a value. **DoD as optional-value flag.** Do **not** invert the default (omitted must stay true). Do **not** replace persist with `--commit`/`--apply` (T291 F8 freeze). |
| `scan-roots --dry-run` | after_help already: “Dry-run only. Never appends events.” CAPABILITIES: dry-run table. The flag is missing so `--dry-run` is unknown. **DoD as no-op `SetTrue`.** Do not write. |
| Progressive `--format` | T290 F10 / CAPABILITIES “json / json / No TTY flip”. **Decline.** |
| clap 5 | Standing. **Decline.** |
| Unify every `--dry-run` in `main.rs` | pin/backup/graph already `SetTrue` default false. erasure wipe is `SetTrue` + `--confirm`. rotate-datakey `require_backup` is `ArgAction::Set` but **not** dry-run. **Decline bag.** |

### 2.3 Code truth (opened)

| Item | Location | Notes |
|------|----------|-------|
| Progressive clap | `main.rs` `GovernedQueryCommands::Progressive` `:2230–2239` | `dry_run: bool` `default_value_t = true, action = ArgAction::Set`. **No `--format`.** |
| Expand clap | `Expand` `:2245–2252` | `handle_id`, `project_id`, `max_chars` — **no format**. Dispatch `:4330–4341` three fields. |
| Trace clap | `Trace` `:2257–2265` | `value_parser = ["auto", "pretty", "human", "text", "json", "markdown", "md"]`; default `"json"`. **Copy this set.** |
| Briefing clap | `BriefingCommands::Project` `:2197–2198`; `Personal` `:2213–2214` | Same `Set` + default true as progressive. |
| ScanRoots clap | `ProjectCommands::ScanRoots` `:3153–3161` | `path`, `root`, `format` (T266 7-token `auto`). **No `dry_run`.** Dispatch `:5251–5256` three fields into `scan_roots(&ctx, path, format)`. |
| rotate-datakey | `require_backup` `:2068` | `ArgAction::Set` default true — **not** `--dry-run`. **Do not touch.** |
| T291 persist SOOT | `governed_query.rs` `TRACE_PROGRESSIVE_PERSIST` `:29–30`; `TRACE_MISSING_NEXT_STEP` `:33–34` | Exact `ai-brains query progressive "what did we decide" --dry-run false`. Unit `trace_missing_next_step__frozen__exact_string` `:295`. |
| Expand run | `run_expand` `:169–219` | Always `emit_json`. Adds `applied_scope`. `apply_unknown_expand_preview` uses `UNKNOWN_HANDLE_PREVIEW` = `"Handle not found."` (`governed_common.rs:140`). Denied → exit 3 after JSON. |
| Trace human helper | `missing_trace_is_human` `:242–248` | `human\|pretty\|text\|markdown\|md`; `auto` → TTY. Reuse / share for expand. |
| Scan-roots handler | `project_paths.rs` `scan_roots` `:255–259` | Never writes. **Do not add a 4th logic param** — discard `dry_run` in the `main.rs` match (`dry_run: _`). |
| Parse tests | `main.rs` `query_trace__format_json__parses` `:470`; `query_trace__format_JSON__clap_invalid_value` `:451`; `scan_roots__format_pretty__parses` `:717` | Copy this style. |
| Hermetic progressive | `governed_first_run_deny_exit.rs:128–131` | Comment: pass explicit `--dry-run true` because Set requires a value. Stay-green; **also** prove bare `--dry-run` parses. |
| CAPABILITIES | `:94` lumps `query progressive` / `expand` / `trace` as json/json no TTY flip | Split: progressive stays JSON-only; expand/trace default json + T291/T314 token set. Scan-roots row `:99` already “Dry-run table”. |
| Hotspots | `project.rs` #1 / `sync.rs` #2 / `governed_common.rs` #3 | Isolation. |

### 2.4 Dependency / standards research (2026-08-28) — snapshot; re-verify at execute

| Pin | Workspace / lock | Ecosystem (today) | Action |
|-----|------------------|-------------------|--------|
| `clap` | workspace **`4.5`** / lock **4.6.1** (checksum `1ddb117e…`) | crates.io **4.6.6** (2026-08-06). GitHub clap-rs latest **v4.6.6**. **No clap 5** (feature `unstable-v5` only). rust-version 1.85. | **No bump.** Optional-value flag is in 4.5.30+ (`#5912`); lock **4.6.1** already has it. |
| `serde` / `serde_json` | workspace `1.0` | HandlePreviewDto frozen | **No bump.** No new DTO keys. |
| `rusqlite` | exact **0.40.2** | not this track | **No bump.** |
| rustc / edition | **1.95.0** / **2024** | — | Unchanged |
| workspace | **0.1.3** | — | **No bump** |
| New crates | — | — | **Zero.** |

**Online / primary sources**

| Claim | Source | Takeaway |
|-------|--------|----------|
| Optional-value bool flag | [clap 4.6.1 `ArgAction::SetTrue`](https://docs.rs/clap/4.6.1/clap/builder/enum.ArgAction.html): “No value is allowed. To optionally accept a value, see `Arg::default_missing_value`.” [clap#5909](https://github.com/clap-rs/clap/discussions/5909) (epage, 2026-02-17): `num_args(0..=1)` + `ArgAction::Set` + `default_missing_value = "true"`; **do not** hide `num_args`. `#5912` merged 2026-02-17, released **4.5.30**. | T314 uses that block. **No `require_equals`** — T291 SOOT is `--dry-run false` with a **space**, not `--dry-run=false` only. |
| `default_missing_value` color example | docs.rs clap 4.6.1 `Arg::default_missing_value` still shows `require_equals(true)` for `--color`. epage explicitly: decide equals vs space; T291 freeze chooses space. | Do not set `require_equals`. |
| Bare `--dry-run` is the Unix flag | [clig.dev](https://clig.dev/) *Arguments and flags*: “`-n`, `--dry-run`: Dry run.” Full-length flags. Consistency across programs. | Bare `--dry-run` must parse on progressive/briefing like pin/backup. |
| Suggest next command / do not lie | clig.dev *Ease of discovery* + *Saying (just) enough* | `--format human` on expand must not still dump JSON. Default stays json (Family C, T266). |
| Human vs JSON | clig.dev *Output*: humans first; `--json` / `--format json` for structure; changing human output is usually OK | Expand human is two stdout lines (`kind` then `preview`). JSON DTO frozen. |
| Consistency | clig.dev *Consistency across programs* | Copy Trace `value_parser` bytes. Do not invent a second token list. |

**N/A:** SQLCipher page encrypt, schtasks, Windows service, llama.cpp `/health`, clap 5 (not this bump), FTS5, T307 reqwest/tower-http, Index SQL.

**Could not verify:** Whether clap 4.6.6 changes `default_missing_value` docs vs 4.6.1 (latest is 4.6.6; we do not bump). Hermetic parse is SoT at execute. Live Manual AC is `cargo run` clap parse, not PATH.

**ledgerful / ai-brains:** `preflight --summary` PATH 4536 pins / 0/0/0 / `Total Word Count` (PATH-behind T315); `ledgerful ledger status --compact` 0 pending / 0 drift; `search "ArgAction::Set"` → `main.rs:2197/:2213/:2238` + `require_backup:2068`; `scan --impact` CLEAN at `ae6615d`; hotspots `project.rs` #1 / `governed_common.rs` #3. Semantic recall of clap still returns the 2026-08-27 audit dump — evidence of ranking (T312 not on PATH), not SoT for clap structs.

---

## 3. Frozen decisions

| ID | Decision |
|----|----------|
| **F0 — Go gate** | Plan-only until user **go**. Planning is DOCS TX `23da7568`. Implement starts a **FEATURE** TX. |
| **F1 — Optional-value `--dry-run`** | Progressive + both briefing subcommands use the **same** clap block: `long`, `default_value_t = true`, `num_args = 0..=1`, `default_missing_value = "true"`, `action = ArgAction::Set`. Semantics: omitted → **true**; `--dry-run` → **true**; `--dry-run true` / `--dry-run=true` → **true**; `--dry-run false` / `--dry-run=false` → **false**. |
| **F2 — No `require_equals`** | T291 SOOT is space-separated `--dry-run false`. Setting `require_equals` would make that a positional and rot AC1. Equals form may also parse (clap); space form is required. |
| **F3 — T291 persist string frozen** | `TRACE_PROGRESSIVE_PERSIST` and `TRACE_MISSING_NEXT_STEP` stay exact. Do **not** switch copy-paste to `--apply` / `--commit` / `--write`. Unit `trace_missing_next_step__frozen__exact_string` stay-green. |
| **F4 — Default stays dry-run** | Omitted `--dry-run` is still true. Do **not** invert to SetTrue-default-false (that would persist traces / write BriefingGenerated without an explicit flag). |
| **F5 — Briefing absorbed** | `briefing project` and `briefing personal` get F1. Same trap, same fix. Do not change `briefing.rs` writer logic (already `if options.dry_run { None }`). |
| **F6 — Progressive JSON-only** | **No** `--format` on `query progressive` (T290 F10). CAPABILITIES progressive row stays json/json no TTY flip. `query_progressive__format_json__unexpected_argument` stay-green. |
| **F7 — Expand `--format` token set** | Copy Trace `value_parser` **exactly** (7 tokens, case-sensitive). Default **`json`**. `JSON` / `Pretty` / `xml` → clap `InvalidValue` (exit 2), not `OutputFormat::parse`. |
| **F8 — Expand default JSON** | Omitted `--format` stays JSON `HandlePreviewDto` + `applied_scope` (today). Do **not** default `auto` (would TTY-flip expand and break TTY operators / stay-green hermetics). `--format auto` is allowed: TTY human / pipe JSON via shared helper. |
| **F9 — Expand human honesty** | Tokens `human\|pretty\|text\|markdown\|md` (and `auto` on TTY): stdout is **two lines**, not a JSON object. Line 1 = `kind`. Line 2 = `preview` (existing string, including `UNKNOWN_HANDLE_PREVIEW`). Do not print `applied_scope` on human. JSON path unchanged `emit_json`. |
| **F10 — Expand deny / fail_cp** | `kind == "Denied"` still exit **3** + existing stderr after stdout. Human Denied still two lines then exit 3. `fail_cp` stays `OutputFormat::Json` (T221) — do not thread format into CP errors this track. |
| **F11 — Scan-roots `--dry-run` no-op** | Add `#[arg(long)] dry_run: bool` (`SetTrue`, default false). Discard in dispatch (`dry_run: _`). **Do not** change `scan_roots` signature or write behavior. after_help one sentence: `` `--dry-run` is accepted (already dry-run-only). `` |
| **F12 — Decline `--commit`/`--apply`** | Persist remains `--dry-run false`. Adding a second write flag would dual-path T291 hermetics. |
| **F13 — Decline clap 5** | Workspace stays `"4.5"` / lock 4.6.1 unless execute re-verify shows a security pin (then a different track). |
| **F14 — Decline bag** | rotate-datakey `require_backup`; erasure `--confirm`; pin/backup/graph `SetTrue` default-false; T321 `safety sync`; T324 empty TERM; T319 handle namespace; T266 Family A rewrite; T240 F2; T263 H2; T307; T308 floors; T313/T315–T325 steal. |
| **F15 — Capture independence** | Clap + stdout branch. No new events. No models. |
| **F16 — Exit codes frozen** | Parse failures stay clap **2**. Expand Unknown **0**. Expand Denied **3**. Progressive deny **3**. Scan-roots success **0**. |
| **F17 — No new required JSON keys** | HandlePreviewDto / ProgressiveQueryResponse / scan-roots envelope / briefing packets frozen. |
| **F18 — PATH** | Do not `cargo install` unless the user asks. Parse SoT is `Cli::try_parse_from` + hermetic/`cargo run`. PATH-behind is not a fail. |
| **F19 — File growth** | Clap + parse tests in `main.rs`. Expand human branch + shared `query_format_is_human` in `governed_query.rs` (reuse `missing_trace_is_human` match). Hermetic add in `governed_vault_pin_honesty.rs` (expand human) + one progressive bare `--dry-run` in `governed_first_run_deny_exit.rs`. **Do not** grow `project.rs`, `sync.rs`, `governed_common.rs`, `project_paths.rs`, `briefing.rs`, retrieval, contracts. |
| **F20 — Pins / crates** | No clap 5, no rusqlite bump, no new crates, workspace **0.1.3**. |
| **F21 — last-PR Cursor `#231`** | **N/A empty** (comments/reviews `[]`). `#230` F8 recency already **T325**. **No T326.** |
| **F22 — Tests** | Naming `function_or_feature__condition__expected_result`. No `unwrap`/`expect`/`panic` in production. `try_parse_from` tests may `panic!` on unexpected Ok/Err (existing `main.rs` pattern). |
| **F23 — Cross-model** | CLI honesty FEATURE. After Phase-1 clean, run read-only `codex-review`. |
| **F24 — Debt file** | `conductor/ISSUES.md` does **not** exist. Deferrals → `conductor/deferred.md`. |
| **F25 — Docs** | CAPABILITIES: split progressive vs expand/trace; scan-roots `--dry-run` accepted. CHANGELOG T314. PROTOCOL-COMPAT: scan-roots keys unchanged; expand JSON keys unchanged; optional note that expand `--format human` is not a wire contract. after_help: progressive/briefing `--dry-run` examples include bare flag **and** `--dry-run false`; expand `--format json` / `--format human`; scan-roots `--dry-run`. |
| **F26 — PowerShell** | `;` not `&&`. |
| **F27 — Stay-green T291** | Missing-trace envelope + human two lines + `--dry-run false` persist hermetic AC5 still green. |
| **F28 — Stay-green T290** | Progressive no `--format`. Granted-empty `next_step` recall overlay unchanged. |
| **F29 — Stay-green T266/T268** | Scan-roots format tokens + envelope keys + `--root` XOR + no writes. |
| **F30 — Stay-green expand JSON** | `query_expand__unknown__preview_nonempty_exit_0` still JSON `kind=Unknown` without `--format`. |
| **F31 — Shared dry-run attrs** | Repeat the F1 attribute block at three sites with a one-line `T314 F1` comment. Do **not** invent a clap derive macro / newtype. |
| **F32 — `query_format_is_human`** | Extract Trace’s match into `query_format_is_human(format: &str) -> bool` in `governed_query.rs`. `missing_trace_is_human` becomes a thin wrapper **or** is replaced at both call sites. Behavior frozen. |
| **F33 — Determinism** | Exact clap tokens; no timestamps in new strings. |
| **F34 — `--dry-run` before positional** | `query progressive --dry-run "what did we decide"` may still fail bool parse (the quoted string is not `true`/`false`). **Not a fail.** Document examples with the query first, then flags — same as today. Bare `--dry-run` **after** the query is the DoD. |

---

## 4. Acceptance criteria

| AC | Proof |
|----|-------|
| **AC1** | Unit `query_progressive__dry_run_bare__parses_true`: `try_parse_from(["ai-brains", "query", "progressive", "q", "--dry-run"])` → `Progressive { dry_run: true, .. }`. **Required red.** |
| **AC2** | Unit `query_progressive__dry_run_false__parses_false`: `"--dry-run", "false"` → `dry_run: false`. `"--dry-run", "true"` → true. Omitted → true. **Required red** (false + omitted can share rstest cases). |
| **AC3** | Units `briefing_project__dry_run_bare__parses_true` and `briefing_personal__dry_run_bare__parses_true`. **Required red.** |
| **AC4** | Unit `query_expand__format_json__parses` (format == `"json"`) + `query_expand__format_human__parses` + default omitted == `"json"`. **Required red.** |
| **AC5** | Units `query_expand__format_JSON__clap_invalid_value` and `query_expand__format_xml__clap_invalid_value` (`ErrorKind::InvalidValue`). **Required red.** |
| **AC6** | Unit `scan_roots__dry_run__parses`: `["ai-brains", "project", "scan-roots", "--dry-run"]` Ok. **Required red.** |
| **AC7** | Unit `query_progressive__format_json__unexpected_argument` still fails clap (F6). Stay-green / required if not already present. |
| **AC8** | `trace_missing_next_step__frozen__exact_string` stay-green (F3). |
| **AC9** | Hermetic `query_expand__unknown__preview_nonempty_exit_0` stay-green (JSON, no `--format`). |
| **AC10** | Hermetic `query_expand__format_human__unknown__two_lines_not_json`: `--format human` exit **0**; stdout is **not** a JSON object (no leading `{`); contains `Unknown`; contains `Handle not found.`; two nonempty lines. **Required red.** |
| **AC11** | Hermetic `progressive__dry_run_bare_no_grants__exit_3_denied_true`: `--dry-run` with **no** value still exit 3 + JSON `denied: true` (extends `:128` comment). **Required red.** |
| **AC12** | Hermetic scan-roots `--dry-run --format json` on a temp vault/cwd: exit **0**; envelope keys unchanged (`api_version`, `scan_root`, `truncated`, `roots`); no `.env` write. Can be parse+one integration; do not require live `C:\dev` scan. |
| **AC13** | Docs: CAPABILITIES split progressive vs expand; scan-roots `--dry-run` accepted; CHANGELOG Unreleased T314; PROTOCOL-COMPAT expand human not a wire contract; after_help examples F25. |
| **AC14** | Manual (source bin): `cargo run -q -p ai-brains-cli -- query expand <uuid> --format json --no-project-context` **parses** (not `--log-format`); `query progressive "q" --dry-run --no-project-context` **parses** (may then usage/deny); `project scan-roots --dry-run --format json --no-project-context` **parses**. PATH-behind is **not** a fail. **Do not** `cargo install`. |
| **AC15** | `git diff -- crates/ai-brains-cli/src/commands/project.rs crates/ai-brains-cli/src/commands/project_paths.rs crates/ai-brains-cli/src/commands/governed_common.rs crates/ai-brains-cli/src/commands/briefing.rs crates/ai-brains-contracts` empty (except if briefing.rs is truly untouched — expected empty). |
| **AC16** | Human expand SOOT: line 1 is exactly DTO `kind`; line 2 is exactly DTO `preview`. No extra banner. |

---

## 5. Design notes

### 5.1 Why optional-value, not SetTrue + `--commit`

Progressive/briefing default is **dry-run true**. `SetTrue` default is false. Flipping to “bare `--dry-run` means true, omitted means false” would persist `QueryTraceRecorded` on every default progressive — the opposite of T152/T291. Optional-value keeps the default and unblocks the Unix flag. T291 `--dry-run false` stays the persist verb.

### 5.2 Why expand human is two lines, not parse-only

Parse-only `--format human` that still `emit_json`s would lie (clig.dev). Trace already human-renders missing envelopes. Expand human is `kind` + `preview` — the two fields operators need. `applied_scope` stays JSON-only (debug). T319 (evidence show on a vault `memory_id`) is a different command; Unknown preview stays `Handle not found.`

### 5.3 Why scan-roots does not grow `project_paths.rs`

`scan_roots` never writes. A 4th `_dry_run` argument is noise on hotspot-adjacent path code. Discard in `main.rs` match. after_help names the alias.

### 5.4 Clap attribute (SoT)

```rust
/// T314 F1 — optional-value dry-run (default true). Bare `--dry-run` and `--dry-run false` both parse.
#[arg(
    long,
    default_value_t = true,
    num_args = 0..=1,
    default_missing_value = "true",
    action = clap::ArgAction::Set
)]
dry_run: bool,
```

Three sites. No macro.

### 5.5 Expand format helper

```rust
fn query_format_is_human(format: &str) -> bool {
    match format {
        "human" | "pretty" | "text" | "markdown" | "md" => true,
        "auto" => std::io::stdout().is_terminal(),
        _ => false,
    }
}
```

Default `"json"` → `_` → false → JSON. Same as Trace missing-path.

---

## 6. Non-goals

clap 5. Rewriting every subcommand’s format parser. Progressive `--format`. Scan-roots writes / auto-register. `--commit`/`--apply`. rotate-datakey `require_backup`. T319 namespace copy. T321 `safety sync` grouping. T324 PowerShell empty TERM. T266 Family A default flips. T315 summary (Completed). T325 F8 recency. Growing `project.rs` / `governed_common.rs`. New contracts DTO keys. Silent `.env`. `cargo install`.

---

## 7. Verification plan (TDD)

Red first (must fail on current `ArgAction::Set` / missing expand format / missing scan-roots dry_run):

1. `query_progressive__dry_run_bare__parses_true` (AC1)
2. `query_progressive__dry_run_false__parses_false` (AC2)
3. `briefing_project__dry_run_bare__parses_true` + personal (AC3)
4. `query_expand__format_json__parses` + human + default json (AC4)
5. `query_expand__format_JSON__clap_invalid_value` (AC5)
6. `scan_roots__dry_run__parses` (AC6)
7. Hermetic expand `--format human` two lines (AC10)
8. Hermetic progressive bare `--dry-run` deny (AC11)

Then green F1/F7/F9/F11. Stay-green AC7/AC8/AC9/AC12/T266/T268/T290/T291. Docs AC13. Manual AC14.

---

## 8. Risk

| Risk | Mitigation |
|------|------------|
| `num_args 0..=1` steals the query positional | F34; examples keep query first; AC1 uses query then `--dry-run` |
| `require_equals` accidentally set | F2 / AC2 space `false` |
| Default inverted → traces persist | F4; omitted unit in AC2 |
| T291 SOOT rot | F3 / AC8 |
| Expand `--format human` still JSON | F9 / AC10 |
| Scan-roots starts writing | F11 / AC12 / AC15 |
| Growing `project.rs` | F19 / AC15 |
| Progressive `--format` sneaks in | F6 / AC7 |
| `#231` leftover dropped | F21 N/A empty; T325 already minted |
| PATH-behind false AC fail | F18 |

---

## 9. Deferred absorb/decline

| Item | Disposition |
|------|-------------|
| Audit `--format` missing on `query expand`; `--dry-run` value on progressive; scan-roots rejects `--dry-run` | **Absorb** F1/F7/F11 / AC1–AC6 / AC10–AC14 |
| Briefing `--dry-run` same Set trap | **Absorb** F5 / AC3 |
| T290 F10 progressive JSON-only | **Affirm** F6 / AC7 |
| T291 `--dry-run false` persist | **Affirm** F3 / AC8 |
| T266 four-family table / auto TTY | **Partial** — expand stays Family **C** default json; tokens copy Trace; do not flip Family A |
| T268 scan-roots dry-run-only | **Affirm** F11 — flag alias, still no write |
| T319 handle vs memory UUID | **Not stolen** |
| T321 `safety sync` write honesty | **Not stolen** |
| T324 PowerShell empty TERM | **Not stolen** (different clap hole) |
| T313 / T315–T318 / T320 / T322 / T323 / T325 | **Not stolen** |
| T307 Blocked / T308 floors | **Not stolen** / **Decline** |
| T263 H2 / T240 F2 / clap 5 | **Decline** F14 / F13 |
| rotate-datakey `require_backup` Set | **Decline** F14 |
| last-PR Cursor `#231` | **N/A empty** F21 — no T326 |
| last-PR `#230` F8 recency | **T325** already Pending |
| conductor/archive / cargo-audit allowlist | **Not related** |
| PATH T315 `Total Word Count` | **Not this DoD** (T315 F18) |

---

## 10. Implement order (on go)

1. Phase 0 re-read Progressive/Expand/ScanRoots/Briefing clap + T291 consts + `run_expand`; rescan deferred; FEATURE TX  
2. Red AC1–AC6 + AC10–AC11  
3. Green F1 three sites; F7 expand clap; F9 human branch; F11 scan-roots field + discard; F32 helper  
4. Stay-green AC7–AC9 / AC12 / T266 / T268 / T290 / T291  
5. Docs F25 / AC13  
6. Manual AC14 → review → full gate → Complete  

---

## 11. Soft residuals (expected)

| Residual | Note |
|----------|------|
| PATH until `cargo install` | F18 |
| `query progressive --dry-run "query text"` bool parse fail | F34 by design |
| Expand `auto` on TTY is human only when explicitly `--format auto` | F8 default json |
| `fail_cp` JSON even if `--format human` | F10 |
| T325 F8 PreferRecency | Placeholder; not this DoD |
| T319 `Handle not found.` on vault memory UUID | Different command |
| clap lock 4.6.1 vs crates.io 4.6.6 | F13 no bump |

---

## 12. Touch map (expected)

| Site | Change |
|------|--------|
| `crates/ai-brains-cli/src/main.rs` | F1 three dry-run attrs; Expand `format`; ScanRoots `dry_run`; dispatch thread/discard; parse units AC1–AC7 |
| `crates/ai-brains-cli/src/commands/governed_query.rs` | `ExpandHandleOptions.format`; F9/F32; stay-green T291 unit |
| `crates/ai-brains-cli/tests/governed_vault_pin_honesty.rs` | AC10 expand human hermetic |
| `crates/ai-brains-cli/tests/governed_first_run_deny_exit.rs` | AC11 bare `--dry-run` |
| `Docs/CAPABILITIES.md` | Split progressive vs expand; scan-roots `--dry-run` accepted |
| `Docs/PROTOCOL-COMPAT.md` | Expand human not a wire contract; scan-roots keys frozen |
| `CHANGELOG.md` | T314 Unreleased |
| `main.rs` after_help strings | F25 examples |

---

## 13. last-PR Cursor / fold-in placeholder

Plan-write: `#231` comments empty; `#230` already T325. No mint. Fold-in later may correct line numbers / HEAD snapshot only.
