# T202 Review Log — Recall + Briefing Clarity

## R1 — Internal (subagent, 2026-08-04)

**Verdict:** NEEDS FIXES (3 Medium, 2 Low)

| ID | Severity | Description | Status |
|----|----------|-------------|--------|
| F-R1-01 | Medium | Briefing CLI help still claims JSON default | fixed_pending_verification |
| F-R1-02 | Medium | OPERATIONS still JSON-by-default for briefing | fixed_pending_verification |
| F-R1-03 | Medium | AC7 renderer unit lock missing | fixed_pending_verification |
| F-R1-04 | Low | CLI-EXIT-CODES omits fail_usage progressive class | fixed_pending_verification |
| F-R1-05 | Low | CAPABILITIES silent on briefing TTY / progressive exit | fixed_pending_verification |

### Fixes applied (R1 → R2)

- `main.rs` briefing format help: markdown on TTY, json otherwise
- OPERATIONS F9 honesty + progressive exit-2 note + capability table
- CLI-EXIT-CODES row 2 + `fail_usage` section
- CAPABILITIES briefing + progressive table
- AC7 units: project `denied_markdown` + personal without_grant markdown assert

## R2 — Internal re-check (orchestrator, 2026-08-04)

| ID | Status | Evidence |
|----|--------|----------|
| F-R1-01 | verified_fixed | help doc-comments updated |
| F-R1-02 | verified_fixed | OPERATIONS surface header + table |
| F-R1-03 | verified_fixed | nextest AC7 unit PASS |
| F-R1-04 | verified_fixed | CLI-EXIT-CODES updated |
| F-R1-05 | verified_fixed | CAPABILITIES section added |

**Verdict:** CLEAN for >Low findings (all Medium+ fixed). Proceed to full gate + Codex cross-model.

## Codex R1 — cross-model (gpt-5.6-luna, 2026-08-04)

**Verdict:** FAIL (P1×2, P2×4)

| ID | Sev | Disposition |
|----|-----|-------------|
| P1-01 endpoint/detail secret leak | P1 | **Validated** → fixed: `public_endpoint_label` + stable detail codes |
| P1-02 string classify mis-bucket | P1 | **Validated** → fixed: `classify_model_error` typed + provider-class wins |
| P2-01 budget erases denied warning | P2 | **Validated** → fixed: preserve denied under budget + unit |
| P2-02 hermetic suite thin | P2 | **Partly valid** — AC3/AC3b/AC5 unit per F18; hermetic AC2/AC9–10 present. Not blocking. |
| P2-03 parent briefing help JSON default | P2 | **Validated** → fixed parent help + OPERATIONS scope |
| P2-04 process DoD open | P2 | **Process** — closeout after CI/merge (not product defect) |

## Codex R2 — re-review after fixes (pending)

Re-run after P1/P2 product fixes + full gate.
