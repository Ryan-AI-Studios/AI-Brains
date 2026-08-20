# T273 — `sync query` dash-leading strings must not be Ledgerful flags

- **Track ID:** T273-SyncQueryLedgerDashFlags
- **Status:** **Planned** (Pending in registry; plan-only until go)
- **Category:** BUGFIX
- **Owner:** —
- **Source:** Cursor Bugbot on PR [#183](https://github.com/Ryan-AI-Studios/AI-Brains/pull/183) (T271) — Medium “Dash queries parsed as ledgerful flags”
- **Depends on:** T271 ✅ Completed (FTS-quote lift + token rescue)
- **Blocks / feeds:** Operators can search the ledger for needles that look like flags (`--limit`, `--days`, `--breaking`, `--json`). Does **not** unblock T269 nightly split, T270 retention, T272 Safety skip.
- **Absorbs:** #183 inline review at `sync_query_ledger.rs:154–160`; placeholder F1–F4; T271 closeout “T273 minted”
- **Not absorbed (DoD):** Restore T90 `sanitize_fts_query` on the ledger argv; T211 F25 blend; Ledgerful crate edits / token-OR; our Query `allow_hyphen_values` (does not steal `--limit`); recall `bridge_search_args` (`ledgerful search` code); T269 / T270 / T272; last-PR #184 Linux Path units (already `#[cfg(windows)]`); clap 5 / pin bumps; contracts DTO
- **Research date:** 2026-08-19 (source HEAD `f3f6cbd`)
- **Ledger:** planning DOCS TX `1d4391ae-3769-4cfa-9d04-8be1c7f138bd`. Implement starts a **BUGFIX** TX on **go**.
- **Isolation:** Do **not** reopen T271 F5–F7 / F19 miss classes. Do **not** edit vault `sanitize_fts_query` callers. Do **not** grow hotspot `project.rs` or `sync.rs`. Do **not** `cargo install`, pin to the live vault, rewrite `.env`, or mutate schtasks.

---

## 1. Objective

1. **A dash-leading `sync query` needle is a QUERY.** After our clap has the string (quoted or after POSIX `--`), `ledgerful ledger search` must treat it as the positional QUERY, not as `-l/--limit`, `-d/--days`, `-b/--breaking`, or `--json`.
2. **Flag-parse failure must not masquerade as ran-empty.** T271 rescue starts only after a **successful** empty phrase probe. A Ledgerful clap error stays **failed** (and `--quiet` still omits that pane).
3. **Keep T271 honesty.** No T90 quotes on the ledger argv. Vault MATCH stays sanitized. `--no-bridge` still skips the pane. Capture independence: vault pane never waits on ledger.
4. **North star.** Operators can find shipped ledger rows that *mention* `--limit` (live T211/T217 entries) instead of a clap usage error.

---

## 2. Live baseline (re-scan 2026-08-19)

### 2.1 Operator dogfood (this machine)

| Signal | Observation |
|--------|-------------|
| HEAD | `f3f6cbd` — T268 squash-merged (#184). Tree **CLEAN**. `main` == `origin/main`. |
| PATH `ai-brains` | **0.1.1**. **Do not `cargo install`.** Tests/manual AC use `cargo run` / hermetic bin. |
| Source `cargo run -p ai-brains-cli -- sync query -- --limit` | Vault pane has hits. Ledger pane: **`Ledger search failed: error: a value is required for '--limit <LIMIT>' but none was supplied`**. **Live hole confirmed.** |
| Source `sync query "--limit" --quiet` | PowerShell still passes argv `--limit`. **Our** clap consumes it as vault `-l/--limit` (missing value) → exit **2**. Quotes are not a remediator at argv level. |
| Source `sync query --days` | Our clap: `unexpected argument '--days' found` + tip `to pass '--days' as a value, use '-- --days'`. Layer-1 already documents POSIX `--`. |
| `ledgerful ledger search --json --limit` | Exit **2**: `a value is required for '--limit <LIMIT>'`. Same for `--days` (value required), `--breaking` (missing `<QUERY>`), `--json --json` (cannot be used multiple times). |
| `ledgerful ledger search --json -- --limit` | Exit **0**. JSON array **≥1** (volatile: T211 “`--limit` 5” + T217 rows). **Ledgerful accepts `--`.** Phrase wrap searches the string `--limit`. |
| `ledgerful ledger search --help` | `<QUERY>` required. Flags: `-c/--category`, `-d/--days`, `-b/--breaking`, `-l/--limit` default 10, `--offset`, `--json`. No `allow_hyphen_values` on QUERY (`C:\dev\Ledgerful\src\cli\args\ledger.rs` `Search { query: String, ... }`). |
| Last GitHub PR | [#184](https://github.com/Ryan-AI-Studios/AI-Brains/pull/184) T268 (2026-08-20). **Cursor Bugbot Medium:** “Windows hint units fail on Linux.” **Already fixed** on this HEAD: those units are `#[cfg(windows)]` + Unix counterpart (`project_paths.rs:639–697`). T268 `review.md` P1 **fixed**. CI requires `ubuntu-24.04`. **Decline — do not mint T274.** Open PRs on this HEAD: Dependabot remotes only. |
| Prior mint source | [#183](https://github.com/Ryan-AI-Studios/AI-Brains/pull/183) Bugbot Medium still **true** at `sync_query_ledger.rs:157–159`. This track. |
| Identity / doctor | Scope `3581317d`; grants 0 of 3 (T241); ledgerful doctor leftover `.changeguard` / sig-pin / timings. 0 pending / 0 drift at plan scan. Do not “fix” here. |

### 2.2 Why the pane is a clap error

Two layers. T273 owns **layer 2**.

```text
Operator:  ai-brains sync query -- --limit
  layer 1 — our clap (already works)
    Query.query = "--limit"     // POSIX -- is clap 4 default
  layer 2 — we spawn (the hole)
    ledgerful ledger search --json --limit
      Ledgerful clap: --limit is -l/--limit, QUERY missing → exit 2
      T271: nonzero → Failed; rescue never starts (needs successful empty [])
      --quiet: Failed pane omitted (looks like --no-bridge)
```

| Hypothesis | Live verdict |
|------------|--------------|
| Quotes around `--limit` bypass our clap | **False** (PowerShell / argv). Operator must use `--` *or* a needle that is not a known flag (`--days` still needs `--` today). |
| Ledgerful rejects POSIX `--` | **False.** Live `--json -- --limit` returns rows. |
| Rescue tokens are also dash-leading | **False.** `extract_fts_tokens` splits on non-alnum, so `--limit extra` → `["limit","extra"]`. Still route tokens through the same argv helper. |
| cwd / System32 | **False for this repro.** cwd is this repo. Keep T271 F2 guard. |
| Vault `--limit` should become the ledger needle without `--` | **Decline.** Known flags win (clap 4.6.6 docs). `sync query --limit 5` is vault cap. |

### 2.3 Code truth

| Item | Location | Notes |
|------|----------|-------|
| Hole | `sync_query_ledger.rs` `run_ledger_search` `:155–160` | `cmd.args(["ledger","search","--json", query])` and human `["ledger","search", query]`. |
| Probe | `probe_ledger_search` `:250` phrase JSON; `:294` token JSON; `:176` human re-run | All three call `run_ledger_search`. One helper must cover all. |
| Forward | `ledger_forward_query` | strip_ansi + trim. **Keep.** No T90. |
| Rescue | `ledger_rescue_tokens` | `contentful_tokens(extract_fts_tokens)` — alphanumeric tokens. |
| Classifier | `ledger_classify_outcome` | Nonzero clap usage → **Failed** (stderr has no `git`/`work directory`/`layout`). |
| Quiet | `ledger_quiet_omits_pane` | Failed + `--quiet` → omit. **Keep.** |
| Our clap | `main.rs` `SyncCommands::Query` `:2803` | `query: String` positional; `-l/--limit` default 5 (vault). **No** `allow_hyphen_values`. **No** `after_help`. |
| T271 units | `sync_query_ledger.rs` `mod tests` | Stay green. New argv units live here. |
| T271 hermetics | `smoke.rs` `sync_query__no_bridge__*` | Stay green. |
| Recall sibling | `retrieval/src/recall.rs` `bridge_search_args` | `["search","--auto-index","--json", query]` — **code** search, not `ledger search`. Soft residual, not DoD. |
| Hotspots | `project.rs` **#1** (4.036); `sync.rs` **#2** (3.730) | Argv helper stays in `sync_query_ledger.rs`. Do not grow either hotspot. |
| Contracts / daemon | none | No DTO. No HTTP. |

### 2.4 Dependency / standards research (2026-08-19)

**Snapshot — re-verify at execute.**

| Pin | Workspace / lock | Ecosystem (today) | Action |
|-----|------------------|-------------------|--------|
| `clap` | workspace **4.5** / lock **4.6.1** | crates.io **4.6.6** (2026-08-06). docs.rs **4.6.6**. **No clap 5.** | **No bump.** No new flags. |
| `serde_json` | lock **1.0.150** | crates.io **1.0.151** | **No bump.** |
| rustc / edition | **1.95.0** / **2024** | — | Unchanged |
| nextest | **0.9.140** | — | Unchanged |
| workspace | **0.1.1** | — | **No bump** |
| New crates | — | — | **Zero.** |

**How to implement (primary sources):**

| Source | What it locks |
|--------|----------------|
| [POSIX / clig — bare `--`](https://clig.dev/) + [clap 4.6.6 `Arg::trailing_var_arg`](https://docs.rs/clap/4.6.6/clap/struct.Arg.html#method.trailing_var_arg) | `--` ends options. “Users still have the option to explicitly escape ambiguous arguments with `--`.” |
| [clap 4.6.6 `Arg::allow_hyphen_values`](https://docs.rs/clap/4.6.6/clap/struct.Arg.html#method.allow_hyphen_values) | “Known flags get precedence over the next possible positional.” Putting this on our `query` would **not** make `sync query --limit` a needle. **Decline** as DoD. |
| Live Ledgerful `ledger search --help` + `Search { query: String }` | QUERY is a plain positional. Live `-- --limit` works. Do **not** edit the Ledgerful repo. |
| T271 F5 / F6 / F19 | No T90 on argv; rescue after successful empty only; clap-usage stderr is Failed. |
| N/A | No new CLI framework, no contracts schema, no Ledgerful `allow_hyphen_values` patch. |

Could not verify: crates.io Ledgerful (sibling local product). Verified against **this machine’s** `C:\dev\Ledgerful` source + live `ledgerful.exe`.

---

## 3. Frozen decisions

| ID | Decision |
|----|----------|
| **F0 — Go gate** | Plan-only until user **go**. Planning is DOCS. Implement starts a **BUGFIX** TX. |
| **F1 — POSIX `--` on every ledger search argv** | Pure helper `ledger_search_argv(query, json) -> Vec<String>` emits `ledger search [--json] -- <query>`. **Always** insert `--`, including non-dash needles. Query is last. |
| **F2 — One helper** | `run_ledger_search` (phrase JSON, token JSON, human re-run) uses **only** `ledger_search_argv`. Do not special-case `query.starts_with('-')`. |
| **F3 — No T90** | Affirm T271 F5. Forwarder stays `strip_ansi.trim()`. Never `sanitize_fts_query` on this argv. |
| **F4 — Rescue / miss classes unchanged** | Affirm T271 F6 / F8 / F19. Flag-parse nonzero stays **Failed**. Rescue still requires successful empty `[]`. `--quiet` still omits Failed. |
| **F5 — Layer 1 (our clap) already works** | Operators pass dash needles as `sync query -- --limit` (or after other flags: `sync query --quiet -- --limit`). Do **not** add `allow_hyphen_values` on `Query.query`. Do **not** rename/remove vault `--limit`. |
| **F6 — after_help required** | `SyncCommands::Query` has **no** `after_help` today. Add a small block: dash-leading needles need `--` before QUERY (`ai-brains sync query -- --limit`); our `--limit` remains the vault cap. |
| **F7 — Decline recall `bridge_search_args`** | `ledgerful search --auto-index --json <query>` is T98/T87 code search. Same-class one-liner is a **soft residual**, not DoD. Do not mint a new placeholder for it. |
| **F8 — Decline #184 Linux Path units** | Already `#[cfg(windows)]` + Unix parent unit on HEAD `f3f6cbd`. T268 review P1 fixed. GHA ubuntu job exists. **Not a leftover.** |
| **F9 — Decline peers** | T269 / T270 / T272 / T240 F2 / T255 bag / T211 F25 / T264 leftover recall drop. |
| **F10 — Decline extras** | No Ledgerful source edits; no vault `--limit` retarget; no `OR` argv; no json-v2 / DTO; no clap 5; no new crates; no `schema_version` on this pane. |
| **F11 — Module** | Helper + units in `sync_query_ledger.rs`. Clap parse AC + `after_help` in `main.rs` (where `Cli` lives). Do **not** grow `sync.rs` / `project.rs`. |
| **F12 — Capture independence** | Argv + docs only. No events. No models. Vault pane unchanged. |
| **F13 — Pins / crates** | No lock bumps. Workspace **0.1.1**. |
| **F14 — Docs** | CAPABILITIES: one additive clause on the T271 ledger-pane bullet (POSIX `--` before QUERY). Root CHANGELOG T273 row. Optional one-line OPERATIONS. No PROTOCOL-COMPAT. |
| **F15 — Tests** | Naming `function_or_feature__condition__expected_result`. Units first (red) on `ledger_search_argv`. T271 AC1–AC19 stay green. No `unwrap`/`expect`/`panic` in production. No PATH-hijack hermetic required. |
| **F16 — PATH-behind** | Do not `cargo install` unless the user asks. Manual AC uses `cargo run`. |
| **F17 — Review** | BUGFIX. Primary review required. Cross-model **optional** (no DTO, no architecture). |
| **F18 — Debt file** | `conductor/ISSUES.md` does **not** exist. Residuals → `conductor/deferred.md`. |
| **F19 — `--` is not the query** | Never forward a lone `--` as QUERY. Empty forward still never-ran (T271 F18) **before** argv is built. |
| **F20 — Human `--json` flag stays ours-not** | `sync query --json` remains unexpected (no Query `--json`). Operator who wants needle `--json` uses `sync query -- --json`. After F1, Ledgerful sees `--json -- --json`. |

---

## 4. Acceptance criteria

| AC | Proof |
|----|-------|
| **AC1** | Unit: `ledger_search_argv("--limit", true)` == `["ledger","search","--json","--","--limit"]` |
| **AC2** | Unit: `ledger_search_argv("--limit", false)` == `["ledger","search","--","--limit"]` (human re-run; no `--json`) |
| **AC3** | Unit: `ledger_search_argv("capture independence", true)` still contains `"--"` immediately before the query (always-on `--`) |
| **AC4** | Unit: for `--days`, `--breaking`, `--json`, `-l`, `-d`, `-b`: last argv is the needle; `"--"` is immediately before it; the needle is **not** adjacent to `"search"` without `"--"` |
| **AC5** | Unit: `ledger_forward_query` / rescue / miss-copy / classifier T271 tests stay green (no T90 quotes; empty query still never-ran **before** argv) |
| **AC6** | Hermetic `sync_query__no_bridge__skips_ledgerful_section` still: has `AI-Brains Recall`, **no** `Ledgerful Ledger Search` |
| **AC7** | Clap: `Cli::try_parse_from(["ai-brains","sync","query","--","--limit"])` → `Query.query == "--limit"` (layer 1 lock) |
| **AC8** | Clap: `sync query --limit` without a value still `MissingRequiredArgument` / “value is required for '--limit'” exit **2** (vault flag stands) |
| **AC9** | Manual (on go): `cargo run -p ai-brains-cli -- sync query -- --limit` from this repo → ledger pane has **≥1** table row **or** ran-empty quoting `'--limit'`. Must **not** print `a value is required for '--limit <LIMIT>'` |
| **AC10** | Manual: `sync query -- --limit --no-bridge` → no ledger section |
| **AC11** | Manual: `ledgerful ledger search --json -- --limit` still ≥1 (control — we did not break Ledgerful) |
| **AC12** | Docs: CAPABILITIES T271 pane bullet names POSIX `--`; CHANGELOG has a T273 row; Query `--help` `after_help` contains `sync query -- --limit` |
| **AC13** | Existing T211 ranking + T231 resolve hermetics stay green (`--no-bridge` path) |

---

## 5. Design notes

### 5.1 Argv helper

```text
fn ledger_search_argv(query: &str, json: bool) -> Vec<String> {
    let mut args = vec!["ledger".into(), "search".into()];
    if json { args.push("--json".into()); }
    args.push("--".into());
    args.push(query.into());
    args
}
```

`run_ledger_search` becomes `cmd.args(ledger_search_argv(query, json))`. No other `cmd.arg*` for the query.

### 5.2 Why always `--`

Conditional insert (`if query.starts_with('-')`) misses `-l` short clusters and future needles. Always-on `--` matches git/cargo and is what live Ledgerful already accepts.

### 5.3 Layer 1 vs layer 2

| Layer | Who | Remediator | This track |
|-------|-----|------------|------------|
| 1 | our clap | operator `--` / clap’s own tip | Lock with AC7/AC8 + `after_help`. Do not steal `--limit`. |
| 2 | Ledgerful clap | we insert `--` | **DoD F1.** |

### 5.4 Empty query

T271 F18: empty after strip+trim → never-ran **before** spawn. F19: do not build argv for that arm. Helper may still be unit-tested with `""` (would emit `--` + empty string) but production must not spawn it.

---

## 6. Non-goals

- Editing `C:\dev\Ledgerful` (phrase wrap, `allow_hyphen_values` on their QUERY, token-OR).
- Making `sync query --limit` (no `--`) mean “search for `--limit`”.
- `allow_hyphen_values` on our Query positional.
- Recall / `ledgerful search` (code) argv (`bridge_search_args`).
- T269 nightly/router, T270 retention classify, T272 Safety skip.
- T90 restore, T211 F25 blend, contracts DTO, clap 5, pin bumps.
- `cargo install`, live `.env` rewrite, schtasks mutate, live vault pin.

---

## 7. Verification plan (TDD)

Red first, then green.

1. **Red units (commit allowed):** `ledger_search_argv__json_dash_limit__end_of_options_before_query`; `ledger_search_argv__human_dash_limit__no_json_flag`; `ledger_search_argv__plain_phrase__still_emits_double_dash`; rstest cases for `--days` / `--breaking` / `--json` / `-l` (AC4).
2. **Red clap (commit allowed):** `sync_query__posix_end_of_options__limit_is_query`; `sync_query__bare_limit_flag__still_requires_value`.
3. **Green:** implement `ledger_search_argv` + wire `run_ledger_search` + Query `after_help`.
4. **Stay green:** T271 unit suite; `sync_query__no_bridge__*`; T211/T231 hermetics.
5. **Manual on go:** AC9–AC11 from this repo with `cargo run`.
6. **Docs:** AC12.
7. Targeted: `cargo nextest run -p ai-brains-cli --lib` + the named tests; `cargo clippy -p ai-brains-cli --all-targets -- -D warnings`.
8. Full gate only at implement finalize (not a plan gate).

---

## 8. Risk

| Risk | Mitigation |
|------|------------|
| Ledgerful stops treating `--` as end-of-options | Live-verified today; AC11 control on go. Fail-closed: clap error stays Failed (honest), not ran-empty. |
| `--` as user query | Empty/forward `--` after trim is a one-token needle. Ledgerful would search `--`. Rare. Do not special-case. |
| Growing `sync.rs` | Helper stays in sibling. |
| Quiet hides the current hole | After green, `--quiet -- --limit` must **show** hits (Success), not omit. AC9 without `--quiet` first; optional quiet+hits check on go. |
| Recall code-search same class | Soft residual — do not balloon into retrieval. |

---

## 9. Deferred absorb / decline

| Item | Source | Disposition |
|------|--------|-------------|
| Dash-leading QUERY parsed as Ledgerful flags | #183 Bugbot Medium; T271 closeout; T268 mint | **Absorb** F1–F4 / AC1–AC5 / AC9 |
| Token rescue never starts after clap fail | #183 body | **Absorb** F4 (already T271 F6; argv fix is the remediator) |
| Placeholder F1–F4 | T273 stub | **Absorb** |
| T90 on ledger argv | T90 / T271 F5 | **Affirm decline** F3 |
| T211 F25 blend / double shell | T211 residual | **Decline** F10 |
| T217 vault OR | T217 | **Decline** — not this argv |
| T268 scan-roots | Completed | **Decline** F9 |
| T269 / T270 / T272 | Pending peers | **Decline** F9 — do not steal |
| T240 F2 / T255 bag | standing | **Decline** F9 |
| last-PR Cursor #184 Linux Path units | #184 Bugbot Medium | **Decline** F8 — already `#[cfg(windows)]` at `project_paths.rs:639+`; T268 review P1 fixed; no T274 |
| recall `bridge_search_args` dash query | live `recall.rs:536–538` | **Decline as DoD** F7 — soft residual (same `--` insert on `ledgerful search`) |
| Our Query `allow_hyphen_values` | clap 4.6.6 docs | **Decline** F5 — known `--limit` still wins |
| clap 5 / pin bumps / DTO | standing | **Decline** F10 / F13 |
| MSI / R-CI-BRANCH / archive changeguard sweep | deferred.md open non-overlap | **Decline** — not CLI query argv |
| `anyhow` RUSTSEC-2026-0190 allowlist | deferred.md #5 | **Decline** — not this track |
| Connector list cursor / CE wipe / nil ProjectId | deferred.md historical | **Decline** — not this track |

---

## 10. Implement order (on go)

1. Phase 0 re-verify (HEAD, live hole, Ledgerful `--` still works, pins).
2. Red units + clap AC (commit).
3. Green helper + `run_ledger_search` wire (commit).
4. Query `after_help` + CAPABILITIES + CHANGELOG.
5. Targeted nextest + clippy.
6. Manual AC9–AC11.
7. Review log + residuals → `deferred.md`.
8. Publish per implement-track Phase 6.

---

## 11. Soft residuals

| Residual | Notes |
|----------|-------|
| PATH `ai-brains` until reinstall | F16 — operator `cargo install` |
| `bridge_search_args` (`ledgerful search`) same `--` | F7 — retrieval crate; not this DoD |
| Ledgerful QUERY `allow_hyphen_values` | Other repo. `--` is our remediator |
| Ledgerful token-OR / stop phrase-wrap | T271 F23 |
| T269 / T270 / T272 | Peers |

---

## 12. Touch map

| File | Change |
|------|--------|
| `crates/ai-brains-cli/src/commands/sync_query_ledger.rs` | Add `ledger_search_argv`; `run_ledger_search` uses it; new units |
| `crates/ai-brains-cli/src/main.rs` | Query `after_help`; clap AC7/AC8 |
| `Docs/CAPABILITIES.md` | One additive clause on T271 ledger-pane bullet |
| `CHANGELOG.md` | T273 row under Unreleased |
| `conductor/conductor.md` | T273 Planned (status stays **Pending**) |
| `conductor/deferred.md` | This absorption table |
| `conductor/tracks/README-T256-T271-CLI-AUDIT.md` | T273 Planned note |
| `conductor/tracks/trackT273-*/review.md` | Post-execute only |

Do **not** touch: `sync.rs` dispatcher, `project.rs`, `recall.rs`, `preflight.rs`, `ai-brains-contracts`, Ledgerful repo, `.env`, `.ledgerful/` state.

---

## 13. AI fold-in

(empty until `/fold-in` after plan review)
