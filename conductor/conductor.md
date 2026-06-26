# AI-Brains Conductor Registry

| Track | Name | Status | Owner | Spec | Description |
|-------|------|--------|-------|------|-------------|
| T61 | Nightly Synthesis Batch Limit | ✅ **Complete** | Hermes | [T61](tracks/trackT61-nightly-synthesis-limit/spec.md) | Fixed nightly hang by limiting synthesis to 50-memory batches |
| T62 | Semantic Search — Stored Embeddings | ✅ **Complete** | Hermes | [T62](tracks/trackT62-semantic-embeddings/spec.md) | Added embedding column, built backfill script, verified semantic recall |
| T63 | Nightly Embedding Integration | ✅ **Complete** | Hermes | [T63](tracks/trackT63-nightly-embedding-integration/spec.md) | Integrated embedding backfill into nightly pipeline; 50 memories auto-embedded per run |
| T64 | Stale Embedding Refresh + WAL Checkpointing | ✅ **Complete** | Hermes | [T64](tracks/trackT64-stale-refresh-wal/spec.md) | Added embedding timestamps, stale refresh, WAL checkpointing |
| T65 | Repo Alias Resolution | ✅ **Complete** | Hermes | [T65](tracks/trackT65-repo-alias-resolution/spec.md) | Auto-detect project IDs from aliases and git repos; scope recall per-repo |
| T66 | Graph-Augmented Recall + Graph Query CLI | ✅ **Code Complete** | Hermes+Claude | [T66](tracks/trackT66-graph-augmented-recall/spec.md) | All 3 phases implemented (recall augmentation, CLI queries, boost config). Pending `MemoryPinned`/`MemorySynthesized` events for full data |
| T67 | Memory Pinning Events | ✅ **Complete** | Hermes+Codex | [T67](tracks/trackT67-memory-pinning-events/spec.md) | Emit `MemoryPinned` events on recall so graph gets `RECALLS`/`SOURCE_FOR` edges |
| T68 | Memory Synthesis Events | ✅ **Complete** | Hermes+Codex | [T68](tracks/trackT68-memory-synthesis-events/spec.md) | Emit `MemorySynthesized` events during nightly so graph gets `SYNTHESIZED_FROM` edges |
| T69 | Live Graph Hook — Incremental Graph Updates | ✅ **Complete** | Claude+Codex | [T69](tracks/trackT69-live-graph-hook/spec.md) | Apply graph projector incrementally on each event append; eliminates need for manual `graph rebuild` after recall/nightly |
| T70 | ChangeGuard Symbol Bridge — Code-Aware Recall | ✅ **Complete** | Codex | [T70](tracks/trackT70-changeguard-symbol-bridge/spec.md) | Ingest ChangeGuard's code symbols (functions and routes) into AI-Brains during nightly so `recall` returns actual code structure |
| T71 | CI Tooling Reproducibility | ✅ **Complete** | Claude | [T71](tracks/trackT71-ci-tooling-reproducibility/spec.md) | All three tools install via cargo install --locked; full gate passes locally; scripts/dev-check.ps1 verifies presence + runs gate |
| T72 | Status & Doc Reconciliation | ✅ **Complete** | Claude | [T72](tracks/trackT72-status-reconciliation/spec.md) | Rewrote Docs/status.md to T71 reality; restored bridge docs to .agents skill; archived stale root artifacts; documented cargo audit quirk in ci-tooling.md |
| T73 | Idempotent `init` | ✅ **Complete** | Claude | [T73](tracks/trackT73-init-safety/spec.md) | `init` refuses on populated vault unless `--force`; structured JSON error envelope on refusal; 2 new tests |
| T74 | Graph Health Smoke Test | ✅ **Complete** | Claude | [T74](tracks/trackT74-graph-health-smoke/spec.md) | nextest smoke that runs init → ingest → pin → recall → `graph update` and asserts nodes/edges ≥ 1, status live |
| T75 | OPERATIONS.md Modernization | ✅ **Complete** | Claude | [T75](tracks/trackT75-operations-modernization/spec.md) | Rewrote OPERATIONS.md to cover daemon, forget, safety sync, sync query, bridge, schedule, restore, and the full env-var surface |
| T76 | CLI Polish (project list + backup restore) | ✅ **Complete** | Claude | [T76](tracks/trackT76-cli-polish/spec.md) | Widened `project list` name column with hint header; added `--force` and `--dry-run` to `backup restore`; 2 new tests |
| T77 | forget --memory-id validation | ✅ **Complete** | Claude | [T77](tracks/trackT77-forget-memory-id-validation/spec.md) | `forget --memory-id=<unknown>` exits 1 with "Memory <id> not found." instead of silently no-op'ing |
| T78 | daemon schedule schtasks quoting | ✅ **Complete** | Claude | [T78](tracks/trackT78-daemon-schedule-quoting/spec.md) | `render_daemon_logon_command` uses single-quote wrapping so schtasks accepts paths with spaces |
| T79 | nightly --skip-import | ✅ **Complete** | Claude | [T79](tracks/trackT79-nightly-skip-import/spec.md) | Opt-out flag for `antigravity_import` in `nightly`; prevents cross-vault contamination on isolated/CI vaults |
| T80 | --no-project-context flag | ✅ **Complete** | Claude | [T80](tracks/trackT80-no-project-context-flag/spec.md) | Global escape hatch so `main()` does not auto-clear `AI_BRAINS_*` env vars when no `.env` exists in cwd |
| T81 | --quiet silences bridge warnings | ✅ **Complete** | Claude | [T81](tracks/trackT81-quiet-bridge-warnings/spec.md) | `recall --quiet`, `preflight --quiet`, `sync query --quiet` suppress the "ChangeGuard bridge query failed" warning |
| T82 | honor context --new-project | ✅ **Complete** | Claude | [T82](tracks/trackT82-context-new-project/spec.md) | `context --new-project` rotates the project_id and prints "Rotating project ID from <old> to fresh UUID." |
| T83 | JSON schemas for agy-hook & sync pull | ✅ **Complete** | Claude | [T83](tracks/trackT83-schemas-for-cli-commands/spec.md) | `agy-hook --schema` and `sync pull --schema` print JSON Schema 2020-12 documents; schemas at `Docs/schemas/` |
| T84 | Self-Healing / Auto-Restart Tooling | ✅ **Complete** | Claude | [T84](tracks/trackT84-self-healing-auto-restart/spec.md) | `daemon update` stops daemon gracefully (force if unresponsive), runs `cargo install`, restarts; `Build-AIBrains.ps1` does the same |
| T85 | Configuration-Based Backend URL and Port Status Checks | ✅ **Complete** | Claude | [T85](tracks/trackT85-config-based-port-status/spec.md) | `daemon status` reads `AI_BRAINS_MODEL_URL`/`AI_BRAINS_EMBEDDING_URL`, parses host:port, probes those; defaults to Ollama :11434 and llama.cpp :8080 |
| T86 | Structured Stdin for Pipeline Tooling | ✅ **Complete** | Claude | [T86](tracks/trackT86-structured-stdin/spec.md) | `recall -` reads query from stdin; `preflight --stdin` reads JSON `{"scope":[...],"max_words":N}` from stdin; TTY guard prevents hanging |
| T87 | Bridge:Vault Result Ratio in Recall | ✅ **Complete** | Claude | [T87](tracks/trackT87-bridge-vault-recall-ratio/spec.md) | Bridge capped at `limit.div_ceil(2)`; `--no-bridge` flag; vault memories always surface; test asserts ≥1 vault hit |
| T88 | Fix `pin` to Print Projection `memory_id` | ✅ **Complete** | Claude | [T88](tracks/trackT88-pin-prints-memory-id/spec.md) | `pin.rs` prints `turn_id` (not `event_id`); `forget --memory-id` now works with the reported UUID |
| T89 | `project set-alias` Command | ✅ **Complete** | Claude | [T89](tracks/trackT89-project-set-alias/spec.md) | `project set-alias <project_id> <alias>` appends `ProjectAliasAdded` event; idempotent; duplicate-alias exits 1 |
| T90 | FTS5 Query Sanitization | ✅ **Complete** | Claude | [T90](tracks/trackT90-fts5-query-sanitization/spec.md) | `sanitize_fts_query` in `ai-brains-retrieval`; wraps tokens in double-quotes; used by recall + sync query; 6 unit tests |
| T91 | Strip ANSI Before Ledger Search in `sync query` | ✅ **Complete** | Claude | [T91](tracks/trackT91-strip-ansi-sync-query/spec.md) | `strip_ansi` then `sanitize_fts_query` applied to query before `changeguard ledger search` in `sync.rs` |
| T92 | Debug and Fix `sync pull --hotspots/--ledger` | ✅ **Complete** | Claude | [T92](tracks/trackT92-sync-pull-hotspots-debug/spec.md) | Fixed bootstrap lineage bug (first import rejected all records); removed direction filter that discarded ChangeGuard-native records |
| T93 | `project detect` Fallback to `.env` Project ID | ✅ **Complete** | Claude | [T93](tracks/trackT93-project-detect-env-fallback/spec.md) | Falls back to `AI_BRAINS_PROJECT_ID` env var with `(from .env)` indicator; exits 1 with clear message when neither slug nor env matches |
| UX | Friendly default project name | ✅ **Complete** | Claude | [UX](tracks/trackUX-friendly-default-project-name/spec.md) | Default name is `(no alias) — <8-char-uuid-prefix>` instead of `Project <full-uuid>`; full id still in dedicated column |
| Docs | WORKFLOWS.md cookbook | ✅ **Complete** | Claude | [Docs/WORKFLOWS.md](../Docs/WORKFLOWS.md) | 6 end-to-end recipes: setup, Antigravity import, hygiene, backup, code-search, daemon/nightly |
| T94 | Connection Handshake Retries & Jitter | ✅ **Complete** | Claude | [T94](tracks/trackT94-connection-handshake-retries/spec.md) | Implement backoff retry patterns in the TCP status checks for backend providers to prevent false-negatives on slow startup |
| T95 | `sync query` Project Isolation | ✅ **Complete** | Claude | [T95](tracks/trackT95-sync-query-project-isolation/spec.md) | Pretty-path `sync query` scopes to `AI_BRAINS_PROJECT_ID` by default; `--global` flag for opt-in unscoped recall |
| T96 | SQLCipher `busy_timeout` | ✅ **Complete** | Claude | [T96](tracks/trackT96-sqlcipher-busy-timeout/spec.md) | Added `PRAGMA busy_timeout = 5000` to fix transient "unable to open database file" under concurrent CLI access |
| T97 | Migrate Shell-Out `changeguard` → `ledgerful` | ✅ **Complete** | Claude | [T97](tracks/trackT97-ledgerful-binary-rename/spec.md) | Updated ~16 callsites to use renamed binary; updated error messages and docs |
| T98 | Pass `--auto-index` to Bridge Calls | ✅ **Complete** | Claude | [T98](tracks/trackT98-bridge-auto-index/spec.md) | Added `--auto-index` to `ledgerful search` in recall bridge; skipped `bridge export` (unsupported) |
| T99 | Fix `backup create` — SQLCipher Key | ✅ **Complete** | Claude | [T99](tracks/trackT99-backup-sqlcipher-key/spec.md) | Backup hangs: add SQLCipher key + busy_timeout pragmas; guard restore against running daemon; delete stale backup files |
| T100 | LLM Request Timeouts | ✅ **Complete** | GLM-5.2 | [T100](tracks/trackT100-llm-request-timeout/spec.md) | `nightly` hangs: add per-request timeouts (120s/30s/10s) to reqwest client; reuse client across requests |
| T101 | Default `recall` to Pretty Format on TTY | ✅ **Complete** | Claude | [T101](tracks/trackT101-recall-pretty-default-tty/spec.md) | Detect TTY and default to pretty format; truncate long content; tests must pass --format explicitly |
| T102 | Suppress Session-ID Noise | ✅ **Complete** | Claude | [T102](tracks/trackT102-suppress-session-noise/spec.md) | Change eprintln to tracing::debug; include session_id in JSON/pretty output as metadata |
| T103 | `daemon schedule/unschedule --dry-run` | ✅ **Complete** | Claude | [T103](tracks/trackT103-daemon-schedule-dry-run/spec.md) | Add --dry-run to both schedule and unschedule; include UAC permission note in output |
| T104 | Backup Retention / Prune Policy | ✅ **Complete** | GLM-5.2 | [T104](tracks/trackT104-backup-retention-prune/spec.md) | `backup prune --keep N --older-than <dur>`; `backup create --keep N` auto-prune; `--dry-run` support |
| T105 | Recall Fallback for Small Vaults | ✅ **Complete** | GLM-5.2 | [T105](tracks/trackT105-recall-fallback-small-vaults/spec.md) | Substring LIKE fallback when FTS5 returns empty; 10K memory CPU guard; contextual no-results hint with --global/--semantic suggestions |
| T106 | Nightly End-to-End Timeout Validation | ✅ **Complete** (partial) | GLM-5.2 | [T106](tracks/trackT106-nightly-timeout-validation/spec.md) | AC4/AC5 blocked by env-var override; T100 timeout logic verified correct; follow-up track needed for --model-url flag |
| T107 | Unified --dry-run for Mutating Commands | ✅ **Complete** | GLM-5.2 | [T107](tracks/trackT107-unified-dry-run/spec.md) | Add --dry-run to pin, forget, ingest; append_event isolated behind if !dry_run; preview to stdout |
| T108 | `project resolve --alias` Flag | ✅ **Complete** | GLM-5.2 | [T108](tracks/trackT108-project-resolve-alias-flag/spec.md) | Accept --alias <name> as alternative to positional arg; conflicts_with declarative XOR; backward compatible |
| T109 | Backup Metadata Header Table | ✅ **Complete** | GLM-5.2 | [T109](tracks/trackT109-backup-metadata-header/spec.md) | `_aibrains_backup_meta` table in backup with timestamp, source path, version; `backup list` subcommand; DROP TABLE after restore |
| T110 | Strip ANSI in `sync query` When Not TTY | ✅ **Complete** | GLM-5.2 | [T110](tracks/trackT110-strip-ansi-sync-query-non-tty/spec.md) | NO_COLOR=1 env var + strip_ansi on output when not TTY; preserve color on TTY |
| T111 | Recall No-Results Hint | ✅ **Complete** | GLM-5.2 | [T111](tracks/trackT111-recall-no-results-hint/spec.md) | Contextual hints: suggest --semantic, --global based on flags; small-vault warning; `hint` field in RecallResponse; sequenced after T105 fallback |
| T112 | Recall Scope Overhaul | ✅ **Complete** | Hermes | [T112](tracks/trackT112-recall-scope-overhaul/spec.md) | Default to project-wide search (no session filter); `--global` clears both project+session; add `--session` flag for explicit session scoping |
| T113 | Env-Var Override Precedence | ✅ **Complete** | Claude | [T113](tracks/trackT113-env-var-override-precedence/spec.md) | Shell env vars > project .env > global .env; replace dotenv_override with dotenv (non-override); unblocks T106 testing |
| T114 | Ingest --dry-run Skip UUID Validation | ✅ **Complete** | GLM-5.2 | [T114](tracks/trackT114-ingest-dry-run-skip-validation/spec.md) | Dry-run accepts placeholder UUIDs; strict validation only on non-dry-run path |
| T115 | Sync Query Daemon Fallback | ✅ **Complete** | Claude | [T115](tracks/trackT115-sync-query-daemon-fallback/spec.md) | Remove daemon gate; local recall + ChangeGuard search proceed without daemon |
| T116 | Backup List Full Paths | ✅ **Complete** | GLM-5.2 | [T116](tracks/trackT116-backup-list-full-paths/spec.md) | Show filename not truncated full path; strip UNC prefix from source_vault_path |
| T117 | Backup Schema Version Fix | ✅ **Complete** | GLM-5.2 | [T117](tracks/trackT117-backup-schema-version-fix/spec.md) | Fix schema_migrations query (MAX(name) not MAX(version)); metadata shows latest migration name |
| T118 | eprintln! to tracing! Migration | ✅ **Complete** | Claude | [T118](tracks/trackT118-eprintln-to-tracing-migration/spec.md) | Migrate 62 eprintln! calls to tracing::info!/warn!/debug!; default log level info; keep eprintln! only for errors+prompts |
| T119 | `backup create --dry-run` | ✅ **Complete** | GLM-5.2 | [T119](tracks/trackT119-backup-create-dry-run/spec.md) | Add --dry-run to backup create; preview path/size without writing file |
| T120 | `backup list` Noise Suppression | ✅ **Complete** | GLM-5.2 | [T120](tracks/trackT120-backup-list-noise-suppression/spec.md) | Content-based discrimination: demote expected-condition warnings to debug!; keep warn! for corruption |
| T121 | `backup list` Source Vault Column Width | ✅ **Complete** | GLM-5.2 | [T121](tracks/trackT121-backup-list-source-vault-width/spec.md) | Widen Source Vault column to 40 chars; right-truncate to show path end not beginning |
| T122 | `graph` Subcommand Hint Stub | ✅ **Complete** | GLM-5.2 | [T122](tracks/trackT122-graph-subcommand-hint-stub/spec.md) | Stub graph subcommand in default build that prints --features graph install hint |
| T123 | Backup Timestamp Parser Robustness | ✅ **Complete** | GLM-5.2 | [T123](tracks/trackT123-backup-timestamp-parser-robustness/spec.md) | Parse nanosecond+timezone timestamp formats from old backups; recover orphaned files |
| T124 | `sync query --no-bridge` Flag | ✅ **Complete** | GLM-5.2 | [T124](tracks/trackT124-sync-query-no-bridge-flag/spec.md) | Add --no-bridge to sync query to skip ChangeGuard search; consistency with recall |
| T125 | `recall --session` Partial Match | ✅ **Complete** | GLM-5.2 | [T125](tracks/trackT125-recall-session-partial-match/spec.md) | Accept session UUID prefix (--session-prefix); add --session-last for most recent session |
| T126 | `backup create` Default Retention | ✅ **Complete** | GLM-5.2 | [T126](tracks/trackT126-backup-create-default-retention/spec.md) | Default --keep 10; --no-prune opt-out; --keep 0 rejected; migration sentinel warning |
| T127 | `sync query` NDJSON Session Passthrough | ✅ **Complete** | GLM-5.2 | [T127](tracks/trackT127-sync-query-ndjson-session-passthrough/spec.md) | NDJSON records carry source session_id instead of hardcoded null |
| T128 | `daemon status` Vault Info | ✅ **Complete** | GLM-5.2 | [T128](tracks/trackT128-daemon-status-vault-info/spec.md) | Show vault path, size, memory count in daemon status output |
| T129 | Tracing Output Format Option | ✅ **Complete** | GLM-5.2 | [T129](tracks/trackT129-tracing-output-format-option/spec.md) | --log-format compact/full/json/off; default compact; RUST_LOG still controls level |
| T130 | `recall` Result Session-ID Field | ✅ **Complete** | GLM-5.2 | [T130](tracks/trackT130-recall-result-session-id-field/spec.md) | Per-result session_id in JSON/pretty/NDJSON; rename top-level to effective_session_id |
| T131 | `backup verify` Command | ✅ **Complete** | GLM-5.2 | [T131](tracks/trackT131-backup-verify-command/spec.md) | Verify integrity of all or single backup; PRAGMA quick_check default + --full for integrity_check |
| T132 | `--run-as-system` Flag for Schedule | ✅ **Complete** | Claude | [T132](tracks/trackT132-schedule-run-as-system/spec.md) | Added --run-as-system to nightly and daemon schedule; /ru SYSTEM appended; elevation error clarified; 5 unit tests pass |
| T133 | Recall Hint to Stdout | ✅ **Complete** | — | [T133](tracks/trackT133-recall-hint-to-stdout/spec.md) | Move no-results hint from eprintln! to println! so PowerShell doesn't wrap as error |
| T134 | `backup list --quiet` Flag | ✅ **Complete** | — | [T134](tracks/trackT134-backup-list-quiet-flag/spec.md) | Suppress WARN noise from old/unopenable backups |
| T135 | `nightly --status` Shows Schedule State | ✅ **Complete** | — | [T135](tracks/trackT135-nightly-status-show-schedule-state/spec.md) | Show whether nightly task is scheduled (Yes/No + time) |
| T136 | `--log-format minimal` Mode | ✅ **Complete** | — | [T136](tracks/trackT136-log-format-minimal-mode/spec.md) | Level + message only, no timestamp/target; between compact and off |
| T137 | Bridge Hits Populate Session-ID | ✅ **Complete** | — | [T137](tracks/trackT137-bridge-hits-populate-session-id/spec.md) | Read session_id from BridgeRecord in bridge query path; omit session= in pretty when None |
| T138 | `backup verify` FAIL Error Reason | ✅ **Complete** | — | [T138](tracks/trackT138-backup-verify-fail-error-reason/spec.md) | Show why each backup failed (wrong key vs corruption vs missing tables) |
| T139 | Project Env Precedence and Preflight Scope | Complete | Codex | [T139](tracks/trackT139-project-env-precedence-preflight-scope/spec.md) | Local `.env` project/session IDs override stale inherited shell IDs with a warning |
| T140 | FTS Query Sanitization for Bridge and Recall | Complete | Codex | [T140](tracks/trackT140-fts-query-sanitization-bridge-recall/spec.md) | Shared sanitizer tokenizes punctuation-heavy prompts so commas cannot crash FTS5 |


---

## Track Status Legend
- **Pending** — Requirements written, no implementation started
- **In Progress** — Active development
- **Complete** — All success criteria met, verified in production
- **Blocked** — External dependency preventing progress
- **Abandoned** — No longer relevant, archived for reference

## Adding a New Track
1. Create `tracks/trackTNN-<name>/spec.md` with problem statement, design, and verification
2. Add entry to table above with **Pending** status
3. Update to **In Progress** when implementation starts
4. Update to **Complete** when all success criteria are met
