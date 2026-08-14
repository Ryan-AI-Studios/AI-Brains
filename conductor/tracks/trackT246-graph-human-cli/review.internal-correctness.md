# T246 internal-correctness review

Reviewer: Grok (read-only). Workspace: `C:\dev\AI-Brains`. Spec: `conductor/tracks/trackT246-graph-human-cli/spec.md`. Date: 2026-08-13.

## Verdict: PASS

CLEAN.

## Findings

None.

## Checks (highest regression risks)

| Risk | Result |
|------|--------|
| Silent JSON key add on neighbors/hierarchy/session | **Clean.** `NeighborsOutput` / `HierarchyOutput` / `SessionOutput` serialize only frozen keys. Hermetic AC7 asserts exact key sets. `NeighborHit` remains `{ external_id, label, direction }`. Pretty-only `kind`/`preview`/`depth` live on `PrettyNeighborRow` / depth tuples, not serde. |
| TTY-switching `graph update` | **Clean.** Update `--format` default is `json`; parser is `json`/`auto`/`human`. Any non-`human` path uses `to_string_pretty`. `auto` does not call `is_terminal`. T74 `test_graph_health_smoke` still `from_str`s stdout. |
| `NeighborHit` fields / `get_neighbors` signature | **Clean.** Still `get_neighbors(&self, memory_id: &str) -> Result<Vec<NeighborHit>>`. Recall `--graph-boost` still reads `neighbor.external_id` only (`crates/ai-brains-retrieval/src/recall.rs`). |
| `ORDER BY` on `get_neighbors` / `get_session_memories` | **Clean.** Neither SQL gained `ORDER BY`. Neighbor/session sort is CLI-local (`sort_neighbor_hits`, `memories.sort()`). Only `get_synthesized_hierarchy_with_depth` has `ORDER BY MIN(r.depth), n.external_id`. |
| `get_synthesized_hierarchy` modified | **Clean.** Old DISTINCT id-only method unchanged. Pretty calls new crate method; JSON calls old then lex-sorts. No CLI CTE. |
| `unwrap`/`expect`/`panic` in production | **Clean.** `graph.rs` production uses `map_err`/`?`. `queries.rs` maps rusqlite errors. Test-only `expect` remains under `#[cfg(test)]`. |
| New crates / lock bumps | **Clean.** No `comfy-table`/`tabled`. Lock still clap **4.6.1**, serde_json **1.0.150**, rusqlite **0.39.0**, is-terminal **0.4.17**. Workspace pins unchanged. |
| Feature-off stubs / density in stubs | **Clean.** Vault-free and post-context stubs still print `FEATURE_UNAVAILABLE` + `GRAPH_REINSTALL_SOOT` and `exit(2)`. No assessor/density. AC11 covered in `exit_contract.rs` and `graph_human_cli.rs`. |
| Live rebuild | **Clean.** Track does not invoke rebuild. `Rebuild` variant still has no `--format`/`--limit`. |

## Spec pins

- **F1:** `--format: String` + `value_parser` on each read variant (not parent `Graph`). `resolve_graph_format` maps `pretty`/`human`/`text` → pretty; `json` → json; `auto` + TTY via `is_terminal::IsTerminal`. No `other => other` passthrough. Unknown → clap `InvalidValue` (AC12 unit in `main.rs`). Residual `_ => "json"` is fail-closed, not token passthrough (plan Phase 1).
- **F2:** Header `DIR LABEL ID KIND PREVIEW`; widths `{:<3} {:<16} {:<36} {:<14} {}`; display `in`/`out` only.
- **F3:** Exact empty strings + `next: ai-brains graph update` unit-pinned. `node_kind` first; one query on miss, `get_*` only when present. Hierarchy wrong kind → `No memory node`; session wrong kind → `No session node`. JSON miss/empty keeps old shapes.
- **F6:** Human = one labeled `println!` per field; `remediation:` only when `Some`.
- **F7:** Crate `get_synthesized_hierarchy_with_depth` uses `MIN(r.depth) … GROUP BY n.external_id`. AC16 diamond fixture: child once at depth 2; old method id set unchanged.
- **F8:** Pretty always `clamp_list_limit`; JSON `if limit.is_some() { clamp } else { usize::MAX }`.
- **F9:** Sort key is raw `incoming`/`outgoing`, then `label`, then `external_id` (unit AC6). Pretty `in`/`out` not used for sort.
- **F10/F11:** Preview only when `kind == "memory"` via `preview_line` 80 + role strip. Missing projection → empty cell (`optional()`). `node_kind` is `query_row` + `optional()`.
- **F12/F13/F15/F16:** No contracts DTO; `after_help` on `GraphCommands`; T213 `graph_health_output__*` units intact.
- **F14/AC14:** CAPABILITIES §9 TTY/pipe + `--limit`; PROTOCOL-COMPAT §5 TTY/pipe **and** array-order row; OPERATIONS paragraph; skill one-liner; CHANGELOG T246 row only.
- **Privacy:** Previews are vault `memory_projection.content` already shown by `memory list`. No key material printed.
- **Capture independence:** Read commands do not append events or load models.

## Live evidence (accepted, not re-run)

Plan Phase 7 + caller note: neighbors of `5a0e0a71-…` pretty shows incoming `RECALLS` with `KIND=session`; JSON keeps `incoming`/`external_id`; session `3b4e95b8-…` pretty 5 memories with previews; nil UUID pretty is honest `No graph node`; `graph update --format human` is labeled. No live `graph rebuild` in this review.

## Not findings

- Pretty N+1 `node_kind`/preview before `--limit` truncate (F11 allows N+1 this track; batch is F17).
- `deferred.md` still saying T246 Planning while code is in-tree (conductor hygiene, not CLI contract).
- Soft residuals F17–F19 (tree/mermaid, projector completeness, F31 freshness) remain out of scope.
