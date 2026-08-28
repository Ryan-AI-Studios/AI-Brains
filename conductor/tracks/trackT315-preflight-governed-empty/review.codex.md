# T315 Codex Re-review

**Verdict PASS**

**HEAD:** `9b1553dd1e56d90b6b97006c54eb6fc6aeb49d9a`  
**Scope:** P3-001/P3-002 closure and regression sweep of the strengthening commit.  
**Prior:** FAIL (P3-001 DTO comment; P3-002 weak legacy insert lock). Orchestrator wrote this file because Codex read-only sandbox could not overwrite it.

## Findings

- **P3-001 — `verified_fixed`**  
  The DTO comment now correctly documents T241 bootstrap precedence, T315 empty-decisions fallback, and omission when neither applies: `preflight.rs:59`.

- **P3-002 — `verified_fixed`**  
  The legacy insert test places `NOT_THE_INSERT_POINT` immediately after `Total Word Count: 42` and asserts `soot_idx < sentinel_idx`: `preflight.rs:1471`.  
  Counterfactual verified by Codex: removing the legacy matcher makes insertion use the later empty-line fallback, placing SOOT after the sentinel. Both the immediate-position assertion and sentinel-order assertion then fail.

- **New defects:** None. Commit `9b1553d` changes only this unit test and its explanatory comments; production behavior is unchanged.

## Verification

- `cargo fmt --check -p ai-brains-cli` — PASS (Codex)
- Targeted legacy insert unit — PASS (orchestrator)
- Full workspace gate — recorded by orchestrator in `review.md` at closeout
