## Spec: Fix nightly Crash and Silent Progress (Track AB-F2)

### Acceptance Criteria

1. **Progress visible**: Each major step (Antigravity scan, summarization, graph rebuild) prints at least one progress line
2. **Antigravity scan reports counts**: Output includes session/turn counts (e.g., "Antigravity import complete. Processed N turns from M sessions.")
3. **No crash on summarization**: `ai-brains nightly` completes with exit code 0, not 255
4. **Resilient summarization**: A single bad session does not abort the entire sweep; errors are logged and skipped
5. **Idempotent**: Running `ai-brains nightly` twice in succession produces consistent results (second run should be fast since nothing new to process)
6. **CI gate passes**: `cargo fmt --check`, `cargo clippy`, `cargo test` all pass
