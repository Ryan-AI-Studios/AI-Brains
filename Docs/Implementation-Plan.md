# AI-Brains Implementation Plan v2 — Hardened, Decomposed, AI-Agent-Ready

> **Note:** As development progressed, some architectural and environment-specific decisions required deviating from this original plan. Please see [Deviations.md](./Deviations.md) for a complete list of changes, including graph decoupling and SQLite fallbacks.

**Product:** AI-Brains
**Document status:** Implementation Plan v2
**Primary platform:** Windows 11, PowerShell 7+
**Secondary platform:** Ubuntu / WSL after Windows reliability is proven
**Primary language:** Rust
**Implementation method:** TDD using conductor/tracks
**Core product rule:** Capture must be fast, durable, encrypted, and independent of every advanced memory feature.

---

## 0. What Changed in v2

This v2 hardens the prior implementation plan in the following ways:

1. **Further decomposes work into smaller AI-implementable slices.**
2. **Separates contracts from implementation more aggressively.**
3. **Adds explicit failure drills and chaos tests.**
4. **Adds adapter capability levels instead of pretending every harness can expose the same events.**
5. **Adds schema invariants and migration rules.**
6. **Adds projection rebuild strategy.**
7. **Adds hook safety rules to avoid breaking Claude/Gemini/Codex JSON protocols.**
8. **Adds a stricter secure-by-default posture.**
9. **Adds a conductor review checklist for each track.**
10. **Adds “red team” tests for forbidden behavior: repo writes, raw tool capture, hidden thinking capture, plaintext fallback, cloud leakage, and direct DB writes by adapters.**

The most important hardening change is this:

> **Capture is a tiny append-only command path. Everything smart is a projection or scheduled job.**

If an implementation agent violates that boundary, the product becomes fragile.

---

## 1. Non-Negotiable Architectural Constraints

These constraints are not preferences. Treat them as testable requirements.

| Constraint | Required Rule |
|---|---|
| Capture independence | Capture must work when graph, embeddings, local models, cloud models, and nightly jobs are offline. |
| Canonical SSOT | SQLCipher-backed append-only event log is the only source of truth. |
| CQRS | Commands append events. Queries read projections. Do not mix them. |
| Event immutability | Never update or delete raw events directly. Use compensating events. |
| No hidden thinking | No event, DTO, table, adapter, or command may store thinking/chain-of-thought fields. |
| No raw tool logs | Tool calls/actions are excluded by default and must not become canonical records. |
| No repo writes | AI-Brains must not write project-local files unless the user explicitly invokes a repo-write command. |
| Secure by default | Encrypted storage, local-only daemon, least privilege, privacy escalation on likely secrets. |
| Graceful degradation | Missing advanced services degrade to raw capture + lexical recall. |
| Commercial-safe stack | Default dependencies must be compatible with commercial product development. |

---

## 2. Design Pattern Summary

## 2.1 CQRS Boundaries

### Command side

Command side does only these things:

1. Validate request shape.
2. Normalize paths.
3. Capture bounded Git metadata.
4. Scan content for likely secrets.
5. Escalate privacy if needed.
6. Append immutable event.
7. Update minimal synchronous projections required for immediate recall.
8. Return structured result.

Command side does **not**:

- Summarize.
- Embed.
- Call graph DB.
- Call cloud model.
- Run RAPTOR.
- Run CRAG.
- Do broad search.
- Persist full diffs by default.

### Query side

Query side may:

- Search FTS.
- Read projections.
- Read graph projection if available.
- Rank memories.
- Assemble preflight.
- Return JSON/markdown.

Query side must not mutate canonical state except for optional access telemetry events if explicitly enabled.

---

## 2.2 Event Sourcing Rules

Use immutable events for all meaningful state changes.

Examples:

| User/Daemon action | Event |
|---|---|
| New install initialized | `SystemInitialized` |
| Recovery kit exported | `RecoveryKitCreated` |
| Project discovered | `ProjectRegistered` |
| Alias added | `ProjectAliasAdded` |
| Session started | `SessionStarted` |
| User prompt captured | `UserPromptRecorded` |
| Final AI response captured | `AssistantFinalRecorded` |
| Session completed | `SessionCompleted` |
| Session failed | `SessionFailed` |
| Memory pinned | `MemoryPinned` |
| Memory forgotten | `MemoryForgotten` |
| Secret detected | `PrivacyEscalated` |
| Nightly job started | `NightlyJobStarted` |
| Conflict detected | `ConflictDetected` |
| Recipe promoted | `RecipePromoted` |

Do not write code that “fixes” an old event. Append a new event.

---

## 2.3 Projection Rules

A projection is a read-optimized view derived from events.

Allowed projection examples:

- `project_projection`
- `session_projection`
- `turn_projection`
- `memory_projection`
- `conflict_projection`
- `recipe_projection`
- `job_projection`
- `fts_projection`
- `graph_projection`
- `embedding_projection`

Projection data may be updated, deleted, and rebuilt.

Projection data is **not** source of truth.

---

## 3. Capability-Based Harness Model

Do not assume every AI harness supports the same capture events. Each adapter must declare its capabilities.

## 3.1 Capability levels

| Level | Name | Description | Example |
|---:|---|---|---|
| 0 | Unsupported | No reliable capture. | Unknown tool |
| 1 | Manual import | User/AI pastes final response into AI-Brains. | Broken Antigravity session |
| 2 | Wrapper capture | AI-Brains launches command and captures bounded IO. | `ai-brains run -- ...` |
| 3 | Hook final capture | Harness exposes final assistant response. | Claude Code Stop hook |
| 4 | Hook prompt + final capture | Harness exposes both prompt and final. | Ideal |
| 5 | Full safe lifecycle | Session start, prompt, final, stop status, subagent stop, metadata. | Best case |

## 3.2 Adapter contract

Every adapter must report:

```json
{
  "harness": "claude",
  "capability_level": 5,
  "supports_session_start": true,
  "supports_user_prompt": true,
  "supports_assistant_final": true,
  "supports_session_stop": true,
  "supports_subagent_stop": true,
  "supports_model_detection": true,
  "stores_tool_calls": false,
  "stores_hidden_thinking": false
}
```

## 3.3 Adapter rule

Adapters parse and normalize. They do not write to the database.

Adapters return neutral events to capture core.

---

