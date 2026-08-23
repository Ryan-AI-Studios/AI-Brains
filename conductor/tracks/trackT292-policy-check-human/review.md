# T292 Review Log — policy check human

**Track:** T292-PolicyCheckHuman  
**Category:** FEATURE / UX  
**FEATURE TX:** `d4349589-0318-4973-b614-2b38db25b8b9`  
**Branch:** `track/T292-policy-check-human`

## Phase-1 internal (implementer)

### DoD / AC matrix

| AC | Status | Evidence |
|----|--------|----------|
| AC1 allow/deny helper units | `verified_fixed` | `format_policy_check_*` units in `policy_cmd.rs` |
| AC2 allow human hermetic | `verified_fixed` | `policy_check_human` omit-principal bootstrap |
| AC3 deny human + SHORT / stderr empty | `verified_fixed` | deny two lines; no `POLICY_DENIED:` on stderr |
| AC4 JSON allow keys | `verified_fixed` | no `next_step` / `found` |
| AC5 stay-green json deny / catalog / soft-resolve | `verified_fixed` | exit_contract + governed_surface + soft_resolve + bootstrap suite |
| AC6 clap JSON/Pretty InvalidValue + default auto | `verified_fixed` | `main.rs` clap units |
| AC7 pipe omit-format JSON | `verified_fixed` | hermetic omit format |
| AC8 show/bootstrap default json | `verified_fixed` | help units |
| AC9 docs + help catalog | `verified_fixed` | CAPABILITIES/OPERATIONS/CLI-EXIT/PROTOCOL + help unit |
| AC10 Manual | `verified_fixed` | `cargo run` allow human/json + Propose deny human |
| AC11 CheckResult no new fields | `verified_fixed` | struct unchanged; AC4 keys |
| AC12 pretty/md/text ≡ human | `verified_fixed` | hermetic aliases |

### Findings

| id | severity | description | status | evidence |
|----|----------|-------------|--------|----------|
| I1 | low-info | PATH still pre-T285 until `cargo install` (F13) | deferred | expected residual |
| I2 | low-info | Propose* human deny still names SHORT bootstrap (F24) | deferred | by design |
| I3 | low-info | show/bootstrap TTY still JSON (F26) | deferred | Family D peers |

No open medium+. No regressions observed on AC5 suite.

## Phase-2 cross-model

Pending Codex (`review.codex.md`).
