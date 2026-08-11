# AI-Brains — Capabilities & Features

**Version:** 0.1.1
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

| Surface | Default TTY | Default non-TTY | Notes |
|---------|-------------|-----------------|-------|
| `recall` | pretty | json | Explicit `--format` wins |
| `preflight` | human | json | TTY default is `human`; `--pretty` / `--format pretty` also human-mode |
| `briefing` | markdown | json | T202 F9; explicit `--format` wins |
| `query progressive` / `expand` / `trace` | json | json | No TTY flip |
| list/show (governed evidence/source/review/...) | json (Human if `--format human`) | json | `OutputFormat::parse` → Json bare |
| `doctor` | human | human | `--json` / `--format json` override |

**Global options:**

| Flag / env | Purpose |
|------------|---------|
| `--vault-path` / `AI_BRAINS_VAULT_PATH` | Vault database path |
| `--key` / `AI_BRAINS_KEY` | Vault SQLCipher key (`x'<64 hex>'`). Zero key refused unless `AI_BRAINS_ALLOW_ZERO_KEY=1` (T187) |
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
- **Wired today:** shared `parse_transcript_for_ingest` (step-shaped + legacy `{role,content}`, prefer `transcript_full.jsonl`), `antigravity::extract_turns` / import + `agy-hook` (message-only SOOT); ProjectChat → `filter_turn`; **Grok** `filter_grok_history_*` (F11 user_query-only) + `grok-hook` / `grok-import`; **OpenCode** `filter_opencode_*` / `filter_opencode_export` (nested + synthetic drop) + `opencode-hook` / `opencode-import`.
- AGY live+batch seamless ingest: **Implemented with caveats** (T236). Grok Build: **Implemented with caveats** (T237). OpenCode: **Implemented with caveats** (T238). Multi-harness nightly orchestration: **Implemented (T239)** (agy → grok → opencode; Claude/Codex install backends remain **T239+**).

### Manual / programmatic
- **`ingest`** — JSON turn from stdin (`session_id`, `project_id`, `harness_id`, `turn_id`, `role`, `content`, `privacy`)
- **`--dry-run`** — preview without write (relaxed validation on dry-run path)

### Harness integrations

