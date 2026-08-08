# T235 — Harness detect + preflight hook install UX

- **Status:** 📋 **Planning** (plan-only until **go**; **AI fold-in 2026-08-08**)
- **Source:** Research 2026-08-08 series; plan research **2026-08-08** (live probe + official hook docs); AI1 + AI2 review fold-in
- **Category:** FEATURE / UX
- **Depends on:** T234 Completed (message-only contract + capability honesty); existing `agy-hook` for AGY install backend
- **Blocks / pairs with:** T236–T238 install backends (writers may stub until those land); T239 doctor/nightly multi-harness summary
- **Related:** T214 preflight summary; T192 doctor; T204 help_ia; T190 reparse; T205 hermetic home; series [README-T234-T239](../README-T234-T239-HARNESS-INGEST.md)
- **Absorbs:** deferred “Detect + preflight hook install UX”; series **C6 Consent** + **C7 No repo pollution** (install UX slice); AI2 **M1–M6** + **L1–L7**; AI1 JSON-merge / PS quoting / labeling / uninstall polish
- **Does not absorb:** Live Grok/OpenCode capture handlers (T237/T238); full AGY `history.jsonl` project binding (T236 — wrapper maps workspace path as alias only); nightly multi-import (T239); T220 preflight summary JSON growth; display ASSISTANT: strip (T224); `fullyIdle` re-queue policy (T236 soft)

## Objective

At session start (`preflight`, optionally `doctor`), and via a dedicated CLI:

1. **Detect** which coding harnesses are **installed on this machine** (not “active this session”).
2. **Report** whether AI-Brains capture wiring is `missing` | `partial` | `ok` | `unknown` | `backend_pending`.
3. **Offer** install on TTY (once; remember decline) or install with explicit consent flags.
4. **Install** only **message-only** hooks/plugins into **user-global** paths by default (C1–C3 via T234; C7).
5. Keep install **idempotent** and **non-destructive** to foreign hooks (namespaced managed ids).

## Live dogfood freeze (2026-08-08, this machine)

| Signal | Observation |
|--------|-------------|
| PATH | `grok`, `agy`, `opencode`, `claude`, `codex` present; `antigravity` binary name **missing** (agy is the CLI) |
| Homes | `~/.grok`, `~/.gemini` (+ `config`, `antigravity-cli`), `~/.config/opencode`, `~/.claude`, `~/.codex`, `~/.ai-brains` all exist |
| Grok hooks dir | **no** `~/.grok/hooks` yet |
| AGY hooks | **no** `~/.gemini/config/hooks.json` and **no** `~/.gemini/antigravity-cli/hooks.json` |
| Claude | `~/.claude/settings.json` exists |
| OpenCode plugins | **no** `~/.config/opencode/plugins` dir |
| AI-Brains wiring | none installed |

## Research freeze (2026-08-08)

| Harness | Official install surface | Primary capture event | User-global path (default) | Notes |
|---------|--------------------------|----------------------|----------------------------|-------|
| **Grok Build** | `~/.grok/hooks/*.json` (+ Claude/Cursor compat) | `UserPromptSubmit`, `Stop` (+ Session*) | `%USERPROFILE%\.grok\hooks\ai-brains.json` | stdin JSON + `GROK_*` env; project hooks need trust — **avoid project default** |
| **AGY / Antigravity** | Named hook entries in `hooks.json` | **`Stop`** (`fullyIdle`); optional PostInvocation | Prefer merge into **`~/.gemini/config/hooks.json`**; also probe `~/.gemini/antigravity-cli/hooks.json` | Schema: top-level **named** hooks → events → handlers; stdin camelCase; `transcriptPath`, `workspacePaths`, `conversationId` |
| **OpenCode** | Global plugins auto-load | `session.idle` (plugin event) | `%USERPROFILE%\.config\opencode\plugins\ai-brains-capture.js` (or `.ts`) | Also `~/.config/opencode/opencode.json` plugin list for npm; local plugins load without npm |
| **Claude Code** | `hooks` in settings JSON | `UserPromptSubmit`, `Stop`, … | `%USERPROFILE%\.claude\settings.json` | Merge additively; three scopes user/project/local |
| **Codex** | `hooks.json` / features.hooks | Claude-like lifecycle | Under `~/.codex` (probe at implement; trust UX) | Feature flag may be required; soft if schema drifts |

