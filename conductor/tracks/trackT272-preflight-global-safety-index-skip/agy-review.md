# Track review: T272-PreflightGlobalSafetyIndexSkip

**Harness:** Antigravity (`agy`)  
**Track:** `conductor/tracks/trackT272-preflight-global-safety-index-skip`  
**Date:** 2026-08-20  
**HEAD:** `9fcfcd8`  

---

## Summary

Track T272 fixes a retrieval over-exclusion bug identified by Cursor Bugbot on PR [#179](https://github.com/Ryan-AI-Studios/AI-Brains/pull/179) (T264):
Under `--global`, `safety_ids` was populated from the entire Safety fetch window (`LIMIT 40`) *before* `take_round_robin` capped the section to 2 pins per project and 8 pins total. Because the downstream Memory Index and Recent loops skipped every `memory_id` in `safety_ids`, memories that were fetched for Safety but capped out by round-robin selection vanished completely from the preflight context window.

T272 resolves this by:
1. Carrying `memory_id` through the Safety pipeline via the generic extra parameter `T = (Option<String>, String)` in `dedup_hotspots_keyed` and `take_round_robin`.
2. Rebuilding `safety_ids` from the surviving entries *after* HOTSPOT deduplication and round-robin capping, rather than inserting during the raw SQL fetch loop.
3. Ensuring Memory Index and Recent sections skip only the memory IDs that are actually *emitted* in the Safety section, allowing capped-out constraints to be indexed and discoverable in multi-project rollups.
4. Preserving all T264 invariants: fetch limits, round-robin constants, project tags, and T180/T265 JSON contracts remain completely untouched.

The specification and test plan are minimal, well-targeted, and adhere to all project standards.

---

## Findings (B/M/m/O)

### Blockers (B)
*None.*

### Major (M)
*None.*

### Minor (m)
- **m1: Adequate word budget in hermetic AC2 test:** When asserting that capped-out pin `A-one` appears in `index_section(&stdout)`, ensure the test invocation specifies an adequate word budget (default `-m 1500` or higher) so that `trim_to_word_budget` does not trim the Memory Index section before assertion.
- **m2: Unit test coverage for `dedup_hotspots_keyed` extras (AC1):** Include a unit test asserting that when two hotspot entries share the same file path, `dedup_hotspots_keyed` retains the `memory_id` of the freshest entry while dropping the duplicate, so only the kept entry's ID enters `safety_ids`.

### Opportunities (O)
- **O1: Test helper symmetry (`index_section`):** Implementing `fn index_section(stdout: &str) -> String` in `crates/ai-brains-cli/tests/preflight_global_isolation.rs` alongside `safety_section` ensures clean, readable section-bounded assertions.
- **O2: Code comment at rebuild site (F1):** Documenting `// Rebuild safety_ids from emitted entries so capped-out pins remain visible in Index (T272)` directly above the HashSet collection in `crates/ai-brains-retrieval/src/preflight.rs` will prevent future regression.

---

## What Looks Solid

1. **Root-Cause Fix:** Rebuilding `safety_ids` post-pipeline is the exact architectural remedy for Bugbot finding #179. It aligns the Index exclusion set with what is visually and structurally presented to the agent.
2. **Minimal Blast Radius:** The changes are confined to `crates/ai-brains-retrieval/src/preflight.rs` and hermetic test additions in `crates/ai-brains-cli/tests/preflight_global_isolation.rs`. Top hotspots (`project.rs`, `sync.rs`, `daemon.rs`, CLI `preflight.rs`) are untouched.
3. **No Retuning of T264 Constants:** The track avoids retuning `GLOBAL_SAFETY_*`, `GLOBAL_INDEX_*`, or SQL fetch limits, preserving the balance established in T264.
4. **Capture Independence & Determinism:** The solution relies strictly on standard library in-memory collections (`HashSet`, `Vec`) and deterministic SQLite `ORDER BY updated_at DESC` ordering without background models or external dependencies.

---

## Deferred Fold-In Table

| Deferred Item | Spec/Plan Disposition | Assessment |
|---------------|------------------------|------------|
| Cursor Bugbot #179 Safety IDs over-exclude Index | Absorbed into DoD (F1–F3 / AC1–AC4) | Directly resolved by rebuilding `safety_ids` post-cap |
| Latent post-dedup hotspot over-exclude | Absorbed into DoD (F1 / AC1) | Addressed by same post-pipeline rebuild |
| Project-scoped shown-safety skip | Absorbed into DoD (F2 / AC3) | Preserves project-scoped exclusion of shown pins |
| T264 Index fetch 80 leftover-heavy | Declined (F17) | Correctly preserved as soft residual |
| Session `HOTSPOT:` content skip | Declined (F18) | Correctly deferred as soft residual |
| T265 JSON format / CLI splitter edits | Declined (F7 / F19) | Preserves T180/T265 JSON contracts |
| Peer tracks (T270, T273) | Declined (F16) | Kept strictly isolated |

---

## Last-PR Cursor Comments

- **Scanned PR:** [#186](https://github.com/Ryan-AI-Studios/AI-Brains/pull/186) (merged 2026-08-20, T269 `Nightly vs Router status split`).
- **Cursor Comments:** None (`[]` on PR #186).
- **Source Finding:** Bugbot finding from PR [#179](https://github.com/Ryan-AI-Studios/AI-Brains/pull/179) (`c5c3a0d4-408f-4ff8-8d39-b3961707fe1a`) is the subject of this track.
- **Disposition:** N/A for PR #186; PR #179 finding is fully absorbed into T272 DoD.

---

## Research / Tools Notes

- **CLI Design Guidelines:** [clig.dev](https://clig.dev/) principles ("Human-readable output is paramount", "Saying just enough") support ensuring relevant pinned constraints are not silently hidden from all sections of a preflight briefing.
- **`rusqlite` / `serde_json` / `clap`:** Locked at `0.39.0`, `1.0.150`, and `4.6.1`.
- **Toolchain / Rust:** `1.95.0` (Edition 2024), workspace `0.1.1`.
- **`ledgerful` / `ai-brains`:**
  - `ai-brains preflight --summary`: Scope `3581317d`, 3,224 pinned memories, 3 active sessions.
  - `ledgerful ledger status --compact`: 0 pending, 0 unaudited drift.
  - `ledgerful search safety_ids`: Located in `crates/ai-brains-retrieval/src/preflight.rs:286/329/467`.

---

## Verdict: Planned

The plan is approved as **Planned**. Implementation should proceed under TDD once the user issues `/implement-track`.