| Integration | Mechanism | Notes |
|-------------|-----------|--------|
| **Detect + install UX (T235)** | `harness status\|install\|uninstall\|reset-decline` | Detect harnesses **installed on machine** (PATH + home); wiring `absent\|missing\|partial\|ok\|backend_pending\|unknown`. User-global only (no repo pollution). Preflight `--summary` shows **Harnesses installed on machine:** sibling section. |
| **agy (Antigravity CLI)** | `harness install --harness agy` → Stop wrapper → `agy-hook --payload` | **Implemented (T235+T236):** hooks.json + wrapper; wrapper stdout = allow-stop JSON only; step-shaped + full transcript prefer; path normalize; env fallback only for `agy-unbound`. Reinstall after T236. |
| **agy-hook** | `agy-hook --payload '{...}'` | Real-time ingest; shared parse + turn-id SOOT; diagnostics on stderr; `--schema` |
| **Antigravity bulk** | `antigravity-import --days N [--force]` | History.jsonl workspace bind; unbound `agy-unbound`; stats on stderr; `--force` skips 300s quiescence; default `allow_default_project=false`. Nightly multi-import (T239) includes AGY; SYSTEM scheduled nightly keeps `--skip-import`. |
| **Grok Build** | `harness install --harness grok` → Stop/SessionEnd wrapper → `grok-hook` | **Implemented with caveats (T237):** `~/.grok/hooks/ai-brains.json` + `~/.ai-brains/hooks/grok-capture.ps1`. **Stop allow = empty stdout** (never AGY `{"decision":"allow"}`). User keep: non-empty `<user_query>`/`<USER_REQUEST>` only (chrome/`synthetic_reason` dropped). Subagent/worktree sessions skipped by default. `source_ts` usually none → `occurred_at` = ingest time. Turn ids `v5(session,"turn-{i}")` on kept index (filter taxonomy change can shift ids). Vendor-compat: Grok may also load Claude/Cursor hooks. |
| **grok-hook** | `grok-hook --payload '{...}'` | Live chat_history ingest; path resolve (percent-encode + `.cwd` + summary.id); diagnostics stderr; `--schema` |
| **Grok bulk** | `grok-import --days N [--force]` | Walks `~/.grok/sessions/**/chat_history.jsonl`; summary bind (`git_root_dir`/`cwd`); unbound `grok-unbound`; never `updates.jsonl`; stats on stderr |
| **OpenCode** | `harness install --harness opencode` → plugin `session.idle` → `opencode-hook` | **Implemented with caveats (T238):** `~/.config/opencode/plugins/ai-brains-capture.js` (or `OPENCODE_CONFIG_DIR/plugins/`). Live prefers SDK `client.session.messages` (CLI export fallback, 120s; temp export unlinked after hook). **Child/subagent** sessions with `parentID` skipped (plugin fail-closed if `session.get` throws). **Synthetic/ignored/editor_context** text parts dropped; bare non-synthetic user text kept. Nested export `{info,parts}` normalized. Turn ids use `msg_*` for stability (`v5(session,msg_id)`); **delta is max turn_index + watermark** (same class as Grok `turn-{i}` residual — not per-msg_id existence). `session.idle` is schema-deprecated (batch is completeness backstop). Never opens `opencode.db`. List default cap 100 (`list_capped` warn when len≥100 even if `--max-sessions` higher). `--pure` / plugin-disable soft. SYSTEM skip-import honesty (T239). |
| **opencode-hook** | `opencode-hook --payload '{...}'` | Live/batch-shaped export or messages path; parentId skip; worktree→directory bind; unbound anti-hijack; `--schema` |
| **OpenCode bulk** | `opencode-import --days N [--force] [--dry-run] [--max-sessions N]` | `opencode session list --format json` + `export`; watermark `~/.ai-brains/opencode-import-cursor.json`; never SQLite; soft skip if binary missing |
| **Claude / Codex** | status + dry-run | Install backends **pending** (T239+); real install does not claim capture ok |
| **Claude hooks** | `Docs/claude-hooks.md` | User-level scripts under `~\.ai-brains\scripts\` |

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
| Init project context | `context` — writes local `.env` (`PROJECT_ID`, `SESSION_ID`, `HARNESS_ID`) |
| Show only | `context --show` |
| Rotate project / session | `--new-project` · `--new-session` |
| Ledger linkage | `--tx-id` / `LEDGERFUL_TX_ID` (legacy `CHANGEGUARD_TX_ID` fallback) |
| List projects | `project list` — **label-first** human table: `label` \| `project_id` (full) \| `memories` \| `last_activity` \| `path`. Label prefers alias; baked/machine names show as `(no alias)`. Active process `AI_BRAINS_PROJECT_ID` gets a `*` prefix on the label. |
| List JSON | `project list --format json` — `{ api_version, projects[{project_id,name,alias,label,memory_count,last_activity,path}], unaliased_count }` (pretty). Path is `null` when absent. No dual `--json` flag. |
| last_activity | Last **memory-projection mutation** (pin / forget / ingest / turn upsert), falling back to project `updated_at` when the project has no memories — **not** “last user chat message only.” Human: relative when age &lt; 365d (`just now` / `Nm` / `Nh` / `Nd`); else `YYYY-MM-DD`. |
| path column | Registered `repository_path_alias_projection.normalized_path` when present (lexicographically first); never invented from cwd/git. Human `—` / JSON `null` when unknown. |
| Unaliased nudge | When ≥1 project has no alias, **stderr** prints count + copy-paste `ai-brains project set-alias <uuid> <suggestion>` (highest-memory unaliased). Empty vault: T198 empty line only — **no** footer. Exit 0 always. |
| Aliases | `project set-alias` · `project resolve` |
| Auto-detect | `project detect` (git slug → vault match → env `PROJECT_ID`); `context` discovery (`.ledgerful` / `.env`) |
| Stop session | `stop-session` |
| Env precedence | CLI flags / shell env > elevation handoff (elevated child only) > project `.env` > global `~\.ai-brains\.env` (always merged for gaps; `--no-project-context` skips project only) |
| Local ID force-set | Project-local `.env` **always force-sets** `AI_BRAINS_PROJECT_ID` / `AI_BRAINS_SESSION_ID` (cwd project beats a stale shell). Other keys still follow shell > project > global gap-fill. |
| Override warn (T223) | When shell had a **different** ID value, CLI may print **one** collapsed stderr line: `Warning: local .env overrides inherited shell: AI_BRAINS_PROJECT_ID (was …)[, AI_BRAINS_SESSION_ID (was …)].` **Session-only** override (PROJECT equal/missing) → no stderr; debug only. **Project** differ still warns unless quiet. |
| Quiet override warn | `AI_BRAINS_QUIET_ENV_WARN=1`/`true`/`yes` (case-insensitive) → no stderr override line (collapsed `debug` only). Honored from **shell env or project `.env` only** (already loaded at emit). Global `~/.ai-brains/.env` alone is **too late** (loads after apply). Distinct from T206 `git/env project mismatch`. |

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

### Memory inventory (T216)
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
| **Scope** | Project default (`AI_BRAINS_PROJECT_ID` / `--project-id`); without project and without `--global` → exit **2**. `--global` → `Scope: global`. |
| **Status** | `--status pinned\|forgotten` (default **pinned**). `forget --list-forgotten` ≡ `memory list --status forgotten` (+ limit/scope/format/tag). |
| **Limit** | Default **50**, max **200** (`clamp_list_limit`); `more_available` / `Showing N of T`. **BREAKING:** list-forgotten no longer dumps unbounded rows. |
| **Summary** | `--summary` always prints **Pinned** + **Forgotten** (ignores `--status`/`--limit`). Under `--global`: by-project table (projects with either count &gt; 0 only; **turn-only projects excluded** — use `project list` for those). With `--tag`, top-line **and** by-project cells use the same two-stage tag filter (F46). |
| **Tags** | Content-prefix heuristic only (`TAGS:` first line after optional role prefix from `pin --tag`); SQL start-anchored `LIKE 'TAGS:%'` (or `ROLE: TAGS:%`) + case-insensitive exact token. Not a schema column. Sparse token density among `TAGS:` rows can under-fill a page under elevated candidate cap (raise `--limit`). |
| **Formats** | human table (Scope + preview with role-prefix strip) or `--format json` (`api_version`, items, total, more_available). |
| **Empty** | `No pinned memories.` / `No forgotten memories.` exit **0**. |

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
| **Semantic** | `--semantic` + stored embeddings; honors `AI_BRAINS_EMBEDDING_URL` (default `http://127.0.0.1:8083`) and `AI_BRAINS_EMBEDDING_MODEL` (default `nomic-embed-text-v1.5`). **Hybrid-arm** cosine floor default **0.55** (`AI_BRAINS_SEMANTIC_MIN_SCORE`). Candidates below floor are dropped before fusion. Symbols share the same embedding space as decisions — hybrid + floor reduce topic drift but do not eliminate symbol noise. Floors are **model/corpus calibrated** for default nomic-embed-text-v1.5 without task prefixes (both query and documents currently unprefixed / symmetric); a future nomic `search_query`/`search_document` prefix cutover requires vault re-embed **and** floor re-tune. |
| **Dual cosine floor (T218)** | When the pre-RRF local arm has **no** `source=="fts"` hit (empty FTS **or substring-only** — substring does **not** count as FTS), a second gate **`SEMANTIC_ONLY_MIN=0.60`** (`AI_BRAINS_SEMANTIC_ONLY_MIN_SCORE`) is applied **after** the 0.55 hybrid-arm filter and **before** RRF. When any true FTS hit exists (incl. T217 rescue), only the hybrid-arm 0.55 floor applies. Gate helper: `has_fts_arm` — **not** `local_hits.is_empty()`. |
| **`--min-score` override (T218 F2b)** | One-shot CLI floor for `--semantic`. When set to **X**, **replaces** both the hybrid-arm default (0.55) **and** the semantic-only default (0.60) with **X** (not `max()`). So `--min-score 0.57` can re-admit residual neighbors that default dual floor would drop. Omit the flag to keep dual-floor defaults. |
| **Hybrid RRF (T215/T218)** | When `--semantic` is on, **FTS-only ranked list + thresholded semantic ranked list** are fused with pure **Reciprocal Rank Fusion** (`score = Σ 1/(k+rank)`, default **k=60**, env `AI_BRAINS_RRF_K`) **before** bridge merge / graph / `rerank_hits`. Equal weights only. **Missing from a list → no RRF summand** (not rank=`len+1`). Both arms → source `"hybrid"` (FTS content/`updated_at` preferred; **pre-fuse cosine preserved** from semantic arm when available); FTS-only → `"fts"` (score is RRF contribution under `--semantic`); semantic-only → `"semantic"`. **Substring hits** are **outside** RRF (merged after fuse by id-dedupe; bridge wins first). Candidate depth per arm: `max(limit*3, 15).min(50)`; final truncate after pin re-rank. Without `--semantic`: no RRF; blend is bridge → FTS → graph → re-rank. `sync query` stays `semantic: false`. |
| **Score polarity (ScoreKind)** | `Bm25LowerBetter` (FTS/substring): composite base = `-score`. `HigherIsBetter` (cosine / RRF): base = `score * RELEVANCE_SCALE` (**500**). `BridgeHigherIsBetter` (Ledgerful Tantivy): base = raw relevance **unscaled / not negated** so large bridge scores (~10–20) stay authority-class. Graph hits **inherit** the parent’s `score_kind`. |
| **JSON score honesty (T218)** | Additive per-result fields: **`score_kind`** closed wire set **`bm25` \| `rrf` \| `bridge` only** (map: `Bm25LowerBetter`→`bm25`, `HigherIsBetter`→`rrf` including fused hybrid and single-list identity RRF, `BridgeHigherIsBetter`→`bridge` — never emit wire kinds `cosine` or `hybrid`; origin stays on **`source`**). Optional **`cosine`** when known (pre-fuse dense sim). JSON **`score`** remains the machine value (RRF under `--semantic` after fuse; BM25 rank without) — **not** rescaled into a fake 0–1 confidence. |
| **Pretty scores (T218 F6)** | Branch on ScoreKind: **RRF / HigherIsBetter** → primary **`rank=#n`** + **`sim=0.XX`** when cosine known (do **not** primary-print raw RRF ~0.016). **BM25** keeps `score={:.3}` (negative polarity). **Bridge** keeps readable raw `score={:.3}`. |
| **Role strip on human previews (T224)** | Pretty recall / `sync query` hit lines and forget human match previews strip leading case-sensitive `USER:` / `ASSISTANT:` / `SYSTEM:` (shared `strip_role_prefix` SOOT; pretty: `trim_start` then strip **before** 500-char truncate; forget: via `memory::preview_line` budgets 100 dry-run/single/UUID, 80 multi-match with `…` on cut). **JSON** `RecallResult.content`, bridge Insight export, and **MemoryPinned** event payloads stay **raw**. **ingest/pin `--dry-run`** previews stay **raw** (write-intent honesty — exactly what will be stored). |
| **Embedding status** | With `--semantic`, JSON includes additive `embedding: { status, endpoint?, detail? }`. Closed statuses: `ok` \| `unreachable` \| `error` \| `no_stored_embeddings` \| `skipped`. Soft-fail: embed down never aborts FTS/bridge recall (exit **0**). Pretty: status line when `status != ok`; when `status == ok` but **zero** semantic hits above the **hybrid-arm** cosine floor (post-0.55 / pre-dual-floor count) and lexical results are non-empty → `Embedding: ok (no semantic hits above threshold; showing lexical)`. All-below-floor is not an error. |
| **Pin-type re-rank (T211/T215)** | After blend/graph and **before** truncate, hits are re-ranked by a **single composite** (`rerank_hits` — the only post-blend ranking entry point; T215 extends it via ScoreKind, does not add a second final sort). Kind boosts: CONSTRAINT **+4**, DECISION **+2**, HOTSPOT **+0.5**; plan-class DECISION **−3** (optional sibling-track **−2**); shipped DECISION **+1**; small recency boost from `updated_at` (including semantic-only hits that SELECT `updated_at`). Sort: effective desc → `updated_at` desc → `memory_id` asc. Content heuristics only — not lifecycle fact (badge `plan/stale?`). |
| **Graph boost** | Neighbor score boost (`--graph-boost`); expansion runs after RRF+bridge merge when `--semantic` |
| **Substring fallback** | When FTS empty on small vaults |
| **Scope** | Project default (`AI_BRAINS_PROJECT_ID` / flags); `--global` widens to all projects (never auto-widens on empty). Empty **pretty** recall, **preflight `--summary`**, and **full preflight human/pretty body (T219)** print a `Scope:` line (`global` or active `project=<alias-or-name> (<uuid>)` / `project=<uuid>` / `project=(none)`). Under `--global`, summary and full body always show `Scope: global` even if env `AI_BRAINS_PROJECT_ID` is set. Non-empty **recall** pretty Scope remains deferred (T228). |
| **Bridge mix** | Ledgerful hits capped so vault memories still surface; `--no-bridge`. Bridge is **outside** RRF (merged after fuse when `--semantic`, first when not); bridge wins on `memory_id` collision. |
| **Formats** | Pretty on TTY by default; JSON / NDJSON; per-result `session_id`; optional JSON `staleness: "plan"` when demoted; additive `score_kind` + optional `cosine` (T218). Under `--semantic` after fuse, JSON `score` is RRF rank contribution (interpret via `score_kind` + `cosine`), not raw BM25 and not a fake confidence. |
| **Hints** | Empty **pretty** always prints next-action text on stdout (**not TTY-only**), after Scope (and Session only when user `--session` / `--session-prefix` / `--session-last` resolved — generated graph-provenance sessions are omitted on empty). JSON empty still sets `hint` + `effective_session_id` (exit **0**). Next-action only when embedding status already explains cause (T202). `--quiet` does not suppress Scope or the empty hint. **T217:** when default lexical is empty, raw token count ≥ 3, and ≥1 contentful token remains, append plain-text “try fewer keywords” (via core token helpers; not for all-stopword queries; no emoji). |

