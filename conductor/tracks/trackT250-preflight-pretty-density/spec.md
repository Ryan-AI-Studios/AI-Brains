# T250 — Preflight pretty density (pass-2)

- **Track ID:** T250-PreflightPrettyDensity
- **Status:** ✅ **Completed** (2026-08-14 PR #165 `bf23f0e`)
- **Category:** UX / FEATURE
- **Owner:** Grok
- **Source:** CLI audit 2026-08-11 P3 — `preflight --pretty` density **7/7** (second pass after T219); T219 soft `--compact` / PrettyOpts
- **Depends on:** T219 pretty readability (newline budget + Scope + role strip + section caps) ✅; T180 full JSON `{text, word_count}` freeze; T214/T220 summary isolation; T216/T224 `strip_role_prefix` SOOT; T241 grants line (leave alone)
- **Blocks / feeds:** Operators can scan `--pretty` without 300–779-char session/recent walls; `--compact` exists. **T251** (device), **T255** (nightly/router) stay separate.
- **Absorbs:** deferred.md “Preflight pretty density (T219 residual)”; T219 F11/F30 `--compact`; placeholder F1–F3 / AC1–AC2; README `preflight --pretty` **7/7**
- **Not absorbed (DoD):** T219 `trim_to_word_budget` / F2b algorithm; T180 JSON key growth; T214/T220 `--summary` rewrite; T224 `strip_role_prefix` SOOT (leading-only); retrieval JSON role strip (T219 F5/F22); is-terminal → `std::io::IsTerminal` (T214 F24 / T249 F12); clap 5 / pin bumps; pager / `comfy-table`; `--max-line` flag; T241 `--install-grants`; T249/T248/T246 product rewrite; marker selection / ranking
- **Research date:** 2026-08-14 (live dogfood + T219 SOOT + CLIG + crates.io pins)
- **AI fold-in:** 2026-08-14 `C:\dev\AI-review.md` **T250** AI1 + AI2. No Highs. **Agree hard:** AI1 M1 governed = chrome-only (no section parser); AI1 L1 extract `truncate_preview_chars` to `display_text`; AI1 L2 hermetic `-m 3000` + seed present; AI1 L3 chrome 32-char unit; AI1 L4 keep Recent recall hint. **Agree:** AI1 L5 OPERATIONS; AI1 L6 after_help additive; AI1 Safety residual note; AI2 M1–M4 / L1–L2 / O1 already planned. **Decline:** AI2 remapped ACs; AI2 M3 `<= 34` byte index. Disposition **§14**.
- **Ledger:** `b54425e3-b479-48ea-b853-368b48aeedd2` (FEATURE)
- **Isolation:** Do **not** change `trim_to_word_budget`, `PreflightContextResponse`, or `--summary` printers. Do **not** rewrite T249/T248/T246. Do **not** change `display_text::strip_role_prefix`. Do **not** print or commit `AI_BRAINS_KEY`.

---

## 1. Objective

1. **Make default `--pretty` scannable.** T219 restored newlines and item-count caps. Live session/recent lines are still 300–779 characters. Apply a Unicode-safe first-line preview cap on **Session** and **Most Recent Memories** only. Keep T219 item counts (safety 8 / turns 6 / sessions 3 / index 15 / recent 3).
2. **Ship `--compact`.** Tighter item caps + first-line-only safety/recent blocks + a shorter line cap. Display-only. JSON and `--summary` ignore the flag.
3. **Strip timestamp-then-role chrome on the pretty path only.** Live Recent lines are `(just now) ASSISTANT: DECISION: …`. Do **not** change `strip_role_prefix` (T216/T219/T224 leading-only SOOT).
4. **Stay capture-independent.** Presentation only. No models, no graph rebuild, no new events, no new crates, no pin bumps.

---

## 2. Live baseline (re-scan 2026-08-14)

### 2.1 Operator dogfood (this machine)

| Signal | Observation |
|--------|-------------|
| `preflight --summary` | Scope `test-alias`; pinned 544; 2 active sessions; in-context 1/43/6; 1076 words. Harnesses grok/agy/opencode `wiring=ok`. Exit **0**. |
| `preflight --pretty --no-hook-prompt -m 1500` | **66 lines / 840 words / 5 headers.** Structure is T219-good (blank lines after `---`). **11 lines > 160 chars.** Longest session line **779**; next 640 / 297. Recent lines 289–511 still start with `(just now) ASSISTANT:`. Memory Index already ~64–68 chars (`truncate_index_summary` 60). Safety CONSTRAINTS ~150–220. Index overflow `+19 more via recall`. Exit **0**. |
| `preflight --compact` | clap `unexpected argument '--compact'` → **exit 2**. Tip suggests `--format`. |
| `preflight --format json` (non-summary) | Compact 2-key `{text, word_count}` (T180). Not re-dogfooded as a wall — keys frozen. |

### 2.2 Why the audit scored 7/7 after T219

| Surface | Truth |
|---------|--------|
| Structure | T219 fixed the single-line wall. Headers, blank lines, item-count caps, F31 `+N` notices, Scope — all live. |
| Density | Item **counts** are fine. Item **length** is not. Session turns and Recent memories dump full pin paragraphs. |
| Chrome | `strip_role_prefix` is leading-only. Retrieval Recent prefixes `(just now) ASSISTANT:` so the role survives. AC5 only rejects lines that *start* with `ASSISTANT:`. |
| `--compact` | T219 F11 deferred the flag (constants-first). Operators who type it get usage **2**. |

### 2.3 Code truth

| Site | Role |
|------|------|
| `commands/preflight.rs` `run()` | TTY default human; `format_preflight_pretty_body(&context.text)` — no caps arg |
| `format_preflight_pretty_body` | T219 F9/F10/F29/F31/F37: item caps, orphan omit, role strip, `+N` |
| `PRETTY_SAFETY_MAX_ITEMS=8` / `TURNS=6` / `SESSIONS=3` / `INDEX=15` | Module consts; Recent `3` is a local const |
| `Preflight` clap | `--pretty`, `--format`, `--summary`, `-m`; **no `--compact`** |
| `PreflightContextResponse` | T180 `{text, word_count}` only |
| `display_text::strip_role_prefix` | Leading `USER:`/`ASSISTANT:`/`SYSTEM:` only (T224 single SOOT) |
| `memory::preview_line` / `truncate_preview_chars` | First non-empty + leading strip + Unicode `…` |
| `retrieval::truncate_index_summary` | 60 chars / keep 57 + `...` — leave alone |
| `retrieval::truncate_turn` | 3 lines / 150 words — leave alone (T219 F38) |
| Hermetic `preflight_pretty_readability.rs` | AC3–AC5/AC7/AC8 — short fixtures; JSON 2-key lock |

### 2.4 Honesty (do not “fix” here)

- JSON `text` is the budgeted retrieval body. Line-capping it would lie to agents (T180 / T219 F5).
- `--summary` is already the compact orientation path (T214/T220). Do not TTY-switch it.
- Marker selection / ranking / `build_legacy_preflight` stay T032/T219 F13.
- Governed `#`/`##` still must not be treated as `---` headers (T219 F14).
- `AI_BRAINS_KEY` never printed.

---

## 3. Research (2026-08-14)

| Topic | Finding | Use in T250 |
|-------|---------|-------------|
| **[CLIG — Saying just enough](https://clig.dev/)** | Too much output drowns what matters; humans first | Default line-cap Session/Recent; `--compact` for even less |
| **CLIG — Human-readable paramount / TTY** | Human output may evolve; scripts pin `--json` | Pretty body may change; JSON keys + text budget frozen |
| **CLIG — Suggest next commands** | Keep T219 F31 `+N` wording | Compact still emits F31 notices with new N |
| **CLIG — Future-proofing** | Changing human is OK; do not break machine | `--compact` ignored on JSON/`--summary` |
| **CLIG — Prefer flags** | Named `--compact` (not a format token) | Bool flag; do not overload `--format compact` (conflicts with T129 `--log-format compact`) |
| **is-terminal 0.4.17** | crates.io: prefer `std::io::IsTerminal` since Rust 1.70 | Soft residual only — **not** DoD (T249 F12 already declined) |
| **clap** | Workspace `4.5` / lock **4.6.1** / crates.io **4.6.6** (2026-08-06). clap **5 not released** | **No bump** |
| **serde_json** | lock **1.0.150** / crates.io **1.0.151** | **No bump**; no DTO growth |
| **chrono** | lock **0.4.44** / crates.io **0.4.45** | Unused here |
| **rustc** | **1.95.0** | Edition 2024 unchanged |
| **T219 F11/L4** | Do not build PrettyOpts for a *soft* residual | Flag is now DoD → small `PrettyCaps` + two constructors only |
| **T216 preview_line** | Unicode truncate + `…` already exists | **Extract** private `truncate_preview_chars` → `pub(crate)` in `display_text` (AI1 L1). Do not fork a third helper in `preflight.rs`. |

---

## 4. Frozen decisions (F1–F16)

| ID | Decision |
|----|----------|
| **F1 — Default pretty line-cap (hard)** | Keep T219 item counts. On **Session** turns and **Most Recent Memories** blocks, emit a Unicode-safe first-line preview of at most **`PRETTY_LINE_MAX = 140`** chars (count `chars()`, keep 139 + `…` when over). Do **not** line-cap Safety, Memory Index, `---` headers, or F31 `+N` notices on the default path. Index stays `truncate_index_summary` (60). |
| **F2 — `--compact` flag (hard)** | `#[arg(long)] compact: bool` on `Preflight`. Display-only. When `human_mode && compact`, use `PrettyCaps::compact()`: safety **3**, turns **2**, sessions **1**, index **5**, recent **2**, `first_line_only` on Safety/Recent blocks, line-cap **`PRETTY_COMPACT_LINE_MAX = 100`** on Session + Recent (+ Safety first line). F31 notices still fire with compact N. |
| **F3 — JSON / summary ignore compact** | `--compact --format json` → T180 2-key compact JSON, **uncapped** `text`. `--summary --compact` → existing summary (human or T220 JSON). No usage error. `--compact` without human_mode is a no-op. |
| **F4 — PrettyCaps (small)** | `format_preflight_pretty_body(text)` stays as a wrapper → `format_preflight_pretty_body_with(text, &PrettyCaps::standard())` so T219 units compile. `PrettyCaps` has the five item caps + `line_max` + `first_line_only`. No heavy `PrettyOpts` / no `--max-line`. **Truncate SOOT (AI1 L1):** promote `memory::truncate_preview_chars` to `pub(crate) display_text::truncate_preview_chars` (same `max_chars==0 → ""` guard; `preview_line` calls it). Pretty line-cap **must** call that helper — do **not** add `truncate_pretty_line` in `preflight.rs`. |
| **F5 — Pretty-only timestamp-role chrome (hard)** | New `strip_pretty_chrome(line) -> &str` used only inside the pretty formatter. Match: trimmed starts with `(`; inner paren **char count** `<= 32` (not a byte index); `)` then whitespace then a leading role token → `strip_role_prefix(after)`. Else `strip_role_prefix(trimmed)`. Bound is fail-closed (over-long parentheticals are not timestamps). Derived from `relative_timestamp` closed set (`just now` / `N min\|hr\|day(s)\|wk\|mo ago` — even `999 mo ago` is 10 chars). **Do not** use AI2 `close_paren_idx <= 34`. **Do not** change `display_text::strip_role_prefix` or `has_leading_role_prefix`. |
| **F6 — Word budget / retrieval assembly unchanged** | No edit to `trim_to_word_budget`, F2b `…`, `truncate_turn`, `truncate_index_summary`, or marker selection. Caps apply **after** the budgeted `context.text`. |
| **F7 — Scope header unchanged** | T219 F6/F6b `format_scope_line` still prefixes human/pretty. Compact does not drop Scope. |
| **F8 — Governed (AI1 M1 hard)** | T219 F14: `#`/`##` are not `---` headers. Governed `build_preflight` emits `render_project_markdown` (`# Project Briefing (governed)` + `##` sections) — **no** `--- Session` / `--- Most Recent` blocks. Pretty/compact therefore classify the body as **Other**: chrome/role-strip only. **Do not** line-cap or item-cap Other / `#` / `##`. **Do not** add a governed section parser (would fight F8/F9). Compact must not crash or strip `##`. |
| **F9 — Capture independence / zero new crates** | String ops + existing Scope SQL. No pager crate, no `comfy-table`. |
| **F10 — Exit codes** | Success **0**. Unknown `--format` stays today's preflight behavior (not T249 case-sensitive `value_parser`). `--compact` is a bool — no case token. |
| **F11 — Docs** | CAPABILITIES pretty-density + `--compact` row; CHANGELOG Unreleased; **OPERATIONS.md** “Generating Preflight Context” one-liner (AI1 L5 — today `--pretty` only). `preflight --help` **new** `after_help` example `ai-brains preflight --pretty --compact` (AI1 L6: additive; `cli_help_ia` only locks Daily group labels). Operator note: default `--pretty` still shows full Safety lines (~150–220); only `--compact` first-line-caps Safety. |
| **F12 — Soft residuals** | is-terminal → std; clap 4.6 workspace pin; retrieval JSON role strip; pager; governed section caps; `--max-line`; T241 `--install-grants`; skill one-liner; HOTSPOT float-score reformat; auto-compact from terminal height |
| **F13 — Isolation** | No T249/T248/T246/T243–T247 rewrite. No `OutputFormat::parse` change. No live `daemon start`/`install`. No `AI_BRAINS_KEY` print. |
| **F14 — Determinism** | Pure formatter; no new timestamps. Relative-time chrome is stripped, not rewritten. |
| **F15 — High findings** | Line-capping JSON `text`; changing `strip_role_prefix` mid-line SOOT; TTY-switching `--summary`; dropping T219 item caps on default; treating `--format compact` as density (collides with log-format); adding a 16th doctor check; growing `PreflightContextResponse`. |
| **F16 — Plan-only until go** | No production code until the user says **go**. |

---

## 5. Acceptance criteria

| ID | Criterion |
|----|-----------|
| **AC1** | Unit: a Session turn / Recent line of 200+ chars through `PrettyCaps::standard()` is ≤ 140 chars and ends with `…` when truncated. A 80-char line is unchanged. |
| **AC2** | Unit: T219 item caps still 8 / 6 / 3 / 15 on `standard()`; existing F31 wording units pass (or updated only if a fixture line exceeds 140 — prefer keep fixtures short). |
| **AC3** | Unit: `PrettyCaps::compact()` caps safety 3 / turns 2 / sessions 1 / index 5 / recent 2; overflow emits F31 notices with compact N. Recent still keeps the trailing `(Use 'recall'…)` hint after `first_line_only` (AI1 L4 — keep). |
| **AC4** | Unit: `strip_pretty_chrome("(just now) ASSISTANT: DECISION: x")` → `"DECISION: x"`; `"(10 hr ago) USER: hi"` → `"hi"`; `"(999 mo ago) ASSISTANT: x"` → `"x"` (AI1 L3 longest `relative_timestamp` form); mid-line `text ASSISTANT: x` unchanged; lowercase `assistant:` unchanged; inner paren **33** chars + role is **not** stripped (fail-closed). |
| **AC5** | Unit: `display_text::strip_role_prefix` SOOT unchanged (leading only; AC9 T219 still green). |
| **AC6** | Unit: `---` headers and `+N more …` notices are never char-truncated. |
| **AC7** | Unit: governed-style `#`/`##` fixture through standard **and** compact — no crash; `##` preserved (T219 AC14 held). A 200-char **non-header** body line under `##` stays **full length** (AI1 M1 — Other is uncapped). |
| **AC8** | Unit: orphan `---` header still omitted (T219 AC18 held). |
| **AC9** | Unit: multibyte (em-dash / CJK) truncate does not panic; no mid-char slice. |
| **AC10** | Hermetic: `--pretty` with **`-m 3000`** (AI1 L2 — must survive `truncate_turn` / word budget). Seed a known 200+ char Session/Recent line; assert that **seed prefix is present** in stdout **and** the displayed line is ≤ 140 and ends with `…`. Scope still present; no display line **starts** with `ASSISTANT:` (T219 AC5 held). |
| **AC11** | Hermetic: `--compact --pretty` exit 0; fewer safety/index items than standard on an over-cap fixture; F31 notices present. |
| **AC12** | Hermetic: `--compact --format json` **`-m 3000`** exit 0; exactly `text` + `word_count`; `text` still contains the **full seeded** long session/recent body (not 100-char capped); no Scope chrome. |
| **AC13** | Hermetic: `--summary --compact` still prints the T214 summary banner + dual counts (not the pretty body). |
| **AC14** | Docs: CAPABILITIES + CHANGELOG + OPERATIONS + `--help`/`after_help` mention `--compact`; Safety residual note (default pretty does not line-cap Safety). |
| **AC15** | Full CI gate; zero new crates; capture-independent. |
| **AC16** | Manual dogfood: live `--pretty` session/recent lines ≤ 140; live `--compact` tighter; live `--format json` 2-key uncapped. |

---

## 6. Non-goals

- Changing which pins/hotspots/decisions are **selected** into preflight
- Growing `PreflightContextResponse` or T220 `PreflightSummaryJson` keys
- Stripping roles inside retrieval for JSON `text`
- Changing `strip_role_prefix` to match mid-line / timestamp prefixes
- TTY-switching `--summary` or default doctor (T192 F10)
- clap 5 / lockfile pin bumps / is-terminal migrate
- Forced pager / `less` / color / `comfy-table`
- `--format compact` as a preflight format token
- Auto `--compact` from terminal height/width
- T241 `--install-grants` / T255 nightly-router / T251 device
- Live daemon start/install

---

## 7. Verification plan

| Phase | Proof |
|-------|-------|
| Red | Unit AC1 (200-char session line still full-length today) |
| Green F1/F4 | `PrettyCaps` + wrapper; AC1/AC2/AC6/AC9 |
| Red chrome | AC4 fails on current `strip_role_on_content_line` |
| Green F5 | `strip_pretty_chrome`; AC4/AC5 |
| Red compact | AC3 |
| Green F2 | compact constructor + clap; AC3/AC11 |
| Hermetic | AC10–AC13; T219 AC3–AC8 still green |
| Targeted | `cargo nextest run -p ai-brains-cli preflight` + clippy `-p ai-brains-cli` |
| Manual | AC16 |
| Full gate | fmt, clippy workspace, nextest workspace, deny, audit, `ledgerful verify` |
| Review | `review.md`; UX/FEATURE; cross-model soft |

---

## 8. Coordination

- **T219:** newline budget + Scope + caps + F31 — keep; absorb F11 `--compact` and add line-cap.
- **T180:** full JSON 2 keys frozen (AC12).
- **T214/T220:** summary isolation (AC13).
- **T216/T224:** `strip_role_prefix` SOOT frozen (AC5); chrome is pretty-only.
- **T241:** grants line on summary — do not touch.
- **T249/T248/T246:** completed presentation peers — do not rewrite.
- **T032:** ANSI/dedup — do not re-open.

---

## 9. Suggested implement snippet (guidance only)

```rust
pub(crate) struct PrettyCaps {
    pub safety_max: usize,
    pub turns_per_session: usize,
    pub max_sessions: usize,
    pub index_max: usize,
    pub recent_max: usize,
    pub line_max: usize,       // applied to Session + Recent (and Safety when compact)
    pub first_line_only: bool, // Safety / Recent blocks
}

impl PrettyCaps {
    pub(crate) fn standard() -> Self { /* 8/6/3/15/3, line_max=140, first_line_only=false */ }
    pub(crate) fn compact() -> Self { /* 3/2/1/5/2, line_max=100, first_line_only=true */ }
}

pub(crate) fn format_preflight_pretty_body(text: &str) -> String {
    format_preflight_pretty_body_with(text, &PrettyCaps::standard())
}
```

Live on go (do not start daemon):

```powershell
ai-brains preflight --pretty --no-hook-prompt -m 1500
ai-brains preflight --pretty --compact --no-hook-prompt
ai-brains preflight --compact --format json
ai-brains preflight --summary --compact
ai-brains preflight --compact   # expect exit 2 today; 0 after if TTY human

# Full gate
cargo fmt --check ; cargo clippy --workspace --all-targets -- -D warnings
cargo nextest run --workspace ; cargo deny check ; cargo audit
ledgerful verify --scope full
```

---

## 10. Risk / review

- **Category:** UX / FEATURE (not SECURITY). Cross-model soft: JSON isolation + `strip_role_prefix` SOOT.
- **Highest regression:** capping JSON `text`; breaking T219 F31 wording; treating `#`/`##` as sections; changing leading-only role strip for recall/forget/memory list.
- **Cap deferred mediums:** ≤3; leftover presentation softs go to F12 / deferred.md.

---

## 11. Suggested implement order (locked)

1. `PrettyCaps` + wrapper (T219 units still compile) → Red/Green AC1 line-cap
2. `strip_pretty_chrome` units (AC4/AC5)
3. Compact constructor units (AC3)
4. Clap `--compact` + `PreflightRunOptions` + hermetics AC10–AC13
5. Docs

---

## 12. Placeholder disposition

| Draft | Disposition |
|-------|-------------|
| F1 Soft `--compact` or lower default caps for non-global | **Absorbed** F2 `--compact` hard; **do not** silently lower default item counts (F1 keeps 8/6/3/15) |
| F2 Keep Scope always-on (T228) | **Absorbed** F7 (T219 F6 already on full body; T228 is recall) |
| F3 JSON compact envelope unchanged (T180/T220) | **Absorbed** F3 / AC12 / AC13 |
| AC1 Hermetic length/structure bounds | **Absorbed** AC10 / AC11 |
| AC2 Live pretty scannable under 800 words | **Absorbed** AC16 (line-cap, not a new word-budget) |

---

## 13. Deferred fold-in

| Item | Source | Disposition |
|------|--------|-------------|
| Preflight pretty density (T219 residual) | deferred.md / README 7/7 | **DoD** F1–F7 / AC1–AC16 |
| T219 F11/F30 `--compact` / PrettyOpts | T219 closeout | **F2/F4** small `PrettyCaps` + flag |
| T219 F22 retrieval JSON role strip | T219 closeout | **Not absorbed** F12 / F3 |
| T219 F22 is-terminal migrate | T219 / T214 F24 | **Not absorbed** F12 |
| T219 F22 clap pin / pager / scope_display | T219 | **Not absorbed** F12 |
| T224 promote `strip_role_prefix` to core | T224 | **Not absorbed** |
| T241 F20 `--install-grants` | T241 | **Not absorbed** (peer) |
| T249 F12/F13 leftover “T250 preflight density” | T249 | **This track** |
| Placeholder F1 lower default caps | spec draft | **Declined** as silent default count change; line-cap instead |

---

## 14. AI fold-in disposition (2026-08-14) — `C:\dev\AI-review.md` AI1 + AI2

No Highs. AI1 is the useful review: governed path is `#`/`##` (caps would be a silent no-op unless pinned), private truncate helper, vacuous hermetic risk, chrome bound, Recent hint. AI2 restates F1–F5 / AC1–AC16 as remapped M1–M4 + remapped AC numbers (T248/T249 repeat).

### Disposition

| ID | Source | Verdict | Action |
|----|--------|---------|--------|
| **AI1 M1** | AI1 | **Agree hard** | **F8** + **AC7**: governed pretty/compact = chrome/role-strip only; Other / `#`/`##` uncapped; no governed section parser |
| **AI1 L1** | AI1 | **Agree hard** | **F4**: extract `truncate_preview_chars` → `display_text`; no third helper |
| **AI1 L2** | AI1 | **Agree hard** | **AC10/AC12**: hermetic `-m 3000` + assert seeded long line present |
| **AI1 L3** | AI1 | **Agree hard** | **F5/AC4**: inner char count ≤32; unit `(999 mo ago)`; 33-char fail-closed |
| **AI1 L4** | AI1 | **Agree hard** | **AC3**: keep `(Use 'recall'…)` after compact Recent |
| **AI1 L5** | AI1 | **Agree** | **F11/AC14**: OPERATIONS one-liner |
| **AI1 L6** | AI1 | **Agree** | **F11**: new `after_help` is additive; `cli_help_ia` group labels only |
| **AI1 Safety residual** | AI1 | **Agree** | **F11**: default pretty does not line-cap Safety; `--compact` does |
| **AI2 M1–M4** | AI2 | **Agree** | Already F1/F2/F4/F5/AC1/AC6 — do **not** copy the broken `truncate_pretty_line` snippet |
| **AI2 L1–L2 / O1** | AI2 | **Agree** | Already F3/F11 / Phase 1 units |
| **AI2 remapped ACs** | AI2 | **Decline** | Keep AC1–AC16 (their AC7–AC12 collide with ours) |
| **AI2 M3 `<= 34`** | AI2 | **Decline** | Inner **char** count ≤32, not byte index 34 |

### Pins locked by fold-in

1. **F8/AC7:** governed markdown is Other — chrome only; 200-char `##` body line stays full length.
2. **F4:** one `display_text::truncate_preview_chars`; `preview_line` + pretty line-cap share it.
3. **AC10/AC12:** `-m 3000` + seed-present (not length-only).
4. **F5/AC4:** `(999 mo ago)` strips; 33-char inner paren does not.
5. **AC3:** compact Recent keeps the recall hint.
6. **F11:** OPERATIONS + Safety residual note + additive `after_help`.

Do not implement until **go**.
