## Spec: Make context Idempotent (Track AB-F4)

### Acceptance Criteria

1. **First run succeeds**: `ai-brains context` in a new directory creates a session and exits 0
2. **Second run succeeds**: `ai-brains context` with an existing session prints session info and exits 0 (no error, no "already exists" message as an error)
3. **--new-session works**: `ai-brains context --new-session` replaces the existing session with a new one
4. **--show works**: `ai-brains context --show` prints the current session details
5. **CI gate passes**: `cargo fmt --check`, `cargo clippy`, `cargo test` all pass
