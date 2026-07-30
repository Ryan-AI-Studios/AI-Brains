# Governed Memory MVP Evaluation (T169 / P9.3)

Trust-first evaluation harness for AI-Brains governed memory: provenance, freshness, scope isolation, citation, erasure honesty, and circularity — **without** mutating the live vault, without network/LLM-as-judge, and without AGPL or proprietary eval frameworks.

## How to run

### Nextest (CI / developer)

```powershell
# Control-plane schema + pure metrics + runner unit tests
cargo nextest run -p ai-brains-control-plane -E 'test(evaluation) | test(scenario__) | test(metric_) | test(report_hash) | test(runner__)'

# Scenario 10 circularity (sources crate)
cargo nextest run -p ai-brains-sources -E 'test(circular) | test(scenario__circular) | test(independent_support)'

# CLI assert_cmd
cargo nextest run -p ai-brains-cli -E 'test(evaluate)'
```

### CLI

```powershell
# All active CP scenarios (1–9); scenario 10 skipped with runner=sources_tests
ai-brains evaluate governed --fixtures fixtures/governed-memory/scenarios

# Filter + write report
ai-brains evaluate governed `
  --fixtures fixtures/governed-memory/scenarios `
  --scenario cold_start_cited_project `
  --report .\evaluate-report.json

