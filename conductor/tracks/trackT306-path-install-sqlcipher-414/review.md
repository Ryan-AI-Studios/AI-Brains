# T306 Review Log — PATH install SQLCipher 4.14

**Track:** T306-PathInstallSqlcipher414  
**Category:** CHORE / OPS  
**CHORE TX:** `927f9b00-c0a6-4fd1-833b-ddf4772baa90`  
**Branch:** `track/T306-path-install-sqlcipher-414`  
**Date:** 2026-08-26

## Scope

Operator PATH reinstall of `ai-brains` via exact `GRAPH_REINSTALL_SOOT` (`cargo install --path crates/ai-brains-cli --locked --features graph`). Prove live `doctor --json` `cipher_page` message contains **`4.14`**. No product `src/` / lock / SOOT edits. No daemon stop. No live encrypt/rebuild. F14: codex-review **not** required (CHORE install-only, zero crate edits).

## DoD / AC matrix

| AC | Status | Evidence |
|----|--------|----------|
| AC1 PATH install + mtime newer | **Met** | `where` cargo bin; Length **26,501,632**; LastWriteTime **2026-08-26 6:54:32 AM** (was 2026-08-25 14:47:44) |
| AC2 `cipher_page` contains `4.14` | **Met** | `cipher_version=4.14.0 community` |
| AC3 `graph_feature=available` | **Met** | message `available`; `graph update --format json` returned nodes/edges |
| AC4 `vault_open` ok | **Met** | `opened read-only` |
| AC5 no key leak | **Met** | filtered `--json` scan — no `AI_BRAINS_KEY` / `x'<64 hex>'` |
| AC6 empty crate/lock diff | **Met** | `git diff -- crates/ Cargo.toml Cargo.lock` empty |
| AC7 degraded summary ok | **Met** | `status=degraded` warn=`graph_density` + `recovery_kit_event` (expected) |
| AC8 conductor + deferred R3 | **Met** | Completed + R3 Done |
| AC9 Phase 0 pins | **Met** | lock 0.40.2; Perl v5.42.2; cwd AI-Brains |

## Manual evidence

```text
# Pre-install (Phase 0)
cipher_page True cipher_version=4.10.0 community

# F1 elevated retry
cargo install --path crates/ai-brains-cli --locked --features graph
→ Finished release in 2.47s; Replaced … ai-brains.exe

# Post-install filtered doctor --json
vault_open True opened read-only
cipher_page True cipher_version=4.14.0 community
graph_feature True available

# --summary (AC7)
doctor: status=degraded  ok=11 warn=2 fail=0 skip=2
attention: recovery_kit_event; graph_density E/N=0.409
```

Did not `daemon stop`. Did not `vault encrypt` / `graph rebuild`. Did not print `AI_BRAINS_KEY`. Did not edit crates.

## Internal findings

| ID | Severity | Description | Status |
|----|----------|-------------|--------|
| R1 | low-info | First F1 replace hit Access denied (PID 28316 hung `ai-brains preflight --summary`). Cleared after elevated reload; daemon untouched. | **deferred** — ops note; not product |
| R2 | low-info | PATH `ai-brainsd` still 4.10-era (mtime 2026-08-22); mixed CLI 4.14 / daemon 4.10 | **deferred** — F8 / **T310** |
| R3 | low-info | T84 `run_update` omits `--features graph` — `ai-brains update` would undo graph-on | **deferred** — F9 / **T310** |
| R4 | low-info | `graph_density` still sparse E/N≈0.409; remediator `graph rebuild` | **deferred** — AC7 / **T308** |
| R5 | low-info | `recovery_kit_event` doctor warn | **deferred** — not this track |
| R6 | low-info | INSTALL.md header still says 0.1.2 | **deferred** — docs drift; not DoD |
| R7 | low-info | Harness reinstall after cargo install (OPERATIONS soft) | **deferred** — F24 soft residual |

No critical / high / medium. Easy DoD items closed by F1 + `--json` proof. F14: no codex-review.

## Gates

- Product tree unchanged → full workspace nextest **not** required for PATH 4.14 proof (F29).
- `ledgerful verify --scope fast` on conductor closeout.
- Phase 6 docs PR still watches GHA CI.
