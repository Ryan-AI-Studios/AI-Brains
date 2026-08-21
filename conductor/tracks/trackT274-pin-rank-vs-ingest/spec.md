# T274 — Pins and DECISION rows must beat harness session dumps

- **Track ID:** T274-PinRankVsIngest
- **Status:** **Placeholder** (Pending until `/plan-track 274`)
- **Category:** FEATURE / UX
- **Owner:** Grok
- **Source:** Live CLI audit 2026-08-21 — `recall`/`search` **10/4**, `--semantic` **8/4**, `preflight --pretty` **9/4**, `--summary` **9/5**, `memory list` **8/7**, `sync query` vault half **9/7**
- **Depends on:** T260 ✅ (symbol stubs); T211 ✅ rerank; T218 ✅ dual floor
- **F0:** Plan-only until **go**. This file is a stub.

## Problem (live)

Harness capture (review-track `## Objective`, fold-in prompts, T248 plan reviews) occupies the same FTS/index as `pin`. `recall "what did we decide about retention"` and even the T270 DECISION sentence returned those dumps in top-3; `memory list --limit 5` is all “just now” ingest; preflight summary **0** in-context decisions/hotspots/constraints on ~3296 project pins; pretty Safety is a captured prompt, not Ledgerful hotspots.

T260 demoted **symbol stubs**. This is a **new** failure mode: session ingest volume.

## How to ≥8 (ideally 10)

`pin` / leading `DECISION:` / `CONSTRAINT:` / `INVARIANT:` outrank session transcripts on `recall` / `search` / `--semantic` / preflight Index. Recency of ingest must not bury a same-day pin. Duplicate near-identical reviews collapse. `sync query` vault half follows the same rank (ledger pane already good — **do not steal T271**).

## Manual DoD (on go)

Hermetic vault: `pin` a unique needle `DECISION: T274-rank-needle-<uuid>`. Optionally ingest a review-track-shaped assistant dump containing “retention” but not the needle. Then:

```powershell
ai-brains recall "T274-rank-needle-<uuid>" --limit 5 --format pretty --no-bridge
ai-brains memory list --limit 5
ai-brains preflight --summary
```

Pass: recall hit #1 is the pin (not `## Objective`); memory list includes the pin in the first 5 **or** pin `updated` ≥ ingest; preflight `--summary` in-context decisions ≥ 1 **or** Index lists the pin. EXIT=0.

## Isolation

No live leftover rebind (T276). No policy bootstrap (T275). No pin→Approved (T263 H2 declined). No clap 5. Do not grow `project.rs`.

## last-PR Cursor

#188 Bugbot Work-table / apply samples → **T284**. Not this track.
