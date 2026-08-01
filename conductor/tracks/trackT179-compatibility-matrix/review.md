# T179 Internal Review R1

**Date:** 2026-07-31  
**Scope:** Track T179 Compatibility Matrix (P12.1) — docs, CI, evidence, hygiene code  
**Method:** Read-only audit of `spec.md` F1–F32 / AC1–AC13 / DoD against `Docs/COMPATIBILITY.md`, `.github/workflows/ci.yml`, `scripts/dev-check.sh`, `evidence/*`, and key code paths (`private_blob`, `daemon_client`, git askpass, package-export tests, deferred).  
**Branch (claimed):** `track/T179-compatibility-matrix`

## Verdict: NEEDS_FIX

Core matrix docs, workflow pins, transport/vault/seed honesty, UDS/DPAPI unit proofs, desktop exclusion, and CFG inventory are largely in place. Closeout is **blocked** by incomplete T1 smoke/evidence bar and a few honesty/docs-sync gaps (deferred still “design-only”; `ci-tooling.md` omits desktop exclude; HTTP + capture-independence smoke not recorded).

No critical security findings. No overclaim of Unix HTTP-as-default. F8 vault wording is exact.

---

## DoD / AC matrix

| ID | Criterion | Status | Notes |
|----|-----------|--------|-------|
| **AC1** | `Docs/COMPATIBILITY.md` §5 table + §5.3 limitations + F8 exact | **PASS** | F8 paragraph matches F8 normative text; limitations cover service/pipe/UDS/HTTP, DPAPI, seeds, Isolation, askpass, arm64, packaging residuals |
| **AC2** | Grep-complete `CFG-INVENTORY.md` + `windows` consumers | **PASS** (minor lag) | 123 sites / 34 files / 6 `windows` consumers listed; method documented. Residual: `cfg(any(windows, test))` not in re-scan pattern (see L1) |
| **AC3** | CI: required `windows-2025` + `ubuntu-24.04` | **PASS** | `.github/workflows/ci.yml` pins both required; `macos-15` soft `continue-on-error`; no `actions-rs`; no `macos-14`/`-latest` for claims |
| **AC4** | Smoke per T1; runner label matches OS | **FAIL / incomplete** | Templates exist with correct labels; most checklist rows still **Pending GHA** / partial local only — T1 bar not met for closeout |
| **AC5** | Capture independence on each T1 tier | **PARTIAL** | Documented matrix invariant; **not** executed/recorded in smoke or CI job |
| **AC6** | Unix build fail-closed; no prod unwrap/expect hygiene debt | **PASS** (local evidence) | `UNIX-BUILD.md`: WSL check+clippy green exclude desktop; DPAPI/UDS fail-closed units present |
| **AC7** | deny + audit green; audit exit code | **PASS (workflow design)** | Linux job runs deny + `cargo audit` with F27 comment; exit-code only (no summary grep). First GHA green not recorded in smoke |
| **AC8** | T174 multi-OS residual → T2 + engine honesty | **PASS** | COMPATIBILITY desktop engines WebView2/WKWebView/WebKitGTK; Isolation Windows-only; deferred §56 absorbs T174 |
| **AC9** | Conductor + deferred updated; no overstated claims | **FAIL** | `conductor/deferred.md` §56 still says design-only / **no `.github/workflows`** |
| **AC10** | Zero new production Cargo deps | **PASS** (spot) | No new prod deps observed in T179 surface; CI Actions only |
| **AC11** | Transport matrix documented; Unix UDS verified | **PASS** | COMPATIBILITY F23 table honest (live UDS); unit `daemon_client__new__uses_os_native_transport_path` + `transport_path()` |
| **AC12** | Device seed DPAPI non-portability documented | **PASS** | COMPATIBILITY §5 + limitations; unit `device_private_blob__open_dpapi_junk__fails_with_dpapi_message` |
| **AC13** | `scripts/dev-check.sh` or justified residual | **PASS** | Exists; mirrors versions; excludes desktop on Linux/macOS; audit exit-code |

### F1–F32 quick map

