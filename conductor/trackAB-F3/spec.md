## Spec: Fix preflight Duplicate Hotspots (Track AB-F3)

### Acceptance Criteria

1. **No duplicate sections**: "HOTSPOT: Brittle files identified by ChangeGuard:" appears exactly once in preflight output
2. **No duplicate paths**: Each hotspot file path appears exactly once within the output
3. **Output structure preserved**: All other sections (bearings, memory index, recent memories) appear unchanged
4. **--pretty flag unaffected**: `--pretty` output is also deduplicated
5. **CI gate passes**: `cargo fmt --check`, `cargo clippy`, `cargo test` all pass
