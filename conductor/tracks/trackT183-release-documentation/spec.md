# T183 — Release Documentation Pack (P12.5)

- **Track ID:** T183-ReleaseDocumentation
- **Phase:** P12 — Release hardening and adoption (Task 5)
- **Status:** ✅ **Completed** (2026-08-01) — release documentation pack shipped; F8 elevated honesty; evidence for T185
- **Depends on:** P0–P11 + P12.1–P12.4 shipped behaviors (T179–T182 complete); soft: T184/T185 consume claims language
- **Blocks / feeds:** T185 claims checklist (non-claims inventory); T184 reviewer packet index; operator adoption
- **Category:** DOCS / RELEASE
- **Master plan:** P12.5 — user-facing pack: install, provenance, agents, review, erasure limits, cloud, sync threat
- **Stop-before:** Marketing overclaims; SOC2/ISO/GDPR “certified”; inventing `doctor` / `recovery export` product CLIs; AGPL-required doc toolchains; claiming page-level SQLCipher / plugin sandbox / perfect deletion / metadata-private sync
- **Deferred absorbed:** T179 install handoff (HANDOFF-T183-T185); T180 PROTOCOL-COMPAT handoff (api_version honesty); T181 residuals (document kit export / doctor **absence** honestly); T182 ADR-0019 non-claims for connectors; COMPATIBILITY §10; RECOVERY-DRILLS residual pointers; status.md staleness; **F8 SQLCipher overclaims in elevated docs**; Implementation-Plan §8 phantom CLI surface. **Not** MSI/notarization product (T185 packaging); **not** implement doctor/recovery-export CLIs; **not** #34.2 DataKey rotation; **not** CONTRIBUTING.md as DoD.
- **Review fold-in:** AI1 BS1–3 + Opp1–2; AI2 F-1..F-11 + O-1..O-8 → **A1–A15**. See §15.

## 1. Objective

Ship a **coherent, user-facing documentation pack** for a distributable developer product that is:

1. **Task-oriented** for install and first vault (Windows-first; secondary OS per T179).  
2. **Accurate** against live CLI/daemon/protocol (elevate existing docs; **fix elevated F8 overclaims**).  
3. **Honest about limits** — erasure, sync, cloud, connectors, recovery kit, encryption surface.  
4. **Navigable** from a single index — stop scattering readers across orphan files.  
5. **Version-aware** — product version + changelog discipline for release.

This track answers **what operators and developers must read** to adopt safely. It does **not** implement missing product features (`doctor`, `recovery export`, MSI).

| After T183 | Present |
|------------|---------|
| `Docs/README.md` index (Diátaxis + Research/Historical section) | Target |
| Install how-to (Windows T1 + Unix honesty + graph/sync notes) | Target |
| Seven master-plan topics covered with links | Target |
| `Docs/SECURITY-LIMITS.md` one-page hub + root `SECURITY.md` stub | Target |
| `CHANGELOG.md` Keep a Changelog + 0.x SemVer note | Target |
| Root `README.md` doc links + **F8-honest** vault wording | Target |
| Elevated docs F8 audit (README, ARCHITECTURE, CAPABILITIES) | Target |
| `status.md` **demoted** historical (not full refresh) | Target |
| Implementation-Plan §8 drift banner | Target |
| Claims cross-check (2-column claim / non-claim + F8 grep) | Target |
| Manual install path + relative link check evidence | Target |
| `ai-brains doctor` product CLI | **No** (DTO may exist; CLI absent — document both) |
| `recovery export` CLI | **No** (honest residual; kit library + RECOVERY-DRILLS) |
| `CONTRIBUTING.md` | **No** (non-goal; PolyForm NC + track-based workflow) |
| Marketing site / Docusaurus required | **No** |

## 2. Live baseline (re-scan 2026-08-01 + fold-in)

### 2.1 Existing doc surface (do not orphan)

