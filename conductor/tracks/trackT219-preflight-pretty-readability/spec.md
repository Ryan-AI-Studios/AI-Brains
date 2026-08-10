# T219 — Preflight pretty readability

- **Track ID:** T219-PreflightPrettyReadability
- **Phase:** Post-audit CLI quality series (T217–T232) — P2 after T218
- **Status:** 🔄 **In Progress** (implementing on branch `feat/T219-preflight-pretty-readability`)
- **Depends on:** T214 Scope SOOT ✅; T032 ANSI/dedup/condensation ✅; T216 `preview_line` role-strip SOOT ✅; T220 summary JSON path ✅ (orthogonal); T180 full JSON key freeze ✅
- **Blocks / feeds:** Operator trust that `--pretty` is scannable; T224 can reuse extracted role-strip helper; residual T228 is **recall** Scope (not this body)
- **Category:** UX / BUGFIX
- **Source:** Non-destructive CLI audit 2026-08-05 — preflight pretty usefulness **8** / **output quality 5**; live re-scan 2026-08-09
- **Deferred absorbed:** deferred.md “T219 pretty body wall”; series README T219 row; T214 soft residual F24 (Scope header on full preflight body); T220 F22/F23 “not T219”
- **Not absorbed:** Change marker **selection** / ranking policy in `build_legacy_preflight`; ledgerful-on-global (T214 F9); grow `PreflightContextResponse` keys (T180); T224 full recall/sync/forget strip (share helper only); T228 non-empty **recall** Scope; clap 5; MSI; pager DoD
- **Research date:** 2026-08-09 (live dogfood + word_budget root cause + clig.dev / clap pins / T216 strip SOOT)
- **AI fold-in:** 2026-08-09 — AI1 affirms F1–F15 core + four blind spots (token invariance, orphan headers, mid-line strip, summary/governed isolation). AI2 **M1–M3** accepted hard; **L1–L5** / **O1–O4** disposition **§14**.
- **Ledger:** TX `7c080cb1-f821-43d1-8f17-c7b6b2f75093` (BUGFIX) open until closeout

## 1. Objective

1. **Stop the single-line wall:** Full preflight body (`--pretty` / human / pretty format) must preserve section structure (newlines between sections and items), not a space-joined word stream.  
2. **Display-only role-prefix strip** on human pretty path: leading `USER:` / `ASSISTANT:` / `SYSTEM:` (T216 case-sensitive token SOOT). Stored vault content and JSON `text` may keep role labels where assembly still emits them — document honestly.  
3. **Scannable structure:** blank lines around `--- Section ---` headers; per-section item caps with **per-section next-step notices** (F9/F31 — not one generic phrase for all sections).  
4. **Scope header on full body** (absorb T214 F24 soft): first line(s) use same T207/T214 `format_scope_line` SOOT (`Scope: global` / `Scope: project=…` / `Scope: project=(none)` — **not** `Repository:` vocabulary).  
5. **Preserve agent path:** non-summary `--format json` remains **exactly** `{text, word_count}` compact (T180). Prefer **newline-preserving** `text` (same word_count semantics) so agents also get structure — not a second formatter. Truncation honesty per F2b.  
6. **Capture independence:** no models, embeddings, or graph required for pretty formatting.  
7. **Zero new production crates.**

## 2. Live baseline (re-scan 2026-08-09)

### 2.1 Dogfood

| Command | Observed |
|---------|----------|
| `preflight --summary` | Dual counts + Scope — OK (T214/T220) |
| `preflight --pretty -m 800` | **One continuous line** (~688 words): bearings + many CONSTRAINT + sessions + ASSISTANT: DECISION wall + Memory Index lines still prefixed `ASSISTANT:` |
| ASSISTANT: count (in body) | ~29 |
| CONSTRAINT: count | ~6 (jammed after header with no blank lines) |
| DECISION: count | ~28 |
| `preflight --format json` (no summary) | Compact 2-key; `text` also newline-collapsed (same budget helper) |
| TTY default human (no flags) | Same body as pretty |

