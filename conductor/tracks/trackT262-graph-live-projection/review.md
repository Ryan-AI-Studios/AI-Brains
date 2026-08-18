# T262 review log — Graph live projection

**Track:** `conductor/tracks/trackT262-graph-live-projection`
**Category:** FEATURE / BUGFIX
**FEATURE TX:** `f71cb7ac-8710-4120-8828-1817da6ee5fc`
**Date:** 2026-08-17

## Scope

A just-pinned DECISION whose printed ingest `turn_id` is what the operator
pastes into `graph neighbors` is a graph **memory** node with a session
`RECALLS` edge, without `graph rebuild`. Hole was identity (printed turn_id ≠
`MemoryId::new()` ≠ hasher turn node), not a dead T69 hook. Pretty missing-node
next is rebuild **iff** `memory_exists`; unknown / leaf / empty edges have no
remediator. JSON keys frozen. No live rebuild, no historical remint, no clap 5,
no DTO, no `cargo install`.

## Reviewers / rounds

| Round | Source | Verdict |
|-------|--------|---------|
| R1 | Implementer vs AC1–AC19 / F0–F35 / DoD | PASS |
| R1b | Independent explore | **PASS** (0 findings) |
| CX1 | Codex FEATURE `gpt-5.4` high | Product FAIL P2 session miss used `memory_exists` only |
| CX2 | Codex re-review after P2 fix | **PASS** — no product findings; CX1-P2 `verified_fixed` |

## Findings

| ID | Severity | Description | Status | Evidence |
|----|----------|-------------|--------|----------|
| CX1-P2 | medium | `graph session` used `memory_exists` only, so a real session missing from `graph_node` got F1b “not a vault memory id” | `verified_fixed` | Session miss now `memory_exists` OR `session_projection` COUNT (never `?`). Hermetic `graph_session__vault_session_missing_graph_node__next_rebuild` 3/3 graph_live_projection. CX2 source-verified. |

## DoD matrix (implementer)

| Item | Status | Evidence |
|------|--------|----------|
| AC1 serde missing/present `turn_id` | met | `crates/ai-brains-events/tests/turn_id_payload.rs` 4/4 |
| AC2 Some(turn_id) memory + RECALLS; `node_kind(event_id)==None` | met | `projector__user_prompt_with_turn_id__memory_node_and_recalls__ac2` |
| AC3 None → turn node is `event_id` | met | `projector__user_prompt_legacy_none__turn_node_is_event_id__ac3` |
| AC4 TurnProjection Some → memory_id matches | met | `turn_projection__with_turn_id__memory_id_matches__ac4` |
| AC5 TurnProjection None still inserts | met | `turn_projection__legacy_none__still_inserts_memory__ac5` |
| AC6 hermetic pin → neighbors JSON RECALLS, no rebuild | met | `pin__graph_on__printed_id_neighbors_json_without_rebuild__ac6` |
| AC7 hermetic pin → pretty `in`/`RECALLS`, no `No graph node` | met | `pin__graph_on__printed_id_neighbors_pretty__ac7` |
| AC8 pretty missing-node vault vs unknown | met | `pretty_no_graph_node__vault_memory__next_rebuild__ac8` + unknown |
| AC9 leaf / no-neighbors / session-empty no remediator | met | `pretty_hierarchy_leaf__no_graph_update_or_rebuild__ac9` |
| AC10 JSON keys frozen | met | `empty_pretty__json_keys_frozen__ac10` |
| AC11 feature-off exit 2 | met | existing `graph_neighbors__format_pretty__feature_off_exit_2` |
| AC12 clap xml exit 2 | met | existing `graph_neighbors__format_xml__clap_invalid_value` |
| AC13 capture independence | met | `capture_does_not_require_graph` |
| AC14 MemoryPinned units stay green | met | `test_projector_links_pinned_recall_memory_to_session` + `graph_aware_store_makes_recall_edge_visible_on_append` |
| AC15 live classify-only `7c3634fe` still `in RECALLS` | met | PATH graph-on Phase 0; no live rebuild |
| AC16 docs | met | CAPABILITIES §9 + OPERATIONS + PROTOCOL-COMPAT + CHANGELOG + skill one-liner |
| AC17 T246 empty-pretty rewritten | met | F31 unit + `graph_human_cli` present-empty / unknown copy |
| AC18 literals compile with explicit `turn_id` | met | capture Some; ~13 hand-built None |
| AC19 `vault_memory_present(Err)` → F1b | met | `vault_memory_present__query_err__false_unknown_graph_copy__ac19` |

## Targeted gates (observed)

- `cargo fmt --check` exit 0
- `cargo clippy -p ai-brains-events -p ai-brains-capture -p ai-brains-store --all-targets -- -D warnings` exit 0
- `cargo clippy -p ai-brains-graph --all-targets -- -D warnings` exit 0
- `cargo clippy -p ai-brains-cli --all-targets --features graph -- -D warnings` exit 0
- events `turn_id_payload` 4/4; store `turn_projection_turn_id` 2/2; graph projector 4/4
- CLI `--features graph -E "test(graph)"` **63/63** (includes AC6/AC7 + T246/T213/T232 keep-green)
- feature-off AC11; `capture_does_not_require_graph`

## Full gate (observed)

- `.\scripts\dev-check.ps1` **[SUCCESS] CI Gate passed!** nextest **3072** passed (1 skipped); deny 0.20.2; audit 0.22.2; 19 allowed warnings
- `ledgerful verify --scope full` passed (fmt 2.6s / clippy 10.7s / nextest 237.3s / deny 6.9s / audit 3.5s) before CX1-P2
- Post-P2 `.\scripts\dev-check.ps1` **[SUCCESS]** nextest **3072** (1 skipped); deny/audit. Graph-on hermetic 3/3.

## Residual / decline

- `DecisionRecorded` projector — F24 soft
- T213 F31 last-event vs last-graph timestamp — decline
- Historical backfill `MemoryPinned` for 36k pins — F35
- Neighbor UUID prefix — F17
- PATH `cargo install` — F22 operator
- T263–T271 / T240 F2 / T255 — declined
