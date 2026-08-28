# T323 — Conclusion in-force resolver

- **Track ID:** T323-ConclusionInForce
- **Status:** **Planned** (Pending until **go**) — **placeholder**. Full F-list on `/plan-track T323`.
- **Category:** FEATURE
- **Owner:** Grok
- **Source:** `conductor/deferred.md` T311 residual **R5** (conclusion in-force soft non-goal). Audit “conclusion/decision propose-only” is **partially stale** after T311 (decision has a read).
- **Depends on:** T311 pattern (CP resolver, not retrieval); T150-style conclusion supersession **if it exists in live src** — Phase 0 must verify; do not invent a chain
- **Blocks / feeds:** Symmetric “what conclusion is in force for term X?”
- **Absorbs:** T311 R5
- **Not absorbed (DoD):** H2; daemon ListInForce; decision `--as-of` (T322); FTS list
- **Research date:** 2026-08-27. T311 isolation: conclusion in-force was a non-goal because conclusion lifecycle may not have `superseded_by`. **If live src has no chain, this track stays honesty + decline, not a fake walker.** Snapshot — re-verify at execute.
- **Ledger:** series DOCS TX `a6d3c404-1d64-4cba-a743-d75ac16c74cd`. Implement **FEATURE** TX on go.
- **Isolation:** Do **not** implement until go. Do **not** copy-paste decision code if the projection shape differs. Do **not** grow `governed_common.rs`. Do **not** print or commit `AI_BRAINS_KEY`.

---

## 1. Objective

1. **If conclusions have a successor chain,** `ai-brains conclusion in-force <TERM>` matches T311 semantics (`in_force` + `chain`, `ReadConclusions`, empty term exit 2).
2. **If they do not,** the full plan **declines** the walker and documents the honest next-step (propose / query) — do not ship a lie.
3. **No pin→Confirmed.** Dual-model stands.
4. **North star.** Capture independence: projection read.

---

## 2. Live baseline (mint 2026-08-27)

| Signal | Observation |
|--------|-------------|
| T311 | Decision walker shipped; conclusion explicitly non-goal |
| Audit | conclusion/decision scored 2 / — as propose-only |

---

## 3. Frozen until full plan

- **F0** plan-only until go.
- Phase 0 gate: live `conclusion_projection` columns.

---

## 6. Non-goals

Inventing `superseded_by` for conclusions without a lifecycle track. H2.

---

## 9. Deferred / last-PR

| Item | Disposition |
|------|-------------|
| T311 R5 | **Absorb** (or decline in full plan if no chain) |
| last-PR `#229` | **N/A empty** |

---

## 12. Touch map (sketch)

CP new module **or** shared walker parameterized by entity. CLI `conclusion.rs`.
