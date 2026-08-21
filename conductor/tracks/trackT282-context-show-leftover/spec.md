# T282 — `context --show` must name leftover shell vs effective `.env`

- **Track ID:** T282-ContextShowLeftover
- **Status:** **Placeholder** (Pending until `/plan-track 282`)
- **Category:** UX / HONESTY
- **Owner:** Grok
- **Source:** Live CLI audit 2026-08-21 — `context --show` **7/7**; `whoami` already shows `shell_project_id=7d97a456` vs effective `3581317d`
- **Depends on:** T240 ✅ whoami; T242 ✅ session quiet; T257 ✅ warning/JSON
- **F0:** Plan-only until **go**.

## Problem (live)

`whoami` is the identity SoT. `context --show` prints effective IDs and model URLs and does **not** mention the leftover shell PROJECT_ID that whoami reports as overridden. Agents that only run `context --show` miss the leftover.

## How to ≥8 (ideally 10)

`context --show` includes a leftover/override line (shell vs `.env` / path owner) without printing `AI_BRAINS_KEY`. No silent write (T240 F2).

## Manual DoD (on go)

From this repo (live leftover shell still true on 2026-08-21):

```powershell
ai-brains context --show
ai-brains project whoami
```

Pass: `--show` mentions the leftover shell id **or** that `.env` overrides shell, matching whoami `shell_project_id` vs `effective_project_id`. stdout contains **no** `x'` key material. Hermetic: TempEnv leftover + fixture `.env`.

## Isolation

No `--write-env` (T240 F2). No T276 rebind. No key in help (T256).
