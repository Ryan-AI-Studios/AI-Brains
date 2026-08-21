# Track review: T274-PinRankVsIngest

**Harness:** Antigravity (`agy`)  
**Track:** `conductor/tracks/trackT274-pin-rank-vs-ingest`  
**Date:** 2026-08-21  
**HEAD:** `9a99117`  

---

## Summary

Track T274 directly resolves the core product defect uncovered during the 2026-08-21 live CLI audit:
`ai-brains recall`, `ai-brains search`, and preflight Index windows return voluminous harness session ingest (such as `## Objective` review prompts and JSON session dumps) instead of explicit, high-value `DECISION:`, `CONSTRAINT:`, and `INVARIANT:` pins.

The root cause was twofold:
1. **Anywhere-in-body marker match:** `classify_pin_kind` searched the entire content body, granting `PinKind::Decision` (+2 boost) to session transcripts that merely discussed decisions.
2. **BM25 candidate saturation:** Because long review prompts repeat search tokens multiple times, FTS5 BM25 ranked them higher than concise pins. With `candidate_depth` limited to 15, all candidate slots were consumed by session chrome before `rerank_hits` was ever invoked.

T274 solves this with a multi-layered architectural approach:
- **Leading-line classification:** `classify_pin_kind` inspects only the first non-empty line (post `ASSISTANT: ` strip), classifying buried mentions as `Other`.
- **Session chrome penalty:** Detects known session chrome patterns (`## Objective`, `# Track Plan Review`, `# AI-Brains Onboarding`, etc.) and applies a `-16.0` penalty in `rerank_hits`.
- **Two-pass lexical candidate selection:** Queries authority pins (`DECISION:*`, `CONSTRAINT:*`, `INVARIANT:*`) first, then fills remaining candidate slots with general matches.
- **In-memory prefer-fill in semantic search & preflight Index:** Ensures authority pins are admitted to candidate sets and Index summaries before recency fill.
- **First-line chrome deduplication:** Collapses identical review prompt headers post-reranking.

The plan is well-bounded, respects all invariant freezes, and directly restores the north star of vault-first recall.

---

## Findings (B/M/m/O)

### Blockers (B)
*None.*

### Major (M)
*None.*

### Minor (m)
- **m1: Parameterized `NOT IN` binding in `match_query` Pass 2 (F7 / AC17):** In `lexical.rs`, when constructing the SQL for Pass 2, ensure the exclusion of Pass 1 IDs uses dynamically generated `?` placeholders (e.g. `format!(" AND mp.memory_id NOT IN ({})", (0..pass1.len()).map(|_| "?").collect::<Vec<_>>().join(", "))`) and appends the IDs into `params_vec`, rather than formatting strings directly into the query.
- **m2: Selective deduplication in `session_chrome::dedupe_session_chrome` (F10 / AC5):** Ensure `dedupe_session_chrome` only collapses hits that are confirmed as session chrome by the detector, while preserving distinct non-chrome hits even if they share common opening punctuation or formatting.

### Opportunities (O)
- **O1: Centralized `AUTHORITY_GLOB_SQL` fragment:** Define `pub(crate) const AUTHORITY_GLOB_SQL: &str = "(mp.content GLOB 'DECISION:*' OR mp.content GLOB 'CONSTRAINT:*' OR mp.content GLOB 'INVARIANT:*' OR mp.content GLOB 'ASSISTANT: DECISION:*' OR mp.content GLOB 'ASSISTANT: CONSTRAINT:*' OR mp.content GLOB 'ASSISTANT: INVARIANT:*')";` in `session_chrome.rs` to maintain single-source-of-truth SQL generation.
- **O2: Early exit in `match_query`:** If Pass 1 returns `pass1.len() >= limit`, `match_query` can immediately return `pass1` without executing the second SQL query.

---

## What Looks Solid

1. **Two-Pass Candidate Generation:** Implementing two-pass querying in `match_query` fixes the exact candidate starvation issue where BM25 saturation prevented pins from reaching `rerank_hits`.
2. **Leading-Line Rule Precision:** Constraining `classify_pin_kind` to the first contentful line prevents long transcripts with buried keywords from stealing Decision boosts.
3. **Capture & Ingestion Independence:** Ingested sessions remain 100% intact, queryable, and uncorrupted; ranking alone ensures pins take precedence when answering queries.
4. **Hotspot Restraint:** Zero edits to top hotspots (`project.rs`, CLI `preflight.rs`, `sync.rs`). New detector logic is cleanly isolated in `crates/ai-brains-retrieval/src/session_chrome.rs`.

---

## Deferred Fold-In Table

| Deferred Item | Spec/Plan Disposition | Assessment |
|---------------|------------------------|------------|
| Recall/search/semantic/preflight dumps over pins | Absorbed into DoD (F1–F12 / AC1–AC7 / AC14–AC15) | Core problem solved via two-pass + chrome penalty |
| `sync query` vault dumps | Absorbed (F14 / AC15) | Follows `recall_full` |
| `memory list` recency inventory | Partial (F13) | Retains honest recency sort (T216) |
| Preflight Safety = `## Objective` | Declined (F23 → T279) | Sourcing from `safety sync` belongs to T279 |
| Policy grants first-run | Declined (F24 → T275) | Governed briefing belongs to T275 |
| Leftover `7d97a456` rebind | Declined (F24 → T276) | Leftover path rebind belongs to T276 |
| PR #188 Bugbot Mediums | Declined (F26 → T284) | Already minted as placeholder Track T284 |
| Hard-excluding session transcripts | Declined (F1) | Preserves conversational recall capability |
| Retuning `candidate_depth` / floors | Declined (F17) | Preserves hybrid RRF constants |

---

## Last-PR Cursor Comments

- **Scanned PR:** [#188](https://github.com/Ryan-AI-Studios/AI-Brains/pull/188) (merged 2026-08-21, T270 `Retention live memory_legacy inventory overlay`).
- **Cursor Comments:** 2 Medium findings on PR #188 (`Apply audit samples prefer inventory` and `Work table hides dispose`).
- **Disposition:** Both findings are already captured and tracked in placeholder **Track T284** (`README-T274-T284-CLI-QUALITY.md` / `conductor.md`). No untracked leftovers.

---

## Research / Tools Notes

- **FTS5 & BM25:** SQLite FTS5 length normalization penalizes short documents against repetitive token-dense transcripts. The two-pass prefer-fill pattern aligns with Elastic function_score / Azure AI Search category filtering best practices.
- **Dependencies:** `clap` (4.6.1), `serde_json` (1.0.150), `rusqlite` (0.39.0), `chrono` (0.4.44).
- **Toolchain / Rust:** `1.95.0` (Edition 2024), workspace `0.1.1`.
- **`ledgerful` / `ai-brains`:**
  - `ai-brains preflight --summary`: Scope `3581317d`, 3,297 pinned memories, 3 active sessions.
  - `ledgerful ledger status --compact`: 0 pending, 0 unaudited drift.
  - `ledgerful search rerank_hits`: Located at `crates/ai-brains-retrieval/src/ranking.rs:248`.

---

## Verdict: Planned

The plan is approved as **Planned**. Implementation should proceed under TDD once the user issues `/implement-track`.
