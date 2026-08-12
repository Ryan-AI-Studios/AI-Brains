# T248 — Retention plan human summary

- **Track ID:** T248-RetentionPlanHuman  
- **Status:** 📋 **Placeholder** (plan-only until **go**)  
- **Category:** UX  
- **Source:** Audit — `retention plan` **E7/Q7** JSON-only, empty classes thin for operators  

## 1. Objective

TTY/human default: short summary (horizons + totals + warnings); JSON for machines (`--format json` or non-TTY).

## 2. Draft decisions

| ID | Decision |
|----|----------|
| **F1** | Human summary lines for totals + honesty warnings. |
| **F2** | JSON shape frozen when `--format json`. |
| **F3** | Zero candidates still explains “nothing to dispose” + horizons. |

## 3. Acceptance (draft)

| AC | Criterion |
|----|-----------|
| AC1 | Human + json hermetic |
| AC2 | Empty candidates non-blank human |
| AC3 | CAPABILITIES |

---

**Placeholder only.**
