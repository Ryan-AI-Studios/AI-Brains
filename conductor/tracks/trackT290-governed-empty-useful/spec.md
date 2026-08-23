# T290 — Granted-empty governed lists/progressive must be useful

- **Track ID:** T290-GovernedEmptyUseful
- **Status:** **Placeholder** (Pending until `/plan-track 290`)
- **Category:** UX
- **Owner:** Grok
- **Source:** Audit 2026-08-22 — evidence/source/review list **6/8**; `query progressive` **6/8** (`denied: false`, `items: []`, `next: recall`)
- **Depends on:** T263 ✅ `next_step`; T275 ✅ grants (live 3 of 3)
- **F0:** Plan-only until **go**.

## Problem (live)

Lists and progressive are authorized-empty. `next_step` names recall but does not show **that 3647 pins exist** or a copy-paste query. U=6: agents stop.

## How to ≥8

Keep arrays empty (no fake Approved). **Populate** human + JSON `next_step` (or additive human-only footer) with `Pinned: N` and a copy-paste `ai-brains recall "<last query or what did we decide>"`. Progressive empty uses the operator query in that recall example.

## Manual DoD (on go)

```powershell
ai-brains evidence list --format json
ai-brains source list --format json
ai-brains review list --format json
ai-brains query progressive "what did we decide about SQLCipher"
```

Pass: each JSON `denied: false` / `items` or `results` **[]**; `next_step` (or documented sibling) contains `recall` **and** a nonzero pin count or the progressive needle in the example. Hermetic granted-empty + ≥1 pin. Exit **0**. Human `--format human` if present shows the same.

## Isolation

No H2. No fabricate evidence rows. T180: prefer existing `next_step` string growth over new required keys.
