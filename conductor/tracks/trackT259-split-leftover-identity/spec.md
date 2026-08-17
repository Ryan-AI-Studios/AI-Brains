# T259 — Split leftover identity `7d97a456`

- **Track ID:** T259-SplitLeftoverIdentity
- **Status:** **Planned** (plan-only until go; conductor stays **Pending**)
- **Category:** FEATURE / UX / OPS
- **Owner:** —
- **Source:** Audit 2026-08-16 — leftover `7d97a456` holds 18,028 memories + many `C:\dev\*` roots; poisons `--global` and `project list` footer
- **Depends on:** T233/T254 path aliases (`list-paths` / `register-path` / `unregister-path`); T212 list; T240 whoami; T258 adopt-path (daily Scope stays separate)
- **Blocks / feeds:** Honest nightly Phase 2 per leftover repo; detect/whoami/adopt-path in those repos. `--global` recall stays T260/T264 (historical pins stay on leftover). T267 list footer stays T267.
- **Absorbs:** Shared-ID inventory (11 roots on `7d97a456`); confirmable per-path rebind (unregister leftover + register dest); honesty that unregister ≠ move memories; never recommend `set-alias 7d97a456 AI-Brains` on **new T259 surfaces**
- **Not absorbed:** Daily Scope rebind (T258 Completed); T267 list-footer algorithm; T260 ranking; T264 global isolation; T268 scan-roots parent/`--root`; T257 warn/JSON; memory reclassification / CE wipe / auto-merge; minting dest projects; live vault mutate this planning pass
- **Research date:** 2026-08-17 (plan HEAD `049064d`; fold-in HEAD `e46a2e1`)
- **AI fold-in:** 2026-08-17 `opencode-review.md` + `agy-review.md` (no grok/claude/codex-plan). No Blockers. **Agree hard:** OC-M1 `from_project_id` is `"<uuid>"` never null. **Agree:** OC-m2 1-off count drift (re-snapshot); OC-m3 PATH vs source adopt-path; OC-O4 intersection AC17; OC-O5 empty-filter AC16; AGY-m1 CP `from != to` InvalidPayload; AGY-O1 `resolve_project_ref` **must** be `pub(crate)`. Disposition **§13**.
- **Ledger:** planning DOCS TX `49463c65-1759-4110-b1f3-14beda6dfe58`. Fold-in DOCS TX `79be45ed-5222-465b-90d0-ae999ae51d72`. Implement starts a **FEATURE** TX on **go**.
- **Isolation:** Do **not** unregister live leftover paths. Do **not** `set-alias 7d97a456 AI-Brains`. Do **not** write live `.env`. Do **not** `cargo install`. Do **not** reopen T240 F2, T255 declines, T257, T258, T267. Do **not** bump clap / add crates.

---

## 1. Objective

Stop treating historical leftover UUID `7d97a456-f2f4-43ea-1f13-211af684ad37` as “the big AI-Brains vault.”

T254 already shipped the primitives (`list-paths`, `unregister-path`, `register-path`). The product hole is that operators **cannot see** which ID owns many unrelated roots, and correcting one root is a two-command dance that can leave a path unbound. T259 adds:

1. **Inventory** — `project list-paths --project <id|alias>` and `--shared-only` so a shared leftover ID is visible without scrolling every root.
2. **Confirmable per-path rebind** — `project rebind-path <path> --to <dest>` print-only by default; `--write --yes` appends `RepositoryPathAliasRemoved` (from) + `RepositoryPathAliasAdded` (to) in **one** store transaction.
3. **Honesty** — rebind does **not** move historical memories. 18,028 leftover pins stay on `7d97a456`. Nightly Phase 2 stops walking the rebound path for leftover and starts walking it for dest.

This advances the north star because capture independence and the append-only log stay intact (compensating path events only; no memory rewrite). Daily Scope for *this* repo is already T258. Leftover sibling repos (`C:\dev\crawlx`, `gimp`, …) cannot become first-class identities until their path alias leaves the dump.

---

## 2. Live baseline (2026-08-17)

### 2.1 Operator dogfood (this machine)

