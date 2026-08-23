# T285 — Recall/search must surface pins, not review-track dumps

- **Track ID:** T285-RecallRankV2
- **Status:** **Placeholder** (Pending until `/plan-track 285`)
- **Category:** UX / QUALITY
- **Owner:** Grok
- **Source:** Live CLI audit 2026-08-22 PATH **0.1.2** — `recall`/`search` **10/4**, `--semantic` **8/4**, `sync query` vault **9/7**
- **Depends on:** T274 ✅ two-pass (live still Q=4); T260 ✅ symbol demote; T211 ✅ rerank
- **F0:** Plan-only until **go**.

## Problem (live)

T274 shipped leading-line classify + Index two-pass. Live `recall "capture independence event log"` / `search "DECISION:"` / `--semantic "SQLCipher page encryption"` still lead with `# AI-Brains Session Onboarding Complete`, T254 reviews, and `## Objective` harness dumps. Unique `DECISION:` vault pins are not in top-3. Daily “what did we decide?” is still broken.

## How to ≥8 (ideally 10)

Human `recall` / `search` / `--semantic` / `sync query` **vault** half: for a contentful query that matches a pinned leading `DECISION:`/`CONSTRAINT:`/`INVARIANT:` line, that pin is in **top-3** and hit #1 is **not** `## Objective`. JSON keys frozen. `forget --match` / `memory list` ORDER stay **T287**. Ledger pane stays T271 (already found T187).

## Manual DoD (on go)

```powershell
ai-brains pin "DECISION: T285 rank-v2 unique canary $(Get-Date -Format o)" --tag t285-canary
ai-brains recall "T285 rank-v2 unique canary" --limit 5 --format pretty --no-bridge
ai-brains search "T285 rank-v2 unique canary" --limit 5 --format pretty --no-bridge
```

Pass: both stdout hit **#1 or top-3** contain `T285 rank-v2 unique canary` and do **not** start with `## Objective`. Hermetic: fixture pin vs chrome dump; chrome is not rank 1. `--semantic` either surfaces the pin or honest `no semantic hits` **plus** lexical pin in the fallback list. `sync query` vault half same pin proof; ledger pane unchanged. Exit **0**.

## Isolation

No T263 H2. No T240 F2. No live `retention apply`. JSON Recall keys frozen. No clap 5.
