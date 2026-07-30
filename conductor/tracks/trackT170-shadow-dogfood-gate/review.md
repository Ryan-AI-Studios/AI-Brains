# T170 Review Log

## R1 — Internal review (2026-07-30)

**Verdict:** NEEDS_FIX → fixes applied (`fixed_pending_verification`)  
**Reviewer:** explore subagent (read-only)  
**Implementer:** Grok Build (R1 fix pass)

### Findings

| ID | Severity | Status | Summary |
|----|----------|--------|---------|
| R1-01 | medium | fixed_pending_verification | Stage B fixture project_id persisted + passed to `briefing project --project-id`; pin non-zero fails Stage B; runbook §2 updated |
| R1-02 | medium | fixed_pending_verification | D20: remove evaluate-report + `--allow-report-overwrite`; clear compare partials on re-run; second run exit 0 verified |
| R1-03 | medium | fixed_pending_verification | Full Stage B under `%TEMP%\t170-stageb-*`: evaluate exit 0, compare_hash recorded, rollback probe executed, evidence/ updated |
| R1-04 | medium | fixed_pending_verification | plan.md Phase E reconciled: executed rollback + honesty checkbox for denied fixture briefing |
| R1-05 | medium | fixed_pending_verification | BOM-less `WriteAllText` for JSON captures; BOM check first-byte `{` on artifacts |
| R1-06 | low | fixed_pending_verification | `sort_compare_arrays` sorts `claim_ids_sample`; unit test order-independent hash |
| R1-07 | low | fixed_pending_verification | `--migrate-report` CLI flag → `paths.migrate_report`; script passes when file exists |
| R1-08 | low | open | Optional Stage B seed integration assertion deferred (easy path partial via unit tests + evidence run) |

No critical/high. Stage C/D deferrals accepted.

### Evidence pointers (R1-03)

- `evidence/stage-b-notes.md` — commands, D24 N/A (locked vault), rollback flag 0/1, honesty on denied briefing
- `evidence/stage-b-compare-summary.json` — compare_hash + hard_checks + basenames only
- `evidence/stage-a-evaluate-summary.json` — report_hash stable

### Test gate (implementer)

```
cargo clippy -p ai-brains-cli --all-targets -- -D warnings   # ok
cargo nextest run -p ai-brains-cli -E 'test(dogfood) | test(compare)'  # 13 passed
```
