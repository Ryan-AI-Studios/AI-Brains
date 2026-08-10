Verdict: **PASS WITH DEFERRED P3**

P0: None.  
P1: None.  
P2: None.

P3:

- **AC10 test depth:** Hermetic tests exercise `semantic_search_with_embedding` and `fuse_local_and_semantic` directly, rather than invoking `recall_full` end-to-end. Production wiring is correct at [recall.rs:372](/C:/dev/AI-Brains/crates/ai-brains-retrieval/src/recall.rs:372). Defer optional full-path/httpmock coverage.
- F18/F19/F20/F21/AC15 remain intentional soft residuals per spec.

Product evidence is complete: dual-floor SOOT, cosine preservation, additive score metadata, ScoreKind-aware pretty output, injection seam, denylist updates, and `cargo fmt --check` pass. No product changes exist after `fc4d370`; current edits are governance closeout only.