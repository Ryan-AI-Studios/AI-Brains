# T256–T271 — Post-T255 live CLI audit (placeholders)

**Source:** Non-destructive CLI audit 2026-08-16 (graph-on PATH install; live vault `C:\dev\ai-brains\vault.db`; agent non-TTY).
**Status:** **T256 Completed** 2026-08-16. **T258 Completed** 2026-08-16. **T259 Completed** 2026-08-17. **T257 In Progress** 2026-08-17 (F0 lifted). Remaining T260–T271 still placeholders.
**Prior closed series:** T217–T232 CLI quality; T233 multi-root; T234–T239 harness ingest; **T240–T255** CLI effectiveness (closer T255 2026-08-16).
**Ledger (registration):** DOCS TX `1d9511b5-798b-4d6c-b0c9-ebb4b07d0b69`.

This series exists because T240–T255 shipped *inspectability* (whoami, doctor identity check, nightly JSON, governed empty-states) and the 2026-08-16 dogfood still failed as a daily product: default Scope is a sandbox, recall returns symbol stubs, governed stores are empty, graph projection lags, and several surfaces still dump JSON / self-next / leak the vault key.

## Audit → track map

Every non-working item, friction point, significant opportunity, and command with **effectiveness or quality &lt; 7** maps to exactly one track. Scores are E / Q from the 2026-08-16 run.

| Finding | Scores / class | Track | Pri |
|---------|----------------|-------|-----|
| `--help` prints live `AI_BRAINS_KEY=x'<hex>'` | 8 / **3**; opp: never print key | **T256** | P0 |
| Identity warning on every command; interleaves JSON (`scope resolve` default + `--format json`) | friction; scope 6/5, json **7/6**; opp: stderr-only | **T257 In Progress** | P0 |
| Daily Scope is `test-alias` `441837f6` (592) not path owner `3581317d` (2,700) | non-working default; opp: rebind path owner | **T258 Completed** | P0 |
| Leftover identity `7d97a456` holds **18,028** memories across many `C:\dev\*` roots | opp: split leftover; poisons `--global` | **T259 Completed** | P0 |
| Recall ranking: symbol stubs beat decisions (`Module sqlite_backend`, `Struct Project`) | default **5/4**; `--semantic` **6/6**; `--global` **3/3**; real-project semantic **4/3**; opp: demote symbols | **T260** | P0 |
| Empty `recall ""` took **5.7 s** | friction | **T261** | P2 |
| Graph unused as daily tool: 21k nodes / 945 edges; recent pin no node; hierarchy empty | neighbors **4/5**; hierarchy **3/4**; opp: live projection | **T262** | P1 |
| Governed store empty (0 evidence / conclusions / decisions) while vault has hundreds of `DECISION:` pins | briefing **4/6** + **3/6**; progressive **3/5**; expand **6/6**; trace **5/4**; evidence/source/review **3/5**; opp: connect or stop advertising | **T263** | P0 |
| `preflight --global` mixes other repos (coordinator 0022/0023, hip-hierarchy) as “safety” | global pretty **5/4**; global summary **7/6** | **T264** | P1 |
| `preflight --format json` is a `{text, word_count}` blob | **7/6** | **T265** | P2 |
| Format policy maze (human vs JSON default; list-paths JSON wall; retention default JSON on non-TTY) | friction; list-paths **7/5**; retention plan **6/5** | **T266** | P1 |
| Next-action is the command you just ran (`harness status`; `whoami` → whoami); `project list` suggests aliasing `7d97a456` as `AI-Brains` | harness **8/6**; list **8/6**; opp: real remediator | **T267** | P1 |
| `project scan-roots` only scans cwd; suggests re-registering an existing path | **4/5** | **T268** | P2 |
| Nightly human mixes Nightly Last Result **0** with Router **267009**; full status completion `probe=timeout` while daemon says Open | friction (nightly scores ≥7) | **T269** | P2 |
| `retention plan` 0 candidates across 35,300 memories (`memory_legacy / none_auto`) | **6/5** | **T270** | P2 |
| `sync query` ledger pane empty for a query this repo’s ledger should hit | **5/5**; friction | **T271** | P1 |

**Scored ≥7 / ≥7 and not tracked unless listed above:** `doctor` (all variants), `preflight --summary` / `--pretty --compact` (project-scoped), `recall --format pretty` (when hits are recent DECISION pins), `search` alias, `scope --format human`, `daemon status`, `nightly --status` / `--quick` / `--format json` / `--dry-run` (Router/probe residuals → T269), `project whoami` / `detect` / `resolve`, `memory list` / `--summary` / forgotten, `retention plan --format human`, `graph update`, `device` / `replicate` empty-states, `policy show` / `check`.

## Suggested implement order

1. **T256** (secret leak — one-file clap env display)
2. **T258** then **T259** (identity; unblocks honest recall/preflight scores)
3. **T257** (warning/JSON; unblocks every scripted command)
4. **T260** (recall ranking — largest daily-quality gap)
5. **T263** (governed honesty vs promotion — pick at plan time)
6. **T262** (graph projection)
7. **T264 / T271 / T266 / T267** (preflight global, ledger pane, format, next-action)
8. **T261 / T265 / T268 / T269 / T270** (latency, JSON envelope, scan-roots, nightly split, retention classify)

Parallel if non-intersecting after T258: T256 ∥ T257 ∥ T261; T264 ∥ T265; T268 ∥ T269.

## Non-goals of this series

- New track after T271 in this registration (no T272+)
- clap 5 / forced dep bumps / new crates
- Silent `.env` rewrite or silent Scope auto-switch (T240 F2 stands until T258 explicitly changes it with confirm)
- Product `nightly-run.cmd` / schedule-Router mutate (T255 decline)
- Doctor 16th model-port check (T255 decline)
- MSI / notarization / App Store
- CE wipe / NIST Purge / multi-device product fill
- Capturing hidden CoT or tool logs

## Registry

See `conductor/conductor.md` T256–T271 and each `trackT2xx-*/spec.md`. Residuals stay in `conductor/deferred.md` until a track closes.
