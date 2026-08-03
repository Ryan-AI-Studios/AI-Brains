# T200 Review — Graph Feature Install Honesty

- **Reviewer:** Grok Build (read-only + verification)
- **Date:** 2026-08-03
- **Branch:** `feat/T200-graph-feature-install-honesty`
- **Scope:** Uncommitted implementation vs F1–F35 / AC1–AC13
- **Decision freeze checked:** A2=no (docs-only A); no `release.yml` graph-on flip; T198 exit 2 + `FEATURE_UNAVAILABLE` preserved
- **Ledger:** TX open (review did not commit/push)

## Verdict: **CLEAN** (Round 2)

Round 1 blocker **H1 is `verified_fixed`**. CI F14 on Windows and Linux uses the spec-allowed filter  
`cargo nextest run -p ai-brains-cli --features graph --profile ci -E 'test(graph)'`  
(with comments documenting full-package Cozo INFO residual). Local filtered graph suite is green on and off. **AC4 / AC9 / AC13 Met.**

**Residual (non-blocking):** M1 graph-on stdout INFO pollution remains a pre-existing product residual; not a regression of install-honesty when the allowed F14 filter path is used. Deferred for a later ops/stdout-contract track.

---

## Round history

| Round | Verdict | Notes |
|-------|---------|--------|
| 1 | NEEDS_FIX | H1: full-package F14 red (11 hermetic JSON tests); filter path green but not yet in CI |
| **2** | **CLEAN** | CI Win+Linux F14 narrowed to `-E 'test(graph)'`; re-verify 3/3 graph-on + 2/2 graph-off |

---

## AC matrix

| AC | Criterion | Status | Evidence |
|----|-----------|--------|----------|
| **AC1** | INSTALL primary = F27 SOOT | **Met** | `Docs/INSTALL.md`: `cargo install --path crates/ai-brains-cli --locked --features graph` |
| **AC2** | INSTALL slim for A2=no; feature-off exit 2 docs | **Met** | Slim = bare `--locked`; exit **2** + `FEATURE_UNAVAILABLE` documented |
| **AC3** | Feature-off exit 2 + FEATURE_UNAVAILABLE (regression) | **Met** | Stubs `main.rs`; smoke `graph__default_build__prints_hint` **PASS** (default nextest); workspace CI still graph-off |
| **AC4** | Feature-on health smoke + **runs in CI** | **Met** | `test_graph_health_smoke` **PASS** with `--features graph`; CI F14 Win+Linux hard step with allowed filter (no `continue-on-error`) |
| **AC5** | CAPABILITIES (§9 + table + needs) + CONTRIBUTING + CHANGELOG | **Met** | CAPABILITIES §9; CONTRIBUTING matrix + filter note; CHANGELOG Unreleased |
| **AC6** | Grep guard both stubs = F27 SOOT | **Met** | `graph_stub__reinstall_hint__matches_install_soot` **PASS** (default + graph-on) |
| **AC7** | Capture tree forbid graph still green | **Met** | CI capture-independence steps unchanged |
| **AC8** | A2=yes size + F13 | **N/A** | A2=no; skipped correctly |
| **AC9** | A2=no docs-only + F14 required | **Met** | Plan/spec A2=no; F14 present and green under allowed filter (Round 2) |
| **AC10** | Full gate | **Partial / process open** | Plan D2 may still be implementer-owned; this review re-ran scoped nextest only (not full fmt/clippy/deny/audit/workspace) |
| **AC11** | Claims: no always-graph; release honesty | **Met** | INSTALL/README/CAPABILITIES state Release `ai-brains.exe` graph-off |
| **AC12** | INSTALL documents GitHub Release graph-off | **Met** | INSTALL Release honesty + F28 |
| **AC13** | Both on and off have CI coverage (F13∨F14) | **Met** | Off = default workspace nextest; on = F14 Win+Linux required filter; macOS F14 omission OK (spec preferred Win+Linux) |

---

