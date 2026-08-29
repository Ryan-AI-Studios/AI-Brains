# T316 — `memory list` preview + forget nudge

- **Track ID:** T316-MemoryListPreview
- **Status:** **Completed** (FEATURE TX `50c73816-3152-499e-bee9-1b5aeb7b0aec`)
- **Category:** UX
- **Owner:** Grok
- **Source:** Live CLI audit 2026-08-27 — `memory list` 6/**6**. Previews are raw first lines (`Let me verify the clap pin...` / `## Objective`). Trailing F36 forget nudge on stderr reads like an error after a successful table. Series README `README-T312-T324-CLI-DOGFOOD.md`.
- **Depends on:** T287 ✅ human prefer-fill ORDER (do **not** reopen JSON recency); T216 ✅ inventory + F36 stderr; T285 `first_contentful_line` / `is_session_chrome`; T224 `preview_line` inherit; T299 forgotten-empty remediator; T312 ATX chrome tokens
- **Blocks / feeds:** Daily inventory skim. Does **not** replace `recall` rank (T312) or list ORDER (T287).
- **Absorbs:** Audit preview + F36 stderr nudge; T216 F36 runtime stderr **supersede** (after_help stays)
- **Not absorbed (DoD):** T287 ORDER / JSON recency; T299 forgotten-empty `Pinned: N` + `next:`; T216 JSON keys / limit 50/200; forget match-preview budgets 100/80; `memory show <id>`; T325 F8 recency; T326 T320 pin-count leftover
- **Research date:** 2026-08-29 (plan-write product HEAD `d1c3bd3` T320 Completed note `#238`; T320 product `#237` `c3abe19`). Fold-in against `120bbfa` (this plan’s own docs commit; ahead **1** of `origin/main` = `d1c3bd3`). Snapshot — **re-verify at execute**.
- **AI fold-in:** 2026-08-29 `agy-review.md` + `opencode-review.md` (HEAD `120bbfa`). **Agy B 0 / M 0.** **OpenCode B 0 / M 0.** **Agree:** Agy m1 HEAD snapshot; OpenCode m1 walk-stop first-non-chrome (F1/F3 + AC19); OpenCode m2 named after_help hermetic (AC14); OpenCode O1 empty `classify_pin_kind` → Other. **Already:** Agy m2 F5/AC5 all-chrome fallback; Agy O1/O2/O3 F9 / F1–F2 / F3; OpenCode O3 T326 Phase 0 re-cite. **Partial:** OpenCode O2 inherit smoke — helper units lock inherit; decline extra briefing/graph hermetics (F14). Disposition **§13**.
- **Ledger:** planning DOCS TX `66b597f7-faf9-4f3e-bb06-6af72811bdc6`. Fold-in DOCS TX `69e50ba1-5c35-49d4-abb3-56f1ff6419c6`. Series mint DOCS `a6d3c404-1d64-4cba-a743-d75ac16c74cd`. Implement starts a **FEATURE** TX on **go**.
- **Isolation:** Do **not** implement until **go**. Do **not** `cargo install`. Do **not** grow hotspot `project.rs` / `sync.rs` / `forget.rs` **production** / `session_chrome.rs` / `ranking.rs` / `doctor.rs` / `governed_common.rs`. Chrome skip lives in `memory.rs` `preview_line` (inherit-only for forget/graph/briefing). Do **not** print or commit `AI_BRAINS_KEY`. Do **not** pin production DECISIONs as planning.

---

## 1. Objective

1. **Human preview is contentful.** After T287 envelope (`first_contentful_line`), skip a closed list of leading chrome lines (`## Objective`, ATX review headings, ` ```json `, `Let me …`) when a later non-chrome line exists within a walk cap. Do **not** retitle a dump as a `DECISION:` pin by searching the whole body.
2. **Forget nudge is not an error.** Drop T216 F36 `eprintln!` on nonempty human list. Success table ends at `Showing N of T`. after_help still documents forget/restore. Do **not** add a fake `next: ai-brains forget --memory-id <id> -f` (placeholder `<id>` is not copy-pasteable).
3. **Keep T287 ORDER.** Human prefer-fill stays. JSON `items[]` recency + keys stay T216. Preview **values** may skip chrome (same class as T287 TAGS skip).
4. **North star.** Capture independence: display only. No new events. No models. Operators who run `memory list --limit 5` must not conclude the command failed because stderr printed a forget hint, and must not see only `## Objective` when the stored body has a later contentful line.

This unblocks daily CLI: T287 mixed the first page when GLOB retain finds pins; this vault still recency-fills (`F32` / R1-1). Preview skip is the remaining skim honesty. F36 stderr is a Windows-first error-lookalike.

---

## 2. Live baseline (re-scan 2026-08-29)

### 2.1 Operator dogfood (this machine)

| Signal | Observation |
|--------|-------------|
| HEAD | Fold-in against plan-write `120bbfa` `docs(conductor): plan T316 memory list preview (chrome-skip, drop F36; mint T326)`. Product `src/` = T320 `#238` `d1c3bd3`. Tree **CLEAN** at fold-in. Branch `track/T316-memory-list-preview`. `origin/main` = `d1c3bd3` (ahead **1**). Plan-write snapshot was `d1c3bd3` / ahead **0** (Agy m1). |
| PATH `ai-brains.exe` | `C:\Users\RyanB\.cargo\bin\ai-brains.exe` **26,897,408** B; LastWriteTime **2026-08-27 8:21:55 PM**; `ai-brains 0.1.3`. **T263/T293 on PATH.** **T287 / T312 / T315 / T314 / T313 / T317 / T319 / T320 not.** List hole **is** on PATH **and** source. **Do not `cargo install`.** Tests/manual AC use hermetic bin / `cargo run`. |
| `preflight --summary` (PATH) | Pinned **4568** (plan start) / **4569** after this session’s ingest. In-context **0/0/0**. `Total Word Count: 728` (PATH-behind T315 `Budget window words:`). **Not this DoD.** |
| PATH `memory list --limit 5 --format human` | Five recency chrome/dump first lines: `## Objective`; `# Track Plan Review: T296…`; ` ```json `; audit-dump prose; `## Objective`. Footer `Showing 5 of 4569`. F36 stderr `Use ai-brains forget --memory-id <id> -f…` **interleaves after Scope / before `status=`** (stderr vs stdout buffering). Exit **0**. |
| Source `cargo run -p ai-brains-cli -- memory list --limit 5 --format human` | **Also recency**, not DECISION-first. Five just-now rows: `## Objective`; `T260 is drafted as a plan…`; T262 review dump; T287 F6 `preview_line` prose; T299 stay-green prose. F36 stderr same. T287 prefer-fill **did not** surface pins on this vault (R1-1 / F32 still live). |
| Source `memory list --format json --limit 1` | Keys T216: `api_version=1`, `scope=project`, `status=pinned`, `returned=1`, `more_available=true`, `limit=1`, `total=4569`. `items[0].preview` = `"## Objective"` (JSON recency F2). |
| `memory list --help` | after_help already names human prefer-fill / JSON recency / forgotten-empty next. **No** chrome-skip sentence yet. clap `--format` tokens `human`/`json` only (Family B default human). |
| Last GitHub PR | [#238](https://github.com/Ryan-AI-Studios/AI-Brains/pull/238) T320 conductor note. `mergedAt` **2026-08-29T03:18:19Z**. Issue comments **[]**. **last-PR product** [#237](https://github.com/Ryan-AI-Studios/AI-Brains/pull/237) T320 glance. `mergedAt` **2026-08-29T03:17:43Z**. Cursor Bugbot **1 medium** (`PinnedCountFailed` invents `pinned=0`) — **still true** on `status.rs:329–340` and `graph.rs:445–458`. **Does not fit T316.** **Mint T326.** `#230` Bugbot already **T325**. Open PRs: **none**. |
| Ledger | 0 pending / 0 drift at scan. This planning TX `66b597f7`. Hotspot **#1** `project.rs` (3.681) — **do not touch.** `sync.rs` **#2**. `governed_common.rs` **#3**. `forget.rs` **#5** (2.848) — **do not grow production.** `session_chrome.rs` **#6** — **do not edit** (import `is_session_chrome` only). |
| `ISSUES.md` | **Does not exist.** |

### 2.2 Why preview + nudge are still the hole

| Layer | Truth |
|-------|--------|
| T287 envelope is TAGS-only | `preview_line` (`memory.rs:53–66`) uses `first_contentful_line` then first-non-empty fallback. That skips `ASSISTANT: TAGS:` → `DECISION:`. It does **not** skip `## Objective` / `Let me verify…` / fences. Those **are** contentful. |
| T287 mix can still recency-fill | Human pinned prefer-fill is live in source (`run_inventory` `:228–265`). Live `3581317d` first page is still recency chrome (T287 conductor R1-1 / F32: GLOB+retain can empty pass-1 while older pins exist). **ORDER is not this DoD.** Preview skip still helps recency-fill rows + JSON recency `items[0]`. |
| JSON shares `preview_line` | `emit_list_json` `:441` calls the same helper. T287 F6: human **and** JSON list previews share it. Chrome skip **inherits** to JSON **values**. Keys frozen. |
| F36 stderr is T216 by design | `emit_list_human` `:556–559` `eprintln!("Use ai-brains forget --memory-id <id> -f…")`. Skip on empty / json / summary. T299 AC4 **requires** that stderr on nonempty forgotten. clig.dev: messages → stderr, data → stdout. **Windows-first conflict:** native stderr is the PowerShell error stream ([about_Redirection](https://learn.microsoft.com/en-us/powershell/module/microsoft.powershell.core/about/about_redirection); native stderr → ErrorRecord / `$Error` in Windows PowerShell 5.1 and ISE). Captured dogfood **interleaves** the hint into the table header. Repo precedent T267/T249/T299 moved *copy-pasteable* next-steps to **stdout** `next:`. F36’s `<id>` is **not** copy-pasteable → **drop**, do not fake `next:`. |
| Shared callers | `forget.rs:19–25` match/multi; `graph.rs:279` neighbor preview; `briefing.rs:77` vault-pin stanza (already Decision/Constraint retain — authority never skipped). Inherit-only. |

### 2.3 Code truth (opened)

| Item | Location | Notes |
|------|----------|-------|
| `preview_line` | `memory.rs:53–66` | Envelope + truncate. **Extend here** after `first_contentful_line`. |
| F36 stderr | `memory.rs:556–559` | Nonempty human only. **Delete this `eprintln!`.** |
| `emit_list_human` | `memory.rs:465–560` | Scope, table, `Showing N of T`, then F36. Empty forgotten T299 remediator **before** return (`:488–508`) — **keep**. |
| `emit_list_json` | `memory.rs:428–462` | Same `preview_line`. Nine envelope keys + item four keys. |
| `prefer_fill_authority` | `memory.rs:99–123` | **Do not change.** |
| `forgotten_empty_remediator` | `memory.rs:31–42` | T299 F26. **Do not change.** |
| clap `MemoryCommands::List` | `main.rs:3345–3372` | `--status` / `-l` / `--global` / `--format human\|json` / `--summary` / `--tag` / `--project-id`. **No new flag.** after_help `:3348`. |
| `first_contentful_line` | `ranking.rs:102–113` | Skip one role + `tags:` line. **Do not edit ranking.rs.** |
| `is_session_chrome` | `session_chrome.rs:24–58` | `## objective`, `# track plan review`, ` ```json `, ATX token set `{review, objective, onboarding, audit, ratings}`. **Import; do not edit.** |
| ATX tokens | `session_chrome.rs:21` | Closed set. Do not grow this track. |
| Forget inherit | `forget.rs:19–25` | `FORGET_PREVIEW_MAX=100` / `FORGET_MULTI_PREVIEW_MAX=80`. **Do not edit forget.rs production.** Units that call `preview_line` inherit skip. |
| Graph inherit | `graph.rs:279` | `preview_line(&c, 80)`. **Do not edit.** |
| Briefing inherit | `briefing.rs:77` | Vault-pin previews after Decision/Constraint retain. **Do not edit.** |
| T216 F36 hermetic | `tests/memory_list_inventory.rs:239–243` | Asserts stderr contains `forget --memory-id`. **Flip** to absent. |
| T299 AC4 | `tests/memory_list_inventory.rs:1410–1450` | Nonempty forgotten **requires** F36 stderr. **Update** with this track. |
| Envelope unit | `memory.rs:699–718` | TAGS→DECISION stay-green; TAGS-only fallback. |
| Help IA Daily | `help_ia.rs:11` | Already includes `status` (T320). **Do not restyle.** Additive memory after_help only. |
| CAPABILITIES | `Docs/CAPABILITIES.md:254–276` | Memory inventory T216/T287. Add chrome-skip + no runtime forget stderr. |
| Line counts | `memory.rs` **721** nonblank (plan-write `Measure-Object -Line`); `forget.rs` **269**. Snapshot only — **F25 80-net is phase diff vs go HEAD**. |
| Contracts | none | CLI-local JSON (T216 F22). PROTOCOL-COMPAT N/A. |

### 2.4 Dependency / standards research (2026-08-29)

| Pin | Workspace / lock | Action |
|-----|------------------|--------|
| `clap` | workspace **4.5** / lock **4.6.1** / crates.io **4.6.6** (2026-08-06; [docs.rs/clap/4.6.6](https://docs.rs/clap/4.6.6/clap/struct.Arg.html)) | **No bump.** No new flag. clap 5 **forbidden**. |
| `serde_json` | lock **1.0.150** | **No bump.** Preview string values only. |
| `rusqlite` | exact **0.40.2** | **No bump.** No SQL change. |
| `uuid` | ws `"1.13"` / lock **1.23.1** | **No bump.** |
| rustc / edition | **1.95.0** / **2024** | Unchanged. |
| workspace version | **0.1.3** | **No bump.** |
| New crates | — | **Zero.** No regex crate (closed prefixes + existing `is_session_chrome`). |

**CLI preview / stderr research (primary sources):**

| Source | What we take | What we decline |
|--------|----------------|-----------------|
| [clig.dev](https://clig.dev/) (fetched 2026-08-29) | Human-first; “Changing output for humans is usually OK”; JSON for scripts; suggest next command; stdout = data; stderr = messages/errors | Keeping F36 on stderr **because** clig says messaging→stderr — Windows-first PowerShell treats native stderr as the error stream; this repo already moved copy-paste `next:` to stdout (T267/T249/T299) |
| [about_Redirection](https://learn.microsoft.com/en-us/powershell/module/microsoft.powershell.core/about/about_redirection) (Learn, 2026-08-24) | Native command stderr is stream 2; `2>&1` merges; Windows PowerShell 5.1 / ISE wrap native stderr as ErrorRecord | Teaching operators to `2>$null` as the product fix |
| .NET 10 CLI stderr note (Learn) | Recommends PowerShell ≥7.2 so stderr does not set `$Error` | Requiring pwsh 7 as DoD — product is Windows-first including 5.1 |
| git `status` / T299 `next:` | Copy-pasteable next command on stdout after success | Fake `next: forget --memory-id <id>` |
| T285/T312 chrome detector | Reuse `is_session_chrome` for heading/fence/ATX | Growing `session_chrome.rs`; using dump-body search for buried `DECISION:` (would retitle dumps; T287 F31 dumps may fill later slots honestly) |

N/A-if-skipped: SQLCipher, schtasks, llama.cpp `/health`, FTS5 `bm25` (list is projection SQL, not FTS). clap `num_args` unused (no new flags).

**Could not verify:** exact COUNT of post-envelope `DECISION:` rows in `3581317d` without vault SQL (do not print `AI_BRAINS_KEY`). Hermetic unique dump body is SoT for skip; Manual `--limit 5` is pass-with-observed-data (T287 R1-1 may still recency-fill the first page).

**ledgerful / ai-brains:** `preflight --summary` Pinned **4568→4569** / in-context 0/0/0 / words **728** (PATH). `recall "memory list preview forget nudge F36"` lexical hits T287/T299 review dumps (PATH dump-first). `ledgerful ledger status --compact` 0 pending / 0 drift at scan; `search "preview_line"` → `memory.rs`, `forget.rs:19–25`, `display_text.rs`, `briefing.rs:77`; `scan --impact` CLEAN at `d1c3bd3`; hotspots as §2.1.

---

## 3. Frozen decisions

| ID | Decision |
|----|----------|
| **F0 — Go gate** | Plan-only until user **go**. Planning is DOCS. Implement starts a FEATURE TX. |
| **F1 — Chrome skip after envelope** | `preview_line` still starts with `first_contentful_line` (T287 F6). Then walk subsequent non-empty lines of the **same stored body** and take the first **non-chrome** line **whatever its kind**. Authority-ness only prevents skip-list eviction; it does **not** rank, search the body, or force-return the envelope line (OpenCode m1). Do **not** change `first_contentful_line` / `ranking.rs`. |
| **F2 — Closed skip list** | A candidate line is chrome when **any**: (a) `is_session_chrome(line)` (heading / ` ```json ` / ATX tokens — import, do not edit `session_chrome.rs`); (b) trimmed line starts with `` ``` ``; (c) ASCII-lower trimmed line starts with a closed agent prefix: `let me `, `now let me `, `i'll `, `i will `. **No regex crate.** Do **not** add `All non-destructive…` or free prose. |
| **F3 — Never skip authority** | If `classify_pin_kind` of a **one-line** candidate is Decision / Constraint / Hotspot, that line is **not chrome** (keep), even if a prefix overlaps. This is a skip-list exemption on the **candidate**, not “stop walking and return the envelope.” Chrome-before-authority (` ```json ` then `DECISION:`) still walks to the Decision (AC19). `classify_pin_kind("")` is `Other` — empty is never authority-elevated (OpenCode O1). Do **not** switch to `is_authority_pin_content` (drops Hotspot — T287 F5). |
| **F4 — Walk cap** | Skip at most **`PREVIEW_CHROME_WALK = 8`** chrome lines after the envelope line. Do **not** scan the rest of an 800-char dump for a buried `DECISION:` (that retitles dumps; T287 F31 / T312 F6 stay). |
| **F5 — All-chrome fallback** | If every walked line is chrome or empty, keep today’s first contentful / TAGS-only fallback (T287 F6 — **not** `""`, **not** `Untitled Memory`). |
| **F6 — Inherit `preview_line`** | forget match/multi, graph neighbor preview, briefing vault-pin stanza pick up skip automatically. Inherit SoT is `preview_line` units (AC1–AC7 / AC19), not extra briefing/graph hermetics (OpenCode O2 declined as DoD — F14). **Do not** edit `forget.rs` / `graph.rs` / `briefing.rs` production. **Do not** unify T216/T224 budgets (80 vs 100 vs 80). |
| **F7 — JSON keys freeze** | T216 F10 nine envelope keys + item `{memory_id, preview, updated_at, project_id}`. Preview **values** may skip chrome. **No** new keys (`chrome_skipped`, `title`, `next_step`). PROTOCOL-COMPAT N/A (CLI-local). |
| **F8 — T287 ORDER freeze** | Human pinned prefer-fill unchanged. JSON + store `list_memories` recency unchanged. Forgotten recency unchanged. **Do not** reopen GLOB / `list_authority_memories` (R1-1 is **not** this DoD). |
| **F9 — Drop F36 stderr** | Remove the nonempty-human `eprintln!` forget/restore hint. Empty / json / summary already skip it. **Supersedes** T216 F36 **runtime** stderr. after_help + CAPABILITIES keep the forget/restore **docs**. |
| **F10 — No fake `next:`** | Do **not** print `next: ai-brains forget --memory-id <id> -f`. Placeholder `<id>` is not copy-pasteable. T299 empty-forgotten `next: ai-brains memory list` **stays**. |
| **F11 — T299 remediator freeze** | `forgotten_empty_remediator` + empty human `Pinned: N` + last-line `next:` unchanged. JSON forgotten nine keys, no `next_step`. |
| **F12 — No new clap flag** | No `--preview-raw` / `--no-nudge` / `--chrome`. Silent display change (T287 F9 class). |
| **F13 — No `memory show`** | T319 deferred. List preview only. |
| **F14 — Isolation** | Edit `memory.rs` (+ CLI inventory hermetic + units). after_help one sentence. Docs. **Do not** grow `project.rs` / `sync.rs` / `forget.rs` production / `session_chrome.rs` / `ranking.rs` / `doctor.rs` / `governed_common.rs` / `query_store.rs`. |
| **F15 — Limit freeze** | Default **50**, max **200**. DoD hermetics may use `--limit 5`. |
| **F16 — Forgotten recency** | `--status forgotten` / `forget --list-forgotten` do not mix. Chrome skip still applies to those **previews** (display). |
| **F17 — Summary freeze** | `--summary` COUNT only. No F36. No preview skip (no previews). |
| **F18 — Capture independence** | Pure string walk + existing classifier. No events, models, embeddings, graph, ledgerful. |
| **F19 — Pins / crates** | No clap 5, no lock bumps, no new crates, workspace **0.1.3**. |
| **F20 — Standing declines** | T263 H2; T240 F2; T308 floors; T307 Blocked; csrf; KIND bump; FTS schema split. |
| **F21 — Tests** | Naming `function_or_feature__condition__expected_result`. No `unwrap`/`expect`/`panic` in production. `tempfile::tempdir` per hermetic. rstest `#[case]` for skip-list cases. |
| **F22 — Cross-model** | UX display change is FEATURE. After Phase-1 clean, run read-only `codex-review`. |
| **F23 — Debt file** | `conductor/ISSUES.md` does **not** exist. Deferrals → `conductor/deferred.md`. |
| **F24 — PowerShell** | `;` not `&&`. |
| **F25 — Line-count 80-net** | `memory.rs` production net **&lt;80 physical lines vs go HEAD**. Test blocks may exceed. F32-class: phase diff, not §2.3 snapshot. |
| **F26 — after_help** | One additive sentence: human/JSON previews skip leading session chrome / `Let me` lines when a later line exists; list does **not** print a forget hint (see `forget --help`). Keep T287 dual-truth sentence. |
| **F27 — T287 R1-1** | Live GLOB+retain empty → recency first page **stays possible**. Manual `--limit 5` is pass-with-observed-data. Hermetic chrome-skip SoT. Do **not** steal a GLOB/overfetch track. |
| **F28 — Stay-green** | T287 `prefer_fill_authority` rstest; TAGS→DECISION unit; TAGS-only fallback; T216 JSON keys; empty `No pinned memories.`; T299 empty remediator; `forget_match_preview` role-strip budgets; help_ia Daily string (already has `status`). |
| **F29 — last-PR Cursor** | `#237` Bugbot medium `PinnedCountFailed` → **T326** (not this DoD). `#238` empty. `#230` → **T325**. **No T327.** |
| **F30 — Decline peers** | T318 backup list; T321 safety sync; T322–T324; T325 F8 recency; T320 glance (Completed); T307 Blocked. |
| **F31 — PATH-behind** | Hermetic / `cargo run` SoT. Do not `cargo install`. |
| **F32 — Dual-truth** | Human + JSON preview **values** skip chrome; JSON **order** stays recency. after_help names both. |
| **F33 — Agent prefixes in `memory.rs`** | `PREVIEW_AGENT_CHROME_PREFIXES` const next to `PREVIEW_MAX_CHARS`. Do not add them to `session_chrome.rs` (ranking impact). |
| **F34 — T266 Family B** | Default stays human. Mix/skip apply to default + `--format human` + JSON preview strings. |
| **F35 — T216 F3 exit 2** | Missing project without `--global` → `fail_usage` exit **2**. Unchanged. |

---

## 4. Acceptance criteria

| ID | Criterion | Proof |
|----|-----------|--------|
| **AC1** | `## Objective\nWe decided SQLCipher` → preview contains `We decided` / `SQLCipher`, not `## Objective` | Unit `preview_line__session_chrome_heading__skips_to_body` |
| **AC2** | `Let me verify the clap pin\nCONSTRAINT: freeze ORDER` → preview starts with `CONSTRAINT:` | Unit `preview_line__let_me_verify__skips_to_next` |
| **AC3** | `ASSISTANT: TAGS: t316\nDECISION: needle` still `DECISION:` not `TAGS:` | Stay-green `preview_line__tags_envelope__decision_not_tags` |
| **AC4** | `ASSISTANT: TAGS: only` still non-empty `TAGS:` fallback | Stay-green `preview_line__tags_only__fallback_non_empty` |
| **AC5** | `## Objective` only → fallback `## Objective` (not empty) | Unit `preview_line__all_chrome__fallback_first_contentful` |
| **AC6** | Authority one-liner never skipped (`DECISION: I'll ship T316`) | Unit `preview_line__authority_line__never_skipped` |
| **AC7** | Walk cap: 8 chrome lines then a body line is kept; 9th chrome-only stays fallback | rstest `preview_line__walk_cap__eight` |
| **AC8** | Nonempty `memory list` human: stderr does **not** contain `forget --memory-id` / `forget --restore` | Hermetic flip of T216 F36 assert |
| **AC9** | Nonempty `forget --list-forgotten` / `--status forgotten`: same stderr absence (updates T299 AC4) | Hermetic `forget_list_forgotten__nonempty__omits_f36_stderr` |
| **AC10** | Empty forgotten still `Pinned: N` + `next: ai-brains memory list`; no F36 | Stay-green T299 AC1 |
| **AC11** | JSON keys exact T216 set; `items[0].preview` on a chrome+body fixture is the body line | Hermetic `memory_list__format_json__preview_skips_chrome` |
| **AC12** | JSON recency order unchanged (newest `updated_at` first) | Stay-green / fixture two rows |
| **AC13** | `prefer_fill_authority` rstest unchanged | Stay-green |
| **AC14** | after_help names chrome-skip + no runtime forget hint | Hermetic `memory_list_help__after_help__names_chrome_skip_and_no_forget_hint` (additive to stay-green `memory_list_help__mentions_human_authority_and_json_recency` `:1237`) |
| **AC19** | Fence then authority: body `` ```json `` then `DECISION: needle` → preview starts with `DECISION:` (first-non-chrome; F3 is not envelope-stop) | Unit `preview_line__fence_then_decision__keeps_decision` |
| **AC15** | Docs: CAPABILITIES inventory row; CHANGELOG; OPERATIONS one-liner | File grep |
| **AC16** | `forget.rs` / `graph.rs` / `briefing.rs` / `ranking.rs` / `session_chrome.rs` / `project.rs` / `sync.rs` production empty of behavior diff (inherit only) | `git diff -- crates/...` name-only |
| **AC17** | Manual `cargo run -p ai-brains-cli -- memory list --limit 5 --format human`: no F36 stderr; pass-with-observed-data on previews (T287 R1-1 may still recency-fill) | Recorded stdout/stderr |
| **AC18** | Exit 0 nonempty; exit 2 missing project; unknown `--format` clap InvalidValue | Stay-green + clap |

---

## 5. Design notes

### 5.1 `skip_leading_preview_chrome`

Pure helper in `memory.rs` (next to `preview_line`):

1. `contentful = first_contentful_line(content.trim_start())`.
2. Collect non-empty trimmed lines after the T285 envelope (same scan as `first_contentful_line`: strip one role, skip one `tags:` line, then remaining lines).
3. Walk at most `PREVIEW_CHROME_WALK` chrome lines (`preview_line_is_chrome`).
4. **First non-chrome wins** (OpenCode m1): return that line whatever its kind. Do **not** return the envelope line just because it is authority. If the envelope is chrome, keep walking until a non-chrome line or the cap.
5. Else fallback to step 1 / today’s first-non-empty + role strip (TAGS-only).
6. `truncate_preview_chars` unchanged.

`preview_line_is_chrome(line)`:

- `classify_pin_kind(line)` ∈ {Decision, Constraint, Hotspot} → **false** (keep). Empty `""` → `Other` (not authority).
- `is_session_chrome(line)` → true.
- `line.trim_start().starts_with("```")` → true.
- ASCII-lower starts with `PREVIEW_AGENT_CHROME_PREFIXES` → true.

No `unwrap`/`expect`/`panic`. Walk uses iterator + counter.

### 5.2 F36

Delete the `eprintln!` block in `emit_list_human`. Do not replace with stdout. after_help documents forget. T299 empty path never reached F36 (`return Ok(())` before it).

### 5.3 Dual-truth

Human table and JSON `preview` strings skip chrome. JSON `items[]` **order** stays recency. Human pinned **order** stays prefer-fill. after_help one sentence.

---

## 6. Non-goals

Reopening list ORDER / GLOB overfetch (T287 R1-1). Changing forget match SQL. `--status forgotten` table rewrite. `memory show`. JSON `next_step`. Growing `forget.rs` production. Editing `session_chrome.rs` / `ranking.rs`. T318 backup list. T321 safety. T322–T324. T325 F8 recency. T326 pin-count fail-open. clap 5. Pin→Approved (H2). Silent `.env`. Floor retune. `Untitled Memory` on list.

---

## 7. Verification plan (TDD)

**Red first** (must fail on today’s tree):

- `preview_line__session_chrome_heading__skips_to_body` — today’s preview is `## Objective`.
- `preview_line__let_me_verify__skips_to_next`
- `preview_line__all_chrome__fallback_first_contentful`
- `preview_line__authority_line__never_skipped`
- `preview_line__fence_then_decision__keeps_decision` (AC19)
- `preview_line__walk_cap__eight` (rstest)
- Hermetic `memory_list__nonempty__omits_f36_stderr` — today’s stderr contains F36.
- Hermetic JSON chrome+body preview (today `## Objective`).
- Hermetic `memory_list_help__after_help__names_chrome_skip_and_no_forget_hint` (AC14) — today’s after_help has T287 dual-truth only.

**Green:** implement skip helper + delete eprintln.

**Stay-green:** AC3/AC4/AC10/AC12/AC13/AC16/AC18.

**Manual AC17:** `cargo run`; PATH-behind not a fail.

---

## 8. Risk

| Risk | Mitigation |
|------|------------|
| Skip retitles dumps as pins | F4 walk cap; F3 skip-list exemption on the **candidate** (not envelope-stop); do not search whole body |
| `I'll` false-positive on a Decision | F3 classify before prefix |
| Chrome-before-authority mis-read | AC19 + F1 first-non-chrome |
| Briefing/graph inherit surprise | Authority retain already filters briefing; inherit SoT = `preview_line` units (F6); AC16 empty diff |
| T299 AC4 bitrot | AC9 updates that hermetic in the same commit |
| Live Manual still `## Objective` first | F27 honesty; hermetic SoT |
| 80-net | Keep helper small; tests in existing `memory.rs` `#[cfg(test)]` + hermetic file |

---

## 9. Deferred absorb/decline

| Item | Disposition |
|------|-------------|
| Audit `memory list` 6/6 preview + F36 stderr | **Absorb** F1–F10 / AC1–AC9 / AC17 |
| T216 F36 stderr next-step | **Supersede runtime** F9; docs/after_help stay |
| T287 ORDER / JSON recency | **Affirm freeze** F8 / F7 |
| T287 R1-1 live GLOB 0 / recency first page | **Partial** F27 — preview still DoD; ORDER not |
| T287 F6 envelope `preview_line` | **Affirm** then **extend** F1 |
| T299 empty `Pinned: N` + `next:` | **Affirm** F11 / AC10; **update** nonempty F36 assert AC9 |
| T224 forget budgets 100/80 | **Affirm** F6 inherit; do not unify |
| T319 no `memory show` | **Decline** F13 |
| T216 JSON keys / limit 50 | **Affirm** F7 / F15 |
| T318 / T321 / T322–T324 | **Not stolen** F30 |
| T325 F8 PreferRecency (`#230`) | **Not stolen** |
| T307 Blocked / T308 floors / H2 / clap 5 / T240 F2 | **Not stolen** / **Decline** F20 |
| last-PR Cursor `#238` | **N/A empty** |
| last-PR Cursor `#237` Bugbot `PinnedCountFailed` invents `pinned=0` | **Mint T326** — still true `status.rs:329–340` + `graph.rs:445–458`; doctor already skips (`doctor.rs:901`) |
| last-PR `#230` F8 recency | **T325** already Pending |
| OpenCode m1 walk-stop / AC19 | **Folded** F1/F3 / AC19 |
| OpenCode m2 after_help hermetic | **Folded** AC14 named test |
| OpenCode O1 empty classify | **Folded** F3 |
| OpenCode O2 inherit smoke | **Partial** F6 — helper units; decline extra briefing/graph hermetics |
| Agy m1 HEAD `d1c3bd3` vs `120bbfa` | **Folded** snapshot `120bbfa` / ahead **1** |
| Agy m2 all-chrome fallback | **Already** F5 / AC5 |
| DOCS TX (plan) | `66b597f7-faf9-4f3e-bb06-6af72811bdc6` |
| DOCS TX (fold-in) | `69e50ba1-5c35-49d4-abb3-56f1ff6419c6` |

---

## 10. Implement order (on go)

1. Phase 0 re-read `preview_line` / F36 / T287 mix / T299 remediator / hermetic F36 asserts; rescan deferred; FEATURE TX.
2. Red units + hermetics (must fail) including AC14 after_help + AC19 fence-then-Decision.
3. Green: `skip_leading_preview_chrome` + drop eprintln + after_help sentence.
4. Stay-green T287/T216/T299/T224; flip F36 asserts.
5. Docs CAPABILITIES / CHANGELOG / OPERATIONS.
6. Manual AC17; targeted clippy/nextest; FEATURE cross-model; full gate; publish (implement-track Phase 6). Never `git push origin main`.

Suggested series order after this plan: **T316 go** (daily inventory skim) or **T325** (F8 recency) or **T326** (glance pin-count honesty). Then T318 / T321. T307 stays Blocked.

---

## 11. Soft residuals

| Residual | Note |
|----------|------|
| T287 R1-1 recency first page when GLOB retain empty | Not this DoD; mint later only with owner |
| PATH until `cargo install` | F31 |
| Agent prefix set may grow (`Now let me check whether…`) | Closed set; extend only with evidence |
| JSON preview values change for chrome rows | F7 by design (not a key change) |
| T326 `PinnedCountFailed` | Separate track |

---

## 12. Touch map

| Path | Change |
|------|--------|
| `crates/ai-brains-cli/src/commands/memory.rs` | `preview_line` skip walk; drop F36 eprintln; units |
| `crates/ai-brains-cli/src/main.rs` | `MemoryCommands::List` after_help one sentence only |
| `crates/ai-brains-cli/tests/memory_list_inventory.rs` | Flip F36; JSON chrome preview; T299 AC4; AC14 after_help hermetic |
| `Docs/CAPABILITIES.md` | Inventory chrome-skip + no runtime forget stderr |
| `Docs/OPERATIONS.md` | One-liner |
| `CHANGELOG.md` | Unreleased |
| `conductor/conductor.md` | T316 Planned; T326 Pending placeholder |
| `conductor/deferred.md` | This plan + T326 mint |
| **Do not touch** | `forget.rs` production, `graph.rs`, `briefing.rs`, `ranking.rs`, `session_chrome.rs`, `query_store.rs`, `project.rs`, `sync.rs`, `doctor.rs`, `status.rs` (T326), retrieval rank, contracts |

---

## 13. AI fold-in disposition (2026-08-29)

Source: `agy-review.md` + `opencode-review.md` (HEAD `120bbfa`). **Agy B 0 / M 0.** **OpenCode B 0 / M 0.**

### Agy

| ID | Verdict | Action |
|----|---------|--------|
| **m1** HEAD `d1c3bd3` vs `120bbfa` | **Agree** | Snapshot fold-in `120bbfa` / ahead **1**; product `src/` = `d1c3bd3` (Agy m1 class) |
| **m2** all-chrome fallback empty string | **Already** | F5 / AC5 — fallback first contentful / TAGS, not `""` |
| **O1** drop F36 stderr | **Already** | F9 / AC8 / AC9 |
| **O2** chrome walk cap 8 | **Already** | F1 / F2 / F4 / AC1 / AC2 / AC7 |
| **O3** authority immunity | **Already** | F3 / AC6; **tightened** OpenCode m1 / AC19 |

### OpenCode

| ID | Verdict | Action |
|----|---------|--------|
| **m1** F3 walk-stop underspecified | **Agree** | F1/F3: walk returns **first non-chrome** whatever its kind; authority is skip-list exemption, not envelope-stop. **AC19** `preview_line__fence_then_decision__keeps_decision` |
| **m2** AC14 after_help unnamed | **Agree** | Named hermetic `memory_list_help__after_help__names_chrome_skip_and_no_forget_hint`. Stay-green existing T287 help test `:1237` |
| **O1** `classify_pin_kind("")` → Other | **Agree** | F3 / §5.1 clause |
| **O2** briefing/graph inherit smoke | **Partial** | F6: inherit SoT is `preview_line` units (AC1–AC7 / AC19). **Decline** extra briefing/graph hermetics as DoD (F14 — do not grow those files). Re-trigger: a caller forks a second preview helper |
| **O3** T326 line citations will drift | **Already** | T326 Phase 0 re-reads `status.rs` / `graph.rs` on `/plan-track T326` |

### Pins locked by fold-in

1. **F1/F3/AC19:** first-non-chrome wins; authority does not stop the envelope.
2. **AC14:** named after_help hermetic (chrome-skip + no runtime forget hint).
3. **F3:** `classify_pin_kind("") == Other`.
4. **F6:** inherit locked by `preview_line` units, not briefing/graph hermetics.
5. **HEAD snapshot:** fold-in `120bbfa` / ahead **1** of `origin/main` `d1c3bd3`.
6. **last-PR Cursor:** `#238` empty; `#237` → **T326**; `#230` → **T325**. **No T327.**

**Planning + fold-in 2026-08-29.** Still **plan-only until go**.
