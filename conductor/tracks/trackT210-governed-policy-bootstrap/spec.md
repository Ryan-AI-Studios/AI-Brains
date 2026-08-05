# T210 — Governed policy bootstrap for discovery

- **Track ID:** T210-GovernedPolicyBootstrap
- **Phase:** Post-T209 skill·CLI audit follow-ups (P1)
- **Status:** 📋 **Proposed / Expanded + AI fold-in** (plan-only until go)
- **Depends on:** T151 policy matrix + grants; T160 `policy show|check` (read-only); T187 vault key; T197 key bootstrap; **T201** exit contract + `details.hint`; **T203** discovery lists + soft-resolve; T209 closed
- **Blocks / feeds:** Operators/agents can actually use `source list` / `evidence list` / `review list` / briefing after vault open without library-only `issue_grant`; residual full grant admin (revoke UI, multi-tenant IdP) stays deferred
- **Category:** FEATURE / DOCS (skill touch) / CONTRACTS (CLI-local response DoD; contracts lift soft)
- **Source:** Non-destructive skill/CLI audit 2026-08-04 — source/evidence/review/briefing **POLICY_DENIED** (audit 3–4); T160/T203 explicitly deferred grant mutation UX
- **Deferred absorbed:** deferred.md T210 placeholder; T160 “Grant issue/revoke admin UX”; T203 residual “Grant admin / approve UX” **partial** (bootstrap only — not full admin); T201 grant-admin leave; live hint loop to `policy show` with no grant path
- **Not absorbed:** Full grant admin (arbitrary issue/revoke matrix UI); multi-user IdP; Approve*/Erase/Export bootstrap; auto-grant on `init` by default; policy engine redesign; clap 5; MSI; T211+ ranking; daemon grant IPC (soft)
- **Research date:** 2026-08-04 (expand + live re-scan + online)
- **AI fold-in:** 2026-08-04 — AI1 affirms F1–F15 core. AI2 **M1–M5** accepted; **L1–L5** affirm/mirror; **L6** soft (ISSUES.md). Disposition **§14**.
- **Ledger:** plan-only until go (`ledgerful ledger start` on go)

## 1. Objective

1. **Operator bootstrap path** so the local CLI principal can obtain **discovery-class** grants (`ReadEvidence`, `ReadConclusions`, `ReadDecisions`) on a resolved scope without writing Rust tests or calling `issue_grant` by hand.  
2. **Register principal** if missing (default well-known System CLI principal, or Human when `AI_BRAINS_PREFLIGHT_PRINCIPAL_ID` / `--principal-id` is set).  
3. **Idempotent** re-run: already-active grants are reported, not double-issued (partial unique index).  
4. **Honesty:** update `POLICY_DENIED` `details.hint` to point at bootstrap (not only `policy show`).  
5. **Empty-vs-deny clarity** in docs/skill: list/show hard exit **3** vs briefing soft `denied`/empty sections.  
6. **Least privilege:** bootstrap does **not** issue Propose*/Approve*/Export/Erase.  
7. Do **not** weaken deny-by-default for unregistered principals or unknown scopes.

## 2. Live baseline (re-scan 2026-08-04)

### 2.1 Audit signal — confirmed live

| Surface | Live result |
|---------|-------------|
| `source list --format json` (authoritative project context) | exit **3** `POLICY_DENIED` / `ReadEvidence denied for list_sources` + hint → `policy show` |
| `review list --format json` | exit **3** `ReadConclusions denied for list_review_items` + same hint |
| `briefing project --format json` | exit **0** soft deny: `warnings[].kind=denied` (“ReadDecisions/ReadConclusions denied…”); empty decisions/conclusions |
| `policy show --scope Repository:441837f6-…` | exit **0** `grants: []` for principal `a1b2a1b2-a1b2-a1b2-a1b2-a1b2a1b2a1b2` |
| `policy check --capability Read{Evidence,Conclusions,Decisions}` | exit **3** all three |
| `policy` subcommands | **`show` / `check` only** — no grant mutation CLI |
| Domain API | `register_principal` + `issue_grant` + `revoke_grant` exist in `ai-brains-control-plane` |
| Daemon wire | **No** IssueGrant / RegisterPrincipal IPC variants |
| Hint SOOT | CLI `POLICY_DENIED_HINT` + daemon duplicate string — show only, no bootstrap |

### 2.2 Why deny is correct (and still broken UX)

