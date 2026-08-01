# T183 Plan — Release Documentation Pack (P12.5)

Status: **Completed** (2026-08-01). Docs/release track — markdown only; no production crate work.

## Track shape

| Phase | Work | Code? |
|-------|------|-------|
| **A** | Gap matrix + research freeze + AI fold-in | No |
| **B** | Index + INSTALL + SECURITY hub + SECURITY.md stub + CHANGELOG | Docs |
| **C** | Topic elevation + F8 audit + drift banners + demotions | Docs |
| **D** | Claims cross-check + install + link-check evidence + closeout | Docs + manual |

## Ledgerful preflight (expansion day)

- [x] `ledgerful doctor` — ready (completion may be unreachable)
- [x] `ledgerful ledger status --compact` — 0 pending / 0 drift
- [x] `ledgerful search "OPERATIONS"` — index seed
- [x] `ledgerful hotspots` — CLI hotspots; avoid unrelated code churn
- [x] At implement: `ledgerful ledger start T183-release-docs --category DOCS --message "Release documentation pack"` (tx `2af69491-1758-40a2-b454-f23dca8219d9`)
- [x] Prefer **search + file read** over `ask` while completion model down
- [x] At close: `ledgerful ledger commit` + optional pin for pack entry decision

## Phase A — Expansion (complete)

- [x] Inventory Docs/* + root README; note missing Docs/README + CHANGELOG + SECURITY.md
- [x] Confirm doctor CLI absent; recovery export absent; F8 overclaims in elevated docs
- [x] Online research: Diátaxis, Keep a Changelog, Good Docs Project, honesty culture
- [x] Roll deferred: T179–T182 handoffs, status staleness, Implementation-Plan §8
- [x] Write expanded `spec.md` + `plan.md`; conductor + deferred §61
- [x] **AI1/AI2 fold-in** → spec §15 (A1–A15)

## Phase B — Core pack skeleton

- [x] Create `Docs/README.md` (Diátaxis map, seven topics, non-claims, version banner, **Research/Historical** section)
- [x] Create `Docs/INSTALL.md` (Windows-first; full §6 locks incl. F8, graph flag, three sync meanings, doctor/kit honesty)
- [x] Create **`Docs/SECURITY-LIMITS.md`** one-page hub (ADR-0016/0018/0019, COMPATIBILITY F8, RECOVERY-DRILLS, cloud, connectors)
- [x] Create root **`SECURITY.md`** stub → `Docs/SECURITY-LIMITS.md` (GitHub Security tab)
- [x] Create root `CHANGELOG.md` (Keep a Changelog; **0.x SemVer note**; **[Unreleased]** seeded with P12 milestones)
- [x] Link COMPATIBILITY, PROTOCOL-COMPAT, RECOVERY-DRILLS, ADRs from index/hub

## Phase C — Topic elevation + honesty + drift

- [x] Topic 2 provenance: CAPABILITIES/ARCHITECTURE user-facing pointers (no false Independent claims)
- [x] Topic 3 agent permissions: OPERATIONS policy/grants + “UI no extra authority”
- [x] Topic 4 correction/review: OPERATIONS review + WORKFLOWS recipe link
- [x] Topic 5 erasure: SECURITY-LIMITS → OPERATIONS erasure honesty + RECOVERY-DRILLS E residual
- [x] Topic 6 cloud: allow_cloud default false; Sealed/local-strict; no cloud-required capture
- [x] Topic 7 sync: optional; ADR-0018 + OPERATIONS multi-device residuals; ACK ≠ wipe proof; three sync names
- [x] Document **doctor DTO without CLI** + **recovery export absence** next to recovery docs
- [x] **F8 elevated audit (required):** reword README “SQLCipher-backed”; ARCHITECTURE “SQLCipher-encrypted” + **“SQLCipher (Full encryption)”**; CAPABILITIES SQLCipher vault lines → COMPATIBILITY F8 honesty
- [x] Root `README.md`: pack links + F8 fix
- [x] **`status.md` demote only:** banner “Frozen at T72…” **Do not** full-refresh body
- [x] **OPERATIONS banner:** replace entire line-5 stale claim
- [x] **Implementation-Plan.md §8:** drift banner
- [x] Soft: banner or archive `Docs/Audit.md` + `Docs/audit2.md`
- [x] Light `CAPABILITIES.md` related-docs row update
- [x] Soft: INSTALL one-liner T186 CI hygiene roadmap pointer

## Phase D — Claims + evidence + closeout

- [x] Write `evidence/CLAIMS-CROSSCHECK.md` as **2-column table** + F8 grep results
- [x] Manual install walkthrough → `evidence/INSTALL-WALKTHROUGH.md`
- [x] Relative link check → `evidence/LINK-CHECK.md` (+ soft `check-links.ps1`)
- [x] Grep new + elevated prose for forbidden claims
- [x] AC1–AC12 checked in `spec.md`
- [x] Conductor → Completed; deferred §61 struck/promoted
- [x] Docs-only: no binary change claim
- [x] Full close after internal R2 + Codex R1 (final Codex R2 gate on ship)

## License gate

- [x] Markdown in-repo; mermaid OK
- [x] No AGPL doc toolchain required
- [x] Third-party quotes attributed if used
- [x] Product license statements unchanged (PolyForm NC + commercial exception)

## Stop-before checklist

- [x] Do not document non-existent doctor / recovery-export CLIs as shipped
- [x] Do not claim SQLCipher page-level / “full encryption” without F8 qualifier
- [x] Do not claim WASI/plugin sandbox or third-party marketplace
- [x] Do not claim NIST Purge / perfect deletion / SOC2
- [x] Do not force Unix CLI→HTTP-only narrative
- [x] Do not expand MSI/App Store as T183 DoD
- [x] Do not full-refresh status.md (demote only)
- [x] Do not make CONTRIBUTING.md DoD