| Doc | Role today | Gap for release pack |
|-----|------------|----------------------|
| Root `README.md` | Quick start + partial doc links | Missing pack links; **line 9 “SQLCipher-backed” overclaim (F8)** |
| **No** `Docs/README.md` | — | **Need** single TOC / sitemap + Research/Historical section |
| **No** `CHANGELOG.md` | — | **Need** Keep a Changelog + Unreleased |
| **No** root `SECURITY.md` | — | GitHub Security tab won’t find `Docs/SECURITY-LIMITS.md` alone |
| `Docs/OPERATIONS.md` | Large ops reference | Banner stale: “17 top-level subcommands”, “T44–T71” (now ~34 cmds / P12) |
| `Docs/WORKFLOWS.md` | How-to recipes | Index link + secondary OS notes |
| `Docs/CAPABILITIES.md` | Feature inventory (v0.1.1) | F8 SQLCipher wording; related-docs row |
| `Docs/ARCHITECTURE.md` | Explanation | **“SQLCipher (Full encryption)”** — strongest F8 overclaim |
| `Docs/COMPATIBILITY.md` | Platform tiers + **F8 SOT** | Normative vault encryption honesty — **copy, don’t paraphrase** |
| `Docs/PROTOCOL-COMPAT.md` | N−1 / api_version honesty | T180 handoff |
| `Docs/RECOVERY-DRILLS.md` | Backup/kit/CE drills | Kit export / doctor residual pointers |
| `Docs/DECISIONS/ADR-0016..0019` | Crypto CE, sync, desktop, connectors | Cite non-claims |
| `Docs/Implementation-Plan.md` §8 | Historical CLI Surface v2 | **~15 phantom commands** (doctor, unlock, recovery export, …) — adoption hazard |
| `Docs/status.md` | **Stale** T72 / 2026-06-02 | **Demote** (do not full-refresh — re-drifts) |
| `Docs/Audit.md`, `audit2.md` | Stale audits | Banner or archive (F8 + “build broken” claims) |
| Hook research / `Docs/RESEARCH/` | Historical | Index under Research/Historical only |

### 2.2 F8 SQLCipher honesty (normative SOT)

**SOT:** `Docs/COMPATIBILITY.md` F8 — vault = **bundled SQLite** + application-level Content Envelope **AES-256-GCM** + OS permissions; **page-level SQLCipher is feature-gated / not live** until feature+CI. Confirmed by `Docs/Deviations.md` §1 and workspace `rusqlite` `bundled` (not `bundled-sqlcipher`).

**Known elevated overclaims to fix (implement Phase C):**

| File | Example claim | Required reword direction |
|------|---------------|---------------------------|
| `README.md` | “SQLCipher-backed append-only history” | Bundled SQLite + CE; SQLCipher page-level feature-gated (F8) |
| `ARCHITECTURE.md` | “SQLCipher-encrypted”; **“SQLCipher (Full encryption)”** | Same; never “full encryption” without F8 qualifier |
| `CAPABILITIES.md` | “SQLCipher append-only log” / vault rows | Same |

### 2.3 Product version & toolchain (live)

| Item | Value |
|------|--------|
| Workspace version | **0.1.1** (`Cargo.toml`) — **0.x SemVer:** minor bumps may break |
| Rust pin | **1.95.0**, target `x86_64-pc-windows-msvc` |
| License | PolyForm NC 1.0.0 + `COMMERCIAL-EXCEPTION.md` |
| CLI surface | ~34 top-level commands; **no** Doctor / RecoveryExport / unlock / lock / install-hooks as planned in Implementation-Plan §8 |
| `contracts::doctor` | DTO exists; **no** `ai-brains doctor` CLI |
| Graph feature | `graph` subcommand requires `--features graph`; capture/recall without it |
| “Sync” name collision | `ai-brains sync` = Ledgerful bridge; `safety sync` = hotspot pin; multi-device = **`replicate`** (not sync) |

### 2.4 Ledgerful preflight (planning day)

| Check | Result |
|-------|--------|
| `ledgerful doctor` | Ready; completion model may be unreachable |
| `ledgerful ledger status` | 0 pending, 0 unaudited drift |
| `ledgerful search "OPERATIONS"` | Index seed |
| `ledgerful hotspots` | CLI dominates — docs track should not churn those unless doc-code drift |
| Semantic `ask` | Prefer hybrid search + file read |

