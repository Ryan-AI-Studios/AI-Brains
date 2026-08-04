All checks pass. Writing the final review.

## Verdict: PASS

## Scope Reviewed

Branch `feat/t204-cli-help-ia` working tree (read-only), cross-model re-review after Codex R1 FAIL. Files: `crates/ai-brains-cli/src/main.rs`, `crates/ai-brains-cli/src/help_ia.rs` (new), `crates/ai-brains-cli/tests/cli_help_ia.rs` (new), `CONTRIBUTING.md`, `Docs/CAPABILITIES.md`, `CHANGELOG.md`, `conductor/conductor.md`, `conductor/deferred.md`, `conductor/tracks/README-T197-T204-CLI-UX.md`. Compared against `spec.md` (F1ΓÇôF36, AC1ΓÇôAC12, ┬º5/┬º12) and `plan.md` (phases A0ΓÇôD5). No files or Git state modified.

## Requirement and DoD Matrix

| AC/F | Status | Evidence |
|------|--------|----------|
| AC1/F7 (group labels) | Met | `help_ia.rs:8-15` ΓÇö Setup/Daily/Operator/Governed/Dangerous/Harness |
| AC2/F4 (dangerous marker ΓëÑ1) | Met | 10 `[dangerous]` markers across main.rs |
| AC3 (CONTRIBUTING groups + CLI-EXIT-CODES) | Met | `CONTRIBUTING.md` diff adds groups pointer, exit-codes link pre-existing |
| AC4 (automated string assert) | Met | `help_ia.rs` unit tests + `cli_help_ia.rs` hermetic tests |
| AC5 (no rename) | Met | `known_commands__still_parse_via_help` test; no argv changes in diff |
| AC6/F9 (progressive project-id) | Met | `Query`, `Progressive`, `Expand` after_help all include `--project-id`/`AI_BRAINS_PROJECT_ID` (main.rs:433,748,756 region) |
| AC7/F31 (Daily before Harness) | Met | `display_order` bands applied; `long_help__daily_commands_before_harness_ingest` test |
| AC8/F8/M4 (consolidated OutputFormat table) | Met | `Docs/CAPABILITIES.md` new table |
| AC9 (CHANGELOG) | Met | Changed section entry |
| AC10 (soft, -h shorter) | Met | `short_help__tip_without_full_group_wall` test |
| AC11 (stop-session in Daily) | Met | `help_ia.rs:11,45-59` |
| AC12/F33/L7/L8 (subcommand markers) | Met | All 10 F33 surfaces marked incl. `daemon update` (fixed) |
| F31 (Graph both cfg arms same order) | Met | Both arms `display_order = 57` (main.rs:357,364) |
| F36/L2 (Global options heading, soft) | Met | `help_heading = "Global options"` on 4 global args |
| F20 (no unwrap/high findings) | Met | `help_ia.rs` clean, no unwrap |

## Findings (P0ΓÇôP3)

None. Both Codex R1 findings are fixed and verified against source:

- **P1-002 (daemon update unmarked)** ΓÇö Fixed. `DaemonCommands::Update` doc comment now `/// [dangerous] Stop daemon, install updated binaries, then restart...` (main.rs:1429). Appendix text updated to `daemon install|uninstall|update` (help_ia.rs:14, CAPABILITIES.md:68). Test suite's F33 surface table now includes `daemon update` (cli_help_ia.rs:81) and would fail without the fix.
- **P2-001 (invalid migrate --confirm invocation)** ΓÇö Fixed. Appendix now reads `migrate governed --confirm` (help_ia.rs:14). The `Migrate` variant's `after_help` example already used the correct `migrate governed ... --confirm` form (main.rs:386) and is unchanged. `Docs/CAPABILITIES.md:68` also corrected. Repo-wide search found no remaining `migrate --confirm` (invalid shorthand) references.

## Completeness Sweep

- All 10 F33 dangerous surfaces present and consistent between code and test assertions (forget, erasure, erasure wipe, retention apply, vault encrypt, vault rotate-datakey, migrate governed, daemon install/uninstall/update).
- `display_order` applied across all 37 top-level commands; bands match F31 (Setup 0, Daily 10-17, Operator 20-27, Governed 30-38, Dangerous 40-41, Harness 50-58); both `Graph` cfg arms identical.
- Progressive/expand/parent-query after_help all carry project-id ceremony (F9/AC6), at both parent (`Query`) and child (`GovernedQueryCommands::Progressive/Expand`) levels.
- Docs closeout: CONTRIBUTING, CAPABILITIES (groups + consolidated table), CHANGELOG all updated; no OutputFormat default flip introduced.
- No argv/command renames; presentation-only diff confirmed by inspection (doc comments, `#[command(...)]` attributes, new `help_ia.rs` const module, test file only).

## Wiring and Regression Review

- `mod help_ia;` correctly declared and consumed via `after_long_help`/`after_help` on `Cli` (main.rs:7,56-57).
- Test count matches recorded gate: `cli_help_ia.rs` (7 tests) + `exit_contract.rs` (11 tests) = 18, matching `review.md`'s "18 passed" note.
- No changes to command dispatch/match arms beyond doc-comment/attribute additions ΓÇö `DaemonCommands::Update` handling and all other match arms untouched, so no behavior regression risk.

## Verification Evidence

- Source-level verification performed via direct file reads and `git diff`/grep (all cargo/clippy/nextest invocations were denied by the sandbox's Bash permission policy in this read-only reviewer session, so I could not independently re-execute the gate).
- Relied on `review.md`'s recorded local gate (`cargo clippy -p ai-brains-cli --all-targets -- -D warnings` clean; `cargo nextest run -p ai-brains-cli --test cli_help_ia --test exit_contract` 18 passed) plus static confirmation that the test file exercises the exact P1-002 fix (daemon update surface in the F33 array).
- Recommend the implementer/CI confirm the full workspace `nextest`/`clippy` pass stated in this task's brief ("Full nextest 2020 pass; clippy workspace clean") is captured in `review.md` before PR merge, since I could not re-run it here.

## Deferred Candidates

None proposed. No new P3s found.

## Completion Decision

**PASS.** Both Codex R1 findings (P1-002, P2-001) are correctly fixed and consistent across all touch points (main.rs, help_ia.rs, CAPABILITIES.md, tests). No regressions found; DoD (AC1ΓÇôAC12, F1ΓÇôF36) is satisfied by static review. Remaining closeout steps (registry status ΓåÆ Completed, ledger transaction, PR creation/merge) are process items pending in `conductor.md`/`plan.md` D3ΓÇôD5, consistent with this series' established pattern (T197ΓÇôT203) where those are done via a separate chore commit after squash-merge ΓÇö not blocking for this code review.
