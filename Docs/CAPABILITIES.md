# AI-Brains — Capabilities & Features

**Version:** 0.1.2
**Platform:** Windows 11 first (PowerShell); Ubuntu 24.04 / WSL and macOS are tiered — see **[COMPATIBILITY.md](COMPATIBILITY.md)** (not a blanket “best-effort” claim)
**Type:** Local-first CLI + optional local daemon (not an MCP server)
**Related docs:** [README.md](README.md) (index) · [INSTALL.md](INSTALL.md) · [SECURITY-LIMITS.md](SECURITY-LIMITS.md) · [OPERATIONS.md](OPERATIONS.md) · [WORKFLOWS.md](WORKFLOWS.md) · [PRD.md](PRD.md) · [COMPATIBILITY.md](COMPATIBILITY.md) · [PROTOCOL-COMPAT.md](PROTOCOL-COMPAT.md) · [RECOVERY-DRILLS.md](RECOVERY-DRILLS.md) · [status.md](status.md) (historical) · [ADR-0019 connector sandbox](DECISIONS/ADR-0019-connector-sandbox-execution-model.md)

---

## 1. Product thesis

AI-Brains is a **durable, project-aware memory layer for AI coding harnesses**. It addresses harness amnesia, parallel-agent isolation, review-agent blindness, and IDE log loss by storing what matters:

```text
User: Do X
AI final response: I did X
```

**Deliberately excluded:** hidden chain-of-thought, tool/action sludge, raw intermediate tool logs.

| Mode | Purpose | When | Works offline (no models/graph)? |
|------|---------|------|----------------------------------|
| **Capture** | Append clean conversation events | Immediate | **Yes** (hard requirement) |
| **Brain** | Summarize, embed, graph, synthesize, inject | Scheduled / on demand | No |

---

## 2. Architectural pillars

| Pillar | Behavior |
|--------|----------|
| **Capture independence** | CLI → daemon → event log works without models, embeddings, or graph DBs |
| **Canonical source of truth** | Every state change is an immutable event in an append-only log (SQLCipher page-level vault + CE — [COMPATIBILITY.md](COMPATIBILITY.md) F8 / T187) |
| **CQRS** | Commands append events; queries read projections only |
| **Capture privacy** | Only user prompts + final assistant responses (no CoT / tool logs) |
| **Privacy inheritance** | Derived memories inherit the strictest privacy of sources |
| **Event sourcing** | No update/delete of raw events; corrections via compensating events |
| **No repo pollution** | Memory defaults to user storage; projects get `.env` IDs only |
| **Path normalization** | Windows drive-case, UNC, WSL `/mnt/c` mappings normalized |
| **Relational graph** | Native SQLite backend (recursive CTEs); Cozo bridge optional/feature-gated |
| **Licensing** | PolyForm Noncommercial 1.0.0 + Small-Entity Commercial Exception; deps stay permissive (see `deny.toml`) |

**Workspace crates:** `core` · `events` · `contracts` · `store` · `crypto` · `path` · `capture` · `retrieval` · `graph` · `models` · `brain` · `scheduler` · `ai-brains-daemon-api` · `ai-brains-api-server` · `ai-brainsd` · `ai-brains-cli`

**Loopback HTTP (T161):** optional authenticated `/v1` REST on `ai-brainsd` (default off; `AI_BRAINS_HTTP=1` / `--http`); same `DaemonRequest`/`DaemonResponse` contracts as named-pipe IPC via `HttpDispatch` → `handle_daemon_request`; bearer token + owner-only ACL; bind loopback-only by default.

### Provenance (user view)

Governed memory separates **what was observed** from **what we conclude** and **what we decide**:

| Kind | Operator meaning | CLI (T160 / T203) |
|------|------------------|-------------------|
| **Evidence / source** | Observed material with fingerprints and origin | `evidence list\|search\|show`, `source list\|show` |
| **Conclusion** | Derived claim that still needs review when required | `conclusion` |
| **Decision** | Accepted commitment (often after review) | `decision`, `review list\|resolve` |

**Rules of thumb:** evidence ≠ conclusion; circular or unrooted promotion stays **Unknown** (not silently “Independent”). Corrections use **compensating events**, not silent rewrite of the log. See [ADR-0011](DECISIONS/ADR-0011-separate-evidence-conclusions-decisions.md) and [OPERATIONS.md](OPERATIONS.md) governed command surface.

---

## 3. CLI surface

```text
Setup:     init
Daily:     recall | preflight | doctor | project | pin | context | stop-session | daemon
Operator:  backup | recovery | vault | retention | device | replicate | nightly | safety
Governed:  scope | briefing | query | evidence | source | review | policy | conclusion | decision
Dangerous: forget | erasure  (+ dual-ops: retention apply, vault encrypt|rotate-datakey, migrate governed --confirm, daemon install|uninstall|update)
Harness:   ingest | harness | antigravity-import | agy-hook | sync | shadow | evaluate | dogfood | graph | migrate
```

Canonical inventory is also on `ai-brains --help` (T204 groups). Partial historical one-liners are obsolete.

**Help information architecture (T204):** `ai-brains --help` appends role groups (Setup / Daily / Operator / Governed / Dangerous / Harness), a short Start-here block, and docs pointers. Names are unchanged; `display_order` only reorders the flat Commands list. Dangerous ops carry a `[dangerous]` about marker at the depth where mutation lives (e.g. `forget`, `erasure wipe`, `retention apply`, `vault rotate-datakey`). Short `-h` keeps a one-line tip only.

### OutputFormat defaults by command (consolidated)

Honesty matrix — **no** blanket TTY default flip for governed JSON surfaces.

Four families (T266):

| Family | Rule | How to force the other |
|--------|------|------------------------|
| **A auto** | TTY human / pipe JSON | `--format human` (table) · `--format json` (object) |
| **B always-human** | pipes stay human | `--format json` when the command has it |
| **C always-JSON** | default JSON | `--format human` opt-in |
| **D governed** | `OutputFormat::parse` silent-JSON on unknown | `--format human` / `--format json` |

| Surface | Default TTY | Default non-TTY | Notes |
|---------|-------------|-----------------|-------|
| `recall` | pretty | json | **A.** Explicit `--format` wins |
| `preflight` | human | json | **A.** TTY default is `human`; `--pretty` / `--format pretty` also human-mode. `--compact` is a bool flag (not a format token) and is ignored on JSON. |
| `briefing` | markdown | json | **A.** T202 F9 + **T227**: `human\|pretty\|text\|markdown\|md` → markdown; only `json` → JSON; unknown → exit **2** |
| `query progressive` / `expand` / `trace` | json | json | No TTY flip |
| list/show (governed evidence/source/review/...) | json (Human if `--format human`) | json | **D.** `OutputFormat::parse` → Json bare |
| `policy check` | human | json | **A.** T292: `--format auto` (default). TTY `allowed:` / `denied:` + SHORT; pipe / `--format json` pretty `CheckResult` / ApiError. Tokens case-sensitive (`JSON`/`Pretty` exit 2). `policy show` / `policy bootstrap` stay **D**. |
| `scope resolve` | human | json | **A.** T249: `--format auto` (default). TTY human (`scope:` / `confidence:` / evidence). Pipe / `--format json` pretty JSON. Keys frozen. Tokens case-sensitive (`JSON`/`Pretty` exit 2). |
| `project list-paths` | human | json | **A.** T266/T254: `--format auto` (default). Tokens `auto\|pretty\|human\|text\|json\|markdown\|md`. `--format pretty` ≡ table. Agents: `--format human`. Scripts: `--format json`. Keys frozen. |
| `project scan-roots` | human | json | **A.** Same token map as list-paths. Dry-run table / JSON envelope. |
| `project whoami` | human | json | **A.** Same token map. Identity remediator, not a default flip. |
| `project adopt-path` | human | json | **A.** Same token map. Print-only by default. |
| `project rebind-path` | human | json | **A.** Same token map. Print-only by default. |
| `graph neighbors` / `hierarchy` / `session` | pretty | json | **A.** T246: TTY table; pipe / `--format json` compact JSON. Keys frozen. T278: session PREVIEW is `{n} memories · first line` (human-only). T293: human prefer-fills authority 1-hop (`DECISION:`/`CONSTRAINT:`/`INVARIANT:`/`HOTSPOT:`); JSON array order stays direction→label→id. |
| `graph update` | json | json | **C.** T246 F6 / T74: default pretty JSON. `--format auto` stays JSON (no TTY switch). `--format human` opt-in labeled lines. |
| `nightly --status` | human | human | **B.** T255 F2: **pipes stay human**. Scripts pass `--format json`. |
| `harness status` | human | human | **B.** Default `human`; `--format json` only when the token is `json`. |
| `memory list` | human | human | **B.** `--format human` (default) or `json`. No `auto`. |
| `project list` | human | human | **B.** `--format human` (default) or `json`. No `auto`. |
| `doctor` | human | human | **B.** `--json` / `--format json` override (full `DoctorReport`). `--summary` is opt-in compact of the same 15-check report (warn+fail attention or `No issues.`). Does **not** TTY-switch. |
| `daemon status` | human | human | **B.** No `--format`. When Stopped and LLM/Embedding TCP is Open, prints `backend TCP Open ≠ daemon` (IPC vs model TCP — not nightly’s T281 `HTTP /health 750ms ≠ daemon TCP`) **before** the last line. Stopped last line: `next: ai-brains daemon start`. Running omits contrast and `next:`. Exit **0** both states. Keyless liveness (T199). |
| `device list` | human | human | **B.** Human table only. No `--format`. Empty = T198 `No enrolled devices. Run \`ai-brains device bootstrap\` first.` |
| `device status` | human | human | **B.** Human-only. Same roster as `device list` + this-machine (`{hostname} (not enrolled)` or hyphen fingerprint) + short honesty `local-only; not PQ; not remote wipe` + **always** last-line `next: ai-brains replicate status` (empty and enrolled). No `--format` (unknown flags clap exit **2**). Not a JSON DTO. |
| `replicate status` | human | human | Human adds `this machine:` (same empty/enrolled label as `device status`). `--format json` is the machine enrollment path (keys frozen; unchanged — no `this_machine`). Does **not** TTY-switch. |
| `retention plan` | human | json | **A.** T248: `--format auto` (default). TTY human class matrix; pipe / `--format json` pretty JSON. Report keys frozen. Live vaults overlay `memory_legacy` as inventory (`none_auto`): pinned→`held`, other statuses→`skip`; `Nothing to dispose.` means no CE/projection work (T270). Human **Work** lists dispose identities (class CE/projection counts), not the class dominant mechanism — a held-majority `secret` still shows a `ce_wipe` Work row (T284). |
| `retention apply` | json | json | **C.** Default JSON (dangerous). `--format auto` does **not** TTY-switch. Opt-in `--format human`. Confirm/scope gates unchanged. |

Operator: `retention plan` on a TTY is a scannable class/horizon matrix (empty vault still prints the schedule). Scripts should pass `--format json`. `retention apply` stays JSON unless `--format human` is explicit. `scope resolve` on a TTY is human; scripts should pass `--format json`. Default `doctor` stays the full 15-check listing. `device list` / `device status` are human-only (no `--format`). Machine enrollment is `replicate status --format json` (keys frozen).

**Global options:**