## 4. Folder Structure v2

Use this exact structure.

```text
ai-brains/
├── Cargo.toml
├── Cargo.lock
├── rust-toolchain.toml
├── rustfmt.toml
├── clippy.toml
├── deny.toml
├── .config/nextest.toml
├── README.md
├── LICENSE
├── SECURITY.md
├── docs/
│   ├── PRD.md
│   ├── IMPLEMENTATION_PLAN.md
│   ├── IMPLEMENTATION_PLAN_V2.md
│   ├── ARCHITECTURE.md
│   ├── SECURITY_MODEL.md
│   ├── RECOVERY_MODEL.md
│   ├── HOOK_ADAPTERS.md
│   ├── DEPENDENCY_LEDGER.md
│   ├── OPERATIONS.md
│   ├── TESTING_STRATEGY.md
│   ├── DECISIONS/
│   │   ├── ADR-0001-event-sourcing.md
│   │   ├── ADR-0002-cqrs.md
│   │   ├── ADR-0003-sqlcipher-canonical-store.md
│   │   ├── ADR-0004-no-repo-writes-by-default.md
│   │   ├── ADR-0005-hook-first-capture.md
│   │   ├── ADR-0006-graph-is-rebuildable-projection.md
│   │   ├── ADR-0007-privacy-inheritance.md
│   │   └── ADR-0008-daemon-single-writer.md
│   └── conductor/
│       ├── README.md
│       ├── track-board.md
│       ├── integration-contracts.md
│       ├── phase-gates.md
│       ├── failure-drills.md
│       ├── review-checklist.md
│       └── handoff-template.md
├── tracks/
│   ├── T00-foundation.md
│   ├── T01-core-domain.md
│   ├── T02-event-contracts.md
│   ├── T03-json-contracts.md
│   ├── T04-crypto-recovery.md
│   ├── T05-store-event-log.md
│   ├── T06-store-projections.md
│   ├── T07-path-normalization.md
│   ├── T08-git-metadata.md
│   ├── T09-security-scanner.md
│   ├── T10-capture-core.md
│   ├── T11-daemon-writer.md
│   ├── T12-cli-foundation.md
│   ├── T13-cli-capture.md
│   ├── T14-harness-claude.md
│   ├── T15-harness-gemini.md
│   ├── T16-harness-codex.md
│   ├── T17-harness-opencode-antigravity.md
│   ├── T18-retrieval-lexical.md
│   ├── T19-preflight-recall.md
│   ├── T20-graph-projection.md
│   ├── T21-model-providers.md
│   ├── T22-nightly-summaries.md
│   ├── T23-conflicts-recipes.md
│   ├── T24-raptor-crag.md
│   ├── T25-scheduler-backups.md
│   ├── T26-retention-forget.md
│   └── T27-e2e-hardening.md
├── crates/
│   ├── ai-brains-core/
│   ├── ai-brains-events/
│   ├── ai-brains-contracts/
│   ├── ai-brains-crypto/
│   ├── ai-brains-store/
│   ├── ai-brains-path/
│   ├── ai-brains-git/
│   ├── ai-brains-security/
│   ├── ai-brains-capture/
│   ├── ai-brains-adapters/
│   ├── ai-brains-daemon-api/
│   ├── ai-brainsd/
│   ├── ai-brains-cli/
│   ├── ai-brains-retrieval/
│   ├── ai-brains-graph/
│   ├── ai-brains-models/
│   ├── ai-brains-brain/
│   ├── ai-brains-scheduler/
│   └── ai-brains-testkit/
├── tests/
│   ├── e2e/
│   │   ├── cli_capture_smoke.rs
│   │   ├── daemon_concurrency.rs
│   │   ├── crash_recovery.rs
│   │   ├── hook_roundtrip_claude.rs
│   │   ├── hook_roundtrip_gemini.rs
│   │   ├── hook_roundtrip_codex.rs
│   │   ├── no_repo_pollution.rs
│   │   ├── preflight_context.rs
│   │   ├── privacy_no_cloud_leak.rs
│   │   ├── nightly_degrades_when_models_missing.rs
│   │   ├── graph_degraded_mode.rs
│   │   └── recovery_restore.rs
│   └── fixtures/
│       ├── harness_payloads/
│       ├── git_repos/
│       ├── path_cases/
│       ├── secrets/
│       ├── sample_vaults/
│       └── model_responses/
└── scripts/
    ├── dev-check.ps1
    ├── dev-check.sh
    ├── verify-licenses.ps1
    ├── install-hooks-sandbox.ps1
    ├── reset-test-env.ps1
    └── run-failure-drills.ps1
```

---

## 5. Crate Responsibilities

## 5.1 `ai-brains-core`

**Purpose:** Pure domain model.

**May depend on:** `serde`, `uuid`, `time`, `thiserror`
**Must not depend on:** database, CLI, daemon, HTTP, filesystem, Git, model providers

### Files

```text
src/
├── lib.rs
├── ids.rs
├── clock.rs
├── privacy.rs
├── project.rs
├── user.rs
├── device.rs
├── harness.rs
├── session.rs
├── turn.rs
├── memory.rs
├── conflict.rs
├── recipe.rs
├── status.rs
├── validation.rs
└── errors.rs
```

### Tests

```text
tests/id_serde_roundtrip.rs
tests/privacy_strictest_wins.rs
tests/session_status_transitions.rs
tests/no_thinking_role_exists.rs
tests/no_tool_call_role_exists.rs
tests/domain_validation_rejects_empty_content.rs
```

---

## 5.2 `ai-brains-events`

**Purpose:** Event definitions and event envelope.

**May depend on:** `ai-brains-core`, `serde`, `serde_json`, `sha2`, `time`, `uuid`
**Must not depend on:** store, CLI, daemon, adapters

### Files

```text
src/
├── lib.rs
├── envelope.rs
├── actor.rs
├── aggregate.rs
├── event_kind.rs
├── payload.rs
├── constructors.rs
├── hash.rs
├── version.rs
├── upcast.rs
└── errors.rs
```

### Required tests

```text
tests/envelope_hash_stable.rs
tests/event_roundtrip.rs
tests/event_upcast_unknown_future_event.rs
tests/no_mutating_event_payloads.rs
tests/no_hidden_thinking_event_kind.rs
tests/no_tool_call_event_kind.rs
tests/privacy_included_on_every_event.rs
```

