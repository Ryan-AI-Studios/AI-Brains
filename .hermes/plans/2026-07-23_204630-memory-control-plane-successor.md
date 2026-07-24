# Governed Memory Control Plane Successor — Implementation Plan

> **Plan ID:** `2026-07-23_204630-memory-control-plane-successor`  
> **Repository:** `C:\dev\ai-brains` (`/mnt/c/dev/ai-brains` from WSL)  
> **Status:** Ready for execution after review; no implementation in this plan commit  
> **Audience:** A fresh coding harness with no prior conversation context  
> **Required method:** strict RED → GREEN → REFACTOR, ChangeGuard/Ledgerful transaction per implementation track, evidence recorded before status changes

## Objective

Evolve AI-Brains into the local-first governed memory control plane defined by:

- `CONTEXT.md`
- `Docs/MEMORY-CONTROL-PLANE-VISION.md`
- `Docs/RESEARCH/memory-systems-comparison-2026-07.md`
- `Docs/DECISIONS/ADR-0010-evolve-ai-brains-into-successor.md`
- `Docs/DECISIONS/ADR-0011-separate-evidence-conclusions-decisions.md`
- `Docs/DECISIONS/ADR-0012-local-first-control-plane-and-public-protocol.md`
- `Docs/DECISIONS/ADR-0013-distinct-briefings-and-scope-hierarchy.md`
- `Docs/DECISIONS/ADR-0014-source-aligned-freshness-and-explicit-conflict.md`
- `Docs/DECISIONS/ADR-0015-event-ledger-erasure-and-encrypted-replication.md`

The first usable product slice must let a cold-start agent request a cited, freshness-aware Project Briefing from a copied/shadow AI-Brains vault, inspect the underlying Evidence, query for more detail, and submit a proposed Conclusion for governed review. It must not mutate the user's live vault during development or evaluation.

---

## 1. Non-negotiable execution rules

1. **Do not rewrite AI-Brains.** Extend its crates and events additively.
2. **Do not rename `Memory` to `Evidence`.** Existing `Memory` events and projections remain readable. New epistemic entities are additive and may link legacy memories as imported evidence.
3. **Do not alter the event envelope in the first four phases.** Keep `CURRENT_SCHEMA_VERSION = 1`; add new event variants and payloads. Introduce envelope v2 only if the content-encryption spike proves it necessary.
4. **Never edit an already-applied SQL migration.** Current migrations stop at `0019_embedding_timestamp.sql`; new work starts at `0020`.
5. **Never test migrations against the live vault.** Use `tempfile` databases, generated fixtures, and a redacted read-only shadow copy.
6. **No chain-of-thought capture.** Persist prompts, finals, structured action evidence, digests, model metadata, and explicit rationale supplied for decisions—not hidden reasoning.
7. **No inferred Decision authority.** An agent may create Evidence and Candidate Conclusions. A Decision requires an authorized human approval event.
8. **No newest-write-wins conflict erasure.** Preserve incompatible claims and expose resolution status.
9. **No plaintext cloud processing by default.** Local deterministic and local-model paths must work without cloud credentials.
10. **No cross-scope leakage.** Repository, Workspace, and Personal data require explicit grants.
11. **No silent degraded mode.** A briefing must report unavailable sources, stale dependencies, budget truncation, and policy filtering.
12. **No commit, merge, or push that includes unrelated files.** In particular, exclude the pre-existing `.agents/skills/codex-review/SKILL.md` modification unless separately authorized.
13. **STOP before merge or push to `main`.** Ryan must explicitly approve that action.
14. **One implementation track per phase.** Start a Ledgerful transaction, create a dedicated branch, implement, verify, record evidence, commit the transaction, and stop for review.

---

## 2. Verified current-state map

The next harness must re-run `git status --short --branch` before implementation, but the plan is based on these inspected components:

| Concern | Existing implementation to reuse | Important gap |
|---|---|---|
| Domain IDs | `crates/ai-brains-core/src/ids.rs` | No Source, Evidence, Conclusion, Decision, Workspace, Principal, Briefing, Review, Grant, or ContentKey IDs |
| Existing Memory | `crates/ai-brains-core/src/memory.rs` | Thin content record; no epistemic class or provenance graph |
| Existing Project | `crates/ai-brains-core/src/project.rs` | No stable repository identity, workspace membership, or explicit grants |
| Actor attribution | `crates/ai-brains-events/src/actor.rs` | Actor exists, but no policy principal/capability model |
| Event envelope | `crates/ai-brains-events/src/envelope.rs` | Has actor, causation, correlation, privacy, and hash; no encrypted content reference |
| Event kinds/payloads | `crates/ai-brains-events/src/event_kind.rs`, `payload.rs` | Existing `DecisionRecorded` is rendered into generic memory and lacks approval authority |
| Upcasting | `crates/ai-brains-events/src/upcast.rs`, `version.rs` | Upcaster is a stub; avoid unnecessary v2 until required |
| SQLite event store | `crates/ai-brains-store/src/event_store.rs` | Suitable append-only authority |
| Migrations | `crates/ai-brains-store/src/migrations.rs`, migrations `0001`–`0019` | Must add only new numbered migrations |
| Projections | `crates/ai-brains-store/src/projections/*` | Generic memory projection flattens evidence, synthesis, and decisions |
| Replay | `crates/ai-brains-store/src/replay.rs` | Hard-coded projection truncation must be extended for every new projection |
| FTS | `crates/ai-brains-store/src/fts.rs`, migration `0007`, `0008` | Search result lacks authority, freshness, dependency, and scope fields |
| Recall | `crates/ai-brains-retrieval/src/recall.rs` | Blending ranks by score/source order, not trust/freshness/policy |
| Preflight | `crates/ai-brains-retrieval/src/preflight.rs` | Produces prose heuristically from generic memories; no typed packet or citations |
| Contracts | `crates/ai-brains-contracts/src/*` | Existing responses do not expose evidence handles, traces, freshness, or policy decisions |
| Daemon protocol | `crates/ai-brains-daemon-api/src/lib.rs` | Supports Ping/Ingest/Sync/Shutdown only |
| Single writer | `crates/ai-brainsd/src/lib.rs` | Reusable queue/spool; query path is narrow and bypasses governed services |
| Capture | `crates/ai-brains-capture/src/command_handler.rs` | Captures turns but has no generic source observation service |
| Synthesis | `crates/ai-brains-brain/src/memory_synthesis.rs` | Produces generic memories; no Candidate Conclusion lifecycle |
| Retention | `crates/ai-brains-brain/src/retention.rs` | Deletes old turn projections; not cryptographic erasure |
| Crypto | `crates/ai-brains-crypto/src/data_key.rs`, `key_wrap.rs`, `sqlcipher.rs` | Vault-level keys exist; no per-content envelope keys or key tombstones |
| Harness adapters | `crates/ai-brains-adapters/src/*` | Reports capture mechanics, not governed read/write capabilities |
| Windows hardening | `crates/ai-brains-cli/src/artifact_security.rs`, `ai-brainsd/src/pipe_security.rs` | Reuse OS security patterns; do not confuse them with application authorization |
| Tauri UI | None | Must be introduced only after stable headless contracts |
| Loopback HTTP | None | Must be added after IPC/domain services are stable |
| Encrypted event sync | Existing Ledgerful bridge + `sync_state` only | Not a multi-device encrypted replication protocol |

