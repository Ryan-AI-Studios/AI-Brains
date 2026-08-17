# T258 — Daily Scope = path owner (no silent switch)

- **Track ID:** T258-DailyScopePathOwner
- **Status:** **Completed** 2026-08-16 (FEATURE TX `6962a7b8-ff3a-4c0b-90cb-b3167d993335`)
- **Category:** FEATURE / UX / OPS
- **Owner:** —
- **Source:** Audit 2026-08-16 — non-working default identity; opportunity “rebind daily Scope to `3581317d`”
- **Depends on:** T240 whoami / mismatch warn / no silent rewrite (F2); T206 detect; T233/T254 register-path; T205 local PROJECT_ID force-set
- **Blocks / feeds:** Honest scores for recall, preflight, briefing, query, memory, pin. T259 leftover split stays separate. T267 whoami remediator string lands here.
- **Absorbs:** Default Scope `test-alias` `441837f6` (592 mem) vs path owner `C:\dev\ai-brains` `3581317d` (2,700); T240 F14 path-owner slice of `project use`; whoami remediations that only say “hand-edit `.env`”; stale T240 runbook that still points at `7d97a456`
- **Not absorbed:** Auto-merge projects; silent `.env` write (T240 F2); splitting `7d97a456` (T259); general `project use <uuid>` (F14 remainder); T257 warn/JSON placement; T267 harness next / list footer
- **Research date:** 2026-08-16 (plan HEAD `e055e29`; fold-in HEAD `d5bed64`)
- **AI fold-in:** 2026-08-16 `opencode-review.md` only (no agy/grok/claude plan review). No Blockers. **Agree hard:** OC-M1 hermetic `--format human` (AC1–AC6). **Agree:** OC-m2 export-prefix soft residual; OC-m3 `project.rs` **1547** lines; OC-m4 already-bound human chrome; OC-m5 `keys_touched` meaning; OC-O6 `--no-project-context` AC16; OC-O7 drop list remediations bullet. Disposition **§13**.
- **Ledger:** planning DOCS TX `f7b86f91-b914-4a93-b951-217c14157e6c`. Fold-in DOCS TX `f38d51f7-fb9e-4c2b-85cb-379cd76b74a8`. Implement starts a **FEATURE** TX on **go**.
- **Isolation:** Do **not** write the live repo `.env`. Do **not** `cargo install`. Do **not** alias `7d97a456` as `AI-Brains`. Do **not** reopen T240 F2, T255 declines, T257, T259. Do **not** bump clap / add crates.

---

## 1. Objective

Make the **daily** project for a registered repo the path-alias owner of that repo **without** violating T240 F2 (no silent auto-switch).

Two layers, one product verb:

1. **Print-only remediator (default):** `project adopt-path` names the path owner and prints the exact `.env` assignment. No file write.
2. **Confirmable write:** `--write-env --yes` rewrites **only** `AI_BRAINS_PROJECT_ID` in cwd `.env`. Other keys stay. No session rotate. No events. No merge.

This advances the north star because daily Scope is where `pin`, `recall`, `preflight`, and capture land. Live Scope is the `context`-hashed sandbox (`441837f6`, last 8 UUID bytes zero). New decisions from this machine have been landing there instead of the path owner (`3581317d`, 2,700 memories). Capture independence holds: identity is `.env` only — no models, embeddings, or graph.

---

## 2. Live baseline (2026-08-16)

### 2.1 Operator dogfood (this machine)

