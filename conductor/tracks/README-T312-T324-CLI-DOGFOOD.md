# T312–T324 — Post-T311 live CLI dogfood (placeholders)

**Source:** Non-destructive CLI audit **2026-08-27** on PATH graph-on `ai-brains` **0.1.3** (elevated install 2026-08-27 **05:52**; CLI **26,842,112** B). Live vault `C:\dev\ai-brains\vault.db`; Scope `3581317d`; pinned **4510**; grants **3 of 3**. Agent non-TTY (pipe → JSON). Plus **entire** `conductor/deferred.md` open residuals that still deserve a track.
**Status:** **T312 Completed** (`#230` `44520d8`). **T315 Planned** (full F-list 2026-08-28, plan DOCS `ca5b1614`, fold-in DOCS `c90c1c71`). T313–T314 / T316–T324 still placeholders. **T325 minted** from `#230` Cursor (F8 recency) — not in the original audit map. Do **not** implement until **go**. **T307 stays Blocked.** **T311 Completed** (`#229`).
**HEAD at T315 plan:** `44520d8` T312 squash. Tree **CLEAN** (product). `origin/main` in sync.
**Ledger (registration):** series mint DOCS `a6d3c404`. T315 plan DOCS `ca5b1614`.
**last-PR Cursor:** [#230](https://github.com/Ryan-AI-Studios/AI-Brains/pull/230) T312 — Bugbot **1 medium** (F8 OR-fill skips PreferRecency). **Minted T325.** `#229` was empty.

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
| `sync query` ledger pane silently phrase-miss → fuzzy token rescue; provenance opaque | 8/**7** | **T313** | P1 |
| `--format` missing on `query expand`; `--dry-run` requires a value on `query progressive`; `scan-roots` rejects `--dry-run` | friction (5 clap errors) | **T314** | P1 |
| `preflight --summary` 0/0/0 + opaque `Total Word Count`; no “run X to populate” | 8/**7** | **T315 Planned** | P0 |
| `memory list` raw first-line previews; forget nudge reads like an error | 6/**6** | **T316** | P2 |
| `graph neighbors` RECALLS spam (19 edges); hierarchy `synthesized_from` empty | 6/**5** | **T317** | P1 |
| `backup list` residual plaintext rows drown the 1 usable; verify repeats per-file | 6/**6** | **T318** | P2 |
| `evidence show` / `source show` on a vault memory UUID → `Handle not found` / `NOT_FOUND` | friction / 2–3/**4** show path | **T319** | P1 |
| No single `ai-brains status` (doctor + nightly + graph + daemon) | opportunity | **T320** | P1 |
| `safety sync` is a write (pins hotspots) but grouped as read-ish; chatty | 5/**5** | **T321** | P2 |
| T311 R2 — `decision in-force` has no `--as-of` | deferred residual | **T322** | P2 |
| T311 R5 — no conclusion in-force | deferred residual | **T323** | P2 |
| T311 R7 — PowerShell `""` drops empty TERM | deferred residual | **T324** | P2 |
| T312 F8 Prefer-OR skips PreferRecency (`#230` Cursor) | last-PR leftover | **T325** | P1 |

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
| T311 R4 `approved_at` column | JSON `updated_at` until T322 plan proves a column is required |
| T311 R6 PATH install | Owner elevated install **done** 2026-08-27 |
| T310 R1 `daemon update` self-replace os error 5 | cargo#3486; OR-path is the live sequence |
| T310 F15 / `ai-brainsd --version` | Do **not** add |
| `recovery_kit_event` doctor warn | Doctor Q=9; ceremony is ops, not this series |
| clap **5** / DTO new required keys / silent `.env` rewrite (T240 F2) | Standing |
| Pipe-only JSON on `recall` with no banner | T266 `--format auto` by design; agents that want a table pass `--format human` |

## Suggested implement order

1. **T315** (summary next-step; T312 already Completed)
2. **T325** (F8 recency leftover) or **T314** (clap friction unblocks every later CLI track)
3. **T313** / **T317** / **T319**
4. **T320** (compose; after doctor/nightly stay stable)
5. **T316** / **T318** / **T321**
6. **T324** (small clap/usage) then **T322** / **T323** (governed reads)

## Non-goals of this series

Live `retention apply --confirm`, CE, `migrate governed`, pin→Approved, clap 5, silent `.env` rewrite, schtasks mutate, graph default-on Cargo, `cargo install` as planning, raising `candidate_depth`, T218 floor retune, `KIND_*` bump without a T312 full plan.

## Registry

See `conductor/conductor.md` T312–T324 and each `trackT3xx-*/spec.md`. Residuals stay in `conductor/deferred.md` until a track closes.
