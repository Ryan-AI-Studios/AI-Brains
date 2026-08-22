# T280 — Policy deny/show hints must match doctor (omit `--scope` when context is authoritative)

- **Track ID:** T280-PolicyHintOmitScope
- **Status:** **Planned** (Pending until **go**)
- **Category:** UX / HONESTY
- **Owner:** Grok
- **Source:** Live CLI audit 2026-08-21 — `policy show` **8/7**, `policy check` **7/7**; deny `details.hint` still `bootstrap --scope …` while doctor says omit `--scope`. Placeholder minted with T274–T284.
- **Depends on:** T210 ✅ bootstrap; T226 ✅ soft-resolve; T241 ✅ doctor grants + SHORT/LONG SOOT; T243 ✅ dual-site HINT freeze (lifted here); T275 ✅ grant-wall (HINT left here)
- **Blocks / feeds:** Agents can copy deny/show remediator without inventing a redundant `--scope …`. Soft-resolve (T226) already fills scope when context is authoritative. Nightly dual-probe **T281**. `context --show` leftover **T282**. `project list` cwd-first **T283**.
- **Absorbs:** Placeholder problem text + Manual DoD; deferred.md “deny/`policy show` `--scope …` vs doctor omit”; T275 F11 leftover (HINT/NEXT_STEP wording); T241 F14 leftover (“Markdown T227 line may stay”); T243 AC12 freeze **lifted** to the new string
- **Not absorbed (DoD):** Runtime split of HINT by resolved context; merging HINT into SHORT; T226 O1 shared resolve wrapper; clap after_help rewrite (already dual examples); live operator `policy bootstrap` without owner confirm; T275 grant-wall / auto-grant; T281–T283; T240 F2; leftover `7d97a456` rebind; clap 5; rusqlite 0.40; DTO keys; doctor 16th check
- **Research date:** 2026-08-22 (plan dogfood HEAD `83080ff` T279 `#195`; product `src/` = T279). Fold-in against `f35884e` (docs-only; crates identical to `83080ff`).
- **AI fold-in:** 2026-08-22 `agy-review.md` + `opencode-review.md`. **B 0 / M 0.** **Already (Agy):** O1 F19/AC11; O2 AC5. **Agree (Agy):** m1 F33/`assert_eq!` three crates + hoist CP const; m2 AC4 grant-wall order. **Agree (OpenCode):** O1 T210 AC8 fn `:548`; F1 length **172**. **Affirm:** #195 N/A; no T285. Disposition **§13**.
- **Ledger:** planning DOCS TX `e51b3b28-d885-46cd-b622-3a7b82ae489a`. Fold-in DOCS TX `6c90e5c4-005a-4409-9aa5-5fc665635539`. Implement starts a **FEATURE** TX on **go**.
- **Isolation:** Do **not** `cargo install`. Do **not** write live `.env` (T240 F2). Do **not** bootstrap the live operator vault unless the owner confirms at **go**. Do **not** leftover-rebind. Do **not** grow hotspot `project.rs` / CLI `preflight.rs` / `doctor.rs` / `sync.rs` / `policy_cmd.rs`. Do **not** print or commit `AI_BRAINS_KEY`. Do **not** enable `AI_BRAINS_GOVERNED_BRIEFING`. Do **not** live `safety sync` without `--dry-run`, `retention apply --confirm`, or `graph rebuild`.

---

## 1. Objective

1. **Deny remediator must not require `--scope …` as the only form.** `policy check` / discovery-list `POLICY_DENIED` `details.hint` still copies `ai-brains policy bootstrap --scope …`. Doctor already says omit `--scope` when project context is authoritative. Agents paste the deny string and pass a redundant or wrong scope.
2. **Briefing markdown next-step must match briefing JSON.** JSON `denial_hint` is already T241 SHORT (no `--scope`). Markdown `BRIEFING_DENIED_NEXT_STEP` still uses the T227 `--scope …` line. Unify markdown next to SHORT.
3. **`policy show` next_step already omits `--scope`.** Affirm T241 SHORT. Do not add `--scope` back. Do not steal doctor LONG into show/preflight.
4. **North star.** Capture independence: ungoverned `recall` / preflight never require grants. No models. No new events. T180 / briefing packet keys frozen. Dual-site HINT (CLI + daemon + CP `query.rs`) stays three byte-equal copies — daemon cannot depend on CLI.

This unblocks the daily product: T226 made `--scope` optional on show/check; T241 made doctor/show discoverable; T275 made Denied human a grant wall. The remaining usefulness hole is **copy-paste honesty** — the deny envelope still trains `--scope …`.

