# T221 — Governed first-run + deny exit honesty

- **Track ID:** T221-GovernedFirstRunDenyExit
- **Phase:** Post-audit CLI quality series (T217–T232) — **P1 honesty** after T220
- **Status:** 📋 **Planning** (plan-only until **go**)
- **Depends on:** T210 `policy bootstrap` ✅; T203 discovery lists + soft-resolve ✅; T202 progressive fail_usage exit 2 ✅; T201 exit contract + `details.hint` ✅; T152 progressive soft-packet shape ✅
- **Blocks / feeds:** Agents/scripts that treat progressive empty as “no knowledge”; residual T226 policy show/check soft-scope; residual T227 briefing format
- **Category:** UX / CONTRACT / BUGFIX
- **Source:** Non-destructive CLI audit 2026-08-05 — governed usefulness **4–5** · quality **5–6** (honest walls, dead product); progressive `denied:true` **exit 0**
- **Deferred absorbed:** deferred.md “Governed usefulness / progressive deny exit 0 → T221”; series README T221 row; T210 residual skill one-liner (if free); placeholder draft F1–F5
- **Not absorbed:** Auto-grant on `init`; full grant admin/revoke (T210 F24–F26); flip **briefing** soft deny to exit 3 (T210 F28 keep); T226 show/check soft-scope; clap 5; MSI; interactive first-run prompt; change `DefaultPolicyEvaluator`
- **Research date:** 2026-08-09 (live dogfood + code map + clig.dev / clap pins / exit-code SOOT)
- **AI fold-in:** 2026-08-09 — AI1 affirms F1–F10 core + research; **M1–M8** accepted (M1 AC3 principal; M3 F17 hard; M5 fail_cp; M4 doctor matrix if F12; M6 dry-run; M7 CODE+hint); **M2/M9/M10** soft/residual. AI2 architecture affirm. Disposition **§14**.
- **Ledger:** plan-only until go (`ledgerful ledger start` on go)

## 1. Objective

1. **Stop the progressive lie:** `query progressive` with policy deny must **not** exit **0** with empty `results` so agents conclude “no knowledge.”  
2. **Align deny exit class** with lists/check: process exit **3** (`EXIT_POLICY_DENIED`) when the progressive packet is a hard policy wall (`denied: true`).  
3. **Surface bootstrap next-step** on progressive deny (and improve human-format deny paths that currently drop `details.hint`).  
4. **First-run reachability:** operator can go vault-open → bootstrap → discovery in one documented path; optional doctor warn when discovery grants empty.  
5. **Preserve T152 packet shape** for tools that parse `ProgressiveQueryResponse.denied` (do **not** replace stdout with bare `ApiError` by default).  
6. **Capture independence:** no models/embeddings/graph required.  
7. **Least privilege:** no auto-grant, no dangerous caps, no interactive yes-prompt.

## 2. Live baseline (re-scan 2026-08-09)

### 2.1 Dogfood (authoritative project context, **no** discovery grants)

| Command | Observed | Exit |
|---------|----------|------|
| `query progressive "why graph"` | Pretty `ProgressiveQueryResponse` with `denied: true`, `results: []`, `denial_reason: "ReadConclusions/ReadDecisions denied"` | **0** ← lie |
| `source list --format json` | `ApiError` `POLICY_DENIED` + `details.hint` bootstrap | **3** OK |
| `policy check --capability ReadDecisions --scope Repository:…` | same hint class | **3** OK |
| `briefing project --format json` | Soft packet `denied: true` + `warnings[].kind=denied` | **0** (T210 F28 keep) |
| `query expand <unknown-uuid>` | `kind: "Unknown"`, empty preview | **0** (not-found class) |
| `policy bootstrap --help` | Discovery-only grants; soft-resolve scope; dry-run | OK (T210) |
| `policy show` without `--scope` | clap required → exit **2** | T226 residual |
| `doctor` | No `policy_grants` / bootstrap check | gap |

### 2.2 Root cause (frozen)

