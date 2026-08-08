# T219 — Preflight pretty readability

- **Status:** 📋 Placeholder (plan-only until go)
- **Source:** Audit — `--pretty -m 800` useful for agents but wall of CONSTRAINT/DECISION + ASSISTANT: noise
- **Scores:** usefulness 8 · **output quality 5**
- **Category:** UX
- **Depends on:** T214 summary; T032 condensation history

## Objective

Human-readable pretty preflight: structure, role strip, section budgets — without starving agent context.

## Draft decisions

- Strip USER/ASSISTANT/SYSTEM on display (share T224 helper)
- Section headers + blank lines; cap lines per section with “+N more via recall”
- Keep JSON/`--format json` full packet for agents
- Optional `--compact` if pretty remains dense

## Non-goals

Change marker selection policy; ledgerful-on-global (T214 residual).
