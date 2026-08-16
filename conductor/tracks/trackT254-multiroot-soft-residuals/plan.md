# T254 Plan — Multi-root soft residuals (T233+)

**Status:** 📋 **Planning** (plan-only until **go**)  
**Spec:** [spec.md](./spec.md) F0–F36 / AC1–AC18  
**Category:** FEATURE / OPS  
**Ledger TX (planning):** `3d30be0a-be88-4cf8-8d44-8d1316bb939a` (DOCS)  
**Ledger TX (on go):** `ledgerful ledger start T254-multiroot-soft-residuals --category FEATURE --message "list-paths + unregister-path Removed event + scan-roots dry-run; refuse-steal; no endpoints ingest"`

---

## Preflight (plan time — 2026-08-15)

| Check | Result |
|-------|--------|
| HEAD / tree | `012b37c` CLEAN; ahead of origin/main by 6 |
| T253 | ✅ Completed in source. PATH `ai-brains` still shows Claude/Codex `install_ready=false` — **OOS** |
| `project list` paths | **All `—`** (zero `repository_path_alias_projection` rows) |
| `whoami` | `path_alias_project_id: null`; detect slug `3581317d`; Scope `test-alias` |
| `C:\dev` `.ledgerful` children | **17** (see spec §2.1) |
| `ledgerful symbols --help` | `--pub --json --limit --auto-index --path` still 0163 |
| `ledgerful endpoints --help` | Separate command (`-m/--method`, `-p/--path`, `--json`) — **not** absorbed |
| Store / CLI | `list_path_aliases` exists; no list/unregister/scan CLI; no Removed event |
| F21 copy | AC13 asserts `unregister-path is soft residual F31` |
| Nightly metrics | `bridge_roots_total/ok/skipped` only |
| Projection | UPSERT steal on `normalized_path` |
| Pins | clap lock **4.6.1**, serde_json **1.0.150**, dirs **6.0.0**, uuid **1.23.1**, camino **1.2.5** — **no bumps** |
| rustc / nextest / workspace | 1.95.0 / 0.9.140 / **0.1.1** |
| Hotspots | `project.rs` **#1** — new code in `project_paths.rs` |
| Ledger | 0 pending at scan; planning TX `3d30be0a` |
| `ISSUES.md` | **Does not exist** — debt is `deferred.md` |
| T255 / T167 / T240 F13–F14 | Peers — **not absorbed** |

---

## Absorbed deferred

| Item | Source | Plan action |
|------|--------|-------------|
| O2 `list-paths` CLI | T233 plan / review P3 | **DoD** F1 / F9–F12 / AC1–AC3 |
| F31 `unregister-path` | T233 F31 + AC13 copy | **DoD** F2 / F8 / F13–F17 / AC4–AC8 |
| F15 `--from-scan` | T233 Phase 5 skipped | **DoD** as `scan-roots` F3 / F20–F23 / AC10–AC12 |
| T233-F44 route method/path_pattern | T233 review + F44 | **Declined** F4 |
| F21 non-atomic CLI | T233 pin | **Acknowledge** F5; **ship** refuse-steal F7 / AC9 |
| `bridge_roots` failed under-sum | T233 review P3 | **DoD** F6 / AC13 |
| Multi-pass merge order under cap | T233 review P3 | **Declined** F33 |
| AC12 full nightly dogfood symbol count | T233 review P3 | **Declined** as hard DoD F33 |
| T212 path seed / verbose name | T233 deferred roll-in | **Declined** F33 |
| T240 F13 detect `--json` / F14 `project use` | T240 residual | **Declined** F33 |
| T229 F8–F12/F14 | T255 | **Not absorbed** |
| T253 PATH-behind pending | live harness status | **Not absorbed** |
| clap 5 / pin bumps | series | **Not absorbed** F26 |
| Auto-register from scan | F33 / T233 F33 | **Not absorbed** F23 |
| Forget symbols on unregister | tempting | **Not absorbed** F16 |
| New doctor check | optional | **Not absorbed** F34 |
| Daemon / contracts DTO | AGENTS contracts rule | **Not absorbed** F27 |

---

## Architecture SoT (files on go)

| Area | Path | Today | T254 change |
|------|------|-------|-------------|
| Event kind / payload | `ai-brains-events` `event_kind.rs` `payload.rs` `lib.rs` | Added only | **+ Removed** + KnownPayload + tag maps |
| Events tests | `event_kind_from_payload.rs` / goldens if any | Added not listed | Add Removed round-trip |
| CP | `grants.rs` `register_path_alias` | Append Added | **+ `unregister_path_alias`** |
| Legacy import | `legacy_import.rs` skip arm | Added skipped | Removed joins skip arm |
| Projection | `repository_identity.rs` | UPSERT steal | DELETE on Removed; refuse-steal UPDATE |
| Query | `query_store.rs` | list + find exist | **Reuse**; no new SQL unless tests need |
| Replay | `replay.rs` | Truncates table | No change |
| CLI enum | `main.rs` `ProjectCommands` | 6 variants | **+ ListPaths + UnregisterPath + ScanRoots** |
| CLI impl | **new** `commands/project_paths.rs` | — | list / unregister / scan |
| Register + F21 copy | `project.rs` `register_path` | F31 residual string | **F8 copy only** (keep fn here) |
| Nightly | `nightly.rs` Phase 2 | no failed | `bridge_roots_failed` |
| Hermetics | `tests/project_register_path.rs` + **new** `tests/project_path_aliases.rs` | AC13 F31 string | Update AC13; new AC suite |
| Store tests | `store/tests/path_aliases.rs` | list/find | Removed + refuse-steal + rebuild |
| Docs | CAPABILITIES / OPERATIONS / WORKFLOWS / CHANGELOG | T233 register-only | F28 |

