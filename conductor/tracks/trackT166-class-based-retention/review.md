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

---

# T166 Internal Review R2 (post-fix re-review)

**Reviewer:** primary (read-only re-review)  
**Scope:** `feature/T166-class-based-retention` vs base `2f3be7d`; claimed fixes for F-001..F-008 (commit `fb4a95d` and later)  
**Authority:** `spec.md` R1–R16 + DoD; R1 findings; production retention surfaces  
**Date:** 2026-07-29  
**Method:** Code + test evidence only (no workspace CI re-run in this pass)

## Verdict: CLEAN

All prior findings F-001..F-008 are **verified_fixed** with code + test evidence. No new **critical / high / medium** regressions found on the daemon split path, join de-dupe, pin hold, or `RetentionApplied` finalize wiring. Two optional low/info residuals below do not block CLEAN.

---

## F-001..F-008 re-verification

| ID | R1 severity | R2 status | Evidence |
|----|-------------|-----------|----------|
| **F-001** | high | **verified_fixed** | Production CLI (`crates/ai-brains-cli/src/commands/retention.rs`) uses `apply_retention_projections` (local projection only) + `DaemonRequest::WipeContentEnvelope` for CE keys. Module docs forbid `AllowAllPolicy` + local wipe. `rg AllowAllPolicy` under `ai-brains-cli/src` hits only comments. `production_apply_requires_daemon(would_ce_wipe > 0)` forces `choose_erasure_path(require_daemon: true)` **before** disposal. Daemon wipe path uses `StorePorts::production_policy` (`ai-brainsd/src/services.rs` `process_wipe_content_envelope`). Fixture `apply_retention` + `AllowAllPolicy` remains **tests-only** (`class_based_retention.rs` integration tests). Tests: `retention_apply__projections_only__defers_ce_no_local_destroy` (wrap stays `active`); CLI unit `production_apply_requires_daemon__ce_candidates__true` / `__projection_only__false`. |
| **F-002** | medium | **verified_fixed** | `collect_candidates`: every stream-B `env.turn_subject_ids` populates `turn_ids_covered_by_envelope` **regardless** of mechanism; stream-A turns skip when join present (`class_based_retention.rs` ~245–272). Test: `retention_plan__linked_turn_held_or_skip_envelope__no_projection_delete` (old turn + within-horizon secret join → no `projection_delete`). |
| **F-003** | medium | **verified_fixed** | R11: `env.memory_subject_ids.iter().any(|m| pinned.contains(m))` → `held` (`class_based_retention.rs` ~448–450). Test: `retention_plan__multi_subject_mixed_pin__held` (pinned + unpinned under one key → `would_held >= 1`, `would_ce_wipe == 0`). Sole-subject test still present. |
| **F-004** | medium | **verified_fixed** | CLI nightly (`commands/nightly.rs` ~282–310) calls `plan_retention` and logs totals (`candidates` / `ce_wipe` / `projection_delete` / `skip` / `held`) with explicit “no apply”. Brain `run_nightly` remains projection-only raw-turn cleanup + R7 intent log (avoids CP dep on brain); comment points to CLI for class dry-run. Spec §6.2 nightly surface satisfied via operator CLI nightly entrypoint. |
| **F-005** | low | **verified_fixed** | `AI_BRAINS_RETENTION_APPLY_CE` / `APPLY_CE_ON_NIGHTLY` documented as **intent-only** in brain `retention.rs`, brain nightly log, CLI `main.rs` after_help (~806), OPERATIONS (~195, ~227). No silent CE enablement. |
| **F-006** | low | **verified_fixed** | `emit_report`: `mode == "apply"` → title `"Retention apply"`, else `"Retention plan"` (`retention.rs` ~219–222). |
| **F-007** | info | **verified_fixed** | OPERATIONS residual row: R15 cascade may mark parent `stale` even if previously `pinned` (~197). |
| **F-008** | info | **verified_fixed** | CLI: `errors_count = report.errors.len()`; non-zero → `Err("retention apply had errors...")` after emit; human path prints `Errors:` section (~198–253). |

