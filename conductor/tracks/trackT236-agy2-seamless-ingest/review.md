# T236 Review Log — AGY 2 seamless ingest

**Track:** T236  
**Branch:** `feat/T236-agy2-seamless-ingest`  
**Ledger TX:** `a4f3806b-caef-47a4-bc43-8131ad43fe25`  
**Status:** Codex **PASS WITH DEFERRED P3** (final engineering gate); process closeout after PR merge  

## Scope

Wrapper stdout SOOT (F8); history.jsonl binding (F9–F13); shared step/legacy parse + turn-id SOOT (F1–F2); F3 env narrow; F29 transcript_full; F30 source_meta path key; stats/`--force`; F17 re-summarize OR; docs honesty.

## Reviewers / rounds

| Round | Reviewer | Verdict | Notes |
|-------|----------|---------|-------|
| IR1 | explore subagent | **FAIL** | P2 dual overview+transcript; AC15; P3 AC6/AC17/capability |
| IR1 fix | orchestrator | — | Prefer transcript over overview; AC17 hook case test |
| CX1 | Codex | **FAIL** | P2 AC6 unproven; P3 capability Partial; process notes |
| CX1 fix | orchestrator | — | AC6 turn project_id + scoped recall; capability Full |
| CX2 | Codex | **PASS WITH DEFERRED P3** | Only sleep-based re-list timing residual |

## Findings disposition

| ID | Sev | Status | Disposition |
|----|-----|--------|-------------|
| F-T236-R01 | P2 | `verified_fixed` | Prefer transcript over overview in discovery |
| F-T236-R02 | P2 | `verified_fixed` | Full gate green (fmt/clippy/nextest 2304/deny/audit) |
| F-T236-R03 / CX AC6 | P2→fixed | `verified_fixed` | turn_projection.project_id + project-scoped recall without --global |
| F-T236-R04 | P3 | `verified_fixed` | AC17 hook path-case test |
| F-T236-R05 | P3 | `verified_fixed` | CapabilityLevel::Full |
| F-T236-R06 | P3 | deferred | Batch query Err→None pre-existing fail-open |
| F-T236-R07 | P3 | deferred | BrainLog harness id vs live agy harness → T239 residual |
| F-T236-R08 / CX P3 | P3 | deferred | sleep in re-list test for timestamp ordering |

## Gate evidence

```
cargo fmt --check OK
cargo clippy --workspace --all-targets -- -D warnings OK
cargo nextest run --workspace → 2304 passed (1 skipped)
cargo deny check OK
cargo audit → allowed warnings only
ledgerful verify --scope fast OK (full gate steps)
```

## Completion decision

Engineering DoD met + fresh Codex PASS WITH DEFERRED P3. Track **Completed** after CI green squash-merge + conductor/deferred/coordinated/pins/ledger closeout.
