# T166 Internal Review R1

**Reviewer:** primary (read-only)  
**Scope:** `feature/T166-class-based-retention` vs base `2f3be7d`  
**Authority:** `spec.md` R1–R16 + DoD §15, `plan.md`, implementation files listed in review request  
**Date:** 2026-07-29  

## Verdict: NEEDS_FIX

Core class matrix, dry-run/confirm, T165 wipe reuse, RetentionApplied audit, R11 pin hold (sole-subject case), R15 cascade, R16 7d orphans, and the §13 test suite are largely in place. Blockers are production CE apply using `AllowAllPolicy` / local in-process wipe (security + T165 parity), incomplete R13 “CE wins when join known,” and missing nightly class dry-run logging.

No production `unwrap()` / `expect()` / `panic!` in the T166 retention modules. No parallel `destroy_content_key_wrap` outside the T165 wipe path. Zero new Cargo dependencies observed in control-plane / retention surfaces. Capture independence preserved (no models/graph on plan/apply).

---

## DoD / R-lock matrix

| Item | Status | Evidence |
|------|--------|----------|
| **R1** Dry-run first; apply needs confirm | **Met** | `plan_retention` is read-only; `apply_retention` refuses when `dry_run \|\| !confirm` (`class_based_retention.rs` ~702–706); CLI refuses without `--confirm` (`commands/retention.rs` 49–66); test `retention_apply__without_confirm__refused` |
| **R2** One CE path = T165 wipe only | **Met** (code path) / **Partial** (CLI surface) | Apply CE loop calls only `wipe_content_envelope` (~785); `rg destroy_content_key_wrap` in crates shows store + T165 wipe + tests only — no retention parallel destroy. **But** CLI uses local wipe + `AllowAllPolicy` instead of daemon/production policy (see F-001) |
| **R3** Legacy ≠ CE labeling | **Met** | Mechanisms use `projection_delete` not CE; honesty `RETENTION_HONESTY_LEGACY_NOT_CE`; nightly log says “not cryptographic erasure”; OPERATIONS table |
| **R4** No plaintext in reports | **Met** | Report DTOs: counts, mechanisms, truncated `sample_ids`, notes; tests assert body/ciphertext strings absent; store scans select ids only |
| **R5** Canonical classes; unknown → unclassified skip | **Met** | `CANONICAL_CLASSES` + `is_canonical_class` in contracts; `resolve_blob_class` / unclassified skip on apply; test `retention_plan__unknown_class__skip` |
| **R6** No age-wipe active approved decisions | **Met** | `list_disposable_decisions` only `Revoked`/`Superseded`; test `retention_plan__approved_decision_active__skip` |
| **R7** Nightly CE opt-in only (default off) | **Met** (safety) / **Partial** (opt-in semantics) | Default `apply_ce_on_nightly: false`; nightly never calls CE even if env set (warn + projection-only). Safer than silent CE. Env flag does **not** enable CE (doc/help can overstate “opt-in”) — F-005 |
| **R8** Capture independence | **Met** | Plan/apply deps: store + CP wipe + events; no models/graph imports in retention modules |
| **R9** Zero new deps | **Met** | CP `Cargo.toml` unchanged dependency set for T166 surface (chrono already present) |
| **R10** Event log append-only | **Met** | Projection `DELETE` only; apply appends `RetentionApplied` / CE events; no event rewrite |
| **R11** Pinned held from age apply | **Partial** | Sole-subject pinned CE candidates → `held` (test `retention_plan__pinned_memory__held`). Multi-subject: hold only if **all** subjects pinned (F-003). Stream A `memory_legacy` never scanned (v1 none-auto; docs slightly overclaim) |
| **R12** RetentionApplied on apply | **Met** | `append_retention_applied` after apply work; dry-run does not; test `retention_apply__appends_retention_applied_event`; payload wired in events crate |
| **R13** No double-count; CE wins when join known | **Partial** | Same `content_key_id` de-duped (test). Turn join suppression only when stream-B mechanism is **`ce_wipe`**, not for any known join (held/skip/within-horizon still allow stream-A projection_delete) — F-002 |
| **R14** Cooldown = terminal `updated_at` | **Met** | Documented in store `retention.rs` module docs + candidate notes; decisions/reviews filter on `updated_at` |
| **R15** Hierarchy parents marked for resynthesis | **Met** | `mark_parents_for_resynthesis` → `status='stale'`; test `retention_apply__hierarchy_parent_marked_for_resynthesis` |
| **R16** Orphan horizon 7d | **Met** | Default `orphan_envelope_days: 7`; age ≥ horizon → `ce_wipe`; younger → skip (seal race); test uses 14d old wrap |
| **DoD** Taxonomy + streams in code/docs | **Met** | Contracts + OPERATIONS class matrix + stream join convention |
| **DoD** Dry-run plan CLI + R4 honesty | **Met** | `ai-brains retention plan`; warnings family present |
| **DoD** Apply confirm: projection + CE via T165 | **Partial** | Works in-process with AllowAll; not production-policy/daemon-aligned (F-001) |
| **DoD** R11/R12/R15/R16 | **Partial** | R12/R15/R16 Met; R11 Partial (F-003) |
| **DoD** R6 / R7 | **Met** / **Partial** | R6 Met; R7 Partial (F-005) |
| **DoD** OPERATIONS | **Met** | Class matrix, dual-path, residuals, env knobs, join convention |
| **DoD** Phase 8 rollup evidence filled | **Unmet** | Plan § Phase 8 rollup still empty checkboxes (process close) |
| **DoD** Zero new deps + full gate | **Partial** | Deps OK; full gate not re-run in this review |
| **DoD** Deferred #34 struck; conductor Complete | **Unmet** | Close-out process (expected pre-final) |
| **DoD** Cross-model review | **Unmet** | Recommended SECURITY-adjacent; not done in R1 |

