## Plan: Fix preflight Duplicate Hotspots (Track AB-F3)

### Summary
`ai-brains preflight --max-words 1000` outputs the hotspot list twice in the same output. The "HOTSPOT: Brittle files identified by ChangeGuard:" section appears duplicated, wasting context tokens.

### Phase 1: Diagnose
- [ ] Task 1.1: Trace preflight output assembly to identify where the duplication originates
- [ ] Task 1.2: Determine if ChangeGuard is called twice, or if output is concatenated twice in preflight formatting

### Phase 2: Fix
- [ ] Task 2.1: Deduplicate hotspot entries by (path, score, reason) tuple
- [ ] Task 2.2: Ensure each "HOTSPOT: Brittle files..." section appears exactly once
- [ ] Task 2.3: Preserve all other preflight sections unchanged

### Phase 3: Verify
- [ ] Task 3.1: `ai-brains preflight --max-words 1000` shows each hotspot path exactly once
- [ ] Task 3.2: `ai-brains preflight --pretty` shows each hotspot path exactly once

### Phase 4: Gate
- [ ] Task 4.1: `cargo fmt --check` passes
- [ ] Task 4.2: `cargo clippy --workspace --all-targets -- -D warnings` passes
- [ ] Task 4.3: `cargo test --workspace` passes