---

## 5.3 `ai-brains-contracts`

**Purpose:** JSON DTOs for CLI, daemon, hooks, and tests.

**May depend on:** `ai-brains-core`, `serde`, `serde_json`
**Must not depend on:** store, adapters, daemon

### Files

```text
src/
├── lib.rs
├── response.rs
├── errors.rs
├── ingest.rs
├── hook.rs
├── preflight.rs
├── recall.rs
├── sessions.rs
├── projects.rs
├── memory.rs
├── backup.rs
├── doctor.rs
└── version.rs
```

### Required tests

```text
tests/api_response_shape.rs
tests/ingest_request_shape.rs
tests/preflight_response_shape.rs
tests/hook_response_has_no_stdout_noise_fields.rs
tests/contracts_are_backward_compatible.rs
```

---

## 5.4 `ai-brains-crypto`

**Purpose:** Data encryption key, DPAPI wrapper, passphrase recovery wrapper, SQLCipher key material.

### Files

```text
src/
├── lib.rs
├── data_key.rs
├── key_wrap.rs
├── dpapi.rs
├── passphrase.rs
├── recovery_kit.rs
├── sqlcipher.rs
├── zeroize.rs
├── test_support.rs
└── errors.rs
```

### Required tests

```text
tests/data_key_generated_randomly.rs
tests/passphrase_wrap_roundtrip.rs
tests/wrong_passphrase_fails.rs
tests/recovery_kit_restores_key.rs
tests/key_material_debug_redacted.rs
tests/sqlcipher_key_zeroized.rs
tests/windows_dpapi_roundtrip.rs
```

### Hardening tests

```text
tests/does_not_write_plaintext_key_to_disk.rs
tests/recovery_kit_missing_reports_actionable_error.rs
```

---

## 5.5 `ai-brains-store`

**Purpose:** SQLCipher event log, migrations, projections, FTS, backup/restore.

### Files

```text
src/
├── lib.rs
├── connection.rs
├── config.rs
├── pragmas.rs
├── migrations.rs
├── event_store.rs
├── transaction.rs
├── projections/
│   ├── mod.rs
│   ├── project.rs
│   ├── session.rs
│   ├── turn.rs
│   ├── memory.rs
│   ├── conflict.rs
│   ├── recipe.rs
│   └── job.rs
├── fts.rs
├── backup.rs
├── restore.rs
├── retention.rs
├── health.rs
└── errors.rs
```

### Migrations

```text
migrations/
├── 0001_event_log.sql
├── 0002_identity_projection.sql
├── 0003_project_projection.sql
├── 0004_session_projection.sql
├── 0005_turn_projection.sql
├── 0006_memory_projection.sql
├── 0007_conflict_recipe_projection.sql
├── 0008_fts.sql
├── 0009_jobs_backups.sql
└── 0010_retention.sql
```

### Required tests

```text
tests/sqlcipher_encrypted_vault.rs
tests/wrong_key_cannot_open.rs
tests/migrations_idempotent.rs
tests/event_append_atomic.rs
tests/event_log_is_append_only.rs
tests/projections_update_from_events.rs
tests/replay_rebuilds_projections.rs
tests/fts_indexes_memory.rs
tests/backup_restore_roundtrip.rs
tests/concurrent_read_single_writer.rs
tests/store_health_reports_projection_lag.rs
```

### Red-team tests

```text
tests/plaintext_fallback_forbidden.rs
tests/direct_event_update_forbidden.rs
tests_raw_tool_call_not_storable.rs
```

---

## 5.6 `ai-brains-path`

**Purpose:** Windows-first path normalization and alias resolution.

### Files

```text
src/
├── lib.rs
├── canonical.rs
├── display.rs
├── windows.rs
├── wsl.rs
├── unc.rs
├── symlink.rs
├── alias.rs
├── project_path.rs
└── errors.rs
```

### Required tests

```text
tests/windows_drive_case_normalized.rs
tests/forward_slashes_normalized.rs
tests/wsl_mnt_c_maps_to_windows.rs
tests/extended_length_prefix_normalized.rs
tests/unc_paths_preserved.rs
tests/symlink_resolution_best_effort.rs
tests/display_path_preserved.rs
tests/malformed_paths_return_error_not_panic.rs
```

---

## 5.7 `ai-brains-git`

**Purpose:** Bounded Git metadata.

### Files

```text
src/
├── lib.rs
├── discover.rs
├── branch.rs
├── commit.rs
├── remote.rs
├── status.rs
├── diffstat.rs
├── command.rs
└── errors.rs
```

### Required tests

```text
tests/git_root_discovered.rs
tests/non_git_directory_degrades.rs
tests/remote_url_hash_stable.rs
tests/branch_detected.rs
tests/commit_detected.rs
tests/dirty_status_detected.rs
tests/diffstat_does_not_capture_full_diff.rs
tests/untracked_filenames_bounded.rs
```

---

## 5.8 `ai-brains-security`

**Purpose:** Secret detection, privacy escalation, redaction.

### Files

```text
src/
├── lib.rs
├── scanner.rs
├── pattern.rs
├── finding.rs
├── escalation.rs
├── redaction.rs
├── policy.rs
└── errors.rs
```

### Required tests

```text
tests/detects_bearer_token.rs
tests/detects_private_key.rs
tests/detects_connection_string.rs
tests/clean_text_not_flagged.rs
tests/likely_secret_escalates_local_only.rs
tests/high_confidence_secret_escalates_sealed.rs
tests/redaction_preserves_readability.rs
tests/sealed_content_not_embeddable.rs
```

---

## 5.9 `ai-brains-capture`

**Purpose:** Capture command handling and conversion into events.

### Files

```text
src/
├── lib.rs
├── command_handler.rs
├── session_start.rs
├── user_prompt.rs
├── assistant_final.rs
├── session_stop.rs
├── malformed.rs
├── privacy.rs
├── metadata.rs
├── git_capture.rs
├── action_digest.rs
└── errors.rs
```

### Required tests

