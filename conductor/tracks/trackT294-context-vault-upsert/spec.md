# T294 — `context` already-initialized must upsert the `.env` project into the vault

- **Track ID:** T294-ContextVaultUpsert
- **Status:** **Planned** (Pending until **go**)
- **Category:** UX / IDENTITY / FEATURE
- **Owner:** Grok
- **Source:** Audit 2026-08-22 leftover **5** roots dest-missing; `context` early-return skips vault ensure. Placeholder minted with T285–T300 (`76c4db9`). T259 ✅ dest must exist / does not mint. T258 ✅ adopt-path (`.env` write, not vault). T240 F2 freeze. T282 ✅ `--show` leftover. T276 ✅ prefer-fill; live leftover Stop-Before.
- **Depends on:** T259 ✅ `rebind-path` dest-must-exist (F9) + memories stay (F5); T258 ✅ adopt-path is the Scope remediator; T240 F2 no silent `.env` rewrite; T282 ✅ `--show` leftover line + KEY redact; T82 ✅ already-initialized early-return + `--new-project` rotate
- **Blocks / feeds:** Operators who run `context` in a leftover repo whose `.env` `PROJECT_ID` is **not** in the open vault get that dest registered (idempotent events) **without** rewriting `.env`, so print-only `rebind-path --to <env-id>` stops saying dest-missing. Live leftover `--write --yes` stays owner-confirm (**T276 F9**). Backup **T295**. Forget-list **T299**. Graph sparse **T300**.
- **Absorbs:** Placeholder problem text + Manual DoD; deferred.md “leftover dest-missing / context skip upsert” (every T285–T293 decline pointer); T259 F9 runbook “run `context` in that repo first” (currently a lie)
- **Not absorbed (DoD):** T258 adopt-path / T240 F2 silent Scope switch; T259 dest mint inside `rebind-path`; T259 F5 `MemoryMoved`; live leftover `rebind-path --write --yes` without owner confirm; gimp/homebrew-tap silent mint without `.env`; `--new-project`; T282 `--show`; T295–T300; clap 5 / rusqlite 0.40
- **Research date:** 2026-08-24 (plan dogfood HEAD `2325adc` T293 `#209`. Product `src/` = T82 early-return **before** `ensure_project_and_session_exists`. PATH **0.1.2** 2026-08-22 19:41 **without** T285–T293 — hole is in **source and PATH**)
- **AI fold-in:** none yet (plan-track). Review-track writes `agy-review.md` / `opencode-review.md` only.
- **Ledger:** planning DOCS TX `dd3a3998-4754-49e8-9558-524c7b1761c3`. Implement starts a **FEATURE** TX on **go**.
- **Isolation:** Do **not** `cargo install`. Do **not** rewrite live `.env`. Do **not** pin production decisions to the live vault as implement. Do **not** live leftover `rebind-path --write --yes` / `unregister-path`. Do **not** live `context` in leftover dirs unless the owner confirms at go (mints dest into the daily vault). Do **not** grow hotspot `project.rs` (**#1** 3.924) / `sync.rs` / `forget.rs` / CLI `preflight.rs` / `project_rebind.rs` / `project_adopt.rs`. This track lives in `commands/context.rs` (hotspot **#5** 2.932 — the bug file) + clap `main.rs`. Reuse `AppContext::ensure_project_and_session_exists` (`src/context.rs:107`) — do **not** duplicate `ProjectRegistered`. Do **not** print or commit `AI_BRAINS_KEY`.

---

## 1. Objective

1. **Already-initialized `context` ensures the `.env` dest in the open vault.** When cwd `.env` has parseable `AI_BRAINS_PROJECT_ID` + `AI_BRAINS_SESSION_ID` and the operator did **not** pass `--new-project` / `--new-session` / `--show`, `context` still calls `ensure_project_and_session_exists` with **those** IDs. Missing dest → append `ProjectRegistered` (and `SessionStarted` if the session is missing). Present dest → zero extra events. Exit **0**.
2. **`.env` bytes stay unchanged on that path.** T240 F2. This is **not** adopt-path (T258 writes Scope). This is **not** rebind (T259 moves a path alias). Vault catches up to the file the operator already chose.
3. **Then print-only rebind dest exists.** `project rebind-path <path> --to <env-id> --format human` no longer exits **1** dest-missing for that id. Rebind still does **not** mint (T259 F9). Memories stay (T259 F5). Live `--write --yes` stays owner-confirm.
4. **North star.** Capture independence: identity events through existing capture `ensure` / `start_session`. No models. No hidden CoT. No silent Scope rewrite. Leftover split can finish without minting **new** IDs that overwrite crawlx/degoo/kinledger `.env`.

This unblocks the leftover runbook. WORKFLOWS already says “in the leftover repo, mint/ensure dest first: `ai-brains context`.” Live `context` prints “already initialized” and returns **before** “Ensure project/session exists in the vault” (`context.rs:135–151` vs `:168–190`). crawlx dest `a1a61a6f-578a-683a-0000-000000000000` is **not** in this vault. Print-only rebind: `Project 'a1a61a6f-…' not found in vault.` exit **1**.

---

## 2. Live baseline (re-scan 2026-08-24)

### 2.1 Operator dogfood (this machine)

| Signal | Observation |
|--------|-------------|
| HEAD | `2325adc` T293 squash `#209`. Tree **CLEAN**. `origin/main` = HEAD. Branch `main`. |
| PATH `ai-brains` | `C:\Users\RyanB\.cargo\bin\ai-brains.exe` mtime **2026-08-22 19:41**, 25 139 712 bytes, **0.1.2**. Has T259 rebind + T258 adopt-path + T282 `--show`. **Does not have T285–T293.** Early-return hole is in **source + PATH**. **Do not `cargo install`.** Tests/manual AC use `cargo run` / hermetic. |
| `preflight --summary` | Scope `C:\dev\ai-brains` (`3581317d`). Pinned **4049** (volatile). In-context **0/0/0**. Word **175**. Grants omitted (live 3 of 3). |
| Leftover inventory | `project list-paths --shared-only --format json`: **5** roots, all `7d97a456-f2f4-43ea-1f13-211af684ad37`, `exists: true`: `C:\dev\crawlx`, `degoo`, `gimp`, `homebrew-tap`, `kinledger`. (Was 11; 6 rebound earlier out of band.) |
| Leftover `.env` dests | **crawlx** `AI_BRAINS_PROJECT_ID=a1a61a6f-578a-683a-0000-000000000000` (session present, no KEY). **degoo** `39dadbbe-bef9-1245-0000-000000000000`. **kinledger** `efb5f6dd-b89b-82de-0000-000000000000`. **gimp** / **homebrew-tap** **NO_ENV**. Hashed tail-zero IDs (T258 “context DefaultHasher”). |
| Vault membership | `project list --format json` contains `7d97a456` and `3581317d`. **Does not** contain `a1a61a6f` / `39dadbbe` / `efb5f6dd`. |
| Print-only rebind dest-missing | `project rebind-path C:\dev\crawlx --to a1a61a6f-578a-683a-0000-000000000000 --format human` (piped → JSON): exit **1**, `Project 'a1a61a6f-…' not found in vault.` **This is the dest-missing hole.** **Did not** `--write`. |
| `context --help` (PATH **and** `cargo run`) | `Initialize or refresh the project context (writes local .env)`. **No** after_help. Dual-truth: already-initialized currently writes **nothing** (early-return) and also ensures **nothing**. |
| Last GitHub PR | [#209](https://github.com/Ryan-AI-Studios/AI-Brains/pull/209) T293 (merged 2026-08-24). `gh pr view --comments`, `/reviews`, `/comments`, `issues/209/comments` all **empty**. **last-PR Cursor: N/A.** Open PRs: Dependabot remotes only (`#61` rusqlite 0.40.2, `#62` chrono 0.4.45, `#59` tokio, `#60` thiserror, `#58` tower-http, actions `#68–#72`). **No leftover to mint. No T301.** |
| Identity / doctor | ledgerful doctor **4** warn (legacy `.changeguard` / sig-pin / sig-version / timings). Optional :8081 / :8083 **unreachable** this pass (volatile; semantic recall embedding was **ok**). **0 pending / 0 drift.** Hotspot **#1** `project.rs` (**3.924**) — **do not touch.** `sync.rs` **#2** (3.610). `governed_common.rs` **#3** (3.595). `forget.rs` **#4**. **`commands/context.rs` #5 (2.932, 321 lines) — extend here.** CLI `preflight.rs` #8. |
| `ISSUES.md` | **Does not exist.** |
| Planning live `context` in leftover dirs | **Not run.** Current source early-returns (no mint, no write). After go, live leftover `context` **would** mint dest into the daily vault — **Stop-Before** unless owner confirms (F27). |

### 2.2 Why dest-missing still blocks leftover split

| Layer | Truth |
|-------|--------|
| T259 F9 | `rebind-path` dest must already exist. Does **not** mint `ProjectRegistered`. Runbook: `context` in that repo first. |
| T82 early-return | `if existing_session && !new_session && !new_project { print already initialized; return Ok(()) }` **before** `ensure_project_and_session_exists` (`:135–151` vs `:168`). |
| `.env` dest ≠ leftover dump | crawlx/degoo/kinledger hashed IDs are **their** context-init IDs, not `7d97a456`. Path alias still points at leftover. Dest is the `.env` id. |
| gimp / homebrew-tap | No `.env`. First-init `context` **writes** `.env` (hashed). Document, not silent mint without a file. |
| T258 | `adopt-path` writes **daily Scope** to the **path owner**. Opposite direction. Cwd AI-Brains `mismatch: false`. Teaching `context` as adopt-path remediator is still declined. |
| T240 F2 | No silent rewrite of `.env` / daily Scope. Upsert vault **to match** `.env`, never the reverse. |
| T276 F9 | Live leftover `--write --yes` is Stop-Before. T294 unblocks dest; it does **not** rebind. |
| Identity `7d97a456` vs `fcb8a40f` | `fcb8a40f` is sibling `C:\dev\ledgerful`. Daily Scope leftover vs that path owner is **T258 in that repo**. Not a new TNN. |

### 2.3 Code truth (opened)

| Item | Location | Notes |
|------|----------|-------|
| Early-return | `commands/context.rs` `run` **`:135–151`** | Session exists + not `--new-session`/`--new-project` → print + **`return Ok(())`**. **Replace return with upsert-then-return.** |
| Ensure (skipped) | `:168–190` | `ensure_project_and_session_exists` then **writes** `.env`. First-init / rotate only. |
| Env parse (lossy) | `:113–133` | `starts_with("AI_BRAINS_PROJECT_ID")` + `split('=').nth(1)` — no trim. **Do not use for upsert.** |
| T282 helper | `file_project_id_from_env_text` **`:27–34`** | `strip_prefix` + trim + nonempty. **Reuse.** Add `file_session_id_from_env_text` analog. |
| AppContext ensure | `src/context.rs` **`:107–166`** | Projection `SELECT 1 FROM project_projection` / `session_projection`; missing → `ProjectRegistered` name `(no alias) — {8hex}` + `CaptureService::start_session`. **Reuse. Do not fork.** |
| clap | `main.rs` `Commands::Context` **`:1581–1596`** | `--new-project` / `--new-session` / `--show` / `--tx-id`. **No** after_help. Docstring “writes local .env”. Dispatch **`:4876`**. |
| Rebind dest | `project_paths.rs` `resolve_project_ref` **`:500–506`** | UUID not in `list_projects` → `Project '{id}' not found in vault.` T259 F9. **Do not mint here.** |
| Rebind AC8 | `tests/project_rebind_path.rs` **`:658`** | Missing `00000000-…0077` exit **1**. **Stay green.** |
| Smoke idempotency | `tests/smoke.rs` `test_cli_context_idempotency` **`:1934`** | Second `context` contains `already initialized`; `.env` bytes equal. **Stay green.** |
| `--show` leftover | `tests/context_show_leftover.rs` | T282. **Stay green. No vault write.** |
| T259 after_help | `main.rs` **`:3033`** | “Does not mint the dest project — run `ai-brains context` in that repo first.” **Becomes true.** Tighten one sentence (F19). |
| CAPABILITIES | `:199` | `context` — writes local `.env`. Dual-truth after this track. |
| WORKFLOWS leftover | `:79–81` | Commented `cd crawlx` / `ai-brains context`. **Currently a lie.** |
| OPERATIONS | `:512–519` | First-init narrative only. |
| PROTOCOL-COMPAT | no `context` JSON row | **N/A** — `context` has no `--format` / JSON. Human stdout may gain one `Vault:` line (T180: changing human is OK). |
| CLI-EXIT-CODES | dest-missing is T259 exit **1** | Malformed `.env` UUID this track: exit **1** `COMMAND_FAILED` (same family). Not clap usage. |
| `ProjectId::from_str` | `ids.rs:38` | `Uuid::parse_str` — accepts hashed tail-zero IDs (`a1a61a6f-578a-683a-…`). **Required** (F15). |
| Hermetic | `tests/common/mod.rs` `hermetic_bin` `:90` / `isolate_empty_home` `:120` | Use these. |

### 2.4 Dependency / standards research (2026-08-24)

| Pin / source | Workspace / live | Action |
|--------------|------------------|--------|
| `clap` | workspace **4.5** / lock **4.6.1** / crates.io **4.6.6** / GitHub **v4.6.6** (2026-08-06) — **no clap 5** (latest release is 4.6.6) | **No bump.** No new flags. after_help only. Snapshot — re-verify at execute. |
| `serde_json` | lock **1.0.150** / crates.io **1.0.151** | **No bump.** |
| `uuid` | lock **1.23.1** / crates.io **1.25.0** | **No bump.** `parse_str` already accepts leftover hashed IDs. |
| `chrono` | lock **0.4.44** / crates.io **0.4.45** (`#62`) | **No bump.** |
| `rusqlite` | lock **0.39.0** / crates.io **0.40.2** (`#61`) | **No bump.** |
| `thiserror` | lock **2.0.18** / crates.io **2.0.20** (`#60`) | **No bump.** |
| `tokio` | lock **1.52.3** (`#59`) | **No bump.** |
| `dotenvy` | lock **0.15.7** = crates.io **0.15.7** | **No bump.** Do not rewrite `.env` via dotenvy. |
| rustc / edition / nextest | **1.95.0** / **2024** / **0.9.140** | Unchanged. |
| workspace version | **0.1.2** | **No bump.** |
| New crates | — | **Zero.** |
| [clig.dev Output](https://clig.dev/#output) (fetched 2026-08-24) | Humans first; if you change state, tell the user; make the default the right thing; actions crossing the program boundary should be explicit; changing human output is usually OK | Default already-initialized **ensures** (no extra flag). Additive `Vault: project and session present.` when ensure ran. No `--yes` — minting an empty dest matching the operator’s existing `.env` is additive, not a Scope switch (contrast T258 `--write-env --yes`). |
| [12-factor III Config](https://12factor.net/config) + dotenv “never modify env vars already set” | Config lives in the environment / `.env`; tools load, they do not silently rewrite | T240 F2. Vault upsert **follows** `.env`. Adopt-path is the confirmable **write**. |
| Event sourcing / CQRS | Canonical SoT is append-only events. `ensure` checks projection then appends `ProjectRegistered` / session start. No update-in-place. `eventcore-sqlite` and similar adapters are **N/A** (we already have `SqliteEventStore`). | Reuse ensure. Do not invent `MemoryMoved`. |
| T180 P-CLI | Human output may evolve; JSON is the wire | `context` has no JSON. PROTOCOL-COMPAT **no new row**. |
| T259 F9 / tenant-split | Dest must exist; historical facts stay on the from-stream | **Affirm.** T294 creates dest via `context`, not via rebind. |
| SQLCipher / schtasks / Windows service | N/A — no backup, no scheduler | N/A (written). |

Training data is not a pin. Re-verify clap / uuid parse at execute.

**Could not verify:** live leftover `context` upsert (Stop-Before). Hermetic dest-missing is the proof. `ledgerful index --incremental` **failed** this pass (index writer killed) — symbols via **grep** + opened files.

**ledgerful / ai-brains:** `preflight --summary` 4049 pins @ `3581317d`; leftover 5 roots; dest `a1a61a6f` missing; `recall` lexical thin on “vault upsert” (T258/T259 review-track dumps); semantic T258 F2 adopt-path print-only. `ledgerful search` failed (index). grep: `ensure_project_and_session_exists` in `src/context.rs:107` + hooks/ingest/pin/sync; early-return only in `commands/context.rs:150`.

---

## 3. Frozen decisions

| ID | Decision |
|----|----------|
| **F0 — Go gate** | Plan-only until user **go**. Planning is DOCS. Implement starts a FEATURE TX. |
| **F1 — Already-initialized upserts env IDs** | When `.env` has parseable `AI_BRAINS_PROJECT_ID` **and** `AI_BRAINS_SESSION_ID` and flags are not `--show` / `--new-project` / `--new-session`: print the existing “already initialized” block, call `ensure_project_and_session_exists` with **those** IDs (not the hashed/discovered `project_id` computed above), print **exact** `Vault: project and session present.`, `return Ok(())`. **Do not** write `.env`. **Do not** auto-trigger `sync pull` (first-init pull stays on the write path only). |
| **F2 — T240 F2** | No silent `.env` / daily Scope rewrite. Bytes of cwd `.env` on the upsert path **equal** before/after (including KEY, comments, blanks, extra keys). Never write `~/.ai-brains/.env`. |
| **F3 — Parse helpers** | Reuse `file_project_id_from_env_text` (T282 trim). Add `file_session_id_from_env_text` with the same `strip_prefix("AI_BRAINS_SESSION_ID=")` + trim + nonempty grammar. **Do not** use `:113` `starts_with` + `split('=')` for upsert (matches `AI_BRAINS_PROJECT_ID_FOO`, no trim). Quote-strip **not** this track (T282 F32 residual). |
| **F4 — Both IDs required to mint** | Upsert only when **both** parse as `ProjectId` / `SessionId` (`FromStr` / `Uuid::parse_str`). Session-present + missing/unparseable project → **do not** mint the hashed/discovered id; keep already-initialized print (hashed display allowed) + skip ensure. Session line present but unparseable (non-UUID garbage) **and** project parses → exit **1**, `.env` unchanged, no events. |
| **F5 — No new clap flags** | No `--ensure-vault` / `--yes`. Default is the remediator (clig). `--show` / `--new-project` / `--new-session` / `--tx-id` frozen. |
| **F6 — `--show` freeze** | T282 leftover line + KEY/VAULT_KEY redact. **No** vault ensure. **No** `.env` write. Stay-green `context_show_leftover.rs`. |
| **F7 — Rotate still writes** | `--new-project` / `--new-session` skip the upsert-only arm (T82). Fall through to ensure **new** ids + `.env` write as today. Smoke `--new-session` stay-green. |
| **F8 — T259 F9 freeze** | `rebind-path` still does **not** mint dest. `resolve_project_ref` / AC8 missing `00000000-…0077` stay exit **1**. After T294, dest from `.env` exists **because context ensured it**. |
| **F9 — Not adopt-path** | T258 stands. `context` is still not the daily-Scope remediator. Whoami remediations stay `adopt-path`. Do **not** rewrite `PROJECT_ID` to the path owner. |
| **F10 — Memories stay** | T259 F5. No `MemoryMoved`. No leftover reclassify. |
| **F11 — Live leftover Stop-Before** | Plan + implement **must not** `rebind-path --write --yes` / `unregister-path` on live leftover roots unless the owner confirms **per path** in the go prompt. Hermetic is sufficient DoD. |
| **F12 — No auto path surgery** | No auto `register-path` / `rebind-path` / bulk `--all`. Operator still confirms rebind per path. |
| **F13 — Isolation hotspots** | **Do not grow** `project.rs` (#1). **Do not edit** `project_rebind.rs` / `project_adopt.rs` / `sync.rs` production / `ranking.rs` / CLI `preflight.rs`. Helpers `pub(crate)` in `commands/context.rs` (already leftover helpers). Do **not** `pub`. Do **not** re-export from `commands/mod.rs`. Split a new file only if production net ≥80 lines. `AppContext::ensure_project_and_session_exists` **reuse** — do not change signature unless a compile forces it (F34). |
| **F14 — Invalid UUID** | Unparseable project (when session parses) **or** unparseable session (when project parses and a session line exists): exit **1** (`COMMAND_FAILED` / `Box<dyn Error>` — **do not** import `fail_usage` / grow `governed_common.rs`). stderr names `AI_BRAINS_PROJECT_ID` and/or `AI_BRAINS_SESSION_ID`. `.env` unchanged. No events. Not clap usage exit 2. |
| **F15 — Hashed tail-zero IDs** | Leftover dests are DefaultHasher UUIDs (`…-000000000000`, version nibble often not v4). `Uuid::parse_str` **must** accept them. Hermetic AC uses a hashed-shape id, not only `Uuid::new_v4()`. Do **not** hardcode live leftover UUIDs `a1a61a6f` / `7d97a456` in product src or `--help`. |
| **F16 — Tests** | Naming `function_or_feature__condition__expected_result`. No `unwrap`/`expect`/`panic` in **production**. Hermetic `tempfile::tempdir` + `hermetic_bin`. New `crates/ai-brains-cli/tests/context_vault_upsert.rs`. Smoke idempotency **stay-green**. Rebind AC8 **stay-green**. |
| **F17 — Docs** | CAPABILITIES Init row: already-initialized **ensures vault dest**, does **not** rewrite `.env`. OPERATIONS `:512` extend (do not add a second context block). WORKFLOWS leftover `:79–81` uncomment/truth: `context` ensures dest without rewriting `.env`. T259 after_help `:3033` one-sentence tighten. Context clap docstring + **new after_help** dual-truth (first-init writes; already-initialized ensures). CHANGELOG. CLI-EXIT-CODES exit **1** mentions malformed `.env` project/session UUID. PROTOCOL-COMPAT **no new JSON row** (F20). |
| **F18 — PATH** | Soft. Source/hermetic SoT. Do not `cargo install` as implement. |
| **F19 — T259 after_help honesty** | Keep “Does not mint dest — run `context` in that repo first.” Add that already-initialized `context` **upserts** that dest **without** rewriting `.env`. Do **not** restyle the rest of rebind after_help. |
| **F20 — PROTOCOL-COMPAT** | No `context` JSON surface. **No** §5 row. Human `Vault:` line is **not** a wire contract. |
| **F21 — Capture independence** | Reuse capture `ensure` / `start_session`. No models, embeddings, or graph **required**. Existing `StoreSink` graph_hook under `--features graph` stays. Feature-off still upserts via SQL events. |
| **F22 — ISSUES.md** | Does not exist. Debt is `deferred.md`. |
| **F23 — Decline peers** | T293 Completed `#209` — not stolen. T295 backup; T296 nightly Router; T297 daemon vs LLM; T298 device; T299 forget-list; T300 graph sparse. T282 leftover `--show` Completed. T258 adopt-path Completed. T283 list cwd-first Completed. |
| **F24 — Standing declines** | T240 F2 reopen; T263 H2; 750 ms; clap 5; rusqlite 0.40; DTO new required keys; dest mint in rebind; bulk leftover split; `MemoryMoved`; silent `.env` rewrite. |
| **F25 — No T301** | #209 Cursor **N/A empty**. Dependabot remotes are not tracks. |
| **F26 — Identity leftover `7d97` vs `fcb8a40f`** | **Not this track.** T258 in that repo / leftover data T276. T294 is dest-missing for **`.env` IDs not in vault**. |
| **F27 — Live leftover `context`** | Running `context` in crawlx/degoo/kinledger **after** this lands **mints dest into the daily vault**. Planning did **not**. Implement Manual may do it **only** if the owner confirms. Default Manual is hermetic + print-only rebind. |
| **F28 — No-`.env` repos** | gimp / homebrew-tap still need first-init `context` (writes `.env`). Do **not** silent-mint a dest without a file. Document in WORKFLOWS. |
| **F29 — Harness fail-open** | If `AI_BRAINS_HARNESS_ID` in `.env` parses, use it for `start_session` when session is missing. Missing or invalid harness → `HarnessId::new()`. Do **not** fail the command. Do **not** write harness to `.env`. |
| **F30 — Cross-model** | FEATURE / identity. After Phase-1 review clean, run read-only `codex-review`. |
| **F31 — Stop-before** | Even after go: no live `.env` rewrite; no leftover `--write --yes` without owner confirm; no extra live `policy bootstrap`; no `retention apply --confirm`; no `graph rebuild`; no schtasks mutate; no `cargo install`. |
| **F32 — Extra stdout** | Additive fourth line **exact** `Vault: project and session present.` after Session ID when F1 ensure ran (both IDs parsed). Do **not** print `Local .env updated successfully.` on this path. First-init chrome frozen. |
| **F33 — Privacy** | `Privacy::LocalOnly` (same as first-init). |
| **F34 — ensure signature** | Reuse `ensure_project_and_session_exists` as-is. Do not add a return “minted vs existed” unless a compile/test cannot lock F32 without it. Idempotent projection check is enough. |
| **F35 — No leftover UUID in product** | Tests may use **fixture** hashed-shape UUIDs (`00000000-…` / `aaaaaaaa-…` / `Uuid::from_bytes` tail-zero). Runbook may name live leftover. New `--help` / stderr **must not** recommend `set-alias 7d97 … AI-Brains` (T259 F1). |
| **F36 — PowerShell** | `;` not `&&`. |
| **F37 — Identity stdout** | No JSON on `context`. Human only. |
| **F38 — Name freeze** | Minted dest name stays ensure’s `(no alias) — {8hex}`. Do not `set-alias`. |

---

## 4. Acceptance criteria

| ID | Criterion |
|----|-----------|
| **AC1** | Unit: `file_session_id_from_env_text` padded `AI_BRAINS_SESSION_ID=  {uuid}  ` → `Some(uuid)`; empty / missing / comment → `None`. `file_project_id_from_env_text` already T282 — stay. **Required red** for the new helper. |
| **AC2** | rstest parse: hashed-shape `a1a61a6f-578a-683a-0000-000000000000` **and** a v4 UUID both `ProjectId::from_str` / `SessionId::from_str` **Ok**. Garbage `not-a-uuid` **Err**. Locks F15. |
| **AC3** | Hermetic (`hermetic_bin` + `isolate_empty_home` + tempdir + `--vault-path`): `init`; write `.env` with **hashed-shape** dest project + v4 session (neither in vault); `context` (no `--new-project`); exit **0**; stdout contains `Context is already initialized for project` **and** exact `Vault: project and session present.`; **does not** contain `Local .env updated successfully.`; `.env` **bytes equal**; `project list --format json` contains the dest `project_id`. **Required red** before early-return is lifted. |
| **AC4** | Same fixture: print-only `project rebind-path <registered-from-path> --to <dest-id> --format human` exit **0**; stdout is **not** dest-missing (`not found in vault` absent). Need a **from** project that owns the path (T259-class seed). **Did not** pass `--write`. |
| **AC5** | Stay-green: `test_cli_context_idempotency` still exit 0, contains `already initialized`, `.env` equal on second run; `--new-session` still rewrites. T282 `--show` leftover suite. T259 `project_rebind_path__dest_missing__exit_1` (`00000000-…0077`) still exit 1. |
| **AC6** | Hermetic: session-only `.env` (`SESSION_ID` valid, **no** `PROJECT_ID`); `context` exit **0**; `.env` bytes equal; `project list` JSON **does not** contain the hashed/discovered cwd id that `context` would have minted on first-init. Locks F4 skip. |
| **AC7** | Hermetic: valid project UUID + `AI_BRAINS_SESSION_ID=not-a-uuid`; exit **1**; stderr contains `AI_BRAINS_SESSION_ID`; `.env` bytes equal; event count unchanged. |
| **AC8** | Hermetic: `--show` on the AC3 fixture does **not** add dest to `project list`; `.env` unchanged. |
| **AC9** | Docs + help: CAPABILITIES Init row names vault upsert / no rewrite; OPERATIONS context paragraph; WORKFLOWS leftover `context` is true; Context `--help` after_help or docstring states already-initialized **does not rewrite** `.env` **and** ensures vault. Hermetic `context --help` contains `already` (or `initialized`) **and** `vault` (or `does not rewrite` / `.env`). Rebind `--help` still says dest is not minted by rebind. |
| **AC10** | Manual (on go, `cargo run -p ai-brains-cli --`, no `--daemon`): hermetic AC3/AC4 commands. **Optional live leftover:** print-only rebind dest-missing **before** (already recorded). Live `context` in crawlx/degoo/kinledger **only if owner confirms**. Live `--write --yes` **only if owner confirms per path**. Default: skip live mutate. **Do not** `cargo install`. |
| **AC11** | First-init (no `.env`) still writes `.env` and prints `Context initialized` + `Local .env updated successfully.` (existing smoke / a focused assert). |
| **AC12** | Feature-off (default CLI test bin): AC3 still registers dest (capture/SQL, not graph). |
| **AC13** | No production `unwrap`/`expect`/`panic` in the touched `context.rs` arm. |
| **AC14** | `context --help` does **not** contain `set-alias` + leftover dump UUID + `AI-Brains` together (T259 F1). |

---

## 5. Design notes

### 5.1 Why not `--ensure-vault` / `--yes`

clig: make the default the right thing. The operator already wrote `.env`. Minting an empty `ProjectRegistered` that **matches** those IDs is additive identity, not a Scope switch. T258 requires `--write-env --yes` because it **changes** daily Scope. T259 requires `--write --yes` because it **moves** a path alias. T294 does neither.

### 5.2 Why not mint dest inside `rebind-path`

T259 F9 is still right: rebind is path-alias surgery. Inventing a project as a side effect of `--to` would hide identity mistakes. `context` is the dest factory. This track makes that factory work when `.env` already exists.

### 5.3 Why not adopt-path

T258 binds daily Scope to the **path owner** (leftover dump for crawlx). Operators in crawlx want dest = **crawlx `.env` hashed id**, then rebind the path **off** leftover **onto** that dest. `adopt-path` would point crawlx Scope at `7d97a456` — worse.

### 5.4 Control flow (already-initialized)

```text
if session_in_env && !new_session && !new_project:
    print already-initialized + ids
    parse project + session (F3)
    if both Ok:
        ensure_project_and_session_exists(env_pid, env_sid, harness, LocalOnly)
        print "Vault: project and session present."
    else if session Ok && project missing:
        skip ensure   # F4
    else if session garbage && project Ok:
        return Err(exit 1)  # F14
    return Ok(())   # never write .env, never sync pull
```

### 5.5 Fixture for AC3/AC4

1. `init` vault in tempdir.
2. Register **from-project** + path alias (reuse T259 fixture style; do **not** edit `project_rebind_path.rs` unless importing a helper already `pub(crate)` — prefer local seed in the new test file).
3. Write `.env` with dest hashed-shape UUID + session v4 (not in vault).
4. `context` → dest in `project_projection`.
5. `rebind-path <path> --to <dest> --format human` print-only exit 0.

### 5.6 Why `Vault:` line

clig: if you change state, tell the user. Ensure is idempotent; “present” is true after both mint and no-op. Smoke only `contains("already initialized")` — extra line allowed.

---

## 6. Non-goals

- Silent `.env` rewrite / T240 F2 reopen / T258 steal.
- Dest mint inside `rebind-path`.
- Auto-rebind leftover 5 roots / bulk `--all`.
- `MemoryMoved` / leftover memory reclassify.
- Live leftover `--write --yes` without owner confirm.
- Silent mint for gimp/homebrew-tap (no `.env`).
- `--new-project` / `--show` restyle.
- `project.rs` hotspot growth.
- clap 5 / rusqlite 0.40 / new crates / workspace 0.1.3.
- `cargo install` / PATH.
- PROTOCOL-COMPAT JSON row.
- T295–T300 peers.
- Identity mismatch leftover vs `fcb8a40f` (T258).

---

## 7. Verification plan (TDD)

**Must fail red before F1 exists:**

- `file_session_id_from_env_text__padded_value__trimmed`
- `context__already_initialized_foreign_hashed_id__upserts_env_bytes_unchanged`
- `context__already_initialized_foreign_hashed_id__rebind_print_only_dest_exists`

**Then green** with ensure on the already-initialized arm.

**Stay-green (run, do not “fix” unless this track broke them):** smoke idempotency; T282 `--show`; T259 dest-missing AC8; T258 adopt-path.

**Manual:** AC10. Unique canary pin **not** required. Live leftover mutate **not** required.

---

## 8. Risk

| Risk | Mitigation |
|------|------------|
| Second `context` in a healthy repo appends duplicate `ProjectRegistered` | ensure projection `SELECT 1` (existing). AC5 event-count optional on already-present dest. |
| Hashed UUID `FromStr` rejects leftover dests | AC2 hashed-shape; uuid `parse_str`. |
| `.env` rewrite sneaks in | AC3 byte-equal; F2; do not call `fs::write` on this arm. |
| Live leftover `context` pollutes daily vault during implement | F27 Stop-Before; hermetic DoD. |
| `project.rs` hotspot growth | F13; helpers in `context.rs`. |
| Operators think T294 rebound 5 roots | F11/F12 docs honesty; list-paths still leftover. |
| Session-only mints hashed sandbox | F4 / AC6. |
| PATH-behind | F18; hermetic/source. |
| `context.rs` hotspot #5 grows a lot | F13 split only if net ≥80 production. |

---

## 9. Deferred absorb / decline

### 9.1 Open overlapping rows (entire `deferred.md` scan)

| Item | Disposition |
|------|-------------|
| leftover dest-missing / context skip upsert (T285–T293 decline pointers) | **Absorb** F1–F4 / AC3–AC4 / AC10 |
| Placeholder Manual `context` + `project list` contains id + print-only rebind dest exists | **Absorb** AC3 / AC4 / AC10 |
| T259 F9 dest must exist / runbook `context` first | **Absorb as honesty** F8 / F19 — rebind still does not mint; `context` now actually ensures |
| T259 F5 memories stay | **Affirm** F10 |
| T240 F2 no silent `.env` | **Affirm** F2 |
| T258 adopt-path / cwd `mismatch: false` | **Decline steal** F9 — Completed; opposite direction |
| T276 F9 live leftover `--write` | **Affirm Stop-Before** F11 |
| T282 `--show` leftover shell | **Decline steal** F6 — Completed |
| T283 list cwd-first | **Decline** — Completed |
| Identity leftover `7d97a456` vs `fcb8a40f` | **Decline** F26 — T258 / leftover data |
| T293 neighbors dump sessions | **Decline** — Completed `#209` |
| T295–T300 peers | **Decline →** those placeholders |
| T263 H2 / 750 ms / clap 5 / rusqlite 0.40 | **Decline** F24 |
| last-PR Cursor **#209** | **N/A empty** — **no T301** F25 |
| Closed T259/T258/T276/T282 DoDs | **Stay closed** |

### 9.2 Last-PR Cursor (#209 T293)

`gh pr view 209 --comments`, `pulls/209/reviews`, `pulls/209/comments`, `issues/209/comments`: **all empty**. Open PR on HEAD: **none** (Dependabot remotes only). **No leftover to mint. No T301.**

### 9.3 Closed rows

T274–T293 Completed rows stay closed. Do not reopen T240 F2, T255 750 ms, T263 H2, T259 dest mint in rebind, T258 silent adopt.

---

## 10. Implement order (on go)

1. Phase 0 re-verify early-return `:150`, ensure `:182`, T282 helpers, leftover 5 roots dest-missing, #209 empty, pins, hotspots.
2. Red: AC1 helper + AC3 hermetic upsert + AC4 rebind dest exists.
3. Green: F1 arm in `context.rs`; clap after_help; no `.env` write.
4. Stay-green AC5/AC6/AC7/AC8.
5. Docs F17/F19. CHANGELOG.
6. Phase-1 review → `review.md`. Cross-model FEATURE (`codex-review`).
7. Full gate. Publish implement-track Phase 6. **Do not** leftover `--write` unless owner confirmed.

---

## 11. Soft residuals

| Residual | Disposition |
|----------|-------------|
| PATH until `cargo install` | F18 |
| Live leftover 5 roots still on `7d97a456` until owner `--write --yes` | F11 |
| gimp / homebrew-tap still no `.env` until first-init | F28 |
| Minted dest label `(no alias) — {8hex}` | F38 |
| Quote-strip `.env` values | T282 F32 |
| Session leftover line on `--show` | T282 F27 |
| Identity `7d97` vs `fcb8a40f` | T258 |
| T295–T300 | Not stolen |

---

## 12. Touch map

| Path | Why |
|------|-----|
| `crates/ai-brains-cli/src/commands/context.rs` | F1 arm + F3 session helper + F32 line |
| `crates/ai-brains-cli/src/main.rs` | Context docstring + after_help; T259 after_help F19 |
| `crates/ai-brains-cli/tests/context_vault_upsert.rs` | **New** AC1–AC4 / AC6–AC8 / AC12 |
| `Docs/CAPABILITIES.md` | Init row |
| `Docs/OPERATIONS.md` | `:512` extend |
| `Docs/WORKFLOWS.md` | leftover `context` truth |
| `Docs/CLI-EXIT-CODES.md` | exit 1 malformed `.env` UUID |
| `CHANGELOG.md` | implement |
| `conductor/conductor.md` / `deferred.md` / `README-T285-T300-CLI-QUALITY.md` | this plan |

**Do not touch:** `project.rs`, `project_rebind.rs`, `project_adopt.rs`, `sync.rs` production, `ranking.rs`, CLI `preflight.rs`, `governed_common.rs`, `src/context.rs` ensure body (reuse), `ci.yml`, live `.env`, leftover `--write`.

---

## 13. AI fold-in

None yet. `/review-track 294` then `/fold-in`.
