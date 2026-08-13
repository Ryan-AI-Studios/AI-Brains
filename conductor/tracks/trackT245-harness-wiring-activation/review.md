# T245 Review Log — Harness wiring activation

**Track:** T245-HarnessWiringActivation  
**Category:** FEATURE / OPS / UX  
**Review requirement:** Hard cross-model (F25) — user-global hooks + PATH bake

---

## Findings

| ID | Severity | Status | Summary | Notes |
|----|----------|--------|---------|-------|
| R1-C1 | medium | verified_fixed | `probe_agy` returned Unknown on corrupt IDE before bundle OR | Fall through to `agy_cli_plugin_bundle_ok`; `probe_agy__corrupt_ide_with_plugin_bundle__ok` |
| R1-N1 | low | verified_fixed | Live F10/AC13 dogfood on real homes | 2026-08-12: three wiring=ok; doctor 3/3 ready; bundle exists; no top-level CLI hooks.json |
| R1-N2 | low | deferred | `pending_track()` still `T239+` on status/install-pending | Doctor F6 uses literal T253; F13 fence |
| R1-N3 | low | deferred | Doctor message only unit-tested, not `doctor` CLI hermetic | Helper is SOOT; `check_harness_wiring` still `ok_msg` |

### Status legend

- `open` — not yet fixed
- `fixed_pending_verification` — implementer claims fix; needs re-verify
- `verified_fixed` — reviewer confirmed
- `deferred` — medium/low only; justification + ISSUES.md

---

## Review passes

| Pass | Reviewer | Date | Result |
|------|----------|------|--------|
| Internal R1 completeness | explore subagent | 2026-08-12 | NEEDS_FIX (F7b corrupt-IDE skips bundle) |
| Internal R1 correctness | explore subagent | 2026-08-12 | CLEAN |
| Internal R1 fix | orchestrator | 2026-08-12 | `90fc12a` + unit PASS |
| Internal R2 F7b re-review | explore subagent | 2026-08-12 | verified_fixed |
| Cross-model CX1 (final) | Codex gpt-5.6-luna high | 2026-08-12 | **PASS WITH DEFERRED P3** (no P0–P2) |

---

## Notes

- F6 all-ok+pending message has **no** `T253` token (pinned second arm). Ready-missing arm includes `T253`.
- OpenCode marker remains `// AI-Brains managed (T238)`.
- Conductor/deferred closeout is a second PR after product merge.
- CX1 deferred P3s = R1-N2 (`pending_track` still `T239+`) and R1-N3 (doctor message helper-only). Qualifying lows; not DoD.
- Local gates: `cargo fmt --check` PASS; `clippy --workspace --all-targets -- -D warnings` PASS; `cargo nextest run --workspace` **2746 passed** (1 skipped).
