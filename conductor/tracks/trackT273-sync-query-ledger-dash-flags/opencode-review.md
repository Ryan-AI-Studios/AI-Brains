# T273 Review — `sync query` dash-leading Ledgerful flags

- **Track:** T273-SyncQueryLedgerDashFlags (BUGFIX)
- **Reviewed at:** HEAD `ee3f127` (docs/conductor plan commit; product tree == `f3f6cbd` T268 squash). Tree CLEAN.
- **Method:** Plan review only. Spec + plan read in full; every code-truth claim re-verified against live `src/`; pins vs `Cargo.lock` + crates.io; `deferred.md` + last-PR Cursor audit; live CLI repros for the acceptance criteria.

## Summary

Sound, well-scoped BUGFIX plan. The root-cause chain is correctly identified (layer 2 only: `run_ledger_search` at `sync_query_ledger.rs:155-160` spawns `ledgerful ledger search [--json] <query>` with no POSIX `--`), the fix (always-on `--` via a single `ledger_search_argv` helper, F1/F2) is the minimal correct remediation, and the `allow_hyphen_values` decline is precisely grounded in clap 4.6.6 docs. One real defect found: **AC10 as written is a dead manual test** — `sync query -- --limit --no-bridge` can never pass because our clap treats `--no-bridge` after `--` as a positional (live: exit 2, `unexpected argument '--no-bridge' found`). The intent works with the flag before `--` (`sync query --no-bridge -- --limit`, live exit 0, Recall only). One-line fix in spec §4 AC10 + plan Phase 4. Verdict: **Planned after fixes**.

## Findings

### B-1 — AC10 is un-runnable as written (dead manual gate)
- **Where:** `spec.md` §4 AC10 (`sync query -- --limit --no-bridge` → no ledger section); reproduced verbatim in `plan.md` Phase 4.
- **Live truth (this machine):**
  - `cargo run -p ai-brains-cli -- sync query -- --limit --no-bridge` → `error: unexpected argument '--no-bridge' found`, exit 2. After POSIX `--`, clap stops option parsing; `--no-bridge` is a stray positional. This manual check can never be green.
  - Correct form: `sync query --no-bridge -- --limit` → Recall section only, no ledger pane, exit 0. Live-verified.
- **Impact:** On go, the implementer runs a manual AC that fails for the wrong reason, risks misreading it as "the `--` approach is broken," and the DoD gate records a false red.
- **Fix (apply at fold-in):** spec AC10 → `sync query --no-bridge -- --limit`; plan Phase 4 line → same. Optionally add a one-line note in AC10 that flags go *before* `--` (layer-1 clap semantics), matching F5's already-correct example `sync query --quiet -- --limit`.
- **Grade: B** (spec/plan defect inside the acceptance criteria; trivial fix, blocking for a clean green DoD).

### m-1 — F5/AC10 ordering rule could be made explicit once
- `spec.md` F5 already documents the correct idiom (`sync query --quiet -- --limit`), but AC10 contradicts it by placing `--no-bridge` after `--`. The B-1 fix makes the two consistent. No further action; note only so fold-in verifies the final wording agrees.

### O-1 — AC8 exit-code assertion is not pinned in the plan
- `spec.md` AC8 ("exit 2") is a live-verified claim, but the plan's Phase 1 clap test `sync_query__bare_limit_flag__still_requires_value` (AC8) does not state whether it asserts `ErrorKind`/exit-code or just that parse fails. Recommend the unit assert the clap error kind (`MissingRequiredArgument`) so the red/green is falsifiable — minor.

### O-2 — `--quiet` honesty check is correctly optional but should run
- Plan Phase 4 marks `--quiet -- --limit` prints-the-pane as optional. This is the one check that proves the "Quiet hides the hole" risk (spec §8) is actually closed. Recommend it be required, not optional, since it is cheap and directly validates the risk row.

## What looks solid

- **Root cause is real and precisely located.** `run_ledger_search` (`sync_query_ledger.rs:149-168`) builds `["ledger","search","--json", query]` (:157) and `["ledger","search", query]` (:159). All three spawn sites (:250 phrase JSON, :294 token JSON, :176 human re-run) route through it — one helper covers all (F2 holds).
- **Always-on `--` beats conditional insert.** Spec §5.2 reasoning is correct: `query.starts_with('-')` would miss multi-word queries with embedded flag-like tokens (`"foo -l"`) and short clusters. Always-on matches git/cargo and is what live Ledgerful accepts (verified: `ledgerful ledger search --json -- --limit` → exit 0, rows id ≥136).
- **`allow_hyphen_values` decline is doc-grounded.** clap 4.6.6 `Arg::allow_hyphen_values` WARNING verbatim: *"Known flags get precedence over the next possible positional argument with `allow_hyphen_values(true)`."* So putting it on `Query.query` would NOT make `sync query --limit` a needle — the decline rationale in F5 and §2.2 is accurate. `trailing_var_arg` doc ("users still have the option to explicitly escape ambiguous arguments with `--`") supports the F1 approach.
- **Rescue honesty preserved.** `ledger_rescue_tokens` splits via `extract_fts_tokens` (`fts.rs:28-34`, split on non-alphanumeric), so dash needles yield alnum tokens (`--limit extra` → `["limit","extra"]`); rescue never produces a dash-leading argv element, and routing tokens through the same helper (spec §2.2 hypothesis row) is consistent. T271 miss/classifier/quiet invariants (`ledger_classify_outcome` :99-116, `ledger_quiet_omits_pane` :119-125) are untouched.
- **Test citations are concrete and exist.** `sync_query__no_bridge__skips_ledgerful_section` (`smoke.rs:86`), `sync_query__no_bridge_ndjson__only_local_records` (:131), T211 ranking hermetics (`tests/sync_query_ranking.rs`, `sync_query_ranking__*` :50-153), T231 resolve units (`sync.rs:589-637`), and `Cli::try_parse_from` precedent (`main.rs` tests, e.g. :98, :265) all verified present. AC7's `try_parse_from` approach has 20+ in-tree precedents.
- **Clap pins verified.** Lock `Cargo.lock:1337` clap 4.6.1 vs crates.io 4.6.6 (2026-08-11, no clap 5); serde_json lock 1.0.150 vs 1.0.151. No bump — right call for a no-new-flags bugfix.
- **Touch map is disciplined.** Helper + units stay in `sync_query_ledger.rs`; clap parse + `after_help` in `main.rs` (where `Cli` lives); `sync.rs`/`project.rs`/`recall.rs` explicitly not touched. No DTO/daemon surface → no contract work needed.
- **Empty-query and lone-`--` hazards handled** (F18/F19, §5.4): empty forward returns never-ran before any spawn; helper may be unit-tested with `""` but production never builds that argv.