### Dependency pins (no bump required)

| Crate | Workspace pin | Lock | Role |
|-------|---------------|------|------|
| clap | 4.5 | 4.6.1 | `harness` subcommand |
| serde_json | 1.0 | 1.0.150 | status JSON + hooks merge |
| dirs | 6.0 | 6.0.0 | home (with USERPROFILE/HOME override parity T205) |
| is-terminal | 0.4 | 0.4.17 | TTY prompt gate |

**No new crates.** PATH presence: walk `PATH` / try resolve without adding `which`. Prompt: existing `stdin().read_line` pattern (backup/forget/device).

### Capture independence

Detect + status + dry-run install **must not** open models, embeddings, or graph. Prefer pure fs/env probes. Preflight harness section must not fail preflight if vault graph is off.

## Frozen direction

| ID | Decision |
|----|----------|
| **F1 Detect set** | Harnesses: `grok`, `agy`, `opencode`, `claude`, `codex`. Map display name “Antigravity 2 / AGY” → id `agy`. |
| **F2 Presence signals** | **Installed-on-machine** if **any** of: (a) PATH resolve of primary binary, (b) known home/config dir exists. Never claim “this session is running harness X” unless a future soft env/parent marker is documented as soft-only. |
| **F3 Binary names** | `grok`; `agy` (not `antigravity`); `opencode`; `claude`; `codex`. Extra soft: `GEMINI_*` / `GROK_*` / `OPENCODE_*` env as **secondary** presence only (document FP risk). |
| **F4 Home roots (Windows)** | Resolve home via **USERPROFILE then HOME** (T205 parity), then: `\.grok`, `\.gemini\antigravity-cli` (+ `\.gemini\config`), `\.config\opencode`, `\.claude`, `\.codex`. |
| **F5 Wiring status enum** | Per harness: `absent` (not installed on machine) \| `missing` (present, no AI-Brains wiring) \| `partial` (some files/markers) \| `ok` (managed marker + expected paths) \| `backend_pending` (present + install requested but capture backend not ready) \| `unknown` (unreadable config / parse fail — fail soft). |
| **F6 Managed marker** | AI-Brains-owned entries use id **`ai-brains-capture`**. **Grok:** managed **file path** SOOT is `~/.grok/hooks/ai-brains.json` (docs schema has no top-level `name` — do **not** require a `"name"` field). **AGY:** top-level key `ai-brains-capture` in hooks.json. **OpenCode:** local plugin file `~/.config/opencode/plugins/ai-brains-capture.js` (or `.ts`) is the marker; `opencode.json` `plugin` array is **npm-only** (optional later). **Claude:** managed command/path under `~/.ai-brains/hooks/` or token in settings. Never delete foreign hooks. |
| **F7 Wiring probes** | **Grok:** file `hooks/ai-brains.json` exists (path marker). **AGY:** hooks.json (config **or** antigravity-cli) has named key `ai-brains-capture`. **OpenCode:** plugin **file** under global plugins dir. **Claude:** settings hooks mention managed path/token. **Codex:** managed entry when schema known; else `unknown`. |
| **F8 Preflight Harness section** | On `preflight --summary`: after T214 lines, append Harness block **only if ≥1 harness is not `absent`**. Implement as **sibling pure fn** `format_harness_summary_lines(&[HarnessStatus]) -> Vec<String>` — **do not** grow `format_preflight_summary_lines` arity (T214 test-locked). Label header: **`Harnesses installed on machine:`** (never “Active harness”). Next-action: exact copy-paste e.g. `ai-brains harness install --harness agy --dry-run`. **No** `PreflightContextResponse` JSON key growth. |
| **F9 Non-TTY / CI** | If stdout is **not** a TTY: never prompt; print status + next-action. Exit 0 for preflight regardless of missing hooks. |
| **F10 TTY consent** | If TTY **and** any harness is `missing`/`partial` **and** not declined **and** backend ready **and** not blocked by F24 never-prompt gates: prompt once `Install capture hooks for <list>? [Y/n]`. Empty/`Y`/`y` → install ready backends; `n`/`N` → persist decline. |
| **F11 Consent persistence** | User-global **`%USERPROFILE%\.ai-brains\harness_hooks.json`**. Fields: `schema_version`, `auto_install` (default false), per-harness `{ declined_at?, installed_at?, install_version?, last_status? }`. Decline suppresses re-prompt until `harness install` or `harness reset-decline`. **Uninstall** clears that harness’s `installed_at` / `last_status` (and may set last_status=`uninstalled`) so status re-prompts honestly. |
| **F12 Install scope default** | **User-global only** (C7). `--scope project` deferred; if mentioned in help/output, note **Grok project hooks require `/hooks-trust`** (`trusted_folders.toml`) so later project install is not a surprise. |
| **F13 CLI surface** | `harness status [--format human\|json]`; `install [--harness <id>\|all] [--yes] [--dry-run]`; `uninstall [--harness …] [--yes] [--dry-run]`; **`reset-decline`** (DoD, not optional). Uninstall removes **only** managed markers/files + managed wrapper scripts. |
| **F14 Install readiness** | Real wiring writes only when ready: **`agy` = ready** (Stop → wrapper → `agy-hook` with **F34 mapping**). **`grok` / `opencode` / `claude` / `codex` = backend_pending** — dry-run ok; real install **must not** claim capture works. `install --harness all` when all pending: exit **0** + **one-line summary** listing pending track ids (not silent success). |
| **F15 AGY install writer** | Merge into `~/.gemini/config/hooks.json` (create if missing) key **`ai-brains-capture`** with **`Stop`** handler only (additive `serde_json::Map` — never replace whole file). Command in hooks.json must be **quoted-safe**: e.g. `powershell.exe -NoProfile -ExecutionPolicy Bypass -File "<abs-path>\.ai-brains\hooks\agy-stop.ps1"` (or equivalent with absolute path, never unquoted spaces). Wrapper path SOOT: `%USERPROFILE%\.ai-brains\hooks\agy-stop.ps1` (Unix soft: shell counterpart). Preserve all other named hooks. Idempotent re-run. Empty remaining object after uninstall → leave `{}` (valid JSON), do not error. |
| **F16 Dry-run** | `--dry-run`: print targets + planned snippets; **zero writes** (no hooks.json, no wrapper, no prefs mutation). |
| **F17 Doctor soft check** | Always soft check id `harness_wiring` severity **ok** or **info** (never fail/degraded solely for missing hooks). **Update** `health_check_order_names__fixed_matrix` and related doctor matrix tests from **11 → 12** checks (same class as T213 `graph_density`). |
| **F18 help_ia** | Add `harness` to **Harness** group inventory string. |
| **F19 CAPABILITIES / OPERATIONS** | Document detect + consent; message-only; AGY ready vs others pending; path table. **Also** flip `antigravity_capability()`: `supports_hooks: true` (or honest Partial notes: “hooks installable via `harness install --harness agy`”) so CAPABILITIES adapter surface matches F14/F15 — not “No real-time hooks” after install path ships. |
| **F20 Exit codes** | status: **0**. install success: **0**. unknown harness: **2** `fail_usage`. all-pending install: **0** + explicit pending summary (F14). File/permission/parse-refuse: **1** + path + manual instructions. |
| **F21 JSON status contract** | `{ "schema_version": 1, "home": "...", "harnesses": [ … ] }` with fields per row: `id`, `display_name`, `present`, `binary`, `home_path`, `wiring`, `install_ready`, `targets`, `next_action`. **Iteration order fixed** to F1: `[grok, agy, opencode, claude, codex]` — never HashMap order. |
| **F22 Pure core** | detect / wiring / prefs / AGY payload map pure + hermetic. No `unwrap`/`expect` in production. |
| **F23 Windows + UTF-8 paths** | USERPROFILE/HOME; skip non-UTF8 PATH entries; native display paths. |
| **F24 Never-prompt gates** | Never prompt when **any** of: non-TTY stdout; `--no-hook-prompt`; **`preflight --stdin`** (stdin is options payload — conflicts with consent `read_line`); CI-style non-interactive. Explicit `--install-hooks` / install `--yes` may install ready backends without prompt. Default: no auto-install without TTY yes, flag, or `auto_install: true`. |
| **F25 auto_install** | `auto_install: true` → non-interactive may install **ready** backends only. Default false. |
| **F26 Secrets** | Never write keys/tokens into hook files. |
| **F27 Capture Privacy banner** | Message-only one-liner (T234). |
| **F28 Uninstall + parse-fail safety** | Remove only managed key/file + managed wrapper. Parse error on hooks.json / settings → **refuse rewrite** (exit 1, path). Same for **prefs**: corrupt `harness_hooks.json` → treat as empty/unknown, **never** destructive rewrite. |
| **F29 Parallel tracks** | T236–T238 flip readiness; T235 table one-file flip. |
| **F30 False-positive labeling** | Human: **`Harnesses installed on machine:`** — never “active session” / “Active harness”. |
| **F31 Module layout** | `commands/harness.rs` + pure `harness/{detect,wiring,prefs,agy_map}.rs` (or under commands/). Zero new crates. |
| **F32 Tests hermetic** | Temp **USERPROFILE+HOME** **and** scrub/inject **PATH** via `TempEnv` (T205) so host `grok`/`claude` on PATH cannot spoil `present=false` fixtures. Never touch real user harness homes. |
| **F33 Manual smoke** | Real machine status / preflight Harness / AGY dry-run; optional real install with consent. |
| **F34 AGY Stop → agy-hook payload map (AI2 M1 hard)** | Wrapper **must** build `agy-hook --payload` JSON matching `Docs/schemas/agy-hook-payload.json` (`additionalProperties: false`, required `transcriptPath`, `sessionId`, `projectHash`): |
| | • `transcriptPath` ← Stop `transcriptPath` (required; if missing, exit 0 soft skip + stderr once) |
| | • `sessionId` ← Stop `conversationId` (must be UUID parseable; else soft skip) |
| | • `projectHash` ← first non-empty `workspacePaths[]` entry as path string (alias key for `resolve_project_id_from_alias` / `ensure_project_alias`); if none → literal `"agy-unbound"` (still valid required field; T236 deepens binding) |
| | Pure fn `map_agy_stop_to_hook_payload(stop: &Value) -> Result<AgyHookPayload, MapSkip>` unit-tested; wrapper only shells `ai-brains agy-hook --payload <json>`. **Do not** widen schema in T235 unless map cannot work (prefer map). |
| **F35 AGY fullyIdle soft** | If Stop has `fullyIdle: false`, wrapper **soft-skips** ingest (exit 0) to avoid mid-task noise; idempotent delta in agy-hook remains safety net. Hard re-queue / policy → T236. |
| **F36 Atomic + reparse writes (AI2 M6)** | All writes to `hooks.json`, prefs, wrappers: **temp file + rename**; refuse reparse/symlink targets via existing `ai_brains_path` reparse helpers (T190 parity) before write. |
| **F37 JSON merge SOOT** | Deserialize hooks.json to `Map<String, Value>`; only insert/update/remove `ai-brains-capture`; syntax error → refuse (F28). |
| **F38 vault-path-free** | Add all `harness` subcommands to `is_vault_path_free` in `main.rs` so status/install work **before** vault init (detect is fs-only). |
| **F39 PATH resolve cost** | Per binary: first successful resolve on PATH; skip `PermissionDenied` entries; no shell-out-per-entry; no network. |
| **F40 Install/uninstall next-action** | On missing/pending/success, print exact next command (clig.dev): e.g. `harness install --harness agy --dry-run` / `harness status`. |
| **F41 reset-decline DoD** | `harness reset-decline [--harness …\|all]` clears declined_at so preflight may offer again. |
| **F42 AI fold-in disposition** | See **§14**. |