---

## Targeted greps (R2)

| Check | Result |
|-------|--------|
| `AllowAllPolicy` in production CLI retention path | **Absent** (comments only in `commands/retention.rs`) |
| `destroy_content_key_wrap` outside T165 wipe / store / tests | **Clean** — no hits in CLI, `class_based_retention.rs`, or store `projections/retention.rs`. Call chain remains T165 `wipe_content_envelope` → store `content_envelope::destroy_content_key_wrap`. |
| `unwrap` / `expect` / `panic!` in production retention modules | **None** in `class_based_retention.rs`, store `projections/retention.rs`, CLI `commands/retention.rs`, brain `retention.rs` (test-only `expect` in contracts unit tests / CP integration tests OK). |

---

## Fresh sweep — fix-introduced surfaces

### Daemon / split apply (F-001 fix)

| Concern | Assessment |
|---------|------------|
| CE requires daemon before mutate | **OK** — gate on pre-apply plan `would_ce_wipe`; `DAEMON_UNAVAILABLE` before `apply_retention_projections`. |
| Local path never destroys wrap | **OK** — projections path only collects `pending_ce_keys`; test asserts wrap remains `active` with nonce present. |
| CE only via T165 daemon wipe | **OK** — CLI loops `DaemonRequest::WipeContentEnvelope` with `confirm: true`, `dry_run: false`. |
| Projection-only without daemon | **OK** — `production_apply_requires_daemon(0) == false`. |
| Double plan TOCTOU | **Acceptable residual (info):** daemon gate uses plan #1; apply re-plans inside `apply_retention_projections`. Race that invents CE after a zero-CE plan is theoretical; CE would still go through `DaemonClient` (policy/daemon errors → report errors), never `AllowAllPolicy`. |

### `finalize_retention_apply` / R12

| Concern | Assessment |
|---------|------------|
| Projection-only apply appends `RetentionApplied` | **OK** — `apply_retention_projections` appends when `pending_ce_keys` empty. |
| Deferred CE appends after CE | **OK** — CLI calls `finalize_retention_apply` when keys non-empty (cascade + audit). |
| Dry-run / no-confirm | **OK** — still refused at CLI and CP. |
| Fixture path still audits | **OK** — in-process `apply_retention` still appends; tests keep R12 coverage. |

### Join de-dupe (R13) / pin hold (R11)

| Concern | Assessment |
|---------|------------|
| content_key de-dupe | **OK** — `seen_content_keys` BTreeSet. |
| Join suppress independent of mechanism | **OK** — F-002 fix. |
| Any-pin hold | **OK** — F-003 fix; pin check before age CE for classified envelopes. |
| Skip noise filter vs joins | **OK** — turn join set filled **before** `candidates.retain` drops pure-skip rows; held/skip envelopes still suppress stream A. |

### Nightly

| Concern | Assessment |
|---------|------------|
| Class dry-run log | **OK** on CLI nightly. |
| CE never auto | **OK** — R7; flag intent-only. |
| Capture independence | **OK** — plan/apply still store + events + T165 wipe only. |

---

## DoD / R-lock matrix (R2)