```text
tests/session_start_appends_event.rs
tests/user_prompt_appends_event.rs
tests/assistant_final_appends_event.rs
tests/session_failed_appends_status.rs
tests/session_aborted_appends_status.rs
tests/empty_prompt_rejected.rs
tests/empty_final_rejected_unless_status_only.rs
tests/thinking_field_ignored.rs
tests/tool_call_field_ignored.rs
tests/security_escalation_applied.rs
tests/git_metadata_attached_when_available.rs
tests/capture_does_not_require_models.rs
tests/capture_does_not_require_graph.rs
```

---

## 5.10 `ai-brains-adapters`

**Purpose:** Harness-specific parsing and hook config rendering.

### Files

```text
src/
├── lib.rs
├── capability.rs
├── adapter.rs
├── neutral_event.rs
├── hook_output.rs
├── claude.rs
├── gemini.rs
├── codex.rs
├── opencode.rs
├── antigravity.rs
├── ollama_claude.rs
├── wrapper.rs
├── install.rs
├── config_patch.rs
└── errors.rs
```

### Required tests

```text
tests/capability_report_claude.rs
tests/capability_report_gemini.rs
tests/capability_report_codex.rs
tests/claude_stop_payload_parsed.rs
tests/claude_subagent_payload_parsed.rs
tests/gemini_payload_parsed.rs
tests/codex_payload_parsed.rs
tests/ollama_claude_model_detected.rs
tests/opencode_degrades_cleanly.rs
tests/antigravity_manual_import.rs
tests/hook_install_user_scope_only.rs
tests/hook_config_patch_is_idempotent.rs
tests/malformed_hook_payload_returns_warning.rs
tests/hook_output_is_json_only.rs
```

---

## 5.11 `ai-brains-daemon-api`

**Purpose:** Shared local API definitions for CLI and daemon.

This crate prevents CLI and daemon from drifting.

### Files

```text
src/
├── lib.rs
├── routes.rs
├── client.rs
├── request.rs
├── response.rs
├── health.rs
├── ingest.rs
├── query.rs
└── errors.rs
```

### Required tests

```text
tests/route_paths_stable.rs
tests/client_serializes_ingest.rs
tests/health_response_shape.rs
tests/error_response_shape.rs
```

---

## 5.12 `ai-brainsd`

**Purpose:** Daemon process with single writer queue.

### Files

```text
src/
├── main.rs
├── app.rs
├── config.rs
├── state.rs
├── unlock.rs
├── writer_queue.rs
├── spool.rs
├── shutdown.rs
├── health.rs
├── routes/
│   ├── mod.rs
│   ├── health.rs
│   ├── ingest.rs
│   ├── preflight.rs
│   ├── recall.rs
│   ├── sessions.rs
│   └── projects.rs
└── errors.rs
```

### Required tests

```text
tests/daemon_health.rs
tests/ingest_route_appends_event.rs
tests/single_writer_serializes_parallel_ingest.rs
tests/daemon_does_not_bind_public_interface.rs
tests/locked_vault_refuses_write.rs
tests/queue_flushes_on_shutdown.rs
tests/spool_replays_after_restart.rs
tests/health_reports_degraded_graph.rs
tests/health_reports_degraded_models.rs
```

---

## 5.13 `ai-brains-cli`

**Purpose:** User and hook command-line interface.

### Files

```text
src/
├── main.rs
├── cli.rs
├── output.rs
├── stdin_json.rs
├── daemon_client.rs
├── direct_mode.rs
├── commands/
│   ├── mod.rs
│   ├── init.rs
│   ├── doctor.rs
│   ├── service.rs
│   ├── schedule.rs
│   ├── install_hooks.rs
│   ├── run.rs
│   ├── ingest.rs
│   ├── preflight.rs
│   ├── recall.rs
│   ├── sessions.rs
│   ├── active.rs
│   ├── conflicts.rs
│   ├── project.rs
│   ├── pin.rs
│   ├── forget.rs
│   ├── seal.rs
│   ├── redact.rs
│   ├── compact.rs
│   ├── export.rs
│   └── backup.rs
└── errors.rs
```

### Required tests

```text
tests/help_output.rs
tests/init_creates_global_user_root.rs
tests/init_does_not_touch_repo.rs
tests/doctor_reports_missing_recovery_kit.rs
tests/ingest_reads_json_stdin.rs
tests/hook_commands_no_stdout_noise.rs
tests/preflight_json_shape.rs
tests/recall_json_shape.rs
tests/forget_soft_deletes_by_default.rs
tests/service_commands_do_not_require_admin_by_default.rs
```

---

## 5.14 `ai-brains-retrieval`

**Purpose:** Read side ranking, recall, and preflight.

### Files

```text
src/
├── lib.rs
├── lexical.rs
├── vector.rs
├── graph.rs
├── rank.rs
├── preflight.rs
├── recall.rs
├── sessions.rs
├── active.rs
├── conflicts.rs
├── privacy_filter.rs
├── staleness.rs
├── word_budget.rs
└── errors.rs
```

### Required tests

```text
tests/lexical_search_returns_memory.rs
tests/project_memory_beats_global.rs
tests/pinned_memory_boosted.rs
tests/stale_memory_warned_and_downgraded.rs
tests/privacy_filter_excludes_never_inject.rs
tests/privacy_filter_excludes_sealed.rs
tests/preflight_under_1500_words.rs
tests/preflight_includes_active_sessions.rs
tests/recall_returns_sources.rs
tests/graph_unavailable_falls_back_to_fts.rs
```

---

## 5.15 `ai-brains-graph`

**Purpose:** Graph projection. Rebuildable. Not source of truth.

### Files

```text
src/
├── lib.rs
├── trait.rs
├── ladybug.rs
├── schema.rs
├── projector.rs
├── edge_builder.rs
├── queries.rs
├── rebuild.rs
├── health.rs
└── errors.rs
```

### Required tests

```text
tests/schema_initializes.rs
tests/projector_creates_project_session_memory_nodes.rs
tests/projector_creates_relationships.rs
tests/rebuild_is_idempotent.rs
tests/graph_projection_lag_reported.rs
tests/graph_unavailable_degrades.rs
tests/related_memory_query.rs
```

---

## 5.16 `ai-brains-models`

**Purpose:** Embeddings and summarizer provider abstraction.

### Files

```text
src/
├── lib.rs
├── provider.rs
├── ollama.rs
├── embedding.rs
├── summarizer.rs
├── request.rs
├── response.rs
├── privacy_gate.rs
├── fallback.rs
└── errors.rs
```

### Required tests