| Flag / env | Purpose |
|------------|---------|
| `--vault-path` / `AI_BRAINS_VAULT_PATH` | Vault database path |
| `--key` / `AI_BRAINS_KEY` | Vault SQLCipher key (`x'<64 hex>'`). Zero key refused unless `AI_BRAINS_ALLOW_ZERO_KEY=1` (T187). Root `--help` / `-h` / `help` document `[env: AI_BRAINS_KEY]` and never echo the live value (T256) |
| `--no-project-context` | CI/hooks: do not load project `.env` or clobber inherited IDs |
| `--log-format` | `compact` \| `full` \| `json` \| `minimal` \| `off` |

Failures use a **dual envelope** model (see [CLI-EXIT-CODES.md](CLI-EXIT-CODES.md)):

| Path | Shape | Stream |
|------|-------|--------|
| Governed **Json** | bare `ApiError` | **stdout** |
| Governed **Human** / Markdown | `CODE: message` | **stderr** |
| Generic non-governed (`handle_cli_result`) | full `ApiResult` | **stderr** |

Normative exit codes **0–7** (including `FEATURE_UNAVAILABLE`→**2**, doctor footnotes, vault key exit **1**): [CLI-EXIT-CODES.md](CLI-EXIT-CODES.md).

---

## 4. Capture & ingest

### Capture Privacy (message-only SOOT — T234)

**Shipped:** shared pure module `ai_brains_adapters::message_only` is the single keep/drop contract for harness → vault content:

| Keep | Drop |
|------|------|
| User prompt text (after chrome strip: `<USER_REQUEST>`, `<user_query>`, strip metadata blocks) | Tool calls / tool results / `VIEW_FILE` / `RUN_COMMAND` / backend tools **regardless of content** |
| Final assistant **visible** text (multipart: text parts only) | `thinking` / `reasoning` / redacted CoT; system chrome; empty after strip |

- **Capture independence:** filter is pure string/JSON — no models, embeddings, or graph.
- **`IngestRequest.thinking`:** field remains on the contracts DTO for serialization compat; adapters / message_only **never populate** it; event builders never write thinking into the event log.
- **Wired today:** shared `parse_transcript_for_ingest` (step-shaped + legacy `{role,content}`, prefer `transcript_full.jsonl`), `antigravity::extract_turns` / import + `agy-hook` (message-only SOOT); ProjectChat → `filter_turn`; **Grok** `filter_grok_history_*` (F11 user_query-only) + `grok-hook` / `grok-import`; **OpenCode** `filter_opencode_*` / `filter_opencode_export` (nested + synthetic drop) + `opencode-hook` / `opencode-import`; **Claude** `filter_claude_jsonl_*` + `claude-hook` / `claude-import`; **Codex** `filter_codex_rollout_*` + `codex-hook` / `codex-import`.
- AGY live+batch seamless ingest: **Implemented with caveats** (T236). Grok Build: **Implemented with caveats** (T237). OpenCode: **Implemented with caveats** (T238). Claude Code + Codex: **Implemented with caveats** (T253 — install_ready; no nightly). Multi-harness nightly orchestration: **Implemented (T239)** (agy → grok → opencode only; Claude/Codex stay excluded).

### Manual / programmatic
- **`ingest`** — JSON turn from stdin (`session_id`, `project_id`, `harness_id`, `turn_id`, `role`, `content`, `privacy`). Empty / whitespace-only / TTY stdin → **exit 2** `fail_usage` with a copy-paste example payload (not EOF `COMMAND_FAILED`). Mid-payload parse (`{`, truncated object) stays **exit 1**.
- **`--dry-run`** — preview without write (relaxed validation on dry-run path). Same empty/TTY usage **exit 2** as live `ingest`.

### Harness integrations

| Integration | Mechanism | Notes |
|-------------|-----------|--------|
| **Detect + install UX (T235/T267)** | `harness status\|install\|uninstall\|reset-decline` | Detect harnesses **installed on machine** (PATH + home); wiring `absent\|missing\|partial\|ok\|backend_pending\|unknown`. User-global only (no repo pollution). Preflight `--summary` shows **Harnesses installed on machine:** sibling section. **Activation (T245+T253):** `ai-brains harness install --harness all-ready --dry-run` then `--yes` (ready order grok → agy → opencode → **claude → codex** — five). PATH bake: wrappers + OpenCode `ai-brains` spawn use the installing exe absolute path with PATH fallback; re-run install after CLI upgrade. Doctor `harness_wiring` (soft ok) lists ready-missing only — no T253 pending bucket once Claude/Codex are install_ready. **T267:** `wiring=ok` is done — human `harness status` **omits** `next:`; JSON `next_action` is the token `none` (key stays). Missing/Partial/Unknown still name `harness install --harness X --dry-run`. Ready-trailer install lines print only for present && not Ok. After a successful install/uninstall, `next:` stays `harness status`. |
| **agy (Antigravity CLI)** | `harness install --harness agy` → Stop wrapper → `agy-hook --payload` | **Implemented (T235+T236+T245):** always merge official IDE `~/.gemini/config/hooks.json` managed key `ai-brains-capture`. **Iff** `~/.gemini/antigravity-cli` already exists, also stage CLI plugin bundle `plugins/ai-brains-capture/{plugin.json,hooks.json}`. Never create `antigravity-cli` just for plugins. Never write undocumented top-level `antigravity-cli/hooks.json`. Wrapper stdout = allow-stop JSON only; step-shaped + full transcript prefer; path normalize; env fallback only for `agy-unbound`. Reinstall after T236 / after CLI upgrade (baked path). |
| **agy-hook** | `agy-hook --payload '{...}'` | Real-time ingest; shared parse + turn-id SOOT; diagnostics on stderr; `--schema` |
| **Antigravity bulk** | `antigravity-import --days N [--force]` | History.jsonl workspace bind; unbound `agy-unbound`; stats on stderr; `--force` skips 300s quiescence; default `allow_default_project=false`. Nightly multi-import (T239) includes AGY; SYSTEM scheduled nightly keeps `--skip-import`. |
| **Grok Build** | `harness install --harness grok` → Stop/SessionEnd wrapper → `grok-hook` | **Implemented with caveats (T237):** `~/.grok/hooks/ai-brains.json` + `~/.ai-brains/hooks/grok-capture.ps1`. **Stop allow = empty stdout** (never AGY `{"decision":"allow"}`). User keep: non-empty `<user_query>`/`<USER_REQUEST>` only (chrome/`synthetic_reason` dropped). Subagent/worktree sessions skipped by default. `source_ts` usually none → `occurred_at` = ingest time. Turn ids `v5(session,"turn-{i}")` on kept index (filter taxonomy change can shift ids). Vendor-compat: Grok may also load Claude/Cursor hooks. |
| **grok-hook** | `grok-hook --payload '{...}'` | Live chat_history ingest; path resolve (percent-encode + `.cwd` + summary.id); diagnostics stderr; `--schema` |
| **Grok bulk** | `grok-import --days N [--force]` | Walks `~/.grok/sessions/**/chat_history.jsonl`; summary bind (`git_root_dir`/`cwd`); unbound `grok-unbound`; never `updates.jsonl`; stats on stderr |
| **OpenCode** | `harness install --harness opencode` → plugin `session.idle` **or** idle `session.status` → `opencode-hook` | **Implemented with caveats (T238+T245):** `~/.config/opencode/plugins/ai-brains-capture.js` (or `OPENCODE_CONFIG_DIR/plugins/`). Marker remains `// AI-Brains managed (T238)`. Dual-subscribe: `session.idle` **or** `session.status` with `status.type == "idle"` (exact; no aliases). `session.idle` is **not** deprecated — dual-subscribe is resilience (both events fire together). Never rewrite `opencode.json`. PATH bake on the `ai-brains` spawn only (PATH fallback). Live prefers SDK `client.session.messages` (CLI export fallback, 120s; temp export unlinked after hook). **Child/subagent** sessions with `parentID` skipped (plugin fail-closed if `session.get` throws). **Synthetic/ignored/editor_context** text parts dropped; bare non-synthetic user text kept. Nested export `{info,parts}` normalized. Turn ids use `msg_*` for stability (`v5(session,msg_id)`); **delta is max turn_index + watermark** (same class as Grok `turn-{i}` residual — not per-msg_id existence). Never opens `opencode.db`. List default cap 100 (`list_capped` warn when len≥100 even if `--max-sessions` higher). `--pure` / plugin-disable soft. SYSTEM skip-import honesty (T239). |
| **opencode-hook** | `opencode-hook --payload '{...}'` | Live/batch-shaped export or messages path; parentId skip; worktree→directory bind; unbound anti-hijack; `--schema` |
| **OpenCode bulk** | `opencode-import --days N [--force] [--dry-run] [--max-sessions N]` | `opencode session list --format json` + `export`; watermark `~/.ai-brains/opencode-import-cursor.json`; never SQLite; soft skip if binary missing |
| **Claude Code** | `harness install --harness claude` → UPS+Stop+SessionEnd wrapper → `claude-hook` | **Implemented with caveats (T253):** user-global `~/.claude/settings.json` + `~/.ai-brains/hooks/claude-capture.ps1`. Live SOOT is `prompt` / `last_assistant_message` after T234 `filter_turn` (no `transcript_path` on the live path). No SessionStart injection. No nightly. Grok-shaped stdin fail-open skip. Batch `claude-import` walks `~/.claude/projects/<encoded-cwd>/*.jsonl` (skip `subagents/` / `isSidechain`). Unbound `claude-unbound`. |
| **claude-hook** | `claude-hook --payload '{...}'` | Live message-only ingest; `--schema` (vault-path-free). Mid-payload garbage exits 1 JSON; unrecognized stdin exits 0. |
| **Claude bulk** | `claude-import --days N [--force] [--dry-run]` | Project JSONL; `--force` skips 300s quiescence; not in nightly |
| **Codex CLI** | `harness install --harness codex` → UPS+Stop wrapper → `codex-hook` | **Implemented with caveats (T253):** user-global `~/.codex/hooks.json` only (never rewrite `config.toml`). Feature key is **`hooks`** (not `codex_hooks`). Live SOOT is `prompt` / `last_assistant_message`. **`/hooks` trust required** for live fire — `wiring=ok` means files exist, not that this session trusted the hash. No SessionStart injection. No nightly. Batch `codex-import` walks `~/.codex/sessions/**/rollout-*.jsonl` (fail-open; format not vendor-stable). Unbound `codex-unbound`. |
| **codex-hook** | `codex-hook --payload '{...}'` | Live message-only ingest; `--schema` (vault-path-free). Missing fields exit 0. |
| **Codex bulk** | `codex-import --days N [--force] [--dry-run]` | Rollout JSONL keep `response_item` message roles only; not in nightly |

**Consent:** TTY preflight may prompt once; decline persists in `~/.ai-brains/harness_hooks.json`. Never prompt when non-TTY, `--no-hook-prompt`, or `preflight --stdin`. Reset with `harness reset-decline`.

### Daemon write path
- **`ai-brainsd`** — single-writer queue for concurrent safety
- **Auto-launch** — CLI spawns daemon when the pipe is unreachable
- **Named pipe:** `\\.\pipe\ledgerful-bridge` (aligned with Ledgerful 0064)
- **Windows service:** `daemon install` / `uninstall` (LocalSystem Session 0 + SDDL cross-session access)
- **Deprecated:** `daemon schedule` / `unschedule` (Task Scheduler logon)
- Lifecycle: `start` · `status` · `stop [--force]` · `update`

Most users never need an explicit start: the CLI auto-launches. A Windows service is optional for always-on Session 0 operation.

---

## 5. Project, session & context

