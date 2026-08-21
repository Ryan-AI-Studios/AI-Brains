# T281 — Nightly Completion timeout vs daemon Open must be one-line obvious

- **Track ID:** T281-NightlyProbeVsTcp
- **Status:** **Placeholder** (Pending until `/plan-track 281`)
- **Category:** OPS / UX
- **Owner:** Grok
- **Source:** Live CLI audit 2026-08-21 — friction: `nightly --status` Completion `probe=timeout (750ms)` while `daemon status` LLM **Open**. Nightly itself scored **8/8** after T269.
- **Depends on:** T269 ✅ heading + `timeout (750ms)`; T255 F18 freeze (do **not** raise 750 ms)
- **F0:** Plan-only until **go**.

## Problem (live)

T269 labeled the budget. Operators still see two truths. `--quick` skips probes (`skipped`). Full status HTTP `/health` can queue (llama.cpp) while daemon TCP connect is Open.

## How to ≥8 (ideally 10)

Human Completion line (or the next line) states **HTTP `/health` 750ms ≠ daemon TCP**. Do **not** raise 750 ms (T255/T269). JSON tokens stay `ok|down|timeout|error|skipped`. `--quick` stays skipped.

## Manual DoD (on go)

```powershell
ai-brains nightly --status
ai-brains daemon status
```

Pass: when Completion is `timeout (750ms)` and daemon LLM is Open, the nightly human block mentions HTTP `/health` or TCP contrast (not only OPERATIONS.md). JSON `probe` is still the raw token `timeout`. EXIT=0.

## Isolation

No schtasks mutate. No clap 5. No doctor 16th. No raise 750 ms.