---

## 3. Target crate and directory layout

Add only when the owning phase begins:

```text
crates/
  ai-brains-control-plane/   # governed application services and command/query handlers
  ai-brains-sources/         # source registry, fingerprints, built-in read connectors
  ai-brains-api-server/      # authenticated loopback HTTP adapter
  ai-brains-sync/            # encrypted event replication client/protocol
apps/
  desktop/                   # Tauri v2 desktop app; generated only after API contracts stabilize
fixtures/
  governed-memory/           # synthetic source trees, events, expected briefings
scripts/
  shadow-vault.ps1           # explicit read-only/copy workflow for Windows dogfood data
  evaluate-briefings.py      # deterministic fixture evaluator; no live mutation
```

Do not create a generic `utils` crate. Domain rules belong in `ai-brains-core`; event facts in `ai-brains-events`; persistence in `ai-brains-store`; orchestration in `ai-brains-control-plane`; retrieval algorithms in `ai-brains-retrieval`.

---

## 4. Phase dependency graph

```text
P0 Baseline + fixtures
 └─> P1 Domain/events/contracts
      ├─> P2 Sources/fingerprints/invalidation
      ├─> P3 Epistemic lifecycle/review
      └─> P4 Scope/principals/policy
           └──────────┬──────────┘
                      v
             P5 Briefing + progressive retrieval
                      |
             P6 Built-in connectors + model provenance
                      |
             P7 Daemon/CLI/HTTP protocol
                      |
             P8 Erasure + retention
                      |
             P9 Shadow migration + evaluation
                      |
             P10 Tauri desktop
                      |
             P11 Encrypted multi-device sync (post-MVP)
                      |
             P12 Release hardening
```

**MVP gate:** P0–P9.  
**Desktop beta gate:** P10.  
**Multi-device gate:** P11.  
Do not let P10 or P11 delay the headless cold-start briefing MVP.

---

# Phase 0 — Baseline, safety harness, and implementation tracks

## Goal

Prove the current repository and copied vault behavior before changing domain semantics. Establish fixtures and rollback evidence.

## Task 0.1 — Create the phase track

**Files:**

- Create `conductor/tracks/trackT147-governed-memory-baseline/spec.md`
- Create `conductor/tracks/trackT147-governed-memory-baseline/plan.md`
- Modify `conductor/conductor.md`

**Steps:**

1. Read `AGENTS.md`, `CONTEXT.md`, the vision, ADRs 0010–0015, and this plan.
2. Run `ledgerful ledger start -m "Establish governed-memory baseline and shadow fixtures"`.
3. Create branch `feature/governed-memory-baseline` if it does not exist.
4. Record current `git status`, Rust toolchain, migration count, vault schema version, and test commands in the track.
5. Do not include unrelated working-tree changes.

## Task 0.2 — Add deterministic fixture infrastructure

**RED tests first:**

- Create `crates/ai-brains-store/tests/governed_fixture_replay.rs`.
- Test that a synthetic event fixture can initialize a fresh vault, replay all existing events, and produce byte-for-byte stable selected projection rows.
- Test that running the fixture twice is idempotent.

**Implementation:**

- Create `fixtures/governed-memory/legacy-v1-events.ndjson` with synthetic, non-sensitive events only.
- Create `fixtures/governed-memory/expected-legacy-projections.json`.
- Create `crates/ai-brains-store/tests/common/governed_fixture.rs` for loading fixtures into `tempfile` databases.
- If integration-test helpers are duplicated elsewhere, extract only the minimal common loader; do not refactor unrelated tests.

**Verify:**

```bash
cargo test -p ai-brains-store --test governed_fixture_replay
```

Expected: RED before fixture loader exists; GREEN after loader and fixtures are implemented.

## Task 0.3 — Add safe shadow-vault tooling

**RED tests first:**

- Create `crates/ai-brains-cli/tests/shadow_vault_refuses_live_target.rs`.
- Test refusal when source and destination canonicalize to the same path.
- Test refusal when destination is inside the configured live vault directory.
- Test `--dry-run` emits planned copy paths without writing.

**Implementation:**

- Create `crates/ai-brains-cli/src/commands/shadow.rs`.
- Modify `crates/ai-brains-cli/src/commands/mod.rs` and `src/main.rs` to add:

```text
ai-brains shadow create --source <vault> --destination <path> [--redact-turn-content] [--dry-run]
```

- Create `scripts/shadow-vault.ps1` as a thin wrapper around the CLI, not a second copy implementation.
- Default to redacting raw turn content while preserving IDs, event kinds, timestamps, hashes, and relationship shape.
- Write `shadow-manifest.json` with source fingerprint, creation time, redaction policy, and tool version.

**Verify:**

```bash
cargo test -p ai-brains-cli --test shadow_vault_refuses_live_target
cargo test -p ai-brains-store --test replay_rebuilds_projections
```

## Task 0.4 — Record baseline gates

Run and record exact output in `conductor/tracks/trackT147-governed-memory-baseline/review.md`:

```bash
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo nextest run --workspace
cargo deny check
cargo audit
ledgerful verify --scope full
```

If `scripts/dev-check.ps1` still fails under Windows PowerShell 5.1, create a separate bug track. Do not silently weaken the gate and do not mix that fix into T147 unless the user approves.

**Phase acceptance:** synthetic replay stable; shadow command cannot target live vault; complete baseline evidence recorded.

---

# Phase 1 — Additive domain, event, and public-contract foundation

## Goal

Represent the new epistemic and scope concepts without changing legacy event meaning or serving behavior.

## Task 1.1 — Add typed IDs

**RED tests first:**

- Extend `crates/ai-brains-core/tests/id_serde_roundtrip.rs` for every new ID.

**Modify:** `crates/ai-brains-core/src/ids.rs`

Add:

```rust
define_id!(SourceId);
define_id!(SourceVersionId);
define_id!(EvidenceId);
define_id!(ConclusionId);
define_id!(DecisionId);
define_id!(WorkspaceId);
define_id!(PrincipalId);
define_id!(GrantId);
define_id!(ReviewItemId);
define_id!(BriefingId);
define_id!(QueryTraceId);
define_id!(ContentKeyId);
define_id!(TombstoneId);
define_id!(ReplicationEventId);
```

