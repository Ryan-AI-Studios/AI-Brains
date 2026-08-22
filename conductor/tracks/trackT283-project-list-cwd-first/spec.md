# T283 — `project list` must not lead with leftover 18k pins

- **Track ID:** T283-ProjectListCwdFirst
- **Status:** **Planned** (Pending until **go**)
- **Category:** UX / HONESTY
- **Owner:** Grok
- **Source:** Live CLI audit 2026-08-21 — `project list` **7/6**; first row leftover `7d97a456` / `C:\dev\crawlx` 18036; cwd `*C:\dev\ai-brains` third. Re-verified **2026-08-22** (HEAD `6d3cbc5` T282 `#198`): leftover **18043** first; cwd `3581317d` **fourth** (3633).
- **Depends on:** T212 ✅ labels + store `ORDER BY memory_count DESC, project_id ASC`; T230 ✅ never-blank; T267 ✅ footer (do **not** reopen leftover-as-AI-Brains); T240 ✅ path-first detect; T258 ✅ adopt-path; T266 ✅ Family B always-human default
- **Blocks / feeds:** Operators scanning `project list` from this repo see the cwd path-owner first. Leftover 11-root rebind stays **T276 F9**. Shell leftover dump stays **T282 Completed**.
- **Absorbs:** Placeholder problem text + Manual DoD; deferred.md “`project list` leftover-first”; T276 F10/closeout list leftover-first pointer; T282 closeout “T283 list cwd-first” peer
- **Not absorbed (DoD):** T267 footer pick/suggestion; T240 F2 silent `.env` write; T258 adopt-path; T276 live leftover rebind; T212 JSON keys / store SQL; T266 Family B default; star-as-sort; `--sort` flag; JSON array reorder; clap 5; rusqlite 0.40; DTO keys; `cargo install`
- **Research date:** 2026-08-22 (plan dogfood HEAD `6d3cbc5` T282 `#198`). Product `src/` = T282. Store order still T212 F13. Fold-in against `dd57150` (docs-only; crates identical to `6d3cbc5`).
- **AI fold-in:** 2026-08-22 `agy-review.md` + `opencode-review.md`. **Agy B 0 / M 0.** **OpenCode B 0 / M 0.** **Agree (Agy):** m2 F37 `with_capacity` + no dup/drop; O1 already F19; O2 last-id AC1. **Agree (OpenCode):** m-1 AC1 once+len; m-3 AC10 max-memory not hardcoded leftover; m-4 AC5 re-env after denylist; m-5 F35 “JSON order unchanged”; O-1 AC3/AC5 `nth(1)`; O-2 F39 comment. **Already:** OpenCode m-2 F19 `:89`; OpenCode O-3 F26. **Decline (Agy m1):** fail-open `resolve_path_alias_for_location` — footer `:112` still `?`; would not stop the command unless T267 is reopened. **Affirm:** #198 N/A; no T285. Disposition **§13**.
- **Ledger:** planning DOCS TX `0535063a-dd76-454e-8c1b-bae350a5d7bd`. Fold-in DOCS TX `254805ef-bfcb-4448-8ad0-e2c66374b19a`. Implement starts a **FEATURE** TX on **go**.
- **Isolation:** Do **not** `cargo install`. Do **not** write live `.env` (T240 F2). Do **not** `adopt-path --write-env`. Do **not** `rebind-path --write`. Do **not** `set-alias 7d97a456 … AI-Brains`. Do **not** grow hotspot `project.rs` with new helpers (call existing; new sibling). Do **not** print or commit `AI_BRAINS_KEY`. Do **not** live `policy bootstrap`, `safety sync` without `--dry-run`, `retention apply --confirm`, or `graph rebuild`. Do **not** mutate schtasks.

---

## 1. Objective

