# T288 — Granted-empty briefing must show vault pins exist (not H2)

- **Track ID:** T288-BriefingUsefulPins
- **Status:** ✅ **Completed**
- **Category:** FEATURE / UX / HONESTY
- **Owner:** Grok
- **Source:** Audit 2026-08-22 — `briefing project` **7/7**; friction “Approved empty vs 3647 pins”. Placeholder minted with T285–T300 (`76c4db9`). T263 ✅ H1 empty_authority + recall next (**H2 declined**). T275 ✅ grant-wall. T287 ✅ `list_authority_memories` (reuse; do not steal list mix).
- **Depends on:** T227 ✅ dual-model (pins ≠ authority arrays); T263 ✅ H1 (**H2 out**); T275 ✅ denied grant-wall; T214 ✅ `count_pinned_memories`; T287 ✅ GLOB-or-TAGS list + `preview_line` envelope
- **Blocks / feeds:** Operators who run `briefing project` after grants see that the vault has pins, labeled **not Approved**, and still `next: recall`. Personal deny `_None_` remains **T289**. Lists/progressive pin-count remains **T290**. Neighbors CLI **T293**.
- **Absorbs:** Placeholder problem text + Manual DoD; deferred.md “briefing granted-empty vs pins (no H2)”; T263 **F24** soft vault pin count (promoted to DoD); T227 dual-model honesty (authority arrays stay empty); live hole: Decisions/Conclusions `_None_` next to **~3822** pinned memories
- **Not absorbed (DoD):** T263 H2 pin→Approved / `DecisionProposed`; T289 personal deny `_None_`; T290 evidence/source/review/progressive `next_step` pin count; T293 neighbors; T294 leftover upsert; T240 F2; T170 D21 governed preflight as authority; clap 5 / rusqlite 0.40; `ProjectBriefingPacket` required keys
- **Research date:** 2026-08-23 (plan dogfood HEAD `f3f2485` T287 `#203`; product `src/` = T287 list mix + T263/T275 briefing; PATH **0.1.2** 2026-08-22 19:41 **without** T285/T286/T287 — briefing hole is in **source and PATH**)
- **AI fold-in:** 2026-08-23 `agy-review.md` + `opencode-review.md` (HEAD `ed28100`). **Agy B 0 / M 0.** **OpenCode B 0 / M 0.** **Agree:** Agy m1 fail-open `Repository:` parse (F14/AC17); OpenCode m1 COUNT source (F4/AC12); OpenCode m2 hermetic pin env (AC1); OpenCode m3 display-only privacy (F36); OpenCode O1 `VaultPinStanza` in `renderer.rs` (F11); OpenCode O2 fetch `limit=32` (F5). **Already:** Agy m2 Hotspot exclude (F5/AC16); Agy O1 PROTOCOL-COMPAT (F25/AC10); Agy O2 rstest 0-pin vs with-pin (AC14+AC4+AC1). **No declines of B/M.** Disposition **§13**.
- **Ledger:** planning DOCS TX `6bf1d41c-a2c6-4b86-8b4b-2dee14690363`. Fold-in DOCS TX `90e5e1d2-683d-4d62-baf1-4f821d423561`. Implement starts a **FEATURE** TX on **go**.
- **Isolation:** Do **not** `cargo install`. Do **not** pin production decisions to the live vault as implement (hermetic needle is SoT; Manual DoD unique canary is allowed on go). Do **not** rewrite `.env`. Do **not** live `policy bootstrap` extra grants (live already **3 of 3**). Do **not** `migrate governed`. Do **not** grow hotspot `project.rs` / `sync.rs` / `forget.rs` production / CLI `preflight.rs` / `governed_common.rs` / `personal.rs` / `session_chrome.rs` / `ranking.rs` / `pin.rs` write. Do **not** print or commit `AI_BRAINS_KEY`.

---

## 1. Objective

1. **Granted-empty briefing shows the vault is not empty.** `ai-brains briefing project --format human` when `denied: false` and both authority arrays are empty prints a **Vault pins (not Approved)** stanza: `Pinned: N` (project inventory COUNT) plus up to **3** leading-line `DECISION:` / `CONSTRAINT:` (incl. `INVARIANT:`) previews. Agents must not conclude this repo has no decisions because `_None_` sat next to thousands of pins.
2. **Dual-model stands (no H2).** `decisions[]` / `conclusions[]` / `constraints[]` stay Approved / Active-Confirmed only. Pin text must **not** appear under `## Decisions (current authority)` as if Approved. `empty_authority` warning kind stays. `BRIEFING_EMPTY_AUTHORITY_NEXT_STEP` stays T263 (recall; ≤140; one line).
3. **JSON scripts see the count too.** Non-TTY default is JSON (T227). CLI `--format json` adds optional `vault_pin_count` + `vault_pin_previews` (T180 additive; omit when not granted-empty). Daemon/HTTP `ProjectBriefingPacket` struct **unchanged**.
4. **North star.** Capture independence: read projection COUNT + T287 authority list + display overlay only. No new events. No hidden CoT. Pins remain `MemoryPinned` text. Governed authority stays a separate, deny-by-default product.

This unblocks daily briefing: T275 unlocked grants (live **3 of 3**); T263 made granted-empty *honest* (`next: recall`); the 2026-08-22 audit still scores **7/7** because honesty without a pin count still *feels* like an empty vault.

---

## 2. Live baseline (re-scan 2026-08-23)

### 2.1 Operator dogfood (this machine)

