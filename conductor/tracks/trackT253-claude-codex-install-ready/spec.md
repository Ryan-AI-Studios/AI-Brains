# T253 — Claude / Codex install_ready (T239+)

- **Track ID:** T253-ClaudeCodexInstallReady
- **Status:** 📋 **Planning** (plan-only until **go**)
- **Category:** FEATURE / HARNESS
- **Owner:** Grok
- **Source:** T238/T239 soft residual S8 / Claude-Codex labels **T239+**; T245 F13 fence; CLI-effectiveness series leftover; audit harness pending
- **Depends on:** T234 message-only ✅; T235 detect/install UX ✅; T236–T238 writer + wrapper lessons ✅; T239 nightly **does not** include these backends (stay excluded); T245 activation + PATH bake + doctor ready-vs-pending ✅
- **Blocks / feeds:** Live Claude Code + Codex message-only capture; `all-ready` includes them; doctor/preflight pending bucket empties; T254/T255 stay separate
- **Absorbs:** deferred.md “Claude/Codex install_ready (T239+)”; placeholder F1–F3 / AC1–AC2; T245 F13 fence + `pending_track` still `T239+`; T235 F14 Claude/Codex `backend_pending`; T239 D16 *labels only* (not nightly expansion)
- **Not absorbed (DoD):** Nightly multi-import of Claude/Codex (T239 D16 stays); SessionStart preflight injection; PreCompact archive; StopFailure ingest; SubagentStop / child ingest; project-scope hooks / Codex project trust as default; rewriting `parse_ingest_request` / T180 / T114 / T107; clap 5 / pin bumps; vault-free hook CLIs; MSI / MDM managed hooks; replacing historical `scripts/target-claude-hook.ps1` / `target-codex-hook.ps1` in-place (new wrappers only)
- **Research date:** 2026-08-15 (live binaries + official hook docs + crate pins + T234–T245 residuals)
- **AI fold-in:** none yet — fold in `C:\dev\AI-review.md` if the user asks, then lock disposition in **§12**
- **Ledger:** plan-only until go. Planning TX `3cadc357-5c06-46c6-b22d-6812bb9a2110` (DOCS). Implement go starts a new FEATURE TX: `ledgerful ledger start T253-claude-codex-install-ready --category FEATURE`
- **Isolation:** Do **not** add Claude/Codex to `run_multi_harness_import`. Do **not** rewrite T234 filter core (add harness helpers). Do **not** change `CapabilityLevel::Full` (tests lock it — honesty is **notes** + docs). Do **not** write repo-local `.claude/` or `.codex/` hooks (C7). Do **not** print or commit `AI_BRAINS_KEY`.

---

## 1. Objective

1. **Claude Code and Codex reach `install_ready`.** `harness install --harness claude|codex` writes real user-global hooks + PATH-baked wrappers. Wiring probe becomes `ok`. `all-ready` includes them. Pending labels (`T239+` / `T253`) go away.
2. **Capture is message-only and fail-open.** Live hooks ingest **user prompt text + final assistant text** only (T234). Never tool I/O, thinking, or SessionStart context injection. Capture never blocks stop.
3. **Honesty over fake ready.** Capability **notes** and CAPABILITIES say what actually works (live events, `/hooks` trust, no nightly). Do not claim nightly completeness or injection. If a surface is not message-only safe, it stays unwired — it does **not** stay `install_ready=false` after this track unless research is overturned on go.
4. **Stay capture-independent.** Detect / install / hook-payload parse must not open models, embeddings, or graph. Zero new crates. No pin bumps.

---

## 2. Live baseline (re-scan 2026-08-15)

### 2.1 Dogfood (this machine)

