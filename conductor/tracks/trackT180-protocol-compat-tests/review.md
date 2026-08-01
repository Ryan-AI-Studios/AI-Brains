# T180 Internal Review

**Reviewer:** Grok Build (read-only internal)  
**Date:** 2026-08-01  
**Branch:** `track/T180-protocol-compat-tests`  
**Scope:** Spec AC1–AC12, F1–F38, landed tests/docs vs policy honesty

## Verdict: PASS (internal R2 pending re-review)

Round 1: PASS WITH DEFERRED (2 medium + 4 low). Round 1→2 fixes applied 2026-08-01 (M1/M2 + L1–L4).

| Severity | Count | Status |
|----------|------:|--------|
| critical | 0 | — |
| high | 0 | — |
| medium | 2 | **verified_fixed** (M1 CLI production paths; M2 sync index paths) |
| low | 4 | **verified_fixed** (L1–L4) |

---

## Requirement and DoD Matrix

### Acceptance criteria (AC1–AC12)

| ID | Criterion | Result | Evidence |
|----|-----------|--------|----------|
| **AC1** | PROTOCOL-COMPAT.md: surfaces, dual policies, API_VERSION×8, unenforced api_version, Upcast stub, fixture paths | **PASS** | `Docs/PROTOCOL-COMPAT.md` §§1–7, §11 |
| **AC2** | T180 id map covers elevated T158 + new additive/honesty | **PASS** | PROTOCOL-COMPAT §9; `protocol_wire.rs` module map; `protocol_compat*.rs` |
| **AC3** | Additive helper on daemon/HTTP public DTOs | **PASS** | `assert_deserializes_with_extra_fields` in daemon-api; HTTP `with_extra_fields` + DTO-level test |
| **AC4** | P-CLI stable keys + compact/pretty inventory + stdin dual-path | **PARTIAL** | Inventory §5 complete; stdin dual-path exercises CLI binary; **compact/pretty style tests do not call production emit paths** (see M1) |
| **AC5** | P-EVENT R0 green; Upcast documented as stub | **PASS** | `protocol_compat_events.rs`; elevate comment on `unknown_payload_roundtrip_preserves_fields.rs`; PROTOCOL-COMPAT §6 |
| **AC6** | P-SYNC index only | **PASS** (doc path imprecise) | Index in PROTOCOL-COMPAT §9.5; no reimplementation. Path claim slightly wrong (L2) |
| **AC7** | Breaking-change checklist | **PASS** | PROTOCOL-COMPAT §8 (alias, honesty, hand-serde, no `#[serde(other)]`, no public `deny_unknown_fields`) |
| **AC8** | Zero new production deps | **PASS** | `ai-brains-daemon-api` deps still contracts+serde only; no insta/proptest/jsonschema added; no AGPL tools |
| **AC9** | Default nextest; suite &lt;60s | **PASS** (claimed) | Plan evidence 32/32 protocol suites; pure unit/integration JSON tests. CLI ingest subprocess tests may be slower but still default-tier shape |
| **AC10** | Conductor + deferred updated | **PARTIAL** | `deferred.md` §57 + conductor In Progress; **Completed** + pin decision still Phase E closeout |
| **AC11** | No top-level fixture tree fork | **PASS** | No `fixtures/protocol/`; goldens under crate `tests/fixtures/`. Existing `fixtures/governed-memory/` pre-dates T180 and is not a protocol fork |
| **AC12** | Bridge + raw BridgeRecord documented and indexed | **PASS** | PROTOCOL-COMPAT §2.2; tests `t180_d_bridge_unknown*`; elevate `parse_live_request_line__raw_bridge_record__wraps_as_sync` |

### Critical / structural F items