| F | Status | Evidence |
|---|--------|----------|
| F1 Win T1 primary | PASS | COMPATIBILITY + Windows CI job |
| F2 Ubuntu 24.04 first non-Win | PARTIAL | CI pin OK; T1 claim ahead of recorded green smoke |
| F3 macOS best-effort / soft pin | PASS | `macos-15` soft; SMOKE-macos T2 residual |
| F4 WSL = Linux bin + `/mnt/c` | PASS | COMPATIBILITY note ¹; SMOKE-wsl optional |
| F5 no nested WSL PR gate | PASS | No WSL e2e in `ci.yml` |
| F6 fail-closed Win-only / Isolation WebView2 | PASS | Docs + cfg surfaces |
| F7 portable = HTTP; live Unix UDS | PASS | No Unix-HTTP-default overclaim |
| F8 vault wording exact | PASS | COMPATIBILITY §4 exact |
| F9 zero new prod Cargo deps | PASS | Spot-check |
| F10 toolchain 1.95.0 | PASS | Workflow + rust-toolchain; targets still Win-only (L3 residual) |
| F11 deny+audit ≥1/PR; nextest/OS | PASS design | Linux job |
| F12 capture independence invariant | PARTIAL | Doc only; not CI/smoke-gated |
| F13 desktop multi-OS not T1 | PASS | Exclude desktop on Linux/macOS CI + dev-check.sh |
| F14 arm64 T3 unless soft | PASS | COMPATIBILITY T3 |
| F15 introduce GHA on implement | PASS | `ci.yml` present |
| F16 smoke under evidence/ | PARTIAL | Files exist; results incomplete |
| F17 fail-closed Unix stubs | PASS | UNIX-BUILD + code |
| F18 no AGPL CI | PASS | Workflow comment / tools |
| F19 native runners T1 | PASS | pins |
| F20 T183/T185 handoff | PASS | HANDOFF + COMPATIBILITY §10 |
| F21 desktop engines | PASS | §6 |
| F22 edition / no unwrap hygiene | PASS (scope) | Clippy hygiene recorded |
| F23 transport matrix | PASS docs+UDS unit; PARTIAL HTTP smoke | |
| F24 required pins | PASS | windows-2025 + ubuntu-24.04 |
| F25 macOS label honesty | PASS | macos-15 not latest |
| F26 release SHA-pin | Residual | PR uses floating majors; no separate release workflow yet |
| F27 audit exit code | PASS | CI + scripts + ci-tooling |
| F28 inventory not §2.1-only | PASS | CFG-INVENTORY |
| F29 DPAPI seed non-portable | PASS | Docs + unit |
| F30 dev-check.sh | PASS | Present + desktop exclude |
| F31 check before clippy Linux | PASS | gate-linux order |
| F32 `/bin/true` askpass | PASS | COMPATIBILITY + `ai-brains-git` |

---

## Findings

| ID | Severity | Description | Files | Required fix | Status |
|----|----------|-------------|-------|--------------|--------|
| **R1-H1** | **high** | **T1 evidence bar incomplete (AC4 / DoD).** Smoke templates Pending; Ubuntu T1 ahead of evidence. | smoke + COMPATIBILITY | Fill smoke with local gate results; qualify Ubuntu T1 | **fixed_pending_verification** — SMOKE-windows (1653 nextest); SMOKE-linux WSL 1587; COMPATIBILITY evidence-bar note; GHA first green still residual on PR |
| **R1-M1** | **medium** | **AC9 deferred stale** (design-only / no workflows). | deferred.md §56 | Update to In Progress | **fixed_pending_verification** |
| **R1-M2** | **medium** | **ci-tooling.md** omits desktop exclude. | Docs/ci-tooling.md | Document exclude | **fixed_pending_verification** |
| **R1-M3** | **medium** | **SMOKE-linux lag** behind UNIX-BUILD. | SMOKE-linux.md | Record WSL PASS | **fixed_pending_verification** |
| **R1-M4** | **medium** | **F12 capture independence** not CI/smoke gated. | ci.yml + smoke | tree check CI step + smoke PASS | **fixed_pending_verification** |
| **R1-M5** | **medium** | **F23 HTTP smoke** not recorded. | smoke | hermetic http_enable_smoke recorded | **fixed_pending_verification** |
| **R1-L1** | **low** | CFG inventory misses `cfg(any(windows, test))`. | CFG-INVENTORY | re-scan optional | **deferred** |
| **R1-L2** | **low** | F26 release SHA-pin residual. | ci.yml | T185 | **deferred** |
| **R1-L3** | **low** | rust-toolchain targets Win-only. | rust-toolchain.toml | residual | **deferred** |
| **R1-L4** | **low** | Plan B0–B2 checkboxes stale. | plan.md | mark done | **open** (hygiene) |

