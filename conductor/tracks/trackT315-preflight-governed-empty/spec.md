# T315 — Preflight summary: empty-decisions next-step + word-count label

- **Track ID:** T315-PreflightGovernedEmpty
- **Status:** **Planned** (Pending until **go**)
- **Category:** UX / HONESTY
- **Owner:** Grok
- **Source:** Live CLI audit 2026-08-27 — `preflight --summary` 8/**7**; in-context 0/0/0 next to thousands of pins; `Total Word Count` opaque. Series README `README-T312-T324-CLI-DOGFOOD.md`. Opportunity (d).
- **Depends on:** T286 ✅ Index/summary pin titles (hermetic); T220 ✅ summary envelope; T265 ✅ json sections; T263 H1 honesty; T241 ✅ optional JSON `next_step` for incomplete grants; T288 ✅ briefing vault-pin stanza (do **not** steal); T290 ✅ granted-empty list `next_step` (do **not** steal)
- **Blocks / feeds:** Daily at-a-glance. Does **not** populate governed stores. Does **not** fix Index `## Objective` (T286 R1-1). T325 is the `#230` Cursor leftover (F8 recency) — **not** this DoD.
- **Absorbs:** Audit 0/0/0 + word-count meaning; T286 closeout “summary still trains empty brain when Index is chrome”; T220 F30 human-label honesty
- **Not absorbed (DoD):** H2 pin→Approved; T286 Index MATCH / retrieval `preflight.rs`; growing `preflight.rs` beyond summary renderer helpers; clap 5; T313–T324 steal; T325 F8 recency
- **Research date:** 2026-08-28 (source HEAD `44520d8` T312 `#230`). Snapshot — **re-verify at execute**.
- **Ledger:** planning DOCS TX `ca5b1614-6849-416d-ad27-1d44a23198d7`. Series mint DOCS `a6d3c404-1d64-4cba-a743-d75ac16c74cd`. Implement starts a **FEATURE** TX on **go**.
- **Isolation:** Do **not** implement until **go**. Do **not** `cargo install`. Do **not** `policy bootstrap` extra grants. Do **not** pin production DECISIONs to the live vault. Do **not** rewrite `.env`. Summary path only; do **not** rewrite `--pretty` Session. Do **not** grow hotspot `project.rs` / `sync.rs` / `governed_common.rs`. Do **not** print or commit `AI_BRAINS_KEY`.

---

## 1. Objective

1. **0 in-context decisions is actionable.** When the budget-window marker scan counts `DECISION:` as **0**, `--summary` prints a copy-paste `next: ai-brains recall "what did we decide"` instead of a dead `In context decisions: 0` next to thousands of vault pins.
2. **Word count is named.** The human label is **`Budget window words:`** — the full preflight `context.word_count`, **not** the size of the summary banner. JSON key `word_count` stays.
3. **Counts stay honest.** Do **not** invent in-context decisions from vault pins (H2). Dual-model: pins via `recall`; Approved via `decision propose` / briefing. Summary is orientation (T170 D21).
4. **North star.** Capture independence: CLI overlay on existing dual counts. No new events. No hidden CoT. Agents who run `preflight --summary` must not conclude this repo has no decisions.

This unblocks daily at-a-glance: T214 already prints `Pinned memories: N`; T286 already puts tagged pins in Index **when they GLOB-or-TAGS into the window**; live Index item 1 is still `## Objective` (T286 R1-1), so the window substring stays **0**. T315 does not reopen Index SQL. It names the next command.

---

## 2. Live baseline (re-scan 2026-08-28)

### 2.1 Operator dogfood (this machine)

| Signal | Observation |
|--------|-------------|
| HEAD | `44520d8` `feat(retrieval): T312 recall rank v3 — authority-OR + verbose-Other (#230)`. Tree **CLEAN**. Branch `track/T315-preflight-governed-empty` (cut from `main`). `origin/main` = `44520d8`. |
| PATH `ai-brains.exe` | `C:\Users\RyanB\.cargo\bin\ai-brains.exe` **26,897,408** B; LastWriteTime **2026-08-27 8:21:55 PM**; `ai-brains 0.1.3`. Owner elevated install after T311 `#229`. **T312 is not on PATH.** Summary hole is **source + PATH**. **Do not `cargo install`.** Tests/manual AC use `cargo run` / hermetic. |
| `preflight --summary` | Scope `C:\dev\ai-brains` (`3581317d`). Pinned **4520** (volatile; T312 plan 4513). In-context hotspots/decisions/constraints **0/0/0**. `Total Word Count: 688`. Grants omitted (live **3 of 3**). Footer `Use --pretty or --format json for full context.` **No next-step.** |
| `preflight --summary --format json` | Pretty envelope. `pinned: 4520`, `in_context_*`: 0/0/0, `word_count: 792` (separate invocation; vault grew from this session’s recalls). **`next_step` omitted** (grants complete — T241 skip). No `grants_status`. |
| `preflight --pretty -m 1500` | Safety honest empty: `No in-context hotspots. next: ai-brains safety sync --dry-run` (T279). **Memory Index line 1: `## Objective -- just now`.** T286 R1-1 **still true**. That is why summary `DECISION:` scan is 0. |
| Last GitHub PR | [#230](https://github.com/Ryan-AI-Studios/AI-Brains/pull/230) T312. `mergedAt` **2026-08-28T02:35:31Z** (`gh pr list` **02:15:20Z** is `createdAt` — do not confuse). Inline Cursor Bugbot **1 medium** on `lexical.rs` F8 OR fill (no PreferRecency retry). **Does not fit T315.** **Mint T325.** Open PRs: **none**. |
| Ledger | 0 pending / 0 drift at scan (before this DOCS TX). Hotspot **#1** `project.rs` (3.749). `sync.rs` #2. `governed_common.rs` **#3** (3.423) — **do not grow.** CLI `preflight.rs` **#9** (1.987) — **this is the renderer; keep the delta small.** |
| `ISSUES.md` | **Does not exist.** |

### 2.2 Why 0/0/0 still trains “empty brain”

| Layer | Truth |
|-------|--------|
| Dual count already shipped | T214: vault `Pinned memories` is SQL COUNT (**4520**). In-context is a **window substring** of assembled `context.text`. Showing N next to 0 did **not** stop the audit from scoring Q=7. |
| T286 hermetic is green | Untagged + TAGS-envelope pins yield `in_context_decisions >= 1` when they **are** in the window (`preflight_summary_json.rs` AC5/AC6). |
| Live Index is still chrome | `--pretty -m 1500` item 1 = `## Objective -- just now`. Drain still breaks a pass when the first addable row’s **full** content exceeds `max_words` (T286 R1-1). No fitting in-scope pin entered pass-1 on this vault. **Not T315 DoD.** |
| Marker scan is honest | CLI `:884–888` `text.matches("DECISION:")` etc. If Index titles `## Objective`, the count **is** 0. Relabeling 0 as “pins exist” would mix vault COUNT into the in-context block (T214 F4 freeze). |
| JSON `next_step` exists | T241 F3: optional, `skip_serializing_if`, set only when discovery grants incomplete. Live 3 of 3 → key **omitted**. T315 **fills that same key** when grants are complete and decisions are 0. **No new required key.** |
| T290 already solved this class | Granted-empty lists print copy-paste `recall "what did we decide"` + `(Pinned: N)`. Summary already has `Pinned memories:` — do **not** duplicate `(Pinned: N)` in the next-step string. |
| T288 is not this surface | Briefing stanza is `## Vault pins (not Approved)`. Do **not** paste that into `--summary`. |
| Word-count opacity | Formatter `:796` `Total Word Count: {word_count}` from `context.word_count`. Struct comment `:53–54` already says budget-window, **not** summary size. CAPABILITIES T220 F30 names the JSON field; the **human label still lies by omission**. |

### 2.3 Code truth (opened)

| Item | Location | Notes |
|------|----------|-------|
| Human formatter | `cli/src/commands/preflight.rs` `format_preflight_summary_lines` `:772–800` | 9 args (T214 F4 / AC19). Banner, Scope, vault block, in-context block, **`Total Word Count:`**, blank, footer. **Change the word-count format string inside this fn. Do not add a 10th arg.** |
| Print | `print_summary` `:854–957` | Marker scan `:884–888`. T241 grants post-hoc `:890–949`. T264 span insert after constraints `:933–944`. T220 JSON branch `:893–918`. |
| JSON DTO | `PreflightSummaryJson` `:41–64` | T220 keys + T241 optional `grants_status` / `next_step` + T264 optional span. `word_count` comment already honest. |
| JSON builder | `build_preflight_summary_json` `:70–104` | Pure; `next_step: None`. T241 assignment is **after** the builder (`:908–912`). T315 assignment **after** T241, only if `next_step.is_none()`. |
| Grants helper | `format_grants_incomplete_line` `:109–119` | Complete (`>= 3`) returns `None`. **Do not change.** |
| Bootstrap SOOT | `governed_common.rs` `POLICY_BOOTSTRAP_SOOT_SHORT` `:160–161` | `next: run \`ai-brains policy bootstrap --dry-run\` then \`ai-brains policy bootstrap\``. JSON T241 uses this **exact** string. |
| Recall needle | `LIST_RECALL_QUERY` `:57` | `"what did we decide"`. **Reuse** in T315 SOOT. Do **not** call `format_authorized_empty_next` (that prefixes `Ungoverned vault search:` + `(Pinned: N)` — list overlay). |
| Units | `preflight.rs` `:1448–1524` | `Total Word Count: 100` assert. Arity-9 AC19 `:1371`. T241 post-hoc `:1395`. |
| Hermetic JSON | `tests/preflight_summary_json.rs` | T220 required keys; T286 AC5/AC6 `in_context_decisions >= 1`. **Stay green**; additive omit-`next_step` when decisions ≥ 1. |
| Retrieval Index | `retrieval/src/preflight.rs` | T274/T286 two-pass. **Do not edit.** |
| Pretty Safety empty | T279 | `next: ai-brains safety sync --dry-run` on **pretty**, not summary. **Do not steal.** |
| T180 full JSON | `PreflightContextResponse` | `{text, word_count, sections}`. **Do not add `next_step`.** |
| Hotspots | `project.rs` #1 / `sync.rs` #2 / `governed_common.rs` #3 / CLI `preflight.rs` #9 | Isolation. |

### 2.4 Dependency / standards research (2026-08-28) — snapshot; re-verify at execute

| Pin | Workspace / lock | Ecosystem (today) | Action |
|-----|------------------|-------------------|--------|
| `clap` | workspace **`4.5`** / lock **4.6.1** | crates.io **4.6.6** (2026-08-06). GitHub clap-rs latest **v4.6.6**. **No clap 5.** | **No bump.** No new flags. |
| `serde` / `serde_json` | workspace `1.0` | skip_serializing_if already on `next_step` | **No bump.** Reuse. |
| `rusqlite` | exact **0.40.2** | not this track | **No bump.** |
| rustc / edition | **1.95.0** / **2024** | — | Unchanged |
| nextest | **0.9.140** (AGENTS.md) | — | Unchanged |
| workspace | **0.1.3** | — | **No bump** |
| New crates | — | — | **Zero.** |

**Online / primary sources**

| Claim | Source | Takeaway |
|-------|--------|----------|
| Suggest the next command | [clig.dev](https://clig.dev/) *Ease of discovery* (fetched 2026-08-28): discoverable CLIs “suggest what command to run next” | Copy-paste `recall` on stdout of `--summary` (orientation, not an error). JSON `next_step` stays on stdout inside the object (T220 purity). |
| Saying (just) enough | clig.dev *Saying (just) enough* | One `next:` line. Do not dump briefing stanza / grant walls / Index dumps into summary. |
| Human-first; stdout is data | clig.dev *Output*: data on stdout; diagnostics on stderr | Human next-step is **stdout** (same as T279 pretty Safety `next:` and T241 grants line). JSON path: the string is a field, not a banner. Harness install chatter stays stderr on JSON (T220 F8). |
| Dual count honesty | T214 F4; CAPABILITIES dual-count row | Vault COUNT and window scan stay two blocks. Do not relabel `In context decisions` as vault totals. |
| Optional JSON keys | T241 `next_step` `skip_serializing_if`; PROTOCOL-COMPAT T220 row | Filling an existing optional key is **not** T180 required-key growth. Document the new **string source** (empty-decisions vs bootstrap). |
| Index is not FTS | Live `index_select_sql` | SQLite FTS5 BM25 / title weights are **N/A** for this track. |

**N/A:** SQLCipher page encrypt, schtasks, Windows service, llama.cpp `/health`, clap 5 (not this bump), FTS5 schema split, T307 reqwest/tower-http.

**Could not verify:** COUNT of in-scope leading-marker `DECISION:` rows in `3581317d` without vault SQL (do not print `AI_BRAINS_KEY`). Hermetic empty vs pin fixtures are the proof. Live Manual AC is `cargo run --summary` still 0 + next-step present.

**ledgerful / ai-brains:** `preflight --summary` 0/0/0 vs **4520** pins; `--pretty -m 1500` Index `1. ## Objective -- just now`; JSON omits `next_step`; `ledgerful ledger status --compact` 0 pending / 0 drift; `search "format_preflight_summary_lines"` → `preflight.rs:772` + units; `scan --impact` CLEAN at `44520d8`; hotspots `project.rs` #1 / `governed_common.rs` #3 / CLI `preflight.rs` #9. Semantic recall of this topic still returns review-track chrome — evidence of ranking, not SoT for the summary renderer.

---

## 3. Frozen decisions

| ID | Decision |
|----|----------|
| **F0 — Go gate** | Plan-only until user **go**. Planning is DOCS TX `ca5b1614`. Implement starts a **FEATURE** TX. |
| **F1 — Next-step trigger** | Human next-step + JSON `next_step` (when T241 did not set it) fire when **`decision_count == 0`**. Do **not** require hotspots==0 or constraints==0. T286 live was **5/0/0**; requiring 0/0/0 would miss that class. |
| **F2 — Exact human / JSON string** | SOOT: `next: ai-brains recall "what did we decide"` (needle = `LIST_RECALL_QUERY`). JSON `next_step` is the **same string** (T241 JSON already includes the `next:` prefix via `POLICY_BOOTSTRAP_SOOT_SHORT`). Length ≪ 140 (T263 F29). |
| **F3 — Reuse needle, not list overlay** | Import `LIST_RECALL_QUERY`. **Forbidden** to call `format_authorized_empty_next` / prefix `Ungoverned vault search:` / suffix `(Pinned: N)`. Summary already prints `Pinned memories:`. |
| **F4 — No H2 / no propose** | Do **not** name `decision propose`, `policy bootstrap` (except existing T241 grants line), or `migrate governed`. Dual-model stands. |
| **F5 — T241 wins JSON `next_step`** | If `grants_status` is `Some` (incomplete discovery), JSON `next_step` stays `POLICY_BOOTSTRAP_SOOT_SHORT`. T315 fills **only** when `next_step.is_none()`. Human T315 line still prints when `decision_count == 0` even if grants are incomplete (two remediators). Do **not** relocate the T241 grants append. |
| **F6 — 9-arg freeze** | `format_preflight_summary_lines` stays **9 arguments** (T214 AC19). Word-count **label** change is inside the fn. Next-step is **post-hoc** (T241/T264 class). |
| **F7 — Word-count human label** | Replace `Total Word Count:` with **`Budget window words:`**. JSON key `word_count` **frozen**. No `word_count_label` key. |
| **F8 — Insert point** | Insert the T315 line **after** the word-count line and **before** the blank + footer (`Use --pretty…`). Helper `insert_after_budget_window_line` (name flexible). If the word-count line is missing, insert before the footer; if no footer, append. |
| **F9 — T214 dual counts frozen** | Vault `Pinned memories` / `Active sessions` / in-context marker labels **unchanged**. Do not mix vault COUNT into `In context decisions`. |
| **F10 — Marker scan frozen** | `text.matches("DECISION:")` / `HOTSPOT:` / `CONSTRAINT:` stay. Do not switch to vault GLOB COUNT. |
| **F11 — Decline Index SQL** | **Do not** edit `retrieval/src/preflight.rs`, `index_pass1_glob_sql`, or Index titles. T286 R1-1 stays a residual until a later Index track. |
| **F12 — Decline pretty Session / Safety** | `--pretty` chrome, T279 Safety `next:`, T219 caps — **not** this DoD. |
| **F13 — T220 / T180 / T265** | Summary JSON required keys stay. Full non-summary JSON stays `{text, word_count, sections}`. No `sections` on summary. No new required key. Optional `next_step` already T241. |
| **F14 — T288 / T290 not stolen** | Briefing stanza and list overlay stay on those surfaces. |
| **F15 — Capture independence** | Overlay + existing SQL COUNTs + existing window scan. No models. No new events. |
| **F16 — Exit 0** | Success still **0**. Empty in-context is orientation, not an error. |
| **F17 — No new CLI flag** | No `--next` / `--pins` / `--explain-counts`. |
| **F18 — PATH** | Do not `cargo install` unless the user asks. Tests/manual AC use `cargo run` / hermetic. |
| **F19 — Live vault pin** | Do **not** pin production DECISIONs as implement SoT. Hermetic empty + pin fixtures are SoT. |
| **F20 — Pins / crates** | No clap 5, no rusqlite bump, no new crates, workspace **0.1.3**. |
| **F21 — last-PR Cursor `#230`** | One real Bugbot medium on T312 F8 Prefer-OR skipping PreferRecency. **Still true** on `lexical.rs:231–250` vs AND recency `:197–213`. **Does not fit T313–T324.** **Mint T325** (placeholder). **Not this DoD.** |
| **F22 — T325 not stolen** | Do not “quickly fix” F8 recency in this track. |
| **F23 — File growth** | Helpers + units in CLI `preflight.rs`. Hermetic adds in `preflight_summary_json.rs`. **Do not** grow `project.rs`, `sync.rs`, `governed_common.rs` (beyond the existing `LIST_RECALL_QUERY` import), retrieval `preflight.rs`, `pin.rs`. |
| **F24 — Decline leftover bag** | T240 F2 silent `.env`; T263 H2; clap 5; density floors; T307 Blocked; T308 floors; T316–T324; doctor 16th. |
| **F25 — Tests** | Naming `function_or_feature__condition__expected_result`. No `unwrap`/`expect`/`panic` in production. |
| **F26 — Cross-model** | CLI honesty FEATURE. After Phase-1 clean, run read-only `codex-review`. |
| **F27 — Debt file** | `conductor/ISSUES.md` does **not** exist. Deferrals → `conductor/deferred.md`. |
| **F28 — Docs** | CAPABILITIES summary rows: empty-decisions `next:`; `Budget window words:` parity with JSON `word_count`. CHANGELOG T315. PROTOCOL-COMPAT: optional `next_step` string sources = T241 bootstrap **or** T315 empty-decisions; no new required keys. |
| **F29 — PowerShell** | `;` not `&&`. |
| **F30 — Stay-green T286** | AC5/AC6 `in_context_decisions >= 1` stay. Those fixtures **omit** T315 `next_step` (decisions ≠ 0). |
| **F31 — Stay-green T241** | Incomplete-grants JSON `next_step` is still bootstrap. Human grants line still appends. |
| **F32 — Stay-green T220** | Required keys, pretty JSON, no banner, no `sections`, `word_count` = budget window. |
| **F33 — Global / none** | Same trigger (`decision_count == 0`). Span line (T264) stays global-only. |
| **F34 — Harness** | Do not print the T315 line inside harness summary. JSON path still omits harness stdout (T220 F8). |
| **F35 — Const site** | Define the SOOT format in `preflight.rs` next to `format_grants_incomplete_line` (reuse `LIST_RECALL_QUERY`). Do **not** add a second needle const. |
| **F36 — `--summary --pretty`** | Still the summary path (T220 F2). Next-step applies. |
| **F37 — Determinism** | Exact SOOT string; no timestamps in the line. |

---

## 4. Acceptance criteria

| AC | Proof |
|----|-------|
| **AC1** | Unit `format_summary_empty_decisions_next__zero__exact_soot`: `decision_count == 0` → `Some("next: ai-brains recall \"what did we decide\"")`. `decision_count == 1` → `None`. **Required red.** |
| **AC2** | `format_preflight_summary_lines__global__…` contains `Budget window words: 100` and does **not** contain `Total Word Count`. **Required red.** |
| **AC3** | `format_preflight_summary_lines__arity_nine_args` stays green (9 args). |
| **AC4** | Unit: given formatter output with `decision_count == 0`, post-hoc insert places the SOOT line **immediately after** `Budget window words:` and **before** the footer. `decision_count == 1` does not insert. **Required red.** |
| **AC5** | CLI hermetic empty vault `preflight --summary` stdout contains the SOOT line **and** `Budget window words:`; exit **0**; no `decision propose`. **Required red.** |
| **AC6** | Same vault `preflight --summary --format json`: parse OK; `next_step` == SOOT; no human banner; T220 required keys present; no `sections`. **Required red.** |
| **AC7** | T286 `preflight__summary_json_tagged_pin__in_context_decisions_nonzero` (and untagged AC5 twin) **stay green** **and** assert `next_step` omitted (or not the T315 SOOT). |
| **AC8** | Unit: envelope with T241 `grants_status` set keeps `next_step == POLICY_BOOTSTRAP_SOOT_SHORT` even when `decision_count == 0`. T315 does not overwrite. |
| **AC9** | `t180_c_preflight_json_keys__cli_format_json__compact_stable_keys` stays green (non-summary). |
| **AC10** | CAPABILITIES summary rows + CHANGELOG Unreleased T315 + PROTOCOL-COMPAT summary JSON note (optional `next_step` = bootstrap **or** empty-decisions). |
| **AC11** | Manual (source bin): `cargo run -q -p ai-brains-cli -- preflight --summary` on this vault still shows `In context decisions: 0` **and** the SOOT line **and** `Budget window words:`. PATH-behind is **not** a fail. **Do not** `cargo install`. |
| **AC12** | `git diff -- crates/ai-brains-retrieval/src/preflight.rs` empty. Index SQL untouched. |
| **AC13** | Human SOOT `chars().count() <= 140`. |
| **AC14** | Trigger unit: `hotspot_count == 5`, `constraint_count == 0`, `decision_count == 0` still inserts (do not require 0/0/0). |
| **AC15** | No new required JSON keys. `pinned` / `in_context_decisions` / `word_count` still present. |

---

## 5. Design notes

### 5.1 Why not “fix the 0” by stuffing a pin into the window

T286 already did that for hermetics. Live Index item 1 is still `## Objective` because pass-1 has no fitting in-scope pin (R1-1). Reopening Index SQL here would steal T286 and grow retrieval `preflight.rs`. Summary honesty is: 0 means the **window** has no `DECISION:` marker; `Pinned memories: 4520` means the **vault** has pins; `next:` names `recall`.

### 5.2 Why not `decision propose`

T263 H1: daily “what did we decide” is `recall` / `search`. `decision propose` creates governed Proposed rows; it does not promote vault pins (H2 declined). Naming propose on `--summary` would retrain agents onto the wrong product.

### 5.3 JSON `next_step` precedence

```text
build envelope (next_step = None)
if grants incomplete:
    grants_status = …
    next_step = POLICY_BOOTSTRAP_SOOT_SHORT   # T241
if next_step.is_none() && decision_count == 0:
    next_step = T315 SOOT
```

Human: T315 insert after word-count; T241 grants still append at end.

### 5.4 Word-count label

JSON already documents budget-window in the field comment and CAPABILITIES. The human string `Total Word Count` is what operators read. Rename to `Budget window words:` — no new JSON field.

### 5.5 Capture independence

No `MemoryPinned`, no governed events, no briefing packet mutation.

---

## 6. Non-goals

- H2 pin→Approved / nightly auto-propose
- T286 Index MATCH / envelope titles / retrieval `preflight.rs`
- `--pretty` Session reorder / Safety SQL (T279)
- T288 briefing stanza / T290 list overlay copy
- T180 required-key growth / `in_context_authority` key
- Growing the 9-arg formatter
- Growing `project.rs` / `sync.rs` / `governed_common.rs`
- clap 5 / pin bumps / new crates
- T313–T324 steal
- T325 F8 PreferRecency on OR-fill
- T307 Blocked dual tower-http
- Doctor 16th check
- `cargo install` / PATH as Complete
- Silent `.env` rewrite (T240 F2)

---

## 7. Verification plan

```powershell
# Red → green
cargo nextest run -p ai-brains-cli --lib preflight
cargo nextest run -p ai-brains-cli -E "test(preflight_summary) | test(format_preflight_summary) | test(format_summary_empty)"
cargo clippy -p ai-brains-cli --all-targets -- -D warnings

# Manual (source bin — PATH is pre-T312)
cargo run -q -p ai-brains-cli -- preflight --summary
cargo run -q -p ai-brains-cli -- preflight --summary --format json
# Do not cargo install; do not policy bootstrap; do not live pin

# Full gate
cargo fmt --check ; cargo clippy --workspace --all-targets -- -D warnings
cargo nextest run --workspace ; cargo deny check ; cargo audit
ledgerful verify --scope full
```

**Red first (on go):**

1. `format_summary_empty_decisions_next__zero__exact_soot` (AC1)
2. `format_preflight_summary_lines__global__scope_and_projects_and_in_context` label (AC2)
3. Insert helper AC4
4. CLI hermetic AC5/AC6

Then green the production path. Stay-green AC3/AC7/AC8/AC9/T220/T241/T286. Full gate before Completed.

---

## 8. Risk

| Risk | Mitigation |
|------|------------|
| Agents treat `next_step` as “run propose” | F4 exact recall SOOT; AC5 forbids `decision propose` |
| T241 bootstrap overwritten | F5 / AC8 |
| T286 pin hermetics grow a next-step | F30 / AC7 omit when decisions ≥ 1 |
| 9-arg arity creep | F6 / AC3 |
| Relabeling in-context 0 as vault pins | F9 / F10 — counts stay; next-step is separate |
| Growing hotspot `#9` | Small helpers only; no pretty rewrite |
| Index still Objective on live vault | F11 — honest; AC11 expects 0 + next |
| `#230` Cursor leftover silently dropped | F21 mint T325 |
| PATH-behind false AC fail | F18 `cargo run` / hermetic |

---

## 9. Deferred absorb/decline

| Item | Disposition |
|------|-------------|
| Audit preflight 0/0/0 + word count (Q=7) | **Absorb** F1/F2/F7 / AC1–AC6 / AC11 |
| T286 live Index `## Objective` (R1-1) | **Decline steal** — F11; residual stands; T315 is next-step honesty |
| T220 F30 word_count meaning vs human label | **Absorb** F7 / AC2 |
| T241 optional `next_step` | **Partial** — reuse key; F5 grants win |
| T288 / T290 granted-empty copy | **Decline steal** — F3 / F14 (reuse needle only) |
| T263 H2 / T240 F2 / clap 5 | **Decline** F4 / F24 / F20 |
| T313 / T314 / T316–T324 | **Not stolen** |
| T307 Blocked / T308 floors | **Not stolen** / **Decline** |
| last-PR Cursor `#230` F8 recency | **Mint T325** — F21 / F22 |
| last-PR `#229` empty | **Superseded** by `#230` |
| conductor/archive / cargo-audit allowlist | **Not related** |
| Pretty Session chrome (T286 residual) | **Decline** F12 |

---

## 10. Implement order (on go)

1. Phase 0 re-read `print_summary` `:854–957` + formatter `:772–800` + T241 JSON `:893–918`; rescan deferred; FEATURE TX  
2. Red AC1/AC2/AC4/AC5/AC6  
3. Green F2/F6/F7/F8 helpers; wire JSON F5; import `LIST_RECALL_QUERY`  
4. Stay-green AC3/AC7/AC8/AC9/AC12/T220/T241  
5. Docs F28 / AC10  
6. Manual AC11 → review → full gate → Complete  

---

## 11. Soft residuals (expected)

| Residual | Note |
|----------|------|
| Live Index still `## Objective` | T286 R1-1; F11 |
| PATH until `cargo install` | F18 |
| In-context still 0 after T315 | **By design** — next-step is the product |
| T325 F8 PreferRecency | Minted; not this DoD |
| T220 `harnesses[]` / `scope_line` / ValueEnum | T220 F22 — not this |
| `is-terminal` migrate | T249 F12 |

---

## 12. Touch map (expected)

| Site | Change |
|------|--------|
| `crates/ai-brains-cli/src/commands/preflight.rs` | Label F7; helpers F1/F2/F8; JSON F5; units AC1–AC4/AC8/AC14 |
| `crates/ai-brains-cli/tests/preflight_summary_json.rs` | AC5/AC6 hermetic; AC7 omit-next on pin fixtures |
| `Docs/CAPABILITIES.md` | Summary next-step + `Budget window words:` |
| `Docs/PROTOCOL-COMPAT.md` | Optional `next_step` string sources (T241 **or** T315) |
| `CHANGELOG.md` | T315 row |
| `main.rs` preflight `after_help` | Optional one sentence |
| retrieval `preflight.rs` / `project.rs` / `sync.rs` / `governed_common.rs` (except import) / contracts DTO | **No** |
| events / store / daemon | **None** |
