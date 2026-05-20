## Spec: Deploy Stale recall.rs Bridge Fix (Track AB-F1)

### Acceptance Criteria

1. **No stale bridge query code**: `rg "bridge query.*--in" crates/` returns zero matches
2. **Binary deploys**: `ai-brains --help` shows the updated binary timestamp
3. **recall works silently**: `ai-brains recall "test" 2>&1` does NOT contain "ChangeGuard bridge query failed" on stderr
4. **Bridge integration active**: When ChangeGuard is available in the working directory, `ai-brains recall "<query>"` returns results that include hits from ChangeGuard's Tantivy search (observable via higher result counts or presence of code-oriented results)
5. **CI gate passes**: `cargo fmt --check`, `cargo clippy`, `cargo test` all pass
