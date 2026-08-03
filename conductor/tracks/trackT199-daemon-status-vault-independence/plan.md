# T199 Plan — Daemon Status Vault Independence

Status: **In Progress** (implement 2026-08-03). Spec: [spec.md](./spec.md).

## Preconditions

- [x] Draft objective (status without vault key)  
- [x] Live re-scan (AI2 6/6 confirmed)  
- [x] Expand F1–F24  
- [x] **AI fold-in** (AI1 affirm; AI2 M1–M7 + L2/L6–L8 + O1/O2; O4 declined) — §14; F25–F30; AC12–AC13  
- [x] Research pins (Status 1×300ms; doctor Safety; thin wrapper)  
- [x] Deferred roll-in  
- [ ] Pin freezes when vault key available (optional)  
- [ ] `ledgerful ledger start T199-DaemonStatusVaultIndependence --category FEATURE` *(on go)*

## Deferred rolled in

| Item | Disposition |
|------|-------------|
| `daemon status` requires vault key | **Absorb** |
| Divergent probes | **Absorb** SOOT + Status/Safety |
| Doctor on fast Status (earlier draft) | **Superseded by M6** — doctor stays **Safety** |
| Daemon silent zero | Honesty F16 only |
| service-only ACL | Soft F17 |
| Status JSON / inactive exit | **T201** |
| Graph install | **T200** |
| Unix PID | Declined O4 |

## Research pins (post fold-in)

| Fact | Pin |
|------|-----|
| Status probe | **1 × 300ms** single-shot |
| Safety probe | **3 × ≥1000ms** + 50ms backoff |
| Doctor policy | **Safety** (not Status) |
| Status policy | Interactive status only |
| Safety wrapper | `probe_restore_daemon_busy` stays in backup.rs |
| Early-route | **`run()` only**; `is_vault_path_free` untouched |
| Exit | **0** Running/Stopped/no key |
| tasklist | Soft-skip (no `?`) |
| Memories | `try_count_pinned_optional` + skip line; all `.ok()?` |
| Hermetic no-key | env_remove KEY + ALLOW (or hermetic_bin_no_key) |
| New crates | Zero |
| run_update probes | Leave direct `client.probe` |

## Phases

### Phase A — Design freeze ✅

- [x] A1–A3 freezes + boundaries  
- [x] **A4** AI fold-in M1–M7  

### Phase B — Shared probe (TDD)

- [x] **B0** Ledger start *(TX 34dd66f0 — orchestrator)*  
- [x] **B1** `daemon_probe` module: `DaemonProbePolicy`, **pub const** attempts/timeouts (F25), `probe_daemon_reachable`  
- [x] **B2** `probe_restore_daemon_busy` → thin Safety wrapper (imports unchanged)  
- [x] **B3** Doctor continues Safety path (wrapper) — no Status for doctor  
- [x] **B4** Unit AC5: Safety ≥3×1000ms; Status == 1×300ms  

### Phase C — Status vault independence

- [x] **C1** Early-route `DaemonCommands::Status` in `run()` (F3); not vault-path-free  
- [x] **C2** `run_status(StatusOptions)` — no AppContext  
- [x] **C3** Soft tasklist (F8/AC12); backends unchanged (F19 debt OK)  
- [x] **C4** Optional vault section F6/F7; `try_count_pinned_optional` (AC13)  
- [x] **C5** Hermetic no-key: env_remove KEY + ALLOW (F15) → AC1/AC2  
- [x] **C6** T128 stopped no vault lines; **new** AC7 running+no-key skip (F26)  
- [x] **C7** Confirm `run_update` probes untouched (F10)  

### Phase D — docs + gate

- [x] **D1** OPERATIONS: status no key; soft F16/F17  
- [x] **D2** CHANGELOG  
- [ ] **D3** Full gate  
- [ ] **D4** Review AC1–AC13  
- [ ] **D5** deferred strike; conductor Completed  
- [ ] **D6** Pin + ledger commit  

## Verification matrix

| AC | Proof |
|----|-------|
| AC1–AC2 | hermetic no-key F15 |
| AC3 | grep SOOT + Safety wrapper |
| AC4 | doctor_cli |
| AC5 | pub const unit |
| AC6–AC7 | smoke + new test |
| AC8–AC9 | docs |
| AC10 | full gate |
| AC11 | review update/start/stop |
| AC12 | tasklist soft-skip review |
| AC13 | try_count unit/review |

## Out of scope

- [ ] Daemon silent-zero product SOOT  
- [ ] Doctor Status policy (rejected M6)  
- [ ] Status JSON / non-zero Stopped (T201)  
- [ ] Graph install (T200)  
- [ ] Unix pgrep PID  
- [ ] Backend async rewrite  
- [ ] Dep bumps  

## Implement notes

1. **Order:** probe + consts + Safety wrap → early-route status → soft tasklist + memories → no-key tests → docs → gate.  
2. **High findings:** key still required; Safety weakened; hermetic still has key; tasklist `?`; memories `?` propagate.  
3. **Stop-before:** Safety weaken; daemon crypto SOOT as DoD; inactive exit ≠ 0.  
4. **After ship:** T200 and/or T201.  
5. **Fold-in note:** Earlier draft put doctor on Status for UX; **M6 supersedes** — doctor stays Safety.  

## Manual test checklist (on implement)

```powershell
Remove-Item Env:AI_BRAINS_KEY -ErrorAction SilentlyContinue
Remove-Item Env:AI_BRAINS_ALLOW_ZERO_KEY -ErrorAction SilentlyContinue
ai-brains daemon status
# Expect: Status: Running|Stopped; exit 0; not vault-key-only JSON
```
)
