# T316 Review Log — memory list preview + forget nudge

**Track:** T316-MemoryListPreview  
**FEATURE TX:** `50c73816-3152-499e-bee9-1b5aeb7b0aec`  
**Branch:** `track/T316-memory-list-preview`  
**Date:** 2026-08-29

## Scope

Chrome-skip after T287 envelope in `preview_line` (walk cap 8; authority never skipped; all-chrome fallback). Drop T216 F36 nonempty-human `eprintln!` forget hint. after_help one sentence. Docs CAPABILITIES / OPERATIONS / CHANGELOG. Hermetic flips AC8/AC9/AC11/AC14. Inherit-only for forget/graph/briefing.

## Internal review (R1)

| id | severity | description | status | evidence |
|----|----------|-------------|--------|----------|
| R1-AC1–7/19 | — | Unit/rstest chrome-skip + walk cap + fence→Decision | verified_fixed | nextest 27/27 filter incl. units |
| R1-AC8/9 | — | F36 stderr omitted nonempty list + forgotten | verified_fixed | hermetics PASS |
| R1-AC11 | — | JSON preview skips chrome; nine keys; no new keys | verified_fixed | hermetic + exact key assert |
| R1-AC14 | — | after_help chrome-skip + no forget hint | verified_fixed | hermetic PASS |
| R1-AC15 | — | Docs | verified_fixed | CAPABILITIES / OPERATIONS / CHANGELOG |
| R1-AC16 | — | Isolation empty diff on forbidden paths | verified_fixed | `git diff --name-only` vs isolation set empty |
| R1-AC17 | — | Manual cargo run; stderr empty; pass-with-observed-data | verified_fixed | stderr empty; previews contentful / R1-1 recency possible |
| R1-F25 | — | production net vs origin/main | verified_fixed | prod lines 631→694 (**+63** &lt;80); tests inflate total |
| R1-scratch | low-info | AC17 stdout/stderr scratch at repo root | verified_fixed | removed before commit |
| R1-AC11-keys | low-info | AC11 initially omitted `project_id` exact-set | verified_fixed | aligned to nine keys + forbid new keys |

**R1 verdict:** clean &gt;low.

## Cross-model (F22) — Codex `review.codex.md`

| id | severity | disposition |
|----|----------|-------------|
| CX-P1 gates incomplete | high (process) | **validated then cleared** — expected mid-implement; full gate + publish still required before Complete |
| CX-P2 review.md gitignored | medium | **validated** — force-add `review.md` / `review.codex.md` like prior tracks (`git add -f`) |
| CX-P3 EOF blank lines plan/spec | low-info | **verified_fixed** — trimmed trailing blanks; `git diff --check` clean |

Codex product verdict (behavior): AC1–AC19 / F36 / isolation / F25 / T326 separation supported. Overall CX file said FAIL only for incomplete publish gates — not a product defect.

**CX2:** re-run after full gate green if needed; otherwise treat as PASS WITH DEFERRED P3 (soft residuals in `deferred.md`).

## Manual evidence (AC17)

```text
cargo run -p ai-brains-cli -- memory list --limit 5 --format human
# stderr: empty (no forget --memory-id / forget --restore)
# stdout: Showing 5 of 4574; previews contentful (not ## Objective-only where body exists)
# T287 R1-1 may still recency-fill first page — F27 pass-with-observed-data

cargo run -p ai-brains-cli -- memory list --format json --limit 1
# keys T216; items[0].preview contentful
```

## Targeted gates (pre-full)

- `cargo fmt -p ai-brains-cli` OK
- `cargo clippy -p ai-brains-cli --all-targets -- -D warnings` OK
- nextest filter (units + hermetics + stay-green): **27 passed**
- AC11 key align: PASS
- Unrelated: first workspace nextest had **10 timeouts** in recall_* hermetics under concurrent load; serial `--test-threads=1` re-run **10/10 PASS** (35–84s each). Not T316; do not broaden-fix.

## Full gate

- `cargo fmt --check` OK
- `cargo clippy --workspace --all-targets -- -D warnings` OK
- `cargo nextest run --workspace` with `NEXTEST_TEST_THREADS=2`: **3633** run, exit **0** (prior default-parallel timeouts on recall_* were load flakes; serial/threads=2 PASS)
- `cargo deny check` OK
- `cargo audit` OK (warnings only, allowed)
- Soft residual: default high-parallelism local flakiness on slow recall hermetics — deferred, not T316 product

## Completion decision

**PASS WITH DEFERRED P3** — product DoD met; soft residuals in `conductor/deferred.md`; publish Phase 6 next.