| Signal | Observation |
|--------|-------------|
| HEAD | `049064d` — T258 Completed (`#171`). Tree CLEAN. `main` = `origin/main`. |
| PATH `ai-brains` | `C:\Users\RyanB\.cargo\bin\ai-brains.exe` (mtime 2026-08-16 08:04). **PATH binary** remediations still say hand-edit + `project list`. **Source** at HEAD already emits adopt-path (`project.rs:823–824`). Do not conclude adopt-path is absent from source. **Do not `cargo install`.** |
| `preflight --summary` | Scope `test-alias` (`441837f6`); mismatch warn → whoami. T258 adopt-path exists in source; live `.env` not rebound this pass. |
| `project whoami --format json` | env/effective `441837f6`; **shell leftover `7d97a456`**; path/detect `3581317d`. `mismatch: true`. Source remediations name `adopt-path`. |
| `project list --format json` | 39 projects; `unaliased_count` **27**. `7d97a456` **18,028** mem, alias empty, first path `C:\dev\crawlx`. Fold-in re-snapshot (2026-08-17): `3581317d` **2,753** mem; `441837f6` **595** mem, `test-alias`, path null, active `*`. Load-bearing leftover / root counts unchanged. |
| `project list` stderr footer | `27 project(s) have no alias.` / `Example: ai-brains project set-alias 7d97a456-f2f4-43ea-1f13-211af684ad37 AI-Brains`. **Harmful.** T267 F3 owns the algorithm. |
| `memory list --summary --global` | Plan-time pinned **35,520**. Fold-in re-snapshot **35,561** (1-off drift; leftover still **18,028**). Top row leftover ~51%. Phase 0 re-counts totals; do not treat vault totals as load-bearing. |
| `project list-paths --format json` | **17** roots / **7** owner IDs. **`7d97a456` owns 11.** `3581317d` owns 1 (`C:\dev\ai-brains`). Six other IDs own 1 each. |
| Leftover 11 roots (all `exists: true`) | `C:\dev\crawlx`, `dedupe`, `degoo`, `family`, `gimp`, `homebrew-tap`, `kinledger`, `ledgerful-action`, `ledgerful-frontend`, `ledgerful-web`, `wondermaker`. |
| Dest projects for those folders | **None** in `project list` (no name/alias/path match except leftover’s first-path crawlx). Operator must create dest out of band (`context` in that repo) **before** rebind. |
| Leftover `last_activity` | `2026-08-16T19:30:43Z` — nightly Phase 2 is **still ingesting** onto leftover. Split is P0 for *future* symbols, not for history. |
| Last GitHub PR | [#171](https://github.com/Ryan-AI-Studios/AI-Brains/pull/171) T258 merged 2026-08-17. `gh pr view --comments`, `/reviews`, `/comments` all **empty**. HEAD is `main` (no open product PR). Open PRs are Dependabot only. **last-PR Cursor: N/A.** |
| Ledgerful | `doctor` ready (legacy `.changeguard` / sig-pin / timings / embed unreachable). 0 pending at plan start. Work root `C:\dev\AI-Brains`. Hotspot **#1** = `project.rs` (3.818, **1549** lines). `project_paths.rs` **467**. `project_adopt.rs` **233**. |
| ai-brains recall | Scoped to test-alias (T258 hole still live). Lexical `--no-bridge` hit T254 fold-in: no camino; Removed owner-scoped; `project_paths.rs` returns `Err` not `process::exit`. No prior “rebind-path” pin. |

### 2.2 Live identity triangle (re-verified)

```text
  repo .env PROJECT_ID ──► 441837f6  test-alias     595 mem   daily Scope *
                           ▲ T258 adopt-path is the remediator (source). Live .env not rebound.

  shell (pre-dotenv)   ──► 7d97a456  (no alias)  18,028 mem  first-path=C:\dev\crawlx
                           ▲ leftover dump. 11 C:\dev\* roots. Do not set-alias AI-Brains.

  path alias / detect  ──► 3581317d  2,753 mem   path=C:\dev\ai-brains
                           ▲ this repo. T258. Do not rebind this path onto leftover.
```

cwd `C:\dev\AI-Brains` vs registered path `C:\dev\ai-brains` — `normalize_for_location_compare` already treats these as the same location (T240).

T254 made every root **listable**. It did not make a shared leftover ID **filterable**, and it did not give a one-shot **rebind** that cannot forget to re-register.

### 2.3 Why this still matters

| Residual | Why it is a product hole / why decline |
|----------|----------------------------------------|
| 11 roots on one leftover ID | Nightly Phase 2 walks them as **one** project. New symbols from gimp/crawlx/family keep landing on `7d97a456`. **DoD: inventory + per-path rebind.** |
| `list-paths` has no filter | 17-row dump. No `--project`, no `--shared-only`. **DoD.** |
| Unregister then register is two commands | Unregister without register leaves detect/whoami with **no** path owner. Operators will half-split. **DoD: composed rebind, one tx.** |
| Historical 18,028 memories | Event log is append-only. There is no `MemoryMoved`. **Decline auto-split of pins** (soft residual / later if ever). Honesty is DoD. |
| `--global` still leftover-first | Path split does **not** unpoison global recall. **Decline to T260/T264.** |
| List footer `set-alias 7d97 … AI-Brains` | Highest-memory unaliased + cwd git slug. **Decline algorithm to T267 F3.** T259 must not repeat that pair on new surfaces. |
| Dest projects missing | Leftover folders have no sibling project row. **Decline dest mint.** Runbook: `context` in that repo, then rebind `--to` that UUID. |
| Live split this session | Stop-before. Planning + implement tests = tempdir only. Operator live rebind is out of band unless the owner asks. |

### 2.4 Code truth

| Item | Location | Notes |
|------|----------|-------|
| `ProjectCommands` | `main.rs` **:2066–2170** | List / Resolve / Detect / SetAlias / RegisterPath / Whoami / AdoptPath / ListPaths `{format}` / ScanRoots / UnregisterPath. **No** RebindPath. ListPaths has **no** `--project` / `--shared-only`. |
| Dispatch | `main.rs` **:4185–4213** | `list_paths(&ctx, format)` only. Exhaustive match. |
| list-paths | `project_paths.rs` `list_paths` **:98–153** | `list_path_aliases` + `list_projects` HashMap join. Frozen JSON keys T254 F10. Sort is store `normalized_path ASC`. |
| unregister-path | `project_paths.rs` `unregister_path` **:347–388** | Normalize; owner lookup; optional `--project` match; `--dry-run` print; else CP `unregister_path_alias`. Returns `Err` (F37). Missing path exit **0**. |
| `resolve_project_ref` | `project_paths.rs` **:391–414** (private); duplicate in `project.rs` **:995** | UUID-in-vault **or** alias. Rebind + list-paths `--project` **reuse** the paths helper (`pub(crate)`). Do not grow `project.rs`. |
| register-path | `project.rs` `register_path` **:953–992** | F21 pre-check; **`process::exit(1)`** on conflict (T254 declined refactor). Rebind **must not** call this (would exit; not one-tx). |
| CP write | `control-plane/grants.rs` `register_path_alias` **:232** / `unregister_path_alias` **:259** | One event each. **Add** `rebind_path_alias` that `append_events(&[Removed, Added])`. |
| Store `append_events` | `event_store.rs` **:242–267** | **One SQLite transaction** for the slice. Partial unregister is avoided if CLI uses this helper. |
| Path owner | `find_path_alias_owner` / `list_path_aliases` | T233/T254. Reuse. No new SQL. |
| List footer | `project.rs` `print_unaliased_footer` **:105–123** | Highest-memory unaliased + `footer_alias_suggestion` = cwd git slug (`AI-Brains`). **Do not edit** (T267). |
| T258 adopt-path | `project_adopt.rs` | Print-only + `--write-env --yes` + `--format auto`. Pattern for rebind flags (`--write --yes`). |
| T254 hermetics | `tests/project_path_aliases.rs` | Unregister / list-paths / scan-roots. Stay green. New file `tests/project_rebind_path.rs`. |
| `context` dest mint | `commands/context.rs` + `context.rs` `ensure_project_and_session_exists` **:107** | `ProjectRegistered` when missing. **Out of band** dest create. Do not call from rebind. |
| Reparse | `ai_brains_path::normalize_for_location_compare` | Same as register/unregister. No `.env` write → no reparse-on-file. |
| help_ia | `help_ia.rs` | Additive CAPABILITIES CONTEXT string only. Root groups unchanged. |
| Hotspots | `project.rs` **#1** (1549 lines). `context.rs` **#4**. | Rebind → **`project_rebind.rs`**. list-paths flags stay in **`project_paths.rs`**. **Do not** grow `project.rs`. **Do not** edit `context.rs`. |

### 2.5 Dependency / standards research (2026-08-17)

| Pin | Workspace / lock | Action |
|-----|------------------|--------|
| `clap` | workspace **4.5** / lock **4.6.1** (builder **4.6.0**) | **No bump.** New `RebindPath` + list-paths flags. `--yes` `requires = "write"` (same derive as T258 `write_env`). |
| crates.io clap | latest **4.6.6** (2026-08-06). docs.rs `Arg::requires` still the relation used by T258. **No clap 5.** | Snapshot — re-verify at execute. |
| `dotenvy` | lock **0.15.7** | **N/A write.** T259 does not touch `.env`. Confirmed still load-only on docs.rs 0.15.7 (T258). |
| `uuid` | workspace **1.13** / lock **1.23.1** | **No bump.** |
| `serde` / `serde_json` | **1.0** / lock **1.0.150** train | **No bump.** CLI-local JSON only. |
| rustc / edition | **1.95.0** / **2024** | Unchanged. |
| workspace version | **0.1.1** | **No bump.** |
| New crates | — | **Zero.** No camino in CLI (T254 pin). `std::path` + `to_string_lossy`. |
| Event sourcing | [Azure Event Sourcing](https://learn.microsoft.com/en-us/azure/architecture/patterns/event-sourcing) + Fowler | Compensating event reverses projection effect; original Added stays. Fits T254 Removed + this rebind pair. **Do not** rewrite `MemoryPinned`. |
| Tenant-split practice | Same sources: historical facts stay on the old stream; new writes go to the new identity | Fits F5 honesty. `--global` leftover dominance is T260/T264, not a memory rewrite. |
| Contracts / T180 | No path-alias DTO today | **No** contracts change. |
| SQLCipher / schtasks | — | **N/A.** |

---

## 3. Frozen decisions

| ID | Decision |
|----|----------|
| **F0 — Go gate** | Plan-only until user **go**. Planning is DOCS. Implement starts a **FEATURE** TX. |
| **F1 — Never leftover-as-AI-Brains** | New T259 surfaces (help, human/JSON remediations, CAPABILITIES/WORKFLOWS runbook) **must not** recommend `set-alias 7d97a456 … AI-Brains`. Do **not** change `print_unaliased_footer` (T267 F3). |
| **F2 — Inventory filters** | `project list-paths` gains optional `--project <id\|alias>` and `--shared-only`. Unfiltered JSON **shape stays T254 F10**. Filters only change which rows appear. `--shared-only` = owner appears **≥2** times in `list_path_aliases`. Both flags may combine (intersection). |
| **F3 — Verb** | `ai-brains project rebind-path <path> --to <project-id\|alias>`. Not `split-path` (implies memory split). Not `unregister`+`register` as the documented happy path. Not `adopt-path` (that is `.env`). Not `project use`. |
| **F4 — Print-only default** | Default rebind is print-only (no events). Write requires **both** `--write` and `--yes`. `--yes` clap-`requires` `--write`. `--write` without `--yes` → exit **2**, no events, stderr names `--yes`. No `--dry-run` twin (T254 unregister keeps `--dry-run`; do not fork that onto rebind). |
| **F5 — Memories stay** | Rebind never moves, copies, forgets, or CE-wipes memories. JSON always `memories_moved: false`. Human always says historical memories stay on `from`. Hermetic: dest and from `memory_count` unchanged aside from fixture seed. |
| **F6 — One transaction** | Write path calls new CP `rebind_path_alias` → `append_events(&[Removed(from), Added(to)])` (store already one SQLite tx). Helper **must** reject `from == to` with `ControlPlaneError::InvalidPayload` (same family as empty-path on register/unregister) so a programmatic caller cannot append a no-op Removed+Added pair. CLI still short-circuits already-bound (F7) and never calls the helper in that case. Do **not** call `project.rs` `register_path` (`process::exit`). Do **not** two separate CLI appends. |
| **F7 — Already bound** | If current owner == dest: exit **0**, `already_bound: true`, `written: false`, **no** events. |
| **F8 — No owner** | Path not in projection → exit **1**, no events, stderr names `register-path`. (Unlike unregister missing-path exit 0 — rebind has nothing to rebind.) |
| **F9 — Dest must exist** | `--to` unresolved or UUID not in `list_projects` → exit **1**, no events. Do **not** mint `ProjectRegistered`. Do **not** call `context` / `ensure_project_and_session_exists`. Runbook: create dest via `context` in that repo first. |
| **F10 — One path** | No bulk. No `--all`. No “split leftover” that walks 11 roots. Operator confirms **per path**. |
| **F11 — No `.env`** | Rebind never writes `.env` / global dotenv. T240 F2 stands. T258 adopt-path remains the Scope remediator. |
| **F12 — Module** | New `crates/ai-brains-cli/src/commands/project_rebind.rs`. list-paths filters stay in `project_paths.rs`. `project.rs` **untouched**. `context.rs` **untouched**. Promote `project_paths::resolve_project_ref` to **`pub(crate)`** and reuse it from rebind + `--project` — do **not** duplicate the helper. |
| **F13 — Format** | `--format auto\|human\|json` (same parser as list-paths / adopt-path: `auto` → JSON when stdout is **not** a TTY). Frozen JSON keys §5.1. |
| **F14 — Hermetic format** | ACs that assert human chrome **must** pass `--format human`. `Command.output()` is a pipe → `auto` is JSON (T258 F26). |
| **F15 — No new events / no merge** | Capture independence. Existing kinds only (`RepositoryPathAliasRemoved` / `Added`). No `MemoryMoved`. No auto-merge leftover → dest. |
| **F16 — Live vault** | Implement + plan **must not** unregister or rebind live leftover roots. Operator live split is out of band unless the owner asks. Tests: `tempfile::tempdir` only. |
| **F17 — Pins / crates** | No clap 5, no dotenvy bump, no new crates, workspace **0.1.1**. No camino in CLI. No contracts DTO. No SQL migration. |
| **F18 — Docs** | CAPABILITIES rebind-path + list-paths filters. WORKFLOWS leftover runbook (never `7d97` + `AI-Brains`). OPERATIONS: unregister/rebind ≠ move memories. Root CHANGELOG T259. help_ia CONTEXT inventory adds `rebind-path`. CLI-EXIT-CODES rows. |
| **F19 — Exit codes** | 0 = print-only / write / already-bound / empty filtered list-paths. 2 = usage (`--write` sans `--yes`; clap `--yes` sans `--write`; empty path after normalize; unknown `--format`). 1 = no owner / dest missing / vault. |
| **F20 — Tests** | New `crates/ai-brains-cli/tests/project_rebind_path.rs` (+ list-paths filter cases there or additive in `project_path_aliases.rs`). T254 / T240 / T258 suites stay green. |
| **F21 — Cross-model** | FEATURE / identity. After Phase-1 review clean, run read-only `codex-review`. |
| **F22 — Debt file** | `conductor/ISSUES.md` does **not** exist. Deferrals → `conductor/deferred.md`. |
| **F23 — PATH vs source** | Live PATH binary may lack T259 rebind **and** T258 adopt-path remediations until `cargo install`. Source at HEAD already emits adopt-path (`project.rs:823–824`). Phase 0 re-reads **source**, not PATH, before concluding a verb is missing. Tests/manual AC use `cargo run` / hermetic bin. |
| **F24 — Decline extras** | T257 warn/JSON; T258 `.env`; T260 ranking; T264 `--global` blender; T267 footer algorithm; T268 scan-roots rewrite; dest mint; bulk split; memory reclassify; silent switch; clap 5; live leftover mutate. |
| **F25 — Filter empty** | `--project` / `--shared-only` with zero rows: human `No path aliases match.` (not the T254 empty-register next-step). Exit **0**. JSON `paths: []`. |
| **F26 — `--to` required** | clap required `--to`. Missing `--to` is clap usage exit **2**. |

---

## 4. Acceptance criteria

| AC | Proof |
|----|-------|
| **AC1** | Hermetic: one project owns two registered paths, another owns one. `project list-paths --shared-only --format json` exits **0**; `paths` contains **only** the two shared-owner rows (ASC); unfiltered JSON keys still T254 F10 (`api_version`, `paths[]` with `project_id,label,alias,normalized_path,exists`). |
| **AC2** | Same fixture. `project list-paths --project <shared-uuid> --format json` returns exactly those two rows. `--project` unknown dest → exit **1**. |
| **AC3** | **Red today** (command missing). Hermetic: path registered to A; dest project B exists; pin one memory on A. `project rebind-path <path> --to <B> --format human` (no write flags) exits **0**. Stdout names A, B, and `memories stay` / `memories_moved`. Event count **unchanged**. Path owner still A. |
| **AC4** | **Red today.** Same fixture. `--format human --write` (no `--yes`) exits **2**. No events. Stderr mentions `--yes`. |
| **AC5** | **Red today.** Same fixture. `--format human --write --yes` exits **0**. `find_path_alias_owner` is B. A’s `memory_count` **unchanged**. Event count **+2**. list-paths no longer shows path on A. |
| **AC6** | Hermetic already-bound (owner == dest). `--format human`: exit **0**, `Already bound`, no `Would rebind` / `Re-run with --write`. No events. `--format json`: `already_bound: true`, `written: false`, `from_project_id` == `to_project_id`, `memories_moved: false`. |
| **AC7** | Hermetic path **not** registered: `--format human --to <B>` exit **1**, no events, stderr mentions `register-path`. |
| **AC8** | Hermetic dest UUID/alias **not** in vault: exit **1**, no events, stderr says dest not found (does **not** mention `set-alias 7d97` / `AI-Brains`). |
| **AC9** | `--yes` without `--write` is clap usage (exit **2**). Missing `--to` is clap usage (exit **2**). |
| **AC10** | `--format json` print-only: one object, keys §5.1, `written: false`, `memories_moved: false`. Parses as JSON. |
| **AC11** | T254 `project_path_aliases` + T240 `project_identity_convergence` + T258 `project_adopt_path` stay green. `project.rs` / `context.rs` untouched (or comment-only — prefer untouched). |
| **AC12** | No contracts DTO; no pin bumps; no new crate; no SQL migration. |
| **AC13** | Docs: CAPABILITIES rebind + filters; WORKFLOWS leftover runbook (no `7d97`+`AI-Brains`); OPERATIONS honesty; CHANGELOG T259; CLI-EXIT-CODES; CONTEXT inventory includes `rebind-path`. |
| **AC14** | Manual (source bin, **do not mutate live leftover**): `cargo run -p ai-brains-cli -- project list-paths --shared-only --format human` lists the 11 leftover roots under `7d97a456` and does **not** list `C:\dev\ai-brains`. `rebind-path C:\dev\crawlx --to <any> --format human` is print-only (confirm live alias rows unchanged via a second `list-paths --format json` hash/count). |
| **AC15** | New T259 `--help` / human error strings do not contain `set-alias` + leftover UUID + `AI-Brains` together. |
| **AC16** | Hermetic: `--project` of a real project that owns **zero** paths, or `--shared-only` on a vault with no multi-root owner. `--format human` exits **0**, stdout contains `No path aliases match.` (not the T254 empty-register next-step). `--format json`: `paths: []`. Pins F25. |
| **AC17** | Same fixture as AC1 (one shared owner with two paths, one singleton). `project list-paths --project <shared-uuid> --shared-only --format json` returns **exactly** the two shared-owner rows. `--project <singleton-uuid> --shared-only --format json` returns `paths: []` (intersection). |
| **AC18** | CP unit: `rebind_path_alias(..., from, from)` returns `InvalidPayload`; no events appended. CLI already-bound (AC6) never reaches this helper. |

Test names (TDD). **Must fail red before F3 exists:** AC3–AC5 (command unknown → clap exit 2). After clap lands and before CP helper: AC3 print-only can go green; AC5 stays red until write.

- `list_paths__shared_only__multi_root_id_only`
- `list_paths__project_filter__only_that_owner`
- `list_paths__project_unknown__exit_1`
- `list_paths__filter_empty__no_match_exit_0`
- `list_paths__project_and_shared_only__intersection`
- `project_rebind_path__print_only__names_from_to_no_events`
- `project_rebind_path__write_without_yes__exit_2_no_events`
- `project_rebind_path__write_yes__rebinds_owner_memories_stay`
- `project_rebind_path__already_bound__exit_0_no_events`
- `project_rebind_path__no_owner__exit_1`
- `project_rebind_path__dest_missing__exit_1`
- `project_rebind_path__yes_without_write__clap_exit_2`
- `project_rebind_path__format_json__print_only_keys`
- `rebind_path_alias__appends_removed_then_added` (CP unit)
- `rebind_path_alias__from_eq_to__invalid_payload` (CP unit)

---

## 5. Design notes

### 5.1 Commands + JSON

```text
ai-brains project list-paths [--project <id|alias>] [--shared-only] [--format auto|human|json]
ai-brains project rebind-path <path> --to <id|alias> [--write] [--yes] [--format auto|human|json]
```

Human print-only rebind (SOOT):

```text
Would rebind path alias <normalized>
from: <from-uuid>
to:   <to-uuid>
Historical memories stay on <from-uuid> (memories_moved=false).
Nightly Phase 2 would stop walking this path for the from-project.
Re-run with --write --yes to apply.
```

Human already-bound:

```text
Already bound to <to-uuid>
No path events.
```

JSON (print-only and write):

```json
{
  "api_version": "1",
  "path": "<normalized>",
  "from_project_id": "<uuid>",
  "to_project_id": "<uuid>",
  "already_bound": false,
  "written": false,
  "memories_moved": false,
  "events_appended": 0
}
```

Write success: `written: true`, `events_appended: 2`. Already-bound / print-only: `events_appended: 0`. No-owner fail is stderr + exit 1 (not this object). `from_project_id` is always a UUID on this object — the no-owner path never emits JSON (F8 / AC7).

### 5.2 CP helper

```text
rebind_path_alias(writer, path, from, to)
  if from == to → Err(InvalidPayload)
  if path normalizes empty → Err(InvalidPayload)  // same as register/unregister
  Removed { project_id: from, normalized_path }
  Added   { project_id: to,   normalized_path }
  writer.append_events(&[removed, added])
```

CLI prechecks (dest exists, owner is `from`, `from != to`, path non-empty) **before** the helper. Projection refuse-steal (T254 F7) still applies if a raced other-owner Added exists — precheck + one tx makes that a test-only race.

### 5.3 Operator leftover runbook (docs only — not executed this pass)

For each leftover root the owner wants to detach (example `C:\dev\crawlx`):

1. `project list-paths --project 7d97a456-f2f4-43ea-1f13-211af684ad37 --format human`
2. In that repo: `ai-brains context` to mint/ensure a dest project + `.env` (not `--new-project` unless they want a fresh UUID; not adopt-path unless they then want daily Scope = dest).
3. `project rebind-path C:\dev\crawlx --to <dest-uuid> --format human` (print-only).
4. `--write --yes` when the print looks right.
5. Confirm `list-paths --project 7d97…` dropped the row; dest `whoami` path owner is dest.

Do **not** `set-alias 7d97a456 AI-Brains`. Do **not** rebind `C:\dev\ai-brains` off `3581317d` as part of leftover cleanup.

### 5.4 What path split does / does not fix

| After rebind of a leftover root | Fixed? |
|--------------------------------|--------|
| Nightly Phase 2 stops ingesting that root onto `7d97a456` | Yes |
| detect / whoami / adopt-path in that repo see dest | Yes (once dest owns the path) |
| Historical 18,028 pins / `--global` leftover-first | **No** — T260 / T264 |
| `project list` footer leftover-as-AI-Brains | **No** — T267 |

---

## 6. Non-goals

- Moving, classifying, or CE-wiping leftover memories
- Bulk / “split all leftover roots”
- Creating dest projects inside rebind
- Changing daily Scope / writing `.env` (T258)
- Changing identity-mismatch warn placement (T257)
- Changing `project list` footer algorithm (T267)
- Rewriting `scan-roots` suggested command (T268)
- `--global` isolation / recall ranking (T264 / T260)
- clap 5, new crates, contracts DTO, SQL migration
- Live leftover mutate, `cargo install`, `set-alias 7d97 AI-Brains`
- Reopening T240 F2 / T255 declines

---

## 7. Verification plan

TDD: failing tests in §4 names first (Red commit allowed). Green = clap + filters + print-only, then CP helper write.

Targeted on go: `cargo nextest run -p ai-brains-cli --test project_rebind_path --test project_path_aliases --test project_adopt_path --test project_identity_convergence` ; `cargo clippy -p ai-brains-cli -p ai-brains-control-plane --all-targets -- -D warnings`.

Full workspace gate only at implement finalize (not a plan gate).

Manual AC14 uses source bin; does **not** `--write --yes` against live leftover.

---

## 8. Risk

| Risk | Mitigation |
|------|------------|
| Operators think rebind moves 18k memories | F5 / JSON `memories_moved: false` / docs honesty |
| Half-split (unregister, forget register) | F6 one tx; do not document two-step as happy path |
| Accidental live leftover mutate | F16; AC14 print-only; stop-before |
| Footer still suggests leftover as AI-Brains | Decline to T267; F1 + AC15 on new surfaces |
| `--global` still unusable | Honest §5.4; T260/T264 |
| Dest missing for all 11 roots | F9 + runbook `context` first |
| `project.rs` hotspot growth | F12 new module |
| PATH-behind | F23; hermetic/source bin |
| register_path `process::exit` | F6 never call it |

---

## 9. Deferred absorb / decline

Entire `conductor/deferred.md` scanned 2026-08-17 (post-P12 through T258 closeout).

| Row / leftover | Disposition |
|----------------|-------------|
| Post-T255 audit: leftover `7d97a456` 18,028 / many `C:\dev\*` | **Absorb** F2–F6 / AC1–AC5 / AC14 |
| T258 closeout: leftover split unchanged; T267 list footer unchanged | **Affirm** — this track is the split; footer stays T267 |
| T240 F14 remainder `project use <uuid>` | **Decline** — general use is not path rebind |
| T240 F2 no silent switch | **Affirm** F11 — no `.env` |
| T267 F3 list footer path-owner-without-label / never `7d97`+`AI-Brains` | **Decline (point T267)** — algorithm stays there. **Partial:** F1/AC15 no that pair on T259 surfaces |
| T267 harness self-next / whoami remediations | **Decline** — T258 shipped adopt-path remediations; harness is T267 |
| T264 `--global` blender | **Decline** — path split does not label Safety lines |
| T260 symbol stubs / ranking | **Decline** |
| T257 warn/JSON interleave | **Decline** |
| T268 scan-roots cwd / re-register suggestion | **Decline** — scan `suggested` still names current owner (T254). T268 owns parent/`--root` |
| T255 declined bag (doctor 16th, persist probe, product `.cmd`, embed sleep) | **Stay closed** |
| T254 F12 softs (auto TTY hermetic, helper wiring, F16 no pin assert, scan count vs log, concurrent F21, T233-F44) | **Decline** — not leftover split |
| T256 F18 PATH leaky help | **Decline** — operator `cargo install`; F23 notes PATH-behind |
| Packaging / R-CI-BRANCH / MSI | **Decline** — not this track |
| Closed/strikethrough rows (T187–T258 ships) | **Stay closed** |
| last-PR Cursor (#171 T258) | **N/A** — issue comments, reviews, and inline review comments all empty. Open PRs Dependabot only. No leftover to mint. Series cap is T271; no T272. |
| Memory reclassify leftover pins by path provenance | **Soft residual** (§11). Fits T259 leftover but is a later importer if ever — not a new placeholder. |

---

## 10. Implement order (on go)

1. Phase 0 re-verify live commands + deferred rescan + clap pin.
2. Red: `project_rebind_path` tests (AC3–AC5 must fail unknown-command).
3. list-paths `--project` / `--shared-only` (AC1–AC2 / AC16–AC17).
4. `RebindPath` clap + `project_rebind.rs` print-only (AC3/AC4/AC6–AC10).
5. CP `rebind_path_alias` + `--write --yes` (AC5 / AC18).
6. Docs + CONTEXT inventory (AC13/AC15).
7. Review loop + FEATURE `codex-review` + full gate.

---

## 11. Soft residuals

| Residual | Why not DoD |
|----------|-------------|
| Reclassify leftover memories onto dest by path/provenance | Needs importer + compensating memory events; F5 declines |
| Exclude leftover from `--global` default | T260/T264 |
| `project list` footer algorithm | T267 F3 |
| Suggest dest via `detect` of the *path* (not cwd) | Implicit dest is dangerous; `--to` required |
| Bulk `--all-shared` | F10 one path |
| Dest mint `--create` | F9; `context` exists |
| Unify `project.rs` copy of `resolve_project_ref` | Soft — T259 only promotes the `project_paths` helper to `pub(crate)` (F12). The `project.rs` duplicate stays (hotspot freeze). |
| Atomic tmp+rename / two-event crash mid-tx | Store `append_events` is already one tx |
| PATH `cargo install` | F23 operator |

---

## 12. Touch map

| Path | Change |
|------|--------|
| `crates/ai-brains-cli/src/main.rs` | `ListPaths` flags; `RebindPath` + dispatch |
| `crates/ai-brains-cli/src/commands/mod.rs` | `project_rebind` |
| `crates/ai-brains-cli/src/commands/project_paths.rs` | `--project` / `--shared-only`; `pub(crate) resolve_project_ref` (F12 hard) |
| `crates/ai-brains-cli/src/commands/project_rebind.rs` | **New** |
| `crates/ai-brains-control-plane/src/grants.rs` (+ lib export) | `rebind_path_alias` |
| `crates/ai-brains-cli/tests/project_rebind_path.rs` | **New** hermetics |
| `crates/ai-brains-cli/tests/project_path_aliases.rs` | Additive filter cases *or* keep filters in new file |
| `Docs/CAPABILITIES.md` / `WORKFLOWS.md` / `OPERATIONS.md` / `CLI-EXIT-CODES.md` | Filters + rebind + leftover runbook |
| `CHANGELOG.md` | T259 |
| `conductor/conductor.md` / `deferred.md` / this folder | Registry + absorb notes |

Do **not** touch: `project.rs` (footer/hotspot), `context.rs`, `project_adopt.rs` (except if a shared format helper is already public — prefer copy the 6-line `use_json_output`), contracts, migrations, live `.env`, live path aliases.

---

## 13. AI fold-in disposition (2026-08-17)

Inputs: `opencode-review.md` (HEAD `e46a2e1`), `agy-review.md` (wrote against `049064d`). No `grok-review.md` / `claude-review.md` / `codex-plan-review.md`. Review files **not** edited.

Both reviews: **no Blockers.** OpenCode one Major (schema null). AGY none. Deferred scan and last-PR Cursor (#171 empty) reaffirmed by both; no leftover to mint.

| ID | Finding | Disposition | Where |
|----|---------|-------------|--------|
| **OC-M1** | §5.1 `from_project_id` `"<uuid-or-null-if-no-owner>"` is unreachable; F8/AC7 are stderr+exit 1 | **Agree hard** | §5.1 pin `"<uuid>"`; JSON never emitted on no-owner |
| **OC-m2** | Vault totals 1-off vs live (35,521 / 595 / 566 at review) | **Agree** | §2.1 fold-in re-snapshot (leftover **18,028** still exact; `3581317d` **2,753**; `441837f6` **595**; global pinned **35,561**). Totals are not load-bearing. Phase 0 re-counts. |
| **OC-m3** | “PATH-behind T258” can be read as adopt-path missing from source | **Agree** | F23 + §2.1 PATH vs source. Source remediations `project.rs:823–824`. |
| **OC-O4** | F2 intersection (`--project` + `--shared-only`) has no AC/test | **Agree** | **AC17** + `list_paths__project_and_shared_only__intersection` |
| **OC-O5** | F25 empty-filter has a named test but no AC row | **Agree** | **AC16** |
| **AGY-m1** | CP `rebind_path_alias` should reject `from == to` | **Agree** | F6 + §5.2 + **AC18** + `rebind_path_alias__from_eq_to__invalid_payload`. Matches existing empty-path `InvalidPayload` on register/unregister. CLI F7 still short-circuits. |
| **AGY-O1** | Promote `resolve_project_ref` to `pub(crate)` | **Agree** (pin hard) | F12 **must** promote; do not duplicate. `project.rs` copy stays (hotspot freeze). |
| Deferred / last-PR | Both reviews affirm §9 | **Already covered** | No new placeholder. `#171` comments/reviews/inline empty. |

### Pins locked by fold-in

1. **§5.1 / AC10:** `from_project_id` is `"<uuid>"`. No-owner never produces this object.
2. **F6 / AC18:** CP helper `from == to` → `InvalidPayload`, no events.
3. **AC16 / F25:** empty filter human `No path aliases match.` / JSON `paths: []` / exit 0.
4. **AC17 / F2:** `--project` + `--shared-only` is intersection.
5. **F12:** `project_paths::resolve_project_ref` is `pub(crate)`; rebind reuses it.
6. **F23 / §2.1:** Phase 0 inspects **source** for T258 remediations; PATH-behind is the installed binary only.
7. **§2.1 totals:** leftover 18,028 / 11 roots / 17 paths / 7 owners stay load-bearing. Other memory totals drift and are re-checked at Phase 0.
