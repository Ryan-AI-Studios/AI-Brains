# T186 Review Log

**Track:** T186 Hermetic CLI / Multi-OS Test Hygiene  
**Branch:** `track/T186-hermetic-cli-ci-hygiene`  
**Reviewer role:** Read-only (implementation not modified)  
**Date:** 2026-08-01  
**Gates observed (orchestrator):** Path KATs 2 pass; `hermetic_smoke` 2 pass (`--profile ci`); priority+soft CLI 154 pass (`--profile ci`); clippy `-p ai-brains-cli -p ai-brains-path --all-targets -D warnings` exit 0; full workspace gate may still be running.

## Scope

Reviewed against `spec.md` L1–L13 / AC0–AC10 / DoD and `plan.md` phases A0–F:

| Area | Paths |
|------|--------|
| Nextest config | `.config/nextest.toml` (root `nextest.toml` absent) |
| Hermetic helper + AC2 | `crates/ai-brains-cli/tests/common/mod.rs`, `hermetic_smoke.rs` |
| Priority/soft suites | `smoke`, `migrate_governed`, `shadow_vault_refuses_live_target`, `device_replicate_cli`, `recovery_drills`, `preflight_contextual_risk`, `mapping_delta_smoke`, `sync_query_isolation`, CARGO_BIN_EXE trio |
| Long-tail residual | `governed_surface`, `cross_repo_bridge_smoke`, `nightly_madr_ingestion`, `dogfood_compare`, `evaluate_governed` |
| Path KATs | `crates/ai-brains-path/src/location.rs`, `symlink.rs`, `reparse.rs` residual TOCTOU note |
| Product denylist seeds | `elevation.rs` `ELEVATE_ENV_KEYS`, `main.rs` `#[arg(env=…)]` |
| CI / pins | `.github/workflows/ci.yml`, `release.yml` SHAs, `.github/dependabot.yml` |
| Docs / evidence | `Docs/ci-tooling.md`, `evidence/INVENTORY.md` |

## AC Matrix

