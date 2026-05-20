# Specification: Track T42 - Shared Knowledge Graph & Unified Search

## Objective
Deprecate AI-Brains' parallel graph and search indices by implementing a CozoDB proxy backend and unified IPC retrieval that delegates to ChangeGuard's Tantivy + CozoDB stack.

## Architecture & Scope
1. **CozoProxyBackend (`ai-brains-graph`)**: Introduce a new graph backend that translates AI-Brains graph mutations (node/edge insertions, reachability queries) into CozoDB Datalog statements and routes them through the BridgeRecord IPC pipe to ChangeGuard's CozoDB instance. This is a full Datalog translation layer — AI-Brains expresses queries in Datalog, not via pre-defined endpoints.
2. **Unified IPC Recall (`ai-brains-retrieval`)**: Update the `recall` function to stop shelling out to `changeguard ledger search`. Instead, perform a unified IPC request where ChangeGuard executes a blended Tantivy search returning both codebase context and conversational memory from the same index.

## Technical Constraints & Mandates
- **CQRS Integrity**: Graph backend purely routes mutations; retrieval purely reads.
- **Capture Independence**: Graph/search changes must NOT introduce dependencies into the capture path.
- **Fail-Open**: If ChangeGuard IPC is unavailable, retrieval must fall back to local FTS5 search gracefully.
- **Zero Copy**: Graph queries should avoid duplicating graph data locally — CozoDB is the single graph store.
- **Path Integrity**: All graph node paths routed through IPC must be normalized.