### Briefing + progressive query (T202)
```powershell
ai-brains briefing project --project-id <uuid>
ai-brains briefing personal --format json
ai-brains query progressive "why was graph backend replaced?" --project-id <uuid>
```

| Feature | Detail |
|---------|--------|
| **Briefing format** | TTY default **markdown**; non-TTY **json**; explicit `--format` wins (dogfood always passes `--format json`) |
| **Denied packets** | Briefing: `denied=true` always seeds `warnings[]` with `kind: "denied"`; markdown includes `> **Denied:** …` one-liner; process exit **0** (soft) |
| **Progressive / expand** | Require project id (`--project-id` or `AI_BRAINS_PROJECT_ID`); missing → exit **2** + copy-paste example on stderr. **T221 honesty:** progressive policy wall → exit **3** with pretty packet still on stdout (`denied`, `denial_hint` bootstrap); expand `kind: Denied` → exit **3** (capability and/or cross-scope); expand `Unknown` → exit **0**. First-run: `policy bootstrap --scope Repository:<uuid>` (System principal when `--principal-id` omitted) |
| **Trace** | No project-id gate; missing/unauthorized → `null` exit **0** |

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
| **Empty** | E1 `items: []` (never null); human `(none)`; exit **0** when policy allows |
| **Policy** | `ReadEvidence` for source/evidence list; `ReadConclusions` for review list; deny → exit **3** + `details.hint` (bootstrap first) |
| **Bootstrap (T210)** | `ai-brains policy bootstrap [--scope …] [--dry-run]` issues discovery grants only (`ReadEvidence`, `ReadConclusions`, `ReadDecisions`, `LocalOnly`); registers principal if missing; idempotent via `active_grants`; no auto-init |
| **Soft-resolve** | Omitted `--scope` fills only when `scope resolve` is authoritative; else **exit 2** `fail_usage` (never exit **6**) |
| **Show** | `source show` / `evidence show` use the same soft-resolve helper |
| **Policy show/check (T226)** | `policy show` and `policy check` soft-resolve omitted `--scope` like discovery lists / bootstrap; always canonicalize via `parse_scope_key` → `scope_identity_key`. `--capability` on check stays clap-required. Erasure / `review resolve` stay clap-required. |
| **Status filter** | Default Active-only on source/evidence projections |
| **Capture independence** | Projection reads only — no models/embeddings; no control-plane→retrieval dependency |

