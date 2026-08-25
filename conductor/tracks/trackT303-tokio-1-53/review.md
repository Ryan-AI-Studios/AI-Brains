# T303 Review Log — tokio 1.53.1

**Track:** T303-Tokio153  
**Category:** CHORE / DEPS  
**CHORE TX:** `46c31a21-dc44-4f2d-b037-0290bb792bb4`  
**Branch:** `track/T303-tokio-1-53`  
**Date:** 2026-08-25

## Scope

Dependabot `#59` tokio **1.52.3 → 1.53.1**. Workspace floor **`1.52` → `1.53`** (`features = ["full"]`) + `cargo update -p tokio --precise 1.53.1` (F8). Lands stable Windows signal MSRV fix `#8300`. Expected F9 `windows-sys` edge re-resolutions. Do not merge Dependabot remote. No tower-http / rusqlite / clap / thiserror bump.

## Pin resolution (execute 2026-08-25)

| Pin | Before | After | crates.io | Notes |
|-----|--------|-------|-----------|-------|
| workspace tokio | `1.52` full | **`1.53`** full | — | F1 minimum floor (caret already allowed 1.53.1) |
| lock tokio | 1.52.3 | **1.53.1** | **1.53.1** (latest 1.53.x) | F8 `--precise`; skip 1.53.0 / 1.52.4 |
| rusqlite | 0.39.0 | **0.39.0** | — | AC4 |
| tower-http | 0.6.11 | **0.6.11** | — | AC4 |
| clap | 4.6.1 | **4.6.1** | — | AC4 |
| thiserror | 1.0.69 + 2.0.20 | same | — | AC4 |
| windows-sys edges | multi 0.45/0.52/0.59/0.60.2/0.61.2 | same set; many consumers → **0.61.2** | — | F9 expected extras |

`cargo pkgid tokio` unique → `tokio@1.53.1`. `git diff -- crates/` empty (no product src).

## DoD / AC matrix

| AC | Status | Evidence |
|----|--------|----------|
| AC1 floor 1.53 + lock 1.53.1 | **Met** | `Cargo.toml:44`; `Cargo.lock` tokio **1.53.1** |
| AC2 full gate | **Met** | `.\scripts\dev-check.ps1` + `ledgerful verify --scope full` **exit 0** (3529 passed / 1 skipped) |
| AC3 daemon/CLI status | **Met** | `cargo nextest run -p ai-brainsd` **87 passed**; `cargo nextest run -p ai-brains-cli daemon_status` **9 passed** (incl. shutdown_signal abort + T199/T297 contrast) |
| AC4 no protected bumps; F9 ok | **Met** | rusqlite/tower-http/clap/thiserror unchanged; windows-sys edges only |
| AC5 CHANGELOG | **Met** | Unreleased Changed row for T303 |

## Internal findings

| ID | Severity | Description | Status |
|----|----------|-------------|--------|
| R1 | low-info | Dependabot `#59` still OPEN — close as superseded after squash; do not delete remote | **deferred** — F6 / standing hygiene |
| R2 | low-info | Multi-version `windows-sys` (0.45 / 0.52 / 0.59 / 0.60.2 / 0.61.2) remains after F9 edge flips toward 0.61.2 | **deferred** — not easy; ecosystem / Tauri unify |
| R3 | low-info | Live `#59` lock edges differ slightly from Dependabot’s exact flips (resolver preferred more `0.61.2`); still F9-class extras only | **deferred** — expected variance; do not hand-edit lock |
| R4 | low-info | `#8095` mpsc drop-waker change — no product src change; daemon-lifetime single consumer verified; residual only if a future hang appears | **deferred** — monitoring note; AC3 green |

No critical / high / medium. Easy lows closed by lock + targeted nextest.

## Cross-model

**Skipped** — track category is **CHORE / DEPS**, not FEATURE / SECURITY / ARCHITECTURE (implement-track Phase 3 gate). Internal DoD matrix covers all ACs.

## Gates

- Targeted AC3: **exit 0** (2026-08-25).
- Full `dev-check.ps1` + `ledgerful verify --scope full`: **exit 0** (2026-08-25; nextest 3529 pass / 1 skipped; deny + audit green with allowed warnings only).
- Publish GHA: Phase 6.
