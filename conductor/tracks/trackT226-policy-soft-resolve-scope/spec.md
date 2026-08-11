# T226 — Policy show/check soft-resolve scope

- **Track ID:** T226-PolicySoftResolveScope
- **Phase:** Post-audit CLI quality series (T217–T232)
- **Status:** ✅ **Completed** 2026-08-11 (PR #130 `5919f26`)
- **Depends on:** T203 soft-resolve helper; T160 `policy show|check`; T201 exit contract; T210 bootstrap (already soft)
- **Blocks / feeds:** Operators can inspect grants and dry-run checks without retyping `--scope` when project context is authoritative; unblocks conversational bootstrap → show → check loops
- **Category:** UX / CONSISTENCY
- **Source:** Non-destructive CLI audit 2026-08-05 — `policy show/check` usefulness **5** · quality **6** (clap-required `--scope` while discovery lists + bootstrap soft-resolve)
- **Deferred absorbed:** deferred.md “policy show/check required scope”; series README T226; T210 soft residual “AC8 success soft-resolve hermetic” **partial** (show/check AC4 success path; bootstrap success path remains soft optional)
- **Not absorbed:** Full grant admin / revoke UI; `erasure request|wipe` scope stay clap-required (destructive); `review resolve` stay required; grant evaluation / matrix changes; daemon IssueGrant IPC; clap 5; MSI; auto-init grants
- **Research date:** 2026-08-11 (live dogfood + code truth + online)
- **AI fold-in:** 2026-08-11 — AI1 **M1–M4 hard**; **L1–L6** fold; **O2–O4** elevate DoD; **O1/O5/O6** soft. AI2 affirms core design + AC matrix (no new mediums). Disposition **§15**.
- **Ledger:** plan-only until go (`ledgerful ledger start` on go)

## 1. Objective

1. When authoritative project scope exists (`AI_BRAINS_PROJECT_ID` / cwd identity), **`policy show`** and **`policy check`** soft-fill omitted `--scope` via existing `resolve_scope_key_for_cli`.
2. When scope is omitted and context is **not** authoritative → **`fail_usage` exit 2** with the same template class as discovery lists / bootstrap (never clap “required arguments were not provided”; never exit **6** for missing scope).
3. Explicit `--scope` always wins (including malformed → existing **6** / control-plane class after parse).
4. Preserve grant evaluation, deny exit **3**, JSON envelopes, and bootstrap semantics unchanged.
5. Docs + help + hermetic locks match the soft-default story for the full `policy {show,check,bootstrap}` family.

## 2. Live baseline (re-scan 2026-08-11)

### 2.1 Operator dogfood (this machine)

| Command | Live result |
|---------|-------------|
| `policy show --format json` (authoritative `AI_BRAINS_PROJECT_ID` set) | exit **2** clap: `required arguments were not provided: --scope <SCOPE>` |
| `policy check --capability ReadEvidence --format json` | same clap exit **2** |
| `policy show --help` | `Usage: … policy show [OPTIONS] --scope <SCOPE>` + doc “required” |
| `policy bootstrap --help` | `Usage: … policy bootstrap [OPTIONS]`; `--scope` optional — soft-resolves when authoritative |
| Discovery lists / bootstrap with auth context | soft-fill (T203 / T210) |

### 2.2 Code truth

| Site | Role |
|------|------|
| `main.rs` `PolicyCommands::Show` / `Check` | `scope: String` clap-required |
| `main.rs` `PolicyCommands::Bootstrap` | `scope: Option<String>` optional |
| `policy_cmd.rs` `ShowOptions` / `CheckOptions` | `scope: String`; comment “clap guarantees --scope” |
| `policy_cmd.rs` `run_bootstrap` | `resolve_scope_key_for_cli(None, …)` when omitted |
| `governed_common.rs` `resolve_scope_key_for_cli` | SOOT helper — explicit wins → authoritative soft-fill → `Err` usage template |
| `exit_contract.rs` | `policy_show__missing_scope__exit_2` expects exit 2 only (no fail_usage template assert); `policy_show__help__scope_required` **locks clap-required Usage** |
| `policy_bootstrap.rs` AC8 | bootstrap omit + no context → fail_usage exit 2 (template) |
| `governed_discovery_reads.rs` AC4/AC5 | list soft-resolve success + fail_usage locks |

### 2.3 Docs honesty gap

| Doc | Claim today |
|-----|-------------|
| `Docs/CLI-EXIT-CODES.md` | Soft-resolve list = source/evidence/review; **still clap-required:** `policy show`, `erasure request` (peers include `policy check`) |
| `Docs/CAPABILITIES.md` | Soft-resolve on lists/show source|evidence; bootstrap optional scope; **no** show/check soft |
| `Docs/OPERATIONS.md` | Examples always pass `--scope` for show/check |
| `CHANGELOG` T201 | Documented BREAKING clap-required show — T226 is the deliberate soft-default follow-through for inspect paths only |

### 2.4 Deps / research pins

| Item | Pin / note |
|------|------------|
| clap workspace | `clap = "4.5"` → **Cargo.lock 4.6.1** |
| crates.io latest (2026-08-11) | clap **4.6.6** — **no pin bump** this track (optional `Option<String>` works on 4.5/4.6) |
| Zero new crates | Required — reuse `resolve_scope_key_for_cli` |
| Domain / migrations | None |
| Capture independence | Grant projection reads only for show; check uses evaluator; no models/embeddings |
| [clig.dev](https://clig.dev/) | Consistency across subcommands; conversation as norm (bootstrap → show → check without retyping scope); ease of discovery; prefer sensible defaults when context is known |
| T203 design lock | Soft-fill **only** when authoritative; non-authoritative never silent |

## 3. Research summary

| Finding | Application |
|---------|-------------|
| clig — consistency across subcommands | show/check must match bootstrap + discovery lists on `--scope` optionality |
| clig — conversation as norm | Operator flow: `policy bootstrap` then `policy show` then `policy check --capability …` without re-passing scope when env/cwd is authoritative |
| clig — make the default right for most users | Authoritative project id is the common dogfood case; clap-required is friction after T210 |
| T201 / T203 exit contract | Missing soft-resolve class = exit **2** `fail_usage`; clap English only for still-required surfaces (erasure, review resolve) |
| T210 F5 | Bootstrap already owns the pattern — copy, don’t reinvent |
| Least privilege | Soft-resolve does **not** change allow/deny; only fills scope key for inspection/check |
| Destructive asymmetry | Keep erasure wipe/request and review resolve clap-required (mutate / CE / ticket) |

## 4. Frozen decisions (F1–F24)

| ID | Decision |
|----|----------|
| **F1 — Surfaces** | Soft-resolve **only** `policy show` and `policy check`. Bootstrap already soft (no behavior change required; optional one-liner unify soft). |
| **F2 — Helper SOOT** | Use `resolve_scope_key_for_cli(options.scope.as_deref(), &identity)` for both paths (explicit wins; empty/whitespace treated as omit via helper trim). **No** `#[arg(env = …)]` on `--scope` (explicit = CLI flag only; `AI_BRAINS_SCOPE` denylist is ambient-strip for hermetic tests, not clap env binding). |
| **F3 — Fail class** | Omit + non-authoritative → `fail_usage` exit **2**, same template family as lists/bootstrap (`--scope` example, `scope resolve`, “not filled silently” / “not authoritative”). **Not** clap “required arguments were not provided”. **Not** exit **6**. |
| **F4 — Success soft-fill** | Omit + authoritative (`AI_BRAINS_PROJECT_ID` High / identity authoritative per existing helper) → fill scope key; proceed with current show/check logic. |
| **F5 — Explicit scope** | Always wins. Malformed explicit key → existing parse / `fail_cp` class (**6**), unchanged. |
| **F6 — Capability still required** | `policy check --capability` stays clap-required. Only `--scope` softens. |
| **F7 — Deny / allow + canonical scope (M1/M4)** | Exit **3** `POLICY_DENIED` + `details.hint` on deny; exit **0** allow; empty grants show exit **0**. **“Resolved scope key” = canonical** after `parse_scope_key` → `scope_identity_key` (mirror `run_bootstrap` ~227–232). Use that string in: human headers, deny messages, `CheckResult.scope`, and `list_applied_grants` query key. Soft-fill already returns canonical; explicit lowercase must not stay raw. |
| **F8 — Principal unchanged** | `resolve_principal` / `--principal-id` / env unchanged. |
| **F9 — Formats** | Json / Human / Markdown paths unchanged after scope resolution (aside from canonical scope strings). |
| **F10 — Clap types** | `scope: Option<String>` on Show + Check; doc comments “optional — soft-resolves when authoritative”. |
| **F11 — Help / after_help (L6)** | Usage optional; examples include omit-when-authoritative. Update **both** parent `policy` `after_help` sites (`main.rs` ~641 **and** ~1211) plus Show/Check command after_help. |
| **F12 — Still clap-required (out of track)** | `erasure request`, `erasure wipe`, `review resolve` (and any other mutate/CE surfaces). Do **not** soft-fill these in T226. **Retain** `assert_help_scope_required` for erasure (M3). |
| **F13 — No domain logic in CLI** | No grant evaluation changes; no new CP APIs; no migrations. |
| **F14 — No clap pin bump** | Stay workspace `4.5` / lock 4.6.1; clap 4.6.6 / clap 5 out of scope. |
| **F15 — Tests first (TDD)** | Red: flip exit_contract + add soft-resolve AC4/AC5 for show+check; Green: clap Option + resolve + canonicalize wire. |
| **F16 — Hermetic AC matrix (L4)** | See §6. Authoritative cases: prefer `hermetic_cmd` / `hermetic_cmd_with_ids` (`common/mod.rs` ~160–176) or equivalent `--no-project-context` + `.env("AI_BRAINS_PROJECT_ID", PROJECT)` — never rely on ambient workspace `.env` alone. |
| **F17 — Docs** | `CLI-EXIT-CODES.md`, `CAPABILITIES.md`, `OPERATIONS.md` (examples), `CHANGELOG` minor. Skill one-liner only if agent-facing policy section lies. Docs phase also greps CONTRIBUTING for stale clap-required claims (O5 soft). |
| **F18 — Contracts** | No DTO shape change. CLI-local only. Daemon IPC untouched. |
| **F19 — Capture independence** | No models/graph dependency. |
| **F20 — Parallel-friendly** | Touch `policy_cmd.rs`, clap Show/Check, exit_contract + policy soft tests, docs. Low conflict with T227+ if they avoid policy. |
| **F21 — Soft residual optional (L5)** | Refactor bootstrap scope resolve to single `resolve_scope_key_for_cli(options.scope.as_deref(), …)` — helper trim/empty-filter (~420) matches bootstrap inline (~214–219). Only if green stays green; not DoD. |
| **F22 — BREAKING honesty** | Softening clap-required → optional is **operator-friendly** (was exit 2 either way when missing). Document as UX fix / minor CHANGELOG; scripts that relied on clap English text for missing scope must accept fail_usage template (same class as T203 lists). |
| **F23 — Canonical SOOT (M1)** | After helper returns a key string, **always** `parse_scope_key` then rebind `scope_key = scope_identity_key(&scope_ref)` before any store query or user-visible string. Malformed → `fail_cp` / exit **6** class (AC7). |
| **F24 — Seeded soft-resolve proofs (O3)** | AC4/AC5 **seed** a grant via `open_seeded_ports` + `issue_grant` (mirror `governed_discovery_reads` AC4 ~362–380). Empty-grants-only is **not** sufficient DoD for soft-fill — prove the resolved key hits the right grant row. |

## 5. Non-goals

- Changing `DefaultPolicyEvaluator` / grant issue / revoke admin.
- Soft-resolve on **destructive** or **mutate** commands (erasure, review resolve, retention CE apply).
- Auto-bootstrap on show/check deny.
- Interactive prompts for scope.
- Daemon HTTP grant/show APIs.
- clap 5 migration.

## 6. Acceptance criteria

| ID | Criterion |
|----|-----------|
| **AC1** | `policy show` without `--scope`, `--no-project-context`, no `AI_BRAINS_PROJECT_ID` → exit **2**. Stderr **must** assert all three (M2): (a) `!contains("required arguments were not provided")`; (b) `contains("--scope")` **or** `contains("scope resolve")`; (c) `contains("not filled silently")` **or** `contains("not authoritative")`. Mirror `review_list__missing_scope__exit_2` + `review_list__non_authoritative__exit_2_fail_usage`. |
| **AC2** | `policy check --capability ReadEvidence` same omit/no-context → exit **2** with the **same three** stderr template asserts as AC1. |
| **AC3** | `policy show --help` and `policy check --help`: Usage does **not** hard-require `--scope`; prefer `usage_line.contains("[OPTIONS]")` (O4); help text soft-resolves when authoritative. **Do not** delete `assert_help_scope_required` — still used by erasure (M3). |
| **AC4** | Authoritative soft-fill + **seeded grant** (F24): omit `--scope` → `policy show --format json` exit **0** with **non-empty** grants for the seeded capability/scope. |
| **AC5** | Authoritative soft-fill + seeded grant for check: omit `--scope` → exit **0** `allowed: true`; JSON `scope` field equals canonical `Repository:<project>` (M4 — requires F23). |
| **AC6** | Explicit `--scope Repository:<uuid>` still works (regression of T160/T201 happy paths). |
| **AC7** | Malformed explicit `--scope not-a-key` → exit **6** class via `fail_cp` after `parse_scope_key` (net-new; L2). Not silent soft-fill. |
| **AC8** | `policy check` missing `--capability` (scope may be present or omit) still clap-required exit **2** with clap English **`required arguments were not provided`** expected (L1 — **opposite** of AC1/AC2). |
| **AC9** | Bootstrap AC8 + discovery AC5 still green (no regression). Explicit-scope deny/show tests in `exit_contract` / `governed_surface` / `policy_bootstrap` stay green. |
| **AC10** | Docs: CLI-EXIT-CODES lists show/check under soft-resolve; removes them from “still clap-required”; CAPABILITIES/OPERATIONS examples honest; CHANGELOG minor. |
| **AC11** | Full gate green; manual dogfood: with project context, `policy show` and `policy check --capability ReadEvidence` without `--scope` no longer print clap required-arguments text. |
| **AC12** | Lowercase explicit parity (O2/M1): `policy show --scope repository:<uuid>` returns same grants / canonical scope strings as `Repository:<uuid>` (proves F23 round-trip + store query key). |

## 7. API / UX contracts

### 7.1 CLI

```text
ai-brains policy show [--scope <SCOPE>] [--format …] [--principal-id …]
ai-brains policy check --capability <CAP> [--scope <SCOPE>] [--format …] [--principal-id …]
```

| Case | Exit | stderr / stdout |
|------|------|-----------------|
| Omit scope, non-authoritative | **2** | fail_usage template on **stderr** |
| Omit scope, authoritative | continue | (then show empty / check allow|deny as today) |
| Explicit good scope | continue | unchanged |
| Explicit bad scope | **6** class | parse error |
| Check deny | **3** | ApiError + hint (**canonical** scope in message) |
| Check allow / show empty|grants | **0** | JSON/human; `CheckResult.scope` and show headers use **canonical** key |

### 7.2 Unchanged

- Bootstrap discovery cap set; deny-by-default matrix.
- `POLICY_DENIED_HINT` text (may still show `--scope …` examples — fine).
- Erasure / review resolve clap-required.

## 8. Testing strategy

| Layer | What |
|-------|------|
| **Red first** | Flip `exit_contract` show missing-scope + help tests; add check twins; add soft-resolve success tests with **seeded grants**; AC7/AC8/AC12 net-new. |
| **Unit** | No new pure helper required if reusing SOOT. Shared `resolve_scope_or_fail_usage` wrapper = **O1 soft residual**, not DoD. |
| **Hermetic integration** | AC1–AC8, AC12 via `hermetic_bin` / `hermetic_cmd*` + tempdir vault; authoritative = F16 pattern. |
| **Regression** | Existing `policy_show__with_scope`, deny check, bootstrap suite, discovery suite, `assert_help_scope_required` erasure. |
| **Manual** | Live vault: omit scope with project id set → show/check succeed or deny (not clap). |

### 8.1 Suggested test names

```text
policy_show__missing_scope_no_context__exit_2_fail_usage   # AC1: 3 stderr asserts (M2)
policy_check__missing_scope_no_context__exit_2_fail_usage  # AC2
policy_show__help__scope_optional_soft_default             # AC3 + [OPTIONS] (O4); keep assert_help_scope_required for erasure (M3)
policy_check__help__scope_optional_soft_default
policy_show__authoritative_project_id__soft_resolve_seeded_exit_0   # AC4 F24
policy_check__authoritative_project_id__soft_resolve_seeded_allow   # AC5 F24+M4
policy_show__explicit_scope__still_works                   # AC6 keep existing
policy_show__malformed_explicit_scope__exit_6_class        # AC7 net-new (L2)
policy_check__missing_capability__clap_required_exit_2     # AC8 clap English expected (L1)
policy_show__lowercase_explicit_scope__canonical_grants    # AC12 O2
```

## 9. Implementation phases (on go)

1. **Red** — AC1–AC8 + AC12 tests fail on current tree (template asserts added, not rename-only).
2. **Green clap** — `Option<String>` + help/after_help on Show/Check **and both** parent sites (~641, ~1211).
3. **Green wire** — helper resolve → **F23 canonicalize** → messages / `CheckResult` / grant list key; drop “clap guarantees” comment.
4. **Docs** — CLI-EXIT-CODES, CAPABILITIES, OPERATIONS, CHANGELOG; optional CONTRIBUTING grep.
5. **Registry** — conductor, deferred strike, series README.
6. **Gate** — fmt/clippy/nextest/deny/audit + `ledgerful verify`; manual dogfood.
7. **Review** — internal + cross-model if FEATURE/UX treated high-risk (default: internal clean + Codex read-only for consistency).

## 10. Risk & blast radius

| Risk | Mitigation |
|------|------------|
| Scripts parse clap “required arguments” string | F22: document template change; exit still 2 |
| Soft-fill wrong scope under Ambiguous confidence | Helper already refuses Low/Ambiguous — reuse tests from T203 |
| Accidental soft-fill on erasure | F12 non-goal; do not touch ErasureCommands; keep help helper (M3) |
| Check capability typo vs missing scope confusion | Capability stays required; AC8 locks clap English |
| Explicit lowercase empty grants | F23 canonicalize before `list_applied_grants` (O2/AC12) |
| Bootstrap dual branch drift | F21 soft unify |

**Blast radius:** low. CLI-only + docs. No store schema. No daemon. No contracts DTO.

## 11. Manual verification (record on complete)

```powershell
# Non-authoritative (expect fail_usage exit 2, not clap required text)
ai-brains --no-project-context policy show --format json
ai-brains --no-project-context policy check --capability ReadEvidence --format json

# Authoritative (expect not clap; 0 empty grants or 3 deny or 0 allow after bootstrap)
ai-brains policy show --format json
ai-brains policy check --capability ReadEvidence --format json
ai-brains policy show --help
ai-brains policy check --help
```

## 12. Definition of Done

- [ ] AC1–AC12 met with hermetic + manual evidence
- [ ] No open critical/high; mediums fixed or ≤3 justified deferred in review.md + ISSUES/deferred
- [ ] Full CI gate green
- [ ] conductor.md T226 Completed; deferred row struck; series README updated
- [ ] Ledger committed; decisions pinned if non-obvious

## 13. Soft residuals (explicit non-DoD)

| Residual | Owner |
|----------|-------|
| Bootstrap scope branch → single helper call | F21 optional (L5) |
| Shared `resolve_scope_or_fail_usage` across evidence/source/review/policy | O1 optional |
| Soft-resolve success hermetic for **bootstrap** (T210 AC8 success soft) | leave soft / optional add-on |
| Soft-resolve `review resolve` / erasure | never without dedicated track + safety review |
| clap pin → 4.6.6 | chore track |
| Full grant admin | deferred T210 F24 lineage |
| CONTRIBUTING stale claims | O5 docs-phase grep only |

## 14. Open questions (resolve on go if needed)

None blocking. If product wants **all** policy subcommands listed in one CAPABILITIES soft-resolve sentence, do that in docs phase.

## 15. AI fold-in disposition (2026-08-11)

| ID | Sev | Disposition |
|----|-----|-------------|
| **M1** | Medium | **Hard** — F7/F23: `parse_scope_key` → `scope_identity_key` before messages, `CheckResult.scope`, and grant queries (bootstrap parity). |
| **M2** | Medium | **Hard** — AC1/AC2 require three stderr asserts (not exit-code-only rewrite). |
| **M3** | Medium | **Hard** — F12: retain `assert_help_scope_required` for erasure; flip only policy show/check help tests. |
| **M4** | Medium | **Hard** — AC5 canonical `scope` field depends on M1/F23. |
| **L1** | Low | **Hard** — AC8 net-new; clap English expected for missing `--capability`. |
| **L2** | Low | **Hard** — AC7 net-new malformed scope → 6. |
| **L3** | Low | **Absorbed by M1** — header vs grant line case consistency. |
| **L4** | Low | **Hard** — F16: `hermetic_cmd*` pattern. |
| **L5** | Low | **Soft** — F21 note trim parity; optional. |
| **L6** | Low | **Hard** — F11: both after_help sites. |
| **O1** | Opp | **Soft residual** — shared wrapper. |
| **O2** | Opp | **Hard** — AC12 lowercase explicit parity. |
| **O3** | Opp | **Hard** — F24 seeded grants for AC4/AC5. |
| **O4** | Opp | **Hard** — AC3 `[OPTIONS]` Usage guard. |
| **O5** | Opp | **Soft** — CONTRIBUTING grep in docs phase. |
| **O6** | Opp | **Hard note** — F2 no env binding on `--scope`. |
| **AI2** | — | Affirms design, AC matrix, docs targets; no additional mediums beyond AI1. |

**Verdict after fold-in:** ready for **go** (plan-only until user says go).
