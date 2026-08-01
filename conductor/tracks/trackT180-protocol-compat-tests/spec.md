# T180 — Backward/Forward Protocol Compatibility Tests (P12.2)

- **Track ID:** T180-ProtocolCompatTests
- **Phase:** P12 — Release hardening and adoption (Task 2)
- **Status:** ✅ **Completed** (2026-08-01) — Codex R1 PASS; local full gate green
- **Depends on:** T158–T161 **Completed** (daemon wire + HTTP `/v1`); T148–T150 event R0 Unknown fidelity; T176–T178 sync AIBR wire KATs (index only); contracts multi-module `API_VERSION = "1"`; no public release tag required (fixture-first)
- **Blocks / feeds:** T183 upgrade notes (limitations: unenforced `api_version`, Upcast stub, BridgePayload capture); T185 wire honesty
- **Category:** FEATURE / RELEASE / TESTING
- **Deferred absorbed:** T158 legacy goldens (elevate); R0 Unknown; E1 shapes already in `protocol_wire.rs` (map + gap-fill); BridgePayload / raw BridgeRecord fallback; T179→T180 handoff. **Not** #34.2; **not** multi-OS; **not** binary N−1 as DoD; **not** `api_version` enforcement (behavior change track).
- **Review fold-in:** AI1 BS1–3 + soft schema/E1; AI2 #1–#9 + A–F (agreed → F25–F38). See §14.

## 1. Objective

Prove **backward and forward** compatibility of **versioned wire protocols** so:

1. **Old clients / recorded fixtures** still deserialize on today’s code (N−1 → N).  
2. **New additive fields** do not break older servers that ignore unknown fields (forward-compat).  
3. **Unknown daemon op types** fail closed (no `#[serde(other)]` silent swallow).  
4. **BridgePayload** unknown types are **captured** as `Unknown(Value)` (opposite policy — document).  
5. **Event R0** preserves unknown payload JSON (hand-written serde; not derived).  
6. Honesty: **`api_version` is declarative, not enforced** today; Upcast is a **stub**.

**Until first public release:** N−1 = frozen golden fixtures. Real binary N−1 is residual after a `v*` tag.

After T180:

| Capability | Present |
|------------|---------|
| PROTOCOL-COMPAT.md (policy + known limitations) | Yes |
| T180 id → existing test map (elevate first) | Yes |
| Additive extra-field helper + tests | Yes |
| Crate-local fixture homes (no root `fixtures/protocol/` fork) | Yes |
| Real `api_version` enforcement | **No** (document only) |
| Working schema Upcast migrations | **No** (stub; R0 is active path) |
| Binary N−1 CI | **No** (residual) |

## 2. Live baseline (re-scan 2026-07-31 + fold-in)

| Surface | Live state |
|---------|------------|
| P-DAEMON | `ai-brains-daemon-api`: adjacent tag `type`/`payload`/`snake_case`; **unknown type fail-closed** |
| P-DAEMON fixtures | `crates/ai-brains-daemon-api/tests/fixtures/` + **~527-line** `protocol_wire.rs` (legacy, governed roundtrips, E1 scope/query/review, policy denied, tags) |
| P-BRIDGE (sub) | `BridgePayload` **untagged** + `Unknown(Value)` catch-all; `VerifyOutcome` uses `flatten` → **cannot** add `deny_unknown_fields`; raw `BridgeRecord` fallback in `parse_live_request_line` |
| P-HTTP | `/v1/*`; body DTOs carry `api_version`; **no validation** of value |
| `API_VERSION` | **Eight** separate `pub const API_VERSION: &str = "1"` in contracts modules (scopes, briefings, knowledge, review, erasure, retention, policy, sources) — not one SOOT |
| P-CLI | Mix **compact** (`to_string`) vs **pretty** (`to_string_pretty`); `--version` is clap **text**; **no** `doctor` command |
| `deny_unknown_fields` | **Only** `DryRunIngestRequest` (`ingest.rs`); production ingest path does **not** use it |
| P-EVENT | `schema_version = 1`; R0 hand-written Payload serde; **Upcast scaffold** with **no** migrations (`upcast_once` always Err except current/future pass-through) |
| P-SYNC | T176–T178 KATs; wire OS-agnostic; DPAPI only private seed storage |
| Docs schemas | `Docs/schemas/agy-hook-payload.json`, `sync-pull-record.json` (narrow; not full `/v1` surface) |
| Public release | No N−1 binary assumed |

