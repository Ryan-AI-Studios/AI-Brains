# T293 — `graph neighbors` of a pin must not be dump-session soup

- **Track ID:** T293-GraphNeighborsPins
- **Status:** **Placeholder** (Pending until `/plan-track 293`)
- **Category:** UX
- **Owner:** Grok
- **Source:** Audit 2026-08-22 — `graph neighbors` **7/8**; PREVIEW filled (T278) but neighbors are T254 review sessions
- **Depends on:** T246 ✅ pretty; T278 ✅ session PREVIEW; T262 ✅ pin = memory node
- **F0:** Plan-only until **go**.

## Problem (live)

Neighbors of a recent pin show `RECALLS` sessions whose PREVIEW is `# Review of Track 254` and `SYNTHESIZED_FROM` ```json`. Useful graph for **pins** is missing. Density sparse is **T300**.

## How to ≥8

Pretty neighbors: prefer pin/authority neighbors (leading `DECISION:` memory nodes) in the first page; session PREVIEW unchanged T278. JSON neighbor keys frozen `{memory_id, neighbors:[{external_id,label,direction}]}`. Human-only sort/filter.

## Manual DoD (on go)

```powershell
$pin = (ai-brains recall "T293" --limit 1 --format json | ConvertFrom-Json).hits[0].id  # plan will freeze a hermetic pin id
ai-brains graph neighbors <pin-id> --format human --limit 8
```

Pass: hermetic pin with a session dump neighbor **and** an authority-pin neighbor → first human data row after header is the **authority** neighbor (or pin), not `## Objective` / Track 254 review. JSON array order documented (freeze vs human-only). PREVIEW still `{n} memories · first line` for sessions. Exit **0**. `graph update` JSON keys unchanged.

## Isolation

No live `graph rebuild` (T300). No floor retune. No Cargo default-on.
