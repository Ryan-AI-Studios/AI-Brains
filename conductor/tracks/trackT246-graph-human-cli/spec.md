# T246 — Graph human CLI presentation

- **Track ID:** T246-GraphHumanCli  
- **Status:** 📋 **Placeholder** (plan-only until **go**)  
- **Category:** UX  
- **Source:** Audit — `graph neighbors` **E7/Q6** raw JSON; hierarchy/session same class  
- **Depends on:** T222 graph-on; T232 density remediations  

## 1. Objective

`graph neighbors|hierarchy|session` usable by humans on TTY (table/pretty), JSON when piped or `--format json`.

## 2. Draft decisions

| ID | Decision |
|----|----------|
| **F1** | Pretty default on TTY; JSON non-TTY (match recall-ish SOOT). |
| **F2** | Columns: direction, label, id, optional preview. |
| **F3** | Empty neighbors honest next-step. |
| **F4** | Feature-off still exit 2 FEATURE_UNAVAILABLE. |

## 3. Acceptance (draft)

| AC | Criterion |
|----|-----------|
| AC1 | Hermetic pretty + json |
| AC2 | Live neighbors for a known memory_id scannable |
| AC3 | CAPABILITIES |

---

**Placeholder only.**
