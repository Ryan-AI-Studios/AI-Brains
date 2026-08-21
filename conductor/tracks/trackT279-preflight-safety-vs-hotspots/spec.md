# T279 — Preflight Safety must not be a captured review-track prompt

- **Track ID:** T279-PreflightSafetyVsHotspots
- **Status:** **Placeholder** (Pending until `/plan-track 279`)
- **Category:** UX / HONESTY
- **Owner:** Grok
- **Source:** Live CLI audit 2026-08-21 — friction: `preflight --pretty` “Repository Bearings & Safety” was a T272 review-track Objective dump; `safety sync --dry-run` listed `project.rs` etc.
- **Depends on:** T264 ✅ global isolation; T272 ✅ skip emitted ids; T219 ✅ pretty readability
- **F0:** Plan-only until **go**.

## Problem (live)

`safety sync --dry-run` works (5 Ledgerful hotspots). Preflight Safety is **session capture** labeled as bearings. Operators cannot tell hotspots from chat. T274 ranks pins; this track is **section identity**.

## How to ≥8 (ideally 10)

Pretty Safety shows hotspot paths (or an honest “no hotspots” + `safety sync --dry-run`) and does **not** use a captured `## Objective` as the Safety body. Session dumps stay under Session.

## Manual DoD (on go)

```powershell
ai-brains safety sync --dry-run
ai-brains preflight --pretty
```

Pass: Safety section does not contain `## Objective` / `review-track` as the first Safety body; either a hotspot path from dry-run appears, or Safety says there are no in-context hotspots. Hermetic: pin a CONSTRAINT with `HOTSPOT:` or inject a fixture row if that is the live SOOT.

## Isolation

Do not retune T264 caps / T272 skip set. Ranking of Index → **T274**.
