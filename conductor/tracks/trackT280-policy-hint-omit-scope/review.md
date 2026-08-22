# T280 review log — Policy hint omit `--scope`

**Track:** T280-PolicyHintOmitScope
**Status:** Completed (full gate green; Phase 6 pending this commit)
**FEATURE TX:** `ebf7885d-68b8-47e2-918c-4f926b28a74f`
**HEAD (implement):** `track/T280-policy-hint-omit-scope`

## Reviewers / rounds

| Round | Reviewer | Result |
|-------|----------|--------|
| R1 | Implementer (Grok) vs spec AC1–AC14 / DoD | **PASS** — red then green; three F1 HINT copies; markdown next = SHORT; no live bootstrap / no `cargo install` |
| R1b | Explore subagent (read-only DoD) | **PASS WITH DEFERRED P3** — P3-1 OPERATIONS first-run + P3-2 CHANGELOG blanks fixed; P3-3 README closeout; P3-4 skill false positive (tracked `.claude` updated) |
| CX1 | Codex gpt-5.6-luna | **Product PASS**; P1-1/P1-2 process (gate + closeout incomplete at review time) |
| Gate | `dev-check.ps1` + `ledgerful verify --scope full` | **PASS** nextest **3296** / 1 skipped |

## Finding fields

id, severity, description, source, files, required_fix, status, evidence.

## Findings

| id | severity | description | source | files | required_fix | status | evidence |
|----|----------|-------------|--------|-------|--------------|--------|----------|
| — | — | R1: no product findings | R1 | — | — | — | Three `assert_eq!` F1 units PASS; hermetic AC5 PASS; classify-only cargo run HINT = F1 |
| P3-1 | low-info | OPERATIONS T221 first-run still taught only `--scope Repository:<uuid>` | R1b | `Docs/OPERATIONS.md:259` | Add omit-scope + dry-run | `verified_fixed` | Matches CAPABILITIES progressive first-run |
| P3-2 | low-info | Extra blank lines after T280 CHANGELOG bullet | R1b | `CHANGELOG.md` | Collapse blanks | `verified_fixed` | Single blank between T280 and T279 |
| P3-3 | low-info | Series README still says T280 Planned | R1b | `README-T274-T284-CLI-QUALITY.md` | Closeout Completed | `verified_fixed` | README + conductor Completed |
| P3-4 | low-info | Commit message claimed skill update vs `.agents` skill | R1b | `.claude/skills/ai-brains/SKILL.md` | — | `out_of_scope` | Tracked skill was updated; `.agents` project skill has no policy-bootstrap section (F19 skip) |
| P1-1 | high (process) | Full workspace gate unfinished at CX1 | CX1 | conductor + review.md | Run `dev-check.ps1` + `ledgerful verify --scope full` | `verified_fixed` (gate) | nextest **3296** passed / 1 skipped; verify `--scope full` exit 0 |
| P1-2 | high (process) | Track closeout unfinished at CX1 | CX1 | conductor.md / plan.md | Completed + publish | `fixed_pending_verification` | Closeout this commit; Phase 6 remaining |

## DoD matrix (AC1–AC14)

| AC | Status | Evidence |
|----|--------|----------|
| AC1 | Met | `policy_denied_hint__wording__omits_required_scope` — `assert_eq!` F1, len 172, no `--scope …` |
| AC2 | Met | daemon `policy_denied_with_hint__includes_details_hint` — `assert_eq!(POLICY_DENIED_HINT, F1)` + details.hint |
| AC3 | Met | CP `query::tests::policy_denied_hint__wording__omits_required_scope` on hoisted module-level const |
| AC4 | Met | `render_project_markdown__denied__next_step_omits_scope_ellipsis` — NEXT_STEP == DENIAL_HINT; order Denied → next → grant-wall → Decisions |
| AC5 | Met | `policy_bootstrap__deny_hint__contains_bootstrap` — exit 3; `--dry-run` + `omit --scope`; no `--scope …` |
| AC6 | Met | `policy_bootstrap__no_scope_no_context__exit_2` PASS |
| AC7 | Met | Doctor unit `omit --scope` / `authoritative` untouched (LONG). `cargo run -- policy show --format json` `next_step` = SHORT |
| AC8 | Met | Renderer T275 grant-wall + no `_None_` units PASS |
| AC9 | Met | `progressive__deny__stderr_code_and_hint_stdout_denial_hint` — `policy bootstrap` and not `--scope …`; recall fallback stays |
| AC10 | Met | `cargo run -p ai-brains-cli --quiet -- policy check --capability ReadEvidence --format json` → F1 HINT. **Did not bootstrap. Did not `cargo install`.** |
| AC11 | Met | CLI-EXIT-CODES no longer presents `--scope …` as the only form. CHANGELOG T280. CAPABILITIES omit note. PROTOCOL-COMPAT no new keys |
| AC12 | Met | Diff omits `project.rs` / CLI `preflight.rs` / `doctor.rs` / `sync.rs` / `policy_cmd.rs`. No clap/rusqlite bump |
| AC13 | Met | `policy_soft_resolve` 3 tests PASS |
| AC14 | Met | `render_personal_markdown__denied__names_recall_not_personal_bootstrap` — no project NEXT_STEP / GRANT_WALL |

## Targeted gates (R1)

```text
cargo nextest run -p ai-brains-cli policy_denied_hint__wording__omits_required_scope policy_bootstrap__deny_hint__contains_bootstrap
  2 passed
cargo nextest run -p ai-brainsd --lib policy_denied_with_hint__includes_details_hint
  1 passed
cargo nextest run -p ai-brains-control-plane --lib policy_denied_hint__wording__omits_required_scope render_project_markdown__denied__next_step_omits_scope_ellipsis
  2 passed
cargo nextest run -p ai-brains-cli --test policy_bootstrap --test governed_first_run_deny_exit --test briefing_format_substance
  33 passed
cargo nextest run -p ai-brains-control-plane --lib render_project_markdown render_personal_markdown briefing_denied
  10 passed
cargo nextest run -p ai-brains-cli --test policy_soft_resolve
  3 passed
cargo clippy -p ai-brains-cli -p ai-brainsd -p ai-brains-control-plane --all-targets -- -D warnings
  exit 0
cargo fmt --check
  exit 0
```

## Manual (classify-only)

```text
cargo run -p ai-brains-cli --quiet -- policy check --capability ReadEvidence --format json
  details.hint = F1 (omit --scope when project context is authoritative)
  Did not run live policy bootstrap.

cargo run -p ai-brains-cli --quiet -- policy show --format json
  next_step = SHORT (no --scope). EXIT=0
```

PATH `ai-brains` remains T270-era until `cargo install` (F13). Source/`cargo run` is DoD.
