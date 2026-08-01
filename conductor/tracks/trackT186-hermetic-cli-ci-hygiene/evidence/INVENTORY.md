# T186 Phase A — Evidence Inventory

**Date:** 2026-08-01  
**Branch:** `track/T186-hermetic-cli-ci-hygiene`  
**Scope:** `crates/ai-brains-cli/tests/` dual-pattern spawn map, ambient denylist cross-walk, path containment, GHA nextest, long-tail residual (L13)

**Grep patterns:**

```text
cargo_bin("ai-brains")
CARGO_BIN_EXE_ai-brains
common::hermetic_bin|hermetic_cmd|hermetic_vault
```

---

## 1. Dual-pattern spawn inventory

### 1.1 Summary totals (current tree)

| Pattern | Sites | Notes |
|---------|------:|-------|
| `Command::cargo_bin("ai-brains")` residual (suites) | **25** | Long-tail only (L13 OK) |
| `Command::cargo_bin("ai-brains")` in helper | **1** | `common/mod.rs` `hermetic_bin` only |
| `env!("CARGO_BIN_EXE_ai-brains")` residual | **0** | Soft trio migrated to `hermetic_bin` |
| `common::hermetic_*` call sites | **~232** | Priority + soft + CARGO_BIN_EXE trio |
| **Long-tail residual** | **25** | 5 files; not DoD blockers |
| **All suite spawn call sites (hermetic + residual)** | **~257** | ≈ plan baseline ~254 |

Historical plan baseline (~254 dual-pattern sites / 16 files): priority + soft assert_cmd + CARGO_BIN_EXE sites rewritten to `common::hermetic_*` (single `cargo_bin` inside helper).

### 1.2 Per-file table

| File | `cargo_bin` | `CARGO_BIN_EXE` | `hermetic_*` | Classification |
|------|------------:|----------------:|-------------:|----------------|
| `smoke.rs` | 0 | 0 | 152 | **priority-migrated** |
| `migrate_governed.rs` | 0 | 0 | 3 | **priority-migrated** |
| `shadow_vault_refuses_live_target.rs` | 0 | 0 | 9 | **priority-migrated** |
| `device_replicate_cli.rs` | 0 | 0 | 28 | **priority-migrated** |
| `recovery_drills.rs` | 0 | 0 | 13 | **priority-migrated** |
| `hermetic_smoke.rs` | 0 | 0 | 6 | **priority-migrated** (AC2 pollution proof) |
| `preflight_contextual_risk.rs` | 0 | 0 | 9 | **soft migrated** |
| `mapping_delta_smoke.rs` | 0 | 0 | 5 | **soft migrated** |
| `sync_query_isolation.rs` | 0 | 0 | 5 | **soft migrated** |
| `cli_capture_smoke.rs` | 0 | 0 | 1 | **soft migrated** (was CARGO_BIN_EXE) |
| `ingest_reads_json_stdin.rs` | 0 | 0 | 1 | **soft migrated** (was CARGO_BIN_EXE) |
| `protocol_compat_cli.rs` | 0 | 0 | 3 | **soft migrated** (was CARGO_BIN_EXE) |
| `governed_surface.rs` | 12 | 0 | 0 | **long-tail residual** |
| `cross_repo_bridge_smoke.rs` | 8 | 0 | 0 | **long-tail residual** |
| `nightly_madr_ingestion.rs` | 3 | 0 | 0 | **long-tail residual** |
| `dogfood_compare.rs` | 1 | 0 | 0 | **long-tail residual** |
| `evaluate_governed.rs` | 1 | 0 | 0 | **long-tail residual** |
| `common/mod.rs` | 1 | 0 | n/a | Helper definition (not a suite) |

**Priority set (DoD AC1):** smoke, migrate_governed, shadow_vault_refuses_live_target, device_replicate_cli, recovery_drills (+ hermetic_smoke for AC2).  
**Soft set (done):** preflight, mapping_delta, sync_query, CARGO_BIN_EXE trio.

### 1.3 Classification roll-up

| Class | Files | Spawn sites (file-local) |
|-------|------:|-------------------------:|
| priority-migrated | 6 | ~211 hermetic |
| soft migrated | 6 | ~24 hermetic |
| long-tail residual | 5 | 25 cargo_bin |
| helper | 1 | 1 cargo_bin |

---

