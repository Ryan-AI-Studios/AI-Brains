# T170 Stage B evidence (sanitized)

## How this evidence was produced

1. **Stage A** — `ai-brains evaluate governed --fixtures fixtures/governed-memory/scenarios` from the worktree root. Record exit code, `report_hash`, and `hard_gates_passed` in `stage-a-evaluate-summary.json` (sanitized; full report may be large and is not required in git).
2. **Stage B rehearsal** — optional local run of `scripts/dogfood-shadow.ps1 -WorkDir <temp>` which:
   - Creates `fixture.db` via `init` + pin under WorkDir
   - Shadows with default redaction
   - Captures governed/legacy JSON via **`--vault-path` only** (D26)
   - Emits `dogfood-compare.json`
   - **Never** sets `AI_BRAINS_VAULT_PATH` to shadow
   - **Never** Stage D / User-level env
3. **D24** — live vault SHA-256 pre/post when a live vault is resolvable; otherwise **N/A**.
4. Human checklist fields filled in `stage-b-notes.md` (no full claim bodies, no PII).

## Deferred

| Stage | Status | Owner | Reason |
|-------|--------|-------|--------|
| **C** | Deferred | Operator | Operator test vault not available in CI/worktree automation |
| **D** | Deferred | User | No live enablement approval; scripts refuse Stage D |

## D26 enforcement

`scripts/dogfood-shadow.ps1` asserts process `AI_BRAINS_VAULT_PATH` is never equal to shadow/migrated paths and always passes `--vault-path` for briefing/preflight/compare inputs.
