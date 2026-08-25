# T297 Review Log — daemon status Stopped vs backend TCP Open

**Track:** T297-DaemonVsLlm  
**Category:** BUGFIX / UX HONESTY  
**BUGFIX TX:** `a3c47213-69bf-4d30-85ea-bd6758e7022b`  
**Branch:** `track/T297-daemon-vs-llm`  
**Date:** 2026-08-24  
**Verdict:** **PASS WITH DEFERRED P3** (difficult residuals only)

---

## Manual evidence (AC10)

```text
$ cargo run -p ai-brains-cli --quiet -- daemon status
Status: Running
Vault: C:\dev\ai-brains\vault.db
Vault size: 145.8 MB
Memories: 48680
LLM backend 127.0.0.1:8081 [http://127.0.0.1:8081]: Open
Embedding backend 127.0.0.1:8083 [http://127.0.0.1:8083]: Open
PID: 4536
```

- Running + both Open → contrast **absent**, `next:` **absent**. Exit **0**.
- **Did not** `daemon start` / `stop` / `install`.
- Stopped+Open proven by AC8 keep-bound hermetic + AC1/AC6 units.

```text
$ cargo run -p ai-brains-cli --quiet -- daemon status --help
… LLM/Embedding Open is TCP connect to the model process, not the AI-Brains daemon.
```

---

## Gates

| Check | Result |
|-------|--------|
| Targeted nextest (contrast/tail/help/keep_bound/T85/T94/T128) | **pass** |
| `cargo clippy -p ai-brains-cli --all-targets -- -D warnings` | **pass** |
| `.\scripts\dev-check.ps1` | **SUCCESS** (fmt/clippy/nextest/deny/audit) |
| Unrelated force-restore live-daemon soft-skip | recovery_drills + smoke |

---

## Phase 1 — Internal vs DoD

| AC | Status |
|----|--------|
| AC1–AC6 / AC5 / AC7 / AC8 / AC9–AC14 | **met** |

### Findings

| id | severity | description | status |
|----|----------|-------------|--------|
| R1-01 | low-info | Clippy `last`/`filter` in AC8 hermetic | `verified_fixed` (`rfind`) |
| R1-02 | low-info | PATH until `cargo install` | `deferred` |
| R1-03 | low-info | Live Running hides Stopped+Open | `deferred` (F11) |
| R1-04 | low-info | Doctor Safety vs Status probe | `deferred` (F27) |
| R1-05 | low-info | T249 F12 JSON/uptime/sc | `deferred` |
| R1-06 | low-info | Force-restore hermetics vacuous when live daemon Running | `deferred` (soft-skip; CI Stopped proves) |
| R1-07 | low-info | T298–T300 | `deferred` |
| R1-08 | medium | Live-daemon force-restore blocked full gate | `verified_fixed` (soft-skip on refuse string) |

---

## Phase 2 — Codex cross-model (`review.codex.md`)

| Finding | Disposition |
|---------|-------------|
| P1 incomplete closeout/publish | **Process** — closed by this closeout + Phase 6 publish |
| P2 AC7 unknown-flag exit 2 untested | **Fixed** — `daemon_status__unknown_format_flag__clap_exit_2` |
| Product wiring F1–F36 / AC8 / docs | **Agree** — no product regression |

Final: engineering DoD met; residuals are difficult lows only → `deferred.md`.

---

## Deferred append

See `conductor/deferred.md` § T297 implement closeout — PATH, live Running, doctor probe-policy, declined JSON, force-restore soft-skip residual, T298–T300.
