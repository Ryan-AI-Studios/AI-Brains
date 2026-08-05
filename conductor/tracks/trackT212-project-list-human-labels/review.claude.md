Confirmed it's untracked/new (no diff since not in main). I have sufficient evidence now. Let me write the final review.

## Verdict: **PASS**

## Scope Reviewed

Working tree on `track/T212-project-list-human-labels` vs `main` (uncommitted + untracked), per spec.md/plan.md F1ΓÇôF42/AC1ΓÇôAC12:

- `crates/ai-brains-store/src/lib.rs` ΓÇö `QueryStore::list_projects_detail`, `ProjectListDetail`
- `crates/ai-brains-store/src/query_store.rs` ΓÇö detail SQL (path scalar subquery), `list_projects` tie-break
- `crates/ai-brains-cli/src/commands/project.rs` ΓÇö `list`, `list_json`, `print_unaliased_footer`, `display_label`, `truncate_chars`, `format_last_activity`, unit tests
- `crates/ai-brains-cli/src/main.rs` ΓÇö `ProjectCommands::List{format}`, after_help, dispatch
- `crates/ai-brains-cli/tests/project_list_labels.rs` (new, hermetic)
- `crates/ai-brains-cli/tests/smoke.rs` (friendly-name regression update)
- `Docs/CAPABILITIES.md`, `CHANGELOG.md`
- `conductor/conductor.md`, `conductor/deferred.md`

Confirmed no unrelated code was moved/touched (diff is additive and scoped to T212). Did not run `cargo test`/`cargo check`/`cargo clippy` ΓÇö sandbox denied build-tool invocation this session; verification below is static (code + hermetic test source read, cross-checked against schema/migrations and all call sites).

## Requirement and DoD Matrix

| ID | Status | Evidence |
|---|---|---|
| AC1 | **Met** | `project_list__alias_acme__human_label_contains_acme` asserts label-first header + alias in stdout |
| AC2 | **Met** | `display_label` strips baked `(no alias) ΓÇö short` to literal `(no alias)`; hermetic test asserts first field and rejects `(no alias) ΓÇö` in the row |
| AC3 | **Met** | `print_unaliased_footer` uses `eprintln!`; hermetic asserts stderr has `project set-alias` + id, stdout does not |
| AC4 | **Met** | Empty vault returns before footer call; hermetic asserts no `set-alias` on stdout or stderr |
| AC5 | **Met** | `--format json` value_parser-restricted; envelope schema matches F9; test asserts `unaliased_count == 1` exactly (not loose) for a 2-project fixture |
| AC6 | **Met** | SQL `COALESCE(MAX(mp.updated_at), p.updated_at)`; hermetic pins a memory and asserts non-empty `last_activity` |
| AC7 | **Met** | `list_projects` signature unchanged `(String,String,String,usize)`; `init.rs`/`resolve`/`detect`/`set_alias` all still call it; only `VaultConnection` implements `QueryStore` |
| AC8 | **Met** | CAPABILITIES ┬º5 documents columns, stderr footer, JSON schema, last_activity semantic (memory-projection mutation, not chat-only), path honesty; CHANGELOG `Unreleased` entry present |
| AC9 | **Not independently re-run this session** (build tools unavailable in sandbox); no production `unwrap`/`expect`/`panic!` found in `project.rs` (single `expect` is inside `#[cfg(test)]`); grep found no TODO/FIXME/`unimplemented!`/`dbg!` in touched files |
| AC10 (soft) | **Deferred as planned** ΓÇö path subquery implemented, no hermetic seed; correctly marked soft in spec/plan |
| AC11 | **Met** | `truncate_chars` uses `.chars().take(n)`, no byte slicing; unit test covers CJK + em-dash at width boundary |
| AC12 (soft) | **Met** | `AI_BRAINS_PROJECT_ID` match prefixes `*` on human label only; hermetic test covers it |