| ID | Decision | Result | Notes |
|----|----------|--------|-------|
| **F1** | Fixture-first N−1 | **PASS** | Legacy + new governed/HTTP fixtures |
| **F2** | Surfaces incl. P-BRIDGE | **PASS** | §1 surfaces table |
| **F3** | Unknown DaemonRequest type fail-closed | **PASS** | `daemon_request__unknown_type__fails_deserialize` + live parse |
| **F4** | Legacy goldens load | **PASS** | wire-gen-0 fixtures + tests |
| **F5** | Governed elevate + file goldens gap | **PASS** | Roundtrips elevated; `governed_resolve_scope_request.json` |
| **F6** | No `deny_unknown_fields` on public wire | **PASS** | Repo grep: only `DryRunIngestRequest` |
| **F7** | Breaking checklist + honesty | **PASS** | §8 |
| **F8** | R0 Unknown active; hand-serde | **PASS** | Elevated + documented |
| **F9** | Zero new prod deps | **PASS** | |
| **F10** | P-SYNC index; wire OS-agnostic | **PASS** | §9.5 / F38 |
| **F11** | Wire OS-agnostic vs T179 transport | **PASS** | Documented |
| **F12** | E1 map existing | **PASS** | protocol_wire E1 tests mapped |
| **F13** | HTTP goldens + honesty | **PASS** | `protocol_compat.rs` HTTP |
| **F14** | CLI pin preflight/scope (not doctor/--version) | **PARTIAL** | Inventory correct; style regression weak (M1). Recall mentioned in F14 text but matrix only requires preflight/scope |
| **F15** | wire-gen-0/1 labels | **PASS** | §7 |
| **F16** | Policy home PROTOCOL-COMPAT.md | **PASS** | |
| **F17** | Default nextest; no binary N−1 DoD | **PASS** | |
| **F18** | No AGPL fuzz | **PASS** | |
| **F19** | Capture independence unchanged | **PASS** | Tests/docs only |
| **F20** | Desktop invoke = P-HTTP | **SOFT GAP** | Not stated in PROTOCOL-COMPAT (L3) |
| **F21** | Spool residual doc | **N/A / residual** | Not in T180 surface inventory; OK out |
| **F22** | Deterministic fixtures | **PASS** | Fixed UUIDs/paths in goldens |
| **F23** | Elevate T158; no fork | **PASS** | |
| **F24** | Binary N−1 residual | **PASS** (residual) | §11 |
| **F25** | Eight API_VERSION; unenforced | **PASS** | Honesty tests `"1"`/`"2"`/`"banana"`; constants test |
| **F26** | Public open; dry-run asymmetric + dual test | **PASS** | CLI binary dual-path |
| **F27** | Crate-local fixtures only | **PASS** | |
| **F28** | Elevate-first | **PASS** | |
| **F29** | Shared additive helper | **PASS** | Daemon-api helper; HTTP copy pattern per §10 |
| **F30** | Upcast stub | **PASS** | v0 Err; current/future pass-through |
| **F31** | Bridge capture + raw fallback | **PASS** | |
| **F32** | Compact vs pretty frozen | **PARTIAL** | Inventory yes; regression tests weak (M1) |
| **F33** | Serialize `api_version` presence | **PASS** (representative) | `ScopeResolvedResponse` |
| **F34** | Soft jsonschema | **PASS** residual | Not DoD |
| **F35** | Soft single SOOT | **PASS** residual | |
| **F36** | Soft enforcement track | **PASS** residual | Honesty, not enforcement |
| **F37** | Fixture-vs-serialize drift | **PASS** (partial strength) | Full equality for scope_resolved + policy_denied; legacy_ping only checks `type` (L1) |
| **F38** | Sync OS-agnostic vs DPAPI storage | **PASS** | Documented |

### Test matrix coverage (spec §5)

