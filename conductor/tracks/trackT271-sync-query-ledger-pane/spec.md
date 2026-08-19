# T271 — sync query ledger pane

- **Track ID:** T271-SyncQueryLedgerPane
- **Status:** **Completed** 2026-08-19
- **Category:** BUGFIX / UX
- **Owner:** —
- **Source:** Audit 2026-08-16 — `sync query` **5/5**; friction “ledger pane empty”
- **Depends on:** T90/T91/T95/T115/T124/T211/T231 ✅
- **Blocks / feeds:** Human vault+ledger search is honest. Does **not** unblock T268 scan-roots, T269 nightly split, T270 retention, T272 Safety skip.
- **Absorbs:** Live `--- Ledgerful Ledger Search --- No ledger entries found matching '"capture" "independence"'` from `C:\dev\AI-Brains` while `ledgerful ledger search capture` returns rows; T90 FTS-quoting on a subprocess that **already** phrase-wraps; stub F1 never-ran vs ran-empty copy; stub F2 System32 cwd guard
- **Not absorbed:** Vault ranking (T260/T211); T217 vault OR rescue; T211 F25 vault↔ledger RRF blend; nightly `verification_gate` cwd; Ledgerful crate changes; `sync query --symbols` (T260 F16); clap 5 / new crates
- **Research date:** 2026-08-19 (plan dogfood product HEAD `e48eaa7`; this fold-in `33f72cf`)
- **AI fold-in:** 2026-08-19 `agy-review.md` + `opencode-review.md` (no grok/claude/codex-plan). **B 0 / M 0.** **Agree hard:** OpenCode F18 empty-query unit (AC17); OpenCode F19 classifier units (AC18/AC19) + first-line **140**-char cap. **Agree:** OpenCode core FTS import path (`ai_brains_core::{contentful_tokens, extract_fts_tokens}`); OpenCode capture-count snapshot is volatile (≥1). **Already covered:** Agy m2 `pub mod` (F10 / touch map); Agy O1 two-phase JSON+human (F9/F17). **Note:** Agy m1 / OpenCode HEAD — plan snapshot was `e48eaa7`; working HEAD is the plan commit `33f72cf`. Disposition **§13**.
- **Ledger:** planning DOCS TX `68c42d13-b398-4c36-8d1b-8fc74d3b6516`. Fold-in DOCS TX `5eb051c2-abb3-4ab6-95f5-80bd12167d19`. Implement starts a FEATURE TX on **go**.
- **Isolation:** Do **not** edit vault `sanitize_fts_query` callers (recall/FTS MATCH). Do **not** edit retrieval `preflight.rs` (T272). Do **not** grow hotspot `sync.rs` with the new helpers. Do **not** `cargo install`, pin to the live vault, rewrite `.env`, or mutate schtasks.

---

## 1. Objective

1. **The ledger pane is honest.** From a git worktree that has `.ledgerful`, `sync query "<terms>"` either shows **at least one** ledger hit or a **named** miss (never-ran / failed / phrase-and-tokens empty). `"No ledger entries found matching '"capture" "independence"'"` is **not** allowed — that string is an FTS AND we invented, not the operator query.
2. **Stop FTS-quoting the Ledgerful argv.** Ledgerful `ledger search` already wraps the whole query as one FTS5 **phrase** (`format!("\"{query}\"")` in `C:\dev\Ledgerful\src\ledger\db\search.rs:26`). Passing `sanitize_fts_query` double-wraps and searches for quotes.
3. **Rescue multi-token phrase misses.** When the phrase search of the operator query returns `[]` and there are ≥2 contentful tokens, retry tokens in **first-seen** order (cap 3). First token that hits wins. Banner: `Note: no phrase match for '<user>'; showing hits for '<token>'.`
4. **Vault pane never waits on ledger.** Capture independence: a ledger miss/fail does not block or blank the vault section. `--no-bridge` stays vault-only (T124).
5. **North star.** Operators can find shipped ledger provenance next to vault pins without scraping markdown or guessing that an empty pane means “ledger is empty.”

