# T186 — Hermetic CLI / Multi-OS Test Hygiene (P12 residual)

- **Track ID:** T186-HermeticCliCiHygiene
- **Phase:** P12 residual / post-T179 CI + test hardening
- **Status:** ✅ **Completed** (2026-08-01) — AC0–AC10; hermetic helper; profile.ci; R-CI-PIN; local nextest 1713
- **Depends on (hard):** T179 Compatibility Matrix (GHA matrix + multi-OS nextest baseline)
- **Depends on (soft):** T181 recovery_drills reuse; T185 claims honesty (tests prove support without ambient env); T184 residual **R-CI-PIN**; T185 release.yml SHA pins for consistency
- **Blocks / feeds:** Flake-resistant PR CI; honest multi-OS green without developer laptop env; Scorecard Pinned-Dependencies for **PR** `ci.yml`
- **Category:** INFRA / TESTING / CI
- **Stop-before:** Changing product platform tiers (F1–F32); AGPL test agents; full `env_clear` without OS allowlist; claiming TOCTOU/#12 closed by soft-canonicalize alone; claiming OpenSSF “fully compliant”
- **Deferred absorbed:** §56/§58 T179 hermetic helper suite + ambient env + soft-canonicalize expansion + no-fail-fast inventory + CI wall-clock notes; §62 **R-CI-PIN** (PR workflow action SHA-pin); T185 non-goal handoff of PR pins (release.yml stays T185). **Not** #34.2; **not** T180 protocol; **not** MSI packaging; **not** branch protection admin (R-CI-BRANCH); **not** doctor/export CLIs.
- **Review fold-in:** AI1 BS1–3 + Opp1–2; AI2 F-1..F-11 + O-1..O-8 → **A1–A12**. See §14. Rejected: AI1 “Fully Compliant” matrix language; actionlint as hard DoD (soft only); mandatory checkout v7 upgrade (prefer same SHAs as release.yml unless intentional major bump).

## 1. Objective

Make **CLI integration tests and PR CI** hermetic, multi-OS honest, and inventory-friendly so:

1. Clean CI runners (no developer ambient `AI_BRAINS_*`) pass the same suites as a well-configured laptop.  
2. Path safety (shadow/migrate dest-under-live-parent) stays correct when macOS resolves `/var` → `/private/var`.  
3. High-churn `assert_cmd` suites share one helper instead of N drifting `.env(...)` call sites.  
4. GHA can surface **all** OS-specific failures when debugging — requires **discoverable** nextest config (`.config/nextest.toml`) **and** `--profile ci`.  
5. Soft/secondary: PR `ci.yml` actions SHA-pinned (OpenSSF Scorecard **Pinned-Dependencies** / R-CI-PIN).

**Product platform rule unchanged:** Windows T1; Linux core T1 (desktop exclude); macOS soft T2 (T179).

