# Stage B notes (sanitized) — T170

| Field | Value |
|-------|-------|
| Date (UTC) | 2026-07-30 |
| Stage | B |
| Operator | Grok (Codex R1 fix automation) |
| Stage D requested? | **N** (deferred — no approval) |
| WorkDir (temp, not committed) | `%TEMP%\t170-stageb-r1-*` (ephemeral) |
| Human checklist | [stage-b-human-checklist.md](./stage-b-human-checklist.md) |

## Commands run

```powershell
$env:PATH = "<worktree>\target\debug;" + $env:PATH
# Honest D24 N/A: do not resolve locked operator live vault
Remove-Item Env:AI_BRAINS_VAULT_PATH -ErrorAction SilentlyContinue
$wd = Join-Path $env:TEMP "t170-stageb-r1-$(Get-Random)"
New-Item -ItemType Directory -Path $wd | Out-Null
.\scripts\dogfood-shadow.ps1 -WorkDir $wd
# exit 0 (D24 status=na)
```

CLI binary: worktree `target\debug\ai-brains.exe` (Codex R1 P1/P2 fixes).

## Stage A

| Field | Value |
|-------|-------|
| Evaluate exit | **0** |
| report_hash | `eda59b44f35a56907b40e5eadd4ad52a9989fff2f22ef82fccef85c8c65f0486` |
| hard_gates_passed | **true** (missing field would **throw**, not default true) |
| scenarios | 10 total / 9 passed / 0 failed / 1 skipped |
| baseline file | WorkDir `stage-a-report-hash.txt` |

Also updated `stage-a-evaluate-summary.json` (same hash as prior baseline).

## D24 live vault integrity

| Field | Value |
|-------|-------|
| Live vault present? | **N** for evidence run (env cleared; no USERPROFILE default vault) → honest **N/A** |
| SHA-256 pre/post | **N/A** |
| hard_checks.live_checksum_unchanged | **true** (N/A only when no live path) |
| hard_checks.live_checksum_verified | **false** |
| Fail-closed check | With `AI_BRAINS_VAULT_PATH` → locked vault: exit **1**, `live_checksum_unchanged=false`, limitations `D24_UNREADABLE` |

## Stage B pipeline results

| Step | Result |
|------|--------|
| fixture init + pin (decision + constraint) | **ok** |
| `fixture-project-id.txt` persisted | **yes** |
| shadow create (default redact) | **ok** |
| migrate governed --confirm | **ok** |
| governed capture | `--vault-path` + `--project-id` BOM-less JSON |
| legacy capture | `preflight --format json` flag off, BOM-less JSON |
| dogfood compare | **ok** — see `stage-b-compare-summary.json` |
| human checklist | **filled** — `stage-b-human-checklist.md` |

### Compare hard_checks (no claim bodies)

| Field | Value |
|-------|-------|
| compare_hash | `e1aaff0e6a17ac1b62ae7608867a420ecb2d46ffe90bc9565dc85b3a7503462b` |
| t169_passed | true |
| live_vault_mutated | false |
| live_checksum_verified | false |
| live_checksum_unchanged | true (honest N/A) |
| paths.migrate_report | present (basename only in summary) |
| claim_ids_sample_count | 11 (T169 seed) |
| governed decision_count | 0 |
| governed denied | true |

### Honesty: fixture pin ≠ governed Decision authority

Stage B pins create **memory pin** events, not full governed Decision/Conclusion claims with grants. `briefing project --project-id <fixture>` correctly scopes but returns `denied=true`. Pipeline + project_id wiring are proven; rich authority claims remain operator Stage C / richer seed territory.

## Rollback probe (executed on work `migrated.db`)

| Probe | Flag | Command | Result |
|-------|------|---------|--------|
| Legacy | `AI_BRAINS_GOVERNED_BRIEFING=0` | `preflight --vault-path … --format json` | exit 0; text does **not** contain `(governed)` |
| Governed mode marker | `=1` | `preflight --vault-path … --format json` | exit 0; text **contains** `(governed)` |
| Authority | n/a | `briefing project --vault-path … --project-id <fixture> --format json` | exit 0; typed packet; `denied=true` |
| Forbidden | — | `preflight --summary` | **not used** (D21) |

## Idempotency (D20)

Second run on same WorkDir: removes existing `evaluate-report.json`, regenerates compare partials, reuses `fixture-project-id.txt`.

## BOM check

`governed-packet.json` / `legacy-preflight.json` first bytes are `{` — **no** UTF-8 BOM.

## Stage C / D

- **Stage C deferred** — owner: operator; reason: no operator test vault in CI. Orchestrator now supports `-ProjectId` + `stage-a-report-hash.txt` drift warn.
- **Stage D deferred** — no live enablement approval; observation (D25) N/A.
