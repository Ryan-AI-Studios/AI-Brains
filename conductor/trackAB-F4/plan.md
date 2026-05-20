## Plan: Make context Idempotent (Track AB-F4)

### Summary
`ai-brains context` errors with "Session X already exists" on every invocation after the first. The Phase 1 Orient workflow calls `context` unconditionally, so this breaks the workflow for every session beyond the initial one.

### Phase 1: Fix
- [ ] Task 1.1: Change the "session already exists" code path from error to success
- [ ] Task 1.2: When session exists, print current session info (equivalent to `--show`) and exit 0
- [ ] Task 1.3: Keep `--new-session` flag for the explicit replacement case
- [ ] Task 1.4: Keep `--show` flag for explicit display

### Phase 2: Verify
- [ ] Task 2.1: First run: `ai-brains context` creates session, exits 0
- [ ] Task 2.2: Second run: `ai-brains context` shows existing session, exits 0 (no error)
- [ ] Task 2.3: `ai-brains context --new-session` still replaces existing session
- [ ] Task 2.4: `ai-brains context --show` still prints session details

### Phase 3: Gate
- [ ] Task 3.1: `cargo fmt --check` passes
- [ ] Task 3.2: `cargo clippy --workspace --all-targets -- -D warnings` passes
- [ ] Task 3.3: `cargo test --workspace` passes