## R1 disposition (orchestrator 2026-07-31)

Also landed Phase B2 Linux nextest hygiene (1587/1587 exclude desktop): Windows-only ACL tests cfg-gated; vault-path-free schema/graph; smoke vault isolation; path nested_start; vault_fs lexical containment; windows_path_parent for bat wrappers.

**Internal R1 remaining >low:** none after fixes — ready for internal R2 re-read / Codex.

---

## Completeness notes

### Implemented well

- **COMPATIBILITY.md** is the normative matrix: tiers, WSL¹, F8 exact, F23 pipe/UDS/HTTP, F29 seed portability, F32 `/bin/true`, desktop engines, arm T3, non-claims including “Unix DaemonClient already uses HTTP” **Forbidden**.
- **CI pins:** `windows-2025` required, `ubuntu-24.04` required with **check → clippy → nextest → deny → audit (exit code)**, `macos-15` soft; desktop excluded on Linux/macOS; no `actions-rs`.
- **`scripts/dev-check.sh`:** F30 present; version floors match; desktop exclude on Linux/Darwin; audit exit-code.
- **Code proofs:** `device_private_blob__open_dpapi_junk__fails_with_dpapi_message`; `daemon_client__new__uses_os_native_transport_path` + `DEFAULT_DAEMON_TRANSPORT_PATH` / `transport_path()`; git askpass Unix `/bin/true`.
- **package-export hermetic tests** pass `--vault-path` (device replicate CLI).
- **UNIX-BUILD.md** honest: desktop needs GTK; core compiles; clippy hygiene listed.
- **HANDOFF-T183-T185.md** covers install order, transport honesty, DPAPI seed, askpass, F8, runner-label match.
- **Cross-links:** PRD, OPERATIONS, CAPABILITIES, ci-tooling → COMPATIBILITY.
- **No Unix-HTTP-as-default overclaim** anywhere audited.

### Placeholders / stubs / overclaims

- Smoke files are **templates with Pending** more than completed smoke (H1, M3).
- Ubuntu **T1** cells without uniform “after CI green” qualifier risk overclaim relative to header honesty sentence (H1).
- deferred §56 still “no workflows” (M1) — underclaims implement progress / misleads.
- Capture independence and HTTP smoke are documented aspirations without recorded PASS (M4, M5).
- Plan Phase E (full gate, ledgerful, closeout pin) still open — appropriate; review does not clear closeout.

### CI / scripts checklist (reviewer)

| Check | Result |
|-------|--------|
| `windows-2025` + `ubuntu-24.04` required | Yes |
| `macos-15` soft | Yes |
| audit exit code only | Yes |
| no `actions-rs` | Yes |
| exclude desktop Linux/macOS CI | Yes |
| `dev-check.sh` excludes desktop on Linux/macOS | Yes |
| F31 check-before-clippy Linux | Yes |
| Zero new prod Cargo deps | No evidence of violation |

### Code spot-checks

| Item | Result |
|------|--------|
| F8 exact in COMPATIBILITY | Match |
| private_blob DPAPI junk test | Present |
| daemon_client UDS path + test | Present |
| package-export `--vault-path` | Present |
| DPAPI Unix error string | “DPAPI is only available on Windows” |

---

## Residual defer candidates (P3 only)

| Item | Severity | Owner | Justification |
|------|----------|-------|---------------|
| F26 release workflow SHA-pin actions | low | T185 | PR floating majors allowed; no release workflow yet (R1-L2) |
| Expand `rust-toolchain.toml` targets | low | When Linux host targets needed | Host gnu works on ubuntu-24.04 today (R1-L3) |
| Optional WSL `workflow_dispatch` smoke (C6) | low | T183/T185 | Spec F5: not PR-required |
| Unify Unix CLI → HTTP | low | Post-T179 | Explicitly not DoD (F23 optional) |
| arm64 soft job | info | Future | Honest T3 today (F14) |
| CFG re-scan include `cfg(any(windows, test))` | low | T179 polish | Completeness only (R1-L1) |
| systemd / launchd units | residual | Ops | Documented non-claim |

**Not deferrable without fix before T179 closeout:** R1-H1, R1-M1, R1-M2 (fix-by-default mediums). R1-M3–M5 should be fixed or explicitly residual-listed with owner in `conductor/ISSUES.md` if defer is chosen (cap ≤3 deferred mediums per track rules).

