# T217–T232 — Post-audit CLI quality series (placeholders)

**Source:** Non-destructive CLI audit 2026-08-05 (scores &lt; 7 + high-leverage improvements).
**Status:** Series active — **T217 closed**; **T220 Planning**; remaining tracks plan-only until go.
**Prior series:** T205–T216 closed (skill·CLI honesty).
**Closed:** **T217** FTS multi-token rescue (PR #110 `1e22e77`, 2026-08-09).
**Planning:** **T220** Preflight summary JSON honesty (machine object; T180 full path freeze).

## Score → track map

| Audit finding (use / quality) | Track | Priority |
|-------------------------------|-------|----------|
| FTS natural phrases empty (7 / **4**) | **T217** ✅ closed PR #110 | P1 |
| Semantic topic drift (6 / **4**) | **T218** Semantic quality v2 | P1 |
| Preflight pretty wall (8 / **5**) | **T219** Preflight pretty readability | P2 |
| `preflight --summary --format json` ignored (**4 / 3**) | **T220** Preflight summary JSON honesty — 📋 Planning | P1 |
| Governed first-run + progressive deny exit0 (**4–5 / 5–6**) | **T221** Governed first-run + deny exit honesty | P1 |
| Graph-off install usefulness (**3** / 9 honesty) | **T222** Graph-on install path | P2 |
| `.env` override double-warn spam | **T223** Quiet env override warnings | P2 |
| `ASSISTANT:` in recall/sync/forget dry-run | **T224** Search/display role-prefix strip | P2 |
| Backup verify INFO flood + fleet unusable (**7 / 6**) | **T225** Backup verify quiet + encrypted backup nudge | P2 |
| `policy show/check` require scope (**5–6**) | **T226** Policy soft-resolve scope | P2 |
| Briefing human→JSON; empty personal (**4–5 / 5–6**) | **T227** Briefing format + substance | P2 |
| Non-empty pretty no Scope (T207 residual) | **T228** Non-empty pretty Scope | P3 |
| Nightly not scheduled; model env only in project `.env` | **T229** Nightly + local router ops (env/health/schedule) | P0 ops / P2 product |
| Global summary blank labels | **T230** Memory/project label fill under global | P3 |
| Dual search mental model (recall vs sync query) | **T231** Unified search UX defaults | P2 |
| Doctor says `graph rebuild` but graph-off install | **T232** Graph density remediation path | P2 |

## Suggested implement order

1. **Ops now (not a code track):** schedule `AI-Brains-Nightly` + ensure `c:\llm\router.bat` on :8081/:8083 — see T229 + `~\.ai-brains\register-nightly-tasks.ps1`.
2. ~~**T217**~~ (empty FTS trap) **closed**; **T220** 📋 Planning (flag lie → machine object); **T221** (governed dead-end) — honesty.
3. **T218** (semantic), **T219** (pretty), **T224** (role strip).
4. **T222/T232** graph install + remediation.
5. **T223, T225–T228, T230–T231** polish.

## Non-goals of this series

- Tag schema migration / auto-forget / CE wipe
- MSI / notarization
- clap 5
- Full multi-device product fill (device empty states already honest)

## Registry

See `conductor/conductor.md` T217–T232 rows and each `trackT2xx-*/spec.md`.

**Related (ops/architecture, not pure UX):** [T233 path-alias multi-root nightly](trackT233-path-alias-multiroot-nightly/spec.md) — Option B vault paths + nightly Phase2 bridge (closes System32 Ledgerful miss).

**Related (harness ingest series):** [T234–T239 seamless multi-harness ingest](README-T234-T239-HARNESS-INGEST.md) — message-only capture, preflight detect/install hooks, AGY2/Grok/OpenCode + nightly multi-import.