### Preflight (session-start briefing)
```powershell
ai-brains preflight --summary
ai-brains preflight --summary --format json
ai-brains preflight --global --summary --format json
ai-brains preflight --pretty -m 1500
ai-brains preflight --scope "src/foo.rs" --global
ai-brains preflight --stdin
```
Synthesizes repo safety/hotspots, session turns, memory index, recent dense memories, under a word budget (default 1500). Index titles use Unicode-safe truncation.

| Feature | Detail |
|---------|--------|
| **Scope honesty (T214)** | `--summary` prints T207 `Scope:` vocabulary (`Scope: global` / `Scope: project=…` / `Scope: project=(none)`). Never labels multi-project content as a single env `Project: <uuid>`. |
| **Full-body Scope (T219)** | Human/pretty full body (not summary) also prefixes the same T207/T214 `Scope:` line + blank line before the budget-window body. **JSON path does not** add Scope chrome. |
| **Pretty multi-line body (T219)** | Human/`--pretty` / `--format human\|pretty` preserves section newlines (word budget no longer space-joins). Blank line after each emitted `--- Section ---` header. Display-only caps: safety **8** items, sessions **3**, turns/session **6**, Memory Index **15**; orphan empty `---` headers omitted. |
| **Role strip display-only (T219/T224)** | Pretty path strips leading case-sensitive `USER:` / `ASSISTANT:` / `SYSTEM:` on index/session/display lines (shared `strip_role_prefix`; same SOOT as `memory list` preview, recall/`sync query` pretty hits, and forget human previews). Stored vault content and non-summary JSON `text` may still embed role labels where assembly emits them. |
| **Per-section +N notices (T219 F31)** | Overflow notices (plain ASCII): safety → `+N more safety entries — ai-brains memory list`; index → `+N more via recall`; turns → `+N more turns in session`; sessions → `+N more sessions`. |
| **Word budget + F2b (T219)** | `trim_to_word_budget` preserves newlines; over-budget appends trailing `…` on its own line (content `word_count` excludes the sentinel). Applies to full pretty body, non-summary JSON `text`, and governed markdown re-budget (same helper). |
| **Governed F1 benefit (T219)** | When governed rendering is on, `#` / `##` markdown survives the pretty formatter (not treated as `---` section headers). F1/F2b still apply to the governed string. No governed section caps in v1. |
| **Dual count model** | **Vault (SQL):** under `--global`, `Projects:` = distinct projects with pinned memories; always `Pinned memories` + `Active sessions` (SQL on projections, capture-independent). **In context:** `HOTSPOT:` / `DECISION:` / `CONSTRAINT:` counts from the budget-window text only — labeled `In context …` so they are not read as vault totals. |
| **Active sessions** | Rollup uses `session_projection` status=`active` (not a missing text marker). |
| **Ledgerful hotspots** | Require **project-scoped** preflight; ledgerful bridge is intentionally **off** under `--global`. |
| **Governed authority** | `--summary` (human or JSON) is orientation only (T170 D21). Use full `preflight --format json` / `briefing` for governed packet truth — never treat summary counts as authority. |
| **Full preflight JSON (T180)** | Non-summary `--format json` remains compact `{text, word_count}` only — never grow those keys. After T219, `text` retains newlines + optional F2b `…` when truncated; no Scope/caps chrome on this path. |
| **Summary JSON (T220)** | `--summary --format json` (case-insensitive) emits a **pretty** machine object on stdout (no human banner). Keys: `api_version` (`"1"`), `scope` (`"global"` \| `"project"` \| `"none"`), `project_id` (uuid string or `null`), `projects` (**only when `scope=="global"`**; omitted under project/none — never `null`), `pinned`, `active_sessions`, `in_context_hotspots` / `in_context_decisions` / `in_context_constraints`, `word_count`. |
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
| **Vault re-rank** | Same `recall_full` + `rerank_hits` path as `recall` (pin-type authority, plan demotion, recency). Default vault **`--limit` / `-l` = 5**. |
| **Plan badge** | Demoted plan-class DECISION lines show **`[plan/stale?]`** before content (content heuristic ≠ governed lifecycle). |
| **Ledger-first** | When not `--no-bridge`, vault is recalled first; if ledger JSON probe is non-empty **and** the top vault hit is plan-class, prints banner `Note: vault top hit is plan/stale; ledger results shown first.` then **ledger section before vault**. Fail/empty/missing `ledgerful` → vault-only (no panic). |
| **Honesty** | Re-rank is presentation only — pins are never auto-forgotten or mutated. **`rerank_hits` is the single post-blend ranking entry point** (F40); T215 extends it via ScoreKind / hybrid RRF rather than adding a second final sort. `sync query` stays `semantic: false` (lexical + bridge ranking only). |

