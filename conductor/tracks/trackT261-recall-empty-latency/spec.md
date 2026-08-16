# T261 — Recall empty-query latency

- **Track ID:** T261-RecallEmptyLatency
- **Status:** **Pending** (placeholder; plan-only until go)
- **Category:** PERFORMANCE / UX
- **Owner:** —
- **Source:** Audit 2026-08-16 — friction: `recall '' --format pretty` **5730 ms**
- **Depends on:** T105 substring fallback; T111/T207 empty hint; T217 rescue ladder
- **Absorbs:** Empty string / all-stopword query taking seconds; miss path should be instant
- **Not absorbed:** Ranking quality (T260); FTS rescue correctness (T217 closed)

---

## 1. Objective

A query with **no contentful tokens** (`""`, whitespace, all stopwords) must short-circuit to the existing empty pretty hint **without** opening FTS rescue, substring LIKE, semantic, or graph expansion.

## 2. Problem (live 2026-08-16)

| Command | Time | Result |
|---------|------|--------|
| `recall zzzznonexistentqueryxyz --format pretty --no-bridge` | 670 ms | Good empty hint |
| `recall "" --format pretty` | **5730 ms** | Same empty hint |

The miss copy is already excellent (T207). The empty query burned ~5 s — likely T105 full-vault LIKE and/or rescue/graph on an empty MATCH.

## 3. Frozen intent (placeholder)

| ID | Intent |
|----|--------|
| **F0** | Plan-only until go. |
| **F1** | After token extract, 0 contentful tokens → empty response + hint. No SQL LIKE. No rescue. No embed. |
| **F2** | Clap still requires `<QUERY>`; `-` stdin empty is the same short-circuit (do not hang). |
| **F3** | Target: empty-query pretty **&lt; 500 ms** on this machine’s vault class (hermetic small vault: milliseconds). |
| **F4** | Real miss with contentful tokens may still run T217/T105. |

## 4. Verification sketch

- Hermetic: empty / whitespace / all-stopword → hint, no LIKE in traced SQL (or unit that rescue/substring not called).
- Existing miss-with-tokens tests stay green.