| Signal | Observation |
|--------|-------------|
| Binaries | **Claude Code 2.1.221** (`~\.local\bin\claude.exe`); **codex-cli 0.145.0** (`AppData\Roaming\npm\codex.ps1`) |
| Homes | `~\.claude` exists (**no** `settings.json` today — only `backups/`, `projects/`, `sessions/`, `telemetry/`). `~\.codex` exists (`config.toml`, `sessions/YYYY/MM/DD/rollout-*.jsonl`, **no** `hooks.json`) |
| `harness status` | grok / agy / opencode **present=yes wiring=ok install_ready=true**; claude / codex **present=yes wiring=missing install_ready=false**; next = `--dry-run  # backend pending (T239+)` |
| `preflight --summary` | Claude/Codex `wiring=missing (pending)` + same T239+ next |
| Doctor `harness_wiring` | Soft ok; splits ready-wired vs **T253 pending** (T245 F6). After this track the pending clause must go empty when both are ready |
| Codex `[features]` | Live `config.toml` has `[features]` / `js_repl = false` — **no** `hooks = false`. Official 2026: hooks **enabled by default**; `hooks = false` opts out; `codex_hooks` is a deprecated alias |
| Claude transcripts | `~\.claude\projects\<encoded-cwd>\<session>.jsonl`. Mixed record `type`s (`user`, `assistant`, `attachment`, `queue-operation`, `last-prompt`, …). User `message` keys `role,content`. Assistant `message` has `role,content` plus model/usage chrome |
| Codex transcripts | `~\.codex\sessions\YYYY\MM\DD\rollout-*.jsonl` records `{timestamp,type,payload}`. Types seen: `session_meta`, `event_msg`, `response_item`. `response_item` payload `type=message` with `role,content` |
| Legacy scripts | Repo still has `scripts/target-claude-hook.ps1` + `target-codex-hook.ps1` (May 2026 SessionStart injection). **Not** installed on this machine. Do **not** deploy them |

### 2.2 Why this is still pending

| Layer | Truth |
|-------|--------|
| Detect | Shipped (T235). Both present via PATH + home. |
| Writers | `install_pending` only — stamps prefs `backend_pending`, **zero hook files**. |
| Adapters | `parse_claude_stop_payload` is a naive `role`/`content`/`stop_reason` NeutralEvent (not T234, not transcript). Codex adapter is **capability-only** (no parser). Capability reports already claim `Full` + `supports_hooks: true` — **overclaim**. |
| Nightly | T239 batch is agy → grok → opencode only (D16). Stay that way. |
| Docs | CAPABILITIES/OPERATIONS/WORKFLOWS say T253 pending. May research docs (`Docs/Claude-Hooks-Research.md`, `Docs/Codex-Hooks-Research.md`) are **stale** vs 2026-08 official surfaces. |

T235–T245 closed detect + three ready backends. T253 closes the two remaining writers + message-only capture paths.

### 2.3 Code truth

| Item | Location | Notes |
|------|----------|-------|
| Readiness | `harness/detect.rs` `install_ready` | `Agy \| Grok \| Opencode` only |
| Pending labels | `HarnessId::pending_track` | Claude/Codex → `"T239+"` (doctor message uses literal `T253`) |
| Install dispatch | `commands/harness.rs` | Ready match arms + `unreachable!` for other ready ids |
| Pending stub | `install.rs` `install_pending` | Dry-run lists targets; real install stamps prefs only |
| Probes | `wiring.rs` `probe_claude` / `probe_codex` | Claude: `settings.json` contains `.ai-brains`+`hooks` **or** `ai-brains-capture`. Codex: `hooks.json` key/`ai-brains` token |
| Targets | `targets_for` | Claude: settings.json only (no wrapper listed). Codex: hooks.json only |
| `all-ready` | `ready_harness_ids` | Filters `install_ready()` — auto-includes after flip |
| Doctor F6 | `doctor.rs` `check_harness_wiring` | Tests assert `2 backend pending (T253): claude, codex` |
| Grok-hook pattern | `main.rs` `GrokHook` / `GrokImport` | `--payload` + `--schema`; schema under `Docs/schemas/` |
| T234 SOOT | `ai-brains-adapters` `message_only.rs` | `filter_turn` / `extract_user_text` — reuse, do not rewrite |
| T245 bake | `install.rs` `resolve_cli_exe_for_wrapper` | Reuse for new wrappers |
| Hotspots | `ledgerful hotspots` | `install.rs` / `detect.rs` **not** top 10. `preflight.rs` rank 7; `governed_common.rs` rank 9 — **call** `fail_usage`, do not grow |

