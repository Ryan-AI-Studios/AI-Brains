# T279 — Preflight Safety must not be a captured review-track prompt

- **Track ID:** T279-PreflightSafetyVsHotspots
- **Status:** **Planned** (Pending until **go**; F0)
- **Category:** UX / HONESTY
- **Owner:** Grok
- **Source:** Live CLI audit 2026-08-21 — friction: `preflight --pretty` “Repository Bearings & Safety” was a T272 review-track Objective dump; `safety sync --dry-run` listed `project.rs` etc. Placeholder minted with T274–T284.
- **Depends on:** T264 ✅ global isolation; T272 ✅ skip emitted ids; T219 ✅ pretty readability; T274 ✅ Index two-pass (Safety SQL left here)
- **Blocks / feeds:** Operators can trust Safety as bearings + live hotspots, not session chrome. Index ranking stays T274. Policy `--scope` **T280**. Nightly dual-probe **T281**. `context --show` leftover **T282**. `project list` cwd-first **T283**.
- **Absorbs:** Placeholder problem text + Manual DoD; deferred.md “preflight Safety = review-track Objective”; T274 F23 leftover (Safety SQL); T274 closeout “AC6 dump buried CONSTRAINT would Safety-steal”; T250 F12 HOTSPOT float **partial** (live line `score={:.2}` only)
- **Not absorbed (DoD):** T272 skip-set / T264 caps / `GLOBAL_SAFETY_*`; T274 Index/rank; `query_ledgerful` Intelligence rewrite; live `safety sync` without `--dry-run`; T280–T283; T240 F2; leftover `7d97a456` rebind; clap 5; rusqlite 0.40; DTO keys; doctor 16th check
- **Research date:** 2026-08-22 (plan dogfood HEAD `631a8f8` T278 `#194`; product `src/` = T278)
- **AI fold-in:** none yet (plan pass). Disposition after review-track → **§13**.
- **Ledger:** planning DOCS TX `4d4dd4b0-1884-4bfc-a0dd-8543aa5de1a5`. Implement starts a **FEATURE** TX on **go**.
- **Isolation:** Do **not** `cargo install`. Do **not** write live `.env` (T240 F2). Do **not** pin the live vault as implement proof. Do **not** run `safety sync` without `--dry-run`. Do **not** live-bootstrap grants (T275). Do **not** leftover-rebind (T276). Do **not** grow hotspot `project.rs` / CLI `preflight.rs` / `sync.rs` / `doctor.rs`. Do **not** print or commit `AI_BRAINS_KEY`. Do **not** enable `AI_BRAINS_GOVERNED_BRIEFING`.

---

## 1. Objective

1. **Safety is not session chrome.** `preflight --pretty` “Repository Bearings & Safety” must not open with `## Objective` / `review-track` skill text. Today `LIKE '%CONSTRAINT:%'` matches buried markers inside harness dumps; the displayed first line is the dump heading.
2. **Safety identity matches `safety sync --dry-run`.** Project-scoped Safety prepends live Ledgerful hotspot paths (same `ledgerful hotspots --json --limit 5` argv as CLI `safety.rs`), rendered as `HOTSPOT: <path> score=<n>`. `--global` does **not** live-inject (T214 F9 analog). Fail-open if Ledgerful is missing/empty.
3. **Honest empty beats a stolen body.** When live inject is empty/skipped **and** no leading-line CONSTRAINT/INVARIANT/HOTSPOT vault pins remain, still emit the Safety header plus `No in-context hotspots. next: ai-brains safety sync --dry-run`. Do **not** omit the section. `--summary` stays the T220 banner (no Safety header).
4. **North star.** Capture independence: SQL GLOB + optional `ledgerful` shell already used by `query_ledgerful`. No models. No new events. T180 `{text, word_count}` + T265 `sections[]` keys frozen. Append-only log unchanged.

This unblocks the daily product: T274 ranked Index; T272 fixed skip-set. The remaining usefulness hole is **section identity** — Safety must be bearings + hotspots, not a captured plan-review prompt.

---

## 2. Live baseline (re-scan 2026-08-22)

### 2.1 Operator dogfood (this machine)

