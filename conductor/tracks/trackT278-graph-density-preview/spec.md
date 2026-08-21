# T278 — Graph density and neighbor previews must be usable

- **Track ID:** T278-GraphDensityPreview
- **Status:** **Placeholder** (Pending until `/plan-track 278`)
- **Category:** FEATURE / UX
- **Owner:** Grok
- **Source:** Live CLI audit 2026-08-21 — `graph update` **7/7** sparse E/N ~0.11; `graph neighbors` **6/6** RECALLS with blank preview
- **Depends on:** T213 ✅ density doctor; T232 ✅ remediator; T246 ✅ TTY pretty; T262 ✅ live projection
- **F0:** Plan-only until **go**.

## Problem (live)

Doctor/update correctly say **sparse** (nodes ~22.7k, edges ~2.5k, pinned ~38.6k). Neighbors of a just-ingested memory show session `RECALLS` but **empty PREVIEW**. Daily graph is still not a tool. T262 live projection works for new pins; density floor 0.5 is unmet. Do **not** require a live `graph rebuild` of 38k pins as the only DoD (mutating, long).

## How to ≥8 (ideally 10)

`--format human` neighbors show a non-blank preview for the neighbor id. Density remediator stays honest (`graph rebuild` when graph-on). Hermetic pin → neighbors row has preview text. Optional: raise E/N on a **hermetic** vault after rebuild, not necessarily the live 38k vault.

## Manual DoD (on go)

```powershell
# hermetic: init, pin unique line, graph update --format human, graph neighbors <id> --format human
```

Pass: neighbors table PREVIEW is not blank for at least one row; update `status` is `live`|`sparse`|`empty` (not a false live); remediator is `ai-brains graph rebuild` when sparse/graph-on. Live rebuild is Stop-Before unless owner confirms.

## Isolation

No Cargo `default` graph-on (T200/T222). No T232 threshold retune unless the full plan says so.
