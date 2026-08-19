# T267 review log — Next-action remediator honesty

**Track:** `conductor/tracks/trackT267-next-action-honesty`
**Category:** FEATURE / UX
**FEATURE TX:** `ce462dfb-bf75-4f63-9b8c-9356f886b457`
**Date:** 2026-08-18

## Scope

Harness `wiring=ok` is done: human `harness status` omits `next:`; JSON
`next_action` is the token `none`. Ready-trailer install lines print only
for present && not Ok. After install/uninstall, `next:` stays
`harness status`. `project list` stderr footer picks cwd path-owner, then
single-path, then orphan, then leftover; cwd git slug only when the picked
id is that owner. Whoami remediations stay T258. No leftover UUID, no clap 5,
no live `.env` / `cargo install`.

## Reviewers / rounds

| Round | Source | Verdict |
|-------|--------|---------|
| R1 | Implementer vs AC1–AC16 / F0–F22 / DoD | PASS |
| R1b | Independent explore | **PASS** (0 findings) |
| CX1 | Codex FEATURE `gpt-5.6-luna` high | Product P1 git-hard-fail |
| CX2 | Codex narrow recheck | **PASS** — P1-01 closed |

## Findings

| ID | Severity | Description | Status | Evidence |
|----|----------|-------------|--------|----------|
| CX1-P1-01 | high | `collect_git_identity` Err propagated; missing git failed `project list` | `verified_fixed` | print wrapper best-effort; `project_list__footer__git_unavailable__exit_0` PASS |
| CX1-P1-02 | medium (process) | Phase 5 / publish still pending when Codex ran | `verified_fixed` | Full gate + CX2 PASS + this closeout |

## DoD matrix (implementer)

| Item | Status | Evidence |
|------|--------|----------|
| AC1 `next_action_for` Ok=`none` | met | `next_action_for__ok__none` PASS |
| AC2 all-ok human omits self-next | met | `harness_status__all_ok__omits_self_next` PASS |
| AC3 all-ok JSON `next_action=none` | met | `harness_status__all_ok__json_next_action_none` PASS |
| AC4 missing still names install `--dry-run` | met | `harness_status__present_missing__next_is_install` PASS |
| AC5 whoami remediations T258 | met | `project_whoami__mismatch__remediations_name_adopt_path` PASS; `project.rs` remediations untouched |
| AC6 cwd owner unaliased + slug | met | `project_list__footer__cwd_owner_unaliased__picks_owner_slug` PASS |
| AC7 multi-path leftover + orphan | met | `project_list__footer__multipath_leftover_plus_orphan__picks_orphan` PASS |
| AC8 T212 AC3/AC4/AC5 | met | three `project_list_labels` tests PASS |
| AC9 all-ok omits ready trailers | met | `harness_status__all_ok__omits_ready_trailers` PASS |
| AC10 install success next=status | met | `harness_install__success__next_is_status` PASS |
| AC11 docs | met | CAPABILITIES harness omit/`none` + footer F3/F3b; PROTOCOL-COMPAT `next_action: "none"`; CHANGELOG T267; after_help one-liner |
| AC12 no DTO / no pin / no leftover UUID / line count | met | no contracts crate; clap lock 4.6.1; grep `7d97a456-f2f4-43ea-1f13-211af684ad37` empty in `crates/`; `project.rs` **1472** (was 1511) |
| AC13 manual source bin | met | `cargo run --bin ai-brains -- harness status`: 5× wiring=ok, **no** `next:`, **no** ready trailers. `project list` stderr `Example: set-alias 33ec90e0-… my-project` — **not** leftover+`AI-Brains` |
| AC14 pure pick/suggestion units | met | 6 units in `project_list_footer.rs` PASS |
| AC15 preflight Ok omits next | met | `format_harness_summary_lines__ok__omits_next` PASS |
| AC16 leftover-only basename | met | `project_list__footer__leftover_only__basename_not_cwd_slug` PASS |
| F2 / F7 / F8 / F9 / F11 / F18 | met | whoami unedited; install next kept; preflight summary unedited; no leftover UUID; peers declined; no `cargo install` |

## Targeted gates (observed)

- `cargo fmt --all` then `cargo clippy -p ai-brains-cli --all-targets -- -D warnings` exit 0
- Units AC1/AC14/AC15: 8/8 PASS
- Hermetics `--test next_action_honesty`: 8/8 PASS
- T212 AC3/AC4/AC5 + T258 AC7: 4/4 PASS

## Full gate (observed)

- `.\scripts\dev-check.ps1` **[SUCCESS] CI Gate passed!** nextest **3143** passed (1 skipped)
- `ledgerful verify --scope full` **passed** (fmt 2.5s / clippy 7.0s / nextest 183.1s / deny 2.4s / audit 2.7s)

## Residual / decline

- Harness status no `value_parser` (T266 F25 / Family B)
- PATH until operator `cargo install` (F18)
- Live leftover still owns many `C:\dev\*` roots (T259 operator rebind)
- Daily 0 of 3 grants (T241)
- T268 scan-roots parent / `--root`
- T265 / T269 / T270 / T271 / T272 stay placeholders
