# Architectural & Implementation Deviations

This document records the intentional deviations from the original `Implementation-Plan.md` that occurred during development, along with the rationale and technical context for each decision.

## 1. Storage & Encryption (Phase 5)
*   **Original Plan:** Use `sqlcipher-bundled` for transparent, full-database AES-256 encryption.
*   **Historical deviation:** Degraded to standard `bundled` SQLite for development when the Windows toolchain lacked OpenSSL/Perl to build SQLCipher from source.
*   **Resolution (T187, 2026-08-02):** Default workspace `rusqlite` features are now `bundled-sqlcipher-vendored-openssl`, `backup`, `fallible_uint` (still pinned at **0.39.0**). New vaults are page-encrypted; wrong key fails closed; legacy plain SQLite files return `LegacyPlaintextVault` with a `vault encrypt` (`sqlcipher_export`) migrate path. Zero keys refused unless `AI_BRAINS_ALLOW_ZERO_KEY=1`. **Not** FIPS; **not** NIST Purge/Destroy; Content Envelope remains a separate layer (page key ≠ content DEK). Build prereq on Windows MSVC: **Perl** on PATH (Strawberry Perl) for `openssl-src`.

## 2. Graph Database Compilation (Phase 8 & 12)
*   **Original Plan:** The `ai-brains-graph` crate, wrapping LadybugDB (a C++ embedded property graph DB), is a mandatory dependency for all retrieval and intelligence operations. Note: The PRD explicitly rejected the original KuzuDB in favor of the active LadybugDB/lbug fork.
*   **Resolution (ADR-0009):** Replaced LadybugDB with a Relational Graph model in SQLite using Recursive CTEs. This eliminates the C++ build friction (MSVC Debug linker failure LNK1248) while maintaining the required graph traversal capabilities within the existing SQLCipher foundation.

## 3. Date & Time Management
*   **Original Plan:** Not strictly specified, but generally leaned towards the `time` crate for lightweight timestamps.
*   **Deviation:** Standardized on `chrono` across all crates.
*   **Rationale:** `chrono` provided better out-of-the-box support for RFC3339 string generation and formatting, which was crucial for generating filename-safe timestamps (e.g., replacing `:` with `-` in backup folder names) and JSON serialization boundaries.

## 4. Retention & Privacy (Phase 11)
*   **Original Plan:** Event log is append-only; projections are deterministic rebuilds. No explicit "forget" mechanism beyond not loading certain nodes into the graph.
*   **Deviation:** Added a `last_accessed_at` column to `turn_projection` and introduced a dedicated `RetentionService` (90-day expiration) and a CLI `forget` command.
*   **Rationale:** Privacy regulations and practical disk space concerns necessitate soft-deletes and data expiration. By updating the `turn_projection` with a `forgotten` status, we prevent sensitive or old data from being retrieved via FTS or Graph, while keeping the underlying Event Log intact for cryptographic auditability.

## 5. Memory Intelligence & RAPTOR (Phase 10)
*   **Original Plan:** Rely heavily on multi-hop graph traversal (LadybugDB) for memory synthesis and long-term intelligence.
*   **Deviation:** Implemented RAPTOR-style hierarchical clustering and CRAG factual verification directly over the FTS/Lexical search read-models.
*   **Rationale:** To protect the background Nightly worker from the instability of the C++ graph build on Windows, the synthesis engine was decoupled from the graph. It currently relies on the standard `QueryStore` interface, ensuring high-level knowledge extraction works purely on the SQLite event projections.

## 6. Local Model Provider Integration (Phase 10 & 14)
*   **Original Plan:** Use `OllamaProvider` as the primary local intelligence engine.
*   **Deviation:** Implemented a custom `LlamaCppProvider` and transitioned to a multi-stage RAG strategy using environment-based model selection.
*   **Rationale:** The user's environment uses a high-performance Intel Arc B580 with a custom `llama-server` router. The standard Ollama API was insufficient for the required multi-model swapping (BGE-M3 for embeddings, Qwen 3.5 for completion) within strict 12GB VRAM limits. The dynamic configuration via `.env` allows for rapid model swaps without code changes.
## 7. Summarization Truncation & Hardware Constraints (Track T34)
*   **Original Plan:** Summarize sessions in a single LLM pass.
*   **Deviation:** Implemented Sequential Chunking with context carryover for sessions exceeding the 38,912 token limit.
*   **Rationale:** The Intel Arc B580 has a soft VRAM limit of ~10 GB. Exceeding this via a large context window (e.g., 64k) triggers a performance-killing spill into System RAM (34 TPS to 2 TPS). By enforcing a 38,912 token limit and processing oversized Antigravity logs in sequential parts, we ensure stable, high-performance summarization for sessions of any length without losing context.
