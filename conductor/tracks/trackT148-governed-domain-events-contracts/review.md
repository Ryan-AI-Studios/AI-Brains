# T148 Review Log — Governed Domain, Events, Contracts

**Track:** T148-GovernedDomainEventsContracts  
**Branch:** `feature/governed-domain-events-contracts`  
**Category:** ARCHITECTURE / FEATURE  
**Ledger tx:** `7802cfff-6e66-4fad-a089-4b0287be94da`  
**Date:** 2026-07-24

## Scope

Additive P1 foundation only:

- R0 preserve-unknown payload + EventKind tag fidelity (option A)
- R1 governed typed IDs (`DecisionId` ≠ `MemoryId`; `KnowledgeId` kept)
- R2 domain modules + pure transition rules
- R3 event facts at `CURRENT_SCHEMA_VERSION = 1` (no migrations)
- R4 public contracts (`chrono::DateTime<Utc>`, `api_version: "1"`)
- R5 `ai-brains-control-plane` ports-only
- Shadow: Unknown pass-through without re-hash when not redacting turns

**Not in scope:** SQL 0020+, workflows, serving, envelope v2.

## Design locks

| Item | Choice |
|------|--------|
| R0 | `Payload::Unknown(serde_json::Value)` + custom ser/de; `EventKind::Unknown(String)` |
| Decision aggregate | Reuse `AggregateType::Decision` for legacy + governed decision facts |
| `DecisionApproved` | `decision_id` + `proposal_event_id` + `approver` + `approved_at` |
| Contracts timestamps | `chrono::DateTime<Utc>` via `offset_to_utc` |

## Reviewers / rounds

| Round | Reviewer | Verdict | Notes |
|-------|----------|---------|-------|
| 1 | Internal (subagent) | PASS w/ residuals M1–M4 | No P0/P1 code defects |
| 1b | Orchestrator fix | M1 expanded payload roundtrips; M4 shadow Unknown unit test | |
| 2 | Codex (`gpt-5.6-luna` high) | FAIL → fixed | P1 process (review/plan); P2 typed query ports; P3 plan aggregate note |
| 2b | Orchestrator fix | P2 `GovernedQueryStore` uses `ConclusionId`/`DecisionId`; P3 plan note; this `review.md` | |

Raw: `review.codex.md` (Codex round 1 output).

## Findings disposition

| ID | Sev | Description | Status |
|----|-----|-------------|--------|
| INT-M1 | P2 | Subset payload roundtrips | **verified_fixed** — all governed variants covered in `remaining_governed_payloads__roundtrip_subset` + key cases |
| INT-M2 | P2 | Known type registry triplication (`KnownPayload` / `is_known_payload_type` / `EventKind`) | **deferred** — see deferred.md #13 |
| INT-M3 | P2 | `ConclusionMarkedStale` allows both optionals None | **deferred** — constructor validation in later workflow track; see deferred.md #14 |
| INT-M4 | P2 | No shadow Unknown unit test | **verified_fixed** — `redact_turn_content__unknown_payload__preserves_hash_and_fields` |
| COD-P1 | P1 | Missing review/plan/conductor closure | **verified_fixed** — this log + plan/conductor update |
| COD-P2 | P2 | `GovernedQueryStore` used `&str` | **verified_fixed** — typed `ConclusionId` / `DecisionId` |
| COD-P3 | P3 | Decision aggregate choice not in plan | **verified_fixed** — plan Phase C2 note |

## Gate evidence (orchestrator-observed)

| Command | Result |
|---------|--------|
| `cargo fmt --check` | PASS |
| `cargo clippy --workspace --all-targets -- -D warnings` | PASS |
| `cargo nextest run --workspace` | **477/477** PASS (pre–shadow unit test; +1 unit test after) |
| `cargo deny check` | PASS |
| `cargo audit` | PASS (pre-existing allowlisted advisories only) |
| `ledgerful verify --scope full` | PASS (all 5 steps) |
| `cargo test -p ai-brains-store --test governed_fixture_replay` | PASS |
| `cargo test -p ai-brains-core` / events / contracts / control-plane | PASS |

Post-fix targeted: control-plane tests 2/2 PASS; clippy control-plane PASS.

## Manual evidence

- R0: `TotallyFutureEvent` Value equality + field recovery in events tests.
- EventKind unknown tag serializes as original string, not `"Unknown"`.
- Legacy NDJSON deserializes fully as known payloads.
- Shadow unit: Unknown + original `payload_hash` preserved under `redact_turn_content`.
- Dual model: `DecisionRecordedPayload.decision_id: MemoryId`; governed `DecisionId` on new facts; ports take typed IDs.

## Residual / deferred

- **#13** Known event-type registry single source of truth (INT-M2).
- **#14** `ConclusionMarkedStale` require version and/or reason (INT-M3).
- Unrelated dirty: `.agents/skills/codex-review/SKILL.md` — **never staged**.

## Completion decision

Engineering DoD for P1 **met** after Codex P2 fix and governance artifacts.  
**Stop before merge/push without Ryan approval.**