## 2. Denylist cross-walk

### 2.1 `elevation.rs` `ELEVATE_ENV_KEYS` (9 keys)

Source: `crates/ai-brains-cli/src/elevation.rs`

| # | Key | In `AMBIENT_DENYLIST`? |
|---|-----|:----------------------:|
| 1 | `AI_BRAINS_VAULT_PATH` | yes |
| 2 | `AI_BRAINS_KEY` | yes |
| 3 | `AI_BRAINS_VAULT_KEY` | yes |
| 4 | `AI_BRAINS_MODEL_URL` | yes |
| 5 | `AI_BRAINS_COMPLETION_MODEL` | yes |
| 6 | `AI_BRAINS_EMBEDDING_URL` | yes |
| 7 | `AI_BRAINS_EMBEDDING_MODEL` | yes |
| 8 | `AI_BRAINS_PROJECT_ID` | yes |
| 9 | `AI_BRAINS_SESSION_ID` | yes |

**Coverage:** 9/9 elevation keys stripped by `common::AMBIENT_DENYLIST`.

### 2.2 `main.rs` product `#[arg(env = …)]` keys (normative shortlist)

| Key | Role | In denylist? |
|-----|------|:------------:|
| `AI_BRAINS_VAULT_PATH` | global vault path | yes |
| `AI_BRAINS_KEY` | vault SQLCipher key | yes |
| `AI_BRAINS_PROJECT_ID` | project scope | yes |
| `AI_BRAINS_SCOPE` | preflight/scope CLI | yes (CLI-only add) |
| `AI_BRAINS_PREFLIGHT_PRINCIPAL_ID` | principal override | yes (CLI-only add) |

`AI_BRAINS_PROJECT_ID` / `PREFLIGHT_PRINCIPAL_ID` appear on multiple subcommands; denylist strips once at spawn (sufficient).

`AI_BRAINS_SESSION_ID` is **not** a clap `env=` on global opts but is product-affecting (pin/sync/context); covered via elevation seed.

### 2.3 `common/mod.rs` `AMBIENT_DENYLIST` (11 keys)

```text
AI_BRAINS_VAULT_PATH
AI_BRAINS_KEY
AI_BRAINS_VAULT_KEY
AI_BRAINS_MODEL_URL
AI_BRAINS_COMPLETION_MODEL
AI_BRAINS_EMBEDDING_URL
AI_BRAINS_EMBEDDING_MODEL
AI_BRAINS_PROJECT_ID
AI_BRAINS_SESSION_ID
AI_BRAINS_SCOPE
AI_BRAINS_PREFLIGHT_PRINCIPAL_ID
```

**Verdict:** Denylist covers **all product-affecting keys** from elevation + the CLI env args called out in T186 L3/§5.2.

**Out of denylist (intentional / non-core ambient risk):**

| Key | Why not denylisted |
|-----|--------------------|
| `AI_BRAINS_SYNC_FAKE_RELAY_PATH` | Test/fake-relay path; suites set explicitly |
| `AI_BRAINS_HARNESS_ID` | Context file write surface, not elevation/clap product gate |
| `AI_BRAINS_ELEVATE_CWD` | UAC handoff only |
| `AI_BRAINS_RETENTION_APPLY_CE` / `AI_BRAINS_GOVERNED_BRIEFING` | Feature flags / docs; not ambient vault identity |

**Frozen denylist for Phase B+:** the 11-key list above unless a new product `#[arg(env=AI_BRAINS_…)]` appears.

---

## 3. Path containment sites

### 3.1 Production

| Site | Path | Mechanism |
|------|------|-----------|
| Shadow refuse dest-under-live-parent | `crates/ai-brains-cli/src/commands/shadow.rs` `refuse_unsafe_destination` | `path_is_same_or_inside(destination, live_parent)` after `paths_refer_to_same_location` checks; uses `resolve_best_effort` via location normalize |
| Migrate dest safety | `crates/ai-brains-cli/src/commands/migrate.rs` `run_governed` | Reuses `shadow::refuse_unsafe_destination` (M3/M6) — same dest-under-parent rule with migrate message rewrite |

Related: `refuse_unsafe_report_path` / `refuse_unsafe_manifest_path` (migrate) guard report/manifest siblings; not the live-parent containment rule.

### 3.2 Integration KATs (CLI)

