# T170 Stage B evidence (sanitized)

## How this evidence was produced

1. **Stage A** — `ai-brains evaluate governed --fixtures fixtures/governed-memory/scenarios` from the worktree root (via `scripts/dogfood-shadow.ps1`). Record exit code, `report_hash`, and `hard_gates_passed` in `stage-a-evaluate-summary.json` (sanitized; full report not committed).
2. **Stage B rehearsal** — local run of `scripts/dogfood-shadow.ps1 -WorkDir <temp>` which:
   - Creates `fixture.db` via `init` + pin under WorkDir; persists `fixture-project-id.txt`
   - Passes `--project-id` to `briefing project` (R1-01)
   - Shadows with default redaction; optional migrate under WorkDir
   - Captures governed/legacy JSON via **`--vault-path` only** (D26), **BOM-less UTF-8** (R1-05)
   - Emits `dogfood-compare.json` including `--migrate-report` when present (R1-07)
   - **Never** sets `AI_BRAINS_VAULT_PATH` to shadow
   - **Never** Stage D / User-level env
3. **D24** — live vault SHA-256 pre/post when readable; if locked → **N/A** (warn, continue). Mismatch still fails when both hashes exist.
4. **Rollback probe** — flag off/on with `preflight --format json` `(governed)` marker + `briefing project` for authority — **not** `--summary` (D21).
5. Human checklist fields filled in `stage-b-notes.md` (no full claim bodies, no PII).

## Artifacts in this directory

| File | Contents |
|------|----------|
| `stage-a-evaluate-summary.json` | Stage A exit / report_hash / hard gates (no scenario claim bodies) |
| `stage-b-compare-summary.json` | compare_hash, hard_checks, counts, path basenames only |
| `stage-b-notes.md` | Commands, D24, rollback probe, honesty notes |

## Deferred

| Stage | Status | Owner | Reason |
|-------|--------|-------|--------|
| **C** | Deferred | Operator | Operator test vault not available in CI/worktree automation |
| **D** | Deferred | User | No live enablement approval; scripts refuse Stage D |

## D26 enforcement

`scripts/dogfood-shadow.ps1` asserts process `AI_BRAINS_VAULT_PATH` is never equal to shadow/migrated paths and always passes `--vault-path` (global, before subcommand) for briefing/preflight/compare inputs.