```text
tests/embedding_request_shape_ollama.rs
tests/summarizer_request_shape_ollama.rs
tests/local_provider_allows_local_only.rs
tests/cloud_provider_rejects_local_only.rs
tests/cloud_provider_rejects_sealed.rs
tests/model_failure_structured.rs
tests_missing_model_degrades.rs
```

---

## 5.17 `ai-brains-brain`

**Purpose:** Nightly intelligence pipeline.

### Files

```text
src/
├── lib.rs
├── nightly.rs
├── job_context.rs
├── project_selector.rs
├── session_summary.rs
├── daily_summary.rs
├── durable_lesson.rs
├── conflict_detection.rs
├── recipe_promotion.rs
├── embedding_job.rs
├── graph_job.rs
├── stale_check.rs
├── retention_job.rs
├── raptor/
│   ├── mod.rs
│   ├── candidate.rs
│   ├── cluster.rs
│   ├── synthesize.rs
│   ├── persist.rs
│   └── evaluate.rs
├── crag/
│   ├── mod.rs
│   ├── retrieve.rs
│   ├── verify.rs
│   └── verdict.rs
└── errors.rs
```

### Required tests

```text
tests/nightly_selects_projects_with_new_events.rs
tests/session_summary_created.rs
tests/daily_summary_created.rs
tests/durable_lesson_created_from_repeated_issue.rs
tests/conflict_detected_from_contradictory_sessions.rs
tests/recipe_promoted_from_windows_workaround.rs
tests/embedding_job_skips_sealed.rs
tests/graph_job_failure_does_not_fail_capture.rs
tests/raptor_clusters_related_memories.rs
tests/crag_downgrades_unsupported_summary.rs
tests/nightly_continues_when_model_unavailable.rs
tests/nightly_records_partial_failure.rs
```

---

## 5.18 `ai-brains-scheduler`

**Purpose:** Windows Task Scheduler, service install helpers, backup scheduling.

### Files

```text
src/
├── lib.rs
├── windows_task.rs
├── service.rs
├── backup.rs
├── command.rs
├── privilege.rs
└── errors.rs
```

### Required tests

```text
tests/render_schtasks_create_command.rs
tests/schedule_time_validated.rs
tests/no_admin_required_by_default.rs
tests/backup_path_under_user_root.rs
tests/service_install_command_rendered.rs
```

---

## 6. Store Schema v2

## 6.1 Event log

```sql
CREATE TABLE events (
    event_id TEXT PRIMARY KEY,
    schema_version INTEGER NOT NULL,
    aggregate_type TEXT NOT NULL,
    aggregate_id TEXT NOT NULL,
    event_type TEXT NOT NULL,
    occurred_at TEXT NOT NULL,
    actor_json TEXT NOT NULL,
    causation_id TEXT,
    correlation_id TEXT,
    privacy TEXT NOT NULL,
    payload_json TEXT NOT NULL,
    payload_hash TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_events_aggregate
ON events(aggregate_type, aggregate_id, occurred_at);

CREATE INDEX idx_events_type_time
ON events(event_type, occurred_at);

CREATE INDEX idx_events_correlation
ON events(correlation_id);

CREATE INDEX idx_events_privacy
ON events(privacy);
```

## 6.2 Event immutability trigger

Add a trigger that blocks updates to the event table.

```sql
CREATE TRIGGER prevent_event_update
BEFORE UPDATE ON events
BEGIN
    SELECT RAISE(ABORT, 'events are immutable');
END;
```

Do not create a trigger blocking delete if hard-delete retention will eventually purge events. Hard-delete must be done only by a dedicated retention path that writes an audit event first.

## 6.3 Session projection

```sql
CREATE TABLE session_projection (
    session_id TEXT PRIMARY KEY,
    project_id TEXT,
    user_id TEXT NOT NULL,
    device_id TEXT NOT NULL,
    harness TEXT NOT NULL,
    model_name TEXT,
    cwd_canonical TEXT,
    cwd_display TEXT,
    git_root TEXT,
    git_branch TEXT,
    git_commit_before TEXT,
    git_commit_after TEXT,
    status TEXT NOT NULL,
    started_at TEXT NOT NULL,
    ended_at TEXT,
    last_event_at TEXT NOT NULL,
    privacy TEXT NOT NULL,
    capture_basis TEXT NOT NULL
);
```

## 6.4 Turn projection

```sql
CREATE TABLE turn_projection (
    turn_id TEXT PRIMARY KEY,
    session_id TEXT NOT NULL,
    project_id TEXT,
    role TEXT NOT NULL CHECK (role IN ('user','assistant_final')),
    content_markdown TEXT NOT NULL,
    content_hash TEXT NOT NULL,
    privacy TEXT NOT NULL,
    basis TEXT NOT NULL,
    created_at TEXT NOT NULL,
    is_redacted INTEGER NOT NULL DEFAULT 0,
    is_forgotten INTEGER NOT NULL DEFAULT 0
);
```

## 6.5 Memory projection

```sql
CREATE TABLE memory_projection (
    memory_id TEXT PRIMARY KEY,
    project_id TEXT,
    user_id TEXT,
    kind TEXT NOT NULL,
    scope TEXT NOT NULL,
    privacy TEXT NOT NULL,
    confidence TEXT NOT NULL,
    basis TEXT NOT NULL,
    content_markdown TEXT NOT NULL,
    content_json TEXT,
    source_hash TEXT NOT NULL,
    created_at TEXT NOT NULL,
    last_accessed_at TEXT,
    expires_at TEXT,
    is_pinned INTEGER NOT NULL DEFAULT 0,
    is_redacted INTEGER NOT NULL DEFAULT 0,
    is_forgotten INTEGER NOT NULL DEFAULT 0
);
```

## 6.6 FTS projection

```sql
CREATE VIRTUAL TABLE memory_fts USING fts5(
    memory_id UNINDEXED,
    project_id UNINDEXED,
    content_markdown,
    content_json,
    tokenize='porter unicode61'
);
```

---

## 7. JSON Contracts

## 7.1 Standard response envelope

Every machine-readable CLI and daemon response must use this shape.

```json
{
  "ok": true,
  "data": {},
  "error": null,
  "warnings": []
}
```

Error response:

