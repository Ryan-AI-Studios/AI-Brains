# T226 — Policy show/check soft-resolve scope — Plan

**Status:** ✅ **Completed** 2026-08-11  
**Category:** UX / CONSISTENCY  
**Depends:** T160 · T201 · T203 · T210  
- [x] `ledgerful ledger start T226-policy-soft-resolve-scope` (tx `20c098ed-…`)

## Goal

Give `policy show` and `policy check` the same **optional `--scope` + authoritative soft-resolve** behavior as discovery lists and `policy bootstrap`, so operators with project context stop hitting clap “required arguments” friction. After resolve, **canonicalize** scope strings like bootstrap.

## Absorbed deferred

| Item | Disposition |
|------|-------------|
| deferred.md “policy show/check required scope” | **DoD** |
| Series README T226 (use 5 / quality 6) | **DoD** |
| T210 soft “AC8 success soft-resolve hermetic” | **Partial:** lock success soft-resolve for **show/check** (AC4/AC5 seeded); bootstrap success path remains soft optional |
| exit_contract help lock that **requires** clap-required Usage for show | **Flip** as part of DoD; **retain** helper for erasure (M3) |

**Not absorbed as DoD:** erasure request/wipe soft-scope; review resolve soft-scope; full grant admin; clap 5; pin bump to 4.6.6; bootstrap one-liner refactor (F21 soft); shared `resolve_scope_or_fail` (O1 soft); auto-bootstrap on deny.

## Research pins (2026-08-11)

| Pin | Evidence |
|-----|----------|
| Live clap wall | Dogfood: `policy show` / `policy check` omit scope → exit 2 + `required arguments were not provided` despite authoritative `AI_BRAINS_PROJECT_ID` |
| Bootstrap already soft | `scope: Option<String>` + `resolve_scope_key_for_cli` in `run_bootstrap` |
| Bootstrap canonicalizes | `parse_scope_key` → `scope_identity_key` (~227–232) — show/check do **not** today (M1) |
| Helper SOOT | `resolve_scope_key_for_cli` — soft-fill returns canonical; **explicit path returns raw** (no canonicalize) |
| Test lock against us | `policy_show__help__scope_required` asserts Usage must **not** contain `[--scope` |
| Missing-scope test weak | `policy_show__missing_scope__exit_2` asserts **exit code only** (M2) |
| Erasure shares helper | `assert_help_scope_required` used by show + `erasure_request__help__scope_required` (M3) |
| Docs lock against us | CLI-EXIT-CODES “Still clap-required after T203: policy show … policy check” |
| Deps | clap workspace **4.5** / lock **4.6.1**; crates.io latest **4.6.6** — **no bump** |
| clig | Consistency across subcommands; conversation bootstrap→show→check; sensible defaults when context known |

## AI fold-in pins (hard)

| ID | Pin |
|----|-----|
| **M1 / F23** | After helper: `parse_scope_key` → `scope_identity_key`; use **canonical** in messages, `CheckResult.scope`, and `list_applied_grants` |
| **M2 / AC1–2** | Missing-scope rewrite must add **3** stderr asserts — not exit-code-only rename |
| **M3** | **Retain** `assert_help_scope_required` for erasure; flip only policy help tests |
| **M4 / AC5** | JSON `scope` == `Repository:<project>` requires M1 |
| **L1 / AC8** | Missing `--capability` → clap English **expected** (opposite of scope fail_usage) |
| **L2 / AC7** | Net-new malformed `--scope not-a-key` → exit 6 class |
| **L4 / F16** | AC4/AC5 use `hermetic_cmd*` / `--no-project-context` + explicit `.env` PROJECT |
| **L6 / F11** | Parent after_help at **both** ~641 and ~1211 |
| **O2 / AC12** | Lowercase explicit `repository:uuid` same grants as `Repository:uuid` |
| **O3 / F24** | Seed grant for AC4/AC5 (empty-only not enough) |
| **O4 / AC3** | Help Usage contains `[OPTIONS]` |
| **O6 / F2** | No `#[arg(env)]` on `--scope` |

**Soft:** L5/F21 bootstrap unify; O1 shared wrapper; O5 CONTRIBUTING grep.

See `spec.md` §15 full disposition.

## Frozen decision index

See `spec.md` §4 **F1–F24**. Hard summary:

1. Soft-resolve **show + check only** (F1).  
2. Reuse `resolve_scope_key_for_cli` (F2).  
3. Non-auth omit → fail_usage exit 2, not clap text (F3).  
4. Auth omit → fill and continue (F4).  
5. Explicit wins; bad explicit → 6 class (F5).  
6. `--capability` stays required (F6).  
7. **Canonical** scope everywhere after parse (F7/F23/M1).  
8. Erasure / review resolve stay clap-required; keep help helper (F12/M3).  
9. Seeded soft-resolve proofs (F24/O3).  
10. TDD red→green; docs honesty (F15/F17).

### Suggested wire sketch (pin on go)

```rust
// run_show / run_check — after AppContext + ports:
let raw_key = match resolve_scope_key_for_cli(options.scope.as_deref(), &ports.identity_store()) {
    Ok(k) => k,
    Err(msg) => return fail_usage(msg),
};
let scope_ref = match parse_scope_key(&raw_key) {
    Ok(s) => s,
    Err(e) => return fail_cp(format, e),
};
let scope_key = scope_identity_key(&scope_ref); // F23 — always canonical
// use scope_key in messages, CheckResult, list_applied_grants, policy.allow
```

## Task checklist

### 0. Preflight (on go)

- [ ] `ledgerful doctor` + `ledgerful ledger status --compact`
- [ ] `ledgerful ledger start T226-policy-soft-resolve-scope --category FEATURE --message "policy show/check soft-resolve scope"`
- [ ] `ledgerful scan --impact` after identifying touch files
- [ ] Confirm no pending TX / unexpected dirty tree

