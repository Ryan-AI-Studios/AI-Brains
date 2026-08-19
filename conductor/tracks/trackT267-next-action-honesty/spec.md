# T267 — Next-action remediator honesty

- **Track ID:** T267-NextActionHonesty
- **Status:** **Completed** (FEATURE TX `ce462dfb-bf75-4f63-9b8c-9356f886b457`)
- **Category:** UX / FEATURE
- **Owner:** —
- **Source:** Audit 2026-08-16 — `harness status` **8/6**; `project list` **8/6**; opportunity “next-action is the remediator”; stub also named whoami→whoami
- **Depends on:** T235/T245/T253 harness ✅; T212 list footer ✅; T240 whoami ✅; T258 adopt-path remediations ✅; T259 leftover split (footer left here) ✅; T266 format (Family B harness/list) ✅
- **Blocks / feeds:** Operators get a command they have not just successfully run. Scan-roots parent/`--root` stays **T268**. Preflight envelope stays **T265**. Ledger pane stays **T271**. Safety-skip leftover stays **T272**.
- **Absorbs:** Audit T267 row (`harness status` self-next; list footer leftover-as-AI-Brains); T259 F3 / closeout footer pointer; T212 F8 footer stays, algorithm changes; T235 F40 install/uninstall next stays `harness status`; T258 F10 whoami remediations **already shipped** (affirm + regression)
- **Not absorbed:** T258 adopt-path verb; T259 leftover split / memory move; T268 scan-roots; T240 F2 silent Scope; T255 declines; clap 5 / new crates; T272 `safety_ids`; doctor/T240 “run `project whoami`” warn
- **Research date:** 2026-08-18 (plan dogfood HEAD `fa90981` T266 `#180`; plan commit `d4555f2`)
- **AI fold-in:** 2026-08-18 `agy-review.md` + `opencode-review.md` (no grok/claude/codex-plan). **B 0 / M 0 / m 5 / O 4.** **Agree hard:** OpenCode **m3** AC10 additive named hermetic; leftover live `path_count` is **11** (not 1) — AC7/AC16 split. **Agree:** Agy **m1** HEAD note; Agy **m2** AC15 Ok-row unit; Agy **O1** / OpenCode implicit pure picker; OpenCode **m1** line-count 1511/1368; OpenCode **m2** pin count volatile; OpenCode **O1** call `collect_git_identity` (already `pub(crate)`). **Partial:** OpenCode **O3** after_help optional. **Decline:** OpenCode **O2** ledgerful hygiene; OpenCode “leftover is single-path so F3(2) picks it”. Disposition **§13**.
- **Ledger:** planning DOCS TX `50c39329-176a-4075-95c1-7638bb6885c0`. Fold-in DOCS TX `205fba7b-98aa-4823-93b2-e02d1c9cc353`. Implement starts a **FEATURE** TX on **go**.
- **Isolation:** Do **not** `cargo install`. Do **not** write live `.env`. Do **not** `set-alias 7d97a456 … AI-Brains`. Do **not** reopen T240 F2 / T255 declines. Do **not** print or commit `AI_BRAINS_KEY`.

---

## 1. Objective

Every `next:` / footer / `remediations[]` entry this track owns must be a **command the operator has not just successfully run**, or `next: none` / omit.

1. **`harness status` when wiring=ok is done.** Human omits the `next:` line. JSON `next_action` is the token `none`. Missing/partial still name `harness install --harness X --dry-run`.
2. **`project list` footer stops aliasing leftover as this repo.** Prefer the cwd path-owner when it has no alias. Never suggest the cwd git slug unless the example id **is** that path-owner.
3. **Whoami remediations stay T258.** They already name `adopt-path`. This track does not rewrite them.

That advances the north star: capture stays grant-independent; the append-only log stays SoT; the CLI tells the operator the **next** useful command instead of echoing the one they just ran or handing them a harmful alias.

No models. No new crates. No clap 5. No events.

---

## 2. Live baseline (2026-08-18)

### 2.1 Operator dogfood (this machine)

