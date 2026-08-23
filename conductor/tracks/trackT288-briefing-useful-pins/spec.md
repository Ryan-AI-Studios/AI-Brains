# T288 — Granted-empty briefing must show vault pins exist (not H2)

- **Track ID:** T288-BriefingUsefulPins
- **Status:** **Placeholder** (Pending until `/plan-track 288`)
- **Category:** UX / HONESTY
- **Owner:** Grok
- **Source:** Audit 2026-08-22 — `briefing project` **7/7**; friction “Approved empty vs 3647 pins”
- **Depends on:** T263 ✅ H1 empty_authority + recall next (**H2 declined — do not reopen**); T275 ✅ grant-wall
- **F0:** Plan-only until **go**.

## Problem (live)

Grants are 3 of 3. Briefing is `## Decisions _None_` / `## Conclusions _None_` + `empty_authority` + `next: recall`. Honest, but agents still treat the vault as empty. Dual model is the friction.

## How to ≥8

Keep Approved arrays empty (no pin→Approved). **Populate** a vault-pins stanza: `Pinned: N` + up to **3** leading-line `DECISION:`/`CONSTRAINT:` first lines labeled **vault pins (not Approved)** + existing recall next. Denied path stays T275 grant-wall (no `_None_`). JSON: additive optional keys only if T180 allows; else human-only stanza.

## Manual DoD (on go)

```powershell
ai-brains briefing project --format human
```

Pass: stdout contains `Pinned:` (or equivalent) with a **nonzero** count for this project; contains `not Approved` (or `vault pins`); still has `next:` naming `recall`; **does not** put pin text under `## Decisions (current authority)` as if Approved. Hermetic: 0 pins → count 0, no fabricated decisions. JSON `denied: false`. Exit **0**.

## Isolation

**No T263 H2.** No live `policy` extra grants. Capture independence: read projection COUNT + recall helper, no new events.
