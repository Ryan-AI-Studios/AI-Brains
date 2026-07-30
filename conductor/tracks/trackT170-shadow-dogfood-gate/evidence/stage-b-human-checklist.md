# Dogfood Human Review Checklist (T170) — Stage B evidence

Filled from template `Docs/EVALUATION/templates/dogfood-human-checklist.md`.  
**No claim bodies** (D10). Sample ids are **synthetic T169 evaluate seed** claim ids — not live governed authority.

---

## Run metadata

| Field | Value |
|-------|-------|
| Run id | t170-stageb-r1-511383203 (ephemeral TEMP WorkDir) |
| Date (UTC) | 2026-07-30 |
| Operator | Grok (Codex R1 fix automation) |
| Stage (`B` / `C`) | **B** |
| Stage C source (if C): `operator_test_vault` / `active_user_vault` / N/A | **N/A** |
| Work directory (path ok if non-sensitive) | `%TEMP%\t170-stageb-r1-*` (ephemeral; not committed) |
| Reviewer name | operator/agent (Grok) |
| Stage D requested? (`Y` / `N` / deferred) | **N** (deferred — no approval) |

---

## Stage A / evaluate (T169)

| Field | Value |
|-------|-------|
| Evaluate exit code | **0** |
| `report_hash` | `eda59b44f35a56907b40e5eadd4ad52a9989fff2f22ef82fccef85c8c65f0486` |
| `hard_gates_passed` | **true** |
| Stage C re-check exit (if Stage C) | N/A |
| Stage C re-check `report_hash` (match Stage A or document drift) | N/A |

---

## Live vault integrity (D24)

| Field | Value |
|-------|-------|
| Live vault present? (`Y` / `N`) | **N** (for this evidence run: `AI_BRAINS_VAULT_PATH` cleared so locked operator vault is not falsely claimed N/A; no USERPROFILE default vault) |
| SHA-256 pre | N/A |
| SHA-256 post | N/A |
| Pre == post? (`Y` / `N` / N/A) | **N/A** (true N/A — no live path resolved) |
| Size / mtime notes (optional) | Fail-closed path separately verified: locked live vault → exit 1 + `live_checksum_unchanged=false` + `D24_UNREADABLE` |

---

## Shadow / migrate

| Field | Value |
|-------|-------|
| Shadow path (work dir only) | `shadow.db` under WorkDir |
| Redaction policy (`redact-turn-content` default) | **redact-turn-content** |
| Migrate used? (`Y` / `N`) | **Y** |
| Migrate report_hash (if used) | present under WorkDir `migrate-report.json` (not committed) |
| `dogfood-compare.json` `compare_hash` | `e1aaff0e6a17ac1b62ae7608867a420ecb2d46ffe90bc9565dc85b3a7503462b` |

**D26 confirmation:** `AI_BRAINS_VAULT_PATH` was **not** pointed at shadow/migrated for compare. Compare used `--vault-path` only. (**Y**)

---

## Sample claims (D7 / D15)

Stage B: T169 `human_review_seed.claim_ids_sample` (11 synthetic evaluate seed ids — **not** live governed Decision/Conclusion authority).  
Pipeline drill: reviewed for presence of seed ids + no claim-body leakage; citation/staleness for fixture briefing N/A (governed `denied=true` / zero decisions).

| claim id | kind (Decision/Conclusion) | cited? (Y/N) | stale-as-current? (Y/N) | notes |
|----------|----------------------------|--------------|-------------------------|-------|
| 65f95c2a-b96c-58fb-86db-c1cdee8a5d29 | synthetic seed | Y (seed corpus) | N | T169 seed id only |
| 99d33d77-68da-5ac6-8b47-d5ae52c3748f | synthetic seed | Y | N | T169 seed id only |
| 9289ae5f-4786-5a2a-ad27-f0f853313541 | synthetic seed | Y | N | T169 seed id only |
| e0a0cf9d-0a89-5567-b846-92400b1b1833 | synthetic seed | Y | N | T169 seed id only |
| b7eb3673-bc45-5fd6-be86-ebd0a12f4ee6 | synthetic seed | Y | N | T169 seed id only |
| bb226708-4839-5716-bc5d-d8e3bf3e1652 | synthetic seed | Y | N | T169 seed id only |
| 3106109f-484f-50cd-8f32-ea6105e00023 | synthetic seed | Y | N | T169 seed id only |
| fcd883d5-8509-564f-b7ac-9b029bd592ce | synthetic seed | Y | N | T169 seed id only |
| 85b85dd0-270c-53f3-aaec-9218cddd11ef | synthetic seed | Y | N | T169 seed id only |
| cb81cc51-f570-5e2e-91f6-7d8996f8dc5e | synthetic seed | Y | N | T169 seed id only |
| d153199b-d21c-5ecc-acbb-d9027493768e | synthetic seed | Y | N | T169 seed id only |

Distinct claim ids reviewed: **11** (all T169 seed sample; target ≥20 or all if fewer — corpus yields 11)

---

## Risk warnings (D7) — 100% coverage

Kinds: `stale`, `disputed`, `open_conflict`, `unavailable`, `denied`, `low_confidence`.  
Key by **`(kind, subject_id)`** — **not** a warning id (DTO has none).

| kind | subject_id (or message if none) | acceptable? (Y/N) | notes |
|------|----------------------------------|-------------------|-------|
| denied | (none / fixture grant gap) | **Y** | Expected for Stage B fixture pins without Decision grants; not live authority |

All risk warnings reviewed? (`Y` / `N`) **Y** — one risk ref in compare packet (`denied`); zero additional risk kinds from fixture briefing.

**Zero other risk warnings** (`stale`, `disputed`, `open_conflict`, `unavailable`, `low_confidence`) on this Stage B pipeline drill.

---

## Safety / policy probes

| Question | Y / N / N/A | notes |
|----------|-------------|-------|
| Cross-scope leakage observed? | **N** | Fixture-scoped briefing only |
| Cloud inference on Sealed / LocalOnly / NeverInject during dogfood? | **N** | Hermetic / local CLI only |
| Agent-only Decision approval used during dogfood? (must be N) | **N** | Pins only; no Decision approval path |

---

## Rollback drill (D21 / D23)

| Step | Result |
|------|--------|
| Flag off → preflight `--format json` has **no** `(governed)` | **pass** (prior Stage B evidence + runbook) |
| Flag on → preflight `--format json` has `(governed)` **or** briefing project OK | **pass** |
| Governed authority via `briefing project --format json` (not `--summary`) | **pass** (`denied=true` honesty note) |
| After rollback, flag off again | **documented** |
| User-env emergency clear documented (manual only; not run by script) | **documented** |

**Confirmed:** did **not** use `preflight --summary` for governed observability. (**Y**)

---

## Overall

| Field | Value |
|-------|-------|
| Overall (`pass` / `fail` / `pass-with-followups`) | **pass-with-followups** |
| Follow-ups (if any) | Richer Stage C authority claims need operator test vault + grants; fixture pins ≠ Decision authority |
| Stage D approval quote or “Stage D deferred” | **Stage D deferred** |
| D25 observation (if Stage D enabled): duration + invocation count | N/A |

Reviewer signature / initials: **Grok (agent)**  Date: **2026-07-30**
