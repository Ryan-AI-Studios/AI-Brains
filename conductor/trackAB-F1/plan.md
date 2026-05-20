## Plan: Deploy Stale recall.rs Bridge Fix (Track AB-F1)

### Summary
`crates/ai-brains-retrieval/src/recall.rs` was modified to use `changeguard search --json` instead of `changeguard bridge query --in`, but the installed binary was never rebuilt. Every `ai-brains recall` call hits the old broken codepath and falls back to local FTS5 only.

### Phase 1: Verify Source and Rebuild
- [ ] Task 1.1: Verify no stale `bridge query --in` invocations remain in the codebase (`rg "bridge query" crates/` — should only show comments and fallback messages)
- [ ] Task 1.2: `cargo build --release`
- [ ] Task 1.3: `cargo install --path .` (deploy to PATH)
- [ ] Task 1.4: Verify fix: `ai-brains recall "verification"` completes without `bridge query --in` error on stderr
- [ ] Task 1.5: Verify bridge integration: recall results include ChangeGuard hits (not just local FTS5)

### Phase 2: Gate
- [ ] Task 2.1: `cargo fmt --check` passes
- [ ] Task 2.2: `cargo clippy --workspace --all-targets -- -D warnings` passes
- [ ] Task 2.3: `cargo test --workspace` passes