## Acceptance criteria

| AC | Criterion |
|----|-----------|
| **AC1** | Fixture home with only `.grok` → status `present=true` for grok, wiring `missing` (or `backend_pending` after install attempt policy). |
| **AC2** | Fixture with managed Grok hook file → wiring `ok` **or** `backend_pending` if capture not ready but file present — freeze: **if managed marker present → at least `partial`/`ok`, never `missing`**. |
| **AC3** | Fixture AGY hooks.json without managed entry → `missing`; after install writer (or simulated) → managed entry present and other hooks preserved. |
| **AC4** | `harness install --dry-run --harness agy` prints config path + wrapper path; temp home unchanged. |
| **AC5** | Non-TTY preflight summary with missing wiring: no prompt; includes next-action command; exit 0. |
| **AC6** | TTY + missing + no decline: prompt path unit-tested with injectable “stdin” or extracted ask fn; decline persists and second run no prompt. |
| **AC7** | `harness status --format json` matches F21 schema_version 1 keys (serde round-trip test). |
| **AC8** | Unknown `--harness foo` → exit 2 usage. |
| **AC9** | Doctor includes soft harness check; missing hooks do not force Fail/Degraded alone. |
| **AC10** | help_ia Harness inventory includes `harness`. |
| **AC11** | CAPABILITIES + OPERATIONS updated. |
| **AC12** | Install never writes under repo project tree by default (assert targets under home). |
| **AC13** | AGY real install in temp home: Stop handler references wrapper; re-run idempotent; foreign keys preserved. |
| **AC14** | `grok`/`opencode` install without readiness: dry-run ok; real install no fake “ok” wiring claim. |
| **AC15** | Capture independence: detect/wiring pure tests do not link models/graph. |
| **AC16** | **Payload map (M1):** Stop fixture `{ conversationId, transcriptPath, workspacePaths, fullyIdle: true }` → payload with `sessionId=conversationId`, `transcriptPath` equal, `projectHash=workspacePaths[0]`; empty workspacePaths → `projectHash="agy-unbound"`. |
| **AC17** | Hermetic detect with **PATH scrub**: host has `grok` on PATH but test PATH empty + only fixture home → `present` follows fixture only. |
| **AC18** | `preflight --stdin` never prompts (even if stdout is TTY in theory). |
| **AC19** | `format_preflight_summary_lines` signature unchanged; harness lines only from sibling formatter. |
| **AC20** | `harness status` succeeds with no vault / no init (vault-path-free). |
| **AC21** | Corrupt hooks.json install refuses rewrite (exit 1); file bytes unchanged. |
| **AC22** | Doctor matrix includes `harness_wiring` as 12th ordered check. |
| **AC23** | Uninstall clears prefs `installed_at` for that harness; managed key + wrapper removed; foreign keys remain. |

