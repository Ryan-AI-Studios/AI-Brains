# T184 Plan — Independent Security Review (P12.6)

Status: **Execute complete** (2026-08-01) — Codex R2 **PASS WITH DEFERRED P3**; local gate green; PR merge pending.

## Track shape

| Phase | Work | Code? | Status |
|-------|------|-------|--------|
| **A** | Research + residual seed + expansion + AI fold-in | No | Done |
| **B** | Charter freeze + packet index + evidence/ | Docs | Done |
| **C** | Automated baseline + independent review pass | Read-only + logs | Done |
| **D** | Remediate critical/high; residual register; closeout | Code + docs | Done |
| **E** | T183 packet verification + residuals↔claims cross-check | Docs | Done |

## Ledgerful preflight

- [x] `ledgerful doctor` — ready
- [x] `ledgerful ledger status --compact` — clean at start
- [x] `ledgerful search "bearer"` / content envelope — security surface hits
- [x] `ledgerful hotspots` — daemon/CLI forget/sync priority list
- [x] At implement: `ledgerful ledger start T184-security-review --category SECURITY` → `2ad44ef4-d800-4c43-8d78-a3498f109c0e`
- [x] Phase C: re-ran hotspots for code audit order
- [ ] At close: `ledgerful ledger commit` + optional pin (after PR green)

## Phase A — Expansion (complete)

- [x] Inventory shipped surfaces
- [x] Online research SSDF/ASVS/OpenSSF
- [x] Seed residual register
- [x] Soft T183 dependency
- [x] Expanded spec/plan/charter/residuals
- [x] AI1/AI2 fold-in A1–A13

## Phase B — Charter + packet

- [x] Create `evidence/` directory
- [x] Freeze `charter.md` (Sync=**Y**, Desktop=**Y**; reviewer + re-verifier named)
- [x] Fresh reviewer session (read-only subagent)
- [x] `evidence/PACKET.md` (root CHANGELOG.md path)
- [x] Packet secret scan — no secrets found
- [x] Residual seed includes R-CI-* and expanded items

## Phase C — Baseline + independent review

- [x] `cargo deny check` + `cargo audit` → `evidence/DENY-AUDIT.md`
- [x] Ignored/unmaintained advisory disposition
- [x] Soft Scorecard file inspection (no CLI required)
- [x] Targeted suites: ai-brains-security, ai-brainsd lib, sync t178_l5
- [x] Design/doc review ADRs + threat models + T183 pack
- [x] Code pass by hotspots + charter surfaces
- [x] CI hygiene walk → R-CI-* dispositions
- [x] Findings in `review.md` schema
- [x] Wall-clock within time-box

## Phase D — Remediation + closeout

- [x] F-1 High fixed (pipe SDDL SY+BA+IU)
- [x] Separate re-reviewer R2 → `verified_fixed`
- [x] Residuals updated; F-6 deferred T186; F-8 admin residual
- [x] Mediums fixed or deferred (1 medium deferred ≤3)
- [x] Full gate (local) before PR
- [x] Closeout paragraph for T185 (no cert language)
- [x] AC1–AC10; conductor Completed; §62 promoted

## Phase E — Claims / residual consistency

- [x] Re-read SECURITY-LIMITS + CLAIMS-CROSSCHECK + F8
- [x] Cross-check residuals ↔ claims → `evidence/RESIDUALS-CLAIMS-CROSSCHECK.md`
- [x] Claim-vs-product gaps dispositioned (F-3/F-4/F-9/F-10)
- [x] SECURITY.md timeline (R-DISCLOSURE-TL closed)
- [x] CHANGELOG path = repo root

## License gate

- [x] Prefer cross-model / human review without AGPL tools
- [x] deny/audit first line
- [x] Scorecard optional Apache-2.0 only
- [x] SBOM generators → T185 preferred

## Stop-before checklist

- [x] No “security reviewed” marketing without charter + disposition
- [x] No ASVS/SOC2/ISO certification claims
- [x] No live user vault secrets in packet
- [x] No weaponized open Critical PoCs committed
- [x] No AGPL-required scanner as DoD
- [x] No claiming #12 / F8 / perfect deletion closed by review alone
- [x] Do not conflate L9 no-cert with blocking future SLSA provenance (T185)
- [x] CI findings: fixed easy (permissions, Dependabot); SHA pins → T186
