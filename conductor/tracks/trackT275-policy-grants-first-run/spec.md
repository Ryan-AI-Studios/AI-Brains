# T275 — Discovery grants must unlock briefing/progressive (or stop looking empty)

- **Track ID:** T275-PolicyGrantsFirstRun
- **Status:** **Placeholder** (Pending until `/plan-track 275`)
- **Category:** FEATURE / UX
- **Owner:** Grok
- **Source:** Live CLI audit 2026-08-21 — `briefing project` **8/3**, `query progressive` **8/3**, `evidence`/`source`/`review list` **6/3**, `policy check` **7/7**, `briefing personal` **5/7**
- **Depends on:** T210 ✅ `policy bootstrap`; T221 ✅ deny exit; T241 ✅ doctor `policy_grants`; T263 ✅ H1 honesty (H2 declined)
- **F0:** Plan-only until **go**.

## Problem (live)

`policy bootstrap --dry-run` already `would_issue` ReadEvidence/ReadConclusions/ReadDecisions. Daily path still **POLICY_DENIED**. Project briefing prints `_None_` under Decisions next to ~3296 vault pins. Progressive deny hints `recall` (good) but still looks like a dead product. Doctor warn `0 of 3` is true; the remediator is not on the default first-run path.

## How to ≥8 (ideally 10)

After a **hermetic** (or operator-confirmed) `policy bootstrap`, `briefing project` is not Denied-empty; `query progressive` / `evidence list` are not POLICY_DENIED. If grants stay empty, briefing **must not** read as “the vault has no decisions” — name `recall` for pins (T263 H1 for *granted-empty*; Denied should still bootstrap). Prefer doctor/preflight one-line that matches `policy show` (omit `--scope` when project context is authoritative → **T280** owns the string).

## Manual DoD (on go)

Hermetic vault + project context. **Do not** bootstrap the live operator vault unless the owner confirms.

```powershell
ai-brains policy bootstrap --dry-run
# then hermetic --confirm bootstrap
ai-brains briefing project --format human
ai-brains evidence list --format json
```

Pass: dry-run `would_issue` ×3; after bootstrap, briefing has no `**Denied:**` (empty_authority + `recall` is OK); evidence list exit 0 (items may be `[]`). Live operator vault: only `--dry-run` unless confirmed.

## Isolation

No pin→Approved (T263 H2). No live grant admin/revoke beyond discovery three. Hint wording → **T280**. Ranking → **T274**.
