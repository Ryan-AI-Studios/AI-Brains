# T201 — CLI Error Envelope + Exit Code Contract

- **Track ID:** T201-CliErrorExitContract
- **Phase:** Post-T200 CLI UX series (P2)
- **Status:** 📋 **Expanded + AI fold-in / Pending** (plan-only; implement on go)
- **Depends on:** T160 governed exit surface; T192 doctor exits; **T197** vault key codes; **T198** FEATURE_UNAVAILABLE→2; T200 install honesty
- **Blocks / feeds:** Operator/script reliability; **T203** review soft-default must not reintroduce exit-6 missing-scope class; T204 help may link exit-code doc
- **Category:** FEATURE / DOCS / CONTRACTS (light)
- **Source:** CLI audit 2026-08-02 P2 — clap exit **2** vs app **6** for missing-scope; POLICY_DENIED without remediation; mixed envelopes
- **Deferred absorbed:** Exit residual; T197–T200 handoffs; OPERATIONS incomplete exit list
- **Not absorbed:** Policy engine redesign; grant admin UX; force single envelope; i18n; MSI; T202; T203 discovery lists (boundary F27)
- **Research date:** 2026-08-03 (expand + live re-scan)
- **AI fold-in:** AI1 affirms F3/F4/F6/F8/F1. AI2 **M1–M7** accepted; **L1/L2/L4/L7/L9** notes; **O1** soft deferred; **O3** absorbed into F8. Disposition §14.
- **Ledger:** plan-only (no TX until implement)

## 1. Objective

1. Publish **normative exit-code table** (`Docs/CLI-EXIT-CODES.md` + CONTRIBUTING/CAPABILITIES/OPERATIONS links).  
2. Align **missing required `--scope` (and peers)** to **exit 2** (USAGE / clap) — not exit 6.  
3. Document **dual error envelopes** + format-dependent streams; fix live “always stderr” docs bugs.  
4. **POLICY_DENIED** carries **structured** `details.hint` (preferred).  
5. Hermetic suite locks exits (including T198 graph exit 2). Document F4 as **breaking** where envelope flips.

## 2. Live baseline (re-scan 2026-08-03; AI2 7/7 + audit confirmed)

### 2.1 Exit constants (T160)

| Const | Value | Live use |
|-------|-------|----------|
| `EXIT_SUCCESS` | 0 | Success; empty-success; daemon status report; doctor ok\|degraded (unless flag) |
| `EXIT_INTERNAL` | 1 | Vault/key; PATH_REFUSED; COMMAND_FAILED; catch-all map; doctor fail |
| `EXIT_USAGE` | 2 | Clap; FEATURE_UNAVAILABLE; graph feature-off |
| `EXIT_POLICY_DENIED` | 3 | POLICY_DENIED, APPROVAL_REQUIRED |
| `EXIT_NOT_FOUND` | 4 | NOT_FOUND |
| `EXIT_DAEMON_UNAVAILABLE` | 5 | Daemon required |
| `EXIT_INVALID_PAYLOAD` | 6 | INVALID_PAYLOAD, NOT_ENVELOPE_BACKED; **today** missing-scope on show/list/erasure-request path |
| `EXIT_HARD_GATE_FAILED` | 7 | Evaluate hard gates |

### 2.2 Missing-scope inventory (F4 expand — M6)

| Path | CLI today | Exit today | Daemon / note |
|------|-----------|------------|---------------|
| `policy check` | clap `scope: String` | **2** | N/A |
| `policy show` | `Option` | **6** Json ApiError stdout | — |
| `review list` | `Option` | **6** | services `list_review_items` None → INVALID_PAYLOAD (**keep** for non-CLI — M1) |
| `erasure request` | `Option` | **6** class via daemon | services requires scope (M6) |
| `erasure wipe` | clap `String` | **2** | Already aligned |

Implementer **must grep** all `scope: Option` in CLI Commands and audit against F3.

### 2.3 Dual envelopes + streams

| Path | Shape | Stream |
|------|-------|--------|
| Governed Json (`fail_api`) | bare `ApiError` | **stdout** |
| Governed Human/Markdown | `CODE: message` | **stderr** |
| Generic `handle_cli_result` | full `ApiResult` | **stderr** always |

Stale docs (required fix): **CAPABILITIES** “on stderr”; **OPERATIONS** same class claims + incomplete exit list (M4/O3).

### 2.4 Routing map