---

## Recommended closeout sequence

1. Fix **R1-M1** deferred text (quick).  
2. Fix **R1-M2** ci-tooling desktop exclude.  
3. Sync **R1-M3** SMOKE-linux from UNIX-BUILD.  
4. Run / record **R1-H1** first green gates → fill SMOKE-windows + SMOKE-linux.  
5. Record **R1-M4** capture tree + **R1-M5** HTTP smoke (or tier demotion).  
6. Optional L1/L4 plan/inventory polish.  
7. Re-review R2 → CLEAN only when H1 + M1–M2 closed and mediums deferred per policy.

---

## Sign-off

| Role | Result |
|------|--------|
| Internal R1 | **NEEDS_FIX** — not CLEAN |
| Blockers | **R1-H1** (AC4 T1 smoke); treat **R1-M1/M2** as required mediums |
| Overclaim of Unix HTTP default | **None found** |
| F8 vault wording | **Exact** |

---

## Internal Review R2

**Date:** 2026-07-31  
**Scope:** Re-verification of R1 findings after orchestrator fixes (read-only except this append).  
**Method:** Re-read `spec.md` AC/DoD, `plan.md`, R1 + dispositions, `Docs/COMPATIBILITY.md`, `Docs/ci-tooling.md`, `.github/workflows/ci.yml`, `scripts/dev-check.sh`, `conductor/deferred.md` §56, `evidence/*`, and spot-check key paths (`http_enable_smoke.rs`, `daemon_client` UDS/pipe units, `private_blob` DPAPI junk unit, capture `Cargo.toml`, CI capture-independence steps).  
**Gates (orchestrator-claimed; not re-executed here):** Windows clippy `-D warnings` + nextest **1653** + deny + audit exit 0; Linux WSL check+clippy+nextest **1587** exclude desktop.

### Verdict: **CLEAN**

All R1 findings **>low** are **verified_fixed** for their claimed fix scope. No new findings **>low**. Residual lows only (pre-existing or honesty polish). Track closeout / Phase E (first GHA PR green, full formal gate, ledger pin) remains operator work — not R2 blockers.

---

### R1 finding verification

| ID | Severity | R1 claim | R2 status | Evidence |
|----|----------|----------|-----------|----------|
| **R1-H1** | high | Smoke filled (Win 1653, Linux WSL 1587); COMPATIBILITY evidence-bar note | **verified_fixed** | `SMOKE-windows.md`: nextest **1653**, clippy, deny, audit exit 0 (local Win11). `SMOKE-linux.md` + `UNIX-BUILD.md`: WSL check/clippy/nextest **1587** exclude desktop. `COMPATIBILITY.md` evidence bar: Windows T1 = local + required `windows-2025`; Ubuntu T1 core = WSL recorded + required `ubuntu-24.04`; **do not claim GHA label green from WSL alone**. First GHA green remains residual (deferred §56) — does not re-open H1 after local T1 bar + honesty qualifier. |
| **R1-M1** | medium | deferred §56 design-only → In Progress | **verified_fixed** | `conductor/deferred.md` §56: **In Progress / Implementing**; explicitly “not design-only”; lists landed CI/docs/smoke + residuals (GHA first green, F26, etc.). |
| **R1-M2** | medium | ci-tooling desktop exclude | **verified_fixed** | `Docs/ci-tooling.md` Linux/macOS section documents `--exclude ai-brains-desktop`; GHA table rows note exclude; `dev-check.sh` note. |
| **R1-M3** | medium | SMOKE-linux lag | **verified_fixed** | `SMOKE-linux.md` checklist filled with WSL PASS rows (toolchain, check, clippy, capture tree, nextest 1587, UDS unit, hermetic HTTP, DPAPI unit); GHA rows still “Pending first PR” — honest. |
| **R1-M4** | medium | Capture independence CI + smoke | **verified_fixed** | `.github/workflows/ci.yml`: capture-independence `cargo tree -p ai-brains-capture` on **gate-windows** and **gate-linux** (forbid sync/models/graph edges). Smoke: Linux tree PASS; Windows PASS via Cargo.toml + Linux tree / CI. Capture `Cargo.toml` has no sync/models/graph deps. |
| **R1-M5** | medium | HTTP hermetic smoke recorded | **verified_fixed** | Both smoke files record `ai-brainsd` `http_enable_smoke` suite; file present (`crates/ai-brainsd/tests/http_enable_smoke.rs`: enable flags + bind parse + `http_dispatch__ping__returns_pong`). Covered by nextest on both OS gates. |
| **Phase B2** | — | Linux nextest green exclude desktop | **confirmed** | `UNIX-BUILD.md` + `SMOKE-linux.md`: **1587 passed**, 0 failed; hygiene classes documented (ACL cfg, vault-path-free schema, `windows_path_parent`, lexical vault_fs, nested_start). |

