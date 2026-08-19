# T265 review log — Preflight JSON envelope

**Track:** `conductor/tracks/trackT265-preflight-json-envelope`
**Category:** FEATURE / CONTRACTS / UX
**FEATURE TX:** `b5b7c4e8-a8a0-4465-be35-625afc6ead0b`
**Date:** 2026-08-19

## Scope

Non-summary `preflight --format json` stays compact with required `text` /
`word_count` and additive always-present `sections[]` (`{id, title, items}`;
E1 `[]`). Split from the same assembled `text` (F5 pretty match + Ledgerful
`contains`; F6 blank-line items; session/index one item when no blank lines).
`--summary --format json` stays T220 (no `sections`). No `json-v2`, no typed
authority arrays, no retrieval/`safety_ids` edits, no clap 5, no lock bumps,
no live `.env` / `cargo install`.

## Reviewers / rounds

| Round | Source | Verdict |
|-------|--------|---------|
| R1 | Implementer vs AC1–AC16 / F0–F26 / DoD | PASS |
| R1b | Independent explore | **PASS** (0 findings) |
| CX1 | Codex FEATURE `gpt-5.6-luna` high | Product **PASS** (0 P0–P3). Process closeout closed by full gate + publish |

## Findings

None from R1.

## DoD matrix (implementer)

| Item | Status | Evidence |
|------|--------|----------|
| AC1 hermetic compact + required keys + sections array | met | `preflight_json_envelope__format_json__required_keys_compact_sections_array` PASS |
| AC2 T180-C compact + text/word_count + sections array; no `len==2` | met | `t180_c_preflight_json_keys__cli_format_json__compact_stable_keys` PASS |
| AC3 split ids in order; session/index one item | met | `split_preflight_sections__legacy_headers__ids_in_order` PASS |
| AC4 two Session headers → two rows | met | `split_preflight_sections__two_sessions__two_section_rows` PASS |
| AC5 no headers empty; preamble discarded | met | `split_preflight_sections__no_headers__empty` + `…leading_preamble__discarded` PASS |
| AC6 governed marker one section | met | `split_preflight_sections__governed_marker__one_section` PASS |
| AC7 N−1 2-key deserialize `sections: []` | met | `preflight_context_response__n_minus_1_two_key__sections_default_empty` PASS |
| AC8 T219 AC7 + T250 AC12 drop `len==2`; sections array | met | both PASS |
| AC9 T264 `[8hex]` in text; sections present | met | `preflight_global_isolation__compact_json__two_keys_and_hex_tags` PASS |
| AC10 summary JSON no `sections`; `api_version=="1"` | met | `preflight_summary_json__format_json__pure_object_no_banner` PASS |
| AC11 empty vault no fabricated `empty_repo` | met | `preflight_json_envelope__empty_vault__sections_empty_or_empty_repo_header` PASS |
| AC12 docs + no pin bumps + no `deny_unknown_fields` + retrieval untouched | met | CAPABILITIES / PROTOCOL-COMPAT / CHANGELOG; clap lock 4.6.1; git diff retrieval empty |
| AC13 manual source bin `-m 200` | met | `cargo run -p ai-brains-cli -- preflight --format json -m 200`: compact object; `sections[0].id=safety` |
| AC14 dogfood extra key still counts `text` | met | `parse_legacy_preflight__extra_sections_key__counts_markers_from_text` PASS |
| AC15 live Ledgerful headers + other | met | `split_preflight_sections__ledgerful_and_other` PASS |
| AC16 sibling emit; no inline sections construction | met | `preflight.rs` JSON arm calls `build_preflight_json`; `preflight_json.rs` exists |
| F5 / F6 / F10 / F11 / F12 / F25 | met | match table; session one-item; no json-v2; retrieval file untouched; `pub mod`; no version key |

## Targeted gates (observed)

- `cargo fmt --check` + `cargo clippy -p ai-brains-cli -p ai-brains-contracts --all-targets -- -D warnings` exit 0
- 15 targeted nextest PASS (units + T180-C + T219/T250/T264/T220 + envelope hermetics + dogfood extra-key)
- `test_preflight_json_output_with_scope` PASS

## Full gate (observed)

- First `dev-check.ps1` aborted: live daemon blocked `backup_restore__daemon_down_force__succeeds` (unrelated; same as T257). Stopped daemon.
- `.\scripts\dev-check.ps1` **[SUCCESS] CI Gate passed!** nextest **3153** passed (1 skipped)
- `cargo deny check` + `cargo audit` included in the gate (allowed unmaintained/unsound warnings only)
- `ledgerful verify --scope full` **passed** (fmt 2.5s / clippy 14.8s / nextest 232.1s / deny 2.4s / audit 2.9s)

## Residual / decline

- Pretty walker ≠ JSON splitter (F12; unify later if they drift)
- Index without blank lines = one item (F6 v1)
- F2b truncated `---` header stays in the previous section’s items (honesty; not a fabricated `index`)
- PATH until operator `cargo install` (F20)
- T272 `safety_ids` over-exclude (peer placeholder)
- json-v2 / typed arrays / summary envelope / clap `value_parser` declined
