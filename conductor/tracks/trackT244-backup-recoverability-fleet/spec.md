# T244 — Backup recoverability fleet

- **Track ID:** T244-BackupRecoverabilityFleet
- **Status:** 📋 **Placeholder** (plan-only until **go**)
- **Category:** OPS / FEATURE
- **Source:** Audit — backup verify **E7**, list **Q7**, 0/21 OK legacy plain; doctor `backup_recent` warn; P1 usable backup
- **Depends on:** T225 quiet verify + usable nudge (shipped)

## 1. Objective

Recoverability story is **green path**: at least one **usable SQLCipher** recent backup; legacy plain fleet labeled/retired without false hope.

## 2. Draft decisions

| ID | Decision |
|----|----------|
| **F1** | Operator create path: `backup create` after encrypt vault (docs + optional doctor one-shot hint). |
| **F2** | List: group/label legacy plain vs usable; don’t bury usable. |
| **F3** | Verify exit 1 when zero usable remains (keep T225 quiet summary). |
| **F4** | Soft: archive/quarantine legacy plain out of default list. |
| **F5** | Live dogfood: create backup → verify ≥1 OK → doctor backup_recent ok. |

## 3. Acceptance (draft)

| AC | Criterion |
|----|-----------|
| AC1 | Hermetic usable backup path |
| AC2 | Live dogfood: ≥1 OK after create |
| AC3 | Docs OPERATIONS recoverability |
| AC4 | Full gate |

---

**Placeholder only. Create mutates disk — only on go with user intent.**
