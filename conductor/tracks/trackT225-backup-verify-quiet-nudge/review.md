# T225 — Review Log

**Track:** T225-BackupVerifyQuietNudge  
**Orchestrator:** Grok  
**Ledger TX:** `2b97a127-cdd6-4973-802a-b4218ac94479` (FEATURE)

## Reviewers / rounds

| Round | Reviewer | Verdict | Notes |
|-------|----------|---------|-------|
| R1 internal | general-purpose subagent (read-only) | **CLEAN** | AC1–AC10, AC13 Met |
| R2 Codex cross-model | gpt-5.6-luna high | **FAIL** | P1-1 process; P2-1 doctor test discrimination; P2-2 verbose+json |
| R2 fix | implementer subagent | done | P2-1 + P2-2 test hardening |
| R2 Codex re-review | gpt-5.6-luna high | **FAIL** | P2-1 date still non-discriminating (2026-08-01 plain stale) |
| R2b fix | orchestrator | done | plain → `vault-2099-12-31…` future in-window |
| R3 Codex re-review | gpt-5.6-luna high | **PASS WITH DEFERRED P3** | P2 closed; F17 soft only |
| R-final Codex | gpt-5.6-luna high | **PASS WITH DEFERRED P3** | Fresh final after governance reconcile; F17 soft only |

## Scope implemented

- Pure `verify_report.rs`: counts, FAIL preview cap=5, trailer, create-nudge predicate
- `run_verify`: quiet default / verbose stream-only / JSON full; progress `debug!`
- Clap `--verbose` on `backup verify`
- Doctor `backup_recent`: usable = Readable\|PreT109; zero usable or stale usable → create
- Smoke M1/M2 + multi-fail + AC2; doctor_cli net-new hermetics (+ P2 hardening)
- CAPABILITIES §11, CHANGELOG, OPERATIONS soft

## Findings

### Codex R1

| ID | Sev | Disposition | Notes |
|----|-----|-------------|-------|
| P1-1 Track closure incomplete | P1 process | **verified_fixed** | Conductor Completed; deferred struck; series README; PR #128 merged; ledger committed; pin recorded |
| P2-1 Doctor PreT109 / mixed-age proof weak | P2 | **verified_fixed** | PreT109 DROP meta ok; stale 2020 usable + future 2099 plain → warn (discriminating) |
| P2-2 Missing `--verbose --format json` test | P2 | **verified_fixed** | mixed OK+FAIL JSON equality with/without `--verbose` |

### Soft residuals (defer after ship)

- F7 optional 3-class fail rollup omitted (M5 allowed)
- F17: verify `--quiet`, JSON summary field, structured `VerifyError` / 4-class

## Gate evidence

| Check | Result |
|-------|--------|
| Targeted nextest (verify/doctor/backup_verify) | 57 passed (implementer) |
| Full gate | **PASS** 2026-08-11: fmt, clippy, nextest **2521** passed (1 skipped), deny, audit |
| ledgerful verify --scope full | **PASS** |
| Manual dogfood | **PASS** — 21-file fleet quiet (0 OK/21 FAIL, ≤5 FAIL—, trailer, nudge, exit 1); verbose stream; JSON 21; doctor warn+create; hermetic 1 OK exit 0; empty exit 0 |
| Codex R1 | FAIL → fixed
| Codex R3 | **PASS WITH DEFERRED P3**
| Codex final | pending re-run after deferred.md table fix |

## Disposition policy

- Critical/High → fix before ship
- Medium → fix by default
- Low/P3 → fix if easy; else deferred.md with justification


## Completion decision

**Completed** 2026-08-11. Engineering DoD met; Codex R3 PASS WITH DEFERRED P3 (F17 soft only); PR #128 squash-merged `927b8db`; CI Win/Linux/macOS green; ledger TX committed; pin recorded.

Raw final: `review.codex.final.md`.
