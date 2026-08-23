# T287 — `memory list` first page must include pins, not only just-now ingest

- **Track ID:** T287-MemoryListAuthority
- **Status:** **Placeholder** (Pending until `/plan-track 287`)
- **Category:** UX
- **Owner:** Grok
- **Source:** Audit 2026-08-22 — `memory list --limit 5` **8/6** (onboarding / review-track “just now”)
- **Depends on:** T216 ✅ recency ORDER (lift, do not silently drop recency); T274 F13 declined mix — **reopen as this track**
- **F0:** Plan-only until **go**.

## Problem (live)

Default `--limit 5` is the last five harness ingest rows (`# AI-Brains Session Onboarding Complete`, T276 review nits). Operators using `memory list` as inventory never see `DECISION:` pins. `--summary` counts (3648) are honest; the table is not useful.

## How to ≥8

Default human first page includes **≥1** leading-line authority pin when the project has any (prefer-fill, then recency fill) **or** a documented `--authority` default-on for human. JSON default order: full plan decides freeze vs human-only. `--summary` unchanged.

## Manual DoD (on go)

```powershell
ai-brains memory list --limit 5
ai-brains memory list --summary
```

Pass: among the 5 human rows, **≥1** preview starts with `DECISION:` / `CONSTRAINT:` / `INVARIANT:` when such pins exist (hermetic: one pin + four chrome ingest → pin present). `--summary` Pinned count still matches. JSON: if order frozen, document; if human-only permute, JSON `[0]` stays recency. Exit **0**.

## Isolation

Do not change `forget --match`. Do not steal T285 `recall_full` except shared classifier. T216 `--status` / limit 50 stay.