---


## 8. Nightly intelligence (“Brain mode”)

```powershell
ai-brains nightly
ai-brains nightly --status
ai-brains nightly --skip-import
ai-brains nightly --skip-import-grok
ai-brains nightly --schedule --start-time "03:00"
ai-brains nightly --schedule --run-as-system --dry-run
```

Pipeline includes:
1. **Multi-harness session import (T239)** — fixed order **agy → grok → opencode** (message-only adapters; never `opencode.db`). `--skip-import` skips all three; per-source `--skip-import-agy` / `--skip-import-grok` / `--skip-import-opencode`. Fail-open per source with **per-source sinks**. Report persisted as `last_multi_import` (`v:1`); `nightly --status` prints a Multi-import block (missing → `never`; corrupt → `unreadable`; OpenCode `list_capped > 0` surfaces a cap warning). **Claude / Codex are not in the nightly batch** (install backends remain T239+). Import progress may interleave non-JSON on stderr under SYSTEM `--log-format json` (accepted).
2. Session summarization (chunked; **38,912-token** context with carryover)
3. Memory synthesis (batch-limited, e.g. 50 memories/run)
4. Embedding backfill + stale refresh + WAL checkpoint
5. Ledgerful **symbol bridge** ingest (functions, routes → code-aware recall)
6. **`MemorySynthesized`** events for graph edges
7. Live graph projection updates

