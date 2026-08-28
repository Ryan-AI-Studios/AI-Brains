# T312 Review Log — Recall rank v3

**Track:** T312-RecallRankV3  
**Category:** FEATURE / UX / RETRIEVAL  
**FEATURE TX:** `7f7e99bb-7dcb-4c84-bb5e-b3ed5dd9fdd3`  
**Branch:** `track/T312-recall-rank-v3`  
**Commits:** Red `11cbd44` (hermetics fail on T285) → Green (this closeout)

## Scope

Authority-OR fill when AND-retain empty (F8); verbose-Other −16 (F6/F7); ATX heading token detector (F5); verbose-Other seed skip (F10). KIND / floors / depth frozen. No `project.rs` / `sync.rs` / CLI `preflight.rs` / `pin.rs` write / `hybrid.rs` floors edits.

## Internal review (R1)

| AC | Status | Evidence |
|----|--------|----------|
| AC1–AC6, AC10–AC17 | **met** | unit + hermetic PASS |
| AC7 T285 / T217 | **met** | stay-green PASS |
| AC12 recall + search | **met** | split tests (no for-loop) |
| AC18 list recency | **stay-green** | store path untouched |
| Manual F42 canary | **met** | `cargo run` hermetic vault → pin #1 |

### Findings (internal)

| id | severity | description | status |
|----|----------|-------------|--------|
| R1-F1 | low | `match_query` clippy `too_many_arguments` | `verified_fixed` — allow + F8 comment |
| R1-F2 | low | PATH pre-T312 until install | `deferred` R2 — F21 |

## Cross-model (codex gpt-5.6-sol) — `review.codex.md`

| Finding | Disposition |
|---------|-------------|
| P1 structured synth not boosted | **out_of_scope / soft R1** — spec §11 F6-by-design; objective pins are DoD; synth is residual |
| P1 gates/provenance pending | **validated** — completing Phase 5–6 this session |
| P1 Red→Green history missing | **validated → fixed** — red hermetic commit `11cbd44` (AC5 FAIL proven); green follows |
| P2 F7/F39/F40 weak discrimination | **validated → fixed** — AC17 score-separated; 799/800 boundary; F40 nonempty-retain test |
| P3 AC12 for-loop | **validated → fixed** — split `recall__` / `search__` tests |

## Gates

- Targeted unit + hermetic T312: PASS
- T285 / T217 stay-green: PASS
- `cargo fmt` + targeted clippy: PASS
- Unrelated `exit_contract` graph feature-off: FAIL under parallel suite once; **PASS alone** (not T312)
- Full `dev-check.ps1` + `ledgerful verify --scope full`: re-run after green
- Manual canary: pin #1 for `t312or backend`

## Soft residuals → `deferred.md`

R1 long Other synths demoted · R2 PATH · R3 live corpus may lack OR pin · R4 raw BM25 display · R5 closed ATX token set · R6 semantic floor freeze