---

## 2. Live baseline (re-scan 2026-08-22)

### 2.1 Operator dogfood (this machine)

| Signal | Observation |
|--------|-------------|
| HEAD | **Plan dogfood:** `83080ff` T279 squash `#195`. Tree **CLEAN**. `origin/main` = HEAD (`git rev-list --left-right --count origin/main...HEAD` = `0 0`). |
| PATH `ai-brains` | `C:\Users\RyanB\.cargo\bin\ai-brains.exe` mtime **2026-08-21 05:55**, 25 368 576 bytes, **0.1.1**. **T270** on PATH (before T274–T279). **HINT string is unchanged since T210** — PATH is valid for this hole. **Do not `cargo install`.** Tests/manual AC use `cargo run` / hermetic. |
| `preflight --summary` | Scope `C:\dev\ai-brains` (`3581317d`). Pinned **3547**. In-context **0/0/0**. Grants **0 of 3** (T275 hermetic; live not bootstrapped). Capture independence holds. SHORT remediator (no `--scope`). |
| `policy show` (JSON) | `grants: []` + `next_step` = T241 **SHORT** (`bootstrap --dry-run` then `bootstrap`). **No `--scope`.** Audit 8/7 is **already closed** for show next_step. Affirm freeze. |
| `policy check --capability ReadEvidence` | Exit **3** `POLICY_DENIED` + `details.hint` = `… bootstrap --scope …` (or check with `policy show --scope …`). **This is the 7/7 hole.** |
| `doctor --summary` | `policy_grants` **warn** `discovery grants empty (0 of 3)` + **LONG** SOOT (`omit --scope when project context is authoritative`). Also backup_recent / recovery_kit_event / graph_density. status=degraded. Matrix 15. |
| `briefing project --format human` (PATH) | Denied + `BRIEFING_DENIED_NEXT_STEP` (`--scope …`). Then **`_None_`** Decisions/Conclusions — **T275 PATH-behind** (grant-wall is source-only until install). **Not this DoD.** Source `renderer.rs` already has T275 grant-wall **and** T227 `--scope …` next. T280 owns the next-step string. |
| `evidence list --format json` | Exit **3** + same HINT `--scope …`. |
| Last GitHub PR | [#195](https://github.com/Ryan-AI-Studios/AI-Brains/pull/195) T279 (2026-08-22). `gh pr view --comments`, `/reviews`, `/comments`, `issues/195/comments` all **empty**. **last-PR Cursor: N/A.** Open PRs: Dependabot remotes only (`#61` rusqlite 0.40.2, `#62` chrono 0.4.45, actions). **No leftover to mint. No T285.** |
| Prior #188 Bugbot | **T284 Completed** `#193`. Not this track. |
| Identity / doctor | ledgerful doctor 4 warn (legacy `.changeguard` / sig-pin / timings / :8081). **0 pending / 0 drift.** Hotspot **#1** `project.rs` (displayScore **3.935**, volatile). `governed_common.rs` **#5** (2.599) — **required const edit**. CLI `preflight.rs` **#7** (2148 lines) — **do not grow.** `doctor.rs` **1855** — **do not grow.** |
| `ISSUES.md` | **Does not exist.** |

### 2.2 Why this still matters

| Residual | Why it is still a product hole / why decline |
|----------|----------------------------------------------|
| Deny HINT `--scope …` | T210 pointed HINT at bootstrap. T226 made `--scope` optional when context is authoritative. T241 LONG tells operators to omit. HINT still requires the ellipsis form. Agents copy deny JSON. **DoD.** |
| Briefing markdown vs JSON | JSON `denial_hint` = SHORT (T241 F7/F14). Markdown next = T227 `--scope …`. T241 F14 explicitly left markdown. T275 F11 froze it for T280. **DoD: markdown = SHORT.** |
| `policy show` next_step | Already SHORT. Audit 8/7 was HINT/markdown, not show JSON. **Affirm. Do not restyle SHORT.** |
| Runtime two-string HINT | Stub “hermetic both arms” is **fail_usage vs deny**, not two deny templates. `--no-project-context` without `--scope` never reaches POLICY_DENIED (T210 AC8 / T226 fail_usage). Daemon/CP do not share CLI context. Static omit-parenthetical is honest. **Decline runtime split.** |
| Merge HINT into SHORT | HINT prefix is T201 “ensure a grant exists”. SHORT is `next: run …`. Keep families. HINT gains dry-run + omit parenthetical (clig + doctor). |
| Live bootstrap | T275 F10. Hermetic is DoD. Owner-confirm at go. |
| Auto-grant / `--install-grants` | T210 F13 / T241 F4. **Decline.** |

### 2.3 Code truth

| Item | Location | Notes |
|------|----------|--------|
| CLI HINT | `governed_common.rs` **`:51`** (934 lines, hotspot **#5**) | Exact: `ensure a grant for this capability exists; run \`ai-brains policy bootstrap --scope …\` (or check with \`ai-brains policy show --scope …\`)`. **Replace in place.** |
| T243 freeze | `governed_common.rs` `policy_denied_hint__wording__unchanged` **`:725`** | Exact-string assert. **Lift** expected to F1. Rename test to `policy_denied_hint__wording__omits_required_scope`. |
| `policy_denied_hint_details()` | **`:140`** | Unchanged helper; reads the const. Call sites: `policy_cmd.rs` `run_check` **`:201`**, `evidence.rs` `:158/:228`, `source.rs` `:135/:222`, `review.rs` `:110`. **Do not grow those files.** |
| Daemon twin | `ai-brainsd/src/services.rs` **`:989`** (1506 lines) | Same literal. Unit `policy_denied_with_hint__includes_details_hint` **`:1226`** substring `bootstrap` + (`policy show` **or** `policy bootstrap`). New string still matches via `policy bootstrap`. Tighten to omit `--scope …` (F1). |
| CP progressive twin | `control-plane/src/query.rs` **`:93`** (815 lines) | **Function-local** const inside the deny arm today (OpenCode). T221 F17. Same literal. **F33:** hoist to module-level file-private (daemon analog) so AC3 is `assert_eq!` without `progressive_query` I/O. Crate has **no** existing `#[cfg(test)]` in this file. |
| Progressive CLI overlay | `governed_query.rs` **`:73/:124/:132/:186/:255`** | Uses CLI `POLICY_DENIED_HINT`. T243 recall fallback stays. **Do not restyle fallback.** |
| Markdown next | `renderer.rs` `BRIEFING_DENIED_NEXT_STEP` **`:13`** (643 lines) | `--scope …`. T275 grant-wall **after** this line (`:82–85`). **Set equal to `BRIEFING_DENIED_DENIAL_HINT` (SHORT).** |
| JSON denial_hint | `BRIEFING_DENIED_DENIAL_HINT` **`:16–17`** | Already SHORT. **Do not change.** |
| SHORT | `POLICY_BOOTSTRAP_SOOT_SHORT` **`:107–108`** | Show / preflight. **Freeze.** |
| LONG | `POLICY_BOOTSTRAP_SOOT_LONG` **`:111`** | Doctor only. **Freeze.** Doctor unit **`:1320`** already `omit --scope` \|\| `authoritative`. |
| Soft-resolve | `resolve_scope_key_for_cli` | T226. **Do not rewrite.** |
| clap after_help | `main.rs` **`:1620/:2204/:2209/:2223/:2239`** | Already dual examples (with `--scope Repository:<uuid>` **and** omit when authoritative). **Freeze. No new flags.** |
| T210 AC8 | `tests/policy_bootstrap.rs` fn **`:548`** (`policy_bootstrap__no_scope_no_context__exit_2`; comment **`:546`**) | `--no-project-context` omit `--scope` → fail_usage exit 2. **Stay green** — this is the no-context arm. (OpenCode O1 — plan `:546` was the comment.) |
| T210 AC7 | `policy_bootstrap__deny_hint__contains_bootstrap` **`:526`** | Substring `bootstrap`. **Tighten** to `!contains("--scope …")` + `omit --scope` (or `authoritative`). |
| T275 grant-wall | `BRIEFING_DENIED_GRANT_WALL` / `HIDDEN` | **Do not edit.** AC16 Personal deny stays. |
| T275 AC13 | HINT byte-equal T243 unit | **Lift** with the new freeze. |
| Docs | `Docs/CLI-EXIT-CODES.md` **`:94`** | Still `bootstrap --scope …`. **Update.** CAPABILITIES progressive first-run **`:322`** still `--scope Repository:<uuid>` as a CI example — keep `--scope` example **and** omit note (F26). |
| `project.rs` | hotspot **#1** (1472) | **Do not touch.** |
| CLI `preflight.rs` | hotspot **#7** (2148) | Uses SHORT. **Do not grow.** |
| `doctor.rs` | **1855** | LONG already. **Do not grow.** |
| `policy_cmd.rs` | **387** | rustdoc `[--scope …]` is POSIX optional notation, **not** the deny ellipsis. **Leave.** |
| `ISSUES.md` | — | Does not exist. |

### 2.4 Dependency / standards research (2026-08-22) — snapshot; re-verify at execute

| Pin | Workspace / lock | Ecosystem (today) | Action |
|-----|------------------|-------------------|--------|
| `clap` | workspace **4.5** / lock **4.6.1** | crates.io **4.6.6** (2026-08-06). GitHub latest tag **v4.6.6**. **No clap 5.** | **No bump.** No new flags. after_help freeze. |
| `serde_json` | lock **1.0.150** | crates.io **1.0.151** (2026-07-20) | **No bump.** Hint is a string; keys frozen. |
| `chrono` | lock **0.4.44** | crates.io **0.4.45** (Dependabot #62 open) | **No bump.** |
| `rusqlite` | lock **0.39.0** + sqlcipher + backup | crates.io **0.40.2** (Dependabot #61; T213 L4) | **No bump.** |
| `uuid` | lock **1.23.1** | crates.io **1.25.0** (2026-08-22) | **No bump.** |
| rustc / edition | **1.95.0** / **2024** | workspace toolchain | Unchanged |
| nextest | **0.9.140** | — | Unchanged |
| workspace | **0.1.1** | — | **No bump** |
| New crates | — | — | **Zero.** No shared hint crate (daemon ↛ CLI). |

**Online / primary sources**

| Claim | Source | Takeaway |
|-------|--------|----------|
| Suggest the next command; first-run setup then real work; dry-run before state change | [clig.dev](https://clig.dev/) (current) — Human-first, Ease of discovery, Conversation as the norm | Deny hint should name `policy bootstrap --dry-run` then `policy bootstrap`, not a required `--scope …`. Doctor already does. |
| Consistency across subcommands | clig.dev | show/check/bootstrap already soft-resolve (T226). HINT must match that story. |
| clap 4 current | crates.io clap **4.6.6**; docs.rs/clap/4.6.6 | Reuse existing parsers. No clap 5. |
| Dual-site freeze | Live T243 unit + T210 AC11 daemon | Three copies stay byte-equal. HINT rewrite = this track. |
| Least privilege / no auto-grant | T210 F13; Entra least privilege (not re-opened) | Mutation stays `policy bootstrap`. |

**N/A:** SQLCipher page encrypt, schtasks, T180 DTO new keys, Windows service, llama.cpp `/health`, graph floors, Safety GLOB (T279 Completed).

**Could not verify:** live briefing after a *confirmed* operator bootstrap (stop-before). PATH briefing `_None_` is T275 PATH-behind. Source renderer is the markdown-next truth.

**ledgerful / ai-brains:** `preflight --summary` 0 of 3 grants @ **3547** pins; `policy check` HINT `--scope …`; `doctor --summary` LONG omit; `ledgerful ledger status --compact` 0 pending / 0 drift; `search "POLICY_DENIED_HINT"` → CLI `:51`, daemon `:989`, CP `:93`, `governed_query.rs` overlays. Recall of “omit --scope hint” returned T275 review-track dumps (PATH-behind T274).

---

## 3. Frozen decisions

| ID | Decision |
|----|----------|
| **F0 — Go gate** | Plan-only until user **go**. Planning is DOCS TX `e51b3b28`. Fold-in is DOCS TX `6c90e5c4`. Implement starts a **FEATURE** TX. |
| **F1 — Deny HINT omit-scope** | Replace `POLICY_DENIED_HINT` (CLI `:51` + daemon `:989` + CP `query.rs` `:93`) with **exactly**: `ensure a grant for this capability exists; run \`ai-brains policy bootstrap --dry-run\` then \`ai-brains policy bootstrap\` (omit --scope when project context is authoritative)`. Three copies **byte-equal**. Must contain `policy bootstrap` and `omit --scope`. Must **not** contain `--scope …` (U+2026) or `bootstrap --scope`. Drop `policy show --scope …` (show already SHORT; one next command — clig). |
| **F2 — Briefing markdown = SHORT** | `BRIEFING_DENIED_NEXT_STEP` becomes `BRIEFING_DENIED_DENIAL_HINT` (same const or `= SHORT` literal). Markdown next matches JSON `denial_hint`. T275 grant-wall **after** next-step stays. Personal deny stays T263 recall (F35 analog — **must not** contain project bootstrap). |
| **F3 — SHORT / LONG freeze** | `POLICY_BOOTSTRAP_SOOT_SHORT` and `_LONG` **unchanged**. Show / preflight keep SHORT. Doctor keeps LONG. Do not steal LONG into show. |
| **F4 — No runtime split** | One HINT string everywhere. `--no-project-context` omit `--scope` is **fail_usage exit 2** (T210 AC8 / T226), not a second deny template. |
| **F5 — T226 isolation** | Do **not** rewrite `resolve_scope_key_for_cli`. Soft-fill when authoritative; fail_usage when not. O1 shared wrapper stays residual. |
| **F6 — clap after_help freeze** | Dual examples already document omit. **No new flags.** `main.rs` after_help **untouched** unless a docs-only typo is found at go (none today). |
| **F7 — T275 isolation** | Grant-wall consts, `_None_` hide, hermetic System bootstrap, Personal deny **unchanged**. T275 AC13 lifts with F1. |
| **F8 — T221 isolation** | Progressive deny exit **3** + HINT + T243 recall fallback stay. Overlay reads the new const. |
| **F9 — T210 isolation** | Bootstrap mutation / discovery trio / `--dry-run` / no auto-grant **unchanged**. AC8 stay green. |
| **F10 — Live vault stop-before** | Plan-only: no bootstrap. On **go**, do **not** bootstrap the operator vault unless the owner confirms. Hermetic is sufficient DoD. Classify-only `cargo run` policy check / briefing human. |
| **F11 — Capture independence** | Recall / search / ungoverned preflight never require grants. HINT copy must not imply capture is blocked. |
| **F12 — Pins / crates** | No clap 5, no rusqlite 0.40, no chrono 0.4.45, no uuid 1.25, no new crates, workspace **0.1.1**. |
| **F13 — PATH** | Do not `cargo install` unless the user asks. |
| **F14 — Contracts** | No new required keys. `details.hint` stays a string. `denial_hint` stays optional string. E1 empty/deny shapes frozen. |
| **F15 — File growth** | HINT literal + T243 unit in `governed_common.rs`. Daemon twin + tighten AC11. CP `query.rs` **hoist** twin (F33) + unit. `renderer.rs` NEXT_STEP alias. Hermetic AC7 tighten. Docs F26. **Do not** grow `project.rs`, CLI `preflight.rs`, `doctor.rs`, `sync.rs`, `policy_cmd.rs`, `evidence.rs` / `source.rs` / `review.rs` (call sites already use the helper). |
| **F16 — Tests** | Naming `function_or_feature__condition__expected_result`. No `unwrap`/`expect`/`panic` in production. Hermetic `tempfile::tempdir`. Unicode ellipsis is **U+2026** (`…`), not `...`. |
| **F17 — Decline peers** | T281 nightly 750 ms; T282 `context --show`; T283 list cwd-first; leftover rebind; T240 F2; T255 750 ms; T263 H2; T266 JSON freeze; T275 live bootstrap; T277 live `--no-prune`; T278 live rebuild; T279 live pin; T284 live apply. |
| **F18 — last-PR Cursor** | #195 empty → **N/A**. #188 closed by T284. Dependabot `#61` rusqlite **not** this track. **No T285.** |
| **F19 — Docs** | CLI-EXIT-CODES POLICY_DENIED paragraph: bootstrap `--dry-run` then bootstrap; omit `--scope` when authoritative; `--scope Repository:<uuid>` remains valid for no-context CI. CAPABILITIES progressive first-run: add omit note. OPERATIONS already has omit. CHANGELOG T280. Skill one-liner if policy bootstrap section exists. PROTOCOL-COMPAT: no new required keys. |
| **F20 — Cross-model** | Dual-site HINT + CLI presentation is FEATURE. After Phase-1 clean, run read-only `codex-review`. |
| **F21 — Debt file** | `conductor/ISSUES.md` does **not** exist. Deferrals → `conductor/deferred.md`. |
| **F22 — PowerShell** | `;` not `&&`. |
| **F23 — Existing tests stay green** | T210 AC1–AC8 (AC7 tightened, not dropped; AC8 fn `:548`); T221 progressive deny + after-bootstrap; T226 soft-resolve; T241 doctor LONG / show SHORT / check catalog; T243 recall fallback; T263 Personal deny; T275 grant-wall + AC16; `render_project_markdown__denied__bootstrap_next_step_no_empty_authority` (order AC4: next then grant-wall then Decisions; string is SHORT). |
| **F24 — No shared crate** | Daemon stays copy-paste twin (T210 / T277 F44 analog). Drift of three copies is locked by AC1 + daemon + CP units. |
| **F25 — POSIX rustdoc** | `policy_cmd.rs` rustdoc `[--scope …]` means optional flag. **Not** the deny ellipsis. Leave. |
| **F26 — `--scope` still valid** | Explicit `--scope Repository:<uuid>` still wins (T226). clap examples keep that form. HINT must not *require* it when context is authoritative. |
| **F27 — T243 test rename** | `policy_denied_hint__wording__unchanged` → `policy_denied_hint__wording__omits_required_scope`. Comment cites T280 F1 (not T243 freeze-as-written). |
| **F28 — Daemon AC11** | Keep `bootstrap` substring. Add `omit --scope` (or `authoritative`) and `!contains("--scope …")`. `policy show` **or** `policy bootstrap` disjunction still passes via bootstrap. |
| **F29 — Classify-only live** | Manual AC uses `cargo run -p ai-brains-cli -- policy check --capability ReadEvidence` from this repo. Do **not** treat PATH T270 as proof. Do **not** bootstrap. |
| **F30 — No doctor 16th** | Matrix stays 15. LONG already omit. |
| **F31 — Progressive HINT vs SHORT** | Progressive `denial_hint` is F1 HINT (ensure a grant exists), not SHORT. Overlay may still append T243 recall fallback on stderr. JSON packet `denial_hint` stays HINT. |
| **F32 — Identity leftover** | `7d97a456` vs `fcb8a40f` is T258/T276/T282. **No T285.** |
| **F33 — Dual-site `assert_eq!` (Agy m1)** | AC1–AC3 lock **exact** F1 equality (`assert_eq!(HINT, F1_LITERAL)`), not substring-only. Hoist CP `query.rs` function-local const to **module-level** file-private (same crate; not `pub` to CLI) so the unit can name it. Daemon twin already module-level. Do **not** add a shared crate. |

---

## 4. Acceptance criteria

| AC | Proof |
|----|-------|
| **AC1** | Unit: CLI `POLICY_DENIED_HINT` `assert_eq!` F1 literal (**172** chars); does **not** contain `--scope …`; contains `omit --scope` and `policy bootstrap` and `--dry-run`. **Required red** (`policy_denied_hint__wording__omits_required_scope`). |
| **AC2** | Unit: daemon `POLICY_DENIED_HINT` `assert_eq!` F1; `policy_denied_with_hint` details.hint matches F1. **Required red.** |
| **AC3** | Unit: hoisted CP `query.rs` module-level const `assert_eq!` F1 (F33; new `#[cfg(test)]` in that file). **Required red.** |
| **AC4** | Unit: `BRIEFING_DENIED_NEXT_STEP == BRIEFING_DENIED_DENIAL_HINT`; neither contains `--scope …`; denied markdown order **Denied → NEXT_STEP → GRANT_WALL → `## Decisions`** (`find` positions strictly increasing). T275 grant-wall still present. **Required red.** (Agy m2.) |
| **AC5** | Hermetic: `policy check --capability ReadEvidence` on empty-grant vault, authoritative project context → exit **3**; `details.hint` contains F1 needles; **no** `--scope …`. **Required red** (tighten T210 AC7). |
| **AC6** | T210 AC8 `--no-project-context` omit `--scope` still exit **2** fail_usage (names `--scope`). This is the no-context arm. |
| **AC7** | T241 doctor empty grants still LONG (`omit --scope` \|\| `authoritative`). Show empty JSON `next_step` still SHORT (no `--scope …`). |
| **AC8** | T275 grant-wall units + Personal AC16 stay green. Denied markdown **must not** regain `_None_` under Decisions. |
| **AC9** | T221 progressive deny still exit **3**; packet `denial_hint` contains `policy bootstrap` and **not** `--scope …`. Recall fallback stderr stays. |
| **AC10** | Manual classify-only (`cargo run`, no bootstrap): `policy check --capability ReadEvidence` HINT matches F1 **or** PATH-behind note if binary not rebuilt — **source/hermetic is DoD.** Pass-with-observed-data. |
| **AC11** | Docs: CLI-EXIT-CODES `:94` no longer presents `--scope …` as the only form. CHANGELOG T280. CAPABILITIES omit note. PROTOCOL-COMPAT no new required keys. |
| **AC12** | No production `unwrap`/`expect`/`panic`; no clap/rusqlite bump; no DTO keys. Diff omits `project.rs` / CLI `preflight.rs` / `doctor.rs` / `sync.rs` / `policy_cmd.rs`. |
| **AC13** | T226 soft-resolve hermetic stay green (show/check omit `--scope` when authoritative). |
| **AC14** | `BRIEFING_DENIED_NEXT_STEP` / F1 HINT **must not** appear on Personal denied markdown (T263 / T275 AC16). |

---

## 5. Design notes

**Two SOOT families stay.** SHORT/LONG are discoverability (show, preflight, doctor, briefing JSON). HINT is the POLICY_DENIED envelope (check, lists, progressive, daemon). T280 does not collapse them. HINT borrows doctor’s omit parenthetical and SHORT’s `--dry-run` then apply.

**Why not runtime omit vs require?** Deny is reached only after a scope key exists (soft-fill or explicit). The useless form is “you already have context; now type `--scope …`”. Missing context never denies — it fail_usage. A second const would triple again across CLI/daemon/CP without a caller that needs it.

**Why drop `policy show` from HINT?** clig: one next command. Empty show already prints SHORT. Deny means “issue grants”, not “inspect again”. Daemon AC11’s `policy show` **or** `policy bootstrap` disjunction still passes.

**Unicode.** Production strings use `…` (U+2026), matching T210. Tests must use that character. ASCII `...` is not the hole.

**Dual-site.** Keep three copies + three units. Do not introduce `ai-brains-contracts` hint const (T180 freeze; not a DTO).

---

## 6. Non-goals

- Auto-grant on `init` / first `preflight` / doctor `--fix`
- `preflight --install-grants`
- T263 H2 pin→Approved
- Runtime context-aware HINT
- Merging HINT into SHORT/LONG
- clap 5 / rusqlite 0.40 / new DTO keys / doctor 16th check
- Live leftover rebind / `.env` rewrite / `cargo install`
- T281–T283 peers
- Shared hint crate / daemon→CLI dep
- Changing deny exit **3** or briefing deny exit **0**

---

## 7. Verification plan

1. **Red:** AC1–AC5 fail against current `--scope …` literals.
2. **Green:** three HINT copies + renderer alias + hermetic AC7 tighten.
3. Targeted: `cargo nextest run -p ai-brains-cli --test policy_bootstrap` + `-p ai-brains-cli` governed_common unit + `-p ai-brainsd` AC11 + `-p ai-brains-control-plane` query/renderer units + T275 renderer tests; clippy `-p ai-brains-cli -p ai-brainsd -p ai-brains-control-plane --all-targets -- -D warnings`.
4. Manual classify-only AC10. **No** live bootstrap.
5. Review log; FEATURE cross-model (F20).
6. Full gate before finalize. implement-track Phase 6 publish.

---

## 8. Risk

| Risk | Mitigation |
|------|------------|
| Dual-site drift (3 copies) | AC1–AC3 byte-equal F1 |
| T275 grant-wall regression | AC8 / AC4 order lock |
| T210 AC8 fail_usage broken | AC6 stay green |
| Agents still copy `--scope` from clap examples | Honest: examples keep both forms; HINT is the copied deny path |
| HINT length (**172** chars; OpenCode) | No historical 140 cap on HINT (T275 140 is grant-wall). AC1–AC3 are `assert_eq!`, not a char budget. |
| PATH-behind until install | F13; hermetic/source DoD |
| Live bootstrap by implementer | F10 Stop-Before |

---

## 9. Deferred absorb / decline

| Item | Disposition |
|------|-------------|
| Audit deny/`policy show` `--scope …` vs doctor omit | **Absorb** F1–F4 / AC1–AC7 / AC10 — show already SHORT (affirm); HINT + markdown next are DoD |
| T275 F11 / closeout “HINT still `--scope …`” | **Absorb** F1 / F2 |
| T241 F14 “Markdown T227 line may stay” | **Absorb** F2 — markdown = SHORT |
| T243 AC12 wording freeze | **Lift** F1 / F27 — new freeze |
| T210 AC7 substring-only | **Tighten** AC5 |
| T226 O1 shared resolve wrapper | **Decline** F5 |
| clap after_help dual-site | **Decline** F6 — already both forms |
| Runtime two-arm HINT | **Decline** F4 |
| last-PR Cursor #195 | **N/A** — comments/reviews empty |
| last-PR #188 Work / apply samples | **Decline** — **T284 Completed** `#193` |
| Dependabot `#61` rusqlite 0.40.2 | **Decline** F12 — **no T285** |
| T281 / T282 / T283 / leftover 11 roots | **Decline** peers |
| T240 F2 / clap 5 / DTO required keys | **Decline** F12/F17 |
| Identity mismatch quiet | **Not this track** — T258 adopt-path; leftover data T276; shell leftover T282 |
| Live operator bootstrap | **F10** |
| Auto-grant / `--install-grants` | **Decline** F9 analog / T241 F4 |

**Entire `deferred.md` scanned.** Closed/strikethrough rows stay closed. Historical CE wipe, MSI, `anyhow` allowlist, archive `changeguard` — not hint wording.

---

## 10. Implement order (on go)

1. Phase 0 re-verify HINT `:51/:989/:93`, NEXT_STEP `:13`, SHORT/LONG, T210 AC7 `:526` + AC8 fn `:548`, T243 unit, T275 grant-wall, deferred rescan, #195 still empty, pins.
2. Red AC1–AC5.
3. Three HINT copies (F1) + T243 rename (F27) + daemon AC11 tighten (F28) + **F33 hoist** CP const to module-level.
4. Renderer NEXT_STEP = DENIAL_HINT (F2); AC4 order Denied → next → grant-wall → Decisions.
5. Hermetic AC5 tighten; AC6/AC7/AC8/AC9 stay green.
6. Docs F19; CHANGELOG.
7. Classify-only AC10. **No** live bootstrap.
8. Review → `review.md`; FEATURE TX; implement-track Phase 6 publish.

---

## 11. Soft residuals

| Residual | Disposition |
|----------|-------------|
| PATH until `cargo install` | F13 |
| Three-copy HINT (no shared crate) | F24 |
| T226 O1 shared resolve wrapper | F5 |
| clap after_help still shows `--scope Repository:<uuid>` first | F6 / F26 — valid CI form |
| HINT **172** chars (no 140 cap) | F1 / OpenCode |
| PATH briefing `_None_` | T275 F18 PATH-behind; not this DoD |
| Live 0 of 3 grants | T275 F10 |
| Live leftover 11 roots | T276 F9 |
| T281–T283 peers | F17 |

---

## 12. Touch map

| Path | Change |
|------|--------|
| `crates/ai-brains-cli/src/commands/governed_common.rs` | F1 const + F27 unit |
| `crates/ai-brainsd/src/services.rs` | F1 twin + F28 unit |
| `crates/ai-brains-control-plane/src/query.rs` | F1 twin **hoist to module-level** (F33) + AC3 unit |
| `crates/ai-brains-control-plane/src/briefings/renderer.rs` | F2 NEXT_STEP = DENIAL_HINT + AC4 |
| `crates/ai-brains-cli/tests/policy_bootstrap.rs` | AC5 tighten T210 AC7 |
| `Docs/CLI-EXIT-CODES.md` / `CAPABILITIES.md` / `CHANGELOG.md` | F19 |
| `conductor/conductor.md` / `deferred.md` / README | Planned + absorb table |

**Do not touch:** `project.rs`, CLI `preflight.rs`, `doctor.rs`, `sync.rs`, `policy_cmd.rs`, `evidence.rs` / `source.rs` / `review.rs` (helpers already), `.env`, live vault.

---

## 13. AI fold-in

Inputs: `agy-review.md` (HEAD `f35884e`) + `opencode-review.md` (HEAD `f35884e`). Product crates identical to `83080ff`. **B 0 / M 0** both harnesses. last-PR #195 still empty. No T285. Do **not** edit the review files.

### Per-AI

| Source | Item | Disposition |
|--------|------|-------------|
| Agy m1 | Dual-site exact string equality vs F1 in CLI / daemon / CP | **Already** AC1–AC3; **folded** F33 — `assert_eq!` (not substring); hoist CP function-local const to module-level so AC3 can name it |
| Agy m2 | Markdown next still precedes `## Decisions` and T275 grant-wall | **Already** AC4 next-before-Decisions; **folded** AC4 order **Denied → NEXT_STEP → GRANT_WALL → `## Decisions`** (`renderer.rs` `:82` / `:85`) |
| Agy O1 | `CLI-EXIT-CODES.md` `:94` | **Already** F19 / AC11 |
| Agy O2 | Hermetic AC7 `!contains("--scope …")` + `omit --scope` | **Already** AC5 / Phase 1 |
| OpenCode O1 | Spec AC8 / T210 AC8 cited `:546` vs live fn `:548` | **Folded** §2.3 — comment `:546`, fn `:548` (`policy_bootstrap__no_scope_no_context__exit_2`). T280 **AC8** is grant-wall stay-green (not that test) |
| OpenCode (length) | F1 string is **172** chars, not ~183 | **Folded** AC1 / §8 / §11 |

### Pins locked by fold-in

1. **F33 / AC1–AC3:** three copies `assert_eq!` F1; CP HINT hoisted to module-level file-private.
2. **AC4:** denied markdown order Denied → next → grant-wall → Decisions.
3. **§2.3:** T210 AC8 live fn `:548` (comment `:546`).
4. **F1 length:** **172** chars (OpenCode count; PowerShell `.Length` 172; no U+2026).
5. **Already:** F19/AC11 docs; AC5 hermetic tighten.
6. **Affirm:** #195 N/A; no T285; no B/M to decline.

---