---

## Findings

### F-001 [high] Production `retention apply` uses `AllowAllPolicy` + local in-process CE wipe

- **description:** `crates/ai-brains-cli/src/commands/retention.rs` hard-codes `AllowAllPolicy` for CE batch apply and runs `wipe_content_envelope` in-process against the local vault. Control-plane docs forbid `AllowAllPolicy` outside tests (`lib.rs` / `policy.rs`). T165 production CE is daemon-required with erase-grant policy (`erasure wipe` → `choose_erasure_path` require_daemon; daemon uses `production_policy`). Spec §6.2 freeze: CE-class apply should require daemon when wipe policy requires it; production CLI should prefer daemon for CE batch. Result: any operator who can open the vault CLI can bulk CE-wipe without `GrantCapability::Erase`, bypassing deny-by-default grants. Projection-local apply is fine; **CE rows are not**.
- **files:**
  - `C:\dev\ai-brains-T166\crates\ai-brains-cli\src\commands\retention.rs` (lines ~90–99)
  - contrast: `C:\dev\ai-brains-T166\crates\ai-brains-cli\src\commands\erasure.rs` (daemon-only wipe)
  - `C:\dev\ai-brains-T166\crates\ai-brains-control-plane\src\lib.rs` (AllowAll test-only mandate)
- **required_fix:** For CE candidates, route through daemon wipe (same path as `erasure wipe`) with `production_policy` / authenticated principal + Erase grant. Keep projection deletes local if desired. Remove production `AllowAllPolicy` from CLI. Document dual path in OPERATIONS. Add regression test that local apply without Erase grant cannot CE (or that CE path is daemon-gated).
- **status:** verified_fixed
- **fix:** Production CLI uses `apply_retention_projections` (no AllowAll, no local wipe) + daemon `WipeContentEnvelope` for CE keys. CE candidates force daemon up-front (`production_apply_requires_daemon`). Tests: `retention_apply__projections_only__defers_ce_no_local_destroy`, CLI unit tests for daemon gate. Fixture path keeps `apply_retention` + AllowAll.

### F-002 [medium] R13 “CE wins when join known” only suppresses turns for `ce_wipe` candidates

- **description:** Spec §5.2.2: when `subject_kind=turn` / `subject_id={session_id}:{turn_index}` is present, stream B **wins** and stream A must **skip** `projection_delete` for that turn. Implementation only inserts into `turn_ids_covered_by_ce` when `c.mechanism == MECHANISM_CE_WIPE` (`class_based_retention.rs` ~253–256). If the envelope is `held`, `skip` (within horizon / stream-B `raw_turn` label), or unclassified, a past-horizon turn projection can still be deleted while the envelope remains — opposite of “CE wins.” Comment at ~269 claims join known + CE covers; condition is narrower than the lock.
- **files:**
  - `C:\dev\ai-brains-T166\crates\ai-brains-control-plane\src\class_based_retention.rs` (~239–272)
  - `C:\dev\ai-brains-T166\conductor\tracks\trackT166-class-based-retention\spec.md` §5.2
- **required_fix:** When any stream-B row lists a turn subject join, suppress stream-A `projection_delete` for that turn identity regardless of stream-B mechanism (or at minimum for ce_wipe **and** held/skip when join exists). Add test: old turn + linked envelope within secret horizon → no `projection_delete` for that turn.
- **status:** verified_fixed
- **fix:** Any `turn_subject_ids` on stream-B suppress stream-A projection_delete. Test: `retention_plan__linked_turn_held_or_skip_envelope__no_projection_delete`.

