# T241 — Policy cold-start bootstrap

- **Track ID:** T241-PolicyColdStartBootstrap
- **Status:** 📋 **Planning** (plan-only until **go**)
- **Category:** FEATURE / UX / GOVERNED
- **Source:** Audit — briefing **4–5**, progressive **3**, evidence/source/review **3**, `policy show` empty **6/5**, `policy check` usage **5**; P0 first-run grants
- **Depends on:** T210 `policy bootstrap` (shipped); T221 deny exit **3** + soft residual doctor F12; T226 soft-resolve scope; T227 briefing denied next-step (markdown only); T240 identity (authoritative Scope for doctor/preflight)
- **Blocks / feeds:** T243 progressive ranking usefulness; every governed discovery surface
- **Absorbs:** T221 F12 doctor `policy_grants` (was soft); guided empty-grant UX from doctor/preflight; `policy show` empty guidance; `policy check` capability discoverability; briefing JSON denial next-step parity; T210 residual skill one-liner (soft); T226 bootstrap success soft-resolve hermetic (soft)
- **Not absorbed:** Full grant admin / revoke UI; auto-init grants; interactive prompt as required path; progressive ranking quality (→ T243); clap 5; daemon IssueGrant IPC; multi-tenant IdP; Approve*/Erase/Export bootstrap
- **Research date:** 2026-08-12 (live dogfood + clig.dev + clap pin + prior T210/T221 decisions)
- **AI fold-in:** 2026-08-12 — AI1 **M1–M3 hard** + L1–L12/O1–O13 agreed; AI2 **M1–M4** restate/affirm (scope skip, `<3` warn, contracts E1, check catalog). Disposition **§12**.
- **Ledger:** plan-only until go (`ledgerful ledger start` on go)

## 1. Objective

First-run (and this machine’s empty-grant vault) can discover and complete **one clear bootstrap path** so **ReadEvidence / ReadConclusions / ReadDecisions** unlock without archaeology. Governed surfaces stay deny-by-default until the operator opts in.

**This track does not reimplement `policy bootstrap`.** T210 already issues discovery grants. T241 makes empty grants **discoverable** and the next step **inescapable** from the surfaces operators already run (doctor, preflight, policy show/check, briefing JSON).

## 2. Problem (live re-scan 2026-08-12)

| Surface | Live result | Gap |
|---------|-------------|-----|
| `policy show` | exit 0 `{"api_version":"1","grants":[]}` | JSON bare empty; human `(none)` — **no bootstrap next-step** |
| `policy check` (no args) | clap exit **2** “required: --capability” | No capability catalog in usage path |
| `policy bootstrap --dry-run` | exit 0 — `would_register` + three `would_issue` | **Works** — path exists, undiscovered |
| `briefing project` (json) | exit 0, `denied: true`, no `denial_hint` field | Markdown has next-step; **JSON lacks progressive parity** |
| `briefing project` (human) | Denied + `next: run policy bootstrap…` | OK (T227) |
| `query progressive "…"` | exit **3** + `denial_hint` + stderr bootstrap | OK (T221) — still dead until grants |
| `evidence list` | exit **3** + hint bootstrap | OK — still dead until grants |
| `doctor` | 14 checks; **no `policy_grants`** | T221 F12 soft residual never shipped |
| `preflight --summary` | Scope + pins + harnesses; **no grants line** | Silent on empty grants |
| Capture / ungoverned | `preflight` legacy + `recall` work without grants | Must stay independent |

Root cause: **discoverability**, not missing mutation API.

## 3. Research summary (2026-08-12)

