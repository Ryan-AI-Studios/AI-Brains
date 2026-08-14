# T246 internal completeness review (read-only)

**Reviewer:** Grok (internal-completeness)  
**Date:** 2026-08-13  
**Scope:** working tree vs spec F1–F16 / AC1–AC16 / §14 pins. Soft F17–F20 not DoD.

## Verdict: PASS

## AC/F matrix (one row each)

| ID | Verdict | Evidence |
|----|---------|----------|
| **F1** | PASS | `GraphCommands` Neighbors/Hierarchy/Session: `format: String`, default `auto`, `value_parser = ["auto","pretty","human","text","json"]` on each variant (not parent `Graph`). `resolve_graph_format`: pretty/human/text→pretty; json→json; auto+TTY→pretty; auto+pipe→json. `_` is fail-closed `"json"` (not `other` passthrough). Probe `std::io::stdout().is_terminal()` via `is_terminal::IsTerminal`. Unknown token clap `InvalidValue`. |
| **F2** | PASS | `format_neighbors_pretty`: title `Neighbors of <id> (n)`, header `DIR LABEL ID KIND PREVIEW`, format `{:<3} {:<16} {:<36} {:<14} {}`. DIR `incoming`→`in`, `outgoing`→`out`. JSON keeps `incoming`/`outgoing`/`external_id`. |
| **F3** | PASS | `node_kind` first. `None` → `No graph node for <id>.`; hierarchy wrong kind → `No memory node`; session wrong kind → `No session node`; present-empty → neighbors / leaf / session-empty copy. All include `next: ai-brains graph update`. Exit `Ok(())`. JSON empty frozen keys. Missing path does not call `get_neighbors` / hierarchy / session walk. |
| **F4** | PASS | Feature-off vault-free + vault stubs still `FEATURE_UNAVAILABLE` + `GRAPH_REINSTALL_SOOT` + `exit_code_feature_unavailable()`. Same `GraphCommands` accepts new flags. `graph --help` still exit 0 (T198 smoke). No density logic in stubs. |
| **F5** | PASS | Keys: neighbors `{memory_id, neighbors:[{external_id,label,direction}]}`; hierarchy `{root,synthesized_from}`; session `{session_id,memories}`. Compact `to_string`. Zero additive keys (`kind`/`preview`/`depth`/`truncated` absent). |
| **F6** | PASS | Update `--format` default `json`; parser `json`/`auto`/`human`. Non-human → `to_string_pretty`. `auto` does not TTY-switch. Human = one `println!` per field; `remediation:` only when `Some`. Still `gather_density_snapshot` + `assess_graph_density`. No `--limit` on Update. |
| **F7** | PASS | `GraphSearch::get_synthesized_hierarchy_with_depth` in `queries.rs`; same CTE; `MIN(r.depth)` `GROUP BY n.external_id` `ORDER BY MIN(r.depth), n.external_id`. CLI does not duplicate CTE. `get_synthesized_hierarchy` unchanged (DISTINCT, JSON path + lex sort). Pretty indent `2 * depth`; no `└`. |
| **F8** | PASS | `-l`/`--limit` on three reads. Pretty always `clamp_list_limit` (None/0→50, max 200) + `… and N more`. JSON `json_take`: `limit.is_some()` else `usize::MAX`. Rebuild/Update have no `--limit`. |
| **F9** | PASS | CLI-local `sort_neighbor_hits` on raw `direction`/`label`/`external_id` (pretty + JSON). Hierarchy pretty uses crate depth-then-id order. JSON `synthesized_from` / `memories` `.sort()`. No `ORDER BY` on `get_neighbors` / `get_session_memories`. |
| **F10** | PASS | `NeighborHit` still three serde fields; `get_neighbors` signature unchanged. Recall still uses `external_id` only. Pretty KIND = `node_kind`; PREVIEW only `kind == "memory"` via `preview_line(..., 80)` + role strip. Missing projection → empty cell. |
| **F11** | PASS | `node_kind`: `query_row` + `optional()`, `SELECT kind FROM graph_node WHERE external_id = ?1 LIMIT 1`. Hierarchy requires `memory`; session requires `session`. No `node_kinds` batch. |
| **F12** | PASS | No new crates (no comfy-table/tabled). Workspace pins unchanged (`clap` 4.5, `rusqlite` 0.39.0, `is-terminal` 0.4, `serde_json` 1.0). No contracts DTO. `default = []`. Capture-independent (read SQL + formatters). |
| **F13** | PASS | No live `graph rebuild` in tests or wiring. Assessor / projector / recall scoring untouched. |
| **F14** | PASS | CAPABILITIES §9 table TTY/pipe + `--format`/`--limit`. PROTOCOL-COMPAT §5 TTY/pipe row **and** array-order row (pre-T246 encounter order). OPERATIONS T246 paragraph. Skill one-liner. CHANGELOG Unreleased T246 row. |
| **F15** | PASS | `#[command(after_help = …)]` on `GraphCommands` enum; neighbors default + `--format json` examples. `display_order = 57` not restacked. |
| **F16** | PASS | `graph_health_output__*` units remain. T74 `test_graph_health_smoke` still parses default `graph update` JSON. Feature-off `graph__default_build` + AC11 tests remain. Orchestrator: clippy + nextest graph-on/off green. |
| **AC1** | PASS | Units: `resolve_graph_format__auto_tty__pretty`, `auto_pipe__json`, `pretty_human_text__pretty_regardless_of_tty`, `json__json_regardless_of_tty`. |
| **AC2** | PASS | `format_neighbors_pretty__incoming_and_outgoing__header_in_out_kinds` + `format_neighbors_json__fixture__keeps_incoming_outgoing_external_id`. |
| **AC3** | PASS | `empty_pretty__missing_and_present__exact_f3_and_graph_update` pins all F3 strings + `graph update`. |
| **AC4** | PASS | `format_hierarchy_pretty__depth_1_and_2__indent_2_and_4_no_box`. |
| **AC5** | PASS | `format_neighbors_pretty__51_rows_default_limit__50_lines_and_1_more`. |
| **AC6** | PASS | `sort_neighbor_hits__incoming_before_outgoing_then_label_then_id`. |
| **AC7** | PASS | Hermetic `graph_neighbors__json_and_pretty__frozen_keys_and_dir` (pin + seeded `RECALLS`). |
| **AC8** | PASS | Hermetic `graph_neighbors__unknown_id__pretty_no_node_json_empty_exit_0`. |
| **AC9** | PASS | Hermetic `graph_session__pretty_and_json__id_preview_and_compact_ids`. |
| **AC10** | PASS | Hermetic `graph_update__default_json_and_human__t213_keys_and_labels`. |
| **AC11** | PASS | Feature-off hermetic in `graph_human_cli.rs` + `exit_contract.rs`. |
| **AC12** | PASS | `graph_neighbors__format_xml__clap_invalid_value` (`ErrorKind::InvalidValue`; clap usage exit 2). |
| **AC13** | PASS | Orchestrator live dogfood (graph-on debug exe, no rebuild): `in RECALLS` + session kind; piped/`--format json` compact frozen keys; session pretty 5 rows + previews; nil UUID `No graph node` + next; update default pretty JSON; `--format human` labeled + remediation. Code path matches. |
| **AC14** | PASS | Docs listed under F14. |
| **AC15** | PASS | `NeighborHit` / `get_neighbors` frozen; retrieval still `neighbor.external_id` only; no scoring change. |
| **AC16** | PASS | `get_synthesized_hierarchy_with_depth__diamond__min_depth_once` (depth 1+2, child once, old id set equal). |
| **§14 pin 1** | PASS | Depth method in graph crate; old DISTINCT stays for JSON. |
| **§14 pin 2** | PASS | Sorted arrays + PROTOCOL-COMPAT array-order row. |
| **§14 pin 3** | PASS | clap reject; `format: String`; no resolve passthrough. |
| **§14 pin 4** | PASS | JSON unlimited unless `--limit` is `Some`. |
| **§14 pin 5** | PASS | Widths 3/16/36/14; preview only `memory`. |
| **§14 pin 6** | PASS | Update default `json`; `auto` JSON; human labeled `println!`. |

## Findings

CLEAN.

Wiring is end-to-end (`GraphCommands` → dispatch → `graph.rs` formatters → `queries.rs` `node_kind` / `get_synthesized_hierarchy_with_depth`). No placeholders (`TODO`/`unimplemented`) in the track surface. Hard F17–F20 residuals correctly not implemented.
