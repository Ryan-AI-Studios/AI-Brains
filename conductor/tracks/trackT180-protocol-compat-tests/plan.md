# T180 Plan — Protocol Compatibility Tests (P12.2)

Status: **In Progress** → implement complete pending review/CI.  
Spec: [spec.md](./spec.md) (F1–F38, AC1–AC12).  
Policy: [Docs/PROTOCOL-COMPAT.md](../../../Docs/PROTOCOL-COMPAT.md).  
Ledger: `87a23c0c-2092-458c-b7ed-1e51d1c67ee0`.

## Preconditions

- [x] Read T158 `protocol_wire.rs` (~527 lines) + `tests/fixtures/*`  
- [x] Read `BridgePayload` + `parse_live_request_line` raw fallback  
- [x] Read event R0 hand-serde + `upcast.rs` stub  
- [x] Confirm 8× `API_VERSION` and **no** runtime validation  
- [x] `ledgerful doctor` + `scan --impact` before edits  

## License gate

- [x] No untrusted binary downloads  
- [x] No AGPL fuzz  
- [x] Zero new production deps  
- [x] Prefer no new dev-deps; if insta/proptest: pin `1.48` / `1.11`  

---

## Phase A — Inventory + policy (docs-first)

- [x] **A1** Write `Docs/PROTOCOL-COMPAT.md`  
- [x] **A2** Inventory: `deny_unknown_fields` (dry-run only); compact vs pretty per CLI command; doctor absent; `--version` text  
- [x] **A3** Map every relevant existing test → `T180-*` id table (elevate-first F28)  
- [x] **A4** Labels: wire-gen-0 = existing legacy fixtures; wire-gen-1 = current freezes  

## Phase B — Daemon / Bridge (elevate + gap-fill)

- [x] **B1** Map T158 suite to `T180-D-*` (module docs + PROTOCOL-COMPAT §9) — do **not** rewrite  
- [x] **B2** Confirm unknown-type test remains the `#[serde(other)]` regression guard  
- [x] **B3** **New:** `assert_deserializes_with_extra_fields` + apply to public wire DTOs  
- [x] **B4** **New:** file-backed golden `governed_resolve_scope_request.json`  
- [x] **B5** Elevate Bridge unknown + raw BridgeRecord fallback into index  
- [x] **B6** F37: fixture JSON vs current serialize drift for key goldens  

## Phase C — HTTP + CLI + EVENT

- [x] **C1** HTTP DTO goldens under `ai-brains-api-server/tests/fixtures/`  
- [x] **C2** `api_version: "1"` accepted + **`api_version: "2"` accepted** honesty (F25)  
- [x] **C3** Assert `api_version` field presence on serialised response DTOs that claim it (F33)  
- [x] **C4** CLI: freeze keys + style for preflight and scope resolve  
- [x] **C5** CLI stdin: dry-run deny_unknown vs production open (F26)  
- [x] **C6** Event R0 elevate; document Upcast stub (no fake migration test)  
- [x] **C7** Soft F34: optional schema check deferred (not DoD)  

## Phase D — Sync index + handoffs

- [x] **D1** P-SYNC index → T176/T178; note wire OS-agnostic / DPAPI not wire (F38)  
- [x] **D2** Default nextest &lt;60s (protocol_compat suites &lt;1s)  
- [x] **D3** Residuals: binary N−1; api_version enforcement; single API_VERSION SOOT  
- [x] **D4** Handoff T183/T185: unenforced version, Upcast stub, Bridge capture policy  

## Phase E — Closeout

- [x] **E1** Full gate: fmt, clippy -D warnings, nextest 1693 pass, deny, audit (allowed warnings only)  
- [x] **E2** Targeted nextest: protocol_compat + protocol_wire + CLI (66+ daemon/http/events; 4 CLI)  
- [x] **E3** Internal R1→fix→R2 **PASS** (0 open findings)  
- [x] **E3b** Codex cross-model clean final gate — R1 **PASS** (0 findings); `review.codex.r1.md`  
- [x] **E4** deferred §57 Completed  
- [x] **E5** Pin after PR green (or at merge)  

---

## Manual evidence

| Check | Command / result |
|-------|------------------|
| Daemon/API/Events protocol_compat + wire | `cargo nextest run -p ai-brains-daemon-api -p ai-brains-api-server -p ai-brains-events --test protocol_compat --test protocol_compat_events --test protocol_wire` → **66/66 pass** |
| CLI dual-path + style (binary) | `cargo nextest run -p ai-brains-cli --test protocol_compat_cli` → **4/4 pass** (preflight compact, scope pretty, dry-run deny, prod open) |
| Full nextest | `cargo nextest run --workspace` → **1693 passed**, 1 skipped (~57s) |
| Clippy | `cargo clippy --workspace --all-targets -- -D warnings` → pass |
| Deny / audit | `cargo deny check` + `cargo audit` → exit 0 (pre-existing allowed audit warnings) |
| Internal review | R1 PASS WITH DEFERRED (M1/M2); R2 **PASS** zero open |

---

## Out of scope checklist

- [x] Runtime api_version enforcement as DoD  
- [x] Upcast v0→v1 implementation as DoD  
- [x] Root `fixtures/protocol/` tree  
- [x] jsonschema required dep  
- [x] Re-implement T178 crypto  
- [x] doctor / --version JSON  

## Residual log

| Item | Severity | Owner |
|------|----------|-------|
| Runtime api_version enforcement | residual F36 | future track |
| Single API_VERSION SOOT | soft F35 | residual |
| Binary N−1 CI | F24 | post-release |
| Docs/schemas jsonschema gate | soft F34 | residual |
