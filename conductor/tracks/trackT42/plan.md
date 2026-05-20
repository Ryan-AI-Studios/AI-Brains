## Plan: Track T42 - Shared Knowledge Graph & Unified Search

### Phase 1: CozoProxyBackend (Graph)
- [x] Task 1.1: Define `CozoProxyBackend` struct in `crates/ai-brains-graph/src/cozo_proxy.rs` implementing the `GraphBackend` trait.
- [x] Task 1.2: Implement Datalog statement translation — convert AI-Brains graph operations into CozoDB Datalog syntax (add_node, add_edge, query_neighbors, query_path).
- [x] Task 1.3: Route all translated Datalog statements through BridgeRecord IPC to ChangeGuard's CozoDB instance via NDJSON temp files.
- [x] Task 1.4: Implement CozoDB response parsing back into AI-Brains graph domain types (`CozoNamedRows`, `GraphNode`, `GraphEdge`, `GraphPath`).
- [x] Task 1.5: Feature-gate the `CozoProxyBackend` — auto-detects `.changeguard/` + `changeguard` CLI; marks unavailable if missing, callers fall back to SQLite.
- [x] Task 1.6: Write 11 tests — Datalog syntax validation, string escaping, availability gating, NamedRows deserialization.

### Phase 2: Unified IPC Recall (Retrieval)
- [x] Task 2.1: Modify `recall()` in `crates/ai-brains-retrieval/src/recall.rs` to use three-phase approach (IPC attempt → local FTS5 supplement → blend/dedup).
- [x] Task 2.2: Parse unified `BridgeRecord::Insight` responses with `memory_id`, `content`, `score`, `source` (code_context vs conversational_memory), and `privacy`.
- [x] Task 2.3: Blend results with bridge hits taking priority, deduplicated by `memory_id`; `RecallHit` now carries `privacy` field.
- [x] Task 2.4: Implement fail-open — bridge failures log warning and fall through to local FTS5 only.
- [x] Task 2.5: Write 4 tests — FTS constructor, bridge constructor with privacy, blend deduplication, bridge failure fallback.

### Phase 3: Verification
- [x] Task 3.1: `cargo fmt --check ; cargo clippy --workspace --all-targets -- -D warnings ; cargo nextest run --workspace` — ALL PASS
- [x] Task 3.2: Workspace-level tests pass with 0 failures across entire workspace.
- [x] Task 3.3: `ai-brains-graph` now exposes `CozoProxyBackend` via `pub mod cozo_proxy`.
