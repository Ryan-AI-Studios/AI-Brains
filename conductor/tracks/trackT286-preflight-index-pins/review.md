# T286 review log — preflight Index TAGS-or-GLOB + envelope titles

**Track:** T286-PreflightIndexPins
**Branch:** `track/T286-preflight-index-pins`
**FEATURE TX:** `59d67348-953d-4d8d-9171-68a7cbfe95c8`
**Reviewers:** implementer (R1) → codex-review (FEATURE)

## Scope

Index pass-1 SQL extra is `index_pass1_glob_sql` (marker+HOTSPOT **or** TAGS envelope, single `AND (`). Index numbered titles use `first_contentful_line`; empty / role-only / TAGS-only → `Untitled Memory`. Summary JSON keys stay T220. CLI `preflight.rs` production / `pin.rs` / `lexical.rs` / T279 Safety SQL untouched.

**Did not:** `project.rs` / `sync.rs` / CLI `preflight.rs` production / `preflight_pretty.rs` / `pin.rs` write / `lexical.rs` / `preflight_safety.rs` / `ci.yml` / clap 5 / rusqlite 0.40 / T287 list ORDER / T288 briefing / T293 neighbors / `cargo install`.

## DoD matrix

| AC | Status | Evidence |
|----|--------|----------|
| AC1 | **Met** | `preflight__index_prefers_tags_envelope_decision_over_objective_dump` PASS (red: Index lacked DECISION) |
| AC2 | **Met** | same test: first numbered line `DECISION:` not `TAGS:` |
| AC3 | **Met** | T274 `preflight__index_prefers_leading_decision_over_objective_dump` PASS |
| AC4 | **Met** | `index_pass1_glob_sql__tags_or_marker__single_and_group` PASS (`TAGS:*` + `ASSISTANT: TAGS:*` + `DECISION:*` + `HOTSPOT:*` + one `AND (` + `debug_assert!(is_safe_sql_ident)`) |
| AC5 | **Met** | CLI `preflight__pretty_index_item1_is_decision_when_tagged_pin_exists` PASS (red: `1. ## Objective`) |
| AC6 | **Met** | `preflight__summary_json_tagged_pin__in_context_decisions_nonzero` in `preflight_summary_json.rs` PASS (red: 0 after stop-session) |
| AC7 | **Met** | T220 `preflight_summary_json__legacy_markers__in_context_counts_meaningful` PASS |
| AC8 | **Met** | T279 `preflight_safety_vs_hotspots` suite in `test(preflight)` 131/131 |
| AC9 | **Met** | `preflight_pretty__json_format__two_keys_and_newlines_in_text` + envelope tests: `sections` present |
| AC10 | **Met** | `index_item_title__empty_envelope__untitled_memory` + hermetic `preflight__index_tags_only_envelope__untitled_memory` PASS |
| AC11 | **Met** | T219 `preflight_pretty__multi_section__multiline_scope_no_assistant_prefix` PASS |
| AC12 | **Met** | AC1 does not assert dumps absent; dump may fill later slots |
| AC13 | **Met** | T264 `preflight_global_isolation` + T272 skip-set tests in 131/131 |
| AC14 | **Met** | AC6 asserts no `index_kind` / `in_context_authority`; T220 keys present |
| AC15 | **Met** | `memory list` ORDER not touched (T287 freeze) |
| AC16 | **Met (hermetic SoT)** | AC5/AC6. Live `cargo run` pretty Index may still be `## Objective` when no fitting in-scope pin enters pass-1 (R1-1). PATH not reinstalled (F21). |

## Findings

| id | severity | description | status | evidence |
|----|----------|-------------|--------|----------|
| R1-1 | low-info | Live `cargo run --pretty -m 1500` Memory Index line 1 was still `## Objective` on project `3581317d` (session dumps). Hermetic AC1/AC5 remain SoT. Drain still breaks a pass when the first addable row's **full** content exceeds `max_words` (pre-existing). Did not pin a live canary (F22). | deferred | not a hermetic ranking bug; F21 PATH; F12 Session chrome |

## Codex CX1 (gpt-5.6-luna, read-only)

Product **PASS**. Verdict at review time was FAIL for open closeout gates.

| id | severity | disposition |
|----|----------|-------------|
| P1-01 | process | **verified_fixed** — `dev-check` 3366/1 skipped + `verify --scope full` exit 0 + closeout + Phase 6 |
| P2-01 | medium | **verified_fixed** — unrolled AC6 dump pins (`792f489`); AC6 re-PASS |
| P3-01 | low-info | **deferred** — R1-1 live Index Objective; appended to `deferred.md` |

## Targeted gates (pre-full)

- `cargo fmt --check` PASS
- `cargo clippy -p ai-brains-retrieval -p ai-brains-cli --all-targets -- -D warnings` PASS
- `cargo nextest run -p ai-brains-retrieval -p ai-brains-cli -E "test(preflight)"` **131 passed**

## Full gate

- `.\scripts\dev-check.ps1` **SUCCESS** nextest **3366** passed / 1 skipped
- `ledgerful verify --scope full` exit 0

## Manual

```
cargo run -p ai-brains-cli --quiet -- preflight --pretty -m 1500 --no-hook-prompt
```

Safety: `No in-context hotspots. next: ai-brains safety sync --dry-run` (T279 honest empty). Index live item 1 still `## Objective` (R1-1). Hermetic AC5/AC6 are DoD.

Did **not** `cargo install`. Did **not** write `.env`. Did **not** pin production decisions.