| Test | File |
|------|------|
| `shadow_create__destination_inside_live_vault_parent__refuses` | `shadow_vault_refuses_live_target.rs` |
| `migrate_governed__refuse_dest_inside_live_parent` | `migrate_governed.rs` |

### 3.3 Path-crate KATs (`ai-brains-path`)

| Test | Status |
|------|--------|
| `path_is_same_or_inside__missing_child_under_existing_parent__true` | **Present** (`location.rs`) |
| `resolve_best_effort__non_existing__returns_input` | Present (fully missing tree → echo input) |
| `resolve_best_effort__missing_child_under_existing_parent__soft_resolves` | **Phase D add** (gap fill) |

macOS `/var` → `/private/var` documented on `resolve_best_effort` in `symlink.rs` (doc comment).

---

## 4. GHA nextest command lines

Source: `.github/workflows/ci.yml` (**T186 Phase E landed**).

### 4.1 Pre-T186 (historical)

| Job | Command |
|-----|---------|
| `gate-windows` | `cargo nextest run --workspace` |
| `gate-linux` | `cargo nextest run --workspace --exclude ai-brains-desktop` |
| `gate-macos` (soft) | `cargo nextest run --workspace --exclude ai-brains-desktop` |

### 4.2 Post-T186 (current workflow)

| Job | Command |
|-----|---------|
| `gate-windows` | `cargo nextest run --workspace --profile ci` |
| `gate-linux` | `cargo nextest run --workspace --exclude ai-brains-desktop --profile ci` |
| `gate-macos` (soft) | `cargo nextest run --workspace --exclude ai-brains-desktop --profile ci` |

Profile.ci: `fail-fast = false`, `retries = 3` via `.config/nextest.toml`. Wall-clock ~15–20 min documented in `Docs/ci-tooling.md`.  
R-CI-PIN: all third-party `uses:` full SHA pins aligned with `release.yml`.

---

## 5. Long-tail residual list (L13)

**L13:** Phased migration — priority suites in DoD; long-tail residual OK if inventoried. Do not require all ~254 sites in one PR.

### 5.1 Soft CARGO_BIN_EXE trio — **migrated** (Phase C7)

| File | Sites | Status |
|------|------:|--------|
| `cli_capture_smoke.rs` | 1 | **Migrated** → `common::hermetic_bin()` + `write_stdin` |
| `ingest_reads_json_stdin.rs` | 1 | **Migrated** → `common::hermetic_bin()` |
| `protocol_compat_cli.rs` | 3 | **Migrated** → `common::hermetic_bin()` |

`env!("CARGO_BIN_EXE_ai-brains")` residual: **0**.

### 5.2 Long-tail residual (`cargo_bin` ambient-risk — not DoD)

| File | Sites | Notes |
|------|------:|-------|
| `governed_surface.rs` | 12 | Largest long-tail surface |
| `cross_repo_bridge_smoke.rs` | 8 | Bridge push/pull/context |
| `nightly_madr_ingestion.rs` | 3 | Nightly-oriented |
| `dogfood_compare.rs` | 1 | Helper factory |
| `evaluate_governed.rs` | 1 | Helper factory |

**Total long-tail cargo_bin residual:** 25 sites / 5 files.  
**Soft residual:** 0.  
**Priority + soft-migrated (hermetic):** smoke, migrate_governed, shadow_*, device_replicate_cli, recovery_drills, hermetic_smoke, preflight_contextual_risk, mapping_delta_smoke, sync_query_isolation, cli_capture_smoke, ingest_reads_json_stdin, protocol_compat_cli.

---

## 6. Inventory exit checklist

| Item | Result |
|------|--------|
| A1 dual-pattern greps | Done — long-tail residual 25 + helper 1; hermetic ~232 |
| A2 classify files | Done — §1.2 |
| A3 denylist cross-walk | Done — 11/11 product keys; frozen |
| A4 path containment sites | Done — shadow + migrate reuse |
| A5 resolve_best_effort KAT gaps | Soft-resolve missing-child KAT added in Phase D |
| A6 GHA nextest lines | Done — `--profile ci` landed on all three jobs |
| A7 this file | `evidence/INVENTORY.md` |
| L13 long-tail | §5.2 inventoried (25 sites / 5 files) |

**Denylist frozen.** Long-tail left residual per L13 (acceptable DoD residual).
