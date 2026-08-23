# T287 review log — human `memory list` prefer-fill authority

**Track:** T287-MemoryListAuthority
**Branch:** `track/T287-memory-list-authority`
**FEATURE TX:** `9d8cdbb9-2c2f-4e78-83d0-60b239f9e1e3`
**Reviewers:** implementer (R1) → codex-review (FEATURE)

## Scope

Human `--status pinned` (default / `--format human`) prefer-fills leading-line `DECISION:` / `CONSTRAINT:` / `INVARIANT:` / `HOTSPOT:` via new `QueryStore::list_authority_memories` (bind-free GLOB-or-TAGS on `mp.content`) + `classify_pin_kind != Other` + `prefer_fill_authority`. Pass-1 over-fetches like T216 F43 so tagged Other GLOB rows cannot starve older pins. `preview_line` uses `first_contentful_line` with TAGS-only fallback (not `""`). JSON and store `list_memories` stay `updated_at DESC`. Forgotten / `--summary` / clap flags / JSON keys unchanged.

**Did not:** `project.rs` / `sync.rs` / `forget.rs` production / `graph.rs` / `session_chrome.rs` / `ranking.rs` / `lexical.rs` / `pin.rs` write / CLI `preflight.rs` / `ci.yml` / clap 5 / rusqlite 0.40 / T288 briefing / T293 neighbors / T299 forgotten-empty / `cargo install`.

## DoD matrix

| AC | Status | Evidence |
|----|--------|----------|
| AC1 | **Met** | `memory_list__human_limit_5__first_row_is_tagged_decision_not_objective` PASS (red: first row `## Objective`, tagged pin preview `TAGS: t287`) |
| AC2 | **Met** | `memory_list__json_limit_5__items0_stays_recency_dump` PASS (`items[0]` Objective; no `mix`/`authority` keys) |
| AC3 | **Met** | `preview_line__tags_envelope__decision_not_tags` + `preview_line__tags_only__fallback_non_empty` PASS; existing role-strip units stay green |
| AC4 | **Met** | `list_memories__limit_plus_one__returns_extra_row_for_more_available` PASS |
| AC5 | **Met** | `list_authority_memories__older_tagged_decision__returned_at_limit_1` PASS (`GLOB 'TAGS:*'` / `ASSISTANT: TAGS:*` / `DECISION:*` / `HOTSPOT:*` + single `AND (`) |
| AC6 | **Met** | `memory_list__format_json__schema_keys` PASS |
| AC7 | **Met** | `memory_list__mix_fixture_summary__counts_dumps_and_pin` + T216 summary unit PASS |
| AC8 | **Met** | `memory_list__forgotten_status__no_authority_promote_of_remaining_pin` PASS |
| AC9 | **Met** | `memory_list__human_limit_5__untagged_decision_prefer_filled` PASS |
| AC10 | **Met** | `memory_list__chrome_only_vault__first_row_stays_objective` PASS (F32) |
| AC11 | **Met** | `memory_list__missing_scope__exit_2_fail_usage` PASS |
| AC12 | **Met** | AC2 asserts no `mix` / `authority` keys; T216 keys present |
| AC13 | **Met** | AC1 asserts no `ASSISTANT: DECISION` / `ASSISTANT: TAGS` on human stdout |
| AC14 | **Met** | `forget_list_forgotten__matches_memory_list_status_forgotten` PASS |
| AC15 | **Met (hermetic SoT)** | AC1/AC9. Live `cargo run -- memory list --limit 5` on `3581317d` still recency chrome: pass-1 GLOB returned **0** rows in this project (F32). PATH not reinstalled (F17). |
| AC16 | **Met** | `prefer_fill_authority__cases__expected_ids` rstest overlap / authority-only / recency-only / limit PASS |
| AC17 | **Met** | `memory_list_help__mentions_human_authority_and_json_recency` PASS |
| AC18 | **Met** | CAPABILITIES Memory inventory T287 row; CHANGELOG Unreleased; OPERATIONS one-liner |

Starve-guard (in-scope F1): `memory_list__human_limit_5__newer_tagged_dumps_do_not_starve_older_pin` PASS.

## Findings

| id | severity | description | status | evidence |
|----|----------|-------------|--------|----------|
| R1-1 | low-info | Live `cargo run -- memory list --limit 5` on project `3581317d` (3777 status=pinned) first row remains `## Objective`. Pass-1 GLOB matched 0 rows in-scope, so F32 recency-fill is honest. Hermetic AC1 remains SoT. Did not `cargo install`. Did not rewrite `.env`. A F18 canary pin did not appear in this project's count (likely leftover-shell `PROJECT_ID` vs `.env` — pin.rs `std::env::var`; not this track). | deferred | T286 Index live residual analog; F17 PATH |

## Codex CX1 (gpt-5.6-luna, read-only)

Product **PASS** (implementation wired; no P0). Verdict at review time was FAIL for open closeout gates.

| id | severity | disposition |
|----|----------|-------------|
| P1-01 | process | **verified_fixed** — `dev-check` 3382/1 skipped + `verify --scope full` exit 0 + closeout + Phase 6 |
| P2-01 | medium | **verified_fixed** — AC2 asserts newest `dump four` needle (`8943200`) |
| P2-02 | medium | **verified_fixed** — `memory_list__human_tag_t287__mix_among_tag_matches_only` PASS |
| P2-03 | medium | **verified_fixed** — AC12 exact T216 top-level key set |

## Targeted gates (pre-full)

- `cargo fmt --check` PASS
- `cargo clippy -p ai-brains-cli -p ai-brains-store --all-targets -- -D warnings` PASS
- `cargo nextest run -p ai-brains-cli -p ai-brains-store --profile ci` **1658 passed** (8 slow)
- CX1 P2 retest: AC2 / tag-mix / AC1 **3 passed**

## Manual

```
cargo run -p ai-brains-cli --quiet -- memory list --limit 5
cargo run -p ai-brains-cli --quiet -- memory list --summary
cargo run -p ai-brains-cli --quiet -- memory list --format json --limit 1
```

Human first page: recency chrome (`## Objective` / review ingest). `--summary` `Pinned: 3777` / `Forgotten: 0`. JSON `items[0].preview` `## Objective`; keys T216; no `mix`. Hermetic AC1/AC2 are DoD.

## Full gate

- `.\scripts\dev-check.ps1` **SUCCESS** nextest **3382** passed / 1 skipped
- `ledgerful verify --scope full` exit 0

Did **not** `cargo install`. Did **not** write `.env`. Did **not** pin production architectural decisions (F18 canary attempted; not in list scope).
