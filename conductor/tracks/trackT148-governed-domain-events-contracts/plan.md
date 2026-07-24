# T148 Plan — Governed Domain, Events, Contracts (P1)

## Preconditions

- [x] Confirm T147 Complete in `conductor.md` and edition 2024 / fixture paths exist.
- [x] `git status --short --branch` — exclude `.agents/skills/codex-review/SKILL.md`.
- [x] `ledgerful doctor` ; `ledgerful ledger status --compact`.
- [x] `ledgerful ledger start` — message: `T148: additive governed domain/events/contracts (P1)` — category `ARCHITECTURE` or `FEATURE`.
- [x] Branch: `feature/governed-domain-events-contracts`.
- [x] Register T148 **In Progress** in `conductor/conductor.md`.
- [x] Re-read: `CONTEXT.md`, ADR-0010, ADR-0011, master plan Phase 1, this spec.
- [x] `ledgerful scan --impact` before first code edit.

### Research notes (do not re-litigate mid-track)

- W3C PROV-DM: provenance as Entity/Activity/Agent — map to Evidence/events/Principal; no PROV crate.
- Ports & adapters: traits in control-plane; adapters in later phases.
- Serde: new **known** variants before catch-all; **unit** `#[serde(other)]` is insufficient for write-back fidelity (R0).
- Keep workspace deps; no hexser / PROV-O / graph framework.
- Contracts timestamps: `chrono::DateTime<Utc>` only (existing convention).
- `DecisionApproved` locked: `decision_id` + `proposal_event_id` + `approver` + `approved_at`.

---

## Phase A0 — Unknown payload fidelity (R0) — **before new governed variants**

> Land this before Phase C’s ~25 new `Payload` variants. T147 shadow + `append_event` re-serialize typed payloads; unit `Unknown` destroys data.

### A0.1 RED

- [x] `unknown_future_payload_degrades_safely.rs` (or rename):
  - Deserialize `{"type":"TotallyFutureEvent","foo":1,"bar":"x"}` → `Payload::Unknown(...)`.
  - Assert **field recovery** (`foo`, `bar`), not merely “no panic”.
  - Round-trip: `to_value` / `from_value` / `to_value` → `Value` equality with input.
- [x] `unknown_payload_roundtrip_preserves_fields.rs`:
  - Full `Envelope` with unknown payload + known `event_type` tag handling.
  - If `EventKind` still collapses tags, RED that `Unknown` preserves original kind string.
- [x] Optional store path: append envelope with unknown payload to tempfile, `read_all_events`, assert payload `Value` equality.

### A0.2 GREEN

- [x] Replace unit `Payload::Unknown` with content-preserving form (**option A preferred**: `Unknown(serde_json::Value)` + custom ser/de that emits original object).
- [x] Preserve `EventKind` original tag (`Unknown(String)` or equivalent custom ser/de).
- [x] Ensure `Serialize` of unknown does **not** emit empty `{"type":"Unknown"}`.
- [x] Projection wildcards still ignore unknown (compile + no behavior change on legacy).
- [x] Shadow: `redact_turn_content` leaves `Unknown` untouched; no hash recompute for non-redacted pass-through.
- [x] `cargo test -p ai-brains-events`
- [x] `cargo test -p ai-brains-store --test governed_fixture_replay`
- [x] Manual or automated: shadow dry-run not required; unit/store tests sufficient if they cover re-serialize path used by append.

**Stop if** custom ser/de forces envelope schema v2 or migration — escalate per spec §9.

---

## Phase A — Typed IDs (Task 1.1)

### A1 RED

- [x] Extend `crates/ai-brains-core/tests/id_serde_roundtrip.rs` for every new ID (serde_json round-trip equality). Prefer one parameterized style or discrete tests; **no** for-loop over IDs in a single `#[test]` that hides failures (use `rstest` only if already a workspace dep — otherwise discrete tests or macro).

### A2 GREEN