### 2.5 Gaps T183 closes

1. No single **Docs entry point** (index).  
2. Install path not polished Windows-first **how-to** with T179 honesty + graph/sync notes.  
3. Seven master-plan topics not mapped as a pack.  
4. Security limits need **hub** + GitHub-discoverable **root SECURITY.md**.  
5. No **CHANGELOG**.  
6. Root README / status / OPERATIONS banners **stale or overclaiming**.  
7. **Elevated F8 SQLCipher overclaims** (not only new prose).  
8. Implementation-Plan §8 **phantom CLI** drift.  
9. Residual honesty for missing doctor / recovery-export.  
10. Explicit **claims cross-check** for T185 (including existing elevated docs).

## 3. Research summary (online + standards, 2026-08-01)

### 3.1 Documentation architecture — Diátaxis

[Diátaxis](https://diataxis.fr/) splits content into four roles:

| Diátaxis type | User need | AI-Brains placement |
|---------------|-----------|---------------------|
| **Tutorial** | Learning path, first success | Soft: elevate root README quick start into INSTALL “first memory” (optional; Diátaxis tutorial quadrant) |
| **How-to** | Goal-oriented steps | `Docs/INSTALL.md` (new), `WORKFLOWS.md` |
| **Reference** | Accurate lookup | `OPERATIONS.md`, `CAPABILITIES.md`, CLI `--help`, PROTOCOL-COMPAT, COMPATIBILITY |
| **Explanation** | Understanding | ARCHITECTURE, ADRs, vision, threat models |

**Rule:** Do not turn ADRs into how-tos. Index puts Research/Historical **away** from primary install path.

### 3.2 Install / adoption guides

Practice sources: [Diátaxis how-to](https://diataxis.fr/) + [The Good Docs Project](https://thegooddocsproject.dev/) template pack ([gitlab.com/tgdp/templates](https://gitlab.com/tgdp/templates) v1.6.0 “Iron” — install-guide template; structure may evolve; use principles, not brittle URLs).

1. Prerequisites first (OS tier, toolchain, privileges).  
2. Minimal happy path to a working vault **before** advanced daemon/service.  
3. Verification step.  
4. Platform variants after primary (Windows T1 first).  
5. Failure modes / residual risks near keys and service install.  
6. Link deep reference — do not duplicate entire OPERATIONS.

### 3.3 Release notes / changelog

| Practice | Application |
|----------|-------------|
| **[Keep a Changelog](https://keepachangelog.com/en/1.1.0/)** 1.1 | `CHANGELOG.md` with Added/Changed/Deprecated/Removed/Fixed/**Security**; human prose |
| **Unreleased** section | Recommended at top; seed with major P12 milestone bullets (control plane, sync, compatibility, recovery drills, ADR-0019) |
| SemVer note | “AI-Brains follows Semantic Versioning. **While at 0.x, minor version bumps may include breaking changes.**” |
| Version banner | Manual from `Cargo.toml`; update when version changes; **CI version-sync check is T185 scope** (not T183 DoD) |
| **Common Changelog** later | Optional stricter style; **incompatible** on Security/Deprecated categories and Unreleased (would fold Security into Changed/Fixed and drop Unreleased). Keep a Changelog is enough for v1. |

### 3.4 Security & privacy documentation honesty

| Topic | Honest framing |
|-------|----------------|
| Cryptographic erasure | Envelope-backed live vault; offline/pre-erase backups remain recoverable; not NIST Purge/Destroy |
| Sync | Optional; untrusted relay; metadata residual; ACK ≠ wipe proof (ADR-0018) |
| Cloud models | Opt-in; `allow_cloud` default false; Sealed/local-strict (T157) |
| Connectors | TrustedBuiltin only (ADR-0019); no plugin sandbox claim |
| Encryption at rest | **COMPATIBILITY F8 exact wording** — no page-level SQLCipher claim |
| Multi-user machine | Loopback HTTP bearer residual; LocalSystem service token residual |

**Forbidden in pack (new + elevated prose T183 touches):** “certified,” “perfect deletion,” “metadata-private sync,” “sandboxed third-party plugins,” “SQLCipher page encryption” / “Full encryption” without F8 qualifier, invented doctor/recovery-export CLI as shipped.

### 3.5 Dependencies / tooling (docs track)

| Tool | Decision |
|------|----------|
| In-repo **Markdown** | **Required** |
| Mermaid | Prefer for diagrams |
| mdbook / Docusaurus / MkDocs | **Optional soft** — not DoD; MIT/Apache only if added later |
| AGPL doc generators | **Forbidden** as required |
| New Rust crates | **None** |
| Link check | Soft script or manual `Test-Path` on relative targets (§12) |

## 4. Design locks

| # | Lock |
|---|------|
| L1 | **Single entry index** at `Docs/README.md` (Diátaxis map + seven topics + non-claims + **Research/Historical** section). |
| L2 | **Elevate, don’t orphan** — OPERATIONS, CAPABILITIES, COMPATIBILITY, PROTOCOL-COMPAT, RECOVERY-DRILLS, ADRs remain SOTs for their domains. |
| L3 | **Windows-first install how-to** (`Docs/INSTALL.md`); secondary OS per T179 handoff. |
| L4 | **Seven master-plan topics** each have a primary home + secondary links (§5). |
| L5 | **Security limits hub** = dedicated short `Docs/SECURITY-LIMITS.md` (one-page executive summary linking ADR-0016/0018/0019, COMPATIBILITY F8, RECOVERY-DRILLS) **plus** root **`SECURITY.md` stub** pointing to it (GitHub Security policy tab). |
| L6 | **`CHANGELOG.md`** Keep a Changelog skeleton; Unreleased + 0.x SemVer stability note; version banners manual until T185. |
| L7 | **Honest product gaps:** no inventing doctor CLI or recovery-export CLI; note `contracts::doctor` DTO without CLI if relevant; document operator workarounds (RECOVERY-DRILLS). |
| L8 | **Claims language:** zero forbidden claims in **new prose and elevated docs** T183 edits (README, ARCHITECTURE, CAPABILITIES at minimum); CLAIMS-CROSSCHECK for T185. |
| L9 | **No AGPL** required doc toolchain; no certification marketing; **CONTRIBUTING.md not DoD**. |
| L10 | **Manual evidence:** install walkthrough + relative link resolution check recorded under track `evidence/`. |

## 5. Master-plan topic matrix (normative mapping)

| # | Topic | Primary home (target) | Elevate from | Normative citations |
|---|-------|----------------------|--------------|---------------------|
| 1 | Installation and local-only mode | **`Docs/INSTALL.md`** + index | OPERATIONS §1, WORKFLOWS §1, COMPATIBILITY, HANDOFF-T183 | Capture offline; models/graph optional; graph feature flag |
| 2 | Source / provenance model | CAPABILITIES + ARCHITECTURE short “user view” | CAPABILITIES, T149 fingerprints | Evidence ≠ conclusion; circularity Unknown (T156) |
| 3 | Agent permissions | OPERATIONS governed CLI + policy | OPERATIONS; T151/T160 | Principals, grants; UI no extra authority |
| 4 | Correction and review | OPERATIONS review + WORKFLOWS | `review list/resolve`; ADR-0011 | Compensating events |
| 5 | Retention / erasure limits | **SECURITY-LIMITS.md** hub | ADR-0016, RECOVERY-DRILLS, OPERATIONS erasure | Ticket ≠ CE; pre-erase residual; NIST non-claim |
| 6 | Optional cloud processing | SECURITY-LIMITS + models section | T157 / CAPABILITIES | `allow_cloud` default false |
| 7 | Sync threat model | SECURITY-LIMITS + OPERATIONS multi-device + ADR-0018 | ADR-0018, T175 threat-model | Optional; metadata residual; ACK ≠ wipe proof; name collision with Ledgerful `sync` |

## 6. Install how-to content locks (from T179 HANDOFF + fold-in)

Must appear in INSTALL (or clearly linked subsections):

1. Windows-first: MSVC, PowerShell, `cargo install` / release binary path.  
2. Secondary: Ubuntu 24.04 / WSL T1 after CI evidence; macOS T2 soft unless promoted.  
3. Daemon transport honesty: Windows named pipe; Unix UDS; portable HTTP+bearer — **not** “Unix already HTTP-default.”  
4. Device seed: Windows DPAPI not portable; multi-machine → passphrase / recovery kit.  
5. Git askpass: `git-askpass-noop.cmd` Windows; `/bin/true` Unix.  
6. Desktop engines: WebView2 Isolation Windows-only; WKWebView / WebKitGTK no Isolation claim.  
7. Vault encryption: **COMPATIBILITY F8 exact honesty** (no paraphrase that reintroduces “SQLCipher full”).  
8. Local-only default: capture without cloud/models.  
9. **Graph:** operations require `--features graph` build; capture/recall work without it.  
10. **Three “sync” surfaces:** `ai-brains sync` = Ledgerful bridge; `ai-brains safety sync` = hotspot pin; multi-device replication = **`ai-brains replicate`** (not `sync`).  
11. Soft one-liner: CI matrix may expand (T186 hermetic hygiene) — see `conductor/conductor.md`.  
12. Doctor / recovery-export: **not shipped** as CLI; see RECOVERY-DRILLS + SECURITY-LIMITS.

## 7. T180 / T181 / T182 / plan-drift handoffs

| Source | T183 action |
|--------|-------------|
| T180 F36/F35/F24 | Document unenforced `api_version`, Upcast stub, Bridge capture — no stricter claim |
| T181 kit export / doctor | **Document absence** + RECOVERY-DRILLS; note doctor DTO ≠ CLI |
| T182 ADR-0019 | SECURITY-LIMITS + index: TrustedBuiltin only |
| Implementation-Plan §8 | **Drift banner** — original design; live CLI = `--help` + conductor |
| status.md | **Demote** historical banner only |
| Audit.md / audit2.md | Soft: banner or `Docs/archive/` |

## 8. Deliverables

| Item | Path | Notes |
|------|------|-------|
| Spec / plan | this track | Expanded + §15 fold-in |
| Docs index | `Docs/README.md` | Diátaxis + topics + Research/Historical |
| Install how-to | `Docs/INSTALL.md` | §6 locks |
| Security hub | `Docs/SECURITY-LIMITS.md` | One-page + links |
| GitHub security stub | root `SECURITY.md` | Points to SECURITY-LIMITS |
| CHANGELOG | `CHANGELOG.md` | Keep a Changelog + 0.x note + Unreleased seed |
| Claims cross-check | `evidence/CLAIMS-CROSSCHECK.md` | **2-column** claim \| non-claim boundary + F8 grep results |
| Install walkthrough | `evidence/INSTALL-WALKTHROUGH.md` | Commands + outcomes |
| Link check evidence | `evidence/LINK-CHECK.md` | Relative targets resolve (manual or script) |
| F8 elevated rewords | README, ARCHITECTURE, CAPABILITIES | Required |
| Implementation-Plan §8 banner | `Docs/Implementation-Plan.md` | Drift notice |
| status.md demote | `Docs/status.md` | Historical notice only |
| Root README | links + F8 fix | |
| CAPABILITIES related-docs | light touch | |
| OPERATIONS banner | replace entire stale sentence | |

## 9. Non-goals

- Full rewrite of every historical hook research doc  
- Implementing `doctor` or `recovery export` product commands  
- Full rewrite/refresh of `status.md` content (demote only)  
- MSI / App Store / notarization packaging (T185 residual)  
- systemd/launchd production units as DoD  
- OpenAPI site / third-party client SDKs  
- Formal compliance certification project  
- AGPL or commercial SaaS doc platforms as required  
- Changing runtime protocol or encryption behavior  
- **`CONTRIBUTING.md`** as DoD (optional soft stub later if external contributors open)  
- Common Changelog migration  
- CI auto-sync of version banners (T185 scope)  

## 10. Deferred items rolled into this track

| Deferred / residual | Action in T183 |
|---------------------|----------------|
| **T179 HANDOFF-T183** + COMPATIBILITY §10 | **Absorb** into INSTALL §6 |
| **T180** protocol honesty | **Absorb** into upgrade notes |
| **T181** doctor / recovery export | **Document honesty** (not implement) |
| **T182** connector non-claims | **Cite** ADR-0019 in SECURITY-LIMITS |
| Elevated **F8 overclaims** | **Fix** README / ARCHITECTURE / CAPABILITIES |
| Implementation-Plan §8 phantoms | **Banner** |
| Missing Docs/README + CHANGELOG + SECURITY | **Create** |
| Stale status.md / OPERATIONS banner | **Demote / replace** |
| MSI/notarization/App Store | **Out of scope** → T185 |
| #34.2 DataKey rotation | **Out of scope** |
| T186 hermetic helpers | Soft pointer only |

## 11. Acceptance criteria

- [x] AC1: `Docs/README.md` index exists with Diátaxis map + seven topics + non-claims + Research/Historical  
- [x] AC2: Install how-to covers Windows T1 path + §6 locks (incl. F8, graph flag, three sync meanings)  
- [x] AC3: All seven master-plan topics have primary homes and live links  
- [x] AC4: `Docs/SECURITY-LIMITS.md` covers CE, sync, cloud, connectors, recovery residuals; root `SECURITY.md` stub exists  
- [x] AC5: `CHANGELOG.md` Keep a Changelog + Unreleased seed + 0.x SemVer note  
- [x] AC6: Root README links pack entry + key docs; F8-honest vault wording  
- [x] AC7: **Zero forbidden claims in new prose and elevated docs T183 edits** (README, ARCHITECTURE, CAPABILITIES F8 rewords required; ARCHITECTURE “Full encryption” specifically fixed)  
- [x] AC8: Claims cross-check artifact is 2-column claim/non-claim + F8 grep of elevated docs  
- [x] AC9: Manual install walkthrough + relative link-check evidence recorded  
- [x] AC10: Implementation-Plan §8 drift banner; status.md demoted; OPERATIONS banner replaced  
- [x] AC11: No new AGPL tools; no production Rust feature work required for DoD  
- [x] AC12: Conductor Completed; deferred §61 promotions recorded  

## 12. Verification plan

```powershell
# Relative link resolution (required evidence):
# For each markdown link target in README.md, Docs/README.md, Docs/INSTALL.md,
# Docs/SECURITY-LIMITS.md, SECURITY.md — resolve path and Test-Path / equivalent.
# Soft: small script under track evidence/ that extracts markdown links and tests paths.

# Install walkthrough (Windows T1):
#   cargo build --release  OR cargo install --path crates/ai-brains-cli
#   Note: graph needs --features graph; capture/recall without it
#   ai-brains --vault-path $env:TEMP\aibrains-t183\vault.db init
#   Documented preflight / recall smoke

# F8 audit grep (elevated docs):
#   Search README.md, Docs/ARCHITECTURE.md, Docs/CAPABILITIES.md, Docs/OPERATIONS.md
#   for "SQLCipher" without nearby honesty qualifiers (bundled / feature-gated / not live / COMPATIBILITY F8)

# Forbidden phrase spot-check on new + elevated prose:
#   certified, perfect deletion, metadata-private, plugin sandbox, invent doctor CLI as shipped
```

## 13. Definition of Done

All seven topics covered with links; install path tested with evidence; security hub + root SECURITY stub; changelog skeleton; F8 elevated overclaims fixed; phantom CLI surface bannered; residuals (doctor/kit export) honest; claims cross-check for T185; no unsupported compliance claims; conductor + deferred updated.

## 14. Review posture

- Category **DOCS / RELEASE** — accuracy vs live product + claims honesty (including elevated docs).  
- T185 is the formal claims gate; T183 must not leave known F8 lies in elevated surfaces.  
- Findings: `conductor/tracks/trackT183-release-documentation/review.md`.

## 15. AI fold-in disposition (2026-08-01)

### AI1

| ID | Disposition | Action |
|----|-------------|--------|
| **BS1** relative link validation | **Accept** | §12 + evidence/LINK-CHECK.md + soft script |
| **BS2** status.md historical notice | **Accept** | Prefer demote (see F-4) |
| **BS3** CLAIMS-CROSSCHECK 2-column format | **Accept** | claim \| non-claim boundary table |
| **Opp1** CHANGELOG Unreleased P12 seed | **Accept** | Structure guidance for CHANGELOG |
| **Opp2** SECURITY-LIMITS one-page hub | **Accept** | Prefer dedicated `Docs/SECURITY-LIMITS.md` |

### AI2 findings

| ID | Sev | Disposition | Action |
|----|-----|-------------|--------|
| **F-1** | High | **Accept** | AC7 expands to elevated docs; Phase C F8 audit rewords |
| **F-2** | High | **Accept** | Implementation-Plan §8 drift banner |
| **F-3** | Med | **Accept** | Root `SECURITY.md` stub → SECURITY-LIMITS |
| **F-4** | Med | **Accept** | **Prefer demote** status.md; do not full-refresh |
| **F-5** | Med | **Accept** | Replace entire OPERATIONS banner (not date-only) |
| **F-6** | Low | **Accept** | Common Changelog incompatibility note in §3.3 |
| **F-7** | Low | **Accept** | 0.x SemVer note in CHANGELOG / L6 |
| **F-8** | Low | **Accept soft** | Banner/archive Audit.md + audit2.md |
| **F-9** | Low | **Accept as non-goal** | CONTRIBUTING.md not DoD |
| **F-10** | Low | **Accept** | ARCHITECTURE “Full encryption” specific reword (subset F-1) |
| **F-11** | Low | **Accept** | Cite Diátaxis + Good Docs Project URLs in §3.2 |

### AI2 opportunities

| ID | Disposition | Action |
|----|-------------|--------|
| **O-1** | **Accept** | F8 grep in claims cross-check Phase D |
| **O-2** | **Accept soft** | Link-check script optional in evidence |
| **O-3** | **Accept** | Three sync meanings in §6 install locks |
| **O-4** | **Accept** | Version banner manual; CI → T185 |
| **O-5** | **Accept** | `--features graph` in install locks |
| **O-6** | **Accept soft** | One-line T186 pointer in install |
| **O-7** | **Accept** | Research/Historical section on Docs/README |
| **O-8** | **Accept soft** | Optional tutorial elevation of README quick start |

### Amendment map (A1–A15)

| A# | Source | Where applied |
|----|--------|---------------|
| A1 | F-1 | AC7, Phase C, §2.2 |
| A2 | F-2 | Phase C, §7, AC10 |
| A3 | F-3 | L5, deliverables |
| A4 | F-4 / AI1 BS2 | Phase C status demote |
| A5 | F-5 | Phase C OPERATIONS banner |
| A6 | F-6 | §3.3 |
| A7 | F-7 | §3.3, L6 |
| A8 | F-8 | Phase C soft archive/banner |
| A9 | F-10 | §2.2, AC7 |
| A10 | F-11 | §3.2 |
| A11 | O-1 | Phase D, AC8 |
| A12 | O-3 | §6 item 10 |
| A13 | O-5 | §6 item 9 |
| A14 | F-9 | §9 non-goals |
| A15 | O-4 | §3.3, L6 |

### Cross-track handoff (fold-in)

| Track | Note |
|-------|------|
| **T185** | Consumes CLAIMS-CROSSCHECK; re-greps elevated docs; expects root SECURITY.md; version-banner CI optional later |
| **T184** | SECURITY-LIMITS is reviewer packet index (CE/sync/cloud/connectors/recovery + ADR-0019) |
| **T186** | Soft install pointer only |
| **T179** | Copy F8 wording exactly |
| **T180** | Upgrade notes: unenforced api_version etc. |
| **T181** | Doctor DTO ≠ CLI; recovery export absent |
