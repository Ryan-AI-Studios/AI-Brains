# Stage B notes (sanitized) — T170

| Field | Value |
|-------|-------|
| Date (UTC) | 2026-07-29 |
| Stage | B |
| Operator | Grok (implementer automation) |
| Stage D requested? | **N** (deferred — no approval) |

## Stage A

| Field | Value |
|-------|-------|
| Evaluate exit | See `stage-a-evaluate-summary.json` |
| report_hash | See summary |
| hard_gates_passed | See summary |

## D24 live vault integrity

| Field | Value |
|-------|-------|
| Live vault present? | Possibly (env / default path may resolve); implementer probe saw a vault file **locked** by another process so SHA could not be read at evidence-write time |
| SHA-256 pre/post | **N/A for this evidence snapshot** (lock) — orchestrator still computes and fails on mismatch when readable |
| Notes | Script never points env at shadow; hash is of resolved live path only; dogfood never opens live for write |

## Pipeline exercised

- [x] Docs runbook + checklist + OPERATIONS section
- [x] `scripts/dogfood-shadow.ps1` (D26 refuse, Stage D refuse, D24 pre/post)
- [x] `ai-brains dogfood compare` unit + integration tests
- [ ] Full local Stage B shadow on implementer machine (optional; CI does not require)

## Human review seed (Stage B)

Uses T169 `human_review_seed.claim_ids_sample` when evaluate report is present (may be &lt;20 — document). Full per-claim sign-off is operator-owned; this track records the automated seed path and template.

## Rollback drill (documented)

Per runbook §8 / script helper output:

1. Flag off → `preflight --format json` no `(governed)`
2. Flag on → `(governed)` probe **or** `briefing project --format json`
3. **Never** `preflight --summary` for governed
4. User-env emergency clear is **manual only**

## Stage C / D

- **Stage C deferred** — owner: operator; reason: no operator test vault in CI.
- **Stage D deferred** — no live enablement approval; observation (D25) N/A.
