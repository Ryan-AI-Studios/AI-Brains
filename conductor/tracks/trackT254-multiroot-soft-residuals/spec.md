# T254 — Multi-root soft residuals (T233+)

- **Track ID:** T254-MultiRootSoftResiduals
- **Status:** 📋 **Planning** (plan-only until **go**)
- **Category:** FEATURE / OPS
- **Owner:** Grok
- **Source:** T233 soft residual — O2 `list-paths`, F31 `unregister-path`, F15 `--from-scan`, T233-F44 route `method`/`path_pattern`, plus honesty leftovers `bridge_roots_failed` and F21 projection steal
- **Depends on:** T233 ✅ **Completed** 2026-08-11 PR #142 `38cdcc2`
- **Blocks / feeds:** Honest path-alias operator surface; T255 stays separate (T229 router residuals)
- **Absorbs:** deferred.md “T233 soft list-paths/unregister/from-scan”; T233 review P3 O2 / F31 / F15; `bridge_roots` failed-count under-sum; F21 projection refuse-steal
- **Not absorbed (DoD):** T233-F44 SymbolRecord / `ledgerful endpoints` ingest; concurrent atomic F21 (multi-operator); T229/T255 router residuals; T240 F13 detect `--json` / F14 `project use`; T167 importer; T253 nightly Claude/Codex; clap 5 / pin bumps; daemon/DTO/T180 rewrite; auto-register / auto `.env` write
- **Research date:** 2026-08-15 (source HEAD `012b37c` at plan; fold-in against `dc16d3a`)
- **AI fold-in:** 2026-08-15 `C:\dev\AI-review.md` **T254** AI1 + AI2. No Highs. **Agree hard:** AI2 drop camino (not a cli/path dep; F26 forbids adding it); AI2 F10 label+alias via in-memory `list_projects` join (not N+1, not new SQL); AI2 CLI-EXIT-CODES in F28/AC15; AI2 owner-scoped Removed DELETE + AC19; AI2 new CLI returns `Err` (no `process::exit` in `project_paths.rs`); AI2 scan-roots one-shot HashMap lookup; AI2 unreadable/cap unit on helper. **Agree:** AI1 F24/F7/F16/F21 bounds; AI2 Removed payload ≡ Added; AI2 `--format auto` reuse whoami `IsTerminal`; AI2 split hermetics; rebuild tests stay in `path_aliases.rs`. **Decline:** AI2 new `list_path_alias_details` SQL helper; AI2 `path_is_same_or_inside` for scan (exact normalize match); refactor `register_path` `process::exit` (F8 copy-only). Disposition **§10**.
- **Ledger:** plan-only until go. Planning TX `3d30be0a-be88-4cf8-8d44-8d1316bb939a` (DOCS). Fold-in TX `391a8aae-e859-4fe2-81b5-d35affe8d190` (DOCS). Implement go starts a new FEATURE TX: `ledgerful ledger start T254-multiroot-soft-residuals --category FEATURE`
- **Isolation:** Do **not** reopen T233 Phase 2 / 0163 SQL delete / ingest cap. Do **not** write repo-local `.ledgerful` or hooks (C7). Do **not** delete or update raw events. Do **not** print or commit `AI_BRAINS_KEY`. Do **not** auto-register test projects (T233 F33).

---

## 1. Objective

1. **Operators can see every registered filesystem root.** `project list` still shows at most one path (lexicographically first). `project list-paths` lists **all** `repository_path_alias_projection` rows.
2. **A wrong `register-path` is correctable.** `project unregister-path` appends a compensating `RepositoryPathAliasRemoved` event. Projection deletes the row. The path is free for another project. Ingested `MemoryPinned` symbols stay (history).
3. **Bootstrap is a dry-run, not a writer.** `project scan-roots [path]` discovers immediate children (and the scan root itself) that contain `.ledgerful` and prints suggested `register-path` commands. It never writes events and never invents a project binding.
4. **Honesty leftovers from T233 ship if cheap.** Nightly logs `bridge_roots_failed`. Projection `ON CONFLICT` refuses to steal a path from another project. Route metadata via `ledgerful endpoints` stays **declined**.

---

## 2. Live baseline (re-scan 2026-08-15)

### 2.1 Dogfood (this machine)

