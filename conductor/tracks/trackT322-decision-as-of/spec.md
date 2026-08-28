# T322 — `decision in-force --as-of`

- **Track ID:** T322-DecisionAsOf
- **Status:** **Planned** (Pending until **go**) — **placeholder**. Full F-list on `/plan-track T322`.
- **Category:** FEATURE
- **Owner:** Grok
- **Source:** `conductor/deferred.md` T311 residual **R2** (`--as-of` soft non-goal). Not the 2026-08-27 CLI audit (T311 shipped after that audit’s “propose-only” note).
- **Depends on:** T311 ✅ `decision in-force`; T150 `superseded_by`; T160 propose
- **Blocks / feeds:** Time-travel “what was in force on date D?”
- **Absorbs:** T311 R2. T311 R4 `approved_at` column **only if** full plan proves `updated_at` is insufficient
- **Not absorbed (DoD):** T311 R1 daemon `ListInForce`; R5 conclusion (T323); R7 empty TERM (T324); H2; FTS `decision list`
- **Research date:** 2026-08-27. T311 uses `decision_valid_at` in `briefings/project.rs`. Bi-temporal “as of” usually needs a valid-time column; do not add `approved_at` in the placeholder. Snapshot — re-verify at execute.
- **Ledger:** series DOCS TX `a6d3c404-1d64-4cba-a743-d75ac16c74cd`. Implement **FEATURE** TX on go.
- **Isolation:** Do **not** implement until go. Do **not** grow `governed_common.rs`. Extend CP `in_force.rs`. Do **not** print or commit `AI_BRAINS_KEY`.

---

## 1. Objective

1. **Point-in-time ruling.** `ai-brains decision in-force <TERM> --as-of <RFC3339>` returns the node that was in force at that instant (or honest none).
2. **Default remains “now”** (T311 behavior).
3. **Stay a governed read.** `ReadDecisions`. No new events. No pin→Approved.
4. **North star.** Capture independence: projection query.

---

## 2. Live baseline (mint 2026-08-27)

| Signal | Observation |
|--------|-------------|
| T311 | `in_force` = current successor; no `--as-of` |
| R4 | JSON `updated_at` used as proxy — may be wrong for as-of |

---

## 3. Frozen until full plan

- **F0** plan-only until go.
- T311 F-list (scope, empty term, format parser) stays unless as-of forces a flag.

---

## 6. Non-goals

Daemon wire (R1). Conclusion in-force (T323). Schema tourism without an AC.

---

## 9. Deferred / last-PR

| Item | Disposition |
|------|-------------|
| T311 R2 | **Absorb** |
| T311 R4 | **Partial** — column only if plan requires |
| last-PR `#229` | **N/A empty** |

---

## 12. Touch map (sketch)

`ai-brains-control-plane` `in_force.rs` + CLI `decision.rs` clap. Tests hermetic valid-at.