| Fact | Implication |
|------|-------------|
| T151 `DefaultPolicyEvaluator` deny-by-default | Unknown principal → deny; missing grant → deny |
| Default CLI principal | Well-known **System** UUID (`cli_principal` / `0xA1B2…`) **or** Human when env UUID set |
| System matrix | Empty `bound_capabilities` → **grant alone** allows; still needs registered principal + grant rows |
| Production vault | Principal **not registered**; **zero** grants on repo scope |
| T160/T203 | Deliberately deferred grant mutation; discovery lists land → deny is now the default operator experience |
| Partial unique | `idx_scope_grant_active_unique (principal_id, scope_key, capability) WHERE revoked_at IS NULL` → re-issue without check → projection conflict |

### 2.3 Code / touch map (AI2-verified)

| Site | Role |
|------|------|
| `ai-brains-cli/src/main.rs` | `PolicyCommands` — only Show/Check today; add **Bootstrap** (soft `Grant` not DoD) |
| `ai-brains-cli/src/commands/policy_cmd.rs` | `run_bootstrap`; mirror `run_check` wiring (`StorePorts::from_store`, `resolve_principal`) |
| `ai-brains-cli/src/commands/governed_common.rs` | `POLICY_DENIED_HINT` update; `policy_denied_hint_details()` stays; **`resolve_scope_key_for_cli`** for omitted `--scope`; **`resolve_principal`** SOOT for principal |
| `ai-brains-cli/src/commands/briefing.rs` | `cli_principal()` — **do not call directly** from bootstrap; use `resolve_principal` (superset, already used by show/check) |
| `ai-brains-control-plane/src/grants.rs` | `register_principal`, `issue_grant` — **call only** |
| `ai-brains-control-plane` adapters | **F7 probe:** `grant_store.active_grants(principal_id, &ScopeRef)` (typed `ScopeGrant`) — **not** `list_applied_grants` (briefing DTO / policy show path). **F33 probe:** `grant_store.get_principal` before register |
| `ai-brainsd/src/services.rs` | Private twin `POLICY_DENIED_HINT` + sync comment — **must** update with CLI; **AC11** daemon unit asserts `bootstrap` substring |
| Response DTO | **CLI-local** `PolicyBootstrapResponse` with `api_version: "1"` for DoD (F19 freeze); contracts lift only if F25 later |
| Hermetic tests | New `tests/policy_bootstrap.rs` — reuse T203 `hermetic_bin` / tempdir / ZERO_KEY pattern from `governed_discovery_reads.rs` |
| Docs | CAPABILITIES, OPERATIONS, INSTALL (post-init bootstrap), CLI-EXIT-CODES, CHANGELOG minor, skill one-liner if agent-facing |

### 2.4 Deps

| Item | Pin / note |
|------|------------|
| clap | Workspace pin **4.5** (resolves ~4.6.x); crates.io latest 4.6.5 — **no pin bump** (F16) |
| Zero new crates | Required — all APIs re-exported from control-plane |
| Domain | Existing events/projections only — **no** new migration |
| Capture independence | Grant append is event log write — no models/embeddings |

## 3. Research summary (2026-08-04)

