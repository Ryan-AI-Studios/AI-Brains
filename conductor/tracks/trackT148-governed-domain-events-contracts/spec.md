# T148 — Governed Domain, Events, and Public Contracts (P1)

- **Track ID:** T148-GovernedDomainEventsContracts
- **Phase:** P1 — Additive domain, event, and public-contract foundation
- **Master plan:** `.hermes/plans/2026-07-23_204630-memory-control-plane-successor.md` (Tasks 1.1–1.5)
- **Execution repo:** `C:\dev\AI-Brains`
- **Status:** Pending
- **Category:** ARCHITECTURE / FEATURE
- **Depends on:** T146 (architecture), T147 (baseline fixtures + edition 2024) — both **Complete**
- **Branch (suggested):** `feature/governed-domain-events-contracts`
- **Relevant ADRs:** ADR-0010 (evolve, don’t rewrite), ADR-0011 (Evidence / Conclusion / Decision)

## 1. Objective

Represent governed-memory epistemic and scope concepts as **additive** domain types, event facts, and versioned public DTOs—**without** changing legacy event meaning, projection behavior, or serving paths.

After T148:

- Core can express Source, Evidence, Conclusion, Decision, Scope, Principal, Freshness, etc.
- Events can serialize/deserialize new fact kinds at `CURRENT_SCHEMA_VERSION = 1`.
- Contracts expose typed DTO surfaces (handles, not prose-only).
- `ai-brains-control-plane` exists with **ports only** (no workflows).
- T147 `fixtures/governed-memory/legacy-v1-events.ndjson` still loads and matches golden projections.

## 2. Context (live inspection 2026-07-24)

| Area | Live state |
|------|------------|
| Edition / toolchain | Edition **2024**, rustc **1.95.0** (T147) |
| IDs | `define_id!` in `ai-brains-core/src/ids.rs`; no Source/Evidence/Conclusion/… IDs yet. Unused `KnowledgeId` exists — **keep**; do not repurpose as DecisionId |
| Domain | Thin `memory`, `project`, `session`; status transitions via `SessionStatus::transition` + `Error::InvalidStatusTransition` |
| Events | Internally tagged `Payload` (`tag = "type"`) ends in unit `#[serde(other)] Unknown` — **does not preserve raw fields** (serde cannot capture content into a unit variant). `EventKind` same pattern. `AggregateType` has no `other` |
| Read→write hazard | `SqliteEventStore::append_event` re-serializes **typed** `envelope.payload` (`event_store.rs`). T147 `shadow create` deserializes `Envelope`s and re-appends (`shadow.rs`) — unknown payloads today become empty `Unknown` on write-back |
| Legacy decision | `DecisionRecorded` / `DecisionRecordedPayload.decision_id: MemoryId` — **must remain readable** |
| Schema version | `CURRENT_SCHEMA_VERSION = 1` — **do not bump** in P1 |
| Projections | Match arms end in `_ => {}` / `_ => return Ok(())` — new payloads are no-ops until P2+ |
| Contracts timestamps | Existing DTOs use `chrono::DateTime<Utc>` (`backup`, `bridge`, `memory`, `sessions`) — **not** bare `String` |
| Contracts | `ApiResult` / `ApiError` without `api_version`; no knowledge/briefing modules |
| Control-plane crate | **Absent** |
| Events integration tests | **No** `crates/ai-brains-events/tests/` yet |
| T147 fixture | `fixtures/governed-memory/legacy-v1-events.ndjson` + golden projections |
| Dirty unrelated | `.agents/skills/codex-review/SKILL.md` — never stage |

### External standards & practices consulted (2026-07-24)

Use as **design alignment**, not new product dependencies:

