# T286 — Preflight Index/summary must not look like an empty brain

- **Track ID:** T286-PreflightIndexPins
- **Status:** **Placeholder** (Pending until `/plan-track 286`)
- **Category:** UX
- **Owner:** Grok
- **Source:** Audit 2026-08-22 — `--pretty` **7/5** Index `1. ## Objective -- just now`; `--summary` **8/7** in-context decisions **0** / 3647 pins
- **Depends on:** T274 ✅ Index two-pass (live still Objective); T279 ✅ Safety (do **not** reopen); T250 ✅ density
- **F0:** Plan-only until **go**.

## Problem (live)

Safety is live hotspots (T279). Session + **Memory Index** still lead with review-track `## Objective`. Summary `In context decisions: 0` trains agents that this repo has no decisions.

## How to ≥8

Pretty Index first item is a leading-line pin (`DECISION:`/`CONSTRAINT:`/`INVARIANT:`) when the vault has one in-scope — **not** `## Objective`. Summary either (a) counts vault pin-authority separately from Approved markers, or (b) shows `Pinned: N` next to in-context 0 so 3647 pins are visible. JSON `sections[]` keys frozen (T265). Safety skip-set stays T279.

## Manual DoD (on go)

```powershell
ai-brains preflight --pretty
ai-brains preflight --summary
```

Pass: `--pretty` **Memory Index** line `1.` does **not** start with `## Objective` when ≥1 in-scope pin exists (hermetic proof). `--summary` stdout contains the pinned count **3647-class** (or `Pinned memories`) and does **not** imply “no decisions in vault.” Safety block still matches `safety sync --dry-run` paths. Hermetic: chrome dump in Index is not item 1. Exit **0**.

## Isolation

Do not steal T285 recall rank (Index renderer only). Do not change T279 Safety SQL. No T180 required-key growth.