- [x] `define_id!` all IDs in `ids.rs` (list in spec R1).
- [x] Do **not** remove or repurpose `KnowledgeId`.
- [x] `cargo test -p ai-brains-core --test id_serde_roundtrip`

---

## Phase B — Domain modules + transitions (Task 1.2)

### B1 RED

Create failing tests:

| File | Assert |
|------|--------|
| `conclusion_state_transitions.rs` | Legal path `Candidate → Active`; illegal `Candidate → Confirmed` without approval; illegal `Stale → Active` without revalidation |
| `decision_requires_approval.rs` | `Proposed → Approved` requires `PrincipalId` approver; reject silent auto-approve. Event-layer fixture (when written) must use locked shape: `decision_id` + `proposal_event_id` + `approver` + `approved_at` |
| `scope_grant_strictest_wins.rs` | When two grants conflict, stricter capability/privacy wins (define matrix in test) |
| `freshness_is_not_age_only.rs` | Old-but-source-aligned ⇒ `Current` or non-Stale; young-but-source-changed ⇒ `Stale` / `RevalidationDue` |

Follow `SessionStatus::transition` style.

### B2 GREEN

- [x] Create domain modules listed in spec; wire `lib.rs`.
- [x] Extend `errors.rs` as needed (`InvalidStatusTransition` reuse OK if messages are specific).
- [x] Pure functions only — no IO.
- [x] `cargo test -p ai-brains-core`
- [x] `cargo clippy -p ai-brains-core --all-targets -- -D warnings`

---

## Phase C — Event facts (Task 1.3)

**Prerequisite:** Phase A0 (R0) green. Do not add bulk governed variants onto unit-`Unknown`.

### C1 RED

Create/extend `crates/ai-brains-events/tests/`:

1. **`governed_payload_roundtrip.rs`**  
   For each new payload: build envelope (fixed times/ids), `serde_json` round-trip, assert equality (or hash-stable fields).  
   Include **`DecisionApproved`** with locked fields: `decision_id`, `proposal_event_id`, `approver`, `approved_at`.

2. **`legacy_v1_events_still_deserialize.rs`**  
   Read `fixtures/governed-memory/legacy-v1-events.ndjson` via path from workspace root:
   ```rust
   // CARGO_MANIFEST_DIR/../../fixtures/governed-memory/legacy-v1-events.ndjson
   ```
   Deserialize each line as `Envelope`; assert non-empty; assert no panic; count matches fixture lines.

3. **R0 tests** (from Phase A0) remain green — do not weaken to “no panic only.”

### C2 GREEN

- [x] Add `EventKind` variants **before** `Unknown`.
- [x] Add `Payload` variants + payload structs; IDs only; optional model provenance on `EvidenceRecorded`.
- [x] Extend `AggregateType` as needed; **document** Decision aggregate reuse.
  - **Decision aggregate choice (locked):** Keep existing `AggregateType::Decision` for both
    legacy `DecisionRecorded` (`decision_id: MemoryId`) and new governed decision facts
    (`DecisionProposed` / `DecisionApproved` / … with `decision_id: DecisionId`). Do not rename.
    Identity dual-model is at the ID newtype layer, not a second aggregate enum.
- [x] Update `lib.rs` re-exports.
- [x] Constructors: only add builders if validation is required (approval fields non-empty); avoid drive-by refactors.
- [x] Keep `CURRENT_SCHEMA_VERSION = 1`.
- [x] Verify projections still compile (wildcard arms already ignore new payloads).
- [x] `cargo test -p ai-brains-events`
- [x] `cargo test -p ai-brains-store --test governed_fixture_replay`

---

## Phase D — Public contracts (Task 1.4)

### D1 RED

- [x] Golden JSON fixtures under `crates/ai-brains-contracts/tests/fixtures/` for briefing shell, knowledge item, policy denial.
- [x] Extend `contracts_are_backward_compatible.rs`:
  - Old ingest/preflight still parse (existing).
  - New DTOs: minimal JSON without optional fields parses via defaults.
  - Policy denial structure has `code` + `message` (not empty success).