| Signal | Observation |
|--------|-------------|
| HEAD | `012b37c` — T253 Completed. T254 still Placeholder until this plan. Ahead of `origin/main` by 6. Tree CLEAN at plan start. |
| `project list` path column | **All `—`.** Zero rows in `repository_path_alias_projection`. Labels `C:\`, `C:\dev\stl`, `C:\dev\ai-brains`, `C:\llm` are **set-alias names**, not filesystem path aliases (T233 F13). |
| `project whoami` | `path_alias_project_id: null`. `detect_project_id` = `3581317d-…` (git slug `AI-Brains`). Effective Scope is `test-alias` (`441837f6-…`) from project `.env`. |
| `C:\dev` children with `.ledgerful` | **17:** AI-Brains, coordinator, CrawlX, Dedupe, Degoo, Family, GIMP, Helping-Hands, homebrew-tap, KinLedger, ledgerful, ledgerful-action, ledgerful-frontend, ledgerful-web, Orca, STL, Wondermaker |
| `.changeguard` only (not a scan marker) | CozoDB-redux, Design, LexBase, Newton, Photo — leftover T142 state; **do not** treat as scan hits |
| Nightly Phase 2 today | Zero aliases → no-op + `register-path` hint (AC7). Multi-root bridge never runs on this vault until someone registers. |
| T253 on PATH | Installed `ai-brains` still reports Claude/Codex `install_ready=false` / preflight `(pending)` — **PATH-behind** vs source `012b37c`. **Out of T254 scope.** |
| Identity / doctor | Unrelated: `test-alias` Scope, ledgerful doctor ambient warns (legacy `.changeguard`, sig-pin, timings). Do not “fix” here. |

### 2.2 Why these residuals still matter

| Residual | Why it is still a product hole |
|----------|--------------------------------|
| O2 `list-paths` | Store `list_path_aliases` exists; no CLI. `project list` shows **first** path only (`ORDER BY path ASC LIMIT 1`). Multi-root projects are invisible. |
| F31 `unregister-path` | Conflict message literally says `unregister-path is soft residual F31`. Hermetic AC13 **asserts that string**. There is no `RepositoryPathAliasRemoved`. A mistaken register is permanent (F21 blocks steal; nothing frees the path). |
| F15 from-scan | 17 `.ledgerful` roots on this machine, 0 aliases. Operators have no discovery command. T233 F33 forbids auto-register. |
| T233-F44 routes | 0163 `symbols --json` has no `method`/`path_pattern`. `ledgerful endpoints --json` is a **separate** command. Reopening `SymbolRecord` would reopen T233 F36/F44. |
| `bridge_roots_failed` | `roots_ok` increments only on symbol `Ok`. Symbol `Err` increments nothing. Logged triple under-sums (`total ≠ ok + skipped + failed`). |
| F21 steal | CLI pre-check is non-atomic. Projection `ON CONFLICT DO UPDATE SET project_id = excluded.project_id` **steals** if a raced `Added` is applied or rebuilt. |

### 2.3 Code truth

| Item | Location | Notes |
|------|----------|-------|
| CLI commands | `main.rs` `ProjectCommands` | `List`, `Resolve`, `Detect`, `SetAlias`, `RegisterPath`, `Whoami` — no list/unregister/scan |
| Register | `commands/project.rs` `register_path` | F21 pre-check + CP `register_path_alias`. Conflict stderr includes **F31 residual string** |
| Dispatch | `main.rs` ~4044 | Exhaustive `ProjectCommands` match — new variants required |
| Store list | `query_store.rs` `list_path_aliases` | `ORDER BY normalized_path ASC` — Phase 2 SoT |
| Store owner | `find_path_alias_owner` | Used by F21 + T240 detect |
| List column | `list_projects_detail` | Scalar subquery **first** path only — **keep** |
| Event | `RepositoryPathAliasAdded` only | No Removed kind / payload / KnownPayload arm |
| Projection | `repository_identity.rs` | UPSERT steal on `normalized_path` PK |
| Replay | `replay.rs` | Already `DELETE FROM repository_path_alias_projection` then replay |
| CP write | `control-plane/grants.rs` `register_path_alias` | Append-only Added. Compensating pair = new `unregister_path_alias` |
| Legacy import | `legacy_import.rs` | Added is `REASON_OUT_OF_MATRIX` skip — Removed must join that arm |
| Nightly Phase 2 | `nightly.rs` `run_phase2_multi_root_bridge` | Logs `bridge_roots_total/ok/skipped`; **no failed** |
| Symbols | `symbol_bridge.rs` `SymbolRecord` | No method/path_pattern (T233-F44). `symbol_content` non-route |
| 0163 | `ledgerful symbols --pub --json` | schemaVersion 1: name/kind/path/line/isPublic/qualifiedName |
| Endpoints | `ledgerful endpoints --json` | Separate catalog (`-m/--method`, `-p/--path`) — **not** symbols |
| Tests | `tests/project_register_path.rs` | AC1/AC2/AC13; AC13 locks F31 residual copy |
| Store tests | `store/tests/path_aliases.rs` | list sort + owner exclusivity |
| Hotspot | `project.rs` | **Rank 1** (score 3.502). New CLI **must not** grow this file |
| Contracts / daemon | none | No path-alias DTO or daemon op |
| Help IA | `CAPABILITIES` CONTEXT | `list/resolve/detect/whoami/set-alias/register-path` |

### 2.4 Dependency / standards research (2026-08-15)

| Pin | Workspace / lock | Ecosystem | Action |
|-----|------------------|-----------|--------|
| `clap` | workspace **4.5** / lock **4.6.1** | crates.io 4.6.x | **No bump** — add `ProjectCommands` variants only |
| `serde` / `serde_json` | **1.0** / lock **1.0.150** | 1.0.151 train | **No bump** |
| `dirs` | **6.0** / lock **6.0.0** | 6.0.0 | **No bump** |
| `uuid` | workspace **1.13** / lock **1.23.1** | 1.23.x | **No bump** |
| `camino` | lock **1.2.5** via **desktop/tauri only** | — | **Do not add** to `ai-brains-cli` / `ai-brains-path`. Display via `std::path` + `to_string_lossy` (same as `project.rs` / `nightly.rs`) |
| `rusqlite` | workspace **0.39.0** SQLCipher | 0.40.x | **No bump** |
| rustc / edition | **1.95.0** / **2024** | — | Unchanged |
| nextest | **0.9.140** | — | Unchanged |
| workspace version | **0.1.1** | — | **No bump** |
| New crates | — | — | **Zero** |
| `ledgerful` CLI | PATH (0163 + `endpoints`) | runtime contract | Scan uses filesystem `.ledgerful` marker only — **no** spawn required for scan-roots |

---

## 3. Frozen decisions (F0–F39)

| ID | Decision |
|----|----------|
| **F0 — Go gate** | Plan-only until user **go**. Planning is DOCS. Implement starts a FEATURE TX. |
| **F1 — list-paths** | Ship `ai-brains project list-paths`. Reads `QueryStore::list_path_aliases`. Does not invent paths. |
| **F2 — unregister event** | Ship compensating `RepositoryPathAliasRemoved { project_id, normalized_path }` — **same field shape as Added**. Event sourcing: **never** delete/update the Added event. Projection `DELETE FROM repository_path_alias_projection WHERE normalized_path = ? AND project_id = ?` (owner-scoped; foreign/out-of-order Removed is a no-op). |
| **F3 — scan-roots** | Ship `ai-brains project scan-roots [path]` as the T233 F15 `--from-scan` surface. **Dry-run only.** Never appends events. Never writes `.env`. |
| **F4 — Decline routes** | Do **not** restore `method`/`path_pattern` on `SymbolRecord`. Do **not** spawn `ledgerful endpoints` in nightly. Do **not** reopen SQL inventory. Document `ledgerful endpoints` as the operator catalog. |
| **F5 — Decline concurrent F21** | Check-then-write stays CLI-ok (T233). Do **not** add a multi-operator transaction. **Do** refuse-steal on projection apply (F7). |
| **F6 — bridge_roots_failed** | Increment on symbol-ingest `Err`. Log `bridge_roots_failed`. Missing roots stay `skipped`. MADR-fail + symbols-ok stays `ok` (existing). |
| **F7 — Refuse-steal** | Projection `ON CONFLICT(normalized_path) DO UPDATE SET project_id = excluded.project_id WHERE repository_path_alias_projection.project_id = excluded.project_id`. Other-owner `Added` is in the log but does **not** move the row. Unregister + register other still works (`Removed` then `Added`). |
| **F8 — No steal via CLI** | CLI F21 pre-check stays. Conflict stderr **drops** `unregister-path is soft residual F31` and names the real command: `ai-brains project unregister-path <path>`. |
| **F9 — list-paths format** | `--format auto` (default): TTY human table; pipe / `--format json` pretty JSON. Reuse whoami `std::io::IsTerminal` (`project.rs` ~743), not a new detector. Unknown format → `fail_usage` exit **2**. |
| **F10 — list-paths JSON** | Frozen keys: `{ "api_version": "1", "paths": [ { "project_id", "label", "alias", "normalized_path", "exists" } ] }`. Sort ASC by `normalized_path`. `exists` = `Path::new(normalized).exists()` (best-effort disk; not a vault claim). **Join:** `list_path_aliases` + one `list_projects` → `HashMap<ProjectId,(name,alias)>` + existing `display_label`. **No** N+1 `get_project_by_id`. **No** new SQL helper. |
| **F11 — list-paths empty** | Human: `No path aliases registered.` + `next: ai-brains project register-path <project_id\|alias> <path>` (stdout). Exit **0**. JSON: `paths: []`. |
| **F12 — list column freeze** | `project list` path column stays first-path-only. Do not expand it to all aliases. |
| **F13 — unregister CLI** | `ai-brains project unregister-path <path>`. Path is unique (F21). Optional `--project <id\|alias>`: if owner ≠ ref → exit **1**. Missing path → idempotent exit **0** (`Path alias '…' is not registered.`). |
| **F14 — unregister dry-run** | `--dry-run` prints the normalized path + owner (or not-registered) and **does not** append. T107 pattern. |
| **F15 — unregister normalize** | Same `normalize_for_location_compare` as register. Empty after normalize → usage exit **2**. Win + WSL forms that normalize equal unregister the same row. |
| **F16 — Symbols stay** | Unregister does **not** forget `MemoryPinned` / `ledgerful:symbol` rows for that root. History remains. Phase 2 simply stops walking the path. |
| **F17 — Not dangerous** | Unregister is reversible (`register-path` again). No `[dangerous]` clap marker. Not CE wipe / forget. |
| **F18 — No migration** | No new SQL migration. Table schema unchanged. Replay already truncates the projection. |
| **F19 — Event blast** | Update every exhaustive `Payload` / `EventKind` / `KnownPayload` arm (events crate, `legacy_import` skip arm, goldens/`event_kind_from_payload` if needed). R0 `Unknown` stays the forward-compat hatch. No daemon wire variant. |
| **F20 — scan marker** | A scan hit is a directory that contains a **`.ledgerful`** child. Do **not** treat `.changeguard` alone as a hit (T142 leftover). |
| **F21 — scan shape** | Default path = cwd. Include the scan root if it itself has `.ledgerful`. Include **immediate** child directories only (no deep recurse, no `node_modules` walk). Cap **200** children (warn + truncate if over). Skip unreadable dirs (warn + continue). Pure helper owns cap + unreadable arms — **unit-test the helper** (Windows ACL `read_dir` fail is rare in CI). Registered lookup: load `list_path_aliases` **once**, normalize each hit with `normalize_for_location_compare`, HashMap lookup. Do **not** call `find_path_alias_owner` per hit. Do **not** use `path_is_same_or_inside` (exact root match, not containment). |
| **F22 — scan output** | Human columns: `path` \| `registered_to` (`project_id` or `—`) \| `disk` (`ok`/`missing`) \| suggested `register-path` (placeholder `<project-id-or-alias>` when unregistered). JSON frozen: `{ api_version, scan_root, truncated, roots: [{ path, registered_project_id, exists, suggested }] }`. Exit **0**. |
| **F23 — scan never binds** | Do **not** guess project from git slug / label / `test-alias`. F33 stands. Operator copies the suggested command and fills the project ref. |
| **F24 — Hotspot** | `project.rs` is ledgerful hotspot **#1**. New list/unregister/scan live in `crates/ai-brains-cli/src/commands/project_paths.rs`. `project.rs` keeps `register_path` + identity (do not relocate T233/T240 tests). `mod.rs` adds the module. Dispatch in `main.rs` only. |
| **F25 — Capture independence** | list / unregister / scan / projection apply must not open models, embeddings, or graph. Scan is filesystem-only. |
| **F26 — Pins / crates** | No clap 5, no lock bumps, no new crates (including **camino**), workspace stays **0.1.1**. Paths: `std::path` + `to_string_lossy`. |
| **F27 — Contracts** | No `ai-brains-contracts` DTO. No daemon/HTTP path-alias op. PROTOCOL-COMPAT: additive EventKind only if that doc lists known kinds; otherwise CHANGELOG + CAPABILITIES. |
| **F28 — Help / docs** | CAPABILITIES path-alias row + CONTEXT line; OPERATIONS table (list-paths / unregister-path / scan-roots); WORKFLOWS triangle; root CHANGELOG; **CLI-EXIT-CODES** (F35 rows); `register-path` after_help; conflict copy. |
| **F29 — Tests** | Naming `function_or_feature__condition__expected_result`. Hermetic vault via existing `project_register_path` helpers (or sibling `project_path_aliases.rs`). Store units for Removed + refuse-steal + rebuild. No `unwrap`/`expect`/`panic` in production. `rstest` if parameterized. `TempEnv` if env. |
| **F30 — Cross-model** | New EventKind is FEATURE. After Phase-1 review clean, run read-only `codex-review`. |
| **F31 — Whoami / detect** | No extra T240 work. They already read the projection; unregister + list-paths fall out. |
| **F32 — Phase 2 loop** | Keep sort ASC, `AI_BRAINS_NIGHTLY_MAX_ROOTS`, missing-skip, per-root continue. Only add `bridge_roots_failed`. |
| **F33 — Decline extras** | Multi-pass merge-order residual; AC12 full nightly symbol-count dogfood as hard DoD (optional live after first register); T212 verbose name; T240 F13/F14; T255; T167. |
| **F34 — Preflight / doctor** | No new doctor check. Soft optional: OPERATIONS note that zero aliases ⇒ Phase 2 no-op (already true). |
| **F35 — Exit codes** | Conflict / owner mismatch → **1**. Usage (empty path, bad format, clap) → **2**. Success including empty list / not-registered unregister / scan → **0**. Document in `Docs/CLI-EXIT-CODES.md`. |
| **F36 — Debt file** | `conductor/ISSUES.md` does **not** exist. Deferrals go to `conductor/deferred.md`. |
| **F37 — New CLI errors** | `project_paths.rs` returns `Err` / `fail_usage` and lets `handle_cli_result` map exits. **No** `std::process::exit` in the new module. Do **not** refactor `register_path`'s existing `process::exit(1)` (F8 is copy-only on hotspot #1). |
| **F38 — No new SQL** | Reuse `list_path_aliases` + `list_projects` + `find_path_alias_owner`. Decline `list_path_alias_details`. |
| **F39 — Events test** | `event_kind_from_payload.rs` does **not** cover Added today. Add a **new** Removed (and Added) round-trip there — do not pretend an existing case exists to extend. |

---

## 4. Acceptance criteria

| ID | Criterion |
|----|-----------|
| **AC1** | `project list-paths` on a vault with 0 aliases: empty copy + `next: … register-path …`; exit 0 |
| **AC2** | After two `register-path` rows (same or different projects): list-paths returns **both**, ASC by normalized path; `project list` still shows only the first path per project |
| **AC3** | JSON `--format json` matches F10 keys; piped `auto` is JSON; TTY `auto` is human (no JSON object) |
| **AC4** | `unregister-path <path>` after register: list-paths no longer contains it; `find_path_alias_owner` is `None`; same path can be registered to another project (exit 0) |
| **AC5** | Unregister missing path: exit 0 + not-registered message; `--dry-run` never appends (event count unchanged) |
| **AC6** | Dual Win/WSL forms that normalize equal: unregister one form removes the row |
| **AC7** | Conflict stderr **must not** contain `soft residual F31`; **must** contain `unregister-path` as a real command. Update AC13 hermetic |
| **AC8** | Rebuild projections after Added + Removed → path absent; after Added + Removed + Added (same or other project) → final owner |
| **AC9** | Raced/other-owner `RepositoryPathAliasAdded` applied directly does **not** change owner (F7). CLI still exit 1 on conflict |
| **AC19** | Store: `Removed` whose `project_id` is **not** the current owner leaves the row unchanged (owner-scoped DELETE). Rebuild Added(A)+Removed(B)+… still owned by A |
| **AC10** | `scan-roots` in a temp tree with one `.ledgerful` child and one without: only the marked child (plus root if marked) listed; vault event count unchanged |
| **AC11** | `scan-roots` never registers; suggested line contains `register-path`; already-registered roots show `registered_project_id` |
| **AC12** | `.changeguard`-only child is **not** a hit |
| **AC13** | Nightly Phase 2: symbol `Err` increments `bridge_roots_failed` (unit or tracing assertion). `ok + skipped + failed` accounts for every considered root |
| **AC14** | No production `unwrap`/`expect`/`panic`. Capture path does not gain model/graph deps |
| **AC15** | Docs: CAPABILITIES / OPERATIONS / WORKFLOWS / CHANGELOG / CONTEXT / **CLI-EXIT-CODES**; help after_help |
| **AC16** | T233 AC1/AC2/AC13 still pass (AC13 copy updated per AC7). Phase 2 System32 independence untouched |
| **AC17** | Full gate green on go: fmt, clippy `-D warnings`, nextest workspace, deny, audit, `ledgerful verify --scope full` |
| **AC18** | Cross-model FEATURE review clean (or only deferred P3 within cap) |

---

## 5. Implementation plan

See [plan.md](./plan.md). Summary:

0. FEATURE ledger + `scan --impact` (project.rs, nightly.rs, grants.rs, payload.rs, event_kind.rs, repository_identity.rs)
1. Red → Green: `list-paths` CLI + hermetics (store already exists)
2. Red → Green: `RepositoryPathAliasRemoved` + owner-scoped DELETE + refuse-steal + CP `unregister_path_alias` + CLI (`Err`, no `process::exit`) + F21 copy
3. Red → Green: `scan-roots` dry-run
4. `bridge_roots_failed`
5. Docs
6. Review (internal → cross-model) / gate / optional live: register `C:\dev\AI-Brains` after list-paths exists (not hard DoD)

---

## 6. Risks

| Risk | Mitigation |
|------|------------|
| EventKind blast (missed match arm) | Compiler + `legacy_import` skip arm + events tests; R0 Unknown is not a substitute for Known |
| Growing hotspot `project.rs` | F24 sibling module |
| Projection refuse-steal vs rebuild honesty | Event remains in the log; CAPABILITIES one-liner: CLI F21 is the operator gate; steal-UPSERT is gone |
| Foreign Removed deletes a live row | F2 owner-scoped DELETE + AC19 |
| Operators expect scan-roots to register | Empty/docs + “never writes” in `--help`; no `--apply` flag in v1 |
| Unregister mistaken for forget | F16 + OPERATIONS: symbols stay; only the Phase 2 walk list changes |
| Live vault still has 0 aliases | list-paths empty + scan-roots is the first honest operator path; do not auto-register the 17 roots |
| PATH-behind T253 pending | Out of scope; do not flip harness labels here |

---

## 7. Operator runbook (after ship)

```powershell
# See what is registered (all roots, not just project-list first path)
ai-brains project list-paths
ai-brains project list-paths --format json

