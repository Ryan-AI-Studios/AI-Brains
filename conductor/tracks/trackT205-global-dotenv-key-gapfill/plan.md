# T205 Plan — Global dotenv KEY gap-fill

Status: **Completed** 2026-08-04 (PR #88 `6a7fd15`). Spec: [spec.md](./spec.md).

**Seed WIP:** always-merge + INSTALL + skill. Implementing F11 + AC suite + docs.

## Absorbed

| Residual | Disposition |
|----------|-------------|
| KEY skipped when VAULT_PATH set | F1 seed |
| Global skipped under no-project-context | F3 |
| skill/INSTALL under-documented | F13 seed partial |
| T113 non-override | F4 |
| Daemon silent zero | F14 out |

## AI fold-in (2026-08-04)

| ID | Action |
|----|--------|
| **M1** | **B6 first on go:** `hermetic_bin_no_key` HOME isolation; fix helper doc; re-green vault_key_bootstrap + doctor missing-key |
| **M2** | B8: shadow.rs / migrate.rs comment freshness |
| **M3** | C2: CAPABILITIES F29 one-liner (not vague “document matrix”) |
| **M4** | B1–B4: new `tests/global_dotenv_key_gapfill.rs`; pattern smoke.rs ~2498 |
| **M5** | B5: re-verify `apply_local_project_context_env` PROJECT/SESSION only |
| **L1** | Soft C6: warn on global parse Err |
| **L4** | Soft C7: `.gitignore` `vault.db.plain-*` |
| **L5** | C3: skill trim if rewrite beyond key/dotenv |
| **L6** | C5: verify INSTALL VAULT_KEY note |

## Phases

### A0 — Expand + fold-in (done)

- [x] Expanded F1–F28  
- [x] AI fold-in → F1–F36 + AC12  
- [x] On go: `ledgerful doctor`; `ledgerful ledger start T205-GlobalDotenvKeyGapfill --category FEATURE --message "Always-merge global dotenv KEY gaps + hermetic HOME isolation"`  
- [x] On go: `ledgerful scan --impact`  

### A1 — Unblock F11 then prove (order matters)

- [x] **B6 (M1 first):** Harden `hermetic_bin_no_key` — empty `USERPROFILE`+`HOME` tempdir; update doc (global still merges); re-run F31 failing tests → green  
- [x] **B1** Red→Green AC1 (KEY only global + vault-path)  
- [x] **B2** AC2 shell KEY wins  
- [x] **B3** AC3 project KEY wins  
- [x] **B4** AC4 no-project-context still gap-fills  
- [x] **B5** Confirm seed main.rs F1–F4; M5 apply_local scope  
- [x] **B7** T113 existing tests green  
- [x] **B8** M2 comment freshness shadow/migrate  

### B — Docs

- [x] **C1** INSTALL verify seed  
- [x] **C2** CAPABILITIES F29 exact one-liner (+ ~372 if stale)  
- [x] **C3** Skill key-home; trim broad rewrite (L5)  
- [x] **C4** CHANGELOG minor  
- [x] **C5** Soft: OPERATIONS / INSTALL daemon VAULT_KEY verify-only  
- [x] **C6** Soft: global parse warn (L1)  
- [x] **C7** Soft: gitignore `vault.db.plain-*` (L4)  

### C — Closeout

- [x] **D1** Primary review CLEAN (F11 evidence)  
- [x] **D2** Full gate; PR; conductor Completed; deferred T205 strike  
- [x] **D3** Ledger commit; optional pin always-merge decision  
- [x] **D4** Never commit secrets / `vault.db.plain-*`  

## Test plan

| Lock | Assert |
|------|--------|
| AC6/F11 | missing-key green with real developer global KEY present |
| AC1–AC4 | new suite + HOME isolation |
| AC5 | T113 smoke |
| AC12 | helper doc honesty |

## Manual

- [x] Global KEY only + `--vault-path` → doctor  
- [x] Shell KEY overrides bad global  
- [x] `--no-project-context` still uses global KEY  

## Stop-before

- Ship without F11/B6  
- Daemon dotenv rewrite as DoD  
- `from_path_override` for project/global  
- Commit plain vault dumps  

## Done when

AC1–AC12 green; F11 proven on developer machine with global KEY; review clear; PR merged.