| Capability | Command / detail |
|------------|------------------|
| Init project context | `context` — first-init writes local `.env` (`PROJECT_ID`, `SESSION_ID`, `HARNESS_ID`); already-initialized ensures those IDs in the open vault and does **not** rewrite `.env` |
| Show only | `context --show` — when pre-dotenv shell `PROJECT_ID` differs from the file, the next line is `shell leftover PROJECT_ID: <uuid> (.env overrides)`; `AI_BRAINS_KEY` / `VAULT_KEY` file lines print `(redacted)` |
| Rotate project / session | `--new-project` · `--new-session` |
| Ledger linkage | `--tx-id` / `LEDGERFUL_TX_ID` (legacy `CHANGEGUARD_TX_ID` fallback) |
| List projects | `project list` — **label-first** human table: `label` \| `project_id` (full) \| `memories` \| `last_activity` \| `path`. Label prefers alias; baked/machine names show as `(no alias)`. Active process `AI_BRAINS_PROJECT_ID` gets a `*` prefix on the label. Human table puts the cwd path-owner first (remaining rows stay memory-desc); JSON array order stays T212 size-desc. |
| List JSON | `project list --format json` — `{ api_version, projects[{project_id,name,alias,label,memory_count,last_activity,path}], unaliased_count }` (pretty). Path is `null` when absent. No dual `--json` flag. Array order is `memory_count DESC, project_id ASC` (not cwd-first). |
| last_activity | Last **memory-projection mutation** (pin / forget / ingest / turn upsert), falling back to project `updated_at` when the project has no memories — **not** “last user chat message only.” Human: relative when age &lt; 365d (`just now` / `Nm` / `Nh` / `Nd`); else `YYYY-MM-DD`. |
| path column | Registered `repository_path_alias_projection.normalized_path` when present (lexicographically first); never invented from cwd/git. Human `—` / JSON `null` when unknown. |
| Unaliased nudge | When ≥1 project has no alias, **stderr** prints count + copy-paste `ai-brains project set-alias <uuid> <suggestion>`. Pick (T267 F3): cwd path-owner if unaliased; else first unaliased with exactly one registered path; else first orphan (zero paths); else highest-memory leftover. Suggestion (F3b): cwd git slug **only** when the picked id is that path-owner; otherwise sanitized basename of the target’s registered path, else `my-project`. Never pair a non-owner id with the cwd slug (leftover dump is not `AI-Brains`). Empty vault: T198 empty line only — **no** footer. JSON has no footer. Exit 0 always. |
| Aliases | `project set-alias` · `project resolve` — **human labels** only (not disk roots) |
| Path aliases (T233/T254) | `project register-path <project_id\|alias> <path>` — filesystem roots for multi-root nightly Phase 2. Normalize via path crate (Win + WSL forms). Same normalized path → one project (conflict exit **1**; stderr names `unregister-path`). Same project re-register idempotent. CLI F21 is the operator gate; projection `ON CONFLICT` **refuses to steal** from another project. |
| List all path aliases (T254/T259/T266) | `project list-paths` — every registered root (not just `project list` first-path). Optional `--project <id\|alias>` and `--shared-only` (owner appears on ≥2 roots). Combined flags are an intersection. `--format auto` (TTY table / pipe JSON); `pretty\|human\|text\|markdown\|md` → table; `json` → object. Tokens case-sensitive (`JSON`/`Pretty` exit **2**). JSON `{ api_version:"1", paths:[{project_id,label,alias,normalized_path,exists}] }` ASC by path. Unfiltered empty: `No path aliases registered.` + `next: ai-brains project register-path <project_id\|alias> <path>` (exit **0**). Filtered empty: `No path aliases match.` / `paths: []` (exit **0**). Unknown `--project` dest exit **1**. |
| Unregister path (T254) | `project unregister-path <path> [--project <id\|alias>] [--dry-run]` — compensating `RepositoryPathAliasRemoved`. Path unique. Missing path idempotent exit **0**. Owner mismatch exit **1**. Does **not** forget `MemoryPinned` / `ledgerful:symbol` history. Reversible via `register-path`. |
| Rebind path (T259) | `project rebind-path <path> --to <id\|alias>` — move **one** path alias to an existing dest project. Default **print-only** (no events). `--write --yes` appends `RepositoryPathAliasRemoved` + `RepositoryPathAliasAdded` in **one** store transaction. `--write` without `--yes` is usage exit **2**. Already bound → exit **0**, no events. No owner → exit **1**, names `register-path`. Dest missing → exit **1** (does **not** mint). JSON `{ api_version, path, from_project_id, to_project_id, already_bound, written, memories_moved, events_appended }` (`from_project_id` is always a UUID; no-owner never emits JSON). **`memories_moved` is always false** — historical pins stay on the from-project. Does **not** write `.env` (use `adopt-path` for daily Scope). Never recommend `set-alias 7d97a456 AI-Brains`. |
| Scan Ledgerful roots (T254/T266/T268) | `project scan-roots [path]` / `project scan-roots --root DIR` — dry-run. `--root` is a named XOR of the positional path (both set → clap exit **2**). Default is **cwd** (not the parent). Immediate children (plus scan root) that contain `.ledgerful`. Never appends events. Never writes `.env`. Never auto-registers. `.changeguard` alone is **not** a hit. Cap 200 children. Same `--format` token map as list-paths. Already-registered hits keep `registered_project_id` and set JSON `suggested` to `""` (human `—`) — not a remediator. Unregistered hits still suggest `register-path`. Implicit-cwd human with zero unregistered hits may print `next: ai-brains project scan-roots --root <git-toplevel-parent>` (not a volume/share root; not on JSON). JSON `{ api_version, scan_root, truncated, roots:[{path,registered_project_id,exists,suggested}] }`. |
| Auto-detect (T240) | `project detect` order: **(1)** path alias of git toplevel else cwd → **(2)** git slug exact-first (T206; ambiguous exit **1** when no path) → **(3)** env `PROJECT_ID` post-dotenv if in vault → **(4)** miss exit **1**. Path owner **always** wins over unique slug hit; stderr notes the slug project (extra note when path has 0 memories and slug has &gt;0). `--export` comments include `source=path_alias` / `git_slug` / `env`. |
| Whoami (T240/T266) | `project whoami` — all identity signals: `effective_project_id`, `env_project_id` (post-dotenv; null under `--no-project-context`), `shell_project_id` (pre-dotenv when set/differs), `path_alias_project_id`, `detect_project_id`, `git_slug`, `git_toplevel`, `mismatch`, `remediations[]`. `--format auto` (TTY human / pipe JSON); same T266 token map as list-paths. Does **not** rewrite `PROJECT_ID`. On mismatch, remediations name `project adopt-path` (print-only / `--write-env --yes`) plus the exact `AI_BRAINS_PROJECT_ID=<path-id>` assignment — not `project list`, not “run whoami”. |
| Adopt path (T258/T266) | `project adopt-path` — bind daily Scope to the path-alias owner of cwd / git toplevel. Default **print-only** (names owner + exact `AI_BRAINS_PROJECT_ID=<id>` assignment; no file write). `--write-env --yes` rewrites **only** that key in cwd `.env` (creates the file if missing). `--write-env` without `--yes` is usage exit **2**. Already bound → exit **0**, no rewrite. No path owner → exit **1**, names `register-path`. Same T266 `--format` token map as whoami. `context` initializes / rotates; it is **not** adopt-path. Never writes `~/.ai-brains/.env`. Never silent auto-switch (T240 F2). |
| Mismatch warn (T240/T257) | Once per process when daily Scope env ≠ path-alias owner (after vault open). Human SOOT (stderr only, after the command): `Warning: project identity mismatch: daily Scope is '{env}', but path is registered to '{path}'. Run 'ai-brains project whoami'.` **JSON-effective silent:** when the command writes JSON on stdout, the human line is omitted so `2>&1` stays one parseable object. `scope resolve` JSON injects exactly one token into existing `warnings[]`: `project_identity_mismatch env=<uuid> path=<uuid>` (no `Warning:`). Skip: `--no-project-context`, argv `--global`, empty env, no path alias, **`project whoami`**, **`project adopt-path`**. Whoami JSON keeps `mismatch: true`. Nightly status JSON is silent (no `warnings` key). **Never** auto-switches Scope. |
| Stop session | `stop-session` |
| Env precedence | CLI flags / shell env > elevation handoff (elevated child only) > project `.env` > global `~\.ai-brains\.env` (always merged for gaps; `--no-project-context` skips project only) |
| Local ID force-set | Project-local `.env` **always force-sets** `AI_BRAINS_PROJECT_ID` / `AI_BRAINS_SESSION_ID` (cwd project beats a stale shell). Other keys still follow shell > project > global gap-fill. |
| Override warn (T223/T242) | When shell had a **different** ID value, CLI may print **one** collapsed stderr line: `Warning: local .env overrides inherited shell: AI_BRAINS_PROJECT_ID (was …)[, AI_BRAINS_SESSION_ID (was …)].` **Session-only** override (PROJECT equal/missing) → no stderr; debug only; **no session marker**. **Project** differ: first warn for a fingerprint, then session-quiet across process spawns. |
| Session quiet (T242) | After the first stderr warn for a given situation, further CLI spawns with the **same fingerprint** suppress stderr (debug only). Fingerprint = SHA-256 of normalized `.env` parent + shell old PROJECT/SESSION + `.env` new PROJECT/SESSION. Marker: empty file under `~\.ai-brains\cache\env-override-warn\<hex>` (never the git worktree). **Re-warn** when cwd/shell/`.env` IDs change (new fingerprint). Manual reset: delete that cache dir. |
| Quiet override warn | `AI_BRAINS_QUIET_ENV_WARN=1`/`true`/`yes` (case-insensitive) → no stderr override line (collapsed `debug` only); no marker required. **Quiet wins over force.** Honored from **shell env or project `.env` only** (already loaded at emit). Global `~/.ai-brains/.env` alone is **too late** (loads after apply). Distinct from T206 `git/env project mismatch` and T240 identity mismatch. |
| Force override warn | `AI_BRAINS_FORCE_ENV_WARN=1`/`true`/`yes` → always stderr for project-differ (ignores session marker). Suppressed when quiet is also set. |

Discovery prefers **`.ledgerful/`**, falls back to legacy **`.changeguard/`**.

---

## 6. Dense memory APIs

### Pin
```powershell
ai-brains pin "DECISION: …" --tag architecture
ai-brains pin --stdin --role user --privacy LocalOnly --dry-run
```
- Roles, privacy, tags, tx-id linkage
- Prints projection `memory_id` for later forget
- Emits **`MemoryPinned`** events for live graph edges

### Forget / restore (soft delete)
```powershell
ai-brains forget --memory-id <uuid> -f
ai-brains forget --match "outdated" -f
ai-brains forget --list-forgotten --limit 5
ai-brains forget --restore <uuid>
ai-brains forget --dry-run …
```
Forgotten items remain in the event log (audit) but drop from FTS / graph / preflight.
**Soft-forget is not CE wipe / not NIST Purge** — `forget --restore` reverses soft-delete; list/restore do not purge ciphertext.

### Memory inventory (T216 / T287)
```powershell
ai-brains memory list                          # default status=pinned, limit 50
ai-brains memory list --status forgotten -l 5
ai-brains memory list --summary
ai-brains memory list --summary --global
ai-brains memory list --format json --limit 3
ai-brains memory list --tag architecture
ai-brains forget --list-forgotten --global --format json   # same backend as --status forgotten
```