# Discover local Ledgerful roots (dry-run; does not write)
ai-brains project scan-roots C:\dev

# Bind a root (existing T233)
ai-brains project register-path <id-or-alias> C:\dev\AI-Brains

# Correct a wrong bind
ai-brains project unregister-path --dry-run C:\dev\AI-Brains
ai-brains project unregister-path C:\dev\AI-Brains
ai-brains project register-path <other-id> C:\dev\AI-Brains
```

---

## 8. Open questions — closed at plan time

1. Ship unregister as a new EventKind vs decline? → **Ship** compensating Removed (F2). Soft-decline would leave F21 uncorrectable.
2. `unregister-path <project> <path>` vs path-only? → **Path-only** (unique). Optional `--project` safety (F13).
3. `--from-scan` flag on `register-path` vs new command? → **`project scan-roots`** (dry-run; F3). A write flag on register-path would imply binding.
4. Default scan root `C:\dev` vs cwd? → **cwd** (F21). `C:\dev` is this machine’s habit, not a product default.
5. `.changeguard` as scan marker? → **No** (F20).
6. Restore route fields via `ledgerful endpoints`? → **Decline** (F4).
7. Atomic F21? → **Decline** concurrent API; **ship** refuse-steal (F5/F7).
8. Forget symbols on unregister? → **No** (F16).
9. New doctor check? → **No** (F34).
10. Absorb T255 / T240 F13–F14 / T167? → **No** (F33).
11. New SQL `list_path_alias_details`? → **No** (F10 / F38) — `list_projects` HashMap join.
12. Add camino to CLI? → **No** (F26).
13. `path_is_same_or_inside` for scan? → **No** (F21) — normalize + exact lookup.
14. Refactor `register_path` `process::exit`? → **No** (F37).

---

## 9. Definition of Done

- [ ] User **go** + FEATURE TX
- [ ] F0–F39 + AC1–AC19
- [ ] AI fold-in (§10) honored
- [ ] Internal review clean; Codex FEATURE review; full gate
- [ ] conductor / deferred / pin updated
- [ ] No repo-local hook or `.ledgerful` writes from this track

---

## 10. AI fold-in (`C:\dev\AI-review.md` 2026-08-15)

AI1 affirms the plan (F24 hotspot, F21 bounds, F7 refuse-steal, F16 symbols stay, dry-run scan, auto format, `bridge_roots_failed`). No Highs. AI2 re-verified code vs plan; findings below.

| ID | Source | Verdict | Plan action |
|----|--------|---------|-------------|
| AI1 BS1 EventKind blast | AI1 | **Agree already F19** | Keep exhaustive arms + new Removed/Added round-trip (F39) |
| AI1 BS2 hotspot `project.rs` | AI1 | **Agree already F24** | Sibling `project_paths.rs`; F8 is one-line copy |
| AI1 BS3 scan bounds | AI1 | **Agree already F20/F21** | Depth 1 + cap 200; unit the helper (AI2 BS7) |
| AI1 BS4 refuse-steal SQL | AI1 | **Agree already F7** | Unchanged |
| AI1 BS5 symbols persist | AI1 | **Agree already F16** | Unchanged |
| AI1 opp 1–3 dry-run / auto / failed | AI1 | **Agree already F3/F9/F6** | Unchanged |
| AI2 BS1 camino not in CLI | AI2 | **Agree hard** | Drop “prefer camino”; `std::path` + `to_string_lossy` (F26) |
| AI2 BS2 F10 join under-specified | AI2 | **Agree problem; decline new SQL** | `list_projects` HashMap + `display_label` (F10 / F38) |
| AI2 BS3 CLI-EXIT-CODES missing | AI2 | **Agree hard** | F28 / AC15 / F35 |
| AI2 BS4 unconditional DELETE | AI2 | **Agree hard** | Owner-scoped DELETE + AC19 (F2) |
| AI2 BS5 `process::exit` split | AI2 | **Agree for new CLI** | F37: new module returns `Err`; do not touch register_path exit |
| AI2 BS6 per-hit owner lookup | AI2 | **Agree hard** | One-shot HashMap (F21) |
| AI2 BS7 ACL unreadable hermetic | AI2 | **Agree** | Unit on helper, not Windows ACL integration |
| AI2 opp 1 Removed ≡ Added | AI2 | **Agree** | F2 shape lock |
| AI2 opp 2 HashMap join | AI2 | **Agree** | Chosen over new SQL |
| AI2 opp 3 whoami `IsTerminal` | AI2 | **Agree** | F9 |
| AI2 opp 4 `path_is_same_or_inside` | AI2 | **Decline** | Exact normalize match only (F21) |
| AI2 opp 5 split hermetic files | AI2 | **Agree already F29** | Update AC13 in `project_register_path.rs`; new suite sibling |
| AI2 opp 6 rebuild in `path_aliases.rs` | AI2 | **Agree** | No new fixture file |

---

**Planning + fold-in 2026-08-15.** Plan-only until **go**.
