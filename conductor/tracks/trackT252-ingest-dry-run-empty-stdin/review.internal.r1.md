# T252 internal review r1 — ingest dry-run empty stdin honesty

**Reviewer:** Grok (read-only)
**Date:** 2026-08-15
**Scope:** Working-tree implementation vs `spec.md` F1–F16 / AC1–AC16
**Mode:** No cargo fmt / clippy --fix / writes to product files. Tests not executed (static audit).

**Verdict: PASS**

No P0–P3 findings. Easy P3s were considered and not invented.

---

## Completeness / isolation / named checks

| Check | Result |
|-------|--------|
| Placeholders / stubs / `todo!` / silent serde→usage fallback | None in new ingest gate. Mid-payload still `Invalid JSON:` → `handle_cli_result` `COMMAND_FAILED`. T86 `read_json_from_stdin` swallow **not** reused. |
| Isolation | No `ai-brains-contracts` ingest DTO change. No `parse_ingest_request` rewrite. T180 tests untouched. T114 `DryRunIngestRequest` still `deny_unknown_fields` + `String` UUIDs. No pin bumps (workspace clap **4.5** / lock **4.6.1**; serde_json lock **1.0.150**; is-terminal lock **0.4.17**). `governed_common` only **called** via `fail_usage`. `cli_help_ia` group-order test unchanged. Ingest **not** added to `is_vault_path_free` / `run_sync_path_free`. |
| F6 | Helper keeps `is_tty: bool`. `trim()` lives only in the helper. `run()` does TTY-before-read via `is_terminal()` (F4), then `ingest_stdin_needs_usage(false, Some(&input))` for empty/whitespace. Does **not** inline trim in two places. |
| F5 | Usage const is single-quoted `echo '{…}' \| ai-brains ingest --dry-run`. `after_help` is a pretty-indented multiline object (same seven keys + zero UUIDs). Extra echo line on after_help is additive; AC8 asserts keys only. |
| F3 | `{` is non-empty → `from_str` → exit **1**. AC5 hermetic. |
| F9 | `Commands::Ingest` dispatched after `AppContext::from_cli` (`main.rs` ~3469). Not in vault-path-free set. |
| F13 | Empty `content` still `Err("content field is empty")` after parse. AC10 asserts not exit 2 and stderr must not contain `stdin is empty or not piped`. |
| AC12 docs | CLI-EXIT-CODES table + T252 footnote; CAPABILITIES §4 ingest + `--dry-run` bullets; OPERATIONS one-liner after `echo $json \| … ingest`; CHANGELOG Unreleased **BREAKING**. Accurate. |
| Tests vs old behavior | AC1–AC3 would fail (old exit 1 `COMMAND_FAILED` / `EOF while parsing`). AC6–AC7 would not compile. AC8 would fail (no `session_id` etc. on `ingest --help`). AC4/AC5/AC9–AC11 are keep-green (AC5/`{` already exit 1). |

---

## F1–F16 audit

| ID | Status | Evidence |
|----|--------|----------|
| **F1** Empty/whitespace → `fail_usage` | **Met** | After read, `ingest_stdin_needs_usage(false, Some(&input))` → `fail_usage(INGEST_EMPTY_STDIN_USAGE)`. Same gate for live and `--dry-run` (before the dry-run split). |
| **F2** Valid dry-run frozen | **Met** | `DryRunIngestRequest`, placeholder strings, preview stdout, no `append_event`. Field empty checks unchanged. |
| **F3** Mid-payload envelope | **Met** | Non-empty `{` not classified usage. `map_err(\|e\| format!("Invalid JSON: {}", e))` still feeds generic `handle_cli_result` → `COMMAND_FAILED` exit 1. |
| **F4** TTY before read | **Met** | `if io::stdin().is_terminal() { return fail_usage(...) }` then `read_to_string`. `is_terminal::IsTerminal` (not `std::io::IsTerminal`). |
| **F5** Example SOOT | **Met** | Const in `ingest.rs` matches spec normative text byte-for-byte (single-quoted JSON, seven keys, hermetic zero UUIDs, `ai-brains ingest --dry-run`). `after_help` multiline indented object. No cmd.exe claim. |
| **F6** Pure gate helper | **Met** | `pub(crate) fn ingest_stdin_needs_usage(is_tty: bool, raw: Option<&str>) -> bool`. Units AC6. Production empty path calls helper; trim not duplicated. `is_tty` kept (not flattened into `stdin().is_terminal()` inside the helper). |
| **F7** Docs + Phase 3 grep | **Met** | Four doc sites + CHANGELOG BREAKING. `plan.md` Phase 3 grep table recorded 2026-08-15 (hooks pipe built payloads; no empty-stdin `COMMAND_FAILED` consumer). |
| **F8** No DTO/daemon | **Met** | `IngestRequest` / `IngestResponse` unchanged (`crates/ai-brains-contracts/src/ingest.rs`). No `--schema`. No contracts T252 hits. |
| **F9** Vault still required | **Met** | Ingest not in `is_vault_path_free`. Dispatch after `AppContext`. Hermetics keep `--vault-path` + `hermetic_bin` zero key. |
| **F10** Pins | **Met** | Workspace `clap = "4.5"`, `serde_json = "1.0"`, `is-terminal = "0.4"`. Lock clap **4.6.1** / serde_json **1.0.150** / is-terminal **0.4.17**. |
| **F11** Isolation | **Met** | Capture `malformed.rs` untouched. T180 `protocol_compat_cli.rs` dual-path tests unchanged. `cli_help_ia` still group-order only. No T234 thinking populate. No shared stdin helper. |
| **F12** Soft residuals | **Met (left residual)** | Vault-free dry-run not added. T86 swallow untouched. `outcome.events[0]` still present on live response path (pre-existing). |
| **F13** Empty field ≠ empty stdin | **Met** | Object with `content: ""` is non-empty stdin → field error exit ≠ 2. |
| **F14** High-finding list | **Met (avoided)** | Did not map all serde errors to usage 2; no TTY hang; no unquoted `{…}`; no T180 flip; no key print; no new crates. |
| **F15** Capture independence | **Met** | String/TTY gate + existing parse. No models/graph/new events on usage path. |
| **F16** Plan-only until go | **N/A** | Implementation present; lock flips after go. |

