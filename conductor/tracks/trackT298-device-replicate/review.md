# T298 Review Log — device/replicate useful empty

**Track:** T298-DeviceReplicate  
**Category:** FEATURE / UX HONESTY  
**FEATURE TX:** `3b29ef23-f048-4846-a73d-732b7f01b2d6`  
**Branch:** `track/T298-device-replicate`  
**Date:** 2026-08-25  
**Verdict:** **PASS WITH DEFERRED P3** (difficult residuals only)

---

## Manual evidence (AC14)

```text
$ cargo run -p ai-brains-cli --quiet -- device status
No enrolled devices. Run `ai-brains device bootstrap` first.
this machine: DESKTOP (not enrolled)
local-only; not PQ; not remote wipe
next: ai-brains replicate status

$ cargo run -p ai-brains-cli --quiet -- replicate status
Multi-device replication status
  relay:           not configured
  enrolled_count:  0
  this machine:    DESKTOP (not enrolled)
  cursors:         0
  honesty:         optional multi-device; not PQ; not remote wipe; not metadata-private
  hint:            run `ai-brains device bootstrap` to enroll first device
```

- Live vault empty-enrolled. Exit **0**. **Did not** bootstrap.
- PATH `ai-brains` **0.1.2** still lacks this-machine (F17) — source/hermetic SoT.

---

## Gates

| Check | Result |
|-------|--------|
| Unit `os_hostname` / `this_machine_label` (9) | **pass** |
| Hermetic `device_status_discoverability` (11) | **pass** |
| Stay-green `empty_states_exit_hygiene` + `device_replicate_cli` (21) | **pass** |
| `cargo clippy -p ai-brains-cli --all-targets -- -D warnings` | **pass** |
| `cargo fmt --check` | **pass** |
| `.\scripts\dev-check.ps1` | **SUCCESS** (fmt/clippy/nextest workspace **3514** + 1 skipped / deny / audit) |
| `ledgerful verify --scope full` | **Verification passed** |
| Codex `review.codex.md` | **PASS after P2 fix** |

---

## Phase 1 — Internal vs DoD

| AC | Status |
|----|--------|
| AC1–AC16 | **met** |

### Findings

| id | severity | description | status |
|----|----------|-------------|--------|
| R1-01 | low-info | PATH until `cargo install` hides T298 | `deferred` (F17) |
| R1-02 | low-info | Live vault stays 0 enrolled (honest AC14) | `deferred` (F13/F25) |
| R1-03 | low-info | `device list --format json` / combined dashboard | `deferred` (F16/F25) |
| R1-04 | low-info | Singular error-copy unify | `deferred` (T251 F12) |
| R1-05 | low-info | T299–T300 placeholders | `deferred` (F24) |
| R1-06 | low-info | clap 4.6 workspace pin / rusqlite 0.40 Dependabot | `deferred` (F14) |

No critical/high/medium. No easy unresolved lows left in-scope.

---

## Phase 2 — Codex cross-model (`review.codex.md`)

| Finding | Disposition |
|---------|-------------|
| P1 incomplete closeout/publish at review time | **Process** — closed by this closeout + Phase 6 publish |
| P2 INSTALL.md named only empty form (Agy O2 / F19 dual) | **Fixed** — tip now names empty `{hostname} (not enrolled)` **and** enrolled hyphen fingerprint |
| Product wiring F1–F27 / AC1–AC16 | **Agree** — no product regression |

Final: engineering DoD met; residuals are difficult lows only → `deferred.md`.

---

## Deferred append

See `conductor/deferred.md` § T298 implement closeout — PATH, live 0 enrolled, declined list JSON/dashboard, singular error unify, T299–T300, pin bumps.