## Deferred fold-in table

| Item | Source | Disposition to apply |
|------|--------|----------------------|
| Dash-leading QUERY parsed as Ledgerful flags | #183 Bugbot Medium; T271 closeout; stub | **Absorb** F1–F4 / AC1–AC5 / AC9 |
| Token rescue never starts after clap fail | #183 body | **Absorb** F4 (argv is the remediator; T271 F6 stands) |
| Placeholder F1–F4 | T273 stub | **Absorb** |
| T90 on ledger argv | T90 / T271 F5 | **Affirm decline** F3 |
| T211 F25 blend / T217 MATCH OR | T211/T217 residuals | **Decline** |
| T268 scan-roots | Completed | **Decline** |
| T269 / T270 / T272 | Pending peers | **Decline** — do not steal |
| T240 F2 / T255 bag | standing | **Decline** |
| last-PR #184 Linux Path units | #184 Bugbot Medium | **Decline** F8 — already `#[cfg(windows)]` (`project_paths.rs:639+`); no T274 |
| recall `bridge_search_args` | live `recall.rs` (`["search","--auto-index","--json", query]`) | **Decline as DoD** F7 — soft residual, different crate |
| Our Query `allow_hyphen_values` | clap 4.6.6 docs | **Decline** F5 — known `--limit` still wins |
| clap 5 / pin bumps / DTO / schema_version | standing | **Decline** F10 / F13 |
| **AC10 dead form (NEW)** | this review B-1 | **Apply**: spec §4 AC10 + plan Phase 4 → `sync query --no-bridge -- --limit` |

## Last-PR Cursor comments

- **PR #184 (T268, merged 2026-08-20)** — Bugbot Medium "Windows hint units fail on Linux": correctly declined (F8). Units are `#[cfg(windows)]` with Unix counterpart (`project_paths.rs:639-697`); T268 review P1 fixed; no T274. Agreed — decline is correct.
- **PR #183 (T271, merged)** — Bugbot Medium "Dash queries parsed as ledgerful flags" at `sync_query_ledger.rs:154-160`: still true on live source (confirmed `:157`), and is this track's root cause. **Absorb** via F1-F4.
- No other open PRs on HEAD (Dependabot remotes only).

## Research / tools notes

- **clap 4.6.6** (docs.rs): `allow_hyphen_values` — "Prior arguments with `allow_hyphen_values(true)` get precedence over known flags but known flags get precedence over the next possible positional argument with `allow_hyphen_values(true)`." `trailing_var_arg` — "users still have the option to explicitly escape ambiguous arguments with `--`." Verified verbatim; both cited accurately in spec §2.4.
- **Live repros** (all this machine): `sync query -- --limit` → ledger pane `a value is required for '--limit <LIMIT>'` (hole); `ledgerful ledger search --json -- --limit` → exit 0 ≥1 rows (control); `sync query --no-bridge -- --limit` → Recall only, exit 0 (correct AC10 form); `sync query --limit` → exit 2 MissingRequiredArgument (AC8).
- **Pins:** clap lock 4.6.1 / crates.io 4.6.6 (no clap 5); serde_json lock 1.0.150 / crates.io 1.0.151; rustc 1.95.0; edition 2024. Snapshot — re-verify at execute (plan Phase 0 already does this).
- **Tools:** `rg` unavailable → `Select-String`. `ledgerful doctor` git found; gemini not found (environment note only). Ledger planning TX `1d4391ae-3769-4cfa-9d04-8be1c7f138bd` (DOCS); implement opens BUGFIX TX on go. `conductor/ISSUES.md` does not exist (verified).

## Verdict

**Planned after fixes.** Core design (F1-F4, helper, always-on `--`, declines) is correct and fully verified against live source. Blocking item: B-1 — fix AC10 + plan Phase 4 wording to `sync query --no-bridge -- --limit` (one line each) before go so the manual gate is runnable. Optionally tighten O-1 (assert clap error kind) and make O-2 (`--quiet` honesty check) required. No re-plan needed.

End chat with **`/fold-in T273`**.