### F-003 [medium] Pin hold requires **all** memory subjects pinned (any-pinned can still CE-wipe)

- **description:** R11 holds CE candidates whose **sole** subject is a pinned memory. Implementation holds only when `memory_subject_ids` non-empty **and** `iter().all(pinned)` (~448–451). If an envelope links pinned memory A and unpinned memory B, apply will `ce_wipe` the wrap and destroy access for the pinned subject as well. Safer R11 interpretation: hold if **any** linked memory subject is pinned (unless operator uses out-of-band T165 wipe).
- **files:**
  - `C:\dev\ai-brains-T166\crates\ai-brains-control-plane\src\class_based_retention.rs` (~448–465)
- **required_fix:** Treat `any` pinned memory subject as `held` for age-based CE; keep sole-subject test; add multi-subject mixed pin test.
- **status:** verified_fixed
- **fix:** `any` pinned memory subject → held. Test: `retention_plan__multi_subject_mixed_pin__held`.

### F-004 [medium] Nightly does not log class-matrix dry-run summary

- **description:** Spec §6.2 / plan Phase D: nightly should log a **class dry-run summary** for the matrix while continuing raw-turn projection cleanup. `brain/src/lib.rs` nightly only runs `RetentionService::run_cleanup` (projection turns) and optional CE env **note**. No `plan_retention` / class counts log.
- **files:**
  - `C:\dev\ai-brains-T166\crates\ai-brains-brain\src\lib.rs` (~165–184)
  - `C:\dev\ai-brains-T166\crates\ai-brains-brain\src\retention.rs`
- **required_fix:** On nightly, call `plan_retention` (or equivalent read-only scan) and log totals/per-class counts (no bodies). Keep CE apply off by default (R7). Optional follow-up: if opt-in CE is ever wired, it must remain confirm/policy-gated.
- **status:** verified_fixed
- **fix:** CLI nightly entrypoint calls `plan_retention` and logs totals (no apply). Kept out of brain to avoid CP dependency bloat / Windows debug stack pressure.

### F-005 [low] `AI_BRAINS_RETENTION_APPLY_CE` is a no-op for enabling nightly CE

- **description:** Env flag is read and logged, but nightly never runs class CE even when set (`retention.rs` ~45–50; lib.rs ~172–176). R7 negative requirement is satisfied (no silent CE). CLI after_help / OPERATIONS can read as “opt-in enables nightly CE,” which is false — only documents intent and points operators at `retention apply --confirm`.
- **files:**
  - `C:\dev\ai-brains-T166\crates\ai-brains-brain\src\retention.rs`
  - `C:\dev\ai-brains-T166\crates\ai-brains-cli\src\main.rs` (~806)
  - `C:\dev\ai-brains-T166\Docs\OPERATIONS.md` (~194, ~224)
- **required_fix:** Align help/OPERATIONS wording: flag does not enable nightly CE; CE remains confirm-gated CLI (or implement true opt-in CE path only after F-001 daemon/policy fix). Prefer one env name (spec mentioned `APPLY_CLASSES`; code uses `APPLY_CE`).
- **status:** verified_fixed
- **fix:** Help + OPERATIONS + brain log strings state flag is intent-only; CE remains CLI+daemon+confirm.

### F-006 [low] Human apply output always titles “Retention plan”

- **description:** `emit_report` human path prints `Retention plan (mode=...)` even when `mode=apply`, slightly weakening CLI honesty.
- **files:** `C:\dev\ai-brains-T166\crates\ai-brains-cli\src\commands\retention.rs` (~141–144)
- **required_fix:** Title from `report.mode` (`dry_run` → plan, `apply` → apply).
- **status:** verified_fixed
- **fix:** Title derives from `report.mode` (`apply` → “Retention apply”).

### F-007 [info] R15 cascade demotes pinned parents to `stale`

- **description:** `mark_parents_for_resynthesis` updates parents with `status IN ('pinned','active')` → `stale`, which removes pin status. Spec R15 intentionally auto-marks parents for resynthesis; hierarchy test expects parent `stale`. Residual: pin protection is lost after cascade; document in OPERATIONS if not already clear.
- **files:** `C:\dev\ai-brains-T166\crates\ai-brains-store\src\projections\retention.rs` (~390–412)
- **required_fix:** Optional: set a resynthesis flag without clearing `pinned`, or document residual that cascade supersedes pin for parent synthesis staleness.
- **status:** verified_fixed
- **fix:** Documented as residual in OPERATIONS class-based retention table (R15 cascade residual).

### F-008 [info] Projection-delete failures do not fail apply exit