| R1 low | R2 |
|--------|-----|
| **R1-L1** CFG `cfg(any(windows, test))` | still **deferred** (inventory pattern gap; no `any(windows, test)` in CFG-INVENTORY) |
| **R1-L2** F26 release SHA-pin | still **deferred** → T185 |
| **R1-L3** rust-toolchain targets Win-only | still **deferred** |
| **R1-L4** plan B0–B2 checkboxes | still **open** (hygiene; B work done in evidence, plan boxes stale) |

---

### New findings (>low)

**None.**

---

### Residual lows only (real)

| ID | Severity | Description | Disposition |
|----|----------|-------------|-------------|
| **R2-L1** | low | **GHA first green unrecorded** — workflow pins correct; smoke GHA rows Pending; deferred §56 already lists residual. Not a docs/code defect after H1 local bar. | residual until PR CI; T185/closeout |
| **R2-L2** | low | **HTTP “health/bearer” vs hermetic dispatch** — smoke Windows row labels “HTTP health/bearer”; tests exercise flag/bind + in-process `DaemonHttpDispatch` (not axum loopback health/bearer over TCP). Linux wording (“hermetic tests”) is safer. No overclaim of Unix-HTTP-default. | optional smoke wording polish; residual OK for T179 |
| **R2-L3** | low | **`cli_capture_smoke` intermittent on `/mnt/c` under full-suite load** — noted in `UNIX-BUILD.md`; not reproduced as product regression; native `ubuntu-24.04` FS preferred for CI latency/stability. | residual / watch on first GHA |
| **R2-L4** | low | Same as R1-L1–L4 (inventory pattern, F26 SHA, toolchain targets, plan checkboxes). | deferred / open hygiene |

---

### Honesty / invariant spot-checks (R2)

| Check | Result |
|-------|--------|
| F8 vault wording exact in COMPATIBILITY §4 | **Match** (bundled SQLite + CE AES-256-GCM; SQLCipher page-level feature-gated) |
| Unix HTTP-as-default overclaim | **None** (Forbidden table + F23 UDS live) |
| CI pins `windows-2025` + `ubuntu-24.04` required; `macos-15` soft | **Yes** |
| Desktop exclude Linux/macOS CI + `dev-check.sh` | **Yes** |
| F31 check-before-clippy on Linux | **Yes** |
| F27 audit exit code only | **Yes** (CI + script; no summary grep) |
| Capture independence CI gated Win+Linux | **Yes** |
| UDS path unit + DEFAULT_DAEMON_TRANSPORT_PATH | **Present** |
| DPAPI junk open fail-closed unit | **Present** |
| deferred §56 no longer “no workflows” / design-only | **Yes** |

---

### AC delta vs R1 (summary)

| AC | R1 | R2 |
|----|----|----|
| AC4 smoke T1 | FAIL incomplete | **PASS** for local evidence + runner-label honesty; GHA first green residual (R2-L1) |
| AC5 capture independence | PARTIAL doc-only | **PASS** CI step both required jobs + smoke |
| AC9 deferred | FAIL stale | **PASS** In Progress / Implementing |
| AC7 deny+audit | design PASS | **PASS** local recorded; GHA residual with workflow present |
| Others AC1–3,6,8,10–13 | largely PASS | **Unchanged PASS** |

---

### Sign-off R2

| Role | Result |
|------|--------|
| Internal R2 | **CLEAN** |
| R1 blockers (H1, M1–M5) | **All verified_fixed** |
| New findings >low | **None** |
| Overclaim of Unix HTTP default | **None** |
| F8 vault wording | **Exact** |
| Not cleared by R2 (operator/closeout) | First GHA green on PR; Phase E full gate / ledger pin / conductor Completed — track residual, not R2 NEEDS_FIX |
