# Protocol Compatibility Policy (T180 / P12.2)

**Status:** Active (fixture-first N−1 until a public `v*` release tag exists)
**Track:** `conductor/tracks/trackT180-protocol-compat-tests/`
**Related:** T158 daemon wire, T148 R0 events, T161 HTTP `/v1`, T176–T178 sync wire, T179 transport matrix

This document is the policy home for **versioned wire protocols**. It does **not** claim runtime `api_version` enforcement or working schema Upcast migrations.

---

## 1. Surfaces

| ID | Surface | Framing | Version | Compat policy |
|----|---------|---------|---------|---------------|
| **P-DAEMON** | `DaemonRequest` / `DaemonResponse` | NDJSON; adjacent tagged `type`/`payload` | Implicit type set; payloads often carry `api_version` | Unknown **type** → **fail closed**. Additive struct fields → **ignored** (no `deny_unknown_fields` on public wire). |
| **P-BRIDGE** | `BridgePayload` / raw `BridgeRecord` | Untagged JSON inside Sync; bare BridgeRecord fallback | No separate version field | Unknown payload shape → **capture** as `Unknown(Value)`. Opposite of P-DAEMON type policy. |
| **P-HTTP** | REST `/v1/*` | HTTP + JSON body | Path `/v1` + body `api_version` string | Same DTO rules as daemon payloads. **`api_version` value is not validated** today. Desktop Tauri invoke uses this surface (**F20**: desktop invoke ≡ P-HTTP). |
| **P-CLI** | JSON/NDJSON stdout + stdin | Command-specific | — | Freeze stable **keys** and **compact vs pretty** per command (see §5). |
| **P-EVENT** | Event envelope + payload | JSON in SQLCipher log | `schema_version: u32` (current = **1**) | R0 `Payload::Unknown` preserves unknown JSON (hand-written serde). Upcast is a **stub**. |
| **P-SYNC** | AIBR / WRAP | Binary/hex KATs | ADR-0018 | Index only in T180; wire is OS-agnostic. DPAPI is **storage**, not wire shape. |

Transport (named pipe / UDS / loopback HTTP) is covered by **T179** (`Docs/COMPATIBILITY.md`), not this file.

---

## 2. Dual policies (must not confuse)

### 2.1 Fail-closed ops (P-DAEMON)

- Adjacent-tagged `DaemonRequest` / `DaemonResponse`.
- **Forbidden:** `#[serde(other)]` on `DaemonRequest` (would silently swallow unknown ops).
- Regression guard: `T180-D-unknown-type` / `daemon_request__unknown_type__fails_deserialize`.
- Live hosts surface `INVALID_REQUEST` via `parse_live_request_line` (never silent drop).

### 2.2 Capture-unknown bridge (P-BRIDGE)

- `BridgePayload` is `#[serde(untagged)]` with a final `Unknown(serde_json::Value)` variant.
- `VerifyOutcome` uses `flatten` → **cannot** combine with `deny_unknown_fields`.
- Legacy clients may send a **bare** `BridgeRecord` (no daemon envelope).
  `parse_live_request_line` falls back: `DaemonRequest` fail → try `BridgeRecord` → `DaemonRequest::Sync`.

---

## 3. `api_version` honesty (F25 / F33 / F36)

### 3.1 Eight module-local constants (not one SOOT)

| Module | Constant location |
|--------|-------------------|
| scopes | `ai_brains_contracts::scopes::API_VERSION` |
| briefings | `ai_brains_contracts::briefings::API_VERSION` (T227 additive warning kinds: `empty_authority` \| `empty_continuity` — only when `!denied`; N−1 clients ignore unknown kinds) |
| knowledge | `ai_brains_contracts::knowledge::API_VERSION` |
| review | `ai_brains_contracts::review::API_VERSION` |
| erasure | `ai_brains_contracts::erasure::API_VERSION` |
| retention | `ai_brains_contracts::retention::API_VERSION` |
| policy | `ai_brains_contracts::policy::API_VERSION` |
| sources | `ai_brains_contracts::sources::API_VERSION` |

All are currently `"1"`. Consolidation to a single re-export is a **residual** (F35), not T180 DoD.

### 3.2 Declarative, not enforced

