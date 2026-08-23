# T297 — `daemon status` must contrast Stopped vs llama.cpp Open

- **Track ID:** T297-DaemonVsLlm
- **Status:** **Placeholder** (Pending until `/plan-track 297`)
- **Category:** UX / HONESTY
- **Owner:** Grok
- **Source:** Audit 2026-08-22 — `daemon status` **9/9** labeled Stopped + Open; still **friction** two truths
- **Depends on:** T199 ✅ keyless status; T281 ✅ nightly timeout ≠ TCP (do **not** unify probes)
- **F0:** Plan-only until **go**.

## Problem (live)

`Status: Stopped` and `LLM backend … Open` on the same report. Operators think the daemon is serving the model. T281 labeled nightly; **status** has no contrast sentence.

## How to ≥8

When daemon **Stopped** and HTTP `:8081`/`:8083` **Open**, print one line: `llama.cpp HTTP Open ≠ daemon` (or frozen const) + keep `next: ai-brains daemon start`. When both Stopped/down, no extra line. Do not start the daemon. Do not raise 750. JSON if any: additive.

## Manual DoD (on go)

```powershell
ai-brains daemon status
```

Pass: this machine (daemon Stopped, :8081 Open) stdout contains **both** `Stopped` and a contrast that HTTP Open is **not** the daemon; still `next: ai-brains daemon start`. Hermetic: mock Stopped+Open → line present; Stopped+down → line absent. Exit **0**. Do **not** `daemon start`.

## Isolation

No service install. No 750 raise. Nightly is T281/T296.