| Signal | Observation |
|--------|-------------|
| HEAD | `e055e29` — T256 Completed (`#170`). Tree CLEAN. `main` = `origin/main`. |
| PATH `ai-brains` | `C:\Users\RyanB\.cargo\bin\ai-brains.exe` (mtime 2026-08-16 08:04). May be PATH-behind T256. **Do not `cargo install`.** |
| `preflight --summary` | Scope `test-alias` (`441837f6-5c55-d075-0000-000000000000`); 563 pinned; mismatch warn → whoami. |
| `project whoami --format json` | See triangle below. `mismatch: true`. Remediations still say “operator rebind; no auto-write” + `project list`. They do **not** name a product verb. |
| `project list --format json` (three IDs) | `441837f6` **test-alias** 592 mem, `active`, path `null`. `3581317d` **2,700** mem, path + alias both `C:\dev\ai-brains` (set-alias used as a path string). `7d97a456` **18,028** mem, path `C:\dev\crawlx`, no alias. |
| Warn SOOT | `Warning: project identity mismatch: daily Scope is '441837f6-…', but path is registered to '3581317d-…'. Run 'ai-brains project whoami'.` |
| `context` today | Writes `.env` (PROJECT + SESSION + HARNESS). Early-returns when a session exists. `--new-project` **mints a fresh UUID**. Hash fallback produces IDs like `441837f6-…-000000000000`. **Not** the remediator. |
| Last GitHub PR | [#170](https://github.com/Ryan-AI-Studios/AI-Brains/pull/170) merged 2026-08-16. `gh pr view --comments`, `/reviews`, `/comments` all **empty**. HEAD is `main` (no open PR). Open PRs are Dependabot only. **last-PR Cursor: N/A.** |
| Ledgerful | `doctor` ready (legacy `.changeguard` / sig-pin / timings). 0 pending at plan start. Work root `C:\dev\AI-Brains`. Hotspot **#1** = `project.rs` (3.689, 1401 lines). `context.rs` is **#4**. |
| ai-brains recall | Scoped to test-alias (the bug). `--global` lexical hit on T240/T254 path-alias decisions; no prior “adopt-path” pin. |

### 2.2 Live identity triangle (re-verified)

```text
  repo .env PROJECT_ID ──► 441837f6  test-alias   592 mem   daily Scope *
                           ▲ context.rs DefaultHasher (UUID tail 000000000000)

  shell (pre-dotenv)   ──► 7d97a456  (no alias)  18,028 mem  path=C:\dev\crawlx
                           ▲ leftover dump (T259). Do not adopt. Do not set-alias AI-Brains.

  path alias / detect  ──► 3581317d  2,700 mem   path=C:\dev\ai-brains
                           ▲ real work for this repo. Alias string is the path (set-alias misuse).
```

cwd `C:\dev\AI-Brains` vs registered path `C:\dev\ai-brains` — `normalize_for_location_compare` already treats these as the same location (T240). `git_toplevel` = `C:/dev/AI-Brains`. `git_slug` = `AI-Brains`.

T240 made this **visible**. It did not make daily commands **correct**. `preflight`, `recall`, `memory list`, `briefing`, `query progressive`, and `pin` all scope to the 592-memory sandbox.

### 2.3 Why this still matters

| Residual | Why it is a product hole / why decline |
|----------|----------------------------------------|
| No first-class adopt | Operators must hand-edit a UUID. whoami remediations say so. **DoD.** |
| `context` is the wrong writer | Session-exists early-return leaves the sandbox. `--new-project` mints a **new** ID. Hash fallback **created** `441837f6`. Teaching `context` as remediator would make this worse. **Decline as remediator.** |
| T240 F14 `project use` | Reserved, never shipped. Absorb the **path-owner** slice as `adopt-path`. General `use <uuid>` stays soft. |
| Silent auto-switch | T240 F2. Still right. **Decline.** |
| Merge `441837f6` → `3581317d` | Compensating import. **Decline** (later if ever). |
| Alias `7d97a456` as `AI-Brains` | T259 / T267. **Decline.** |
| Live operator rebind this session | Out of band unless the owner asks. Planning + implement tests use tempdir only. |

### 2.4 Code truth

| Item | Location | Notes |
|------|----------|-------|
| Daily Scope force-set | `main.rs` `apply_local_project_context_env` **:2442** | Local `.env` **always** force-sets `AI_BRAINS_PROJECT_ID` / `SESSION_ID` over shell. |
| Dotenv order | `main.rs` **:2670–2713** | cwd `.env` (if exists) → force IDs → global `~/.ai-brains/.env` gap-fill. |
| `ProjectCommands` | `main.rs` **:2066–2155** | List / Resolve / Detect / SetAlias / RegisterPath / Whoami / ListPaths / ScanRoots / UnregisterPath. **No** AdoptPath. **No** Use. |
| Whoami | `project.rs` `whoami` / `build_whoami_report` **:742–857** | Remediations: “no auto-write” + hand-set PROJECT_ID + `project list`. |
| Warn SOOT | `project.rs` `identity_mismatch_warn_line` **:325–327** | Points at `project whoami`. T257 owns placement. **Do not change the string** except if T257 later asks. |
| Path owner | `find_path_alias_owner` + `resolve_path_alias_for_location` | T233/T240. Reuse. |
| Existing `.env` writer | `commands/context.rs` **:139–170** | Filter-then-append PROJECT/SESSION/HARNESS/TX. `fs::write`. Also `ensure_project_and_session_exists` + sync pull. **Do not call this from adopt-path.** |
| T254 sibling module | `commands/project_paths.rs` | Pattern: new file so `project.rs` hotspot does not grow. |
| Hermetic whoami | `tests/project_identity_convergence.rs` | Fields + mismatch + warn-on-list. Remediations **text is not asserted**. |
| Reparse refuse | `ai_brains_path::is_reparse_or_symlink` / `refuse_if_reparse` | Use on write path. |
| `fail_usage` | `governed_common.rs` `EXIT_USAGE = 2` | Project commands today return `Err` (exit 1) or clap exit 2. Prefer clap `requires` for `--yes`; product exit **2** when `--write-env` lacks `--yes`. |
| help_ia | `help_ia.rs` | No project-subcommand lock for whoami. Additive after_help only. |
| Hotspots | `project.rs` **#1** (**1547** total lines; 1401 non-blank). `context.rs` **#4** (183 total / 164 non-blank). | Adopt logic → **`project_adopt.rs`**. Whoami remediations = **minimal** string edit in `project.rs`. **Do not** grow `context.rs`. |

### 2.5 Dependency / standards research (2026-08-16)

| Pin | Workspace / lock | Action |
|-----|------------------|--------|
| `clap` | workspace **4.5** / lock **4.6.1** (builder **4.6.0**) | **No bump.** New `ProjectCommands::AdoptPath` with `requires = "write_env"` on `--yes`. |
| crates.io clap | latest **4.6.6** (2026-08-06). **No clap 5.** | Forbidden future-bump guard. Snapshot — re-verify at execute. |
| `dotenvy` | workspace **0.15** / lock **0.15.7** | **Read-only.** docs.rs 0.15.7 public API is `dotenv` / `from_path` / `from_path_iter` / overrides. **No write / set / serialize.** Confirmed 2026-08-16. |
| crates.io dotenvy | **0.15.7** latest on crates.io (0.16 unpublished on GitHub main). | **No bump.** Do not add a write crate. |
| 12-factor III | [12factor.net/config](https://12factor.net/config) | Config in env vars, orthogonal keys, not checked into git. Fits: touch **one** key (`AI_BRAINS_PROJECT_ID`); never KEY; cwd `.env` already gitignored; print-only default. |
| Windows replace | `std::fs::write` is what `context.rs` uses today | Match that (no new tempfile crate). Refuse reparse before write. Soft residual: atomic tmp+rename. |
| rustc / edition | **1.95.0** / **2024** | Unchanged. |
| workspace version | **0.1.1** | **No bump.** |
| New crates | — | **Zero.** No camino in CLI (T254 pin). `std::path` + `to_string_lossy`. |
| Best practice | Confirm-before-mutate (retention `--confirm`, erasure `--confirm`). dotenv libraries load, they do not author. Line-preserving rewrite is the in-tree pattern (`context.rs`). | Fits this repo: print-only default + dual flag `--write-env --yes`; reuse `find_path_alias_owner`. |

---

## 3. Frozen decisions

| ID | Decision |
|----|----------|
| **F0 — Go gate** | Plan-only until user **go**. Planning is DOCS. Implement starts a **FEATURE** TX. |
| **F1 — Verb** | `ai-brains project adopt-path`. Not `context`. Not `project use`. T240 F14 path-owner slice is this verb. General `project use <uuid>` remains soft residual. |
| **F2 — T240 F2 stands** | Never silently change effective PROJECT_ID. Default adopt-path is **print-only**. Startup / detect / whoami / warn never write `.env`. |
| **F3 — Dual confirm** | Write requires **both** `--write-env` and `--yes`. `--yes` clap-`requires` `--write-env`. `--write-env` without `--yes` → exit **2**, no write, stderr names `--yes`. |
| **F4 — Target file** | cwd `.env` only (same file `apply_local_project_context_env` reads). Never `~/.ai-brains/.env`. Never a `--path` override in v1 (tests `current_dir` into tempdir). |
| **F5 — One key** | Rewrite touches **only** `AI_BRAINS_PROJECT_ID`. Preserve comments, blanks, `AI_BRAINS_KEY`, `AI_BRAINS_VAULT_PATH`, `AI_BRAINS_SESSION_ID`, `AI_BRAINS_HARNESS_ID`, `LEDGERFUL_TX_ID`, unknown keys. Do **not** rotate session. Do **not** call `ensure_project_and_session_exists`. Do **not** `sync pull`. |
| **F6 — Missing `.env`** | Print-only: would create. `--write-env --yes`: create a file containing only `AI_BRAINS_PROJECT_ID=<path-owner>\n`. |
| **F7 — Already bound** | When env PROJECT_ID (post-dotenv, or the file’s PROJECT_ID if `--no-project-context`) **equals** path owner: exit **0**, `already_bound: true`, **no** file rewrite. |
| **F8 — No path owner** | Exit **1**. Stderr: register the cwd/toplevel via `project register-path`. No write. |
| **F9 — Path owner source** | Same helper as whoami: `resolve_path_alias_for_location` (git toplevel else cwd → `find_path_alias_owner`). Adopt **that** UUID. No `--project-id` override (that would be `project use`). |
| **F10 — Whoami remediations** | On mismatch, remediations[] **must** name `ai-brains project adopt-path` (print-only) and the exact `AI_BRAINS_PROJECT_ID=<path-id>` assignment, plus the two honesty lines (no auto-switch; set-alias ≠ register-path). Must **not** say “run `project whoami`”. Must **not** include `project list` (T267 owns the list footer). Warn line stays T240 SOOT (points at whoami) — T257 owns warn placement. |
| **F11 — `context` freeze** | Do not edit `context.rs` except a one-line honesty comment if forced. Docs must say: `context` initializes / rotates; it is **not** adopt-path. |
| **F12 — Module** | New `crates/ai-brains-cli/src/commands/project_adopt.rs`. Dispatch from `main.rs`. Pure rewrite helper unit-tested. `project.rs` only: remediations string + `pub(crate)` reuse of path-resolve if needed. |
| **F13 — Format** | `--format auto\|human\|json` (same parser as whoami `project.rs:748-752`: `auto` → JSON when stdout is **not** a TTY). Frozen JSON keys §5.1. |
| **F14 — Reparse** | Before write: `is_reparse_or_symlink` + `refuse_if_reparse` on the `.env` path. Exit **1**. |
| **F15 — No events / no merge** | Capture independence. No `ProjectAliasAdded`. No memory move. `441837f6` pins stay on `441837f6`. |
| **F16 — No leftover alias** | Never recommend `set-alias 7d97a456 AI-Brains`. Never adopt shell leftover. |
| **F17 — Live `.env`** | Implement + plan **must not** write `C:\dev\AI-Brains\.env`. Operator rebind is out of band unless the owner asks. Tests: `tempfile::tempdir` only. |
| **F18 — Pins / crates** | No clap 5, no dotenvy bump, no new crates, workspace **0.1.1**. No camino in CLI. |
| **F19 — Docs** | CAPABILITIES adopt-path row. WORKFLOWS §0 runbook: replace “Edit `.env`” with `project adopt-path` / `--write-env --yes`. Root CHANGELOG T258. Do **not** reprint T240’s `7d97a456` as this repo’s main project. |
| **F20 — Exit codes** | 0 = print-only / write / already-bound. 2 = usage (`--write-env` sans `--yes`; clap `--yes` sans `--write-env`). 1 = no path owner / reparse / IO / vault. |
| **F21 — Tests** | New file `crates/ai-brains-cli/tests/project_adopt_path.rs`. Additive remediations assert may live there (prefer not to churn T240 field tests). T240 `project_identity_convergence` stays green. |
| **F22 — Cross-model** | FEATURE / identity UX. After Phase-1 review clean, run read-only `codex-review`. |
| **F23 — Debt file** | `conductor/ISSUES.md` does **not** exist. Deferrals → `conductor/deferred.md`. |
| **F24 — PATH-behind** | Live PATH binary may lack adopt-path until `cargo install`. Tests/manual AC use `cargo run` / hermetic bin. |
| **F25 — Decline extras** | T257 warn/JSON; T259 split; T267 harness/list footer; silent switch; merge; `project use <uuid>`; `context --new-project` as remediator; writing global dotenv; clap 5. |
| **F26 — Hermetic format** | AC1–AC6 fixtures that assert human chrome **must** pass `--format human`. `Command.output()` / nextest stdout is a pipe, so `auto` emits JSON (same as whoami / `project_paths.rs:70`). AC13 is the JSON lock. Do not assert `AI_BRAINS_PROJECT_ID=<B>` on `auto`. |

---

## 4. Acceptance criteria

| AC | Proof |
|----|-------|
| **AC1** | **Red today** (command missing). Hermetic: two projects; cwd registered to B; `.env` has `PROJECT_ID=A` plus a dummy `AI_BRAINS_KEY=x'deadbeef…'` and `AI_BRAINS_SESSION_ID=<uuid>`. `project adopt-path --format human` (no write flags) exits **0**. Stdout names B and contains the exact line `AI_BRAINS_PROJECT_ID=<B>` (F26 — **not** `auto`). File bytes **unchanged**. |
| **AC2** | **Red today.** Same fixture. `project adopt-path --format human --write-env` (no `--yes`) exits **2**. File unchanged. Stderr mentions `--yes`. |
| **AC3** | **Red today.** Same fixture. `project adopt-path --format human --write-env --yes` exits **0**. File `PROJECT_ID` is B. KEY line and SESSION line **byte-identical**. No new `HARNESS_ID` invented. |
| **AC4** | **Red today.** Registered path, **no** `.env`. `--format human --write-env --yes` creates `.env` whose only assignment is `AI_BRAINS_PROJECT_ID=<B>\n`. |
| **AC5** | Hermetic already-bound (`.env` PROJECT_ID = path owner). `--format human`: exit **0**, stdout contains `Already bound to path owner` and the path-owner UUID, **no** `Would set` / `Re-run with --write-env`. File unchanged. `--format json` (same fixture or AC13 sibling): `already_bound: true`, `written: false`, `from_project_id` == `to_project_id`. |
| **AC6** | Hermetic **no** path alias: `--format human` exit **1**, no write, stderr mentions `register-path`. |
| **AC7** | Whoami mismatch remediations (JSON) contain `project adopt-path` and `AI_BRAINS_PROJECT_ID=<path-id>`. Combined remediations string does **not** contain `` `ai-brains project whoami` `` and does **not** contain `project list`. Existing T240 field ACs stay green. |
| **AC8** | `project_identity_convergence` suite stays green (warn still says `project whoami`). |
| **AC9** | `context.rs` untouched (or comment-only). `cargo nextest` context/T82 tests stay green. |
| **AC10** | No contracts DTO; no pin bumps; no new crate. No event appended on write (hermetic: event count unchanged aside from fixture seed). |
| **AC11** | Docs: CAPABILITIES adopt-path; WORKFLOWS §0 uses adopt-path (no `7d97a456` + `AI-Brains`); CHANGELOG T258. |
| **AC12** | `--yes` without `--write-env` is clap usage (exit **2**). |
| **AC13** | `--format json` print-only: one object, keys §5.1, `written: false`. Stdout parses (T257 still owns mismatch-warn interleave). |
| **AC14** | Write on a `.env` that `refuse_if_reparse` would trip: exit **1**, no replace. Unit-test the helper with `is_reparse=true`; hermetic symlink only if cheap on Windows. |
| **AC15** | Manual (source bin, **do not write live `.env`**): `cargo run -p ai-brains-cli -- project adopt-path --format human` in this repo is print-only and names `3581317d-601e-44f7-ab84-fde90aa12d3c`. Confirm live `.env` mtime/hash unchanged. (Force `human` — agent stdout is often a pipe.) |
| **AC16** | Hermetic `--no-project-context`: `.env` already has path owner B; process env `AI_BRAINS_PROJECT_ID` is a different A. `project adopt-path --format human` exits **0**, `already_bound` (F7 file-id branch), file unchanged. Does **not** treat shell A as the bind source. |

Test names (TDD). **Must fail red before F1 exists:** AC1–AC4 (command unknown → clap exit 2, not our contract). After clap lands and before write helper: AC1 print-only can go green; AC3/AC4 stay red until rewrite.

- `project_adopt_path__print_only__names_owner_no_write`
- `project_adopt_path__write_env_without_yes__exit_2_no_write`
- `project_adopt_path__write_env_yes__rewrites_only_project_id`
- `project_adopt_path__missing_env__write_creates_project_id_only`
- `project_adopt_path__already_bound__exit_0_no_rewrite`
- `project_adopt_path__no_path_owner__exit_1_no_write`
- `project_whoami__mismatch__remediations_name_adopt_path`
- `project_adopt_path__yes_without_write_env__clap_exit_2`
- `project_adopt_path__format_json__print_only_keys`
- `project_adopt_path__no_project_context__file_project_id_already_bound`
- `rewrite_project_id_in_env__preserves_other_keys` (unit)
- `rewrite_project_id_in_env__refuse_reparse` (unit)

---

## 5. Design notes

### 5.1 Command + JSON

```text
ai-brains project adopt-path [--write-env] [--yes] [--format auto|human|json]
```

Human print-only (SOOT):

```text
Would set AI_BRAINS_PROJECT_ID=<path-owner> in <abs-cwd>\.env
from: <env-or-file-or-(none)>
to:   <path-owner>
Other keys would be left untouched.
Re-run with --write-env --yes to apply.
```

Human already-bound (SOOT — AC5):

```text
Already bound to path owner <path-owner>
No .env write.
```

Must **not** print the print-only `Would set` / `Re-run with --write-env` block when `already_bound`.

JSON (frozen):

```json
{
  "api_version": "1",
  "action": "adopt-path",
  "env_path": "<absolute>",
  "from_project_id": "<uuid-or-null>",
  "to_project_id": "<uuid-or-null>",
  "written": false,
  "already_bound": false,
  "keys_touched": ["AI_BRAINS_PROJECT_ID"]
}
```

`to_project_id` is null only on AC6 (no path owner). `keys_touched` is the key set the action **would or does** touch — always `["AI_BRAINS_PROJECT_ID"]`, including when `written: false` or `already_bound: true`. It is not “keys mutated on this invocation.”

### 5.2 Rewrite helper (pure)

`rewrite_project_id_line(existing: &str, new_id: &str) -> String`

- If a line starts with `AI_BRAINS_PROJECT_ID` (match `context.rs` `starts_with("AI_BRAINS_PROJECT_ID")`): replace that line with `AI_BRAINS_PROJECT_ID={new_id}`.
- `export AI_BRAINS_PROJECT_ID=…` is **not** matched (same as `context.rs`). Soft residual §11 — do not dual-write a second unexported line *and* claim export-form replace as DoD.
- Else append `\nAI_BRAINS_PROJECT_ID={new_id}\n` (keep existing trailing newline rules from `context.rs`).
- Do not parse values through dotenvy (quotes/comments stay on **other** lines).
- Unquoted UUID write form (same as `context.rs`).

`adopt_write(path, new_id, is_reparse) -> Result<(), AdoptError>` applies refuse-then-`fs::write`.

### 5.3 Why not `context` or `project use`

| Option | Why not v1 |
|--------|------------|
| `context` | Wrong ID source (ledgerful / hasher / `--new-project`). Rotates session. Sync pull. Early-return. Created the sandbox. |
| `project use <uuid>` | Broader T240 F14. Operators would pass `7d97a456` (shell leftover). Path-owner-only is the safe verb. |
| `project use --path-owner` | Extra noun. T267 stub already says `adopt-path`. |

### 5.4 Whoami remediations (mismatch)

Replace the hand-edit bullet with:

1. Daily Scope comes from `.env` / shell (not auto-switched). *(keep)*
2. `Run \`ai-brains project adopt-path\` (print-only) or \`ai-brains project adopt-path --write-env --yes\`.`
3. `To bind daily Scope to the path owner, set AI_BRAINS_PROJECT_ID=<path-id> in project .env.`
4. set-alias vs register-path. *(keep)*
5. **Drop** “Run `project list`”. Remediations are the adopt verb + assignment + the two honesty lines only. T267 owns the list footer.

---

## 6. Non-goals

- Silent Scope switch / detect auto-write
- Merging `441837f6` memories into `3581317d`
- Splitting or aliasing `7d97a456` (T259 / T267)
- T257 stderr-only / JSON interleave
- Changing mismatch **warn** SOOT (still points at whoami)
- `project use <uuid>`
- Editing `context.rs` behavior
- Writing global `~/.ai-brains/.env`
- Live operator `.env` rebind unless asked
- clap 5 / dotenvy bump / new crates / camino
- Contracts DTO / PROTOCOL-COMPAT
- `cargo install`

---

## 7. Verification plan

1. **Red:** add `project_adopt_path.rs`. Filter **fails** because `adopt-path` is unknown (clap exit 2) — that **is** the red for AC1–AC4. Do not “fix” by asserting clap’s unknown-subcommand text as the product contract.
2. **Green clap:** add `AdoptPath` so AC1 (print-only `--format human`) and AC2/AC12 (usage) can pass; AC3/AC4 still red until rewrite. Do **not** assert human chrome on `--format auto` (F26).
3. **Green write:** helper + `--write-env --yes`. AC3–AC6, AC13–AC14, **AC16**.
4. **Whoami:** AC7 remediations. AC8 T240 suite green.
5. **Docs:** AC11.
6. Targeted: `cargo clippy -p ai-brains-cli --all-targets -- -D warnings` ; nextest `project_adopt_path` + `project_identity_convergence` + context tests.
7. Phase-1 review → `review.md`. FEATURE → `codex-review`.
8. Manual AC15 print-only on this repo; **do not** `--write-env --yes` on live `.env`.
9. Full gate before finalize. `ISSUES.md` does not exist.

---

## 8. Risk

| Risk | Mitigation |
|------|------------|
| Implementer writes live `.env` | F17 / AC15 print-only; stop-before |
| Rewrite clobbers KEY | AC3 dummy KEY line; F5 |
| Operators run `context --new-project` | F11 / docs honesty |
| `project use` would adopt leftover | F1 verb is path-only |
| Hotspot `project.rs` grows | F12 new module |
| T257 JSON interleave on adopt-path json | AC13 parses stdout; warn stays stderr (T257) |
| Windows reparse `.env` | F14 |
| PATH binary lacks command | F24 |
| T240 warn test breaks if we change SOOT | F10 / AC8 — do not change warn |

---

## 9. Deferred absorb / decline

Entire `conductor/deferred.md` scanned 2026-08-16 (post-P12 through T256 closeout + T257–T271 map + historical T142–T196).

| Item | Disposition |
|------|-------------|
| Daily Scope `test-alias` `441837f6` vs path `3581317d` (audit P0) | **Absorb** F1–F10 / AC1–AC7 / AC15 |
| T240 F14 `project use` | **Partial absorb** — path-owner remediator ships as `adopt-path`. General `use <uuid>` stays soft residual. |
| T240 F2 no silent rewrite | **Keep** F2 — default print-only |
| T240 F13 detect `--json` | **Decline** — not this track |
| T240 operator runbook (`set-alias 7d97… AI-Brains`) | **Absorb** F19 / AC11 — rewrite WORKFLOWS §0 |
| Whoami remediations = hand-edit / “run whoami” (T267 F2 slice) | **Absorb** F10 / AC7. Harness next + list footer stay **T267**. |
| T257 identity warn / JSON interleave | **Point** — do not steal |
| T259 leftover `7d97a456` split | **Point** — do not steal |
| T260–T271 remainder | **Point** — not identity rebind |
| T206 soft F8/F10/F24 (`--json` source / `context --show`) | **Decline** |
| T212 no auto-alias / list footer | **Point T267** |
| T254 `scan-roots` never writes `.env` | **Honor** — adopt-path is the only new writer, confirm-gated |
| T223 / T242 env-override warn | **Decline** — do not re-spam; write does not change warn policy |
| T255 declined bag | **Decline** — stay declined |
| T256 PATH-behind / help leak | **Not related** — closed; F24 is the same *class* of PATH honesty only |
| `context` writes `.env` without confirm | **Decline as rewrite** — do not change context; F11 honesty |
| R-CI-BRANCH / MSI / notarization / App Store | **Not related** — packaging / admin |
| `anyhow` RUSTSEC-2026-0190 allowlist | **Not related** |
| `#34.2` DataKey rotation | **Closed** T189 — not related |
| T142 archive `changeguard` strings | **Not related** |
| T210–T232 / T234–T255 / T256 soft residuals | **Not related** unless they are T240 F14 (absorbed above) |
| last-PR Cursor (#170 + open HEAD PR) | **N/A** — #170 comments/reviews/inline all empty; HEAD is `main`; open PRs are Dependabot (no Cursor/Bugbot findings). **No leftover to mint.** |
| Closed/strikethrough deferred rows | Stay closed |

---

## 10. Implement order (on go)

1. Phase 0: re-verify whoami triangle + `ProjectCommands` still has no AdoptPath; rescan `deferred.md` + last PR Cursor. Confirm live `.env` will not be written.
2. Red: `project_adopt_path.rs` (unknown subcommand).
3. Green clap + print-only + usage ACs.
4. Green rewrite helper + `--write-env --yes`.
5. Whoami remediations AC7.
6. Docs AC11.
7. Targeted clippy + nextest.
8. Review + FEATURE codex-review.
9. Manual AC15 print-only.
10. Full gate; conductor **Completed**; deferred closeout line.

---

## 11. Soft residuals

| Residual | Notes |
|----------|-------|
| General `project use <uuid>` | T240 F14 remainder — not DoD |
| Atomic tmp+rename write | `fs::write` matches `context.rs` |
| Share rewrite helper with `context.rs` | Isolation; do not refactor context |
| `--dry-run` alias | Default is already print-only |
| Hermetic junction/symlink AC14 on Windows | Unit helper is DoD; live symlink optional |
| PATH binary lacks adopt-path | F24 — operator `cargo install` |
| T257 warn on adopt-path JSON | Peer track |
| `3581317d` set-alias is a path string | Honesty only; T212/T267 |
| Live operator rebind of this repo | Out of band |
| `export AI_BRAINS_PROJECT_ID=` prefix | Same `starts_with` as `context.rs` — not DoD. dotenvy last-wins if a second unexported line is appended |

---

## 12. Touch map

| Path | Change |
|------|--------|
| `crates/ai-brains-cli/src/commands/project_adopt.rs` | **New** — print/write + pure rewrite |
| `crates/ai-brains-cli/src/commands/mod.rs` | `pub mod project_adopt` |
| `crates/ai-brains-cli/src/main.rs` | `ProjectCommands::AdoptPath` + dispatch |
| `crates/ai-brains-cli/src/commands/project.rs` | Remediations strings only (F10) |
| `crates/ai-brains-cli/tests/project_adopt_path.rs` | **New** hermetic ACs |
| `Docs/CAPABILITIES.md` | adopt-path row |
| `Docs/WORKFLOWS.md` | §0 runbook |
| `CHANGELOG.md` | T258 row |
| `conductor/tracks/trackT258-daily-scope-path-owner/{spec,plan}.md` | This plan |
| `conductor/conductor.md` | Pending row text (still **Pending**) |
| `conductor/deferred.md` | Absorb note |
| `conductor/tracks/README-T256-T271-CLI-AUDIT.md` | T258 planned note |

Do **not** touch: `context.rs` (behavior), `key_resolve.rs`, `env_warn.rs`, contracts, daemon, `project_paths.rs` (unless a tiny `pub(crate)` reuse is forced), live `.env`.

---

## 13. AI fold-in disposition (2026-08-16)

Source: `opencode-review.md` (OpenCode) only. No `agy-review.md` / `grok-review.md` / `claude-review.md` / `codex-plan-review.md`. No Blockers. Re-verified at fold-in HEAD `d5bed64`: `whoami` format parser `project.rs:748-752` (`auto` → `!stdout().is_terminal()`); `context.rs:64` `starts_with("AI_BRAINS_PROJECT_ID")` (no `export `); `project.rs` **1547** total lines; `requires =` already at `main.rs:587`; `resolve_path_alias_for_location` `project.rs:264`. Review re-confirmed deferred + last-PR Cursor N/A — **no leftover to mint**.

### OpenCode

| ID | Verdict | Action |
|----|---------|--------|
| **M1** AC1–AC6 human strings vs `--format auto` on a pipe | **Agree hard** | **F26** / AC1–AC6 / AC15 force `--format human`. Live: same `IsTerminal` as whoami / `project_paths.rs:70`. Hermetic `Command.output()` is never a TTY. |
| **m2** `export ` prefix not matched | **Agree** (soft) | §5.2 honesty + §11 residual. Match `context.rs`. Not DoD. |
| **m3** `project.rs` line-count 1401 vs 1547 | **Agree** | §2.4: **1547** total (`Get-Content`.Count). 1401 was `Measure-Object -Line` (non-blank). New-module decision unchanged. |
| **m4** already-bound human body unspecified | **Agree** | §5.1 human already-bound SOOT + AC5. |
| **m5** `keys_touched` when `written: false` | **Agree** | §5.1: would-or-does-touch, not mutated-this-invocation. |
| **O6** `--no-project-context` file-id already-bound | **Agree** | **AC16** / test `project_adopt_path__no_project_context__file_project_id_already_bound`. Pins F7 file branch. |
| **O7** drop `project list` remediations bullet | **Agree** | F10 / §5.4 item 5 is **drop**, not optional. |

### Pins locked by fold-in

1. **F26 / AC1–AC6 / AC15:** hermetic human-chrome fixtures pass `--format human`. Never assert `AI_BRAINS_PROJECT_ID=<B>` on `auto`.
2. **AC5 / §5.1:** already-bound human is `Already bound to path owner` + `No .env write.` — not the `Would set` block.
3. **AC16 / F7:** `--no-project-context` already-bound reads the **file** PROJECT_ID, not the shell env.
4. **F10 / §5.4:** remediations do not include `project list`.
5. **§2.4:** `project.rs` is 1547 lines; adopt still goes in `project_adopt.rs`.
