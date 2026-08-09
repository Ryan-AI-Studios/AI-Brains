# T236 Review Log — AGY 2 seamless ingest

**Track:** T236  
**Branch:** `feat/T236-agy2-seamless-ingest`  
**Ledger TX:** `a4f3806b-caef-47a4-bc43-8131ad43fe25`  
**Status:** Internal review round 1 FAIL → fixes → re-verify; Codex pending  

## Scope

Wrapper stdout SOOT (F8); history.jsonl binding (F9–F13); shared step/legacy parse + turn-id SOOT (F1–F2); F3 env narrow; F29 transcript_full; F30 source_meta path key; stats/`--force`; F17 re-summarize OR; docs honesty.

## Reviewers / rounds

| Round | Reviewer | Verdict | Notes |
|-------|----------|---------|-------|
| IR1 | explore subagent | **FAIL** | P2 dual overview+transcript; AC15 gate open; P3s AC6/AC17/capability/etc. |
| IR1 fix | orchestrator | — | Prefer transcript over overview in `scan_brain_dir`; unit test; AC17 hook case test |
| IR2 | (pending re-check after gate) | | |
| CX1 | Codex | pending | |

## Findings disposition

| ID | Sev | Status | Disposition |
|----|-----|--------|-------------|
| F-T236-R01 | P2 | `verified_fixed` | `scan_brain_dir` prefers `transcript.jsonl` over `overview.txt`; unit test proves F29 full path reachable |
| F-T236-R02 | P2 | open | AC15 full gate — run before PR merge |
| F-T236-R03 | P3 | deferred | AC6 hermetic project-scoped recall — anti-hijack + global prove binding; full scoped recall soft |
| F-T236-R04 | P3 | `verified_fixed` | `agy_hook__path_case_normalize__same_alias` added |
| F-T236-R05 | P3 | deferred | CapabilityLevel remains Partial (scheduled skip-import + connector deferred) — notes aligned |
| F-T236-R06 | P3 | deferred | Batch query Err→None pre-existing fail-open |
| F-T236-R07 | P3 | deferred | BrainLog harness id vs live agy harness — residual / T239 |
| F-T236-R08 | P3 | deferred | sleep in re-list test for timestamp ordering |

## Internal re-check evidence (post R01/R04 fix)

```
cargo nextest run -p ai-brains-adapters -E 'test(scan_brain) or test(history_index) or test(parse_transcript) or test(import_antigravity)'
→ 10 passed

cargo nextest run -p ai-brains-cli -E 'test(agy_hook)'
→ path_case + no_env_hijack + schema pass

cargo clippy -p ai-brains-adapters -p ai-brains-cli --all-targets -- -D warnings
→ OK
```

## Completion decision

Not complete until: full workspace gate green, Codex PASS, PR CI green, squash-merge, conductor/deferred/coordinated closeout, ledger commit.
