# T282 — `context --show` must name leftover shell vs effective `.env`

- **Track ID:** T282-ContextShowLeftover
- **Status:** **Planned** (Pending until **go**)
- **Category:** UX / HONESTY
- **Owner:** Grok
- **Source:** Live CLI audit 2026-08-21 — `context --show` **7/7**; `whoami` already shows `shell_project_id=7d97a456` vs effective `3581317d`. Re-verified **2026-08-22** (HEAD `65108cd` T281 `#197`).
- **Depends on:** T240 ✅ whoami + shell capture; T242 ✅ session-quiet override warn; T257 ✅ JSON-effective silent; T256 ✅ `--help` hide_env_values (do **not** reopen)
- **Blocks / feeds:** Agents that only run `context --show` see leftover shell `PROJECT_ID` the same way `project whoami` already does. `project list` cwd-first **T283**.
- **Absorbs:** Placeholder problem text + Manual DoD; deferred.md “`context --show` misses leftover shell”; T276 F10/F11 shell leftover pointer; T206 CHANGELOG soft “no `context --show` mismatch warn” **declined as DoD** (T240 stderr already; cwd `mismatch: false`)
- **Not absorbed (DoD):** T240 F2 silent `.env` write; T258 adopt-path; T276 live leftover rebind; T256 `--help` restyle; T206 L3 path-mismatch line on `--show`; T242 warn restyle; `--format` / JSON dump; vault-free `--show`; SESSION leftover line; T283 list cwd-first; clap 5; rusqlite 0.40; DTO keys; `cargo install`
- **Research date:** 2026-08-22 (plan dogfood HEAD `65108cd` T281 `#197`; product `src/` = T281; `--show` last product-touched with context write path, still the raw `.env` dump)
- **Ledger:** planning DOCS TX `fe4e6895-6619-490d-8bbb-72a0fab55bb7`. Implement starts a **FEATURE** TX on **go**.
- **Isolation:** Do **not** `cargo install`. Do **not** write live `.env` (T240 F2). Do **not** `adopt-path --write-env`. Do **not** `rebind-path --write`. Do **not** grow hotspot `project.rs` / `sync.rs` / `forget.rs`. Do **not** print or commit `AI_BRAINS_KEY`. Do **not** live `policy bootstrap`, `safety sync` without `--dry-run`, `retention apply --confirm`, or `graph rebuild`. Do **not** mutate schtasks.

---

## 1. Objective

1. **Leftover shell is visible on `context --show`.** `project whoami` is the identity SoT and already prints `shell_project_id` when pre-dotenv shell `AI_BRAINS_PROJECT_ID` differs from `.env`. `--show` dumps cwd `.env` `AI_BRAINS_*` lines and the repository path and **does not** mention that leftover. T242’s stderr override warn is **session-quiet** after the first fingerprint, so later `--show` runs look like `.env` is the only ID in the process. Print one stdout leftover line when captured shell ≠ file `AI_BRAINS_PROJECT_ID`.
2. **Do not leak key material.** `--show` currently prints every `AI_BRAINS_*` file line. Live project `.env` has no KEY (global dotenv). If KEY / `VAULT_KEY` is in the file, redact the value. T256 already hid clap `--help` env values — do **not** reopen help.
3. **North star.** Capture independence: read-only dump + leftover + redact. No events. No models. No `.env` write. No identity rewrite. No new crates. No pin bumps.

This unblocks daily honesty for the Windows-first vault: a leftover `7d97a456` in the parent shell is already overridden by cwd `.env` `3581317d`, but agents that skip `whoami` never see it.

---

## 2. Live baseline (re-scan 2026-08-22)

### 2.1 Operator dogfood (this machine)