---

## Phase 0 — Ledger + impact (on go)

- [ ] `ledgerful ledger status --compact` (0 pending)
- [ ] `ledgerful ledger start T254-multiroot-soft-residuals --category FEATURE --message "…"`
- [ ] `ledgerful scan --impact` (project.rs, nightly.rs, grants.rs, events, repository_identity.rs)
- [ ] Confirm `project.rs` still hotspot #1 — do not add the new commands there

## Phase 1 — Red → Green: `list-paths` (F1 / F9–F12 / AC1–AC3)

- [ ] Hermetic failing tests: empty vault; two aliases ASC; JSON keys; `project list` still first-path-only
- [ ] Clap `ListPaths { format }` default `auto`
- [ ] Implement in `project_paths.rs`; dispatch in `main.rs`
- [ ] Empty next-step mentions `register-path`
- [ ] Targeted: `cargo nextest run -p ai-brains-cli -E 'test(list_paths)'` ; clippy `-p ai-brains-cli`

## Phase 2 — Red → Green: unregister (F2 / F7 / F8 / F13–F19 / AC4–AC9 / AC16)

- [ ] Events: payload + EventKind + KnownPayload + exports; unit round-trip
- [ ] Store: apply DELETE; refuse-steal UPSERT; rebuild tests
- [ ] CP: `unregister_path_alias`; normalize empty → InvalidPayload
- [ ] CLI: `unregister-path <path> [--project] [--dry-run]`
- [ ] Missing → exit 0; owner mismatch → exit 1; dry-run no append
- [ ] Replace F21 residual string; update `register_path__conflict_other_project__exit_1`
- [ ] `legacy_import` skip arm includes Removed (compile-driven)
- [ ] Targeted: events + store + cli hermetics + `grant_isolation` if it touches aliases

## Phase 3 — Red → Green: `scan-roots` (F3 / F20–F23 / AC10–AC12)

- [ ] Hermetic temp tree: `.ledgerful` child hits; plain child misses; `.changeguard`-only misses; event count unchanged
- [ ] Already-registered child shows `registered_project_id`
- [ ] Cap 200 documented; unreadable child warn + continue
- [ ] `--help` says dry-run / never writes
- [ ] Targeted cli hermetics

## Phase 4 — `bridge_roots_failed` (F6 / AC13)

- [ ] Counter + tracing field
- [ ] Unit: one missing + one symbol-err + one ok → numbers add up
- [ ] Do not change MADR-fail-but-symbols-ok = ok

## Phase 5 — Docs (F28 / AC15)

- [ ] CAPABILITIES path-alias row + CONTEXT inventory
- [ ] OPERATIONS table + unregister / scan-roots + refuse-steal one-liner
- [ ] WORKFLOWS triangle
- [ ] CHANGELOG Unreleased Added
- [ ] `register-path` / new after_help
- [ ] conductor + deferred closeout **only at track complete** (not this planning commit)

## Phase 6 — Review / gate (F30 / AC14 / AC17 / AC18)

- [ ] Internal review vs spec until clean (mediums fixed or ≤3 justified)
- [ ] `codex-review` FEATURE
- [ ] Manual: `list-paths` on live vault (empty + next); `scan-roots C:\dev` (17-ish `.ledgerful`, 0 writes); optional `--dry-run` unregister of a temp register in a **temp vault** (do not mutate live aliases without user ask)
- [ ] Full gate: `cargo fmt --check ; cargo clippy --workspace --all-targets -- -D warnings ; cargo nextest run --workspace ; cargo deny check ; cargo audit ; ledgerful verify --scope full`
- [ ] Pin decisions; conductor Complete; deferred strike

---

## Frozen open questions (plan lock)

| # | Question | Decision |
|---|----------|----------|
| 1 | New EventKind vs decline unregister? | **Ship Removed** |
| 2 | Path-only vs project+path? | **Path-only** + optional `--project` |
| 3 | `--from-scan` on register-path? | **`scan-roots` command** |
| 4 | Default scan root? | **cwd** |
| 5 | `.changeguard` marker? | **No** |
| 6 | `ledgerful endpoints` ingest? | **Decline** |
| 7 | Atomic F21? | **Decline**; refuse-steal yes |
| 8 | Forget symbols? | **No** |
| 9 | New doctor check? | **No** |
| 10 | T255 / T240 F13–F14 / T167? | **No** |

---

## Manual evidence (fill on go)

| Step | Command | Expected | Result |
|------|---------|----------|--------|
| Empty list | `ai-brains project list-paths` | Empty + `next: … register-path` | |
| Scan | `ai-brains project scan-roots C:\dev` | `.ledgerful` children listed; no vault write | |
| Hermetic register/list/unregister | nextest `project_path_aliases` | AC1–AC12 green | |
| Live register | **only if user asks** | do not auto-bind the 17 roots | |

---

**Planning 2026-08-15.** Say **go** to start the FEATURE TX and TDD Phase 1.