### 2.1 Surface inventory (normative)

| ID | Surface | Framing | Version | Compat policy |
|----|---------|---------|---------|---------------|
| **P-DAEMON** | `DaemonRequest`/`DaemonResponse` | NDJSON; adjacent tagged | Implicit `type` set; payloads often `api_version` | Unknown **type** → **fail**; additive struct fields → ignore (no `deny_unknown_fields` on public wire) |
| **P-BRIDGE** | `BridgePayload` / raw `BridgeRecord` | Untagged JSON inside Sync / bare fallback | None separate | Unknown → **capture** `Unknown(Value)`; opposite of P-DAEMON type policy |
| **P-HTTP** | REST `/v1/...` | HTTP + JSON | Path `/v1` + body `api_version` string | Same DTO rules; **value not enforced** today |
| **P-CLI** | JSON/NDJSON stdout + stdin | Command-specific | — | Freeze **keys** + **compact vs pretty** per command |
| **P-EVENT** | Envelope + payload | JSON in log | `schema_version: u32` | R0 Unknown active; Upcast **stub** for future v1→v2 |
| **P-SYNC** | AIBR/WRAP | Binary/hex KATs | ADR-0018 | Index only |

### 2.2 `api_version` honesty (**F25**)

| Fact | Implication for T180 |
|------|----------------------|
| 8 module-local `API_VERSION = "1"` constants | Document list; **per-DTO bump** policy if any breaks (dual fixtures for that DTO only). Consolidation to single SOOT = residual refactor, not DoD |
| Never validated on HTTP or daemon paths | Any string (incl. `"2"`, `"banana"`) accepted after deserialize |
| Tests | Assert `"1"` present/accepted **and** assert `"2"` accepted today with comment “update when enforcement lands” |
| F7 bump language | Aspirational until enforcement track exists — checklist must say so |

### 2.3 `deny_unknown_fields` policy (**F26** — AI1 BS1 + AI2 #8)

| Path | Behavior |
|------|----------|
| Public wire DTOs (contracts / daemon-api payloads) | **MUST NOT** use `deny_unknown_fields` (forward-compat) |
| Crypto/signature exact-input types | Exception only if required; document |
| `DryRunIngestRequest` only | Keep or document; **asymmetric** vs production ingest (dry-run strict, prod open) |
| Phase A | Audit; remove only if found on **public production** wire (currently none beyond dry-run) |

### 2.4 Fixture homes (**F27** — AI1 BS2)

**Canonical = crate-local only.** Do **not** create a parallel top-level `fixtures/protocol/` tree.

| Surface | Path |
|---------|------|
| P-DAEMON | `crates/ai-brains-daemon-api/tests/fixtures/` |
| P-HTTP | `crates/ai-brains-api-server/tests/fixtures/` (create as needed) |
| P-CLI | `crates/ai-brains-cli/tests/fixtures/` |
| P-EVENT | `crates/ai-brains-events/tests/fixtures/` (or existing test modules) |
| Index | `Docs/PROTOCOL-COMPAT.md` points at these paths + T180 id map |
| `wire-gen-0` | **Alias** of existing daemon-api legacy fixtures (document; do not duplicate) |
| `wire-gen-1` | Current generation freezes in same crate trees |

### 2.5 Elevate vs write-new (**F28** — AI2 #5)

Existing `protocol_wire.rs` already covers most §5.1 rows. T180 work:

| Work | Action |
|------|--------|
| Legacy, unknown type, governed roundtrips, E1 samples, policy denied | **Map** to `T180-D-*` ids (comments/table); do not rewrite |
| Additive extra-field | **Write new** helper + tests |
| Fixture-based governed goldens (file-backed N−1) | **Write new** where only code-roundtrip exists today |
| `api_version: "2"` accepted | **Write new** honesty test |
| Bridge unknown + raw BridgeRecord | **Elevate** existing tests into index |

## 3. Research summary (online + standards, 2026-07-31)

### 3.1 Compatibility practices

| Practice | T180 |
|----------|------|
| Golden fixtures | Crate-local `include_str!` |
| Additive evolution | Default serde ignore unknown fields |
| Fail-closed ops | Adjacent-tagged enums; forbid `#[serde(other)]` on DaemonRequest |
| Capture-unknown (bridge) | Untagged + Unknown variant |
| N / N−1 | Fixture-first |
| Hermetic CI | No untrusted binary download |

