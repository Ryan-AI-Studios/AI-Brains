# T210 Plan — Governed policy bootstrap for discovery

Status: **Completed** (2026-08-05, PR #93 `d52df25`). Spec: [spec.md](./spec.md).

## Absorbed

| Residual | Disposition |
|----------|-------------|
| Audit source/evidence/review/briefing POLICY_DENIED (3–4) | F1–F12, F21, AC1–AC11 |
| T160 grant mutation deferred | Partial: bootstrap only |
| T203 “grant admin” decline | Partial: discovery set only (F2) |
| Hint → `policy show` dead-end | F12 + AC7/AC11 |
| Empty vs deny (list hard / briefing soft) | F28 docs |
| Live vault: grants `[]`, all Read* deny | Confirmed §2.1 |
| AI2 M1–M5 | F7, F33, sketch, AC11, F37 |

## Research (2026-08-04)

| Source | Takeaway |
|--------|----------|
| Live CLI | source/review list exit 3; briefing soft denied; policy show empty; no grant CLI |
| CP `issue_grant` / `register_principal` | Ready SOOT; Actor::System; Privacy on event |
| `active_grants` vs `list_applied_grants` | **F7 uses active_grants** (typed); list_applied = show/briefing path only |
| Partial unique grant index | Idempotency probe mandatory |
| clig.dev | Setup conversation; suggest next cmd; dry-run; tell user on state change |
| Least privilege (RBAC 2025–26) | Discovery reads only; no Approve/Erase |
| OWASP / T151 | Keep deny default; explicit grant |
| clap workspace | Pin **4.5** (resolves ~4.6.x); latest 4.6.5 — no pin bump |
| device bootstrap | Naming precedent for first-run enable |

## AI fold-in (2026-08-04)

| ID | Source | Action |
|----|--------|--------|
| **AI1 #1–#5** | Dead-end, least privilege, idempotent, dry-run, invariants | **Affirm** |
| **M1** | `active_grants` for F7 | **Accept** |
| **M2** | `get_principal` DoD before register | **Accept** F33 |
| **M3** | Explicit probe→issue sketch | **Accept** |
| **M4** | Daemon hint unit AC11 | **Accept** |
| **M5** | clap env on principal_id | **Accept** F37 |
| **L1–L5** | soft-resolve, format json, CLI-local DTO, fail_cp, INSTALL | **Affirm** |
| **L6** | ISSUES.md absent | Soft → deferred.md |

## Phases

### A0 — Expand + fold-in (this session)

- [x] Live re-scan POLICY_DENIED + empty grants  
- [x] Spec F1–F40 + AC1–AC11  
- [x] Plan phases + RED checklist  
- [x] AI fold-in §14 + plan table  
- [x] Conductor/deferred Planning + fold-in note  
- [x] On **go**: `ledgerful ledger start T210-governed-policy-bootstrap --category FEATURE --message "policy bootstrap discovery grants; register principal; active_grants idempotent; dual-site POLICY_DENIED hint; docs"`  
- [x] On go: `ledgerful scan --impact` before production edits  

### A1 — Red (TDD)

- [x] **B1** Hermetic AC1: seeded vault + scope, no grants → `policy check ReadEvidence` exit 3  
- [x] **B2** Hermetic AC2: `policy bootstrap --scope …` exit 0 + three caps; `registered: "registered"`  
- [x] **B3** Hermetic AC3: three `policy check` allow exit 0  
- [x] **B4** Hermetic AC4: `source list` + `review list` exit 0 (items may be empty)  
- [x] **B5** Hermetic AC5: second bootstrap `already_present` + `registered: "already"`, exit 0  
- [x] **B6** Hermetic AC6: dry-run does not create grants / no append  
- [x] **B7** AC7: CLI hint contains `bootstrap`  
- [x] **B8** Soft AC8: soft-resolve scope path  
- [x] **B9** AC11: daemon unit `POLICY_DENIED_HINT` contains `bootstrap`  

### B — Green

- [x] **C1** Clap `PolicyCommands::Bootstrap { scope: Option, dry_run, principal_id (env PREFLIGHT), format default json }`  
- [x] **C2** `run_bootstrap` per sketch below (resolve_principal → get_principal → register? → scope → active_grants → issue)  
- [x] **C3** CLI-local JSON/human response + next-command human hint  
- [x] **C4** Update `POLICY_DENIED_HINT` (CLI) + daemon `services.rs` twin + AC7/AC11  
- [x] **C5** after_help examples  
- [x] **C6** No contracts DTO (F19 freeze)  

### C — Docs

- [x] CAPABILITIES policy bootstrap  
- [x] OPERATIONS / INSTALL post-init bootstrap step  
- [x] CLI-EXIT-CODES remediation text  
- [x] CHANGELOG minor  
- [ ] Skill one-liner if agent-facing  

### D — Review + gate

- [x] Targeted nextest (cli policy_bootstrap + daemon AC11) + clippy  
- [x] Manual live vault bootstrap → lists (dry-run on live scope OK)  
- [x] Regression: show/check/init unchanged; T203 governed_discovery_reads green  
- [x] Internal review log  
- [x] Cross-model soft (dual-site hint) — Claude PASS (Codex rate-limited)  
- [x] Full gate; ledger commit; conductor Completed; deferred strike (post-PR)  

## Implementation sketch (normative flow — M3)

```text
policy bootstrap [--scope KEY] [--dry-run] [--principal-id UUID] [--format json|human]
  → principal = resolve_principal(options.principal_id.as_deref())   // F4; env via clap F37
  → ports = StorePorts::from_store(SqliteEventStore::new((*ctx.conn).clone()))
  → scope_key = match options.scope:
        Some(s) => s
        None => resolve_scope_key_for_cli(None, &ports.identity_store())  // F5/F39
                 or fail_usage(2)
  → scope_ref = parse_scope_key(&scope_key).map_err(fail_cp)?          // F11
  → grant_store = ports.grant_store()
  → registered_status:
        if dry_run:
          get_principal → Some => "already" | None => "would_register"
        else:
          match get_principal(principal.id)? {                        // F33 DoD
            Some(_) => "already"
            None => { register_principal(...); "registered" }
          }
  → active = grant_store.active_grants(principal.id, &scope_ref)?     // F7 M1
  → for cap in [ReadEvidence, ReadConclusions, ReadDecisions]:       // F2
        if active.iter().any(|g| g.capability == cap):
            status = already_present
        else if dry_run:
            status = would_issue
        else:
            grant_id = issue_grant(writer, clock, principal.id, scope_ref.clone(),
                                   cap, Privacy::LocalOnly)?          // F6
            status = issued
  → emit CLI-local PolicyBootstrapResponse (api_version "1"); exit 0  // F10/F19
```

**Reuse:** `issue_grant`, `register_principal`, **`active_grants`**, **`get_principal`**, `resolve_principal`, `resolve_scope_key_for_cli`, `parse_scope_key`, `fail_usage` / `fail_cp` / `emit_json`, `SystemClock`.

**Do not:** change DefaultPolicyEvaluator; auto-grant on init; bootstrap Erase/Approve; `list_applied_grants` for idempotency; re-register without probe; new migration; domain logic in CLI beyond thin wiring; contracts DTO this track.

## Manual test script (on go)

```powershell
# After vault key available and project context authoritative:
ai-brains policy show --scope "Repository:<uuid>" --format json
ai-brains policy bootstrap --scope "Repository:<uuid>" --format json
ai-brains policy bootstrap --scope "Repository:<uuid>" --format json   # second = already
ai-brains policy check --capability ReadEvidence --scope "Repository:<uuid>"
ai-brains source list --format json
ai-brains review list --format json
ai-brains briefing project --format json
# Expect: lists exit 0; briefing without denied warning (or grants path allows sections)
```

## Stop-before

- Full grant admin / revoke productization  
- Auto-grant on init  
- Approve*/Erase in bootstrap set  
- Scope creep into T211+  

## Out of scope (explicit)

- MSI, clap 5, multi-tenant IdP, daemon IssueGrant wire (soft F25), revoke CLI (F26), contracts DTO (F19 freeze CLI-local)
