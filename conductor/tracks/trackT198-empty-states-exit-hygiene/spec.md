# T198 — Empty States + Exit Hygiene

- **Track ID:** T198-EmptyStatesExitHygiene
- **Phase:** Post-T197 CLI UX series (P1)
- **Status:** 🚧 **Implement complete** (pending full gate / ledger commit)
- **Depends on:** **T197** Completed (PR #80); T186 hermetic CLI; T122 graph stub (**amend** exit 0 → **EXIT_USAGE=2**); T160 governed exit helpers
- **Blocks / feeds:** T200 graph install honesty (shares FEATURE_UNAVAILABLE + exit 2); T201 full exit matrix
- **Category:** FEATURE / DOCS
- **Source:** CLI audit 2026-08-02 P1 + scores &lt;7 (backup verify, project list, dogfood, graph E2, fingerprint, detect)
- **Deferred absorbed:** CLI empty states / silent fails / graph exit 0 residual
- **Not absorbed:** Full exit taxonomy (**T201**); default install enables graph (**T200**); governed discovery JSON lists (**T203**); recall/briefing/forget UX (**T202**); `daemon status` (**T199**); MSI / R-CI-BRANCH
- **Research date:** 2026-08-03 (expand + live re-scan)
- **AI fold-in:** AI1 **M1–M7**, **L1–L8**, soft **O2**; AI2 affirm dogfood/graph/verify/list/fingerprint. O3 declined; O4 → T203. Disposition §14.
- **Ledger:** plan-only (no TX until implement)

## 1. Objective

1. **Empty success ≠ blank output** for audited paths.  
2. **Feature-unavailable is not success:** default-build `graph *` exits **2** (`EXIT_USAGE`) with hint + `FEATURE_UNAVAILABLE` code string.  
3. **Missing-input errors never silent:** all dogfood `GovernedCliError::emitted` silent sites fixed (not only `read_json_file`).  
4. **Align** `device fingerprint` empty with `device list` (stdout + exit 0).  
5. Do not redesign full exit matrix (T201); do not enable graph by default (T200).

## 2. Live baseline (re-scan 2026-08-03; AI1 8/8 confirmed)

| Command | Live | Gap |
|---------|------|-----|
| `backup verify` empty | Human blank; JSON `{results:[]}` only; exit 0 | Non-blank human + status/message |
| `backup list` empty | `No backups found.` | OK reference |
| `project list` empty | Header only | Empty-state line |
| `project detect` miss | stderr + exit 1 | Soft: mention `context` |
| `dogfood compare` errors | **15+ sites** `GovernedCliError::emitted(...)` without prior `emit_error` → exit 6 **empty streams** | P0 silent class |
| `graph *` feature-off | Two `#[cfg(not(feature = "graph"))]` stubs → `Ok(())` exit **0** | Exit **2** + hint |
| `device fingerprint` empty | `ok_or(...)?` → COMMAND_FAILED exit **1** | Match list: stdout + Ok exit **0** |
| `device list` empty | println + Ok | Reference pattern |

### 2.1 Exit table (full — AI1)

| Const | Value | T198 use |
|-------|-------|----------|
| `EXIT_SUCCESS` | 0 | Empty success |
| `EXIT_INTERNAL` | 1 | Not for feature-unavailable |
| **`EXIT_USAGE`** | **2** | **Graph feature-off (F2)** — clap-style “not available this build” |
| `EXIT_INVALID_PAYLOAD` | 6 | dogfood missing/invalid inputs |
| 3–5, 7 | — | Unchanged (T201) |

## 3. Research summary

| Finding | Application |
|---------|-------------|
| Clap/ecosystem: exit 2 = usage / unavailable | Graph → **EXIT_USAGE**, not 1 |
| `emitted: true` means already printed | Only set after `emit_error` / use `fail_api` |
| T122 exit 0 discoverability | Superseded by honesty (F2) |
| evaluate hard-gate `emitted` after report | **Not** a silent bug — out of F4 scope |

## 4. Frozen decisions (F1–F28)

| ID | Decision |
|----|----------|
| **F1 — Empty success non-blank** | Audited empty-**success** paths: ≥1 human line on **stdout**. **project detect miss is failure** (exit 1, stderr) — not empty success (L2). |
| **F2 — Graph feature-unavailable (M1/M6)** | Without `--features graph`, invoking graph **actions** prints reinstall hint and exits **`EXIT_USAGE` (2)**. JSON/code string: **`FEATURE_UNAVAILABLE`**. Change **both** stubs: `main.rs` ~1809 (`run_sync_path_free`) and ~2807 (defensive duplicate). **Not** exit 1. |
| **F3 — graph --help** | Clap help remains exit **0**. |
| **F4 — dogfood silent class (M2/L7/L8)** | **All** dogfood.rs sites that return `GovernedCliError::emitted` without prior emit must convert to **`fail_api(OutputFormat::Json, ApiError::new("INVALID_PAYLOAD", …))`** (or equivalent emit-then-emitted). Inventory at implement (~15 sites including `read_json_file`, parse helpers, extractors). **Exclude** evaluate.rs hard-gate pattern (report already on stdout — L8). Exit remains **6**. Messages must include **paths** when path-related. |
| **F5 — backup verify empty (M4)** | Zero discovered backups (no usable paths): human **`No backups to verify.`** (pin). JSON via `VerifyOutput`: `results: []`, **`status: "ok"`**, **`message: Some("No backups to verify.")`** with `#[serde(skip_serializing_if = "Option::is_none")]` on message; non-empty path keeps `message: None` (no schema noise). Exit **0**. Explicit `--path` missing/fail already has body + exit 1 (L3). |
| **F6 — project list empty (M5)** | Keep header; if empty, stdout second line **`No projects registered. (0 projects)`** (pin). Exit **0**. Soft JSON later → T203 (O4). |
| **F7 — device fingerprint empty (M3)** | Restructure (not ok_or): if no local/enrolled device → **println** same copy as list (`No enrolled devices. Run \`ai-brains device bootstrap\` first.`) + **`return Ok(())`** → exit **0**. Remove Err path for empty enroll. |
| **F8 — project detect** | Keep exit 1 + stderr. Soft: mention `ai-brains context` as well as set-alias/init. |
| **F9 — Additive only** | Non-empty verify: skip_serializing_if so existing fields dominate; no drop of path/status/check/tables. |
| **F10 — No new exit numbers** | Do not invent 8+; **use existing EXIT_USAGE=2** (not a “rework” of 2–5 semantics beyond assigning feature-unavailable to 2). |
| **F11 — T122 test (L6)** | Flip `graph__default_build__prints_hint`: `!success`, **`code == Some(2)`**, hint still on stdout. Help test stays exit 0. |
| **F12 — Hermetic** | T197 no-silent-zero respected; hermetic vaults + keys. |
| **F13 — Capture independence** | Unchanged. |
| **F14 — Contracts** | CLI-local VerifyOutput fields OK. |
| **F15 — Docs** | CHANGELOG must call out **graph exit 0 → 2**. Soft OPERATIONS. |
| **F16 — Soft other empty (L1)** | Cap DoD to audit register. Note candidates deferred: `forget --match` empty via tracing; symbol-bridge empty via tracing → **T202/T203**, not T198 DoD. |
| **F17 — dogfood stream (L4)** | Errors use **OutputFormat::Json** → envelope on **stdout** (governed one-stream convention). Do not switch to Human/stderr-only. |
| **F18 — No T200 install flip** | Graph stays optional feature. |
| **F19 — Not T199/T202** | Unchanged. |
| **F20 — Review** | FEATURE; exit-code regression on graph scripts. |
| **F21 — Determinism** | Stable empty strings. |
| **F22 — High findings** | Silent dogfood (any remaining site); blank verify; graph exit 0 or wrong code 1; fingerprint still exit 1; verify JSON breaks consumers (non-additive). |
| **F23 — Series** | P1 after T197; parallel T199 OK. |
| **F24 — Shared constant (M7/O2)** | `FEATURE_UNAVAILABLE: &str`; `exit_code_feature_unavailable() -> EXIT_USAGE` (**2**). Soft: map `"FEATURE_UNAVAILABLE" => EXIT_USAGE` in `exit_code_for_api_error` for T200. |
| **F25 — F4 site inventory required** | Plan B1 lists every dogfood `::emitted` call; all fixed or justified exclude. |
| **F26 — VerifyOutput field rules** | `status` always present or only on empty — freeze: **always serialize status** for empty path; for non-empty either omit via skip if default empty string unused, or always `"ok"` when any_failed false / `"fail"` when any_failed — prefer **message only on empty-success path**; non-empty leaves message None. |
| **F27 — Graph user-facing line** | Soft: first line or prefix `FEATURE_UNAVAILABLE:` then existing reinstall hint. |
| **F28 — No count field** | Decline O3 `count: 0` on verify JSON (use `results.len`). |

## 5. Residual disposition

| Residual | Disposition |
|----------|-------------|
| Empty / silent / graph exit 0 | **Absorb** |
| Graph default install | **T200** |
| Full exit matrix | **T201** (may refine FEATURE_UNAVAILABLE numeric later) |
| forget/symbol-bridge tracing empty | **T202/T203** (L1) |
| project list JSON | **T203** soft |
| daemon status | **T199** |

## 6. Acceptance criteria

| AC | Criterion | Proof |
|----|-----------|-------|
| **AC1** | `backup verify` 0 backups: human non-blank; JSON results[] + status ok + message; exit 0 | Hermetic |
| **AC2** | `project list` empty: empty-state line on stdout; exit 0 | Hermetic |
| **AC3** | dogfood missing file: non-empty error body; exit 6; **all inventoried silent sites fixed** | Hermetic + grep inventory |
| **AC4** | `graph update` default build: exit **2**; hint present; help exit 0 | Flipped smoke test |
| **AC5** | fingerprint no enroll: stdout bootstrap next-step; exit **0** | Hermetic |
| **AC6** | Hermetic suite AC1–AC5 | nextest |
| **AC7** | Non-empty verify still per-file OK/FAIL (incl. missing --path) | Regression |
| **AC8** | CHANGELOG notes graph 0→2 | Diff |
| **AC9** | Full gate | Process |
| **AC10** | Both graph stub sites exit 2 | Code review |
| **AC11** | FEATURE_UNAVAILABLE helper returns 2; soft api_error map | Unit |

## 7. Non-goals

- Full exit taxonomy (T201)  
- Default-on graph install (T200)  
- Governed discovery / project list JSON (T203)  
- forget/symbol-bridge tracing empty as DoD  
- evaluate hard-gate emit pattern rewrite  
- MSI / R-CI-BRANCH  
- Verify JSON `count` field  

## 8. Handoffs

| To | What |
|----|------|
| deferred empty/silent/graph0 | Strike on ship |
| T122 AC2 exit 0 | **Superseded** by F2 |
| T200 | FEATURE_UNAVAILABLE + exit 2; install policy |
| T201 | May refine codes |
| T202/T203 | tracing empty lists (L1) |

## 9. Implementation sketch

### 9.1 dogfood (all silent sites)

```text
// replace GovernedCliError::emitted(6, msg) without prior emit:
fail_api(OutputFormat::Json, ApiError::new("INVALID_PAYLOAD", msg_with_path))
```

### 9.2 backup verify empty

```text
if paths.is_empty() {
  if json { VerifyOutput { results: vec![], status: "ok".into(), message: Some("No backups to verify.".into()) } }
  else { println!("No backups to verify."); }
  return Ok(());
}
```

### 9.3 graph stub (both sites)

```text
println!("FEATURE_UNAVAILABLE: The graph subcommand requires...");
println!("Reinstall with: cargo install ... --features graph");
std::process::exit(EXIT_USAGE); // 2
```

### 9.4 fingerprint

```text
if devices.is_empty() { /* same as list */ println!(...); return Ok(()); }
// find local ...
```

## 10. Verification plan

1. Dogfood inventory grep zero remaining silent-emitted (dogfood only).  
2. Hermetic AC1–AC5.  
3. Graph test code==2.  
4. Verify regression.  
5. CHANGELOG + full gate.  

## 11. Risks

| Risk | Mitigation |
|------|------------|
| Scripts assumed graph exit 0 | CHANGELOG callout |
| Partial dogfood fix | F25 full inventory |
| Exit 1 vs 2 confusion | F2 freeze 2 only |
| Verify JSON field noise | skip_serializing_if on message |

## 12. Implement notes

1. **Order:** dogfood F4 inventory fix → backup verify → project list → graph both stubs + test exit 2 → fingerprint restructure → soft detect → CHANGELOG.  
2. **High findings:** F22.  
3. **Stop-before:** T200 install; T201 matrix; evaluate rewrite.  
4. **Category:** FEATURE.  

## 13. Research pins (post fold-in)

| Fact | Pin |
|------|-----|
| Graph exit | **2** EXIT_USAGE |
| FEATURE_UNAVAILABLE | string + helper → 2 |
| dogfood silent | **all dogfood emitted sites** via fail_api |
| evaluate hard-gate | exclude from F4 |
| fingerprint | Ok(()) + stdout |
| VerifyOutput | status + optional message |
| Graph stubs | **both** cfg sites |

## 14. AI fold-in disposition (2026-08-03)

### AI1 required

| ID | Disposition | Fold-in |
|----|-------------|---------|
| **M1** EXIT_USAGE=2 for graph | **Agree** | F2 (reject AI2 “exit 1”) |
| **M2** 15 dogfood silent sites | **Agree** | F4/F25 full inventory |
| **M3** fingerprint Ok restructure | **Agree** | F7 |
| **M4** VerifyOutput status/message | **Agree** | F5/F26 |
| **M5** project list second line | **Agree** | F6 (already) |
| **M6** both graph stubs | **Agree** | F2/AC10 |
| **M7** helper returns 2 | **Agree** | F24 |

### AI1 low

| ID | Disposition |
|----|-------------|
| **L1** forget/symbol-bridge tracing empty | Defer T202/T203; F16 |
| **L2** detect stays stderr | Agree F1/F8 |
| **L3** --path missing already OK | Agree |
| **L4** dogfood JSON errors stdout | Agree F17 |
| **L5** additive verify fields | Agree |
| **L6** test assert code 2 | Agree F11 |
| **L7** fail_api pattern | Agree F4 |
| **L8** exclude evaluate hard-gate | Agree F4 |

### Opportunities

| ID | Disposition |
|----|-------------|
| **O1** custom graph help | No change |
| **O2** exit_code_for_api_error map | Soft accept F24 |
| **O3** count field | **Declined** F28 |
| **O4** project list JSON | **T203** |

### AI2

| Item | Disposition |
|------|-------------|
| dogfood silent root cause | Agree (subset of M2) |
| graph non-zero | Agree; **exit 2** not 1 |
| verify / list / fingerprint empty | Agree |

### Declined

| Item | Why |
|------|-----|
| Graph exit 1 (AI2 summary) | EXIT_USAGE=2 better for scripts |
| count:0 on verify JSON | Marginal; results.len |
| Expanding DoD to all tracing empties | Scope cap F16 |
)