| Finding | Application |
|---------|-------------|
| [clig.dev](https://clig.dev/) — ease of discovery; suggest next command; first-run setup then real work | Doctor/preflight/show empty → name `policy bootstrap` |
| clig — dry-run for moderate danger; conversation as norm | Keep dry-run first in remediations: `bootstrap --dry-run` then apply |
| clig — change state → tell the user | Bootstrap success already reports issued/already; T241 does not change issue path |
| clig — help lists common flags/examples first | `policy check` after_help + missing-cap usage list discovery + full catalog |
| T210 F2/F3/F13 | Discovery Read* only; never Approve/Erase/Export/Propose; **no auto-init** |
| T210 F34 | Non-interactive default; flags/env only |
| T221 F12 M4 | Doctor `policy_grants` warn only; never alone force fail; matrix + cwd honesty |
| T151 deny-by-default | Empty grants remain correct security posture until bootstrap |
| clap workspace pin **4.5** (crates.io latest **4.6.6**) | **No pin bump** (series non-goal) |
| Progressive `denial_hint` (contracts) | Briefing packet should mirror optional `denial_hint` for JSON cold-start |
| Capture independence | Ungoverned recall/preflight never require grants; doctor/preflight grants check is advisory |

## 4. Frozen decisions (F0–F32)

| ID | Decision |
|----|----------|
| **F0 — Scope** | Discoverability + empty-state guidance around **existing** T210 bootstrap. No policy matrix redesign. No new grant capability set. |
| **F1 — Doctor `policy_grants` (DoD; absorb T221 F12 + AI2 M1–M2)** | When vault open + **authoritative** project scope resolved for CLI principal: probe discovery caps. If **active_count < 3** → **warn** `policy_grants` (message includes `discovery grants incomplete (N of 3)` when N>0, or empty when N=0) + **long** SOOT rem (F14). If **active_count == 3** → **ok** `discovery grants active (3 of 3)`. Never alone force overall **Fail** — **warn → Degraded** only (AI1 L3). **Skip** when vault closed / open-failed / no authoritative scope (`no authoritative project scope resolved in current context` — AI2 M1). |
| **F1b — Doctor StorePorts path (AI1 M2 hard)** | Doctor has **no** `AppContext`. Construct: `StorePorts::from_store(SqliteEventStore::new(vault_conn.clone()))` then `resolve_scope_key_for_cli(None, &ports.identity_store())`, `resolve_principal(None)`, `grant_store.list_applied_grants(principal.id, &scope_key, Some(&DISCOVERY_CAP_LABELS))` (or filter after list). **No** raw projection SQL. **No** `AppContext`. Clone `VaultConnection` (Arc) before `SqliteEventStore::new`. On list/resolve `Err` → skip (not fail). |
| **F2 — Doctor matrix (AI1 L4 + AI2 O1)** | 14 → **15**. Push `check_policy_grants(...)` **between** `check_project_identity` and `check_integrity`. Indices: `[12]=project_identity`, `[13]=policy_grants`, `[14]=integrity`. Update `Vec::with_capacity(15)`, `health_check_order_names__fixed_matrix`, all `checks.len()==14` sites. CAPABILITIES doctor table row (F18). Document cwd/env Scope coupling. |
| **F3 — Preflight summary (AI1 L2 hard)** | When project-scoped + authoritative + discovery incomplete (`<3`) → grants/next line with **short** SOOT. **Do not** change `format_preflight_summary_lines` / `build_preflight_summary_json` arity (keep 9-arg AC19). **Post-hoc append** grants line to returned `Vec<String>`. JSON: additive optional field on summary envelope (e.g. `grants_status` / `next_step`), not a 10th positional arg. Global: no grants line. Prefer warn-only on incomplete (omit OK density line). |
| **F4 — No `preflight --install-grants` as DoD** | Declined as DoD. Mutation stays on `policy bootstrap`. Soft F20 may add opt-in later. |
| **F5 — `policy show` empty (AI1 L6)** | **Human:** after `(none)`, print **short** SOOT (F14). **JSON:** `ScopeGrantsResponse` gains `#[serde(default, skip_serializing_if = "Option::is_none")] next_step: Option<String>`. `ScopeGrantsResponse::new(grants)` **unchanged**; `run_show` sets `resp.next_step = Some(short_soot)` when `grants.is_empty()` before emit. Non-empty → leave `None` (omit). E1: `grants: []` not null. |
| **F6 — `policy check` catalog (AI1 M1 hard)** | Make `--capability` `Option<String>` in clap. When `None` → **`fail_usage` exit 2** (not clap required-arg path). Message **must not** contain `required arguments were not provided`. Shape: leading line that `--capability` is required, then **discovery first** then remaining caps from single `CAPABILITY_CATALOG` constant (F6b). When `Some` → existing parse path; unknown → `INVALID_PAYLOAD`. Deny still exit **3** via `policy_denied_hint_details()` — **do not** conflate missing-cap usage with policy deny (AI1 L12). `CheckOptions.capability: Option<String>`; `if let Some(cap)` only — no production `unwrap`. |
| **F6b — Shared constants (AI1 L1/L7)** | Move discovery labels + catalog to `governed_common`: `DISCOVERY_CAP_LABELS` (3) and `CAPABILITY_CATALOG` (discovery first, then Propose*/Approve*/Export/Erase). Both `policy_cmd` and doctor import. Same list feeds Check `after_help` and fail_usage body. |
| **F7 — Briefing `denial_hint` (AI1 M3 hard)** | Add `denial_hint: Option<String>` to both briefing packets (contracts). **`empty_denied` sets `denial_hint: None`** — contracts must **not** hardcode CLI bootstrap text. Control-plane callers set `Some(bootstrap_hint)` after `empty_denied` / on denied paths using CP constant (align with `BRIEFING_DENIED_NEXT_STEP` / short SOOT family — must contain `policy bootstrap`). Soft exit **0** unchanged. Enumerate struct-literal sites (Phase 3 / §12). |
| **F8 — Deny exit invariants** | Progressive / evidence / source / review deny still exit **3** with bootstrap hint. Do not flip briefing to hard exit 3. |
| **F9 — Capture independence** | Doctor/preflight grants checks must not block vault open, recall, or legacy preflight. |
| **F10 — No auto-init / no silent grant** | Reaffirm T210 F13. |
| **F11 — No interactive required path** | Reaffirm T210 F34. |
| **F12 — Least privilege** | Bootstrap set unchanged (T210 F2). |
| **F13 — Domain logic** | Forbidden beyond probe + emit. CP store ports only. |
| **F14 — SOOT (AI1 L5)** | **Short** (show human, preflight, briefing `denial_hint`, progressive family): `next: run \`ai-brains policy bootstrap --dry-run\` then \`ai-brains policy bootstrap\``. **Long** (doctor remediation only): short + ` (omit --scope when project context is authoritative)`. Both must contain `policy bootstrap`. Markdown T227 line may stay if it already contains bootstrap. |
| **F15 — Principal (AI1 L10)** | Doctor/preflight: `resolve_principal(None)` only (reads `AI_BRAINS_PREFLIGHT_PRINCIPAL_ID`). **No** new `DoctorOptions` principal field. |
| **F16 — Scope resolve (AI2 M1)** | Doctor uses **`resolve_scope_key_for_cli`**, not `check_project_identity` comparison logic. **Skip** when resolve fails / non-authoritative. Do not invent grants for wrong project. |
| **F17 — Hermetic suite** | **7 DoD** + soft: AC1 doctor incomplete/empty warn; AC2 doctor skip no scope; AC3 matrix 15; AC4+AC5 show human+JSON (one or two tests); AC6 check no-capability fail_usage (no clap required text); AC7 briefing denial_hint; AC8 progressive exit 3 regression; AC9 preflight post-hoc line (unit). Soft F22. |
| **F18 — Docs (AI2 L2 + AI1 O13)** | CAPABILITIES doctor table: `policy_grants` after `project_identity`, severity warn, scope-coupled. OPERATIONS/INSTALL cold-start sequence. CHANGELOG minor. Contracts E1 for `next_step` / `denial_hint`. |
| **F19 — Contracts E1** | `next_step` + `denial_hint` optional with `skip_serializing_if`. No daemon wire change. |
| **F20 — Soft: `preflight --install-grants`** | Out of DoD. |
| **F21 — Soft: skill one-liner** | Soft. |
| **F22 — Soft: bootstrap success soft-resolve hermetic** | Soft; if free during F25 live dogfood, record success path (AI1 L9). |
| **F23 — Zero new crates / no clap bump** | clap **4.5** (lock ~4.6.1 OK). |
| **F24 — Review (AI1 O12 hard)** | FEATURE primary review. Cross-model **hard** — both doctor matrix + contracts `denial_hint` land. |
| **F25 — Live dogfood sequence (AI1 O11)** | On go only, record exit codes: (1) `policy bootstrap --dry-run`; (2) `policy bootstrap`; (3) `policy show` → 3 grants; (4) `query progressive "test"` exit 0 **or** briefing `denied: false`; (5) `evidence list` exit 0. |
| **F26 — Determinism** | Catalog order: discovery trio then remaining caps (stable). Doctor check order fixed. JSON omit rules stable. |
| **F27 — Parallel (AI1 L11)** | Coordinate with T249 if concurrent on doctor presentation. Low conflict with T243/T245 if they avoid doctor matrix + contracts briefings. |
| **F28 — Stop-before** | No auto-mass-grant; no progressive deny-exit change; no grants required for capture/recall. |
| **F29 — Matrix length sites** | Update **all** of: `with_capacity`, matrix unit `expected.len()`, hermetic `checks.len()` (and any other hard-coded 14). |
| **F30 — F6 hermetic pin (AI1 M1)** | `policy check --scope Repository:<uuid>` **without** `--capability` → exit **2**, stderr contains ReadEvidence/ReadConclusions/ReadDecisions, **does not** contain `required arguments were not provided`. |
| **F31 — Partial grants** | Incomplete discovery (`1` or `2` of `3`) → same **warn** as empty (bootstrap idempotent fills missing). Preflight summary treats incomplete same as empty for next line. |
| **F32 — `next_step` only when grants empty** | For policy show: `next_step` when `grants.is_empty()` only. Doctor incomplete (`1–2` of 3) still has non-empty show list — rem lives on doctor, not forced on show. |

## 5. Residual disposition

| Residual | Disposition |
|----------|-------------|
| T221 F12 doctor `policy_grants` | **Absorb DoD** F1–F2 |
| deferred.md “Policy grants empty → governed dead-end” | **Absorb** this track |
| T210 skill one-liner | Soft F21 |
| T210/T226 bootstrap success soft-resolve hermetic | Soft F22 |
| Full grant admin / revoke | **Decline** |
| Auto-init grant | **Decline** F10 |
| `preflight --install-grants` | Soft F20 decline as DoD |
| Progressive ranking | **Out** → T243 |
| clap 5 / MSI | **Out** |

## 6. Acceptance criteria

| AC | Criterion | Proof |
|----|-----------|-------|
| **AC1** | Doctor authoritative + discovery active_count < 3 → `policy_grants` **warn** + rem contains `policy bootstrap`; roll-up may be **Degraded** not Fail | Hermetic / unit |
| **AC2** | Doctor without authoritative scope → `policy_grants` **skip** (message may note no authoritative scope) | Hermetic / unit |
| **AC3** | Matrix exactly **15**; `[12]=project_identity`, `[13]=policy_grants`, `[14]=integrity` | Unit |
| **AC4** | `policy show` empty human contains short SOOT / bootstrap; exit 0 | Hermetic |
| **AC5** | `policy show` empty JSON: `grants: []`, `next_step` present with bootstrap; non-empty omits `next_step` | Hermetic |
| **AC6** | `policy check` without `--capability` → exit **2** + discovery cap names; **no** `required arguments were not provided` (F30) | Hermetic |
| **AC7** | Briefing denied JSON includes `denial_hint` containing bootstrap; exit still **0** | Hermetic |
| **AC8** | Progressive deny still exit **3** + bootstrap hint (regression lock) | Existing or thin hermetic |
| **AC9** | Preflight summary: incomplete discovery → grants/next line; **9-arg formatters unchanged** (post-hoc append) | Unit / hermetic |
| **AC10** | Capture independence: ungoverned path docs + no new grant requirement on recall | Review / smoke |
| **AC11** | Docs + CHANGELOG + contracts E1; CAPABILITIES doctor row | Grep / review |
| **AC12** | Full CI gate green; no production unwrap | Gate |
| **AC13** | Soft: live F25 sequence after bootstrap | Manual on go |
| **AC14** | Soft: partial grants (1–2 of 3) still doctor **warn** | Unit if free |

## 7. Non-goals

- Full grant issue/revoke admin CLI matrix
- Auto-grant on `init` or first `preflight`
- Interactive consent wizard as sole path
- Policy engine / DefaultPolicyEvaluator changes
- Daemon IssueGrant IPC
- Progressive ranking / search unify (T243)
- clap 5, MSI, packaging
- Changing exit **3** deny contract

## 8. Verification plan

1. **Red:** hermetic/unit AC1–AC7 fail against current code.  
2. **Green:** doctor check + show empty + check catalog + briefing denial_hint + preflight summary line.  
3. Targeted: `cargo nextest run -p ai-brains-cli --test policy_bootstrap` + doctor unit matrix + contracts unit; clippy `-p ai-brains-cli -p ai-brains-contracts`.  
4. Manual (go): dry-run → bootstrap → `policy show` non-empty → `query progressive` / `briefing project` / `evidence list`.  
5. Review log; **hard** cross-model (F24).  
6. Full gate before finalize.

## 9. Risks

| Risk | Mitigation |
|------|------------|
| Doctor false warn on wrong Scope | F16 skip when non-authoritative; cwd honesty docs |
| Matrix length regressions | F29 + AC3 |
| Contracts struct-literal break | F7 site list; empty_denied None + callers |
| Doctor builds AppContext / raw SQL | F1b StorePorts-only path |
| Clap vs fail_usage exit-2 confusion | F6/F30 hermetic pin |
| Dual remediation wording drift | F14 short/long; substring `policy bootstrap` |
| Preflight AC19 arity break | F3 post-hoc append |
| Operators expect auto-grant | F10 docs + explicit next only |

## 10. Touch map (expected)

| Area | Files (indicative) |
|------|-------------------|
| Doctor | `doctor.rs` — `check_policy_grants`, matrix 15, StorePorts path |
| Policy CLI | `policy_cmd.rs` — show empty, check optional capability |
| Shared | `governed_common.rs` — DISCOVERY_CAP_LABELS, CAPABILITY_CATALOG, short SOOT |
| Clap | `main.rs` PolicyCommands::Check capability Option |
| Preflight | `preflight.rs` post-hoc grants line + optional JSON field |
| Contracts | `scopes.rs` `next_step`; `briefings.rs` `denial_hint` |
| Briefing build | CP project/personal/renderer + callers set hint |
| Tests | doctor unit; policy hermetic; briefing; exit_contract if needed |
| Docs | CAPABILITIES, OPERATIONS/INSTALL, CHANGELOG |

## 11. Definition of Done

- All DoD ACs green; soft ACs noted.  
- Hard cross-model clean (F24).  
- Conductor T241 **Completed** only after go + gate + review.  
- deferred.md empty-grant row closed.  
- Pin decisions.  
- No live mutating bootstrap during plan-only.

## 12. AI fold-in disposition (2026-08-12)

Source: `C:\dev\AI-review.md` (AI1 + AI2). **No Highs.** Spec design affirmed.

### AI1

| ID | Verdict | Action |
|----|---------|--------|
| **M1** F6 exit-2 / fail_usage shape | **Agree hard** | F6, F30, AC6 |
| **M2** StorePorts construction | **Agree hard** | F1b |
| **M3** denial_hint sites + empty_denied None | **Agree hard** | F7 + site list in plan |
| **L1** shared DISCOVERY_CAPS | **Agree** | F6b |
| **L2** preflight post-hoc / no arity change | **Agree hard** | F3, AC9 |
| **L3** warn → Degraded | **Agree** | F1 |
| **L4** insert between project_identity and integrity | **Agree** | F2 (already) |
| **L5** short vs long SOOT | **Agree** | F14 |
| **L6** CLI-set next_step | **Agree** | F5 |
| **L7** single CAPABILITY_CATALOG | **Agree** | F6b |
| **L8** hermetic count map | **Agree** | F17 |
| **L9** F22 during live | **Agree soft** | F22 |
| **L10** resolve_principal(None) | **Agree** | F15 |
| **L11** T249 coordinate | **Agree soft** | F27 |
| **L12** usage ≠ policy deny | **Agree** | F6 |
| **O1–O13** | **Agree** | folded into F* above; **O12** → F24 hard |

### AI2

| ID | Verdict | Action |
|----|---------|--------|
| **M1** authoritative skip | **Agree** | F16 (reaffirm) + AC2 skip message |
| **M2** active_count < 3 warn | **Agree hard** | F1, F31 (was plan note; F1 said “zero” — **fixed**) |
| **M3** contracts next_step / denial_hint | **Agree** | F5, F7, F19 |
| **M4** capability optional + catalog | **Agree** | F6 (reaffirm) |
| **L1** SOOT family | **Agree** with AI1 short/long split | F14 |
| **L2** docs matrix | **Agree** | F18 |
| **O1–O2** matrix test + hermetic | **Agree** | AC3, F17 |

### Declined / not new design

| Item | Why |
|------|-----|
| Auto-init / install-grants DoD | Still F4/F10 |
| Clap pin bump | F23 |
| Hardcoding bootstrap text in contracts empty_denied | F7 — callers set |

### Pins locked by fold-in

1. F1 incomplete = `active_count < 3`; warn → Degraded.  
2. F1b StorePorts-only doctor path.  
3. F3 post-hoc preflight; no 9-arg breakage.  
4. F6/F30 fail_usage catalog; no clap “required arguments” text.  
5. F7 empty_denied `None`; CP sets hint; site list in plan.  
6. F14 short/long SOOT.  
7. F24 hard cross-model.

---

**Plan-only until user says go.**
