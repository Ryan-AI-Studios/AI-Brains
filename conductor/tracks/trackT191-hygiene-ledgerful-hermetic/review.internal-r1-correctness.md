# T191 Internal R1 — Correctness Review (Second Pass)

- **Track:** T191-HygieneLedgerfulHermetic
- **Branch:** `agent/T191-hygiene-ledgerful-hermetic`
- **Reviewer role:** Read-only second-pass correctness (not rename completeness alone)
- **Date:** 2026-08-02
- **Scope files (primary):**
  - `crates/ai-brains-cli/src/commands/symbol_bridge.rs`
  - `crates/ai-brains-capture/src/verification_gate.rs`, `lib.rs`, `command_handler.rs`
  - `crates/ai-brains-cli/tests/common/mod.rs`
  - Long-tail hermetic: `governed_surface.rs`, `cross_repo_bridge_smoke.rs`, `nightly_madr_ingestion.rs`, `dogfood_compare.rs`, `evaluate_governed.rs`
  - Call-site renames: `safety.rs`, `nightly.rs`, `preflight.rs`, `recall.rs`, `intervention.rs`, `cozo_proxy.rs`
  - Fixture/docs: `bridge_record_shape.rs`, `briefings.rs`
- **Method:** Source read + workspace greps for dual-read, leftovers, hermetic/zero-key, production `unwrap`/`expect`, capture export, cross-repo skip. No production edits.

---

## Verdict: **PASS**

No P0/P1 correctness defects found in the dual-read tag migration, hermetic L13 wiring, capture gate rename, or identifier renames. Residual notes are P3 (coverage/cosmetic/pre-existing soft edges).

---

## Focus area results

### 1. `symbol_already_ingested` dual-read — false-negative / double-ingest

**Verdict: correct (no double-ingest on tag flip; no false-negative for known tags).**

Constants (F2):

```12:15:crates/ai-brains-cli/src/commands/symbol_bridge.rs
/// Legacy source_tag written by pre-T191 symbol ingest (durable in vault events).
pub const SOURCE_TAG_SYMBOL_LEGACY: &str = "changeguard:symbol";
/// Canonical source_tag for new symbol ingest writes (T191 F2).
pub const SOURCE_TAG_SYMBOL: &str = "ledgerful:symbol";
```

Dedup path:

```240:260:crates/ai-brains-cli/src/commands/symbol_bridge.rs
fn symbol_already_ingested(event_store: &dyn EventStore, memory_uuid: Uuid) -> bool {
    event_store
        .read_events(memory_uuid)
        .map(|events| {
            events.iter().any(|event| match &event.payload {
                Payload::MemoryPinned(payload) => {
                    is_symbol_source_tag(payload.source_tag.as_deref())
                }
                _ => false,
            })
        })
        .unwrap_or(false)
}

/// Dual-read: either legacy or canonical symbol source_tag counts as ingested (F2).
fn is_symbol_source_tag(tag: Option<&str>) -> bool {
    matches!(
        tag,
        Some(SOURCE_TAG_SYMBOL_LEGACY) | Some(SOURCE_TAG_SYMBOL)
    )
}
```

Write path uses **only** `SOURCE_TAG_SYMBOL` (`ledgerful:symbol`).

| Scenario | Result |
|----------|--------|
| Existing `changeguard:symbol` | OR dual-read → skip → **no double-ingest** |
| Existing `ledgerful:symbol` | skip → **no double-ingest** |
| No prior MemoryPinned with either tag | ingest once; writes new tag |
| Second ingest same UUID | skip (idempotent) |

**Identity key** remains `Uuid::new_v5(NAMESPACE_URL, "{project_id}:{qualified_name}")` — tag dual-read is orthogonal and does not broaden UUID collisions.

**Proof tests present:**

- `symbol_dedup__legacy_tag_only__no_double_ingest`
- `symbol_dedup__new_tag_only__no_double_ingest`
- `symbol_ingest__writes_ledgerful_symbol_tag`
- `symbol_ingestion_is_idempotent_and_recallable` (new-write round-trip)

**Soft edge (pre-existing, not a T191 regression):** `read_events` `Err` → `unwrap_or(false)` fails open toward re-ingest. Same fail-open existed under the old single-tag equality check. Catalogued as P3 below.

