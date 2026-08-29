# T312–T324 — Post-T311 live CLI dogfood (placeholders)

**Source:** Non-destructive CLI audit **2026-08-27** on PATH graph-on `ai-brains` **0.1.3** (elevated install 2026-08-27 **05:52**; CLI **26,842,112** B). Live vault `C:\dev\ai-brains\vault.db`; Scope `3581317d`; pinned **4510**; grants **3 of 3**. Agent non-TTY (pipe → JSON). Plus **entire** `conductor/deferred.md` open residuals that still deserve a track.
**Status:** **T312 Completed** (`#230` `44520d8`). **T314 Completed** (`#232` `cd7bfde`). **T315 Completed** (`#231` `ae6615d`). **T313 Completed** (`#233` `dae7df3`, FEATURE `a58ee509`). **T317 Completed** (`#234` `fa353c7`, FEATURE `39e0e1e4`). **T319 Completed** (`#235` `e03c49d`, FEATURE `ce627277`). **T320 Completed** (`#237` `c3abe19`, FEATURE `a700986c`). **T316 Completed** (chrome-skip + drop F36; FEATURE `50c73816`). **T318 Completed** (usable-only Default list + F6 stdout + mixed-verify summary; FEATURE `93fbf235`). **T321 Completed** (`#243` `0eef80b`, FEATURE `3fadf62c`). **T322 Completed** (`#244` `766a6c8`, FEATURE `331ce060`). **T323 Completed** (`#245` `5b50d56`, FEATURE `4bef80a8`). **T324 Planned** (PowerShell 5.1 empty TERM: `--term` + omit→fail_usage; Pending until go). **T325 minted** from `#230` Cursor (F8 recency). **T326 minted** from `#237` Cursor (`PinnedCountFailed` fake `pinned=0`) — not in the original audit map. **T307 stays Blocked.** **T311 Completed** (`#229`).
**HEAD at T313 implement:** track branch off `cd7bfde` T314 `#232`.
**Ledger (registration):** series mint DOCS `a6d3c404`. T315 plan DOCS `ca5b1614`. T314 plan DOCS `23da7568`. T314 fold-in DOCS `0d3c2e80`. T314 FEATURE `26f296f5`. T313 plan DOCS `bdf8fddd`. T313 fold-in DOCS `5fa5626e`. T313 FEATURE `a58ee509`. T317 plan DOCS `0db2a64d`. T317 fold-in DOCS `e1ef2696`. T317 FEATURE `39e0e1e4`. T319 plan DOCS `844bdbed`. T319 fold-in DOCS `09c2659f`. T319 FEATURE `ce627277`. T320 plan DOCS `dcb67912`. T320 fold-in DOCS `a92f9b07`. T320 FEATURE `a700986c`. T316 plan DOCS `66b597f7` (mints T326). T316 fold-in DOCS `69e50ba1`. T316 FEATURE `50c73816`. T318 plan DOCS `156b2a03`. T318 FEATURE `93fbf235`. T321 plan DOCS `956c8463`. T321 FEATURE `3fadf62c`. T322 plan DOCS `d8e6e556`. T322 FEATURE `331ce060`. T323 plan DOCS `61b188d1`. T323 fold-in DOCS `853b18d9`. T323 FEATURE `4bef80a8`. T324 plan DOCS `3b998d33`.
**last-PR Cursor:** [#232](https://github.com/Ryan-AI-Studios/AI-Brains/pull/232) T314 — comments **empty**. `#230` Bugbot **1 medium** (F8 OR-fill skips PreferRecency) already **T325**. `#231` was empty.

Scores below are **Usefulness / Quality** from that audit (1–10). Every command with **U&lt;8 or Q&lt;8**, plus every “doesn’t work,” friction, and significant-opportunity item, maps to **exactly one** track unless **declined**.

Live re-check this pass (PATH 0.1.3, cwd `C:\dev\AI-Brains`):

| Signal | Observation |
|--------|-------------|
| `preflight --summary` | Pinned **4510**. In-context hotspots/decisions/constraints **0/0/0**. `Total Word Count: 781`. |
| `recall "graph backend" --no-bridge --limit 3` (piped) | **#1** the audit dump itself (score **−4.06**). **#2** T309 OpenCode plan-audit `## Objective`. **#3** `# Review of Track 253`. No leading `DECISION:` pin. T285 shipped; live rank still dump-first. |
| `query expand` clap | No `--format`. `query progressive --dry-run` is `ArgAction::Set` (requires `true`/`false`). `project scan-roots` has no `--dry-run` (always dry-run). |
| `memory list` | stderr F36 forget nudge. Previews still first-line ingest. |
| backup list | T295 left ≥1 usable; Default table still lists residual fleet; F6 one-line stderr summary exists. |
| T311 PATH | Owner reinstall **2026-08-27 8:21:55 PM** **26,897,408** B (mint was 05:52 / 26,842,112 B). Ranking hole is still **source + PATH** (T285 on PATH; T312 not). Do **not** `cargo install` as planning. |

Pins (workspace, snapshot — re-verify at execute): clap **4.5** / lock **4.6.1**; rusqlite **0.40.2**; workspace **0.1.3**.

## Audit → track map

| Finding | U/Q or class | Track | Pri |
|---------|--------------|-------|-----|
| `recall` / `--semantic` still rank review dumps over DECISION pins; negative BM25 can lead | 9/**8** FTS but finding #2; `--semantic` 9/**7** | **T312 Completed** | P0 |
| `sync query` ledger pane silently phrase-miss → fuzzy token rescue; provenance opaque | 8/**7** | **T313 Completed** | P1 |
| `--format` missing on `query expand`; `--dry-run` requires a value on `query progressive`; `scan-roots` rejects `--dry-run` | friction (5 clap errors) | **T314 Completed** | P1 |
| `preflight --summary` 0/0/0 + opaque `Total Word Count`; no “run X to populate” | 8/**7** | **T315 Completed** | P0 |
| `memory list` raw first-line previews; forget nudge reads like an error | 6/**6** | **T316** | P2 |
| `graph neighbors` RECALLS spam (19 edges; live **11** on `431f6505-…`); hierarchy `synthesized_from` empty | 6/**5** | **T317 Completed** | P1 |
| `backup list` residual plaintext rows drown the 1 usable; verify repeats per-file | 6/**6** | **T318 Completed** | P2 |
| `evidence show` / `source show` on a vault memory UUID → `Handle not found` / `NOT_FOUND` | friction / 2–3/**4** show path | **T319 Completed** | P1 |
| No single `ai-brains status` (doctor + nightly + graph + daemon) | opportunity | **T320 Completed** | P1 |
| `safety sync` is a write (pins hotspots) but grouped as read-ish; chatty | 5/**5** | **T321 Completed** | P2 |
| T311 R2 — `decision in-force` has no `--as-of` | deferred residual | **T322 Completed** | P2 |
| T311 R5 — no conclusion in-force | deferred residual | **T323 Planned** | P2 |
| T311 R7 — PowerShell `""` drops empty TERM | deferred residual | **T324 Planned** | P2 |
| T312 F8 Prefer-OR skips PreferRecency (`#230` Cursor) | last-PR leftover | **T325** | P1 |
| T320 glance `PinnedCountFailed` invents `pinned=0` (`#237` Cursor) | last-PR leftover | **T326** | P1 |

## Declined (written — not minted)

| Item | Why |
|------|-----|
| `doctor` / `nightly --status --quick` / `whoami` / `scope resolve` / `pin --dry-run` / `recovery export` / `retention plan` / `harness status` / `context --show` / `project list` | U≥8 **or** Q≥8 on 2026-08-27; not the friction list |
| T263 **H2** pin → Approved; nightly auto-populate governed from pins | Standing. T288/T290/T315 honesty + next-step, not promotion |
| Governed `query`/`evidence`/`source`/`review`/`briefing` empty collections | T263 H1 + T288/T290 already honest empty. Do **not** remint populate |
| `conclusion` / `decision` “propose-only, no read” | **T311** shipped `decision in-force`. Residual reads are T322/T323 |
| `sync pull` / `sync push` no-op without relay | T92 exists; honest empty. Not a daily single-machine product |
| `device status` / `replicate status` dead on one machine | **T298** Completed honesty |
| `dogfood` / `evaluate` / `migrate governed` | Dev-only, correctly gated |
| T307 dual `tower-http` | Already **Blocked** (reqwest#3062) |
| T308 / T278 density floor retune | Standing |
| T311 R1 daemon `ListInForce` | No daemon consumer yet; F13 soft. Mint later if a DTO caller appears |
| T311 R3 sibling Approved same term | T311 F7 earliest-root **by design** |
| T311 R4 `approved_at` column | **T322 declined** — hop-stop uses superseded/revoked `updated_at`; event field stays unprojected |
| T311 R6 PATH install | Owner elevated install **done** 2026-08-27 |
| T310 R1 `daemon update` self-replace os error 5 | cargo#3486; OR-path is the live sequence |
| T310 F15 / `ai-brainsd --version` | Do **not** add |
| `recovery_kit_event` doctor warn | Doctor Q=9; ceremony is ops, not this series |
| clap **5** / DTO new required keys / silent `.env` rewrite (T240 F2) | Standing |
| Pipe-only JSON on `recall` with no banner | T266 `--format auto` by design; agents that want a table pass `--format human` |

## Suggested implement order

1. **T315** Completed (`#231`)
2. **T314** Completed (`#232`)
3. **T313** Completed (`#233`)
4. **T317** Completed (`#234`) / **T319** Completed (`#235`) / **T325** (F8 recency leftover)
5. **T320 Completed** (unified `status` glance)
6. **T316 Completed** / **T318 Completed** / **T321 Completed** / **T326** (`#237` pin-count)
7. **T322 Completed** (`#244`) / **T323 Completed** (`#245`) / **T324 Planned** (5.1 empty TERM `--term`)

## Non-goals of this series

Live `retention apply --confirm`, CE, `migrate governed`, pin→Approved, clap 5, silent `.env` rewrite, schtasks mutate, graph default-on Cargo, `cargo install` as planning, raising `candidate_depth`, T218 floor retune, `KIND_*` bump without a T312 full plan.

## Registry

See `conductor/conductor.md` T312–T324 and each `trackT3xx-*/spec.md`. Residuals stay in `conductor/deferred.md` until a track closes.