| Test id | Landed? | Strength |
|---------|---------|----------|
| `T180-D-legacy-*` | Yes (elevate) | Strong file goldens |
| `T180-D-unknown-type` | Yes | Strong (serde + live) |
| `T180-D-governed-roundtrip-*` | Yes (elevate) | Strong |
| `T180-D-e1-*` | Yes (elevate) | Strong |
| `T180-D-additive-extra-field` | Yes | Strong on request DTOs + envelope ping |
| `T180-D-fixture-governed-*` | Yes | Deserialize N−1 |
| `T180-D-bridge-unknown` | Yes | Strong capture + roundtrip + known Query |
| `T180-D-raw-bridge-fallback` | Yes (elevate live) | Strong |
| `T180-H-api-version-1` | Yes | Route-level knowledge query |
| `T180-H-api-version-unenforced` | Yes | `"2"` on query + resolve_scope |
| `T180-H-dto-goldens` | Yes | Two fixtures |
| `T180-H-unknown-route` | Yes | Status only (L4) |
| `T180-H-additive-extra-field` | Yes | HTTP body + DTO |
| `T180-C-preflight-json-keys` | Yes | Keys strong; style weak (M1) |
| `T180-C-scope-json-pretty` | Yes | Keys strong; style weak (M1) |
| `T180-C-stdin-dry-run-deny` | Yes | CLI subprocess — strong core assert |
| `T180-C-stdin-prod-open` | Yes | CLI subprocess — strong |
| `T180-E-schema-v1` | Yes | Strong |
| `T180-E-r0-unknown` | Yes | Thin elevate + full suite |
| `T180-E-upcast-stub` | Yes | Strong stub honesty |
| `T180-S-index` | Docs + misnamed constant test | Index is doc; test is F25 constants (L2) |

---

## Findings

### M1 — CLI compact/pretty tests do not exercise production emission paths
- **severity:** medium  
- **description:** `t180_c_preflight_json_keys__serialize_is_compact_shape` calls `serde_json::to_string` on a constructed DTO; `t180_c_scope_json_pretty__emit_style_is_pretty` calls `serde_json::to_string_pretty` itself. Neither invokes `preflight` JSON path (`serde_json::to_string` in `commands/preflight.rs`) nor `governed_common::emit_json` (`to_string_pretty` used by `scope resolve --format json`). A switch of production style would **not** fail these tests. Stable **keys** on the DTO are still guarded; F32 style freeze is not. Stdin dual-path correctly uses the CLI binary.  
- **files:**  
  - `crates/ai-brains-cli/tests/protocol_compat_cli.rs`  
  - `crates/ai-brains-cli/src/commands/preflight.rs` (prod compact)  
  - `crates/ai-brains-cli/src/commands/governed_common.rs` (`emit_json`)  
  - `crates/ai-brains-cli/src/commands/scope.rs`  
- **required_fix:** Pin style via CLI binary smoke for `preflight --format json` and `scope resolve --format json`.  
- **status:** verified_fixed  
- **evidence:** `protocol_compat_cli.rs` now runs `CARGO_BIN_EXE_ai-brains` for preflight (compact) and `scope resolve --format json --local` (pretty); nextest 4/4 pass after fix.

### M2 — PROTOCOL-COMPAT P-SYNC index path is inaccurate
- **severity:** medium  
- **description:** §9.5 previously claimed tests live under `crates/ai-brains-sync/tests/` (mostly empty); real KATs are unit tests in `src/`.  
- **files:** `Docs/PROTOCOL-COMPAT.md` §9.5  
- **required_fix:** Update index to crate module paths.  
- **status:** verified_fixed  
- **evidence:** §9.5 table lists wrap.rs, envelope.rs, relay.rs, tests/kats hex, CLI TwinVaults paths.

### L1 — Legacy ping fixture-drift assert is weaker than peers
- **severity:** low  
- **status:** verified_fixed  
- **evidence:** full structural `assert_eq!(again, original)` for legacy ping.

### L2 — `t180_s_index__api_version_constants_are_one` is not a P-SYNC index test
- **severity:** low  
- **status:** verified_fixed  
- **evidence:** renamed to `t180_f25_api_version_constants__all_modules__are_one`; comment clarifies P-SYNC is docs-only.

### L3 — F20 desktop→P-HTTP not stated in policy home
- **severity:** low  
- **status:** verified_fixed  
- **evidence:** P-HTTP row notes desktop invoke ≡ P-HTTP (F20).

### L4 — `T180-H-unknown-route` checks status only
- **severity:** low  
- **status:** verified_fixed  
- **evidence:** PROTOCOL-COMPAT documents bare Axum 404 as defined shape; test comment aligned.  

---

## Completeness

### Landed (implementation)

