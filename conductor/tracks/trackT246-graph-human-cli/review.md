# T246 Review Log — Graph human CLI presentation

**Track:** T246-GraphHumanCli  
**Category:** FEATURE / UX  
**Review requirement:** Cross-model (JSON key freeze + feature-off + recall API)

---

## Findings

| ID | Severity | Status | Summary | Notes |
|----|----------|--------|---------|-------|
| T246-R1 | P2 process | `out_of_scope` | Red→Green two-commit provenance | Series convention: one product commit (T240–T247). Tests written first in worktree; coordinator integrated. Not a functional gap. |
| T246-R2 | P2 | `verified_fixed` | Hermetic gaps: present-empty, wrong-kind, hierarchy CLI, JSON limit/sort, update `--format auto` | CX2 verified; plus session-missing pretty `413d984` |

### Status legend

- `open` — not yet fixed
- `fixed_pending_verification` — implementer claims fix; needs re-verify
- `verified_fixed` — reviewer confirmed
- `deferred` — medium/low only; justification + ISSUES.md

---

## Review passes

| Pass | Reviewer | Date | Result |
|------|----------|------|--------|
| Internal R1 completeness | explore subagent | 2026-08-13 | **CLEAN** — see `review.internal-completeness.md` |
| Internal R1 correctness | explore subagent | 2026-08-13 | **CLEAN** — see `review.internal-correctness.md` |
| Orchestrator live AC13 | graph-on `target\debug\ai-brains.exe` | 2026-08-13 | **PASS** — see Manual evidence |
| Cross-model CX1 | Codex gpt-5.6-luna high | 2026-08-13 | **FAIL** — P2 R1 process TDD commits; P2 R2 hermetic gaps. No P0/P1. |
| Internal R2 | explore | 2026-08-13 | R2 mostly covered; added session-missing pretty hermetic |
| Cross-model CX2 (final) | Codex gpt-5.6-luna high | 2026-08-13 | **PASS** — 0 P0–P3. R1 out_of_scope series convention; R2 verified. |

---

## Manual evidence (no live rebuild)

```
cmd: target\debug\ai-brains.exe graph neighbors 5a0e0a71-1ee7-445b-84a9-aa06fe499c2e --format pretty
Neighbors of 5a0e0a71-… (2)
DIR LABEL            ID                                   KIND           PREVIEW
in  RECALLS          28a3e316-… session
in  RECALLS          3b4e95b8-… session
exit: 0

cmd: piped / --format json (same id)
{"memory_id":"5a0e0a71-…","neighbors":[{"external_id":"28a3e316-…","label":"RECALLS","direction":"incoming"},…]}
keys: memory_id, neighbors, external_id, label, direction

cmd: target\debug\ai-brains.exe graph session 3b4e95b8-a011-48a8-b5ea-72e36c6a2458 --format pretty
Memories in session 3b4e95b8-… (5)
5a0e0a71-… DECISION: T243 …
(+ 4 more rows with previews)
exit: 0

cmd: graph neighbors 00000000-0000-0000-0000-000000000000 --format pretty
No graph node for 00000000-0000-0000-0000-000000000000.
next: ai-brains graph update
exit: 0

cmd: graph update
pretty JSON: nodes=1331 edges=119 density=warn status=sparse remediation=rebuild

cmd: graph update --format human
status: sparse
density: warn
…
remediation: ai-brains graph rebuild
```

F13 stop-before honored: did not run live `graph rebuild`.

---

## Targeted gates (orchestrator)

```
cargo clippy -p ai-brains-cli -p ai-brains-graph --all-targets --features graph -- -D warnings
  PASS

cargo nextest run -p ai-brains-graph
  12/12 PASS (AC16 diamond + node_kind)

cargo nextest run -p ai-brains-cli --features graph -- graph
  53/53 PASS (units AC1–AC6, hermetics AC7–AC10, T74 smoke)

cargo nextest run -p ai-brains-cli --test smoke graph__default_build
  PASS

cargo nextest run -p ai-brains-cli --test exit_contract graph
  2/2 PASS (AC11 feature-off --format pretty)
```

---

## Notes

- `get_neighbors` / `NeighborHit` / `get_synthesized_hierarchy` unchanged.
- Pretty hierarchy uses `get_synthesized_hierarchy_with_depth` (`MIN(depth)` GROUP BY).
- JSON `--limit` gated on `is_some()`; pretty always clamps 50/200.
- Soft residuals F17–F19 stay deferred (tree/mermaid, projector completeness, T213 F31).
