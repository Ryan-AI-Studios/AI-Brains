# Core Mandates - AI-Brains

1. **Capture Independence**: Hard prerequisite. The capture path (CLI -> Daemon -> Event Log) must work when graph DB, embeddings, local models, and cloud providers are offline.
2. **Canonical Event Store**: SQLCipher-backed append-only event log is the only source of truth. Never update or delete raw events. Use compensating events for corrections.
3. **CQRS Strictness**: Commands append events. Queries read projections. Do not mix read and write logic in the same transaction or service layer.
4. **No Hidden Thinking / Tool Logs**: Do not store internal chain-of-thought, model reasoning, or raw tool logs. Capture only the final assistant response and the user prompt.
5. **Privacy Inheritance**: Derived memories (summaries, clusters) must inherit the strictest privacy flag (`local_only`, `never_inject`, `sealed`) from their source events.
6. **No Repo Writes by Default**: AI-Brains must not write project-local files. Use global user storage (`$env:USERPROFILE\.ai-brains`) unless the user explicitly invokes a repo-write command.
7. **Rust Safety & Idioms**: No `unwrap()`, `expect()`, or `panic()` in production code. Use explicit error types. Use `zeroize` for sensitive key material in memory.
8. **Path Normalization**: All paths must be normalized to handle Windows drive-case, forward/backward slashes, UNC prefixes, and WSL `/mnt/c/` mappings consistently.
9. **TDD (Two-Commit Minimum)**: Behavioral correctness must be proven via tests before implementation. Commit 1 = failing tests (Red). Commit 2+ = implementation (Green).
10. **Provenance via ChangeGuard**: Every track implementation and major architectural decision must be recorded in the `changeguard ledger`.
11. **CI Gate**: Before every commit, the following must pass:
    `cargo fmt --check ; cargo clippy --workspace --all-targets -- -D warnings ; cargo nextest run --workspace ; cargo deny check ; cargo audit`
12. **Commercial License Safety**: Only use dependencies with permissive licenses (MIT, Apache, BSD). Reject AGPL, GPL, and SSPL components in the core path.