## Non-goals

- Implementing Grok/OpenCode/Claude/Codex **capture** parsers beyond status (T237/T238 / existing Claude partial).
- Full AGY `history.jsonl` project binding (T236); T235 only maps `workspacePaths[0]` → `projectHash` alias string.
- Nightly multi-harness orchestration (T239).
- Growing preflight JSON DTO with harness array (T220 territory if ever desired).
- Auto-trust of project hooks; MDM/enterprise managed hooks.
- Widening `agy-hook-payload.json` schema (prefer F34 map).
- GUI installers; MSI PATH registration.
- Competing with claude-mem beyond coexistence (never wipe foreign hooks).

## Risks

| Risk | Mitigation |
|------|------------|
| Leftover home dirs → false “using harness” | F2/F30 wording |
| AGY dual hooks.json locations | Probe both; write SOOT to `~/.gemini/config/hooks.json`; document secondary |
| Windows command quoting / spaces in USERPROFILE | F15 PowerShell `-File` absolute path |
| Foreign hooks clobber | F37 Map-only merge; F28 refuse on parse error |
| Preflight latency | F39 first-hit PATH resolve |
| Consent prompt vs `--stdin` | F24 never-prompt includes `--stdin` |
| Host PATH breaks hermetic tests | F32 PATH scrub |
| AGY payload field mismatch | **F34 hard** + AC16 |
| Crash mid-write corrupts hooks | F36 tempfile+rename + reparse refuse |
| Install claims capture works for Grok early | F14 readiness gate + pending summary |
| Capability lie `supports_hooks: false` | F19 flip antigravity capability notes |

