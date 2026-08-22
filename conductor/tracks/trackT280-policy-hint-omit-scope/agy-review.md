# Track review: T280-PolicyHintOmitScope

**Harness:** Antigravity (`agy`)  
**Track:** `conductor/tracks/trackT280-policy-hint-omit-scope`  
**Date:** 2026-08-22  
**HEAD:** `f35884e`  

---

## Summary

Track T280 resolves an ergonomics and honesty inconsistency in policy error remediation discovered during the 2026-08-21 CLI audit:
When a command is denied with `POLICY_DENIED` (exit code 3), the error envelope’s `details.hint` instructed operators and agents to run `ai-brains policy bootstrap --scope …` (forcing a redundant `--scope` parameter), even though `doctor` and `policy show` already explain that `--scope` should be omitted when project context is authoritative. Furthermore, the markdown briefing renderer still embedded the old `--scope …` next-step while the JSON packet used the updated short SOOT.

T280 resolves these discrepancies cleanly:
1. **Deny Hint Scope Omission (F1):** Updates `POLICY_DENIED_HINT` across all three decoupled definitions (`ai-brains-cli`, `ai-brainsd`, and `ai-brains-control-plane`) to:
   ```
   ensure a grant for this capability exists; run `ai-brains policy bootstrap --dry-run` then `ai-brains policy bootstrap` (omit --scope when project context is authoritative)
   ```
2. **Briefing Markdown & JSON Alignment (F2):** Sets `BRIEFING_DENIED_NEXT_STEP` equal to `BRIEFING_DENIED_DENIAL_HINT` (`"next: run \`ai-brains policy bootstrap --dry-run\` then \`ai-brains policy bootstrap\`"`), unifying the human markdown rendering with the machine-readable JSON packet.
3. **Preserving Established SOOT Constants:** Keeps `POLICY_BOOTSTRAP_SOOT_SHORT` (preflight/show) and `POLICY_BOOTSTRAP_SOOT_LONG` (doctor) frozen.
4. **Preserving No-Context Validation:** Retains `fail_usage` (exit code 2) when commands are executed with `--no-project-context` without an explicit `--scope`.

The plan is well-bounded, maintains capture independence, and leaves hotspots (`project.rs`, CLI `preflight.rs`, `doctor.rs`) untouched.

---

## Findings (B/M/m/O)

### Blockers (B)
*None.*

### Major (M)
*None.*

### Minor (m)
- **m1: Dual-site byte-equality verification (F1 / AC1–AC3):** Ensure unit tests across all three decoupled locations (`governed_common.rs`, `services.rs`, `query.rs`) assert exact string equality against the F1 literal to prevent subtle drift across crates.
- **m2: Markdown next-step ordering preservation (F2 / AC4):** Verify that in `renderer.rs`, updating `BRIEFING_DENIED_NEXT_STEP` preserves the relative order where the remediation line precedes `## Decisions` and the T275 grant-wall.

### Opportunities (O)
- **O1: Documentation alignment in `CLI-EXIT-CODES.md` (F19 / AC11):** Update line 94 of `CLI-EXIT-CODES.md` to reflect the updated `POLICY_DENIED` remediation hint.
- **O2: Tightening hermetic bootstrap test assertions (AC5):** In `tests/policy_bootstrap.rs`, update `policy_bootstrap__deny_hint__contains_bootstrap` to assert `!hint.contains("--scope …")` and `hint.contains("omit --scope")`.

---

## What Looks Solid

1. **Copy-Paste Remediation Honesty:** Eliminates misleading prompt guidance that previously caused agents to invent redundant or unparseable `--scope` values when running within project repositories.
2. **Unification of Briefing Surfaces:** Harmonizes markdown next-steps with JSON denial hints.
3. **Decoupled Architecture Respect:** Maintains three separate byte-equal string constants with unit tests rather than introducing an unnecessary shared dependency from daemon to CLI.
4. **Hotspot Restraint:** Zero edits to `project.rs`, CLI `preflight.rs`, `doctor.rs`, `sync.rs`, or `policy_cmd.rs`.

---

## Deferred Fold-In Table

| Deferred Item | Spec/Plan Disposition | Assessment |
|---------------|------------------------|------------|
| Deny/`policy show` `--scope …` vs doctor omit | Absorbed into DoD (F1–F4 / AC1–AC7 / AC10) | Solved via HINT update + markdown next unification |
| T275 F11 HINT leftover | Absorbed (F1) | Updated to include dry-run and omit-scope parenthetical |
| T241 F14 markdown T227 leftover | Absorbed (F2) | Unified markdown next-step with SHORT denial hint |
| T243 AC12 freeze | Lifted (F1 / F27) | Updated to new exact-string freeze |
| Runtime two-arm HINT | Declined (F4) | Static omit-parenthetical is sufficient and simpler |
| Last-PR Cursor #195 | N/A (empty) | Scanned with 0 findings |

---

## Last-PR Cursor Comments

- **Scanned PR:** [#195](https://github.com/Ryan-AI-Studios/AI-Brains/pull/195) (merged 2026-08-22, T279 `Safety live hotspots and leading GLOB`).
- **Cursor Comments:** 0 comments (`[]` on PR #195).
- **Disposition:** N/A (no pending findings).

---

## Research / Tools Notes

- **CLI Design Guidelines:** clig.dev recommends suggesting exact next commands and promoting safe dry-run steps prior to mutating system state.
- **Dependencies:** `clap` (4.6.1), `serde_json` (1.0.150), `rusqlite` (0.39.0), `chrono` (0.4.44), `uuid` (1.23.1).
- **Toolchain / Rust:** `1.95.0` (Edition 2024), workspace `0.1.1`.
- **`ledgerful` / `ai-brains`:**
  - `ai-brains preflight --summary`: Scope `3581317d`, 3,547 pinned memories, 3 active sessions.
  - `ledgerful ledger status --compact`: 0 pending, 0 unaudited drift.
  - `ledgerful search POLICY_DENIED_HINT`: Located at `crates/ai-brains-cli/src/commands/governed_common.rs:51`, `crates/ai-brainsd/src/services.rs:989`, and `crates/ai-brains-control-plane/src/query.rs:93`.

---

## Verdict: Planned

The plan is approved as **Planned**. Implementation should proceed under TDD once the user issues `/implement-track`.
