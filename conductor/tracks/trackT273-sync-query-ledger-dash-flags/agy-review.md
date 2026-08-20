# Track review: T273-SyncQueryLedgerDashFlags

**Harness:** Antigravity (`agy`)  
**Track:** `conductor/tracks/trackT273-sync-query-ledger-dash-flags`  
**Date:** 2026-08-19  
**HEAD:** `f3f6cbd`  

---

## Summary

Track T273 resolves a bug identified by Cursor Bugbot on PR [#183](https://github.com/Ryan-AI-Studios/AI-Brains/pull/183) (T271):
When an operator passes a query string that starts with a dash (e.g. `--limit`, `--days`, `--breaking`, or `--json`) to `ai-brains sync query -- <needle>`, the downstream invocation of `ledgerful ledger search` in `run_ledger_search` forwarded the needle as the raw argument directly after `search` or `--json`. As a result, Ledgerful's clap parser consumed the query as a CLI option (e.g. `-l/--limit`), causing a CLI usage failure (`a value is required for '--limit <LIMIT>' but none was supplied`) instead of performing a positional search.

T273 solves this by:
1. Introducing a pure helper `ledger_search_argv(query, json)` that always inserts the POSIX option terminator `--` immediately before the `<QUERY>` argument (`["ledger", "search", "--json", "--", query]` or `["ledger", "search", "--", query]`).
2. Ensuring `run_ledger_search` routes all phrase and token queries through this helper.
3. Adding explicit `after_help` documentation on `SyncCommands::Query` explaining the use of `--` for dash-leading query strings.
4. Preserving all T271 invariants: no T90 quotes on ledger argv, token rescue only after successful empty phrase search, and capture independence.

The specification and test plan are minimal, well-targeted, and adhere to all project standards.

---

## Findings (B/M/m/O)

### Blockers (B)
*None.*

### Major (M)
*None.*

### Minor (m)
- **m1: Literal `"--"` query token edge case (AC4):** In addition to testing flags like `--days`, `--breaking`, `--json`, `-l`, `-d`, `-b`, include a unit test case for needle `"--"` (`ledger_search_argv("--", true)` $\rightarrow$ `["ledger", "search", "--json", "--", "--"]`). This confirms that literal double-dash queries are safely formatted without ambiguity.
- **m2: Clear distinction in `after_help` (F6 / AC12):** The `after_help` block for `SyncCommands::Query` in `main.rs` should explicitly contrast `ai-brains sync query -- --limit` (searching for the text `"--limit"`) with `ai-brains sync query "text" --limit 10` (setting the vault result limit to 10).

### Opportunities (O)
- **O1: Pure helper visibility:** Marking `pub(crate) fn ledger_search_argv` in `sync_query_ledger.rs` facilitates direct unit testing in `mod tests` without exposing internal argv generation outside the crate.
- **O2: Scope containment on retrieval code search:** Citing `recall.rs:536` (`bridge_search_args`) as a soft residual rather than inflating this track's scope preserves BUGFIX focus.

---

## What Looks Solid

1. **Live Reproduction Confirmed:** Running `cargo run -p ai-brains-cli -- sync query -- --limit` on HEAD `f3f6cbd` reliably reproduces the `Ledger search failed: error: a value is required for '--limit <LIMIT>'` error, while running `ledgerful ledger search --json -- --limit` succeeds with exit 0 and returns valid matches.
2. **Minimal Blast Radius:** The change is confined to `sync_query_ledger.rs` (argv generation + units) and `main.rs` (`after_help` + clap parse unit), avoiding churn in `sync.rs` or `ai-brains-retrieval`.
3. **No Regressions on Layer 1 / Clap Invariants:** The plan avoids attempting `allow_hyphen_values` on `Query.query`, respecting clap's standard precedence rules where known flags like `--limit` take priority unless preceded by `--`.
4. **Honest Error Handling:** Flag-parse failures from Ledgerful remain classified as `Failed` rather than falsely triggering token rescue.

---

## Deferred Fold-In Table

| Deferred Item | Spec/Plan Disposition | Assessment |
|---------------|------------------------|------------|
| Dash-leading QUERY parsed as Ledgerful flags | Absorbed into DoD (F1–F4 / AC1–AC5 / AC9) | Direct fix via POSIX `--` option terminator |
| Token rescue never runs after clap error | Absorbed into DoD (F4) | Fixed by preventing false clap error on dash queries |
| T90 FTS sanitization on ledger argv | Affirmed decline (F3) | Preserves T271 unsanitized ledger search design |
| Query `after_help` missing | Absorbed into DoD (F6 / AC12) | Documents POSIX `--` escape pattern |
| Last PR #184 Linux path units | Declined (F8) | Verified already resolved with `#[cfg(windows)]` on HEAD |
| `retrieval/src/recall.rs` bridge search args | Declined as DoD (F7) | Properly deferred as soft residual |
| T269, T270, T272 peer tracks | Declined (F9) | Kept isolated |

---

## Last-PR Cursor Comments

- **Scanned PR:** [#184](https://github.com/Ryan-AI-Studios/AI-Brains/pull/184) (merged 2026-08-20, T268 `scan-roots parent / --root`).
- **Cursor Bugbot Comment:** Medium finding on `crates/ai-brains-cli/src/commands/project_paths.rs:639+` regarding Windows-style path tests running on Linux without `#[cfg(windows)]`.
- **Disposition:** Spec §2.1 and §9 correctly identified that this issue was already resolved prior to merge on HEAD `f3f6cbd` (where Windows tests are gated with `#[cfg(windows)]` and Unix tests are provided). No additional placeholder or track is needed.

---

## Research / Tools Notes

- **`clap`:** Locked at `4.6.1`. POSIX `--` option termination is the standard CLI convention across clap derive applications.
- **`serde_json`:** Locked at `1.0.150`.
- **Toolchain / Rust:** `1.95.0` (Edition 2024), workspace `0.1.1`.
- **`ledgerful` / `ai-brains`:**
  - `ai-brains preflight --summary`: Scope `3581317d`, 3,146 pinned memories, 2 active sessions.
  - `ledgerful ledger status --compact`: 0 pending, 0 unaudited drift.
  - `ledgerful search run_ledger_search`: Located in `crates/ai-brains-cli/src/commands/sync_query_ledger.rs:149`.

---

## Verdict: Planned

The plan is approved as **Planned**. Implementation should proceed under TDD once the user issues `/implement-track`.
