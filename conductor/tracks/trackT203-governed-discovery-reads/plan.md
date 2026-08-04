# T203 Plan — Governed Discovery Read Paths

Status: **Completed** (2026-08-04). Spec: [spec.md](./spec.md). PR #86 squash-merged `2748d12`.

## Absorbed from deferred / prior tracks

| Residual | Disposition |
|----------|-------------|
| Governed source/evidence discovery lists | **This track** |
| Soft scope resolve (T202 F12 / deferred residual) | **This track** F4–F6 |
| T201 F27 soft-default boundary | **Honor** — no exit-6 reintroduction |
| evidence/source show missing-scope exit 6 (T201 F36 leave) | Prefer **F7** absorb |
| project list JSON (T198 O4) | Soft **not** DoD → T204/later |
| Progressive auto project fill | Soft F16 |
| Help IA / OutputFormat TTY matrix | **T204** |

## AI fold-in (2026-08-04)

| ID | Action in plan |
|----|----------------|
| M1 | B3: `items` + `#[serde(alias = "sources")]`; AC9 dual deserialize |
| M2 | D3: exit_contract path flip; AC4 env with PROJECT_ID; AC5 unset; hermetic_bin notes |
| M3 | C2: helper `Result<String,_>`; fail_usage at call site; no-project-context still resolves |
| M4 | B2: lift `sanitize_fts_query` **or** CLI pre-sanitize — **no** CP→retrieval dep |
| M5 | B2: LIMIT+1 more_available; AC15 |
| M6 | D4: replace scope None arms at source.rs:~59, evidence.rs:~55 |
| L2/L3/L5/L7 | C1 search subcommand; F29 help; list deny hint; Active default + AC16 |

## Phases

### A0 — Inventory & freezes

- [x] Live re-scan + research  
- [x] Expanded F1–F36 + AC1–AC14  
- [x] AI fold-in → F1–F42 + AC15–AC16  
- [x] On go: `ledgerful ledger start`; re-grep `scope: Option` inventory  

### A1 — Contracts + ports (Red first)

- [x] B0: failing tests list_sources / list_evidence (empty, happy, scope filter, Active filter, limit+more_available)  
- [x] B1: `EvidenceListRow` + port methods + `ports_are_implementable` mock  
- [x] B2: adapters SQL — Active default; LIMIT+1; FTS JOIN; **sanitizer path (M4)**  
- [x] B3: contracts — List*Request; EvidenceListResponse; SourceListResponse **items + alias sources (M1)**  
- [x] B4: daemon-api variants + protocol wire + alias deserialize test  

### B — CLI list surfaces

- [x] C1: clap `source list`, `evidence list`; soft `evidence search` **requires** `--query` (L3)  
- [x] C2: `resolve_scope_key_for_cli` (F6/M3) — return key; call site `fail_usage`  
- [x] C3: local list + `ReadEvidence` + `policy_denied_hint_details()` (L5)  
- [x] C4: daemon list paths  
- [x] C5: human empty `(none)` + json emit + more_available  

### C — Review soft-default + show residual

- [x] D1: reopen `review list --scope` → `Option<String>`  
- [x] D2: soft-fill authoritative; else `fail_usage` exit 2  
- [x] D3 (**M2 highest risk**):  
  - Update `review_list__missing_scope__exit_2` → assert **fail_usage** stderr (F33), not clap required text  
  - **AC4:** hermetic with `AI_BRAINS_PROJECT_ID` set / seeded project → exit 0 without `--scope`  
  - **AC5:** hermetic PROJECT_ID unset (+ `--no-project-context` as needed) → exit 2  
  - Document `hermetic_bin()` env: does **not** assume clap path; shell PROJECT_ID can make resolve authoritative  
- [x] D4 (**M6**): show helper insertion before `parse_scope_key` / `expand_handle`; soft deny hint F42  

### D — Docs, gate, closeout

- [x] E1: CAPABILITIES / OPERATIONS / CLI-EXIT-CODES / CHANGELOG  
- [x] E2: deferred.md strike; conductor Completed  
- [x] E3: primary review CLEAN → Codex R1 FAIL → fix → Codex R2 **PASS**  
- [x] E4: full gate 2009; CI green; PR #86 squash-merge `2748d12`; ledger commit; pin 

## Test plan (minimum)

| Lock | Assert |
|------|--------|
| AC1–AC3 | list empty/happy/FTS |
| AC4–AC5 | soft-resolve success vs fail_usage exit 2 (env-controlled) |
| AC6 | deny + hint |
| AC7–AC8 | limit + scope isolation |
| AC9 | protocol + serde alias |
| AC15 | more_available |
| AC16 | Active-only default |
| Soft AC13 | show missing-scope |

## Order / deps

1. Sanitizer lift decision (M4) if FTS in adapters  
2. Ports + contracts (TDD)  
3. Daemon protocol  
4. CLI list + helper  
5. Review soft-default + exit_contract (M2)  
6. Show F7  
7. Docs + gate  

Prefer after **T202** (done — `fail_usage` at `governed_common.rs:182`). Coordinate `main.rs` if **T204** concurrent.

## Stop-before

1. Reintroduce CLI exit-**6** for missing-scope class.  
2. Unbounded list/FTS dump.  
3. Cross-scope leakage.  
4. Models/embeddings on discovery path.  
5. control-plane depends on `ai-brains-retrieval`.  
6. Removing daemon defensive None arms without honesty doc.  
7. Scope expands into grant admin / observe / CE.

## Manual checklist (on ship)

```powershell
ai-brains scope resolve --format json
ai-brains source list --format json
ai-brains source list --scope 'Repository:…' --format json
ai-brains evidence list --format json
ai-brains evidence list --query 'keyword' --scope 'Repository:…' --format json
ai-brains review list --format json
# non-authoritative context → exit 2
ai-brains review list --format json; echo $LASTEXITCODE
```

## Notes

- Workspace clap **4.5** (crates.io 4.6.5 — **no bump**).  
- `SourceListResponse` rename: **alias required** (M1), not CHANGELOG-only.  
- Highest effort: **M2 hermetic env** + **M4 sanitizer lift**.  
- Preflight/recall may fail without vault key in agent env — non-blocking for plan docs.  
