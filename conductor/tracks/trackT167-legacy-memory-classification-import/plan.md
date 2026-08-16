# T167 Plan — Legacy Memory Classification Importer

Status: ✅ **Completed** 2026-08-16. Importer shipped with T168; this closeout marks the plan boxes that already have tests in `crates/ai-brains-control-plane/tests/legacy_import.rs`.

Normative: `spec.md` locks **L1–L20**, matrix §5, plan_hash §6.1, policy §6.2.

## Phase 0 — Inventory freeze

- [x] Map master-plan Task 9.1 → live `Payload` / `SourceKind` / `DecisionState` / `GrantCapability`
- [x] Confirm fixture + expected-legacy-projections
- [x] Fold deferred #16 / #18 / #10 / #2 / ADR-0016
- [x] Online research: idempotent migration, provenance, dry-run, under-promote
- [x] AI review fold-in: no RecordEvidence; no observe_source; has_evidence; two-pass; plan_hash; L18–L20

## Phase A — RED (classification pure)

- [x] Add `legacy_import.rs` module skeleton + `mod` in `lib.rs`
- [x] RED: `classify__memory_pinned__evidence_not_conclusion`
- [x] RED: `classify__memory_synthesized__candidate_only`
- [x] RED: `classify__decision_recorded__proposed_plus_review`
- [x] RED: `classify__forgotten_memory__excluded`
- [x] RED: `classify__forgotten_then_restored__included`
- [x] RED: `classify__synth_referencing_forgotten_source__unsupported_true` (two-pass)
- [x] RED: `classify__unknown_payload__unresolved`
- [x] RED: `classify__idempotent_ids__stable_v5`
- [x] RED: `classify__evidence_id_prefers_memory_id`
- [x] RED: `classify__decision_id_not_memory_id_cast`
- [x] RED: `classify__missing_scope_without_default__skipped`
- [x] RED: `classify__default_scope_fallback__used`
- [x] RED: `classify__session_summary__evidence_digest`
- [x] RED: `classify__preserves_source_tag_metadata`
- [x] RED: `plan_hash__same_input_same_hash`
- [x] RED: `plan_hash__reordered_actions_same_hash`
- [x] RED: `plan_hash__omits_body_plaintext`
- [x] RED: `report__no_full_plaintext_by_default`
- [x] RED: fixture NDJSON → frozen plan totals golden

## Phase B — GREEN (classify)

- [x] Two-pass `classify_legacy` (§5.0): Pass 1 status+evidence map; Pass 2 conclusions/decisions
- [x] Stable namespaces `NS_LEGACY_*` + `id_from_command`
- [x] EvidenceId prefer memory_id; DecisionId from event_id only
- [x] Reason codes incl. `forgotten_source` / `missing_source` / `missing_scope`
- [x] `ImportOpts.default_scope` (L19)
- [x] Canonical `plan_hash` (§6.1) — BTreeMap sort, no bodies
- [x] GREEN all Phase A classify / plan_hash tests

## Phase C — SourceKind + apply (raw events)

- [x] Add `SourceKind::LegacyAiBrains` in `ai-brains-core`
- [x] Exhaustiveness fixes in `sources.rs` (known):
  - [x] `source_identity_string` kind_label match
  - [x] `compute_fingerprint` match → treat like Manual/Hermes (`fingerprint_external`) **if hit**
  - [x] `normalize_locator_for_kind` (likely `_` already)
  - [x] Any other match sites from `ledgerful scan --impact` / clippy
- [x] **Do not call `observe_source`** — build `SourceRegistered` + `EvidenceRecorded` via `build_event`
- [x] Importer-computed SHA-256 fingerprint on evidence content
- [x] Raw `ConclusionProposed` / `DecisionProposed` / `ReviewItemOpened` (pre-assigned `NS_LEGACY_REVIEW` id — not `ReviewItemId::new()`)
- [x] Add `GovernedQueryStore::has_evidence` + `StoreGovernedQuery` adapter + store query
- [x] Detect-already-done: has_evidence / has_conclusion / has_decision
- [x] `ApplyOpts.confirm` gate (L1)
- [x] Append `LegacyImportApplied` on successful apply (L20) — new payload + EventKind (mirror RetentionApplied shape: plan_hash, counts, no bodies)
- [x] RED/GREEN: `apply__second_run__zero_new_aggregates`
- [x] RED/GREEN: `apply__ensures_legacy_source_once`
- [x] RED/GREEN: `apply__appends_legacy_import_applied`
- [x] RED/GREEN: `apply__does_not_call_observe_source` (no SourceVersionRecorded from import)
- [x] Privacy = envelope privacy only (L12)

## Phase D — Wiring & docs

- [x] Export public API from `ai-brains-control-plane`
- [x] OPERATIONS / CAPABILITIES: CP API only; CLI T168; no live default; no RecordEvidence claim
- [x] Expand fixture if needed: DecisionRecorded, MemorySynthesized, forget/restore, forgotten-source synth
- [x] Manual: classify fixture report counts (no live apply)

## Phase E — Gate

- [x] `cargo fmt --check`
- [x] `cargo clippy -p ai-brains-control-plane -p ai-brains-core -p ai-brains-events --all-targets -- -D warnings`
- [x] `cargo nextest run -p ai-brains-control-plane -p ai-brains-core -p ai-brains-events`
- [x] `cargo deny check` (no new deps; no drive-by version bumps)
- [x] Full workspace gate before finalize
- [x] `ledgerful verify` as appropriate
- [x] Review log clean of open high
- [x] Mark conductor **Completed** + evidence note

## License gate

- [x] Zero new crates (L7)

## Out of scope

- [x] CLI `migrate governed` (T168)
- [x] Differential report file schema (T168)
- [x] Live vault migration (T170)
- [x] Auto DecisionApproved / ConclusionConfirmed
- [x] CE seal/wipe of imported content
- [x] `source_tag` rewrite changeguard→ledgerful
- [x] ChangeGuard type renames (#1)
- [x] Workspace dep version bumps (sqlx 0.9, base64 0.23, tower-http 0.7, …) — deferred #40
