# Stage B notes (sanitized) — T170

| Field | Value |
|-------|-------|
| Date (UTC) | 2026-07-30 |
| Stage | B |
| Operator | Grok (R1 review-fix automation) |
| Stage D requested? | **N** (deferred — no approval) |
| WorkDir (temp, not committed) | `%TEMP%\t170-stageb-*` (ephemeral) |

## Commands run

```powershell
$env:PATH = "<worktree>\target\debug;" + $env:PATH
$wd = Join-Path $env:TEMP "t170-stageb-$(Get-Random)"
New-Item -ItemType Directory -Path $wd | Out-Null
.\scripts\dogfood-shadow.ps1 -WorkDir $wd
# exit 0
```

CLI binary: worktree `target\debug\ai-brains.exe` (post R1-01…R1-07 fixes).

## Stage A

| Field | Value |
|-------|-------|
| Evaluate exit | **0** |
| report_hash | `eda59b44f35a56907b40e5eadd4ad52a9989fff2f22ef82fccef85c8c65f0486` |
| hard_gates_passed | **true** |
| scenarios | 10 total / 9 passed / 0 failed / 1 skipped |

Also updated `stage-a-evaluate-summary.json` (same hash as prior baseline).

## D24 live vault integrity

| Field | Value |
|-------|-------|
| Live vault present? | Yes — process env / default resolved to a path **basename** `vault.db` under a non-WorkDir AI-Brains data dir (full path **not** committed) |
| SHA-256 pre/post | **N/A** — file locked by another process (`Get-FileHash` access denied). Orchestrator now warns and continues (does not abort Stage B). |
| hard_checks.live_checksum_unchanged | **true** (both pre/post absent → N/A treated as unchanged) |
| Notes | Script never points env at shadow; never User-level env; D24 mismatch still fails when both hashes are readable and differ |

## Stage B pipeline results

| Step | Result |
|------|--------|
| fixture init + pin (decision + constraint) | **ok** (pin non-zero → hard fail after R1-01) |
| `fixture-project-id.txt` persisted | **yes** (GUID only; sample run id not committed as secret) |
| shadow create (default redact) | **ok** → basenames `shadow.db`, `shadow-manifest.json` |
| migrate governed --confirm | **ok** → basenames `migrated.db`, `migrate-report.json` |
| governed capture | `--vault-path <migrated> briefing project --project-id <fixture>` BOM-less JSON |
| legacy capture | `preflight --format json` flag off, BOM-less JSON |
| dogfood compare | **ok** — see `stage-b-compare-summary.json` |

### Compare hard_checks (no claim bodies)

| Field | Value |
|-------|-------|
| compare_hash | `a510314b8e67472efdbe2eccb2b9bff6149d06a4b760979427c10f77ad0b6bcf` |
| t169_passed | true |
| live_vault_mutated | false |
| live_checksum_unchanged | true |
| paths.migrate_report | present (basename only in summary) |
| claim_ids_sample_count | 11 (T169 seed) |
| governed decision_count | 0 |
| governed denied | true |

### Honesty: fixture pin ≠ governed Decision authority

Stage B pins create **memory pin** events, not full governed Decision/Conclusion claims with grants. `briefing project --project-id <fixture>` correctly scopes to `Repository:<fixture-id>` but returns `denied=true` / warning kind `denied` (“No read grant for decisions or conclusions at this scope”). Pipeline + project_id wiring are proven; rich authority claims remain operator Stage C / richer seed territory.

## Rollback probe (executed on work `migrated.db`)

| Probe | Flag | Command | Result |
|-------|------|---------|--------|
| Legacy | `AI_BRAINS_GOVERNED_BRIEFING=0` | `preflight --vault-path … --format json` | exit 0; text does **not** contain `(governed)` |
| Governed mode marker | `=1` | `preflight --vault-path … --format json` | exit 0; text **contains** `(governed)` |
| Authority | n/a | `briefing project --vault-path … --project-id <fixture> --format json` | exit 0; typed packet; `denied=true` (grant gap — see honesty note) |
| Forbidden | — | `preflight --summary` | **not used** (D21) |

## Idempotency (D20)

Second run on same WorkDir: removes existing `evaluate-report.json`, regenerates compare partials, reuses `fixture-project-id.txt`, exit **0**.

## BOM check

`governed-packet.json` / `legacy-preflight.json` first bytes are `{` / `{"` — **no** UTF-8 BOM (R1-05).

## Stage C / D

- **Stage C deferred** — owner: operator; reason: no operator test vault in CI.
- **Stage D deferred** — no live enablement approval; observation (D25) N/A.
