CX2 verdict: **PASS WITH P3-1**. Product changes are clean; all CX1 product findings are verified fixed.

### P0

None.

### P1

None.

- **CX1-P1-1 — `verified_fixed`:** post-rerank preferred partition removed. Merge feeds the unchanged reranker, followed directly by dedupe and truncate in [recall.rs](C:/dev/AI-Brains/crates/ai-brains-retrieval/src/recall.rs:558).
- **CX1-P1-2:** full gate remains a pending closeout step as expected; not re-filed as a product failure.

### P2

None.

- **CX1-P2-1 — `verified_fixed`:** unscoped lexical search runs only when `scoped.len() < depth` in [recall.rs](C:/dev/AI-Brains/crates/ai-brains-retrieval/src/recall.rs:320).
- **CX1-P2-2 — `verified_fixed`:** substring fallback selects and maps the COALESCE project ID in [lexical.rs](C:/dev/AI-Brains/crates/ai-brains-retrieval/src/lexical.rs:359), with focused coverage.
- **CX1-P2-3 — `verified_fixed`:** AC2/AC3 use 15 chrome rows; leftover authority has a separately named merge test in [recall_global_prefer.rs](C:/dev/AI-Brains/crates/ai-brains-retrieval/tests/recall_global_prefer.rs:66).

### P3

**P3-1 — Review log retains pre-CX1 evidence**

[review.md](C:/dev/AI-Brains/conductor/tracks/trackT276-leftover-7d97-rebind/review.md:69) still describes AC2 as using leftover DECISION rows, and [the closing note](C:/dev/AI-Brains/conductor/tracks/trackT276-leftover-7d97-rebind/review.md:115) says the removed post-rerank partition remains in `recall.rs`.

Required fix: update those statements to reflect the chrome AC2 fixture, separate authority merge test, and removal of the partition. Review-log-only; no product re-review needed.

Fresh sweep also confirmed:

- No `ranking.rs` production retune—only `project_id: None` in a test constructor.
- No leftover/project exclusion SQL or hardcoded leftover UUID.
- No `RecallResult`/DTO `project_id`.
- `cargo fmt --check` and `git diff --check` passed.
- Four prebuilt pure merge tests passed.
- Rebuilding DB-backed tests was blocked by the enforced read-only sandbox (`target\.cargo-lock` and temp-directory writes denied), not by test assertions. The supplied post-fix targeted and clippy gate results remain consistent with the inspected code.