| Signal | Observation |
|--------|-------------|
| HEAD | `f3f2485` T287 squash `#203`. Tree **CLEAN**. `origin/main` = HEAD. |
| PATH `ai-brains` | `C:\Users\RyanB\.cargo\bin\ai-brains.exe` mtime **2026-08-22 19:41**, 25 139 712 bytes, **0.1.2**. Has T274. **Does not have T285/T286/T287.** Briefing hole is in **source + PATH** (T287 did not touch briefing). **Do not `cargo install`.** Tests/manual AC use `cargo run` / hermetic. |
| `preflight --summary` | Scope `C:\dev\ai-brains` (`3581317d`). Pinned **volatile 3820**. In-context **0/4/0**. Word **1467**. Grants omitted (3 of 3). |
| `memory list --summary` | `Pinned: 3822` / `Forgotten: 0` (volatile). Uses session-join `count_memories` (`query_store.rs:70–75`). Stanza COUNT is **`count_pinned_memories`** (`:707–711`, `mp.project_id = ?` only) — OpenCode fold-in ~**3821** on the same scope. **Do not** treat 3822 as AC12 equality. |
| `briefing project --format human` | **Allowed.** `## Decisions (current authority)` `_None_`; `## Conclusions` `_None_`; `_No current authority_`; `next: \`ai-brains recall\` / \`search\` for vault pins; typed Approved needs propose + approve`. **No `Pinned:`**. Exit **0**. **This is the 7/7 hole.** |
| `briefing project --format json` | `denied: false`, `decisions: []`, `conclusions: []`, `warnings[].kind: empty_authority`, no `vault_pin_count`. Agents (non-TTY) see this by default. |
| `briefing personal` | Denied + T263 Personal recall next. **T289** — do not steal. |
| Last GitHub PR | [#203](https://github.com/Ryan-AI-Studios/AI-Brains/pull/203) T287 (2026-08-23). `gh pr view --json comments,reviews` **empty**; issue comments **0**; `/comments` **[]**; `/reviews` **[]**. **last-PR Cursor: N/A.** Open PRs: Dependabot remotes only (`#61` rusqlite 0.40.2, `#62` chrono 0.4.45, `#59` tokio, `#60` thiserror, actions `#68–#72`). **No leftover to mint. No T301.** |
| Identity / doctor | ledgerful doctor 4 warn (legacy `.changeguard` / sig-pin / timings / :8081 unreachable). **0 pending / 0 drift** at scan. Hotspot **#1** `project.rs` (3.977) — **do not touch.** `sync.rs` #2. `forget.rs` #3. `governed_common.rs` **#5** — **do not grow.** `session_chrome.rs` #6. `personal.rs` **#7** — **do not touch** (T289). CLI `preflight.rs` **#8** — **do not grow.** Extend `briefing.rs` + `renderer.rs`. |
| `ISSUES.md` | **Does not exist.** |

### 2.2 Why granted-empty still trains “empty vault”

| Layer | Truth |
|-------|--------|
| Dual-model is working | T227 F3 / T263 F3: briefing authority = Approved + Active/Confirmed only. Live `decisions: []` is **correct**. Pins are not Approved. |
| T263 H1 shipped | `empty_authority` + `BRIEFING_EMPTY_AUTHORITY_NEXT_STEP` names `recall`. Agents still stop at `_None_`. |
| T263 F24 was soft | “SQL COUNT of pinned `DECISION:`/`CONSTRAINT:` prefixes on empty_authority. **Not DoD.** Must not enter authority arrays.” **This track promotes COUNT + samples to DoD**, still outside authority arrays. |
| T275 is not the hole | Live grants **3 of 3**; packet is `denied: false`. Grant-wall already hides `_None_` when denied. |
| T287 GLOB 0 residual | T287 closeout R1-1: live `3581317d` pass-1 GLOB matched **0** in-scope. **Must not** use GLOB COUNT as `Pinned: N` (Manual would fail). Inventory `count_pinned_memories` is SoT for the number. Samples from `list_authority_memories` + retain may be empty live — stanza still prints `Pinned: N`. |
| Non-TTY default JSON | T227: no `--format` + pipe → JSON. Human-only stanza would miss agents. **JSON overlay is DoD.** |

### 2.3 Code truth (opened)

| Item | Location | Notes |
|------|----------|-------|
| CLI run | `crates/ai-brains-cli/src/commands/briefing.rs` `run_project` `:67` | `build_project_briefing` then `render_project_markdown` / `serde_json::to_string_pretty`. **Overlay here.** `cli_principal` System `0xA1B2…`. |
| clap | `main.rs` `BriefingCommands::Project` `:1826–1836` | `--project-id` / `--max-words` default **1500** / `--dry-run` default **true** / `--format`. **No new flag.** |
| Markdown | `control-plane/.../briefings/renderer.rs` `render_project_markdown` `:66` | Decisions `_None_` `:105–106`; empty_authority footer `:137–141`. **Keep signature.** Add `render_project_markdown_with_vault_pins(packet, Option<&VaultPinStanza>)`. |
| Empty next | `BRIEFING_EMPTY_AUTHORITY_NEXT_STEP` `:36–37` | Frozen T263 F29 / AC14 (≤140, one line, contains `recall`). **Do not lengthen.** |
| Denied | `BRIEFING_DENIED_GRANT_WALL` `:22`; `empty_section_placeholder` `:198` | T275. Overlay **off** when `packet.denied`. |
| Packet build | `briefings/project.rs` | `empty_authority` warning `:178` when allowed-empty. Constraints from **governed conclusions** (`extract_constraints` `:672`) — **not** MemoryPinned. |
| DTO | `contracts/.../briefings.rs` `ProjectBriefingPacket` `:155` | Required keys frozen. `denied` / `denial_hint` optional. **Do not add fields** (every `empty_denied` / `empty_project` literal breaks — T287 analog). |
| Count | `QueryStore::count_pinned_memories` `store/src/lib.rs:136`; impl `:699–715` | T214. `Some` → `mp.project_id = ?` **only**. **Use this for `Pinned: N`.** Not `count_memories` session-join (`:70–75`) — OpenCode m1. |
| Samples | `QueryStore::list_authority_memories` `:60` | T287. GLOB-or-TAGS on `mp.content`. SQL `LIMIT ?` **before** Rust retain (`:99–107`). Fetch **32** (F5). **Reuse; no new store method.** |
| Preview | CLI `memory.rs` `preview_line` `:37`; envelope unit `:642` | T287. Import `pub(crate)`. Max **80**. AC9 inherit — do not reimplement. |
| Classifier | `ranking.rs` `PinKind` `:65` — `Constraint` / `Decision` / `Hotspot` / `Other`. `INVARIANT:` → `Constraint` (`:129–130`). | Preview retain **`== Decision \|\| == Constraint`** only (Agy m2). **Exclude Hotspot** (T279). **Do not edit ranking.rs.** |
| Pin CLI | `pin.rs` `run` `:19–34` | Hard-requires `AI_BRAINS_PROJECT_ID` **and** `AI_BRAINS_SESSION_ID`. Hermetic pin must use `hermetic_cmd` / `hermetic_cmd_with_ids` (`tests/common/mod.rs:164–180`). `DEFAULT_PROJECT` **equals** `governed_vault_pin_honesty.rs` `PROJECT` (`aaaaaaaa-…`). OpenCode m2. |
| Privacy | `MemoryListRow` `lib.rs:178–184` — **no** privacy field; `list_memory_rows` SELECT `:100–103` no privacy column | Stanza may include NeverInject/Sealed text. **Parity with `memory list`** (T216 F34 / T287 F38). Not a governed injection surface (F36). |
| T263 hermetic | `cli/tests/governed_vault_pin_honesty.rs` `briefing_project__granted_empty__empty_authority_names_recall` `:87` | Additive ACs. Existing asserts still hold with `Pinned: 0` extra. |
| T227 substance | `briefing_format_substance.rs` AC6 seeded Approved | Overlay **off** when authority non-empty — test stays green. |
| Governed preflight | `retrieval/src/preflight.rs` `:254` `render_project_markdown(packet)` | **Do not pass stanza. Do not grow CLI `preflight.rs`.** T170 D21. |
| JSON overlay analog | `governed_common.rs` `apply_authorized_empty_list_next` `:60` | T263 F8 / T243. **Do not grow** hotspot #5; briefing-local overlay. |
| Hotspots | `project.rs` #1 / `personal.rs` #7 / `preflight.rs` #8 | Isolation. |

### 2.4 Deps / pins (researched 2026-08-23 — snapshot, re-verify at execute)

| Item | Workspace / lock | crates.io / upstream (this pass) | Decision |
|------|------------------|----------------------------------|----------|
| clap | Cargo.toml `4.5`; lock **4.6.1** | crates.io **4.6.6**. GitHub clap-rs latest **v4.6.6** (2026-08-06). **No clap 5.** | **No bump** |
| rusqlite | **0.39.0** SQLCipher | crates.io **0.40.2**. Dependabot `#61`. | **No bump** |
| serde_json | lock **1.0.150** | crates.io **1.0.151** | **No bump** |
| chrono | lock **0.4.44** | crates.io **0.4.45** (`#62`) | **No bump** |
| tokio | lock **1.52.3** | crates.io **1.53.1** (`#59`) | **No bump** |
| thiserror | workspace **2.0** | crates.io **2.0.20** (`#60`) | **No bump** |
| uuid | lock **1.23.1** | not this track | **No bump** |
| rustc / nextest / workspace | **1.95.0** / **0.9.140** / **0.1.2** | — | Freeze |
| Zero new crates | Required | — | No regex / extra JSON crate |

### 2.5 Online / product research

| Finding | Application |
|---------|-------------|
| [clig.dev](https://clig.dev/) — **Human-first**; “Changing output for humans is usually OK”; scripts should use `--json` for stability | **F2** human stanza; **F3** JSON additive extras (not required-key churn). Dual-truth: human + CLI JSON; daemon packet frozen. |
| clig — “Make it easy to see the current state”; “suggest commands they can run next” | Stanza shows inventory COUNT (state); keep T263 `next: recall` (already shipped). Do **not** replace recall with seed-Approved. |
| clig — conversation after setup | Grants exist (T275). After setup, the next useful fact is “pins exist; they are not Approved.” |
| T180 `PROTOCOL-COMPAT.md` §8 / §3.1 | Prefer **additive optional** fields; N−1 ignore unknowns; never `deny_unknown_fields`. CLI overlay keys are extra JSON properties, **not** struct fields. |
| T227 dual-model / T263 F3 | **Never** scrape `MemoryPinned` into `decisions[]` / `conclusions[]`. T167 pins → Evidence on migrate only. |
| T243 / T263 F8 CLI overlay | Precedent: mutate serialized JSON `Value`, do not change DTOs. Copy locally in `briefing.rs`. |
| T283 / T287 human permute + JSON freeze | Analog: human *and* CLI JSON may add *display* facts; store/DTO contracts stay. |
| Nielsen empty-state class (clig empathy / “saying just enough”) | Distinguish **no access** (T275 deny) vs **empty product** (no Approved) vs **nonempty collection** (vault pins). T288 labels the third. |
| N/A | SQLCipher page encrypt, schtasks, llama.cpp `/health`, FTS5 `bm25(title,body)` (COUNT is projection SQL), clap 5 (not released). |

**Could not verify:** exact live GLOB-authority row count in `3581317d` without vault SQL (do not print `AI_BRAINS_KEY`). T287 R1-1 said **0** at closeout; inventory COUNT **3822** is the Manual number. Hermetic unique needle is SoT for sample lines.

**ledgerful / ai-brains:** `preflight --summary` Pinned **3820** / in-context **0/4/0** / word **1467**. `briefing project --format human` granted-empty `_None_` + recall next. `ledgerful ledger status --compact` 0 pending / 0 drift; `search "render_project_markdown"` → `renderer.rs` + `briefing.rs:11` + `preflight.rs:254` + CP tests; `scan --impact` CLEAN at `f3f2485`; hotspots as §2.1. Semantic recall of “T263 F24” still returns T263 plan-audit chrome — live src is SoT.

---

## 3. Frozen decisions

| ID | Decision |
|----|----------|
| **F0 — Go gate** | Plan-only until user **go**. Planning is DOCS. Implement starts a **FEATURE** TX. |
| **F1 — Dual-model / no H2** | Never inject pin text, pin counts, or `MemoryPinned` rows into `decisions[]` / `conclusions[]` / `constraints[]`. `empty_authority` kind stays. No `classify_legacy` / `decision propose` / auto-approve / live `migrate governed`. T227 F3 / T263 F3 / F11 stand. |
| **F2 — Human stanza (granted-empty only)** | When `!packet.denied && packet.decisions.is_empty() && packet.conclusions.is_empty()`: after `BRIEFING_EMPTY_AUTHORITY_NOTICE` + `NEXT_STEP`, before `## Constraints` / Warnings, emit heading `## Vault pins (not Approved)`, then `Pinned: {n}`, then up to 3 `- {preview}` lines. `## Decisions (current authority)` stays `_None_`. |
| **F3 — CLI JSON overlay** | Same gate: after `to_string_pretty(&packet)`, parse `Value`, insert `vault_pin_count` (u64) and `vault_pin_previews` (array of strings, max 3). **E1:** omit both keys when overlay off (denied / authority non-empty / collect fail-open). Granted-empty **0 pins:** emit `vault_pin_count: 0` and `vault_pin_previews: []` (never `null`). Required packet keys unchanged. |
| **F4 — Count is inventory, not GLOB** | `n` = `count_pinned_memories(Some(&project_id))` (T214; `mp.project_id = ?` only, `query_store.rs:707–711`). Includes harness ingest. **Do not** COUNT GLOB authority rows as `Pinned: N` (T287 R1-1 live GLOB 0). **Do not** switch to `count_memories` session-join (`:70–75`) just to match `memory list --summary` (OpenCode m1: live ~1-off). AC12 is **nonzero**, not equality to 3822. |
| **F5 — Previews are leading-line Decision/Constraint** | `list_authority_memories` (T287) with `status=pinned`, project scope, `tag=None`, **`limit = 32`** (SQL LIMIT before retain — OpenCode O2; 8 newest can be Hotspot/TAGS-only). Then retain `classify_pin_kind == Decision \|\| == Constraint` (Invariant maps to Constraint; Agy m2 explicit). **Exclude Hotspot and Other.** Cap **3**. `preview_line(..., 80)` (T287 envelope). Empty after retain → human `_No leading-line DECISION/CONSTRAINT samples in this scope._` when `n > 0`; omit that line when `n == 0`. |
| **F6 — Next-step const frozen** | Do **not** edit `BRIEFING_EMPTY_AUTHORITY_NEXT_STEP` / `NOTICE` / denied consts / Personal consts. T263 AC1/AC14 stay green. Stanza is **extra lines**, not a 140-char squeeze. |
| **F7 — Denied path frozen** | Overlay **off**. T275 grant-wall + hidden placeholder stay. No `Pinned:` on denied markdown. No overlay keys on denied JSON. |
| **F8 — Personal is T289** | Do not edit `personal.rs` / `render_personal_markdown` Preferences `_None_`. |
| **F9 — No new clap flag** | No `--pins` / `--include-vault` / `--authority`. Silent overlay on granted-empty. |
| **F10 — DTO freeze** | **Do not** add fields to `ProjectBriefingPacket` / `BriefingWarningDto`. No new warning kind `vault_pins`. Daemon/HTTP JSON stays today’s serde of the struct. |
| **F11 — Renderer signature** | `render_project_markdown(packet)` **unchanged** (preflight + existing units). Define **`VaultPinStanza { count: u64, previews: Vec<String> }` in `renderer.rs`** (OpenCode O1); re-export via `briefings/mod.rs` + `lib.rs`. New `render_project_markdown_with_vault_pins(packet, Option<&VaultPinStanza>)`. `None` ≡ today’s render. CLI human uses the new fn. |
| **F12 — File growth** | Overlay collect + JSON mutate in `briefing.rs` (or `briefing_vault_pins.rs` if `briefing.rs` would grow ≥80 net — prefer **same file** first). Stanza format + heading const in `renderer.rs`. Hermetic additive in `governed_vault_pin_honesty.rs`. **Do not** grow `governed_common.rs`, `preflight.rs`, `personal.rs`, `project.rs`, `query_store.rs`, `session_chrome.rs`, `ranking.rs`, `pin.rs` write, `.github/workflows/ci.yml`. |
| **F13 — Fail-open collect** | Scope parse / COUNT / list errors → overlay **off** (today’s packet). Do not fail the briefing command. No `unwrap`/`expect`/`panic` in production. |
| **F14 — Scope from packet** | Parse `packet.scope.scope_key` with **`strip_prefix("Repository:")` only** (exact; T226). Then `ProjectId::from_str` (`ids.rs:38` → `Uuid::parse_str`). `Personal:` / no prefix / empty / uuid `Err` → overlay **off**. Overlay path must **not** `?` parse onto `run_project`’s `Result` (Agy m1; F13). Do **not** re-resolve cwd. `--project-id` already baked into the packet. |
| **F15 — Reuse T287 store** | Call existing `list_authority_memories` + `count_pinned_memories`. **No** new `QueryStore` method. **No** `MemoryListFilter` field. **No** shared GLOB helper. **No** USER/SYSTEM TAGS GLOB (T285 F7 / T287 F29). |
| **F16 — last-PR Cursor** | #203 empty → **N/A**. Dependabot remotes not this track. **No T301.** |
| **F17 — PATH** | Do not `cargo install` unless the user asks. Tests/manual AC use `cargo run` / hermetic. |
| **F18 — Live vault pin** | Do **not** pin production DECISIONs as implement. Hermetic unique needle is SoT for samples. Manual unique canary allowed on go. |
| **F19 — Capture independence** | COUNT + list + formatters only. No models, embeddings, graph, ledgerful, new events. `--dry-run` default true stays. |
| **F20 — Tests** | Naming `function_or_feature__condition__expected_result`. `tempfile::tempdir` per hermetic. **AC14 required rstest `#[case]`** for overlay-gate (denied / nonempty / empty). **AC17 required** fail-open scope parse. Hermetic pin: `hermetic_cmd` / `hermetic_cmd_with_ids` so both env vars are set (OpenCode m2). |
| **F21 — Cross-model** | FEATURE (operator remediator + additive CLI JSON). After Phase-1 clean, run read-only `codex-review`. |
| **F22 — Debt file** | `conductor/ISSUES.md` does **not** exist. Deferrals → `conductor/deferred.md`. |
| **F23 — Decline peers** | T289 personal; T290 lists/progressive; T293 neighbors; T294 leftover upsert; T287 list mix (Completed). |
| **F24 — Decline H2 / F2 / pins** | T263 H2; T240 F2; clap 5; rusqlite 0.40; chrono 0.4.45; no new crates; workspace **0.1.2**. |
| **F25 — Docs** | CAPABILITIES dual-model row: granted-empty CLI stanza + overlay keys; authority arrays still empty; stanza is **display-only** (not injection). CHANGELOG T288. `briefing project` after_help one sentence. OPERATIONS one-liner. PROTOCOL-COMPAT briefings row: CLI `vault_pin_count` / `vault_pin_previews` optional extras on granted-empty; daemon/HTTP DTO **unaugmented** (Agy O1). |
| **F26 — PowerShell** | `;` not `&&`. |
| **F27 — Governed preflight freeze** | `retrieval/src/preflight.rs` keeps `render_project_markdown(packet)` with no stanza. T170 D21: `--summary` is not governed authority. |
| **F28 — Existing tests stay green** | T263 granted-empty / denied / AC14; T275 no `_None_` denied; T227 AC6 substance; T227 unknown format exit 2; renderer empty_authority units. Overlay is additive (`Pinned: 0` / extra JSON keys do not break those asserts). |
| **F29 — Daemon/HTTP freeze** | Soft T263 F25 analog. CLI is DoD. Do not overlay in `ai-brainsd` / HTTP routes this track. |
| **F30 — Heading SOOT** | Exact `## Vault pins (not Approved)` (const `BRIEFING_VAULT_PINS_HEADING`). Manual DoD “not Approved” / “vault pins” both satisfied by the heading. Count line is `Pinned: {n}` only (no second “not Approved” essay). |
| **F31 — T266 Family B** | Default TTY markdown / non-TTY JSON unchanged. Overlay applies to both markdown (new render) and JSON (Value). |
| **F32 — Chrome-only samples** | `n > 0` and 0 Decision/Constraint previews is honest (T287 R1-1 class). Do not fabricate `DECISION:` lines from `## Objective`. |
| **F33 — `max_words` budget** | Overlay is **after** CP `apply_budget`. Do not re-run budget on the stanza (a few lines). Do not drop empty_authority next to keep the stanza. |
| **F34 — Preview uniqueness** | Same `memory_id` must not appear twice in the 3. Preserve `list_authority_memories` order (newest first, T287 ORDER freeze). |
| **F35 — Isolation hotspots** | Do not edit `project.rs` / `sync.rs` / `forget.rs` production / `context.rs` / `claude_hook.rs` / `codex_hook.rs`. |
| **F36 — Privacy display parity** | Stanza reuses `list_authority_memories` (no privacy column). NeverInject/Sealed pins **may** preview — same as `memory list` (T216 F34 / T287 F38). **Do not** add `is_injectable_privacy` as DoD (OpenCode m3). Not a governed injection surface. Re-trigger: owner asks briefing samples to be injection-safe. |

---

## 4. Acceptance criteria

| AC | Proof |
|----|-------|
| **AC1** | CLI hermetic: discovery grants + pin via **`common::hermetic_cmd(&vault)`** (or `hermetic_cmd_with_ids(&vault, PROJECT, DEFAULT_SESSION)` — OpenCode m2; `pin.rs:28–34` requires both env vars; `PROJECT` == `DEFAULT_PROJECT` `aaaaaaaa-…`) `--tag t288 -- "DECISION: {needle}"` → `briefing project --format human --project-id {PROJECT}` stdout contains `## Vault pins (not Approved)`, `Pinned:` with a **nonzero** digit, `{needle}` (or `DECISION:`), `recall`; `## Decisions (current authority)` still `_None_`; pin text does **not** appear as a `- **…** [Approved]` claim under Decisions. Exit **0**. **Required red.** |
| **AC2** | Same fixture: `--format json` → `denied: false`, `decisions: []`, `conclusions: []`, `warnings` still include `kind: empty_authority`, `vault_pin_count` ≥ 1, `vault_pin_previews` array contains a string with `DECISION:` or `{needle}`. Exit **0**. **Required red.** |
| **AC3** | Unit: `render_project_markdown_with_vault_pins(&empty_allowed, Some(&VaultPinStanza { count: 12, previews: vec!["DECISION: x".into()] }))` inserts heading **after** `BRIEFING_EMPTY_AUTHORITY_NEXT_STEP` and **before** `## Freshness`; Decisions body is `_None_`; `render_project_markdown(&empty_allowed)` (no pins) does **not** contain the heading. |
| **AC4** | Hermetic granted-empty **0 pins**: human contains `Pinned: 0` and heading; does **not** contain a fabricated `DECISION:` preview line; JSON `vault_pin_count == 0` and `vault_pin_previews == []`. Exit **0**. |
| **AC5** | Hermetic denied (no grants): human does **not** contain `## Vault pins`; JSON has **no** `vault_pin_count` key; T275 grant-wall / no `_None_` still holds. Exit **0**. |
| **AC6** | Unit: `BRIEFING_EMPTY_AUTHORITY_NEXT_STEP.chars().count() <= 140` and `!contains('\n')` **stays green** (T263 AC14). Const text unchanged. |
| **AC7** | T227 AC6 seeded Approved+conclusion hermetic **stays green** and stdout does **not** contain `## Vault pins` (overlay off when authority non-empty). |
| **AC8** | Unit: `serde_json::to_value` of a default `ProjectBriefingPacket` / `empty_denied` has **no** `vault_pin_count` key (DTO freeze). |
| **AC9** | Unit: `preview_line` of tagged `ASSISTANT: TAGS: t288\nDECISION: needle` in a stanza preview contains `DECISION:` and does not start with `TAGS:` (inherit T287 F6; do not reimplement). |
| **AC10** | Docs: CAPABILITIES dual-model sentence + `briefing project` after_help one sentence + PROTOCOL-COMPAT briefings CLI extras + CHANGELOG T288. |
| **AC11** | No new crate. No clap 5. No `unwrap`/`expect`/`panic` in production. `cargo clippy -p ai-brains-cli -p ai-brains-control-plane --all-targets -- -D warnings` clean on go. |
| **AC12** | Manual (source/hermetic bin, not PATH): `cargo run -p ai-brains-cli -- briefing project --format human` contains `Pinned:` with **nonzero** **and** `not Approved` **and** `recall`; Decisions stay `_None_`. JSON `denied: false`. Exit **0**. Live `N` is `count_pinned_memories(Some)` (volatile; plan-time ~3821) — **not** required equal to `memory list --summary` 3822 (OpenCode m1). |
| **AC13** | Store recency / `list_authority_memories` units **stay green**. Do not change `query_store.rs`. |
| **AC14** | rstest `#[case]`: overlay-gate helper — denied → skip; nonempty decisions → skip; granted-empty → apply. **Required.** |
| **AC15** | Hermetic: one untagged `## Objective` pin (chrome) + no DECISION pin → `Pinned:` ≥ 1 and **no** `DECISION:` preview (F32). Proves COUNT ≠ GLOB samples. |
| **AC16** | Hotspot-only: `HOTSPOT: crates/foo.rs` pin → count ≥ 1, previews **omit** that line (F5 `== Decision \|\| == Constraint`; Agy m2). |
| **AC17** | Unit: scope-parse helper — `Repository:{valid-uuid}` → `Some(ProjectId)`; `Personal:{uuid}` / `"not-a-scope"` / `"Repository:"` / `"Repository:not-a-uuid"` → `None` (Agy m1 / F14). Must not panic. |

---

## 5. Design notes

### 5.1 Two products, one stanza

| Product | Corpus | How you fill it |
|---------|--------|-----------------|
| Vault pins | `MemoryPinned` text | `ai-brains pin` / harness ingest / `recall` |
| Governed authority | Approved decisions + Active/Confirmed conclusions | `decision propose` + approve (not this track) |

The stanza is a **cross-link**, not a merge. Label **not Approved** is load-bearing.

### 5.2 Why CLI overlay, not CP packet fields

`build_project_briefing` talks to `GovernedQueryStore`, not `QueryStore::list_authority_memories`. Putting pin SQL in CP mixes governed read with ungoverned inventory and forces DTO fields (T287 `MemoryListFilter` lesson). CLI already holds `ctx.conn: VaultConnection`. T243/T263 list `next_step` overlay is the pattern.

### 5.3 Placement

Insert after the empty_authority footer (`NOTICE` + `NEXT_STEP`) so the conversation is: “no Approved → recall; also, here is the vault inventory.” Putting the stanza at EOF (after Budget) hides it from word-skimming agents.

### 5.4 Live GLOB 0

Do not block Manual DoD on sample lines. Inventory COUNT is the “vault is not empty” proof. Hermetic AC1/AC2 prove samples when a pin exists.

### 5.5 JSON for agents

This TUI/agent default is non-TTY → JSON. Human-only would miss the primary reader. T180 additive extras on the CLI emit are enough; N−1 ignore.

### 5.6 Two COUNT surfaces (OpenCode m1)

| Surface | SQL | Plan-time live |
|---------|-----|----------------|
| `memory list --summary` | `count_memories` session-join `(sp.project_id = ? OR mp.project_id = ?)` | **3822** |
| T288 stanza / T214 preflight | `count_pinned_memories` `mp.project_id = ?` | **~3821** |

Both nonzero. Manual AC12 does **not** lock 3822. Implement review records the two sources; do not “fix” the 1-off by switching SQL.

---



## 6. Non-goals

- Pin → `DecisionProposed` / Approved (H2)
- Filling `constraints[]` from MemoryPinned
- Personal briefing Preferences `_None_` (T289)
- `evidence` / `source` / `review` / `query progressive` pin count (T290)
- Graph neighbors pin-first (T293)
- Leftover dest upsert (T294)
- Governed preflight markdown stanza (T170)
- Daemon/HTTP overlay
- New clap flags / clap 5 / rusqlite 0.40
- `cargo install` / live `.env` / live migrate / extra live grants
- Growing hotspot files listed in F12/F35
- Changing `BRIEFING_EMPTY_AUTHORITY_NEXT_STEP`
- USER/SYSTEM TAGS GLOB
- New `QueryStore` method
- `is_injectable_privacy` filter on stanza previews (F36; T287 F38 analog)

---

## 7. Verification plan (TDD)

**Red first (required names):**

1. `briefing_project__granted_with_decision_pin__human_stanza_not_under_decisions` (AC1)
2. `briefing_project__granted_with_decision_pin__json_overlay_count_and_previews` (AC2)
3. `render_project_markdown_with_vault_pins__some__inserts_after_empty_authority` (AC3)
4. `briefing_project__granted_empty_zero_pins__pinned_zero_no_fabricated_decision` (AC4)
5. `should_overlay_vault_pins__rstest_denied_nonempty_empty` (AC14)
6. `parse_repository_project_id__rstest_personal_garbage_valid` (AC17)

Then AC5/AC7/AC15/AC16 hermetic; AC6/AC8/AC9 units (some already exist — stay green).

Manual AC12 on go with `cargo run` (not PATH).

---

## 8. Risk

| Risk | Mitigation |
|------|------------|
| Agents treat stanza as Approved | Heading **not Approved**; Decisions stay `_None_`; JSON arrays stay `[]` |
| Live GLOB 0 looks incomplete | F4 COUNT; F32 honest empty samples; hermetic needle SoT |
| DTO literals break | F10 no struct fields |
| Preflight word budget eats next-step | F11 preflight still `None`; F33 stanza after CP budget on CLI only |
| Fail-closed COUNT errors hide briefing | F13 fail-open |
| Hotspot samples look like Safety | F5 exclude Hotspot |
| Dual-truth scripts | F3 documented extras; after_help; PROTOCOL-COMPAT |

---

## 9. Deferred absorb / decline

| Item | Disposition |
|------|-------------|
| Audit `briefing project` granted-empty `_None_` vs 3k pins | **Absorb** F2–F5 / AC1–AC4 / AC12 |
| Dual model: briefing = Approved; pins via recall | **Absorb** F1 (keep split) + F2 (cross-link) |
| T263 F24 soft vault pin COUNT | **Absorb / promote** F4 |
| T263 F3 / T227 F3 never scrape pins into authority | **Affirm** F1 |
| T263 F29 next-step ≤140 | **Affirm** F6 / AC6 |
| T275 denied grant-wall | **Affirm** F7 / AC5 |
| T287 `list_authority_memories` / `preview_line` | **Reuse** F5 / F15 / AC9 / AC13 |
| T287 R1-1 live GLOB 0 | **Absorb as F4/F32** — COUNT not GLOB |
| Placeholder JSON “if T180 allows else human-only” | **Absorb JSON overlay** F3 — T180 additive; agents are non-TTY |
| Personal deny `_None_` | **Decline → T289** |
| Lists/progressive pin count | **Decline → T290** |
| Graph neighbors dump sessions | **Decline → T293** |
| leftover dest-missing / context skip upsert | **Decline → T294** |
| T287 human list mix | **Completed** — do not reopen |
| T240 F2 / T263 H2 / 750 ms / clap 5 / rusqlite 0.40 | **Decline** F24 |
| last-PR Cursor #203 | **N/A** empty — **no T301** |
| Identity mismatch leftover `7d97a456` | **Not this track** — T258 / T294 |
| Dependabot `#61` rusqlite 0.40.2 | **Decline** — no bump |
| Open rows T289–T300 | **Not related** except named declines above |
| Closed T274–T287 tables | **Stay closed** |

---

## 10. Implement order (on go)

1. Phase 0 re-verify HEAD / deferred / #203 still empty / pins / live briefing read-only
2. FEATURE TX
3. Red AC1/AC2/AC3/AC14/AC17
4. Green: `VaultPinStanza` in `renderer.rs` + render-with-pins + CLI collect/overlay (limit 32; fail-open parse)
5. Red/green AC4/AC5/AC15/AC16
6. Docs AC10
7. Clippy + targeted nextest + full gate
8. Manual AC12
9. Phase-1 review → codex-review
10. Publish: push `track/T288-*` → PR → watch GHA `CI` green → squash-merge → prune

---

## 11. Soft residuals

| Residual | Notes |
|----------|-------|
| PATH until `cargo install` | F17 |
| Live GLOB 0 samples | F32; not a fail if COUNT shown |
| Daemon/HTTP no overlay | F29 |
| Personal `_None_` | T289 |
| Lists/progressive | T290 |
| Governed preflight no stanza | F27 |
| Duplicate overlay vs T263 list helper | F12 — do not grow `governed_common.rs` |
| `memory list --summary` vs stanza COUNT (~1) | F4 / §5.6 — two SQL predicates |
| NeverInject/Sealed in previews | F36 display-only; re-trigger if owner wants injection-safe samples |

---

## 12. Touch map

| Path | Change |
|------|--------|
| `crates/ai-brains-cli/src/commands/briefing.rs` | Collect stanza; call render-with-pins; JSON overlay |
| `crates/ai-brains-control-plane/src/briefings/renderer.rs` | Const + `VaultPinStanza` + `render_project_markdown_with_vault_pins`; units |
| `crates/ai-brains-control-plane/src/briefings/mod.rs` + `lib.rs` | **Required** re-export `VaultPinStanza` + `render_project_markdown_with_vault_pins` (F11 / OpenCode O1) |
| `crates/ai-brains-cli/tests/governed_vault_pin_honesty.rs` | Additive hermetic AC1/AC2/AC4/AC5/AC15/AC16 |
| `crates/ai-brains-cli/src/main.rs` | `briefing project` after_help one sentence |
| `Docs/CAPABILITIES.md` | Dual-model stanza sentence |
| `Docs/PROTOCOL-COMPAT.md` | CLI extras vs daemon packet |
| `CHANGELOG.md` | T288 row (on go) |
| `conductor/conductor.md` / `deferred.md` / this spec | Registry |

**Do not touch:** `project.rs`, `preflight.rs` (CLI), `personal.rs`, `governed_common.rs`, `query_store.rs`, `session_chrome.rs`, `ranking.rs`, `pin.rs`, contracts `ProjectBriefingPacket` fields, `ci.yml`.

---

## 13. AI fold-in

Inputs (not edited): `agy-review.md` + `opencode-review.md` at HEAD `ed28100`. Live verify: `query_store.rs` `:70–75` vs `:707–711`; `pin.rs` `:28–34`; `tests/common/mod.rs` `:164–180` `DEFAULT_PROJECT` == `aaaaaaaa-…`; `PinKind` `:65`; `list_memory_rows` LIMIT `:99–107`; `MemoryListRow` no privacy `:178–184`; `preview_line` envelope `:642`; `render_project_markdown` `:66`. Pins **snapshot — re-verify at execute** (clap lock 4.6.1 / crates.io 4.6.6; rusqlite 0.39.0 / 0.40.2; no clap 5).

### Pins locked by fold-in

1. **F14/AC17 (Agy m1):** overlay scope parse is `strip_prefix("Repository:")` then `ProjectId::from_str`; Personal/garbage/`?` on `run_project` **forbidden**.
2. **F4/AC12/§5.6 (OpenCode m1):** stanza COUNT is `count_pinned_memories` (`mp.project_id`); Manual is nonzero — not equality to `memory list --summary` 3822.
3. **AC1 (OpenCode m2):** hermetic pin uses `hermetic_cmd` / `hermetic_cmd_with_ids`; both `PROJECT_ID` and `SESSION_ID`.
4. **F36 (OpenCode m3):** display-only privacy parity with `memory list`; no `is_injectable_privacy` DoD.
5. **F11 (OpenCode O1):** `VaultPinStanza` lives in `renderer.rs`; re-export `mod.rs` + `lib.rs`.
6. **F5 (OpenCode O2):** fetch `limit = 32` (SQL LIMIT before retain).
7. **Already:** Agy m2 Hotspot exclude = F5/AC16; Agy O1 PROTOCOL-COMPAT = F25/AC10; Agy O2 0-pin vs with-pin = AC14 gate + AC4 + AC1 (do not mix collect into overlay-gate rstest).

### Per-AI disposition

| Source | Item | Disposition |
|--------|------|-------------|
| Agy | B / M | None filed |
| Agy | **m1** fail-open `Repository:` / `ProjectId` parse | **Already** F13/F14; **tightened** F14 + **AC17** |
| Agy | **m2** Hotspot exclude `Decision \|\| Constraint` | **Already** F5 / AC16 |
| Agy | **O1** PROTOCOL-COMPAT CLI extras vs daemon DTO | **Already** F25 / AC10 |
| Agy | **O2** rstest denied / nonempty / 0-pin / with-pin | **Already** AC14 + AC4 + AC1 — overlay-gate is three cases; pin presence is collect |
| OpenCode | B / M | None filed |
| OpenCode | **m1** 3822 session-join vs ~3821 `count_pinned_memories` | **Folded** F4 / AC12 / §5.6 |
| OpenCode | **m2** pin env `PROJECT_ID`+`SESSION_ID` | **Folded** AC1 / F20 / plan Phase 1 |
| OpenCode | **m3** NeverInject/Sealed in previews | **Folded** F36 / F25 display-only |
| OpenCode | **O1** `VaultPinStanza` in `renderer.rs` | **Folded** F11 required re-export |
| OpenCode | **O2** over-fetch 8→32 | **Folded** F5 `limit = 32` |
| both | last-PR #203 Cursor | **Affirm N/A** — no T301 |
| both | deferred T289/T290/T293/T294 / H2 | **Affirm** |

No Blockers. No Majors. No new placeholder minted. Do **not** edit `*-review.md`.