## Implement order (when go)

1. Pure detect + wiring + **PATH-scrub hermetic** + JSON status (F21 order).  
2. prefs load/save + decline + corrupt-prefs refuse.  
3. **F34 payload map** pure + tests (AC16) before wrapper ship.  
4. CLI harness + **vault-path-free** + dry-run + AGY writer/wrapper (F15/F36/F37).  
5. Preflight sibling formatter + never-prompt gates (F8/F24).  
6. Doctor matrix 12th check + help_ia + capability flip + docs.  
7. Manual smoke + full gate.

## Series linkage

| Track | After T235 |
|-------|------------|
| T236 | Deepen workspace→project binding; fullyIdle policy; may refine projectHash |
| T237 | Grok `install_ready` + capture; mention project-hook trust |
| T238 | OpenCode plugin body; file marker already defined |
| T239 | Reuse detect module for multi-harness status |

---

## §14 — AI review fold-in disposition (2026-08-08)

### AI1 (architecture affirm + safety polish)

| Item | Verdict | Disposition |
|------|---------|-------------|
| Architecture diagram / F1–F33 affirm | Agree | Affirmed; no change to scope |
| Robust JSON merge / parse-fail refuse | Agree | **Elevate** F28 + **F37** |
| PowerShell `-File` quoting for spaces | Agree | **Elevate** F15 |
| “On machine” labeling | Agree | **Elevate** F8/F30 header string |
| Clean uninstall (managed only; leave `{}`) | Agree | **Elevate** F15/F28 + **AC23** |
| Module/test file map | Agree | Plan phases + F31 |
| AC1–AC15 verification matrix | Agree | Affirmed; extended AC16–AC23 |