1. **Human `project list` leads with the cwd path-owner.** Store order is memories-desc, so leftover `7d97a456` (~18k) wins the table. The active `*` is visible on `3581317d` but that row is fourth. Operators/agents who run `project list` to answer “which project is this repo?” read the leftover dump first. Promote the cwd path-owner to the first **data** row on the human table only.
2. **JSON stays inventory-by-size.** T212 JSON is the machine contract. Scripts that treat `projects[0]` as the largest stay valid. Human is not a wire contract ([clig.dev](https://clig.dev/) *Changing output for humans is usually OK*).
3. **North star.** Capture independence: display reorder only. No events. No models. No `.env` write. No identity rewrite. No leftover UUID hardcoded. No new crates. No pin bumps.

This unblocks daily honesty for the Windows-first vault: leftover volume is still real (T276), but the table you run from AI-Brains must not pretend crawlx is this repo.

---

## 2. Live baseline (re-scan 2026-08-22)

### 2.1 Operator dogfood (this machine)

| Signal | Observation |
|--------|-------------|
| HEAD | **Plan dogfood:** `6d3cbc5` T282 squash `#198`. Tree **CLEAN**. `origin/main` = HEAD (`0 0`) after `git fetch --all --prune`. |
| PATH `ai-brains` | `C:\Users\RyanB\.cargo\bin\ai-brains.exe` mtime **2026-08-22 14:49**, 25 443 840 bytes, **0.1.1**. Older than T282 merge (19:37). List hole is in **source too** (T282 did not touch list). **Do not `cargo install`.** Tests/manual AC use `cargo run` / hermetic. |
| `preflight --summary` | Scope `C:\dev\ai-brains` (`3581317d`). Pinned **3633** (volatile). In-context **5/0/0**. Grants **0 of 3** (T275 hermetic; live not bootstrapped). Capture independence holds. |
| PATH `project list` | Header then first data row `(no alias) 7d97a456-… 18043 … C:\dev\crawlx`. Cwd `*C:\dev\ai-brains 3581317d-… 3633` is **fourth**. Footer stderr: `27 project(s) have no alias.` / `Example: ai-brains project set-alias 33ec90e0-… my-project`. **T267 already avoids leftover-as-AI-Brains.** |
| PATH `project whoami` (non-TTY → JSON) | `effective` / `env` / `path_alias` / `detect` = `3581317d-…`. **`shell_project_id`: `7d97a456-…`.** `mismatch: false`. `remediations: []`. Cwd identity is T258-complete; the list hole is **sort**, not env vs path. |
| Last GitHub PR | [#198](https://github.com/Ryan-AI-Studios/AI-Brains/pull/198) T282 (2026-08-22). `gh pr view --comments`, `/reviews`, `/comments`, `issues/198/comments` all **empty**. **last-PR Cursor: N/A.** Open PRs: Dependabot remotes only (`#61` rusqlite 0.40.2, `#62` chrono 0.4.45, `#59` tokio, actions `#68–#72`). **No leftover to mint. No T285.** |
| Prior #188 Bugbot | **T284 Completed** `#193`. Not this track. |
| Identity / doctor | ledgerful doctor 4 warn (legacy `.changeguard` / sig-pin / timings / models unreachable). **0 pending / 0 drift** at scan. Hotspot **#1** `project.rs` (displayScore **3.908**, 1472 lines) — **do not grow helpers**. `sync.rs` #2. `forget.rs` #3. `context.rs` #4. `governed_common.rs` #5. Intended touch: `list()` call-site + **new sibling**. |
| `ISSUES.md` | **Does not exist.** |

### 2.2 Why this still matters

| Residual | Why it is still a product hole / why decline |
|----------|----------------------------------------------|
| Human table leftover-first | clig.dev: human-first + saying (just) enough + most important at the top. Cwd path-owner is the question `project list` answers in this repo. **DoD.** |
| Star already marks active | `*` is env `AI_BRAINS_PROJECT_ID` (T212). Leftover env would star leftover and keep it first if we sorted by star. Path-owner is T240/T258 SoT. **Star is not the sort key.** |
| Reorder JSON `projects[]` | T212 F13 + clig.dev “use `--json` to keep output stable.” Agents/scripts that pick largest-first stay valid. **Decline as DoD.** |
| Change store `ORDER BY` | `list_projects` is shared (init / detect / preflight / paths). `list_projects_detail` feeds JSON + footer. **Decline.** Human-only CLI permute. |
| Footer leftover-as-AI-Brains | **T267 Completed.** Live footer already picks orphan `33ec90e0` + `my-project`. **Do not reopen.** Pass original memory-desc vec to footer. |
| Hardcode leftover UUID | T267 F9. Next dump would miss. **Decline.** |
| `--sort cwd\|memories` flag | Prefer-flags is clig.dev, but this is one operator table. Default must be right (clig.dev). Extra flag is discoverability debt. **Decline.** |
| Promote env/`*` when no path-owner | Would keep leftover-first when shell/env is leftover and cwd is unregistered. **Decline as DoD.** No owner → leave memory-desc. |

### 2.3 Code truth

| Item | Location | Notes |
|------|----------|--------|
| Human + JSON list | `project.rs` **`:26–67`** / **`:69–102`** | Fetch `list_projects_detail`; JSON returns immediately; human prints store order then footer. **Promote only the human loop.** Footer keeps original `&projects`. |
| Store order | `query_store.rs` **`:549–567`** `list_projects`; **`:584–611`** `list_projects_detail` | `ORDER BY memory_count DESC, p.project_id ASC` (T212 F13). **Do not edit.** |
| `ProjectListDetail` | `ai-brains-store/src/lib.rs` **`:197–209`** | `Clone`. Fields frozen. **Do not add `cwd` / `sort_rank`.** |
| JSON envelope | `project.rs` **`:494–512`** | `api_version`, `projects[]`, `unaliased_count`; row `active` skip-if-false. **Keys frozen.** |
| clap | `main.rs` **`:2636–2643`** | `List { format }` `human` default / `json`. Family **B** (T266). **No new flags.** `after_help` already names T267 footer. **Additive one sentence** on cwd-first (F35). |
| Path owner | `project.rs` **`:226–237`** `resolve_path_alias_for_location` | Already `pub(crate)`. Git toplevel else cwd. **Reuse. Do not fork.** |
| Git identity | `project.rs` **`:166–208`** `collect_git_identity` | Footer already fail-open. Same for promote. |
| Footer | `project_list_footer.rs` **`:21–41` / `:82–132`** | Assumes memory-desc among unaliased. **Pass original vec.** **Do not edit** this module as DoD. |
| Label / truncate / activity | `project.rs` **`:383` / `:436` / `:451`** | Already `pub(crate)`. Human print stays in `list()`. |
| Hermetic list | `tests/project_list_labels.rs` | Shape / empty / JSON keys / star. Finds rows **by id**, not `[0]`. **Stay green.** New file `tests/project_list_cwd_first.rs`. |
| Footer hermetics | `tests/next_action_honesty.rs` | T267 AC6/AC7/AC16. **Stay green** because footer order unchanged. |
| register-path fixture | `tests/project_register_path.rs` | Pattern for hermetic path owner. |
| Modules | `commands/mod.rs` **`:44–48`** | Add `project_list_order` next to `project_list_footer`. |
| Docs | `Docs/CAPABILITIES.md` **`:202–203`**; `Docs/OPERATIONS.md` **`:519–522`** (stale T76 columns) | Additive cwd-first on human; JSON order frozen. OPERATIONS refresh to T212 columns + this sort. |
| Skill | `.agents/skills/ai-brains/SKILL.md` — **no** `project list` match (F19 no-op). `.claude/skills/ai-brains/SKILL.md` **`:89`** command table already names `project list`. **One sentence** on that row (no new section). |
| `ISSUES.md` | — | Does not exist. |

### 2.4 Dependency / standards research (2026-08-22) — snapshot; re-verify at execute

| Pin | Workspace / lock | Ecosystem (today) | Action |
|-----|------------------|-------------------|--------|
| `clap` | workspace **4.5** / lock **4.6.1** | crates.io **4.6.6** (`cargo search`). **No clap 5.** | **No bump.** No new flags. |
| `serde_json` | lock **1.0.150** | crates.io **1.0.151** | **No bump.** JSON keys frozen. |
| `chrono` | lock **0.4.44** | crates.io **0.4.45** (Dependabot #62 open) | **No bump.** |
| `rusqlite` | lock **0.39.0** + sqlcipher + backup | crates.io **0.40.2** (Dependabot #61) | **No bump.** Store SQL untouched. |
| `uuid` | lock **1.23.1** | crates.io **1.25.0** | **No bump.** |
| `tokio` | workspace **1.52** / lock **1.52.3** | crates.io **1.53.1** (Dependabot #59) | **No bump.** |
| rustc / edition | **1.95.0** / **2024** | workspace toolchain | Unchanged |
| nextest | **0.9.140** | — | Unchanged |
| workspace | **0.1.1** | — | **No bump** |
| New crates | — | — | **Zero.** No table crate. |

**Online / primary sources**

| Claim | Source | Takeaway |
|-------|--------|----------|
| Human-first; human output may change; JSON is the stable machine surface | [clig.dev](https://clig.dev/) Output + Future-proofing (source repo current; fetched 2026-08-22) | Reorder **human** only. Freeze JSON array order. |
| Most important information first; saying (just) enough | clig.dev Philosophy | Cwd path-owner is the first row of the operator table. Omit extra columns/flags. |
| Default must be right; prefer flags only when needed | clig.dev Arguments and flags | No `--sort`. Default human is cwd-first. |
| kubectl `config get-contexts` | [kubernetes.io generated docs](https://kubernetes.io/docs/reference/kubectl/generated/kubectl_config/kubectl_config_get-contexts/) | Marks current with `*`; does **not** require current-first. Our `*` already exists (env). Sort is **path-owner**, not kubectl-copy. |
| T266 Family B | live CAPABILITIES `:107` | `project list` default **human** even on pipes. Cwd-first applies to default + `--format human`. JSON opt-in unchanged. |

**N/A:** SQLCipher page encrypt, schtasks, T180 DTO new keys, Windows service, Safety GLOB (T279 Completed), policy HINT (T280 Completed), nightly HTTP vs TCP (T281 Completed), `--show` leftover (T282 Completed `#198`).

**Could not verify:** whether every operator vault still has leftover `7d97a456` as max-count (this machine **does**, 18043). DoD is the promote helper + hermetic small-cwd vs large-other; live classify-only is pass-with-observed-data.

**ledgerful / ai-brains:** `preflight --summary` 0 of 3 grants @ **3633** pins; PATH list leftover-first + whoami `mismatch: false`; `ledgerful ledger status --compact` 0 pending / 0 drift; `search "list_projects_detail"` → `query_store.rs:584` + `project.rs:27`; `scan --impact` CLEAN at `6d3cbc5`; `hotspots --json --limit 5` `project.rs` #1 — do not grow helpers. Semantic recall returned T262–T266 review-track dumps (PATH-behind ranking) — not used as SoT.

---

## 3. Frozen decisions

| ID | Decision |
|----|----------|
| **F0 — Go gate** | Plan-only until user **go**. Planning is DOCS TX `0535063a`. Fold-in is DOCS TX `254805ef`. Implement starts a **FEATURE** TX. |
| **F1 — Human cwd-first** | After the header, the first data row is the cwd **path-owner** (`resolve_path_alias_for_location`) **iff** that id is present in `list_projects_detail`. Helper `promote_cwd_owner(rows, cwd_owner) -> Vec<ProjectListDetail>`: if `cwd_owner` is Some nonempty **and** `rows.iter().position(|r| r.project_id == owner)` hits, that row is index 0 and the rest keep **stable relative order**. Exact string, **not** `contains`, **not** case-fold, **not** 8-hex prefix. **Do not** special-case `7d97a456`. Construct with **F37**. Result `len == rows.len()`; promoted id appears **once** (F33). |
| **F2 — JSON freeze** | `--format json` prints store order (T212 F13: `memory_count DESC, project_id ASC`). **No** new keys. **No** `sort` / `cwd_first` field. `api_version` stays `"1"`. |
| **F3 — Footer freeze** | `print_unaliased_footer(ctx, &projects)` keeps the **unpromoted** vec. T267 F3/F3b/F9 stand. **Do not edit** `project_list_footer.rs`. |
| **F4 — No `.env` write** | Affirm T240 F2. List is read-only. |
| **F5 — clap flags freeze** | No `--sort`. `format` remains `human` \| `json`. Family **B** (T266) stands. |
| **F6 — T266 Family B freeze** | Default stays human (including pipes). Cwd-first applies to default + `--format human` only. |
| **F7 — Star is env, not sort** | T212 `*` on `AI_BRAINS_PROJECT_ID` unchanged. Promote **does not** use `active_id`. |
| **F8 — No owner → no promote** | `None` / empty / id not in rows → `rows.to_vec()` (memory-desc). Unregistered cwd keeps leftover-first — honest. |
| **F9 — Module** | `promote_cwd_owner` + units live in **new** `crates/ai-brains-cli/src/commands/project_list_order.rs`. `mod.rs` registers it. `list()` calls promote for the human loop only. **No new helpers in `project.rs`.** Prefer **≤15** net lines there (call + iterate promoted). Do **not** grow `sync.rs` / `forget.rs` / `query_store.rs`. |
| **F10 — Decline star-as-sort** | Stub “or `*` active” is the happy case where env == path-owner. When they differ, path-owner wins (T240). Star-only fallback would put leftover env first. **Not DoD.** |
| **F11 — Store SQL freeze** | `list_projects` / `list_projects_detail` `ORDER BY` untouched. |
| **F12 — Pins / crates** | No clap 5, no rusqlite 0.40, no chrono 0.4.45, no uuid 1.25, no tokio 1.53, no new crates, workspace **0.1.1**. |
| **F13 — PATH** | Do not `cargo install` unless the user asks. Tests/manual use `cargo run` / hermetic. |
| **F14 — Contracts** | No DTO. PROTOCOL-COMPAT: human is **not** a wire contract (list-paths analog). JSON keys unchanged. E1: missing cwd-owner → first JSON/human row stays largest (absent promote), not `null`. |
| **F15 — Capture independence** | Display/docs only. No events. No models. |
| **F16 — Stop-before live mutate** | Even after go: do not write live `.env`, do not `adopt-path --write-env`, do not `rebind-path --write`, do not `set-alias` leftover as `AI-Brains`, do not live bootstrap / apply / rebuild / `safety sync` without `--dry-run`. |
| **F17 — Decline peers** | T267 footer restyle; leftover 11-root rebind (T276 F9); T240 F2; T258 adopt-path; T282 product (Completed); T255 750 ms; T263 H2; T275 live bootstrap; T277 live `--no-prune`; T278 live rebuild; T279 live pin; T284 live apply; T281 product (Completed). |
| **F18 — last-PR Cursor** | #198 empty → **N/A**. #188 closed by T284. Dependabot `#61` rusqlite **not** this track. **No T285.** |
| **F19 — Docs** | CAPABILITIES List projects: additive “human table puts the cwd path-owner first (remaining rows stay memory-desc); JSON array order stays T212 size-desc.” List JSON row: “array order is `memory_count DESC, project_id ASC` (not cwd-first).” OPERATIONS Listing Projects: replace stale T76 columns with T212 columns + cwd-first. Root CHANGELOG T283. CLI-EXIT-CODES unchanged (list still exit **0**). **No new skill section.** `.agents/skills/ai-brains/SKILL.md` stays no-op (no `project list` match). `.claude/skills/ai-brains/SKILL.md` **`:89`** table: one sentence that human `project list` leads with the cwd path-owner. |
| **F20 — Exit 0** | Unchanged. Empty vault still T198. |
| **F21 — Tests** | Naming `function_or_feature__condition__expected_result`. Units for F1. Hermetic cwd-small vs leftover-large (human vs JSON). No `unwrap`/`expect`/`panic` in production. |
| **F22 — Cross-model** | Honesty UX on identity inventory (easy T212/T267 regression). After Phase-1 review clean, run read-only `codex-review`. |
| **F23 — Debt file** | `conductor/ISSUES.md` does **not** exist. Deferrals → `conductor/deferred.md`. |
| **F24 — PowerShell** | `;` not `&&`. |
| **F25 — Compare source** | `project_id == cwd_owner` exact. No UUID parse (invalid leftover still matches if equal). Trim owner only if `resolve_*` already returns trimmed ids (it does). |
| **F26 — Probe fail-open** | `current_dir` Err → no promote. Git collect fail-open (`unwrap_or_default`) like footer. Store `resolve_path_alias_for_location` Err **fails the command** (same as footer `project_list_footer.rs` `:112` `?`). **Do not** `unwrap_or`/`ok()` that store error on the promote probe (OpenCode O-3). Agy m1 fail-open-on-resolve **declined** — footer would still `?`, so the command still dies unless T267 is reopened. Re-trigger: owner wants list to succeed when path-alias lookup errors. |
| **F27 — Classify-only live** | Manual AC uses `cargo run -p ai-brains-cli -- project list` from this repo. **Do not** treat PATH as proof. **Do not** write `.env`. |
| **F28 — Existing tests stay green** | T212 `project_list_labels.rs`; T267 `next_action_honesty.rs` footer; T198 empty; T230 never-blank units; T254 register-path list JSON. |
| **F29 — No leftover UUID in product** | Affirm T267 F9. Fixture ids in tests are hermetic UUIDs, not `7d97a456`. |
| **F30 — Columns freeze** | Header `label` / `project_id` / `memories` / `last_activity` / `path`. Widths 30/36/8/12. Truncate stays. |
| **F31 — Identity leftover** | `7d97a456` vs `fcb8a40f` in other trees is T258/T276. This cwd is path-owner `3581317d` vs leftover size-winner. **No T285.** |
| **F32 — No JSON `cwd_first`** | Adding a key is a T180-class growth. Human-only permute is enough. Soft residual if agents later need an explicit marker. |
| **F33 — One promote** | `promote_cwd_owner` runs **once**. First data row `project_id` appears once (no duplicated row). Unit proof: result `len == rows.len()` and promoted-id `count == 1` (OpenCode m-1). |
| **F34 — Hermetic isolate** | AC3–AC6 **must** `isolate_empty_home` (T205/T282 analog). |
| **F35 — after_help** | Additive one sentence on List `after_help` (`main.rs` `:2636–2638`): `human table puts the cwd path-owner first; JSON order unchanged`. **Not** “JSON stays memory-count DESC” (that can read as a new JSON promise). **Not** a clap flag. (OpenCode m-5.) |
| **F36 — Do not grow `query_store.rs`** | CLI permute. Store stays T212 F13. |
| **F37 — Allocate once (Agy m2)** | `promote_cwd_owner` uses `Vec::with_capacity(rows.len())`, pushes the matched row, then extends every other row in original order. No second allocate, no dup, no drop. Already-first and no-hit paths may `to_vec()`. |
| **F38 — AC5 re-env after strip (OpenCode m-4)** | `tests/common/mod.rs` `AMBIENT_DENYLIST` includes `AI_BRAINS_PROJECT_ID` (`:51`). `hermetic_bin` strips it. AC5 **must** `.env("AI_BRAINS_PROJECT_ID", leftover_id)` **after** `hermetic_bin()` (same pattern as `project_list_labels.rs` `pin_memory` `:89`). |
| **F39 — Call-site comment (OpenCode O-2)** | Rustdoc on `promote_cwd_owner`: cwd owner comes from `resolve_path_alias_for_location` (`project.rs` `:226–237`). One comment on the `list()` call. **Comment-only.** Do **not** add helpers or comments on the resolve fn itself (hotspot `project.rs`). |

---

## 4. Acceptance criteria

| AC | Proof |
|----|-------|
| **AC1** | Unit: three `ProjectListDetail` rows (ids `a`,`b`,`c` counts 30,20,10). `promote_cwd_owner(&rows, Some("c"))` (last) → `[c,a,b]`; **`out.len() == 3`**; **`out.iter().filter(|r| r.project_id == "c").count() == 1`** (OpenCode m-1). `Some("b")` (middle) → `[b,a,c]` (Agy O2). `Some("a")` equals input (already first). Named `promote_cwd_owner__middle_id__becomes_first` (keep name; include last+middle+len/once in that test or sibling units). **Required red.** |
| **AC2** | Unit (rstest `#[case]`): `None`, `Some("")`, `Some("missing")` all return a clone equal to input (including empty vec). **Required red.** |
| **AC3** | Hermetic `tests/project_list_cwd_first.rs` **must** `isolate_empty_home`: two projects; leftover-shaped has **more** pins + other registered path; cwd dir `register-path` to the **smaller** project; `project list` with `.current_dir(cwd_dir)` exit **0**; **`stdout.lines().nth(1)`** (OpenCode O-1) contains the cwd project_id and does **not** contain the leftover id. Named `project_list__human__cwd_owner_smaller_count__first_data_row`. |
| **AC4** | Same fixture: `project list --format json` `projects[0].project_id` is the **larger** leftover-shaped id; cwd id is present later. JSON keys still `api_version` / `projects` / `unaliased_count`. Named `project_list__json__still_memory_desc`. |
| **AC5** | Same fixture; after `hermetic_bin()` **re-set** `.env("AI_BRAINS_PROJECT_ID", leftover_id)` (F38 — denylist `:51` strips ambient). Human **`stdout.lines().nth(1)`** is still **cwd** id; leftover row (later) may show `*`. Proves F7/F10. Named `project_list__human__star_on_leftover_env__cwd_still_first`. |
| **AC6** | Hermetic: two projects, **no** `register-path` for cwd; leftover larger. Human first data row is leftover (F8). Named `project_list__human__no_path_owner__memory_desc`. |
| **AC7** | T267 `next_action_honesty` footer tests stay green (original order). |
| **AC8** | T212 `project_list__format_json__shape_and_unaliased_count` stays green. |
| **AC9** | T198 empty vault still `No projects registered. (0 projects)` exit 0; no promote panic. |
| **AC10** | Manual classify-only (`cargo run`, **no** `.env` write): from this repo, human **`stdout.lines().nth(1)`** contains `3581317d-601e-44f7-ab84-fde90aa12d3c`. JSON: `projects[0]` is the **max-`memory_count`** row in the array (this machine observed leftover `7d97a456` / 18043 — pass-with-observed-data; do **not** hard-fail if another id becomes largest). Footer must **not** contain `set-alias 7d97a456` + `AI-Brains`. Exit **0**. Source/hermetic is DoD — **not PATH.** (OpenCode m-3.) |
| **AC11** | Docs: CAPABILITIES + OPERATIONS name human cwd-first + JSON size-desc; CHANGELOG T283; `.claude` `:89` one sentence; `.agents` skill unchanged. PROTOCOL-COMPAT no new required keys. CLI-EXIT-CODES list exit 0 unchanged. |
| **AC12** | No production `unwrap`/`expect`/`panic`; no clap/rusqlite bump; no DTO keys; `query_store.rs` / `project_list_footer.rs` / `sync.rs` / `forget.rs` absent from the product diff (or comment-only). `project.rs` has **no new named helpers**. |
| **AC13** | Default `project list` (no `--format`) is still a table, not JSON (T266 Family B). |
| **AC14** | Promoted human stdout contains cwd id **once** as a project_id column (F33). |

---

## 5. Design notes

### 5.1 Human layout (cwd is AI-Brains, leftover larger)

```
label                          project_id                           memories last_activity path
*C:\dev\ai-brains              3581317d-601e-44f7-ab84-fde90aa12d3c     3633 2m           C:\dev\ai-brains
(no alias)                     7d97a456-f2f4-43ea-1f13-211af684ad37    18043 32m          C:\dev\crawlx
C:\dev\ledgerful               fcb8a40f-…                                5139 …
```

Star stays on env. First row is path-owner even if leftover has more memories.

### 5.2 Gate

```text
promote_cwd_owner(rows, cwd_owner)
  Some(id) in rows → Vec::with_capacity(len); push match; extend rest
  else            → rows as-is (memory-desc)
```

JSON / footer: original `rows`.

### 5.3 Why not JSON / store / star

- JSON is T212 inventory-by-size; clig.dev says keep `--json` stable.
- Store `ORDER BY` is shared.
- Star is env; leftover env would recreate the hole.

### 5.4 Footer

T267 last-resort `unaliased[0]` is highest-memory **among unaliased**. Promoting the human table must not change that. Pass the store vec.

---

## 6. Non-goals

- Silent `.env` rewrite (T240 F2)
- `adopt-path --write-env` / `rebind-path --write`
- Live leftover 11-root rebind (T276 F9)
- Reopen T267 footer / leftover-as-AI-Brains
- Reorder JSON `projects[]` / add `cwd_first`
- Change `query_store` `ORDER BY`
- `--sort` / Family A `auto` on `project list`
- Star-as-sort / star-only fallback
- Hardcoded leftover UUID
- clap 5 / rusqlite 0.40 / new DTO keys
- `cargo install`
- Live `policy bootstrap` / `retention apply --confirm` / `graph rebuild` / `safety sync` without `--dry-run`

---

## 7. Verification plan

1. **Red:** AC1–AC2 fail (helper missing).
2. **Green:** `promote_cwd_owner` in `project_list_order.rs`; `list()` human loop uses it; JSON + footer do not.
3. Targeted: `cargo nextest run -p ai-brains-cli promote_cwd_owner --test project_list_cwd_first --test project_list_labels --test next_action_honesty` + clippy `-p ai-brains-cli --all-targets -- -D warnings`.
4. Manual classify-only AC10. **No** `.env` write.
5. Review log; FEATURE cross-model (F22).
6. Full gate before finalize. implement-track Phase 6 publish.

---

## 8. Risk

| Risk | Mitigation |
|------|------------|
| T267 footer last-resort changes | F3 / AC7 — original vec |
| JSON `[0]` consumers | F2 / AC4 |
| T240 F2 accidental write | F4; list has no `fs::write` |
| Hotspot `project.rs` growth | F9 sibling module |
| Star leftover env still first | AC5 / F7 / F10 |
| Duplicate first row | F33 / AC14 |
| PATH-behind until install | F13; hermetic/source DoD |
| Unicode / Windows paths | Promote is UUID compare; path probe reuses T240 normalize |

---

## 9. Deferred absorb / decline

| Item | Disposition |
|------|-------------|
| Audit `project list` leftover-first (7/6) | **Absorb** F1–F8 / AC1–AC6 / AC10 |
| Placeholder “cwd path-owner (or `*` active) first”; JSON freeze vs human-only | **Absorb** F1 human-only; **F2** JSON freeze; **F10** decline star-as-sort |
| T276 F10 / closeout `project list` leftover-first | **Absorb** (this track) |
| T282 closeout T283 peer | **Absorb** (this track) |
| T267 footer leftover-as-AI-Brains | **Decline** F3 — already Completed; pass original vec |
| T212 labels / JSON keys / store ORDER BY | **Decline** F2 / F11 / F30 — freeze |
| T230 never-blank | **Decline** — labels unchanged |
| last-PR Cursor #198 | **N/A** — comments/reviews empty |
| last-PR #188 Work / apply samples | **Decline** — **T284 Completed** `#193` |
| Dependabot `#61` rusqlite 0.40.2 | **Decline** F12 — **no T285** |
| leftover 11 roots | **Decline** — T276 Completed; live rebind owner-confirm |
| T240 F2 / clap 5 / DTO required keys | **Decline** F4/F12/F17 |
| Identity mismatch quiet `7d97` vs `fcb8a40f` | **Not this track** — T258 adopt-path; leftover data T276; list sort **this track** |
| JSON reorder / `--sort` / star-as-sort | **Decline** F2 / F5 / F10 |
| Historical CE wipe, MSI, `anyhow` allowlist, archive `changeguard` | **Decline** — not list chrome |

**Entire `deferred.md` scanned.** Closed/strikethrough rows stay closed. Open overlapping row is this placeholder (absorb). No new placeholder minted (#198 empty).

---

## 10. Implement order (on go)

1. Phase 0 re-verify `project.rs` `:26–102`, `query_store.rs` `:567/:611`, `project_list_footer.rs` `:21–41`, clap List `main.rs` `:2636–2643`, T212/T267 hermetics, deferred rescan, #198 still empty, pins. `git fetch --all --prune`; if `origin/main` moved, reconcile (no rebase over user work; never `git push origin main`).
2. Red AC1–AC2.
3. `project_list_order.rs` + `list()` human permute; JSON + footer original.
4. Hermetic `tests/project_list_cwd_first.rs` AC3–AC6 (`isolate_empty_home`); AC7–AC9 stay green.
5. Docs F19 (CAPABILITIES + OPERATIONS + CHANGELOG + after_help; `.claude` `:89` one-liner; **no** new skill section; `.agents` skill no-op).
6. Classify-only AC10. **No** `.env` write.
7. Review → `review.md`; FEATURE TX; implement-track Phase 6 publish. `scripts/dev-check.ps1` (not repo-root `dev-check.ps1`).

---

## 11. Soft residuals

| Residual | Disposition |
|----------|-------------|
| PATH until `cargo install` | F13 |
| JSON `cwd_first` marker | F32 |
| Star-only fallback when no path-owner | F10 |
| `--sort` flag | F5 |
| Live leftover 11 `C:\dev\*` roots | T276 F9 — owner-confirm rebind |
| Live 0 of 3 grants | T275 F10 |
| OPERATIONS was stale T76 columns | F19 refresh this track |
| Duplicate git/path probe vs footer | F9 — T267 signature freeze |

---

## 12. Touch map

| Path | Change |
|------|--------|
| `crates/ai-brains-cli/src/commands/project_list_order.rs` | **New** — `promote_cwd_owner` + units |
| `crates/ai-brains-cli/src/commands/mod.rs` | `pub mod project_list_order` |
| `crates/ai-brains-cli/src/commands/project.rs` | Human loop uses promoted vec; JSON + footer original |
| `crates/ai-brains-cli/src/main.rs` | List `after_help` additive sentence (F35) |
| `crates/ai-brains-cli/tests/project_list_cwd_first.rs` | **New** hermetic AC3–AC6 |
| `Docs/CAPABILITIES.md` | List + List JSON additive |
| `Docs/OPERATIONS.md` | Listing Projects refresh |
| `CHANGELOG.md` | T283 row |
| `.claude/skills/ai-brains/SKILL.md` | F19 one sentence on `:89` |
| `conductor/conductor.md` | Placeholder → Planned; Completed on go+merge |
| `conductor/deferred.md` | This absorption; closeout on implement |
| `conductor/tracks/README-T274-T284-CLI-QUALITY.md` | T283 Planned |

**Do not touch:** `query_store.rs`, `project_list_footer.rs`, `env_warn.rs`, `sync.rs`, `forget.rs`, `context.rs`, contracts, `Cargo.toml` / lock, `.agents/skills/ai-brains/SKILL.md`, live `.env`.

---

## 13. AI fold-in

Inputs: `agy-review.md` (HEAD `dd57150`) + `opencode-review.md` (HEAD `dd57150`). Product crates identical to `6d3cbc5`. **Agy B 0 / M 0.** **OpenCode B 0 / M 0.** last-PR #198 still empty. No T285. Do **not** edit the review files.

### Per-AI

| Source | Item | Disposition |
|--------|------|-------------|
| Agy **m1** | Fail-open `current_dir` / git / `resolve_path_alias_for_location` so list never dies | **Partial already F26** for `current_dir` + git. **Decline** store-resolve fail-open — footer `:112` still `?`, so the command still exits unless T267 is reopened. Re-trigger: owner wants list to succeed when path-alias lookup errors. |
| Agy **m2** | `Vec::with_capacity(len)` + push match + extend rest (no dup/drop) | **Folded** F37 / AC1 len+once |
| Agy O1 | Refresh OPERATIONS T76 columns | **Already** F19 / AC11 |
| Agy O2 | Units: first / middle / last / unknown / empty / empty slice | **Already** AC1 already-first + AC2 None/empty/missing/empty vec; **folded** middle `Some("b")` into AC1 |
| OpenCode **m-1** | Unit: promoted id `count == 1` and `len` unchanged | **Already** F33; **folded** AC1 asserts |
| OpenCode **m-2** | Confirm `.claude` `:89` exists; no new section; `.agents` no-op | **Already** F19 — verified `:89` Project identity row; `.agents` zero `project list` matches |
| OpenCode **m-3** | AC10 JSON: assert max-`memory_count` row, not hardcoded leftover UUID | **Folded** AC10 |
| OpenCode **m-4** | AC5 re-`.env` `AI_BRAINS_PROJECT_ID` after `hermetic_bin` denylist strip | **Folded** F38 / AC5 (`common/mod.rs` `:51`) |
| OpenCode **m-5** | F35 after_help: “JSON order unchanged” not “stays memory-count DESC” | **Folded** F35 |
| OpenCode O-1 | AC3/AC5 `stdout.lines().nth(1)` not whole-stdout contains | **Folded** AC3 / AC5 / AC10 |
| OpenCode O-2 | Doc-comment tying promote to `resolve_path_alias_for_location` | **Folded** F39 — helper rustdoc + `list()` comment; **not** on the resolve fn (hotspot) |
| OpenCode O-3 | Keep F26: resolve Err fails the command | **Already** F26; affirmed vs Agy m1 |

### Pins locked by fold-in

1. **F26:** store-resolve Err still fails list (footer parity). Agy m1 fail-open-on-resolve declined.
2. **F37 / AC1:** `with_capacity` + len unchanged + promoted id once; middle+last cases.
3. **F35:** after_help “JSON order unchanged”.
4. **F38 / AC5:** re-env after `hermetic_bin` strip.
5. **AC3/AC5/AC10:** first data row is `lines().nth(1)`; JSON `[0]` is max memory_count.
6. **F39:** comment-only; no new `project.rs` helpers.
7. **Affirm:** F19 `:89` no new skill section; #198 N/A; no T285; no B/M to decline except Agy m1 store-resolve.
