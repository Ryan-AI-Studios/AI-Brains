# T307 Review Log — reqwest / tower-http unify

**Status:** **Blocked** (F3 Stop-Before). No product bump.
**Category:** CHORE / DEPS (halt is DOCS provenance)
**Ledger:** DOCS TX `a4f3ba1d-d478-4768-a2b5-1eb6bebf254f`
**Date:** 2026-08-26

## Phase 0 evidence (AC9 / F22 / AC1)

| Check | Result |
|-------|--------|
| cwd | `C:\dev\AI-Brains` |
| `cargo info reqwest` | **version 0.13.4** (latest crates.io) |
| Declared `tower-http` range | docs.rs `/crate/reqwest/0.13.4/source/Cargo.toml.orig`: `tower-http = { version = "0.6.8", default-features = false, features = ["follow-redirect"] }` |
| master `Cargo.toml` | same `0.6.8` pin (fetched 2026-08-26) |
| `cargo info tower-http` | latest **0.7.0** |
| Lock | `tower-http` **0.6.11** + **0.7.0** (dual unchanged) |
| `cargo tree -i tower-http@0.6.11 --locked` | reqwest → models / desktop (expected under F3) |
| reqwest#3062 | **open**, `merged: false`, last update **2026-07-13T13:49:37Z** |
| CHORE product TX | **Not started** (F3) |

## Findings

| ID | Severity | Description | Status | Evidence |
|----|----------|-------------|--------|----------|
| R1 | low-info | Dual `tower-http` remains until crates.io reqwest allows 0.7 | deferred | F3 / deferred.md R1 |
| R2 | low-info | `#3062` still open; do not git-dep | deferred | F11; `unknown-git = "deny"` |
| R3 | low-info | tower-http 0.7.1 unpublished (#712/#722) | deferred | F12 |
| R4 | low-info | `multiple-versions = "warn"` not flipped | deferred | F21 |
| R5 | low-info | Dual hyper/http / desktop 0.1.2 out of scope | deferred | Decline |

No critical / high / medium. No regression (no product code change).

## Cross-model

**Skipped** (F16) — F3 halt, no bump.

## DoD

| AC | Result |
|----|--------|
| AC1 | **Met via F3** — Blocked + dated crates.io / Cargo.toml evidence |
| AC2 | F3 path — `0.6.11` still present (expected) |
| AC3–AC8 | N/A product (no bump) |
| AC9 | **Met** — Phase 0 re-verify |

## Closure

Conductor → **Blocked**. Residuals appended to `conductor/deferred.md`. T308 / T309 / T310 not stolen.