Do not reuse `MemoryId` for new Decisions. Keep legacy `DecisionRecordedPayload.decision_id: MemoryId` readable.

## Task 1.2 — Add domain modules

**Create:**

- `crates/ai-brains-core/src/source.rs`
- `crates/ai-brains-core/src/evidence.rs`
- `crates/ai-brains-core/src/conclusion.rs`
- `crates/ai-brains-core/src/decision.rs`
- `crates/ai-brains-core/src/scope.rs`
- `crates/ai-brains-core/src/principal.rs`
- `crates/ai-brains-core/src/briefing.rs`
- `crates/ai-brains-core/src/freshness.rs`
- `crates/ai-brains-core/src/model_provenance.rs`

**Modify:** `crates/ai-brains-core/src/lib.rs`

Required enums/structures:

```rust
pub enum SourceKind { GitRepository, File, ObsidianVault, Ledgerful, HermesSession, Honcho, Manual, Other(String) }
pub enum EvidenceStatus { Active, Superseded, Unavailable, Erased }
pub enum ConclusionState { Candidate, Active, Confirmed, Stale, Disputed, Superseded, Rejected }
pub enum DecisionState { Proposed, Approved, Superseded, Revoked }
pub enum ScopeRef { Repository(ProjectId), Workspace(WorkspaceId), Personal(UserId) }
pub enum GrantCapability { ReadEvidence, ReadConclusions, ReadDecisions, ProposeConclusion, ApproveConclusion, ProposeDecision, ApproveDecision, Export, Erase }
pub enum FreshnessState { Current, RevalidationDue, Stale, SourceUnavailable, Unknown }
```

Every externally injected item must carry a stable ID, scope, privacy, recorded time, source/dependency handles, and freshness state. Use `time::OffsetDateTime` consistently with events unless a repository convention requires otherwise.

**RED tests:** create:

- `crates/ai-brains-core/tests/conclusion_state_transitions.rs`
- `crates/ai-brains-core/tests/decision_requires_approval.rs`
- `crates/ai-brains-core/tests/scope_grant_strictest_wins.rs`
- `crates/ai-brains-core/tests/freshness_is_not_age_only.rs`

Tests must reject illegal transitions such as `Candidate → Confirmed` without an approval authority and `Stale → Active` without source revalidation.

## Task 1.3 — Add event facts, not service commands

**Modify:**

- `crates/ai-brains-events/src/event_kind.rs`
- `crates/ai-brains-events/src/payload.rs`
- `crates/ai-brains-events/src/aggregate.rs`
- `crates/ai-brains-events/src/lib.rs`
- `crates/ai-brains-events/src/constructors.rs` only where builders need validation

Add events grouped by aggregate:

```text
SourceRegistered
SourceObserved
SourceVersionRecorded
SourceUnavailable
EvidenceRecorded
EvidenceSuperseded
ConclusionProposed
ConclusionActivated
ConclusionConfirmed
ConclusionMarkedStale
ConclusionDisputed
ConclusionSuperseded
ConclusionRejected
DecisionProposed
DecisionApproved
DecisionSuperseded
DecisionRevoked
WorkspaceRegistered
RepositoryJoinedWorkspace
ScopeGrantIssued
ScopeGrantRevoked
PrincipalRegistered
ReviewItemOpened
ReviewItemResolved
BriefingGenerated
QueryTraceRecorded
ContentErasureRequested
ContentErased
```

Rules:

- Payloads reference IDs; do not embed mutable projection snapshots.
- `DecisionApproved` must include approver `PrincipalId`, approval time, and the proposal ID.
- `ConclusionMarkedStale` must identify the changed source version or unavailable-source reason.
- `EvidenceRecorded` must include source/version/fingerprint and model provenance when extraction used a model.
- New variants must deserialize safely alongside old events.
- Keep `CURRENT_SCHEMA_VERSION = 1` in this phase.

**RED tests:**

- Create `crates/ai-brains-events/tests/governed_payload_roundtrip.rs`.
- Create `crates/ai-brains-events/tests/legacy_v1_events_still_deserialize.rs` using Phase-0 fixture.
- Create `crates/ai-brains-events/tests/unknown_future_payload_degrades_safely.rs`.

## Task 1.4 — Add versioned public DTOs

**Create:**

- `crates/ai-brains-contracts/src/sources.rs`
- `crates/ai-brains-contracts/src/knowledge.rs`
- `crates/ai-brains-contracts/src/scopes.rs`
- `crates/ai-brains-contracts/src/briefings.rs`
- `crates/ai-brains-contracts/src/review.rs`
- `crates/ai-brains-contracts/src/policy.rs`

**Modify:** `crates/ai-brains-contracts/src/lib.rs`

Contract rules:

- IDs serialized as strings.
- Additive optional fields use `#[serde(default)]` where old clients may omit them.
- Every response includes `api_version` or uses the repository's established response envelope version.
- Briefing and query contracts expose evidence handles; do not return only prose.
- Policy denials are structured errors, not empty success responses.

**RED tests:**

- Extend `contracts_are_backward_compatible.rs`.
- Create golden JSON tests under `crates/ai-brains-contracts/tests/fixtures/`.

## Task 1.5 — Add the application-service crate skeleton

**Create:**

- `crates/ai-brains-control-plane/Cargo.toml`
- `crates/ai-brains-control-plane/src/lib.rs`
- `crates/ai-brains-control-plane/src/errors.rs`
- `crates/ai-brains-control-plane/src/ports.rs`

**Modify:** root `Cargo.toml` workspace members and dependencies.

Define ports only:

```rust
pub trait EventWriter { /* append governed events atomically */ }
pub trait GovernedQueryStore { /* typed projection reads */ }
pub trait Clock { /* reuse ai_brains_core::clock where possible */ }
pub trait Fingerprinter { /* deterministic source fingerprint */ }
pub trait PolicyEvaluator { /* principal + capability + scope */ }
```

Do not implement business workflows yet.

**Verify phase:**

```bash
cargo test -p ai-brains-core
cargo test -p ai-brains-events
cargo test -p ai-brains-contracts
cargo test -p ai-brains-control-plane
cargo test -p ai-brains-store --test governed_fixture_replay
```

**Phase acceptance:** new types/events/contracts compile and round-trip; legacy fixture remains readable; no serving behavior changes.

---

# Phase 2 — Source registry, evidence, fingerprints, and dependency invalidation

## Goal

Make freshness source-aligned and prove that source changes stale only dependent conclusions.

## Task 2.1 — Add source/evidence schema

**Create migrations:**

- `crates/ai-brains-store/migrations/0020_source_evidence.sql`
- `crates/ai-brains-store/migrations/0021_knowledge_dependencies.sql`

`0020` tables:

- `source_projection`
- `source_alias_projection`
- `source_version_projection`
- `evidence_projection`
- `evidence_fts` plus insert/update/delete triggers

Required unique constraints:

- source stable identity within scope
- `(source_id, fingerprint)` for versions
- evidence id

`0021` tables:

- `knowledge_dependency_projection` with parent type/id and evidence/source-version dependency
- `invalidation_queue_projection`

Indices must cover source, scope, status, recorded time, fingerprint, and dependency reverse lookup.

**Modify:** `crates/ai-brains-store/src/migrations.rs` to register migrations 20 and 21 in order.

**RED tests:**

- Create `crates/ai-brains-store/tests/source_evidence_migrations.rs`.
- Assert migration from a v19 fixture preserves old row counts and adds expected constraints.
- Assert rerunning initialization is idempotent.

## Task 2.2 — Add projections

**Create:**

- `crates/ai-brains-store/src/projections/source.rs`
- `crates/ai-brains-store/src/projections/evidence.rs`
- `crates/ai-brains-store/src/projections/dependency.rs`

**Modify:**

- `crates/ai-brains-store/src/projections/mod.rs`
- `crates/ai-brains-store/src/replay.rs`

Replay must truncate and rebuild new projections in foreign-key-safe order. Never truncate `events`.

**RED tests:**

- Create `crates/ai-brains-store/tests/source_evidence_projections.rs`.
- Extend `replay_rebuilds_projections.rs` to compare source/evidence/dependency rows before and after replay.

## Task 2.3 — Implement deterministic source fingerprints

**Create crate:**

- `crates/ai-brains-sources/Cargo.toml`
- `src/lib.rs`
- `src/fingerprint.rs`
- `src/normalization.rs`
- `tests/fingerprint_stability.rs`

Initial algorithms:

- File/Markdown: SHA-256 over normalized UTF-8 bytes plus canonical source identity; preserve original content separately.
- Git repository: commit OID plus relevant dirty-path digest; do not hash `.git` wholesale.
- Ledgerful: bridge record hash/lineage where supplied.
- External API: provider revision/ETag when authoritative; otherwise canonical payload hash.

Normalization must be versioned. A normalization algorithm change produces a new `normalizer_version`; it must not silently look like a source content change.

## Task 2.4 — Implement source observation transaction

**Create:**

- `crates/ai-brains-control-plane/src/sources.rs`
- `crates/ai-brains-control-plane/tests/source_observation.rs`

Workflow:

1. Resolve principal and scope.
2. Check `ReadEvidence`/capture policy.
3. Resolve/register source stable identity.
4. Compute fingerprint.
5. If fingerprint unchanged, update observation metadata without duplicating evidence.
6. If changed, append `SourceVersionRecorded` and `EvidenceRecorded` in one store transaction.
7. Queue reverse dependencies for invalidation.
8. Return IDs and explicit `changed` status.

**RED cases:** unchanged observation is idempotent; changed source creates one version; failed event append leaves no partial projections.

## Task 2.5 — Implement deterministic invalidation

**Create:**

- `crates/ai-brains-control-plane/src/invalidation.rs`
- `crates/ai-brains-control-plane/tests/dependency_invalidation.rs`

Rules:

- A changed source version marks dependent Conclusions `Stale`.
- Independent Conclusions remain unchanged.
- Decisions are never silently revoked; they receive a stale-support warning and review item.
- Source unavailable creates `SourceUnavailable`, freshness `SourceUnavailable`, and review according to criticality.
- Revalidation with unchanged authoritative content clears only the matching reason.

**Acceptance scenario:** fixture has two conclusions backed by different files; edit one file; only one becomes stale; briefing reports the stale one and cites changed source version.

## Task 2.6 — Capture structured action and verification evidence

**Modify:**

- `crates/ai-brains-capture/src/action_digest.rs`
- `crates/ai-brains-capture/src/verification_gate.rs`
- `crates/ai-brains-capture/src/command_handler.rs`

**Create tests:**

- `crates/ai-brains-capture/tests/action_evidence.rs`
- extend verification-gate unit tests for evidence conversion

Record bounded structured evidence for commands, changed-file summaries, test/gate names, exit codes, artifact handles, and verification outcomes. Reuse the existing Ledgerful `VerificationGate` response rather than adding a second verification path.

Critical distinction: `GateDecision::Proceed` after a Ledgerful IPC failure is a **fail-open capture decision**, not evidence that verification passed. Store the backend-unavailable state explicitly; never synthesize a successful verification result. Preserve current bounded-capture and no-chain-of-thought rules.

**RED cases:** successful gate produces attributable verification Evidence; blocked gate records the real risk/drift response; unavailable backend permits capture but marks verification `Unavailable`; no raw command output or secret-bearing environment is persisted by default.

## Task 2.7 — Extend the event-driven graph projection

**Modify:**

- `crates/ai-brains-graph/src/projector.rs`
- `crates/ai-brains-graph/src/rebuild.rs`
- `crates/ai-brains-graph/src/queries.rs`

**Create/extend tests:**

- governed graph projection tests under `crates/ai-brains-graph/tests/`
- replay/rebuild equivalence tests for governed events

Project governed events into explicit nodes and typed edges:

- Evidence → SourceVersion (`OBSERVED_FROM`)
- Conclusion → Evidence (`DERIVED_FROM`)
- Decision → Conclusion/Evidence (`SUPPORTED_BY`)
- Conflict → conflicting Conclusion/Decision (`CONFLICTS_WITH`)
- successor → predecessor (`SUPERSEDES`)
- Workspace → Repository (`CONTAINS`)

The event ledger remains authoritative; the graph is rebuildable and must not become an independent write path. Missing/degraded graph support must not change authority or freshness results.

**RED cases:** graph rebuild is idempotent; every dependency edge can be traced back to an event ID; stale/superseded state updates do not erase historical edges; graph unavailability falls back to projection queries with an explicit degraded trace.

**Phase acceptance:** source changes invalidate only real dependents; structured verification evidence distinguishes Passed, Failed/Blocked, and Unavailable; governed provenance/conflict graph rebuild is deterministic.

---

# Phase 3 — Evidence, Conclusion, Decision, conflict, and review lifecycle

## Goal

Stop flattening automatic extraction, derived claims, and approved commitments into generic memories.

## Task 3.1 — Add lifecycle/review schema

**Create migration:** `0022_epistemic_review.sql`

Tables:

- `conclusion_projection`
- `conclusion_evidence_projection`
- `decision_projection`
- `decision_support_projection`
- `review_item_projection`
- `claim_conflict_projection`

Do not delete legacy `conflict_projection`; add compatibility mapping only if needed.

## Task 3.2 — Add projections and query interfaces

**Create:**

