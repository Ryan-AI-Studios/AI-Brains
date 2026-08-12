# T242 — Env override warning session quiet

- **Track ID:** T242-EnvOverrideSessionQuiet
- **Status:** 📋 **Placeholder** (plan-only until **go**)
- **Category:** UX / POLISH
- **Source:** Audit P1 — warning on nearly every command; T223 residual F18 (clap quiet / session-once)
- **Depends on:** T223 collapse multi-key line (shipped)

## 1. Objective

Reduce stderr noise so daily output is scannable: **at most one** env-override notice per process (or session flag / `AI_BRAINS_QUIET_ENV_WARN` default for interactive).

## 2. Draft decisions

| ID | Decision |
|----|----------|
| **F1** | Once-per-process memoization of the Warning line (or opt-in once-per-TTY session). |
| **F2** | Keep full detail under `--verbose` / debug log. |
| **F3** | Do not hide mismatches that change scope (identity still T240). |
| **F4** | Hermetic: N commands → ≤1 warning line when keys stable. |

## 3. Acceptance (draft)

| AC | Criterion |
|----|-----------|
| AC1 | Multi-subcommand script: single warning max for same override set |
| AC2 | T223 collapse behavior preserved |
| AC3 | Docs CAPABILITIES one-liner |

---

**Placeholder only.**
