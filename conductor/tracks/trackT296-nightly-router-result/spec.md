# T296 — Nightly Router last-result must not look like Nightly success/failure

- **Track ID:** T296-NightlyRouterResult
- **Status:** **Placeholder** (Pending until `/plan-track 296`)
- **Category:** UX / HONESTY
- **Owner:** Grok
- **Source:** Audit 2026-08-22 — `nightly --status --quick` **8/8** split exists; friction: `Router: Ready last result: 267014` + `task terminated (SCHED_S_TASK_TERMINATED)`
- **Depends on:** T269 ✅ `Nightly:` heading; T255 ✅ read-only Router; T281 ✅ timeout line (do **not** raise 750)
- **F0:** Plan-only until **go**.

## Problem (live)

Nightly Last Result **0** is success. Router line still dumps schtasks `267014` / `SCHED_S_TASK_TERMINATED` next to `Ready`. Operators mix the two.

## How to ≥8

Human Router last-result: map known codes (`0`/`267014`/`SCHED_S_TASK_TERMINATED`) to one honest phrase (`terminated` / `running` / `ready`) **or** omit the numeric when `Ready`. JSON frozen. `--quick` still `probe=skipped`. 750 ms not raised.

## Manual DoD (on go)

```powershell
ai-brains nightly --status --quick
```

Pass: human stdout still has `Nightly:` separate from `Router:`. Router line does **not** present `267014` as if Nightly failed (or labels it `terminated`/`Ready` without the raw code). Nightly `Last task result: 0` still present. JSON `--format json` keys unchanged. Hermetic/unit: raw `267014` → frozen label. Exit **0**.

## Isolation

No schtasks mutate. No doctor 16th. No persist probe.
