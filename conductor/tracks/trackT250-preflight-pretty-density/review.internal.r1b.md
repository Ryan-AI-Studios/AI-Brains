# T250 Internal Review R1b — Preflight pretty density

**Track:** T250-PreflightPrettyDensity  
**Category:** UX / FEATURE  
**Branch:** `feature/T250-preflight-pretty-density`  
**Reviewer:** Grok (read-only correctness, r1b)  
**Date:** 2026-08-14  
**Known gates (orchestrator):** clippy `-D warnings` PASS after routing standard pretty through `format_preflight_pretty_body`; nextest preflight 66/66 PASS.

## Verdict: PASS

No High/Medium/Low product findings on the 14 hunted risk categories. Pretty density is display-only; JSON/`--summary` isolation, `strip_role_prefix` SOOT, governed Other, chrome char-bound, turn counting, and hermetic seed-line selection hold.

## Findings (id, severity, evidence, required_fix, status)

None.

## Hunt checklist (no invented issues)

| # | Risk | Result | Evidence |
|---|------|--------|----------|
| 1 | JSON text accidentally line-capped | **Clear** | `run()` applies PrettyCaps only inside `human_mode`. Else `PreflightContextResponse { text: context.text, word_count }` — raw post-F1 body. Summary returns before pretty. Hermetic AC12: `--compact --format json -m 3000` keeps full seed in `text`, exactly two keys, no Scope. |
| 2 | `strip_role_prefix` behavior changed | **Clear** | `display_text::strip_role_prefix` still leading-only `USER:`/`ASSISTANT:`/`SYSTEM:`; mid-line and lowercase left. T224 units unchanged. `preview_line` still strip-then-`truncate_preview_chars`. Pretty chrome is a **new** `strip_pretty_chrome` in `preflight.rs` only. |
| 3 | `#` / `##` treated as `---` headers | **Clear** | `is_legacy_section_header` requires trim start `---` **and** end `---`. `#`/`##` stay in Other. Units: T219 `format_preflight_pretty_body__governed_markdown__preserves_hash_headers`; T250 `format_preflight_pretty_body_with__governed_hash_headers__full_body_both_caps` (standard **and** compact). |
| 4 | Other/governed 200-char lines capped | **Clear** | Other arm: `display_pretty_line(line, None)` — chrome only. AC7 unit: 200-char `z` body under `##` present in full on both `standard()` and `compact()`. No governed section parser. |
| 5 | Headers or F31 notices truncated | **Clear** | Headers emitted as `h.trim()` (never `display_pretty_line`). F31 strings (`+N more safety…`, `+N more turns…`, `+N more via recall`, `+N more sessions`) pushed raw. Unit `format_preflight_pretty_body__long_header_and_notice__not_line_capped` locks 160-char header + exact safety notice, no `…`. |
| 6 | Turn counting after chrome strip | **Clear** | `is_session_turn_start` uses `has_leading_role_prefix(line.trim_start())` on the **original** retrieval line. Display is a later `display_pretty_line`. Comment matches producer: `ROLE: {truncate_turn}` — role only on first physical line (T219 M1). Counting after chrome would zero `turn_total` (stripped body has no leading role). Multi-line unit still green. |
| 7 | `unwrap`/`expect`/`panic` in production | **Clear** | T250 production (`PrettyCaps`, `strip_pretty_chrome`, `display_pretty_line`, `emit_item_block`, `format_preflight_pretty_body*`) uses `unwrap_or` / `unwrap_or_else` only. All `expect`/`panic!` in this file are under `#[cfg(test)]`. `rest[close+1..]` is at ASCII `)` so a char boundary. |
| 8 | for-loops in **new** tests | **Clear** | T250 units unroll fixtures (no `1..=N` case loops). T219 fixture/scan loops left in place. Hermetic AC10 `for line in stdout.lines()` is the same AC5 “no leading ASSISTANT:” scan, not rstest-style parameterization. |
| 9 | `--format compact` as a token | **Clear** | Preflight `--format` stays `Option<String>` (help: `human \| json \| pretty`). Density is `#[arg(long)] compact: bool`. Clap unit `preflight__compact_pretty__parses`. No `value_parser` adding `compact` (would collide with T129 `--log-format compact`). Unknown `--format compact` stays today’s non-human → JSON path (F10). |
| 10 | `PreflightContextResponse` growth | **Clear** | Contracts struct still `{ text, word_count }` only. JSON constructor unchanged. AC12 `obj.len() == 2`. `PreflightSummaryJson` is the pre-existing T220 CLI envelope — not grown here. |
| 11 | `first_line_only` dropping Recent recall hint | **Clear** | Recent skips hint blocks in the item take, then re-appends original lines matching `is_recall_hint_line` with **no** line-cap. Compact unit `pretty_caps_compact__item_caps_and_f31_and_recent_hint` asserts `(Use 'recall'…)` kept after first-line-only + recent cap 2. |
| 12 | Race/wrong Recent seed (index-only 60-char `...`) | **Clear** | Retrieval Index is `truncate_index_summary` 60 + ASCII `...`. AC10 finds a `T250SEEDLONG` line that is **not** numbered `N. `; asserts Unicode `…`, `chars() ≤ 140`, full seed absent, `-m 3000`, seed prefix present. Index-only would `expect` fail (fail-closed), not pass on 60-char `...`. Pin-last + overflow fixture; 66/66 includes this test. |
| 13 | Byte-index 34 chrome bound | **Clear** | `inner.chars().count() <= 32` (not `close <= 34`). `(999 mo ago)` strips; 33-char inner + role is fail-closed (`strip_pretty_chrome__inner_33_chars_with_role__fail_closed`). Declined AI2 M3 not implemented. |
| 14 | Dead code / unused wrapper | **Clear** | `format_preflight_pretty_body` → `_with(..., standard())` (F4). `run()` uses the wrapper on non-compact human; T219 units still call it. Compact uses `_with(..., compact())`. Single truncate SOOT: `display_text::truncate_preview_chars` (no `truncate_pretty_line` in `preflight.rs`; no leftover private helper in `memory.rs`). Clippy `-D warnings` PASS. |