- `store/src/projections/conclusion.rs`
- `store/src/projections/decision.rs`
- `store/src/projections/review.rs`
- `store/src/projections/claim_conflict.rs`

**Modify:** `store/src/projections/mod.rs`, `replay.rs`, `lib.rs`, `query_store.rs`.

Add typed query methods rather than returning tuples. Place row DTOs in `ai-brains-store/src/rows/` if query complexity warrants it.

## Task 3.3 — Implement proposal and approval commands

**Create:**

- `control-plane/src/conclusions.rs`
- `control-plane/src/decisions.rs`
- `control-plane/src/review.rs`
- corresponding integration tests

Required workflows:

- `propose_conclusion`
- `approve_conclusion`
- `reject_conclusion`
- `correct_conclusion`
- `propose_decision`
- `approve_decision`
- `supersede_decision`

Rules:

- Every proposal references Evidence IDs or is explicitly `unsupported` and cannot become authoritative.
- Agent proposals default to Candidate.
- Protected categories—security policy, privacy scope, deletion, spending, legal/compliance claims, deployment authorization—require human approval.
- Correction creates new evidence/conclusion and supersession links; do not mutate historical event payloads.
- Review resolution records principal and reason.

## Task 3.4 — Replace automatic synthesis authority

**Modify:**

- `crates/ai-brains-brain/src/memory_synthesis.rs`
- `crates/ai-brains-brain/src/conflict_detection.rs`
- `crates/ai-brains-brain/src/recipe_promotion.rs`

Automatic synthesis must emit `ConclusionProposed`, not `MemoryPinned`/authoritative `MemorySynthesized`, for new governed workflows. Preserve legacy mode behind an explicit compatibility path during migration.

Add model provenance: provider, model, local/cloud, prompt-template version, extraction-policy version, input evidence IDs, output hash, started/completed times. Never store hidden chain-of-thought.

## Task 3.5 — Conflict resolution tests

Create `crates/ai-brains-control-plane/tests/conflict_resolution.rs` with:

- same claim, different valid time: retain both and select time-valid result
- repository-specific rule vs workspace rule: repository scope wins only in repository context
- user-approved decision vs agent candidate: decision wins authority, candidate remains visible
- two active incompatible claims with equal authority: return unresolved conflict, not merged prose
- superseded decision: historical query can retrieve old state; current briefing uses successor

**Phase acceptance:** no agent-only path can create approved Decision authority; conflicts remain auditable.

---

# Phase 4 — Repository/workspace identity, principals, grants, and policy

## Goal

Resolve the correct context automatically without leaking unrelated project or personal data.

## Task 4.1 — Add scope/principal schema

**Create migration:** `0023_scopes_principals_grants.sql`

Tables:

- `workspace_projection`
- `workspace_repository_projection`
- `repository_identity_projection`
- `principal_projection`
- `scope_grant_projection`
- `policy_decision_log`

Repository identity fields:

- stable `ProjectId`
- canonical Git remote(s)
- Ledgerful project/transaction aliases
- Windows path aliases
- WSL path aliases
- worktree/clone aliases
- last verified time

Paths are aliases, never authority by themselves.

## Task 4.2 — Implement identity resolver

**Create:**

- `control-plane/src/scope_resolver.rs`
- `control-plane/tests/scope_resolution.rs`

Resolution confidence inputs:

1. explicit project ID
2. Git repository identity/remote
3. Ledgerful project identity
4. registered canonical path alias
5. current path heuristic

Return:

```rust
ResolvedScope { scope, confidence, evidence, warnings, alternatives }
```

Tests must cover Windows/WSL aliases, clones, worktrees, ambiguous nested repositories, and missing Git metadata. Ambiguity below threshold must warn and require override before Personal data is considered.

## Task 4.3 — Implement application authorization

**Create:**

- `control-plane/src/policy.rs`
- `control-plane/tests/policy_matrix.rs`

Do not reuse filesystem ACL terminology. Policy inputs are principal, capability, scope, content privacy, connector trust, and cloud/local route.

Default matrix:

- Human owner: all capabilities except destructive bulk erase still requires explicit confirmation at UI/CLI layer.
- Attributable agent: read granted project scope, propose conclusions, never approve protected decisions.
- Connector principal: capture only its declared source kind and scope.
- Background system principal: only explicit scheduled capabilities.
- Unknown principal: deny.

Log denials and high-impact grants; do not log sensitive content.

## Task 4.4 — Extend adapter capability declarations

**Modify:**

- `ai-brains-adapters/src/capability.rs`
- adapter-specific files
- capability tests

Separate transport support from governed capabilities:

```rust
pub struct AdapterCapability {
    /* existing fields */
    pub governed_reads: Vec<GrantCapability>,
    pub governed_writes: Vec<GrantCapability>,
    pub principal_binding: Option<PrincipalId>,
}
```

Use additive defaults in serialized representations.

**Phase acceptance:** policy tests prove no cross-project or personal-data injection without grant; every agent action is attributable.

---

# Phase 5 — Typed Project/Personal briefings and progressive retrieval

## Goal

Deliver fast, cited, budget-aware context first, then allow governed follow-up queries with full evidence traces.

## Task 5.1 — Define briefing packet contracts

**Modify/create:**

- `core/src/briefing.rs`
- `contracts/src/briefings.rs`
- golden contract fixtures

`ProjectBriefingPacket` sections:

- resolved repository/workspace and confidence
- active task/session handoff
- approved decisions
- confirmed/active conclusions
- constraints/invariants
- stale/disputed items
- source freshness summary
- Ledgerful impact/hotspots
- evidence handles
- unavailable-source warnings
- token/word budget and truncation report

`PersonalContinuityBriefingPacket` sections:

- confirmed preferences
- approved personal constraints
- recent continuity summary
- unresolved corrections/review items
- explicit grants applied

Personal packet must never be silently embedded in Project packet.

## Task 5.2 — Add briefing projection/cache schema

**Create migration:** `0024_briefings_query_traces.sql`

Tables:

- `briefing_cache_projection`
- `query_trace_projection`
- `retrieval_feedback_projection`

Cache key includes scope, briefing type, policy version, source version vector, and budget. Any dependency change invalidates matching cache entries.

## Task 5.3 — Build deterministic briefing service

**Create:**

- `control-plane/src/briefings/mod.rs`
- `project.rs`
- `personal.rs`
- `budget.rs`
- `renderer.rs`
- `tests/project_briefing.rs`
- `tests/personal_briefing.rs`

Do not begin with LLM summarization. First version selects typed projection rows deterministically and renders JSON plus human Markdown.

Selection order is authority/freshness/policy first, relevance second. A high vector score cannot overcome `Stale`, `Disputed`, unauthorized scope, or erased content.

Performance acceptance on synthetic fixture:

