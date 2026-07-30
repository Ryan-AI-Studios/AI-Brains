# T170 Stage B evidence (sanitized)

## How this evidence was produced

1. **Stage A** — `ai-brains evaluate governed --fixtures fixtures/governed-memory/scenarios` from the worktree root (via `scripts/dogfood-shadow.ps1`). Record exit code, `report_hash`, and `hard_gates_passed` in `stage-a-evaluate-summary.json` (sanitized; full report not committed). Missing `hard_gates_passed` / unparseable report aborts.
2. **Stage B rehearsal** — local run of `scripts/dogfood-shadow.ps1 -WorkDir <temp>` which:
   - Creates `fixture.db` via `init` + pin under WorkDir; persists `fixture-project-id.txt`
   - Passes `--project-id` to `briefing project` (R1-01)
   - Shadows with default redaction; optional migrate under WorkDir
   - Captures governed/legacy JSON via **`--vault-path` only** (D26), **BOM-less UTF-8** (R1-05)
   - Emits `dogfood-compare.json` including `--migrate-report` when present (R1-07)
   - **Never** sets `AI_BRAINS_VAULT_PATH` to shadow
   - **Never** Stage D / User-level env
3. **D24** — live vault SHA-256 pre/post when a live path resolves:
   - **No live path** → true N/A (`live_checksum_unchanged=true`, `live_checksum_verified=false`)
   - **Live path + both hashes equal** → pass
   - **Live path + unreadable/partial/mismatch** → **fail-closed** (exit non-zero; compare `live_checksum_unchanged=false` + `D24_UNREADABLE`)
   - Evidence re-run cleared locked `AI_BRAINS_VAULT_PATH` for honest N/A; fail-closed path was smoke-tested separately
4. **Rollback probe** — flag off/on with `preflight --format json` `(governed)` marker + `briefing project` for authority — **not** `--summary` (D21).
5. Human checklist: `stage-b-human-checklist.md` (synthetic T169 seed ids; no claim bodies).

## Artifacts in this directory

| File | Contents |
|------|----------|
| `stage-a-evaluate-summary.json` | Stage A exit / report_hash / hard gates (no scenario claim bodies) |
| `stage-b-compare-summary.json` | compare_hash, hard_checks, counts, path basenames only |
| `stage-b-notes.md` | Commands, D24, rollback probe, honesty notes |
| `stage-b-human-checklist.md` | Filled D7/D15 human review checklist (Codex R1 P1-04) |

## Deferred

| Stage | Status | Owner | Reason |
|-------|--------|-------|--------|
| **C** | Deferred | Operator | Operator test vault not available in CI/worktree automation; use `-ProjectId` when multi-project |
| **D** | Deferred | User | No live enablement approval; scripts refuse Stage D |

## D26 enforcement

`scripts/dogfood-shadow.ps1` asserts process `AI_BRAINS_VAULT_PATH` is never equal to shadow/migrated paths and always passes `--vault-path` (global, before subcommand) for briefing/preflight/compare inputs.
