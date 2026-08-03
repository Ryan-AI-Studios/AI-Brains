# T198 Plan — Empty States + Exit Hygiene

Status: **Implement complete (pending D3–D6 gate/review/commit)** (2026-08-03). Spec: [spec.md](./spec.md). Ledger TX: `c85543c8-f36f-4c46-ac71-f12e5357c38e`.

## Preconditions

- [x] Audit register + T122 amend  
- [x] Live re-scan (backup/project/dogfood/graph/fingerprint)  
- [x] Expand F1–F24  
- [x] **AI fold-in** (AI1 M1–M7/L1–L8/O2; AI2 affirm; exit **2** not 1; 15 dogfood sites) — §14; F25–F28; AC10–AC11  
- [x] Pin fold-in  
- [x] `ledgerful ledger start T198-EmptyStatesExitHygiene --category FEATURE` *(TX c85543c8…)*

## Deferred rolled in

| Item | Disposition |
|------|-------------|
| Empty / silent / graph exit 0 | **Absorb** |
| Graph default install | **T200** |
| Full exit matrix | **T201** |
| forget/symbol-bridge tracing empty | **T202/T203** |
| daemon status w/o key | **T199** |

## Research pins (post fold-in)

| Fact | Pin |
|------|-----|
| Graph exit | **EXIT_USAGE = 2** |
| FEATURE_UNAVAILABLE | string; helper → 2 |
| dogfood | **all silent emitted sites** → fail_api Json |
| evaluate hard-gate | **exclude** |
| fingerprint | Ok(()) + list copy; exit 0 |
| VerifyOutput | status + optional message (skip if none) |
| Graph stubs | **both** cfg sites |
| New crates | Zero |

## Phases

### Phase A — Design freeze ✅

- [x] A1–A3 freezes + T122/T200 boundary  
- [x] **A4** AI fold-in M1–M7  

### Phase B — dogfood + empty success (TDD)

- [x] **B0** Ledger start  
- [x] **B1** Inventory all dogfood `GovernedCliError::emitted` sites; convert silent ones to `fail_api` (F4/F25)  
- [x] **B2** backup verify empty human + JSON (F5)  
- [x] **B3** project list empty line (F6)  

### Phase C — graph + fingerprint

- [x] **C1** Both graph stubs → exit 2 + FEATURE_UNAVAILABLE (F2/F24)  
- [x] **C2** Flip smoke test: code == 2; help stays 0 (F11)  
- [x] **C3** Soft: `exit_code_for_api_error` map FEATURE_UNAVAILABLE → 2 (O2)  
- [x] **C4** fingerprint empty → Ok + bootstrap stdout (F7)  
- [x] **C5** Soft: project detect + context (F8)  

### Phase D — docs + gate

- [x] **D1** CHANGELOG: graph exit 0→2  
- [x] **D2** Soft OPERATIONS (skipped — soft only; CHANGELOG covers break)  
- [x] **D3** Full gate (local 1907 nextest; CI windows/linux/macos all SUCCESS)  
- [x] **D4** Review AC1–AC11 (internal R1 CLEAN ×2; Codex final **PASS**)  
- [x] **D5** deferred strike; conductor Completed (PR #81 `5cc0418`)  
- [x] **D6** Pin + ledger commit

## Verification matrix

| AC | Proof |
|----|-------|
| AC1–AC5 | hermetic |
| AC3 inventory | grep zero silent dogfood |
| AC4 | code==Some(2) |
| AC10 | both stubs |
| AC11 | helper unit |
| AC7–AC9 | regression / changelog / gate |

## Out of scope

- [ ] T200 default graph install  
- [ ] T201 full codes  
- [ ] T199 daemon status  
- [ ] T202/T203 tracing empties as DoD  
- [ ] evaluate hard-gate rewrite  
- [ ] verify JSON count field  

## Implement notes

1. **Order:** dogfood inventory → verify empty → project list → graph exit 2 + test → fingerprint → soft detect → CHANGELOG.  
2. **High findings:** silent dogfood remaining; graph exit 0/1; fingerprint still Err; blank verify.  
3. **Stop-before:** install flip; full exit matrix.  
4. **After ship:** T199 and/or T200.  
)