SYSTEM-mode schedules bake vault/model env into a wrapper script so Session 0 has config. **SYSTEM keeps `--skip-import` by default** — completeness path is user-principal `nightly --schedule` or manual `nightly` (not Session 0 import).

---

## 9. Graph

Requires a build with **`--features graph`** for the live backend. Recommended source install (INSTALL SOOT):

```powershell
cargo install --path crates/ai-brains-cli --locked --features graph
```

**Feature-off honesty (T198):** on default / slim / GitHub Release `ai-brains.exe` builds (no graph feature), every `ai-brains graph *` subcommand exits **2** with a `FEATURE_UNAVAILABLE:` prefix and a reinstall hint for the command above. `graph --help` remains exit **0**. GitHub Release `ai-brains.exe` is currently graph-off; see [INSTALL.md](INSTALL.md).

| Command | Purpose |
|---------|---------|
| `graph update` | Health report: `nodes`, `edges`, `pinned_memories`, `memory_nodes`, `edge_node_ratio`, `density` (`ok`\|`warn`\|`skip`), `status` (`live`\|`sparse`\|`empty`), `note`, optional `remediation` (T213) |
| `graph rebuild` | Full resync (recovery) |
| `graph neighbors <memory_id>` | 1-hop neighbors |
| `graph hierarchy <memory_id>` | Synthesis chain |
| `graph session <session_id>` | Memories in a session |

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