### D2 GREEN

- [x] Implement modules: `sources`, `knowledge`, `scopes`, `briefings`, `review`, `policy`.
- [x] Export from `lib.rs`.
- [x] Include `api_version` on new response envelopes (constant `"1"` unless documented otherwise).
- [x] All new instant fields: **`chrono::DateTime<Utc>`** (not `String`, not raw `time::OffsetDateTime` in contracts).
- [x] One shared `OffsetDateTime` → `DateTime<Utc>` helper for domain→contract mapping.
- [x] Evidence handles on briefing/knowledge types (`evidence_id: String` / list).
- [x] `cargo test -p ai-brains-contracts`
- [x] `cargo clippy -p ai-brains-contracts --all-targets -- -D warnings`

---

## Phase E — Control-plane skeleton (Task 1.5)

### E1 RED

- [x] Workspace fails to resolve `ai-brains-control-plane` until crate exists — add crate then:
  - Unit test or doctest that ports are object-safe **or** simply compile-only crate with `#[cfg(test)]` mock implementing each trait returning `Ok`/`Err` stubs.
  - Prefer: `tests/ports_are_implementable.rs` with dummy structs implementing all traits (no panic).

### E2 GREEN

- [x] `Cargo.toml` member; deps: `ai-brains-core`, `ai-brains-events`, `thiserror`, `time` as needed; edition workspace.
- [x] `errors.rs`, `ports.rs`, `lib.rs`.
- [x] **No** workflow modules.
- [x] `cargo test -p ai-brains-control-plane`
- [x] `cargo clippy -p ai-brains-control-plane --all-targets -- -D warnings`

---

## Phase F — Verification + review

### F1 Targeted

```powershell
cargo test -p ai-brains-core
cargo test -p ai-brains-events
cargo test -p ai-brains-contracts
cargo test -p ai-brains-control-plane
cargo test -p ai-brains-store --test governed_fixture_replay
```

### F2 Full gate

```powershell
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo nextest run --workspace
cargo deny check
cargo audit
ledgerful verify --scope full
```

### F3 Review

- [x] Internal review vs this `spec.md` (completeness, no placeholders, no serving changes).
- [x] Cross-model read-only review (`codex-review` skill) — ARCHITECTURE/FEATURE.
- [x] Fix P0–P2; defer only per AGENTS caps; append residuals to `deferred.md` / ISSUES.
- [x] Write `review.md` with commands, exit codes, dispositions.

### F4 Finalize

- [x] Mark plan checkboxes; conductor → **Complete** only after DoD.
- [x] `ledgerful ledger commit`.
- [x] Stage only T148 files; **exclude** codex-review skill.
- [x] Stop before merge/push without Ryan approval.

---

## Acceptance evidence checklist

| Item | Command / artifact |
|------|---------------------|
| Unknown payload **preserves fields** on round-trip | R0 tests (Value equality, not panic-only) |
| EventKind tag preserved when unknown | R0 EventKind test |
| ID round-trip | `id_serde_roundtrip` |
| Illegal transitions rejected | conclusion/decision/freshness tests |
| DecisionApproved shape | round-trip includes `proposal_event_id` + `approver` |
| Legacy fixture deserializes | `legacy_v1_events_still_deserialize` |
| Golden projections still match | `governed_fixture_replay` |
| Contracts timestamps are chrono | code review + DTO compile |
| Contracts additive | golden + backward_compat tests |
| Ports compile | control-plane tests |
| Full gate | F2 table in `review.md` |

## Explicit exclusions

- No P2 migrations/fingerprints/invalidation services.
- No CLI/daemon changes.
- No live vault.
- No envelope v2.
- No new third-party architecture frameworks.

## Stop conditions

See `spec.md` §9. Halt and ask if exhaustiveness forces large projection rewrites or if Decision ID dual-model is ambiguous in code review.
```