| Feature | Detail |
|---------|--------|
| **Primary** | `memory list` is **read-only** (not `[dangerous]`); never appends events. |
| **Human pinned prefer-fill (T287)** | Default/`--format human` `--status pinned` prefer-fills leading-line `DECISION:` / `CONSTRAINT:` / `INVARIANT:` / `HOTSPOT:` (TAGS envelope skipped in preview). `--format json` and store `list_memories` stay `updated_at DESC` recency. Forgotten lists are recency-only. |
| **Scope** | Project default (`AI_BRAINS_PROJECT_ID` / `--project-id`); without project and without `--global` → exit **2**. `--global` → `Scope: global`. |
| **Status** | `--status pinned\|forgotten` (default **pinned**). `forget --list-forgotten` ≡ `memory list --status forgotten` (+ limit/scope/format/tag). |
| **Limit** | Default **50**, max **200** (`clamp_list_limit`); `more_available` / `Showing N of T`. **BREAKING:** list-forgotten no longer dumps unbounded rows. |
| **Summary** | `--summary` always prints **Pinned** + **Forgotten** (ignores `--status`/`--limit`). Under `--global`: by-project table (projects with either count &gt; 0 only; **turn-only projects excluded** — use `project list` for those). With `--tag`, top-line **and** by-project cells use the same two-stage tag filter (F46). |
| **Labels never blank (T230)** | Inventory tables that use `display_label` never emit an empty label: empty/whitespace **name** with empty alias → `(no alias)`. **Orphan** `project_id`s (memories without a `project_projection` row) show the same token. Alias wins over empty name; alias is **not** trimmed. Non-summary list JSON stays **id-only** (no `label` field on items). |
| **Tags** | Content-prefix heuristic only (`TAGS:` first line after optional role prefix from `pin --tag`); SQL start-anchored `LIKE 'TAGS:%'` (or `ROLE: TAGS:%`) + case-insensitive exact token. Not a schema column. Sparse token density among `TAGS:` rows can under-fill a page under elevated candidate cap (raise `--limit`). |
| **Formats** | human table (Scope + preview with role-prefix strip) or `--format json` (`api_version`, items, total, more_available). Summary JSON `by_project[].label` is always a non-empty string when present. |
| **Empty** | `No pinned memories.` / `No forgotten memories.` exit **0**. Forgotten-empty (human) also prints `Pinned: N` (same COUNT as `--summary`) and last-line `next: ai-brains memory list` (`--global` appends `--global` on that next only). JSON empty forgotten stays nine keys (`items: []`); no `next_step`. |

---

## 7. Retrieval & orientation

### Recall
```powershell
ai-brains recall "auth flow" --limit 5
ai-brains recall "login" --semantic --graph-boost 0.1
ai-brains recall "query" --global --no-bridge --quiet
ai-brains recall -   # query from stdin
```

| Feature | Detail |
|---------|--------|
| **FTS5** | Default lexical path; sanitized queries (quoted tokens; `_` split like unicode61). FTS5 BM25 `rank` is more-negative-is-better (`ScoreKind::Bm25LowerBetter`). **T217 multi-token rescue (recall only):** when primary full-AND is empty and the query has ≥3 tokens, opt-in ladder runs contentful-AND then contentful-OR (cap 8 tokens, stopwords fixed/literal, negators kept); every MATCH uses `ORDER BY rank LIMIT` (hard cap 200 / `candidate_depth`). `forget --match` stays strict R0 (no rescue). `source` remains `"fts"`. |
| **Semantic** | `--semantic` + stored embeddings; honors `AI_BRAINS_EMBEDDING_URL` (default `http://127.0.0.1:8083`) and `AI_BRAINS_EMBEDDING_MODEL` (default `nomic-embed-text-v1.5`). **Hybrid-arm** cosine floor default **0.55** (`AI_BRAINS_SEMANTIC_MIN_SCORE`). Candidates below floor are dropped before fusion. Default semantic SELECT also GLOB-excludes T70 stubs (T260); `--symbols` can still mix them in the shared embedding space. Floors are **model/corpus calibrated** for default nomic-embed-text-v1.5 without task prefixes (both query and documents currently unprefixed / symmetric); a future nomic `search_query`/`search_document` prefix cutover requires vault re-embed **and** floor re-tune. |
| **Dual cosine floor (T218)** | When the pre-RRF local arm has **no** `source=="fts"` hit (empty FTS **or substring-only** — substring does **not** count as FTS), a second gate **`SEMANTIC_ONLY_MIN=0.60`** (`AI_BRAINS_SEMANTIC_ONLY_MIN_SCORE`) is applied **after** the 0.55 hybrid-arm filter and **before** RRF. When any true FTS hit exists (incl. T217 rescue), only the hybrid-arm 0.55 floor applies. Gate helper: `has_fts_arm` — **not** `local_hits.is_empty()`. |
| **`--min-score` override (T218 F2b)** | One-shot CLI floor for `--semantic`. When set to **X**, **replaces** both the hybrid-arm default (0.55) **and** the semantic-only default (0.60) with **X** (not `max()`). So `--min-score 0.57` can re-admit residual neighbors that default dual floor would drop. Omit the flag to keep dual-floor defaults. |
| **Hybrid RRF (T215/T218)** | When `--semantic` is on, **FTS-only ranked list + thresholded semantic ranked list** are fused with pure **Reciprocal Rank Fusion** (`score = Σ 1/(k+rank)`, default **k=60**, env `AI_BRAINS_RRF_K`) **before** bridge merge / graph / `rerank_hits`. Equal weights only. **Missing from a list → no RRF summand** (not rank=`len+1`). Both arms → source `"hybrid"` (FTS content/`updated_at` preferred; **pre-fuse cosine preserved** from semantic arm when available); FTS-only → `"fts"` (score is RRF contribution under `--semantic`); semantic-only → `"semantic"`. **Substring hits** are **outside** RRF (merged after fuse by id-dedupe; bridge wins first). Candidate depth per arm: `max(limit*3, 15).min(50)`; final truncate after pin re-rank. Without `--semantic`: no RRF; blend is bridge → FTS → graph → re-rank. `sync query` stays `semantic: false`. |
| **Score polarity (ScoreKind)** | `Bm25LowerBetter` (FTS/substring): composite base = `-score`. `HigherIsBetter` (cosine / RRF): base = `score * RELEVANCE_SCALE` (**500**). `BridgeHigherIsBetter` (Ledgerful Tantivy): base = raw relevance **unscaled / not negated** so large bridge scores (~10–20) stay authority-class. Graph hits **inherit** the parent’s `score_kind`. |
| **JSON score honesty (T218)** | Additive per-result fields: **`score_kind`** closed wire set **`bm25` \| `rrf` \| `bridge` only** (map: `Bm25LowerBetter`→`bm25`, `HigherIsBetter`→`rrf` including fused hybrid and single-list identity RRF, `BridgeHigherIsBetter`→`bridge` — never emit wire kinds `cosine` or `hybrid`; origin stays on **`source`**). Optional **`cosine`** when known (pre-fuse dense sim). JSON **`score`** remains the machine value (RRF under `--semantic` after fuse; BM25 rank without) — **not** rescaled into a fake 0–1 confidence. |
| **Pretty scores (T218 F6)** | Branch on ScoreKind: **RRF / HigherIsBetter** → primary **`rank=#n`** + **`sim=0.XX`** when cosine known (do **not** primary-print raw RRF ~0.016). **BM25** keeps `score={:.3}` (negative polarity). **Bridge** keeps readable raw `score={:.3}`. |
| **Role strip on human previews (T224)** | Pretty recall / `sync query` hit lines and forget human match previews strip leading case-sensitive `USER:` / `ASSISTANT:` / `SYSTEM:` (shared `strip_role_prefix` SOOT; pretty: `trim_start` then strip **before** 500-char truncate; forget: via `memory::preview_line` budgets 100 dry-run/single/UUID, 80 multi-match with `…` on cut). **JSON** `RecallResult.content`, bridge Insight export, and **MemoryPinned** event payloads stay **raw**. **ingest/pin `--dry-run`** previews stay **raw** (write-intent honesty — exactly what will be stored). |
| **Embedding status** | With `--semantic`, JSON includes additive `embedding: { status, endpoint?, detail? }`. Closed statuses: `ok` \| `unreachable` \| `error` \| `no_stored_embeddings` \| `skipped`. Retrieval emits **`skipped`** when the query is contentless (T261: `detail=contentless_query`, `endpoint=None`; no embed HTTP). Soft-fail: embed down never aborts FTS/bridge recall (exit **0**). Pretty: status line when `status != ok`; when `status == ok` but **zero** semantic hits above the **hybrid-arm** cosine floor (post-0.55 / pre-dual-floor count) and lexical results are non-empty → `Embedding: ok (no semantic hits above threshold; showing lexical)`. All-below-floor is not an error. |
| **T70 symbol stubs (T260)** | Default `recall` / `search` / `sync query` vault arm / daemon recall **exclude** T70 code-symbol one-liners (`Module` / `Struct` / `Function` / … `{kind} {name} ({path}:{line})`) from the candidate set (SQL `GLOB` + in-memory detector). **`--symbols`** restores a **mix** (stubs + decisions), not symbols-only. Pretty badges `[symbol]` in the same chrome slot as `[plan/stale?]` (outside the 500-char slice). JSON `content` stays raw; **no** `is_symbol` DTO field. Identical stub `content` collapses to one row after `rerank_hits`. `forget --match` is unfiltered and still finds stubs. |
| **Contentless query (T261)** | Zero contentful tokens (`""`, whitespace, punctuation-only, all-stopwords, single-char-only such as `"a"` / `"i"`) is the existing T207 empty envelope: no FTS rescue, no substring LIKE, no Ledgerful bridge, no embed HTTP, no graph expansion, no `MemoryPinned`. `--symbols` does **not** override. `--semantic` + contentless sets `embedding.status=skipped`, `detail=contentless_query`, `endpoint=None` (no provider call). Apostrophe fragments stay T217 per-token (`"can't"` contentless; `"i'll"` / `"don't"` contentful). `forget --match` stays unfiltered (all-stopword still MATCH). Piped `recall -` with empty/whitespace stdin is this empty envelope (exit **0**); TTY `recall -` still refuses. |
| **Pin-type re-rank (T211/T215/T274/T285)** | After blend/graph and **before** truncate, hits are re-ranked by a **single composite** (`rerank_hits` / `rerank_hits_with_query` — the only post-blend ranking entry point; T215 extends it via ScoreKind, does not add a second final sort). Kind boosts: CONSTRAINT **+4**, DECISION **+2**, HOTSPOT **+0.5**; plan-class DECISION **−3** (optional sibling-track **−2**); shipped DECISION **+1**; T70 stub **−16** composite (`SYMBOL_PENALTY`, when included via `--symbols` or a leak); session-chrome **−16** (`SESSION_CHROME_PENALTY`) for closed-list dumps (`## Objective`, `# Track Plan Review`, `# AI-Brains Session Onboarding`, `# Review of Track`, …). Kind markers are **leading-line only after pin envelope** (strip one `USER:`/`ASSISTANT:`/`SYSTEM:`, skip a `TAGS:` line; buried `decision:` / JSON keys are Other; leading `INVARIANT:` is Constraint). Authority first-line query-token overlap **+16** (`LEADING_QUERY_BONUS`) inside the same sort. Lexical MATCH is **two-pass**: authority GLOB **or** `TAGS:` envelope first, in-memory retain, recency retry when retain is empty, then recency/BM25 fill (session dumps stay recallable; they do not occupy the whole `candidate_depth`). Near-duplicate chrome sharing a first line collapses after rerank. Sort: effective desc → `updated_at` desc → `memory_id` asc. Content heuristics only — not lifecycle fact (badge `plan/stale?`). No new Recall JSON keys. |
| **Graph boost** | Neighbor score boost (`--graph-boost`); expansion is **hop-1 by default** (including lexical, not only `--semantic`). Chrome-shaped parents do not seed neighbors (T260 stubs + T285 session chrome). |
| **Substring fallback** | When FTS empty on small vaults (**contentful** misses only). Contentless queries never COUNT/LIKE (T261). |
| **Scope** | Project default (`AI_BRAINS_PROJECT_ID` / flags); `--global` widens to all projects (never auto-widens on empty). **Pretty** recall (empty **and** non-empty), **sync query** vault section, **preflight `--summary`**, and **full preflight human/pretty body (T219)** print a `Scope:` line (`global` or active `project=<alias-or-name> (<uuid>)` / `project=<uuid>` / `project=(none)`). Under `--global`, pretty recall/sync vault, summary, and full body always show `Scope: global` even if env `AI_BRAINS_PROJECT_ID` is set. Non-empty recall order: Scope → Session → Embedding? → hits (no blank between Scope/Session). JSON paths do **not** add a `scope` field. **T276 prefer-fill:** `--global` still means all projects (no leftover drop). When a pre-clear effective project exists, a second lexical MATCH scoped to that project is merged **ahead** of the unscoped window so cwd unique pins can enter `candidate_depth`. Pretty `--global` hit lines lead with a T264-class `[`+8 hex+`]` (or upgraded `display_label`); project-scoped pretty stays unlabeled. JSON `RecallResult` does **not** grow `project_id`. Leftover path split remains `project rebind-path` (memories stay). |
| **Bridge mix** | Ledgerful hits capped so vault memories still surface; `--no-bridge`. Bridge is **outside** RRF (merged after fuse when `--semantic`, first when not); bridge wins on `memory_id` collision. |
| **Formats** | Pretty on TTY by default; JSON / NDJSON; per-result `session_id`; optional JSON `staleness: "plan"` when demoted; additive `score_kind` + optional `cosine` (T218). Under `--semantic` after fuse, JSON `score` is RRF rank contribution (interpret via `score_kind` + `cosine`), not raw BM25 and not a fake confidence. |
| **Hints** | Empty **pretty** always prints next-action text on stdout (**not TTY-only**), after Scope (and Session only when user `--session` / `--session-prefix` / `--session-last` resolved — generated graph-provenance sessions are omitted on empty). JSON empty still sets `hint` + `effective_session_id` (exit **0**). Next-action only when embedding status already explains cause (T202). `--quiet` does not suppress Scope or the empty hint. **T217:** when default lexical is empty, raw token count ≥ 3, and ≥1 contentful token remains, append plain-text “try fewer keywords” (via core token helpers; not for all-stopword queries; no emoji). |