**T167:** Legacy import still preserves `changeguard:symbol` verbatim (`legacy_import.rs` + “no changeguard→ledgerful rewrite” comment). Dual-read does not rewrite durable tags. AC4 satisfied at code level.

### 2. `is_symbol_source_tag` / constants wiring

**Verdict: correct.**

- Write: `Some(SOURCE_TAG_SYMBOL.to_string())` only.
- Dedup: exact match on both const values via `matches!`.
- No residual production write of `"changeguard:symbol"` outside the legacy const + T167 preserve fixtures.
- Grep of production write sites: only `symbol_bridge` symbol ingest sets the symbol tags.

### 3. Hermetic migration correctness (env / flags / zero-key)

**Verdict: correct.**

| Check | Result |
|-------|--------|
| AC6 bare `Command::cargo_bin` in five L13 files | **0** matches |
| Helper `hermetic_bin` | strips denylist + sets `AI_BRAINS_ALLOW_ZERO_KEY=1` |
| Denylist F9/AC10 | includes `LEDGERFUL_TX_ID` + `CHANGEGUARD_TX_ID` |
| `governed_surface` / `nightly_madr` / `dogfood_compare` / `evaluate_governed` | all use `common::hermetic_*` (or factory → `hermetic_bin`) |
| In-process vault open after hermetic init (`cross_repo_bridge_smoke`) | each test sets `TempEnv::set("AI_BRAINS_ALLOW_ZERO_KEY", "1")` before `VaultConnection::open` with zero key |
| `nightly_madr` unit-style tests | use `DataKey::generate()` (not zero-key) — no ALLOW_ZERO_KEY required |
| Lost `--no-project-context` / project-session flags | dogfood/evaluate still pass `--no-project-context` explicitly; governed/nightly/cross_repo do not require ambient project IDs for their assertions |

No evidence that hermetic migration dropped a required env/flag for the five long-tail files. Zero-key after hermetic CLI init is covered for both child processes (helper) and parent in-process opens (TempEnv).

### 4. `cross_repo` skip path when ledgerful missing

**Verdict: correct (F17b).**

```173:184:crates/ai-brains-cli/tests/cross_repo_bridge_smoke.rs
fn test_cross_repo_e2e_integration_with_ledgerful() -> Result<(), Box<dyn std::error::Error>> {
    let _allow_zero = TempEnv::set("AI_BRAINS_ALLOW_ZERO_KEY", "1");
    // F17b: ledgerful-only (no changeguard binary fallback); skip if missing.
    let binary = "ledgerful";
    if std::process::Command::new(binary)
        .arg("--version")
        .output()
        .is_err()
    {
        println!("Skipping E2E test: {binary} CLI not found in PATH.");
        return Ok(());
    }
```

- No `changeguard` binary fallback.
- Skip is on spawn failure only (`output().is_err()`), which is the correct “binary not on PATH” signal.
- Non-skip path still asserts `ledgerful init` / `scan` success.

### 5. `cozo_proxy` `project_id: "Ledgerful"` — test breakage

**Verdict: no breakage observed.**

- Production bridge envelopes in `send_datalog_mutation` / `run_datalog_query` use hardcoded `project_id: "Ledgerful".to_string()` (brand placeholder, not a UUID).
- Cozo unit tests only assert Datalog syntax / escape behavior — **no** assert on `project_id`.
- `bridge_record_shape.rs` JSON + assertion pair both use `"Ledgerful"` in lockstep (F16).

Rename-only risk if an external consumer keyed on the old brand string `"ChangeGuard"` is **out of workspace evidence** (wire protocol field names unchanged; F11 out of scope for protocol rename).

### 6. Capture independence + `LedgerfulVerificationBackend`

**Verdict: correct.**

- `VerificationGate::production()` boxes `LedgerfulVerificationBackend`.
- Backend shells `ledgerful bridge export …`; IPC/CLI failure → `GateDecision::ProceedUnavailable` (**fail-open**).
- `CaptureService::new()` still installs the production gate; tests use trait `VerificationBackend` / mocks without requiring models or graph.
- `action_evidence.rs` production path documents fail-open when Ledgerful unreachable — still consistent with rename.
- Capture path does not depend on embeddings/graph for gate operation (F13).