```text
// control-plane query.rs progressive_query:
if !can_read {
    return Ok(ProgressiveQueryResponse { denied: true, results: [], ... });
}
// CLI governed_query.rs run_progressive:
let resp = progressive_query(...)?;
println!("{}", serde_json::to_string_pretty(&resp)?);  // always Ok → exit 0
Ok(())
```

CP soft-packet is intentional (T152). **CLI never maps `denied` → exit 3.** Lists use `fail_api` → exit 3. Progressive is the outlier agents misread.

### 2.3 Human deny drops hint

`emit_error(Human)` prints only `CODE: message` and **ignores** `details.hint`. JSON list denials already carry bootstrap; human path does not. Placeholder F3.

### 2.4 Expand deny vs unknown

`expand_handle` returns `kind: "Denied"` (empty preview) on policy/scope wall, or other kinds when found. CLI always exit 0. **Unknown handle ≠ policy deny** — do not force exit 3 on Unknown.

### 2.5 Touch map

| Site | Role |
|------|------|
| `ai-brains-cli/src/commands/governed_query.rs` | After progressive Ok: if `denied` → emit packet + exit **3**; expand: if `kind == "Denied"` → exit **3** |
| `ai-brains-cli/src/commands/governed_common.rs` | `POLICY_DENIED_HINT`; `emit_error` human include hint; maybe `fail_policy_packet` helper |
| `ai-brains-cli/src/commands/doctor.rs` | Soft optional `policy_grants` warn when vault open + authoritative scope + zero discovery caps |
| `ai-brains-control-plane/src/query.rs` | Keep `Ok(denied packet)`; on deny set **`denial_hint`** = bootstrap template (F17 hard); success path `None` |
| `ai-brains-contracts/src/briefings.rs` | Additive `denial_hint: Option<String>` (`default`, `skip_serializing_if`); update `new()` + progressive golden / protocol_compat |
| `ai-brains-cli` progressive/expand CP `?` | Map via **`fail_cp`** so `PolicyDenied` → exit **3** not generic 1 (F33) |
| Hermetic tests | New or extend suite; **AC3** bootstrap System principal only (no `bbbb…` `--principal-id` trap — F31) |
| Docs | CLI-EXIT-CODES, OPERATIONS empty-vs-deny + Denied semantics (M8), CAPABILITIES, CHANGELOG, skill one-liner |

### 2.6 Deps

| Item | Pin / note |
|------|------------|
| clap | Workspace **4.5** (resolved **4.6.1**); crates.io latest **4.6.6** (2026-08) — **no bump** |
| serde_json | Workspace **1.0** — no bump |
| Zero new crates | Required |
| Capture independence | Exit mapping + hint strings only |

## 3. Research summary (2026-08-09)

