# T240–T255 — Post-install CLI effectiveness (placeholders)

**Source:** Non-destructive CLI audit 2026-08-11 (global graph-on install; live vault).
**Status:** 📋 **Placeholder series** — plan-only until each track gets **go**. **T243 Completed** 2026-08-12 PR #153. **T245 Completed** 2026-08-12 PR #155. **T247 Completed** 2026-08-13 PR #157. **T246 Completed** 2026-08-13 PR #159. **T248 Completed** 2026-08-14 PR #161. **T249 Completed** 2026-08-14 PR #163.
**Prior closed series:** T217–T232 CLI quality; T233 multi-root; T234–T239 harness ingest.

## Audit → track map

| Finding (E / Q or improvement) | Track | Pri |
|--------------------------------|-------|-----|
| Default project wrong (`test-alias` vs git/path); `project detect` **6/7**; path/env/alias/register-path three answers | **T240** ✅ **Completed** PR #144 | P0 |
| Governed cold-start: briefing **4–5**, progressive query **3**, evidence/source/review **3**, `policy show` empty **6/5**, `policy check` usage **5** | **T241** ✅ **Completed** PR #151 | P0 |
| `.env` override warning spam on nearly every command | **T242** ✅ **Completed** PR #147 | P1 |
| Dual search mental model; progressive dead-end; T231 soft (search noun / recall text arm) | **T243** ✅ **Completed** PR #153 | P1 |
| Backup fleet 0 OK / 21 FAIL legacy; list **Q7**; verify **E7**; doctor `backup_recent` warn / false-usable PreT109 | **T244** ✅ **Completed** PR #149 | P1 |
| Harness `wiring=missing` despite `install_ready` | **T245** ✅ **Completed** PR #155 | P1 |
| Graph neighbors **7/6** JSON-only; hierarchy/session human missing | **T246** ✅ **Completed** PR #159 — TTY pretty; frozen JSON keys; crate `*_with_depth`; update human opt-in | P2 |
| `nightly --status` 4–6s; Last Result **101** residual | **T247** ✅ **Completed** PR #157 — live Last Result **1** named; `"skipped"` literal; LIST/V struct | P1–P2 |
| `retention plan` **7/7** JSON-only / empty classes thin | **T248** ✅ **Completed** PR #161 — TTY `auto` human; `memory_legacy` → `skip`; JSON keys frozen; apply default JSON | P2 |
| `scope resolve` always JSON **Q7**; `daemon status` **Q7**; no `doctor --summary` | **T249** ✅ **Completed** PR #163 — TTY `auto` human; JSON keys frozen; Stopped `next:`; real `--summary` | P2 |
| `preflight --pretty` density **7/7** (second pass after T219) | **T250** | P3 |
| `device status` missing; multi-device discoverability | **T251** | P3 |
| `ingest --dry-run` empty stdin **5/7** | **T252** | P3 |
| Claude/Codex `install_ready` (**T239+** residual) | **T253** | P2 |
| T233 soft: list-paths / unregister-path / from-scan / route method | **T254** | P3 |
| T229 soft: doctor model ports / JSON status / embed sleep (F8–F12/F14) | **T255** | P3 |

**Scored ≥8 and not tracked here unless improvement-listed:** `doctor`, `recall` FTS/semantic/empty, most `preflight --summary*`, `project list`, `memory list`, `harness status` (activation → T245), `pin --dry-run`, `graph update` (pretty → T246), `sync query`, `replicate status`, `safety sync --dry-run`.

## Suggested implement order

1. **T240** identity (unblocks honest daily scores)
2. **T241** policy bootstrap (unblocks governed)
3. **T242** env quiet (readability everywhere)
4. **T244** backup fleet (recoverability)
5. **T245** harness wiring
6. **T247** nightly status residual
7. **T243** search unify
8. **T246** graph pretty
9. **T248–T252** presentation polish
10. **T253–T255** residuals / soft-after

Parallel after T240+T241 if non-intersecting: T242 ∥ T244 ∥ T245.

## Non-goals of this series

- MSI / notarization / App Store
- clap 5 / forced dep bumps
- Re-implement T233 multi-root core
- Unbounded dump-all / CE wipe product fill
- Concurrent multi-operator register-path atomicity

## Registry

See `conductor/conductor.md` T240–T255 and each `trackT2xx-*/spec.md`.