### 2.2 Root cause (frozen)

```text
// crates/ai-brains-retrieval/src/word_budget.rs
pub fn trim_to_word_budget(input: &str, max_words: usize) -> String {
    input.split_whitespace().take(max_words).collect::<Vec<_>>().join(" ")
}
// Final assembly in build_legacy_preflight / governed retag:
//   trim_to_word_budget(&sections.join("\n\n"), max_words)
// → destroys ALL newlines → single-line wall for pretty AND JSON text.
```

Secondary readability debt (after newlines restored):

| Site | Issue |
|------|--------|
| Safety section | Strips only `ASSISTANT: ` prefix; CONSTRAINT blocks still dense; no per-item cap beyond SQL LIMIT 10 + 15% words |
| Session turns | Re-emits `{ROLE}: {content}` — noisy for humans |
| Memory Index | `truncate_index_summary(first_line)` keeps `ASSISTANT: DECISION:…` |
| CLI pretty path | `println!("{}", context.text)` — no Scope header, no post-process |

### 2.3 Touch map

| Site | Role |
|------|------|
| `ai-brains-retrieval/src/word_budget.rs` | **F1** newline-preserving trim; unit tests |
| `ai-brains-retrieval/src/preflight.rs` | Soft: safety/index may already look better after F1; optional assembly polish if free (not required if CLI pretty transform covers display) |
| `ai-brains-cli/src/commands/preflight.rs` | Pretty/human path: Scope + `format_preflight_pretty_body`; keep JSON path on raw `context.text` (post-F1) |
| `ai-brains-cli/src/commands/memory.rs` | Extract / re-export pure `strip_role_prefix` (or move shared helper); `preview_line` calls it |
| Soft: `commands/display_text.rs` (or `role_prefix.rs`) | Shared pure strip for T219 + T224 prep |
| Hermetic | `tests/preflight_pretty_readability.rs` (new) — multi-section vault; assert newlines + no leading ASSISTANT on display lines + Scope |
| Unit | word_budget preserve/blank/truncate; strip_role_prefix; pretty body caps |
| Docs | CAPABILITIES preflight pretty; CHANGELOG; soft skill one-liner |
| Protocol | `t180_c_preflight_json_keys` stays 2 keys; optional assert `text` contains `\n` when multi-section fixture |

### 2.4 Deps / pins (researched 2026-08-09)

| Item | Workspace / note |
|------|------------------|
| clap | Workspace **`4.5`** (resolved **4.6.1**); crates.io latest **4.6.6** — **no bump** DoD |
| is-terminal | **0.4.17** (final of 0.4); soft residual → `std::io::IsTerminal` — not T219 DoD |
| serde / serde_json | **1.0** — no DTO growth on `PreflightContextResponse` |
| strip-ansi-escapes | Existing (T032) — no change |
| Zero new crates | No `comfy-table`, pager crates, or CLI framework swap |

### 2.5 Online / product research