| Frozen decision | Status | Notes |
|---|---|---|
| F4 display_label order | **Met** | alias ΓåÆ `(no alias)` prefix strip ΓåÆ `Project <uuid-ish>`/full/short id match ΓåÆ name; manual string ops only (no regex, F42) |
| F5 columns | **Met** | `label \| project_id \| memories \| last_activity \| path` |
| F6 path subquery | **Met** | Correlated scalar subquery with `ORDER BY normalized_path ASC LIMIT 1`; not a multi-row JOIN ΓÇö no duplicate-row risk |
| F7 last_activity semantic | **Met** | SQL matches; CAPABILITIES documents honestly; `updated_at` is written via `Utc::now().to_rfc3339()` elsewhere in the store, confirmed consistent with `format_last_activity`'s primary RFC3339 parse path |
| F8 footer stderr | **Met** | `eprintln!`; empty-vault short-circuits before it; JSON path never calls it |
| F9 `--format json` only | **Met** | clap `value_parser = ["human","json"]`, no dual `--json`; schema fields match; `path` is `null` (no `skip_serializing_if`) when absent, matching F38 |
| F11 keep 4-tuple | **Met** | Verified via grep of all `list_projects()` callers |
| F13/F41 ORDER BY both | **Met** | Both `list_projects` and `list_projects_detail` add `, p.project_id ASC` |
| F14 truncation widths | **Met** | Label 30 chars, path 40 chars, project_id never truncated |
| F16 active `*` | **Met** | Human-only; JSON has optional `active` field, omitted when false/absent |
| F36 char-safe truncate | **Met** | Old byte-slice `&name[..min(30,len)]` (confirmed present in `main`'s version via diff) replaced entirely |
| F18 no new crates/no clap bump | **Met** | Only `chrono`/`serde_json`, both pre-existing deps |
| F26 (soft) git suggestion | **Met (bonus)** | `footer_alias_suggestion` reuses existing `get_git_repo_slug` |

## Findings

None at P0ΓÇôP2. No placeholders, stubs, silent fallbacks, or skipped tests found in the T212 diff.

## Completeness Sweep

- No TODO/FIXME/`unimplemented!`/`todo!`/`dbg!` in any touched file.
- The two low-severity items noted in the implementer's internal self-review (`review.md`: loose `unaliased_count >= 1` assert; stale smoke assert message) are **already fixed** in the current working tree ΓÇö `project_list_labels.rs:350` now asserts `unaliased_count == 1` exactly, and `smoke.rs`'s assert message was updated to match label-first behavior. Nothing outstanding from that self-review.
- `review.codex.raw.log` confirms Codex hit its usage-limit error before producing any findings ΓÇö consistent with the "Codex rate-limited" note; this review supersedes it as the cross-model check.

## Wiring and Regression Review

- `main.rs` dispatch: `ProjectCommands::List { format } => commands::project::list(&ctx, format)` ΓÇö reachable in production, not just tests.
- Only `VaultConnection` implements `QueryStore`, so the new trait method could not silently be a no-op elsewhere.
- All 5 production call sites of `list_projects()` (`init.rs`, and 4 in `project.rs`: resolve/detect/detect env-fallback/set_alias) still consume the unchanged 4-tuple; the added `ORDER BY project_id ASC` tie-break only removes previously-undefined ordering for equal `memory_count` rows and cannot regress correctness.
- Grepped all `smoke.rs` `.arg("list")` call sites: 10 of 11 are `backup list` (unrelated); the one `project list` case (`test_project_list_friendly_default_name`) was reviewed and updated correctly for label-first output.
- `empty_states_exit_hygiene.rs` and `vault_key_bootstrap.rs` project-list regressions check only the T198 empty line and missing-key error family ΓÇö unaffected by the column/format changes.
- Default project name generation (`context.rs:136`, `(no alias) ΓÇö {short}`) is exactly the form `display_label` special-cases; the legacy `Project <uuid>` form (no longer produced by any code path but present in old data per spec baseline) is still handled by the F4 step-3 branch.

## Verification Evidence

- Read full diffs of every touched production and test file against `main`.
- Confirmed schema (`migrations/0003_project_projection.sql`, `0006_memory_projection.sql`) is `NOT NULL` on `updated_at`, so `last_activity` COALESCE always yields a value ΓÇö consistent with the code's defensive empty-string handling.
- Confirmed timestamp write format (`chrono::Utc::now().to_rfc3339()`) matches the parser's primary path in `format_last_activity`.
- Grepped for any other `impl QueryStore` ΓÇö none, confirming F11.
- Grepped for stray `--json` flags, regex usage in `display_label`, and unwrap/expect/panic outside `#[cfg(test)]` ΓÇö none found.
- Could not execute `cargo test`/`clippy`/`nextest` ΓÇö blocked by sandbox policy for this read-only review session (same limitation the internal self-review flagged for AC9); this is a gate to run before/at merge, not a defect in the implementation.

## Deferred Candidates

None proposed. AC10 (path hermetic seed) and F24 (`--verbose` raw name) are already correctly tracked as soft/deferred in spec.md/plan.md ΓÇö no new deferred.md entry needed.

## Completion Decision

**PASS.** All hard AC1ΓÇôAC9/AC11 and frozen decisions F4/F6/F8/F9/F11/F13/F16/F36 are implemented, wired to production, and covered by hermetic/unit tests that would catch regressions (label stripping, stderr-only footer, JSON schema, multibyte truncate, 4-tuple preservation). Docs are accurate and match the shipped behavior, including the last_activity honesty note. No placeholders or scope narrowing found. Remaining gate before merge: run the full CI suite (`fmt`/`clippy -D warnings`/`nextest`/`deny`/`audit`) as normal operator/CI process ΓÇö not a review-blocking gap given the static evidence collected.