- HTTP and daemon paths **do not** reject unknown or future `api_version` strings.
- Tests assert `"1"` is present/accepted **and** that `"2"` (and other values) still deserialize today.
- When enforcement lands (future track F36): dual fixtures + doc bump per DTO; bump language alone does **not** reject at runtime until then.

### 3.3 Serialize presence (F33)

Responses that claim `api_version` MUST serialize the field. Value enforcement is out of scope.

---

## 4. `deny_unknown_fields` policy (F26)

| Path | Behavior |
|------|----------|
| Public wire DTOs (contracts / daemon-api) | **MUST NOT** use `deny_unknown_fields` (forward-compat) |
| Crypto / exact-input types | Exception only if required and documented |
| `DryRunIngestRequest` (CLI dry-run only) | **Has** `deny_unknown_fields` — intentional asymmetry vs production ingest |
| Production ingest (`IngestRequest` via `parse_ingest_request`) | Open (unknown fields ignored by serde / validation focuses on required fields) |

---

## 5. CLI compact vs pretty inventory (F14 / F32)

| Command / path | Style | Notes |
|----------------|-------|-------|
| `recall` JSON | **compact** (`to_string`) | Additive T218 (optional, default+skip): per-result `score_kind` (`bm25`\|`rrf`\|`bridge` only) + `cosine`; existing `score` / `source` / `staleness` / response `embedding` unchanged. N−1 clients ignore unknowns. |
| `preflight` JSON (`--format json`, non-summary) | **compact** (`to_string`) | Keys: `text`, `word_count` only (T180 freeze) |
| `preflight --summary --format json` | **pretty** (`to_string_pretty`) | T220 CLI-local envelope: `api_version`, `scope` (`global`\|`project`\|`none`), `project_id`, optional `projects` (global only), `pinned`, `active_sessions`, `in_context_*`, `word_count` (full budget text). Does **not** grow `PreflightContextResponse`. |
| `scope resolve --format json` | **pretty** (`to_string_pretty` via `emit_json`) | T249 TTY/pipe split — default `auto` is TTY human / pipe JSON. JSON **keys unchanged**: `api_version`, `scope`, `confidence`, `authoritative`, `evidence`, `warnings`, `alternatives`. Human path is **not** a wire contract. `--format` tokens are **case-sensitive** (`JSON` / `Pretty` exit 2). |
| Governed mutations (briefing, erasure, …) | **pretty** (`emit_json`) | Machine-clean stdout |
| `ingest` success response | **compact** | `event_id`, `processed` |
| `graph` neighbors/hierarchy/session | TTY **pretty** human; **compact** JSON when piped or `--format json` | **keys unchanged**. T246 adds `--format` so the TTY/pipe split is gated (changing compact↔pretty without a flag remains breaking). |
| Array order (`graph` neighbors/hierarchy/session) | sorted (T246) | Array order: sorted for determinism (neighbors: direction→label→id; hierarchy/session: lexicographic). Pre-T246: SQL encounter order. |
| `graph update` report | **pretty** | T213 fields: `nodes`, `edges`, `pinned_memories`, `memory_nodes`, `edge_node_ratio`, `density` (`ok`\|`warn`\|`skip`), `status` (`live`\|`sparse`\|`empty`), `note`, optional `remediation`. Opt-in `--format human` is **not** the default and is not the T74 parse contract. `--format auto` stays pretty JSON (no TTY switch). |
| `backup create` JSON | **compact** | |
| `dogfood compare` / `evaluate governed` | **pretty** | |
| `agy-hook --schema` / `sync pull --schema` | pretty schema docs | Not versioned wire ops |
| `--version` | clap **text** | Do **not** pin as JSON |
| `doctor` | **pretty** (human default) or JSON via `--json` / `--format json` | T192; `DoctorReport` schema_version=1; `--summary` is human-only presentation (same 15-check report). JSON still full schema_version=1. Not governed OutputFormat missing→Json default. Does **not** TTY-switch. |
| `retention plan` | TTY **pretty** human; **pretty** JSON (`to_string_pretty` / `emit_json`) when piped or `--format json` | T248. **keys unchanged**: `api_version`, `generated_at`, `mode`, `horizons`, `classes`, `totals`, `cascade`, `warnings`, `errors_count`, optional `errors`. Human path is **not** a wire contract. `--format` tokens are **case-sensitive** (`JSON` / `Pretty` exit 2). |
| `retention apply` | **pretty** JSON by default | `--format auto` does **not** TTY-switch. Opt-in `--format human` is not the parse contract. Confirm/scope gates unchanged. |

