# T264 review log — Preflight global isolation

**Track:** `conductor/tracks/trackT264-preflight-global-isolation`
**Category:** FEATURE / UX
**FEATURE TX:** `02ee555e-b659-4999-87b2-8477f23169f9`
**Date:** 2026-08-18

## Scope

`--global` preflight is a vault rollup that must label and cap per project so
foreign DECISIONs are not this-repo law. Tags live in retrieval text (`[8hex]` /
`[unknown]`); pretty upgrades the leading tag via `display_label`. Summary adds
span honesty. Project-scoped pretty stays unlabeled. Leftover drop from
`recall --global` is declined (F11). No `project.rs` edit, no T180 key growth,
no clap 5, no live `.env` / `cargo install`.

## Reviewers / rounds

| Round | Source | Verdict |
|-------|--------|---------|
| R1 | Implementer vs AC1–AC14 / F0–F30 / DoD | PASS |
| R1b | Independent explore | **PASS WITH DEFERRED P3** — P3-5 session-skip after tag `verified_fixed`; P3-1/2/3/4 deferred |
| CX1 | Codex FEATURE `gpt-5.6-luna` high | Product **FAIL** P2 prefix-as-identity |
| CX2 | Codex re-review | Product **FAIL** P2 pretty prefix collision (retrieval identity verified_fixed) |
| CX3 | Codex re-review after collision helper | **PASS WITH DEFERRED P3** |

## Findings

| ID | Severity | Description | Status | Evidence |
|----|----------|-------------|--------|----------|
| R1b-P3-1 | low-info | Index fetch window 80 can stay leftover-heavy | `deferred` | Tags still honest; AC10 is Safety |
| R1b-P3-2 | low-info | Span can count a Recent section later trimmed by word budget | `deferred` | F7 is post-cap pre-pretty; session emit now counted only when kept |
| R1b-P3-3 | low-info | `preflight.rs` pretty formatter still hosts peel call sites | `deferred` | Logic lives in `preflight_pretty.rs`; full move is a later pretty track |
| R1b-P3-4 | low-info | AC5 does not independently assert Index/Recent tags | `deferred` | Prefix helper is shared; Safety/Session locked |
| R1b-P3-5 | low-info | Tagged Safety broke multi-line session skip | `verified_fixed` | Skip uses untagged `safety_for_skip` |
| CX1-P2-1 | medium | RR/span used 8-hex prefix as identity | `verified_fixed` | `project_key` = full UUID; collision unit |
| CX2-P2 | medium | Pretty lookup picked first prefix match | `verified_fixed` | `unique_project_id_for_tag` None on collision |

## DoD matrix (implementer)

| Item | Status | Evidence |
|------|--------|----------|
| AC1 round-robin leftover/other interleave | met | `take_round_robin__leftover_then_other__interleaves_per_project` |
| AC2 empty / unknown / per_project=1 | met | `take_round_robin__empty_and_unknown__respects_max` |
| AC3 peel + chrome still strips | met | peel unit + `peel_global_tag__tagged_timestamp_role__chrome_still_strips_remainder` |
| AC4 upgrade alias / missing / `]` / leading-only | met | `upgrade_global_tag__alias_missing_and_bracket` |
| AC5 two-project pretty labels + two-line pin | met | `preflight_global_isolation__two_projects__pretty_labels_and_no_unlabeled_safety` |
| AC6 project-scoped no tags / no span | met | `preflight_global_isolation__project_scoped__no_tags_no_span` |
| AC7/AC8 summary span + JSON key | met | `preflight_global_isolation__summary_span_and_json_key` |
| AC9 compact JSON 2-key + `[8hex]` in text | met | hermetic AC9 + `t180_c_preflight_json_keys` |
| AC10 3A+1B B appears A capped (Safety) | met | `preflight_global_isolation__three_a_one_b__b_appears_a_capped` |
| AC11 compact still tagged + T250 numbers | met | `preflight_global_isolation__compact_still_tagged` |
| AC12 T214/T220/T250 stay green | met | those hermetic files + T180 |
| AC13 docs | met | CAPABILITIES + PROTOCOL-COMPAT + CHANGELOG |
| AC14 manual source bin | met | `cargo run -p ai-brains-cli -- preflight --global --pretty --compact -m 400 --no-hook-prompt` exit 0; Safety `[C:\dev\ai-brains]`; Session `d6fb6231` `[(no alias)]` (foreign; pass-with-observed-data) |
| F11 leftover recall drop declined | met | no recall/search filter |
| F23 no `project.rs` | met | call `display_label` / `truncate_chars` only |
| F30 first-line only | met | AC5 continuation untagged |

## Targeted gates (observed)

- `cargo clippy -p ai-brains-retrieval -p ai-brains-cli --all-targets -- -D warnings` exit 0
- retrieval AC1/AC2 units PASS
- CLI peel/upgrade/chrome units PASS
- hermetic isolation 6/6 after AC10 Safety-section scope
- T214 / T220 / T250 / T180 keep-green
- retrieval preflight include/dedup/governed/word-budget keep-green

## Full gate (observed)

- `.\scripts\dev-check.ps1` **[SUCCESS] CI Gate passed!** nextest **3100** (1 skipped) (pre-CX2 pretty-collision helper)
- After CX1-P2 + CX2-P2: retrieval RR 3/3; CLI collision unit; clippy retrieval+cli `-D warnings`
- `ledgerful verify --scope full` **passed** (fmt 3.1s / clippy 21.1s / nextest 330.0s / deny 11.7s / audit 3.2s) after CX1-P2

## Residual / decline

- Recall leftover-first under `--global` — F11; not a silent exclude
- Safety LIKE still omits pure `DECISION:` — F13
- Span vs marker-count disagreement — documented §5.4
- PATH until `cargo install` — F21
- `display_label` extract out of `project.rs` — soft, not this track