# Soft failures become exit 7
ai-brains evaluate governed --fixtures fixtures/governed-memory/scenarios --strict-soft
```

Default fixtures path: `fixtures/governed-memory/scenarios`. Each scenario runs in a **new hermetic tempfile vault** (never the live vault).

### Optional Python

**CLI-only decision for v1:** no required Python wrapper. Operators may shell to `ai-brains evaluate governed` from any stdlib script; a `scripts/evaluate-briefings.py` helper is **not** required for CI. If added later, it must stay stdlib-only (no pip deps).

## Exit codes (E22)

| Code | Name | Meaning | T170 branching |
|------|------|---------|----------------|
| **0** | success | Hard gates passed | Product OK for trust gates |
| **1** | `EXIT_INTERNAL` | Harness/path refuse / unexpected | Eval **tool** broken — infra blocked |
| **2** | usage | Clap usage | Fix flags |
| **6** | `INVALID_PAYLOAD` | Bad scenario schema / unknown program/metric | Fix fixtures |
| **7** | `HARD_GATE_FAILED` | Trust hard gates failed (or `--strict-soft` soft fail) | **Trust regression — product blocked** |

Do **not** treat exit 1 and exit 7 the same: 7 means the harness worked and scored a trust fail.

## Hard vs soft metrics

| Metric | Default gate | Definition |
|--------|--------------|------------|
| `stale_as_current_count` | **Hard** (E9a) | Warning kind ∈ {stale, disputed, rejected, unavailable} whose `subject_id` is also in `decisions[]`/`conclusions[]` current authority. Target **0**. |
| `unauthorized_scope_leakage_count` | **Hard** | Foreign-scope authority claim ids visible without grant. Denied empty packet → 0. Target **0**. |
| `cross_project_leakage_count` | **Hard** (scen 5+) | Beta claim ids visible when briefing as Alpha. Target **0**. |
| `current_claim_count` | **Hard floor** (E23) | `decisions.len() + conclusions.len()` ≥ `min_valid_claims_count` (default 1). Empty “pass” by zero-recall is a hard fail. |
| `uncited_current_claim_count` | Hard when required | Current claims with empty `evidence_handles`. |
| `citation_coverage` | Soft | cited / max(claims,1); **N/A soft-skip** when min_valid=0 and 0 claims. |
| `budget_compliant` | Soft | `used_words <= max_words` (or truncate flags). |
| `latency_ms` | Soft only | Recorded; **excluded from `report_hash`**. Never hard-gated in default CI. |
| `conflict_unmerged` | Hard (scen 4) | Both incompatible claims current without open_conflict/disputed warning. |
| `ce_subject_absent` | Hard (scen 8) | Fail count if wiped subject still in authority (0 = absent = pass). |
| `must_be_absent_present_count` | Hard (scen 3/6/7/8) | Seed-supplied claim ids that must **not** appear in current authority after invalidation/supersession/wipe path. Target **0**. |
| `scope_key_stable` | Hard (scen 9) | Two path spellings → same `scope_key`. |
| `independent_support_false_positive` | Hard (scen 10, sources tests only) | Sources helpers: Echo/unlabeled must not `may_count_as_independent_support`. **Not** a CP metric (schema rejects on CP seed scenarios). |

## Scenario catalog

| # | Id | Status | Stack | Notes |
|---|-----|--------|-------|-------|
| 1 | `cold_start_cited_project` | active | T152 | min claims + citations |
| 2 | `interrupted_task_resumption` | active | T152 | open-work authority floor |
| 3 | `source_edit_stales_conclusion` | active | T149 | dependent not current; stale-as-current=0 |
| 4 | `conflicting_scoped_claims` | active | T150 | conflict_unmerged=0 |
| 5 | `personal_and_cross_project_denied` | active | T151 | Alpha must not see Beta |
| 6 | `human_correction_supersedes` | active | T150 | successor current; citations |
| 7 | `source_unavailable` | active | T149 | unavailable/stale honesty |
| 8 | `erased_evidence_removes_derived` | active | T165 | in-process wipe, **no daemon** |
| 9 | `windows_wsl_repo_alias` | active | path+T151 | same scope_key |
| 10 | `circular_external_writeback` | active | T156 sources tests | `runner: sources_tests`; CP report marks skipped |

Fixtures: `fixtures/governed-memory/scenarios/*.json` (`schema_version: 1`).

## Report artifact

`evaluate-report.json` (stdout and optional `--report`):

- `hard_gates_passed`, per-scenario hard/soft maps, `soft_failures[]`
- `human_review_seed`: ≤20 claim ids sorted by `(scenario_id, claim_id)`; sorted warning ids → **T170** human review
- `report_hash`: hex SHA-256 of canonical JSON with `created_at` and **all** `latency_ms` stripped; scenarios sorted by id
- `limitations[]` (always present)

## Mapping to T185

Release claims must only cite evidence present in evaluate reports / scenario hard gates. Soft metrics and latency are **not** product quality claims. T185 indexes report paths + hashes; this doc’s catalog is the human-readable source of truth for which scenarios exist.

## Limitations

1. **Synthetic fixtures only** — not LoCoMo / LongMemEval / BEAM scores; do not claim superiority from vendor conversational benches.
2. **No LLM-as-judge** — outcome-based packet/state asserts only.
3. **CE honesty** — scenario 8 exercises in-process `wipe_content_envelope`; **no NIST Purge/Destroy claim**.
4. **v1 seeds = Rust programs only** — T168 redacted-shadow vault seeds are **not** required (T170 / later).
5. **Scenario 10** — pure circularity helpers in `ai-brains-sources`; full `propose_conclusion` support-graph wiring remains residual where not live.
6. **OutboundIndex empty in production** — rule 2 circularity is test/fixture-seeded only.
7. **Live vault never mutated** — hermetic tempfile only; refuse report paths that look like vault DBs / reparse.

## License / deps

- No AGPL/GPL eval dependencies.
- Zero new Rust crates for T169 (promoted existing workspace `tempfile` / `ai-brains-crypto` into control-plane deps for hermetic vaults).
- Optional external tools (DeepEval, Ragas, Promptfoo) are **not** product deps and are not required for CI.

## Related tracks

| Track | Relation |
|-------|----------|
| T152 / T149 / T150 / T151 / T156 / T165 | Domain APIs under test |
| **T170** | Live/redacted dogfood; human 20-claim review from `human_review_seed`; exit 7 branching — full runbook: [SHADOW-DOGFOOD-GATE.md](SHADOW-DOGFOOD-GATE.md) |
| **T185** | Evaluation artifact index / claims-with-evidence |

### T170 dogfood pointer

After T169 hard gates pass (exit **0**), operators follow the progressive shadow dogfood gate before any live enablement:

- Runbook: [SHADOW-DOGFOOD-GATE.md](SHADOW-DOGFOOD-GATE.md) (Stages A–D; D1–D26; D26 = `--vault-path` not env for shadow)
- Checklist: [templates/dogfood-human-checklist.md](templates/dogfood-human-checklist.md)
- Orchestrator: `scripts/dogfood-shadow.ps1`
- Compare: `ai-brains dogfood compare --governed … --legacy … --out …`