| After T186 | Present |
|------------|---------|
| Shared hermetic CLI command builder | Target |
| Priority suites migrated (smoke, migrate, shadow, device, recovery, pin paths) | Target |
| Path soft-resolve KATs expanded + documented | Target |
| **`.config/nextest.toml` discoverable** + GHA `--profile ci` | Target (Critical prerequisite) |
| Fix nextest `slow-timeout` terminate syntax | Target |
| `Docs/ci-tooling.md` hermetic-test section | Target |
| PR `ci.yml` full SHA pins (R-CI-PIN) | Default in scope |
| Desktop multi-OS T1 / protocol N−1 | **No** |
| openat / cap-std TOCTOU close | **No** (#12 residual) |

## 2. Live baseline (re-scan 2026-08-01; re-verified at AI fold-in)

### 2.1 Inventory

| Asset | Status |
|-------|--------|
| `assert_cmd` | Workspace **2.2** (2.2.2; MIT OR Apache-2.0) — inherits parent env; `env` / `env_remove` / `env_clear` |
| CLI spawn sites | **~254 total across 16 files:** ~249 `cargo_bin("ai-brains")` + **5** `Command::new(env!("CARGO_BIN_EXE_ai-brains"))` in `cli_capture_smoke.rs`, `ingest_reads_json_stdin.rs`, `protocol_compat_cli.rs` |
| Heavy files | `smoke.rs` alone ~152 `cargo_bin` sites; `recovery_drills`, `device_replicate_cli`, `governed_surface` large |
| Partial hermetic | Many suites set project/session/vault; **device_replicate** documents ambient vault honesty |
| Gaps | Files without `AI_BRAINS_PROJECT_ID` string still inherit ambient; **CARGO_BIN_EXE** sites miss `cargo_bin`-only greps |
| `TempEnv` | Prefer **assert_cmd env** for CLI children; `TempEnv` only for process-level pollution tests (AC2) |
| Path soft-resolve | `resolve_best_effort` exists; some KATs already (e.g. missing child under parent) — expand if gaps |
| **nextest config (Critical)** | File is at repo-root **`nextest.toml`** — **invisible** to nextest 0.9.140 (auto-discovers only **`.config/nextest.toml`**). Verified: `cargo nextest show-config … --profile ci` → `profile 'ci' not found`. With `--config-file nextest.toml`, profile loads but warns on unknown key `terminate-after-slow-timeout` |
| nextest content (inert until moved) | Intended: `profile.default` retries=1, `slow-timeout=30s`; **`profile.ci` retries=3, `fail-fast=false`**. **Actual CI/local today:** nextest **built-in** defaults (`fail-fast=true`, retries=0) |
| GHA nextest | `cargo nextest run --workspace` **without** `--profile ci` (and profile would fail until config moved) |
| Action pins | Floating majors on `ci.yml`; **T185 `release.yml` already SHA-pins** checkout/toolchain/cache — reuse same SHAs for consistency |
| Dependabot | cargo + github-actions weekly (T184) — required for SHA-pin maintenance |
| Runner image drift | `windows-2025` image may ship newer Rust (e.g. 1.97.x); repo **pins 1.95.0** via toolchain — no action |
| Wall-clock | ~**15–20 min** Win+Linux; document budget |
| `env_clear` in tests | **None today** (L4 precautionary) |

### 2.2 Ledgerful preflight (planning day)

| Check | Result |
|-------|--------|
| `ledgerful doctor` | Ready; 0 pending / 0 drift |
| `ledgerful search "hermetic"` | Control-plane eval hermetic vaults; CLI evaluate hermetic notes |
| `ledgerful search "assert_cmd"` | cli tests + workspace dep; `TempEnv` docs prefer assert_cmd env |
| `ledgerful hotspots` | CLI command modules (product code) — T186 edits are mostly **tests** + CI YAML |

### 2.3 Gaps T186 closes

1. **nextest config not discovered** → `profile.ci` is a no-op today (Critical).  
2. No shared hermetic builder → drift and CI/laptop asymmetry.  
3. Ambient `AI_BRAINS_*` (including keys/models) can make laptop-green / CI-red (or reverse).  
4. Soft-canonicalize edge KATs may be incomplete for missing-child containment.  
5. GHA does not use `--profile ci` (after config is discoverable).  
6. R-CI-PIN: PR actions not SHA-pinned (Scorecard / T184 residual).  
7. Inventory must cover **CARGO_BIN_EXE** spawn sites, not only `cargo_bin`.

## 3. Research summary (online + ecosystem, 2026-08-01)

### 3.1 assert_cmd (2.2.x)

| Practice | Application |
|----------|-------------|
| Default env | **Inherits parent** — CI and laptop both leak ambient vars into the child |
| Hermetic set | Prefer explicit `.env(key, val)` for every required `AI_BRAINS_*` |
| Strip pollution | Prefer **targeted** `.env_remove("AI_BRAINS_…")` for known keys over full clear |
| `env_clear` | Prefer **never**. If used, restore full OS allowlist (L4). Full clear without restore breaks binary resolution and Windows process create |
| Timeouts | Optional `.timeout(Duration)` for hung CLI (soft) |
| `cargo_bin` vs `CARGO_BIN_EXE` | Prefer unify on `assert_cmd::Command::cargo_bin` so hermetic helper applies. Soft: migrate the 5 `CARGO_BIN_EXE` sites |
| Helper errors | **Test helpers may `.expect("ai-brains bin")`** (not production code). Prefer `Result` only if it simplifies callers — either is fine if documented |
| Integration-test modules | Each `tests/*.rs` is a **separate crate**. Shared `common` needs `mod common;` per file (or `#[path]`) and **`#[allow(dead_code)]`** on unused helpers to keep clippy `-D warnings` green |
| Assertions | Use `assert_cmd`/`predicates` helpers; don’t reimplement exit/stdout checks |
| Pollution proof | Dedicated test (e.g. `hermetic_smoke.rs`) sets polluted parent `AI_BRAINS_PROJECT_ID` via `TempEnv`, runs `hermetic_cmd`, asserts success with hermetic project (AC2) |

### 3.2 cargo-nextest

| Practice | Application |
|----------|-------------|
| Config path (**Critical**) | Official location: **`.config/nextest.toml`** from workspace root (https://nexte.st/docs/configuration/). Root `nextest.toml` is **not** auto-discovered. **Move** file (preferred) so CI + `dev-check.ps1` / `dev-check.sh` pick it up without `--config-file` |
| Process isolation | One process per test — **does not** clear inherited env from GHA/shell |
| Fail-fast default | Built-in `true`; first failure hides remaining OS-specific fails — **current reality until config discovered** |
| **profile.ci** | After move: `fail-fast = false`, retries=3 — **wire GHA** via `--profile ci` |
| `slow-timeout` syntax | nextest 0.9.140: **`terminate-after-slow-timeout` is unknown** (silently ignored with warning). Use table form: `slow-timeout = { timeout = "30s", terminate = true }`. Document that >30s tests are **killed** (except `__slow` overrides if configured) |
| PR speed | Local default profile may keep fail-fast; CI uses `ci` profile |
| Env overrides (soft docs) | `NEXTEST_RETRIES` / `NEXTEST_FAIL_FAST` for per-PR escapes — document only |
| Local dry-run | Before full GHA: `cargo nextest run --profile ci --workspace --no-run` (or list) to prove profile loads |
### 3.3 Path soft-canonicalize

| Practice | Application |
|----------|-------------|
| Existing API | `resolve_best_effort`: exists → canonicalize; missing → longest existing ancestor + suffix |
| macOS | `/var` → `/private/var` is the motivating case for T179 soft fix |
| Containment | Compare soft-resolved dest vs parent for “dest under live vault parent” refuse |
| Limits | Soft-resolve is **not** openat/cap-std; **#12 TOCTOU** remains residual honesty |

### 3.4 OpenSSF / GHA pin hygiene (R-CI-PIN)

| Practice | Application |
|----------|-------------|
| Scorecard Pinned-Dependencies | Pin third-party Actions to **full 40-char commit SHA** with version comment |
| Dependabot | **Required companion:** `github-actions` ecosystem already present (T184) — keep so SHA pins auto-bump |
| Scope split | **T185** `release.yml` already pins 7 actions; **T186** pins **PR `ci.yml`** only (checkout, rust-toolchain, rust-cache) |
| Consistency | Prefer **same SHAs as release.yml** for shared actions: checkout `11d5960a…` # v4, rust-toolchain `e97e2d8c…` # v1, rust-cache `e18b4977…` # v2 — unless intentionally upgrading major |
| checkout major note | Floating `@v4` received security backports (pwn-request refusal on `pull_request_target`, mid-2026). Current CI uses `pull_request:` (unaffected). SHA-pin prevents surprise tag moves. Soft: upgrade to v7 only if deliberate |
| actionlint (soft) | MIT static workflow linter — optional Linux job step; **not** DoD / not R-CI-SAST closure |

### 3.5 Multi-OS CI honesty

| Practice | Application |
|----------|-------------|
| Runner labels | Unchanged: `windows-2025`, `ubuntu-24.04`, `macos-15` soft |
| Desktop | Linux/macOS continue `--exclude ai-brains-desktop` |
| Toolchain | Explicit **1.95.0** pin overrides image-default Rust (image may be newer, e.g. 1.97.x) — correct; note only |
| Hermetic proof | Green on clean GHA without secrets or developer `AI_BRAINS_*` is the bar |

## 4. Normative locks (L1–L13)

| ID | Lock |
|----|------|
| **L1** | **Hermetic by construction.** CLI integration tests that require project/session/vault MUST set those via CLI flags and/or explicit assert_cmd env — never rely on ambient developer env for pass. |
| **L2** | **Shared helper.** New/migrated suites use a documented helper that applies the hermetic env baseline. Ad-hoc `.env` only for *additional* keys with comment. Shared module must be clippy-clean under multi-binary integration tests (`#[allow(dead_code)]` as needed). |
| **L3** | **Ambient strip = product elevation list + CLI env args.** Prefer **`env_remove`** over `env_clear`. Denylist **must** cover `elevation.rs` `ELEVATE_ENV_KEYS` plus `AI_BRAINS_SCOPE` and `AI_BRAINS_PREFLIGHT_PRINCIPAL_ID` (see §5.2). Never leave ambient vault keys set. |
| **L4** | **env_clear allowlist (full).** If any test uses `env_clear`, restore before spawn: **Windows** `PATH`, `SystemRoot`, `SystemDrive`, `COMSPEC`, `PATHEXT`, `TEMP`, `TMP`; **Unix** `PATH`, `HOME`, `USER`, `LANG`, `TMPDIR` (and documented OS needs). Prefer never using `env_clear`. |
| **L5** | **tempfile only.** Vault/fs fixtures stay under `tempfile::tempdir()` / `NamedTempFile`; no writes outside temp (AGENTS). |
| **L6** | **Path KATs.** Soft-resolve: missing child under existing parent; documented Darwin `/var` behavior; no regression of live-parent refuse. |
| **L7** | **Discoverable nextest config + CI profile.** Config lives at **`.config/nextest.toml`**. Required GHA (and documented local CI-mirror) nextest steps use `--profile ci` after config is discoverable. Fix `slow-timeout` terminate table syntax in the same change. |
| **L8** | **No platform tier changes.** COMPATIBILITY F1–F32 unchanged. |
| **L9** | **#12 honesty.** Soft-canonicalize does not claim TOCTOU closed. |
| **L10** | **No AGPL** test or CI tools. |
| **L11** | **R-CI-PIN.** PR `ci.yml` third-party `uses:` pinned to full SHA + version comment; Dependabot remains; prefer same SHAs as T185 release.yml for shared actions. |
| **L12** | **No secrets in logs.** Tests must not print vault keys, recovery kit material, or bearer tokens. |
| **L13** | **Phased migration.** Priority suites in DoD; long-tail residual OK if inventoried. Do not require all ~254 sites in one PR. |

## 5. Deliverables

| # | Deliverable | Path (proposed) | Notes |
|---|-------------|-----------------|-------|
| D0 | nextest config fix | **Move** `nextest.toml` → `.config/nextest.toml`; fix `slow-timeout` table | Prerequisite for AC5; first implement action |
| D1 | Hermetic CLI helper | `crates/ai-brains-cli/tests/common/mod.rs` | Builder + denylist; `#[allow(dead_code)]`; `mod common` per consumer file |
| D2 | Ambient pollution test | e.g. `tests/hermetic_smoke.rs` | AC2 automated proof |
| D3 | Suite migration (priority) | smoke, migrate_governed, shadow_*, device_replicate_cli, recovery_drills | Soft: CARGO_BIN_EXE trio → cargo_bin + helper |
| D4 | Path KATs | path crate / CLI shadow | Soft-resolve edges if gaps remain |
| D5 | GHA nextest | `.github/workflows/ci.yml` | `--profile ci` Win/Linux/(soft) macOS |
| D6 | Docs | `Docs/ci-tooling.md` | Hermetic rules; `.config/nextest.toml`; profile.ci; slow-timeout kill behavior; wall-clock |
| D7 | Action SHA pins | `.github/workflows/ci.yml` | R-CI-PIN; align SHAs with release.yml |
| D8 | Evidence | `evidence/INVENTORY.md` + GHA note | Spawn inventory includes CARGO_BIN_EXE |
| D9 | Conductor/deferred | Completed; §58/§64 | |

### 5.1 Helper sketch (normative shape, not final API)

```rust
// tests/common/mod.rs — each integration binary: mod common;
// #[allow(dead_code)] // not every helper used by every binary
pub fn hermetic_cmd(vault: &Path) -> assert_cmd::Command {
    // Test helper: expect is acceptable here (not production)
    let mut cmd = Command::cargo_bin("ai-brains").expect("ai-brains bin");
    for key in AMBIENT_DENYLIST {
        cmd.env_remove(key);
    }
    cmd.arg("--vault-path").arg(vault);
    cmd.env("AI_BRAINS_PROJECT_ID", DEFAULT_PROJECT);
    cmd.env("AI_BRAINS_SESSION_ID", DEFAULT_SESSION);
    cmd
}
```

Constants: fixed UUIDs (existing smoke style) preferred for determinism.

### 5.2 Ambient denylist (minimum — product-aligned)

**Canonical seed:** `crates/ai-brains-cli/src/elevation.rs` `ELEVATE_ENV_KEYS` (9 keys), plus CLI-arg env not in elevation:

| Key | Source |
|-----|--------|
| `AI_BRAINS_VAULT_PATH` | elevation |
| `AI_BRAINS_KEY` | elevation — **must strip** (wrong vault key risk) |
| `AI_BRAINS_VAULT_KEY` | elevation |
| `AI_BRAINS_MODEL_URL` | elevation |
| `AI_BRAINS_COMPLETION_MODEL` | elevation |
| `AI_BRAINS_EMBEDDING_URL` | elevation |
| `AI_BRAINS_EMBEDDING_MODEL` | elevation |
| `AI_BRAINS_PROJECT_ID` | elevation |
| `AI_BRAINS_SESSION_ID` | elevation |
| `AI_BRAINS_SCOPE` | preflight CLI env |
| `AI_BRAINS_PREFLIGHT_PRINCIPAL_ID` | main.rs `#[arg(env=…)]` |

Phase A3: re-grep `#[arg(env` / `AI_BRAINS_` in CLI and **diff** against this table; expand denylist if new product-affecting keys appear.

## 6. Acceptance criteria

| # | Criterion |
|---|-----------|
| **AC0** | nextest config is at **`.config/nextest.toml`** (or every invocation uses `--config-file` — prefer move); `cargo nextest show-config … --profile ci` succeeds; `slow-timeout` terminate syntax valid (no unknown-key warning). |
| **AC1** | Shared hermetic helper exists and is used by ≥ priority suite set (smoke + migrate + shadow + device + recovery). |
| **AC2** | Automated ambient pollution test (polluted parent project id) proves helper isolation (L1). |
| **AC3** | Clean GHA `gate-windows` + `gate-linux` nextest green after changes. |
| **AC4** | Path soft-resolve KATs / no live-parent refuse regression; Darwin note if applicable. |
| **AC5** | GHA nextest uses `--profile ci` (L7) **after** AC0. |
| **AC6** | `Docs/ci-tooling.md`: hermetic rules, `.config/nextest.toml`, profile.ci, slow-timeout kill behavior, wall-clock, optional NEXTEST_* overrides. |
| **AC7** | All third-party `uses:` in `ci.yml` full SHA pins + comments; Dependabot github-actions present. |
| **AC8** | No new production deps; test helpers MIT/Apache only; no AGPL. |
| **AC9** | Conductor Completed; deferred closed; #12 still honest. |
| **AC10** | Inventory evidence greps **both** `cargo_bin("ai-brains")` and `CARGO_BIN_EXE_ai-brains`; long-tail residual listed if not fully migrated. |

## 7. Non-goals

| Out of scope | Owner |
|--------------|--------|
| Platform tier changes F1–F32 | T179 / COMPATIBILITY |
| Desktop WebKitGTK T1 | Packaging residual |
| Protocol wire N−1 | **T180** |
| Backup drill product expansion | **T181** (reuse helper only) |
| Release workflow / SBOM / claims | **T185** |
| Branch protection enable | R-CI-BRANCH repo admin |
| openat / cap-std TOCTOU | Future path-hardening / #12 |
| Migrate **all** 249 call sites in one PR | Prefer priority suites + helper; residual long-tail OK with inventory |
| Rewriting control-plane hermetic eval vaults | Already T169-shaped |

## 8. License / commercial

- No production crate license change.  
- `assert_cmd` / `tempfile` / `predicates` already workspace MIT/Apache-class.  
- No AGPL CI or test tools.  
- Do not log vault keys or recovery secrets.

## 9. Risk register

| Risk | Mitigation |
|------|------------|
| Helper churn across ~254 sites | Priority first; long-tail residual (L13) |
| env_clear breaks Windows | Prefer env_remove; full L4 allowlist if clear used |
| Profile.ci increases CI wall time | Accept retries=3; document budget |
| SHA-pin churn | Dependabot github-actions weekly |
| Soft-resolve KATs only on Linux CI | Unit tests path crate; macOS soft optional |
| Overclaim hermetic = secure | L9 #12 honesty |
| Implementer assumes root nextest.toml works | AC0 first; verified broken today |

## 10. Definition of Done

- **AC0–AC10** met (including L11 SHA pins by default).  
- Full local gate green; GHA Win+Linux green.  
- No Critical/High introduced; mediums per AGENTS.  
- Ledger clean after implement transaction.

## 11. Suggested sequencing

1. **Move** `nextest.toml` → `.config/nextest.toml` + fix `slow-timeout` terminate syntax; prove `--profile ci`.  
2. Inventory (both spawn patterns + elevation denylist cross-walk).  
3. Helper + pollution test + smoke canary.  
4. Migrate priority suites; soft CARGO_BIN_EXE unify.  
5. Path KATs if gaps.  
6. GHA `--profile ci` + SHA pins (align release.yml) + docs.  
7. Review + deferred closeout.

## 12. Expand checklist (design complete when)

- [x] Online research (assert_cmd 2.2, nextest profiles, Scorecard pins, soft-resolve)  
- [x] Deferred absorption (§56/§58, R-CI-PIN from T184/T185)  
- [x] Ledgerful doctor/status/search/hotspots  
- [x] Live inventory (~254 spawns; nextest path Critical verified)  
- [x] Locks L1–L13 + AC0–AC10  
- [x] AI1/AI2 fold-in (A1–A12)  
- [ ] Implement only on user go-ahead  

## 13. References

- assert_cmd 2.2.2: https://docs.rs/assert_cmd  
- cargo-nextest config: https://nexte.st/docs/configuration/ (`.config/nextest.toml`)  
- OpenSSF Scorecard Pinned-Dependencies  
- Project: `.config/nextest.toml` (target), `elevation.rs` `ELEVATE_ENV_KEYS`, `ci.yml`, `release.yml` SHAs, `resolve_best_effort`  
- T179 / T184 R-CI-PIN / T185 release pins  

## 14. AI review fold-in (2026-08-01)

### 14.1 Accepted → amendments

| ID | Source | Fold-in |
|----|--------|---------|
| **A1** | AI2 F-1 Critical (verified) | Move nextest config to `.config/nextest.toml`; AC0; first implement step |
| **A2** | AI2 F-2 | `slow-timeout = { timeout = "30s", terminate = true }`; document kill behavior |
| **A3** | AI2 F-3 | Inventory ~254 sites / CARGO_BIN_EXE; plan A1 dual grep |
| **A4** | AI2 F-4 / O-4 | Denylist = elevation + SCOPE + PREFLIGHT_PRINCIPAL_ID |
| **A5** | AI2 F-5 | Note checkout floating-major backport; SHA-pin strategy; soft v7 |
| **A6** | AI2 F-6 | windows-2025 image Rust drift note (pin 1.95.0 OK) |
| **A7** | AI2 F-7 / O-3 | actionlint optional soft only |
| **A8** | AI2 F-8; AI1 BS2 | L4 full OS allowlist |
| **A9** | AI2 F-9; AI1 | `.expect` OK in test helpers; dead_code allow for common |
| **A10** | AI2 F-10 / F-11 | Align ci.yml SHAs with release.yml; move path auto-fixes dev-check |
| **A11** | AI1 BS1 / Opp2 | Integration-test common module pattern + automated pollution test |
| **A12** | AI1 BS3 / L13 | Phased migration; long-tail residual OK |

### 14.2 Rejected / partial

| Item | Disposition |
|------|-------------|
| AI1 “Fully Compliant” standards matrix | **Reject** — align only; no compliance claim language |
| actionlint as DoD | **Soft** opportunity only |
| Mandatory checkout v7 | **Partial** — prefer same SHA as release.yml; v7 optional deliberate upgrade |
| Dual std::process helper for CARGO_BIN_EXE | Prefer migrate to cargo_bin (O-2); dual helper only if migration blocked |

### 14.3 Status after fold-in

Still **Proposed / Expanded**. **Implement first action = AC0 nextest path fix** (not helper). No production product code required for AC0.  
