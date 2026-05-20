## Plan: Fix nightly Crash and Silent Progress (Track AB-F2)

### Summary
`ai-brains nightly` appeared to hang on "Scanning for Antigravity sessions..." but actually completed the scan (3534 turns, 67 sessions) after several minutes with zero progress output. It then crashed during "Summarizing sessions..." with exit code 255. Two issues: (1) no progress indicators make the scan look hung, and (2) the summarization step crashes.

### Phase 1: Fix Progress Reporting
- [ ] Task 1.1: Add progress output during Antigravity scan (e.g., "Scanning Antigravity... found N sessions (M turns)")
- [ ] Task 1.2: Add progress output during summarization (e.g., "Summarizing session N/M...")
- [ ] Task 1.3: Each long-running step should print at least one line so the user knows it's working

### Phase 2: Fix Summarization Crash (exit 255)
- [ ] Task 2.1: Reproduce the crash: run `ai-brains nightly` and capture the panic/error during summarization
- [ ] Task 2.2: Fix root cause (likely: unhandled session format, null field, or timeout in the summarizer)
- [ ] Task 2.3: Add error handling so one bad session doesn't crash the entire nightly sweep; log and skip instead

### Phase 3: Verify
- [ ] Task 3.1: `ai-brains nightly` prints progress for each major step
- [ ] Task 3.2: `ai-brains nightly` completes without crashing (success or clean skip per step)
- [ ] Task 3.3: A failing summarization of one session does not abort the entire sweep

### Phase 4: Gate
- [ ] Task 4.1: `cargo fmt --check` passes
- [ ] Task 4.2: `cargo clippy --workspace --all-targets -- -D warnings` passes
- [ ] Task 4.3: `cargo test --workspace` passes