## Verification run (Round 2)

| Command | Result |
|---------|--------|
| `cargo nextest run -p ai-brains-cli --features graph --profile ci -E 'test(graph)'` | **PASS** 3/3 (F9 SOOT + live_graph unit + `test_graph_health_smoke`) |
| `cargo nextest run -p ai-brains-cli -E 'test(graph)'` | **PASS** 2/2 (F9 SOOT + feature-off hint) |

### CI yaml F14 (confirmed Round 2)

| Job | Filter present | Command |
|-----|----------------|---------|
| **gate-windows** | **Yes** | `cargo nextest run -p ai-brains-cli --features graph --profile ci -E "test(graph)"` |
| **gate-linux** | **Yes** | `cargo nextest run -p ai-brains-cli --features graph --profile ci -E 'test(graph)'` |

Comments on both jobs document that full-package `--features graph` hits pre-existing Cozo INFO stdout pollution in hermetic JSON tests. Spec F14 explicitly allows full package **or** `-E 'test(graph)'` + known smoke.

### Spot greps / freeze (unchanged from Round 1)

| Check | Result |
|-------|--------|
| Cargo `default = []` | OK (no flip) |
| `release.yml` no `--features graph` | OK |
| F27 SOOT in INSTALL + CAPABILITIES | OK |
| Stubs NOT edited | OK (match SOOT; F9 guard only) |

---

## Findings

### H1 — HIGH — **verified_fixed** — F14 full-package nextest fails under `--features graph`

**Round 1:** CI used full-package `cargo nextest run -p ai-brains-cli --features graph --profile ci` → 11 hermetic JSON tests fail because graph-on emits `CozoProxyBackend initialized` INFO on **stdout** before JSON bodies.

**Round 2 fix:** Minimal T200 path (spec-allowed): narrow F14 to  
`cargo nextest run -p ai-brains-cli --features graph --profile ci -E 'test(graph)'`  
on **both** Windows and Linux, with comments explaining residual.

**Disposition:** **`verified_fixed`**. CI uses filter; local filtered suite green (3/3 graph-on profile ci; 2/2 default). AC4/AC9/AC13 Met. Full-package graph-on nextest remains red by design until M1 product fix — **not claimed green**.

---

### M1 — MEDIUM — **deferred** — Graph-on CLI pollutes stdout with tracing INFO (product residual)

Independent of CI filter: graph-on paths that construct live graph (e.g. `recall --format json`) can emit non-JSON INFO prefix lines on **stdout**. Scripts/`jq` on the recommended INSTALL primary (graph-on) remain fragile.

**Disposition:** **Deferrable medium residual** — pre-existing product behavior, **not a regression of T200 install honesty** once F14 uses the allowed filter path. Not in T200 DoD when filter SOOT is chosen. Track for later ops/stdout-contract work (log level and/or writer → stderr; optional hermetic `RUST_LOG` denylist). Do **not** claim “full package graph-on nextest green” until fixed.

**Follow-up (suggested):** later track — Cozo/`CozoProxyBackend` init log → `debug`/`trace` and/or ensure tracing writer is stderr for machine-readable CLI stdout.

---

### L1 — LOW — open / process — Soft / hygiene (non-blocking)

| Item | Note | Status |
|------|------|--------|
| AC10 full gate | Plan D2; not re-run by Round 2 review | process open |
| conductor / deferred closeout | D4 deferred strike + Completed status at ship | process |
| `FEATURE_UNAVAILABLE` dead_code warnings | Graph-on bin warns unused const/fn; does not fail nextest | residual low |
| F9 uses `expect` in test | Allowed for tests | OK |
| `append_events` batch unit test | Soft P3 | residual low |
| macOS no F14 | **OK** per AC13 | OK |
| Stub dedupe (F30) | Soft residual; out of DoD | OK |
| CHANGELOG F14 wording | Mentions full-package-shaped command; CI actually uses filter. Accurate enough with CONTRIBUTING filter note; optional tighten | low honesty polish |