### 3.2 Serde specifics

| Topic | Finding | T180 |
|-------|---------|------|
| Default ignore unknown fields | Serde JSON default | F6 foundation |
| `deny_unknown_fields` + `flatten` | **Unsupported** together | BridgePayload cannot add deny |
| `#[serde(other)]` | Allowed on unit variant of tagged enums | **Forbidden** on DaemonRequest (F3); unknown-type test is regression guard |
| Hand-written Payload serde | Enables R0 | **Must not** replace with derived tag enum |
| `#[serde(alias)]` | Rename without hard break | Breaking-change checklist option |
| `insta` / `proptest` | Optional | Prefer zero; if added pin `insta = "1.48"`, `proptest = "1.11"`, cases ≤100 |

### 3.3 Dependency posture

| Crate | Action |
|-------|--------|
| serde / serde_json | Hold workspace `1.0` floors; no T180 bump required |
| New production deps | **Forbidden** |
| `jsonschema` for Docs/schemas | **Soft residual** only (AI1 Opp1); not DoD; schemas cover hook/sync-pull only |

## 4. Frozen design decisions (F1–F38)

| ID | Decision |
|----|----------|
| **F1** | Fixture-first N−1 until release binary exists. |
| **F2** | Surfaces: P-DAEMON, **P-BRIDGE**, P-HTTP, P-CLI, P-EVENT, P-SYNC(index). |
| **F3** | Unknown **DaemonRequest** `type` fails deserialize; no `#[serde(other)]`. |
| **F4** | Legacy ping/ingest/sync/shutdown goldens always load. |
| **F5** | Governed ops: elevate roundtrips; add file goldens where N−1 proof needs files. |
| **F6** | Additive optional object fields; public wire **without** `deny_unknown_fields`. |
| **F7** | Breaks: version bump (per-DTO `api_version` and/or `/vN` and/or `schema_version`) + dual fixtures + doc. Until enforcement exists, `api_version` bump alone does **not** reject old/new at runtime — honesty required. Optional `#[serde(alias)]` for renames. |
| **F8** | Event R0 Unknown preserve is **active** forward-compat; hand-written Payload serde must not be replaced with derived. |
| **F9** | Zero new production deps; prefer zero new dev-deps. |
| **F10** | P-SYNC index only; T178 wire OS-agnostic; DPAPI not wire shape. |
| **F11** | Wire JSON OS-agnostic; transport (pipe/UDS/HTTP) is T179. |
| **F12** | E1: map existing protocol_wire E1; gap-fill only; new public DTOs require E1. |
| **F13** | HTTP: DTO goldens + honesty tests for unenforced `api_version`. |
| **F14** | CLI: pin stable JSON for **scope resolve / preflight / recall** (not doctor; not `--version` text). |
| **F15** | `wire-gen-0` / `wire-gen-1` labels; gen-0 = existing daemon legacy fixtures. |
| **F16** | Policy home: `Docs/PROTOCOL-COMPAT.md`. |
| **F17** | Default nextest; no binary N−1 job as DoD. |
| **F18** | No AGPL fuzz required. |
| **F19** | Capture independence unchanged. |
| **F20** | Desktop invoke = P-HTTP. |
| **F21** | Document spool / fire-and-forget no-panic if residual. |
| **F22** | Deterministic fixtures (fixed UUIDs/timestamps). |
| **F23** | Elevate T158 suite; do not fork. |
| **F24** | Binary N−1 residual post-release (project-owned artifacts only). |
| **F25** | **Eight** `API_VERSION` constants; per-DTO bump policy; **not validated** at runtime. |
| **F26** | Public wire: no `deny_unknown_fields`; dry-run ingest asymmetry documented + dual-path test. |
| **F27** | **Crate-local fixtures only** (no root fixtures tree). |
| **F28** | Elevate-first; write-new only for additive helper, file goldens, honesty cases. |
| **F29** | Shared test helper `assert_deserializes_with_extra_fields` (AI1 BS3) for public wire DTOs. |
| **F30** | Upcast = **stub**; document future-version pass-through + R0. |
| **F31** | Bridge unknown = capture; raw BridgeRecord fallback documented. |
| **F32** | CLI compact vs pretty frozen per command; style switch is breaking unless flagged. |
| **F33** | Response/request serialize asserts `api_version` field presence where DTO claims it (not value enforcement). |
| **F34** | Soft: optional Docs/schemas validation for hook/sync-pull only; not full matrix DoD. |
| **F35** | Soft: single `contracts::API_VERSION` re-export residual (not DoD). |
| **F36** | Soft: `api_version` enforcement track residual. |
| **F37** | Fixture-vs-current-serialize drift tests for key goldens (N−1 proof). |
| **F38** | P-SYNC index notes OS-agnostic wire vs DPAPI storage cfg. |

