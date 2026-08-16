# T267 — Next-action remediator honesty

- **Track ID:** T267-NextActionHonesty
- **Status:** **Pending** (placeholder; plan-only until go)
- **Category:** UX
- **Owner:** —
- **Source:** Audit 2026-08-16 — `harness status` **8/6**; `project list` **8/6**; opportunity “next-action is the remediator”; `whoami` tells you to run `whoami`
- **Depends on:** T235/T245 harness; T212 list footer; T240 whoami; T258 adopt-path (for the whoami remediator string)
- **Absorbs:** `next: ai-brains harness status` when wiring=ok; whoami remediations include “run whoami”; list footer `set-alias 7d97a456 … AI-Brains`
- **Not absorbed:** Implementing adopt-path (T258); splitting 7d97 (T259)

---

## 1. Objective

Every `next:` / footer / remediations[] entry must be a **command the operator has not just successfully run**, or `next: none` / omit.

## 2. Problem (live 2026-08-16)

`harness status` (5/5 wiring=ok): each row `next: ai-brains harness status`. JSON `next_action` same. Dead end.

`project whoami` remediations: “Run `ai-brains project whoami`” while already running it.

`project list` stderr example aliases the **largest unaliased** project as `AI-Brains` — that ID is `7d97a456` (leftover dump), not the path owner.

## 3. Frozen intent (placeholder)

| ID | Intent |
|----|--------|
| **F0** | Plan-only until go. |
| **F1** | Harness wiring=ok → no next, or `next: none`. Missing → `harness install --harness X --dry-run`. |
| **F2** | `whoami` remediations name `project adopt-path` / the T258 verb, or the exact `.env` assignment — never `whoami`. |
| **F3** | `project list` example alias uses a **path-owner without a label**, not “largest memory count”. Never suggest `AI-Brains` for `7d97a456`. |
| **F4** | Shared “don’t next yourself” helper if more than two call sites. |

## 4. Verification sketch

- Hermetic harness ok fixture: no `harness status` in next_action.
- List footer fixture: example id ≠ reserved leftover; label ≠ `AI-Brains` unless that id is the path owner.
