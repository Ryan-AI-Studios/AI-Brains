# Track Completion Audit — T238

## Verdict: FAIL

## Scope Reviewed

Read-only audit of `spec.md`, `plan.md`, implementation diff, tests, wiring, docs, governance files, and prior review log.

## Requirement and DoD Matrix

| Area | Result |
|---|---|
| AC1–AC2 nested/flat filtering | Met |
| AC3–AC4 hook path and shared IDs | Partial; no CLI hook integration proof and index-based delta |
| AC5–AC8 binding, watermark, force/dry-run | Mostly met |
| AC9–AC11 install/uninstall | Partial; marker validation is too permissive |
| AC12 missing binary | Met |
| AC13 path normalization | Met |
| AC14 never `opencode.db` | Implementation met; test is source-string based |
| AC15 documentation/governance | Partial; track remains Implementing/Planning |
| AC16 timeout fail-open | Partial; subprocess is not killed by Rust timeout |
| AC17 capture independence | Default build met; graph-enabled CLI path still constructs graph hooks |
| AC18–AC20 marker, tool filtering, hermetic batch | Mostly met |
| AC21 child-session safety | Partial; roots-only list flag absent and plugin lookup failure continues |
| AC22 synthetic chrome | Met |
| AC23 list-cap warning | Partial for vendor hard-cap cases |
| F1–F7 message filter | Met |
| F8–F15 live plugin | Partial; privacy cleanup and true ID delta remain |
| F16–F26 batch import | Partial; roots flag, timeout, corruption handling, and export binding gaps |
| F27–F33 install/readiness | Mostly met |
| F34–F40 docs/capability/config | Partial; relocated config is not detected consistently |
| Phase 6 DoD | Not met |

## Findings

### [P1] Live plugin leaves raw tool/reasoning exports in `%TEMP%`

`writeTempExport` and `exportViaCli` persist full SDK/CLI exports, including tool and reasoning parts, but no cleanup occurs afterward. Evidence: [install.rs:621](/C:/dev/AI-Brains/crates/ai-brains-cli/src/harness/install.rs:621), [install.rs:629](/C:/dev/AI-Brains/crates/ai-brains-cli/src/harness/install.rs:629), [install.rs:665](/C:/dev/AI-Brains/crates/ai-brains-cli/src/harness/install.rs:665).

This conflicts with the project Capture Privacy mandate. Raw sensitive content remains on disk after ingestion.

### [P1] Child-session exclusion is not guaranteed

`run_opencode_list` does not pass the required roots-only option; it only invokes `session list --format json -n N` ([opencode.rs:635](/C:/dev/AI-Brains/crates/ai-brains-adapters/src/opencode.rs:635)). The plugin also proceeds when `client.session.get` fails, so a child session can be ingested when parent metadata is unavailable ([install.rs:696](/C:/dev/AI-Brains/crates/ai-brains-cli/src/harness/install.rs:696)).

AC21 is therefore not fail-closed.

### [P1] F14 message-ID delta semantics are not implemented

The shared append path skips by positional index ([opencode.rs:170](/C:/dev/AI-Brains/crates/ai-brains-adapters/src/opencode.rs:170)); callers derive that index from `MAX(turn_index)` ([opencode.rs:847](/C:/dev/AI-Brains/crates/ai-brains-adapters/src/opencode.rs:847), [opencode_hook.rs:123](/C:/dev/AI-Brains/crates/ai-brains-cli/src/commands/opencode_hook.rs:123)).

It does not check whether each `v5(session,msg_id)` already exists. Reordering, filtering changes, or an inserted message can cause missed turns or duplicates.

### [P1] Required completion verification and closure are missing

Phase 6 remains unchecked: full workspace gate, manual live test, `ledgerful verify`, pins, ledger commit, and conductor completion ([plan.md:87](/C:/dev/AI-Brains/conductor/tracks/trackT238-opencode-seamless-ingest/plan.md:87)).