## 5. Test matrix (minimum)

### 5.1 P-DAEMON / P-BRIDGE

| Test id | Source | Assert |
|---------|--------|--------|
| `T180-D-legacy-*` | Elevate T158 | Legacy fixtures deserialize |
| `T180-D-unknown-type` | Elevate | Fail closed (guards against `#[serde(other)]`) |
| `T180-D-governed-roundtrip-*` | Elevate | Existing roundtrips |
| `T180-D-e1-*` | Elevate | Existing E1 samples |
| `T180-D-additive-extra-field` | **New** | Helper injects unknown fields; public DTOs still deserialize |
| `T180-D-fixture-governed-*` | **New** (gap) | File-backed goldens for high-traffic ops |
| `T180-D-bridge-unknown` | Elevate | BridgePayload → Unknown; roundtrip |
| `T180-D-raw-bridge-fallback` | Elevate | Bare BridgeRecord accepted as Sync |

### 5.2 P-HTTP

| Test id | Assert |
|---------|--------|
| `T180-H-api-version-1` | `"1"` accepted on core routes |
| `T180-H-api-version-unenforced` | `"2"` **also** accepted today (honesty) |
| `T180-H-dto-goldens` | Representative bodies deserialize |
| `T180-H-unknown-route` | 404 / defined error shape |

### 5.3 P-CLI

| Test id | Assert |
|---------|--------|
| `T180-C-json-stable-keys` | preflight or scope resolve keys + style (compact/pretty) frozen |
| `T180-C-stdin-policy` | dry-run deny_unknown **vs** production open |

### 5.4 P-EVENT

| Test id | Assert |
|---------|--------|
| `T180-E-schema-v1` | schema_version == 1 |
| `T180-E-r0-unknown` | Unknown payload re-serialize stable |
| `T180-E-upcast-stub-doc` | Doc-only / optional unit that stub returns Err for v0 (no fake migration) |

### 5.5 P-SYNC

| Test id | Assert |
|---------|--------|
| `T180-S-index` | Table of T176/T178 tests; wire OS-agnostic note |

## 6. Helper (normative sketch)

```rust
// tests/common or daemon-api test module
pub fn assert_deserializes_with_extra_fields<T: serde::de::DeserializeOwned>(
    base_fixture_json: &str,
) {
    let mut val: serde_json::Value =
        serde_json::from_str(base_fixture_json).expect("fixture JSON");
    if let Some(obj) = val.as_object_mut() {
        obj.insert("_test_unknown_string".into(), "unknown_value".into());
        obj.insert("_test_unknown_number".into(), 42.into());
        obj.insert(
            "_test_unknown_object".into(),
            serde_json::json!({"nested": true}),
        );
    }
    let _: T = serde_json::from_value(val)
        .expect("public wire DTO must tolerate additive unknown fields");
}
```

Apply to **public** wire DTOs (not dry-run-only types that intentionally deny).

## 7. Acceptance criteria

| ID | Criterion |
|----|-----------|
| **AC1** | PROTOCOL-COMPAT.md: surfaces, dual policies (fail-closed vs bridge capture), API_VERSION×8, unenforced api_version, Upcast stub, fixture paths |
| **AC2** | T180 id map covers elevated T158 suite + new additive/honesty tests |
| **AC3** | Additive helper used on daemon/HTTP public DTOs |
| **AC4** | P-CLI stable keys + compact/pretty inventory + stdin dual-path |
| **AC5** | P-EVENT R0 green; Upcast documented as stub |
| **AC6** | P-SYNC index only |
| **AC7** | Breaking-change checklist (incl. alias, api_version honesty, Payload hand-serde) |
| **AC8** | Zero new production deps; deny/audit green |
| **AC9** | Default nextest; suite &lt;60s |
| **AC10** | Conductor + deferred updated |
| **AC11** | No top-level fixture tree fork |
| **AC12** | Bridge + raw BridgeRecord documented and indexed |

