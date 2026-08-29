# T321 — `safety sync` write honesty

- **Track ID:** T321-SafetySyncHonesty
- **Status:** **Planned** (Pending until **go**)
- **Category:** UX / SAFETY
- **Owner:** Grok
- **Source:** Live CLI audit 2026-08-27 — `safety sync` 5/**5**. Works, but **writes** (pins hotspots); grouped as read-only-ish (`Synchronize` / `sync`); output chatty. Series README `README-T312-T324-CLI-DOGFOOD.md`.
- **Depends on:** T279 ✅ Safety live inject + remediator `safety sync --dry-run`; T272 ✅ `safety_ids`; T314 ✅ `--dry-run` optional-value on *other* commands (this flag is already a bare bool — do not steal T314); T70/T0163 `symbol_bridge` `schemaVersion` 1 envelope analog
- **Blocks / feeds:** Operators who think `sync` family is read-only. Restores T279 live-inject after Ledgerful JSON envelope. Does **not** populate governed stores. Does **not** steal T322–T326.
- **Absorbs:** Audit write-surprise + chatter; live `hotspots --json` `{schemaVersion, files[]}` (CLI JSON fail → text-mode chatter + displayScore; retrieval fail-open → preflight Safety 0); T279 F29 parser drift **partial** (copy-not-share envelope, cap differs); docs `WORKFLOWS.md` JSON-`LedgerEntry` lie; session-start `antigravity-rule.md` write-without-preview
- **Not absorbed (DoD):** Changing hotspot pin **schema** (`HOTSPOT:` blob); growing `project.rs` / CLI `preflight.rs` / `pin.rs` production / `help_ia.rs`; T264 global mix; T279 Safety SQL/GLOB; T279 F35 Command timeout crate; dry-run-**by-default** (breaking); `--quiet` / `--format json` / `--confirm`; clap 5
- **Research date:** 2026-08-29 (plan-write product HEAD `16edc3f` T318 Completed note `#242`; T318 product `#241` `3bac49e`). Snapshot — **re-verify at execute**.
- **Ledger:** planning DOCS TX `956c8463-c577-44cf-a614-169d77117446`. Series mint DOCS `a6d3c404-1d64-4cba-a743-d75ac16c74cd`. Implement starts a **FEATURE** TX on **go**.
- **Isolation:** Do **not** implement until **go**. Do **not** pin live hotspots as planning or as implement proof. Do **not** `cargo install`. Do **not** print or commit `AI_BRAINS_KEY`. Do **not** grow hotspot `project.rs` (#1) / `sync.rs` (#2) / `governed_common.rs` (#3) / `forget.rs` (#5) / CLI `preflight.rs` (#7).

---

## 1. Objective

1. **Write is obvious.** Default `ai-brains safety sync` still **pins**. `--help` / after_help / clap about say **pin**. A stdout banner `Pinning N Ledgerful hotspot(s) into the vault.` runs **before** `pin::run`. `--dry-run` is the preview (`would pin`, not `would sync`).
2. **Quieter success.** Drop progress theater (`Scanning for Ledgerful Hotspots...`, `Ledgerful scan complete…`, text-mode `--json not available` on stdout). Keep the product: dry-run table / write details + `pin.rs` `Memory {id} successfully pinned`. Do **not** add `--quiet`.
3. **JSON envelope matches live Ledgerful.** `ledgerful hotspots --json` is `{ "schemaVersion": 1, "files": [...] }`, not a top-level array. Restore the JSON path (raw `score`) in CLI **and** retrieval parse (T279 live inject). Legacy `[...]` stay-green.
4. **Do not change what gets pinned** except scores coming from JSON `score` instead of text-table displayScore (restore T279 F2). Blob still starts `HOTSPOT: Brittle files identified by Ledgerful:`.
5. **North star.** Capture independence: CLI honesty + parse. Pins remain explicit events via existing `pin::run`. Preflight Safety live-inject must work **without** a write (T279). Docs must not tell operators to pin at every session start.

This unblocks daily CLI: `sync` looks like `sync query` (read); default is a write; stdout starts with “Scanning…”; JSON fail hides behind a text fallback that prints displayScore **3.65** while T279 wanted raw **0.04**; preflight in-context hotspots stay **0** because retrieval still looks for `[`.

---

## 2. Live baseline (re-scan 2026-08-29)

### 2.1 Operator dogfood (this machine)

| Signal | Observation |
|--------|-------------|
| HEAD | `16edc3f` `chore(conductor): T318 Completed note with PR #241 squash 3bac49e` (`#242`). Product `src/` = T318 `#241` `3bac49e`. Tree **CLEAN**. Branch `track/T321-safety-sync-honesty` off `main` = `origin/main`. Ahead **0** at plan start. |
| PATH `ai-brains.exe` | `C:\Users\RyanB\.cargo\bin\ai-brains.exe` **26,897,408** B; LastWriteTime **2026-08-27 8:21:55 PM**; `ai-brains 0.1.3`. **T279/T272 on PATH.** T312–T318 **not**. `safety.rs` **unchanged** by T316/T318 — hole **is** on PATH **and** source. **Do not `cargo install`.** Tests/manual AC use hermetic / `cargo run`. |
| `preflight --summary` (PATH) | Pinned **4616**. In-context **0/0/0**. `Total Word Count: 701` (PATH-behind T315 `Budget window words:`). **Not this DoD** except: **0 hotspots** is the envelope hole (retrieval parse). |
| `safety --help` | Parent about `Manage repository safety signals`. Subcommand `Synchronize Ledgerful hotspots into the AI-Brains vault`. **No** after_help. |
| `safety sync --help` | `--limit` default **5**; `--dry-run` “Preview what would be synced without pinning”. **No** after_help. No “pin” in about. |
| PATH `safety sync --dry-run` | `Scanning for Ledgerful Hotspots...` then `Ledgerful scan complete (text mode, --json not available: no JSON array found in ledgerful output).` then `--- Dry Run: would sync 5 hotspot(s) ---` with **displayScore-like** `3.65/3.45/3.32/2.95/2.81` + freq + complexity. Exit **0**. **No pin.** |
| `ledgerful hotspots --json --limit 5` | Object `{ "schemaVersion": 1, "files": [ { path, score, displayScore, complexity, frequency } ], "resultCount": 5, "limit": 5 }`. First line `{` — T279 F36 finder wants `[`. Raw `score` ~**0.037**; `displayScore` ~**3.65**. |
| Last GitHub PR | [#242](https://github.com/Ryan-AI-Studios/AI-Brains/pull/242) T318 conductor note. `mergedAt` **2026-08-29T13:10:38Z**. Issue/review/inline comments **[]**. Product [#241](https://github.com/Ryan-AI-Studios/AI-Brains/pull/241) T318 list. `mergedAt` **2026-08-29T13:09:20Z**. Comments **[]**. `#237` Bugbot already **T326**. `#230` already **T325**. Open PRs: **none**. **No T327 from Cursor.** Envelope is **this** track (live src), not a leftover mint. |
| Ledger | 0 pending / 0 drift. Hotspot **#1** `project.rs` (**3.648**) — **do not touch.** `sync.rs` **#2**. `governed_common.rs` **#3**. `forget.rs` **#5**. CLI `preflight.rs` **#7**. `safety.rs` **not** in top 10. |
| `ISSUES.md` | **Does not exist.** |

### 2.2 Why this is still the hole

| Layer | Truth |
|-------|--------|
| Default is a write | `#[arg(long)] dry_run: bool` — absent = **false** → `pin::run(..., dry_run)` after building a HOTSPOT blob. `pin` itself is Daily; `safety` is Operator (`help_ia.rs:12`) but named `sync` next to `sync query`. |
| Chatty stdout | Always `Scanning…` (`safety.rs:16`). JSON path prints `Ledgerful scan complete: N`. Fail-JSON prints the **entire** `json_err` on stdout then falls back to text (`:24–32`). Write also prints `Syncing top N…`, a details fence, then `Safety synchronization complete` **plus** `pin.rs` `Memory {id} successfully pinned`. |
| `--dry-run` is already the preview | T279 remediator **is** `ai-brains safety sync --dry-run` (`SAFETY_EMPTY`). Flipping default to dry-run would break OPERATIONS examples and require `--dry-run false` to pin (T291-class). **Do not flip.** Banner + about/after_help. |
| JSON envelope | Live Ledgerful matches `symbol_bridge.rs` `schemaVersion` 1 object-with-array. CLI finder `:116–118` and retrieval `parse_hotspots_json` `:31–33` still want a line starting `[`. CLI **text-falls-back** (works, wrong scores). Retrieval **fail-opens empty** → pretty Safety remediator, in-context hotspots **0**. Operators following `next: safety sync --dry-run` then drop `--dry-run` to “fix” Safety — that is the write surprise. |
| Docs | `WORKFLOWS.md:344` claims JSON `LedgerEntry` array (false — human table + pin). `antigravity-rule.md:58` session-start `ai-brains safety sync` **without** `--dry-run` (write every session; T279 already live-injects). |

### 2.3 Code truth (opened)

| Item | Location | Notes |
|------|----------|-------|
| `run` | `commands/safety.rs:11–100` | Scan lines → fetch → empty healthy → dry-run table → write details → `pin::run` → complete. **Banner + drop scan + envelope.** |
| `fetch_hotspots_json` | `:102–128` | `ledgerful hotspots --json --limit N`; first `[` line. **Accept `{` envelope `files[]`.** |
| `fetch_hotspots_text` | `:130–172` | Markdown table. **Keep as fallback**; demote diagnostic to `tracing::warn!`. |
| `render_hotspots` | `:174–187` | `HOTSPOT: Brittle files…` + numbered path/score/freq/complexity. **Keep leading line;** scores from JSON `score`. Freq/complexity in **blob** stay (pin schema); human details may drop freq/complexity (F5). |
| clap `SafetyCommands::Sync` | `main.rs:3716–3725` | `--limit` default 5; `--dry-run` bool. **No new flag.** after_help **none** — add. about **Synchronize** → **Pin**. |
| Dispatch | `main.rs:5427–5430` | `run(&ctx, *limit, *dry_run)`. |
| `pin::run` | `pin.rs:19–119` | Write prints `Memory {turn_id} successfully pinned`. Dry-run `[dry-run] Would pin memory`. **Do not edit production.** Safety already returns before pin on `--dry-run`. |
| Retrieval parse | `preflight_safety.rs:29–62` | F36 `[` + cap `LIVE_HOTSPOT_LIMIT=5`. **Envelope `files[]` + stay-green array.** Fail-open empty stays F35. |
| `SAFETY_EMPTY` | `preflight_safety.rs:8` | `next: ai-brains safety sync --dry-run`. **Freeze.** |
| Spawn argv | `preflight_safety.rs:111–125` | Same `hotspots --json --limit 5`. **Do not change argv.** |
| Preflight help | `main.rs:1185–1197` / `:1748` | `safety sync --dry-run`. **Stay-green.** |
| `help_ia` | Operator includes `safety` | **Freeze.** Do not move to Daily. |
| Hermetics | **No** `safety sync` CLI test today | Help + clap default + format helpers + retrieval envelope unit are the reds. |
| T279 parse units | `preflight_safety.rs:151–178` | `log_then_array` + cap-5. **Stay-green.** |
| Line counts | `safety.rs` **168** nonblank / **187** physical; `preflight_safety.rs` **227** physical; `pin.rs` **138**. Snapshot — **F22 80-net is phase diff vs go HEAD**. |
| Contracts | none | No DTO. PROTOCOL-COMPAT N/A. Retrieval `parse_hotspots_json` is crate-private (`mod preflight_safety` not `pub use`). **Copy-not-share** (F29) because CLI `--limit` is unbounded vs inject cap 5. |

### 2.4 Dependency / standards research (2026-08-29)

| Pin | Workspace / lock | Action |
|-----|------------------|--------|
| `clap` | workspace **4.5** / lock **4.6.1** / crates.io **4.6.6** (2026-08-06; [docs.rs/clap/4.6.6](https://docs.rs/clap/4.6.6/clap/struct.Command.html) `after_help`) | **No bump.** Additive after_help + about. clap 5 **forbidden**. |
| `serde_json` | lock **1.0.150** | **No bump.** Envelope deserialize local. |
| `rusqlite` | exact **0.40.2** | **No bump.** No SQL. |
| `uuid` | ws `"1.13"` / lock **1.23.1** | **No bump.** |
| rustc / edition | **1.95.0** / **2024** | Unchanged. |
| workspace version | **0.1.3** | **No bump.** |
| New crates | — | **Zero.** No wait-timeout (T279 F35 decline). |

**CLI / JSON research (primary sources):**

| Source | What we take | What we decline |
|--------|----------------|-----------------|
| [clig.dev Output](https://clig.dev/#output) (fetched 2026-08-29) | Human-first; “If you change state, tell the user”; “saying (just) enough”; “Display output on success, but keep it brief”; “Changing output for humans is usually OK”; `-n` / `--dry-run` describes changes without running; stdout = data; “Make the default the right thing” | Dry-run-**by-default** (clig `--dry-run` is the preview flag, not the default). Keeping scan theater because “show progress” — this spawn is sub-second |
| [clig.dev Arguments](https://clig.dev/#arguments-and-flags) | `--dry-run` standard name; prefer flags | New `--confirm` / `--commit` / `--quiet` |
| [about_Redirection](https://learn.microsoft.com/en-us/powershell/module/microsoft.powershell.core/about/about_redirection) | Native stderr is stream 2 / ErrorRecord | Putting the pin banner on stderr |
| T279 F2 / F3 / F21 | Raw `score={:.2}`; remediator `--dry-run`; do not live-pin as proof | Rewriting Safety SQL; Command timeout crate |
| `symbol_bridge.rs:41–54` | `schemaVersion` 1 object + named array (`symbols`) | Requiring exact `schemaVersion==1` reject (accept `files[]` whenever present; unknown object without `files` → JSON Err → CLI text fallback / retrieval empty) |
| `pin` / `backup create` | Default writes; `--dry-run` opt-in | `query progressive` default-true dry-run (governed persist — different family) |

N/A-if-skipped: SQLCipher, schtasks, llama.cpp `/health`, FTS5, clap `num_args` (no new flags), T180 DTO keys.

**Could not verify:** whether older Ledgerful still emits a top-level array (stay-green hermetic covers it). Exact live `score` vs `displayScore` mapping formula (not needed — use raw `score`). Do not print `AI_BRAINS_KEY`.

**ledgerful / ai-brains:** `preflight --summary` Pinned **4616** / in-context **0/0/0** / words **701** (PATH). `recall "safety sync dry-run pin hotspots"` lexical hits the 2026-08-27 audit dump (PATH dump-first / T312 not installed). `ledgerful ledger status --compact` 0 pending / 0 drift; `search "fetch_hotspots_json"` → `safety.rs:102`; `hotspots` table matches dry-run 5 files; `scan --impact` CLEAN at `16edc3f`.

---

## 3. Frozen decisions

| ID | Decision |
|----|----------|
| **F0 — Go gate** | Plan-only until user **go**. Planning is DOCS. Implement starts a FEATURE TX. |
| **F1 — Default stays write** | `dry_run: bool` absent = **false**. Do **not** default `--dry-run`. Do **not** T314 optional-value / `--dry-run false` to pin. T279 remediator stays a **preview**. OPERATIONS write example stays valid. |
| **F2 — Write banner** | When `!dry_run` and `hotspots` nonempty, stdout **before** `pin::run`: `Pinning {n} Ledgerful hotspot(s) into the vault.` Helper `format_write_banner(n) -> String`. |
| **F3 — Help honesty** | `SafetyCommands::Sync` about names **pin**. `--dry-run` help: preview **without pinning**. One additive **after_help**: default pins Ledgerful hotspots into the vault; `--dry-run` previews; preflight already live-injects without pinning. |
| **F4 — Drop scan theater** | Delete `Scanning for Ledgerful Hotspots...` and `Ledgerful scan complete: N`. Text-mode / JSON-err diagnostic → `tracing::warn!` (not stdout). Fail path still `return Err(...)` when **both** JSON and text fail. |
| **F5 — Dry-run / details quieter** | Header SOOT `--- Dry Run: would pin {n} hotspot(s) ---` (not `would sync`). Human rows: `  {i}. {path} (score: {score:.2})` using **raw** JSON `score` (T279 F2). Drop freq/complexity from **human** list. Write details fence may stay; same row shape. Pin **blob** `render_hotspots` may keep freq/complexity (schema freeze). |
| **F6 — pin.rs freeze** | Do **not** edit `pin.rs` production. Keep `Memory {id} successfully pinned`. **Drop** `Safety synchronization complete. N hotspot(s) pinned to vault.` (duplicate). `--dry-run` still returns **before** `pin::run`. |
| **F7 — Envelope parse** | CLI `fetch_hotspots_json` and retrieval `parse_hotspots_json` accept: (a) object with `files` array (`schemaVersion` optional); (b) legacy top-level array after first `[` line (T279 F36). Finder: first line `trim_start` `{` **or** `[`. Raw field `score` (ignore `displayScore`). CLI does **not** cap (ledgerful `--limit` already applied). Retrieval still caps `LIVE_HOTSPOT_LIMIT=5`. |
| **F8 — T279 remediator / GLOB freeze** | `SAFETY_EMPTY` exact. Preflight after_help `safety sync --dry-run` stay-green. Safety SQL/GLOB / skip-set / `--global` no live-inject **unchanged**. Do **not** grow CLI `preflight.rs`. |
| **F9 — Flags** | **No** new clap flag (`--quiet` / `--format` / `--confirm` / `--usable-only`). `--limit` default **5** freeze. |
| **F10 — No list JSON** | `safety sync` stays human stdout. Do **not** add `--format json`. `WORKFLOWS.md` LedgerEntry claim is **wrong** — fix docs. |
| **F11 — Isolation** | Edit `safety.rs` + `main.rs` SafetyCommands about/after_help + retrieval `parse_hotspots_json` + hermetics + docs. **Do not** grow `project.rs` / `sync.rs` / `forget.rs` production / `doctor.rs` / `governed_common.rs` / `pin.rs` production / CLI `preflight.rs` / `help_ia.rs` / `session_chrome.rs`. |
| **F12 — No live pin as proof** | Planning and implement **proof** = `--dry-run` + hermetics + parse units. Do not pin the operator vault. |
| **F13 — Decline extra CLI** | No `--quiet`, JSON summary, `VerifyError`-class, `--commit`, dry-run-default. |
| **F14 — Capture independence** | Presentation + JSON parse. No models, embeddings, graph, new events except the existing pin path when the operator **omits** `--dry-run`. |
| **F15 — Pins / crates** | No clap 5, no lock bumps, no new crates, workspace **0.1.3**. |
| **F16 — Standing declines** | T263 H2; T240 F2; T308 floors; T307 Blocked; csrf; KIND bump. |
| **F17 — Tests** | Naming `function_or_feature__condition__expected_result`. No `unwrap`/`expect`/`panic` in production. `tempfile` not required for parse units. |
| **F18 — Cross-model** | UX + parse is FEATURE. After Phase-1 clean, run read-only `codex-review`. |
| **F19 — Debt file** | `conductor/ISSUES.md` does **not** exist. Deferrals → `conductor/deferred.md`. |
| **F20 — PowerShell** | `;` not `&&`. |
| **F21 — F36 finder** | Extend, do not delete array path. Object pretty-print first line `{` is live SoT. |
| **F22 — Line-count 80-net** | `safety.rs` production net **&lt;80 physical vs go HEAD**. Retrieval parse + one test may grow `preflight_safety.rs`. Snapshot §2.3 is not the cap. |
| **F23 — after_help required** | Named hermetic AC1. |
| **F24 — Stay-green** | T279 `parse_hotspots_json__log_then_array__one_path` + cap-5; `SAFETY_EMPTY` contains `safety sync --dry-run`; `preflight__help__names_session_safety_hotspots`; `preflight__no_bearings__emits_safety_sync_remediator`; pin `--dry-run` `[dry-run] Would pin memory` (if a pin hermetic exists — do not add). |
| **F25 — last-PR Cursor** | `#242` / `#241` comments **empty**. `#237` → **T326**. `#230` → **T325**. **No T327.** Envelope is absorbed here. |
| **F26 — Decline peers** | T322–T324; T325 F8 recency; T326 pin-count; T318 Completed; T307 Blocked. |
| **F27 — PATH-behind** | T315/T312–T318 not on PATH — **not this hole**. Hermetic / `cargo run` SoT. Do not `cargo install`. |
| **F28 — Dual-truth** | Preflight Safety **live-injects** (read). `safety sync` **pins** (write). after_help names both. |
| **F29 — Copy-not-share** | Do **not** `pub use` retrieval parse into CLI (inject cap 5 vs CLI `--limit` 20). Copy envelope field names. Comment both parsers. Closes T279 F29 *drift* without a shared helper. |
| **F30 — Stdout product** | Banner / dry-run table / details on **stdout**. Windows-first: do not `eprintln!` the banner. |
| **F31 — Text fallback stays** | Old Ledgerful / broken JSON still uses `fetch_hotspots_text`. Do not delete. |
| **F32 — Pin blob leading line freeze** | `HOTSPOT: Brittle files identified by Ledgerful:` stays. Score values may change 3.65 → 0.04 when JSON path works (restore, not schema). |
| **F33 — Docs honesty** | OPERATIONS lead with `--dry-run`; write example commented as **pins**. `WORKFLOWS.md` drops LedgerEntry JSON lie. `antigravity-rule.md` session-start uses `--dry-run` **or** `preflight` (T279). CAPABILITIES/CHANGELOG. |
| **F34 — help_ia freeze** | Safety stays Operator. Do not add `safety` to Daily. |
| **F35 — T279 F35** | Unbounded `ledgerful hotspots` wait — **decline** timeout crate this track. |
| **F36 — Empty hotspots** | Keep `No hotspots identified. Safety layer is healthy.` (no banner, no pin). |

---

## 4. Acceptance criteria

| ID | Criterion | Proof |
|----|-----------|--------|
| **AC1** | `safety sync --help` after_help names **pin** / default writes **and** `--dry-run` preview; about contains `pin` (case-insensitive ok) | New `safety_sync_help__after_help__names_pin_and_dry_run` |
| **AC2** | `Cli::try_parse_from(["ai-brains","safety","sync"])` → `dry_run == false`; with `--dry-run` → `true` | New `safety_sync_clap__default__dry_run_false` |
| **AC3** | `format_write_banner(5)` contains `Pinning`, `5`, `vault`; does **not** contain `Scanning` / `sync` | Unit `format_write_banner__names_pinning_and_count` |
| **AC4** | `format_dry_run_header(5)` contains `would pin` and **not** `would sync` | Unit `format_dry_run_header__would_pin_not_sync` |
| **AC5** | Retrieval envelope fixture `{schemaVersion:1, files:[{path, score:0.037…}]}` → one `LiveHotspot` with **raw** score (not displayScore 3.65); legacy array test stay-green | New `parse_hotspots_json__envelope_v1_files__raw_score` + stay-green `:151` |
| **AC6** | CLI envelope unit (same shape) returns path + raw score; ignores `displayScore` | New `fetch_or_parse_hotspots_json__envelope_v1_files__raw_score` in `safety.rs` `#[cfg(test)]` (parse helper, no spawn) |
| **AC7** | Production `safety.rs` has **no** `Scanning for Ledgerful Hotspots` and **no** `Ledgerful scan complete` string | Source/unit or `include_str!` assert in the help/parse test file |
| **AC8** | `SAFETY_EMPTY` still contains `safety sync --dry-run` | Stay-green `preflight_safety.rs` unit + `preflight__no_bearings__emits_safety_sync_remediator` |
| **AC9** | Preflight `--help` still names `safety sync --dry-run` | Stay-green `preflight__help__names_session_safety_hotspots` |
| **AC10** | `--limit` default 5; unknown flag clap exit 2 | Stay-green parse + clap |
| **AC11** | `pin.rs` production empty of behavior diff | `git diff -- crates/ai-brains-cli/src/commands/pin.rs` empty |
| **AC12** | CLI `preflight.rs` / `help_ia.rs` / `project.rs` / `doctor.rs` production empty of behavior diff | name-only `git diff` |
| **AC13** | Docs: CAPABILITIES hotspot row; OPERATIONS §7 `--dry-run` first; WORKFLOWS no `LedgerEntry`; antigravity-rule session-start `--dry-run` or `preflight`; CHANGELOG T321 | File grep |
| **AC14** | Manual `cargo run -p ai-brains-cli -- safety sync --dry-run`: **no** `Scanning`; **no** `text mode`; header `would pin`; scores **&lt; 1.0** (raw); exit 0; no new pin. **Do not** omit `--dry-run` | Recorded stdout |
| **AC15** | Empty-hotspots path (fixture / stub): `Safety layer is healthy` / no `Pinning` | Unit or hermetic if cheap; else covered by empty-branch source + AC14 when N=0 (live N=5 — skip live empty) |
| **AC16** | T279 cap-5 array still 5 | Stay-green `parse_hotspots_json__more_than_five__caps` |

---

## 5. Design notes

### 5.1 `run` emit (CLI)

1. Fetch (JSON envelope then text fallback). **No** scanning println.
2. Empty → `No hotspots identified. Safety layer is healthy.` return.
3. Dry-run → `format_dry_run_header` + path/score rows + `--- End Dry Run ---` return (**before** pin).
4. Write → `format_write_banner` + details rows + `pin::run(..., false)` + **no** extra complete line.

Helpers live in `safety.rs` (`pub(crate)` or private + `#[cfg(test)]` via super).

### 5.2 Envelope (CLI + retrieval)

```text
{ "schemaVersion": 1, "files": [ { "path", "score", "displayScore", "complexity", "frequency" } ], "resultCount", "limit" }
```

- Deserialize `files` (rename `schemaVersion` optional).
- Legacy: first `[` line → `Vec<T>` (T279).
- Extra JSON fields ignored (serde default).
- CLI struct keeps complexity/frequency for `render_hotspots` blob.
- Retrieval `LiveHotspot` stays path+score.

### 5.3 Why not dry-run-by-default

Placeholder asked to pick. **Banner-only:**

- T279 F3 remediator is already the preview; empty Safety must not require `--dry-run false` to mean “look”.
- `pin` / `backup create` default write.
- clig `--dry-run` is the opt-in preview flag.
- Breaking OPERATIONS / antigravity-rule / scripts that already pin.

Honesty is about + after_help + banner, not a clap default flip.

### 5.4 Dual-truth

Pretty Safety shows live paths **without** pinning (T279). `safety sync` **creates** a vault HOTSPOT memory. after_help must say that so the remediator is not mistaken for a populate-write.

---

## 6. Non-goals

Dry-run-by-default / `--dry-run false` persist. `--quiet` / `--format json` / `--confirm`. Editing `pin.rs` production. Growing CLI `preflight.rs` / `help_ia.rs` / `project.rs`. T279 GLOB/SQL retune. T279 F35 timeout crate. T264 caps. T322–T326. clap 5. Pin→Approved (H2). Silent `.env`. Live pin as Complete proof. Shared retrieval↔CLI parse helper (F29). Deleting text fallback.

---

## 7. Verification plan (TDD)

**Red first** (must fail on today’s tree):

- AC1 after_help / about `pin` — today’s help is `Synchronize` / `synced`, no after_help.
- AC2 is **green-on-arrival** for `dry_run == false` (lock the freeze) — write in Phase 1 anyway.
- AC3/AC4 helpers **absent**.
- AC5 envelope unit — today `parse_hotspots_json` of live object → empty (no `[`).
- AC6 CLI envelope unit **absent**.
- AC7 `Scanning` string **present**.

**Green:** helpers + about/after_help + envelope both parsers + drop scan/complete/text-mode stdout + banner + `would pin` + drop duplicate complete.

**Stay-green:** AC8/AC9/AC10/AC11/AC12/AC16.

**Manual AC14:** `cargo run -- safety sync --dry-run` only.

---

## 8. Risk

| Risk | Mitigation |
|------|------------|
| Operators miss that default writes | F1 keep write; F2 banner; F3 after_help; F33 docs `--dry-run` first |
| Envelope-only parse breaks old array Ledgerful | F7/F21/F24/F31 dual parse + text fallback |
| Retrieval/CLI parser drift again | F29 comment + AC5/AC6 twin fixtures |
| Raw scores look “too small” vs table 3.65 | T279 F2 already shipped 0.05; CAPABILITIES sentence; displayScore is Ledgerful UI |
| Pin blob score change | F32 restore JSON path; not a new kind |
| Accidental live pin in implement | F12; Phase 0/4 forbid omitting `--dry-run` |
| 80-net | Drop more lines than add; helpers small |

---

## 9. Deferred absorb/decline

| Item | Disposition |
|------|-------------|
| Audit `safety sync` 5/5 write surprise + chatter | **Absorb** F1–F6 / AC1–AC4 / AC7 / AC14 |
| Placeholder dry-run-default vs banner | **Pick banner** F1 (not breaking) |
| T279 remediator `safety sync --dry-run` | **Affirm freeze** F8 |
| T279 F21 no live pin as proof | **Affirm** F12 |
| T279 F29 CLI vs retrieval parse drift | **Partial** F7/F29 copy-not-share envelope |
| T279 F35 unbounded wait | **Decline** F35 |
| T279 F32 session HOTSPOT skip / T272 skip-set / T264 caps | **Decline** / **affirm freeze** F8 |
| Live JSON `{files[]}` (dogfood 2026-08-29) | **Absorb** F7 / AC5 / AC6 |
| `WORKFLOWS.md` LedgerEntry JSON | **Absorb** F10 / AC13 |
| `antigravity-rule.md` session-start write | **Absorb** F33 / AC13 |
| T316 F36 stderr analog | **Analog only** F30 — banner stays stdout |
| T318 / T322–T324 / T325 / T326 | **Not stolen** |
| T307 Blocked / T308 floors / H2 / clap 5 / T240 F2 | **Not stolen** / **Decline** |
| last-PR Cursor `#242` / `#241` | **N/A empty** |
| last-PR `#237` / `#230` | **T326** / **T325** already Pending — **no T327** |
| `ISSUES.md` | **Does not exist** |

---

## 10. Implement order (on go)

1. Phase 0 re-read `safety.rs` `run`/`fetch_*`/`render_hotspots` + clap Sync + retrieval `parse_hotspots_json` + T279 units + `--dry-run` dogfood. Confirm envelope still `{files[]}`. Start FEATURE TX.
2. Red: AC1–AC7 tests (must fail except AC2 lock).
3. Green: envelope both parsers; emit helpers; drop scan/complete; banner; about/after_help.
4. Stay-green T279 + pin.rs empty diff + preflight help.
5. Docs AC13. Manual AC14 dry-run only.
6. FEATURE cross-model; full gate; conductor Completed; implement-track Phase 6 publish.

---

## 11. Soft residuals

| Item | Note |
|------|------|
| PATH until owner `cargo install` | F27 — hermetic/`cargo run` SoT |
| Text fallback if Ledgerful JSON regresses | F31 — warn-only diagnostic |
| In-context hotspots 0 on **PATH** until install | Expected; source parse is SoT |
| Unbounded `ledgerful hotspots` wait | F35 declined |
| Pin UUID line after banner | F6 pin.rs freeze — useful id |
| `displayScore` not shown | By design F5 |

---

## 12. Touch map

| Path | Change |
|------|--------|
| `crates/ai-brains-cli/src/commands/safety.rs` | Emit + envelope parse + helpers + units |
| `crates/ai-brains-cli/src/main.rs` | Sync about + after_help + AC1/AC2 clap tests (near other help tests) |
| `crates/ai-brains-retrieval/src/preflight_safety.rs` | Envelope parse + AC5 unit |
| `Docs/CAPABILITIES.md` | Hotspot pin honesty |
| `Docs/OPERATIONS.md` | §7 `--dry-run` first |
| `Docs/WORKFLOWS.md` | Drop LedgerEntry lie; `--dry-run` for discovery |
| `Docs/antigravity-rule.md` | Session-start preview |
| `CHANGELOG.md` | T321 row |
| `conductor/conductor.md` / `deferred.md` / series README | Registry + absorb |

**Do not touch:** `pin.rs` production, CLI `preflight.rs`, `help_ia.rs`, `project.rs`, `doctor.rs`, `sync.rs`, `forget.rs` production, `session_chrome.rs`, contracts, `Cargo.toml` pins.