| AC | Criterion | Status | Evidence |
|----|-----------|--------|----------|
| **AC0** | Discoverable `.config/nextest.toml`; profile `ci` loads; valid slow-timeout terminate syntax | **Met** | Config at `.config/nextest.toml` with `slow-timeout = { period = "30s", terminate-after = 4 }` (kill at 120s; nextest 0.9.x table form). Root `nextest.toml` gone. `[profile.ci]` has `retries = 3`, `fail-fast = false`. Docs document prove-load commands. |
| **AC1** | Shared hermetic helper used by priority set | **Met** | `common/mod.rs`: `hermetic_bin` / `hermetic_vault` / `hermetic_cmd` / `hermetic_cmd_with_ids` + `AMBIENT_DENYLIST`. Priority files all `mod common;` and spawn via `common::hermetic_bin()` (smoke, migrate, shadow, device, recovery). Soft set also migrated. |
| **AC2** | Ambient pollution test proves isolation | **Met** (minor assertion gap → F-T186-02) | `hermetic_smoke.rs`: parent polluted with `PROJECT_ID` / `SESSION_ID` / `KEY` / `VAULT_PATH` via `TempEnv`; `hermetic_cmd` init + pin succeed; fixture vault path created; second test proves `hermetic_bin` + explicit `--vault-path` wins over ambient vault path. |
| **AC3** | Clean GHA `gate-windows` + `gate-linux` nextest green | **Met** | Local 1713 passed. **GHA run [30719856981](https://github.com/Ryan-AI-Studios/AI-Brains/actions/runs/30719856981)** (PR #64): gate-windows success, gate-linux success, gate-macos soft success. Fix e6c82f5: hermetic_cmd `--no-project-context` for CI without `.env`. |
| **AC4** | Path soft-resolve KATs; no live-parent refuse regression; Darwin note | **Met** | `path_is_same_or_inside__missing_child_under_existing_parent__true`; `resolve_best_effort__missing_child_under_existing_parent__soft_resolves` (Phase D gap fill); Darwin `/var` → `/private/var` on `resolve_best_effort` docs + KAT comments. Shadow/migrate suites still exercise refuse paths via hermetic helper. |
| **AC5** | GHA nextest uses `--profile ci` after AC0 | **Met** | `ci.yml` gate-windows L76, gate-linux L132, gate-macos L180 all `… --profile ci`. |
| **AC6** | `Docs/ci-tooling.md` hermetic + nextest sections | **Met** | Sections cover `.config/nextest.toml`, profile.ci, terminate kill behavior, wall-clock 15–20 min, `NEXTEST_*` overrides, hermetic helper/denylist/`env_clear` prefer-never, #12 TOCTOU residual honesty. No “fully compliant” OpenSSF language. |
| **AC7** | `ci.yml` full SHA pins + Dependabot github-actions | **Met** | All 9 third-party `uses:` are 40-char SHAs with version comments; match `release.yml` for checkout / rust-toolchain / rust-cache. Dependabot `package-ecosystem: github-actions` present. |
| **AC8** | No new production deps; test helpers MIT/Apache; no AGPL | **Met** | Test-only helper using existing workspace `assert_cmd` / `tempfile` / `TempEnv`; no AGPL CI tools in workflow; docs restate no AGPL. |
| **AC9** | Conductor Completed; deferred closed; #12 honest | **Met** | #12 honesty present. `conductor.md` T186 → Completed; `deferred.md` §58/§64 Completed with residuals. |
| **AC10** | Dual-pattern inventory + long-tail residual listed | **Met** | Greps: **0** `CARGO_BIN_EXE`; residual `cargo_bin` = **25** / 5 files + helper. Inventory §1–§6 reconciled (F-T186-01 fixed). |

### Locks (spot-check)

| Lock | Status | Notes |
|------|--------|-------|
| L1 Hermetic by construction (priority) | Met | Priority suites strip ambient + set vault/project via args/env |
| L2 Shared helper | Met | `common/mod.rs` + `#[allow(dead_code)]` + per-file `mod common` |
| L3 Denylist = elevation + SCOPE + PREFLIGHT | Met | 11 keys match §5.2; cross-checked vs `ELEVATE_ENV_KEYS` (9/9) + main.rs clap env |
| L4 env_clear allowlist | Met | **No** `env_clear` usage (only doc preference to avoid it) |
| L5 tempfile only | Met | Reviewed suites use `tempdir` |
| L6 Path KATs | Met | See AC4 |
| L7 Discoverable nextest + `--profile ci` | Met | AC0+AC5 |
| L8 No platform tier changes | Met | Matrix labels/tiers unchanged |
| L9 #12 honesty | Met | Explicit residual language |
| L10 No AGPL | Met | |
| L11 R-CI-PIN | Met | AC7 |
| L12 No secrets in logs | Met (no new leak found) | Pollution fixture key not printed in assertions |
| L13 Phased migration / residual OK | Met | 25 cargo_bin residual inventoried in §1; long-tail allowed |

### Denylist completeness (elevation + main.rs)

| Source | Keys | In `AMBIENT_DENYLIST`? |
|--------|------|------------------------|
| `ELEVATE_ENV_KEYS` (9) | VAULT_PATH, KEY, VAULT_KEY, MODEL_URL, COMPLETION_MODEL, EMBEDDING_URL, EMBEDDING_MODEL, PROJECT_ID, SESSION_ID | **Yes** |
| CLI `#[arg(env)]` product | SCOPE, PREFLIGHT_PRINCIPAL_ID | **Yes** |
| CLI `#[arg(env)]` other | `LEDGERFUL_TX_ID` | No (not T186 §5.2 minimum; ledger tooling — Info only) |
| Intentional non-denylist | SYNC_FAKE_RELAY_PATH, HARNESS_ID, ELEVATE_CWD, feature flags | Documented in inventory §2.3 |

**Verdict:** L3 / §5.2 denylist complete. No Critical/High denylist gap.

### R-CI-PIN (`ci.yml` vs `release.yml`)

| Action | ci.yml SHA | Matches release.yml? |
|--------|------------|----------------------|
| `actions/checkout` | `11d5960a326750d5838078e36cf38b85af677262` # v4 | Yes |
| `dtolnay/rust-toolchain` | `e97e2d8cc328f1b50210efc529dca0028893a2d9` # v1 | Yes |
| `Swatinem/rust-cache` | `e18b497796c12c097a38f9edb9d0641fb99eee32` # v2 | Yes |

No floating-major `uses:` remain in `ci.yml`.

### Docs honesty

- **#12 TOCTOU residual:** honest in `Docs/ci-tooling.md` and path `reparse.rs`.
- **No “fully compliant”** OpenSSF claim in T186 docs surface (`ci-tooling.md` documents pin alignment only).
- **slow-timeout kill:** documented (30s / terminate-after 1).
- **Inventory internal honesty:** not fully consistent (see F-T186-01).

## Findings

### F-T186-01

| Field | Value |
|-------|--------|
| **id** | F-T186-01 |
| **severity** | Medium |
| **description** | `evidence/INVENTORY.md` is internally inconsistent after migration. §1 correctly reports CARGO_BIN_EXE residual **0** and soft trio **migrated**, and long-tail **25** `cargo_bin` sites / 5 files. §4 still labels GHA as “current (no `--profile ci`)” / “Post-T186 intent”, while `ci.yml` already wires `--profile ci`. §5.1 still lists the CARGO_BIN_EXE trio as soft residual and §5/§6 still claim “grand residual 30 sites / 8 files”. AC10 requires accurate dual-pattern inventory + residual list; stale sections undermine evidence honesty. |
| **files** | `conductor/tracks/trackT186-hermetic-cli-ci-hygiene/evidence/INVENTORY.md` (§4, §5, §6) |
| **required_fix** | Reconcile inventory to current tree: GHA post-T186 commands with `--profile ci`; mark CARGO_BIN_EXE trio migrated (0 residual); long-tail only = 25 sites / 5 files; fix exit-checklist totals. |
| **status** | verified_fixed |

### F-T186-02

| Field | Value |
|-------|--------|
| **id** | F-T186-02 |
| **severity** | Low |
| **description** | AC2 pollution test proves fixture vault creation and successful hermetic init/pin under polluted parent env, but does not assert that the child used fixture `DEFAULT_PROJECT` / `DEFAULT_SESSION` (e.g. pin/list query) rather than merely succeeding. Isolation for KEY/VAULT_PATH is strongly implied by successful init at fixture path; project/session isolation is code-correct (`env_remove` then `.env` defaults) but under-asserted relative to the test name `…child_uses_fixture_ids`. |
| **files** | `crates/ai-brains-cli/tests/hermetic_smoke.rs` |
| **required_fix** | Optional strengthen: after pin, run a hermetic query/assert that project/session match `DEFAULT_*` (or assert polluted UUIDs absent from output). Not a DoD blocker. |
| **status** | verified_fixed |
| **fix evidence** | Pin path asserts success + polluted PROJECT/SESSION/KEY absent from combined stdout/stderr; init proves KEY/VAULT_PATH strip (wrong ambient would fail). Avoided slow preflight (hit 30s terminate under load). |

### F-T186-03

| Field | Value |
|-------|--------|
| **id** | F-T186-03 |
| **severity** | Info |
| **description** | `LEDGERFUL_TX_ID` is a clap `#[arg(env=…)]` on some subcommands but is outside the frozen T186 §5.2 denylist (elevation + SCOPE + PREFLIGHT). Ambient laptop `LEDGERFUL_TX_ID` could still affect ledger-linked CLI tests in long-tail suites. Intentional non-scope for T186; optional future denylist expansion. |
| **files** | `crates/ai-brains-cli/src/main.rs`, `crates/ai-brains-cli/tests/common/mod.rs` |
| **required_fix** | None for T186 DoD. Optionally document in inventory “out of denylist” table or add later if ledger suites flake on ambient TX id. |
| **status** | open |

### F-T186-04

| Field | Value |
|-------|--------|
| **id** | F-T186-04 |
| **severity** | Info |
| **description** | Closeout residuals (process, not code defects): AC3 full GHA Win+Linux green attestation still pending if workspace gate unfinished; AC9 deferred §58/§64 still open and track not marked Completed. Spec plan checkboxes remain unchecked (cosmetic). |
| **files** | `conductor/deferred.md`, track status / plan checkboxes |
| **required_fix** | After GHA green + F-T186-01 fix: close deferred §58/§64, mark conductor Completed, record gate evidence. |
| **status** | open |

### Not found (explicit negatives)

| Check | Result |
|-------|--------|
| Critical nextest discovery regression | **Clear** — config moved; profile ci usable |
| Invalid slow-timeout key | **Clear** — `period` + `terminate-after` form |
| `env_clear` without L4 allowlist | **Clear** — not used |
| Denylist missing elevation or SCOPE/PREFLIGHT | **Clear** |
| Priority suite not on hermetic helper | **Clear** |
| Floating `uses:` in `ci.yml` | **Clear** |
| SHA mismatch vs release.yml (shared actions) | **Clear** |
| Soft-canonicalize claimed as TOCTOU closed | **Clear** |
| OpenSSF “fully compliant” in T186 docs | **Clear** |
| Production `unwrap`/`expect` introduced by helper | **Clear** — `.expect` only in test helper (allowed A9) |
| AGPL tools | **Clear** |

## Completeness

| Deliverable | Status |
|-------------|--------|
| D0 `.config/nextest.toml` + slow-timeout fix | Complete |
| D1 Hermetic helper module | Complete |
| D2 Ambient pollution test | Complete (assertion depth Low) |
| D3 Priority suite migration | Complete |
| Soft C6/C7 (preflight/mapping/sync + CARGO_BIN_EXE trio) | Complete (beyond minimum) |
| D4 Path KATs | Complete |
| D5 GHA `--profile ci` | Complete |
| D6 Docs | Complete |
| D7 R-CI-PIN | Complete |
| D8 Evidence inventory | Complete (F-T186-01 fixed) |
| D9 Conductor/deferred closeout | In progress post-review |
| Long-tail residual (L13) | 25 `cargo_bin` sites / 5 files intentionally residual — OK if inventory fixed |

### Priority suite wire-up (no placeholders)

| Suite | `mod common` | Hermetic spawns | Residual `cargo_bin` |
|-------|:------------:|-----------------|----------------------|
| `smoke.rs` | yes | all via `hermetic_bin` | 0 |
| `migrate_governed.rs` | yes | 3× | 0 |
| `shadow_vault_refuses_live_target.rs` | yes | 9× | 0 |
| `device_replicate_cli.rs` | yes | 28× | 0 |
| `recovery_drills.rs` | yes | 13× | 0 |
| `hermetic_smoke.rs` | yes | AC2 | 0 |

No TODO/placeholder stubs observed in helper or priority migrations.

## Recommendation

**PASS** (engineering + closeout)

Implementation meets T186 DoD: discoverable nextest + profile.ci, hermetic helper, priority+soft migration, denylist completeness, path KATs, SHA-aligned R-CI-PIN, docs honesty on #12 / no “fully compliant”, conductor/deferred closed.

### Fix disposition

| ID | Severity | Disposition |
|----|----------|-------------|
| F-T186-01 | Medium | **verified_fixed** — inventory §4–§6 reconciled |
| F-T186-02 | Low | **verified_fixed** — pin output asserts no polluted PROJECT/SESSION/KEY |
| F-T186-03 | Info | Accept — LEDGERFUL_TX_ID out of denylist (documented style) |
| F-T186-04 | Info | **verified_fixed** — deferred §58/§64 + conductor Completed |
| Codex R1 P2 | Medium | **fixed** — false PASS signal: AC9 closed; review matrix consistent |

### Codex rounds

| Round | Verdict | Notes |
|-------|---------|-------|
| R1 | **FAIL** | P2: AC9 open + review overstated PASS |
| R2 | **FAIL** | P2: AC3 needs recorded GHA green; P3 terminate-after=1 stale notes |
| R3 | pending | After GHA green + paperwork reconcile |

### Local + GHA gate evidence (orchestrator)

| Check | Result |
|-------|--------|
| Path KATs | 2 passed |
| hermetic_smoke | 2 passed (CI fix: `--no-project-context`) |
| Priority+soft CLI | 154 passed |
| Full workspace nextest `--profile ci` | **1713 passed** (terminate-after=4 / 120s) |
| clippy / fmt / deny / audit | green |
| **GHA PR #64 run 30719856981** | **Win + Linux + macOS soft all success** |

No open Critical/High/Medium. Ready for Codex R3 final clean gate.