| Signal | Observation |
|--------|-------------|
| HEAD | **Plan dogfood:** `65108cd` T281 squash `#197`. Tree **CLEAN**. `origin/main` = HEAD (`0 0`) after `git fetch --all --prune`. |
| PATH `ai-brains` | `C:\Users\RyanB\.cargo\bin\ai-brains.exe` mtime **2026-08-22 14:49**, 25 443 840 bytes, **0.1.1**. Newer than the T270 2026-08-21 binary (likely post-T281 install). `--show` still has **no** leftover line. **Do not `cargo install`.** Tests/manual AC use `cargo run` / hermetic. |
| `preflight --summary` | Scope `C:\dev\ai-brains` (`3581317d`). Pinned **3619** (volatile). In-context **0/0/0**. Grants **0 of 3** (T275 hermetic; live not bootstrapped). Capture independence holds. |
| PATH `context --show` | `--- Current Context ---` then `AI_BRAINS_MODEL_URL` / `COMPLETION_MODEL` / `EMBEDDING_*` / `PROJECT_ID=3581317d-…` / `SESSION_ID` / `HARNESS_ID` / `Repository: C:\dev\AI-Brains`. **No leftover UUID `7d97a456`. No `(.env overrides)`. No `x'`.** |
| PATH `project whoami` (non-TTY → JSON) | `effective` / `env` / `path_alias` / `detect` = `3581317d-…`. **`shell_project_id`: `7d97a456-f2f4-43ea-1f13-211af684ad37`.** `mismatch: false`. `remediations: []`. Cwd identity is T258-complete; leftover is **shell vs `.env`**, not env vs path. |
| T242 warn on `--show` | **Absent** this session (session-quiet after first fingerprint). `should_warn_project_context_override` includes `"context"` (`main.rs` `:3019`) so the first spawn can stderr-warn; later `--show` does not. **That is why `--show` must carry leftover on stdout.** |
| Last GitHub PR | [#197](https://github.com/Ryan-AI-Studios/AI-Brains/pull/197) T281 (2026-08-22). `gh pr view --comments`, `/reviews`, `/comments`, `issues/197/comments` all **empty**. **last-PR Cursor: N/A.** Open PRs: Dependabot remotes only (`#61` rusqlite 0.40.2, `#62` chrono 0.4.45, `#59` tokio, actions `#68–#72`). **No leftover to mint. No T285.** |
| Prior #188 Bugbot | **T284 Completed** `#193`. Not this track. |
| Identity / doctor | ledgerful doctor 4 warn (legacy `.changeguard` / sig-pin / timings / models unreachable). **0 pending / 0 drift** at scan. Hotspot **#1** `project.rs` (displayScore **3.917**, volatile). `sync.rs` #2. `forget.rs` #3. `governed_common.rs` #4. **`context.rs` #5** (displayScore **2.662**) — intended touch; do **not** grow #1–#4. |
| `ISSUES.md` | **Does not exist.** |

### 2.2 Why this still matters

| Residual | Why it is still a product hole / why decline |
|----------|----------------------------------------------|
| `--show` omits leftover shell | whoami SoT already has the field. Agents that run `--show` as “what IDs am I on?” miss `7d97a456`. clig.dev: human-first + saying (just) enough + ease of discovery — do not hide the leftover only on a different subcommand. **DoD.** |
| T242 stderr already says override | Session-quiet after first fingerprint (`~\.ai-brains\cache\env-override-warn\`). `--show` is the durable surface. **Do not restyle T242.** |
| T206 L3 “`--show` mismatch warn” | Env vs **path** mismatch. Cwd is `mismatch: false`. T240 already stderr-warns when env ≠ path. Adding that to `--show` duplicates whoami/T240. **Decline as DoD.** |
| Dump `AI_BRAINS_KEY` from file | Live cwd `.env` has none (KEY lives in global dotenv; `--show` is **file-only**). The dump loop is still `starts_with("AI_BRAINS_")` — a local KEY would leak into transcripts. 12-factor config litmus: credentials must not appear when dumping config. Placeholder already forbids `x'`. **DoD redact.** T256 help stays frozen. |
| Delegate `--show` to `whoami` | Piped whoami is JSON (T266). `--show` is a dump even when piped. **Decline.** |
| `--format json` on `--show` | Not in T266 inventory. Agents already parse the dump. **Decline.** |
| Vault-free `--show` | Dispatch still builds `AppContext` (`main.rs` `:4557–4562`). Soft residual. **Decline as DoD.** |
| SESSION leftover line | T242 session-only differ is already debug-only. Placeholder is PROJECT. **Decline as DoD.** |
| Print leftover when no `.env` | Startup **clears** shell `PROJECT_ID` when cwd has no `.env` (`main.rs` `:3285+`). Suffix `(.env overrides)` would **lie**. **Decline as DoD** (must not print that suffix without a file PROJECT_ID). |

### 2.3 Code truth

| Item | Location | Notes |
|------|----------|--------|
| `--show` dump | `context.rs` **`:19–35`** | Reads cwd `.env`; prints `AI_BRAINS_*` lines + `Repository:`. Early `return Ok(())` **before** write (`:170`). **Insert leftover after `Repository`; redact KEY lines in the loop.** |
| No `.env` | `context.rs` **`:29–34`** | `No .env file found in {}. Run 'ai-brains context' to initialize.` **Do not** append `(.env overrides)`. |
| Write path | `context.rs` **`:37–176`** | **Do not change** except freeze: `--show` still returns first (`--new-project` + `--show` = show-only). |
| clap | `main.rs` **`:1284–1298`** | `Context { new_project, new_session, show, tx_id }`. **No `--format`.** No `after_help`. **Freeze flags.** |
| Dispatch | `main.rs` **`:4557–4562`** | Always opens vault. `--show` does not use `ctx` today. **Do not** make `--show` vault-free as DoD. |
| Shell capture | `main.rs` **`:3256–3263`** | Pre-dotenv `record_shell_project_id`. Comment names whoami. **Reuse as-is** — `--show` reads the same OnceLock. **Do not grow this block.** |
| Capture helpers | `project.rs` **`:156–163`** | `record_shell_project_id` / `shell_project_id_captured` already `pub`. **Call from `context.rs`. Do not add new helpers to `project.rs` (hotspot #1).** |
| whoami leftover rule | `project.rs` **`:703–709`** | `(Some(shell), Some(env)) if shell != env` → Some; `(Some(shell), None)` → Some. `--show` DoD is the **file-exists differ** arm only (F1). |
| whoami human | `project.rs` **`:770–783`** | `shell_project_id:      {uuid}` or `(none or same as env)`. **Do not** copy the full whoami table onto `--show`. |
| T242 warn gate | `main.rs` **`:3009–3026`** | `"context"` is in the list. **Do not edit.** |
| T242 SOOT | `env_warn.rs` **`:124–183`** | `Warning: local .env overrides inherited shell: AI_BRAINS_PROJECT_ID (was …).` **Stderr. Do not restyle. Leftover is stdout and must not start with `Warning:`.** |
| T256 help | `main.rs` **`:997`** `hide_env_values = true` | Clap 4.6.6 `Arg::hide_env_values` is **help-only** ([docs.rs/clap/4.6.6](https://docs.rs/clap/4.6.6/clap/struct.Arg.html#method.hide_env_values)). **Does not** redact `--show` file lines. Custom map in `context.rs`. |
| Hermetic `--show` | — | **No tests today** (grep `tests/` for `context --show` = empty). New file `tests/context_show_leftover.rs`. T257 `warning_json_stdout_hygiene.rs` uses `context` **write** to mint `.env` — **do not grow that file.** |
| Leak helper | `ai_brains_crypto::test_support::assert_no_secret_leakage` | T181 / T256. **Reuse** on `--show` stdout+stderr when fixture plants a dummy KEY in the **file**. |
| Dummy KEY | T256 `tests/cli_help_secret_redaction.rs` `DUMMY_KEY` | Distinct from `ZERO_SQLCIPHER_KEY`. Plant in fixture `.env`; vault still uses hermetic zero KEY. |
| Docs | `Docs/CAPABILITIES.md` **`:199`**; `Docs/OPERATIONS.md` **`:513`** | Show-only row is one cell. **Additive** leftover + redact. CHANGELOG T282 on go. |
| Skill | `.agents/skills/ai-brains/SKILL.md` / `.claude/skills/ai-brains/SKILL.md` | **No** `context --show` subsection. **Do not** mint one as DoD (T281 F19 analog). |
| `ISSUES.md` | — | Does not exist. |

### 2.4 Dependency / standards research (2026-08-22) — snapshot; re-verify at execute

| Pin | Workspace / lock | Ecosystem (today) | Action |
|-----|------------------|-------------------|--------|
| `clap` | workspace **4.5** / lock **4.6.1** (builder **4.6.0**) | crates.io **4.6.6** (2026-08-06). docs.rs `hide_env_values` still Arg/help-only. **No clap 5.** | **No bump.** No new flags. |
| `serde_json` | lock **1.0.150** | crates.io **1.0.151** | **No bump.** No JSON dump. |
| `chrono` | lock **0.4.44** | crates.io **0.4.45** (Dependabot #62 open) | **No bump.** |
| `rusqlite` | lock **0.39.0** + sqlcipher + backup | crates.io **0.40.2** (Dependabot #61) | **No bump.** |
| `uuid` | lock **1.23.1** | crates.io **1.25.0** | **No bump.** |
| `tokio` | workspace **1.52** / lock **1.52.3** | crates.io **1.53.1** (Dependabot #59) | **No bump.** |
| rustc / edition | **1.95.0** / **2024** | workspace toolchain | Unchanged |
| nextest | **0.9.140** | — | Unchanged |
| workspace | **0.1.1** | — | **No bump** |
| New crates | — | — | **Zero.** |

**Online / primary sources**

| Claim | Source | Takeaway |
|-------|--------|----------|
| Human-first; saying (just) enough; ease of discovery | [clig.dev](https://clig.dev/) (current; source repo updated 2026-05-16) | Leftover belongs on `--show` (the dump operators already run), not only on `whoami`. Omit the line when shell == file. |
| Credentials must not appear in dumped config | [12-factor Config](https://12factor.net/config) litmus (open-source without compromising credentials) | Redact `AI_BRAINS_KEY` / `AI_BRAINS_VAULT_KEY` on the file dump. Keep model URLs / PROJECT_ID / SESSION_ID. |
| clap `hide_env_values` | [docs.rs/clap/4.6.6 `Arg::hide_env_values`](https://docs.rs/clap/4.6.6/clap/struct.Arg.html#method.hide_env_values) | Help-only. T256 already set it. **Cannot** fix `--show`. Custom redact. |
| Competing config sources (shell vs file) | T205 force-set + T240 F4 whoami + T242 session-quiet | `.env` wins for daily Scope; leftover must still be named. Do not silent-switch (T240 F2). |

**N/A:** SQLCipher page encrypt, schtasks, T180 DTO new keys, Windows service, Safety GLOB (T279 Completed), policy HINT (T280 Completed), nightly HTTP vs TCP (T281 Completed `#197`).

**Could not verify:** whether every operator machine still has leftover `7d97a456` in the parent shell (this session **does**). DoD is the differ arm, hermetic + this-machine classify-only.

**ledgerful / ai-brains:** `preflight --summary` 0 of 3 grants @ **3619** pins; PATH `--show` dump without leftover + whoami `shell_project_id=7d97a456`; `ledgerful ledger status --compact` 0 pending / 0 drift; `search "shell_project_id_captured"` → `project.rs:160/:704`; `scan --impact` CLEAN at `65108cd`; `hotspots --json --limit 5` `project.rs` #1 — do not grow; `context.rs` #5 — intended. Semantic recall of leftover/`--show` returned T251/T276 review-track dumps (PATH-behind T274 ranking) — not used as SoT.

---

## 3. Frozen decisions

| ID | Decision |
|----|----------|
| **F0 — Go gate** | Plan-only until user **go**. Planning is DOCS TX `fe4e6895`. Implement starts a **FEATURE** TX. |
| **F1 — Leftover line (stdout, differ only)** | After the `Repository:` line (when `.env` exists), print exactly `format!("{SHELL_LEFTOVER_PREFIX}{id}{SHELL_LEFTOVER_SUFFIX}")` iff captured shell is Some nonempty **and** file `AI_BRAINS_PROJECT_ID` (trim, first matching line) is Some nonempty **and** `shell != file` (exact string, **not** `contains`, **not** case-fold — T269 F27 analog). Consts in `context.rs`: `SHELL_LEFTOVER_PREFIX = "shell leftover PROJECT_ID: "` (**27** chars including trailing space); `SHELL_LEFTOVER_SUFFIX = " (.env overrides)"` (**17** chars including leading space). Example with a 36-char UUID is **80** chars. Helpers: `format_shell_leftover_line(id) -> String`; `leftover_shell_vs_file(shell, file) -> Option<String>` (`None` for same / missing / empty / `"7d97a456" vs file None`). Source of shell = `shell_project_id_captured()` (already recorded in `main`). Source of file = parse cwd `.env` text, **not** post-dotenv process env (so `--no-project-context` still names leftover vs file). **Do not** start with `Warning:`. **Do not** print path-mismatch. **Do not** print remediations / `adopt-path`. **Do not** print leftover when there is no file PROJECT_ID (F26). |
| **F2 — No `.env` write** | Affirm T240 F2. `--show` returns before `fs::write`. `--show` + `--new-project` / `--new-session` is still show-only. Hermetic: file bytes unchanged. |
| **F3 — KEY redact on dump** | In the `AI_BRAINS_*` print loop: `AI_BRAINS_KEY=` / bare `AI_BRAINS_KEY` → print `AI_BRAINS_KEY=(redacted)`; `AI_BRAINS_VAULT_KEY=` / bare → `AI_BRAINS_VAULT_KEY=(redacted)`. Other `AI_BRAINS_*` lines print as-is (model URLs, PROJECT_ID, SESSION_ID, HARNESS_ID, `VAULT_PATH`). Consts `SHOW_REDACTED_KEY` / `SHOW_REDACTED_VAULT_KEY`. Helper `map_show_env_line(line) -> Option<String>` (`None` = skip non-`AI_BRAINS_` as today). **Do not** dump process-env KEY. **Do not** reopen T256 `--help`. |
| **F4 — Dump shape freeze** | Header `--- Current Context ---` stays. No TTY/JSON switch. Piped `--show` stays the dump. **No** `--format`. **No** clap `after_help` as DoD. |
| **F5 — whoami freeze** | Do not change whoami JSON keys, human labels, remediations, or `--no-project-context` nulling. `--show` is **not** whoami. |
| **F6 — T242 freeze** | Do not restyle `env_warn.rs`. Leftover is additive stdout. First-run stderr warn may still appear in hermetics — leftover must be on **stdout**. |
| **F7 — T256 freeze** | `hide_env_values` on `--key` stays. Root `--help` still `[env: AI_BRAINS_KEY]` without value. |
| **F8 — clap flags freeze** | No new Context flags. `show` remains a bool long flag. |
| **F9 — Module** | Consts + helpers + their units live in `context.rs`. Hermetic suite is `tests/context_show_leftover.rs`. **Do not** grow `project.rs` / `sync.rs` / `forget.rs` / `main.rs` (except docs comments if a one-line “`--show` also reads the capture” is needed — prefer **zero** `main.rs` / `project.rs` product diff). |
| **F10 — Decline path-mismatch on `--show`** | T206 L3 / T240 mismatch warn stay on stderr + whoami. Cwd `mismatch: false`. |
| **F11 — Decline vault-free `--show`** | Dispatch still opens the vault. Soft residual. |
| **F12 — Pins / crates** | No clap 5, no rusqlite 0.40, no chrono 0.4.45, no uuid 1.25, no tokio 1.53, no new crates, workspace **0.1.1**. |
| **F13 — PATH** | Do not `cargo install` unless the user asks. Tests/manual use `cargo run` / hermetic. |
| **F14 — Contracts** | No DTO. PROTOCOL-COMPAT untouched. E1: leftover line is **absent** when shell == file or no file PROJECT_ID (not `null`). |
| **F15 — Capture independence** | Dump/docs only. No events. No models. |
| **F16 — Stop-before live mutate** | Even after go: do not write live `.env`, do not `adopt-path --write-env`, do not `rebind-path --write`, do not live bootstrap / apply / rebuild / `safety sync` without `--dry-run`. |
| **F17 — Decline peers** | T283 list cwd-first; leftover 11-root rebind; T240 F2; T255 750 ms; T263 H2; T275 live bootstrap; T277 live `--no-prune`; T278 live rebuild; T279 live pin; T284 live apply; T281 product (Completed). |
| **F18 — last-PR Cursor** | #197 empty → **N/A**. #188 closed by T284. Dependabot `#61` rusqlite **not** this track. **No T285.** |
| **F19 — Docs** | CAPABILITIES Show-only row: additive “when pre-dotenv shell `PROJECT_ID` differs from the file, next line is `shell leftover PROJECT_ID: <uuid> (.env overrides)`; `AI_BRAINS_KEY` / `VAULT_KEY` file lines print `(redacted)`”. OPERATIONS `--show` bullet: same. Root CHANGELOG T282. CLI-EXIT-CODES unchanged (show still exit **0**). Skill: **no-op** (no `--show` subsection). |
| **F20 — Exit 0** | Unchanged. Leftover / redact / missing `.env` are still success. |
| **F21 — Tests** | Naming `function_or_feature__condition__expected_result`. Units for F1 consts/helpers + F3 map. Hermetic leftover / same-shell / KEY redact / no-write / no-env no-suffix. No `unwrap`/`expect`/`panic` in production. |
| **F22 — Cross-model** | Honesty UX on identity dump (easy T240/T256 regression). After Phase-1 review clean, run read-only `codex-review`. |
| **F23 — Debt file** | `conductor/ISSUES.md` does **not** exist. Deferrals → `conductor/deferred.md`. |
| **F24 — PowerShell** | `;` not `&&`. |
| **F25 — Compare source** | File PROJECT_ID vs captured shell, not `std::env::var` after force-set. Trim both. No UUID parse (invalid leftover still prints). |
| **F26 — No-`.env` leftover** | Not DoD. Must **not** print `(.env overrides)` when the file has no PROJECT_ID. Existing no-`.env` sentence stays. |
| **F27 — SESSION leftover** | Not DoD. T242 session-only remains debug. |
| **F28 — Classify-only live** | Manual AC uses `cargo run -p ai-brains-cli -- context --show` from this repo. Do **not** treat PATH as proof of the new line. Do **not** write `.env`. |
| **F29 — Existing tests stay green** | T240 whoami hermetics; T242 session-quiet; T256 help redact; T257 JSON-silent; T258 adopt-path. |
| **F30 — `x'` assertion** | Hermetic `--show` stdout+stderr must not contain the planted dummy `x'…'` / T256 `DUMMY_KEY` prefix. Do **not** globally forbid `x'` on `--help` (doctor after_help shape). `--show` has no format example. |
| **F31 — Identity leftover** | `7d97a456` vs `fcb8a40f` in other trees is T258/T276. This cwd leftover is shell `7d97a456` vs `.env` `3581317d`. **No T285.** |
| **F32 — Quoted file values** | v1 does **not** strip quotes. Hermetic writes unquoted IDs (live `.env` is unquoted). Soft residual. |

---

## 4. Acceptance criteria

| AC | Proof |
|----|-------|
| **AC1** | Unit: `SHELL_LEFTOVER_PREFIX` `assert_eq!` `"shell leftover PROJECT_ID: "` (**27** chars); `SHELL_LEFTOVER_SUFFIX` `assert_eq!` `" (.env overrides)"` (**17** chars); `format_shell_leftover_line` with a 36-char UUID is **80** chars and starts with the prefix and ends with the suffix; does **not** start with `Warning:`. **Required red** (`format_shell_leftover_line__known_uuid__frozen_80`). |
| **AC2** | Unit (rstest `#[case]`): `leftover_shell_vs_file(Some(shell), Some(file))` is `Some(format_shell_leftover_line(shell))` iff `shell != file` and both nonempty. `None` for same id, `None`/`Some`, `Some`/`None`, empty, `None`/`None`. **Required red.** |
| **AC3** | Unit: `map_show_env_line("AI_BRAINS_KEY=x'deadbeef…'") == Some("AI_BRAINS_KEY=(redacted)")`; same for `AI_BRAINS_VAULT_KEY=…`; `AI_BRAINS_PROJECT_ID=<uuid>` passthrough; `"# comment"` / `"LEDGERFUL_TX_ID=…"` → `None`. **Required red.** |
| **AC4** | Hermetic `tests/context_show_leftover.rs`: tempfile `.env` with `AI_BRAINS_PROJECT_ID={env_id}`; child env `AI_BRAINS_PROJECT_ID={shell_id}` (different UUIDs); `context --show` exit **0**; stdout contains file PROJECT_ID line; stdout contains **exact** leftover line for `shell_id`; leftover is **after** `Repository:`; stdout does **not** put leftover only on stderr. |
| **AC5** | Hermetic: shell == file PROJECT_ID → stdout does **not** contain `SHELL_LEFTOVER_PREFIX` and does **not** contain `(.env overrides)`. |
| **AC6** | Hermetic: fixture `.env` includes T256-class dummy `AI_BRAINS_KEY=x'…'` (not `ZERO_SQLCIPHER_KEY`); `--show` stdout+stderr contain `AI_BRAINS_KEY=(redacted)`; `assert_no_secret_leakage`; no `AI_BRAINS_KEY=x'`. Vault still opens via hermetic zero KEY. |
| **AC7** | Hermetic: snapshot `.env` bytes; `--show` (and `--show --new-project`) leave bytes unchanged; stdout has header (no “initialized” / “Local .env updated”). |
| **AC8** | Hermetic: no `.env` file + leftover shell env → stdout contains the existing `No .env file found` sentence; does **not** contain `(.env overrides)`. |
| **AC9** | Existing T240 whoami JSON still has `shell_project_id` when shell ≠ env (`project_identity_convergence.rs` leftover field assert). |
| **AC10** | Manual classify-only (`cargo run`, **no** `.env` write): from this repo, `context --show` still dumps `PROJECT_ID=3581317d-…`. **If** `project whoami` JSON has `shell_project_id` `7d97a456-…`, `--show` stdout contains that UUID **and** `(.env overrides)` **and** no `x'`. **If** whoami omits `shell_project_id`, leftover line is **absent**. Exit **0**. Pass-with-observed-data. Source/hermetic is DoD — **not PATH.** |
| **AC11** | Docs: CAPABILITIES + OPERATIONS name leftover + redact; CHANGELOG T282. PROTOCOL-COMPAT no new required keys. CLI-EXIT-CODES show exit 0 unchanged. |
| **AC12** | No production `unwrap`/`expect`/`panic`; no clap/rusqlite bump; no DTO keys; `project.rs` / `sync.rs` / `forget.rs` / `env_warn.rs` / T256 `--key` arg absent from the product diff (or comment-only `main.rs` if F9 one-liner is used — prefer zero). |
| **AC13** | T256 `cli_help_secret_redaction` still green (`[env: AI_BRAINS_KEY]` without `=`). |
| **AC14** | T242 session-quiet suite still green. |

---

## 5. Design notes

### 5.1 Human layout (`.env` exists, leftover present)

```
--- Current Context ---
AI_BRAINS_MODEL_URL=…
AI_BRAINS_PROJECT_ID=3581317d-601e-44f7-ab84-fde90aa12d3c
…
Repository: C:\dev\AI-Brains
shell leftover PROJECT_ID: 7d97a456-f2f4-43ea-1f13-211af684ad37 (.env overrides)
```

### 5.2 Gate

```text
leftover_shell_vs_file(captured, file_project_id)
  Some  → println leftover after Repository
  None  → omit (same / missing file id / missing shell)
```

### 5.3 Why not whoami / T242 / T206

- whoami piped is JSON; `--show` must stay a dump.
- T242 is stderr + session-quiet — the hole this session.
- T206 L3 is env vs path; cwd path already matches `.env`.

### 5.4 Redact

File dump ≠ clap help. Map KEY lines before println. Model URLs stay — they are not credentials in the 12-factor sense used here (local loopback).

---

## 6. Non-goals

- Silent `.env` rewrite (T240 F2)
- `adopt-path --write-env` / `rebind-path --write`
- Path-mismatch line on `--show` (T206 L3 / T240)
- Restyle T242 override warn
- Reopen T256 `--help`
- `--format json` / TTY-switch dump
- Vault-free `--show`
- SESSION leftover line
- No-`.env` leftover suffix `(.env overrides)`
- Quote-stripping file values
- T283 `project list` cwd-first
- clap 5 / rusqlite 0.40 / new DTO keys
- `cargo install` / live leftover 11-root rebind
- Live `policy bootstrap` / `retention apply --confirm` / `graph rebuild` / `safety sync` without `--dry-run`

---

## 7. Verification plan

1. **Red:** AC1–AC3 fail (consts/helpers missing).
2. **Green:** helpers + dump loop + leftover println in `context.rs`.
3. Targeted: `cargo nextest run -p ai-brains-cli leftover_shell map_show_env format_shell_leftover --test context_show_leftover` + T240 whoami leftover field + T256 help; clippy `-p ai-brains-cli --all-targets -- -D warnings`.
4. Manual classify-only AC10. **No** `.env` write.
5. Review log; FEATURE cross-model (F22).
6. Full gate before finalize. implement-track Phase 6 publish.

---

## 8. Risk

| Risk | Mitigation |
|------|------------|
| T240 F2 accidental write | AC7 bytes-unchanged; `--show` returns first |
| KEY leak in transcripts | AC3 / AC6 / F3 / F30 |
| Leftover only on stderr (T242) | AC4 stdout exact line |
| `(.env overrides)` lie without file id | AC8 / F26 |
| whoami JSON regression | AC9 / F5 |
| T256 help regression | AC13 / F7 |
| Hotspot `project.rs` growth | F9 — call existing `shell_project_id_captured` only |
| PATH-behind until install | F13; hermetic/source DoD |
| Unicode / Windows | F1 is ASCII. AC1 `assert_eq!` |

---

## 9. Deferred absorb / decline

| Item | Disposition |
|------|-------------|
| Audit `context --show` misses leftover shell vs `.env` (whoami has it) | **Absorb** F1–F4 / AC1–AC5 / AC10 |
| Placeholder “without printing `AI_BRAINS_KEY`” | **Absorb** F3 / AC3 / AC6 / AC10 |
| T276 F10/F11 / closeout shell leftover → T282 | **Absorb** (this track) |
| T206 CHANGELOG / L3 `context --show` mismatch warn | **Decline** F10 — T240 stderr; cwd `mismatch: false` |
| T242 session-quiet hides override | **Partial** — motivation; **do not** restyle T242 (F6) |
| T256 `--help` env values | **Decline** F7 — already Completed; file dump is this track |
| last-PR Cursor #197 | **N/A** — comments/reviews empty |
| last-PR #188 Work / apply samples | **Decline** — **T284 Completed** `#193` |
| Dependabot `#61` rusqlite 0.40.2 | **Decline** F12 — **no T285** |
| T283 list cwd-first / leftover 11 roots | **Decline** peers |
| T240 F2 / clap 5 / DTO required keys | **Decline** F2/F12/F17 |
| Identity mismatch quiet `7d97` vs `fcb8a40f` | **Not this track** — T258 adopt-path; leftover data T276; shell leftover **this track** (cwd) |
| Vault-free `--show` | **Decline as DoD** F11 |
| SESSION leftover / no-`.env` leftover suffix | **Decline as DoD** F26 / F27 |
| Historical CE wipe, MSI, `anyhow` allowlist, archive `changeguard` | **Decline** — not `--show` chrome |

**Entire `deferred.md` scanned.** Closed/strikethrough rows stay closed. Open overlapping row is this placeholder (absorb). T283 remains a Pending placeholder.

---

## 10. Implement order (on go)

1. Phase 0 re-verify `context.rs` `:19–35`, `main.rs` `:1284–1298/:3256–3263/:4557–4562`, `project.rs` `:156–163/:703–709`, T240/T242/T256 hermetics, deferred rescan, #197 still empty, pins. `git fetch --all --prune`; if `origin/main` moved, reconcile (no rebase over user work; never `git push origin main`).
2. Red AC1–AC3.
3. Consts + helpers in `context.rs`; dump loop redact + leftover println after `Repository:` using **captured** shell vs **file** id.
4. Hermetic `tests/context_show_leftover.rs` AC4–AC8; AC9/AC13/AC14 stay green.
5. Docs F19 (CAPABILITIES + OPERATIONS + CHANGELOG; **no** new skill section).
6. Classify-only AC10. **No** `.env` write.
7. Review → `review.md`; FEATURE TX; implement-track Phase 6 publish. `scripts/dev-check.ps1` (not repo-root `dev-check.ps1`).

---

## 11. Soft residuals

| Residual | Disposition |
|----------|-------------|
| PATH until `cargo install` | F13 |
| Vault-free `--show` | F11 |
| SESSION leftover line | F27 |
| No-`.env` leftover naming (without lie-suffix) | F26 |
| Quote-strip file PROJECT_ID | F32 |
| T283 list cwd-first | Peer placeholder |
| Live leftover 11 `C:\dev\*` roots | T276 F9 — owner-confirm rebind |
| Live 0 of 3 grants | T275 F10 |
| T242 first-run stderr still possible | F6 — leftover is stdout |

---

## 12. Touch map

| Path | Change |
|------|--------|
| `crates/ai-brains-cli/src/commands/context.rs` | F1 consts/helpers; redact loop; leftover println; `#[cfg(test)]` units |
| `crates/ai-brains-cli/tests/context_show_leftover.rs` | **New** hermetic AC4–AC8 |
| `Docs/CAPABILITIES.md` | Show-only row additive |
| `Docs/OPERATIONS.md` | `--show` bullet additive |
| `CHANGELOG.md` | T282 row |
| `conductor/conductor.md` | Planned → (on go) In Progress / Completed |
| `conductor/deferred.md` | This absorption; closeout on implement |
| `conductor/tracks/README-T274-T284-CLI-QUALITY.md` | T282 Planned |

**Do not touch:** `project.rs` (call existing capture only), `env_warn.rs`, T256 `--key`, `sync.rs`, `forget.rs`, `daemon.rs`, `nightly.rs`, contracts, `Cargo.toml` / lock, skill `--show` section, live `.env`.