- warm deterministic Project Briefing p95 under 200 ms
- cold local Project Briefing p95 under 1.5 s excluding optional external connector refresh
- packet respects configured budget
- every authoritative claim has at least one evidence or decision handle

Use benchmark harness, not one timing assertion susceptible to CI noise.

## Task 5.4 — Replace preflight internals incrementally

**Modify:**

- `retrieval/src/preflight.rs`
- `contracts/src/preflight.rs`
- `cli/src/commands/preflight.rs`

Add compatibility adapter:

- new governed briefing service returns typed packet
- existing `preflight` text output renders that packet when feature flag/config `governed_briefing = true`
- legacy path remains available for one migration cycle

Do not infer counts by matching strings like `DECISION:` once typed packet is active.

## Task 5.5 — Add progressive query service

**Create/modify:**

- `retrieval/src/governed.rs`
- `control-plane/src/query.rs`
- `contracts/src/knowledge.rs`
- tests `governed_query_trace.rs`

Response must include:

- compact answer/result set
- evidence handles and source versions
- freshness and conflict status
- applied scope/policy
- ranking components
- trace ID
- `more_available`

Full trace fetched separately by ID. Default response must not dump raw source content.

**Phase acceptance:** north-star cold-start fixture returns current cited packet; follow-up query resolves handle to source preview; stale conclusion cannot appear as current truth.

---

# Phase 6 — Built-in connectors and model provenance

## Goal

Read from existing tools broadly while keeping durable write-back governed and preventing circular self-confirmation.

## Task 6.1 — Define connector port and capability manifest

**Create:**

- `ai-brains-sources/src/connector.rs`
- `ai-brains-sources/src/manifest.rs`
- `ai-brains-sources/tests/connector_contract.rs`

A connector declares source kinds, read/write operations, scopes, freshness mechanism, credential needs, and sandbox mode. Initial connectors are trusted built-ins but still pass policy checks.

## Task 6.2 — Implement local Markdown/Obsidian connector

**Create:** `ai-brains-sources/src/markdown.rs`, `obsidian.rs`.

Requirements:

- read-only by default
- canonical vault-relative identity
- frontmatter and content fingerprint
- source preview with line anchors where possible
- ignore rules and maximum-file limits
- symlink/reparse escape protection
- proposed write-back artifact instead of direct note mutation

Tests use a synthetic vault fixture with renamed file, changed content, duplicate title, and path traversal attempt.

## Task 6.3 — Implement Git and Ledgerful connectors

**Create:** `git.rs`, `ledgerful.rs`.

Reuse existing capture metadata and `BridgeRecord`; do not shell out from the briefing renderer. Refresh connectors before briefing through bounded source services, cache results, and report timeout/unavailability.

## Task 6.4 — Implement Hermes and Honcho read adapters

Create adapters behind feature flags/configuration. First release is read-only:

- Hermes: selected session/event summaries, attributable source IDs
- Honcho: confirmed profile/conclusion data with provider IDs and timestamps

Circularity guard: an item previously written by this control plane and read back from an external provider cannot become independent supporting evidence. Preserve origin event ID/source lineage.

## Task 6.5 — Record model provenance and cloud-policy gate

**Modify:**

- `ai-brains-models/src/lib.rs`
- provider implementations
- `control-plane/src/policy.rs`

Extend responses with provider, model, endpoint class, local/cloud, request template/version, timestamps, and usage where available. Never persist API keys or complete prompts containing disallowed content.

Tests:

- `Sealed`/local-only evidence cannot route to cloud provider
- local provider remains usable without network
- cloud denial returns structured explanation
- extracted Candidate Conclusion retains input Evidence IDs and model metadata

**Phase acceptance:** local Markdown + Git + Ledgerful produce source-versioned Evidence; external memory adapters cannot self-amplify circular claims.

---

# Phase 7 — Daemon, CLI, IPC, and authenticated loopback API

## Goal

Expose the same governed service operations to CLI, agents, desktop, and automation without letting any adapter become the domain model.

## Task 7.1 — Extend daemon request/response protocol

**Modify:** `ai-brains-daemon-api/src/lib.rs` or split into modules if file becomes unwieldy.

Add versioned request variants for:

- resolve scope
- get Project/Personal briefing
- query knowledge
- inspect evidence/source
- propose conclusion/decision
- list/resolve review items
- request erasure

Backward compatibility tests must prove old Ping/Ingest/Sync JSON still deserializes.

## Task 7.2 — Refactor daemon into writer and governed query services

**Modify/create:**

- `ai-brainsd/src/lib.rs`
- `ai-brainsd/src/router.rs`
- `ai-brainsd/src/services.rs`
- daemon tests

All mutations pass through the existing single-writer queue. Queries use a consistent read snapshot. Spool only idempotent mutation commands with command IDs; do not spool sensitive plaintext if content-envelope phase is active.

## Task 7.3 — Add CLI commands

**Create command modules:**

- `briefing.rs`
- `source.rs`
- `evidence.rs`
- `conclusion.rs`
- `decision.rs`
- `review.rs`
- `scope.rs`
- `policy.rs`

Examples:

```text
ai-brains briefing project --format json --budget 1500
ai-brains briefing personal --format json
ai-brains query "why was graph backend replaced?" --trace compact
ai-brains evidence show <id> --source-preview
ai-brains conclusion propose --evidence <id> --claim "..."
ai-brains review list
ai-brains review approve <id>
```

Human-readable output goes to stdout only in human mode; JSON/NDJSON remains machine-clean.

## Task 7.4 — Add authenticated loopback HTTP adapter

**Create crate:** `ai-brains-api-server`.

Use the current approved Rust HTTP stack after a short dependency/license/security check. Bind only `127.0.0.1` by default. Generate a random bearer token stored with restrictive user-only permissions; reuse Windows security helpers where applicable. Reject missing/incorrect token and non-loopback bind unless explicit secure configuration exists.

Endpoints under `/v1` mirror contracts; no business logic in handlers.

Security tests:

- no-auth denied
- wrong token denied
- valid token succeeds
- CORS deny by default
- request size limits
- source-preview path cannot escape registered root
- privacy/policy errors remain structured

**Phase acceptance:** CLI, named-pipe IPC, and HTTP return equivalent contract fixtures for the same briefing/query.

---

# Phase 8 — Content-envelope encryption, retention, and cryptographic erasure

## Goal

Make sensitive content unrecoverable on deletion while preserving minimal safe provenance and dependency cleanup.

## Task 8.0 — Mandatory cryptography spike and ADR

Before schema/code, create a short spike track answering:

- per-item vs per-scope content keys
- AEAD algorithm and nonce strategy
- key wrapping under existing `DataKey`/DPAPI/passphrase recovery model
- backup/recovery interaction
- FTS/embedding behavior for encrypted content
- crash consistency between event append, ciphertext write, and key deletion
- safe tombstone fields
- old plaintext event migration and impossibility boundaries