| Finding | Application |
|---------|-------------|
| [clig.dev](https://clig.dev/) — ease of discovery; suggest next command; conversation as norm; first-run setup then real work | Bootstrap is the **setup** step; deny hint must name the next command |
| clig — change state → tell the user; success brief | Bootstrap prints issued / already-present; exit 0 when no-op success |
| clig — dry-run for multi-step / moderate danger | `--dry-run` required for bootstrap (reports plan, no append) |
| Least privilege (Azure RBAC, OSO 2025, industry 2026) | Bootstrap **read discovery only**; never Approve/Erase/Export/Propose |
| OWASP A01 / T151 | Deny default stays; bootstrap is **explicit** grant issue, not silent allow |
| T151 matrix | System empty-bound + grant OK; Human needs grant; unknown principal → register first |
| T160 design lock | Domain logic **forbidden** in CLI — call CP `register_principal` / `issue_grant` |
| T201 F6 | Structured `details.hint` already; update template only |
| T203 soft-resolve | Reuse `resolve_scope_key_for_cli` (or sibling) for omitted `--scope` |
| Unique active grant index | Bootstrap **must** probe **`active_grants`** before `issue_grant` (M1) |
| device bootstrap precedent | Product already has first-device `device bootstrap` naming — `policy bootstrap` is parallel “first-run enable” |
| AI2 dual-site hint | CLI + daemon private const; no shared crate — **AC7 + AC11** both assert `bootstrap` substring |

## 4. Frozen decisions (F1–F40)

| ID | Decision |
|----|----------|
| **F1 — Command** | Add **`ai-brains policy bootstrap`**. Primary UX. Not a top-level rename. |
| **F2 — Discovery capability set** | Bootstrap issues **exactly** these three (when missing): `ReadEvidence`, `ReadConclusions`, `ReadDecisions`. **Hard refuse** any other capability in this path. |
| **F3 — No dangerous caps** | Never bootstrap Propose*, Approve*, Export, Erase. Full admin / single-cap arbitrary grant is **soft optional** only as `policy grant` (see F24) — not DoD. |
| **F4 — Principal** | Target via **`resolve_principal(options.principal_id.as_deref())` only** (same as show/check — not a direct `cli_principal()` fork). Env/`--principal-id` → Human UUID; else well-known System CLI principal. Kind for `register_principal` matches that construction. |
| **F5 — Scope** | Explicit `--scope` **or** soft-resolve via **`resolve_scope_key_for_cli(None, &ports.identity_store())`** when authoritative. Missing + non-authoritative → `fail_usage` exit **2** (not 6). |
| **F6 — Privacy** | Issued grants use **`Privacy::LocalOnly`** always. No CloudOk bootstrap. |
| **F7 — Idempotent grants (M1)** | Parse `scope_key` → `ScopeRef` first. Call **`grant_store.active_grants(principal.id, &scope_ref)`** (typed `ScopeGrant` — **not** `list_applied_grants`). For each discovery cap: if any grant has that capability → **`already_present`** (do **not** `issue_grant`); else issue. Exit **0** when all three already present. |
| **F8 — Path** | **Local control-plane** only (`StorePorts::from_store` like show/check). No daemon IssueGrant wire in DoD. Soft F25. |
| **F9 — Dry-run** | `--dry-run` / `-n`: resolve principal+scope, report would-register / would-issue / already_present; **zero** event appends (no register, no issue). |
| **F10 — Output** | Default **json** (`default_value = "json"` like show/check — **F38**). Shape: `api_version`, `principal_id`, `scope`, `registered` status string **`registered` \| `already` \| `would_register`**, `grants: [{capability, status: issued\|already_present\|would_issue, grant_id?}]`, `dry_run` bool. Human: short lines + next-command hint (`source list` / `review list` / `briefing project`). Sort grants by capability name (**F30**). |
| **F11 — Exit codes** | Success (issued or no-op) → **0**. Soft-resolve fail → **2**. Vault key / internal → **1**. Invalid scope parse → **`fail_cp`** (same as show — typically exit **6**). Never exit 3 on successful bootstrap. |
| **F12 — Hint upgrade (required)** | Update CLI `POLICY_DENIED_HINT` **and** daemon `services.rs` private twin to mention bootstrap first, show secondary. Suggested template: `ensure a grant for this capability exists; run \`ai-brains policy bootstrap --scope …\` (or check with \`ai-brains policy show --scope …\`)`. Keep dual-site sync comment. **AC7** = CLI; **AC11** = daemon unit (M4) — comment alone is insufficient. |
| **F13 — No auto-init grant** | **`init` does not auto-issue** discovery grants. Explicit bootstrap preserves deny-by-default until operator opts in. **INSTALL** documents bootstrap after init (**F23** / L5). |
| **F14 — No policy engine change** | Do not alter `DefaultPolicyEvaluator` matrix. Bootstrap only writes events projections already understand. |
| **F15 — Domain in CLI** | Forbidden. Call CP only. |
| **F16 — Zero new crates / no clap bump** | Workspace clap pin **4.5** stay (resolves 4.6.x OK). |
| **F17 — Capture independence** | No models/graph for bootstrap. |
| **F18 — Event sourcing** | Grants append `PrincipalRegistered` / `ScopeGrantIssued` only. No raw projection SQL. No revoke in DoD. |
| **F19 — Contracts (L3 freeze)** | **DoD = CLI-local** serde struct with `api_version: "1"` + stable JSON in AC. Do **not** add contracts DTO this track. Lift to `ai-brains-contracts` only if F25 daemon IPC lands later. |
| **F20 — Series** | After T209. Before T211. |
| **F21 — Hermetic (≥7)** | (1) deny before bootstrap; (2) bootstrap issues three; (3) lists exit 0 empty or items; (4) re-bootstrap no-op already_present + registered `already`; (5) dry-run no grants; (6) CLI hint contains bootstrap; (7) daemon hint unit AC11. Soft: soft-resolve scope AC8. |
| **F22 — High pre-ship** | Silent allow without grant; bootstrap Erase/Approve; double-issue unique crash; skip `active_grants` probe; wrong principal; production unwrap; auto-init grant; register re-append noise without get_principal. |
| **F23 — Docs / skill** | CAPABILITIES policy section; OPERATIONS governed bootstrap step; INSTALL post-init bootstrap; CHANGELOG minor; CLI-EXIT-CODES POLICY_DENIED remediation; optional skill one-liner. |
| **F24 — Soft `policy grant`** | Optional thin `policy grant --capability X --scope` for non-dangerous caps if free — **not DoD**. Refuse Erase/Approve* even if free. |
| **F25 — Soft daemon IPC** | IssueGrant over pipe **out of DoD**. Local path sufficient for single-user. |
| **F26 — Soft revoke** | `policy revoke` **out of DoD** → residual deferred.md after ship (L6: `ISSUES.md` absent — use deferred.md). |
| **F27 — Soft personal scope** | Personal briefing bootstrap when scope Personal — include if free; Repository is DoD. |
| **F28 — Empty vs deny honesty** | Docs table: list/show deny = exit 3; briefing deny = exit 0 + `denied`/warnings. Bootstrap clears both when grants present. |
| **F29 — after_help** | Examples: `policy bootstrap --scope Repository:<uuid>`; omit scope when project authoritative; `policy show` after. |
| **F30 — Determinism** | Sort grants in response by capability name; stable status labels. |
| **F31 — command_id** | Not required (not daemon mutation spool). Local append is fine without command_id. |
| **F32 — Review category** | FEATURE. Primary review required. Cross-model soft when dual-site hint lands. |
| **F33 — Register probe DoD (M2)** | **Required:** before `register_principal`, call `grant_store.get_principal(principal.id)`. If `Some` → skip register, report `registered: "already"`. If `None` and not dry-run → `register_principal`, report `registered: "registered"`. Dry-run missing → `would_register`. **Do not** re-append PrincipalRegistered on re-bootstrap. |
| **F34 — No interactive prompt** | Non-interactive always. Flags/env only. |
| **F35 — Actor** | Events use existing `Actor::System` via `build_event` path in `issue_grant` (no change). Honesty: bootstrap is operator-initiated System-actor grant issue (same as tests). |
| **F36 — Residual map** | Full grant admin, revoke CLI, multi-tenant, Approve bootstrap, auto-init → deferred.md after ship. |
| **F37 — Clap principal env (M5)** | Bootstrap `principal_id: Option<String>` **must** use `#[arg(long, env = "AI_BRAINS_PREFLIGHT_PRINCIPAL_ID")]` — parity with Show/Check. |
| **F38 — Clap format default (L2)** | Bootstrap `format` uses `#[arg(long, default_value = "json")]` like Show/Check. |
| **F39 — Soft-resolve wiring (L1)** | When `--scope` omitted: `resolve_scope_key_for_cli` + `ports.identity_store()`; when provided: use as-is then `parse_scope_key` + `fail_cp` on error. |
| **F40 — AI1 affirm** | Dead-end remediation, least privilege F2/F3/F6, F7 unique-index protection, F9 dry-run, F13/F14 invariants — all above. |

## 5. Residual disposition

| Residual | Disposition |
|----------|-------------|
| T210 placeholder POLICY_DENIED | **Absorb** F1–F12, F21 |
| T160 grant mutation deferred | **Partial absorb** — bootstrap only |
| T203 grant admin decline | **Partial absorb** — not full admin (F24 soft) |
| Hint loop to show only | **Absorb** F12 + AC7/AC11 |
| Empty vs deny confusion | **Absorb** F28 docs |
| Full revoke / multi-tenant / Approve | **Decline** F26 |
| Auto-grant on init | **Decline** F13 |
| Daemon IssueGrant | Soft F25 decline as DoD |
| T211+ | Out |

## 6. Acceptance criteria

| AC | Criterion | Proof |
|----|-----------|-------|
| **AC1** | Hermetic vault + seeded scope: before bootstrap, `source list` (or `policy check ReadEvidence`) → exit **3** | Hermetic |
| **AC2** | `policy bootstrap --scope Repository:<uuid>` → exit **0**; `registered: "registered"` first time; issues three caps | Hermetic |
| **AC3** | After bootstrap, `policy check` for each of three → allowed / exit **0** | Hermetic |
| **AC4** | After bootstrap, `source list` / `review list` with scope → exit **0** (empty items OK) | Hermetic |
| **AC5** | Second bootstrap → exit **0**, all grants `already_present`, `registered: "already"`, no unique-index crash, no second PrincipalRegistered required | Hermetic |
| **AC6** | `--dry-run` → exit **0**, zero event appends (grants still absent if pre-empty) | Hermetic |
| **AC7** | CLI `POLICY_DENIED_HINT` (and/or live deny JSON) contains `policy bootstrap` | Unit or hermetic |
| **AC8** | Soft-resolve: authoritative project env + no `--scope` → bootstrap success **or** unset → fail_usage exit **2** — at least one path locked | Hermetic soft-required |
| **AC9** | Docs: CAPABILITIES + OPERATIONS or INSTALL mention bootstrap; CHANGELOG entry | Grep / review |
| **AC10** | Full CI gate green; no production unwrap | Gate |
| **AC11** | Daemon `POLICY_DENIED_HINT` (services.rs) unit test asserts substring `bootstrap` (M4 dual-site) | `ai-brainsd` unit |

## 7. Non-goals

- Packaging (MSI/notarization)
- clap 5 multi-heading
- Policy matrix redesign / AllowAll in production
- Multi-user identity providers
- Content erasure / Approve bootstrap
- Relicensing
- T211–T216 scope

## 8. Verification plan

1. **Red:** hermetic tests AC1–AC7 fail; daemon AC11 fails until string updated.  
2. **Green:** implement bootstrap + dual-site hint.  
3. Targeted: `cargo nextest run -p ai-brains-cli --test policy_bootstrap`; daemon unit for AC11; optional exit_contract if hint tested there.  
4. Clippy package + workspace gate before finalize.  
5. Manual: live vault `policy bootstrap --scope Repository:…` then `source list` / `review list` / `briefing project`.  
6. Review log + cross-model soft for dual-site hint.

## 9. Risks

| Risk | Mitigation |
|------|------------|
| Operators treat bootstrap as full admin | Docs + F2 hard set; refuse dangerous caps |
| Double-issue unique index | F7 **`active_grants`** probe before every issue |
| Hint drift CLI vs daemon | F12 + **AC7 + AC11** (not comment-only) |
| Re-bootstrap PrincipalRegistered noise | F33 **DoD** `get_principal` probe |
| Soft-resolve flaky hermetic | Document hermetic env like T203 F37 |
| Scope expands to full grant CLI | Cap F24 soft; stop-before full admin |

## 10. Definition of Done

- [ ] Spec F-decisions + AC1–AC11 met  
- [ ] `policy bootstrap` shipped + hermetic suite  
- [ ] Hint strings updated CLI + daemon (+ tests)  
- [ ] Docs/skill honesty  
- [ ] Review clean for critical/high; mediums fixed or deferred ≤3  
- [ ] Full gate green; conductor Completed; deferred.md struck for T210  
- [ ] Ledger commit clean  

## 11. Suggested order note

… → T209 closed → **T210** → T211/T215 → T212–T214/T216.

## 14. AI fold-in disposition (2026-08-04)

| ID | Source | Action |
|----|--------|--------|
| **AI1 #1** | Dead-end + bootstrap + hint | **Affirm** F1/F12 |
| **AI1 #2** | Least privilege three Read* + LocalOnly | **Affirm** F2/F3/F6 |
| **AI1 #3** | Idempotent / unique index | **Affirm** F7 (clarified active_grants) |
| **AI1 #4** | Dry-run zero events | **Affirm** F9 |
| **AI1 #5** | No matrix change / no auto-init | **Affirm** F13/F14 |
| **M1** | Probe `active_grants` not `list_applied_grants` | **Accept** F7 + §2.3 |
| **M2** | `get_principal` before register → DoD | **Accept** F33 elevated |
| **M3** | Explicit parse → active_grants → match → issue sketch | **Accept** plan sketch + F7 |
| **M4** | Dual-site hint test | **Accept** AC11 + F12 |
| **M5** | Bootstrap clap `env = AI_BRAINS_PREFLIGHT_PRINCIPAL_ID` | **Accept** F37 |
| **L1** | Soft-resolve + identity_store | **Affirm** F5/F39 |
| **L2** | format default json | **Affirm** F38 |
| **L3** | CLI-local response DTO | **Accept freeze** F19 |
| **L4** | fail_cp on scope parse | **Affirm** F11 |
| **L5** | INSTALL post-init step | **Affirm** F13/F23 |
| **L6** | ISSUES.md missing | **Soft** — use deferred.md for residuals (same as T209) |