| Finding | Application |
|---------|-------------|
| [clig.dev](https://clig.dev/) — human-readable paramount; say (just) enough | Restore structure; cap dense sections; keep JSON for machines |
| clig.dev — suggest next step | **Per-section** next command (F31): safety → `memory list`; index → `recall`; sessions → plain “more turns/sessions” |
| clig.dev — changing human output OK; machine stay stable | Pretty body may change freely; T180 keys frozen |
| T216 F9 role strip | Case-sensitive leading `USER:`/`ASSISTANT:`/`SYSTEM:` + trim; not mid-line |
| T032 | ANSI already stripped; do not re-open condensation algorithm |
| T170 D21 | Summary still not governed authority; full pretty is orientation + memory surface |

## 3. Frozen decisions (F1–F42)

| ID | Decision |
|----|----------|
| **F1 — Root fix (hard)** | Change `trim_to_word_budget` to **preserve newlines** while still limiting to `max_words` via `split_whitespace` word counting. Blank lines between sections (`\n\n`) must survive when budget allows. **word_count** remains `split_whitespace().count()` (unchanged semantics). Cumulative `used` counter; stop immediately when `used >= max_words` so **`word_count(&result) <= max_words` always** (AI1 blind #1 / O4). |
| **F2 — Trim algorithm (frozen)** | Walk lines via `input.split('\n')` then **`line.trim_end_matches('\r')`** (L3 CRLF). For non-empty lines, take whitespace tokens until budget exhausted; join on-line tokens with single spaces; emit `\n` between original lines; preserve empty lines (blank paragraphs) while budget remains. When budget hits mid-line, stop — do **not** invent section headers. **No** context-specific markers inside the pure helper (retrieval keeps `"... [Index Truncated]"` etc.). |
| **F2b — Truncation sentinel (M1 hard)** | When input had **more** whitespace words than `max_words` (truncation actually occurred), append a single trailing sentinel **`…`** on its own line if the output does not already end with `…` / `...`. Sentinel **does not** count toward `max_words` / `word_count` of content (document: content words ≤ max; sentinel is chrome). Unit: over-budget ends with `…`; under-budget does **not** append sentinel. JSON `text` inherits this honesty (agents can detect cuts). |
| **F3 — JSON text benefits** | Non-summary JSON `text` = same post-budget string (newlines + F2b sentinel). **Do not** add pretty-only fields to `PreflightContextResponse`. word_count = content whitespace words (F32). **Also** improves governed markdown and `truncate_turn` partials (O1) — same helper. |
| **F4 — Pretty/human path only for display polish** | Role strip, Scope header, section caps, blank-line emphasis apply when `human_mode` (`pretty` \|\| format human/pretty \|\| TTY default human). **JSON path does not** run CLI display polish (F5). |
| **F5 — JSON no double-transform** | JSON stdout remains compact serialize of `{text, word_count}` only. Role prefixes that retrieval still embeds may remain in JSON `text` in v1 (honest docs); soft residual: optional strip inside retrieval assembly later. Caps / Scope chrome **never** appear on JSON path. |
| **F6 — Scope header on full body (T214 F24)** | Human/pretty full body **prefix** with `format_scope_line(...)` (T207/T214 SOOT: `Scope: global` / `Scope: project=<alias-or-name> (<uuid>)` / `Scope: project=<uuid>` / `Scope: project=(none)`) + blank line before body. **Reject** AI1 draft `Scope: Repository:<uuid>` vocabulary. **Do not** print summary dual-count block on full body. Under `--global`, always `Scope: global`. |
| **F6b — Alias lookup (L5)** | Resolve alias **only in CLI** `run()` / pretty branch — mirror `print_summary` (`get_project_by_id` + `format_scope_line`). **Never** re-resolve in `build_legacy_preflight` / retrieval. |
| **F7 — Role strip signature (M3 hard)** | `pub(crate) fn strip_role_prefix(line: &str) -> &str` — **borrow, no alloc**: if leading case-sensitive `USER:` / `ASSISTANT:` / `SYSTEM:`, return remainder after `trim_start` on the suffix; else return `line` (or trimmed line policy: match T216 — operate on already-trimmed first line). Mid-line / lowercase unchanged. Callers allocate only when needed (`preview_line` truncates to owned String). |
| **F8 — Helper location + converge** | Extract once (`commands/display_text.rs` or `memory.rs` + `pub(crate)`). **Both** `preview_line` and `content_has_tag` inline loops **must** call it (no dual SOOT). T224 imports later — **do not** implement recall/sync strip in T219. |
| **F9 — Section budgets (pretty path)** | Pure `format_preflight_pretty_body(text) -> String` (F29 constants; no heavy `PrettyOpts` plumbing unless `--compact` ships — L4): |
| | • Recognize section headers matching **only** legacy `--- … ---` (full-line or line-start). **Do not** treat `#` / `##` markdown as section headers (M2). |
| | • **Orphan headers (AI1 #2 hard):** if a `--- … ---` header would be emitted with **zero** content lines under it after caps/budget, **omit** that header (or drop trailing orphan). |
| | • **Safety / bearings:** cap items at **`PRETTY_SAFETY_MAX_ITEMS = 8`**; overflow → F31 safety notice. |
| | • **Sessions:** **`PRETTY_TURNS_PER_SESSION = 6`**, **`PRETTY_MAX_SESSIONS = 3`**; overflow → F31 session notices. |
| | • **Memory Index:** strip role prefix on numbered lines; **`PRETTY_INDEX_MAX = 15`**; overflow → F31 index notice. |
| | • **Most Recent Memories:** keep top-3 intent; strip role prefixes. |
| | Caps are **display-only** — no SQL/ranking change. |
| **F10 — Blank lines** | Ensure a blank line after each emitted `--- Section ---` header and between major sections when pretty formatting. |
| **F11 — Optional `--compact`** | Soft residual F30 only. If deferred, use **module `const` caps** directly — do **not** build flag + `PrettyOpts` machinery for a soft residual (L4). |
| **F12 — Summary path unchanged** | `--summary` human + JSON (T214/T220) **out of scope** for body rewrite. `options.summary` continues early-return via `print_summary` only (AI1 #4). |
| **F13 — Marker selection unchanged** | No change to which memories enter preflight, safety LIKE filters, dedup_hotspots, low-signal filters, or onboarding % budget — only **display** + **word_budget newline** behavior. |
| **F14 — Governed path (M2 hard)** | When governed, body is `#`/`##` markdown; **F1/F2b still apply**. Pretty Scope header still printed. **No** governed section caps in v1. Formatter **must not crash** and **must preserve** `#`/`##` lines (must not strip or re-bucket them as `--- … ---` content). Soft residual: governed-specific caps later. |
| **F15 — Capture independence** | Pretty formatting pure string ops + optional Scope SQL already used by summary — no models/graph. |
| **F16 — Zero new crates** | — |
| **F17 — High findings** | Leaving space-join trim; silent over-budget JSON with no F2b sentinel; claiming JSON shape changed; stripping stored vault content; changing marker selection; breaking T180 key count; mid-line role strip; mangling governed `##` via `---` regex; dual strip SOOT in memory.rs. |
| **F18 — Hermetic locks (legacy)** | Multi-section legacy vault: pretty multi-line + Scope + no leading `ASSISTANT:` on index/session; JSON 2 keys + `\n` in text. |
| **F19 — Unit locks** | word_budget preserve + truncate + F2b sentinel + CRLF + F32 invariant; `strip_role_prefix`; pretty caps + orphan header; pure formatter without vault I/O (O2). |
| **F20 — Docs** | CAPABILITIES: pretty readability; Scope SOOT; role strip display-only; per-section +N wording; JSON newline + F2b truncation honesty; governed F1 benefit one-liner; CHANGELOG. Soft skill one-liner. |
| **F21 — Determinism** | Pure formatters; no timestamps added by pretty layer. |
| **F22 — Soft residuals** | `--compact`; is-terminal migrate; clap 4.6 workspace pin; strip role in retrieval for JSON; `scope_display.rs`; pager; T224 consumers; `truncate_turn` double-newline cosmetic polish beyond unit lock (L1). |
| **F23 — Not T224/T228** | Full search-path strip is T224; non-empty **recall** Scope is T228. Preflight Scope on full body **is** this track (F6). |
| **F24 — Exit codes** | Unchanged success **0**. |
| **F25 — Review** | UX/BUGFIX; primary review required. Cross-model soft. |
| **F26 — Implement order** | Red word_budget (AC1/AC2/AC15/AC16) → Green F1/F2b → Red strip helper (AC9) → Green F7/F8 converge → pure pretty formatter units (AC6/AC14/orphan) → wire CLI Scope (F6b) → hermetic legacy AC3–5/7 + governed AC14 → docs → gate. Prefer pure unit over hermetic where possible (O2). |
| **F27 — Ledger TX** | On go: `ledgerful ledger start T219-preflight-pretty-readability --category UX` (or BUGFIX). |
| **F28 — Plan-only** | No production code until user **go**. |
| **F29 — Caps constants** | `pub(crate) const` in preflight module; formatter takes no required opts struct in v1. |
| **F30 — compact soft** | See F11. |
| **F31 — +N wording (L2/O3 hard)** | Plain ASCII, **per section** (clig.dev suggest-command): |
| | • Safety/bearings: `+N more safety entries — ai-brains memory list` |
| | • Memory Index: `+N more via recall` |
| | • Sessions (turns): `+N more turns in session` |
| | • Sessions (count): `+N more sessions` |
| | No emoji. |
| **F32 — word_count after F1** | Content `word_count(&text) <= max_words` always; F2b `…` excluded from content count (or document if counted — prefer **exclude** so AC10 stays ≤ max_words). |
| **F33 — Stdout purity** | Human pretty: Scope + body on stdout (env warnings stderr). JSON: one compact object. |
| **F34 — Partial safety strip today** | Retrieval already strips `ASSISTANT: ` on safety entries — keep; pretty path strip is defense-in-depth for sessions/index. |
| **F35 — Parallel-friendly** | Low conflict with T224 if shared helper lands first; avoid dual edits to `recall.rs` pretty. |
| **F36 — Series order** | After T218. Peers: T224 then graph/install polish. |
| **F37 — Orphan section headers** | See F9 bullet (AI1 #2). |
| **F38 — truncate_turn (L1 soft→unit)** | Soft residual: unit-lock `truncate_turn` post-F1 does not produce pathological `\n\n...` if free during F1 work; not ship-blocker if pre-existing cosmetic only. |
| **F39 — Mid-line strip freeze** | F7 only; no regex global replace of “assistant”/“user” (AI1 #3). |
| **F40 — Summary isolation** | F12; AI1 #4 affirmed. |
| **F41 — Research pins re-verified (AI2)** | clap crates.io 4.6.6 / lock 4.6.1; is-terminal 0.4.17; no bump DoD. |
| **F42 — AI fold-in** | See §14; plan-only until go. |

## 4. Acceptance criteria

| ID | Criterion |
|----|-----------|
| **AC1** | Unit: `trim_to_word_budget` multi-line under budget **preserves** `\n` / blank structure (not space-joined). |
| **AC2** | Unit: over-budget multi-line truncates with structure until cut; **not** single-line flatten. |
| **AC3** | Hermetic pretty (legacy multi-section): body multi-line; `\n` separates `--- ` header from content. |
| **AC4** | Hermetic pretty: includes T207/T214 `Scope:` vocabulary (not `Repository:`). |
| **AC5** | Hermetic pretty: no displayed Memory Index / session line begins with `ASSISTANT:`. |
| **AC6** | Unit/hermetic: over-cap sections emit F31 notices with correct N (safety wording ≠ index wording). |
| **AC7** | Non-summary JSON: exactly `text` + `word_count`; multi-section `text` contains `\n`. |
| **AC8** | Human `--summary` regression unchanged. |
| **AC9** | Unit: `strip_role_prefix` + `preview_line` + `content_has_tag` SOOT (leading strip; mid-line leave; lowercase leave). |
| **AC10** | Content word budget: `word_count` of budgeted body ≤ `max_words` (existing under-1500 spirit). |
| **AC11** | Docs: CAPABILITIES + CHANGELOG (incl. F2b + governed F1 note). |
| **AC12** | Full CI gate; zero new crates; capture-independent. |
| **AC13** | Manual dogfood: live `--pretty -m 800` multi-line + Scope; JSON 2-key with newlines. |
| **AC14** | Unit (or hermetic): fixture with `#`/`##` governed-style lines through pretty formatter — no crash; `##` lines preserved; Scope still prepended on human path (M2). |
| **AC15** | Unit F2b: over-budget output ends with `…` sentinel; under-budget does not. |
| **AC16** | Unit F1+F32: e.g. `trim_to_word_budget("a b c\n\nd e f", 3)` → `word_count(&result)==3` (content) and result contains `\n` (O4). |
| **AC17** | Unit CRLF: input with `\r\n` yields `\n` structure without stray `\r` tokens (L3). |
| **AC18** | Unit: orphan `--- Header ---` with zero following items is omitted (F37). |

## 5. Non-goals

- Changing which pins/hotspots/decisions are **selected** into preflight  
- Ledgerful under `--global`  
- Growing `PreflightContextResponse`  
- Implementing T224 recall/sync/forget strip (helper extract only)  
- T228 non-empty recall Scope  
- Forced pager / less  
- clap 5 / dependency bumps  
- Governed multi-project packet  
- Auto `--global`  

## 6. Verification plan

| Phase | Proof |
|-------|-------|
| Red | Unit: current `trim_to_word_budget("a\n\nb c", 10)` is `"a b c"` — assert desired `"a\n\nb c"` fails |
| Green F1/F2b | word_budget units AC1/AC2/AC15/AC16/AC17 + retrieval preflight tests |
| Red strip | AC9 dual-callers |
| Green strip | F7/F8 converge |
| Pure pretty | AC6/AC14/AC18 without vault |
| Hermetic | AC3–AC5/AC7 legacy; AC14 governed-style |
| Targeted | nextest retrieval word_budget/preflight + cli preflight + protocol_compat; clippy both |
| Manual | AC13 dogfood |
| Full gate | fmt, clippy workspace, nextest workspace, deny, audit, ledgerful verify |
| Review | `review.md`; soft cross-model |

## 7. Risks

| Risk | Mitigation |
|------|------------|
| Tests assert exact single-line body | Grep/update hermetic fixtures; F1 unit first |
| word_count drift / off-by-one | cumulative `used`; AC10/AC16 |
| Silent JSON mid-cut | F2b `…` sentinel + AC15 |
| Over-aggressive caps starve humans | Caps display-only; JSON full budget text; F31 next-step |
| Divergent strip SOOT vs T216 | Single `&str` helper; both callers; AC9 |
| Scope SQL failure | Mirror `print_summary` (F6b) |
| Governed `##` mangled by `---` logic | F14 + AC14; only match `---` headers |
| Orphan empty section headers | F37 + AC18 |
| PrettyOpts over-engineering | F11/F29 constants-first (L4) |

## 8. Coordination

- **T032:** ANSI/dedup history — do not re-open.  
- **T214:** dual summary + Scope SOOT; absorb F24 full-body Scope.  
- **T216:** `preview_line` + `content_has_tag` → shared strip.  
- **T220:** summary JSON orthogonal.  
- **T180:** full JSON keys frozen.  
- **T224:** consumer of strip helper later.  
- **T228:** recall non-empty Scope later.  
- **T170:** summary ≠ governed authority.  

## 9. Suggested implement snippet (guidance only)

```rust
// word_budget.rs — preserve newlines + F2b sentinel (sketch)
pub fn trim_to_word_budget(input: &str, max_words: usize) -> String {
    if max_words == 0 {
        return String::new();
    }
    let total_words = word_count(input);
    let mut out = String::new();
    let mut used = 0usize;
    let mut first_line = true;
    for line in input.split('\n') {
        let line = line.trim_end_matches('\r');
        if used >= max_words {
            break;
        }
        if !first_line {
            out.push('\n');
        }
        first_line = false;
        if line.is_empty() {
            continue;
        }
        let mut parts = Vec::new();
        for tok in line.split_whitespace() {
            if used >= max_words {
                break;
            }
            parts.push(tok);
            used += 1;
        }
        out.push_str(&parts.join(" "));
    }
    if total_words > max_words && !out.ends_with('…') && !out.ends_with("...") {
        if !out.is_empty() && !out.ends_with('\n') {
            out.push('\n');
        }
        out.push('…');
    }
    out
}

// display_text.rs / memory.rs
pub(crate) fn strip_role_prefix(line: &str) -> &str {
    for prefix in ["USER:", "ASSISTANT:", "SYSTEM:"] {
        if let Some(rest) = line.strip_prefix(prefix) {
            return rest.trim_start();
        }
    }
    line
}
```

```rust
// preflight.rs human_mode — alias lookup mirrors print_summary (F6b)
if human_mode {
    let name_alias = options.project_id.as_ref().and_then(|pid| {
        ctx.conn.get_project_by_id(pid).ok().flatten().map(|p| /* label SOOT */)
    });
    let scope = super::recall::format_scope_line(
        options.global,
        options.project_id.as_ref(),
        name_alias.as_ref(),
    );
    let body = format_preflight_pretty_body(&context.text); // const caps; no PrettyOpts required
    println!("{scope}\n\n{body}");
}
```

## 10. Series note

Suggested order after T218: **T219** (this) → **T224** (search strip using shared helper) → graph/install peers.

## 14. AI fold-in (2026-08-09)

Sources: `C:\dev\AI-review.md` — AI1 (architecture affirm + 4 blind spots) + AI2 (M1–M3 mediums, L1–L5, O1–O4).

| Item | Source | Disposition |
|------|--------|-------------|
| Root cause space-join + architecture diagram | AI1 | **Affirmed** — F1–F5 already plan |
| Token invariance `used >= max_words` | AI1 #1 | **Absorbed** F1 + **AC16** |
| Orphan section headers after cut/caps | AI1 #2 | **Absorbed** F9/F37 + **AC18** hard |
| Mid-line role strip risk | AI1 #3 | **Affirmed** F7/F39 (leading only) |
| Summary / governed isolation | AI1 #4 | **Affirmed** F12/F40; governed F14 expanded |
| AC1–AC13 table | AI1 | **Affirmed**; extended AC14–AC18 |
| Actionable items 1–7 (budget, strip, pretty, Scope, JSON, docs, tests) | AI1 | **Affirmed** implement order F26 |
| AI1 Scope example `Repository:` | AI1 draft | **Rejected** — keep T207/T214 `Scope: project=…` / `global` (F6) |
| **M1** JSON truncation honesty | AI2 Med | **Absorbed** **F2b** trailing `…` + docs + **AC15** |
| **M2** governed `##` vs `---` formatter | AI2 Med | **Absorbed** F14 + **AC14** hard |
| **M3** `strip_role_prefix` → `&str` + converge both loops | AI2 Med | **Absorbed** F7/F8 rewrite + **AC9** |
| **L1** truncate_turn `\n\n...` | AI2 Low | **Soft** F38 unit if free |
| **L2 / O3** per-section +N wording | AI2 Low/Opp | **Absorbed** **F31 hard** (safety → memory list) |
| **L3** CRLF | AI2 Low | **Absorbed** F2 `trim_end_matches('\r')` + **AC17** |
| **L4** PrettyOpts over-engineer | AI2 Low | **Absorbed** F11/F29 constants-first |
| **L5** Scope alias only in CLI | AI2 Low | **Absorbed** **F6b** |
| **O1** document governed + truncate_turn F1 | AI2 Opp | **Absorbed** F3/F20 |
| **O2** pure unit formatters | AI2 Opp | **Absorbed** F19/F26 |
| **O4** F1+F32 invariant unit | AI2 Opp | **Absorbed** **AC16** |
| Dep pins clap/is-terminal | AI2 | **Affirmed** F41 — no bumps |
| Verdict “go after M1–M3” | AI2 | **Accepted** — fold-in complete; still plan-only until user **go** |

**Rejected / not absorbed:** inventing `Scope: Repository:` labels; implementing T224 search paths now; hard DoD `--compact` / pager; counting F2b `…` as a content word (prefer exclude so AC10 stays clean).