| Area | Artifacts |
|------|-----------|
| Policy | `Docs/PROTOCOL-COMPAT.md` (surfaces, dual policies, honesty, deny_unknown, CLI inventory, checklist, id map, residuals) |
| P-DAEMON elevate | `protocol_wire.rs` T180 map comments; legacy/E1/governed/unknown-type |
| P-DAEMON gap-fill | `protocol_compat.rs` — additive helper, honesty, drift, bridge, governed golden |
| Fixtures | `governed_resolve_scope_request.json`; HTTP `query_knowledge_v1.json`, `resolve_scope_v1.json` |
| Live boundary elevate | `daemon_dispatch_shared.rs` T180 comments on unknown-type + raw BridgeRecord |
| P-HTTP | `ai-brains-api-server/tests/protocol_compat.rs` |
| P-CLI | `protocol_compat_cli.rs` (stdin dual-path strong) |
| P-EVENT | `protocol_compat_events.rs` + R0 elevate pointer |
| Conductor process | Plan A–D checked; deferred §57 “In Progress / Implemented” |

### Not done / closeout (expected Phase E)

- [ ] E1 full CI gate  
- [ ] E2 targeted nextest including **CLI** (`plan.md` manual evidence still says CLI “pending”)  
- [ ] E3 Conductor → Completed after review  
- [ ] E4 deferred §57 finalization after merge  
- [ ] E5 pin decision (fixture-first + dual policies + unenforced api_version)  

### Explicit non-claims verified (honesty)

| Claim avoided | How verified |
|---------------|--------------|
| Runtime `api_version` enforcement | Honesty tests require `"2"` (and `"banana"`) accepted; docs F36 residual |
| Working Upcast migrations | Stub tests require v0 `Err`; docs F30 |
| Root `fixtures/protocol/` | Absent |
| New production deps | Cargo.toml surfaces inspected; zero added |
| doctor / `--version` JSON | Not pinned; inventory says absent/text |

### Placeholders / incomplete wiring

- No `TODO`/`FIXME`/`todo!`/`unimplemented!` in new protocol_compat suites.  
- No fake migration implementation.  
- Soft F34 jsonschema correctly not required.

---

## Residual risks

1. **CLI style drift (M1):** Scripts that assume compact preflight or pretty scope JSON could break without CI signal if production emit helpers change.  
2. **Index drift (M2):** Release/docs tracks (T183/T185) may cite wrong paths for sync KATs.  
3. **Unenforced `api_version`:** Documented correctly; clients may assume enforcement — handoff to T183/T185 must keep honesty language.  
4. **Eight local `API_VERSION` constants:** Bump discipline remains per-DTO; consolidation residual F35.  
5. **Bridge vs Daemon dual policy:** Easy to confuse in future enum changes; unknown-type + Bridge Unknown tests mitigate.  
6. **Dry-run vs prod ingest asymmetry:** Documented and dual-tested via real CLI; operators may still be surprised.  
7. **Binary N−1 / enforcement / Upcast migrations / jsonschema:** Explicit residuals; not T180 DoD.

---

## Clearance recommendation

1. **Fix M1 and M2** (or formally defer with justification + `conductor/ISSUES.md`, cap ≤3 mediums).  
2. Optionally tidy L1–L4.  
3. Run CLI `protocol_compat_cli` nextest and record evidence in `plan.md`.  
4. Complete Phase E closeout + pin; then Conductor **Completed**.

**No critical/high blockers.** Track is feature-complete for the intended fixture-first protocol compat program; remaining work is regression-test strength, index accuracy, and process closeout.

---

## Round 2 re-review

**Reviewer:** Grok Build (read-only internal re-reviewer)  
**Date:** 2026-08-01  
**Scope:** Verify M1/M2/L1–L4 `verified_fixed` claims; spot-check PROTOCOL-COMPAT §9.5 + F20; CLI style freezes via binary; fresh regression sweep.

### Verdict: PASS

All six Round-1 findings are **confirmed fixed** in code/docs. No new critical / high / medium findings. No new low findings that require deferral. Phase E process closeout remains out of scope for this re-review (unchanged).

