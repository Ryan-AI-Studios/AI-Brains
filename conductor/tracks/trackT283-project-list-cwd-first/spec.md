# T283 — `project list` must not lead with leftover 18k pins

- **Track ID:** T283-ProjectListCwdFirst
- **Status:** **Placeholder** (Pending until `/plan-track 283`)
- **Category:** UX
- **Owner:** Grok
- **Source:** Live CLI audit 2026-08-21 — `project list` **7/6**; first row leftover `7d97a456` / `C:\dev\crawlx` 18036; cwd `*C:\dev\ai-brains` is third
- **Depends on:** T212 ✅ labels; T230 ✅ never-blank; T267 ✅ footer (do not reopen leftover-as-AI-Brains)
- **F0:** Plan-only until **go**.

## Problem (live)

Sort is memories-desc, so leftover wins the table. Active `*` is visible but not first. Footer set-alias example is an unaliased UUID (T267), not the leftover split.

## How to ≥8 (ideally 10)

Human list: cwd path-owner (or `*` active) is the **first data row**, then others. JSON keys frozen; add a sort only if the full plan proves it is backward-compatible (or `--format human` only). Do **not** auto-alias leftover as `AI-Brains` (T267).

## Manual DoD (on go)

```powershell
ai-brains project list
```

Pass: from cwd AI-Brains, the first table data row is `3581317d` / `C:\dev\ai-brains` (starred), not leftover `7d97a456`. Footer does not suggest `set-alias 7d97a456 AI-Brains`. Hermetic: two projects, cwd owner smaller count, still first on human.

## Isolation

No live rebind (T276). No T240 F2. JSON machine order: full plan decides freeze vs human-only sort.