---

## F1–F35 relevant checklist (implementation)

| ID | Status | Notes |
|----|--------|-------|
| F1 docs-only A | OK | `default = []` unchanged |
| F2 cost gate | N/A | No Cargo flip |
| F3 exit 2 honesty | OK | Regression smoke green |
| F4 capture independence | OK | CI steps retained |
| F5 INSTALL + Release honesty | OK | Primary / slim / Release sections |
| F6 CONTRIBUTING matrix | OK | Includes optional `-E 'test(graph)'` note |
| F7 CAPABILITIES all graph refs | OK | |
| F8 Docs README | OK | |
| F9 SOOT grep guard | OK | |
| F10 no crates/deps | OK | |
| F11 feature-on smoke | OK | |
| F12 feature-off smoke | OK | |
| F13 | N/A | A2=no |
| F14 feature-on CI hard | **OK (filter path)** | Win+Linux; hard step; green locally |
| F15 JSON envelope | Out → T201 | |
| F17 size evidence | Skipped OK | A2=no |
| F18 claims | OK | |
| F21 SOOT determinism | OK | |
| F23 `--locked` | OK | |
| F27 SOOT pin | OK | |
| F28 Release honesty scope | OK | |
| F29 dual artifact | Out | |
| F33 slim branch A2=no | OK | |
| F34 CI missing on/off | **Mitigated** | Both covered |

---

## Completeness sweep

- **Placeholders / TODO T200:** None incomplete in INSTALL, CAPABILITIES, CONTRIBUTING, smoke F9, ci.yml F14, live_graph, CHANGELOG, dev-check note.
- **Orchestrator claims vs diff:** Docs, F9, CI filter add, CHANGELOG, `append_events`, soft dev-check, stubs untouched. F14 green under allowed filter — no overclaim of full-package graph-on green.
- **`live_graph.rs` `append_events`:** Legitimate graph-on compile unblock; not a decision-freeze violation.
- **No Cargo default flip / no release.yml flip / exit 2 preserved:** Confirmed.

---

## Easy P3s (non-blocking; Round 2)

1. ~~Document local F14 filter in CONTRIBUTING~~ — **done** (matrix + “or filter with `-E 'test(graph)'`”).
2. Drop Cozo init to `debug` (product half of M1) — still recommended residual UX fix; not T200 clearance blocker.
3. Mark conductor T200 Completed / strike deferred on D4 at ship only.
4. Optional F9 assert for `FEATURE_UNAVAILABLE` literal (already runtime-covered).
5. Unit test `GraphAwareEventStore::append_events` batch visibility.
6. Optional CHANGELOG: note CI uses `-E 'test(graph)'` so readers do not assume full-package graph-on is green.

No **new** Easy P3 blockers from Round 2 re-verify.

---

## Recommended implementer sequence (post-CLEAR)

1. ~~Fix H1~~ **done** (narrow F14 filter).
2. ~~Re-run filtered F14-equivalent + default graph suite~~ **done Round 2**.
3. Full gate (AC10) + D3/D4/D5 closeout / ledger commit when shipping.
4. Optionally residual-track M1 (stdout INFO) for a later track — not blocking T200.

---

## Summary for ship readiness

| Area | Ready? |
|------|--------|
| Decision freeze (A2 / release / exit 2) | Yes |
| Docs honesty (INSTALL / CAPABILITIES / CONTRIBUTING / README / CHANGELOG) | Yes |
| F9 SOOT guard + stubs | Yes |
| Feature-off CI | Yes (workspace default) |
| Feature-on local smoke | Yes |
| Feature-on CI (F14) | **Yes — verified_fixed (filter path)** |
| Full gate / ledger close | Implementer process (AC10); not review clearance blocker for H1 |
| M1 stdout pollution | Residual deferred (not T200 DoD regression) |

**Round 2 clearance: CLEAN.** H1 no longer blocks. M1 deferred with justification.