```json
{
  "ok": false,
  "data": null,
  "error": {
    "code": "VAULT_LOCKED",
    "message": "AI-Brains vault is locked.",
    "retryable": true,
    "suggested_action": "Run `ai-brains unlock` or start the daemon."
  },
  "warnings": []
}
```

## 7.2 Ingest final request

```json
{
  "schema_version": 1,
  "harness": "claude",
  "session_external_id": "optional-harness-session-id",
  "cwd": "C:\\dev\\LegisAI",
  "model_name": "glm-5.1:cloud",
  "assistant_final": "I implemented X.",
  "status": "completed",
  "metadata": {
    "pid": 1234,
    "hook": "Stop"
  }
}
```

## 7.3 Forbidden fields

If hook payloads contain these fields, adapters must ignore them and should not pass them into capture core:

```text
thinking
chain_of_thought
reasoning
tool_calls
tool_results
actions
trace
scratchpad
hidden
```

If these fields are present, adapters may record a warning count, not the content.

---

## 8. CLI Surface v2

> **Drift notice (T183 / T188 / T192 / 2026-08-02):** This §8 lists the **original design-plan** CLI surface. Some names were **renamed, never built, or only exist as DTOs** (for example `unlock`/`lock`, `install-hooks` as shown below). **Shipped since:** `recovery export` (T188), `doctor` (T192 read-only health).
> **Live operator CLI** = `ai-brains --help` and the track registry in `conductor/conductor.md`.
> **Adoption docs** = [`Docs/README.md`](README.md) · [`Docs/INSTALL.md`](INSTALL.md) · [`Docs/OPERATIONS.md`](OPERATIONS.md).
> Do not treat this section as a shipping checklist.

## 8.1 Setup

```powershell
ai-brains init
ai-brains doctor
ai-brains unlock
ai-brains lock
```

## 8.2 Daemon/service

```powershell
ai-brains service install
ai-brains service start
ai-brains service stop
ai-brains service status
```

## 8.3 Scheduling

```powershell
ai-brains schedule install --time 03:00
ai-brains schedule run-now
ai-brains schedule status
```

## 8.4 Harness integration

```powershell
ai-brains install-hooks --harness claude
ai-brains install-hooks --harness gemini
ai-brains install-hooks --harness codex
ai-brains uninstall-hooks --harness claude
ai-brains hook status
ai-brains hook doctor
```

## 8.5 Wrapper

```powershell
ai-brains run -- claude
ai-brains run -- gemini
ai-brains run -- codex
ai-brains run -- ollama launch claude --model glm-5.1:cloud -- --dangerously-skip-permissions
```

## 8.6 Ingest commands

These are primarily for hooks.

```powershell
ai-brains ingest-session-event --format json
ai-brains ingest-turn --format json
ai-brains ingest-final --format json
```

Input should come from stdin.

## 8.7 Query commands

```powershell
ai-brains preflight --cwd . --format json
ai-brains recall "what happened with the auth refactor?" --project . --format json
ai-brains context --project . --format json
ai-brains sessions --project . --since 7d --format json
ai-brains active --project . --format json
ai-brains conflicts --project . --format json
```

## 8.8 Memory commands

```powershell
ai-brains pin "text" --project .
ai-brains forget --id <memory-id>
ai-brains forget --id <memory-id> --hard
ai-brains seal --id <memory-id>
ai-brains redact --id <memory-id>
ai-brains compact --project . --since today
```

## 8.9 Project commands

```powershell
ai-brains project list
ai-brains project show .
ai-brains project alias add --project LegisAI --path C:\dev\LegisAI
ai-brains project policy set --project LegisAI --privacy local_only
ai-brains project exclude --path C:\dev\SecretProject
```

## 8.10 Backup/export

```powershell
ai-brains backup create
ai-brains backup restore --from <path>
ai-brains recovery export
ai-brains export --project .
```

---

## 9. Phase Plan v2

## Phase 0 — Foundation and Conductor

### Objective

Create repository, workspace, dependency gate, conductor docs, and track docs.

### Files

```text
Cargo.toml
rust-toolchain.toml
deny.toml
.config/nextest.toml
scripts/dev-check.ps1
docs/conductor/*
tracks/T00-foundation.md
```

### Tests/gates

```powershell
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo nextest run --workspace
cargo deny check
cargo audit
```

### Done when

- Empty workspace builds.
- Dependency/license gates run.
- Track board exists.
- Conductor checklist exists.

---

## Phase 1 — Domain and Event Contracts

### Objective

Create domain model and immutable event contract before implementation.

### Tracks

- T01 Core Domain
- T02 Event Contracts
- T03 JSON Contracts

### Tests first

```text
ai-brains-core/tests/no_thinking_role_exists.rs
ai-brains-events/tests/no_hidden_thinking_event_kind.rs
ai-brains-events/tests/envelope_hash_stable.rs
ai-brains-contracts/tests/api_response_shape.rs
```

### Done when

- No hidden thinking/tool event type exists.
- Events hash stably.
- JSON contracts are stable.
- Contracts do not depend on store/daemon/CLI.

---

## Phase 2 — Crypto and Store

### Objective

Create encrypted canonical event store.

### Tracks

- T04 Crypto Recovery
- T05 Store Event Log
- T06 Store Projections

### Tests first

```text
ai-brains-crypto/tests/recovery_kit_restores_key.rs
ai-brains-store/tests/sqlcipher_encrypted_vault.rs
ai-brains-store/tests/event_log_is_append_only.rs
ai-brains-store/tests/replay_rebuilds_projections.rs
```

### Done when

- Vault opens only with correct key.
- Recovery kit works.
- Events immutable.
- Projections can rebuild from events.

---

## Phase 3 — Identity: Path, Git, Security

### Objective

Normalize project identity and security classification.

### Tracks

- T07 Path Normalization
- T08 Git Metadata
- T09 Security Scanner

### Tests first

```text
ai-brains-path/tests/wsl_mnt_c_maps_to_windows.rs
ai-brains-git/tests/diffstat_does_not_capture_full_diff.rs
ai-brains-security/tests/high_confidence_secret_escalates_sealed.rs
```

### Done when

- Windows/WSL paths normalize.
- Git metadata captured without full diff.
- Secrets escalate privacy.

---

## Phase 4 — Capture Without Daemon

### Objective

CLI can capture session start, user prompt, final response, and failure/abort.

### Tracks