## 8. Deferred.md absorption

| Item | Disposition |
|------|-------------|
| T158 goldens + protocol_wire breadth | Elevate / map |
| R0 Unknown | Elevate |
| E1 existing | Map; gap-fill only |
| Bridge / raw BridgeRecord | P-BRIDGE |
| T179 handoff | Absorb |
| api_version enforcement | Residual F36 |
| Single API_VERSION SOOT | Residual F35 |
| jsonschema vs Docs/schemas | Soft F34 |
| Binary N−1 | F24 residual |
| #34.2 / multi-OS / WRAP reimpl | Out |

## 9. Non-goals

| Out | Owner |
|-----|--------|
| Enforce api_version at runtime | Future track |
| Implement Upcast migrations | Future schema bump |
| Infinite history / third-party clients | — |
| OpenAPI / full jsonschema gate | Soft only |
| Root fixtures/protocol tree | Rejected F27 |
| doctor / --version JSON claims | Incorrect — do not pin |

## 10. License / commercial

Fixtures/tests only; no AGPL tools; project-owned binaries only if N−1 residual lands.

## 11. Risks

| Risk | Mitigation |
|------|------------|
| Claiming version enforcement | F25 honesty tests |
| Assuming Upcast works | F30 |
| Ignoring Bridge opposite policy | F31 |
| Fixture fragmentation | F27 |
| Redundant rewrite of protocol_wire | F28 |
| dry-run vs prod stdin surprise | F26 dual test |
| Pretty/compact script break | F32 |

## 12. Definition of Done

- [x] F1–F38 reflected  
- [x] AC1–AC12 green  
- [x] PROTOCOL-COMPAT.md published  
- [x] Conductor → Completed after review (internal R2 + Codex R1 PASS)  

## 13. Implementation priority

1. **A** — Inventory (API_VERSION×8, deny_unknown, compact/pretty, elevate map) + PROTOCOL-COMPAT.md  
2. **B** — Map T158 ids; **new** additive helper + honesty tests; optional file goldens  
3. **C** — HTTP/CLI/EVENT gap-fill  
4. **D** — P-SYNC index; residual notes for T183/T185  
5. **E** — Gate + closeout  

**Shipped 2026-08-01** (branch `track/T180-protocol-compat-tests`).

## 14. AI review disposition

| Source | Item | Disposition |
|--------|------|-------------|
| AI1 BS1 | No deny_unknown on public wire | **Agree** — F26; live only dry-run has it; audit not mass rewrite |
| AI1 BS2 | Crate-local fixtures | **Agree** — F27 |
| AI1 BS3 | Extra-field helper | **Agree** — F29 |
| AI1 Opp1 | jsonschema vs Docs/schemas | **Soft** — F34; not DoD; no required new dep |
| AI1 Opp2 | E1 every DTO | **Partial** — elevate existing; require for new public DTOs; not unbounded “every” rewrite |
| AI2 #1 | 8 API_VERSION | **Agree** — F25 |
| AI2 #2 | Unenforced api_version | **Agree** — F25 + honesty tests |
| AI2 #3 | Upcast stub | **Agree** — F30 |
| AI2 #4 | BridgePayload | **Agree** — F2/F31 P-BRIDGE |
| AI2 #5 | Elevate vs new | **Agree** — F28 |
| AI2 #6 | F14 doctor/version | **Agree** — F14 corrected |
| AI2 #7 | compact vs pretty | **Agree** — F32 |
| AI2 #8 | deny dry-run only | **Agree** — F26 |
| AI2 #9 | pin insta/proptest if used | **Agree** |
| AI2 A | other regression | Covered by unknown-type test |
| AI2 B | fixture vs serialize drift | **Agree** soft-strong — F37 |
| AI2 C | raw BridgeRecord | **Agree** — F31 |
| AI2 D | api_version presence | **Agree** — F33 |
| AI2 E | serde alias policy | **Agree** — F7 |
| AI2 F | hand-written Payload | **Agree** — F8 |
| AI2 G | pin serde_json minor | Soft residual (T185); not T180 DoD |