Use reviewed, established crypto primitives; do not implement cryptography manually. Record decision in a new ADR. If the design cannot prove erasure semantics for legacy plaintext events, state that plainly and define migration limitations.

## Task 8.1 — Add envelope schema after spike approval

Candidate migration name: `0025_content_envelopes_erasure.sql`.

Likely tables:

- `encrypted_content_blob`
- `content_key_projection`
- `erasure_request_projection`
- `tombstone_projection`

Do not finalize field names until ADR approval.

## Task 8.2 — Add crypto service

**Create:**

- `ai-brains-crypto/src/content_envelope.rs`
- `ai-brains-crypto/src/content_key_store.rs`
- known-answer and tamper tests

Requirements:

- authenticated encryption
- zeroized key material
- tamper detection
- key IDs separated from content IDs
- no key/ciphertext debug output

## Task 8.3 — Replace soft forget for governed content

Keep legacy `MemoryForgotten` behavior for legacy memories. Add governed erasure command:

1. policy/confirmation check
2. append `ContentErasureRequested`
3. remove wrapped content key atomically or through crash-safe state machine
4. delete derived plaintext projections/FTS/embeddings/cache
5. mark dependent conclusions/decisions unavailable/stale
6. append `ContentErased` with minimal safe tombstone
7. verify ciphertext is undecryptable

Tests must inspect database files/backups/FTS tables and ensure plaintext is absent. Do not claim secure physical media erasure; claim cryptographic erasure under documented key assumptions.

## Task 8.4 — Class-based retention

Extend `brain/src/retention.rs` to operate by content class and risk:

- raw turns: bounded default
- structured action evidence: longer
- approved decisions: retained until superseded/revoked policy
- secrets: never captured or shortest retention
- review traces: bounded and content-minimized

Dry-run and report required before destructive execution.

**Phase acceptance:** governed content erased from all readable projections and caches; dependent items surface broken support; legacy limitation documented.

---

# Phase 9 — Shadow migration, evaluation, and MVP dogfood

## Goal

Prove the successor on synthetic and redacted real data without mutating the live vault.

## Task 9.1 — Legacy memory classification importer

**Create:** `control-plane/src/legacy_import.rs` and tests.

Rules:

- legacy `MemoryPinned`: import as Evidence with source `LegacyAiBrains`, not as Confirmed Conclusion
- legacy `DecisionRecorded`: import as `DecisionState::Proposed` or `ApprovedLegacy` only if explicit historical approval evidence exists; otherwise review required
- `MemorySynthesized`: Candidate Conclusion
- session summaries: Evidence/digest with source session IDs
- forgotten memories remain excluded and are not resurrected

Importer must be idempotent and record original event IDs.

## Task 9.2 — Shadow replay and differential report

Create CLI:

```text
ai-brains migrate governed --source <shadow-vault> --destination <new-vault> --report <path> --dry-run
```

Report:

- event counts before/after
- entity classification counts
- unresolved mappings
- privacy changes
- source/dependency gaps
- content hashes (never plaintext in report)
- replay consistency
- rollback instructions

## Task 9.3 — Evaluation corpus

**Create:**

- `fixtures/governed-memory/scenarios/*.json`
- `scripts/evaluate-briefings.py`
- `Docs/EVALUATION/GOVERNED-MEMORY-MVP.md`

Required scenarios:

1. cold-start current cited project briefing
2. interrupted task resumption
3. source edit stales dependent conclusion
4. conflicting scoped claims
5. personal preference denied without grant
6. human correction supersedes agent inference
7. source unavailable
8. erased evidence removes derived content
9. Windows/WSL repository alias resolution
10. circular external-memory write-back rejection

Metrics:

- citation coverage
- stale-as-current rate (target 0)
- unauthorized-scope leakage (target 0)
- approved-decision attribution (target 100%)
- briefing budget compliance
- latency distributions
- reviewer correction rate

## Task 9.4 — Shadow dogfood gate

Run read-only against a redacted AI-Brains shadow copy. Compare legacy preflight and governed briefing. Human review must inspect at least 20 sampled claims and every stale/conflict/privacy warning before enabling governed mode on the live vault.

Rollback: disable governed feature flag and retain old projections/events. No live migration until explicit user approval.

**MVP acceptance:** all trust gates pass; north-star briefing is useful and cited; live vault remains unchanged.

---

# Phase 10 — Tauri desktop memory-operations client

## Goal

Provide a cohesive UI for governance and inspection without rebuilding Obsidian.

## Task 10.0 — Scaffold only after API contracts freeze

Create `apps/desktop` with Tauri v2 and TypeScript UI using the repository-approved frontend stack at implementation time. Add `apps/desktop/src-tauri` to the Rust workspace only after clean scaffold verification.

Do not copy business logic into TypeScript. The UI calls `/v1` or Tauri commands backed by the same control-plane service.

## Task 10.1 — Minimum screens

- Home: Project/Personal briefing selector and freshness warnings
- Search/query with compact trace
- Evidence/source preview and deep link
- Review inbox
- Conclusion/Decision detail with dependency graph
- Scope/grant inspector
- Retention/erasure center
- Connector health/policy status

## Task 10.2 — Security/UX requirements

- CSP locked down; no arbitrary remote content execution
- external links opened through safe shell API
- source preview rendered as inert text/Markdown
- explicit scope indicator always visible
- destructive actions require typed/clear confirmation and show dependency impact
- stale/disputed badges not color-only
- keyboard-accessible review flow
- no analytics by default

## Task 10.3 — Tests

- Rust command contract tests
- frontend unit tests for state and rendering
- Playwright/Tauri integration tests for review and erasure flows
- screenshot/visual checks for stale, conflict, denied, empty, and offline states

**Desktop beta acceptance:** all primary operations work offline; UI never grants authority unavailable through service contracts.

---

# Phase 11 — Encrypted event replication (post-MVP)

## Goal

Synchronize end-to-end encrypted event envelopes through an untrusted relay; never sync mutable SQLite files.

## Task 11.0 — Protocol threat model and ADR

Define device identity, enrollment, revocation, event ordering, duplicate handling, divergent lineage, deletion propagation, replay attacks, relay metadata leakage, and disaster recovery. Obtain security review before implementation.

## Task 11.1 — Add sync crate and schema

**Create:** `crates/ai-brains-sync`.

Candidate migration: `0026_replication_state.sql` with device cursors, encrypted envelope IDs, acknowledgement state, and revocation records—no plaintext content.

Protocol properties:

- client-side encryption/signing
- idempotent event IDs
- per-device cursor
- local projection rebuild
- explicit conflicts, never last-write-wins
- tombstone/erasure propagation
- revoked device cannot decrypt future events

## Task 11.2 — Implement fake relay first

