# T243 — Search surface unify

- **Track ID:** T243-SearchSurfaceUnify
- **Status:** 📋 **Placeholder** (plan-only until **go**)
- **Category:** UX / FEATURE
- **Source:** Audit P1 dual mental model; progressive **E3**; T231 soft residuals (search noun; recall text→pretty; invalid-env clap)
- **Depends on:** T231 decision table; T241 grants for progressive to be usable

## 1. Objective

One operator story: **what to run when** — `recall` (default vault search), `sync query` (vault+ledger), `query progressive` (governed) — without progressive as a surprise dead-end.

## 2. Draft decisions

| ID | Decision |
|----|----------|
| **F1** | CAPABILITIES/WORKFLOWS matrix: goal → command (extend T231 table). |
| **F2** | Soft: optional `search` alias → `recall` (or document only). |
| **F3** | Progressive deny: first-line human next-step includes bootstrap + “try `recall`”. |
| **F4** | Soft: `recall --format text` → pretty arm (T231 residual). |
| **F5** | Soft: invalid-env clap converge recall↔sync. |

## 3. Acceptance (draft)

| AC | Criterion |
|----|-----------|
| AC1 | Docs matrix complete + linked from help after_long_help |
| AC2 | Progressive deny mentions `recall` and bootstrap |
| AC3 | Optional alias/text arm if in DoD |

---

**Placeholder only.**
