# T299 — Empty `forget --list-forgotten` must point at live pins

- **Track ID:** T299-ForgetListUseful
- **Status:** **Placeholder** (Pending until `/plan-track 299`)
- **Category:** UX
- **Owner:** Grok
- **Source:** Audit 2026-08-22 — `forget --list-forgotten` **6/8** `No forgotten memories.`; prior series declined honest empty — **reopened** U&lt;8
- **Depends on:** T216 ✅ bounded list
- **F0:** Plan-only until **go**.

## Problem (live)

Honest empty. U=6: no next, no pinned count. Operators don’t know inventory is `memory list`.

## How to ≥8

Empty forgotten: keep `No forgotten memories.` + `Pinned: N` + `next: ai-brains memory list`. Non-empty path unchanged. JSON: additive `next_step` if keys allow; else human-only.

## Manual DoD (on go)

```powershell
ai-brains forget --list-forgotten --limit 5
ai-brains memory list --summary
```

Pass: forgotten-empty stdout contains `No forgotten` **and** `Pinned:` matching `--summary` Pinned (same scope) **and** `memory list` in `next:`. Hermetic empty forgotten + ≥1 pin. Exit **0**.

## Isolation

No auto-forget. Limit 50 default stays.
