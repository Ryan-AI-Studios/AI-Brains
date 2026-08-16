# T167 — Legacy Memory Classification Importer (P9.1)

- **Track ID:** T167-LegacyMemoryClassificationImport
- **Phase:** P9 Task 9.1
- **Status:** ✅ **Completed** 2026-08-16 — importer shipped with T168 (`legacy_import.rs`, `SourceKind::LegacyAiBrains`, `LegacyImportApplied`). Closeout only; no second importer surface.
- **Depends on:** T148–T152 domain/events (Evidence/Conclusion/Decision/Review); T149 source ports; legacy payloads still deserialize; T163–T166 **not** required for import correctness (CE honesty only)
- **Category:** FEATURE / ARCHITECTURE
- **Master plan:** Task 9.1 in `.hermes/plans/2026-07-23_204630-memory-control-plane-successor.md`
- **Normative ADRs:** [ADR-0011](../../../Docs/DECISIONS/ADR-0011-separate-evidence-conclusions-decisions.md) (E/C/D authority); [ADR-0015](../../../Docs/DECISIONS/ADR-0015-event-ledger-erasure-and-encrypted-replication.md) (ledger + forget honesty); [ADR-0016](../../../Docs/DECISIONS/ADR-0016-content-envelope-cryptography.md) (legacy plaintext **cannot** claim CE)

## 1. Objective

Implement an **idempotent legacy → governed classification importer** that maps historical AI-Brains events into:

| Governed entity | Authority honesty |
|-----------------|-------------------|
| **Evidence** | Observation under source **`LegacyAiBrains`** — never Confirmed Conclusion |
| **Candidate Conclusion** | Derived claim only (`ConclusionState::Candidate`) — never auto-Confirmed |
| **Proposed Decision + Review** | Commitment candidate — never silent `Approved` without human approval path |
| **Excluded** | Forgotten memories, unknown payloads, non-matrix kinds |

…**without promoting inferred content to Approved / Confirmed authority**, recording **original event IDs** for provenance, and defaulting to **classify/dry-run** (no live-vault writes).

Closes P9.1 acceptance: idempotent import; original IDs retained; forgotten never resurrected; under-promote.

## 2. Live baseline (re-scan 2026-07-29)

| Area | Live state |
|------|------------|
| Fixture | `fixtures/governed-memory/legacy-v1-events.ndjson` (Project + Session + turns + one `MemoryPinned`) |
| Expected projections | `fixtures/governed-memory/expected-legacy-projections.json` (legacy memory/turn freeze — **not** governed E/C/D) |
| CP module | `legacy_import.rs` — **absent** |
| `SourceKind` | `GitRepository`, `File`, `ObsidianVault`, `Ledgerful`, `HermesSession`, `Honcho`, `Manual`, `Other(String)` — **no** `LegacyAiBrains` yet |
| `DecisionState` | `Proposed` / `Approved` / `Superseded` / `Revoked` — **no** `ApprovedLegacy` variant |
| `ConclusionState` | includes `Candidate` … `Confirmed` (ADR-0011) |
| Evidence write path | via `observe_source` → `EvidenceRecorded`; no free-form “import evidence” helper yet |
| Idempotency pattern | uuid v5 `id_from_command(NS_*, command_id)` in `command_id.rs` (T159+) |
| Soft forget | `MemoryForgotten` → `memory_projection.status = 'forgotten'`; FTS filters forgotten |
| Restore | `MemoryRestored` exists; final stream state decides import eligibility |
| CLI migrate | **T168** — out of this track |
| Migrations | Through **0026**; next free **0027** if forced (prefer **no** schema for v1) |

## 3. Research summary (online + standards, 2026-07-29)

### 3.1 Data migration practices (2025–2026 industry)

| Takeaway | T167 |
|----------|------|
| Pre-migration inventory + mapping doc before cutover | Frozen classification matrix §5 + fixture inventory §2 |
| **Idempotent writes** / natural keys so retries do not duplicate | uuid v5 from original `event_id` (L3) |
| Dry-run / pilot on representative subset first | `dry_run` default; fixture RED before apply (L1) |
| Validation = counts + checksums + spot logic — not “record count only” | `ImportReport` totals + unresolved + hash of plan (no plaintext bodies) |
| Document mapping + exception log for audit | Plan rows carry `reason_code` + `original_event_id` |
| Phased / parallel-run rather than big-bang live cutover | Destination-only apply; live migrate = T170 human gate |
| Preserve audit/provenance across modernization | Original event ids + `LegacyAiBrains` source lineage |
| Automated classification during migration under governance | Conservative under-promote (never auto-Approved) |