The registry still marks T238 as Planning/Implementing. `ledgerful doctor` and `ledgerful ledger status --compact` could not run because the Ledgerful database was unavailable. No full gate or live install/idle/recall evidence was provided.

### [P2] Managed-marker protection is not header-scoped

Install/uninstall use `contains(OPENCODE_PLUGIN_MARKER)` rather than verifying the marker header ([install.rs:779](/C:/dev/AI-Brains/crates/ai-brains-cli/src/harness/install.rs:779)). A foreign file containing the marker later in its body could be overwritten or removed.

The wiring probe also treats any same-name `.js`/`.ts` file as valid ([wiring.rs:138](/C:/dev/AI-Brains/crates/ai-brains-cli/src/harness/wiring.rs:138)).

### [P2] `OPENCODE_CONFIG_DIR` detection is incomplete

Install and probing honor the environment variable, but harness presence detection only checks the default `.config/opencode` path ([detect.rs:163](/C:/dev/AI-Brains/crates/ai-brains-cli/src/harness/detect.rs:163)). A relocated installation can report `absent`, preventing the relocated probe from running.

### [P2] Rust export timeout leaks the child process

`run_opencode_export_blocking` waits on a worker thread with `recv_timeout`, but has no `Child` handle to terminate the `opencode export` process after timeout ([opencode.rs:603](/C:/dev/AI-Brains/crates/ai-brains-adapters/src/opencode.rs:603)). Repeated live fallbacks can leave orphaned processes.

### [P2] List-cap honesty fails when users request more than 100

The warning compares against the requested `max_sessions`, not OpenCode’s known vendor cap. With `--max-sessions 500`, a 100-row vendor-capped result produces no `list_capped` warning.

### [P2] Seam behavior is not behaviorally tested

Tests cover filtering and batch import, but no CLI-level `opencode-hook` vault test or executable plugin test proves SDK success, SDK fallback, in-flight suppression, parent lookup, temp cleanup, or live/batch parity. The install test only searches generated JavaScript strings.

### [P2] Corrupt cursor handling is a silent fallback

Malformed cursor JSON silently becomes an empty cursor ([opencode.rs:479](/C:/dev/AI-Brains/crates/ai-brains-adapters/src/opencode.rs:479)), potentially causing broad re-export without warning or a corruption statistic.

## Completeness Sweep

- No `TODO`, `FIXME`, `todo!`, or `unimplemented!` remains in the main OpenCode path.
- `config_dir_override` is declared but unused ([opencode.rs:347](/C:/dev/AI-Brains/crates/ai-brains-adapters/src/opencode.rs:347)).
- Contracts remain unchanged; the new payload schema is present.
- No dependency bump was introduced.
- `git diff --check` fails on trailing whitespace in modified track documentation.
- `opencode.db` is not opened by the implementation.

## Wiring and Regression Review

Core registration is wired through `main.rs`, command modules, adapter exports, harness install/uninstall, readiness detection, wiring, preflight, and help IA.

AGY/Grok paths appear unchanged in the reviewed diff. The primary regressions are OpenCode-specific: child exclusion, live privacy cleanup, ID-based delta handling, relocated-config detection, and incomplete executable seam coverage.

## Verification Evidence

Provided/reviewed:

- Targeted OpenCode/adapters suites: 17/17 reported passing.
- CLI install/help/wiring checks: 10/10 reported passing.
- Adapters + CLI clippy reported clean.
- Static source/doc inspection completed.

Not completed or unavailable:

- Full workspace gate.
- `ledgerful verify --scope full`.
- Manual live OpenCode install → idle → recall test.
- Current-tree cross-model review.
- Ledgerful doctor/status: failed with `unable to open database file`.
- `git diff --check`: failed on trailing whitespace.

## Deferred Candidates

None. The P1/P2 findings are completion blockers and should not be deferred.

## Completion Decision

T238 is not complete. Do not mark the track Completed or close its ledger transaction until the privacy cleanup, fail-closed child handling, message-ID delta implementation, verification gates, and required manual/runtime evidence are resolved.