- T10 Capture Core
- T12 CLI Foundation
- T13 CLI Capture

### Tests first

```text
ai-brains-capture/tests/thinking_field_ignored.rs
ai-brains-capture/tests/tool_call_field_ignored.rs
ai-brains-cli/tests/ingest_reads_json_stdin.rs
tests/e2e/cli_capture_smoke.rs
```

### Done when

- Capture works without daemon.
- Capture works without graph/models.
- Hook-facing commands produce JSON only.
- Raw tool/thinking fields ignored.

---

## Phase 5 — Daemon and Concurrency

### Objective

Single writer queue supports multiple concurrent AI sessions.

### Tracks

- T11 Daemon Writer

### Tests first

```text
ai-brainsd/tests/single_writer_serializes_parallel_ingest.rs
ai-brainsd/tests/spool_replays_after_restart.rs
tests/e2e/daemon_concurrency.rs
tests/e2e/crash_recovery.rs
```

### Done when

- Three concurrent sessions append cleanly.
- Daemon local-only.
- Spool recovers after daemon restart.
- Crash leaves session interrupted, not lost.

---

## Phase 6 — Harness Adapters

### Objective

Hook/wrapper adapters for Claude, Gemini, Codex, OpenCode, Antigravity degraded mode, Ollama-launched Claude.

### Tracks

- T14 Harness Claude
- T15 Harness Gemini
- T16 Harness Codex
- T17 Harness OpenCode/Antigravity

### Tests first

```text
ai-brains-adapters/tests/capability_report_claude.rs
ai-brains-adapters/tests/claude_stop_payload_parsed.rs
ai-brains-adapters/tests/hook_output_is_json_only.rs
ai-brains-adapters/tests/antigravity_manual_import.rs
```

### Done when

- Capability levels reported.
- User-level hook install only.
- Hook config idempotent.
- Malformed payloads warn and degrade.

---

## Phase 7 — Retrieval and Preflight

### Objective

Recall and preflight context work before embeddings/graph.

### Tracks

- T18 Retrieval Lexical
- T19 Preflight Recall

### Tests first

```text
ai-brains-retrieval/tests/lexical_search_returns_memory.rs
ai-brains-retrieval/tests/preflight_under_1500_words.rs
ai-brains-retrieval/tests/privacy_filter_excludes_sealed.rs
tests/e2e/preflight_context.rs
```

### Done when

- FTS recall works.
- Preflight JSON has markdown brief.
- Privacy filters work.
- Pinned project memory ranks high.

---

## Phase 8 — Graph Projection

### Objective

Build rebuildable graph projection.

### Tracks

- T20 Graph Projection

### Tests first

```text
ai-brains-graph/tests/rebuild_is_idempotent.rs
ai-brains-graph/tests/graph_unavailable_degrades.rs
```

### Done when

- Graph projection rebuilds.
- Graph failures do not affect capture.
- Retrieval can fall back to FTS.

---

## Phase 9 — Model Providers and Nightly Base

### Objective

Add local model provider and basic nightly summaries.

### Tracks

- T21 Model Providers
- T22 Nightly Summaries

### Tests first

```text
ai-brains-models/tests/cloud_provider_rejects_local_only.rs
ai-brains-brain/tests/nightly_continues_when_model_unavailable.rs
ai-brains-brain/tests/session_summary_created.rs
```

### Done when

- Privacy gate works.
- Nightly degrades if model unavailable.
- Session/daily summaries created.

---

## Phase 10 — Conflicts, Recipes, RAPTOR, CRAG

### Objective

Add high-value memory intelligence.

### Tracks

- T23 Conflicts Recipes
- T24 RAPTOR CRAG

### Tests first

```text
ai-brains-brain/tests/conflict_detected_from_contradictory_sessions.rs
ai-brains-brain/tests/recipe_promoted_from_windows_workaround.rs
ai-brains-brain/tests/raptor_clusters_related_memories.rs
ai-brains-brain/tests/crag_downgrades_unsupported_summary.rs
```

### Done when

- Basic conflict detection works.
- Execution recipes promoted.
- RAPTOR level-1 clustering works.
- CRAG can downgrade unsupported generated summary.

---

## Phase 11 — Scheduler, Backup, Retention

### Objective

Install 3am heartbeat, backups, retention.

### Tracks

- T25 Scheduler Backups
- T26 Retention Forget

### Tests first

```text
ai-brains-scheduler/tests/render_schtasks_create_command.rs
tests/e2e/recovery_restore.rs
ai-brains-brain/tests/retention_soft_deletes_unaccessed_raw_turns.rs
```

### Done when

- Scheduled nightly command works.
- Backups restore with recovery kit.
- Raw turns expire after 90 days if unaccessed.
- Forget defaults to soft-delete.

---

## Phase 12 — E2E Hardening

### Objective

Make the system resilient in realistic conditions.

### Tracks

- T27 E2E Hardening

### Required E2E tests

```text
tests/e2e/cli_capture_smoke.rs
tests/e2e/daemon_concurrency.rs
tests/e2e/crash_recovery.rs
tests/e2e/hook_roundtrip_claude.rs
tests/e2e/hook_roundtrip_gemini.rs
tests/e2e/hook_roundtrip_codex.rs
tests/e2e/no_repo_pollution.rs
tests/e2e/preflight_context.rs
tests/e2e/privacy_no_cloud_leak.rs
tests/e2e/nightly_degrades_when_models_missing.rs
tests/e2e/graph_degraded_mode.rs
tests/e2e/recovery_restore.rs
```

### Done when

All E2E tests pass on Windows 11.

---

## 10. Failure Drills

Create `docs/conductor/failure-drills.md` and implement these as manual or automated drills.

## 10.1 Capture path failure drills

| Drill | Expected result |
|---|---|
| Kill daemon mid-ingest | CLI spools event or returns retryable error; no data corruption. |
| Kill process during session | Session marked `interrupted` on next health scan. |
| Graph unavailable | Capture succeeds; graph health degraded. |
| Ollama unavailable | Capture succeeds; nightly partial failure recorded. |
| Wrong vault key | No plaintext fallback; actionable error. |
| Malformed hook payload | Warning returned; harness not crashed. |
| Huge final response | Bounded write; warning if size exceeds policy. |
| Payload contains private key | Privacy escalated to sealed; no embedding/cloud. |

