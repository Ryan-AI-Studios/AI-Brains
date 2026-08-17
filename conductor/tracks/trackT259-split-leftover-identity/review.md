# T259 review log — Split leftover identity

**Status:** Phase-1 R1/R1b clean; Codex CX1 **product PASS** (P2-1 help_ia CONTEXT = false positive / §2.4); full `dev-check.ps1` **PASS** (3013 + 1 skip)
**FEATURE TX:** `1dfea3fc-3109-4788-81f1-23b02aa24b1a`
**Reviewer:** implementer (Grok)

## Scope

`project list-paths --project` / `--shared-only` inventory filters plus
`project rebind-path` print-only remediator. `--write --yes` appends
`RepositoryPathAliasRemoved` + `RepositoryPathAliasAdded` in one store
transaction. Historical memories stay. Live leftover roots not rebound.
Live `.env` not written.

## DoD / AC matrix (R1)

| AC / DoD | Status | Evidence |
|----------|--------|----------|
| AC1 `--shared-only` multi-root only; T254 F10 keys | **met** | `list_paths__shared_only__multi_root_id_only` PASS |
| AC2 `--project` filter + unknown dest exit 1 | **met** | `list_paths__project_filter__only_that_owner` + `list_paths__project_unknown__exit_1` PASS |
| AC3 print-only names from/to, no events | **met** | `project_rebind_path__print_only__names_from_to_no_events` PASS |
| AC4 `--write` sans `--yes` exit 2 | **met** | `…write_without_yes__exit_2_no_events` PASS |
| AC5 `--write --yes` owner dest; memories stay; +2 events | **met** | `…write_yes__rebinds_owner_memories_stay` PASS |
| AC6 already-bound | **met** | `…already_bound__exit_0_no_events` PASS (human + JSON) |
| AC7 no owner exit 1, names register-path | **met** | `…no_owner__exit_1` PASS |
| AC8 dest missing exit 1; no leftover-as-AI-Brains | **met** | `…dest_missing__exit_1` PASS |
| AC9 clap `--yes` sans `--write` / missing `--to` | **met** | `…yes_without_write__clap_exit_2` PASS |
| AC10 print-only JSON keys; `from_project_id` uuid | **met** | `…format_json__print_only_keys` PASS |
| AC11 T254 / T240 / T258 green; `project.rs` / `context.rs` untouched | **met** | 49 targeted CLI tests PASS; `git diff` no `project.rs` / `context.rs` |
| AC12 no DTO / pins / crates / SQL | **met** | `Cargo.lock` / workspace crates unchanged; no migration |
| AC13 docs | **met** | CAPABILITIES filters + rebind + CONTEXT inventory; WORKFLOWS leftover runbook; OPERATIONS honesty; CLI-EXIT-CODES; CHANGELOG T259 |
| AC14 manual print-only | **met** | `--shared-only --format human` lists 11 leftover `7d97a456` roots, not `C:\dev\ai-brains`. Print-only rebind of `C:\dev\crawlx` → `3581317d`; list-paths JSON count 17/17 unchanged |
| AC15 no leftover `set-alias` + `AI-Brains` on new surfaces | **met** | `…help__no_leftover_as_ai_brains` + dest-missing combined text |
| AC16 empty filter | **met** | `list_paths__filter_empty__no_match_exit_0` PASS |
| AC17 `--project` + `--shared-only` intersection | **met** | `list_paths__project_and_shared_only__intersection` PASS |
| AC18 CP `from == to` InvalidPayload | **met** | `rebind_path_alias__from_eq_to__invalid_payload` PASS |
| F5 memories stay | **met** | AC5 memory_count unchanged; JSON `memories_moved: false` |
| F6 one tx | **met** | `rebind_path_alias` → `append_events(&[Removed, Added])`; `…appends_removed_then_added` PASS |
| F11 no `.env` | **met** | rebind never opens `.env` |
| F12 new module; `resolve_project_ref` `pub(crate)` | **met** | `project_rebind.rs`; helper reused |
| F16 no live leftover mutate | **met** | AC14 print-only; `LIST_PATHS_UNCHANGED=true` |
| T267 footer untouched | **met** | `project.rs` not in diff |

## Targeted gates

| Command | Result |
|---------|--------|
| `cargo fmt` then `cargo fmt --check` | pending (fmt ran during implement) |
| `cargo clippy -p ai-brains-cli -p ai-brains-control-plane --all-targets -- -D warnings` | **PASS** |
| `cargo nextest run -p ai-brains-cli --test project_rebind_path --test project_path_aliases --test project_adopt_path --test project_identity_convergence` | **49 passed** |
| `cargo nextest run -p ai-brains-control-plane --test grant_isolation` | **14 passed** (incl. AC18 + append order) |

## Findings

| ID | Severity | Description | Status | Evidence |
|----|----------|-------------|--------|----------|
| R1-P3-1 | low | No-owner / dest-missing `--format json` never emits frozen object; generic `COMMAND_FAILED` | **deferred** | F8/AC7/AC8 are stderr+exit 1 by pin; same CLI fallback as T258 |
| R1-P3-2 | low | `project.rs` still has a private `resolve_project_ref` copy | **deferred** | Spec §11 soft residual; F12 hotspot freeze |

### R1 (implementer) — no P0–P2.

### R1b (independent explore, 2026-08-17)

Verdict: **PASS WITH DEFERRED P3**. P0–P2 none. Same two P3s as above. No new findings. Easy P3s: none.

## Notes (not findings)

- AC7/AC8 eprint then `Err` so `handle_cli_result` also emits `COMMAND_FAILED` JSON. Spec requires stderr + exit 1, not the frozen rebind object.
- CONTEXT inventory lives in `Docs/CAPABILITIES.md` §16 (help_ia Daily line is still the T204 group string `project`, not per-verb).
- Soft residuals: leftover memory reclassify; `--global` leftover-first (T260/T264); list footer (T267); dest mint; bulk `--all`; PATH `cargo install`.

## Completeness sweep (R1)

No TODO/FIXME/stub in `project_rebind.rs` / `rebind_path_alias`. Wired:
`ProjectCommands::RebindPath` → `project_rebind::run` → CP helper.
`ListPaths` now carries `project` + `shared_only`. Docs match. Live leftover
not written.

## Codex CX1

Product **PASS**. Codex P2-1 (`help_ia.rs` must name `rebind-path`) is
**false positive**: spec §2.4 “Additive CAPABILITIES CONTEXT string only.
Root groups unchanged.” CONTEXT inventory is CAPABILITIES §16 (updated).

## Full gate

| Command | Result |
|---------|--------|
| `cargo fmt --check` | PASS |
| `cargo clippy --workspace --all-targets -- -D warnings` | PASS (via `dev-check.ps1` + `ledgerful verify --scope fast`) |
| `cargo nextest run --workspace` | **3013 passed**, 1 skipped |
| `cargo deny check` | PASS |
| `cargo audit` | PASS (19 allowed warnings) |
| `.\scripts\dev-check.ps1` | **[SUCCESS] CI Gate passed!** |
| `ledgerful verify --scope fast` | **Verification passed** |