### Briefing + progressive query (T202 / T227)
```powershell
ai-brains briefing project --project-id <uuid> --format human
ai-brains briefing personal --format human
ai-brains briefing project --project-id <uuid> --format json
ai-brains query progressive "why was graph backend replaced?" --project-id <uuid>
```

| Feature | Detail |
|---------|--------|
| **Briefing format (T227)** | TTY default **markdown**; non-TTY **json**. Explicit aliases `human`, `pretty`, `text`, `markdown`, `md` → **markdown**; only `json` → JSON. Unknown `--format` → **exit 2** (`fail_usage`) + accepted list on stderr, **zero stdout**. Dogfood/scripts should pass `--format json` explicitly. |
| **Dual model (pins ≠ authority)** | Preflight `--summary` pin/marker counts and legacy MemoryPinned memories are **orientation only** — they are **not** injected into briefing authority (`decisions[]` / `conclusions[]`). Briefing is a governed authority probe (Approved decisions + Active/Confirmed conclusions + grants). **T288:** granted-empty CLI `briefing project` adds a display-only **Vault pins (not Approved)** stanza (`Pinned: N` + up to 3 leading-line DECISION/CONSTRAINT previews) and optional JSON extras `vault_pin_count` / `vault_pin_previews`; authority arrays stay empty; not an injection surface. |
| **Empty honesty (T227 / T263)** | Allowed (`!denied`) empty project → `empty_authority` warning + markdown next-step naming **`recall` / `search`** (vault pins are not Approved). Allowed empty personal continuity → `empty_continuity` warning + next-step (**no** synthetic summary). **Never** emit empty_* when `denied=true`. |
| **Denied packets** | Project briefing: `denied=true` seeds `warnings[]` `kind: "denied"` + bootstrap next-step (`policy bootstrap`); exit **0**. **T275:** Denied human is a **grant wall** (not `_None_` empty vault); pins remain via `recall` / `search`. After CLI `policy bootstrap` (System default — omit `--principal-id`), `briefing project` is `denied: false` and `evidence list` exits **0**. Personal briefing deny names **`recall`** (optional continuity; not a required bootstrap); exit **0**. **T289:** Denied Personal human omits `_None_` under Preferences/Continuity and uses `_(optional continuity; not a missing vault)_` — not empty prefs / not a required Personal bootstrap. JSON stays `denied: true` + empty arrays (no new keys). Allowed-empty Personal still `_None_`. |
| **Progressive / expand** | Require project id (`--project-id` or `AI_BRAINS_PROJECT_ID`); missing → exit **2** + copy-paste example on stderr. **T221 honesty:** progressive policy wall → exit **3** with pretty packet still on stdout (`denied`, `denial_hint` bootstrap); expand `kind: Denied` → exit **3** (capability and/or cross-scope); expand `Unknown` → exit **0** with non-empty `preview`. First-run: `policy bootstrap --dry-run` then `policy bootstrap` (omit `--scope` when project context is authoritative; `--scope Repository:<uuid>` remains valid for no-context CI). System principal when `--principal-id` omitted. |
| **Trace** | No project-id gate; missing/unauthorized → JSON envelope (`found: false` + `next_step` copy-paste `query progressive … --dry-run false`) exit **0**. `--format human` two lines (`No trace` + `next:`). Found stays `QueryTraceDto` JSON. |

### Governed discovery lists (T203)
```powershell
ai-brains source list --scope Repository:<uuid> --format json
ai-brains source list --format json   # soft-fill when AI_BRAINS_PROJECT_ID / context is authoritative
ai-brains evidence list --scope Repository:<uuid>
ai-brains evidence list --query keyword --scope Repository:<uuid>
ai-brains evidence search --query keyword --scope Repository:<uuid>
ai-brains review list --format json   # soft-default scope (authoritative) or fail_usage exit 2
```

| Feature | Detail |
|---------|--------|
| **Commands** | `source list`, `evidence list` (optional `--query` FTS), `evidence search` (requires `--query`) |
| **Bounds** | Default limit **50**, hard clamp **200**; `more_available` via LIMIT+1 |
| **Empty** | E1 `items: []` (never null); human `(none)`; exit **0** when policy allows; authorized-empty JSON `next_step` is copy-paste `ai-brains recall "what did we decide"` plus `(Pinned: N)` when local COUNT succeeds (T290; vault pins are not governed evidence). Human prints that same next line after `(none)`. |
| **Policy** | `ReadEvidence` for source/evidence list; `ReadConclusions` for review list; deny → exit **3** + `details.hint` (bootstrap first) |
| **Bootstrap (T210)** | `ai-brains policy bootstrap [--scope …] [--dry-run]` issues discovery grants only (`ReadEvidence`, `ReadConclusions`, `ReadDecisions`, `LocalOnly`); registers principal if missing; idempotent via `active_grants`; no auto-init |
| **Soft-resolve** | Omitted `--scope` fills only when `scope resolve` is authoritative; else **exit 2** `fail_usage` (never exit **6**) |
| **Show** | `source show` / `evidence show` use the same soft-resolve helper |
| **Policy show/check (T226 + T241)** | `policy show` and `policy check` soft-resolve omitted `--scope` like discovery lists / bootstrap; always canonicalize via `parse_scope_key` → `scope_identity_key`. **T241:** `--capability` on check is optional at clap; omit → exit **2** discovery-first catalog via `fail_usage` (not clap required-arg English). Empty `policy show` prints bootstrap SOOT / JSON `next_step`. Erasure / `review resolve` stay clap-required. |
| **Status filter** | Default Active-only on source/evidence projections |
| **Capture independence** | Projection reads only — no models/embeddings; no control-plane→retrieval dependency |

### Preflight (session-start briefing)
```powershell
ai-brains preflight --summary
ai-brains preflight --summary --format json
ai-brains preflight --global --summary --format json
ai-brains preflight --pretty -m 1500
ai-brains preflight --pretty --compact
ai-brains preflight --scope "src/foo.rs" --global
ai-brains preflight --stdin
```
Synthesizes repo safety/hotspots, session turns, memory index, recent dense memories, under a word budget (default 1500). Index titles use Unicode-safe truncation.