| File | Role |
|------|------|
| `governed_common.rs` | Exit SOOT; map; fail_api |
| `policy_cmd.rs` | show clap-required; deny + details.hint |
| `review.rs` | list clap-required (CLI); remove None fail_api |
| `erasure.rs` / main clap | request scope clap-required |
| `ai-brainsd/src/services.rs` | **Keep** defensive None arms (M1) |
| `main.rs` | handle_cli_result vault path (exit 1 hardcoded) |
| `Docs/CLI-EXIT-CODES.md` | New SOOT |
| CAPABILITIES / OPERATIONS / CONTRIBUTING / CHANGELOG | Honesty + links + breaking |
| Tests | New `exit_contract` suite preferred + smoke regression |

## 3. Research summary

| Finding | Application |
|---------|-------------|
| clap MissingRequiredArgument → exit 2 | F4 preferred = zero custom exit logic |
| Dual envelope force | High agent risk — document dual; F4 flip is **documented breaking** only for missing-scope edge |
| sysexits 64 | Not adopted; stay 0–7 |
| ApiError.details | Prefer structured hint (M2) |
| clap 4.5 / 4.6 | No bump |

## 4. Frozen decisions (F1–F36)

| ID | Decision |
|----|----------|
| **F1 — Normative table** | `Docs/CLI-EXIT-CODES.md`: 0–7 + FEATURE_UNAVAILABLE→2; doctor footnote (F14); status exit 0; **130 OS footnote** (not in 0–7 table); vault codes via handle_cli_result exception (L4). |
| **F2 — No new product exits** | No 8+ product codes. 130 pre-existing only. |
| **F3 — Missing-required = USAGE (2)** | Forgot always-required flags → exit **2**, not 6. Malformed provided values still 6/CP. |
| **F4 — Clap-required inventory (M1/M6/M7)** | **Preferred (freeze):** clap-required `--scope: String` on **policy show**, **review list**, and **erasure request** (and any other F3 sites found by grep). **Daemon** None→INVALID_PAYLOAD arms **retained** for HTTP/pipe (M1). **Breaking:** missing-scope flips from Json ApiError stdout exit 6 → clap English stderr exit 2 — **CHANGELOG BREAKING required** (M7); not a “silent” flip. **Alt** USAGE map only if clap-required blocked; record at implement. |
| **F5 — INVALID_PAYLOAD stays 6** | Malformed ids, unknown capability, dogfood, bad JSON, etc. |
| **F6 — POLICY_DENIED structured hint (M2)** | Prefer **`details.hint`** non-empty string via `ApiError::with_details(json!({"hint":…}))`. Message stays terse (`{cap} denied for principal {p} on {scope}`). Exit **3**. AC asserts `details.hint` present on Json. Soft: Human mode may append hint to message if details not shown. Soft: APPROVAL_REQUIRED same pattern if free. |
| **F7 — Vault codes** | Exit **1**; document handle_cli_result bypass of `exit_code_for_api_error` (L4). |
| **F8 — Envelope + stream honesty (M4/L2/O3)** | Doc dual shapes. **Required edits:** CAPABILITIES governed-envelope claims; OPERATIONS envelope + incomplete exit list (link CLI-EXIT-CODES). Grep Docs for “on stderr” governed claims. State: Json→stdout bare; Human→stderr CODE; generic→stderr ApiResult. |
| **F9 — Converge soft** | Do not force single envelope in T201. |
| **F10 — FEATURE_UNAVAILABLE** | Exit 2; human stubs OK; soft Json later. |
| **F11 — fail_api Json stdout** | Unchanged. |
| **F12 — Human CODE: message** | stderr. |
| **F13 — Contracts** | No new DTO field; use `details`. |
| **F14 — Doctor (M5)** | Doc: 0 ok\|degraded; **`--fail-on-degraded` promotes degraded→1**; fail→1; clap 2. |
| **F15 — Daemon status** | Exit 0 Running\|Stopped. |
| **F16 — Capture independence** | Unchanged. |
| **F17 — Zero new deps** | No clap bump. |
| **F18 — Hermetic locks (M3)** | **≥6 locks:** (1) missing-scope class exit **2** (policy show); (2) missing-scope exit **2** (review list); (3) POLICY_DENIED exit 3 + **details.hint**; (4a) **graph feature-off exit 2** + FEATURE_UNAVAILABLE; (4b) vault key missing/wrong exit 1 + VAULT_KEY_* / locked; (5) success exit 0 sample; (6) INVALID_PAYLOAD exit 6 sample (e.g. unknown capability). Prefer `tests/exit_contract.rs` (L8 soft). Graph lock may invoke existing smoke or re-assert in suite — **must run under default no-graph CI**. |
| **F19 — High findings** | Missing-scope still 6; deny without structured hint; stderr docs bug remains; silent envelope flip without CHANGELOG; invent 8+; T198 exit 2 regress; erase F4 sites without grep inventory. **Note:** F4 clap flip is **accepted documented breaking**, not silent. |
| **F20 — Series** | After T200; parallel T202 OK; **coordinate review.rs with T203** (F27). |
| **F21 — Determinism** | Stable codes; stable hint template. |
| **F22 — Review (L9)** | FEATURE; **cross-model required** when F4 clap flip lands (envelope change for agents). |
| **F23 — Clap English** | OK for missing required after F4; no clap→Json parser DoD. |
| **F24 — USAGE map arm** | Only if F4 alt; conditional in plan. |
| **F25 — APPROVAL_REQUIRED** | Exit 3; soft hint parity. |
| **F26 — Exit 130** | OS footnote in CLI-EXIT-CODES only. |
| **F27 — T203 boundary (conflict flag)** | T201 makes review list `--scope` **required** for class honesty. T203 soft-default discovery **must not** reintroduce exit-6 missing-scope. Allowed T203 patterns: additive `--from-context` / detect that **fills** scope before run; or explicit later track reopening Option with USAGE exit 2 only. Flag in plan stop-before if T203 ships concurrent with conflicting Option. |
| **F28 — INSTALL/claims** | No F8 encryption claim change. |
| **F29 — Hint pin (F6)** | Exact template freeze at implement, e.g. `ensure a grant for {cap} exists for this principal on this scope; use \`ai-brains policy show --scope …\``. |
| **F30 — INVALID_TRANSITION (L7)** | Soft audit: add explicit map arm (prefer **EXIT_INTERNAL=1** state machine, or 6 if treated as client error). Document in CLI-EXIT-CODES. Not silent catch-all forever for known CP codes. |
| **F31 — Map completeness soft** | Soft unit: known `api_error_from_cp` codes have explicit arms (L1). |
| **F32 — AI1 affirm** | clap-required preferred; deny hint; dual envelope doc; 0–7 table — all above. |
| **F33 — No --exit-codes flag DoD** | O1 deferred. |
| **F34 — CHANGELOG AC** | Must include **BREAKING** missing-scope class 6→2 for listed commands. |
| **F35 — Daemon doc note** | CLI-EXIT-CODES: raw IPC/HTTP missing scope may still return INVALID_PAYLOAD (6); CLI always sends scope after F4. |
| **F36 — Grep inventory DoD** | Plan B0: list every CLI `scope: Option` + disposition (clap-required / leave / T203). |