**Sources (practice, not legal advice):** industry migration checklists (planning → validate → rollback); data lineage/provenance distinction (provenance = origin + authorship for regulated audit).

### 3.2 Authority & privacy alignment (project + general)

| Principle | T167 |
|-----------|------|
| ADR-0011: auto capture ≠ auto authority | Pins → Evidence; synth → Candidate only |
| GDPR-style storage limitation (technical) | Import does not resurrect forgotten; does not invent longer retention |
| NIST AI RMF (high-level): provenance + risk of over-trusting AI outputs | No agent-only upgrade to Approved / Confirmed |
| ADR-0016: legacy plaintext ≠ CE | Imported content **never** labeled cryptographic erasure |
| Capture independence | Importer is pure CP + events/store ports — **no** models/graph required |

### 3.3 Dependencies

| Item | Pin / action |
|------|----------------|
| `uuid` (v5) | Workspace **1.13+** — reuse `id_from_command` pattern |
| `serde` / `serde_json` / `time` | Existing |
| `ai-brains-events` / `store` / `core` | Existing ports |
| New crates | **Forbidden** |
| AGPL ETL / migration SaaS / unknown-git importers | **Forbidden** |
| `aes-gcm` / CE stack | **Not used** by importer |

## 4. Design locks (normative)

| ID | Lock |
|----|------|
| **L1** | **Classify-first:** public API defaults to **plan only** (`dry_run = true`). Apply requires explicit `confirm` / `dry_run = false`. |
| **L2** | **No live-vault default:** module never opens `%USERPROFILE%\.ai-brains` (or vault locator) by itself. Callers (T168) supply event stream + optional `EventWriter`/`GovernedQueryStore` for a **destination** vault. |
| **L3** | **Natural keys:** governed **domain** ids are **deterministic** via uuid v5 + kind namespaces (§5.4). Second apply → zero new aggregates (detect-already-done on projection). **Plan determinism** (`plan_hash`) is the contract; applied **envelope** `event_id` / `occurred_at` remain volatile (workspace `EventBuilder` v4 + clock), matching all other CP apply paths. |
| **L4** | **Under-promote:** never emit `ConclusionConfirmed` or `DecisionApproved` from the importer. Never invent authority from agent/system inference. |
| **L5** | **Forgotten = exclude:** final stream state `forgotten` (after `MemoryForgotten` without later `MemoryRestored`) **must not** produce active Evidence. Soft-forget tombstone honesty preserved. **Cascade:** Candidate Conclusions whose `source_memory_ids` are all forgotten/unmapped still may be planned as `unsupported = true` with reason `forgotten_source` / `missing_source` — never drop silently without a reason code; never invent Evidence for forgotten sources. |
| **L6** | **CE honesty:** import **must not** claim CE, destroy wraps, or set envelope `content_class` as if sealed. Future retention may tag class `memory_legacy` (T166) — out of T167 apply. |
| **L7** | **Zero new Cargo dependencies.** |
| **L8** | **Ensure source once (raw events):** importer appends a single `SourceRegistered` for kind **`LegacyAiBrains`** (stable `SourceId` via v5) before first `EvidenceRecorded` when absent. **Must not** call `observe_source` (that path fingerprints file/git content, records versions, and runs invalidation — wrong for legacy import). Absorbs deferred **#16** ensure-source discipline for this path. |
| **L9** | **`ApprovedLegacy` is not a `DecisionState`:** live enum has no such variant. **v1 rule:** every `DecisionRecorded` → `DecisionProposed` + open **review item**; human (or later T168 tooling) approves via normal `approve_decision`. Optional future flag may auto-approve **only** if payload + actor criteria are frozen later — **not** in v1 (payload has no approval fields). |
| **L10** | **Unknown / out-of-matrix:** `Payload::Unknown` and unlisted kinds → `unresolved` / `skip` with reason — never silent invent. |
| **L11** | **Provenance field:** every planned/applied row records `original_event_id` (+ original `memory_id` / `decision_id` when present). |
| **L12** | **Privacy inheritance (v1):** imported entity privacy = **source envelope privacy** only. If envelope privacy is missing/unparseable → **Sealed**. No multi-input merge in v1. |
| **L13** | **Capture independence:** no `ai-brains-models` / graph required. |
| **L14** | **Event log append-only:** apply only **appends** new governed events; never rewrites or deletes legacy events. |
| **L15** | **No plaintext in reports:** plan/report expose counts, ids, hashes, reason codes — not full memory bodies (may include truncated summary ≤ N chars for operator dry-run only if flag set; default **off**). |
| **L16** | **Session summaries (§5):** `SessionSummaryCreated` → Evidence (digest), not Confirmed Conclusion. Hierarchical `MemorySynthesized` → Candidate Conclusion. |
| **L17** | **source_tag fidelity (deferred #2):** preserve `MemoryPinned.source_tag` as metadata in Evidence summary / fingerprint sidecars — **do not** rewrite `changeguard:symbol` → `ledgerful:symbol` (dedup identity). |
| **L18** | **Direct event build:** importer builds `SourceRegistered` / `EvidenceRecorded` / `ConclusionProposed` / `DecisionProposed` / `ReviewItemOpened` / `LegacyImportApplied` via `build_event` (same pattern as `invalidation.rs` for reviews). **Does not** call `observe_source`, `propose_conclusion`, or `propose_decision` for the bulk path (those gate on capabilities and side-effects that do not fit import). |
| **L19** | **Default scope fallback:** `ImportOpts.default_scope: Option<ScopeRef>`. When an action needs scope and the event has no `project_id` / resolvable scope: if `default_scope` is **Some** → use it; if **None** → reason `missing_scope` (skip). **Do not** silently invent Personal or `ProjectId::nil()`. T168 may prompt and pass a scope. |
| **L20** | **Durable apply audit:** every successful **apply** (confirm=true, at least one new append or idempotent completion) appends **`LegacyImportApplied`** with `plan_hash`, class counts, timestamp — **no plaintext bodies**. Dry-run never appends it. |

## 5. Classification matrix (v1 — frozen)

### 5.0 Two-pass algorithm (required)

1. **Pass 1 — status + evidence map:** walk stream in log order; compute final memory status (pin / forget / restore); for each eligible pin/summary, compute `EvidenceId` (§5.4) and record `memory_id → EvidenceId` map (and summary memory_id when present).
2. **Pass 2 — conclusions + decisions:** emit Conclusion/Decision actions using the map; `source_memory_ids` resolve to EvidenceIds when present and not forgotten; forgotten/missing → omit from `evidence_ids`, set `unsupported = true`, reason `forgotten_source` or `missing_source` as appropriate.
3. **Dry-run still runs both passes** and computes full `plan_hash` (audit artifact without append).

### 5.1 Mapped (must implement)

| Legacy input | Final eligibility | Governed output | Notes |
|--------------|-------------------|-----------------|-------|
| `MemoryPinned` | Not forgotten at end of stream | `SourceRegistered` (once) + `EvidenceRecorded` | Source kind **`LegacyAiBrains`**. Evidence summary = pin content (or hash-only in report). **Not** Conclusion. **EvidenceId** from `memory_id` when present (§5.4). |
| `SessionSummaryCreated` | Linked `memory_id` not forgotten (if status known) | `EvidenceRecorded` | Digest Evidence; provenance includes `session_id` + original event id. Prefer `memory_id` for EvidenceId when present. |
| `MemorySynthesized` | Synth memory itself not forgotten | `ConclusionProposed` **Candidate** | Two-pass (§5.0): map `source_memory_ids` → EvidenceIds; partial/missing → `unsupported = true` + reason. **Never** Confirmed. |
| `DecisionRecorded` | Always (decision aggregate) | `DecisionProposed` + `ReviewItemOpened` | Statement from title/decision/context. **DecisionId** from **event_id** (not MemoryId cast). **Never** `DecisionApproved` (L9). Review via raw `ReviewItemOpened` + `NS_LEGACY_REVIEW` (no `open_review_item` helper exists). |
| `MemoryForgotten` | — | **Exclude** pin from import; no resurrection | Compensating stream fact. |
| `MemoryRestored` | Restores eligibility if later than forget | Re-include pin if final state active | Stream-order final status. |

### 5.2 Pass-through / ignore for classification (not ECD entities)

| Legacy input | Behavior |
|--------------|----------|
| `ProjectRegistered`, `ProjectAliasAdded` | Not classified as Evidence; optional note in report “scope context only” |
| `SessionStarted` / `Completed` / `Failed` | Ignore for E/C/D |
| `UserPromptRecorded` / `AssistantFinalRecorded` | Ignore (remain turn history if vault already has them; T168 copy) |
| Governed kinds already present (`EvidenceRecorded`, `ConclusionProposed`, …) | **Skip** — not “legacy”; count as `already_governed` |
| `SystemInitialized`, `RecoveryKitCreated`, metrics, gate rejects, policy, grants, erasure, retention, … | Skip / out-of-matrix |

### 5.3 Unresolved (reason codes)

| Code | When |
|------|------|
| `unknown_payload` | `Payload::Unknown` |
| `out_of_matrix` | e.g. `RecipePromoted`, `ConflictDetected`, `FeedbackMetric`, … |
| `forgotten` | Memory final status forgotten (pin/summary excluded) |
| `forgotten_source` | Synth references a forgotten `source_memory_id` (still may plan Candidate with `unsupported`) |
| `missing_source` | Synth references unknown/unmapped memory id |
| `empty_content` | Pin/synth/decision with empty required text after trim |
| `already_imported` | Detect-already-done on derived id (apply idempotency) |
| `already_governed` | Event is already a governed ECD kind |
| `missing_scope` | No resolvable project scope **and** `ImportOpts.default_scope` is None (L19) |

### 5.4 Stable id namespaces (uuid v5)

Reuse the T159 algorithm (`id_from_command` / DNS-namespace v5). New frozen names:

| Constant | Purpose | Name input |
|----------|---------|------------|
| `NS_LEGACY_SOURCE` | Vault-global `SourceId` | Fixed string `"legacy-ai-brains"` |
| `NS_LEGACY_EVIDENCE` | `EvidenceId` | **Prefer `memory_id`** (UTF-8 uuid) when present on pin/summary; else original **event_id**. Enables stable bi-directional memory↔evidence lookup. |
| `NS_LEGACY_CONCLUSION` | `ConclusionId` | Original synth **event_id** |
| `NS_LEGACY_DECISION` | `DecisionId` | Original `DecisionRecorded` **event_id** — **never** reinterpret `DecisionRecordedPayload.decision_id: MemoryId` as `DecisionId` |
| `NS_LEGACY_REVIEW` | `ReviewItemId` | Same decision **event_id** (deterministic; not `ReviewItemId::new()`) |
| `NS_LEGACY_IMPORT_BATCH` | Aggregate id for `LegacyImportApplied` | Caller `command_id` or plan_hash |

**Do not** use random `MemoryId::new()` / `ReviewItemId::new()` for import keys (deferred **#10** spirit).

### 5.5 `SourceKind::LegacyAiBrains`

Additive enum variant on `ai_brains_core::source::SourceKind` (serde PascalCase). Prefer first-class variant over `Other("LegacyAiBrains")`.

- Display name: `"Legacy AI-Brains"`
- Locator: optional vault path / fixture id from caller; may be empty
- Single source row per destination vault (L8)
- **Exhaustiveness sites (known):** `sources.rs` — `source_identity_string` kind_label match; `compute_fingerprint` match (route to `fingerprint_external` **if** ever called — importer still **skips** observe_source); `normalize_locator_for_kind` (fall through `_` ok). Run `ledgerful scan --impact` on `SourceKind` before edit; fix any other exhaustive matches CI finds.

### 5.6 Apply event shapes (minimum)

1. **Ensure source (raw):** `SourceRegistered` if absent (`source_id` = v5(NS_LEGACY_SOURCE, "legacy-ai-brains")). **Not** via `observe_source`.
2. **Evidence (raw):** `EvidenceRecorded` with `source_id`, `summary`, `fingerprint` = hex SHA-256 of content bytes (importer-computed; workspace `sha2` — no fingerprinter port required).
3. **Conclusion (raw):** `ConclusionProposed` Candidate; `unsupported` per §5.0; `scope` from project or `default_scope`.
4. **Decision (raw):** `DecisionProposed` + `ReviewItemOpened` (`review_item_id` = v5(NS_LEGACY_REVIEW, decision event_id); subject = title; `related_decision_id` set). No CP `open_review_item` helper exists today.
5. **Audit (raw):** `LegacyImportApplied` (L20) after apply batch — plan_hash, counts, no bodies.
6. **Never:** `DecisionApproved` / `ConclusionConfirmed` / CE wipe / `RetentionApplied` / `observe_source`.

### 5.7 Idempotency probes

| Entity | Probe |
|--------|--------|
| Conclusion | existing `has_conclusion` |
| Decision | existing `has_decision` |
| Evidence | **add** `GovernedQueryStore::has_evidence(EvidenceId) -> Result<bool>` (+ `StoreGovernedQuery` adapter). Do **not** misuse `evidence_privacy` as a presence probe. |
| Source | existing `get_source` / list by identity |
| Review | open list or get by id if available; else detect via prior apply / plan already_imported |

## 6. API surface (control-plane)

**Module:** `crates/ai-brains-control-plane/src/legacy_import.rs`  
**Export** from `lib.rs`.

```text
classify_legacy(events, opts) -> ImportPlan
apply_legacy_import(writer, query, clock, plan, ApplyOpts) -> ImportReport
```

| Type | Role |
|------|------|
| `ImportOpts` | `dry_run` (default **true**), `include_truncated_summaries` (default false), `default_scope: Option<ScopeRef>` (L19), principal/actor for envelopes |
| `ImportPlan` | ordered `Vec<ImportAction>`, totals, unresolved, `plan_hash` (§6.1). Dry-run builds the **full** plan (derived ids + hash) as the audit artifact. |
| `ImportAction` | kind, original_event_id, derived_id, reason_code, mechanism (`would_append` / `skip`); optional metadata (source_tag); **no** body fields in hash view |
| `ApplyOpts` | `confirm: bool` — must be true to append |
| `ImportReport` | applied/skipped/already_imported/unresolved counts, truncated event ids, `plan_hash`, whether `LegacyImportApplied` appended |

### 6.1 Canonical `plan_hash`

1. Build a sorted map keyed by `(original_event_id, action_kind)` (stable string kind tags).
2. Each value is an **ActionView**: `derived_id`, `reason_code`, `mechanism`, optional non-body metadata (`source_tag`, `unsupported` bool). **Exclude** content/summary/statement/title body fields.
3. Serialize with `serde_json` over a structure that sorts keys (e.g. `BTreeMap` then `serde_json::to_vec`).
4. `plan_hash` = lowercase hex SHA-256 of those bytes.
5. Same logical plan with reordered input actions → **same** hash. Body-only changes → same hash. Id/reason changes → different hash.

### 6.2 Policy / capability honesty

**There is no `RecordEvidence` capability** (`GrantCapability` has `ReadEvidence`, `ProposeConclusion`, `ProposeDecision`, …). Spec §6 must not claim it.

| Approach | T167 choice |
|----------|-------------|
| `observe_source` + `ReadEvidence` | **Rejected** — wrong side-effects (B4) |
| Per-event propose_* with grants | **Rejected** for bulk import (L18) |
| **Direct `build_event` append** | **Required** (L18) |

**Import boundary policy (v1):** caller (T168/tests) is responsible for whether import is allowed (path safety, dest vault choice). Importer may optionally check a single coarse grant if one is added later; **do not** invent `RecordEvidence`. Tests may use `AllowAllPolicy` only for unrelated harness paths — import apply path does not require policy evaluator for each entity. Document: production CLI must not open live vault as dest without explicit operator flags (T168).

## 7. Non-goals

| Out of scope | Owner |
|--------------|--------|
| CLI `migrate governed` + differential report file | **T168** |
| Evaluation corpus / metrics | **T169** |
| Live vault cutover / dogfood gate | **T170** |
| Content-envelope seal of imported text | P8 residual / later |
| Rewriting historical legacy events | Forbidden |
| Auto-approve decisions from `DecisionRecorded` | L9 |
| Port-level connector list cursors | deferred #23 |
| ChangeGuard → Ledgerful symbol rename / source_tag migration | deferred #1 / #2 (preserve only) |
| Full projection rebuild truncate of legacy conflict tables | deferred #19 |

## 8. Deferred items absorbed / related

| Deferred | Disposition |
|----------|-------------|
| **#16** ensure-source for evidence | **Absorbed (L8):** Legacy path always `SourceRegistered` before Evidence |
| **#18** session-summary dual-path residual | **Partial absorb (L16):** importer classifies `SessionSummaryCreated` as Evidence digest; does **not** rewrite live synthesizer |
| **#10** turn memory_id non-determinism | **Spirit absorb (L3/§5.4):** Evidence prefers stable `memory_id`; DecisionId from event_id not MemoryId cast |
| **#2** `changeguard:symbol` source_tag | **Absorb (L17):** preserve; no rewrite |
| **#20** nil ProjectId sentinel | **Spirit:** do not invent authoritative scope via `ProjectId::nil()` |
| ADR-0016 legacy ≠ CE | **L6** |
| T166 `memory_legacy` class | Document only — retention tagging later |
| **#15** source_alias write path | **Out of scope** |
| **#19** replay truncate legacy tables | **Out of scope** |
| **#1** ChangeGuard type renames | **Out of scope** |

Append a short **#39** note in `conductor/deferred.md` for T167 freezes (mirror T165/T166 pattern).

## 9. Testing strategy (TDD)

### 9.1 RED first (names)

| Test | Assert |
|------|--------|
| `classify__memory_pinned__evidence_not_conclusion` | Pin → Evidence action; zero Conclusion actions |
| `classify__memory_synthesized__candidate_only` | Synth → ConclusionProposed Candidate; no Confirmed |
| `classify__decision_recorded__proposed_plus_review` | Decision → Proposed + review; no Approved |
| `classify__forgotten_memory__excluded` | Pin then Forget → forgotten skip; no Evidence |
| `classify__forgotten_then_restored__included` | Forget then Restore → Evidence |
| `classify__synth_referencing_forgotten_source__unsupported_true` | Two-pass: forgotten child → unsupported + `forgotten_source` |
| `classify__unknown_payload__unresolved` | Unknown → unresolved reason |
| `classify__idempotent_ids__stable_v5` | Same memory_id/event_id → same derived ids |
| `classify__evidence_id_prefers_memory_id` | Pin with memory_id → EvidenceId = v5(NS, memory_id) ≠ v5(NS, event_id) when they differ |
| `classify__decision_id_not_memory_id_cast` | DecisionId = v5(NS_LEGACY_DECISION, event_id) ≠ MemoryId bytes cast |
| `classify__missing_scope_without_default__skipped` | No project + default_scope None → missing_scope |
| `classify__default_scope_fallback__used` | No project + default_scope Some → uses fallback |
| `plan_hash__same_input_same_hash` | Identical inputs → identical plan_hash |
| `plan_hash__reordered_actions_same_hash` | Shuffle action order before hash → same hash |
| `plan_hash__omits_body_plaintext` | Changing only summary body → same plan_hash |
| `apply__second_run__zero_new_aggregates` | Apply twice → second already_imported (uses `has_evidence`) |
| `apply__ensures_legacy_source_once` | One SourceRegistered for many Evidence |
| `apply__appends_legacy_import_applied` | Confirm apply → one LegacyImportApplied with plan_hash |
| `apply__does_not_call_observe_source` | No SourceVersionRecorded / invalidation side-effects |
| `classify__session_summary__evidence_digest` | SessionSummaryCreated → Evidence |
| `classify__preserves_source_tag_metadata` | source_tag in plan metadata |
| `report__no_full_plaintext_by_default` | Default plan serialization omits body fields |
| Fixture: `legacy-v1-events.ndjson` → frozen plan totals | Golden counts |

### 9.2 Harness rules

- `tempfile` vaults only; no live vault
- No network; no models
- Prefer pure `classify` unit tests with constructed envelopes; apply tests with store ports
- Naming: `function_or_feature__condition__expected_result`

## 10. License / commercial

- PolyForm NC + Small-Entity Commercial Exception unchanged
- No AGPL importers / eval frameworks
- `cargo deny check` clean; zero new deps

## 11. Definition of Done

- [ ] `legacy_import.rs` + exports; two-pass classify; raw apply (L18)
- [ ] `SourceKind::LegacyAiBrains` + raw ensure-source (no observe_source)
- [ ] `has_evidence` on GovernedQueryStore + adapter
- [ ] `LegacyImportApplied` payload/kind + apply audit (L20)
- [ ] Classification matrix §5 + plan_hash §6.1 proven by tests
- [ ] Idempotent apply; dual-id rules §5.4
- [ ] Forgotten never resurrected; forgotten_source cascade honest
- [ ] No auto Approved/Confirmed; no CE claims; no RecordEvidence fiction
- [ ] Fixture NDJSON plan golden
- [ ] Manual: classify fixture → report counts (no live apply)
- [ ] conductor Completed + review clean of open high when implemented

## 12. Expand checklist

- [x] Inventory legacy payload variants
- [x] Freeze `ApprovedLegacy` → none in v1
- [x] Source registry `LegacyAiBrains`
- [x] RED test names (incl. review fold-in)
- [x] Unknown payload fidelity (L10)
- [x] Deferred fold-in (#16, #18, #10, #2, ADR-0016)
- [x] AI review fold-in 2026-07-29 (L18–L20, two-pass, has_evidence, plan_hash §6.1, no RecordEvidence, no observe_source)

## 13. Open questions (resolved)

| Question | Resolution |
|----------|------------|
| Cascade / re-synth on purge? | **T166 R15** — not T167 |
| Auto-approve historical decisions? | **No** (L9) |
| Where does CLI live? | **T168** |
| Schema migration for original_event_id column? | **No** in v1 — derive id = provenance |
| Add `LegacyImportApplied` event? | **Yes on apply** (L20) — plan_hash + counts; no bodies |
| Default scope for early sessions without project_id? | **Caller-supplied** `ImportOpts.default_scope` (L19); else `missing_scope`. Not silent Personal. T168 may prompt. |
| RecordEvidence capability? | **Does not exist** — direct `build_event` (L18 / §6.2) |
| Dual-id MemoryId vs event_id? | Evidence prefers **memory_id**; DecisionId from **event_id** only (§5.4) |

## 14. Review fold-in log (2026-07-29)

| Source | Finding | Disposition |
|--------|---------|-------------|
| AI1 | Provenance + v5 + under-promote | Already L3/L4/L11 — reaffirmed |
| AI1 | default_scope for early sessions | **Agreed** as optional L19 (not silent Personal) |
| AI1 | EvidenceId prefer memory_id | **Agreed** §5.4 |
| AI1 | forgotten_source cascade | **Agreed** L5 + two-pass + reason codes |
| AI1 | LegacyImportApplied durable audit | **Agreed** L20 (reversed earlier optional-no) |
| AI2 B1 | No RecordEvidence | **Agreed** §6.2 |
| AI2 B2 | No open_review_item helper | **Agreed** raw ReviewItemOpened + NS_LEGACY_REVIEW |
| AI2 B3 | event_id volatility | **Agreed** L3 plan-determinism note |
| AI2 B4 | Do not observe_source | **Agreed** L8/L18 |
| AI2 B5 | has_evidence port | **Agreed** §5.7 |
| AI2 B6 | Two-pass synth linkage | **Agreed** §5.0 |
| AI2 B7 | DecisionId ≠ MemoryId cast | **Agreed** + RED test |
| AI2 B8 | Simplify L12 multi-input | **Agreed** |
| AI2 B9 | SourceKind exhaustiveness sites | **Agreed** §5.5 list |
| AI2 B10 | plan_hash canonicalization | **Agreed** §6.1 |
| AI2 | Dep pin bumps in T167 | **Declined** — separate INFRA; note in deferred #40 |
| AI2 | evidence_privacy as probe | **Declined** — use has_evidence |