### `INGEST_EMPTY_STDIN_USAGE`

Matches spec §4 normative block:

```
stdin is empty or not piped. Pipe a JSON turn. Example:
  echo '{"session_id":"00000000-0000-0000-0000-000000000001",...}' | ai-brains ingest --dry-run
```

---

## AC1–AC16 audit

| ID | Status | Evidence |
|----|--------|----------|
| **AC1** empty `--dry-run` exit 2 | **Met** | `ingest__dry_run__empty_stdin__usage_exit_2` + `assert_empty_stdin_usage`: exit 2, problem text, `ingest --dry-run`, `session_id`, no `COMMAND_FAILED`, no `EOF while parsing`, stdout empty. |
| **AC2** whitespace | **Met** | `ingest__dry_run__whitespace_stdin__usage_exit_2` uses `"\n  \n"`. |
| **AC3** live empty | **Met** | `ingest__live__empty_stdin__usage_exit_2` same helper. |
| **AC4** placeholder UUIDs | **Met** | Existing `ingest__dry_run__accepts_placeholder_uuids` unchanged (preview + success). |
| **AC5** `{` COMMAND_FAILED | **Met** | `ingest__dry_run__truncated_object__command_failed`: exit 1; combined has `COMMAND_FAILED` **or** `Invalid JSON`; no usage phrase. |
| **AC6** helper units | **Met** | `(true, None)` / `(false, Some(""))` / `(false, Some(" \n"))` true; `(false, Some("{"))` and valid object false. |
| **AC7** const keys + `'{` | **Met** | Seven keys + `ai-brains ingest --dry-run` + `'{` + `}'`. |
| **AC8** `ingest --help` keys | **Met** | `ingest__help__contains_example_keys` key presence only. `cli_help_ia` `long_help__daily_commands_before_harness_ingest` still group-order. |
| **AC9** T180 dual-path | **Met** | `t180_c_stdin_dry_run_deny__unknown_field__rejected` + `t180_c_stdin_prod_open__unknown_field__accepted` unchanged. |
| **AC10** empty content ≠ usage | **Met** | `ingest__dry_run__errors_on_empty_content`: `!success`, `code != 2`, stderr must not contain `stdin is empty or not piped`. Production still `content field is empty`. |
| **AC11** live UUID reject | **Met** | `ingest__non_dry_run__still_validates_uuids` unchanged. |
| **AC12** docs | **Met** | See docs section below. |
| **AC13** no contracts/crates | **Met** | No new contracts type; `IngestRequest` fields unchanged; no new crate. |
| **AC14** manual dogfood | **N/A this review** | Plan Phase 5. Static review cannot record live TTY/pipe. Implementation supports all three cases (TTY-before-read + empty usage + `{` envelope). |
| **AC15** targeted nextest/clippy | **Not executed** | Tests and code are present and would compile against the helper/const. Reviewer did not run nextest/clippy. |
| **AC16** plan-only | **N/A** | After go. |

**Would new tests fail against old behavior?** Yes for AC1–AC3 (exit 1 EOF JSON), AC6–AC7 (missing helper/const), AC8 (no after_help keys). Keep-green AC4/AC5/AC9–AC11 would still pass on old code — that is the intended freeze.

---

## Docs AC12 detail

| File | Accurate? |
|------|-----------|
| `Docs/CLI-EXIT-CODES.md` L13 table + L32 T252 footnote | Yes. Empty/TTY → 2 `fail_usage`; mid-payload → 1 envelope. Zero stdout stated. |
| `Docs/CAPABILITIES.md` §4 Manual / programmatic | Yes. Lists seven keys; empty/TTY → 2; `{` → 1; `--dry-run` same usage class. |
| `Docs/OPERATIONS.md` after ingest sample (L52) | Yes. One-liner immediately after `echo $json \| ai-brains … ingest`. |
| `CHANGELOG.md` Unreleased Changed | Yes. **T252 … (BREAKING):** 1 JSON → 2 human; non-empty parse stays 1. |

Phase 3 BREAKING grep is recorded in `plan.md` (hooks/scripts pipe built payloads; `.agents/skills/ai-brains/scripts/ingest.ps1` absent).

---

## Isolation (do-not-touch)

| Surface | Touched? |
|---------|----------|
| `ai-brains-contracts` `IngestRequest`/`IngestResponse` | No |
| `ai-brains-capture` `parse_ingest_request` | No |
| T180 deny_unknown_fields / `protocol_compat_cli.rs` | No rewrite |
| T114 `DryRunIngestRequest` shape | No (gate only **before** `from_str`) |
| `governed_common.rs` | Call `fail_usage` only |
| `is_vault_path_free` / `run_sync_path_free` | Ingest not added |
| Workspace/lock pins | Unchanged vs F10 |
| `cli_help_ia` group order | Unchanged |
| T86 stdin helpers | Unchanged (swallow remains F12) |

---

## Findings

None.

---

## Verdict

**PASS**