Use an in-memory/file relay in tests. Prove two clients converge after offline divergence, duplicates, reordering, and retry. Only then define/deploy a network relay.

## Task 11.3 — Security tests

- relay cannot decrypt payload
- tampered envelope rejected
- replayed envelope idempotent
- revoked device excluded from new keys
- erasure reaches all enrolled devices with explicit acknowledgement status
- metadata leakage documented

**Acceptance:** convergence and privacy threat-model gates pass; sync remains optional and local-only mode unchanged.

---

# Phase 12 — Release hardening and adoption

## Goal

Turn the dogfood implementation into a distributable developer product without overstating security or replacing source tools.

Tasks:

1. Upgrade compatibility matrix for Windows 11 + WSL, Linux, and macOS.
2. Backward/forward protocol tests across at least one prior released CLI/daemon version.
3. Backup, restore, and recovery-kit drills with encrypted governed content.
4. Connector sandbox decision: trusted built-ins initially; capability-scoped WASI/subprocess plugins only after threat-model review.
5. Documentation:
   - installation and local-only mode
   - source/provenance model
   - agent permissions
   - correction and review
   - retention/erasure limits
   - optional cloud processing
   - sync threat model
6. Independent security review of HTTP auth, connector path handling, model routing, content keys, and sync.
7. Publish only claims backed by evaluation artifacts. Never claim compliance certifications or perfect deletion.

---

## 5. Cross-phase testing discipline

For every behavior change:

1. Write one focused failing test.
2. Run it and capture the expected failure.
3. Implement the minimum behavior.
4. Run the focused test to green.
5. Run the owning crate tests.
6. Refactor only while green.
7. Run affected integration tests.
8. At phase end run the full repository gate.

Required full gate:

```bash
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo nextest run --workspace
cargo deny check
cargo audit
ledgerful verify --scope full
```

Do not accept a wrapper script's exit code without reading its output. A script that failed to locate or parse itself ran zero tests.

---

## 6. Feature flags and rollout

Add config flags with conservative defaults:

```toml
[governed_memory]
enabled = false
shadow_only = true
legacy_preflight_fallback = true
allow_personal_grants = false
allow_cloud_extraction = false
http_api_enabled = false
sync_enabled = false
```

Rollout stages:

1. compiled but disabled
2. synthetic fixtures
3. shadow read-only
4. local live read-only briefing
5. governed proposals; no approvals through agents
6. human review/approval
7. optional desktop
8. optional sync

Each stage has a one-command rollback to the prior mode and leaves existing event history readable.

---

## 7. Commit and track boundaries

Suggested implementation tracks/branches:

| Track | Branch | Scope |
|---|---|---|
| T147 | `feature/governed-memory-baseline` | P0 |
| T148 | `feature/governed-memory-domain` | P1 |
| T149 | `feature/source-freshness-invalidation` | P2 |
| T150 | `feature/epistemic-review-lifecycle` | P3 |
| T151 | `feature/scopes-principals-policy` | P4 |
| T152 | `feature/typed-progressive-briefings` | P5 |
| T153 | `feature/governed-source-connectors` | P6 |
| T154 | `feature/governed-api-surface` | P7 |
| T155 | `security/content-envelope-erasure` | P8 |
| T156 | `feature/governed-shadow-migration` | P9 |
| T157 | `feature/memory-operations-desktop` | P10 |
| T158 | `security/encrypted-event-sync` | P11 |
| T159 | `release/governed-memory-hardening` | P12 |

Track numbers are proposals. Before creating each, inspect `conductor/conductor.md` and use the next free ID.

Within a track, make small commits after coherent green units. Do not merge or push `main` without explicit approval. Record exact tests and artifact paths in the track's `review.md`; a filename or checked box is not completion evidence.

---

## 8. Mandatory stop conditions

Stop and ask Ryan before:

- mutating or migrating the live vault
- enabling cloud inference for local/private content
- approving a Decision through an agent principal
- changing the event envelope schema version
- choosing/implementing content-envelope cryptography without the ADR review
- registering a Windows service/task or changing machine ACLs
- deploying a sync relay
- committing an unrelated working-tree file
- merging or pushing `main`

If blocked by permissions, unavailable services, or Windows-only manual actions, create a timestamped Obsidian note in `C:\Users\RyanB\Documents\Hermes\Today\` describing the attempt, failure, and exact recovery steps.

---

## 9. Definition of MVP done

MVP is done only when all are evidenced:

- [ ] Legacy v1 event fixture remains readable and replay-stable.
- [ ] Source observations are idempotent and fingerprinted.
- [ ] Source changes stale only dependent Conclusions.
- [ ] Evidence, Conclusion, and Decision authority are distinct.
- [ ] No agent-only path approves protected Decisions.
- [ ] Repository/Workspace/Personal scope tests show zero unauthorized leakage.
- [ ] Project Briefing is typed, cited, budget-aware, and freshness-aware.
- [ ] Personal Continuity Briefing is distinct and grant-gated.
- [ ] Progressive queries expose compact evidence handles and on-demand traces.
- [ ] Local Markdown, Git, and Ledgerful connectors work without cloud services.
- [ ] Model-assisted extraction records provenance and respects local/cloud policy.
- [ ] CLI and daemon expose the same versioned contracts.
- [ ] Authenticated loopback API passes security tests or remains disabled outside test mode.
- [ ] Legacy import is idempotent and never upgrades inferred content to approved authority.
- [ ] Synthetic evaluation passes trust gates.
- [ ] Redacted shadow-vault evaluation is reviewed by a human.
- [ ] Live vault was not mutated during development/evaluation.
- [ ] Full Rust/security/Ledgerful gate passes with exact output recorded.
- [ ] Documentation states limitations honestly.

---

## 10. First task for the implementing harness

Do not start coding immediately. Execute exactly this orientation:

```bash
cd /mnt/c/dev/ai-brains
git status --short --branch
git log -5 --oneline
ledgerful doctor
ledgerful scan --impact
```

Then read:

1. `AGENTS.md`
2. `CONTEXT.md`
3. `Docs/MEMORY-CONTROL-PLANE-VISION.md`
4. ADRs 0010–0015
5. `Docs/RESEARCH/memory-systems-comparison-2026-07.md`
6. this plan
7. `crates/ai-brains-events/src/{envelope,event_kind,payload,upcast,version}.rs`
8. `crates/ai-brains-store/src/{migrations,replay,event_store}.rs`
9. `crates/ai-brains-retrieval/src/{preflight,recall}.rs`
10. `crates/ai-brainsd/src/lib.rs`

Report:

- current branch and unrelated changes
- next free track ID
- whether baseline gates are green
- whether the live vault can remain untouched
- any mismatch between this plan and current code

Only after that report should the harness create T147/start the Ledgerful transaction and begin Task 0.2 with a failing test.