### 1. Red — hermetic tests (M2/M3/L1/L2/O2–O4)

- [ ] Rewrite `policy_show__missing_scope__exit_2` → AC1 **three** stderr asserts (M2); rename to `…__fail_usage`
- [ ] Add `policy_check__missing_scope_no_context__exit_2_fail_usage` with same three asserts (AC2)
- [ ] Flip `policy_show__help__scope_required` → optional soft-default + `[OPTIONS]` (AC3/O4)
- [ ] **Retain** `assert_help_scope_required` for `erasure_request__help__scope_required` (M3)
- [ ] Add `policy_check__help__scope_optional_soft_default`
- [ ] Add AC4 show soft-resolve with **seeded grant** via `open_seeded_ports` + `issue_grant` (F24)
- [ ] Add AC5 check soft-resolve seeded allow + assert `scope == Repository:{PROJECT}` (M4)
- [ ] Keep/extend explicit scope happy path (AC6)
- [ ] Add AC7 malformed explicit scope → exit 6 (net-new L2)
- [ ] Add AC8 missing `--capability` → clap English `required arguments were not provided` (L1)
- [ ] Add AC12 lowercase explicit scope grant parity (O2)
- [ ] Confirm red: nextest on these tests fails before production change

### 2. Green — clap + wire (M1/F23/L6)

- [ ] `PolicyCommands::Show.scope` / `Check.scope` → `Option<String>`; update doc + command after_help
- [ ] Parent policy after_help **both** sites (~641 **and** ~1211): omit-when-authoritative examples (L6)
- [ ] `ShowOptions` / `CheckOptions` → `Option<String>`
- [ ] `run_show` / `run_check`: resolve → **canonicalize** → messages / CheckResult / grant query (M1)
- [ ] Optional soft F21: bootstrap single helper call (only if green stays green)

### 3. Docs

- [ ] `Docs/CLI-EXIT-CODES.md` — move show/check into soft-resolve list; remove from still-required; erasure peers still required
- [ ] `Docs/CAPABILITIES.md` — policy show/check soft-resolve sentence
- [ ] `Docs/OPERATIONS.md` — optional-scope examples for show/check
- [ ] `CHANGELOG.md` minor UX entry
- [ ] Skill touch only if agent policy section still claims required scope for show
- [ ] Soft O5: grep CONTRIBUTING for stale show/check required claims

### 4. Registry + deferred

- [ ] Strike deferred.md T226 row as closed (on complete)
- [ ] Update series README + conductor.md Completed (on complete)
- [ ] Append soft residuals if any (O1/F21)

### 5. Verify

- [ ] `cargo fmt --check`
- [ ] `cargo clippy --workspace --all-targets -- -D warnings`
- [ ] `cargo nextest run -p ai-brains-cli` (at least exit_contract, policy_*, governed_*)
- [ ] Full gate: nextest workspace + deny + audit
- [ ] `ledgerful verify --scope fast` then full before finalize
- [ ] Manual dogfood (spec §11) — record output snippets in plan or review

### 6. Review + finalize

- [ ] `conductor/tracks/trackT226-policy-soft-resolve-scope/review.md`
- [ ] Internal clean; Codex read-only if FEATURE flagged
- [ ] `ledgerful ledger commit` with evidence
- [ ] Pin: `ai-brains pin "DECISION: T226 policy show/check soft-resolve + canonical scope_identity_key; erasure stays clap-required"`

## Touch map (expected)

| Path | Change |
|------|--------|
| `crates/ai-brains-cli/src/main.rs` | Show/Check `Option<String>`; help text × parent sites |
| `crates/ai-brains-cli/src/commands/policy_cmd.rs` | options + resolve + **canonicalize** wire |
| `crates/ai-brains-cli/tests/exit_contract.rs` | flip missing-scope + help; AC7/AC8; keep erasure helper |
| `crates/ai-brains-cli/tests/policy_bootstrap.rs` and/or new soft tests | AC4/AC5 seeded; AC12 |
| `Docs/CLI-EXIT-CODES.md` | soft-resolve inventory |
| `Docs/CAPABILITIES.md` | policy soft sentence |
| `Docs/OPERATIONS.md` | examples |
| `CHANGELOG.md` | minor |
| `conductor/*` | registry |

## Out of scope reminders

- Do **not** change `ErasureCommands` scope types.
- Do **not** delete `assert_help_scope_required`.
- Do **not** change grant matrix or bootstrap discovery set.
- Do **not** bump clap.

## Manual evidence (2026-08-11)

```
# Debug binary (PATH install may lag until cargo install)
target\debug\ai-brains.exe policy show --help
  → Usage: … policy show [OPTIONS]
  → --scope optional — soft-resolves when authoritative
  (not: Usage … --scope <SCOPE> required)

# Hermetic non-authoritative (AC1/AC2 nextest): exit 2 fail_usage, not clap text
# Live cwd with project/.env may soft-fill even with --no-project-context when
# AI_BRAINS_PROJECT_ID or git identity is authoritative — expected helper behavior.

target\debug\ai-brains.exe policy show --format json
  → exit 0, {"api_version":"1","grants":[…]}  (no clap required-arguments)
```

## Closeout

- PR: **#130** squash-merged `5919f26`
- Gate: fmt/clippy OK; nextest workspace **2534 passed** (1 skipped); deny/audit/ledgerful full green; CI Win/Linux/macOS green
- Soft residual: O1 shared `resolve_scope_or_fail_usage`; T210 bootstrap success soft hermetic optional
- Reviews: internal CLEAN; Codex R1 FAIL→fix; R2 **PASS WITH DEFERRED P3** (process); final after closeout