Changing compact ↔ pretty for a pinned command is a **breaking** CLI contract unless gated by a new flag.

---

## 6. Event schema + Upcast (F8 / F30)

| Fact | Detail |
|------|--------|
| Current schema | `CURRENT_SCHEMA_VERSION = 1` |
| R0 Unknown | Hand-written `Payload` serde; unknown types → `Payload::Unknown(Value)` with full JSON preserved |
| Must not | Replace hand-written Payload serde with a derived adjacent-tagged enum without a dedicated track |
| Upcast | **Stub**: `upcast_once` always `Err(UnknownVersion)` for historical versions; current/future pass-through |
| Active forward-compat | R0 Unknown, **not** Upcast migrations |

---

## 7. Fixture homes (F27)

**Canonical = crate-local only.** Do **not** create a top-level `fixtures/protocol/` tree.

| Surface | Path |
|---------|------|
| P-DAEMON | `crates/ai-brains-daemon-api/tests/fixtures/` |
| P-HTTP | `crates/ai-brains-api-server/tests/fixtures/` |
| P-CLI | `crates/ai-brains-cli/tests/fixtures/` (as needed) |
| P-EVENT | `crates/ai-brains-events/tests/` (+ any local fixtures) |
| Index | This document + T180 id map (§9) |

| Label | Meaning |
|-------|---------|
| **wire-gen-0** | Existing daemon-api **legacy** fixtures (`legacy_*.json`) |
| **wire-gen-1** | Current freezes (scope_resolved, policy_denied, new goldens) |

---

## 8. Breaking-change checklist (F7)

Before a wire break:

1. Prefer **additive** optional fields (default serde ignores unknowns on read).
2. For renames: consider `#[serde(alias = "...")]` to keep N−1 fixtures loading.
3. Required-field removal or type change → dual fixtures (gen-0 + gen-1) + doc.
4. Per-DTO `api_version` bump and/or `/vN` path and/or `schema_version` bump as appropriate.
5. **Honesty:** until enforcement exists, bumping `api_version` alone does **not** reject old/new clients.
6. CLI compact/pretty style switch is breaking unless flagged.
7. Payload hand-serde: do not silently switch to `#[derive(Deserialize)]` on `Payload`.
8. Never add `#[serde(other)]` on `DaemonRequest`.
9. Never add `deny_unknown_fields` on public production wire DTOs.

---

## 9. T180 test id map (elevate-first F28)

### 9.1 P-DAEMON / P-BRIDGE

| T180 id | Existing or new | Location / name |
|---------|-----------------|-----------------|
| `T180-D-legacy-ping` | Elevate | `daemon_request__legacy_ping_json__deserializes` |
| `T180-D-legacy-shutdown` | Elevate | `daemon_request__legacy_shutdown__deserializes` |
| `T180-D-legacy-ingest` | Elevate | `daemon_request__legacy_ingest_json__deserializes` |
| `T180-D-legacy-sync` | Elevate | `daemon_request__legacy_sync_json__deserializes` |
| `T180-D-legacy-pong` | Elevate | `daemon_response__legacy_pong__deserializes` |
| `T180-D-legacy-error` | Elevate | `daemon_response__error_api_error__roundtrip` |
| `T180-D-unknown-type` | Elevate | `daemon_request__unknown_type__fails_deserialize` (+ live `parse_live_request_line__unknown_type__*`) |
| `T180-D-governed-roundtrip-*` | Elevate | `daemon_request__resolve_scope__roundtrip`, briefing, query, inspect, propose_*, review, erasure, wipe |
| `T180-D-e1-scope-empty` | Elevate | `daemon_response__scope_resolved__e1_empty_arrays` |
| `T180-D-e1-query-empty` | Elevate | `daemon_response__query_empty_results__e1` |
| `T180-D-e1-review-empty` | Elevate | `daemon_response__review_list_empty__e1` |
| `T180-D-policy-denied` | Elevate | `daemon_response__error_policy_denied__roundtrip` |
| `T180-D-additive-extra-field` | **New** | `protocol_compat.rs` helper + tests |
| `T180-D-fixture-drift-*` | **New** (F37) | fixture JSON vs re-serialize for key goldens |
| `T180-D-api-version-presence` | **New** | serialize asserts `api_version` on claiming DTOs |
| `T180-D-bridge-unknown` | Elevate / **New** | `BridgePayload` Unknown capture + roundtrip |
| `T180-D-raw-bridge-fallback` | Elevate | `parse_live_request_line__raw_bridge_record__wraps_as_sync` |