## 5. Residual disposition

| Residual | Disposition |
|----------|-------------|
| Exit + envelope residual | **Absorb** |
| clap vs 6 missing-scope | **Absorb** F3/F4 |
| POLICY_DENIED no hint | **Absorb** F6 structured |
| stderr docs bug | **Absorb** F8 |
| Graph exit 0 | **Closed T198** — F18 4a lock |
| Single envelope force | Soft F9 decline as DoD |
| T203 soft-default | **Boundary F27** |
| `--exit-codes` flag | Soft deferred |
| Help footer link | Soft **T204** |

## 6. Acceptance criteria

| AC | Criterion | Proof |
|----|-----------|-------|
| **AC1** | CLI-EXIT-CODES.md: 0–7 + FEATURE_UNAVAILABLE + doctor `--fail-on-degraded` + status 0 + 130 footnote + vault exception + daemon IPC note | Diff |
| **AC2** | `policy show` missing `--scope` → exit **2** | Hermetic |
| **AC3** | `review list` missing `--scope` → exit **2** | Hermetic |
| **AC3b** | `erasure request` missing `--scope` → exit **2** | Hermetic |
| **AC4** | POLICY_DENIED Json: `details.hint` non-empty string; exit 3 | Hermetic |
| **AC5** | F18 suite ≥6 locks incl. 4a graph exit 2 | nextest |
| **AC6a** | CAPABILITIES + OPERATIONS stream honesty (no false “always stderr” for governed Json) | Diff + Docs grep |
| **AC6b** | CONTRIBUTING links CLI-EXIT-CODES | Diff |
| **AC7** | CHANGELOG **BREAKING** missing-scope 6→2 | Diff |
| **AC8** | Graph exit 2 green in exit-contract suite and/or smoke; no exit 8+ | Tests |
| **AC9** | Full gate | Process |
| **AC10** | Dual envelope + format-dependent streams documented | Doc review |
| **AC11** | Clap help shows scope required on flipped commands | Hermetic `--help` |
| **AC12** | Daemon None arm retained (M1); F36 inventory recorded in plan notes | Review |
| **AC13** | Cross-model review after F4 (F22) | Process |

