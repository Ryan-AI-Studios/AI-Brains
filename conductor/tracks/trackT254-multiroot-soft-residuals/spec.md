# T254 — Multi-root soft residuals (T233+)

- **Track ID:** T254-MultiRootSoftResiduals  
- **Status:** 📋 **Placeholder** (plan-only until **go**)  
- **Category:** FEATURE / OPS  
- **Source:** T233 soft residual — list-paths, unregister-path, from-scan, route method/path_pattern  
- **Depends on:** T233 completed  

## 1. Objective

Close optional T233 follow-ups without reopening multi-root core.

## 2. Draft decisions

| ID | Decision |
|----|----------|
| **F1** | `project list-paths` CLI (all aliases). |
| **F2** | `unregister-path` compensating event (or explicit soft decline). |
| **F3** | Soft `--from-scan` dry-run for `.ledgerful` roots. |
| **F4** | Soft: route enrichment via ledgerful endpoints (not SQL). |

## 3. Acceptance (draft)

| AC | Criterion |
|----|-----------|
| AC1 | Chosen subset shipped or explicitly declined in deferred |
| AC2 | No regression T233 AC3/AC12 |

---

**Placeholder only.**
