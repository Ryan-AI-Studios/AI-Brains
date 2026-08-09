# T238 — OpenCode seamless ingest

- **Status:** 📋 **Planning** (plan-only until go; no production code / ledger TX)
- **Source:** Research 2026-08-08 — live OpenCode **1.18.15**, official [plugins](https://opencode.ai/docs/plugins/) + [CLI](https://opencode.ai/docs/cli/) + [SDK](https://opencode.ai/docs/sdk/) docs, live `opencode export` / `session list --format json`, T234 `filter_opencode_*` fixtures, T235 wiring marker, T236/T237 dual-path lessons; crates.io pins under caret (**no intentional dep bump**); `Docs/Opencode-Hooks-Research.md` **stale** (2026-05 config shape wrong — do not implement from it); **AI review fold-in 2026-08-09** (§14)
- **Category:** FEATURE
- **Depends on:** T234 (message-only SOOT ✅); T235 (detect/wiring/`harness *` ✅; OpenCode marker + `backend_pending`); T236/T237 lessons (unbound anti-hijack, path-keyed meta, turn-id SOOT, install honesty, capture independence, **subagent skip / synthetic chrome**)
- **Absorbs:** deferred “OpenCode plugin + export batch”; T234 “wire T238” for `filter_opencode_*`; T235 OpenCode install backend; live export-shape gap vs flat fixture; **AI2 M1–M6** (child-session idle skip; synthetic/ignored text drop; idle-deprecation honesty; list cap-100; live SDK messages; full part-type set); **AI2 M7–M9** (compaction-skip key; OPENCODE_CONFIG_DIR caveat; worktree-prefer bind); **AI1** affirms (export timeout, foreign plugins, T239+ labels) — already in baseline
- **Related:** series [README-T234-T239](../README-T234-T239-HARNESS-INGEST.md); T239 nightly multi-harness consumer of `opencode-import`
- **Does not absorb:** multi-harness nightly orchestration / SYSTEM `--skip-import` re-enable (T239); Claude/Codex `install_ready` (remain pending — **labels → `T239+` / follow-up**, not “T238”); T224 display strip; T220 preflight JSON growth; project-scope plugins as default (C7); raw `opencode.db` / multi-GB SQLite as primary API; session.created preflight inject / context injection; npm-published plugin package

## Objective

Make **OpenCode** capture **seamless and bound to the right project**:

1. **Live:** user-global plugin (`session.idle`, with **child-session skip**) → thin call → `opencode-hook` ingests **message-only** turns (prefer **SDK messages** for live; CLI export for batch) with worktree/directory→project binding.
2. **Batch:** `opencode-import` runs `opencode session list --format json` + `opencode export <id>` for sessions updated since watermark — **never** open `~/.local/share/opencode/opencode.db` as content SOOT.
3. **Install honesty:** `harness install --harness opencode` writes the managed plugin (no `backend_pending` stamp); wiring probe `ok` when marker present; `install_ready() → true`.
4. **Docs:** CAPABILITIES / OPERATIONS / CHANGELOG / series README; correct or supersede stale `Docs/Opencode-Hooks-Research.md` claims.

## Diagnosis (frozen — 2026-08-08 live + docs + AI2 source verification 2026-08-09)

| ID | Fact |
|----|------|
| **D1** | **CLI version live:** OpenCode **1.18.15**. Config dir: `%USERPROFILE%\.config\opencode` (`opencode.jsonc`, `package.json` with `@opencode-ai/plugin` **1.18.3**, no `plugins/` file yet). npm package latest can track CLI (1.18.15). Data dir: `%USERPROFILE%\.local\share\opencode\` includes **`opencode.db` (multi-GB risk)** + tool-output blobs — **not** content SOOT. |
| **D2** | **Plugin load (official):** local JS/TS under `~/.config/opencode/plugins/` (global) or `.opencode/plugins/` (project) are **auto-loaded at startup**. npm packages via `plugin` array in config. Load order: global config → project config → global plugins dir → project plugins dir. **Do not require** inventing a `plugins: { ai-brains: {enabled} }` object (stale research doc). Zero-deps `.js` ESM works without `@opencode-ai/plugin` import. |
| **D3** | **Primary live event:** `session.idle` (docs still list it; fires when session becomes idle). Plugin context: `{ project, client, $, directory, worktree, … }`. Fail-open on plugin errors. |
| **D4** | **Session list JSON** (`opencode session list --format json -n N`): objects with `id` (`ses_…`), `title`, `updated` / `created` (**epoch ms**), **`projectId`** (camelCase — not `projectID`), **`directory`**. Default **max-count 100** (no pagination). Batch list passes `roots: true` → **children excluded from list** (live has no such filter — see D19). |
| **D5** | **Export JSON** (`opencode export <sessionID>`): top-level **`{ info, messages }`**. **`info`:** `id`, `slug`, `projectID`, **`directory`**, `title`, `agent`, `model`, `time.{created,updated}`, … **`messages[]`:** each `{ info, parts }` — **not** flat `{role, content}`. Export is own-session messages (children not embedded). |
| **D6** | **Message `info`:** `role` (`user` \| `assistant`), stable **`id`** (`msg_…`), `sessionID`, `time.created` (ms), optional `time.completed`, optional `agent` on **user**; assistant often has **`mode`** (not always `agent`), `finish` (`stop` \| `tool-calls`), path/cwd on assistant. |
| **D7** | **Part types (SDK union):** `text` \| `subtask` \| `reasoning` \| `file` \| `tool` \| `step-start` \| `step-finish` \| `snapshot` \| `patch` \| `agent` \| `retry` \| `compaction`. Live histogram: text keep; **tool** (not only `tool_use`), reasoning, steps, compaction drop. **file** may carry `source.text` — explicit drop required. |
| **D8** | **T234 fixture gap (blocking):** `filter_opencode_message` expects top-level `role`/`type` + `parts`/`content`. Live export nests role under **`info.role`**. Flat fixture remains valid for unit tests; production path **must unwrap** export messages first. |
| **D9** | **User chrome:** sample real prompts are **bare text** (no `<user_query>` wrap). **Unlike Grok F11**, bare non-empty **non-synthetic** user text **is kept** after `extract_user_text`. |
| **D10** | **Export cost:** one long session ~**18 MB** / 1064 messages; smaller ~5 MB / 247 msgs. Batch **must** watermark + timeout; full re-export every night is unacceptable. Live re-export-per-idle is O(n) cost — prefer SDK messages (D22). |
| **D11** | **T235 today:** `probe_opencode` = file `~/.config/opencode/plugins/ai-brains-capture.js` **or** `.ts` → `ok`; real install → `install_pending` stamps `backend_pending`; `install_ready()` **false**; `pending_track=T238`. |
| **D12** | **T236/T237 lessons:** (a) path normalize before alias; (b) env project id **only** for unbound; (c) path-keyed `source_meta`; (d) unified turn-id live+batch; (e) `--force` / dry-run; (f) never populate `thinking`; (g) scheduled SYSTEM may still `--skip-import` — honesty only (T239); (h) **subagent skip** and **synthetic chrome drop** are hard (T237 M2/M3 analogs). |
| **D13** | **Dep pins:** workspace `serde`/`serde_json` 1.0, `clap` 4.5, `uuid` 1.13, edition 2024. crates.io latest under caret (serde 1.0.229, serde_json 1.0.151, clap 4.6.x, uuid 1.24) — **do not bump** for T238 unless gate forces. No new crates for plugin install. |
| **D14** | **Stale research:** `Docs/Opencode-Hooks-Research.md` (2026-05) proposes wrong config registration shape and PS adapter path; **supersede** with OPERATIONS + this track — do not implement Stage 2 as written. |
| **D15** | **Vendor flags:** `opencode --pure` / `OPENCODE_DISABLE_DEFAULT_PLUGINS` can disable plugins — document; capture independence via **batch** still works if binary present. |
| **D16** | **Claude/Codex** still pending — **out of T238 implement scope**; after OpenCode ships, pending labels must **not** read “T238” (use **T239+** / follow-up). |
| **D17** | **Message ids stable** (`msg_*`) — better turn-id substrate than Grok’s index-only history (prefer id-based keys). |
| **D18** | **Explicit non-goal:** `opencode db` / raw SQLCipher/SQLite on `opencode.db` as primary ingest API (schema drift + size). |
| **D19** | **Child/subagent sessions (AI2 M1):** `session.idle` payload is **`{ sessionID }` only**. Task tool creates child sessions with **`parentID`**. Plugin event bus delivers idle for **every** session (no roots filter). Batch list is roots-only by accident — **live path must skip** when `client.session.get` shows `parentID` (or equivalent). |
| **D20** | **Synthetic text parts (AI2 M2):** `TextPart` carries **`synthetic?: boolean`**, **`ignored?: boolean`**. Server injects synthetic user text: tool echo (“Called the Read tool…”), “executed by the user”, MCP resource text, compaction continue (`metadata.compaction_continue`), plan-approval, reminders, subagent result injection, editor_context `<system-reminder>…`. **F2 “all non-empty text” would leak** (T237 M2 class). |
| **D21** | **`session.idle` deprecation (AI2 M3):** server schema marks Idle **deprecated** but still publishes; docs still list it. Future removal → live dies silently; **batch watermark is completeness backstop**. Likely successor: `session.status` with `status.type === "idle"`. |
| **D22** | **Live SDK messages (AI2 M5):** plugin `client.session.messages` returns `{ info, parts }[]` — same shape as export messages — **without** cold-start CLI export (18MB+). Event hook is fire-and-forget → concurrent idles possible. |
| **D23** | **List cap (AI2 M4):** default `-n` / max-count **100**, no pagination; field is **`projectId`**. Bind on **`directory`** (authoritative for project). |
| **D24** | **Config relocate (AI2 M8):** `OPENCODE_CONFIG_DIR` / `OPENCODE_CONFIG` / XDG can move config; hardcoded `~/.config/opencode` may miss load + false wiring. |
| **D25** | **Worktree bind (AI2 M9):** plugin context exposes **`worktree`** (git root) and **`directory`**; prefer worktree for alias stability (parity with Grok `git_root_dir` > cwd). |
| **D26** | **Compaction skip key (AI2 M7):** AssistantMessage uses **`mode`**, not always `agent`; compaction continuations are often **synthetic user** parts (`metadata.compaction_continue`) — skip-by-`info.agent=="compaction"` is wrong. |

## Frozen direction

### A. Message-only filter (export-aware)

| ID | Decision |
|----|----------|
| **F1** | **SOOT remains** `ai_brains_adapters::message_only` + OpenCode helpers. Add **`normalize_opencode_export_message(&Value) -> Option<Value>`** (or equivalent) that maps live `{info, parts}` → filter-ready record: top-level `role` from `info.role`, `parts` as-is, optional `id` / `timestamp` from `info`. Keep accepting flat fixture shape (role at top). |
| **F2** | **Keep (AI2 M2 hard):** `user` + `assistant` with non-empty **non-synthetic, non-ignored text parts only**. User: `extract_user_text` on joined kept text (bare text OK — D9). Assistant: join kept `type=="text"` parts only. |
| **F3** | **Drop hard (AI2 M2 + M6):** roles system/tool/function/reasoning/thinking; **text parts** where `synthetic === true` **or** `ignored === true` **or** `metadata.kind === "editor_context"`; part types **`tool`**, `tool_use`, `tool_call`, `tool_result`, `reasoning`, `thinking`, `redacted_thinking`, `step-start`, `step-finish`, `compaction`, **`snapshot`**, **`patch`**, **`agent`**, **`retry`**, **`subtask`**, **`file`**; sole-tool JSON (F15); empty after strip. Extend `is_tool_or_thinking_part_type` (or sibling helper) accordingly. Prefer **structured flags** over content heuristics for synthetic chrome. |
| **F4** | **Export document filter:** `filter_opencode_export(&Value) -> Vec<IngestableTurn>` walks `messages[]` via normalize + filter; fail-open per malformed message. Also accept JSONL of flat messages (existing fixture path). |
| **F5** | **source_ts:** prefer `info.time.created` (ms → RFC3339 or ISO string); else none → ingest time. Honesty in CAPABILITIES. |
| **F6** | Never populate `IngestRequest.thinking`. Capture independence: no models/embeddings/graph on path. |
| **F7** | Golden fixtures: (a) nested live-shaped export with text/reasoning/tool/step/**snapshot/patch/file/subtask** parts; (b) **synthetic chrome matrix** (Read-tool echo, executed-by-user, editor_context, compaction_continue) → zero user memories; real bare prompt kept. Keep flat `opencode_messages.jsonl` regression. |

### B. Live plugin path

| ID | Decision |
|----|----------|
| **F8** | **Managed install artifact:** write user-global **`~/.config/opencode/plugins/ai-brains-capture.js`** (T235 marker SOOT — prefer **`.js`** zero-deps ESM; `.ts` optional not required). Create `plugins/` dir if missing. **No** project `.opencode/plugins` by default (C7). **Do not** rewrite user `opencode.json(c)` for v1 (auto-load is enough). Prefer default home path; honor **`OPENCODE_CONFIG_DIR`** when set for install + probe (AI2 M8 — F40). |
| **F9** | **Event wired for DoD:** `session.idle` only (D21 deprecation honesty in CAPABILITIES). Plugin returns `{ event: async ({ event }) => { … } }`. Soft-skip non-idle. Optional future: also handle `session.status` idle without removing idle DoD. |
| **F10** | **Child/subagent skip (AI2 M1 hard):** on idle, resolve `sessionID` from `event.properties.sessionID` (tolerant). Call **`client.session.get`** (or equivalent); if session has **`parentID`** → **skip ingest**, increment `skipped_child_session`, never throw. Pass `parentID` (if any) into hook payload so Rust can re-check / log. Soft residual: opt-in child ingest (S11). |
| **F11** | **CLI:** `ai-brains opencode-hook` (`--payload` and/or flags). Schema `Docs/schemas/opencode-hook-payload.json` + `--schema`. Payload: `sessionId`, `directory` / `worktree` / `projectHash`, optional `parentId`, optional `exportPath` / `messagesPath` (tests + live). |
| **F12** | **Live content path (AI2 M5 hard — promote former S4):** prefer **`client.session.messages`** in the plugin → serialize export-shaped `{ info, messages }` to a temp file (or hook payload) → `opencode-hook` → F4. **Fallback:** CLI `opencode export <sessionId>` (timeout **120s**) if SDK fetch fails. **Batch** remains CLI export only (F19). Hermetic tests inject fixture path. |
| **F13** | **Turn ids:** **`v5(session, msg_id)`** when message `info.id` present; else fallback `v5(session, "turn-{i}")` on kept index. Same live+batch. |
| **F14** | **Delta:** skip already-ingested turn ids; idempotent re-fire OK. |
| **F15** | **In-flight guard (AI2 M5 medium→hard soft-floor):** plugin keeps per-session in-flight flag; concurrent idle for same session soft-skips while one run active. Full min-interval debounce remains soft **S1**. Idempotent msg-id turns remain the vault-level guarantee. |

### C. Batch import

| ID | Decision |
|----|----------|
| **F16** | **Command:** `ai-brains opencode-import [--days N] [--force] [--dry-run] [--max-sessions N]`. Requires `opencode` on PATH; if missing → **soft skip**, clear stderr status, exit 0 (or non-fatal code consistent with other importers). |
| **F17** | **Discovery (AI2 M4 hard):** `opencode session list --format json` with **`-n` / `--max-count` sized for window** (default at least cover `--days` volume; if returned length **== cap**, warn on stderr `list_capped` and optionally raise cap once). Deserializer **tolerant** of `projectId` / `projectID`. Filter by `updated` within `--days` (default e.g. 7) unless `--force` under `--max-sessions`. Roots-only list already excludes children — document; do not open children via list. |
| **F18** | **Watermark:** store cursor under user storage e.g. `~/.ai-brains/opencode-import-cursor.json` (session id → last `updated` ms and/or last kept msg id). Skip sessions with `updated <= cursor` unless `--force`. **Never** full re-export all history every run. |
| **F19** | **Per session (batch):** `opencode export <id>` with **120s** timeout; parse F4; bind project; append turns; update cursor only on success. |
| **F20** | **Binding (AI2 M9 hard):** prefer **`worktree`** (plugin / export path root if present) → **`info.directory`** / list `directory` → **normalize** → project alias. Missing → `opencode-unbound` / display “(unbound OpenCode)”. |
| **F21** | **Unbound anti-hijack:** env `AI_BRAINS_PROJECT_ID` only when unbound/empty. Default non-interactive `allow_default_project: false`. |
| **F22** | **source_meta:** path-keyed (`source_meta:opencode:{stable_session_or_path_key}`). |
| **F23** | **Stats (stderr honesty):** `found`, `exported`, `imported_turns`, `skipped_watermark`, `skipped_days`, `skipped_missing_binary`, `skipped_child_session`, `skipped_synthetic` (optional), `export_errors`, `unbound_project`, `bound_via_worktree`, `bound_via_directory`, `timed_out`, `list_capped`. |
| **F24** | **Never** open `opencode.db` / tool-output cache as content SOOT. |
| **F25** | Reuse T236 unsummarized OR-query (harness-agnostic) if new turns appended. |
| **F26** | Hermetic tests: mock list+export JSON fixtures (no real opencode required for unit/integration). Optional `__slow` live probe behind ignore if binary present. |

### D. Install / detect / wiring

| ID | Decision |
|----|----------|
| **F27** | `HarnessId::Opencode.install_ready() → true`; `pending_track` → `None`. |
| **F28** | `install_opencode`: write managed plugin `.js` only; **never** delete foreign plugins; **never** rewrite unrelated keys in `opencode.json(c)` for v1. Uninstall removes only managed plugin file; clear backend_pending prefs. |
| **F29** | Dry-run zero writes; idempotent reinstall; overwrite only if our marker header present (`// AI-Brains managed (T238)`); corrupt/foreign same-name → refuse. |
| **F30** | Wiring: marker file present → `ok` (existing probe). |
| **F31** | No PreflightContextResponse growth. |
| **F32** | Claude/Codex: `install_ready=false`; pending labels **`T239+`** (or “follow-up”) — **not** “T238”. Update detect tests + install_pending_summary strings. |
| **F33** | Plugin body must not hardcode user profile path; resolve `ai-brains` via PATH. |
| **F40** | **Config dir (AI2 M8):** if `OPENCODE_CONFIG_DIR` (or documented OpenCode config dir env) is set, install/probe under that root’s `plugins/`; CAPABILITIES caveat when env relocates config away from default probe. |

### E. Docs / capability / series

| ID | Decision |
|----|----------|
| **F34** | CAPABILITIES: OpenCode **Implemented** with caveats: **no SQLite**; live = `session.idle` (**deprecated-in-schema risk** — batch backstop); **child sessions skipped**; **synthetic/ignored text dropped**; bare non-synthetic user text kept; live prefers **SDK messages** (export fallback); batch = list+export+watermark; list default cap honesty; `source_ts` from msg time when present; `--pure`/plugin-disable soft; **OPENCODE_CONFIG_DIR** relocate; message-id turn keys; SYSTEM skip-import honesty (T239). |
| **F35** | OPERATIONS: install + plugin behavior (idle, parentID skip, in-flight) + `opencode-hook` + `opencode-import` + watermark + “never open opencode.db”. |
| **F36** | CHANGELOG + series README T238 Completed on ship. |
| **F37** | **Supersede** stale claims in `Docs/Opencode-Hooks-Research.md` (banner: historical; see OPERATIONS/T238). |
| **F38** | Zero new crates; no dep bump. |
| **F39** | Adapter capability: Partial→**Implemented**/Full notes; `supports_hooks: true` meaning plugin events. |

### F. Soft residuals (not DoD)

| ID | Residual |
|----|----------|
| **S1** | Min-interval debounce beyond in-flight guard (F15). |
| **S2** | Optional skip of compaction-related turns keyed on **`synthetic` + `metadata.compaction_continue`** (AI2 M7) — **not** `info.agent == "compaction"`. Default already drops synthetic text (F3). |
| **S3** | Live `message.updated` incremental ingest. |
| **S4** | ~~SDK messages for live~~ → **promoted F12**. Residual: pure-export live if SDK shape drifts. |
| **S5** | npm-published `@ai-brains/opencode-plugin`. |
| **S6** | Project-local plugin install opt-in. |
| **S7** | Import `--json` machine report. |
| **S8** | Wire Claude/Codex install_ready (later track). |
| **S9** | Nightly multi-harness orchestration (T239). |
| **S10** | `experimental.session.compacting` pre-archive hook. |
| **S11** | Opt-in child/subagent session ingest (default skip is hard F10). |
| **S12** | Dual-subscribe `session.status` idle as primary if `session.idle` removed. |

## Non-goals

- Opening / querying `opencode.db` as primary content API.
- Ingesting tool/reasoning/file/snapshot/patch/subtask parts, synthetic chrome, tool-output cache files.
- Default ingest of **child/subagent** sessions (skipped unless S11).
- session.created memory **injection** / preflight inside OpenCode (capture-only track).
- Blocking or altering OpenCode tool execution (`tool.execute.before` deny).
- npm package publish.
- Auto-merge of user `opencode.json` plugin array (v1 file auto-load).
- Multi-harness nightly orchestration (T239).
- Claude/Codex seamless install rewrite.
- Re-enable SYSTEM scheduled import (T239).
- Dep version bumps for their own sake.
- MSI packaging.
- ACP bridge / TUI theming.

## Acceptance criteria

| AC | Criterion |
|----|-----------|
| **AC1** | Live-shaped export fixture (nested `info.role` + text/reasoning/tool/step/**snapshot/patch/file/subtask/agent/retry**) → **only** user+assistant **non-synthetic text**; zero tool/reasoning/step/other part content. |
| **AC2** | Flat fixture `opencode_messages.jsonl` still passes (regression). |
| **AC3** | Hermetic `opencode-hook` with fixture export/messages path → appends kept turns; `thinking` None; turn ids stable on `msg_*`. |
| **AC4** | Same fixture → same turn ids live path vs batch path. |
| **AC5** | Batch bind via worktree/directory → correct project alias (not env hijack when bound). |
| **AC6** | Unbound → `opencode-unbound`; env project id only when unbound. |
| **AC7** | Re-import unchanged watermark → zero duplicate turns; cursor advanced. |
| **AC8** | `--force` reprocesses despite watermark; dry-run zero vault writes. |
| **AC9** | Real install writes plugin marker; `install_ready`; wiring `ok`; no backend_pending. |
| **AC10** | Dry-run install zero writes. |
| **AC11** | Uninstall managed plugin only; foreign plugins preserved; no destructive wipe of `opencode.jsonc`. |
| **AC12** | Missing `opencode` binary → soft skip with clear status (batch); no panic. |
| **AC13** | Path case variants → same project alias (normalize). |
| **AC14** | **Never** reads `opencode.db` on import/hook path (code search + design). |
| **AC15** | CAPABILITIES/OPERATIONS/CHANGELOG; OpenCode pending cleared; Claude/Codex labels not “T238”; honesty F34 (idle deprecation, child skip, synthetic drop, list cap). |
| **AC16** | Export timeout / oversized failure fail-open (session skipped, counted, continue). |
| **AC17** | Capture independence (no models/graph on path). |
| **AC18** | Plugin file contains managed marker; reinstall idempotent. |
| **AC19** | Part type `tool` (live) dropped even if unexpected text fields present. |
| **AC20** | `session list` + export integration hermetic with injected JSON (no network). |
| **AC21** | **Child skip (AI2 M1):** fixture/plugin path with `parentID` set → **zero** ingest; `skipped_child_session` increments (or equivalent). |
| **AC22** | **Synthetic chrome matrix (AI2 M2):** synthetic “Called the Read tool…”, “executed by the user”, `ignored: true`, `metadata.kind=editor_context`, `compaction_continue` → **zero** user memories; real bare user prompt kept. |
| **AC23** | List at cap: when mock returns length == max-count, stderr warns `list_capped` (or documented raise-and-retry once). |

## Risks

| Risk | Mitigation |
|------|------------|
| Export shape drift | Live fixture + tolerant serde; normalize layer; fail-open per message |
| 18MB+ export latency | Watermark; 120s timeout; live SDK messages (F12); `--max-sessions` |
| Synthetic chrome leak | F3 structured flags / AC22 Phase 1 Red |
| Child session pollution | F10 parentID skip / AC21 |
| `session.idle` removal | F34 honesty; batch backstop; S12 |
| List silent drop >100 | F17 cap sizing + warn AC23 |
| Plugin runtime (Bun) quirks | Pure JS; fail-open; batch independent |
| T234 flat-only filter | F1 unwrap hard / AC1 |
| Concurrent idle thrash | F15 in-flight + msg-id idempotency |
| Stale research doc | F37 banner |
| OPENCODE_CONFIG_DIR relocate | F40 + CAPABILITIES caveat |
| Cursor file corruption | Atomic write; `--force` recovery |

## §14 AI review fold-in (2026-08-09)

| Item | Source | Disposition |
|------|--------|-------------|
| Export timeout / fail-open / foreign plugins / T239+ labels | AI1 | **Affirmed** (already F12/F19/F28/F32) |
| Nested normalizer + part denylist + hook/import/install/docs | AI1 summary table | **Affirmed** implementation map |
| **M1** child/subagent `session.idle` | AI2 High | **F10 hard** / **AC21** / CAPABILITIES |
| **M2** `synthetic`/`ignored`/editor_context text | AI2 High | **F2/F3 hard** / **AC22** / Phase 1 Red first |
| **M3** `session.idle` deprecated | AI2 Medium | **F34** / **S12** / batch backstop |
| **M4** list cap-100 + `projectId` casing | AI2 Medium | **F17 hard** / **AC23** |
| **M5** live `client.session.messages` + in-flight | AI2 Medium | **F12 hard** (S4 promoted); **F15** in-flight; batch keeps export |
| **M6** full part-type union | AI2 Medium | **F3** / **AC1** expand |
| **M7** compaction skip key | AI2 Low | **S2** rewrite (synthetic + metadata) |
| **M8** OPENCODE_CONFIG_DIR | AI2 Low | **F40** / F34 caveat |
| **M9** prefer worktree | AI2 Low | **F20 hard** |
| Docs contradictions (permission events, shell.env) | AI2 note | **Out of scope** (not used by T238) |

## Verification plan (go day)

1. Targeted: adapters message_only + opencode fixtures (AC1/AC2/AC19/AC22 first); cli harness install/opencode-hook/import (AC21 child).
2. Manual: `harness install --harness opencode` → restart OpenCode → short turn → idle → recall; `opencode-import --days 1`; confirm no tool/reasoning/**synthetic chrome**; confirm no db open; child-task path if easy.
3. Full gate + `ledgerful verify --scope full`.
4. Internal + codex-review (FEATURE / harness).
5. Pins: nested export SOOT; synthetic drop; child skip; live SDK messages; no SQLite primary; msg-id turns.
