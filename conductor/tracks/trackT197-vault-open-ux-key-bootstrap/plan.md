# T197 Plan — Vault Open UX + Key Bootstrap

Status: **Expanded + AI fold-in** (plan-only 2026-08-02). Spec: [spec.md](./spec.md). Series: [README-T197-T204-CLI-UX.md](../README-T197-T204-CLI-UX.md).

## Preconditions

- [x] Draft objective + AC from CLI audit  
- [x] Re-scan open paths + silent zero defaults + rusqlite features  
- [x] Expand freezes F1–F28 + research (community SQLCipher / config_log)  
- [x] Roll deferred: CLI spam + key bootstrap  
- [x] **AI fold-in** (AI1 affirm; AI2 M1–M7, L1–L7, O3–O4; O1/O2 future) — disposition spec §14; F29–F32; AC10–AC13  
- [ ] Pin fold-in decision (`ai-brains pin`)  
- [ ] `ledgerful ledger start T197-VaultOpenUxKeyBootstrap --category FEATURE` *(on go only)*

## Deferred rolled in

| Item | Disposition |
|------|-------------|
| CLI SQLCipher spam + key bootstrap | **Absorb** |
| Silent zero default UX | **Absorb** (F2) — all 7 sites |
| Doctor vs recall lock inconsistency | **Absorb** |
| Full exit matrix | **T201** (codes partial F8) |
| daemon status w/o vault | **T199** |
| Empty states | **T198** |
| MSI / R-CI-BRANCH | **Out** |

## Research pins (2026-08-02 + fold-in)

| Fact | Pin |
|------|-----|
| Spam **primary** | `rusqlite::trace::config_log` + feature `trace` |
| Spam **secondary** | Commercial `cipher_log_level` only (optional no-op on community) |
| Missing key | **Missing** — no silent zero |
| Resolver | `ai-brains-cli` + **`try_from_raw`** |
| Sites | **7** incl. migrate + shadow |
| install() | store OnceLock; CLI + daemon main + windows_service + tests |
| Doctor | skipped (missing) vs fail (wrong) |
| JSON codes | VAULT_KEY_MISSING / FORMAT / ZERO / VAULT_LOCKED |
| init | **generate + print once** |
| dotenv | project `.env` + `~/.ai-brains/.env` (existing) |
| rusqlite | 0.39.x + `trace`; no 0.40 unless blocked |

## Phases

### Phase A — Design freeze (plan-only ✅)

- [x] **A1** Freezes F1–F28  
- [x] **A2** Message family + codes  
- [x] **A3** Shared resolve + no silent zero  
- [x] **A4** Spam control research  
- [x] **A5** AI fold-in M1–M7 / L1–L7 → F1 rewrite, F9–F11, F19, F27, F29–F32, AC10–13  

### Phase B — Inventory + shared resolve + log policy (TDD)

- [x] **B0** Ledger start FEATURE *(TX `177eecdd-446e-4bec-b769-d4611484154a` — orchestrator owns commit)*  
- [x] **B1** Inventory: (1) all 7 resolve sites; (2) tests that pass `None` / rely on silent zero — hermetic_bin sets explicit zero+ALLOW; migrate/shadow/daemon fixtures TempEnv ALLOW  
- [x] **B2** Enable rusqlite workspace feature **`trace`**  
- [x] **B3 Red→Green:** `sqlcipher_log_policy::install()` OnceLock + filter + `cipher_log_level=NONE` (F1/F26/F27/F29)  
- [x] **B4 Red→Green:** `resolve_operator_sqlcipher_key` + `KeyResolveError` variants + unit matrix  
- [x] **B5 Red→Green:** wire **all 7** sites; JSON code map at CLI edge  
- [x] **B6** Wrong-key stderr capture: no `hmac check failed`  

### Phase C — Doctor + init + command family

- [x] **C1** Doctor F9: missing → skipped; wrong → fail; no spam  
- [x] **C2** AppContext paths consistent (recall/preflight/project)  
- [x] **C3** init F19: generate + print once when key absent  
- [x] **C4** Soft: daemon main + windows_service install() (F11)  

### Phase D — Docs + tests

- [x] **D1** INSTALL bootstrap (PS + bash + dotenv paths)  
- [x] **D2** OPERATIONS env expand  
- [x] **D3** doctor help link  
- [x] **D4** Hermetic AC1–AC8 / AC10–AC12  
- [ ] **D5** Manual live wrong-key note if free  

### Phase E — Gate + closeout

- [x] **E1** Full gate *(targeted: store+cli+daemon nextest 676 pass; clippy -D warnings)*  
- [x] **E2** Review vs AC1–AC13 *(implementer self-check)*  
- [x] **E3** CHANGELOG Unreleased  
- [ ] **E4** deferred strike; conductor Completed *(orchestrator)*  
- [ ] **E5** Pin DECISION; ledger commit *(orchestrator)*  

## Verification matrix

| AC | Proof |
|----|-------|
| AC1 no spam | process + install() |
| AC2 family | tests |
| AC3 format pre-open | unit try_from_raw |
| AC4 missing | unit + process |
| AC5 docs | review |
| AC6 hermetic suite | nextest |
| AC7 secrets/gate | review |
| AC8 zero/wrong | tests |
| AC9 gate | process |
| AC10 7 sites | grep |
| AC11 doctor skipped vs fail | tests |
| AC12 JSON codes | unit |
| AC13 install sites | review |

## Out of scope checklist

- [ ] DPAPI auto-unlock  
- [ ] Password managers  
- [ ] Bare-hex auto-wrap  
- [ ] TTY prompt default  
- [ ] T198 empty states  
- [ ] T199 daemon status  
- [ ] T201 full exit matrix  
- [ ] MSI / App Store / R-CI-BRANCH  
- [ ] Disable page HMAC  
- [ ] Commercial cipher_log as DoD  
- [ ] doctor --deep integrity  
- [ ] rusqlite 0.40 bump as DoD  

## Implement notes (for go-ahead)

1. **Order:** B1 inventory → trace feature + install() → shared resolve → 7 sites → doctor F9 → init F19 → docs → tests.  
2. **High findings:** F32 (silent zero, spam, secrets, HMAC off, migrate/shadow left, missing/wrong conflated).  
3. **Stop-before:** crypto redesign; T199/T201 creep; MSI.  
4. **After ship:** T198 + T199 per series README.  
)
