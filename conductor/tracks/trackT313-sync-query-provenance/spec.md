# T313 — `sync query` ledger provenance (rescued heading)

- **Track ID:** T313-SyncQueryProvenance
- **Status:** **Planned** (Pending until **go**)
- **Category:** UX / HONESTY
- **Owner:** Grok
- **Source:** Live CLI audit 2026-08-27 — `sync query` 8/**7**; ledger half “silently” downgrades phrase-miss → first-seen token rescue. Series README `README-T312-T324-CLI-DOGFOOD.md`.
- **Depends on:** T271 ✅ F6 token rescue + F7 banner; T273 ✅ POSIX `--` before QUERY; T231 ✅ always-pretty default; T211 ✅ ledger-first reorder
- **Blocks / feeds:** Daily vault+ledger search honesty. Vault rank remains **T312**. F8 recency remains **T325**.
- **Absorbs:** Audit “can’t tell which results came from where”; T271 F7 “reopen if rescue looks like a phrase hit” — **heading**, not a new banner sentence
- **Not absorbed (DoD):** Vault ranking (T312); T325 PreferRecency; Ledgerful phrase-wrap / token-OR; T231 `--format json` combined envelope; T92 pull/push; growing hotspot `sync.rs`; clap 5
- **Research date:** 2026-08-28 (plan-write product HEAD `cd7bfde` T314 `#232`). Snapshot — **re-verify at execute**.
- **Ledger:** planning DOCS TX `bdf8fddd-84f9-4d9d-9b7d-64887dd834e2`. Series mint DOCS `a6d3c404-1d64-4cba-a743-d75ac16c74cd`. Implement starts a **FEATURE** TX on **go**.
- **Isolation:** Do **not** implement until **go**. Do **not** `cargo install`. Do **not** grow hotspot `sync.rs` (#2) — heading + print live in `sync_query_ledger.rs`. Do **not** edit Ledgerful. Do **not** print or commit `AI_BRAINS_KEY`.

---

## 1. Objective

1. **Phrase vs rescue is obvious in the section chrome.** When T271 token rescue produces hits, the ledger heading itself says the pane is a **rescued token**, not the user phrase. Today the heading is identical for phrase hits and rescue, so a 10-row table looks like a successful phrase search.
2. **Keep the T271 F7 banner sentence exact.** `Note: no phrase match for '<user>'; showing hits for '<token>'.` stays. Heading is the extra signal; do not rename `Note:` (T211 ledger-first uses the same prefix for a different fact).
3. **Vault vs ledger panes stay labeled.** `--- AI-Brains Recall ---` unchanged. Ledger heading stays `--- Ledgerful Ledger Search ---` on phrase hits / misses; rescued hits append `(rescued token: '<tok>')`.
4. **Do not disable rescue.** Empty phrase → first-seen token rescue stays (T271 F6). Honesty, not a miss.
5. **North star.** Capture independence: CLI overlay on `ledgerful ledger search`. No new events. No hidden CoT.

This unblocks daily CLI: operators who type `sync query "graph backend"` must not walk away thinking the ledger had a phrase hit for that string.

---

## 2. Live baseline (re-scan 2026-08-28)

### 2.1 Operator dogfood (this machine)

| Signal | Observation |
|--------|-------------|
| HEAD | `cd7bfde` `feat(cli): T314 unify --format / --dry-run clap semantics (#232)`. Tree **CLEAN**. Branch at plan-write: `main` == `origin/main`. |
| PATH `ai-brains.exe` | `C:\Users\RyanB\.cargo\bin\ai-brains.exe` **26,897,408** B; LastWriteTime **2026-08-27 8:21:55 PM**; `ai-brains 0.1.3`. T271/T273 **are** on PATH. **T312 / T315 / T314 are not.** T313 heading hole **is** on PATH (same `print_ledger`). **Do not `cargo install`.** Tests/manual AC use `cargo run` / hermetic / units. |
| `preflight --summary` (PATH) | Pinned **4544**. In-context **0/0/0**. `Total Word Count: 737` (PATH-behind T315 `Budget window words:`). **Not this DoD.** |
| `ledgerful ledger search --json -- "graph backend"` | `[]` (4 bytes). **Phrase miss.** |
| `ledgerful ledger search --json -- graph` | **Hits** (~10 KB). First-seen rescue token. |
| `ledgerful ledger search --json -- backend` | Hits (smaller). Not first-seen. |
| `ledgerful ledger search --json -- T314` | **3** phrase hits. **Control:** heading must stay generic (no `rescued`). |
| PATH `sync query "graph backend" --limit 3 --quiet` | Vault pane first (T211 ledger-first did **not** fire). Then: `--- Ledgerful Ledger Search ---` / `Note: no phrase match for 'graph backend'; showing hits for 'graph'.` / `10 matching entries for 'graph':` + comfy-table. **Banner is present. Heading is generic.** That is the hole. |
| Last GitHub PR | [#232](https://github.com/Ryan-AI-Studios/AI-Brains/pull/232) T314. `mergedAt` **2026-08-28T11:05:01Z**. Issue comments **[]**. Review comments **[]**. Reviews **[]**. Commit comments **[]**. **last-PR Cursor: N/A empty.** `#230` Bugbot already **T325**. Open PRs: **none**. **No T326.** |
| Ledger | 0 pending / 0 drift at scan (before this DOCS TX). Hotspot **#1** `project.rs` (3.732) — **do not touch.** `sync.rs` **#2** (3.420) — **do not grow.** `governed_common.rs` **#3**. CLI `preflight.rs` #7 (T315) — **do not touch.** |
| `ISSUES.md` | **Does not exist.** |

### 2.2 Why the pane still looks silent

T271 shipped the banner. Live dogfood **prints it**. The 2026-08-27 audit still scored Q=7 because:

| Hole | Why it is still a product hole / why decline extras |
|------|-----------------------------------------------------|
| Generic heading | Phrase hit and rescue share `--- Ledgerful Ledger Search ---`. A 10-row Ledgerful table (`10 matching entries for 'graph':`) looks like success for the user phrase. **DoD: heading names the token.** |
| `Note:` prefix | T211 ledger-first also prints `Note: vault top hit is plan/stale; ledger results shown first.` Two Notes. Do **not** rename F7 (stay-green AC9 / T271). Heading is the differentiator. |
| Banner is one line | Easy to skip above a 10-row table. Keep it; do not make it the only signal. |
| Token `graph` is broad | T271 F6 first-seen by design (length-sort would pick `independence` / T199). **Decline scoring.** Honesty of “this is `graph`, not `graph backend`” is the fix. |
| ndjson has no ledger | T231 F33: machines use `recall` / `--format ndjson` is vault `BridgeRecord`s only. Placeholder said “JSON if a key already exists” — **no key exists.** **Decline** a combined JSON envelope. |
| Vault rank dump-first on PATH | T312 source-only. **Not this DoD.** |

### 2.3 Code truth (opened)

| Item | Location | Notes |
|------|----------|-------|
| Probe + rescue | `sync_query_ledger.rs` `probe_ledger_search` `:228–322` | Phrase JSON empty → first-seen tokens cap **3**. First JSON hit → `banner: Some(ledger_rescue_banner(&forward, token))` `:307–312`. |
| F7 banner | `ledger_rescue_banner` `:155–158` | Exact `Note: no phrase match for '{user}'; showing hits for '{token}'.` Unit `:663–667` stay-green. |
| `LedgerProbeResult` | `:27–34` | `non_empty`, `display`, `banner`. **No `rescued_token` today.** |
| Token cap | `LEDGER_RESCUE_TOKEN_CAP = 3` `:9` | Frozen. |
| Argv | `ledger_search_argv` `:43–51` | Always `ledger search [--json] -- QUERY` (T273). |
| Print | `sync.rs` `print_ledger` closure `:563–571` | Hardcoded `println!("\n--- Ledgerful Ledger Search ---");` then optional banner then display. **This is the hole.** |
| T211 ledger-first | `sync.rs` `:574–577` | `Note: vault top hit is plan/stale; ledger results shown first.` then `print_ledger`. Keep. |
| Vault heading | `sync.rs` `:541` | `--- AI-Brains Recall ---`. Isolation `sync_query_isolation.rs:127` + smoke T124. **Do not change.** |
| ndjson | `sync.rs` `:433–482` | Vault `BridgeRecord` stream. **No ledger pane.** T231 freeze. |
| clap | `main.rs` `SyncCommands::Query` `:3622–3647` | `format: Option<String>` (pretty/text/ndjson). Default pretty. `--no-bridge`, `--quiet`, `--limit` vault-only default **5**. |
| T124 hermetic | `tests/smoke.rs:86` `sync_query__no_bridge__skips_ledgerful_section` | Asserts stdout contains `AI-Brains Recall` and **does not** contain `Ledgerful Ledger Search`. Rescued heading still contains that substring — skip path unchanged. |
| CAPABILITIES | `:399–404` | T231 always-pretty; T271 pane bullet names F7 banner, **not** a rescued heading. |
| OPERATIONS | `:203` | Two-section headings listed generic. |
| Ledgerful wrap | `C:\dev\Ledgerful\src\ledger\db\search.rs:25` | `format!("\"{query}\"")` still. **Do not edit.** |
| Hotspots | `project.rs` #1 / `sync.rs` #2 | Isolation. |

### 2.4 Dependency / standards research (2026-08-28) — snapshot; re-verify at execute

| Pin | Workspace / lock | Ecosystem (today) | Action |
|-----|------------------|-------------------|--------|
| `clap` | workspace **`4.5`** / lock **4.6.1** | crates.io **4.6.6** (2026-08-06). **No clap 5.** | **No bump.** No new flags. |
| `serde_json` | workspace `1.0` | Probe parse only | **No bump.** |
| `rusqlite` | exact **0.40.2** | not this track | **No bump.** |
| rustc / edition | **1.95.0** / **2024** | — | Unchanged |
| workspace | **0.1.3** | — | **No bump** |
| New crates | — | — | **Zero.** |

**Online / primary sources**

| Claim | Source | Takeaway |
|-------|--------|----------|
| Do not lie about what ran | [clig.dev](https://clig.dev/) *Human-first* + *Saying (just) enough* + *Output*: humans first; changing human output is usually OK; stdout is data | A generic heading over a rescued 10-row table **lies by omission**. Heading rewrite is the human chrome. |
| Disclose a rewritten query | clig.dev *Ease of discovery* / *Conversation*: `brew update jq` tells you to run `upgrade`; git “Did you mean”. | Name the token in chrome. Do not silently substitute. |
| Phrase ≠ token | [SQLite FTS5 §3.2 Phrases](https://www.sqlite.org/fts5.html#fts5_phrases) + live Ledgerful `search.rs:25` | `"graph backend"` is adjacent ordered tokens. Token `graph` is a different MATCH. Banner already says so; heading must too. |
| T271 F6 first-seen | This repo T271 F6 / AC4 | Length-sort declined (`independence` / T199). **Do not reopen.** |

**N/A:** SQLCipher page encrypt, schtasks, Windows service, clap 5 (not this bump), vault FTS MATCH / T312 rank, T307 reqwest/tower-http, Index SQL.

**Could not verify:** Whether Ledgerful will ever stop phrase-wrapping spaces (other repo). Live `search.rs:25` still wraps. T313 does **not** wait on that.

**ledgerful / ai-brains:** `preflight --summary` PATH 4544 / 0/0/0 / `Total Word Count`; `recall` of T271 still surfaces plan-audit dumps (PATH-behind T312) — not SoT for clap/src. `ledgerful search print_ledger` → `sync.rs:560` closure. `scan --impact` CLEAN at `cd7bfde`. Hotspots as §2.1.

---

## 3. Frozen decisions

| ID | Decision |
|----|----------|
| **F0 — Go gate** | Plan-only until user **go**. Planning is DOCS TX `bdf8fddd`. Implement starts a **FEATURE** TX. |
| **F1 — Rescued heading** | When `rescued_token` is `Some(tok)` and `tok` is non-empty, ledger heading is exactly `--- Ledgerful Ledger Search (rescued token: '{tok}') ---`. Otherwise exactly `--- Ledgerful Ledger Search ---`. Leading newline before the heading stays (`println!("\n{heading}")`). |
| **F2 — F7 banner sentence frozen** | `ledger_rescue_banner` string **unchanged**. Unit `ledger_rescue_banner__phrase_empty_token_hit__locked_sentence` stay-green. Do not rename `Note:` (T211 collision is why the **heading** changes). |
| **F3 — `rescued_token` field** | Add `rescued_token: Option<String>` to `LedgerProbeResult`. Set `Some(token)` only on the F6 JSON-hit return (`:307–312`). All other arms `None`. Invariant: `banner.is_some() == rescued_token.as_deref().is_some_and(\|s\| !s.is_empty())`. |
| **F4 — Do not disable rescue** | T271 F6 first-seen, cap **3**, `contentful_tokens` via `ai_brains_core`. No scoring. No length-sort. No skip of short tokens like `graph`. |
| **F5 — Do not FTS-quote the argv** | T271 F5 / T273 argv helper unchanged. |
| **F6 — T273 `--` frozen** | `ledger_search_argv` still inserts `--` before QUERY. All T273 units stay-green. |
| **F7 — T231 always-pretty** | Default pretty even non-TTY. ndjson remains vault-only. **No** combined JSON object. **No** new PROTOCOL-COMPAT DTO. Placeholder “JSON if a key exists” → **no key exists → decline**. |
| **F8 — Vault heading frozen** | `--- AI-Brains Recall ---` exact. Do not append `(vault)`. |
| **F9 — T211 ledger-first frozen** | `Note: vault top hit is plan/stale; ledger results shown first.` exact. Reorder rules unchanged. Rescue heading still applies when ledger-first. |
| **F10 — Module / hotspot** | Heading helper + `print_ledger_section` in `sync_query_ledger.rs`. `sync.rs` **calls** it (delete the closure). Do **not** grow ranking/resolve in `sync.rs`. Do **not** touch `project.rs` / `governed_common.rs` / retrieval ranking. |
| **F11 — Decline extras** | No Ledgerful source edits; no `--limit` on ledger argv (vault 5 vs ledger 10 stays T211); no merged multi-token table; no footer after the table; no stderr duplicate of the heading; no `sync query --symbols`; no clap 5; no new crates. |
| **F12 — Decline peers** | T312 rank; T316–T324 placeholders; T325 F8 recency; T307 Blocked; T308 floors; T263 H2; T240 F2; T92 pull/push. |
| **F13 — Pins / crates** | No lock bumps. Workspace **0.1.3**. |
| **F14 — Docs** | CAPABILITIES T271 pane bullet: heading rewrite + F7 banner. OPERATIONS two-section sentence names rescued heading. CHANGELOG Unreleased T313. No PROTOCOL-COMPAT row (no DTO). |
| **F15 — Tests** | Naming `function_or_feature__condition__expected_result`. Units first (red). No `unwrap`/`expect`/`panic` in production. `println!` in the print helper is the existing CLI pattern. |
| **F16 — PATH-behind** | Do not `cargo install` unless the user asks. Manual AC uses `cargo run` / hermetic. PATH-behind is not a fail. |
| **F17 — `--no-bridge` / `--quiet`** | T124 skip pane unchanged. `--quiet` still omits never-ran/failed; hits (phrase or rescued) still print heading + banner + table. |
| **F18 — Miss classes unchanged** | never-ran / failed / ran-empty copy stays T271 F1. Miss panes use the **generic** heading (no `rescued`). |
| **F19 — Capture independence** | Display only. No events. No models. Vault `recall_full` still runs first in the non-reorder path. |
| **F20 — last-PR Cursor `#232`** | **N/A empty** (comments/reviews `[]`). `#230` F8 recency already **T325**. **No T326.** |
| **F21 — Cross-model** | FEATURE / UX honesty. Primary review required. Cross-model **optional** (no contracts DTO, no architecture). |
| **F22 — Debt file** | `conductor/ISSUES.md` does **not** exist. Residuals → `conductor/deferred.md`. |
| **F23 — PowerShell** | `;` not `&&`. |
| **F24 — Stay-green T271/T273/T124/T211/T231** | Banner unit, argv units, `ledger_json_non_empty`, smoke `--no-bridge`, isolation vault header, ndjson vault-only. |
| **F25 — Empty rescued token** | `Some("")` or whitespace-only → treat as phrase heading (generic). Production rescue never sets empty (token comes from `contentful_tokens`). |

---

## 4. Acceptance criteria

| AC | Proof |
|----|-------|
| **AC1** | Unit `ledger_section_heading__rescued_token__names_token`: `ledger_section_heading(Some("graph"))` equals `--- Ledgerful Ledger Search (rescued token: 'graph') ---`. Contains `rescued token` and `'graph'`. **Required red.** |
| **AC2** | Unit `ledger_section_heading__phrase_hit__generic`: `ledger_section_heading(None)` equals `--- Ledgerful Ledger Search ---`. Does **not** contain `rescued`. **Required red.** |
| **AC3** | Unit `ledger_section_heading__empty_token__generic`: `Some("")` → same as AC2. |
| **AC4** | `ledger_rescue_banner__phrase_empty_token_hit__locked_sentence` stay-green (F2). |
| **AC5** | T273 argv units stay-green (`ledger_search_argv__*`). |
| **AC6** | Hermetic `sync_query__no_bridge__skips_ledgerful_section` stay-green (no `Ledgerful Ledger Search`). |
| **AC7** | Isolation vault header `--- AI-Brains Recall ---` stay-green (`sync_query_isolation.rs`). |
| **AC8** | Unit (or small formatter test) `print_ledger_section` / preamble: rescued `LedgerProbeResult` stdout starts with a blank line then AC1 heading then F7 banner then display. Phrase-hit result: blank line then AC2 heading, **no** F7 banner. Can be a pure `format_ledger_section_lines` helper if `println!` is awkward to capture — **prefer a pure `Vec<String>` / joined string** so the unit does not spawn. **Required red.** |
| **AC9** | All `LedgerProbeResult { ... }` literals compile with `rescued_token: None` except the F6 hit arm (`Some(token)`). |
| **AC10** | Docs: CAPABILITIES pane bullet names rescued heading + F7; OPERATIONS two-section sentence; CHANGELOG Unreleased T313. |
| **AC11** | Manual (source bin): `cargo run -q -p ai-brains-cli -- sync query "graph backend" --limit 3 --quiet --no-project-context` from this repo → stdout contains AC1 heading **and** F7 banner for user `graph backend` / token `graph` **and** `matching entries for 'graph'`. Must **not** claim a phrase match for `graph backend` in the heading. PATH-behind is **not** a fail. **Do not** `cargo install`. |
| **AC12** | Manual control: `cargo run … sync query "T314" --limit 3 --quiet --no-project-context` (live phrase hit, n≥1 at plan-write) → heading is AC2 generic; **no** F7 banner. If Phase 0 phrase search for `T314` is empty, pick another live phrase-hit token and record it. |
| **AC13** | `git diff -- crates/ai-brains-cli/src/commands/project.rs crates/ai-brains-retrieval crates/ai-brains-contracts C:\dev\Ledgerful` empty (Ledgerful is out of repo; assert no path under `crates/` except `ai-brains-cli` sync_query_ledger + the `sync.rs` call-site shrink). |
| **AC14** | `--format ndjson` still emits no `Ledgerful Ledger Search` (T231). Stay-green existing ndjson hermetic. |

---

## 5. Design notes

### 5.1 Why heading, not a louder banner

F7 is a locked sentence with a T271 unit and CAPABILITIES copy. Changing it would look like a new product string and would still share the `Note:` prefix with T211. The **section title** is what operators scan. Phrase vs rescue must differ there.

### 5.2 Pure lines helper

```rust
pub(crate) fn ledger_section_heading(rescued_token: Option<&str>) -> String {
    match rescued_token {
        Some(tok) if !tok.is_empty() => {
            format!("--- Ledgerful Ledger Search (rescued token: '{tok}') ---")
        }
        _ => "--- Ledgerful Ledger Search ---".to_string(),
    }
}

pub(crate) fn format_ledger_section_lines(section: &LedgerProbeResult) -> Vec<String> {
    let mut lines = vec![
        String::new(), // leading blank (today's `\n` before heading)
        ledger_section_heading(section.rescued_token.as_deref()),
    ];
    if let Some(ref banner) = section.banner {
        lines.push(banner.clone());
    }
    if let Some(ref text) = section.display {
        lines.push(text.clone());
    }
    lines
}

pub(crate) fn print_ledger_section(section: &LedgerProbeResult) {
    println!("{}", format_ledger_section_lines(section).join("\n"));
}
```

`sync.rs` replaces the closure with `print_ledger_section(section)`.

Display text from Ledgerful already includes its own trailing newline / table. Join with `\n` must not double-gap vs today: today is `println!(heading); println!(banner); println!(text)`. Equivalent: heading, banner, text each on their own println. Prefer **three println!** matching today (blank line via `\n` on first heading println) over a single join if display already ends with `\n`. **Execute matches today’s spacing.** AC8 asserts heading + banner presence/order, not an extra blank-line count vs Ledgerful’s table.

### 5.3 Why not `--limit` on rescue

Audit “10 rows” is Ledgerful’s default. Capping rescue would hide shipped rows and reopen T211 F27 (vault 5 vs ledger 10). Heading honesty is enough.

### 5.4 Why not JSON

T231 F33: `sync query` is human-first; agents use `recall`. Adding a machine envelope is a contract track. Placeholder allowed JSON **only if a key already exists**. None does.

---

## 6. Non-goals

- Editing `C:\dev\Ledgerful` (true token-OR / stop phrase-wrapping).
- Vault T312 rank / T325 PreferRecency / T217 MATCH.
- Combined `--format json` for vault+ledger.
- Passing vault `--limit` into ledger search.
- Rescue scoring / skip-short-token.
- Footer after the table.
- Renaming T211 or T271 `Note:` sentences.
- Growing `sync.rs` with ranking/resolve.
- clap 5 / new crates / silent `.env` / `cargo install` / live pin.

---

## 7. Verification plan (TDD)

Red first (must fail while heading is hardcoded generic in `sync.rs`):

1. `ledger_section_heading__rescued_token__names_token` (AC1)
2. `ledger_section_heading__phrase_hit__generic` (AC2)
3. `ledger_section_heading__empty_token__generic` (AC3)
4. `format_ledger_section_lines__rescued__heading_then_banner` (AC8)

Green: helpers + `rescued_token` field + `print_ledger_section`; `sync.rs` call-site.

Stay-green: AC4–AC7 / AC14 / T273 argv / T271 banner / `ledger_json_non_empty`.

Manual AC11–AC12 on go. Docs AC10. No full workspace nextest as a plan gate.

---

## 8. Risk

| Risk | Mitigation |
|------|------------|
| Heading snapshot tests elsewhere | `rg "Ledgerful Ledger Search"` — only OPERATIONS/CAPABILITIES/smoke substring. Smoke skip still matches. |
| `sync.rs` hotspot grows | F10 extract print; AC13 |
| F7 sentence rot | F2 / AC4 |
| Disable rescue by “fixing” `graph` | F4 |
| JSON envelope sneaks in | F7 / AC14 |
| T211 Note confused with F7 | F1 heading; F9 keep T211 string |
| PATH-behind false AC fail | F16 / AC11 source bin |
| `#232` leftover dropped | F20 N/A empty; T325 already minted |
| Display join doubles newlines | §5.2 match today’s three `println!` |

---

## 9. Deferred absorb / decline

Entire `conductor/deferred.md` scanned 2026-08-28.

| Item | Disposition |
|------|-------------|
| Audit `sync query` phrase→fuzzy opacity / “can’t tell which results came from where” | **Absorb** F1–F3 / AC1–AC3 / AC8 / AC11 |
| T271 F7 reopen if rescue looks like a phrase hit | **Absorb as heading** F1; **affirm** F2 banner exact |
| T271 F6 first-seen + cap 3 | **Affirm** F4 |
| T271 F5 no FTS-quote / T273 `--` | **Affirm** F5 / F6 |
| T271 residual rescue scoring / merge tables / Ledgerful token-OR | **Decline** F11 / F12 (other repo / §11) |
| T271 residual `--limit` on ledger | **Decline** F11 |
| T211 F12 ledger-first / F25 blend | **Affirm** F9; **decline** blend |
| T231 always-pretty / ndjson vault-only | **Affirm** F7 / AC14 |
| T124 `--no-bridge` | **Affirm** F17 / AC6 |
| T312 vault rank dump-first | **Not stolen** (Completed; PATH-behind is T312 F21) |
| T314 clap `--format` / `--dry-run` | **Completed** `#232` — **not stolen** |
| T314 implement PATH / F34 positional steal / expand auto | **Not this DoD** |
| T315 empty-decisions / word-count label | **Completed** — **not stolen** (PATH still `Total Word Count:`) |
| T316–T324 placeholders | **Not stolen** |
| T325 F8 PreferRecency (`#230`) | **Not stolen** |
| T307 Blocked / T308 floors | **Not stolen** / **Decline** |
| T263 H2 / T240 F2 / clap 5 | **Decline** F12 |
| T92 pull/push / T298 device | **Decline** |
| last-PR Cursor `#232` | **N/A empty** F20 — **no T326** |
| last-PR `#230` F8 recency | **T325** already Pending |
| conductor/archive / cargo-audit allowlist | **Not related** |
| PATH T315 `Total Word Count` / T312 dump-first | **Not this DoD** |

---

## 10. Implement order (on go)

1. Phase 0 re-read `sync_query_ledger.rs` probe + `sync.rs` `print_ledger` + T271 banner unit; rescan deferred; FEATURE TX.
2. Red AC1–AC3 + AC8 heading/lines units (must fail on generic heading).
3. Green F1 helper, F3 field, F10 `print_ledger_section`; wire rescue arm `rescued_token: Some(token.clone())`; shrink `sync.rs` closure.
4. Stay-green AC4–AC7 / AC14 / T273 / T271 `ledger_json_non_empty`.
5. Docs F14 / AC10.
6. Manual AC11–AC12 → review → full gate → Complete.

---

## 11. Soft residuals (expected)

| Residual | Note |
|----------|------|
| PATH until `cargo install` | F16 |
| Token `graph` still a broad rescue | F4 honesty, not scoring |
| T211 `Note:` vs F7 `Note:` same prefix | Heading differentiates |
| ndjson still vault-only | F7 |
| Ledgerful still phrase-wraps | F11 other repo |
| 10 ledger rows on rescue | F11 no `--limit` |
| T312 PATH dump-first | Other track |
| T325 F8 PreferRecency | Placeholder |

---

## 12. Touch map (expected)

| Site | Change |
|------|--------|
| `crates/ai-brains-cli/src/commands/sync_query_ledger.rs` | F1 heading; F3 field; F10 print/lines; rescue arm sets token; units AC1–AC4 / AC8 |
| `crates/ai-brains-cli/src/commands/sync.rs` | Replace `print_ledger` closure with `print_ledger_section` (**shrink**) |
| `Docs/CAPABILITIES.md` | T271 pane bullet: rescued heading |
| `Docs/OPERATIONS.md` | Two-section sentence names rescued form |
| `CHANGELOG.md` | T313 Unreleased |
| `conductor/conductor.md` | T313 Planned (status **Pending**) |
| `conductor/deferred.md` | This absorption table |
| `conductor/tracks/README-T312-T324-CLI-DOGFOOD.md` | T313 Planned |

**Do not touch:** `project.rs`, retrieval ranking / `preflight.rs`, contracts, daemon, Ledgerful sources, `.env`, schtasks, `main.rs` Query clap.

---

## 13. AI fold-in disposition

Not yet. Inputs will be `agy-review.md` + `opencode-review.md` after `/review-track T313`. This planning pass has no harness files.

**Planning 2026-08-28.** Still **plan-only until go**.