| Item | Status | Notes |
|------|--------|-------|
| **R1** Dry-run first; confirm apply | **Met** | Unchanged + CLI refuse paths |
| **R2** One CE path | **Met** | Production = daemon wipe; fixture = `wipe_content_envelope`; no parallel destroy |
| **R3** Legacy ≠ CE | **Met** | |
| **R4** No plaintext reports | **Met** | |
| **R5** Canonical + unclassified skip | **Met** | |
| **R6** No age-wipe active approved | **Met** | |
| **R7** Nightly CE default off | **Met** | Intent flag honesty fixed |
| **R8** Capture independence | **Met** | |
| **R9** Zero new deps | **Met** | (not re-audited Cargo.lock; surface uses existing crates) |
| **R10** Append-only events | **Met** | |
| **R11** Pin hold | **Met** | any linked memory subject |
| **R12** RetentionApplied on apply | **Met** | Split path: projections-only now; deferred CE via finalize |
| **R13** No double-count; CE wins when join known | **Met** | any join suppresses stream A |
| **R14** Cooldown `updated_at` | **Met** | |
| **R15** Hierarchy cascade | **Met** | Residual pin→stale documented |
| **R16** Orphan 7d | **Met** | |
| **DoD** dual-path apply | **Met** | OPERATIONS documents projection local + CE daemon |
| **DoD** process close (Phase 8 rollup, #34 strike, conductor Complete, cross-model, full gate) | **Unmet** | Process — not product blockers for R2 CLEAN |

---

## New residuals (R2)

### F-009 [info] No direct test that `finalize_retention_apply` appends `RetentionApplied`

- **description:** R12 is proven for in-process `apply_retention`. Production split defers audit to `finalize_retention_apply`; CLI wires it, but there is no unit/integration test asserting RetentionApplied after deferred CE + finalize (only wrap-not-destroyed on projections step).
- **status:** fixed_pending_verification
- **fix:** `finalize_retention_apply__appends_final_audit_and_cascades` + `retention_apply__projections_append_audit_before_ce`. Pre-CE audit always; finalize second event.

### F-010 [low] R15 cascade uses planned CE memory subjects, not wipe-success filter

- **description:** After daemon CE loop, CLI always passes full `pending_cascade_memory_ids` into `finalize_retention_apply`, including subjects whose wipe errored. In-process `apply_retention` has the same shape (cascade after batch wipe attempts). Spec ties cascade to **disposed** subjects; partial CE failure can still mark parents `stale`.
- **status:** fixed_pending_verification
- **fix:** `pending_cascade_by_key` + `cascade_memory_ids_for_keys`; CLI filters to wiped/already_erased; in-process same. Tests: cascade filter + failed CE no cascade.

### Other residuals (unchanged / mild)

- Nightly raw-turn `delete_old_turns` clock (`last_accessed_at` only) vs class plan `COALESCE(last_accessed_at, occurred_at)`.
- `memory_legacy` stream-A scan still none-auto in v1.
- Process DoD / full CI gate / cross-model review still for track close-out.

---

## Missing tests (R2 update)

| Spec §13 / fix gap | Present? |
|--------------------|----------|
| Prior §13 suite (empty vault → hierarchy cascade, etc.) | Yes |
| join known → skip stream-A (F-002) | **Yes** |
| multi-subject mixed pin (F-003) | **Yes** |
| projections path defers CE / no local destroy (F-001) | **Yes** |
| CLI daemon-gate unit tests (F-001) | **Yes** |
| nightly class dry-run log | **Yes** (CLI path; no automated assert on log line) |
| finalize → RetentionApplied | **No** (F-009) |
| cascade only on successful CE | **No** (F-010) |

---

## R2 summary

Product blockers from R1 are closed:

1. Production CE no longer uses `AllowAllPolicy` / local wipe; daemon + production policy parity with T165.  
2. R13 join precedence covers held/skip as well as `ce_wipe`.  
3. R11 pin hold is any-subject.  
4. Nightly class dry-run summary lands on the CLI nightly path; CE remains confirm+daemon only.  
5. Honesty polish (env flag, apply title, R15 residual doc, non-zero exit on errors) verified.

**R2 verdict: CLEAN** — no open critical/high/medium. F-009 (info) and F-010 (low) may defer to `conductor/ISSUES.md` if desired; process DoD remains for track closure.

---

# Codex R1 findings — dispositions (post-fix)

**Source:** `review.codex.r1.md`  
**Date:** 2026-07-29  

| ID | Severity | Status | Disposition |
|----|----------|--------|-------------|
| **Codex-P1 R12 audit durability** | P1 | **verified_fixed** (R3) | See Internal Review R3 §1. |
| **Codex-P1 unsafe horizons** | P1 | **verified_fixed** (R3) | See Internal Review R3 §2. |
| **Codex-P2 R15 cascade on failed CE** | P2 | **verified_fixed** (R3) | See Internal Review R3 §3. |
| **Codex-P3 full content_key in errors** | P3 | **verified_fixed** (R3) | See Internal Review R3 §4. |
| **Codex-P3 production split tests** | P3 | **verified_fixed** (R3) | See Internal Review R3 §1–3 test table. |

---

# T166 Internal Review R3 (Codex R1 fix re-review)

**Reviewer:** primary (read-only re-review)  
**Scope:** `feature/T166-class-based-retention` vs base `2f3be7d`; Codex R1 fix commit `c6f7e6e` and later on branch  
**Authority:** `spec.md` R1–R16; R1/R2 findings; Codex `review.codex.r1.md` P1–P3; production retention surfaces  
**Date:** 2026-07-29  
**Method:** Code + test evidence only (no workspace CI re-run in this pass)

## Verdict: CLEAN

All Codex R1 P1/P2/P3 items and prior F-001..F-010 claims are **verified_fixed** with code + test evidence. No new **critical / high / medium** regressions from the fix set. One optional **low** residual (brain nightly horizon path) does not block CLEAN.

---

## 1. R12 — RetentionApplied durability (Codex-P1)

| Claim | Result | Evidence |
|-------|--------|----------|
| Pre-CE audit after projections even when CE pending | **Met** | `apply_retention_projections` always calls `append_retention_applied` after projection deletes (`class_based_retention.rs` ~848–850), including when `ce_keys` non-empty; warning `ce_pending=N (RetentionApplied pre-CE; finalize after daemon wipe)`. |
| Finalize second event after daemon CE | **Met** | CLI calls `finalize_retention_apply` when `pending_ce_keys` non-empty (`commands/retention.rs` ~147–203); finalize appends a second `RetentionApplied` (~883). |
| Projection-only still one event | **Met** | CLI only enters CE/finalize block when keys non-empty; empty CE → single audit from projections path. Fixture `apply_retention` still single audit post-work. |
| Crash mid-daemon-CE leaves durable audit | **Met** | Pre-CE append happens **before** daemon loop returns to caller; projections already audited. |

**Tests:**

- `retention_apply__projections_append_audit_before_ce` — CE pending → exactly 1 `RetentionApplied` pre-finalize + `ce_pending=` warning.
- `finalize_retention_apply__appends_final_audit_and_cascades` — pre-CE + finalize → ≥2 audits; cascade marks parent stale.
- Prior: `retention_apply__appends_retention_applied_event` (fixture path).

**Status:** **verified_fixed** (closes F-009).

---

## 2. Horizon parsing (Codex-P1)

| Claim | Result | Evidence |
|-------|--------|----------|
| No panic on overflow construction | **Met** | `parse_positive_horizon_days` uses `Duration::try_days`; `cutoff_days_before` uses `try_days` + `checked_sub_signed` with epoch fallback (select almost nothing, not everything). No `Duration::days` in production class-based path. |
| No negative horizons | **Met** | `v <= 0` rejected; env invalid → class default. |
| Clamp `1..=36500` | **Met** | `MAX_RETENTION_HORIZON_DAYS = 36_500`; oversize rejected. |
| Checked duration | **Met** | As above; OPERATIONS ~229 documents validation. |

**Tests:**

- `parse_positive_horizon_days__rejects_non_positive_and_huge` — 0, −7, non-int, max+1 err; 90 and max ok.
- `retention_config_from_env__negative_falls_back_to_default` (TempEnv) — negative/zero/huge fall back; valid 45 accepted; secret_days > 0.

**Status:** **verified_fixed** for class-based / CLI plan+apply path.

---

## 3. R15 cascade only successful CE keys (Codex-P2)

| Claim | Result | Evidence |
|-------|--------|----------|
| Production CLI filters wiped / already_erased | **Met** | `successful_ce_keys` only on those statuses; `cascade_memory_ids_for_keys(&pending_cascade_by_key, &successful_ce_keys)` before finalize. |
| Map is per-key (not flat all planned subjects) | **Met** | `pending_cascade_by_key: BTreeMap<String, Vec<String>>` collected only under `MECHANISM_CE_WIPE`. |
| In-process fixture path same filter | **Met** | `apply_retention` builds `successful_ce_keys` the same way; cascade uses `cascade_memory_ids_for_keys`. |
| Failed CE → no parent stale | **Met** | Test with empty successful list. |

**Tests:**

- `cascade_memory_ids_for_keys__only_successful_keys`
- `finalize_retention_apply__failed_ce_keys_do_not_cascade` — parent stays `active`
- CLI unit `cascade_filter__successful_keys_only`

**Status:** **verified_fixed** (closes F-010).

---

## 4. Truncated ids in errors (Codex-P3)

| Claim | Result | Evidence |
|-------|--------|----------|
| CE error strings use truncated key display | **Met** | CLI: `key_disp = truncate_id(key)` in all `ce_wipe {key_disp}: …` branches. In-process: `key_disp = truncate_id(key_str)` for invalid key + wipe errors. |
| Same helper as sample_ids | **Met** | contracts `truncate_id` (MAX 36 + ellipsis). |

**Test:** CLI `error_key_ids_use_truncate_id` (UUID ≤36 preserved; longer truncated).

**Status:** **verified_fixed**.

---

## 5. Production safety greps (regression + prior locks)

| Check | Result |
|-------|--------|
| Production CLI `AllowAllPolicy` | **Absent** — comments only in `commands/retention.rs` |
| Production CE path | Daemon `WipeContentEnvelope` only; gate `production_apply_requires_daemon(would_ce_wipe > 0)` |
| `destroy_content_key_wrap` outside T165 wipe / store / tests | **Clean** — no hits in CLI, class_based_retention, store projections/retention |
| `unwrap` / `expect` / `panic!` in production retention modules | **None** in `class_based_retention.rs`, store `projections/retention.rs`, CLI `commands/retention.rs`, brain `retention.rs` |
| Parallel destroy in retention | **None** |

---

## 6. Fresh sweep — regressions from Codex fixes

| Concern | Assessment |
|---------|------------|
| Double audit same `command_id` | **OK** — intentional pre-CE + final; same aggregate id, two append-only events. |
| Finalize still runs when all CE fail | **OK** — cascade empty; second audit carries errors; R12 final tallies. |
| Cascade mark with empty slice | **OK** — `mark_parents_for_resynthesis` → 0 parents. |
| Pre-CE report cascade count 0 | **OK** — true cascade only at finalize / in-process after successful CE. |
| Horizon silent fallback vs hard fail | **Acceptable** — invalid env falls back to default (fail-safe for operator start); documented in OPERATIONS. |
| Brain nightly `days_from_env` not fully aligned | **Low residual (F-011)** — see below; pre-existing parallel path, not introduced by dual-path CE fix. |
| Projection-only `apply_retention_projections` exact audit count test | **Info** — implied by code (no finalize when keys empty) + fixture R12; optional extra unit test not required for CLEAN. |

### F-011 [low] Brain nightly horizon still `> 0` only + `Duration::days`

- **description:** Class path sanitizes `AI_BRAINS_RETENTION_*` to `1..=36500` with checked chrono. Brain `RetentionService::days_from_env` only requires `*d > 0` and nightly uses `Utc::now() - Duration::days(self.retention_days)`, so a huge positive env could theoretically panic on chrono overflow on the **legacy raw-turn** nightly path (not mass-select via negative). No CE. Not introduced by Codex R1 CE/audit fixes; consistency gap only.
- **files:** `crates/ai-brains-brain/src/retention.rs` (~26–31, ~63)
- **required_fix (optional):** Reuse `parse_positive_horizon_days` / same max + `try_days` checked cutoff.
- **status:** open (deferrable)

---

## DoD / R-lock matrix (R3 delta)

| Item | R2 | R3 |
|------|----|----|
| **R12** RetentionApplied | Met (split path) | **Met** + pre-CE durability verified |
| **R15** cascade | Met | **Met** + success-key filter verified |
| Horizons safe | (not in R2 matrix) | **Met** on class path |
| Error id truncation | (not scored) | **Met** |
| F-001..F-008 | verified_fixed | unchanged |
| F-009 / F-010 | fixed_pending_verification | **verified_fixed** |
| Process DoD (Phase 8, #34, conductor Complete, full gate) | Unmet | **Unmet** (process; not product blockers) |

---

## Missing tests (R3 update)

| Gap | Present? |
|-----|----------|
| pre-CE RetentionApplied with CE pending | **Yes** |
| finalize second RetentionApplied + cascade | **Yes** |
| cascade only successful CE keys | **Yes** |
| horizon reject / env fallback | **Yes** |
| truncated error key ids | **Yes** (CLI unit) |
| brain huge-horizon no-panic | **No** (F-011 optional) |

---

## R3 summary

Codex R1 blockers are closed:

1. **R12:** Every confirmed production apply appends `RetentionApplied` after projections **before** daemon CE; finalize adds a second final audit when CE was pending; projection-only remains a single event.  
2. **Horizons:** Class config rejects ≤0 and >36500, falls back on invalid env, uses checked duration construction (no chrono panic / no future cutoffs).  
3. **R15:** Cascade subjects drawn only from successful wipe keys (CLI + in-process).  
4. **R4-ish ids:** Apply error strings truncate content_key ids.  
5. **Safety:** No production `AllowAllPolicy` for CE, no parallel destroy, no `unwrap`/`expect`/`panic!` in production retention modules.

**R3 verdict: CLEAN** — no open critical/high/medium. F-011 (low) may defer to `conductor/ISSUES.md`. Process DoD / full CI gate remain for track closure.

---

# Codex R2 findings — dispositions (post-fix)

**Source:** `review.codex.r2.md`  
**Date:** 2026-07-29  

| ID | Severity | Status | Disposition |
|----|----------|--------|-------------|
| **Codex-R2-P1 R12 audit before destructive work** | P1 | **verified_fixed** | `apply_retention_projections` and in-process `apply_retention` now build planned report, append `RetentionApplied` **before** any projection delete / CE wipe. If audit append fails, no deletes run. Warning text: `ce_pending=N (RetentionApplied pre-mutation; …)`. Finalize still appends second audit after successful CE cascade. Tests: `retention_apply__projections_append_audit_before_ce`, `retention_apply__projections_audit_before_projection_delete`. |
| **Codex-R2-P1 default CE scope random Personal UUID** | P1 | **verified_fixed** | CLI `resolve_retention_apply_scope`: when `would_ce_wipe > 0`, requires non-empty `--scope` parseable as `Repository:/Personal:/Workspace:<uuid>`; `INVALID_PAYLOAD` if missing/empty; never `UserId::new()`. Projection-only (`would_ce_wipe == 0`) may omit scope. Scope + daemon gates run **before** `apply_retention_projections`. Help/after_help examples show `--scope`. Tests: `resolve_retention_apply_scope__ce_without_scope__err`, `__projection_only_omits_scope__ok`, `__ce_with_scope__ok`, `production_apply_requires_scope__ce_candidates__true`. |
| **Codex-R2-P3 daemon err.message full keys** | P3 | **verified_fixed** | CLI `DaemonResponse::Error` → `ce_wipe {key_disp}: {err.code}` only (no `err.message`). In-process wipe errors use `ce_wipe_error_code` short codes (`policy_denied`, `not_envelope_backed`, …) instead of `Display` which embeds full key. Test: `daemon_error_line__code_only_no_message`. |

## R4 notes (Codex R2 fix re-review self-check)

| Concern | Result |
|---------|--------|
| Pre-mutation audit order | Plan build → `append_retention_applied` → projection deletes → (CLI) CE → finalize |
| Random Personal scope | Removed; `UserId::new()` no longer in retention CLI apply path |
| Scope gate before mutation | Plan first, then `resolve_retention_apply_scope` + daemon, then projections |
| Double audit with CE | Intentional: pre-mutation + finalize final tallies |
| Projection-only scope | Optional / unused |