### AI2 (M1–M6 / L1–L7)

| ID | Sev | Verdict | Disposition |
|----|-----|---------|-------------|
| **M1** | high | **Agree** — Stop ≠ agy-hook fields (`sessionId`/`projectHash` missing) | **F34 hard** + **AC16**; pure map + wrapper; do not ship “ready” without this |
| **M2** | med | Agree — PATH pollutes hermetic tests | **F32** PATH scrub + **AC17** |
| **M3** | med | Agree — `preflight --stdin` vs consent stdin | **F24** + **AC18** |
| **M4** | med | Agree — do not grow T214 formatter arity | **F8** sibling fn + **AC19** |
| **M5** | med | Agree — vault gate blocks pre-init status | **F38** + **AC20** |
| **M6** | med | Agree — atomic write + reparse | **F36** + **AC21** |
| L1 | low | Agree — fixed harness order | **F21** |
| L2 | low | Agree — Grok path marker not `name` field | **F6/F7** |
| L3 | low | Agree — OpenCode local plugin file is marker | **F6/F7** |
| L4 | low | Agree — Grok project trust note | **F12** |
| L5 | low | Agree — corrupt prefs refuse | **F28** |
| L6 | low | Agree — doctor matrix 11→12 | **F17** + **AC22** |
| L7 | low | Agree — all-pending install summary | **F14** |
| Blind: capability `supports_hooks` | Agree | **F19** flip antigravity capability honesty |
| Blind: fullyIdle | Partial | **F35** soft skip; deep policy → T236 |
| Blind: PATH cost | Agree | **F39** |
| Blind: uninstall prefs clear | Agree | **F11** + **AC23** |
| Dep pins no bump | Agree | Affirmed (lock clap 4.6.x; no intentional bump) |

### Declined / out of scope

| Item | Why |
|------|-----|
| Widen agy-hook schema in T235 | Prefer F34 map; schema change only if map impossible |
| Full history.jsonl binding | T236 |
| Project-scope install DoD | Deferred; F12 note only |
| Preflight JSON harness array | T180/T214 freeze; T220+ |