**Backup list honesty (T209):** each `vault-*.db.bak` is classified under the current vault key:

| Class | Table token (when meta empty) | Default noise |
|-------|-------------------------------|---------------|
| Readable (meta OK) | real metadata values | silent |
| Pre-T109 (openable, no meta table) | `(no metadata)` | debug only |
| Legacy plain (SQLite magic header) | `(legacy plain)` | debug + one summary |
| Key mismatch (size ≥ 512, not plain, key fails) | `(unreadable key)` | debug + one summary |
| Corrupt (short/unreadable garbage) | `(corrupt)` | per-file `WARN` |

Default list prints one **stderr** summary when any legacy-plain or key-mismatch residuals exist (`not fully readable` … `--verbose` … `verify`). `--verbose` adds per-file detail and omits the summary. `--quiet` suppresses summary and metadata WARNs (table tokens still apply). Dual `--quiet --verbose` → quiet wins. Exit **0** for any mix of classes after a successful scan. The list→verify cross-link remains valid; verify is now quiet-by-default (T225).

**Backup verify quiet default (T225):** human `backup verify` prints **counts** (`Verified N backup(s): X OK, Y FAIL.`) plus the first **5** `filename: FAIL — {reason}` lines (T138 reason preserved); OK lines omitted. When fail &gt; 5, a trailer points at `--verbose`. When `ok == 0 && total >= 1`, a create nudge includes `ai-brains backup create`. **`--verbose`** = full per-file OK/FAIL stream only (no summary, trailer, or nudge). **JSON** always returns full `results[]` (verbose ignored). Progress tracing is `debug` (no INFO flood under product default `RUST_LOG`). Any FAIL → exit **1**; empty → exit **0** (`No backups to verify.`).

