**Findings**

- `P3` `thread::sleep` remains in the re-summarize proof test, so the known low-grade timing residual is still present rather than eliminated. Evidence: [get_unsummarized_relist.rs](/abs/path/C:/dev/AI-Brains/crates/ai-brains-store/tests/get_unsummarized_relist.rs:90), [get_unsummarized_relist.rs](/abs/path/C:/dev/AI-Brains/crates/ai-brains-store/tests/get_unsummarized_relist.rs:120).

No `P0`/`P1`/`P2` findings are supported by the current branch state I could inspect.

**DoD Matrix**

| Area | Result | Evidence |
|---|---|---|
| F8 / AC18 wrapper stdout contract | Pass | Wrapper SOOT is `{"decision":"allow"}` only, never `"continue"`, diagnostics stay on stderr, and child `agy-hook` stdout is captured: [install.rs](/abs/path/C:/dev/AI-Brains/crates/ai-brains-cli/src/harness/install.rs:113), [install.rs](/abs/path/C:/dev/AI-Brains/crates/ai-brains-cli/src/harness/install.rs:123), [install.rs](/abs/path/C:/dev/AI-Brains/crates/ai-brains-cli/src/harness/install.rs:164), [install.rs](/abs/path/C:/dev/AI-Brains/crates/ai-brains-cli/src/harness/install.rs:527) |
| F3 / F12 / AC5 / AC7 / AC16 / AC17 / AC20 project binding and anti-hijack | Pass | Hook narrows env fallback to empty/`agy-unbound` only and otherwise normalizes path aliases before resolve/create: [agy_hook.rs](/abs/path/C:/dev/AI-Brains/crates/ai-brains-cli/src/commands/agy_hook.rs:44), [agy_hook.rs](/abs/path/C:/dev/AI-Brains/crates/ai-brains-cli/src/commands/agy_hook.rs:52), [antigravity.rs](/abs/path/C:/dev/AI-Brains/crates/ai-brains-adapters/src/antigravity.rs:369), [antigravity.rs](/abs/path/C:/dev/AI-Brains/crates/ai-brains-adapters/src/antigravity.rs:467), [mapping_delta_smoke.rs](/abs/path/C:/dev/AI-Brains/crates/ai-brains-cli/tests/mapping_delta_smoke.rs:197), [antigravity_import_t236.rs](/abs/path/C:/dev/AI-Brains/crates/ai-brains-adapters/tests/antigravity_import_t236.rs:73) |
| AC6 project-scoped recall without `--global` | Pass | Prior soft gap is now explicitly covered: import proof asserts `turn_projection.project_id` equals the history-bound project, and CLI smoke proves project-scoped recall under the bound project id without `--global`: [antigravity_import_t236.rs](/abs/path/C:/dev/AI-Brains/crates/ai-brains-adapters/tests/antigravity_import_t236.rs:135), [antigravity_import_t236.rs](/abs/path/C:/dev/AI-Brains/crates/ai-brains-adapters/tests/antigravity_import_t236.rs:150), [mapping_delta_smoke.rs](/abs/path/C:/dev/AI-Brains/crates/ai-brains-cli/tests/mapping_delta_smoke.rs:281) |
| F2 / AC19 turn-id SOOT | Pass | Hook and batch now share `generate_turn_id_for_ingest`, with `turn-{step_index}` preferred and legacy `agy-turn-*` retired: [agy.rs](/abs/path/C:/dev/AI-Brains/crates/ai-brains-adapters/src/agy.rs:99), [agy.rs](/abs/path/C:/dev/AI-Brains/crates/ai-brains-adapters/src/agy.rs:106), [agy.rs](/abs/path/C:/dev/AI-Brains/crates/ai-brains-adapters/src/agy.rs:338) |
| F29 / AC21 transcript-full preference | Pass | Discovery now prefers `transcript.jsonl` over `overview.txt`, and parsing prefers sibling `transcript_full.jsonl` when present: [antigravity.rs](/abs/path/C:/dev/AI-Brains/crates/ai-brains-adapters/src/antigravity.rs:176), [agy.rs](/abs/path/C:/dev/AI-Brains/crates/ai-brains-adapters/src/agy.rs:122), [agy.rs](/abs/path/C:/dev/AI-Brains/crates/ai-brains-adapters/src/agy.rs:145), [agy.rs](/abs/path/C:/dev/AI-Brains/crates/ai-brains-adapters/src/agy.rs:377) |
| AC13 re-summarize relist | Pass with deferred timing residual | Query logic is correct and now re-queues sessions with turns after `summarized_at`: [query_store.rs](/abs/path/C:/dev/AI-Brains/crates/ai-brains-store/src/query_store.rs:77). Test coverage exists, but still uses sleeps: [get_unsummarized_relist.rs](/abs/path/C:/dev/AI-Brains/crates/ai-brains-store/tests/get_unsummarized_relist.rs:18), [get_unsummarized_relist.rs](/abs/path/C:/dev/AI-Brains/crates/ai-brains-store/tests/get_unsummarized_relist.rs:90) |
| F21 capability / docs honesty | Pass | Antigravity adapter is now `CapabilityLevel::Full`, hooks are supported, and docs consistently describe live+batch behavior while preserving the scheduled `--skip-import` caveat: [antigravity.rs](/abs/path/C:/dev/AI-Brains/crates/ai-brains-adapters/src/antigravity.rs:23), [antigravity_manual_import.rs](/abs/path/C:/dev/AI-Brains/crates/ai-brains-adapters/tests/antigravity_manual_import.rs:7), [CAPABILITIES.md](/abs/path/C:/dev/AI-Brains/Docs/CAPABILITIES.md:123), [CAPABILITIES.md](/abs/path/C:/dev/AI-Brains/Docs/CAPABILITIES.md:136), [OPERATIONS.md](/abs/path/C:/dev/AI-Brains/Docs/OPERATIONS.md:55), [antigravity-rule.md](/abs/path/C:/dev/AI-Brains/Docs/antigravity-rule.md:1), [WORKFLOWS.md](/abs/path/C:/dev/AI-Brains/Docs/WORKFLOWS.md:70) |
| AC15 gate | Pass by provided branch evidence; not rerunnable here | I could not rerun Cargo-based verification on Sunday, August 9, 2026 because the read-only sandbox blocks `target\\debug\\.cargo-lock` writes. The user-provided state says the prior full run was green and AC6-focused tests were re-verified. |

**Execution Notes**

I was able to run prebuilt pure test binaries that do not require temp-dir writes:
- `antigravity_manual_import ... ok`
- `tests::test_deterministic_turn_id ... ok`

I could not rerun the hermetic tempdir-based proofs in this sandbox because `%TEMP%` is not writable; those failures were environmental permission failures, not product assertions.

**Completion Decision**

`PASS WITH DEFERRED P3`

Engineering DoD for the requested re-review items is met on the inspected branch: AC6 is now directly proven in both storage and CLI scope terms, F21 is now `Full`, and F8/F3/F29/F12/F2 are implemented and documented consistently. The remaining deferred item is the already-known sleep-based timing proof in [get_unsummarized_relist.rs](/abs/path/C:/dev/AI-Brains/crates/ai-brains-store/tests/get_unsummarized_relist.rs:90).

External closeout gates remain outside this verdict: merge, pins, ledger commit/clean status, and conductor status flip from `In Progress` to `Completed`.