### 2.4 Dependency / standards research (2026-08-15)

| Pin | Workspace / lock | Ecosystem | Action |
|-----|------------------|-----------|--------|
| `clap` | workspace **4.5** / lock **4.6.1** | crates.io **4.6.6** | **No bump** |
| `serde` / `serde_json` | **1.0** / lock **1.0.228** / **1.0.150** | 1.0.151 train | **No bump** |
| `dirs` | **6.0** / lock **6.0.0** | 6.0.0 | **No bump** |
| `is-terminal` | **0.4** / lock **0.4.17** | 0.4.17 | **No bump** |
| `uuid` | **1.13** / lock **1.23.1** | caret | **No bump** |
| rustc | **1.95.0** | pin | unchanged |
| nextest | **0.9.140** | min 0.9.140 | unchanged |

**Zero new crates.** JSON merge + existing `atomic_write_str` + PATH bake.

---

## 3. Official hook freeze (2026-08-15)

### 3.1 Claude Code 2.1.221

Source: [Hooks reference](https://code.claude.com/docs/en/hooks) (fetched 2026-08-15).

| Topic | Freeze |
|-------|--------|
| User-global config | `~/.claude/settings.json` `hooks` object. Merge additively across user/project/local/plugin. **C7: write user-global only.** |
| Structure | Event → matcher group[] → handler[] (`type: command`, `command`, optional `args` exec form, `timeout`, `statusMessage`, `name`) |
| Live events for capture | **`UserPromptSubmit`** (stdin `prompt`); **`Stop`** (stdin `last_assistant_message`, `transcript_path`, `stop_hook_active`, `cwd`, `session_id`); **`SessionEnd`** (end reasons `clear`/`resume`/`logout`/`prompt_input_exit`/`bypass_permissions_disabled`/`other`) |
| Transcript lag | Official: `transcript_path` **may not include the current turn’s final assistant** at Stop. Use `last_assistant_message` for the just-completed assistant text. |
| Stop allow | **Omit** `decision`. `decision: "block"` + `reason` continues the agent. Exit **2** also continues. Capture must **exit 0**, **no** `decision`, **no** `additionalContext`, **no** exit 2. Empty stdout is the allow path. |
| Matcher | UPS / Stop have **no matcher support** (silently ignored). SessionEnd matcher = end reason — omit / `*` to always fire. |
| Exec form | Prefer `command: "powershell.exe"` + `args: ["-NoProfile","-ExecutionPolicy","Bypass","-File","<wrapper>"]` (official Windows example). Avoid shell-form `$` expansion. |
| Timeout | Default 600s for command hooks; UPS default 30s. Pin handler `timeout: 30`. SessionEnd shared budget 1.5s (raise per-hook up to 60s) — SessionEnd is **safety-net only**, keep work tiny. |
| Cloud | Claude Code on the web **does not** read `~/.claude/settings.json`. Honesty: local CLI/IDE only. |

### 3.2 Codex CLI 0.145.0

Source: [Hooks](https://learn.chatgpt.com/docs/hooks) / [developers.openai.com/codex/hooks](https://developers.openai.com/codex/hooks) (fetched 2026-08-15).

| Topic | Freeze |
|-------|--------|
| Discovery | `~/.codex/hooks.json` **or** inline `[hooks]` in `config.toml`. Prefer **one representation**. T253 writes **`hooks.json` only** — never rewrite `config.toml` feature flags. |
| Default | Hooks **on by default**. `[features].hooks = false` opts out. `codex_hooks` is a **deprecated alias**. Do **not** write `codex_hooks`. Do **not** force `hooks = true`. |
| Trust | Non-managed command hooks must be **reviewed and trusted** (`/hooks`). Install writes files; **live fire requires operator trust**. Print that next-action. `--dangerously-bypass-hook-trust` is not our SOOT. |
| Project layer | `<repo>/.codex/hooks.json` needs project trust. **C7: user-global only.** |
| Live events | **`UserPromptSubmit`** field `prompt` (official). **`Stop`** fields `last_assistant_message`, `stop_hook_active`, `turn_id`. **`SessionEnd`** exists now (May docs said it did not); default timeout **1s**, max **3s**; output is advisory. |
| Transcript | `transcript_path` **is not a stable interface**. Do not treat rollout JSONL as a frozen public schema. Batch import is best-effort fail-open (F12). |
| Stop stdout | Event page: “expects JSON on stdout when it exits 0. Plain text is invalid.” Common page: “Exit 0 with no output is treated as success.” **SOOT:** emit exactly `{"continue":true}` (no `decision`, no `additionalContext`, no `systemMessage`). Never `decision: "block"`. Never exit 2. |
| SessionEnd | Too tight for vault ingest as primary path. **Soft** — optional empty/`continue` handler or skip. Live user+assistant is UPS + Stop. |
| `commandWindows` | Optional Windows override. Prefer the same `powershell.exe -File` command on all platforms we generate (Windows is DoD). |

### 3.3 Stale local docs (do not implement from these)

| Doc | Stale claim | 2026-08 truth |
|-----|-------------|---------------|
| `Docs/Codex-Hooks-Research.md` (2026-05-12) | Need `[features].codex_hooks = true`; no SessionEnd | Hooks default-on via `features.hooks`; SessionEnd exists; trust required |
| `Docs/codex-hooks.md` | Still shows `codex_hooks = true`; Level 4 + SessionStart injection | Injection **out of T253**; feature key is `hooks` |
| `Docs/Claude-Hooks-Research.md` / `Docs/claude-hooks.md` | SessionStart preflight + PreCompact archive as default loop | Capture-only; SessionStart/PreCompact **not DoD** |

Supersede those claims in CAPABILITIES/OPERATIONS and a short honesty banner on the research docs (do not rewrite them as product specs).

---

## 4. Problem analysis

1. **Writers were intentionally fenced** (T235 F14, T245 F13) until a message-only-safe backend existed.
2. **2026 official surfaces are now sufficient** for that backend: Claude `last_assistant_message` + project JSONL; Codex `prompt` + `last_assistant_message`.
3. **Capability already says Full** — operators who `harness install --harness claude` get a pending stamp, not capture. That is the honesty bug.
4. **Codex Stop is not Grok empty-stdout.** Emitting AGY `{"decision":"allow"}` or Grok empty stdout is undefined / invalid. Dedicated Codex allow JSON.
5. **Codex `/hooks` trust** means `wiring=ok` ≠ “will fire this session.” Docs + install stdout must say so.
6. **Grok still merges `~/.claude/settings.json`** (T237 D18). A Claude Stop wrapper **will** be invoked from Grok with a Grok stdin shape. Fail-open.
7. **Nightly expansion is a T239 contract change** (`MultiImportReport` sources, skip flags, SYSTEM skip). Not this track.

---

## 5. Frozen decisions

| ID | Decision |
|----|----------|
| **F0 — Plan-only** | No production code and **no live `harness install` without `--dry-run`** until **go**. Do not write `~\.claude\settings.json`, `~\.codex\hooks.json`, or `~\.ai-brains\hooks\claude-*.ps1` / `codex-*.ps1` until go. Dry-run research OK. |
| **F1 — Product is ready** | Do **not** leave Claude/Codex `install_ready=false` after implement. Official 2026 surfaces support T234-safe capture. “Stay pending” only if go-time research finds a block (then halt — Stop-Before). |
| **F2 — Flip readiness** | `HarnessId::Claude` and `HarnessId::Codex`: `install_ready() → true`; `pending_track() → None`. Update `install_ready__agy_and_grok` and every `T239+` / pending-only assert. |
| **F3 — `all-ready` / `all`** | After F2, `all-ready` iterates grok → agy → opencode → **claude → codex** (HARNESS_ORDER, no extra sort). `all` already includes them. |
| **F4 — Dispatch** | Replace `install_pending` / `uninstall_pending` for Claude/Codex with `install_claude` / `install_codex` / uninstall twins. Extend harness.rs match arms; keep the `unreachable!` only for theoretically ready-but-unmatched ids. |
| **F5 — Claude install writer** | Merge user-global `~/.claude/settings.json` (create `{}` if missing). Managed handler **name** `ai-brains-capture`. Events **UserPromptSubmit + Stop + SessionEnd**. Same wrapper `~\.ai-brains\hooks\claude-capture.ps1`. Exec-form `powershell.exe` + `-File`. PATH bake (T245 F8). Preserve foreign hook groups. Idempotent. Parse-fail → refuse rewrite exit 1. |
| **F6 — Codex install writer** | Merge user-global `~\.codex\hooks.json` (create `{ "hooks": {} }` if missing). Managed **name** `ai-brains-capture`. Events **UserPromptSubmit + Stop** only (SessionEnd not DoD). Wrapper `~\.ai-brains\hooks\codex-capture.ps1`. Same bake + refuse-on-corrupt. **Never** edit `config.toml`. If `[features].hooks = false` is set, still write `hooks.json` and **warn** + next `set hooks = true` (or remove the key). Never write `codex_hooks`. |
| **F7 — C7 / consent** | User-global only. `--yes` / TTY / `auto_install` unchanged. Never write repo `.claude/settings.json` or `.codex/hooks.json`. Hermetic targets under temp home. |
| **F8 — Claude Stop stdout** | Wrapper: suppress `claude-hook` stdout; host stdout **empty**; **exit 0**. Never `decision`, `continue`, `hookSpecificOutput`, exit 2. Diagnostics stderr only. Dedicated `claude_wrapper_*` helpers (do **not** reuse AGY allow JSON or Grok empty helper by name — may share a “empty stdout” string helper if that is clearer). |
| **F9 — Codex Stop / UPS stdout** | Wrapper: suppress hook CLI stdout; host stdout is **exactly** `{"continue":true}` (no trailing junk); **exit 0**. Never `decision: "block"`, never `additionalContext`. Dedicated `codex_wrapper_continue_stdout()`. Hermetic body test. |
| **F10 — Claude live content** | **Assistant SOOT:** `last_assistant_message` (official). **User SOOT:** `UserPromptSubmit.prompt` after `extract_user_text` / `filter_turn`. SessionEnd: ingest `last_assistant_message` if present (safety net); do **not** block on transcript completeness. Optional overlay of transcript user/assistant via `filter_turn` is allowed for SessionEnd **only** when records are `type=user\|assistant` with `message.content` text — drop every other `type`. Transcript lag means Stop must **not** wait on the file for the current assistant. |
| **F11 — Codex live content** | **User SOOT:** `UserPromptSubmit.prompt` + `filter_turn(user, …)`. **Assistant SOOT:** `Stop.last_assistant_message` + `filter_turn(assistant, …)`. **Do not** parse `transcript_path` on the live path (unstable). Empty prompt / empty last message → skip that role. |
| **F12 — Codex batch import (hard, fail-open)** | `codex-import [--days N] [--force] [--dry-run]`. Walk `~\.codex\sessions\**\rollout-*.jsonl` (or `CODEX_HOME`). Keep only records `type=response_item` whose `payload.type=message` and `payload.role` is `user`/`assistant`. Run `filter_turn`. Drop `event_msg` / `session_meta` / unknown. Malformed line → skip. **Honesty:** format is not a vendor-stable API; if it drifts, import soft-skips rather than claiming 0-session success as “complete.” |
| **F13 — Claude batch import (hard)** | `claude-import [--days N] [--force] [--dry-run]`. Walk `~\.claude\projects\<encoded-cwd>\*.jsonl`. Keep `type=user\|assistant` + `message.content` text (string **or** text parts; drop tool/thinking parts). Bind via decoded project folder → path alias (T237 percent-decode lesson). Unbound `claude-unbound`. Skip `subagents/` / `isSidechain=true` by default. |
| **F14 — CLI surface** | `claude-hook --payload|--schema`; `codex-hook --payload|--schema`; `claude-import` / `codex-import` flags mirror grok-import. Schemas `Docs/schemas/claude-hook-payload.json` + `codex-hook-payload.json`. Payload fields: `sessionId`, `projectHash` (`cwd` or `claude-unbound` / `codex-unbound`), `event`, plus role-specific text (`prompt` / `lastAssistantMessage`). `deny_unknown_fields`. help_ia **Harness** group. |
| **F15 — Turn ids + thinking** | Live: `v5(session, "{event}:{turn_id-or-stable}")` — prefer Claude record `uuid` / Codex `turn_id` when present; else `v5(session, "turn-{i}")` on kept index for batch. **`thinking` always `None`.** Delta skip existing turns. |
| **F16 — Binding** | `cwd` from hook stdin → normalize → path alias. Env `AI_BRAINS_PROJECT_ID` **only** when unbound. `allow_default_project: false` non-interactive. Path-keyed `source_meta:claude:…` / `source_meta:codex:…`. |
| **F17 — Nightly not DoD** | Do **not** add Claude/Codex to `run_multi_harness_import` / skip flags / `last_multi_import` schema. Docs keep “nightly = agy → grok → opencode.” Soft residual F34. |
| **F18 — SessionStart / PreCompact / StopFailure / Subagent** | **Not wired.** No additionalContext injection. No PreCompact archive. StopFailure error strings are not memory. SubagentStop / sidechain skip (T237/T238). |
| **F19 — Legacy scripts** | Do not install `scripts/target-*-hook.ps1`. New wrappers under `~\.ai-brains\hooks\`. Research docs get a banner: “historical; T253 wrappers supersede.” |
| **F20 — Probe / targets** | After install, `probe_wiring` → **ok** via managed name **or** wrapper path token (keep today’s substring probe **plus** require the managed `name` when JSON parses). `targets_for` lists settings/hooks.json **and** the wrapper. |
| **F21 — Doctor / preflight** | F6 pending bucket empty when both ready. Tests that require `T253: claude, codex` flip to the all-ready-wired / ready-missing arms. Preflight next for missing = `--dry-run` **without** `# backend pending`. Status footer lists Claude/Codex ready lines like Grok. |
| **F22 — Codex trust honesty** | Install success + dry-run print: `next: in Codex run /hooks and trust ai-brains-capture`. `wiring=ok` means files exist, **not** that the current Codex session has trusted the hash. |
| **F23 — Grok merge fail-open** | Claude wrapper / `claude-hook`: unrecognized stdin (missing `hook_event_name` / Claude fields, or Grok camelCase-only) → exit 0, no ingest, stderr once. Never panic, never block Grok stop. |
| **F24 — Capture independence** | Detect / install / payload map / message-only parse: no models, embeddings, graph. Hook CLIs may open the vault to append (same as `grok-hook`). |
| **F25 — Adapter honesty** | Keep `CapabilityLevel::Full` + `supports_hooks: true` (locked tests). **Rewrite notes** to: install via `harness install --harness claude\|codex`; live UPS+Stop; no SessionStart injection; no nightly; Codex `/hooks` trust; message-only. Replace naive `parse_claude_stop_payload` with T234-filtered helpers; add Codex parse helpers. |
| **F26 — Uninstall** | Remove only managed handler(s) named `ai-brains-capture` + our wrapper. Leave `{}` / empty `hooks` object. Foreign groups stay. Clear prefs `installed_at`. |
| **F27 — Atomic + reparse** | `atomic_write_str` + T190 reparse refuse (same as T235 F36). |
| **F28 — Secrets** | Never write keys/tokens into hook files. Never print `AI_BRAINS_KEY`. |
| **F29 — Zero new crates / no clap 5** | No pin bumps. |
| **F30 — Docs (hard)** | CAPABILITIES (table + §8 honesty); OPERATIONS activation + path table; WORKFLOWS “Activate” includes claude/codex; CHANGELOG T253; skill one-liner; CLI-EXIT-CODES only if new usage exits; research-doc banners. |
| **F31 — High findings if…** | Fake `install_ready` without writers; SessionStart injection; ingest tools/thinking; Codex `decision:block` / AGY allow JSON on Codex Stop; force-edit `config.toml`; write `codex_hooks`; repo-local hooks; nightly schema change; clap 5 / new crates; unwrap in production; Grok-invoked Claude wrapper blocks stop; print vault key. |
| **F32 — Parallel** | Touches `detect.rs`, `install.rs`, `wiring.rs`, `commands/harness.rs`, `doctor.rs` tests, `main.rs` clap, new hook/import command modules, adapters `claude.rs`/`codex.rs`/`message_only` helpers, docs, hermetics. T252 shipped — no ingest rewrite. T254/T255 do not overlap. |
| **F33 — Soft residuals** | See **F34**. |
| **F34 — Soft (not DoD)** | Nightly Claude/Codex sources + skip flags; Codex SessionEnd ingest (3s); Claude PreCompact; UserPromptSubmit-only vs transcript-only experiment after live dogfood; Unix `.sh` wrappers; plugin-packaged hooks; `commandWindows` dual command; fingerprint turn-ids if filter churn; HTTP/MCP hook types; is-terminal migrate. |

---

## 6. Functional requirements

| ID | Requirement |
|----|-------------|
| **R1** | `install_ready()` true for Claude and Codex. |
| **R2** | `harness install --harness claude --dry-run` prints settings path + wrapper + events + zero writes. |
| **R3** | Same for Codex (`hooks.json` + wrapper + `/hooks` trust note). |
| **R4** | Real install (temp home) merges without destroying foreign keys; re-run idempotent; probe `ok`. |
| **R5** | `all-ready` dry-run lists five plans after flip (or however many ready). |
| **R6** | `claude-hook` / `codex-hook` `--schema` prints 2020-12 schema; `--payload` ingests message-only. |
| **R7** | Empty/whitespace last message or prompt → skip that role, exit 0 (hook path). |
| **R8** | Mid-payload garbage on hook CLI → fail-open exit 0 on wrapper; CLI may exit 1 with JSON (document). Wrapper must still satisfy F8/F9 stdout. |
| **R9** | Import commands dry-run list sessions; `--force` skips quiescence (300s, same as Grok). |
| **R10** | Doctor/preflight pending T253 clause gone when both ready and present. |
| **R11** | Docs + CHANGELOG + capability notes. |
| **R12** | Hermetic tests never touch real `~\.claude` / `~\.codex`. |

---

## 7. Acceptance criteria

| AC | Criterion |
|----|-----------|
| **AC1** | Units: `HarnessId::Claude.install_ready()` and `Codex.install_ready()` are true; `pending_track()` is `None`. |
| **AC2** | Hermetic: `install_claude(home, true)` / `install_codex(home, true)` → `DryRun`, temp home file set unchanged. |
| **AC3** | Hermetic: real `install_claude` creates/merges `settings.json` with named `ai-brains-capture` on UPS+Stop+SessionEnd; foreign `PreToolUse` group preserved; second run idempotent; `probe_wiring` = `Ok`. |
| **AC4** | Hermetic: real `install_codex` writes `hooks.json` UPS+Stop named handler; **does not** create/modify `config.toml`; probe `Ok`. |
| **AC5** | Hermetic: uninstall removes only managed handlers + wrapper; foreign remain. |
| **AC6** | Wrapper body unit: Claude empty stdout + exit-0 contract string; Codex stdout == `{"continue":true}`; neither contains `decision`. |
| **AC7** | `claude_filter` / `codex_filter`: user+assistant kept; tool/thinking/system dropped; `thinking` None on ingest request. |
| **AC8** | Claude UPS `prompt` + Stop `last_assistant_message` map to hook payload; Grok-shaped stdin → skip (F23). |
| **AC9** | Codex UPS `prompt` + Stop `last_assistant_message` map; missing fields → skip. |
| **AC10** | `claude-import` hermetic fixture project JSONL → only user/assistant text; sidechain skipped. |
| **AC11** | `codex-import` hermetic rollout with `session_meta` + `event_msg` + `response_item` message → only message roles. |
| **AC12** | `harness install --harness all-ready` (hermetic PATH/home) includes claude + codex plans. |
| **AC13** | Doctor unit: ready-wired grok/agy/opencode/claude/codex → **no** `T253` / `backend pending` clause. |
| **AC14** | `harness status --format json` schema_version 1 unchanged; claude/codex `install_ready=true`. |
| **AC15** | help_ia Harness inventory includes `claude-hook`, `codex-hook`, `claude-import`, `codex-import`. |
| **AC16** | CAPABILITIES + OPERATIONS + WORKFLOWS + CHANGELOG updated; research docs bannered. |
| **AC17** | Capture-independence: new parse/filter unit tests do not link models/graph. |
| **AC18** | Corrupt `settings.json` / `hooks.json` install refuses; bytes unchanged. |
| **AC19** | C7: hermetic targets under temp home only. |
| **AC20** | Live dogfood **on go**: dry-run then `--yes` for claude and codex (or `all-ready`); `harness status` both `ok`; preflight no pending next; **zero** new files under `C:\dev\AI-Brains`. Record `/hooks` trust note. Optional: do **not** require a live Stop fire in DoD (F34). |

---

## 8. Non-goals

- Adding Claude/Codex to nightly multi-import (T239 D16).
- SessionStart / additionalContext preflight injection (legacy scripts).
- Project-local hooks; Codex `--dangerously-bypass-hook-trust`.
- Forcing `[features].hooks = true` in the user’s `config.toml`.
- Rewriting T234 core; changing CapabilityLevel.
- clap 5; pin bumps; new crates.
- Desktop / HTTP hook types / MCP hook types.
- T254 multi-root residuals; T255 nightly/router residuals.
- Replacing Grok’s merge of Claude settings (vendor behavior).

---

## 9. Risks

| Risk | Mitigation |
|------|------------|
| Codex Stop JSON contract drift | Pin `{"continue":true}`; hermetic; docs; fail-open |
| Codex `/hooks` trust forgotten | Install stdout + OPERATIONS recipe |
| Grok fires Claude wrapper | F23 skip |
| Transcript format drift (Codex batch) | Fail-open keep-list; honesty |
| SessionEnd 3s budget | Not primary path |
| Over-wide `settings.json` merge | Map-only insert of our matcher groups; refuse parse fail |
| Capability Full overclaim remains | Notes + CAPABILITIES caveats |
| Live Stop not dogfooded | AC20 files+status; live fire soft F34 |

---

## 10. Verification

- Red: AC1–AC13 failing tests first (TDD).
- Green: writers + parsers + clap.
- Targeted: `cargo nextest run -p ai-brains-cli -p ai-brains-adapters` + clippy those crates.
- Manual on go: AC20.
- Docs grep: `T239+` pending labels for claude/codex in product docs should become T253-completed language.
- Full gate before finalize.
- Cross-model: FEATURE/high-risk — run when implement is otherwise clean (`codex-review`).

---

## 11. Docs / contracts

| Surface | Change |
|---------|--------|
| `Docs/CAPABILITIES.md` | Claude/Codex row: install_ready via T253; caveats (no nightly, `/hooks`, no injection) |
| `Docs/OPERATIONS.md` | Path table + activation includes claude/codex; feature key `hooks` not `codex_hooks` |
| `Docs/WORKFLOWS.md` | Activate recipe: `all-ready` now five; `/hooks` step |
| `CHANGELOG.md` | T253 |
| `Docs/schemas/claude-hook-payload.json` | New |
| `Docs/schemas/codex-hook-payload.json` | New |
| Research docs | Stale banner |
| `ai-brains-contracts` | **No** DTO change unless a hook schema is considered contracts — file JSON schemas only |
| CLI-EXIT-CODES | Only if new usage-class exits appear (unknown harness already 2) |

---

## 12. AI fold-in disposition

None yet. If `C:\dev\AI-review.md` is provided for T253, fold agreed items here (same protocol as T245/T252). Until then this section is a placeholder.

---

**Plan-only until go.**
