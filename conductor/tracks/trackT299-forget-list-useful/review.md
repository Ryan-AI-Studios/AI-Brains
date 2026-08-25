# T299 Review Log — forget list useful empty

**Track:** T299-ForgetListUseful  
**Status:** Cleared for publish (DoD + gates + Codex feature PASS)  
**Implementer:** Grok  
**Ledger FEATURE TX:** `8723f58f-04b9-4306-a241-f93a42322da8`

## Scope verified

| AC | Result | Evidence |
|----|--------|----------|
| AC1 | pass | `forget_list_forgotten__empty_with_pin__pinned_count_and_next` |
| AC2 | pass | `forget_list_forgotten__empty_matches_memory_list_status_forgotten` `assert_eq!` |
| AC3 | pass | `forget_list_forgotten__empty_zero_pins__pinned_zero_and_next` |
| AC4 | pass | `forget_list_forgotten__nonempty__omits_t299_remediator` |
| AC5 | pass | `forget_list_forgotten__empty_json__no_next_step_or_pinned` (exact nine-key set) |
| AC6 | pass | `forget_list_forgotten__global_empty__next_includes_global` |
| AC7 | pass | `memory_list__pinned_empty__omits_forgotten_next` |
| AC8 | pass | `memory_list__summary__omits_t299_next` |
| AC9 | pass | existing missing-scope exit 2 stay-green |
| AC10 | pass | `forgotten_empty_remediator__cases` rstest (4 cases) |
| AC11 | pass | `forget_list_forgotten__empty_tag__pinned_matches_summary_tag` |
| AC12 | pass | CAPABILITIES / OPERATIONS / WORKFLOWS / CHANGELOG / CLI-EXIT-CODES / after_help |
| AC13 | pass | `forget --help` lists `--list-forgotten` + `memory list` |
| AC14 | pass | Manual `cargo run` — `Pinned: 4161` matches `--summary`; **did not** forget live pins |
| AC15 | pass | No contracts/pin bumps; `forget.rs` production unchanged |
| AC16 | pass | `memory_list_inventory` 37/37; JSON freeze |

## Findings

| id | severity | description | status | evidence |
|----|----------|-------------|--------|----------|
| R1 | low-info | PATH 0.1.2 lacks T299 until owner install | deferred | F17 |
| R2 | low-info | Live Forgotten: 0 — empty Manual SoT | deferred | F13/F25 |
| R3 | low-info | Declined JSON/`--summary`/histogram/`--offset` | deferred | F10/F9/F24 |
| R4 | low-info | T300 not stolen | deferred | F24 |
| R5 | low-info | clap/rusqlite Dependabot not stolen | deferred | F14 |
| Codex P3-1 | low-info | Exact nine-key JSON assert | verified_fixed | Strengthened AC5 test |

No open critical/high/medium.

## Codex (`review.codex.md`)

- Feature implementation: **PASS**
- P1-1 full gate / P1-2 commit+publish: process pending at review time → **addressed** by `dev-check.ps1` SUCCESS + `ledgerful verify --scope full` + Phase 6 publish
- P2 closeout metadata: reconciled on Completed
- P3-1 exact key set: **fixed**

## Gates

- Targeted: remediator 4/4; inventory 37/37; clippy `-p ai-brains-cli` PASS
- Full: `.\scripts\dev-check.ps1` **SUCCESS** (fmt/clippy/nextest/deny/audit)
- `ledgerful verify --scope full` **Verification passed**

## Manual AC14 (2026-08-25)

```
cargo run -q -p ai-brains-cli -- forget --list-forgotten --limit 5
→ No forgotten memories. / Pinned: 4161 / next: ai-brains memory list

cargo run -q -p ai-brains-cli -- memory list --summary
→ Pinned: 4161 / Forgotten: 0
```