- **description:** Spec requires non-zero exit on CE failure (implemented). Projection `delete_*` errors are pushed to `errors` but `apply_retention` still returns `Ok` if no `ce_wipe` error prefix; CLI exit check matches that. Operators may miss partial projection failure if they only watch process exit.
- **files:**
  - `C:\dev\ai-brains-T166\crates\ai-brains-control-plane\src\class_based_retention.rs` (~747–834)
  - `C:\dev\ai-brains-T166\crates\ai-brains-cli\src\commands\retention.rs` (~114–119)
- **required_fix:** Optional: non-zero exit when `errors_count > 0`, or surface projection errors more loudly in human output.
- **status:** verified_fixed
- **fix:** CLI non-zero exit when `errors_count > 0`; human path prints Errors section.

---

## Missing tests

| Spec §13 pattern | Present? | Notes |
|------------------|----------|-------|
| `retention_plan__empty_vault__zero_counts` | Yes | CP tests |
| `retention_plan__raw_turns_past_horizon__projection_delete` | Yes | |
| `retention_plan__envelope_secret_class__ce_wipe` | Yes | + no plaintext |
| `retention_plan__unknown_class__skip` | Yes | |
| `retention_plan__approved_decision_active__skip` | Yes | |
| `retention_plan__pinned_memory__held` | Yes | sole-subject CE |
| `retention_plan__no_double_count_same_content_key` | Yes | |
| `retention_plan__orphaned_envelope__listed` | Yes | |
| `retention_apply__without_confirm__refused` | Yes | |
| `retention_apply__raw_turns__deletes_projection_only` | Yes | events retained |
| `retention_apply__envelope__calls_wipe_not_parallel_ce` | Yes | wrap destroyed |
| `retention_apply__appends_retention_applied_event` | Yes | |
| `retention_apply__idempotent_second_run` | Yes | |
| `retention_apply__hierarchy_parent_marked_for_resynthesis` | Yes | |
| `retention_report__contains_honesty_warnings` | Yes | (+ contracts unit) |
| `nightly_default__no_ce_without_opt_in` | Yes | CP + brain |
| `classification__blob_content_class_stream_b_only` | Yes | |
| **Gap:** join known → skip stream-A projection always | **No** | Needed for F-002 |
| **Gap:** multi-subject mixed pin hold | **No** | Needed for F-003 |
| **Gap:** CE apply denies without Erase / daemon-gated | **No** | Needed for F-001 |
| **Gap:** nightly class dry-run log | **No** | Needed for F-004 |
| **Gap:** orphan apply CE (wrap-only) | **No** | Nice-to-have; plan listed CE via wipe |

---

## Residual notes

- **Parallel CE (R2 code):** Clean. `destroy_content_key_wrap` only via T165 wipe store path + unit/schema tests.
- **Report leakage (R4):** No content/body fields on `RetentionPlanReport`. Sample ids are truncated identities (`content_key:…`, `turn:…`). Good.
- **Apply without confirm (R1):** Refused at CP and CLI — not a bug.
- **RetentionApplied (R12):** Appended on apply (including empty apply); not on plan.
- **Double-count same content_key (R13 partial):** De-dupe for keys is solid; join precedence incomplete (F-002).
- **Pinned sole-subject (R11 partial):** Held correctly; multi-subject hole (F-003).
- **Orphan 7d (R16):** Default and classification correct; young orphans skipped to avoid seal races.
- **unwrap/expect/panic:** None in production retention modules (`class_based_retention.rs`, store `projections/retention.rs`, CLI `retention.rs`, brain `retention.rs`).
- **Nightly raw turns vs class scan clocks:** Nightly `delete_old_turns` uses `last_accessed_at < ?` only; class plan uses `COALESCE(last_accessed_at, occurred_at)`. Pre-existing / mild inconsistency if null `last_accessed_at` exists.
- **memory_legacy:** No stream-A memory scan (v1 none auto). OPERATIONS row “pinned → held” is only meaningful for envelope-linked memories today.
- **Process DoD:** Phase 8 rollup evidence, deferred #34 strike, conductor Complete, cross-model review, full CI gate — still open for track close, not scored as product defects above.
- **Positive:** Strong TDD coverage for the main happy paths; honesty warning family matches T165 spirit; stream independence residual documented; zero new deps; capture path not entangled.

---

## Suggested fix order

1. **F-001** (high) — daemon + production policy for CE rows  
2. **F-002** (medium) — R13 join precedence  
3. **F-003** (medium) — pin hold any-subject  
4. **F-004** (medium) — nightly class dry-run log  
5. **F-005–F-008** — honesty / polish  

Re-run: `cargo nextest run -p ai-brains-control-plane -p ai-brains-cli -p ai-brains-brain -p ai-brains-store -p ai-brains-contracts -p ai-brains-events` and full workspace gate before R2 review.
