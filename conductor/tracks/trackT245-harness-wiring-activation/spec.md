# T245 — Harness wiring activation

- **Track ID:** T245-HarnessWiringActivation  
- **Status:** 📋 **Placeholder** (plan-only until **go**)  
- **Category:** OPS / FEATURE  
- **Source:** Audit P1 — harness status excellent (**9/9**) but **wiring=missing** for all ready harnesses  
- **Depends on:** T235 detect/install UX; T236–T238 backends ready for grok/agy/opencode  
- **Absorbs:** Operator activation path; preflight install prompts; doctor harness_wiring remediation  
- **Not absorbed:** Claude/Codex install_ready (→ T253)

## 1. Objective

Ready harnesses (grok, agy, opencode) go from **install_ready** → **wired** on this machine with dry-run → confirm path; capture path becomes live.

## 2. Draft decisions

| ID | Decision |
|----|----------|
| **F1** | Document recommended order: grok → agy → opencode. |
| **F2** | `harness install --harness X --yes` after dry-run; preflight offer remains opt-in. |
| **F3** | Post-install: `harness status` shows wiring=present; doctor harness_wiring improves. |
| **F4** | No repo-local pollution (C7). |
| **F5** | Soft: batch `harness install --all-ready --dry-run`. |

## 3. Acceptance (draft)

| AC | Criterion |
|----|-----------|
| AC1 | Dry-run plans match T235/T237 honesty |
| AC2 | Live: at least one harness wiring=present after install |
| AC3 | OPERATIONS + CAPABILITIES |

---

**Placeholder only. Install writes user-global hooks — only on go.**