## 7. Non-goals

- Policy engine / grant admin redesign  
- Clap JSON error parser  
- Force single envelope  
- Exit 8+  
- Removing daemon defensive None arms  
- T203 soft-default implementation  
- `--exit-codes` CLI flag  
- clap 4.6 bump  

## 8. Handoffs

| To | What |
|----|------|
| deferred | Strike exit residual on ship |
| T198/T200 | FEATURE_UNAVAILABLE→2 ratified + hermetic lock |
| T203 | F27: no exit-6 reintroduction; coordinate review list |
| T204 | Soft help link |
| HTTP/desktop | details.hint pattern optional reuse |

## 9. Implementation sketch

### 9.1 Clap-required (preferred)

```rust
// main.rs / clap structs: policy show, review list, erasure request
#[arg(long)]
scope: String, // was Option<String>
// delete runtime None → fail_api(INVALID_PAYLOAD) branches in CLI only
// daemon services.rs: keep None → INVALID_PAYLOAD
```

### 9.2 POLICY_DENIED hint

```rust
fail_api(
  format,
  ApiError::new(
    "POLICY_DENIED",
    format!("{cap} denied for principal {p} on {scope}"),
  ).with_details(serde_json::json!({
    "hint": "ensure a grant for this capability exists for this principal on this scope; try `ai-brains policy show --scope …`"
  })),
)
```

### 9.3 CHANGELOG breaking (required)

```markdown
### Breaking
- Missing `--scope` on `policy show`, `review list`, and `erasure request` now exits **2** (clap usage on stderr)
  instead of **6** with JSON `INVALID_PAYLOAD` on stdout. Pass `--scope` explicitly.
```

## 10. Verification plan

| Layer | What |
|-------|------|
| Inventory | F36 scope Option grep |
| Hermetic | AC2–AC5, AC3b, AC8, AC11 |
| Docs | AC1, AC6a/b, AC10 |
| Process | AC7, AC9, AC12, AC13 |
| Regression | dogfood 6; T198 graph 2 |

## 11. Stop-before

- Removing daemon None arms  
- Silent envelope flip without CHANGELOG BREAKING  
- Soft-defaulting review scope under T201 DoD (T203)  
- Renumbering 3/5/6  
- Concurrent T203 that reopens Option to exit 6  

## 12. Suggested implement order

1. F36 inventory + CLI-EXIT-CODES draft.  
2. F4 clap-required flips + remove CLI None branches.  
3. F6 details.hint on deny.  
4. F8 docs stream fixes (CAPABILITIES/OPERATIONS).  
5. exit_contract hermetic suite (F18).  
6. CHANGELOG BREAKING + CONTRIBUTING link.  
7. Cross-model + full gate.

## 14. AI fold-in disposition (2026-08-03)

| ID | Source | Disposition |
|----|--------|-------------|
| AI1 §1–4 | clap-required, deny hint, dual envelope, 0–7 | **Accept** |
| **M1** | Daemon None arm retained + doc | **Accept** → F4, F35, AC12 |
| **M2** | Prefer details.hint; assert in AC4 | **Accept** → F6 |
| **M3** | F18 4a/4b separate; ≥6 locks | **Accept** → F18 |
| **M4** | CAPABILITIES + OPERATIONS stderr bugs required | **Accept** → F8, AC6a |
| **M5** | doctor --fail-on-degraded footnote | **Accept** → F14, AC1 |
| **M6** | erasure request + grep inventory | **Accept** → F4, F36, AC3b |
| **M7** | F4 breaking CHANGELOG | **Accept** → F19, F34, AC7 |
| L1/L7 | Map catch-all / INVALID_TRANSITION | Soft F30–F31 |
| L2 | Format-dependent streams | **Accept** → F8 |
| L4 | Vault bypass map | **Accept** → F7 |
| L8 | exit_contract.rs | Soft prefer |
| L9 | Cross-model after F4 | **Accept** → F22, AC13 |
| O1 | --exit-codes flag | Decline DoD F33 |
| O2 | clap ErrorKind assert | Soft |
| O3 | OPERATIONS incomplete exits | **Accept** → F8 |
| O4 | ApiResult status fields | Affirm dual OK |
| T203 tension | F4 vs soft-default | **Accept flag** → F27 |

**Baseline:** AI2 fully confirmed. **Verdict target:** M1–M7 folded; preferred path clap-required + structured deny hint + docs honesty + breaking note.
)
