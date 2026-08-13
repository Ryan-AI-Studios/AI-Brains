# T247 Review Log — Nightly status residual

**Track:** T247-NightlyStatusResidual  
**Category:** OPS / BUGFIX / PERF / UX  
**Review requirement:** Cross-model (status + schedule parse)

---

## Findings

| ID | Severity | Status | Summary | Notes |
|----|----------|--------|---------|-------|
| — | — | — | Internal R1 completeness + correctness: no findings | Live AC8–AC10 recorded in plan.md |

### Status legend

- `open` — not yet fixed
- `fixed_pending_verification` — implementer claims fix; needs re-verify
- `verified_fixed` — reviewer confirmed
- `deferred` — medium/low only; justification + ISSUES.md

---

## Review passes

| Pass | Reviewer | Date | Result |
|------|----------|------|--------|
| Internal R1 completeness | explore subagent | 2026-08-13 | **CLEAN** |
| Internal R1 correctness | explore subagent | 2026-08-13 | **CLEAN** |
| Orchestrator manual AC8–AC10 | built `target\debug\ai-brains.exe` | 2026-08-13 | **PASS** — `--quick` 167 ms `probe=skipped` exit 0; default 926 ms (completion timeout / embedding ok) exit 0; missing `.cmd` + Last scheduled run 8/13 vs vault 2026-08-02; `--quick` without `--status` exit 2 |
| Cross-model CX1 | Codex gpt-5.6-luna high | 2026-08-13 | **Product CLEAN** — no P0/P2/P3 code findings. P1-01 process closeout (this PR). |
| Cross-model CX2 (final) | Codex gpt-5.6-luna high | 2026-08-13 | **PASS** — no P0–P3. CX1 P1-01 resolved by closeout `f68e917`. |

---

## Manual evidence (no live mutate)

```
cmd: target\debug\ai-brains.exe nightly --status --quick
elapsed_ms: 167
Last task result: 1
hint: process failed / missing action / CLI error
Action target missing: C:\Users\RyanB\.ai-brains\nightly-run.cmd
next: ai-brains nightly --schedule --dry-run
probe=skipped / probe=skipped
exit: 0

cmd: target\debug\ai-brains.exe nightly --status
elapsed_ms: 926
Completion probe=timeout ; Embedding probe=ok
Last scheduled run: 8/13/2026 3:00:01 AM
Last nightly run: 2026-08-02T07:03:58.159733500+00:00
exit: 0

cmd: target\debug\ai-brains.exe nightly --quick
exit: 2
```

F10 stop-before honored: did not reschedule or write `nightly-run.cmd`.

---

## Notes

- `"skipped"` is a string literal; no `ProbeStatus::Skipped`; models crate untouched.
- Status probes `tokio::join!` + 750 ms; run-path stays 2 s sequential.
- LIST /V primary; PS only after successful LIST /V with missing last_result.
- Conductor/deferred closeout is a second PR after product merge.
