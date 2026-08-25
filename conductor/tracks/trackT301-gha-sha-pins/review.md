# T301 Review Log — GHA SHA-pin refresh

**Track:** T301-GhaShaPins  
**Category:** INFRA / SECURITY  
**INFRA TX:** `3571d90d-b7c2-4204-8556-7a2b50c2d017`  
**Branch:** `track/T301-gha-sha-pins`  
**Date:** 2026-08-25

## Scope

Refresh SHA-pinned GitHub Actions in `.github/workflows/ci.yml` and `release.yml` to Dependabot target majors (`#68–#72`) without floating tags. Zero crate edits. Do not merge Dependabot remotes.

## SHA resolution (execute 2026-08-25)

| Action | Tag | object.type | Pin (commit) |
|--------|-----|-------------|--------------|
| `actions/checkout` | v7.0.1 | commit | `3d3c42e5aac5ba805825da76410c181273ba90b1` |
| `actions/upload-artifact` | v7.0.1 | commit | `043fb46d1a93c77aae656e7c1c64a875d1fc6a0a` |
| `actions/download-artifact` | v8.0.1 | commit | `3e5f45b2cfb9172054b4087a40e8e0b5a5461e7c` |
| `actions/attest` | v4.2.2 | commit | `1e69f48acb82d1966a394da916b4c1698aa569d6` |
| `softprops/action-gh-release` | v3.0.2 | **tag** `fe965f7a…` → peel | `3d0d9888cb7fd7b750713d6e236d1fcb99157228` |

Attest: latest v4.x patch is v4.2.2 (F11), not Dependabot’s v4.2.1.

## DoD / AC matrix

| AC | Status | Evidence |
|----|--------|----------|
| AC1 checkout v7 all sites | **Met** | 3× `ci.yml` + 1× `release.yml` → `@3d3c42e5… # v7.0.1` |
| AC2 artifacts/attest/gh-release | **Met** | upload `@043fb46d… # v7.0.1`; download `@3e5f45b2… # v8.0.1` (×2); attest `@1e69f48a… # v4.2.2`; gh-release peeled `@3d0d9888… # v3.0.2` |
| AC3 no floating `@vN` | **Met** | `Select-String` for five actions `@v[0-9]` → empty |
| AC4 no `pull_request_target` / `workflow_run` | **Met** | workflow scan empty |
| AC5 GHA CI green | **Met on publish** | track PR `gh run watch --exit-status` (Phase 6) |
| AC6 CHANGELOG + release header table | **Met** | Unreleased Security row; header table dated 2026-08-25 |
| AC7 no Cargo.lock / crate edits | **Met** | `git diff --name-only -- Cargo.toml Cargo.lock crates/` empty |

## Input compatibility (execute)

- upload-artifact v7 / download-artifact v8: `name` + `path` still valid.
- attest v4: `subject-path` still valid; Node 24 runtime.
- action-gh-release v3: `files` / `generate_release_notes` / `fail_on_unmatched_files` still valid; Node 24.

## Internal findings

| ID | Severity | Description | Status |
|----|----------|-------------|--------|
| R1 | low-info | Release workflow not exercised by PR CI (tag-only); YAML review + input-compat check only | **deferred** — by design F5 / §5; residual in `deferred.md` |
| R2 | low-info | Node 20 runner deprecation timeline not tracked here | **deferred** — residual already named in spec §8–12 |
| R3 | low-info | `dtolnay/rust-toolchain` + `Swatinem/rust-cache` unchanged (out of Dependabot batch) | **deferred** — F2 / non-goal |

No critical / high / medium. Easy lows: none open (AC3/AC4/AC7 closed by verify).

## Cross-model

Codex SECURITY/INFRA → `review.codex.md` (2026-08-25, gpt-5.6-luna high).

| Finding | Disposition |
|---------|-------------|
| P0 | None |
| **P1-01** pending AC5/publish + local full gate | **verified_fixed** after local gate exit 0; GHA cleared on Phase 6 watch |
| P2 / P3 | None |

Product/security pins (AC1–AC4, AC6–AC7, F10, F11) **PASS**. No validated >low product findings. Residual lows R1–R3 → `deferred.md`.

## Gates

- Local AC verify: done (above).
- Full `dev-check.ps1` + `ledgerful verify --scope full`: **exit 0** (2026-08-25).
- Publish GHA: Phase 6.
