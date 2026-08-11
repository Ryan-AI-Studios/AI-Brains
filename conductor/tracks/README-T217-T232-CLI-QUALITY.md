# T217–T232 — Post-audit CLI quality series (placeholders)

**Source:** Non-destructive CLI audit 2026-08-05 (scores &lt; 7 + high-leverage improvements).
**Status:** Series active — **T217 + T218 + T219 + T220 + T221 + T222 + T223 + T224 + T232 closed**; **T225 closed**; remaining tracks placeholders.
**Prior series:** T205–T216 closed (skill·CLI honesty).
**Closed:** **T217** FTS multi-token rescue (PR #110 `1e22e77`); **T218** semantic quality v2 (PR #116 `fc4d370`); **T219** preflight pretty readability (PR #118 `496ddd7`); **T220** preflight summary JSON honesty (PR #112 `6f4f67b`); **T221** governed first-run + deny exit honesty (PR #114 `b3c4b0f`); **T222** graph-on install path (PR #122 `c1ac594`, 2026-08-10); **T223** quiet env override warnings (PR #126 `7ff8f7f`, 2026-08-10); **T224** search role-prefix strip (PR #120 `a18fae6`, 2026-08-10); **T232** density remediation (PR #124 `33b28d0`, 2026-08-10). **T225** backup verify quiet + doctor usable nudge (PR #128 `927b8db`, 2026-08-11).
**Next honesty:** **T226**–**T228**, **T230**–**T231** / ops **T229**.

## Score → track map

| Audit finding (use / quality) | Track | Priority |
|-------------------------------|-------|----------|
| FTS natural phrases empty (7 / **4**) | **T217** ✅ closed PR #110 | P1 |
| Semantic topic drift (6 / **4**) | **T218** ✅ closed PR #116 | P1 |
| Preflight pretty wall (8 / **5**) | **T219** ✅ closed PR #118 | P2 |
| `preflight --summary --format json` ignored (**4 / 3**) | **T220** ✅ closed PR #112 | P1 |
| Governed first-run + progressive deny exit0 (**4–5 / 5–6**) | **T221** ✅ closed PR #114 | P1 |
| Graph-off install usefulness (**3** / 9 honesty) | **T222** ✅ closed PR #122 | P2 |
| `.env` override double-warn spam | **T223** ✅ closed PR #126 | P2 |
| `ASSISTANT:` in recall/sync/forget dry-run | **T224** ✅ closed PR #120 | P2 |
| Backup verify INFO flood + fleet unusable (**7 / 6**) | **T225** ✅ closed PR #128 | P2 |
| `policy show/check` require scope (**5–6**) | **T226** Policy soft-resolve scope | P2 |
| Briefing human→JSON; empty personal (**4–5 / 5–6**) | **T227** Briefing format + substance | P2 |
| Non-empty pretty no Scope (T207 residual) | **T228** Non-empty pretty Scope | P3 |
| Nightly not scheduled; model env only in project `.env` | **T229** Nightly + local router ops (env/health/schedule) | P0 ops / P2 product |
| Global summary blank labels | **T230** Memory/project label fill under global | P3 |
| Dual search mental model (recall vs sync query) | **T231** Unified search UX defaults | P2 |
| Doctor says `graph rebuild` but graph-off install | **T232** ✅ closed PR #124 | P2 |

## Suggested implement order

1. **Ops now (not a code track):** schedule `AI-Brains-Nightly` + ensure `c:\llm\router.bat` on :8081/:8083 — see T229 + `~\.ai-brains\register-nightly-tasks.ps1`.
2. ~~**T217**~~ **closed**; ~~**T218**~~ **closed** PR #116; ~~**T219**~~ **closed** PR #118; ~~**T220**~~ **closed**; ~~**T221**~~ **closed**; ~~**T222**~~ **closed** PR #122; ~~**T223**~~ **closed** PR #126; ~~**T224**~~ **closed** PR #120.
3. ~~**T232**~~ **closed** PR #124 — capability-aware rebuild vs reinstall.
4. ~~**T225**~~ **closed** PR #128 — then **T226–T228, T230–T231** polish (or ops **T229**).

## Non-goals of this series

- Tag schema migration / auto-forget / CE wipe
- MSI / notarization
- clap 5
- Full multi-device product fill (device empty states already honest)

## Registry

See `conductor/conductor.md` T217–T232 rows and each `trackT2xx-*/spec.md`.

**Related (ops/architecture, not pure UX):** [T233 path-alias multi-root nightly](trackT233-path-alias-multiroot-nightly/spec.md) — Option B vault paths + nightly Phase2 bridge (closes System32 Ledgerful miss). Upstream coordinated **0163** (`ledgerful symbols`) ✅ **Completed** 2026-08-09 — T233 **unblocked** (still placeholder until go).

**Related (harness ingest series):** [T234–T239 seamless multi-harness ingest](README-T234-T239-HARNESS-INGEST.md) — message-only capture, preflight detect/install hooks, AGY2/Grok/OpenCode + nightly multi-import.

