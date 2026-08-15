# T252 Completeness Gate

**Date:** 2026-08-15  
**Verdict:** COMPLETE (internal)

Internal R1 completeness vs spec: **PASS** (`review.internal.r1.md`)  
Internal R1b correctness/tests: **PASS** (`review.internal.r1b.md`)  
No open P0–P3. Implementation matches F1–F15 / AC1–AC13. AC16 N/A after go.

## Orchestrator-observed gates (not in static reviews)

| Gate | Result |
|------|--------|
| `cargo nextest run -p ai-brains-cli -E "binary(ingest_reads_json_stdin) + test(ingest_stdin_needs_usage) + test(ingest_empty_stdin_usage)"` | **12/12 PASS** |
| `cargo nextest run -p ai-brains-cli -E "binary(protocol_compat_cli) + binary(cli_help_ia)"` | **11/11 PASS** (AC9 T180 + AC8 group-order) |
| `cargo clippy -p ai-brains-cli --all-targets -- -D warnings` | PASS (implementer-observed) |
| AC14(1) `echo '' \| target\debug\ai-brains.exe ingest --dry-run` | exit **2**, usage example, no EOF JSON; repeated byte-stable |
| AC14(2) `echo '{' \| … ingest --dry-run` | exit **1** `COMMAND_FAILED` / `Invalid JSON` |
| AC14(3) CREATE_NEW_CONSOLE TTY, 8s timeout | exit **2** immediately; usage on stderr; empty stdout; no hang |

Ready for Codex cross-model review.
