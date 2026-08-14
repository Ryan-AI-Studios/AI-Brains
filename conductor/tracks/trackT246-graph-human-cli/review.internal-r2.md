# T246 internal R2 review — hermetic coverage of CX1 P2 T246-R2

**Reviewer:** Grok (read-only verification of commit `b06bb98` tests)  
**Date:** 2026-08-13  
**Source finding:** `review.codex.md` T246-R2 — required edge cases under-tested end-to-end  
**Evidence:** `crates/ai-brains-cli/tests/graph_human_cli.rs` vs production `crates/ai-brains-cli/src/commands/graph.rs` + `GraphCommands` dispatch in `main.rs`  
**Observed gate (orchestrator):** `cargo nextest run -p ai-brains-cli --features graph --test graph_human_cli` → 8/8 PASS  

Tests invoke `common::hermetic_vault` → `Command::cargo_bin("ai-brains")` (production binary), not formatter units.

## Verdict: FAIL

Most R2 cases now have a hermetic CLI test that would fail if that production path regressed. One listed diagnostic is still unit-only: **`graph session` missing-node**. That is a distinct `session()` `kind.is_none()` branch (`pretty_no_graph_node`) and is not exercised by `graph_hierarchy_session__missing_wrong_kind_empty__f3_copy`.

R2 is not fully closed.

## Bullet matrix

| R2 bullet | Test | Production path hit | Would fail on regression? | Result |
|-----------|------|---------------------|---------------------------|--------|
| present-but-empty neighbors | `graph_neighbors__present_empty__pretty_no_neighbors` | `neighbors()`: `node_kind` Some + empty `get_neighbors` → `pretty_no_neighbors` | Yes — exact `No neighbors for mem-leaf.` + `next: ai-brains graph update`; missing-node copy (`No graph node`) would fail | **PASS** |
| hierarchy/session missing, wrong-kind, empty diagnostics | `graph_hierarchy_session__missing_wrong_kind_empty__f3_copy` | hierarchy None / wrong kind / leaf; session wrong kind / empty | Hierarchy missing, both wrong-kinds, both empties: yes. **Session missing: no CLI call.** Merging session `None` into empty or wrong-kind copy would stay green | **FAIL** |
| hierarchy CLI pretty/JSON | `graph_hierarchy__pretty_and_json__indent_and_sorted_ids` | `hierarchy()` pretty (`get_synthesized_hierarchy_with_depth` + 2-space indent) and JSON (`get_synthesized_hierarchy` + lex sort) | Yes — indent/`└` and frozen keys + `["child-a","child-b"]` | **PASS** |
| JSON unlimited without `--limit` | `graph_neighbors__json_limit_and_sort__unlimited_unless_flag` | `neighbors()` JSON `json_take(None)` → `usize::MAX` | Yes — 52 hits; silent `clamp_list_limit(None)` (50) fails | **PASS** |
| explicit `--limit` clamp | same test, `--limit 2` | clap `--limit` → `json_take(Some(2))` → truncate | Yes for wiring/truncation. Does **not** prove `0→50` or max `200` | **PASS** (bounds residual) |
| JSON array ordering | same test (neighbors); hierarchy test for `synthesized_from` | `sort_neighbor_hits` (direction then id); hierarchy `.sort()` | Yes for incoming-before-outgoing + outgoing lex ids + hierarchy lex. Label tie-break and session lex remain unit / untested | **PASS** (narrow) |
| `graph update --format auto` stays pretty JSON | `graph_update__default_json_and_human__t213_keys_and_labels` | `update()`: `format == "human"` only → else `to_string_pretty` | Yes if `auto` is treated as `human`. Does not assert indented pretty vs compact; piped child cannot prove a TTY-only switch | **PASS** (TTY residual) |

## Remaining gaps (block R2 closure)

1. **Session missing (R2 text, not optional).** No `graph session <unknown> --format pretty` assertion of `No graph node`. Hierarchy missing is covered; session `None` is a separate match arm. Helper strings are still unit-pinned in `empty_pretty__missing_and_present__exact_f3_and_graph_update` — that is the coverage R2 rejected as insufficient.

## Residuals (do not flip this verdict by themselves)

- Diagnostic CLI cases (except present-empty neighbors) do not assert `next: ai-brains graph update` or exit 0. Dropping `PRETTY_NEXT` on hierarchy/session diagnostics would not fail the new hermetic.
- JSON unlimited / `--limit` only on `neighbors`. Hierarchy/session share `json_take`; forgetting it on those commands would not fail these tests.
- `--limit 2` is explicit apply, not F8 clamp bounds (`None`/`0` → 50, max 200). Pretty default-50 remains the unit `format_neighbors_pretty__51_rows_default_limit__50_lines_and_1_more`.
- Session JSON `memories` lex order not asserted (`graph_session__pretty_and_json` uses `.any(...)`).
- Neighbor sort does not vary `label` on the CLI fixture (unit AC6 still covers label).
- `--format auto` is parseable JSON with `status`/`density`; not a `to_string_pretty` shape check.

## Not gaps

- Tests are production-bin hermetics (`cargo_bin`), graph-on `#[cfg(feature = "graph")]`.
- Present-empty vs unknown-id (`graph_neighbors__unknown_id__pretty_no_node_json_empty_exit_0`) are distinct.
- `graph update` default (no flags) still pretty-JSON T213 keys; `--format human` still `status:` / `density:`.

## Required to flip PASS

Add one hermetic: `graph session <nil-or-unknown> --format pretty` → `No graph node` (and ideally `next: ai-brains graph update`, exit 0), so session missing cannot collapse into empty-session or wrong-kind copy.