## 10.2 Recovery drills

| Drill | Expected result |
|---|---|
| Delete active vault, restore backup | Events recovered. |
| Restore on new test home | Recovery kit unlocks. |
| Missing recovery kit | Doctor reports high-priority warning. |
| Corrupt projection | Rebuild projection from event log. |
| Corrupt graph | Rebuild graph from event log/projections. |

## 10.3 Security drills

| Drill | Expected result |
|---|---|
| Try to install hooks in repo scope | Rejected unless explicit flag. |
| Try to write `.ai-brains` in project | Rejected by default. |
| Try to send `local_only` memory to cloud | Blocked. |
| Try to query sealed memory through preflight | Excluded. |
| Try to store tool calls | Ignored/not representable. |

---

## 11. Conductor Review Checklist

The Conductor must review each track with this checklist.

```markdown
## Track Review Checklist

- [ ] Tests were written before implementation or clearly identified as regression tests.
- [ ] Track touched only allowed files.
- [ ] No hidden thinking/tool-call storage added.
- [ ] No repo-local writes added.
- [ ] No cloud processing added without privacy gate.
- [ ] Capture path does not depend on graph/model/nightly.
- [ ] Errors are actionable and do not leak secrets.
- [ ] JSON output shape remains stable.
- [ ] Projection data is rebuildable.
- [ ] Event log remains append-only.
- [ ] License gate passes.
- [ ] Windows path behavior tested if relevant.
- [ ] Handoff notes updated.
```

---

## 12. Track Template v2

Every track file must follow this template.

```markdown
# Track Txx — Name

## Owner
Unassigned

## Status
Not Started

## Objective

## Scope

## Out of Scope

## Files Owned

## Files Allowed To Touch

## Files Forbidden To Touch

## Public Contracts Consumed

## Public Contracts Produced

## Required Tests First

## Implementation Steps

## Failure Modes To Handle

## Security Requirements

## Acceptance Criteria

## Commands To Run

## Handoff Notes
```

---

## 13. Implementation Agent Rules

These are direct instructions for AI implementation agents.

1. Do not skip phases.
2. Do not skip tests.
3. Do not loosen security to make a test pass.
4. Do not add repo writes.
5. Do not add raw tool-call storage.
6. Do not add hidden thinking storage.
7. Do not make capture depend on graph/model/nightly.
8. Do not use graph/vector/summaries as source of truth.
9. Do not call cloud models with `local_only`, `never_inject`, or `sealed` memory.
10. Do not add dependencies with forbidden licenses.
11. Do not use `unwrap`, `expect`, `panic`, `todo`, or `unimplemented` in production code.
12. Do not print hook-facing progress text to stdout.
13. Do not change public contracts without conductor approval.
14. Do not mutate events.
15. If blocked, update the track file and stop.

---

## 14. Recommended First Implementation Sprint

The first implementation sprint should not try to integrate all harnesses.

### Sprint 1 target

Build a working encrypted local capture loop:

```text
ai-brains init
ai-brains ingest-session-event
ai-brains ingest-turn
ai-brains ingest-final
ai-brains sessions
ai-brains recall
```

### Sprint 1 must include

- Encrypted vault.
- Recovery kit.
- Event log.
- Basic projections.
- FTS recall.
- Path normalization.
- Git metadata.
- No daemon yet.
- No graph yet.
- No model calls yet.
- No harness hooks yet.

### Sprint 1 proof

Run:

```powershell
ai-brains init
ai-brains ingest-session-event < sample-session.json
ai-brains ingest-turn < sample-user-prompt.json
ai-brains ingest-final < sample-final.json
ai-brains sessions --project .
ai-brains recall "what did the AI just do?" --project .
```

Expected result:

- Session exists.
- User prompt exists.
- Final response exists.
- Recall returns the final response.
- No repo files were written.
- Vault is encrypted.
- Wrong key cannot open vault.

This is the smallest meaningful version.

---

## 15. Recommended Second Implementation Sprint

### Sprint 2 target

Add daemon concurrency and Claude hook capture.

### Sprint 2 must include

- `ai-brainsd`.
- Single writer queue.
- Spool/replay on daemon failure.
- Claude adapter.
- User-level hook installer.
- E2E test for two sessions in same project and one in another.

### Sprint 2 proof

Run three simulated sessions concurrently and verify:

- All events captured.
- Two map to Project A.
- One maps to Project B.
- Active session query works.
- No tool/thinking fields stored.

---

## 16. Recommended Third Implementation Sprint

### Sprint 3 target

Add preflight and nightly degraded intelligence.

### Sprint 3 must include

- Preflight JSON/markdown.
- Privacy filtering.
- Pinned memories.
- Basic nightly session/daily summaries.
- Model unavailable degraded mode.
- Backup creation.

### Sprint 3 proof

- Preflight under 1500 words.
- Sealed memory excluded.
- Pinned memory included.
- Nightly job records partial failure if model missing.
- Backup restore works.

---

## 17. Final Release Gate

AI-Brains is not release-ready until all of this passes:

```powershell
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo nextest run --workspace
cargo deny check
cargo audit
scripts/run-failure-drills.ps1
```

And the following statements are true:

- Capture works offline.
- Capture works if graph/model unavailable.
- Vault storage honesty (**F8**): bundled SQLite + application-level Content Envelope (CE) AES-256-GCM + OS filesystem permissions — **not** a live “full DB encryption” / page-level SQLCipher claim by default. See [COMPATIBILITY.md](COMPATIBILITY.md) F8, [SECURITY-LIMITS.md](SECURITY-LIMITS.md), and [RELEASE-CLAIMS.md](RELEASE-CLAIMS.md) (**R-F8**, **R-ZERO-KEY**).
- Recovery kit restores.
- No repo writes by default.
- Hidden thinking/tool calls are not stored.
- Privacy gates block cloud leakage.
- Event log is append-only.
- Projections rebuild.
- Windows path normalization passes.
- Three concurrent sessions across two projects pass E2E.
- Preflight is useful and under 1500 words.
- Nightly jobs degrade gracefully.
- **Release process:** execute [RELEASE-CHECKLIST.md](RELEASE-CHECKLIST.md) (claims scan, deny/audit, SBOM, NOTICE, platform smoke, residual citation, human sign-off) before public `v*` / binary distribution claims.
