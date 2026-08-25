# T302 Review Log — thiserror + chrono patches

**Track:** T302-CargoPatchThiserrorChrono  
**Category:** CHORE / DEPS  
**CHORE TX:** `aec6d64e-82e4-4593-ab1c-628f3112d329`  
**Branch:** `track/T302-cargo-patch-thiserror-chrono`  
**Date:** 2026-08-25

## Scope

Lockfile-only Dependabot patches `#60` / `#62`: `thiserror` 2.0.18→**2.0.20** and `chrono` 0.4.44→**0.4.45**. Precise pkgids (F8). Leave `thiserror@1.0.69`. Expected F9 extras only. Do not merge Dependabot remotes.

## Pin resolution (execute 2026-08-25)

| Crate | Before | After | crates.io latest | Notes |
|-------|--------|-------|------------------|-------|
| thiserror (workspace) | 2.0.18 | **2.0.20** | 2.0.20 | `cargo update -p thiserror@2.0.18 --precise 2.0.20` |
| thiserror-impl | 2.0.18 | **2.0.20** | 2.0.20 | syn **2.0.117 → 3.0.3** (F9) |
| thiserror (transitive) | 1.0.69 | **1.0.69** | — | unchanged (AC7) |
| chrono | 0.4.44 | **0.4.45** | 0.4.45 | tz-only (#1787/#1789) |
| iana-time-zone → windows-core | 0.62.2 | **0.61.2** | — | F9; both 0.61.2 + 0.62.2 remain |
| rusqlite / tokio / tower-http / clap | 0.39.0 / 1.52.3 / 0.6.11 / 4.6.1 | same | — | AC7 no bump |

Workspace carets unchanged: `thiserror = "2.0"`, `chrono = { version = "0.4", features = ["serde"] }`.

## DoD / AC matrix

| AC | Status | Evidence |
|----|--------|----------|
| AC1 thiserror 2.0.20 | **Met** | `Cargo.lock` thiserror + thiserror-impl **2.0.20**; `cargo pkgid` lists `thiserror@2.0.20` |
| AC2 chrono 0.4.45 | **Met** | `Cargo.lock` chrono **0.4.45** |
| AC3 workspace carets 2.0 / 0.4 | **Met** | `Cargo.toml:41,48` unchanged |
| AC4 full gate | **Met** | `.\scripts\dev-check.ps1` + `ledgerful verify --scope full` **exit 0** (2026-08-25) |
| AC5 no crates/ edits | **Met** | `git diff -- crates/` empty |
| AC6 CHANGELOG | **Met** | Unreleased Changed row for T302 |
| AC7 allowlist + 1.0.69 | **Met** | F9 only; thiserror 1.0.69 present; rusqlite/tokio/tower-http/clap unchanged |

## Internal findings

| ID | Severity | Description | Status |
|----|----------|-------------|--------|
| R1 | low-info | Dependabot PRs `#60`/`#62` closed as superseded after squash; remotes not deleted | **deferred** — F5 / standing hygiene |
| R2 | low-info | Dual thiserror major lines (1.0.69 via Tauri/json-patch + 2.x workspace) remain ambiguous for bare `-p thiserror` | **deferred** — out of scope; Tauri stack / T305+ non-goal |
| R3 | low-info | Dual `windows-core` 0.61.2 + 0.62.2 after F9 edge flip | **deferred** — expected F9; not easy to unify without Tauri/windows stack work |

No critical / high / medium. Easy lows: none open (AC1–AC3, AC5–AC7 closed by lock verify).

## Cross-model

**Skipped** — track category is **CHORE / DEPS**, not FEATURE / SECURITY / ARCHITECTURE (implement-track Phase 3 gate). Internal DoD matrix covers all ACs.

## Gates

- Local lock verify: done (above).
- Full `dev-check.ps1` + `ledgerful verify --scope full`: **exit 0** (2026-08-25; nextest 3529 pass / 1 skipped; deny + audit green with allowed warnings only).
- Publish GHA: Phase 6.