| Feature | Detail |
|---------|--------|
| **Scope honesty (T214)** | `--summary` prints T207 `Scope:` vocabulary (`Scope: global` / `Scope: project=…` / `Scope: project=(none)`). Never labels multi-project content as a single env `Project: <uuid>`. |
| **Full-body Scope (T219)** | Human/pretty full body (not summary) also prefixes the same T207/T214 `Scope:` line + blank line before the budget-window body. **JSON path does not** add Scope chrome. |
| **Pretty multi-line body (T219)** | Human/`--pretty` / `--format human\|pretty` preserves section newlines (word budget no longer space-joins). Blank line after each emitted `--- Section ---` header. Display-only caps: safety **8** items, sessions **3**, turns/session **6**, Memory Index **15**; orphan empty `---` headers omitted. |
| **Pretty density (T250)** | Default human/`--pretty` line-caps **Session** turns and **Most Recent Memories** at **140** Unicode chars (`…` when truncated). T219 item counts stay **8** / **6** / **3** / **15** / recent **3**. Safety, Memory Index, `---` headers, and `+N` notices are **not** line-capped on the default path. Default `--pretty` still shows full Safety lines (~150–220); only `--compact` first-line-caps Safety. Governed `#`/`##` still not treated as `---` headers; Other/governed body lines are not line-capped. |
| **`--compact` (T250)** | Tighter display caps: safety **3** / turns **2** / sessions **1** / index **5** / recent **2**. First-line-only on Safety/Recent; line-cap **100** on Session + Recent + Safety first line. F31 `+N` notices still fire with compact N. Compact Recent keeps the `(Use 'recall'…)` hint. |
| **JSON / `--summary` ignore `--compact`** | `--compact --format json` stays compact `{text, word_count, sections}` with uncapped `text` (T180 required keys + T265 `sections`). `--summary --compact` stays the T214 summary banner. |
| **Role strip display-only (T219/T224)** | Pretty path strips leading case-sensitive `USER:` / `ASSISTANT:` / `SYSTEM:` on index/session/display lines (shared `strip_role_prefix`; same SOOT as `memory list` preview, recall/`sync query` pretty hits, and forget human previews). Stored vault content and non-summary JSON `text` may still embed role labels where assembly emits them. |
| **Per-section +N notices (T219 F31)** | Overflow notices (plain ASCII): safety → `+N more safety entries — ai-brains memory list`; index → `+N more via recall`; turns → `+N more turns in session`; sessions → `+N more sessions`. |
| **Word budget + F2b (T219)** | `trim_to_word_budget` preserves newlines; over-budget appends trailing `…` on its own line (content `word_count` excludes the sentinel). Applies to full pretty body, non-summary JSON `text`, and governed markdown re-budget (same helper). |
| **Governed F1 benefit (T219)** | When governed rendering is on, `#` / `##` markdown survives the pretty formatter (not treated as `---` section headers). F1/F2b still apply to the governed string. No governed section caps in v1. |
| **Dual count model** | **Vault (SQL):** under `--global`, `Projects:` = distinct projects with pinned memories; always `Pinned memories` + `Active sessions` (SQL on projections, capture-independent). **In context:** `HOTSPOT:` / `DECISION:` / `CONSTRAINT:` counts from the budget-window text only — labeled `In context …` so they are not read as vault totals. |
| **Global isolation (T264)** | `--global` stays a vault rollup, not this-repo law. Retrieval prefixes Safety / Index / Recent **item first lines** and Session **headers** with `[` + 8-hex project id + `]` (`[unknown]` when missing). Human/pretty upgrades that leading tag via `display_label` (truncate 32, `]` → `·`). Per-project caps after recency: Safety **2**/vault **8**, Index **3**/vault **15**, Recent **1**/vault **3**, Sessions **1**/project. Index/Recent skip the **emitted** Safety ids only (post HOTSPOT-suppress + path-dedup + round-robin); a CONSTRAINT that lost a Safety slot can still appear in Index (T272). Summary adds `In context spans N projects` (SQL identity of emitted items; may disagree with marker counts). Summary JSON optional `in_context_project_span` (global only). Project-scoped pretty is unlabeled. Does **not** drop leftover from `recall --global`. **T276:** pretty `recall` / `search` / `sync query` `--global` hit lines use the same tag grammar; prefer-fill is ranking/chrome, not a filter. |
| **Active sessions** | Rollup uses `session_projection` status=`active` (not a missing text marker). |
| **Ledgerful hotspots (T279)** | Pretty **Repository Bearings & Safety** is live `ledgerful hotspots --json --limit 5` (`HOTSPOT: <path> score=n`, project-scoped) plus leading-line GLOB `CONSTRAINT:` / `INVARIANT:` / `HOTSPOT:` (not LIKE-anywhere; dumps with buried markers do not steal the section). `--global` does **not** live-inject (cwd scan is one repo). Honest empty still emits the Safety header and `next: ai-brains safety sync --dry-run`. `safety sync` without `--dry-run` pins; preflight does not. Bridge Intelligence (`bridge export --hotspots`) is a separate section and stays off under `--global`. |
| **Memory Index two-pass (T274/T286)** | Index pass-1 is leading-marker GLOB **or** TAGS envelope (`index_pass1_glob_sql`: `DECISION:`/`CONSTRAINT:`/`INVARIANT:`/`HOTSPOT:` **or** `TAGS:` / `ASSISTANT: TAGS:`). Pass-2 recency-fills other injectable rows (dumps may appear as items 2+). Numbered titles use `first_contentful_line` (skip one role prefix + a `TAGS:` line); empty / role-only / TAGS-only → `Untitled Memory`. Untagged `DECISION:` still wins (T274). Summary `In context decisions` stays a window substring of `DECISION:` (no new JSON key). |
| **Governed authority** | `--summary` (human or JSON) is orientation only (T170 D21). Use full `preflight --format json` / `briefing` for governed packet truth — never treat summary counts as authority. |
| **Full preflight JSON (T180/T265)** | Non-summary `--format json` is compact `{text, word_count, sections}`. Required: `text` (string), `word_count` (usize). Additive always-present `sections` (array; E1 empty `[]`, never `null`, never omitted). Each section is `{id, title, items}` with `id` one of `safety` \| `session` \| `index` \| `recent` \| `ledgerful` \| `empty_repo` \| `governed` \| `other` (string, not a rust enum on the wire). Split from the same `text` (T219 newlines + optional F2b `…`; no Scope/caps chrome). Compact `to_string`. `--summary --format json` stays T220 (no `sections` key). |
| **Summary JSON (T220)** | `--summary --format json` (case-insensitive) emits a **pretty** machine object on stdout (no human banner). Keys: `api_version` (`"1"`), `scope` (`"global"` \| `"project"` \| `"none"`), `project_id` (uuid string or `null`), `projects` (**only when `scope=="global"`**; omitted under project/none — never `null`), `pinned`, `active_sessions`, `in_context_hotspots` / `in_context_decisions` / `in_context_constraints`, `word_count`. T264 additive: optional `in_context_project_span` (**only when `scope=="global"`**; omitted under project/none). |
| **Summary `scope` three-valued** | `--global` → `"global"` (`project_id: null`, include `projects`); resolved project → `"project"`; unresolved (no global, no project id) → `"none"` (`project_id: null`, omit `projects`). Under `"none"`, vault SQL counts are vault-wide (same as human `Scope: project=(none)` honesty). |
| **Summary `word_count`** | Full preflight **budget-window** text size (`context.word_count`), **not** the byte/size of the summary JSON payload (parity with human `Total Word Count:`). |
| **Summary `in_context_*`** | Case-sensitive marker scan of rendered budget text (`HOTSPOT:` / `DECISION:` / `CONSTRAINT:`). Under governed rendering those markers may be absent → counts can be **0** (orientation only; not governed claim authority). |
| **Summary + install-hooks** | `--install-hooks` still runs side effects on the JSON path; install status lines go to **stderr** so stdout stays one pure JSON document. No interactive install prompt on the JSON path. |

### Unified vault + ledger search
```powershell
ai-brains sync query "rust" --format pretty
ai-brains sync query "term" --no-bridge --global --quiet
ai-brains sync query "path TOCTOU" --limit 5 --format pretty
```