| Source | Takeaway for T148 |
|--------|-------------------|
| [W3C PROV-DM](https://www.w3.org/TR/prov-dm/) | Entity / Activity / Agent map loosely to Evidence & sources / derivation events / Principal. Do **not** implement full PROV ontology. |
| DecPROV / decision provenance literature | Decisions are first-class “why” objects, not generic memory rows — matches ADR-0011. |
| Provenance-enhanced KG work (e.g. DEC framing, 2026) | Keep epistemic stance explicit; do not collapse competing claims into one “fact.” |
| Hexagonal / ports & adapters (Cockburn; Rust ports-as-traits guides 2025–2026) | Ports = traits owned by application crate; adapters later. **No** third-party hex framework crate (`hexser`, etc.). |
| Serde enum evolution | New **known** variants go before the catch-all. Unit `#[serde(other)]` is **insufficient** for event-sourcing write-back: unknown content must round-trip. Internally tagged enums need custom `Deserialize`/`Serialize` (or dual raw+typed storage) to preserve unknown objects — derive alone cannot fill a content-bearing `other` for this shape ([serde enum representations](https://serde.rs/enum-representations.html); community notes on tagged + other limits). |
| Additive API versioning | Prefer optional fields + `#[serde(default)]` and an explicit `api_version` string on new response envelopes; avoid breaking renames. |
| Workspace deps (already pinned) | `thiserror` 2.x, `serde` 1.x, `uuid` 1.x, `time` 0.3, `chrono` 0.4 (contracts), edition 2024 — **reuse**; do not introduce graph/PROV crates. |

## 3. In scope

0. **Preserve-unknown payload (and event-kind tag) fidelity** so cross-version shadow / replay cannot strip governed facts (blocking for adding ~25 new variants).
1. Typed IDs for governed entities (Task 1.1).
2. Domain modules + legal state transitions (Task 1.2).
3. Event kinds, aggregates, payloads, constructors validation where needed (Task 1.3).
4. Versioned public DTOs + golden JSON fixtures (Task 1.4).
5. `ai-brains-control-plane` skeleton: errors + ports only (Task 1.5).
6. RED→GREEN tests listed below; full gate before Complete.

## 4. Out of scope

- SQL migrations `0020+`, source fingerprints, invalidation workflows (P2).
- Lifecycle services, review queues implementation (P3) — types/events only.
- Scope policy enforcement runtime (P4) — types/capabilities only.
- Briefing generation / progressive retrieval (P5).
- CLI/daemon/HTTP serving changes (P7).
- Changing meaning of `Memory*` / `DecisionRecorded` events.
- Envelope schema v2 / content encryption (P8).
- Staging unrelated skill file; merge/push without Ryan approval.

## 5. Requirements

### R0 — Unknown event fidelity (blocking; do before or with first new Payload variants)

**Problem (verified):** `Payload::Unknown` is a unit variant with `#[serde(other)]`. Deserialization drops all unknown fields. `append_event` and `shadow create` re-serialize the typed enum, so any unrecognized payload is permanently emptied in the destination vault. That violates the spirit of append-only event authority and breaks Phase 9/11 cross-version assumptions once T148 introduces many new variants.

**Required design (choose one; document in `review.md`):**

| Option | Shape | Notes |
|--------|--------|--------|
| **A (preferred)** | `Payload::Unknown(serde_json::Value)` with **custom** `Serialize`/`Deserialize` | On unknown `type`, capture the **entire** JSON object; on serialize, emit that object **verbatim** (not `{"type":"Unknown"}`). Known variants keep current internally tagged shape. |
| **B** | Envelope dual field: keep `payload_json: String` (or `Value`) as source of truth + best-effort typed `payload` | Larger envelope change; only if A cannot preserve `payload_hash` / equality cleanly. Still **no** schema_version bump if wire format of stored events remains the raw JSON object in `payload_json` column. |

Also for `EventKind`:

- Prefer `EventKind::Unknown(String)` (or equivalent) that preserves the **original tag string** on deserialize and emits that string on serialize—not the literal `"Unknown"`.
- If custom impl is shared, keep PascalCase for known kinds.

**Behavioral requirements:**

1. Deserialize unknown payload → non-empty preserved content; re-serialize → JSON **semantically equal** to input (same `type` + fields; key order may follow `serde_json::Value` rules—assert via `Value` equality, not raw string equality unless using option B raw string).
2. `payload_hash`: for `Unknown`, either (a) recompute hash from canonical serialization of preserved `Value` consistently, or (b) when shadow copies, **do not recompute** hash for non-redacted events—copy envelope fields as-is. Shadow redaction must only rewrite known turn payloads; **`Unknown` must pass through unchanged** (including `payload_hash`).
3. Projections: `Unknown` remains a no-op (`_ => {}`).
4. **Do not** land ~25 new variants until R0 tests are green (ordering: R0 → then R3 variants).

**RED tests (strengthen beyond “does not panic”):**

- `unknown_future_payload_degrades_safely.rs` (rename ok):  
  - Input JSON with `"type":"TotallyFutureEvent","foo":1,"bar":"x"`.  
  - Assert deserializes to `Payload::Unknown(...)`.  
  - Assert re-serialized `Value` equals input `Value`.  
  - Assert original fields recoverable (e.g. `foo == 1`).
- `unknown_payload_shadow_roundtrip_preserves_raw.rs` (events unit test **or** store/cli test):  
  Append/load path that round-trips an envelope with unknown payload through serialize→deserialize→serialize without field loss. Prefer pure events-crate test plus one store tempfile append/read if cheap.
- Known legacy fixtures still deserialize (no regression to unit-Unknown data loss on **known** types).

### R1 — Typed IDs

Add via existing `define_id!` macro in `ids.rs`:

```text
SourceId, SourceVersionId, EvidenceId, ConclusionId, DecisionId,
WorkspaceId, PrincipalId, GrantId, ReviewItemId, BriefingId,
QueryTraceId, ContentKeyId, TombstoneId, ReplicationEventId
```

Rules:

- `DecisionId` is **not** `MemoryId`.
- Legacy `DecisionRecordedPayload.decision_id: MemoryId` unchanged.
- Every new ID: serde JSON round-trip (extend `id_serde_roundtrip.rs` with `rstest` or explicit cases; assert equality).

### R2 — Domain modules and transitions

Create modules (names match master plan):

| Module | Minimum types |
|--------|----------------|
| `source` | `SourceKind` |
| `evidence` | `EvidenceStatus` |
| `conclusion` | `ConclusionState` + `transition` / approval-gated rules |
| `decision` | `DecisionState` + approval-gated rules |
| `scope` | `ScopeRef`, grant strictest-wins helpers |
| `principal` | principal identity shell (id + kind as needed for events) |
| `briefing` | briefing kind / handle shell (not full packet builder) |
| `freshness` | `FreshnessState`; freshness ≠ recency |
| `model_provenance` | model/provider/workflow version fields for derivations |

Required enums (exact set from master plan / CONTEXT.md):

```rust
SourceKind { GitRepository, File, ObsidianVault, Ledgerful, HermesSession, Honcho, Manual, Other(String) }
EvidenceStatus { Active, Superseded, Unavailable, Erased }
ConclusionState { Candidate, Active, Confirmed, Stale, Disputed, Superseded, Rejected }
DecisionState { Proposed, Approved, Superseded, Revoked }
ScopeRef { Repository(ProjectId), Workspace(WorkspaceId), Personal(UserId) }
GrantCapability { ReadEvidence, ReadConclusions, ReadDecisions, ProposeConclusion, ApproveConclusion, ProposeDecision, ApproveDecision, Export, Erase }
FreshnessState { Current, RevalidationDue, Stale, SourceUnavailable, Unknown }
```

Transition rules (must be enforced in pure functions, tested):

| Illegal without extra authority | Allowed example |
|---------------------------------|-----------------|
| `ConclusionState::Candidate → Confirmed` without human approval principal | `Candidate → Active` under policy (non-protected path) as pure function allows **only** if API takes `ApprovalAuthority` and rejects `None` for Confirmed |
| `Stale → Active` without source revalidation token | `Stale → Active` with `RevalidationProof` |
| `DecisionState::Proposed → Approved` without approver | `Proposed → Approved` with `PrincipalId` approver |

Prefer extending `ai_brains_core::Error` with specific variants (or reuse `InvalidStatusTransition` with clear `from`/`to` strings). **No** `unwrap`/`expect` in production.

Timestamps:

| Layer | Rust type | Notes |
|-------|-----------|--------|
| Domain (`ai-brains-core`) + events | `time::OffsetDateTime` | Matches `Envelope.occurred_at` |
| Contracts DTOs | **`chrono::DateTime<Utc>`** | Matches existing `backup` / `bridge` / `memory` / `sessions` — **not** bare `String` |
| Conversion | Single helper | e.g. `fn offset_to_utc(t: OffsetDateTime) -> DateTime<Utc>` in contracts or a tiny shared util — **one place**, not per-module ad hoc |

### R3 — Event facts (not service commands)

**Modify:** `event_kind.rs`, `payload.rs`, `aggregate.rs`, `lib.rs`, `constructors.rs` only as needed.

New `AggregateType` values (additive; keep existing): e.g. `Source`, `Evidence`, `Conclusion`, `Workspace`, `Principal`, `Grant`, `ReviewItem`, `Briefing`, `QueryTrace` (and any strictly required for event grouping). Existing `Decision` aggregate remains for both legacy and new decision events if appropriate—or use new aggregate only for new payloads; **document choice in plan.md**. Prefer: keep `Decision` for decision events; do not rename.

New `EventKind` + matching `Payload` variants (master plan list):

```text
SourceRegistered, SourceObserved, SourceVersionRecorded, SourceUnavailable,
EvidenceRecorded, EvidenceSuperseded,
ConclusionProposed, ConclusionActivated, ConclusionConfirmed,
ConclusionMarkedStale, ConclusionDisputed, ConclusionSuperseded, ConclusionRejected,
DecisionProposed, DecisionApproved, DecisionSuperseded, DecisionRevoked,
WorkspaceRegistered, RepositoryJoinedWorkspace,
ScopeGrantIssued, ScopeGrantRevoked,
PrincipalRegistered,
ReviewItemOpened, ReviewItemResolved,
BriefingGenerated, QueryTraceRecorded,
ContentErasureRequested, ContentErased
```

Payload rules:

- Reference IDs; **no** mutable projection snapshots embedded.
- **`DecisionApproved` (decided shape — lock before Phase B/C tests):**
  ```text
  decision_id: DecisionId          // identity of the decision aggregate
  proposal_event_id: Uuid          // event_id of the DecisionProposed fact (required)
  approver: PrincipalId            // human (or authorized) approver — required
  approved_at: OffsetDateTime      // approval time — required
  ```
  Domain transition `Proposed → Approved` requires `approver: PrincipalId` (and may take `proposal_event_id` only at event layer). No silent auto-approve.
- `ConclusionMarkedStale`: changed `SourceVersionId` and/or unavailable reason.
- `EvidenceRecorded`: source id/version/fingerprint; optional `ModelProvenance` when model-derived.
- Catch-all is **R0 content-preserving** `Unknown`, not unit `#[serde(other)]` alone.
- New known variants are listed **before** the catch-all.
- `CURRENT_SCHEMA_VERSION = 1`.

### R4 — Public contracts

Create:

- `sources.rs`, `knowledge.rs`, `scopes.rs`, `briefings.rs`, `review.rs`, `policy.rs`

Rules:

- IDs as strings in JSON.
- Timestamps: **`chrono::DateTime<Utc>`** on all new contract fields that represent instants (see R2 table).
- Additive optional fields: `#[serde(default)]` / `skip_serializing_if` as existing style.
- New response types include `api_version: String` (constant **`"1"`** unless a contracts module already defines another scheme—document if different).
- Briefing/query DTOs expose **evidence handles** (ids + optional cite labels), not prose-only.
- Policy denials: structured `ApiError`-compatible code/details (e.g. `POLICY_DENIED`), never empty success.

Extend `contracts_are_backward_compatible.rs`; add golden JSON under `tests/fixtures/`.

### R5 — Control-plane ports skeleton

New workspace member `crates/ai-brains-control-plane`:

```rust
// ports only — no real adapters
pub trait EventWriter { /* append governed events atomically */ }
pub trait GovernedQueryStore { /* typed projection reads */ }
pub trait Clock { /* thin; may wrap/delegate to ai_brains_core::clock */ }
pub trait Fingerprinter { /* deterministic source fingerprint */ }
pub trait PolicyEvaluator { /* principal + capability + scope */ }
```

- `errors.rs` with `thiserror` (+ `miette` only if CLI-facing later; internal `thiserror` is enough for skeleton).
- **No** business workflows, no SQLite, no CLI wiring.
- Sync traits (core is sync; async only at daemon boundary).

### R6 — Compatibility gates

- `cargo test -p ai-brains-store --test governed_fixture_replay` still green.
- Legacy NDJSON deserializes to full `Envelope` list in events tests.
- R0 unknown round-trip tests green **before** marking event-phase complete.
- No new migrations; no live vault; no serving behavior change.
- Shadow behavior: document that post-T148 binaries preserve unknown payloads on copy; optional follow-up note in T147 residuals if old binaries remain in the wild (cannot fix old binaries retroactively—R0 protects **new** code paths).

## 6. Files

### Create

| Path |
|------|
| `crates/ai-brains-core/src/{source,evidence,conclusion,decision,scope,principal,briefing,freshness,model_provenance}.rs` |
| `crates/ai-brains-core/tests/{conclusion_state_transitions,decision_requires_approval,scope_grant_strictest_wins,freshness_is_not_age_only}.rs` |
| `crates/ai-brains-events/tests/{governed_payload_roundtrip,legacy_v1_events_still_deserialize,unknown_future_payload_degrades_safely,unknown_payload_roundtrip_preserves_fields}.rs` |
| `crates/ai-brains-contracts/src/{sources,knowledge,scopes,briefings,review,policy}.rs` |
| `crates/ai-brains-contracts/tests/fixtures/*.json` |
| `crates/ai-brains-control-plane/{Cargo.toml,src/lib.rs,src/errors.rs,src/ports.rs}` |
| `conductor/tracks/trackT148-governed-domain-events-contracts/{spec,plan,review}.md` |

### Modify

| Path | Change |
|------|--------|
| `crates/ai-brains-core/src/ids.rs` | New IDs |
| `crates/ai-brains-core/src/lib.rs` | `pub mod` exports |
| `crates/ai-brains-core/src/errors.rs` | Transition/approval errors as needed |
| `crates/ai-brains-core/tests/id_serde_roundtrip.rs` | All new IDs |
| `crates/ai-brains-events/src/{event_kind,payload,aggregate,lib,constructors}.rs` | R0 unknown fidelity + new facts |
| `crates/ai-brains-cli/src/commands/shadow.rs` | Only if needed so `Unknown` is pass-through (no forced re-hash/strip); prefer events-layer fix alone |
| `crates/ai-brains-contracts/src/lib.rs` | Export new modules |
| `crates/ai-brains-contracts/tests/contracts_are_backward_compatible.rs` | New DTO compat cases |
| Root `Cargo.toml` | workspace member + path deps if needed |
| `conductor/conductor.md` | T148 status |

### Explicitly do not modify (unless forced by exhaustiveness)

- Migrations `0001`–`0019`
- Projection SQL / serving CLI (if exhaustiveness forces `_` arms only, touch minimally and note)
- T147 fixtures content (read-only consume)

## 7. Migration / rollback

- **No SQL migrations.**
- Rollback = revert commits; additive types only.
- If a match is forced exhaustive without `_`, restore `_ => {}` no-ops.

## 8. Definition of Done

- [x] **R0** unknown payload (and event-kind tag) round-trip preserves fields; unit `Unknown` data-loss path eliminated.
- [x] All R1–R6 requirements met with RED→GREEN evidence.
- [x] `DecisionApproved` shape locked as in R3 (decision_id + proposal_event_id + approver + approved_at).
- [x] `cargo test -p ai-brains-core`
- [x] `cargo test -p ai-brains-events` (new tests)
- [x] `cargo test -p ai-brains-contracts`
- [x] `cargo test -p ai-brains-control-plane`
- [x] `cargo test -p ai-brains-store --test governed_fixture_replay`
- [x] Full gate: fmt, clippy `-D warnings`, nextest workspace, deny, audit, `ledgerful verify --scope full`
- [x] Cross-model review for ARCHITECTURE/FEATURE (codex-review skill) before Complete
- [x] `review.md` with evidence; residuals → `deferred.md` / ISSUES per AGENTS
- [x] Ledger transaction committed; skill file excluded
- [x] Stop before merge/push without approval

## 9. Stop conditions

- Need envelope v2 or migration to express a field (R0 option B touching stored columns differently than today).
- Serving behavior would have to change for tests to pass beyond shadow pass-through.
- Scope creep into P2 fingerprints/workflows.
- R0 cannot preserve unknown payloads without unacceptable complexity — stop and propose minimal Envelope dual-field design for approval before continuing R3 variants.
```