| Signal | Observation |
|--------|-------------|
| HEAD | `631a8f8` T278 squash `#194`. Tree **CLEAN**. `origin/main...HEAD` **00**. Product `src/` includes T274–T278. |
| PATH `ai-brains` | `C:\Users\RyanB\.cargo\bin\ai-brains.exe` mtime **2026-08-21 05:55**, 25 368 576 bytes, **0.1.1**. **T270** on PATH (before T274–T278). Safety SQL is unchanged since T274 (F23 left it here) — **PATH is valid for this hole.** **Do not `cargo install`.** Tests/manual AC use `cargo run` / hermetic. |
| `preflight --summary` | Scope `C:\dev\ai-brains` (`3581317d`). Pinned **3516**. In-context **0/0/0**. Grants **0 of 3** (T275 hermetic; live not bootstrapped). Capture independence holds. |
| `project whoami` | Not this track. Shell leftover `7d97a456` is T282 / T258; leftover volume T276. |
| `safety sync --dry-run` | **5** hotspots: `project.rs` (score 0.05), `sync.rs`, `forget.rs`, `context.rs`, `governed_common.rs`. Matches `ledgerful hotspots` table. |
| `preflight --pretty --compact -m 400` | Safety body is **`## Objective` only.** Then Sessions. **No** Ledgerful Intelligence header. **No** dry-run paths. |
| `preflight --pretty -m 800` | Safety is the full T272 `review-track` Objective dump (skill text; buried CONSTRAINT likely). Sessions follow. Index is still chrome (`# Track Plan Review`, ` ```json`) — PATH-behind T274; **not this DoD.** Intelligence still **absent** (`query_ledgerful` returned `None`). |
| Last GitHub PR | [#194](https://github.com/Ryan-AI-Studios/AI-Brains/pull/194) T278 (2026-08-22). `gh pr view --comments`, `/reviews`, `/comments`, `issues/194/comments` all **empty**. **last-PR Cursor: N/A.** Open PRs: Dependabot remotes only (`#61` rusqlite 0.40.2, `#62` chrono 0.4.45, actions). **No leftover to mint. No T285.** |
| Prior #188 Bugbot | **T284 Completed** `#193`. Not this track. |
| Identity / doctor | ledgerful doctor 4 warn (legacy `.changeguard` / sig-pin / timings / :8081). **0 pending / 0 drift.** Hotspot **#1** `project.rs` (**3.944**). CLI `preflight.rs` **#7** (2.257, **2027** lines). Retrieval `preflight.rs` **1087**. `project.rs` **1332**. `doctor.rs` / `sync.rs` / `main.rs` — **do not grow.** |
| `ISSUES.md` | **Does not exist.** |

### 2.2 Why this still matters

| Residual | Why it is still a product hole / why decline |
|----------|----------------------------------------------|
| LIKE-anywhere Safety SQL | `preflight.rs` **`:294/:301`**: `LIKE '%CONSTRAINT:%' OR '%INVARIANT:%' OR '%HOTSPOT:%'`. T274 `classify_pin_kind` is **leading-line**; Safety is still substring. Review-track bodies that mention CONSTRAINT match; first line displayed is `## Objective`. Audit friction. **DoD.** |
| Safety ≠ dry-run paths | `safety sync --dry-run` shells `ledgerful hotspots --json --limit 5` (`safety.rs` `:102–128`). Preflight never does. Vault HOTSPOT pins are rare (live 0) and suppressed when `query_ledgerful` succeeds (`:326`). Intelligence section is a **different** source (`bridge export --hotspots`) and was **empty** this dogfood. **DoD: inject the dry-run argv into Safety.** |
| Stolen section omitted vs empty | If we only GLOB, this vault’s Safety would go empty (0 leading CONSTRAINT in the window) and **omit** the header (`:353` `if !safety_entries.is_empty()`). Operators then see Sessions first. CLIG: tell the user the next command. **DoD: honest empty.** |
| T274 Index two-pass | Already shipped `#189`. PATH-behind until install. **Do not retune Index here.** Buried CONSTRAINT dumps that drop out of Safety may appear in Index (T274 wanted that; AC6 deferred CONSTRAINT because it would steal Safety — **absorb here**). |
| `query_ledgerful` Intelligence | Separate `---` section, skipped under `--global` (T214 F9 / T264). Live empty. **Decline rewrite / merge.** Duplicate paths if Intelligence later fills is a **soft residual.** |
| Live `safety sync` (pin) | Mutates the vault (pins a HOTSPOT blob). **Stop-Before.** Dry-run + live inject are enough. |
| T272 skip-set | Post-cap emitted **vault** ids. Live lines have no `memory_id`. **Do not retune HashSet.** |
| T264 caps | `GLOBAL_SAFETY_FETCH=40` / per-project 2 / max 8. **Freeze.** |

### 2.3 Code truth

| Item | Location | Notes |
|------|----------|--------|
| Safety SQL | retrieval `preflight.rs` **`:290–305`** | LIKE anywhere LIMIT 40 global / 10 project. **Replace with leading-line GLOB** (`safety_marker_glob_sql`). |
| HOTSPOT suppress | **`:325–327`** | `continue` before push when `has_cg_intelligence && content.contains("HOTSPOT:")`. **Keep** for vault rows. Also suppress vault HOTSPOT when live inject is non-empty (F7). |
| Skip set | **`:345–349`** | T272: `safety_ids` from **emitted** vault entries. Live lines **must not** enter this set. |
| Safety emit | **`:351–376`** | Omits section when empty. **Always emit** header + body or F3 empty line. |
| Intelligence | `query_ledgerful` **`:695`** | `ledgerful bridge export --hotspots`. **Do not edit** as DoD. |
| Dry-run fetch | CLI `safety.rs` `fetch_hotspots_json` **`:102–128`** | `ledgerful hotspots --json --limit N`; JSON starts at line with `[`. **Copy pattern into retrieval sibling** — do **not** depend CLI→retrieval. |
| clap Safety | `main.rs` `SafetyCommands::Sync` **`:2947–2956`** | `--limit` default **5**; `--dry-run`. **No new flags on preflight.** |
| Summary counts | CLI `preflight.rs` **`:886–888`** | `text.matches("HOTSPOT:")` on assembled window. **Do not grow this file.** Live lines **must** contain `HOTSPOT:`. |
| Compact Safety | `main.rs` preflight `after_help` **`:1080`** | Default pretty does **not** line-cap Safety; `--compact` first-line-caps Safety **100**. Live `HOTSPOT: path score=n` must be the first line of each item. |
| Index GLOB | `session_chrome.rs` `index_marker_glob_sql` **`:73`** | Includes **DECISION:** — **must not** reuse for Safety. New `safety_marker_glob_sql` = CONSTRAINT/INVARIANT/HOTSPOT + `ASSISTANT: ` variants. |
| T272 hermetic | `tests/preflight_global_isolation.rs` AC2/AC3 | Leading `CONSTRAINT: A-two` etc. **Stay green** (GLOB matches leading). |
| T219/T250 pretty | `tests/preflight_pretty_readability.rs` | Summary must **not** print Bearings (`:292/:553`) — retrieval always-emit is pretty-path only; summary formatter stays T220. |
| T180 / T265 | `PreflightContextResponse` | `{text, word_count}` + additive `sections[]`. **No new keys.** |
| Hermetic helper | `tests/common/mod.rs` `AMBIENT_DENYLIST` **`:42`** | Must **set** skip-env after strip so host `ledgerful hotspots` cannot pollute integration tests (cwd is this repo). |
| Hotspots | `project.rs` #1 3.944; CLI `preflight.rs` #7 | **Do not touch.** Helpers in retrieval `preflight_safety.rs` + `session_chrome.rs`. |

### 2.4 Dependency / standards research (2026-08-22) — snapshot; re-verify at execute

| Pin | Workspace / lock | Ecosystem (today) | Action |
|-----|------------------|-------------------|--------|
| `clap` | workspace **4.5** / lock **4.6.1** | crates.io **4.6.6** (2026-08-06). GitHub latest tag **v4.6.6**. **No clap 5** (4.x is current track). | **No bump.** No new preflight flags. Additive `after_help` only. |
| `serde_json` | lock **1.0.150** | crates.io **1.0.151** | **No bump.** JSON parse of hotspot array is local. T180 keys frozen. |
| `chrono` | lock **0.4.44** | crates.io **0.4.45** (Dependabot #62 open) | **No bump.** |
| `rusqlite` | lock **0.39.0** + sqlcipher + backup | crates.io **0.40.2** (Dependabot #61; T213 L4 `table_exists`) | **No bump.** GLOB is SQL, not a 0.40 API. |
| `uuid` | lock **1.23.1** | crates.io **1.25.0** | **No bump.** |
| rustc / edition | **1.95.0** / **2024** | workspace toolchain | Unchanged |
| nextest | **0.9.140** | — | Unchanged |
| workspace | **0.1.1** | — | **No bump** |
| New crates | — | wait-timeout / regex | **Zero.** No Command timeout crate (F35 fail-open on spawn). No `regex` in retrieval (T211 F18). |

**Online / primary sources**

| Claim | Source | Takeaway |
|-------|--------|----------|
| LIKE `%…%` matches anywhere; GLOB is case-sensitive Unix glob | [SQLite expr](https://www.sqlite.org/lang_expr.html) §5 (current) | `CONSTRAINT:*` / `ASSISTANT: CONSTRAINT:*` is the T274 F8 subset. LIKE-anywhere is the dump steal. |
| GLOB prefix can use an index (no leading wildcard) | [SQLite optoverview](https://www.sqlite.org/optoverview.html) §5 LIKE optimization | Safety GLOB `CONSTRAINT:*` is prefix — better than `%CONSTRAINT:%`. Not a reason to add indexes this track. |
| Human-first CLI; say what to do next | [clig.dev](https://clig.dev/) Human-first + “If you change state, tell the user” (empty-state analog: don’t go silent) | Honest empty + `next: ai-brains safety sync --dry-run`. Do not omit Safety. |
| clap 4 current | crates.io clap **4.6.6**; GitHub `clap-rs/clap` latest **v4.6.6** | No new args. clap 5 not the current track. |
| rusqlite 0.40.2 | GitHub `rusqlite/rusqlite` latest **0.40.2** (2026-08-08) | Standing freeze; T213 L4. **No T285.** |

**N/A:** SQLCipher page encrypt, schtasks, T180 DTO new keys, Windows service, llama.cpp `/health`, graph floors.

**Could not verify:** whether `query_ledgerful` Intelligence is empty because `bridge export --hotspots` is opt-in (`LEDGERFUL_BRIDGE`) or because this tree has no `hotspot_delta` records. Live `ledgerful hotspots` works without the bridge. **Do not** debug/enable the bridge as DoD.

**ledgerful / ai-brains:** `preflight --summary` 0/0/0 vs **3516** pins; `ledgerful ledger status --compact` 0 pending / 0 drift; `search "safety_sql"` → `:290/:307`; `search "query_ledgerful"` → `:695` + recall bridge. Semantic recall of “Safety hotspots” returned T272 review-track dumps (the hole).

---

## 3. Frozen decisions

| ID | Decision |
|----|----------|
| **F0 — Go gate** | Plan-only until user **go**. Planning is DOCS TX `4d4dd4b0`. Implement starts a **FEATURE** TX. |
| **F1 — Leading-line Safety GLOB** | Replace LIKE-anywhere with `safety_marker_glob_sql("m.content")`: `CONSTRAINT:` / `INVARIANT:` / `HOTSPOT:` at start, plus `ASSISTANT: ` variants. Case-sensitive SQL (T274 F8). Buried `CONSTRAINT:` in `## Objective` dumps **must not** match. |
| **F2 — Live inject (project-scoped)** | When `!global` and skip-env is off, prepend up to **5** lines from `ledgerful hotspots --json --limit 5` (same argv as `safety.rs`). Render `HOTSPOT: {path} score={score:.2}` (one line per path; path first so `--compact` 100 still shows the file). Fail-open (F35). |
| **F3 — Honest empty** | If live inject empty/skipped **and** vault GLOB empty: still push `--- Repository Bearings & Safety ---\n` + `SAFETY_EMPTY` (`No in-context hotspots. next: ai-brains safety sync --dry-run`). **Do not** put `HOTSPOT:` in that string. `--summary` does **not** print the header (T219 AC8/AC13 stay). |
| **F4 — `--global` no live inject** | T214 F9 analog. Global Safety = vault GLOB + T264 caps/tags only. Cwd Ledgerful is one repo; do not pretend it is a multi-project rollup. |
| **F5 — T272 skip stands** | `safety_ids` = emitted **vault** memory ids only. Live lines have no id and **must not** be inserted into `safety_raw`. |
| **F6 — T264 caps freeze** | `GLOBAL_SAFETY_FETCH=40`, per-project 2, max 8, LIMIT 10 project. Do not retune. Round-robin still applies to vault entries only. |
| **F7 — Vault HOTSPOT suppress** | Keep `:326` Intelligence suppress. Also skip vault HOTSPOT rows when live inject returned ≥1 path (avoid dup with F2). CONSTRAINT/INVARIANT vault rows still emit (bearings). |
| **F8 — Bearings after live hotspots** | Order: live `HOTSPOT:` lines, then leading CONSTRAINT/INVARIANT(/vault HOTSPOT if live empty). Section name stays `Repository Bearings & Safety`. |
| **F9 — Summary counts stay `matches()`** | CLI `preflight.rs` `:886–888` **untouched**. Live lines must contain `HOTSPOT:` so `in_context_hotspots` can be ≥1 when inject works. |
| **F10 — JSON keys frozen** | T180 `{text, word_count}` + T265 `sections[]`. No `hotspots[]` array. Safety content may change; splitter still keys on Bearings header. |
| **F11 — Intelligence unchanged** | Do **not** edit `query_ledgerful` / fallback. Duplicate later is soft. |
| **F12 — Pins / crates** | No clap 5, no rusqlite 0.40, no chrono 0.4.45, no new crates, workspace **0.1.1**. |
| **F13 — Hermetic skip-env** | `AI_BRAINS_PREFLIGHT_SKIP_LIVE_HOTSPOTS` truthy → skip F2. `hermetic_bin` **sets** `1` after ambient strip (not only denylist) so repo-cwd `ledgerful hotspots` cannot leak into integration tests. Operator/default: unset = inject on. |
| **F14 — File growth** | New retrieval `preflight_safety.rs` (fetch + parse + `format_safety_hotspot_line` + `SAFETY_EMPTY` + skip-env). `safety_marker_glob_sql` in `session_chrome.rs`. `preflight.rs` swaps SQL + prepends + always-emit. **Do not** grow CLI `preflight.rs`, `project.rs`, `sync.rs`, `doctor.rs`, `safety.rs` (CLI dry-run stays). |
| **F15 — T250 float partial** | Live line uses `score={:.2}` (T250 F12 soft, this format only). Do **not** restyle vault pin tables / Intelligence bullets. |
| **F16 — Compact 100 freeze** | T250 `--compact` Safety first-line cap 100 unchanged. Default pretty does not line-cap Safety. |
| **F17 — Decline peers** | T280 hint; T281 nightly 750 ms; T282 `context --show`; T283 list cwd-first; leftover rebind; T240 F2; T255 750 ms; T263 H2; T266 JSON freeze; T275 live bootstrap. |
| **F18 — last-PR Cursor** | #194 empty → **N/A**. #188 closed by T284. Dependabot `#61` rusqlite **not** this track. **No T285.** |
| **F19 — Capture independence** | GLOB + optional ledgerful shell. No models. No new events. No graph default-on. |
| **F20 — PATH** | Do not `cargo install` unless the user asks. |
| **F21 — Stop-before live vault** | No live `pin`. No `safety sync` without `--dry-run`. No leftover `--write --yes`. No grant bootstrap. No `.env` rewrite. |
| **F22 — Tests** | Naming `function_or_feature__condition__expected_result`. rstest for GLOB cases if ≥3. No `unwrap`/`expect`/`panic` in production. `TempEnv` for skip-env units. |
| **F23 — Cross-model** | Retrieval + CLI presentation is FEATURE. After Phase-1 clean, run read-only `codex-review`. |
| **F24 — Debt file** | `conductor/ISSUES.md` does **not** exist. Deferrals → `conductor/deferred.md`. |
| **F25 — Existing tests stay green** | T272 AC2/AC3 leading CONSTRAINT; T219 summary-no-Bearings; T274 Index AC6 (dump may now appear in Index — **do not** assert it stays out of Safety via LIKE); T220 AC5 markers; T264 tags; T265 `sections[]`. |
| **F26 — Docs** | CAPABILITIES preflight Safety row: live hotspots + leading GLOB + honest empty. CHANGELOG T279. OPERATIONS one sentence (`safety sync --dry-run` vs preflight). Skill one-liner if preflight section exists. PROTOCOL-COMPAT: no new required keys. |
| **F27 — PowerShell** | `;` not `&&`. |
| **F28 — No preflight flag** | No `--live-hotspots` / `--no-hotspots`. Skip-env is the test SOOT. `--quiet` does not skip inject. |
| **F29 — `safety.rs` fetch stays** | Do **not** extract a shared crate this track (T277 F44 analog: file-local helper). Drift of JSON parse is a soft residual. |
| **F30 — after_help** | Additive one sentence on `preflight`: Safety is live Ledgerful hotspots (project-scoped) or leading CONSTRAINT/INVARIANT/HOTSPOT pins, not session dumps; empty names `safety sync --dry-run`. |
| **F31 — Governed briefing** | Do **not** enable `AI_BRAINS_GOVERNED_BRIEFING`. Legacy assembly only. |
| **F32 — Session HOTSPOT skip** | T272 F18 `content.contains("HOTSPOT:")` in Sessions **unchanged** (soft residual, not DoD). |
| **F33 — Live classify-only** | Manual AC uses `cargo run -p ai-brains-cli -- preflight --pretty --compact -m 400` from this repo (skip-env **unset**). Do **not** treat PATH T270 binary as proof. |
| **F34 — Empty is pretty-path** | Always-emit is `build_legacy_preflight` text. Summary formatter must not grow. If `--summary` ever printed Bearings, T219 AC8 would fail — keep that split. |
| **F35 — Fail-open live fetch** | Spawn fail / non-zero / no `[` JSON / parse err / empty array → no inject, continue. **No** `wait-timeout` crate. Hang is a soft residual (live `hotspots --json --limit 5` was fast this session). |

---

## 4. Acceptance criteria

| AC | Proof |
|----|-------|
| **AC1** | Unit: `safety_marker_glob_sql` contains `CONSTRAINT:*` / `INVARIANT:*` / `HOTSPOT:*` / `ASSISTANT: CONSTRAINT:*` and does **not** contain `DECISION:`. **Required red.** |
| **AC2** | Unit rstest or cases: `format_safety_hotspot_line` → `HOTSPOT: crates/foo.rs score=0.05`; empty path declined (skip row). Score `{:.2}`. **Required red.** |
| **AC3** | Hermetic CLI (skip-env **on** via helper): pin `CONSTRAINT: T279-bearing-needle-<uuid>` + newer `## Objective\n… CONSTRAINT: buried …`. Pretty Safety contains the bearing needle; does **not** contain `## Objective`. EXIT 0. **Required red.** |
| **AC4** | Hermetic CLI skip-on: vault with **no** leading CONSTRAINT/INVARIANT/HOTSPOT pins. Pretty contains `--- Repository Bearings & Safety ---` **and** `safety sync --dry-run` (F3). Does **not** contain `## Objective`. **Required red.** |
| **AC5** | T272 `preflight_global_isolation__capped_out_safety__appears_in_index` still passes (leading CONSTRAINT GLOB + skip-set). |
| **AC6** | T219 `preflight_pretty__summary__dual_model_unchanged` still has **no** Bearings header on `--summary`. |
| **AC7** | Compact JSON: serde still `{text, word_count}` (+ T265 `sections` if present). No new required keys. |
| **AC8** | Unit: skip-env truthy → `fetch_live_hotspots` returns empty without spawning (TempEnv). Unset + mocked JSON parse unit does **not** need a live binary. |
| **AC9** | Parse unit: stdout with a log line then `[{"path":"crates/ai-brains-cli/src/commands/project.rs","score":0.05,"complexity":21.0,"frequency":9.1}]` → one hotspot path `project.rs`. Missing `[` → empty (fail-open). |
| **AC10** | Manual classify-only (`cargo run`, skip-env **unset**): `preflight --pretty --compact -m 400` Safety contains `HOTSPOT:` and at least one of `project.rs` / `sync.rs` / `forget.rs` / `context.rs` / `governed_common.rs`, **or** F3 remediator if Ledgerful spawn fails. Pass-with-observed-data. **Did not** `safety sync` without dry-run. |
| **AC11** | `--global --pretty` (hermetic, skip on): no requirement that Safety contains cwd `project.rs`. Live inject skipped (F4). |
| **AC12** | `--summary` after AC3 fixture: `in_context_constraints >= 1` (bearing in window). Empty remediator does **not** bump `in_context_hotspots`. |
| **AC13** | Diff omits `project.rs` / CLI `preflight.rs` (except `after_help` in `main.rs` if that is where preflight help lives) / `doctor.rs` / `sync.rs` / `safety.rs`. Prefer: CLI `preflight.rs` **zero hunks**; `main.rs` after_help only. |
| **AC14** | `SAFETY_EMPTY` const unit: no `HOTSPOT:` substring; contains `safety sync --dry-run`. |

---

## 5. Design notes

### 5.1 Why GLOB + live inject (not GLOB-only)

GLOB alone stops the steal (AC3). This vault would then omit Safety (0 leading bearings in the window). The audit compared Safety to `safety sync --dry-run` **paths**. Injecting the same argv makes the section true; F3 covers Ledgerful-down and hermetic.

### 5.2 Why not reuse `index_marker_glob_sql`

That helper includes `DECISION:` (Index pass-1). Safety is CONSTRAINT/INVARIANT/HOTSPOT only. New helper next to it (T274 F36 shape: identifier-checked, bind-free GLOB list).

### 5.3 Why not merge Intelligence

`bridge export --hotspots` is `hotspot_delta` records (opt-in bridge). Live scan is `ledgerful hotspots`. They diverged this session (Intelligence absent, dry-run 5). Merging would steal T214/T264 and still miss the scan. Soft residual if both later print paths.

### 5.4 Hermetic vs live shell

Integration tests run the CLI binary from this repo cwd. `ledgerful hotspots` **would succeed** and leak `#1 project.rs` into Safety, breaking “Safety body is exactly these pins” assertions. F13 sets skip in `hermetic_bin`. Units that test parse/format do not spawn.

### 5.5 `safety_ids` and Index

Dumps that lose Safety via GLOB become eligible for Index (T274 two-pass). That is intended (T274 AC6 deferred CONSTRAINT steal). Do **not** add a chrome detector skip in Safety beyond GLOB.

---

## 6. Non-goals

- Live `safety sync` pin / nightly hotspot ingest
- Enabling `LEDGERFUL_BRIDGE` / fixing Intelligence empty
- T274 rank / Index SQL / `memory list` ORDER
- T272 HashSet algorithm / `GLOBAL_*` retune
- T280 deny `--scope` / T281 750 ms / T282 `--show` / T283 list order
- clap 5 / rusqlite 0.40 / DTO keys / doctor check 16
- Shared crate for `fetch_hotspots_json`
- Command timeout crate
- `--live-hotspots` flag
- Session `HOTSPOT:` skip (T272 F18)

---

## 7. Verification plan (TDD)

**Red first (must fail on HEAD `631a8f8`):**

1. `safety_marker_glob_sql__includes_constraint_not_decision` — AC1
2. `format_safety_hotspot_line__path_and_score__hotspot_prefix` — AC2
3. `parse_hotspots_json__log_then_array__one_path` — AC9
4. `preflight__buried_constraint_dump__not_in_safety` — AC3
5. `preflight__no_bearings__emits_safety_sync_remediator` — AC4
6. `safety_empty_const__no_hotspot_marker` — AC14
7. `skip_live_hotspots_env__truthy__no_spawn` — AC8

**Green:** F1 SQL swap; F2/F13/F35 sibling; F3 always-emit; F7 suppress; F30 after_help; hermetic_bin sets skip.

**Stay green:** AC5 T272; AC6 T219 summary; AC7 JSON keys; T264 tags; T220 counts.

**Manual:** AC10 classify-only `cargo run`; AC11 global; **no** live pin; **no** `cargo install`.

---

## 8. Risk

| Risk | Mitigation |
|------|------------|
| Hermetic tests pick up host hotspots | F13 set skip in `hermetic_bin`; AC8 |
| Preflight latency / hang on `ledgerful hotspots` | limit 5; live was fast; fail-open; no timeout crate (F35 soft hang) |
| Intelligence + Safety both list paths | F11 leave Intelligence; soft dup |
| `--summary` counts jump when inject works | Intended (F9); empty remediator has no `HOTSPOT:` |
| T272 AC2 word budget | Helper skip-on; leading CONSTRAINT still GLOB |
| CLI `preflight.rs` growth | F14 / AC13 zero hunks on that file |
| PATH-behind | F20 / F33 `cargo run` |

---

## 9. Deferred absorb / decline

| Item | Disposition |
|------|-------------|
| Audit Safety = review-track Objective | **Absorb** F1–F3 / AC3–AC4 / AC10 |
| T274 F23 Safety SQL leftover | **Absorb** F1 |
| T274 closeout AC6 buried CONSTRAINT Safety-steal | **Absorb** AC3 (dump must not appear in Safety) |
| T250 F12 HOTSPOT float reformat | **Partial** F15 — live line only |
| T272 skip / T264 caps | **Affirm freeze** F5/F6 |
| T272 F18 session HOTSPOT skip | **Decline** F32 |
| `query_ledgerful` Intelligence empty | **Decline** F11 |
| deny/`policy show` `--scope` | **Decline → T280** |
| nightly Completion vs daemon Open | **Decline → T281** |
| `context --show` leftover shell | **Decline → T282** |
| `project list` leftover-first | **Decline → T283** |
| leftover `7d97a456` 11 roots | **Decline** — T276 Completed; live rebind owner-confirm |
| last-PR Cursor #194 | **N/A** — comments/reviews empty |
| last-PR #188 Work / apply samples | **Decline** — **T284 Completed** `#193` |
| Dependabot `#61` rusqlite 0.40.2 | **Decline** F12 — **no T285** |
| T240 F2 / clap 5 / DTO required keys | **Decline** F12/F17 |
| Identity mismatch quiet | **Not this track** — T258 adopt-path; leftover data T276; shell leftover T282 |
| Live `safety sync` pin | **Decline** F21 |
| T275 live bootstrap | **Decline** F17 |

**Entire `deferred.md` scanned.** Closed/strikethrough rows stay closed. Historical CE wipe, MSI, `anyhow` allowlist, archive `changeguard` — not Safety identity.

---

## 10. Implement order (on go)

1. Phase 0 re-verify SQL `:290–305`, skip `:345`, emit `:353`, `safety.rs` fetch `:102`, clap limit 5, T272 AC2, skip-env helper, deferred rescan, #194 still empty, pins.
2. Red AC1–AC4/AC8/AC9/AC14.
3. `session_chrome.rs` `safety_marker_glob_sql`.
4. `preflight_safety.rs` parse/format/fetch/skip/empty.
5. `preflight.rs` GLOB + prepend + always-emit + F7.
6. `hermetic_bin` sets skip=1; denylist includes the key.
7. `main.rs` after_help; docs.
8. Green + AC5/AC6; classify-only AC10.
9. Review → `review.md`; FEATURE TX; implement-track Phase 6 publish.

---

## 11. Soft residuals

| Residual | Disposition |
|----------|-------------|
| PATH until `cargo install` | F20 |
| Intelligence + Safety path dup if bridge later fills | F11 |
| CLI `safety.rs` vs retrieval JSON parse drift | F29 |
| Unbounded `ledgerful hotspots` wait | F35 |
| Session `HOTSPOT:` skip | F32 / T272 F18 |
| T250 `--max-line` / pager | T250 F12 remainder |
| Doctor 16th check | T255 F11 |
| Live leftover 11 roots | T276 F9 |
| Live 0 of 3 grants | T275 F10 |

---

## 12. Touch map

| Path | Change |
|------|--------|
| `crates/ai-brains-retrieval/src/session_chrome.rs` | `safety_marker_glob_sql` |
| `crates/ai-brains-retrieval/src/preflight_safety.rs` | **new** — fetch/parse/format/empty/skip |
| `crates/ai-brains-retrieval/src/lib.rs` | `mod preflight_safety` |
| `crates/ai-brains-retrieval/src/preflight.rs` | SQL + prepend + always-emit + F7 |
| `crates/ai-brains-cli/src/main.rs` | preflight `after_help` additive |
| `crates/ai-brains-cli/tests/common/mod.rs` | denylist + set skip=1 |
| `crates/ai-brains-cli/tests/preflight_*.rs` | AC3/AC4 hermetic (new or existing file) |
| `Docs/CAPABILITIES.md` / `CHANGELOG` / `OPERATIONS.md` | F26 |
| `conductor/conductor.md` / `deferred.md` / README | Planned + absorb table |

**Do not touch:** `project.rs`, CLI `preflight.rs`, `sync.rs`, `doctor.rs`, `safety.rs`, `ranking.rs`, `graph_density.rs`, `.env`, live vault.

---

## 13. AI fold-in

Reserved. This planning pass has no `*-review.md` yet.