## Wiring (isolation)

| Path | Behavior | OK? |
|------|----------|-----|
| `human_mode && compact` | Scope + `format_preflight_pretty_body_with(..., compact())` | Yes |
| `human_mode && !compact` | Scope + `format_preflight_pretty_body` (standard wrapper) | Yes |
| JSON non-summary (`--compact` ignored) | Compact `{text, word_count}` from raw `context.text` | Yes |
| `--summary` (`--compact` ignored) | Early `print_summary`; T214 banner / T220 JSON | Yes |
| `strip_role_prefix` / recall / forget / memory list | Unchanged leading-only SOOT | Yes |
| `trim_to_word_budget` / `truncate_turn` / `truncate_index_summary` | Untouched | Yes |

## Caps vs spec F1/F2

| Mode | safety | turns | sessions | index | recent | line_max | first_line_only |
|------|--------|-------|----------|-------|--------|----------|-----------------|
| `PrettyCaps::standard()` | 8 | 6 | 3 | 15 | 3 | 140 | false (Safety **not** line-capped) |
| `PrettyCaps::compact()` | 3 | 2 | 1 | 5 | 2 | 100 | true (Safety/Recent first line; Session still multi-line) |

F31 notices use the active N. Default pretty still shows full Safety lines (F11 residual note in `after_help` / CAPABILITIES / OPERATIONS).

## AC snapshot (code review, not a new gate run)

| ID | Status | Notes |
|----|--------|-------|
| AC1 | Met | Session/Recent 200-char → ≤140 + `…`; 80-char unchanged |
| AC2 | Met | `pretty_caps_standard__t219_item_caps` 8/6/3/15/3; T219 F31 units still call wrapper |
| AC3 | Met | Compact 3/2/1/5/2 + F31 + recall hint |
| AC4 | Met | chrome units including `(999 mo ago)` and 33-char fail-closed |
| AC5 | Met | `strip_role_prefix` units unchanged |
| AC6 | Met | long header + exact F31 notice |
| AC7 | Met | `#`/`##` + 200-char Other body both modes |
| AC8 | Met | T219 orphan unit unchanged |
| AC9 | Met | `truncate_preview_chars` em-dash / CJK units |
| AC10 | Met | hermetic `-m 3000` + seed prefix + non-index ≤140 `…` |
| AC11 | Met | hermetic compact fewer lines + F31 |
| AC12 | Met | hermetic compact JSON uncapped 2-key |
| AC13 | Met | hermetic `--summary --compact` still T214 banner |
| AC14 | Present | CAPABILITIES + CHANGELOG + OPERATIONS + Preflight `after_help` (not a hunt fail if wording polish remains) |
| AC15–16 | Process | Full workspace gate / live dogfood owned by orchestrator; not re-run in this pass |

## Completeness

T250 is presentation-only. Highest-regression surfaces from spec F15 were checked against source and hermetics; none landed. No production `unwrap`/`expect`/`panic` in the new pretty path. Wrapper is required (F4) and live.