---

## 2. Live baseline (re-scan 2026-08-19)

### 2.1 Operator dogfood (this machine)

| Signal | Observation |
|--------|-------------|
| HEAD | Plan dogfood `e48eaa7` (T265 PR #182, product `src/` unchanged). Fold-in working HEAD `33f72cf` (this plan commit). Local `main` **1** ahead of `origin/main`. Tree **CLEAN**. |
| PATH `ai-brains` | **0.1.1**, mtime **2026-08-18 20:08**. Pre-T265 (JSON still 2-key). **PATH-behind — do not `cargo install`.** Ledger probe code is the same as HEAD (`sync.rs` untouched by T265). |
| Source `--version` | `ai-brains 0.1.1` (workspace). rustc **1.95.0**. |
| Daily Scope | `3581317d` (`C:\dev\ai-brains`) — T258 rebound. Grants **0 of 3** (T241, not this track). Pin count **volatile** (~3099). |
| cwd | `C:\dev\AI-Brains`. `ledgerful doctor` work root + state dir = this repo `.ledgerful`. **Not System32.** |
| `ledgerful ledger search "capture independence"` | `No ledger entries found matching 'capture independence'.` JSON `[]`. |
| `ledgerful ledger search '"capture" "independence"'` | `No ledger entries found matching '"capture" "independence"'.` JSON `[]`. **Exact audit chrome.** |
| `ledgerful ledger search capture` | **≥1** (volatile: plan-time **5**; fold-in **9**). AC15 is ≥1, not a frozen count. |
| `ledgerful ledger search independence` | **1** hit (T199 daemon vault independence — **wrong topic**). |
| `ledgerful ledger search "capture OR independence"` | Empty (Ledgerful phrase-wraps the whole string, so `OR` is a literal). |
| `ledgerful ledger search T234` | **2** hits. Search works. |
| PATH `ai-brains sync query "capture independence" --quiet` | Vault pane **has hits**. Ledger pane: `No ledger entries found matching '"capture" "independence"'`. **Live hole confirmed.** |
| Last merged PR | [#182](https://github.com/Ryan-AI-Studios/AI-Brains/pull/182) T265. Issue comments / reviews / inline **empty**. No open PR on `main`. |
| T272 leftover | Still true at retrieval `preflight.rs:329` + `:467`. **Not this track.** |

### 2.2 Why the pane is false-empty

```text
AI-Brains probe_ledger_search
  strip_ansi(query)
  sanitize_fts_query → `"capture" "independence"`   // T90 FTS5 AND
  ledgerful ledger search --json <that>

Ledgerful search_ledger (other repo, do not edit)
  fts_query = format!("\"{query}\"")                 // whole query is ONE phrase
  WHERE ledger_fts MATCH ?1
```

| Hypothesis (stub) | Live verdict |
|-------------------|--------------|
| cwd / System32 / wrong `.ledgerful` | **False for the audit repro.** cwd is this repo; `capture` hits the same DB. Keep as a **guard** (F2) for elevation/System32. |
| T95 project-scope on ledger rows | **False.** Probe passes no project id. T95 is vault-only. |
| Wrong binary (`ledgerful search` vs `ledger search`) | **False.** Code already uses `ledger search` (history FTS), not code-symbol `search`. |
| Quoted AND too strict / T217-class | **True, and worse:** Ledgerful is **phrase**, not token-AND. Unquoted `capture independence` is also empty (no entry has those two tokens adjacent). Quoted form adds a second lie (quotes in the needle). |
| Ledger actually empty | **False.** Token `capture` returns 5 committed rows. |

T90 still belongs on **vault FTS MATCH**. It does **not** belong on the Ledgerful argv: Ledgerful already quote-protects hyphens (`dead-code` → column-qualifier footgun, comment at `search.rs:22–25`).

### 2.3 Code truth

| Item | Location | Notes |
|------|----------|-------|
| Pretty `sync query` | `sync.rs` `run_query` `:409` | Always pretty unless `ndjson`. T231 F33. |
| Vault arm | `:487–503` `recall_full` | `no_bridge: true`, `include_symbols: false`, `semantic: false`. **Do not change ranking.** |
| Ledger probe | `probe_ledger_search` `:584–657` | `Command::new("ledgerful")` + `["ledger","search","--json", sanitized]`. Then human re-run. |
| Non-empty detect | `ledger_json_non_empty` `:661` | Pretty JSON array **or** NDJSON object lines. Keep. |
| Sanitize | `ai_brains_core::fts::sanitize_fts_query` | `match_and(extract_fts_tokens)` → `"tok" "tok"`. Vault SoT. |
| Contentful tokens | `ai_brains_core::{contentful_tokens, extract_fts_tokens}` (crate-root re-export; `lib.rs:30–32`) | Rescue import **must not** go through `ai_brains_retrieval` — that crate re-exports **only** `sanitize_fts_query`. CLI already depends on `ai-brains-core`. **First-seen** order (not `select_or_tokens` length-sort). |
| `--no-bridge` | clap `SyncCommands::Query` `main.rs:2730` | T124. Skip probe entirely. Hermetic `sync_query__no_bridge__skips_ledgerful_section`. |
| `--limit` | vault only, default **5** (T211 F27) | Ledgerful `--limit` stays its default **10**. Do not retarget. |
| `--quiet` | T81 | Suppress spawn-fail tracing. Quiet + never-ran → omit pane (today’s `None`). |
| Ledgerful search | `C:\dev\Ledgerful\src\ledger\db\search.rs` | Phrase wrap. `get_layout()` via `gix::discover(cwd)`. Fail-closed if not a git repo. |
| Hotspot | `sync.rs` **#2** (3.646), **786** lines | New helpers + units → sibling `sync_query_ledger.rs`. Dispatcher stays in `sync.rs`. |
| `project.rs` | hotspot **#1** | Do not touch. |
| Contracts / daemon | none for this pane | No DTO. No HTTP. |
| Existing units | `ledger_json_non_empty__*` in `sync.rs` | **Move** with the helper. `resolve_sync_project_id` stays. |

### 2.4 Dependency / standards research (2026-08-19)

| Pin | Workspace / lock | crates.io / docs (snapshot — re-verify at execute) | Action |
|-----|------------------|-----------------------------------------------------|--------|
| clap | workspace **4.5** / lock **4.6.1** | **4.6.6** (docs.rs current) | **No bump.** No new flags. |
| serde_json | lock **1.0.150** | **1.0.151** | **No bump.** Parse probe stdout only. |
| uuid | lock **1.23.1** | — | **No bump.** |
| rustc / edition | **1.95.0** / **2024** | — | Unchanged. |
| nextest | **0.9.140** | — | Unchanged. |
| workspace | **0.1.1** | — | **No bump.** |
| New crates | — | — | **Zero.** Reuse `contentful_tokens`. |
| clap 5 | not released (max 4.6.x) | — | Forbidden. |

**How to implement (primary sources):**

| Source | What it locks |
|--------|----------------|
| [SQLite FTS5 §3.2 Phrases](https://www.sqlite.org/fts5.html#fts5_phrases) | A quoted string is **one phrase** (ordered adjacent tokens). `"capture independence"` matches only documents that contain those tokens **in order, adjacent**. That is why the unsanitized two-word query is empty. |
| [SQLite FTS5 §3.7](https://www.sqlite.org/fts5.html#fts5_boolean_operators) | `OR` / implicit AND exist — **but** Ledgerful wraps the *entire* argv as one quoted phrase, so we **cannot** send `capture OR independence`. Live confirmed: that query is empty. |
| Ledgerful `search.rs:22–26` | Hyphen protection is **their** wrap. Our T90 wrap is redundant and harmful on this argv. |
| T217 rescue (this repo, `lexical.rs`) | Sequential fallback after empty AND. **Pattern to copy** on the CLI side only. Do **not** change vault MATCH. |
| [clig.dev — errors / honesty](https://clig.dev/#errors) | Tell the user what happened. Silent pane-omit on spawn fail looks like `--no-bridge`. Ran-empty must quote the **user** string. |
| N/A | No new CLI framework, no clap ValueEnum, no contracts schema. |

Could not verify: Ledgerful crates.io (it is a sibling local product, not a registry pin). Verified against **this machine’s** `C:\dev\Ledgerful` source + live `ledgerful ledger search --help`.

---

## 3. Frozen decisions

| ID | Decision |
|----|----------|
| **F0 — Go gate** | Plan-only until user **go**. Planning is DOCS. Implement starts a FEATURE TX. |
| **F1 — Miss classes** | Three named outcomes besides hits: **never-ran** (CLI missing / spawn fail / System32 cwd / not-a-git-repo), **failed** (non-zero after spawn), **ran-empty** (success + `[]` after phrase **and** token rescue). Never print T90-quoted needles. |
| **F2 — System32 / bad cwd** | If `current_dir` is `C:\Windows\System32` or `C:\Windows\SysWOW64` (case-insensitive), **do not spawn**. Copy: `Ledger search did not run: cwd is a Windows system directory (not a git worktree). cd to the repo.` Do **not** invent a `{cwd}\.ledgerful`. Non-Windows: skip this arm. |
| **F3 — `--no-bridge`** | Unchanged (T124). No ledger section. Existing hermetic stays green. |
| **F4 — Capture independence** | Vault `recall_full` always runs first in the non-reorder path (T211 F12 reorder still allowed when ledger is non-empty **and** vault top is plan-class). Ledger miss/fail **never** blanks vault. No events. No models. |
| **F5 — Do not FTS-sanitize the argv** | Probe/human/rescue pass `strip_ansi(query).trim()` (T91). **Never** `sanitize_fts_query` on `ledgerful ledger search`. Vault MATCH keeps T90. |
| **F6 — Token rescue** | After a **successful** phrase probe of the user query returns empty: take `contentful_tokens(extract_fts_tokens(user))` via `use ai_brains_core::{contentful_tokens, extract_fts_tokens}` (not `ai_brains_retrieval`). If `len >= 2`, retry each token (max **3**, **first-seen** order) via `--json`. First `ledger_json_non_empty` wins → human re-run **that token** + banner. If all empty → ran-empty with the **user** query. Length-sort (`select_or_tokens`) is **declined**: it would pick `independence` first and surface T199. |
| **F7 — Banner** | Only when rescue fires and hits: `Note: no phrase match for '<user>'; showing hits for '<token>'.` User/token are the stripped strings (no added FTS quotes). |
| **F8 — Quiet** | `--quiet` + never-ran/failed → omit ledger section (today’s `None` + T81). `--quiet` + ran-empty or hits → still print the pane (content, not a warning). |
| **F9 — T211 F12 lift (display only)** | Keep `--json` probe + `ledger_json_non_empty` + ledger-first reorder when `non_empty`. Change: missing/fail print a miss line unless `--quiet`; empty phrase may rescue. Do **not** change ranking, vault limit, or plan-class detection. |
| **F10 — Module** | New `crates/ai-brains-cli/src/commands/sync_query_ledger.rs`: query forwarder, miss copy, System32 predicate, rescue picker, `probe_ledger_search`, `ledger_json_non_empty`, their units. `sync.rs` dispatches + prints. `pub mod` in `commands/mod.rs` (siblings are `pub mod`). **Move** existing `ledger_json_non_empty` tests. Do **not** move `resolve_sync_project_id`. |
| **F11 — Decline extras** | No Ledgerful source edits; no `--days`; no vault `--limit` on ledger; no merged multi-token table; no `OR` argv; no `sync query --symbols`; no T211 F25 blend; no json-v2 / contracts DTO; no clap 5; no `schema_version` on this pane. |
| **F12 — Decline peers** | T268 / T269 / T270 / T272 / T240 F2 / T255 bag / T264 leftover recall drop. |
| **F13 — Pins / crates** | No lock bumps. No new crates. Workspace **0.1.1**. |
| **F14 — Docs** | CAPABILITIES: one honesty bullet (Ledgerful phrase search + token rescue + miss classes). Root CHANGELOG T271 row. Optional one-line OPERATIONS. No PROTOCOL-COMPAT (no DTO). |
| **F15 — Tests** | Naming `function_or_feature__condition__expected_result`. Units first (red) including AC17 empty-query + AC18/AC19 classifier. No `unwrap`/`expect`/`panic` in production. No PATH-hijack hermetic required (Windows-flaky); banner/forwarder/miss/classifier are pure. Existing T124 / T211 / T231 hermetics stay green. |
| **F16 — PATH-behind** | Do not `cargo install` unless the user asks. Manual AC uses `cargo run` / hermetic bin. |
| **F17 — Human re-run** | Hits (phrase or rescued token): still re-run without `--json` for the table (T211). On phrase-empty before rescue, do **not** print Ledgerful’s empty line as a first pane. |
| **F18 — Empty user query** | After strip+trim, if empty → never-ran/failed class: `Ledger search did not run: query is empty.` Do **not** apply `is_contentless_query` (stopwords) to the ledger argv — that is a vault T261 gate. |
| **F19 — Non-git stderr** | Pure classifier (no subprocess): spawn-err → **never-ran**; success+empty after rescue → **ran-empty**; nonzero + stderr (case-insensitive) contains `git` / `work directory` / `layout` → **never-ran**; other nonzero → **failed**. Surface the **first stderr line**, then cap at **140** chars (T250 `PRETTY_LINE_MAX` / T263 one-line). Empty stderr on nonzero → `Ledger search failed.` Do **not** import `project.rs::truncate_chars` (hotspot #1) — local cap in the sibling. |
| **F20 — Review** | BUGFIX / UX. Primary review required. Cross-model **optional** (no contracts DTO, no architecture). |
| **F21 — Debt file** | `conductor/ISSUES.md` does **not** exist. Residuals → `conductor/deferred.md`. |
| **F22 — Subprocess cap** | Worst case: 1 phrase JSON + 3 token JSON + 1 human = **5**. Phrase hit stays **2** (today). No sleep-for-async. |
| **F23 — Do not touch Ledgerful** | Phrase wrap stays their hyphen defense. If we need true OR, that is a **Ledgerful** track later (soft residual). |

---

## 4. Acceptance criteria

| AC | Proof |
|----|-------|
| **AC1** | Unit: `ledger_forward_query("capture independence")` == `capture independence` (no added `"`). Also `ledger_forward_query("")` / whitespace / `"\\n"` → `""`. |
| **AC2** | Unit: ANSI-colored `"capture independence"` → same as AC1 after `strip_ansi` (T91). |
| **AC3** | Unit: `sanitize_fts_query` is **not** in the forwarder (assert forwarded ≠ `"capture" "independence"`). |
| **AC4** | Unit: `ledger_rescue_tokens("capture independence")` == `["capture", "independence"]` (first-seen). **Not** `["independence", "capture"]`. |
| **AC5** | Unit: rescue picker with phrase-empty + first token JSON `[]` + second token JSON `[{…}]` selects the **second** token. |
| **AC6** | Unit: ran-empty copy contains the **user** query and does **not** contain `'"capture" "independence"'`. |
| **AC7** | Unit: System32 / SysWOW64 paths → never-ran, **no** argv built. `C:\dev\AI-Brains` → not that class. |
| **AC8** | Unit: missing-CLI / spawn-err → never-ran string contains `did not run` (or locked equivalent), not `No ledger entries found matching`. |
| **AC9** | Unit: banner formatter matches F7 exactly for user=`capture independence`, token=`capture`. |
| **AC10** | Existing `ledger_json_non_empty` array/empty/ndjson/blank stay green after the move. |
| **AC11** | Hermetic `sync_query__no_bridge__skips_ledgerful_section` still: has `AI-Brains Recall`, **no** `Ledgerful Ledger Search`. |
| **AC12** | Existing T211 ranking + T231 resolve hermetics stay green (`--no-bridge` path). |
| **AC13** | Manual (on go): `cargo run -p ai-brains-cli -- sync query "capture independence"` from this repo → ledger pane has **≥1** table row **or** F7 banner + hits for `capture`. Must **not** print `'"capture" "independence"'`. |
| **AC14** | Manual: `sync query "capture independence" --no-bridge` → no ledger section. |
| **AC15** | Manual: `ledgerful ledger search capture` still ≥1 (control — we did not break Ledgerful). |
| **AC16** | Docs: CAPABILITIES + CHANGELOG mention phrase + rescue + miss classes. |
| **AC17** | Unit: `ledger_miss_copy__empty_query__did_not_run` — empty forward → never-ran copy contains `query is empty` and `did not run`; not `No ledger entries found matching`. |
| **AC18** | Unit: `ledger_classify_outcome__nonzero_git_stderr__never_ran` — fixture stderr `Failed to find work directory for repository` → **never-ran** (no subprocess). |
| **AC19** | Unit: `ledger_classify_outcome__nonzero_other_stderr__failed` — fixture stderr `fts5: syntax error near "."` → **failed**; first line capped at 140 chars. |

---

## 5. Design notes

### 5.1 Forwarder

```text
fn ledger_forward_query(raw: &str) -> String {
    ai_brains_retrieval::strip_ansi(raw).trim().to_string()
}
```

No FTS quote. No stopword strip. Trim only.

### 5.2 Probe flow

```text
if no_bridge → None
if System32/SysWOW64 → Some(NeverRan) unless quiet
if forward.trim().is_empty() → Some(NeverRan empty-query) unless quiet
phrase = ledgerful ledger search --json <forward>
spawn err → NeverRan (unless quiet)
nonzero → Failed / NeverRan-from-stderr (unless quiet)
if ledger_json_non_empty(phrase) → human(forward) → Hits { rescued: None }
else
  for token in contentful_tokens(forward).take(3)
    if ledger_json_non_empty(json(token)) → human(token) → Hits { rescued: Some(token) }
  RanEmpty { user: forward }
```

Print path: Hits → optional F7 banner + human table. Miss → one line (F1). Quiet + NeverRan/Failed → skip section.

### 5.3 Ledger-first (T211)

`non_empty` is true for phrase **or** rescued hits. Reorder rules unchanged.

### 5.4 Why first-seen tokens

`independence` is longer than `capture`. `select_or_tokens` would rescue T199 (daemon vault independence) first — off-topic for the product constraint. First-seen keeps the operator’s lead word.

Import (OpenCode O, verified): `use ai_brains_core::{contentful_tokens, extract_fts_tokens};` — `ai_brains_retrieval` re-exports only `sanitize_fts_query` (`fts_utils.rs`).

---

## 6. Non-goals

- Editing `C:\dev\Ledgerful` (true token-OR / no phrase wrap).
- Vault T217 OR MATCH, T260 ranking, T261 contentless, T272 Safety skip.
- `sync query --semantic` / `--symbols`.
- Passing vault `--limit` into ledger search.
- Merging multiple token tables into one.
- Machine JSON object for `sync query` (T231 always-pretty stands).
- Silent Scope / `.env` rewrite (T240 F2).
- `cargo install` / live nightly mutate / policy bootstrap.

---

## 7. Verification plan (TDD)

Red first (named tests in `sync_query_ledger.rs`):

1. `ledger_forward_query__user_phrase__not_fts_quoted` (AC1/AC3)
2. `ledger_forward_query__empty__returns_empty` (AC1)
3. `ledger_forward_query__ansi_stripped` (AC2)
4. `ledger_rescue_tokens__capture_independence__first_seen_capture` (AC4)
5. `ledger_rescue_pick__first_token_empty_second_hits__selects_second` (AC5)
6. `ledger_miss_copy__ran_empty__uses_user_query_not_quotes` (AC6)
7. `is_windows_system_cwd__system32_and_syswow64__true` (AC7)
8. `ledger_miss_copy__never_ran__did_not_run` (AC8)
9. `ledger_miss_copy__empty_query__did_not_run` (AC17)
10. `ledger_classify_outcome__nonzero_git_stderr__never_ran` (AC18)
11. `ledger_classify_outcome__nonzero_other_stderr__failed` (AC19)
12. `ledger_rescue_banner__phrase_empty_token_hit__locked_sentence` (AC9)

Green: implement forwarder + miss copy + probe in the sibling; `sync.rs` calls it.

Stay green: AC10–AC12 hermetics/units.

Manual AC13–AC15 on go. No full workspace nextest as a plan gate.

---

## 8. Risk

| Risk | Mitigation |
|------|------------|
| Extra `ledgerful` processes on multi-token empty | Cap 3 tokens; phrase hit stays 2 procs. |
| Rescue shows a weakly related token | First-seen + banner honesty. Soft: later scoring residual. |
| `sync.rs` hotspot grows | Sibling module (F10). |
| T90 tests assume ledger argv is sanitized | Those tests are vault/core; grep before green. If a CLI test snapshots `'"capture" "independence"'`, update it — that snapshot **is** the bug. |
| Non-git cwd (tempdir hermetics) | Default hermetics use `--no-bridge` or already tolerate missing ledger. Probe fail → miss line unless quiet. Do not fail the command. |
| PATH-behind after ship | F16. Operator `cargo install`. |

---

## 9. Deferred absorb / decline

Entire `conductor/deferred.md` scanned 2026-08-19.

| Item | Disposition |
|------|-------------|
| Audit ledger pane false-empty (5/5) | **Absorb** F1–F7 / AC1–AC9 / AC13 |
| Stub F1 never-ran vs ran-empty | **Absorb** F1 / F8 / AC6 / AC8 |
| Stub F2 System32 | **Absorb** F2 / AC7 (guard; **not** the live repro) |
| Stub F3 `--no-bridge` | **Affirm** F3 / AC11 / AC14 |
| Stub F4 vault independence | **Affirm** F4 |
| T90 sanitize on ledger argv | **Absorb as lift** — T90 stays on vault MATCH; **remove** from probe |
| T91 strip ANSI | **Affirm** F5 / AC2 |
| T95 project isolation | **Decline** — vault-only; ledger is repo-scoped via cwd/`gix` |
| T211 F12 empty → vault-only | **Partial** F9 — reorder + `--json` stay; miss/rescue display changes |
| T211 F25 full blend / double shell | **Decline** F11 — still soft residual |
| T217 vault OR rescue | **Decline** — pattern only; do not change MATCH |
| T231 always-pretty / F32 resolve | **Affirm** — do not flip |
| T260 `--symbols` / leftover `--global` | **Decline** F12 |
| T214 F9 ledgerful-on-global | **Decline** — different surface |
| T265 `sections[]` / T266 format / T267 footer | **Decline** — Completed peers |
| T268 / T269 / T270 / T272 | **Decline** F12 — stay Pending |
| T240 F2 / T255 bag | **Decline** F12 |
| last-PR Cursor #182 | **N/A** — comments/reviews empty. #179 Bugbot still **T272** |
| R-CI-BRANCH / MSI / packaging | **Not related** — admin/packaging |
| Closed strikethrough rows (T187–T267 series) | Stay closed |

---

## 10. Implement order (on go)

1. Phase 0 re-verify (live hole still quoted-empty; Ledgerful still phrase-wraps; pins).
2. Red units AC1–AC9 + AC17–AC19 in the sibling (may `mod` first with `#[cfg(test)]` stubs).
3. Green: forwarder, miss copy, System32, rescue, move probe + `ledger_json_non_empty`.
4. Wire `sync.rs` print/reorder. Keep T124/T211/T231 green.
5. Docs F14. Targeted nextest `-p ai-brains-cli`.
6. Manual AC13–AC15. Review log. FEATURE TX commit. **No push to main.**

---

## 11. Soft residuals

| Residual | Why not DoD |
|----------|-------------|
| Ledgerful token-OR / stop phrase-wrapping user spaces | Other repo |
| Merge all token hits into one table | Extra parser for comfy-table |
| Pass `--limit` through | Vault 5 vs ledger 10 is T211; leave |
| Single subprocess (pretty-print JSON probe) | T211 F12 double shell residual |
| PATH `cargo install` | Operator / F16 |
| Rescue scoring / pick “best” token | Banner is enough |
| `is_contentless_query` on ledger | T261 vault gate; F18 |

---

## 12. Touch map

| Path | Change |
|------|--------|
| `crates/ai-brains-cli/src/commands/sync_query_ledger.rs` | **New.** Probe + forward + rescue + miss + units. |
| `crates/ai-brains-cli/src/commands/mod.rs` | `pub mod sync_query_ledger;` |
| `crates/ai-brains-cli/src/commands/sync.rs` | Delete moved probe/json helpers; call sibling; **do not** grow ranking/resolve. |
| `Docs/CAPABILITIES.md` | Honesty bullet. |
| `Docs/CHANGELOG.md` | T271 row. |
| `conductor/conductor.md` | T271 Planned (status **Pending**). |
| `conductor/deferred.md` | This absorption table. |
| `conductor/tracks/README-T256-T271-CLI-AUDIT.md` | T271 Planned. |

**Do not touch:** retrieval `preflight.rs`, `fts.rs` sanitizer body, `project.rs`, Ledgerful sources, contracts, daemon, `.env`, schtasks.

---

## 13. AI fold-in disposition (2026-08-19)

Sources: `agy-review.md` + `opencode-review.md`. No Blockers / Majors. Both verdicts **Planned**. Fold-in working HEAD `33f72cf`. T272 still `:329` + `:467`. `ledger search capture` fold-in count **9** (volatile).

### Agy

| ID | Verdict | Action |
|----|---------|--------|
| **m1** HEAD `e48eaa7` vs `33f72cf` | **Agree (note)** | §2.1 records both snapshots |
| **m2** `pub mod sync_query_ledger` | **Already covered** | F10 + §12 touch map already require `commands/mod.rs` `pub mod` |
| **O1** JSON then human | **Already covered** | F9 / F17 two-phase probe |

### OpenCode

| ID | Verdict | Action |
|----|---------|--------|
| **m** F18 empty-query unit | **Agree hard** | AC17 + `ledger_miss_copy__empty_query__did_not_run`; AC1 empty-forward |
| **m** F19 classifier untested | **Agree hard** | AC18 / AC19 pure classifier units; F19 names the mapping |
| **m** plan HEAD stale | **Agree (note)** | Same as Agy m1 |
| **O** capture count 5→≥9 | **Agree** | §2.1 / plan preflight: ≥1 volatile; AC15 already ≥1 |
| **O** import `ai_brains_core` | **Agree** | F6 / §2.3 / §5.4 |
| **O** F19 truncate unbound | **Agree** | F19 first line then **140** chars; no `project.rs` import |

### Pins locked by fold-in

1. **AC17 / F18:** empty forward is a required never-ran unit, not prose-only.
2. **AC18 / AC19 / F19:** classifier is a pure helper; git/work-directory/layout stderr → never-ran; other nonzero → failed; first line, 140-char cap.
3. **F6:** `use ai_brains_core::{contentful_tokens, extract_fts_tokens}`.
4. **AC15:** control is ≥1, not a frozen hit count.
5. **F10:** `pub mod` stays (Agy m2 already there).

**Planning + fold-in 2026-08-19.** Still **plan-only until go**.
