# Dogfood Human Review Checklist (T170)

Copy this template into the track `evidence/` directory (or operator notes). Fill every **Required** field. Do **not** paste full claim bodies or turn content into shared logs (D10).

---

## Run metadata

| Field | Value |
|-------|-------|
| Run id | |
| Date (UTC) | |
| Operator | |
| Stage (`B` / `C`) | |
| Stage C source (if C): `operator_test_vault` / `active_user_vault` / N/A | |
| Work directory (path ok if non-sensitive) | |
| Reviewer name | |
| Stage D requested? (`Y` / `N` / deferred) | |

---

## Stage A / evaluate (T169)

| Field | Value |
|-------|-------|
| Evaluate exit code | |
| `report_hash` | |
| `hard_gates_passed` | |
| Stage C re-check exit (if Stage C) | |
| Stage C re-check `report_hash` (match Stage A or document drift) | |

---

## Live vault integrity (D24)

| Field | Value |
|-------|-------|
| Live vault present? (`Y` / `N`) | |
| SHA-256 pre | |
| SHA-256 post | |
| Pre == post? (`Y` / `N` / N/A) | |
| Size / mtime notes (optional) | |

If no live vault: write **N/A** and do not invent hashes.

---

## Shadow / migrate

| Field | Value |
|-------|-------|
| Shadow path (work dir only) | |
| Redaction policy (`redact-turn-content` default) | |
| Migrate used? (`Y` / `N`) | |
| Migrate report_hash (if used) | |
| `dogfood-compare.json` `compare_hash` | |

**D26 confirmation:** `AI_BRAINS_VAULT_PATH` was **not** pointed at shadow/migrated for compare. Compare used `--vault-path` only. (`Y` / `N`)

---

## Sample claims (D7 / D15)

Stage B: use T169 `human_review_seed.claim_ids_sample`.  
Stage C: stratified Decision/Conclusion sample (up to 5 each, fill to 20 by sorted id).

For each sample claim:

| claim id | kind (Decision/Conclusion) | cited? (Y/N) | stale-as-current? (Y/N) | notes |
|----------|----------------------------|--------------|-------------------------|-------|
| | | | | |
| | | | | |
| | | | | |
| | | | | |
| | | | | |
| | | | | |
| | | | | |
| | | | | |
| | | | | |
| | | | | |
| | | | | |
| | | | | |
| | | | | |
| | | | | |
| | | | | |
| | | | | |
| | | | | |
| | | | | |
| | | | | |
| | | | | |

Distinct claim ids reviewed: ____ (target ≥20 or all if fewer)

---

## Risk warnings (D7) — 100% coverage

Kinds: `stale`, `disputed`, `open_conflict`, `unavailable`, `denied`, `low_confidence`.  
Key by **`(kind, subject_id)`** — **not** a warning id (DTO has none).

| kind | subject_id (or message if none) | acceptable? (Y/N) | notes |
|------|----------------------------------|-------------------|-------|
| | | | |
| | | | |
| | | | |

All risk warnings reviewed? (`Y` / `N`) ____

---

## Safety / policy probes

| Question | Y / N / N/A | notes |
|----------|-------------|-------|
| Cross-scope leakage observed? | | |
| Cloud inference on Sealed / LocalOnly / NeverInject during dogfood? | | |
| Agent-only Decision approval used during dogfood? (must be N) | | |

---

## Rollback drill (D21 / D23)

| Step | Result |
|------|--------|
| Flag off → preflight `--format json` has **no** `(governed)` | |
| Flag on → preflight `--format json` has `(governed)` **or** briefing project OK | |
| Governed authority via `briefing project --format json` (not `--summary`) | |
| After rollback, flag off again | |
| User-env emergency clear documented (manual only; not run by script) | |

**Confirmed:** did **not** use `preflight --summary` for governed observability. (`Y`)

---

## Overall

| Field | Value |
|-------|-------|
| Overall (`pass` / `fail` / `pass-with-followups`) | |
| Follow-ups (if any) | |
| Stage D approval quote or “Stage D deferred” | |
| D25 observation (if Stage D enabled): duration + invocation count | |

Reviewer signature / initials: _______________  Date: _______________
