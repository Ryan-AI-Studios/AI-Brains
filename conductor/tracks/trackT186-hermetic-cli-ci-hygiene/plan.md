# T186 Plan — Hermetic CLI / Multi-OS Test Hygiene

**Status:** ✅ **Implementing / Review** (2026-08-01) — engineering complete; Codex + PR pending  
**Spec:** [spec.md](./spec.md) (§14 fold-in)  
**Category:** INFRA / TESTING / CI  
**Ledger:** `af71cd6d-a6c7-4504-b586-d6f9fbcca546` (INFRA)

## Phase overview

| Phase | Name | Outcome |
|-------|------|---------|
| **A0** | nextest config fix | `.config/nextest.toml` discoverable; slow-timeout terminate fixed |
| **A** | Inventory | Dual-pattern spawn map + elevation denylist cross-walk |
| **B** | Helper | common module + pollution test + smoke canary |
| **C** | Suite migration | Priority assert_cmd suites |
| **D** | Path honesty | Soft-resolve KATs if gaps |
| **E** | CI wire-up | `--profile ci` + R-CI-PIN SHA pins |
| **F** | Docs + closeout | ci-tooling; deferred; conductor Completed |

---

## Phase A0 — nextest config (Critical prerequisite / AC0)

- [x] **A0.1** Move `nextest.toml` → **`.config/nextest.toml`**
- [x] **A0.2** Fix slow-timeout: `slow-timeout = { period = "30s", terminate-after = 4 }` (kill at 120s; after=1 killed cross_repo e2e under load)
- [x] **A0.3** Keep `[profile.ci]` `fail-fast = false`, `retries = 3`
- [x] **A0.4** Verify: `cargo nextest show-config` / list with `--profile ci` succeeds; no unknown-key warning
- [x] **A0.5** `dev-check.ps1` / `dev-check.sh` need no path change (auto-discovery)
- [x] **A0.6** Soft dry-run: profile ci loads

**Exit:** AC0 green locally.

---

## Phase A — Inventory

- [x] **A1** Dual-pattern grep (cargo_bin + CARGO_BIN_EXE)
- [x] **A2** Classify files
- [x] **A3** Denylist cross-walk vs elevation + SCOPE + PREFLIGHT
- [x] **A4** Path containment sites
- [x] **A5** resolve_best_effort KATs vs gaps
- [x] **A6** GHA nextest lines
- [x] **A7** `evidence/INVENTORY.md`
- [x] **A8** evidence/ dir
- [x] **A9** ledgerful scan --impact

---

## Phase B — Hermetic helper (TDD)

- [x] **B1** `tests/common/mod.rs` with `#[allow(dead_code)]` + disallowed_methods allow for test expect
- [x] **B2** Consumer `mod common;`
- [x] **B3** `hermetic_bin` / `hermetic_vault` / `hermetic_cmd` / `hermetic_cmd_with_ids`
- [x] **B4** No full `env_clear`
- [x] **B5** AC2 pollution test (`hermetic_smoke.rs`)
- [x] **B6** smoke canary migrated

---

## Phase C — Priority suite migration

- [x] **C1** `smoke.rs`
- [x] **C2** `migrate_governed.rs`
- [x] **C3** `shadow_vault_refuses_live_target.rs`
- [x] **C4** `device_replicate_cli.rs`
- [x] **C5** `recovery_drills.rs`
- [x] **C6** Soft: preflight / mapping / sync_query
- [x] **C7** Soft: CARGO_BIN_EXE → hermetic_bin
- [x] **C8** Long-tail residual inventoried (25 sites / 5 files)
- [x] **C9** No secrets in failure output (L12)

---

## Phase D — Path / OS honesty

- [x] **D1** missing-child soft-resolve KAT added
- [x] **D2** macOS `/var` → `/private/var` documented
- [x] **D3** shadow/migrate containment still green
- [x] **D4** Soft: macOS GHA note in docs

---

## Phase E — CI wire-up

- [x] **E1** `ci.yml` nextest `--profile ci` (Win/Linux/macOS)
- [x] **E2** profile.ci after A0; wall-clock docs
- [x] **E3** R-CI-PIN SHA pins aligned with release.yml
- [x] **E4** Dependabot github-actions present
- [x] **E5** Soft: actionlint skipped (not DoD)
- [x] **E6** checkout pin notes in workflow header

---

## Phase F — Docs + closeout

- [x] **F1** `Docs/ci-tooling.md` hermetic + nextest sections
- [x] **F2** Stale root nextest / floating pin wording updated
- [x] **F3** Internal review; F-T186-01/02 fixed
- [x] **F4** Full local gate (nextest 1713 + fmt/deny/audit/clippy targeted)
- [x] **F5** deferred §58/§64 Completed
- [x] **F6** Conductor T186 → ✅ Completed
- [x] **F7** Optional pin decision (skip unless material)
- [ ] **F8** Codex clean final gate + PR squash-merge (after GHA green)

---

## Verification matrix

| Check | Result |
|-------|--------|
| `show-config` / list `--profile ci` | Pass |
| Pollution test AC2 | Pass (~6s) |
| Priority+soft nextest | 154 pass |
| Full workspace nextest `--profile ci` | **1713 pass** |
| Path KATs | 2 pass |
| clippy cli+path -D warnings | Pass |
| fmt / deny / audit | Pass |
| GHA Win+Linux | Pending PR |

---

## Stop-before (observed)

- #12 TOCTOU not claimed closed  
- No env_clear without L4  
- No platform tier creep  
- No AGPL tools  
- No OpenSSF “fully compliant” claim  