| Feature | Detail |
|---------|--------|
| **When to use** | Human vault **+ Ledgerful ledger** in one view. Agent/JSON → `recall`. Decision table: [§15](#15-typical-agent-workflows). |
| **Always-pretty default (T231 F33)** | Default format is **pretty** even non-TTY (intentional human-first). Machines use `recall` JSON or explicit `--format ndjson`. `--format text` ≡ pretty path. |
| **Project resolve (T231 F32)** | Missing/invalid/whitespace `AI_BRAINS_PROJECT_ID` → `Scope: project=(none)` vault-wide — **never** random UUID. `--global` → `Scope: global`. NDJSON `project_id` field is `""` when none. |
| **Vault re-rank** | Same `recall_full` + `rerank_hits` path as `recall` (pin-type authority, plan demotion, recency). Default vault **`--limit` / `-l` = 5**. |
| **Plan badge** | Demoted plan-class DECISION lines show **`[plan/stale?]`** before content (content heuristic ≠ governed lifecycle). |
| **Ledger-first** | When not `--no-bridge`, vault is recalled first; if ledger JSON probe is non-empty **and** the top vault hit is plan-class, prints banner `Note: vault top hit is plan/stale; ledger results shown first.` then **ledger section before vault**. Miss/fail does **not** reorder (T211) and does **not** blank vault; non-quiet prints a named miss line (T271). |
| **Ledger pane (T271)** | Ledgerful `ledger search` is a **phrase** search (it wraps the whole argv). The CLI forwards `strip_ansi(query).trim()` — **not** T90 FTS AND quotes. The spawn always inserts POSIX `--` immediately before QUERY so dash-leading needles (`--limit`, `--days`) are search text, not Ledgerful flags (T273; operator form: `sync query -- --limit`). Multi-token phrase misses retry up to 3 first-seen contentful tokens; first hit wins with `Note: no phrase match for '<user>'; showing hits for '<token>'.` Named misses: never-ran / failed / ran-empty (user query, never `'"tok" "tok"'`). `--no-bridge` skips the pane. `--quiet` omits never-ran/failed (not ran-empty/hits). |
| **Honesty** | Re-rank is presentation only — pins are never auto-forgotten or mutated. **`rerank_hits` is the single post-blend ranking entry point** (F40); T215 extends it via ScoreKind / hybrid RRF rather than adding a second final sort. `sync query` stays `semantic: false` (lexical + bridge ranking only). |

---


## 8. Nightly intelligence (“Brain mode”)

```powershell
ai-brains nightly
ai-brains nightly --status
ai-brains nightly --status --quick
ai-brains nightly --status --format json
ai-brains nightly --skip-import
ai-brains nightly --skip-import-grok
ai-brains nightly --schedule --start-time "03:00"
ai-brains nightly --schedule --run-as-system --dry-run
```

Pipeline includes:
1. **Multi-harness session import (T239)** — fixed order **agy → grok → opencode** (message-only adapters; never `opencode.db`). `--skip-import` skips all three; per-source `--skip-import-agy` / `--skip-import-grok` / `--skip-import-opencode`. Fail-open per source with **per-source sinks**. Report persisted as `last_multi_import` (`v:1`); `nightly --status` prints a Multi-import block (missing → `never`; corrupt → `unreadable`; OpenCode `list_capped > 0` surfaces a cap warning). **Claude / Codex are not in the nightly batch** (T253 live+manual import only). Import progress may interleave non-JSON on stderr under SYSTEM `--log-format json` (accepted).
2. **Soft model-endpoint probe (T229)** — after multi-import, before summarize; 2s `/health` then `/v1/models`; non-fatal warn if completion (`:8081`) or embedding (`:8083`) is down
3. Session summarization (chunked; **38,912-token** context with carryover)
4. Memory synthesis (batch-limited, e.g. 50 memories/run)
5. Embedding backfill + stale refresh + WAL checkpoint (UTF-8-safe truncate — T229 F5)
6. **Phase 2 multi-root bridge (T233/T254)** — for each registered path alias (sorted ASC; optional `AI_BRAINS_NIGHTLY_MAX_ROOTS`): MADR export + `ledgerful symbols --pub --json --limit N --auto-index` with **explicit root** (`current_dir`). Zero aliases → no-op + `register-path` hint. Per-root failures warn + continue. Logs `bridge_roots_total/ok/skipped/failed` (`ok + skipped + failed` accounts for every considered root; missing = skipped; symbol `Err` = failed; MADR-fail + symbols-ok = ok). Symbol source_tag `ledgerful:symbol` (dual-read legacy `changeguard:symbol`). Cap default **5000** (`AI_BRAINS_NIGHTLY_MAX_SYMBOLS`). No SQL open of `.ledgerful/state/ledger.db`; no System32 cwd dependence. Route catalog remains `ledgerful endpoints` (not symbols).
7. **`MemorySynthesized`** events for graph edges
8. Live graph projection updates

**`nightly --status` honesty (T247):**

- **Endpoints:** Completion + Embedding **host:port** + model names (env defaults `127.0.0.1:8081` / `:8083`; `user:pass@` redacted; never vault keys)
- **Soft probe:** `probe=ok|down|timeout|error` on default `--status`; **`--quick` prints `probe=skipped`** (string; no HTTP; no `ProbeStatus::Skipped`)
- Default probes **parallel 750 ms**; run-path pre-summarize still **2s**
- **Last task result** via **LIST /V primary** (CSV is next-run fallback only; never col 5). Decode **1** vs **101** vs Event ID **101**. Hint is a following line.
- **Last scheduled run** (scheduler) vs vault **Last nightly run** both printed
- Missing action target (quoted `.cmd` / `.bat` / `.exe` that does not exist) + `next: ai-brains nightly --schedule --dry-run`
- Multi-import block (T239) unchanged
- Status **exit 0** when down / timeout / missing action / nonzero last result
- **JSON / Router (T255):** `--format json` emits a CLI-local machine object. Default `--format human`; piped `nightly --status` stays human. Additive read-only `Router:` Last Result line — does **not** register, start, or repair `AI-Brains-Router`. `doctor` remains the frozen **15**-check matrix (not the model-port matrix)
- **Nightly vs Router (T269 / T281 / T296):** human prints `Nightly: AI-Brains-Nightly` before the schedule block so Last Result **0** is not the same object as Router. Human Router omits `267014` / `267009` decimals: `Ready` + `last run: terminated` (or Status-only when Running). JSON still carries raw `router.last_result` + `SCHED_S_*` hints. Human `probe=timeout` is labeled `probe=timeout (750ms)` (HTTP `/health` within 750 ms). On Completion human timeout, the next line is `HTTP /health 750ms ≠ daemon TCP`. `--quick` stays `probe=skipped` (no contrast). JSON probe tokens and the 750 ms budget are unchanged. `daemon status` Open remains TCP connect, not `/health`

SYSTEM-mode schedules bake vault/model env into a wrapper script so Session 0 has config (global dotenv gap-fill T205). **SYSTEM keeps `--skip-import` by default** — completeness path is user-principal `nightly --schedule` or manual `nightly` (not Session 0 import). See [OPERATIONS.md](OPERATIONS.md) dual-path table + local router (`c:\llm\router.bat` / `AI-Brains-Router`).

---

## 9. Graph

Requires a build with **`--features graph`** for the live backend. Recommended source install (INSTALL SOOT):

```powershell
cargo install --path crates/ai-brains-cli --locked --features graph
```

**Feature-off honesty (T198):** on default / slim / GitHub Release `ai-brains.exe` builds (no graph feature), every `ai-brains graph *` subcommand exits **2** with a `FEATURE_UNAVAILABLE:` prefix and a reinstall hint for the command above. `graph --help` remains exit **0**. GitHub Release `ai-brains.exe` is currently graph-off; see [INSTALL.md](INSTALL.md).

| Command | Purpose |
|---------|---------|
| `graph update` | Health report: default remains **pretty JSON** (T213 keys: `nodes`, `edges`, `pinned_memories`, `memory_nodes`, `edge_node_ratio`, `density` (`ok`\|`warn`\|`skip`), `status` (`live`\|`sparse`\|`empty`), `note`, optional `remediation`). Opt-in `--format human` prints labeled lines. `--format auto` does **not** TTY-switch (stays JSON). Feature-off still exit **2**. |
| `graph rebuild` | Full resync (recovery). T246 does **not** rebuild automatically. |
| `graph neighbors <memory_id>` | 1-hop neighbors. TTY pretty human (or `--format pretty`/`human`/`text`); **compact** JSON when piped or `--format json`. `--limit`/`-l` (pretty default 50 max 200; JSON unlimited unless `--limit` given). Pretty columns DIR/LABEL/ID/KIND/PREVIEW: `memory` is the content line; `session` is `{n} memories · first line` (T278); other kinds stay blank. T293: human prefer-fills authority 1-hop (memory/session leading `DECISION:`/`CONSTRAINT:`/`INVARIANT:`/`HOTSPOT:`) before dump sessions; JSON order unchanged (direction→label→id). The id printed by `pin` is this memory id (and the graph node) after T262. Empty pretty: no graph node + vault has the memory → `next: ai-brains graph rebuild`; unknown id → `(not a vault memory id)` with **no** remediator; present-but-empty edges have **no** remediator. Never `graph update` for a missing node (`update` is health-only). JSON keys unchanged. |
| `graph hierarchy <memory_id>` | Synthesis chain. Same TTY/pipe `--format` / `--limit` as neighbors. Empty pretty distinguishes no graph node (same missing-node next as neighbors) vs honest leaf (`No SYNTHESIZED_FROM children`, no remediator). JSON keys unchanged. |
| `graph session <session_id>` | Memories in a session. Same TTY/pipe `--format` / `--limit` as neighbors. Empty pretty: no graph node + vault has the session (`session_projection`) or a memory with that id → `next: ai-brains graph rebuild`; unknown id → `(not a vault memory id)` with **no** remediator; session node present but no memories has **no** remediator. JSON keys unchanged. |

> **Feature-off:** all rows above exit **2** + `FEATURE_UNAVAILABLE` when the binary was built without `--features graph`.

**Live Graph Hook:** incremental projection on each event append (graph-on builds only).

**Cozo / bridge lifecycle noise (T208):** on graph-on builds, Cozo proxy init is **quiet by default** (debug-level + default EnvFilter `ai_brains_graph=warn`). Normal `recall` / `sync query` / pin-style paths must not print `CozoProxyBackend initialized` under an unset `RUST_LOG`. Escape hatch for operators debugging graph/Ledgerful bridge availability:

```powershell
$env:RUST_LOG = 'ai_brains_graph=debug'   # only — =info will not show init after demote
ai-brains recall "q" --format pretty --limit 1
```

`--log-format off` still silences all tracing output (including debug).

---

## 10. Ledgerful integration

| Feature | AI-Brains side |
|---------|----------------|
| Binary | Shells out to `ledgerful` |
| State dirs | `.ledgerful/` preferred; `.changeguard/` legacy |
| Hotspot pin | `safety sync [--limit N] [--dry-run]` |
| Symbol bridge | Nightly ingest → recall returns code structure |
| Unified query | `sync query` |
| NDJSON pull/push | `sync pull` / `sync push` |
| IPC pipe | `\\.\pipe\ledgerful-bridge` |
| Opt-in bridge | Ledgerful default **off**; enable `[bridge] enabled=true` or `LEDGERFUL_BRIDGE=1` |

Explicit `ledgerful bridge export` / `import` remain pure-local without opt-in. Implicit push/IPC paths require opt-in on the Ledgerful side.

---

## 11. Backup & hygiene

```powershell
ai-brains backup
ai-brains backup create --output-dir D:\backups --dry-run
ai-brains backup list [--quiet] [--verbose]
ai-brains backup verify [--full] [--verbose] [--format json]
ai-brains backup prune --keep N --older-than <dur>
ai-brains backup restore <path> [--force] [--dry-run]
ai-brains recovery export --output <path> [--passphrase-file] [--dry-run] [--force|--overwrite]
```
Backup suite with metadata headers, integrity checks, and restore **hard-fail** when the daemon/service is reachable via robust IPC probe (T188; `--force` never overrides). SQLCipher-encrypted vaults and backups (T187). Default retention keeps 10 backups. Plain→encrypted migrate: `ai-brains vault encrypt` (`sqlcipher_export`).

**Backup list honesty (T209 / T244):** each `vault-*.db.bak` is classified under the current vault key. After the key opens, **both** product core tables (`events` + `memory_projection`) are required before meta classification (T244 F1).

| Class | Key / cores / meta | Table token (when meta empty) | Default noise |
|-------|--------------------|-------------------------------|---------------|
| Readable | key opens + both cores + meta OK | real metadata values | silent |
| Pre-T109 | key opens + both cores + meta unusable/absent | `(no metadata)` | debug only |
| Incomplete | key opens + **missing** core tables | `(no core tables)` | debug + residual summary |
| Legacy plain | SQLite magic header | `(legacy plain)` | debug + residual summary |
| Key mismatch | size ≥ 512, not plain, key fails | `(unreadable key)` | debug + residual summary |
| Corrupt | short/unreadable garbage | `(corrupt)` | per-file `WARN` |

**Usable / class decision table (T244):**

| Key opens | Core tables (both) | Meta OK | Class | Doctor usable | Verify (integrity ok) |
|-----------|--------------------|---------|-------|---------------|------------------------|
| n/a plain header | — | — | LegacyPlain | no | FAIL legacy plain |
| no | — | — | KeyMismatch / Corrupt | no | FAIL key |
| yes | no | — | **Incomplete** | **no** | FAIL missing core tables |
| yes | yes | no | PreT109 | **yes** | OK if integrity passes |
| yes | yes | yes | Readable | **yes** | OK if integrity passes |

**Usable SOOT:** doctor `backup_recent` and list “usable” = `Readable \| PreT109` only (`is_usable_class`). Incomplete is never usable.

Default list prints one **stderr** summary when any non-usable residual exists (`not recoverable under current key` … `--verbose` … `verify`), counting legacy plain / incomplete / key / corrupt. CLI list sorts **usable-first** (then newest timestamp; unparseable last within band); brain `list_backups` stays timestamp-desc for doctor. `--verbose` adds per-file detail and omits the summary. `--quiet` suppresses summary and metadata WARNs (table tokens still apply). Dual `--quiet --verbose` → quiet wins. Exit **0** for any mix of classes after a successful scan.

**Recoverability green path:** when doctor warns no usable encrypted backup (or fleet is all legacy/incomplete), run `ai-brains backup create --no-prune` (keep residual KeyMismatch / legacy / incomplete files; omit `--no-prune` only when default keep-10 prune is OK) then `ai-brains backup verify` and expect ≥1 OK under the current key. Doctor-usable means the vault sibling `backups/` directory — custom `--output-dir` is a manual export and is **not** scanned by `backup list` (default dir) or doctor `backup_recent`.

**Current-key create after KEY change (T277):** changing `AI_BRAINS_KEY` does **not** rewrite old `.bak` files — they stay KeyMismatch (`(unreadable key)`). Do not transcode. Create a **new** snapshot under the current key (`--no-prune` if residuals must be kept). Exhibit: `vault-2026-08-12T15-50-06.db.bak` (T244 live create; later KeyMismatch). Create fail-closes after write: `classify_backup_read` must be doctor-usable (`Readable` / `PreT109`); Incomplete / missing cores is deleted and never printed as `Backup created and verified:`. A FAIL-heavy residual fleet with **≥1 OK** is success (verify still exits **1**).

**Backup verify quiet default (T225 / T244):** human `backup verify` prints **counts** (`Verified N backup(s): X OK, Y FAIL.`) plus the first **5** `filename: FAIL — {reason}` lines (T138 reason preserved); OK lines omitted. Verify requires **both** core tables (`missing core tables` when fewer than two); JSON `tables` still lists whichever of `events`/`memory_projection` were found. When fail &gt; 5, a trailer points at `--verbose`. When `ok == 0 && total >= 1`, a create nudge includes `ai-brains backup create`. **`--verbose`** = full per-file OK/FAIL stream only (no summary, trailer, or nudge). **JSON** always returns full `results[]` (verbose ignored). Progress tracing is `debug` (no INFO flood under product default `RUST_LOG`). Any FAIL → exit **1**; empty → exit **0** (`No backups to verify.`).

**Recovery export (T188 / T194):** writes RecoveryKit JSON (`schema_version: 1`) to a restricted file path only (never kit JSON on stdout). Passphrase via file or zero-echo TTY (`rpassword`). Kits embed Argon2id params in `passphrase.kdf` (algorithm=argon2id, version=19, m=19456, t=2, p=1); pre-T194 kits without `kdf` dual-read fixed legacy constants.

**Doctor (T192):** `ai-brains doctor` is a **read-only** operator health surface.

```text
ai-brains doctor
  [--format human|json]           # default human (does not TTY-switch)
  [--json]                        # force JSON (overrides --format)
  [--summary]                     # opt-in compact of the same 15-check report
  [--fail-on-degraded]            # exit 1 when status=degraded
  [--kit-path <path>] [--passphrase-file <path>]
  [--backup-max-age <Nd|Nh|Nw>]   # default 7d
  [--full]                        # PRAGMA integrity_check
```

Check matrix (fixed order, **15** checks): `vault_exists`, `vault_open` (`open_read_intent` only — never migrates), `schema_readable`, `cipher_page`, `daemon_reachable` (info: up/down never fails alone), **`backup_recent`** (soft; T225/T244 class-aware: usable = Readable \| PreT109 only via `is_usable_class` — Incomplete / plain / key / corrupt never usable; no usable encrypted under current key → warn + `ai-brains backup create` only even if Incomplete/plain timestamps are recent; otherwise ages **newest usable** vs `--backup-max-age`, default 7d), `recovery_kit_event` (soft; event ≠ offline file proof), `recovery_kit_file` (hard when `--kit-path` set; skip otherwise — no default kit path search), `zero_key_escape` (soft / R-ZERO-KEY), **`graph_feature`** (soft info: `available`|`unavailable` via compile-time `cfg!(feature = "graph")`; remediation = INSTALL primary SOOT when unavailable; never alone fail/degraded), **`graph_density`** (soft; SQL counts only — warn on empty/sparse/orphan/projection lag; skip when tables missing / open failed / pinned count failed / small empty vault; never alone forces fail; capture-independent even on graph-off binaries; T232 capability-aware remediation: graph-on → `ai-brains graph rebuild`, graph-off → `GRAPH_REINSTALL_SOOT`), **`harness_wiring`** (soft ok — T245+T253 ready-missing only when Claude/Codex are install_ready; no T253 pending clause; next is `ai-brains harness install --harness all-ready --dry-run`; never alone fail/degraded), **`project_identity`** (soft; T240 — warn when env `PROJECT_ID` ≠ path-alias owner of cwd/git toplevel when both present; remediation → `project whoami`; never alone forces fail; read-only vault_conn), **`policy_grants`** (soft; T241 — after `project_identity`; when vault open + authoritative project scope, probe discovery ReadEvidence/ReadConclusions/ReadDecisions for default CLI principal; **warn** when active_count < 3 with long bootstrap remediation; **ok** when 3 of 3; **skip** when vault closed / no authoritative scope / list error; never alone forces Fail — warn → Degraded only; StorePorts-only, no AppContext; cwd/`AI_BRAINS_PROJECT_ID` coupled), `integrity` (only with `--full`). Overall: fail ≻ degraded ≻ ok. Exit 0 for ok|degraded (default); 1 for fail; clap usage 2. Never creates vault or `backups/`; never prints secrets. Residual: offline kit without `--kit-path` remains operator responsibility (see RECOVERY-DRILLS).

---

## 12. Privacy & crypto

- Privacy levels from cloud-ok through sealed; pins default to **`LocalOnly`**
- Preflight/recall filter non-injectable / sealed content
- Vault open with SQLCipher page encryption + Content Envelope — [COMPATIBILITY.md](COMPATIBILITY.md) F8 / T187; busy timeout under concurrent CLI access
- Key via `AI_BRAINS_KEY` / crypto recovery path; `zeroize` for secrets

---

## 13. Models & local AI routing

| Variable | Role |
|----------|------|
| `AI_BRAINS_MODEL_URL` | Completion (default `http://127.0.0.1:8081`) |
| `AI_BRAINS_COMPLETION_MODEL` | Model name |
| `AI_BRAINS_EMBEDDING_URL` | Embeddings (default `http://127.0.0.1:8083`) |
| `AI_BRAINS_EMBEDDING_MODEL` | e.g. `nomic-embed-text-v1.5` |

`daemon status` probes configured host:port with handshake retries. LLM HTTP client uses per-request timeouts.

---

## 14. Configuration hierarchy

CLI gap-fill order (after optional elevation handoff override on elevated child only):

1. CLI flags / process (shell) environment
2. Project-local `.env` when `--no-project-context` is unset (from `context` / cwd)
3. Global `~\.ai-brains\.env` — **always** merged for gaps (`dotenvy` non-override), including under `--no-project-context` (KEY, vault path, model URLs)

---

## 15. Typical agent workflows

| Intent | Command |
|--------|---------|
| Session start | `preflight --summary` / `--pretty` |
| “What did we decide?” | `recall "…" --semantic` |
| Persist a decision | `pin "DECISION: …"` |
| Correct a memory | `forget` / `restore` |
| Sync brittle files | `safety sync` |
| Overnight brain | `nightly` (+ schedule) |
| Hygiene | `backup` · `project list` |

### Start here: which search? (T231 / T243)

Three surfaces, three corpora — not three UIs on one index:

| Intent | Command |
|--------|---------|
| Human, vault only | `recall "…"` / `search "…"` (TTY pretty; `search` is a visible alias of `recall`) |
| Agent / pipe / scripts | `recall "…"` JSON (or `search`) |
| Human, vault **+ ledger** / plan vs shipped | `sync query "…" --format pretty` |
| “What did we decide?” (daily) | **`recall` / `search`** (vault `DECISION:` / `CONSTRAINT:` pins). Briefing / `query progressive` read **only** Approved decisions + Active/Confirmed conclusions — grants do not copy pins into authority. |
| Governed conclusions / decisions | `query progressive "…"` (Approved + Active/Confirmed only; needs discovery grants; **not** vault FTS). Granted-empty → `next_step` is copy-paste `recall` of the operator query plus `(Pinned: N)` when COUNT succeeds (not the U+2026 ellipsis). |
| Embeddings / hybrid | `recall "…" --semantic` (not `sync query`) |
| Machine stream of vault hits | `sync query "…" --format ndjson` **or** `recall --format json` |
| Invalid `AI_BRAINS_PROJECT_ID` | **`recall` / `search`** → clap **exit 2**; **`sync query`** → vault-wide `Scope: project=(none)` exit **0** (F36 — clap env parse vs manual resolve; not converged) |
| `text` format | **`recall` and `sync query`**: `text` ≡ pretty |

`evidence search` is a nested evidence noun, not the top-level `search` alias.

**Project resolve (T231 F32 fix):** `sync query` missing / empty / whitespace / invalid `AI_BRAINS_PROJECT_ID` → `project_id = None` → `Scope: project=(none)` — **never** a random UUID. Valid UUID → scoped. `--global` → `Scope: global` (vault-wide).

**Sync always-pretty (T231 F33 intentional):** `sync query` defaults to **pretty** even when non-TTY (human-first unified pane). Agents that need JSON should use `recall` (non-TTY default) or explicit `sync query --format ndjson`.

End-to-end recipes: [WORKFLOWS.md](WORKFLOWS.md) (“Find something”).

---

## 16. Capability map

```text
CAPTURE          ingest · agy-hook · antigravity-import · daemon queue
CONTEXT          context · project list/resolve/detect/whoami/set-alias/register-path/list-paths/rebind-path/unregister-path/scan-roots · stop-session
DENSE MEMORY     pin · forget/restore · safety sync
RETRIEVAL        recall (FTS · semantic · graph-boost · bridge) · preflight · sync query
INTELLIGENCE     nightly (summarize · embed · synthesize · multi-root Phase2 bridge)
GRAPH            neighbors · hierarchy · session · update · rebuild · live projector
INTEGRATION      Ledgerful (search/hotspots/bridge/pipe) · multi-harness hooks
OPS              init · backup suite · daemon service · schedule · update
PRIVACY/CRYPTO   vault+CE (F8) · privacy levels · path normalization · no CoT
LEGACY IMPORT    classify_legacy / apply_legacy_import (T167) + `migrate governed` CLI (T168)
EVALUATION       `evaluate governed` (T169) + shadow dogfood gate / `dogfood compare` (T170)
```

### Legacy → governed classification import (T167 / P9.1) + migrate CLI (T168)

**Control-plane API** plus operator CLI. Callers (or `ai-brains migrate governed`) supply a legacy event stream + destination ports; the importer never opens the live vault itself.

| API / CLI | Role |
|-----------|------|
| `classify_legacy` | Dry-run plan + `plan_hash` (default); under-promotes pins→Evidence, synth→Candidate, decisions→Proposed+Review |
| `apply_legacy_import` | Confirm-gated append via raw `build_event`; no `observe_source` / no `RecordEvidence` capability |
| `plan_report_json` | Operator report (ids/counts/hash; optional truncated snippets; never full bodies by default) |
| `ai-brains migrate governed` | T168: dry-run differential report; `--confirm` dest materialize + T167 apply; live/reparse refuse; no plaintext in report |

See [OPERATIONS.md](OPERATIONS.md#legacy--governed-classification-import-t167--p91) and [OPERATIONS.md](OPERATIONS.md#governed-migrate-cli-t168--p92) for operator rules (idempotency, forgotten cascade, CE honesty, re-apply, force-overwrite).

### Governed evaluation harness (T169 / P9.3)

| API / CLI | Role |
|-----------|------|
| `ai-brains evaluate governed` | Run scenario corpus; JSON report + exit 0/1/6/7; never mutates live vault |
| CP `evaluation/` | Pure metrics, seed programs 1–9, hermetic runner |
| sources nextest | Scenario 10 circularity hard gates |

See [EVALUATION/GOVERNED-MEMORY-MVP.md](EVALUATION/GOVERNED-MEMORY-MVP.md) and [OPERATIONS.md](OPERATIONS.md#governed-evaluate-cli-t169--p93).

### Shadow dogfood gate (T170 / P9.4)

| API / CLI / script | Role |
|--------------------|------|
| `scripts/dogfood-shadow.ps1` | Stage A evaluate + D24 live hash + shadow/migrate under WorkDir + compare capture; **never** Stage D / User env / `AI_BRAINS_VAULT_PATH`→shadow |
| `ai-brains dogfood compare` | Pure-serde compare packet (`dogfood-compare.json`); fingerprints + `compare_hash`; D15 seed + warning_refs |
| Human checklist | [EVALUATION/templates/dogfood-human-checklist.md](EVALUATION/templates/dogfood-human-checklist.md) |

See [EVALUATION/SHADOW-DOGFOOD-GATE.md](EVALUATION/SHADOW-DOGFOOD-GATE.md) and [OPERATIONS.md](OPERATIONS.md#shadow-dogfood-gate-t170--p94).

---

## 17. What it is not

- Not a cloud SaaS memory product by default
- Not a full IDE replacement
- Not an MCP server (CLI/hooks only)
- Graph-heavy features need a **`--features graph`** build (recommended: `cargo install --path crates/ai-brains-cli --locked --features graph`). Feature-off binaries exit **2** with `FEATURE_UNAVAILABLE` on `graph *` (T198). GitHub Release `ai-brains.exe` is currently graph-off. Healthy local models may still be needed for semantic/embedding paths.
- Capture **must not** depend on intelligence features
- Ledgerful bridge **push/IPC enrichment is opt-in** on the Ledgerful side
- Not a third-party connector plugin host — release connectors are first-party **`TrustedBuiltin` only** ([ADR-0019](DECISIONS/ADR-0019-connector-sandbox-execution-model.md))


---

## 18. Install & entry points

```powershell
# From AI-Brains repo
.\scripts\Build-AIBrains.ps1   # → ~\.cargo\bin\ai-brains.exe + ai-brainsd.exe

ai-brains init
ai-brains context
ai-brains preflight --summary
ai-brains pin "DECISION: …"
ai-brains recall "…" --semantic
ai-brains daemon status          # optional always-on: daemon install (elevated)
```
