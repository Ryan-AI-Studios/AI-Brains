# T291 review log — query trace missing envelope + human next

**Track:** T291-QueryTraceNext
**Branch:** `track/T291-query-trace-next`
**FEATURE TX:** `585cee1d-d763-4b87-a4e9-23ce2ff32526`
**Reviewers:** implementer (R1) → codex-review (FEATURE)

## Scope

Lift T263 F6 scalar JSON token `null` for missing/unauthorized `query trace`. Default/`--format json` stdout is a CLI-local pretty envelope `{api_version, found: false, trace_id, next_step}` with `next_step` copy-paste `query progressive "what did we decide" --dry-run false`. `--format human` (and `pretty`/`text`/`markdown`/`md`) prints two lines. Found `QueryTraceDto` JSON frozen (no wrap; `--format` ignored). Exit 0; no project-id gate. #206 Bugbot: `sanitize_recall_query` treats `$` / backtick as space boundaries + final trim.

**Did not:** invent traces; wrap found as `{trace:…}`; flip progressive `--dry-run` default; invent `--trace`; QueryTraceDto new fields; daemon GetQueryTrace; T292 policy-check human; T293 neighbors; T294 leftover; T299 forget-list; T240 F2; T263 H2; clap 5 / rusqlite 0.40; `cargo install`; `.env` write; extra live `policy bootstrap`; grow `project.rs` / `sync.rs` / `forget.rs` / CLI `preflight.rs` / `personal.rs` / `briefing.rs` / CP `query.rs`. Envelope lives in `governed_query.rs`. `governed_common.rs` sanitizer collapse only.

## DoD matrix

| AC | Status | Evidence |
|----|--------|----------|
| AC1 | **Met** | `trace_missing_next_step__frozen__exact_string` PASS (red: `T291_RED_STUB`; green: exact F8; no `\n` / U+2026 / `--trace`; shares `TRACE_PROGRESSIVE_PERSIST`) |
| AC2 | **Met** | `query_trace__unknown__stdout_envelope_exit_0` PASS (not token `null`; `found: false`; `api_version: "1"`; F8 `next_step`) |
| AC3 | **Met** | `query_trace__unknown__human_two_lines` PASS (exactly two lines; `No trace`; `next:` + `query progressive` + `--dry-run false`) |
| AC4 | **Met** | `query_trace__missing_project__still_exit_0_envelope` PASS (exit 0; envelope not token `null`) |
| AC5 | **Met** | `query_trace__after_persist__returns_dto_not_envelope` PASS (System bootstrap omit `--principal-id`; `--dry-run false` not `progressive_cmd`; DTO `query_trace_id`; no `found`; `--format human` still JSON DTO) |
| AC6 | **Met** | T290 formatter/hints + `progressive_recall_fallback__exact__ellipsis_unchanged` PASS |
| AC7 | **Met** | `query_trace__format_JSON__clap_invalid_value` + `query_trace__format_json__parses` PASS (clap `value_parser`, not `OutputFormat::parse`) |
| AC8 | **Met** | rstest `a $ b` → `a b`; `$ $` / ` $ ` → `LIST_RECALL_QUERY`; existing tab/newline/quotes/`echo $(hi)`/80-cap stay green |
| AC9 | **Met** | `sanitize_trace_id__newline_dollar_quotes__single_line_safe` PASS |
| AC10 | **Met** | `query_trace__help__names_envelope_not_json_token_null` PASS; CAPABILITIES Trace; OPERATIONS both null phrases; CLI-EXIT-CODES; PROTOCOL-COMPAT §5 + §3.1; after_help `:1654` / `:1957` / Trace; CHANGELOG |
| AC11 | **Met** | Manual `cargo run -p ai-brains-cli -- query trace missing-id` + `--format human` (below) |
| AC12 | **Met** | `query_trace_dto__serialized__has_no_found_or_next_step` PASS |
| AC13 | **Met** | CP `get_query_trace` `Ok(None)` remains SoT for miss / cross-principal / no-grant (untouched). CLI locks `None` → envelope (AC2). Unauthorized is indistinguishable by design (F6). |

## Findings

| id | severity | description | status | evidence |
|----|----------|-------------|--------|----------|
| R1-1 | low-info | PATH `ai-brains` still T281-era until `cargo install`. Source/hermetic SoT. | deferred | F17 |
| R1-2 | low-info | Found `--format human` still prints QueryTraceDto JSON. | deferred | F10 by design |

No critical / high / medium. Internal R1 **PASS**.

## Targeted gates (pre-full)

- `cargo fmt --check` (via `cargo fmt --all`) PASS
- `cargo clippy -p ai-brains-cli --all-targets -- -D warnings` PASS
- Red was assertion-fail (not compile-error-only): AC1 stub `T291_RED_STUB`; AC2 stdout still `null`; AC8 `a $ b` → `a  b`
- CLI units AC1/AC7/AC8/AC9/AC10/AC12 **16 PASS**
- Hermetic `query_trace__*` **7 PASS** (includes AC5 persist)
- T290 stay-green formatter/hints **8 PASS**

## Manual

```
cargo run -p ai-brains-cli --quiet -- query trace missing-id
# stdout pretty envelope found=false, trace_id=missing-id, F8 next_step. EXIT:0. Not the token null.

cargo run -p ai-brains-cli --quiet -- query trace missing-id --format human
# No trace for missing-id.
# next: ai-brains query progressive "what did we decide" --dry-run false
# EXIT:0

cargo run -p ai-brains-cli --quiet -- query trace missing-id --format JSON
# clap InvalidValue possible values list; EXIT:2

cargo run -p ai-brains-cli --quiet -- query trace --help
# after_help names envelope + human two lines; does not contain "JSON token null"
```

PATH not reinstalled (F17). Did not write `.env`. Did not extra `policy bootstrap`. Did not `cargo install`.

## Codex CX1 (gpt-5.6-luna, read-only)

Product **PASS**. No product P0–P2. Process P1 (`T291-COMP-001` pending full gate / publish) — same class as T289/T290; **verified_fixed** after closeout + Phase 6.

| id | severity | disposition |
|----|----------|-------------|
| P1 T291-COMP-001 | process | **verified_fixed** after closeout + Phase 6 (local Completed is not published) |

## Full gate

- `.\scripts\dev-check.ps1` **SUCCESS** nextest **3443** passed / 1 skipped (8 slow)
- `ledgerful verify --scope full` exit 0 (`fmt` / workspace clippy / nextest / deny / audit)

Did **not** `cargo install`. Did **not** write `.env`. Did **not** extra `policy bootstrap`. Daemon left **Stopped**.
