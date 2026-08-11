**Findings**

- `P2` [symbol_bridge.rs](/C:/dev/AI-Brains/crates/ai-brains-cli/src/commands/symbol_bridge.rs:625) still fails open on the T233 safety-net path. For absolute symbol paths, the filter only rejects out-of-root files when both `canonicalize(file)` and `canonicalize(project_root)` succeed; the fallback branch at [symbol_bridge.rs](/C:/dev/AI-Brains/crates/ai-brains-cli/src/commands/symbol_bridge.rs:639) returns `true` on any canonicalization failure. That means a stale/deleted absolute path outside the registered root is still ingested into the alias project instead of being dropped. This violates the locked L6 behavior (“absolute outside root drop”) and can cross-pollute symbol memories across roots, which is exactly the edge this track is supposed to harden. Coverage also misses this case: [symbol_bridge.rs](/C:/dev/AI-Brains/crates/ai-brains-cli/src/commands/symbol_bridge.rs:1042) only tests an existing outside file, not a missing/stale absolute path.

**Verdict**

`FAIL`

**Assumptions**

Read-only review only. I did not rerun `nextest`, `clippy`, or the full workspace gate in this session; I relied on the checked-in diff, tests, `review.md`, and the gate results stated in your prompt.