### 9.2 P-HTTP

| T180 id | Assert |
|---------|--------|
| `T180-H-api-version-1` | `"1"` accepted on core routes |
| `T180-H-api-version-unenforced` | `"2"` **also** accepted today (honesty) |
| `T180-H-dto-goldens` | Representative bodies deserialize |
| `T180-H-unknown-route` | 404 / defined error shape |
| `T180-H-additive-extra-field` | Public HTTP body DTOs tolerate extra JSON fields |

### 9.3 P-CLI

| T180 id | Assert |
|---------|--------|
| `T180-C-preflight-json-keys` | compact JSON keys `text`, `word_count` |
| `T180-C-scope-json-pretty` | scope resolve JSON uses pretty style + stable keys |
| `T180-C-stdin-dry-run-deny` | dry-run ingest rejects unknown fields |
| `T180-C-stdin-prod-open` | production ingest tolerates unknown fields |

### 9.4 P-EVENT

| T180 id | Assert / source |
|---------|-----------------|
| `T180-E-schema-v1` | `CURRENT_SCHEMA_VERSION == 1` |
| `T180-E-r0-unknown` | Elevate `unknown_payload_roundtrip_preserves_fields` |
| `T180-E-upcast-stub` | Stub returns Err for historical schema versions |

### 9.5 P-SYNC (index only)

| T180 id | Points to |
|---------|-----------|
| `T180-S-index` | T176 schema/crate + T177 engine + **T178** security suite (F1–F28 wire KATs). Wire OS-agnostic; DPAPI is private seed **storage** only (F38). |

Representative T178 / sync locations (T180 does **not** re-implement them):

| Area | Path |
|------|------|
| WRAP / signed-bytes KATs | `crates/ai-brains-sync/src/wrap.rs`, `signed_bytes.rs` (unit tests) |
| Envelope / L5 tamper | `crates/ai-brains-sync/src/envelope.rs` (`t178_l5_*`) |
| Adversarial / memory / file relay | `crates/ai-brains-sync/src/relay.rs` |
| Hex fixtures | `crates/ai-brains-sync/tests/kats/` (e.g. `wrap_seeded_ct.hex`) |
| TwinVaults / CLI replicate | `crates/ai-brains-cli/tests/` (device_replicate / dogfood) + control-plane where applicable |

**HTTP unknown route:** bare Axum `404 Not Found` (no custom JSON error envelope) is the defined contract for unmapped `/v1/*` paths today.

---

## 10. Additive helper (F29)

```text
assert_deserializes_with_extra_fields<T>(base_fixture_json)
```

Injects `_test_unknown_string`, `_test_unknown_number`, `_test_unknown_object` into the top-level JSON object, then deserializes as `T`. Applied to **public** wire DTOs only (not dry-run-only deny types).

Implementation: `crates/ai-brains-daemon-api/tests/protocol_compat.rs` (shared pattern copied where needed for HTTP/CLI unit surfaces).

---

## 11. Residuals (not T180 DoD)

| Residual | Owner |
|----------|-------|
| Runtime `api_version` enforcement | Future track (F36) |
| Single `contracts::API_VERSION` re-export | Soft F35 |
| Working Upcast migrations | Future schema bump |
| Binary N−1 CI artifacts | F24 post-release |
| Optional Docs/schemas jsonschema gate | Soft F34 |
| serde_json minor pin | T185 / supply-chain |
| Multi-OS transport | T179 |

Handoff notes for **T183** (release docs) and **T185** (claims/SBOM): document unenforced `api_version`, Upcast stub, and Bridge capture policy — do not claim stricter guarantees without evidence.

---

## 12. Non-goals

- Infinite N history / third-party client certification
- OpenAPI / full `/v1` jsonschema matrix as DoD
- Root `fixtures/protocol/` tree
- `--version` JSON pinning (doctor JSON is product `DoctorReport` schema_version=1, not a version pin)

- Re-implementing T178 crypto
- Changing capture independence or privacy inheritance

---

*Last updated: T180 implementation (2026-08-01).*
