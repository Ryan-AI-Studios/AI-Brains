# T280 — Policy deny/show hints must match doctor (omit `--scope` when context is authoritative)

- **Track ID:** T280-PolicyHintOmitScope
- **Status:** **Placeholder** (Pending until `/plan-track 280`)
- **Category:** UX / HONESTY
- **Owner:** Grok
- **Source:** Live CLI audit 2026-08-21 — `policy show` **8/7**, `policy check` **7/7**; deny `details.hint` still `bootstrap --scope …` while doctor says omit `--scope`
- **Depends on:** T210 ✅ bootstrap; T226 ✅ soft-resolve; T241 ✅ doctor grants
- **F0:** Plan-only until **go**.

## Problem (live)

Doctor remediator: `policy bootstrap` (omit `--scope` when project context is authoritative). `policy show` `next_step` and POLICY_DENIED `details.hint` still use `bootstrap --scope …`. Agents copy the deny string and pass a redundant/wrong scope.

## How to ≥8 (ideally 10)

One SOOT hint: when `AI_BRAINS_PROJECT_ID` / whoami path owner is authoritative, omit `--scope` in `policy show` next_step, deny hint, and progressive `denial_hint`. Keep `--scope` in examples for no-context CI.

## Manual DoD (on go)

From this repo (project context set):

```powershell
ai-brains policy show
ai-brains policy check --capability ReadEvidence
ai-brains doctor --summary
```

Pass: `policy show` next_step and deny hint do **not** require `--scope …` as the only form; they match doctor’s omit-scope wording (or include an omit-scope alternative). `--no-project-context` deny may still name `--scope`. Hermetic both arms.

## Isolation

No live bootstrap (T275). No clap 5. Shared hint const only.