| Finding | Application |
|---------|-------------|
| [clig.dev](https://clig.dev/) — return zero only on success; non-zero on failure; map important failure modes | Progressive deny is failure for scripts/agents → exit **3** |
| clig — suggest next command on error; first-run setup then real work | Bootstrap one-liner on deny; doctor warn optional |
| clig — stdout primary output; stderr messaging | Keep progressive **packet on stdout**; remediation on **stderr** (and human `emit_error`) |
| HTTP 403 analogy (authz denied ≠ empty 200) | Exit 3 with structured body is honest; pure exit 0 empty is the lie |
| T201 / CLI-EXIT-CODES | `POLICY_DENIED` → **3** already normative for lists/check |
| T210 F28 | list/show hard deny exit 3; **briefing soft exit 0** — do not flip briefing in T221 |
| T152 ProgressiveQueryResponse | Preserve `denied`/`denial_reason` fields for parsers |
| T202 | Missing project id already exit **2** — keep |
| Dual-consumer CLI errors (2026) | Machine exit code + human next-step; both audiences |
| AI1 M3 stdout-only agents | Track audience often captures **stdout only** → in-band `denial_hint` **hard** (F17) |
| AI1 M1 principal trap | Progressive uses `cli_principal()`; bootstrap hermetics often pass Human `--principal-id` → AC3 must align principals (F31) |

## 4. Frozen decisions (F1–F36)

| ID | Decision |
|----|----------|
| **F1 — Progressive exit (hard)** | When `ProgressiveQueryResponse.denied == true`, CLI process exits **`EXIT_POLICY_DENIED` (3)**. Never exit **0** on policy wall (includes **`--dry-run`** deny — F34). |
| **F2 — Packet preserved (hard)** | On progressive deny, **still print** pretty `ProgressiveQueryResponse` on **stdout** (same shape as today + F17 field when denied). Do **not** replace with bare `ApiError` as DoD. |
| **F3 — Emit then exit (hard)** | Pattern: fill `denial_hint` if needed → `emit_json(&resp)?` → stderr CODE+hint (F4) → `Err(GovernedCliError::emitted(EXIT_POLICY_DENIED, …))` with `emitted: true`. |
| **F4 — Progressive stderr CODE then hint (hard, M7)** | On progressive deny (and expand Denied): stderr emits **`POLICY_DENIED: …`** line **then** `POLICY_DENIED_HINT` (same dual-line shape as F5 after human `emit_error`). Not hint-only. |
| **F5 — Human list/check deny hint (hard)** | `emit_error(Human\|Markdown)`: if `details.hint` is a non-empty string, print it on stderr after the `CODE: message` line. JSON path already serializes full `ApiError`. F5 blast radius: only deny sites attach `details.hint` today. |
| **F6 — Expand Denied (hard)** | When expand preview `kind` equals **`Denied`** (exact CP string), CLI exit **3** + F4 stderr. **`Unknown` / found kinds stay exit 0.** |
| **F7 — Briefing keep soft (hard non-goal)** | Do **not** change briefing project/personal deny to exit 3. T210 F28 + OPERATIONS stay. Document progressive vs briefing difference. |
| **F8 — CP soft-packet path (hard)** | Do **not** change `progressive_query` success path to `Err(PolicyDenied)` for the normal deny-by-capability case. Keep `Ok(denied packet)`. Still fill **F17** on that packet. |
| **F9 — No interactive prompt (hard)** | Decline AskOnce / TTY grant prompt. Non-interactive only. |
| **F10 — No auto-init grant (hard)** | `init` does not bootstrap. Explicit `policy bootstrap` remains. |
| **F11 — No policy matrix change (hard)** | No `DefaultPolicyEvaluator` / grant semantics change. |
| **F12 — Doctor policy_grants (soft DoD, M4)** | If free: when vault open + authoritative project scope resolved, probe discovery grants for CLI principal; if **zero** of three Read* caps → **warn** `policy_grants` + bootstrap remediation. Never alone force overall **fail**. Skip when no scope / vault closed. **If ships:** (1) update `health_check_order_names__fixed_matrix` + `Vec::with_capacity`; (2) docs: check is **cwd / `AI_BRAINS_PROJECT_ID` authoritative-scope**, not `--vault-path` identity; (3) hermetic/unit Ok\|Warn when scope+seeded empty grants (not only skip). Soft residual if time-box slips. |
| **F13 — Missing project id (unchanged)** | Progressive/expand missing project → exit **2** `fail_usage` (T202). |
| **F14 — Authorized empty (unchanged)** | Progressive with grants + zero hits → `denied: false`, exit **0**, empty `results` (true empty knowledge). |
| **F15 — Trace (unchanged)** | `query trace` missing → `null` exit 0 (T198 empty-success). |
| **F16 — Zero new crates / no clap bump** | clap pin **4.5**; serde_json **1.0**. |
| **F17 — denial_hint hard (M3)** | Additive `denial_hint: Option<String>` on `ProgressiveQueryResponse` (`#[serde(default, skip_serializing_if = "Option::is_none")]`). On capability deny construction set to same bootstrap wording as `POLICY_DENIED_HINT` (or shared constant if free without CLI→CP dep — may duplicate string with dual-site comment, or set from CLI after Ok if prefer no CP string). **DoD:** denied stdout JSON contains bootstrap substring (agents that ignore stderr). Update construction sites + golden fixture / protocol_compat as needed. **Overrides** earlier “prefer no contracts.” |
| **F18 — Daemon/HTTP progressive status** | Soft residual: local CLI is DoD. Daemon `QueryKnowledge` / HTTP remain 200 + denied packet (403 path exists for `DaemonResponse::Error` only). Document; no Tauri/parser break this track. |
| **F19 — Domain in CLI** | Forbidden beyond exit/emit mapping, optional CLI-side `denial_hint` fill, doctor probe, fail_cp wrap. |
| **F20 — Docs (hard)** | CLI-EXIT-CODES: progressive deny → **3** (packet on stdout + stderr remediation). OPERATIONS empty-vs-deny table + progressive row; **`Denied` may mean cross-scope or capability miss** (M8) — not existence of grants alone. CAPABILITIES progressive honesty. CHANGELOG minor (honesty BREAKING-ish exit). Skill one-liner if free. |
| **F21 — Hermetic hard suite** | AC1–AC6, AC10–AC12 (see §6). Soft AC9 if F12. |
| **F22 — High pre-ship** | Silent exit 0 on progressive deny; missing `denial_hint` on denied packet; flipping briefing exit; fail_api replacing packet; expand Unknown→3; AC3 principal mismatch; CP `?` → exit 1 on PolicyDenied; production unwrap; auto-init; interactive. |
| **F23 — Capture independence** | No models/graph. |
| **F24 — Series order** | After T220. Next: T218 / T219 / peers. |
| **F25 — T226 boundary** | Policy show/check soft-resolve **out** of T221. |
| **F26 — Determinism** | Sort any new doctor checks; stable hint = `POLICY_DENIED_HINT` SOOT (CLI + daemon twin unchanged). |
| **F27 — Review category** | FEATURE / CONTRACT. Primary review required. Cross-model soft (exit + contracts). |
| **F28 — Contracts (M3 override)** | **DoD includes** additive `denial_hint` (F17). Backward-compatible serde defaults. Protocol_compat / golden update required when field appears on deny fixtures. |
| **F29 — Residual map** | Daemon HTTP 200+denied; interactive bootstrap; full admin; T226; `--principal-id` on progressive (F32); trace `applied_policy` string (F36); briefing exit debate. |
| **F30 — Expand dual signal** | Exit 3 on Denied + F4 stderr; stdout still preview JSON. |
| **F31 — AC3 principal binding (M1 hard)** | Progressive/expand always use `cli_principal()` (System unless `AI_BRAINS_PREFLIGHT_PRINCIPAL_ID`). Hermetic AC3 **must** bootstrap that same principal: prefer **omit `--principal-id`** on bootstrap (defaults to System) **or** set `AI_BRAINS_PREFLIGHT_PRINCIPAL_ID` on both bootstrap and progressive. **Do not** reuse `policy_bootstrap.rs` Human `bbbb…` principal without matching progressive env — that traps AC3 as still-denied. |
| **F32 — Soft `--principal-id` progressive/expand (M2)** | Optional clap `env = AI_BRAINS_PREFLIGHT_PRINCIPAL_ID` parity with source/policy. **Not DoD.** If free, removes M1 trap class for operators. |
| **F33 — CP error path exit 3 (M5 hard)** | `run_progressive` / `run_expand` must **not** raw-`?` `ControlPlaneError` into generic exit **1**. Route through `fail_cp(OutputFormat::Json, err)` (or map `PolicyDenied` → exit 3 with emitted envelope). Latent today (deny is Ok-packet) but hard contract so future CP hard-deny does not regress. |
| **F34 — dry-run deny exit 3 (M6 hard)** | `--dry-run` progressive with deny → same exit **3** + packet + hints (not report-only success). Lock in AC1b. |
| **F35 — Denied semantics docs (M8 hard docs)** | Document expand/progressive Denied = **capability miss and/or cross-scope**; exit 3 does not prove which. No CP split this track. |
| **F36 — Soft trace applied_policy (M9)** | Packet `applied_policy: "DefaultPolicyEvaluator"` vs trace status `"denied"` — optional align later; **out of DoD**. |

## 5. Residual disposition

| Residual | Disposition |
|----------|-------------|
| Progressive deny exit 0 | **Absorb** F1–F4, F17, F21 |
| stdout-only agent remediation | **Absorb** F17 hard (M3) |
| Human deny drops hint | **Absorb** F5 |
| Expand Denied exit 0 | **Absorb** F6, F30, F35 docs |
| AC3 principal trap | **Absorb** F31 hard (M1) |
| CP `?` PolicyDenied → exit 1 | **Absorb** F33 (M5) |
| dry-run deny untested | **Absorb** F34 / AC1b (M6) |
| First-run bootstrap discoverability | **Absorb** F4–F5, F17, F20; soft F12 |
| T210 skill one-liner | Soft absorb F20 |
| Briefing exit 0 soft deny | **Decline** F7 (keep) |
| Auto-init / interactive | **Decline** F9–F10 |
| Full grant admin | **Decline** (T210 residual) |
| T226 show/check soft-scope | **Out** F25 |
| Daemon progressive wire status | Soft F18 (M10 affirm) |
| `--principal-id` progressive | Soft F32 (M2) |
| Trace applied_policy string | Soft F36 (M9) |

## 6. Acceptance criteria

| AC | Criterion | Proof |
|----|-----------|-------|
| **AC1** | Hermetic vault + project id, **no** grants: `query progressive "x"` → exit **3**; stdout JSON has `"denied": true` and empty results | Hermetic |
| **AC1b** | Same vault: `query progressive "x" --dry-run` → exit **3** + `denied: true` (F34) | Hermetic |
| **AC2** | Progressive deny: **stderr** contains `POLICY_DENIED` and `policy bootstrap`; **stdout** JSON contains bootstrap substring via `denial_hint` (or equivalent field) | Hermetic |
| **AC3** | After `policy bootstrap` targeting **same principal as progressive** (F31: omit `--principal-id` / System default): progressive → exit **0** and `"denied": false` (empty results OK) | Hermetic |
| **AC4** | `source list` human (or unit on `emit_error`) deny path includes bootstrap substring on **stderr** after CODE line | Hermetic / unit |
| **AC5** | Expand with policy `Denied` kind → exit **3**; Unknown handle still exit **0** | Hermetic / unit |
| **AC6** | Missing project progressive still exit **2** with example (T202 no regression) | Hermetic |
| **AC7** | Docs: CLI-EXIT-CODES + OPERATIONS/CAPABILITIES progressive deny exit 3 + Denied semantics (F35); CHANGELOG | Grep / review |
| **AC8** | Full CI gate green; no production unwrap | Gate |
| **AC9** | Soft: doctor `policy_grants` warn when grants empty + authoritative scope; matrix test updated if shipped (F12/M4) | Hermetic or unit if F12 |
| **AC10** | Briefing deny still exit **0** (non-regression lock) | Hermetic soft-required |
| **AC11** | Denied progressive stdout includes `denial_hint` (or documented field) with bootstrap; field omitted/`null` when not denied | Hermetic + serde/unit |
| **AC12** | Soft: unit or hermetic that `ControlPlaneError::PolicyDenied` mapped path exits **3** (F33) — may use fail_cp unit if hard to inject | Unit preferred |

## 7. Non-goals

- Auto-grant on `init` or silent AllowAll  
- Interactive bootstrap prompt  
- Full grant admin / revoke / Approve*/Erase bootstrap  
- Flip briefing soft deny to hard exit 3  
- T226 `policy show|check` soft-resolve  
- Daemon/HTTP progressive 200→403 flip (F18)  
- clap 5 / MSI / packaging  
- Daemon IssueGrant IPC  
- Ranking / semantic quality (T218)  
- Progressive multi-format human pretty UI  
- Trace `applied_policy` string align (F36 soft)  

## 8. Verification plan

1. **Red:** hermetic AC1/AC1b/AC2/AC3/AC11 fail on current main.  
2. **Green:** CLI exit map + F17 contracts + human emit_error + fail_cp wrap; optional doctor.  
3. Targeted: `cargo nextest run -p ai-brains-cli --test <suite>`; unit `governed_common` emit_error; contracts golden if touched.  
4. Clippy package + workspace gate.  
5. Manual: live vault without grants → progressive exit 3 + stdout/stderr bootstrap; bootstrap **without** wrong principal; progressive exit 0; briefing still soft.  
6. Review log; cross-model soft for exit + contracts.

## 9. Risks

| Risk | Mitigation |
|------|------------|
| Scripts that assumed progressive exit 0 on deny | Document BREAKING-ish honesty fix in CHANGELOG; exit 3 is correct |
| Consumers only parse stdout | F17 `denial_hint` + `denied:true` |
| fail_api temptation breaks packet parsers | F2 hard preserve packet |
| Expand Unknown misclassified as deny | F6 kind string exact `Denied` only |
| AC3 false fail (wrong principal) | F31 hermetic rules |
| Doctor matrix break / cwd lie | F12 M4 constraints or soft skip |
| Briefing accidentally flipped | AC10 lock |
| Hint string drift CLI vs CP | Prefer CLI fill of `denial_hint` after Ok **or** dual-site comment + tests |

## 10. Definition of Done

- [ ] Spec F-decisions + AC1–AC8, AC10–AC11 (AC9/AC12 soft) met  
- [ ] Progressive + expand deny exit honesty shipped  
- [ ] `denial_hint` + human deny hint shipped  
- [ ] Docs honesty (incl. F35)  
- [ ] Review clean critical/high; mediums fixed or deferred ≤3  
- [ ] Full gate green; conductor Completed; deferred.md struck for T221 progressive row  
- [ ] Ledger commit clean  

## 11. Suggested order note

… → ~~T220~~ closed → **T221** → T218 / T219 / peers; T226 separate.

## 14. AI fold-in disposition (2026-08-09)

Sources: `C:\dev\AI-review.md` — **AI1** (M1–M10 deep findings + research verify) + **AI2** (architecture affirm + AC map + dual-stream discipline).

| Item | Source | Disposition |
|------|--------|-------------|
| **Diagnosis** progressive println + Ok; denied unmapped | AI1 + AI2 | **Affirmed** — §2.2 root cause |
| **F1–F3 emit-then-exit** GovernedCliError | AI1 + AI2 | **Affirmed** — already plan |
| **F5/F6/F7** human hint, expand Denied, briefing soft | AI1 + AI2 | **Affirmed** |
| **clig / clap pins / dual-site hint / F18 daemon 200** | AI1 research table | **Affirmed** |
| **M1** AC3 principal-binding trap (System vs Human bootstrap) | AI1 Med-High | **Absorbed** **F31 hard** + AC3 wording |
| **M2** `--principal-id` on progressive/expand | AI1 Low/Med | **Soft** F32 — not DoD |
| **M3** stdout-only agents miss stderr remediation; promote F17 | AI1 Med-High | **Absorbed** **F17 hard** + AC2/AC11; **F28 override** |
| **M4** doctor 12-check matrix + cwd coupling | AI1 Med | **Absorbed into F12** if ships; else soft residual with constraints documented |
| **M5** CP `PolicyDenied` via `?` → exit 1 | AI1 Med | **Absorbed** **F33 hard** + AC12 soft unit |
| **M6** `--dry-run` deny untested | AI1 Low | **Absorbed** **F34** + **AC1b** |
| **M7** progressive stderr hint-only vs CODE+hint | AI1 Low | **Absorbed** **F4** CODE then hint |
| **M8** Denied = cross-scope or capability | AI1 Low | **Absorbed** **F35** docs hard |
| **M9** trace applied_policy string | AI1 Low | **Soft residual** F36 — not DoD |
| **M10** HTTP 403 plumbing exists; F18 sound | AI1 Low | **Affirmed** F18 — no track work |
| Dual-stream stdout packet / stderr hint | AI2 | **Affirmed** F2/F4 |
| Denied vs Unknown expand | AI2 | **Affirmed** F6 |
| Soft doctor F12 | AI2 | **Affirmed soft** + M4 gates |
| Auto-implement tone | — | Plan-only until **go** |

**Rejected / not absorbed as DoD:** flipping briefing to exit 3; replacing progressive stdout with bare `ApiError`; auto-init grant; interactive prompt; daemon/HTTP 200→403 this track; full grant admin; T226 soft-scope; clap bump; forcing F12 doctor if matrix/cwd work slips (keep soft).
