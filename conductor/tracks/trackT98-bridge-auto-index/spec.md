# Track T98: Pass `--auto-index` to ChangeGuard Bridge Calls

**Status:** Pending
**Started:** —
**Owner:** —
**Priority:** P3 — UX polish; eliminates "stale index" warnings during AI-Brains operations.
**Source:** `C:\dev\testing_report.md` Item 2; ChangeGuard already supports `--auto-index`.

---

## Problem Statement

AI-Brains shells out to ChangeGuard for bridge queries (`changeguard search`, `changeguard ask`, `changeguard ledger search`). When the ChangeGuard index is stale (HEAD has changed since last index), these commands print a warning and fall back to cached data. The user sees:

```
[33mGraph state: STALE (289 files affected) - run 'changeguard index'[39m
```

ChangeGuard already supports `--auto-index` on `search`, `ask`, `hotspots`, and `dead-code` commands — it automatically refreshes the index before executing. AI-Brains does not pass this flag, so the user must manually run `changeguard index --incremental` to clear the stale state.

AI-Brains should pass `--auto-index` when shelling out to ChangeGuard for bridge queries, so the LLM always gets ground-truth working-tree data without manual intervention.

## Acceptance Criteria

**AC1:** `query_ledgerful_bridge` in `crates/ai-brains-retrieval/src/recall.rs` passes `--auto-index` when invoking `ledgerful search`.

**AC2:** Verified that `bridge export` and `bridge query` commands do NOT support `--auto-index` (only `search`, `ask`, `hotspots`, `dead-code` do). Callsites in `preflight.rs`, `intervention.rs`, and `verification_gate.rs` are skipped because they use `bridge export`.

**AC3:** When `--quiet` is set on the AI-Brains command, the ledgerful call also suppresses stderr (`Stdio::null`) so the auto-index refresh output does not clutter the user's terminal. The index refresh happens silently.

**AC4:** No regression in existing tests. The `--auto-index` flag is a no-op when the index is already fresh.

## Design Notes

- The `--auto-index` flag tells ChangeGuard to run `index --incremental` internally before executing the query. It is fast (incremental, only changed files) and a no-op when the index is already current.
- For the `ledger search` call in `sync.rs`, `--auto-index` is NOT applicable (ledger search doesn't use the code index). Only `search`, `ask`, `bridge query`, and `bridge export` commands support it.
- The flag should be added to the `cmd.args([...])` array in each callsite. For `search`, it goes before the query: `["search", "--auto-index", "--json", &query]`.
- This track is low-risk because `--auto-index` is additive — if the flag is not supported by an older ChangeGuard version, the command fails, but AI-Brains already handles bridge failures gracefully (T81).

## Files

- `crates/ai-brains-retrieval/src/recall.rs` — `query_changeguard_bridge` (~line 248).
- `crates/ai-brains-retrieval/src/preflight.rs` — `query_changeguard` (~line 284) and `query_changeguard_fallback` (~line 383).
- `crates/ai-brains-brain/src/intervention.rs` — `query_changeguard_risk_alerts` (~line 227).
- `crates/ai-brains-capture/src/verification_gate.rs` — `query_changeguard_verification` (~line 173).
- `crates/ai-brains-graph/src/cozo_proxy.rs` — bridge import/export calls (~lines 157, 229) — evaluate whether `--auto-index` helps here (these are graph operations, not search; may not apply).

## Tests (TDD)

**Red:** `test_recall_passes_auto_index_to_bridge` — mock the ChangeGuard binary (or use a test harness that captures args), invoke `recall`, assert `--auto-index` is in the args list. Fails because the flag is not passed.

**Green:** Add `--auto-index` to the args. Test passes.

> Note: If mocking the binary is too complex for a unit test, a simpler approach: extract the arg list into a testable function (e.g. `fn bridge_search_args(query: &str, auto_index: bool) -> Vec<&str>`) and test that directly.

## Verification

- `cargo nextest run -p ai-brains-retrieval`
- Manual: modify a file in a git repo (so ChangeGuard index goes stale), run `ai-brains recall "test"`, confirm no "STALE" warning appears and the query uses fresh data.
- Manual: run `ai-brains preflight --summary` after a file change — confirm fresh hotspot data.

## Out of Scope

- Making `--auto-index` configurable via an AI-Brains env var (hardcode it for now; if it causes latency issues on large repos, a config flag can be added later).
- Changes to ChangeGuard's `--auto-index` implementation (already shipped).
- The `sync query` path's `changeguard ledger search` call (does not use the code index).