**Recovery export (T188 / T194):** writes RecoveryKit JSON (`schema_version: 1`) to a restricted file path only (never kit JSON on stdout). Passphrase via file or zero-echo TTY (`rpassword`). Kits embed Argon2id params in `passphrase.kdf` (algorithm=argon2id, version=19, m=19456, t=2, p=1); pre-T194 kits without `kdf` dual-read fixed legacy constants.

**Doctor (T192):** `ai-brains doctor` is a **read-only** operator health surface.

```text
ai-brains doctor
  [--format human|json]           # default human
  [--json]                        # force JSON (overrides --format)
  [--fail-on-degraded]            # exit 1 when status=degraded
  [--kit-path <path>] [--passphrase-file <path>]
  [--backup-max-age <Nd|Nh|Nw>]   # default 7d
  [--full]                        # PRAGMA integrity_check
```

Check matrix (fixed order, **13** checks): `vault_exists`, `vault_open` (`open_read_intent` only — never migrates), `schema_readable`, `cipher_page`, `daemon_reachable` (info: up/down never fails alone), **`backup_recent`** (soft; T225 class-aware: usable = Readable \| PreT109 only — no usable encrypted under current key → warn + `ai-brains backup create` even if plain timestamps are recent; otherwise ages **newest usable** vs `--backup-max-age`, default 7d), `recovery_kit_event` (soft; event ≠ offline file proof), `recovery_kit_file` (hard when `--kit-path` set; skip otherwise — no default kit path search), `zero_key_escape` (soft / R-ZERO-KEY), **`graph_feature`** (soft info: `available`|`unavailable` via compile-time `cfg!(feature = "graph")`; remediation = INSTALL primary SOOT when unavailable; never alone fail/degraded), **`graph_density`** (soft; SQL counts only — warn on empty/sparse/orphan/projection lag; skip when tables missing / open failed / pinned count failed / small empty vault; never alone forces fail; capture-independent even on graph-off binaries; T232 capability-aware remediation: graph-on → `ai-brains graph rebuild`, graph-off → `GRAPH_REINSTALL_SOOT`), **`harness_wiring`** (soft info — harness hook presence; never alone fail/degraded), `integrity` (only with `--full`). Overall: fail ≻ degraded ≻ ok. Exit 0 for ok|degraded (default); 1 for fail; clap usage 2. Never creates vault or `backups/`; never prints secrets. Residual: offline kit without `--kit-path` remains operator responsibility (see RECOVERY-DRILLS).

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
| Code + memory | `recall` or `sync query` |
| Persist a decision | `pin "DECISION: …"` |
| Correct a memory | `forget` / `restore` |
| Sync brittle files | `safety sync` |
| Overnight brain | `nightly` (+ schedule) |
| Hygiene | `backup` · `project list` |

End-to-end recipes: [WORKFLOWS.md](WORKFLOWS.md).

---

## 16. Capability map

```text
CAPTURE          ingest · agy-hook · antigravity-import · daemon queue
CONTEXT          context · project list/resolve/detect/set-alias · stop-session
DENSE MEMORY     pin · forget/restore · safety sync
RETRIEVAL        recall (FTS · semantic · graph-boost · bridge) · preflight · sync query
INTELLIGENCE     nightly (summarize · embed · synthesize · symbol bridge)
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
