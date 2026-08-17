# T257 — Identity warning + JSON stdout hygiene

- **Track ID:** T257-WarningJsonStdoutHygiene
- **Status:** **Planned** (plan-only until go; conductor stays **Pending**)
- **Category:** UX / CONTRACTS-adjacent (CLI-local; no DTO bump)
- **Owner:** —
- **Source:** Audit 2026-08-16 — friction #1; `scope resolve` **6/5**; `scope --format json` **7/6**; opportunity “warnings on stderr; JSON one object”
- **Depends on:** T240 mismatch warn (F3 once/process, F3b skips); T249 scope TTY human; T255 nightly JSON; T258 adopt-path remediator (source)
- **Blocks / feeds:** Scripted / agent `2>&1` parse of every JSON-effective command. Unblocks honest scores for scope / nightly JSON / whoami JSON. Does **not** fix daily Scope (T258) or leftover `7d97a456` (T259).
- **Absorbs:** Same identity warning on nearly every vault-open command; merged stdout+stderr is not one JSON object; `scope` JSON `"warnings": []` while the human just saw a mismatch; whoami prints “run whoami”; dry-run / table mid-block under merged streams; T259 residual “T257 owns JSON interleave” (identity-warn half only)
- **Not absorbed:** Fixing the underlying mismatch (T258); leftover split (T259 Completed); format-default maze (T266); preflight `{text, word_count}` envelope (T265); T223/T242 env-override warn; T206 git/env detect warn; clap 5 / new crates
- **Research date:** 2026-08-17 (plan dogfood HEAD `ed329b1`; fold-in HEAD `2b3f859`)
- **AI fold-in:** 2026-08-17 `agy-review.md` only (no opencode/grok/claude/codex-plan). No Blockers / Majors. **Agree:** Agy-m1 HEAD note. **Already covered / tightened:** Agy-m2 both `scope.rs` emit sites (F3/F24/**AC17**); Agy-O1 `print_json_stdout` lives in `identity_warn.rs` (F8/F11). Disposition **§13**.
- **Ledger:** planning DOCS TX `b033f134-fb4a-4eb4-bf07-b46087a83a71`. Fold-in DOCS TX `886450fe-3c49-4b44-9073-f5d598297d5a`. Implement starts a **FEATURE** TX on **go**.
- **Isolation:** Do **not** write the live repo `.env`. Do **not** `cargo install`. Do **not** reopen T240 F2, T255 declines, T258, T259, T266. Do **not** bump clap / add crates. Do **not** print `AI_BRAINS_KEY`.

---

## 1. Objective

Make JSON-effective CLI stdout a **single parseable object** even when the caller merges stderr (`2>&1`, agent capture). Keep the T240 identity-mismatch line as a **human** diagnostic.

Three layers, no Scope rewrite:

1. **Human SOOT never on stdout.** The line `Warning: project identity mismatch: … Run 'ai-brains project whoami'.` is stderr-only today (`eprintln!`). Keep that. Do not print it before `{`, mid-object, or inside a dry-run / table block on stdout.
2. **JSON-effective commands suppress the human line.** When the command actually emits JSON, do **not** eprintln the SOOT line. `scope resolve --format json` (and default `auto` on a pipe) injects a **stable token** into the existing `warnings[]` array. `project whoami` already has `mismatch: true`. Nightly status has no `warnings[]` — stay silent; identity is whoami’s job.
3. **Human remediator is quiet.** `project whoami` and `project adopt-path` already show the triangle / adopt verb. Do not also print “run whoami”.

This advances the north star because capture independence includes **agent transcripts**. Today every vault-open command appends a human warning that turns `2>&1 | ConvertFrom-Json` into “Additional text encountered… W”. Scripts and harnesses cannot consume the vault as a machine. No events, no models, no graph.

---

## 2. Live baseline (2026-08-17)

### 2.1 Operator dogfood (this machine)

| Signal | Observation |
|--------|-------------|
| HEAD | Plan-time dogfood: `ed329b1` (T259 `#172`). This fold-in: `2b3f859` (T257 plan docs). Product `src/` for warn/`emit_json`/`scope.rs` **unchanged** since `ed329b1`. Tree CLEAN at fold-in. `main` ahead of `origin/main` by **1**. |
| PATH `ai-brains` | `C:\Users\RyanB\.cargo\bin\ai-brains.exe` (mtime **2026-08-16 08:04**, 24 796 672 bytes). **PATH-behind** T256/T258/T259. Whoami remediations still say “operator rebind; no auto-write” + `project list`. **Do not `cargo install`.** |
| Source / debug | `target\debug\ai-brains.exe` (mtime 2026-08-16 22:26). Whoami remediations **name `adopt-path`**. Same identity warn on stderr. |
| `preflight --summary` | Scope `test-alias` (`441837f6`); 568 pinned; warn → whoami. Live `.env` not rebound (T258 out of band). |
| Split-stream classify (PATH; stdout / stderr files) | **Every** vault-open command: `stdout_has_warn=false`, `stderr_has_warn=true`. JSON stdout **parses**. Human stdout has no SOOT. |
| `scope resolve` / `--format json` | Both JSON on this pipe. stdout 303 bytes, starts `{`. `warnings` **empty array**. stderr 190 bytes = one SOOT line. |
| `2>&1 \| Out-String` then `ConvertFrom-Json` | Starts with `{ "api_version"`. **Fails:** `Additional text encountered after finished reading JSON content: W` (line 15). Merge order today = **JSON then Warning**. |
| Audit 2026-08-16 mid-object (`{` then warn then `"api_version"`) | **Not reproduced** with split files or `2>&1 \| Out-String` on this PATH binary. Still a failed parse either way. Do not plan a third writer. |
| `nightly --status --format json --quick` | stdout parses (1487 bytes). stderr SOOT. |
| `nightly --schedule --dry-run` | stdout is two lines: `[dry-run] Would execute:` + `  schtasks /create …`. stderr SOOT. Split is clean. Merged stream is preview **then** warn. |
| `project whoami --format json` | `mismatch: true`. env `441837f6`, path `3581317d`. stderr SOOT **and** the report. PATH remediations stale; **source** names adopt-path. |
| `project list` | Human table on stdout. stderr = SOOT + set-alias footer (T267). T240 hermetic locks this warn. |
| `doctor --format json` `2>&1` | **Parses.** Doctor is early-routed **before** `AppContext` / `maybe_warn`. No identity SOOT. Keep. |
| Last GitHub PR | [#172](https://github.com/Ryan-AI-Studios/AI-Brains/pull/172) T259 merged 2026-08-17. `gh pr view --comments`, `/reviews`, `/comments`, issue comments all **empty**. HEAD is `main` (no open product PR). Open PRs are Dependabot #58–#72 / #68–#72. **last-PR Cursor: N/A.** |
| Ledgerful | `doctor` ready (legacy `.changeguard` / sig-pin / timings / :8081 unreachable). 0 pending at plan start. Work root `C:\dev\AI-Brains`. Hotspot **#1** = `project.rs` (**1549** lines, 3.808). `governed_common.rs` **864**. `nightly_status.rs` **592**. |
| ai-brains recall | Scoped to test-alias (T258 hole still live). Lexical/semantic: T240 once/process; T248/T249 format; T258 adopt-path. No prior “JSON stdout hygiene” pin. |

### 2.2 Why this still matters

| Residual | Why it is a product hole / why decline |
|----------|----------------------------------------|
| Human SOOT on every vault-open process | T240 F3 is **once per process**, not once per day. Each `ai-brains` spawn is a new process. Agents feel “every command.” **DoD: keep once/process; skip remediator commands; suppress on JSON-effective.** |
| `2>&1` is not one object | clig.dev: machine output on stdout, messaging on stderr. Agents and PowerShell **do** merge. Split-stream parse already works. **DoD: JSON-effective = no SOOT on either stream.** |
| `scope` `"warnings": []` | Wire object **denies** the mismatch the human just saw. `warnings[]` already exists (T158/T249; T180 E1 `[]`). **DoD: additive token.** |
| whoami prints “run whoami” | Remediator + diagnostic. Source remediations already name adopt-path. **DoD: skip stderr on whoami / adopt-path.** |
| Dry-run mid-block (audit) | Live split is clean. Merge puts warn **after** the preview (F4 “after the block”). Delay eprintln to `handle_cli_result` so it cannot sit between `Would execute:` and `schtasks` even if a future writer flushes early. |
| Mid-object `{` / warn / fields | Not reproduced 2026-08-17. `emit_json` is `to_string_pretty` + **one** `println!`. Do not invent a custom serializer. |
| Doctor has no warn | Early-route. Identity is whoami. **Decline** adding AppContext to doctor just to warn. |
| T223 env-override line | Different SOOT, already has quiet + session marker. **Decline.** |
| Format-default maze | **T266.** Do not change `scope` / `list-paths` / retention defaults. |
| Fix `441837f6` vs `3581317d` | **T258.** Print-only adopt-path exists in source. Do not write live `.env`. |

### 2.3 Code truth

| Item | Location | Notes |
|------|----------|-------|
| Warn emit | `project.rs` `maybe_warn_identity_mismatch` **:332–356** | `eprintln!` + `Once`. Called from `main.rs` **:3275** after `AppContext::from_cli`, **before** the command `match`. |
| SOOT line | `identity_mismatch_warn_line` **:325–328** | Exact T240 F3 string. Hint is `project whoami`. |
| Skip | `should_skip_identity_mismatch_warn` **:308–322** | `--no-project-context`, `--global`, empty env, no path. **No** whoami / adopt-path skip today. Units **:1381–1409**. |
| Hermetic lock | `tests/project_identity_convergence.rs` **:407** | `project list` (human) **must** still have stderr SOOT (T240 AC4). whoami JSON test asserts `mismatch`, **not** stderr SOOT. |
| Scope JSON | `scope.rs` `emit_json(&wire)` via `governed_common::emit_json` **:197–200** | `to_string_pretty` + `println!`. `warnings` cloned from CP (`map_resolved_scope` **:565**). |
| Scope format | `resolve_scope_format` → `format_resolve::resolve_human_json_format` | `auto` + pipe → json (T249). Default non-TTY is JSON. |
| Whoami JSON | `project.rs` `whoami` **:754–755** | Own `println!(to_string_pretty)`. `mismatch` **:811–814**. Source remediations **:823** name adopt-path. |
| Nightly JSON | `nightly_status.rs` `emit_nightly_status_json` **:134–137** | Pretty string; **no** `warnings` key (T255 F5 freeze). Do **not** add one. |
| Dry-run | `nightly.rs` `format_schedule_dry_run_preview` **:1131–1132** | One string, two lines. Other arms `println!("[dry-run] Would execute:")` then later the command — delay warn so it cannot land between. |
| Flush hook | `main.rs` `handle_cli_result` **:2868** | Single Ok/Err join after `run` **and** after `run_sync_path_free`. Warn-pending flush goes **here** (path-free commands never computed a pending warn). |
| Early-route (no warn) | doctor / init / recovery / encrypt / harness / schemas | Keep. `print_schema` (**main.rs:48**) is vault-path-free. |
| Hotspot | `project.rs` **1549** / **#1** | New emit/pending helpers live in **`identity_warn.rs`**. Do not add a product verb to `project.rs`. Moving the existing warn fns **out** is in scope. |
| `to_string_pretty` stdout | ~15 `commands/*.rs` sites | Hook via shared `print_json_stdout` (notes machine mode). **Not** harness file writes, **not** `print_schema`. |
| Contracts | `ScopeResolvedResponse.warnings: Vec<String>` | E1 default `[]`. T180 additive array content. **No** new field. |
| PROTOCOL-COMPAT | scope JSON **keys** frozen | `warnings` already listed. Adding a string is not a key change. |

### 2.4 Dependency / standards research (2026-08-17)

| Pin / claim | Workspace / lock / live | Action |
|-------------|-------------------------|--------|
| `clap` | workspace **4.5** / lock **4.6.1** (builder **4.6.0**) / crates.io **4.6.6** (2026-08-06) | **No bump.** No new flag. Snapshot — re-verify at execute. |
| clap 5 | not released (max 4.6.x) | Forbidden unless this track is that bump. |
| `serde_json` | lock **1.0.150** / crates.io **1.0.151** | **No bump.** Keep `to_string_pretty` + one write (docs.rs: serialize to `String`, then print). Do **not** switch to incremental `to_writer` on stdout. |
| `uuid` / `dirs` / `dotenvy` | lock **1.23.1** / **6.0.0** / present | **No bump.** No `.env` write. |
| rustc / edition | **1.95.0** / **2024** | Unchanged. |
| workspace version | **0.1.1** | **No bump.** |
| New crates | — | **Zero.** |
| clig.dev (fetched 2026-08-17) | “Send output to stdout”; “Send messaging to stderr”; machine-readable on stdout | Fits. Agents that merge streams are why JSON-effective must also **omit** the message. |
| GNU / POSIX | diagnostics on stderr | Already true (`eprintln!`). Remaining hole is merge, not the fd. |
| PowerShell `2>&1` | Native stderr becomes `ErrorRecord`; `Out-String` appended **after** stdout today | AC9 concat-stdout-then-stderr is the portable proof (not a live PS wrapper). |
| SQLCipher / schtasks / daemon | N/A | Status/dry-run already exist. Do not mutate tasks. |

Training data is not a pin. “I think clap already does JSON hygiene” is not evidence — clap is unused for this warn.

---

## 3. Frozen decisions

| ID | Decision |
|----|----------|
| **F0 — Go gate** | Plan-only until user **go**. Planning is DOCS. Implement starts a **FEATURE** TX. |
| **F1 — stdout purity** | The T240 SOOT line **never** appears on stdout (before `{`, mid-object, after `}`, or inside a table / dry-run preview). |
| **F2 — JSON-effective silent** | When the command emits JSON on stdout, do **not** eprintln the SOOT line. Concat(stdout, stderr) of a successful JSON command must `serde_json::from_str` as one value. |
| **F3 — scope token** | If mismatch fired and the surface is `scope resolve` JSON, push **exactly one** stable token onto `warnings[]` via a shared inject helper **before** each `emit_json(&wire)` in `scope.rs`. Token SOOT: `project_identity_mismatch env=<uuid> path=<uuid>`. Must **not** contain `Warning:` or the T240 sentence. |
| **F4 — do not spray** | Do **not** inject the token into briefing / retention / erasure / evidence / review / conclusion `warnings[]`. Those arrays are domain honesty (T165/T166/T180). |
| **F5 — remediator skip** | `should_skip` is true when argv is `project whoami` or `project adopt-path` (consecutive tokens). whoami JSON keeps `mismatch: true`. Do not add the SOOT string to `remediations[]`. |
| **F6 — once / delay** | Keep `Once` + F3b skips. **Do not** eprintln at `main.rs:3275`. Record pending state; `handle_cli_result` flushes human SOOT only when pending && !machine_stdout && !skipped. |
| **F7 — T240 AC4 stands** | Human `project list` (hermetic) **still** prints SOOT on stderr. Updating that test to “never warn” is a regression. |
| **F8 — `print_json_stdout`** | One helper **in `identity_warn.rs`**: `to_string_pretty` + `println!` + `note_machine_stdout()`. Hook CLI **stdout** pretty printers under `commands/` (emit_json, emit_error JSON, whoami, nightly_status, list/paths/adopt/rebind, preflight, memory, graph, briefing, harness **status**, replicate, governed_query). **Not** harness file writes. **Not** `print_schema`. Do **not** put the helper in `format_resolve.rs`. |
| **F9 — contracts** | No new field on `ScopeResolvedResponse`. No `ai-brains-contracts` bump. PROTOCOL-COMPAT: keys unchanged; document additive token in CAPABILITIES. |
| **F10 — pins** | No clap 5, no lock bumps, no new crates, workspace **0.1.1**. |
| **F11 — hotspot** | New module `crates/ai-brains-cli/src/commands/identity_warn.rs` (pending state, skip, token, flush, **`print_json_stdout`**). Do **not** grow `project.rs` with new verbs. Moving existing warn fns **out** of `project.rs` is the intended shrink. |
| **F12 — capture independence** | Warn/docs only. No events. No models. No graph. No `.env` write. |
| **F13 — PATH-behind** | Do **not** `cargo install` unless the user asks. Tests use hermetic / `cargo run` / debug bin. |
| **F14 — T240 F2** | No silent Scope switch. No live `.env` rewrite. |
| **F15 — T255 JSON freeze** | Nightly status keys **unchanged**. No `warnings` key. F2 silence is enough. |
| **F16 — T249 format** | Do not change `scope` default `auto` or token map. T266 owns the maze. |
| **F17 — T223/T242** | Env-override warn is out of scope. Do not reuse `AI_BRAINS_QUIET_ENV_WARN`. No new global `--quiet`. |
| **F18 — doctor** | Stay early-route / warn-free. Do not open the vault just to warn. |
| **F19 — dry-run** | After F6, human dry-run stdout is the preview only. If SOOT prints, it is stderr **after** the command (handle_cli_result). Never between `Would execute:` and `schtasks`. |
| **F20 — FEATURE + cross-model** | New machine-visible `warnings[]` content + 2>&1 contract. After Phase-1 review clean, run read-only `codex-review`. |
| **F21 — debt file** | `conductor/ISSUES.md` does **not** exist. Deferrals → `deferred.md`. |
| **F22 — tests** | Naming `function_or_feature__condition__expected_result`. New hermetic file `tests/warning_json_stdout_hygiene.rs`. Do not rewrite T240 / T249 / T255 suites except if a skip change breaks an **asserted** stderr line (only `project list` does). No `unwrap`/`expect`/`panic` in production. |
| **F23 — is-terminal** | `std::io::IsTerminal` (already migrated 2026-08-16). Do not add `is-terminal` crate. |
| **F24 — both scope emit sites** | Live `scope.rs` has **two** `emit_json(&wire)` calls: `run_resolve_local` **:94** and `run_resolve_daemon` **:128**. Both must call the shared inject helper first. Do **not** change the daemon resolver / `ai-brainsd`. Default hermetic `scope resolve` is local; AC17 is the daemon-arm lock. |
| **F25 — token idempotent** | Inject only if `warnings` does not already start with `project_identity_mismatch`. Never duplicate. |
| **F26 — decline extras** | T258 live rebind; T259 leftover mutate; T260–T271 except the absorb row; clap ValueEnum; color/pager; `comfy-table`; shared format-policy (T266). |

---

## 4. Acceptance criteria

| AC | Proof |
|----|-------|
| **AC1** | Unit: `should_skip_identity_mismatch_warn` is **true** for argv containing consecutive `project` `whoami` and consecutive `project` `adopt-path`, even when env and path differ. Existing F3b cases stay true. Differing env/path with `project list` stays **false**. |
| **AC2** | Unit: `identity_mismatch_json_token("e", "p")` == `project_identity_mismatch env=e path=p`. Does not contain `Warning:`. |
| **AC3** | Hermetic: `scope resolve --format json` on a mismatch fixture exits **0**. stdout parses as one object. stdout has **no** `Warning:` / `project identity mismatch` as raw text. stderr has **no** T240 SOOT. `warnings` contains the AC2 token with both fixture UUIDs. |
| **AC4** | Hermetic: existing `project_list__env_differs_path__mismatch_warn` (or equivalent) **still** finds T240 SOOT on **stderr** and not on stdout. |
| **AC5** | Hermetic: `project whoami --format json` mismatch fixture: stdout parses; `mismatch == true`; stderr has **no** T240 SOOT. |
| **AC6** | Hermetic: `project whoami --format human` mismatch fixture: stdout has no T240 SOOT; stderr has no T240 SOOT; remediations / human text still name `adopt-path` (source contract). |
| **AC7** | Hermetic: `nightly --status --format json` (temp vault, `--quick` if required) exits **0**; stdout parses; no T240 SOOT on stdout or stderr; **no** new `warnings` key. |
| **AC8** | Hermetic / unit: dry-run preview string has no T240 SOOT. Process: `nightly --schedule --dry-run` stdout does not contain `project identity mismatch`; if stderr has SOOT, it is **not** inserted between the two preview lines on stdout. |
| **AC9** | Hermetic: `String::from_utf8(stdout) + String::from_utf8(stderr)` of AC3 parses as one JSON value (`serde_json::from_str`). This is the `2>&1` proof. |
| **AC10** | Existing T240 identity hermetics (whoami fields, detect path-wins, `--no-project-context` no-warn) stay green. |
| **AC11** | Docs: CAPABILITIES mismatch-warn row (JSON-effective silent + token + remediator skip). Root CHANGELOG T257. PROTOCOL-COMPAT scope row: keys unchanged; `warnings[]` may include the token. |
| **AC12** | No contracts DTO field; no pin bumps; no new crate; `project.rs` line count does not grow (move-out allowed). |
| **AC13** | Hermetic: `--no-project-context` and `--global` still produce **no** T240 SOOT (list or recall). |
| **AC14** | Hermetic: `scope resolve --format human` mismatch: human stdout has no T240 SOOT; stderr **has** T240 SOOT (human non-remediator). |
| **AC15** | Unit: inject helper does not duplicate the token if `warnings` already contains it. |
| **AC16** | Manual (source bin, classify-only): live `scope resolve --format json` split + concat parse; do **not** paste keys; do **not** write `.env`. |
| **AC17** | Both `scope.rs` JSON arms call the same inject helper before `emit_json`: `run_resolve_local` (`:94` today) and `run_resolve_daemon` (`:128` today). Proof = review grep / unit on the helper (AC2/AC15). **No** live-daemon hermetic required. |

---

## 5. Design notes

### 5.1 Pending state

```text
AppContext::from_cli
  → record_identity_mismatch(ctx)   // Once; no eprintln
  → command match (may call print_json_stdout → note_machine_stdout)
  → handle_cli_result
       flush_identity_mismatch_warn()  // eprintln only if pending && !machine && !skip
```

`record_identity_mismatch` is the renamed body of `maybe_warn_identity_mismatch` without the print. Store `Option<Mismatch { env, path }>` in a `OnceLock`. Skip paths store `None` and mark skipped so flush is a no-op.

### 5.2 Token (frozen)

```text
project_identity_mismatch env=441837f6-5c55-d075-0000-000000000000 path=3581317d-601e-44f7-ab84-fde90aa12d3c
```

Prefix `project_identity_mismatch` is the machine key. Ids are the same strings T240 already prints. Scope JSON **keys** stay `api_version`, `scope`, `confidence`, `authoritative`, `evidence`, `warnings`, `alternatives`.

### 5.3 Why suppress on JSON instead of “stderr only”

clig.dev is already satisfied for a **correct** consumer (jq reads stdout). The audit hole is agents that merge streams. The only merge that `ConvertFrom-Json` accepts is **no extra text**. F2 is that contract. Human TTY / human-default pipes (`project list`, `nightly --status` default human, dry-run) still get stderr SOOT (F7/F14/F19).

### 5.4 PATH vs source

PATH remediations are pre-T258. Source `project.rs:823` names adopt-path. Phase 0 on go re-reads **source**. Do not conclude adopt-path is missing. Do not `cargo install`.

### 5.5 Capture independence

No `MemoryPinned`, no compensating events, no vault writes. Identity remains a read of env + path-alias projection.

---

## 6. Non-goals

- Rebinding live `.env` / daily Scope (T258)
- Splitting leftover `7d97a456` (T259 Completed)
- `project list` footer algorithm (T267)
- Format-default maze / list-paths TTY (T266)
- Preflight `{text, word_count}` (T265)
- T223/T242 env-override chrome
- Doctor 16th check / AppContext for doctor
- Adding `warnings` to nightly status JSON
- clap 5 / pin bumps / new crates
- Silent Scope switch (T240 F2)
- `cargo install` / PATH refresh
- Mutating `AI-Brains-Nightly` / Router

---

## 7. Verification plan (TDD)

Red first (new file; expect fail because JSON commands still eprintln **or** `warnings[]` empty):

1. `scope_resolve_json__mismatch__stdout_parses_token_no_soot` (**must red** on empty `warnings` and/or stderr SOOT)
2. `scope_resolve_json__mismatch__concat_streams_parse` (**must red** — AC9)
3. `whoami_json__mismatch__no_stderr_soot` (**must red** if skip not implemented)
4. `whoami_human__mismatch__no_stderr_soot` (**must red**)
5. `nightly_status_json__mismatch__no_soot_no_warnings_key`
6. Guards (green ok): T240 `project_list` stderr SOOT; `--no-project-context`; `--global`

Units in `identity_warn.rs`: AC1 / AC2 / AC15.

Then green: record+flush+token+`print_json_stdout`. Re-run T240 / T249 / T255 targeted nextest.

---

## 8. Risk

| Risk | Mitigation |
|------|------------|
| T240 AC4 goes red | F7; do not suppress human `project list` |
| whoami JSON test looks at stderr | It does not (fields only). AC5 is additive |
| Token treated as a new DTO field | F9 / PROTOCOL-COMPAT keys unchanged |
| Miss a `println!(to_string_pretty)` | F8 inventory; residual → §11 |
| Delay warn surprises humans | F6 prints **after** the block (F4 audit ask) |
| PATH binary still noisy | F13 honesty |
| Spray into retention honesty | F4 explicit decline |
| Hotspot `project.rs` grows | F11 extract |

---

## 9. Deferred absorb / decline

Entire `conductor/deferred.md` scanned 2026-08-17 (post-P12 through T259 closeout + T256–T271 map + historical T142–T196).

| Item | Disposition |
|------|-------------|
| Identity warn on every command / JSON interleave (`scope` 6/5, json 7/6) | **Absorb** F1–F6 / F8 / AC3–AC9 / AC14 |
| T240 F3 once/process + F3b skips | **Keep** F6; extend skip F5 |
| T240 AC4 hermetic list warn | **Keep** F7 / AC4 |
| T259 residual “T257 owns JSON interleave” | **Absorb** identity-warn / `2>&1` half. Rebind no-owner `COMMAND_FAILED` stays **T259 soft** |
| T258 daily Scope `441837f6` vs path owner | **Point** — do not steal; live `.env` out of band |
| T259 leftover `7d97a456` | **Closed** — do not reopen |
| T266 format maze / list-paths JSON wall | **Point** — F16 |
| T265 preflight JSON envelope | **Point** — not this token |
| T267 list footer leftover-as-AI-Brains | **Point** |
| T223/T242 env-override warn / `AI_BRAINS_QUIET_ENV_WARN` | **Decline** F17 |
| T206 git/env detect warn | **Decline** — detect-only |
| T249 F12 shared format helper | **Decline reopen** F16 |
| T255 declined bag (doctor 16th, persist probe, `.cmd`, clap 5) | **Decline** — stay declined |
| T255 nightly JSON keys | **Keep** F15 |
| T256 F18 PATH-behind | **Same class** F13 |
| T240 F2 no silent Scope switch | **Decline reopen** F14 |
| Doctor warn-free JSON | **Keep** F18 |
| R-CI-BRANCH / MSI / notarization / App Store | **Not related** — packaging / admin |
| `anyhow` RUSTSEC-2026-0190 allowlist | **Not related** |
| `#34.2` DataKey rotation | **Closed** T189 — not related |
| T142 archive `changeguard` strings | **Not related** |
| T210–T232 / T234–T255 / T260–T271 other rows | **Not related** unless they mention identity-warn/JSON interleave (only T257 row + T259 pointer) |
| last-PR Cursor (#172 + open HEAD PR) | **N/A** — #172 comments/reviews/inline/issue all empty; HEAD is `main`; open PRs are Dependabot (no Cursor/Bugbot findings). **No leftover to mint.** |
| Closed/strikethrough deferred rows | Stay closed |

---

## 10. Implement order (on go)

1. Phase 0: re-read `maybe_warn` / `emit_json` / both `scope.rs` emit sites / `handle_cli_result`; rescan `deferred.md` + last PR Cursor; clap still 4.6.x.
2. Red: `warning_json_stdout_hygiene.rs` + identity_warn units (AC1/AC2/AC3/AC5/AC9 must red).
3. Green: extract `identity_warn.rs`; record/flush; token inject on **both** scope emit sites (AC17); `print_json_stdout` in that module.
4. Docs AC11.
5. Targeted clippy + nextest (new file + T240 + T249 + T255).
6. Review + FEATURE `codex-review`.
7. Manual AC16 classify-only.
8. Full gate; conductor **Completed**; deferred closeout line.

---

## 11. Soft residuals

| Residual | Notes |
|----------|-------|
| PATH `ai-brains` still noisy until reinstall | F13 — operator `cargo install` |
| Recall / `sync query` JSON if a site misses `print_json_stdout` | F8 inventory; leftover site → follow-up, not a new track unless it fails AC9-class concat |
| Doctor never identity-warns | F18 honesty |
| T223 env-override can still trail JSON | Decline F17; separate SOOT |
| Human warn **after** the table (order change vs today) | F6; T240 asserts presence not order |
| `scope` human still says `next: whoami` (T249) | Keep; not the T240 SOOT |

---

## 12. Touch map

| Path | Change |
|------|--------|
| `crates/ai-brains-cli/src/commands/identity_warn.rs` | **New** — pending, skip, token, flush, print helper |
| `crates/ai-brains-cli/src/commands/mod.rs` | `pub mod identity_warn` |
| `crates/ai-brains-cli/src/commands/project.rs` | Move warn fns out; whoami JSON uses `print_json_stdout` |
| `crates/ai-brains-cli/src/main.rs` | `record_*` at :3275; `flush_*` in `handle_cli_result` |
| `crates/ai-brains-cli/src/commands/governed_common.rs` | `emit_json` / JSON `emit_error` → helper |
| `crates/ai-brains-cli/src/commands/scope.rs` | Shared inject before **both** `emit_json(&wire)` (`run_resolve_local` + `run_resolve_daemon`) |
| `crates/ai-brains-cli/src/commands/nightly_status.rs` | JSON emit notes machine |
| Other `commands/*` stdout pretty printers | Call helper (F8) |
| `crates/ai-brains-cli/tests/warning_json_stdout_hygiene.rs` | **New** hermetic ACs |
| `Docs/CAPABILITIES.md` | Mismatch-warn row |
| `Docs/PROTOCOL-COMPAT.md` | Additive token note on scope JSON |
| `CHANGELOG.md` | T257 row (on implement) |
| `conductor/tracks/trackT257-warning-json-stdout-hygiene/{spec,plan}.md` | This plan |
| `conductor/conductor.md` | Pending row text |
| `conductor/deferred.md` | Absorb note |
| `conductor/tracks/README-T256-T271-CLI-AUDIT.md` | T257 Planned |

Do **not** touch: `key_resolve.rs`, `help_ia.rs`, `ai-brainsd` resolver, contracts structs, nightly status key list, live `.env`, leftover path aliases.

---

## 13. AI fold-in disposition (2026-08-17)

Source: `agy-review.md` (Antigravity) only. No `opencode-review.md` / `grok-review.md` / `claude-review.md` / `codex-plan-review.md` in the track dir. No Blockers / Majors. Re-verified at fold-in HEAD `2b3f859`: `maybe_warn` still `eprintln!` + `Once` at `project.rs:332`; `main.rs:3275` still before the command match; `emit_json` still one `to_string_pretty` + `println!`; `scope.rs` still two emit sites (`run_resolve_local:94`, `run_resolve_daemon:128`); `warnings[]` still empty on live PATH `scope resolve --format json`. Review re-confirmed deferred + last-PR Cursor N/A — **no leftover to mint**. Product `src/` unchanged since plan dogfood `ed329b1`.

### Antigravity

| ID | Verdict | Action |
|----|---------|--------|
| **m1** spec HEAD `ed329b1` vs `2b3f859` | **Agree** | §2.1: plan-time dogfood SHA vs fold-in SHA. Product warn/`emit_json`/`scope.rs` unchanged. Phase 0 checkbox. |
| **m2** dual local + daemon inject | **Already covered** | F3 / F24. **Tightened:** F24 names `run_resolve_local` **:94** and `run_resolve_daemon` **:128**; **AC17** locks both arms. No live-daemon hermetic. |
| **O1** `print_json_stdout` in `identity_warn.rs` | **Already covered** | F8 / F11. **Tightened:** helper **must** live in `identity_warn.rs`; do not put it in `format_resolve.rs`. |

### Pins locked by fold-in

1. **§2.1:** dogfood HEAD `ed329b1` vs fold HEAD `2b3f859`; product src unchanged.
2. **F24 / AC17:** both `scope.rs` `emit_json(&wire)` sites get the shared inject helper.
3. **F8 / F11:** `print_json_stdout` lives in `identity_warn.rs`, not `format_resolve.rs`.
4. **F0** until go. No product crate edits this pass.

---

**Planning + fold-in 2026-08-17.** Still **plan-only until go**.