| Signal | Observation |
|--------|-------------|
| HEAD | **Plan dogfood:** `fa90981` T266 `#180` (product `src/` for the holes). **Plan commit / this fold-in:** `d4555f2` (docs only). Product `src/` unchanged since `fa90981`. |
| PATH `ai-brains` | `0.1.1` (`C:\Users\RyanB\.cargo\bin\ai-brains.exe`). Same holes as source for this track (T266 did not touch next-action). **Do not `cargo install`.** |
| This session | Agent **non-TTY**. `preflight --summary` pin count is **volatile** (plan 3046 → review 3047 → fold-in 3058). Scope `3581317d`. Grants **0 of 3**. Five harnesses **wiring=ok**. Preflight harness block **already omits** `next:` on ok. Do **not** lock the pin count. |
| `harness status` (PATH) | 5/5 `present=yes wiring=ok`. **Each row** `next: ai-brains harness status`. Trailer still prints five `X ready: … install --harness X --dry-run`. **Live hole.** |
| `project whoami --format human` | `effective`/`env`/`path`/`detect` = `3581317d`. `shell_project_id` leftover `7d97a456`. `mismatch: false`. `remediations: (none)`. T258 F10 already shipped — no “run whoami”. |
| `project list` footer | `27 project(s) have no alias.` / `Example: ai-brains project set-alias 7d97a456-f2f4-43ea-1f13-211af684ad37 AI-Brains`. **Live hole.** |
| List rows | Leftover `7d97a456`: **18032**, alias empty, `ProjectListDetail.path` = first path `C:\dev\crawlx`. **`list-paths --project 7d97a456` = 11 roots** (crawlx, dedupe, degoo, family, gimp, homebrew-tap, kinledger, ledgerful-action, ledgerful-frontend, ledgerful-web, wondermaker). Path owner `3581317d`: alias already `C:\dev\ai-brains`. `unaliased_count=27`. Only one **unaliased list row** shows a path column — that is first-path, **not** `path_count==1`. F3(2) does **not** pick leftover on this vault; F3(3) orphan wins. |
| Last GitHub PR | [#180](https://github.com/Ryan-AI-Studios/AI-Brains/pull/180) T266. Issue comments **[]**. Review comments **[]**. Reviews **[]**. Open PRs on `main`: none (Dependabot only, other heads). |
| #179 Cursor leftover | T272 still true (`preflight.rs:329` insert before `:336` cap). Already a Pending placeholder. **Do not remint.** |
| Identity / grants | Daily Scope rebound to path owner. 0 of 3 grants is T241, not this track. |

### 2.2 Why these residuals still matter

| Residual | Why it is a product hole / why decline |
|----------|----------------------------------------|
| `harness status` self-next | clig.dev: suggest the **next** command in a workflow (`git status` suggests `git add`, not `git status`). Five ok rows that say “run status” are a dead end. JSON scripts that read `next_action` will loop. **DoD.** |
| Five `X ready: install` trailers | Same surface. Install is not the remediator when wiring=ok. **DoD as omit-when-ok.** |
| List footer leftover-as-AI-Brains | Highest-memory unaliased + cwd git slug. Leftover is a multi-root dump (`C:\dev\crawlx` first path). Aliasing it `AI-Brains` steals this repo’s name (T259 F1). Cwd owner **already has** an alias. **DoD: pick + suggestion rules, not a hardcoded leftover UUID.** |
| Whoami → whoami | **Closed by T258 F10.** Live remediations on mismatch name `adopt-path`. Affirm + keep AC7 green. **Do not re-implement.** |
| Doctor “Run `project whoami`” | T240 SOOT. Doctor is not `whoami`. **Decline.** |
| Install success `next: harness status` | You just ran **install**. Status is the real next. T235 F40. **Decline changing.** |
| Preflight ok rows | Already omit `next:`. **Regression only.** |
| Shared “don’t next yourself” helper | Only one self-next factory (`next_action_for`). Footer is a different bug. **Decline F4.** |

### 2.3 Code truth

| Item | Location | Notes |
|------|----------|-------|
| `next_action_for` | `crates/ai-brains-cli/src/harness/wiring.rs` **:286–308** | `WiringStatus::Ok => "ai-brains harness status"`. Absent = `n/a (not installed on machine)`. Missing/Partial/Unknown = `install --harness {id} --dry-run`. |
| Status JSON | `HarnessStatus.next_action` + `StatusReport` schema_version **1** | T235 F21 required key. Tests only assert `!next_action.is_empty()`. |
| Human status | `harness.rs` `run_status` **:58–59** | Prints `next:` for every **present** harness (ignores wiring). |
| Ready trailer | `harness.rs` **:64–68** | Five static `X ready: install --dry-run` lines always. |
| Install/uninstall next | `harness.rs` **:162, :305** | `next: ai-brains harness status` after write. **Keep.** |
| Preflight harness block | `preflight.rs` `format_harness_summary_lines` **:832–837** | `next:` only for Missing/Partial/Unknown. Ok already silent. |
| List footer | `project.rs` `print_unaliased_footer` **:104–122** | `unaliased[0]` = highest memory (list is DESC). |
| Suggestion | `footer_alias_suggestion` **:125–136** | cwd `get_git_repo_slug` else `my-project`. Ignores whether target owns cwd. |
| Sanitize | `sanitize_alias_suggestion` **:399** | Keep. Footer + T206 env-fallback both use it. |
| Path owner | `resolve_path_alias_for_location` **:265** (`pub(crate)`) | Reuse. Do not fork. |
| Path counts | `QueryStore::list_path_aliases` | `(ProjectId, path)` rows. Count in CLI. **Do not** grow `ProjectListDetail`. |
| `ProjectListDetail` | `ai-brains-store` | `alias`, `path` (first path only), `memory_count`. Order memory DESC. |
| Whoami remediations | `project.rs` **:763–781** | T258: adopt-path + `AI_BRAINS_PROJECT_ID=` + honesty. No `` `ai-brains project whoami` ``. |
| T212 hermetics | `tests/project_list_labels.rs` AC3–AC5 | Footer on stderr; JSON has no footer; empty vault no footer. **Stay green.** |
| T258 hermetic | `tests/project_adopt_path.rs` AC7 | Whoami remediations name adopt-path. **Stay green.** |
| Hotspot | `project.rs` **#1** (3.970, **1511** total / **1368** non-blank) | Extract footer to **`project_list_footer.rs`**. AC12 measures **total** lines vs HEAD. |
| `get_git_repo_slug` | `project.rs` **:245** private | Footer uses **`collect_git_identity`** (`:200`, already `pub(crate)`). Do not require widening the wrapper. |
| `preflight.rs` | hotspot **#9** (2002 non-blank) | Ok-row unit **required** (AC15). Existing units: absent + Missing only. Do **not** rewrite summary (T265). |
| `governed_common.rs` | hotspot **#5** | Do not edit. |
| Contracts / T180 | no `next_action` DTO | CLI-local `StatusReport` only. |

### 2.4 Dependency / standards research (2026-08-18)

| Pin / source | Workspace / live | Action |
|--------------|------------------|--------|
| `clap` | workspace **4.5** / lock **4.6.1** / crates.io **4.6.6** | **No bump.** No new flags. Snapshot — re-verify at execute. |
| `serde_json` | lock **1.0.150** / crates.io **1.0.151** | **No bump.** StatusReport already serde. |
| `uuid` | workspace **1.13** / lock **1.23.1** | **No bump.** No leftover UUID constant. |
| rustc / edition | **1.95.0** / **2024** | Unchanged. |
| workspace version | **0.1.1** | **No bump.** |
| clap 5 / new crates | — | **Forbidden / zero.** |
| [CLIG — Ease of discovery](https://clig.dev/#ease-of-discovery) | Suggest what command to run **next** | Ok status has no next. Missing has install `--dry-run`. |
| [CLIG — Suggest commands](https://clig.dev/#output) | `git status` suggests `git add` / `git restore`, not `git status` | Self-next is the anti-pattern. |
| [CLIG — Saying just enough](https://clig.dev/#saying-just-enough) | Too much output drowns the useful line | Omit ok `next:` and ok install trailers. |
| [CLIG — Future-proofing](https://clig.dev/#future-proofing) | Changing **human** output is usually OK; JSON keys stay | Human omit is OK. JSON **key** `next_action` stays; **value** `none` is documented. |
| T180 / PROTOCOL-COMPAT | compact 2-key freeze; harness StatusReport is CLI-local | Additive sentence: `wiring=ok` → `next_action: "none"`. No contracts crate. |
| SQLCipher / schtasks | N/A — presentation only | N/A (written). |

---

## 3. Frozen decisions

| ID | Decision |
|----|----------|
| **F0 — Go gate** | Plan-only until user **go**. Planning is DOCS. Implement starts a **FEATURE** TX. |
| **F1 — Harness ok is done** | `next_action_for(id, WiringStatus::Ok)` returns the exact token **`none`**. Human `harness status` **omits** the `next:` line when wiring is Ok **or** `next_action == "none"`. Absent stays `n/a (not installed on machine)` and is **not** printed (already gated on `present`). Missing / Partial / Unknown stay `ai-brains harness install --harness {id} --dry-run`. |
| **F2 — Whoami remediations** | **Affirm T258 F10.** Do **not** edit `build_whoami_report` remediations. Existing `project_whoami__mismatch__remediations_name_adopt_path` stays green. |
| **F3 — List footer pick** | Human stderr only (T212). Among `alias.is_empty()` rows, pick in this order: (1) cwd path-owner (`resolve_path_alias_for_location`); (2) first unaliased whose `list_path_aliases` count == **1**; (3) first unaliased with count == **0** (orphan); (4) leftover / shared last (`unaliased[0]`). Empty unaliased → no footer (T212 AC4). |
| **F3b — List footer suggestion** | Use cwd git slug **iff** the picked id **equals** the cwd path-owner. Otherwise: sanitized basename of the target’s registered path (last non-empty path component) if any, else `my-project`. **Never** emit `set-alias <id> <cwd-slug>` when `<id>` is not the cwd path-owner. **No** hardcoded leftover UUID. |
| **F4 — Decline generic helper** | Do **not** add a workspace-wide “don’t next yourself” helper. One factory (`next_action_for`) + one footer picker. |
| **F5 — JSON keys frozen** | `StatusReport` / `HarnessStatus` keys + `schema_version: 1` stay. `next_action` remains a required non-empty string. Value **`none`** for Ok is the contract change. Do **not** omit the key. Do **not** add a footer to `project list --format json`. |
| **F6 — Ready trailer** | The five `X ready: install --dry-run` lines print only for harnesses that are **present and not Ok**. If every present harness is Ok, print **none** of them. Message-only / install-ready one-liners after that stay. |
| **F7 — Install/uninstall next** | After a successful install or uninstall, keep `next: ai-brains harness status`. That is a different command. Backend-pending / Codex `/hooks` lines stay. |
| **F8 — Preflight** | `format_harness_summary_lines` already omits Ok `next:`. **Required** unit `format_harness_summary_lines__ok__omits_next` (AC15). Do **not** rewrite preflight pretty/summary JSON (T265). |
| **F9 — No leftover UUID** | Product code must **not** special-case `7d97a456-f2f4-43ea-1f13-211af684ad37`. The F3/F3b rules make leftover-as-AI-Brains impossible on this machine and any other. |
| **F10 — Module** | New `crates/ai-brains-cli/src/commands/project_list_footer.rs`. `pick_unaliased_footer_target` + `footer_alias_suggestion` are **pure** (`pub(crate)`, no `AppContext`, no IO). Print wrapper does I/O. Slug/owner come from **`collect_git_identity`** + `resolve_path_alias_for_location` (already `pub(crate)`). Do **not** require widening private `get_git_repo_slug`. Keep `sanitize_alias_suggestion` in `project.rs` (T206). **Do not** grow detect / whoami / `display_label`. **Do not** edit `context.rs`. **Do not** grow `ProjectListDetail`. |
| **F11 — Decline peers** | T265 envelope; T268 scan-roots; T269 Router split; T270 classify; T271 ledger pane; T272 `safety_ids`. |
| **F12 — Decline T240 F2 / T255** | No silent Scope/`.env`. No doctor 16th. No product `.cmd`. No live `schtasks`. |
| **F13 — clap / crates** | No clap 5. No lock bumps. No new crates. Workspace **0.1.1**. Family B stays (T266 F25): do **not** add `--format auto` to `harness status` or `project list`. |
| **F14 — Docs** | CAPABILITIES: harness ok → omit/`none`; list footer pick + “cwd slug only for cwd owner”. Root CHANGELOG T267. PROTOCOL-COMPAT **additive** one sentence on `next_action: "none"` for ok. CLI-EXIT-CODES unchanged. `project list` `after_help` is **already neutral** (`main.rs:2419` — “a set-alias example is printed on stderr”). Optional one-liner polish only; **not** a correctness DoD. |
| **F15 — Tests** | Naming `function_or_feature__condition__expected_result`. Units for `next_action_for` + footer pick/suggestion. Hermetics in new `tests/next_action_honesty.rs` (or additive in `project_list_labels.rs` / harness suite). T212 / T235 / T258 / T240 suites stay green. No `unwrap`/`expect`/`panic` in production. |
| **F16 — Cross-model** | FEATURE / operator remediator. After Phase-1 review clean, run read-only `codex-review`. |
| **F17 — Debt file** | `conductor/ISSUES.md` does **not** exist. Deferrals → `conductor/deferred.md`. |
| **F18 — PATH-behind** | Do **not** `cargo install` unless the user asks. Tests/manual AC use `cargo run` / hermetic bin. |
| **F19 — Capture independence** | Presentation + docs only. No events. No `set-alias` write. No models. |
| **F20 — Stop-before** | Even after go: do not write `.env`, do not live `set-alias` leftover, do not `rebind-path --write`, do not mutate Nightly/Router. |
| **F21 — Doctor / T240 warn** | Doctor remediation and identity warn that name `project whoami` stay. Those commands are not `whoami` / `harness status` / `project list`. |
| **F22 — T212 footer chrome** | Still stderr. Still `N project(s) have no alias.` + one `Example: ai-brains project set-alias <uuid> <label>` when a target exists. JSON still no footer. |

---

## 4. Acceptance criteria

| AC | Proof |
|----|-------|
| **AC1** | Unit: `next_action_for(Grok, Ok) == "none"`. `next_action_for(Grok, Missing)` contains `install --harness grok --dry-run`. Absent still starts with `n/a`. |
| **AC2** | Hermetic: all five harnesses present + wiring=ok. `harness status` (human) exit **0**, stdout has **no** `next: ai-brains harness status`. |
| **AC3** | Same fixture `--format json`: parses `schema_version == 1`; every `wiring == "ok"` row has `next_action == "none"`; keys `id`/`wiring`/`next_action` present. |
| **AC4** | Hermetic: one harness present + missing marker. Human contains `next: ai-brains harness install --harness <id> --dry-run`. JSON that row’s `next_action` is that command (not `none`). |
| **AC5** | Existing `project_whoami__mismatch__remediations_name_adopt_path` stays green. Combined remediations still omit `` `ai-brains project whoami` `` and `project list`. |
| **AC6** | Hermetic: leftover-shaped project (high memory, path `C:\dev\crawlx`, no alias) **and** cwd registered to an **unaliased** path-owner. Git slug `AI-Brains` (or fixture slug). `project list` stderr `Example:` contains the **path-owner** id + the slug, and does **not** pair leftover id with the slug. |
| **AC7** | Hermetic **this-vault shape**: leftover-shaped unaliased with **≥2** registered paths + cwd path-owner **already aliased** + fixture slug `AI-Brains` + **one unaliased orphan**. Stderr `Example:` names the **orphan** id (F3(3)). Does **not** contain `set-alias <leftover-id>` and does **not** contain leftover+`AI-Brains`. Named: `project_list__footer__multipath_leftover_plus_orphan__picks_orphan`. |
| **AC8** | `project_list_labels` AC3 / AC4 / AC5 stay green (one unaliased still named; empty no footer; JSON no footer). |
| **AC9** | Same fixture as AC2: human stdout has **no** `install --harness` ready-trailer lines. AC4 fixture still prints the install remediator for the missing id. |
| **AC10** | **Additive** hermetic `harness_install__success__next_is_status` — **no** existing test asserts that string today (`install.rs` only matches `InstallOutcome::Installed`). After `harness install --harness <ready> --yes` on a temp home, stdout contains `next: ai-brains harness status`. |
| **AC11** | Docs: CAPABILITIES harness ok = omit/`none`; list footer F3/F3b; CHANGELOG T267; PROTOCOL-COMPAT one sentence on `next_action: "none"`. `after_help` polish is **optional** (F14). |
| **AC12** | No contracts DTO. No pin bumps. No new crate. Grep product `src/` has no `7d97a456-f2f4-43ea-1f13-211af684ad37`. `project.rs` **total** line count (`(Get-Content).Count`, HEAD **1511**) does not grow. |
| **AC13** | Manual (source bin): `harness status` on this machine omits per-row `next:` for the five ok harnesses. `project list` stderr example is **not** leftover+`AI-Brains`. Do **not** pin. Do **not** `cargo install`. Do **not** write `.env`. |
| **AC14** | Units (no vault, **pure**): `pick_unaliased_footer_target` + `footer_alias_suggestion` cover F3 order and F3b (cwd slug only when ids match). |
| **AC15** | Unit `format_harness_summary_lines__ok__omits_next`: Ok row prints `wiring=ok` and does **not** contain `next:`. Missing sibling (existing) still prints install `--dry-run`. |
| **AC16** | Hermetic last-resort: leftover-shaped unaliased (one or more paths) + cwd owner **aliased** + slug `AI-Brains` + **no** orphan / no other unaliased. `Example:` is leftover id + path basename (`crawlx`), **not** `AI-Brains`. Named: `project_list__footer__leftover_only__basename_not_cwd_slug`. |

---

## 5. Design notes

### 5.1 `next_action` token

```text
ok              → "none"
absent          → "n/a (not installed on machine)"
missing|partial|unknown → "ai-brains harness install --harness {id} --dry-run"
backend_pending → keep today’s “backend pending ({track}); see Docs/CAPABILITIES.md”
```

Human printer: skip `next:` when the token is `none`. Do not print `next: none` (noise). JSON keeps the key so scripts can branch on `== "none"` without a missing-field case.

### 5.2 Footer pick (pure)

```text
candidates = projects where alias.trim() is empty
if candidates empty → no footer
if cwd_owner in candidates → that row
else first candidate with path_count == 1
else first candidate with path_count == 0
else candidates[0]   // still memory-DESC from list_projects_detail
```

`path_count` is computed from `list_path_aliases` (already loaded or one extra query). Do not add a SQL column this track. Do **not** treat `ProjectListDetail.path` as count — that field is first path only. Live leftover has **11** aliases, so F3(2) skips it and F3(3) picks the first orphan.

### 5.3 Footer suggestion (pure)

```text
if Some(cwd_owner) == target.id && slug is some non-empty sanitized → slug
else if target.path is some → sanitized last path component (skip empty / drive-only)
else → "my-project"
```

Never return the cwd slug for a non-owner. That is the entire leftover-as-AI-Brains fix.

### 5.4 Why not hardcode leftover?

`7d97a456-…` is **this vault’s** dump id. Shipping it in product would miss the next dump and couple CLI to one operator. F3/F3b are the general rule T259 F1 asked for.

### 5.5 Hotspot

`project.rs` is ledgerful hotspot **#1**. New algorithm + path-count join belongs in `project_list_footer.rs`. `list()` keeps one call. Units live next to the helper.

---

## 6. Non-goals

- clap 5 / lock bumps / new crates / Family A `auto` on harness or `project list`
- T258 remediations rewrite / `project use` / silent `.env`
- T259 leftover split, memory move, live `rebind-path --write`
- T265 preflight `{text, word_count}` envelope
- T268 scan-roots parent / `--root` / already-registered suggestion
- T269 nightly vs Router
- T270 retention classify
- T271 sync query ledger pane
- T272 `--global` Safety skip
- T240 F2 / T255 doctor 16th / product `.cmd` / live tasks
- Doctor / identity-warn “run `project whoami`”
- Hardcoded leftover UUID
- Workspace-wide next-action helper
- Color, pager, `comfy-table`
- `cargo install` / live `set-alias` leftover / printing `AI_BRAINS_KEY`

---

## 7. Verification plan

TDD: failing units + hermetics first (Phase 1), then `next_action_for` + human omit + trailer (Phase 2), then footer extract (Phase 3), then docs (Phase 4).

| Phase | Proof |
|-------|-------|
| Red | AC1 / AC2 / AC3 / AC6 / AC7 / AC10 / AC14 / AC15 / AC16 fail on today’s strings |
| Green harness | F1 / F5 / F6; AC1–AC4 / AC9 / AC10 / AC15 |
| Green footer | F3 / F3b / F10; AC6–AC8 / AC14 / AC16 |
| Freeze | AC5 whoami; AC8 T212; AC10 install next |
| Docs | AC11 / AC12 |
| Manual | AC13 source bin, classify-only |
| Gate | fmt / clippy `-D warnings` / targeted nextest / deny / audit on go. Full workspace gate at finalize. |
| Review | `review.md` then `codex-review` (F16) |

---

## 8. Risk

| Risk | Mitigation |
|------|------------|
| Scripts parse `next_action` and exec it in a loop | Value becomes `none` (not a command). Document in CAPABILITIES + PROTOCOL-COMPAT. |
| Scripts grepped `next: ai-brains harness status` on ok status | Human change is allowed (CLIG). Install-success path still prints that line (F7 / AC10). |
| T212 AC3 over-fit to leftover-first | AC8: single-unaliased fixture still names that id. AC6/AC7/AC16 are the new cases. |
| Treating leftover first-path as `path_count==1` | Live leftover has **11** aliases. AC7 requires ≥2 paths + orphan. |
| Hotspot `project.rs` churn | F10 extract. AC12 line-count does not grow. |
| Accidental leftover UUID constant | F9 / AC12 grep. |
| Accidental T265 / T268 / T272 steal | F11. Stop-before if a review asks to flip those. |
| PATH-behind | F18. Manual AC uses `cargo run`. |

---

## 9. Deferred absorb / decline

| Item | Disposition |
|------|-------------|
| Audit harness self-next; list leftover-as-AI-Brains (8/6) | **Absorb** F1 / F3 / F3b / F6 / AC2–AC4 / AC6–AC7 / AC9 / AC13 |
| T259 F3 / closeout: footer leftover-as-AI-Brains | **Absorb** F3 / F3b / F9. Algorithm lives here. |
| T212 F8 copy-paste set-alias footer | **Partial** — chrome stays (F22); pick/suggestion change |
| T258 F10 whoami remediations (T267 F2 slice) | **Affirm** F2 / AC5 — already shipped; do not re-edit |
| T258 closeout: T267 list footer unchanged | **Absorb** — this track is that leftover |
| T235 F40 install/uninstall next | **Affirm** F7 / AC10 |
| T266 F13 / F25 harness Family B + footer decline | **Absorb** presentation; **affirm** Family B (F13) |
| T266 F25 no `value_parser` on harness | **Decline** — still Family B; not this track |
| Shared “don’t next yourself” helper (stub F4) | **Decline** F4 — one factory |
| Doctor / T240 “run `project whoami`” | **Decline** F21 |
| T265 envelope / T268 scan / T269 / T270 / T271 | **Decline** F11 |
| T272 #179 `safety_ids` | **Decline** F11 — still true; already minted |
| last-PR Cursor #180 | **N/A** — comments/reviews empty |
| T240 F2 / T255 bag | **Decline** F12 |
| R-CI-BRANCH / MSI / packaging | **Not related** — packaging |
| `anyhow` RUSTSEC-2026-0190 allowlist | **Not related** |
| Connector cursor / CE / DataKey rotation | **Not related** |
| Daily 0 of 3 grants | **Not related** — T241 |

---

## 10. Implement order (on go)

1. Phase 0 re-verify pins + deferred rescan + confirm `next_action_for(Ok)` still returns `harness status` and list footer still leftover+`AI-Brains`.
2. Red: AC1 / AC2 / AC3 / AC6 / AC7 / AC10 / AC14 / AC15 / AC16 tests.
3. Green harness: `none` token + human omit + trailer F6 + AC15 unit.
4. Green footer: extract module; F3/F3b via `collect_git_identity`; T212 stay green; AC7 + AC16.
5. Docs: CAPABILITIES + CHANGELOG + PROTOCOL-COMPAT sentence. after_help optional.
6. Targeted clippy/nextest; Phase-1 review; codex-review; full gate; publish.

---

## 11. Soft residuals

| Residual | Notes |
|----------|-------|
| Harness status no `value_parser` (T266 F25) | Family B. Soft only. |
| PATH `cargo install` | F18. |
| Live leftover still owns many `C:\dev\*` roots | T259 operator rebind. Not this track. |
| Daily 0 of 3 grants | T241. |
| T268 scan-roots already-registered suggestion | Peer. |
| Codex `/hooks` next on install | Keep (F7). Not self-next of `status`. |

---

## 12. Touch map

| Path | Why |
|------|-----|
| `crates/ai-brains-cli/src/harness/wiring.rs` | `next_action_for` Ok → `none` + AC1 unit |
| `crates/ai-brains-cli/src/commands/harness.rs` | Omit `next:` when none/Ok; F6 trailer |
| `crates/ai-brains-cli/src/commands/project_list_footer.rs` | **New.** Pick + suggestion + print |
| `crates/ai-brains-cli/src/commands/mod.rs` | `pub mod project_list_footer` |
| `crates/ai-brains-cli/src/commands/project.rs` | Replace footer body with call; do not grow whoami/detect |
| `crates/ai-brains-cli/src/commands/preflight.rs` | **Required** AC15 Ok-row unit only (no summary rewrite) |
| `crates/ai-brains-cli/src/main.rs` | `after_help` optional one-liner (F14) |
| `crates/ai-brains-cli/tests/next_action_honesty.rs` | New hermetics AC2–AC4 / AC6 / AC7 / AC9 / AC10 / AC16 |
| `Docs/CAPABILITIES.md` | Harness ok + list footer |
| `Docs/PROTOCOL-COMPAT.md` | Additive `next_action: "none"` |
| `CHANGELOG.md` | T267 row |
| `conductor/conductor.md` | T267 Planned note (status stays Pending until implement) |
| `conductor/deferred.md` | Absorb/decline |
| **Do not touch** | `governed_common.rs`; `context.rs`; `ai-brains-retrieval`; `ai-brains-contracts`; `project.rs` remediations; doctor whoami string; graph next; nightly; scan-roots behavior |

---

## 13. AI fold-in disposition (2026-08-18)

Source: `agy-review.md` + `opencode-review.md`. No grok / claude / `codex-plan-review.md`. **B 0 / M 0.** Online research: **N/A** for new APIs (no pin/DTO change). Live re-verify: leftover `list-paths --project 7d97a456` = **11** roots; `project.rs` **1511** total / **1368** non-blank; `get_git_repo_slug` still private; no install-success next-string hermetic; `format_harness_summary_lines` has no Ok unit; `after_help` already neutral.

### Agy

| ID | Verdict | Action |
|----|---------|--------|
| **m1** HEAD `fa90981` vs `d4555f2` | **Agree** | §2.1: plan dogfood vs plan/fold-in commit |
| **m2** preflight Ok-row unit | **Agree** | **F8** required + **AC15** `format_harness_summary_lines__ok__omits_next` |
| **O1** pure picker/suggestion | **Agree** | **F10** — no `AppContext` / no IO on pick + suggestion |

### OpenCode

| ID | Verdict | Action |
|----|---------|--------|
| **m1** `project.rs` 1368 vs 1511 | **Agree** | §2.3 + **AC12**: 1511 total / 1368 non-blank; measure total |
| **m2** pin count 3046 vs 3047 | **Agree** | §2.1: volatile (fold-in **3058**); do not lock |
| **m3** AC10 “existing hermetic” missing | **Agree hard** | **AC10** additive `harness_install__success__next_is_status` |
| **O1** widen `get_git_repo_slug` | **Agree** (cheaper form) | Call `collect_git_identity` (already `pub(crate)`). Widening the wrapper is optional equivalent |
| **O2** ledgerful doctor hygiene | **Decline** | Not T267. Same standing hygiene as T266 fold-in |
| **O3** `after_help` already neutral | **Partial** | **F14** / **AC11**: optional one-liner; not correctness DoD |
| “leftover `path_count==1` so F3(2) picks it” | **Decline** (false) | Live leftover has **11** aliases. **AC7** = multi-path + orphan; **AC16** = leftover-only last resort |

### Pins locked by fold-in

1. **AC10** is additive and named. Do not hunt an existing install-success next assert.
2. **AC15** Ok-row preflight unit is required (not “if missing”).
3. **AC7** leftover fixture must have **≥2** paths + an orphan. **AC16** covers leftover-only basename.
4. **AC12** uses **total** line count (HEAD 1511).
5. Footer slug/owner via **`collect_git_identity`**, not a required `get_git_repo_slug` visibility bump.
6. `after_help` is optional polish. CAPABILITIES + CHANGELOG + PROTOCOL-COMPAT stay required.
7. §2.1: `fa90981` product vs `d4555f2` docs. Pin count is volatile.

**Planning + fold-in 2026-08-18.** Still **plan-only until go**.
