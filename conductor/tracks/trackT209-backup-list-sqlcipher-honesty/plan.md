# T209 Plan — Backup list SQLCipher honesty

Status: **In Progress / Implemented** (ledger open; orchestrator closeout). Spec: [spec.md](./spec.md).

## Absorbed

| Residual | Disposition |
|----------|-------------|
| Audit backup list **4/3** | F2–F9 + F31 |
| T208 “T209 backup WARN flood” | Closed when this ships |
| T120/T134 incomplete post-T187 | F4/F5/F8 extend |
| AI2 live plain WARN repro | §2.1 confirmed |
| M1 missing/empty | Corrupt footnote |
| M2 KeyMismatch vs Corrupt | **F31 size ≥ 512** |
| M3 doctor API | **ListMode::Quiet** |
| M4 dual flags | Runtime quiet wins |
| M5 hermetic RUST_LOG | F16 AC1 env_remove / AC2 warn |
| L1/L2 class + eprintln | F14 / F6 required |

## Research (2026-08-04)

| Source | Takeaway |
|--------|----------|
| Live + AI2 | Plain bak → key verification WARN; encrypted salt ≠ plain |
| Error strings | Same for garbage vs wrong-key → size heuristic F31 |
| rusqlite | 0.39.0 keep; 0.40.1 out of scope |
| tracing / subscriber | 0.1.44 / 0.3.23 — keep |
| T208 denylist | RUST_LOG stripped in hermetic_bin |
| clig.dev | Expected residual → debug + summary |
| Zetetic | Header-first; post-key schema read |

## AI fold-in (2026-08-04)

| ID | Source | Action |
|----|--------|--------|
| **AI1 #1–#4** | Header, classes, table, flags | **Affirm** |
| **M1** | missing/empty → Corrupt | **Accept** |
| **M2** | Discriminator | **Accept F31** size≥512 |
| **M3** | ListMode | **Accept** |
| **M4** | No conflicts_with | **Accept** |
| **M5** | AC1/AC2 env | **Accept F16** |
| **L1** | BackupInfo.class | **Required** |
| **L2** | eprintln summary | **Required F6** |
| **L3** | Serialize | Soft F24 |
| **L4** | header redundancy | Decline |
| **L5** | verify regression | Affirm F13 |
| **L6** | ISSUES.md | Soft F35 |
| **AC9** | Soft → **required** | F34 |

## Phases

### A0 — Expand + fold-in (done)

- [x] Code map + prior tracks  
- [x] Spec F1–F35 + AC1–AC10  
- [x] AI fold-in §14  
- [x] Conductor/deferred Planning + fold-in note  
- [x] On **go**: `ledgerful ledger start T209-backup-list-sqlcipher-honesty --category FEATURE --message "Classify backup list plain/key/corrupt; ListMode; default quiet summary; --verbose; table tokens"`  
- [x] On go: `ledgerful scan --impact`  

### A1 — Red (TDD)

- [x] **B1** Unit: plain header → `LegacyPlain` (no key success required)  
- [x] **B2** Unit: size &lt; 512 garbage → `Corrupt`  
- [x] **B3** Unit: size ≥ 512 key-fail → `KeyMismatch`  
- [x] **B4** Unit: openable meta → `Readable`  
- [x] **B5** Hermetic AC1: plain + `env_remove("RUST_LOG")` → no per-file WARN + `(legacy plain)`  
- [x] **B6** Hermetic AC2: short garbage + `RUST_LOG=warn` → per-file WARN  
- [x] **B7** Hermetic AC3: multi plain → ≤1 eprintln summary  
- [x] **B8** Hermetic AC4/AC5: verbose / quiet / both  
- [x] **B9** AC6 pre-T109 smoke green  
- [x] **B10** AC7 table tokens  
- [x] **B11** AC9 large wrong-key → summary, not N WARNs  

### B — Green

- [x] **C1** `BackupReadClass` + `ListMode` + `from_flags` + `BackupInfo.class` (soft Serialize)  
- [x] **C2** Header-first classify + F31 size gate  
- [x] **C3** `list_backups(ListMode)` noise: Corrupt `warn!`; expected `debug!`; counts for summary  
- [x] **C4** clap `--verbose`; doctor `ListMode::Quiet`; **no** conflicts_with  
- [x] **C5** `run_list` tokens + **eprintln!** summary (F6)  

### C — Docs + closeout

- [x] **D1** CAPABILITIES §11 + CHANGELOG (AC8)  
- [x] **D2** Soft OPERATIONS one-liner  
- [x] **D3** Review + brain/cli nextest → full gate (2081 nextest; clippy/deny/audit green)  
- [x] **D4** PR #92 squash-merged `02a0d7d`; conductor Completed; deferred strike; soft L3/L4 residuals only 

## Test plan

| Lock | Assert |
|------|--------|
| Unit plain | `LegacyPlain` |
| Unit &lt;512 | `Corrupt` |
| Unit ≥512 key-fail | `KeyMismatch` |
| AC1 | unset RUST_LOG; no per-file WARN; `(legacy plain)` |
| AC2 | RUST_LOG=warn; Corrupt WARN present |
| AC3 | ≤1 summary line |
| AC4–5 | verbose / quiet / dual |
| AC6 | pre-T109 |
| AC7 | tokens |
| AC9 | large wrong-key summary only |

Suite: `backup_list_honesty.rs` + brain classify units.

## Manual (on go)

- [ ] Post-encrypt-only clean  
- [ ] Plain residual tokens + summary  
- [ ] Garbage WARN  
- [ ] Flag matrix  
- [ ] Verify plain refuse  
- [ ] Soft doctor  

## Stop-before

- Auto-delete plain  
- rusqlite 0.40  
- Restore/verify rewrite  
- T210  
- Implement without **go**  

## Done when

AC1–AC9 green; AC10 soft; review clear; gate green; residual closed.

## Implement notes

1. **Header first** — never KeyMismatch plain.  
2. **F31:** `MIN_PLAUSIBLE_BACKUP_BYTES = 512`.  
3. **ListMode::from_flags(quiet, verbose)** — quiet wins.  
4. Doctor + brain tests use new API in same PR.  
5. Summary = **eprintln!**; Corrupt = **tracing::warn!**.  
6. AC1: `env_remove("RUST_LOG")` only for default-filter proof.  
7. Pin summary substrings; PowerShell `;` not `&&`.  
