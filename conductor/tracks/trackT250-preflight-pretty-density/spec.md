# T250 — Preflight pretty density (pass-2)

- **Track ID:** T250-PreflightPrettyDensity
- **Status:** 📋 **Placeholder** (plan-only until **go**)
- **Category:** UX
- **Source:** Audit — `preflight --pretty` **E7/Q7**; T219 soft residuals (`--compact`, retrieval strip)

## 1. Objective

Second-pass readability: tighter sections, optional `--compact`, less wall without losing safety pins.

## 2. Draft decisions

| ID | Decision |
|----|----------|
| **F1** | Soft `--compact` or lower default caps for non-global. |
| **F2** | Keep Scope always-on (T228). |
| **F3** | JSON compact envelope unchanged (T180/T220). |

## 3. Acceptance (draft)

| AC | Criterion |
|----|-----------|
| AC1 | Hermetic length/structure bounds |
| AC2 | Live pretty scannable under 800 words default path |

---

**Placeholder only.**