### 7. Public export from capture lib

**Verdict: correct.**

```25:28:crates/ai-brains-capture/src/lib.rs
pub use verification_gate::{
    GateDecision, LedgerfulVerificationBackend, VerificationBackend, VerificationGate,
    VerifyResponse,
};
```

- Hard rename (F15): no `ChangeGuardVerificationBackend` type or alias remains in `*.rs`.
- Workspace call sites compile against new name; no leftover public alias required (workspace-only consumers).

### 8. Production `unwrap` / `expect`

**Verdict: none introduced in production paths of touched modules.**

| Module | Production `unwrap`/`expect` |
|--------|------------------------------|
| `symbol_bridge.rs` | none |
| `verification_gate.rs` | none (test module only) |
| `safety.rs` | none |
| `preflight.rs` / `recall.rs` | none (`unwrap_or` on Option parse is fine) |
| `capture` src production | none new; test-only unwraps remain |

---

## F23 / residual greps (correctness-adjacent)

| Pattern | Production residual |
|---------|---------------------|
| `ChangeGuardHotspot` / `ChangeGuardVerificationBackend` | **0** |
| `query_changeguard*` / `query_symbols_from_changeguard` / `ingest_*_from_changeguard` / `refresh_changeguard_index` | **0** |
| Write-site `"changeguard:symbol"` | only `SOURCE_TAG_SYMBOL_LEGACY` + T167 preserve tests (allowed) |
| `Command::new("ledgerful")` | all production bridge/safety/nightly/capture sites |
| Bare `Command::cargo_bin` in five L13 files | **0** |

Allowed residuals remain: `.changeguard/` path discovery fallback, deprecated path wrappers, T167 preserve comment/fixtures, archive docs.

---

## Findings

### P0 — none

### P1 — none

### P2 — none

### P3 — residual / coverage (non-blocking)

| ID | Severity | Finding | Disposition |
|----|----------|---------|-------------|
| **T191-R1-C01** | P3 | F12 mentioned “mixed” dual-dedup test; suite has legacy-only + new-only + write-tag + idempotent round-trip, but no explicit multi-symbol mixed-tag vault case. OR logic is trivial and covered by the two single-tag cases. | Optional follow-up test; **not** a logic defect |
| **T191-R1-C02** | P3 | `symbol_already_ingested` still treats `read_events` `Err` as “not ingested” (`unwrap_or(false)`), allowing re-pin under store failure. Pre-existing fail-open; dual-read neither improves nor worsens it. | Defer / ISSUES if desired |
| **T191-R1-C03** | P3 | Cosmetic leftovers: `cross_repo` vars/files (`cg_init`, `cg_export*.ndjson`); nightly temp `cg_madr_export.ndjson`. Not behavioral. | Optional cleanup |
| **T191-R1-C04** | P3 | `cozo_proxy` production still hardcodes non-UUID brand `project_id` (`"Ledgerful"`). Rename-consistent; no in-repo test failure path. | Informational; not a T191 DoD miss |

---

## Acceptance mapping (correctness lens)

| AC | Result |
|----|--------|
| AC1 types renamed | Pass |
| AC2 query/ingest/refresh renames | Pass |
| AC3 dual-read + new-write tag | Pass |
| AC4 T167 preserve | Pass (code + tests unchanged intent) |
| AC5–AC6 hermetic L13 | Pass |
| AC10 denylist TX_IDs | Pass |
| AC11 F23 greps | Pass (allowed residuals only) |
| F13 capture independence | Pass |
| F15 hard rename export | Pass |
| F17b ledgerful-only skip | Pass |

---

## Summary

T191’s **correctness-critical** change is the source_tag dual-read before write flip. Implementation matches F2/F12/F22: both tags count as ingested; new writes use `ledgerful:symbol` only; unit tests lock the two failure modes (legacy re-ingest / new re-ingest). Hermetic L13 preserves zero-key open semantics via `hermetic_bin` + explicit TempEnv where the parent process opens the vault. Capture gate rename preserves fail-open production behavior and public API surface. No production panic paths (`unwrap`/`expect`) introduced in reviewed production code.

**PASS — clear for correctness; optional P3 cleanups only.**
