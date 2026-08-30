# T326 Review Log — status/graph PinnedCountFailed fail-open + 0.1.4

**Track:** T326-StatusPinnedCountFailOpen
**Category:** BUGFIX / UX / CHORE
**BUGFIX TX:** `986c12ef-91a3-4a7d-a2ae-49bf664d8739`
**Date:** 2026-08-30

## Internal review (R1)

| id | severity | description | source | files | required_fix | status | evidence |
|----|----------|-------------|--------|-------|--------------|--------|----------|
| AC1 | — | empty-graph COUNT fail is glance `error`, not `live`/`pinned=0` | spec | `status.rs` | `graph_section_from_gather` Err | verified_fixed | nextest PASS; red was `Ok pinned=Some(0) status=Some("live")` |
| AC2 | — | shared builder COUNT fail is `Err` (update + rebuild) | spec | `graph.rs` | `graph_health_from_gather` Err | verified_fixed | `--features graph -E "test(graph)"` PASS; red was `pinned_memories=0 status=live` |
| AC3 | — | T320 tables-missing envelope stay-green | spec | `status.rs` | no rewrite | verified_fixed | `status_envelope__graph_err__error_keeps_others` PASS |
| AC4 | — | gather PinnedCountFailed unit stay-green | spec | `graph_density.rs` | no gather change | verified_fixed | `gather_density_snapshot__pinned_table_missing__pinned_count_failed` PASS |
| AC5 | — | doctor 15-check + skip arm freeze | spec | `doctor.rs` | no edit | verified_fixed | `health_check_order_names__fixed_matrix` PASS; `git diff doctor.rs` empty |
| AC6 | — | T300 JSON health keys stay-green | spec | `graph.rs` | Ok-path keys frozen | verified_fixed | `rebuild_with_daemon_state__format_json__health_keys` PASS |
| AC7 | — | glance JSON COUNT fail omits status/nodes/edges | spec | `status.rs` | T320 F4 envelope | verified_fixed | AC1 serialize assert |
| AC8 | — | human `graph: error=` not `pinned=0` | spec | `status.rs` | existing error arm | verified_fixed | AC1 `format_status_graph_line` |
| AC9 | — | workspace / source `--version` 0.1.4 | spec | `Cargo.toml` | bump | verified_fixed | unit `cargo_pkg_version__workspace__is_0_1_4`; `cargo run -- --version` → `ai-brains 0.1.4`; PATH still `0.1.3` |
| AC10 | — | CHANGELOG `## [0.1.4]` + Fixed T326 | spec | `CHANGELOG.md` | cut Unreleased | verified_fixed | `check-version-banners.ps1` `CHANGELOG ## [0.1.4]: True` exit 0 |
| AC11 | — | Docs 0.1.4 + F36 honesty + CLI-EXIT-CODES | spec | Docs | headers + rows | verified_fixed | CAPABILITIES status/update/rebuild; CLI-EXIT-CODES exit **1** update/rebuild, glance **0** |
| AC12 | — | graph-off clippy/nextest; graph-on clippy + `-E "test(graph)"` | spec | — | F37 | verified_fixed | clippy `-D warnings` graph-off and `--features graph`; nextest 1723 graph-off; 119 graph-on filter |
| AC13 | — | isolation: doctor/project/sync/governed_common/contracts empty | spec | — | no grow | verified_fixed | `git diff` those paths empty; desktop still `0.1.2`; path-dep `0.1.0` |
| AC14 | — | no `--fail-on-pinned-count` | spec | clap | no flag | verified_fixed | `status --help` / `graph update --help` |
| AC15 | — | PATH honesty; no install; no live drop | spec | — | F20/F21 | verified_fixed | PATH `ai-brains 0.1.3`; source glance COUNT succeeds (`graph.status=sparse`, no `error`) |
| AC16 | — | `MIN_EDGE_NODE_RATIO == 0.50` in existing env-default test | spec | `graph_density.rs` | add assert | verified_fixed | `threshold_env__invalid_falls_back_to_defaults` PASS |
| R1 | — | Glance does not construct `pinned_memories: 0` | spec F1 | `status.rs` | Err const | verified_fixed | `PinnedCountFailed { .. } => Err(PINNED_COUNT_FAILED_MSG.into())` |
| R2 | — | Shared builder prefixes TablesMissing analog | spec F4 | `graph.rs` | `Failed to count pinned memories: ` | verified_fixed | callers `:520/:539/:769` via `graph_health_report` → helper |
| R3 | — | Doctor skip copy-not-share byte-identical | spec F3/F4 | `doctor.rs` | no const import | verified_fixed | grep skip string still literal |
| R4 | — | No production unwrap/expect/panic | AGENTS | product | `?` only | verified_fixed | helpers return `Result` |
| R5 | low-info | PATH until owner `cargo install --features graph` | spec §11 | — | — | deferred | → `deferred.md` |
| R6 | low-info | Doctor literal vs const drift risk | spec §11 | — | — | deferred | copy-not-share (F3) |
| R7 | low-info | No git tag `v0.1.4` | spec F28 | — | — | deferred | T185 public-tag path |
| R8 | low-info | COUNT fail rare on healthy vault | spec §11 | — | — | deferred | hermetic SoT |

**Cross-model:** Skipped (BUGFIX; not FEATURE/SECURITY/ARCHITECTURE gate).

**Medium+/critical:** none open.

## Completeness

- `graph_section_from_gather` + `graph_health_from_gather` extracted (F8 copy-not-share).
- `PINNED_COUNT_FAILED_MSG` on `graph_density.rs`.
- Workspace **0.1.4**; lock 22 workspace package version fields (44-line analog `#217`).
- No clap 5 / rusqlite bump / T307 / desktop bump / `ci.yml` edit / `cargo install`.
