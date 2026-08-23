# T289 — Personal briefing deny must not look like empty preferences

- **Track ID:** T289-PersonalBriefing
- **Status:** **Placeholder** (Pending until `/plan-track 289`)
- **Category:** UX
- **Owner:** Grok
- **Source:** Audit 2026-08-22 — `briefing personal` **4/7**; deny + `_None_` Preferences/Continuity
- **Depends on:** T263 F4 ✅ Personal deny names `recall`; T275 F35 personal grant-wall analog
- **F0:** Plan-only until **go**.

## Problem (live)

Personal deny prints `> **Denied:**` then `## Preferences _None_` / `## Continuity _None_`. Same “empty vault” training as pre-T275 project briefing. Optional surface, but U=4.

## How to ≥8

When denied: **omit** `_None_` section bodies (grant-wall / optional-continuity sentence + `next: recall`). Do **not** require Personal `policy bootstrap` as the primary next (T263). When granted-empty: one line that personal continuity is unused, not a missing vault.

## Manual DoD (on go)

```powershell
ai-brains briefing personal --format human
```

Pass: denied stdout **does not** contain `_None_` under Preferences or Continuity; contains `recall`; does **not** tell the operator to `policy bootstrap` Personal as required. Hermetic deny fixture. Exit **0** (soft deny). JSON `denied: true` unchanged.

## Isolation

No auto Personal grant. No T263 H2. Project briefing is **T288**.