### Prior finding verification

| ID | Claimed status | Re-review | Evidence |
|----|----------------|-----------|----------|
| **M1** | verified_fixed | **CONFIRMED** | `crates/ai-brains-cli/tests/protocol_compat_cli.rs`: `t180_c_preflight_json_keys__cli_format_json__compact_stable_keys` and `t180_c_scope_json_pretty__cli_format_json__pretty_stable_keys` invoke `env!("CARGO_BIN_EXE_ai-brains")` (via `run_cli`). Preflight asserts compact (`!line.contains('\n')`) + keys `text`/`word_count`; scope asserts pretty (`contains('\n')`) + seven stable keys. Production paths match: `preflight.rs` `serde_json::to_string`; `governed_common::emit_json` → `to_string_pretty` used by `scope.rs` for `OutputFormat::Json`. No test-local `to_string`/`to_string_pretty` style freeloading. |
| **M2** | verified_fixed | **CONFIRMED** | `Docs/PROTOCOL-COMPAT.md` §9.5 indexes real locations: `ai-brains-sync/src/{wrap,signed_bytes,envelope,relay}.rs` (unit tests present), `tests/kats/wrap_seeded_ct.hex`, CLI `device_replicate_cli.rs` / dogfood. Does **not** claim empty `tests/` as primary KAT home. |
| **L1** | verified_fixed | **CONFIRMED** | `t180_d_fixture_drift__legacy_ping__roundtrip_stable` uses full `assert_eq!(again, original)` structural equality (parity with scope_resolved / policy_denied). |
| **L2** | verified_fixed | **CONFIRMED** | Renamed `t180_f25_api_version_constants__all_modules__are_one`; comment states F25 + P-SYNC index is docs-only §9.5. Old `t180_s_index__*` name absent from repo. |
| **L3** | verified_fixed | **CONFIRMED** | P-HTTP surface row: “Desktop Tauri invoke uses this surface (**F20**: desktop invoke ≡ P-HTTP).” |
| **L4** | verified_fixed | **CONFIRMED** | §9.5 closing note: bare Axum `404 Not Found` is the defined contract; `t180_h_unknown_route__returns_404` comment aligns and asserts status only (intentional). |

### Spot-checks (task-specific)

| Check | Result |
|-------|--------|
| PROTOCOL-COMPAT §9.5 paths | Accurate (src unit tests + kats hex + CLI replicate/dogfood) |
| F20 note | Present on P-HTTP row (§1) |
| CLI style freezes use production binary | Yes (`CARGO_BIN_EXE_ai-brains` for preflight + `scope resolve --format json --local`) |

### New findings

| Severity | Count | Notes |
|----------|------:|-------|
| critical / high / medium / low | 0 | None opened |

### Fresh regression sweep (fixes only)

- No leftover weak style tests re-calling `to_string` / `to_string_pretty` in `protocol_compat_cli.rs`.
- No stale `t180_s_index__*` symbol references.
- §9.5 path claims match on-disk modules and `tests/kats/wrap_seeded_ct.hex`.
- `--local` is a real `scope resolve` flag (`ResolveOptions.local` → `PathFlags` / `run_resolve_local` → `emit_json`); not a dead flag.
- Ingest dual-path still CLI-binary (unchanged strength).
- **Non-regression residual (informational only, not a finding):** the bare-404 contract sentence is placed at the end of §9.5 (P-SYNC) rather than under §9.2 (P-HTTP). Content is correct and test-referenced; organization is slightly awkward but does not re-open L4.
- **Round-1 residual risks #1–2 (M1 style / M2 index paths):** resolved by this fix round. Other residuals (unenforced `api_version`, Upcast stub, binary N−1, eight constants, dry-run asymmetry) unchanged and still documented.
- **Process:** Phase E closeout (CI evidence in plan, Conductor Completed, deferred §57 finalize, pin decision) still open — expected, not a product regression.

### Clearance

- **No further code/docs fixes required** for Round-1 findings.  
- Proceed with Phase E closeout when ready.  
- Round-1 “Clearance recommendation” items 1–2 are **satisfied**.
