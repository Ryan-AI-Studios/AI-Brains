# T233 Plan — Path-alias multi-root nightly (Option B)

**Status:** 🔄 **In Progress** (user go 2026-08-11)  
**Spec:** [spec.md](./spec.md) §11 fold-in  
**Category:** FEATURE / OPS / ARCHITECTURE  
**Upstream:** Coordinated **0163-SymbolsInventory** ✅ **Completed** 2026-08-09 (Ledgerful PR #159 `3fe44367`)  
**Ledger TX (on go):**  
`ledgerful ledger start T233-path-alias-multiroot-nightly --category FEATURE --message "register-path CLI + Phase1/2 multi-root; ledgerful symbols --json (no System32 cwd)"`

---

## AI fold-in (2026-08-11) — `C:\dev\AI-review.md` AI1 + AI2

Both AIs re-verified codebase + live 0163; **no Highs**. Verdict: fold M1–M3 + selected lows, then go.

### Disposition table

| ID | Source | Verdict | Plan action |
|----|--------|---------|-------------|
| **M1** path-conflict / silent UPSERT | AI1+AI2 | **Agree** | Pin F21: pre-check on store (not bare UPSERT); exit **1**; message; non-atomic OK for single-operator CLI |
| **M2** SQL path delete + SymbolRecord | AI2 (+ AI1 architecture) | **Agree** | **Delete** `query_symbols_from_ledgerful`, `sqlite_table_exists`; drop `method`/`path_pattern`; **not** feature-gated residual |
| **M3** `take(500)` vs `--limit 5000` | AI1 raise; AI2 options | **Agree raise** | Cap = request limit: **5000** (env `AI_BRAINS_NIGHTLY_MAX_SYMBOLS` default **5000**). Decline “keep 500 throttle” as default (silently discards 90% of inventory) |
| **M3/F9** `current_dir(root)` non-fatal | AI1 | **Agree** | Already F9; reinforce match arms |
| **M4/L4** MADR `current_dir(root)` | AI1+AI2 | **Agree** | Pin call site `ingest_madr_from_ledgerful` + `root: &Path` |
| **L1** alias\|UUID resolve | AI1 | **Agree** | Phase 1 (already); explicit |
| **L1-init** `ledgerful init` per root | AI2 | **Agree** | OPERATIONS + AC12 note |
| **L2** `indexStatus` handling | AI2 | **Agree** | Skip root + warn when present and not usable (F6) |
| **L3** multi-pass algorithm | AI2 | **Agree** | Max depth **2**; `read_dir` top-level; accept partial + warn |
| **L5** MADR empty `project_id` | AI2 | **Agree** | Attribute to **alias-owning** project (F12), never Phase-1 global/nil alone |
| **L6** `symbol_in_project` | AI2 | **Agree keep safety net** | Relative paths pass; absolute outside root drop |
| **L7–L10** source_tag / identity / clap / no reqwest | AI2 | **Agree no action** | Confirmed correct |
| **O1** 0163 JSON unit tests | AI1 | **Agree DoD** | Phase 3 hermetic fixtures |
| **O2** `list-paths` CLI | AI1 | **Soft residual** | Store list required; CLI optional not DoD |
| **O3** default max symbols **500** | AI2 opp | **Decline as default** | Default **5000** to match F37; env may lower |
| **O9** cross-model ARCHITECTURE | AI2 | **Agree** | Phase 7 required |
| **O12** `--auto-index` primary | AI2 | **Agree** | **No** separate `ledgerful index` step; delete/replace `refresh_ledgerful_index` |
| Route `method`/`path_pattern` | both | Soft residual | Lost vs SQL join; content non-route arm only |
| Atomic F21 under concurrency | AI2 | **Acknowledge** | Check-then-write non-atomic; OK for CLI; not a concurrent API |

### Pins locked by fold-in (implementer must follow)

1. **F21 conflict (Phase 1):**  
   - Normalize path → `QueryStore::find_path_alias_owner(normalized) -> Option<ProjectId>` (new thin SELECT on `repository_path_alias_projection`; same data as CP `find_by_path_alias`).  
   - If `Some(other)` and `other != target` → **exit 1**, stderr:  
     `path alias '<path>' is already registered to project <other_id>; choose a different path (unregister-path is soft residual F31)`.  
   - If same project → idempotent success (re-register OK).  
   - Check-then-write is **not** atomic; acceptable for single-operator CLI.

2. **Delete SQL inventory path (Phase 3):**  
   - **Delete** `query_symbols_from_ledgerful`, `sqlite_table_exists`, and any direct open of `.ledgerful/state/ledger.db` from `symbol_bridge.rs`.  
   - **Delete** `refresh_ledgerful_index` as a separate step; freshness = **`symbols --auto-index`**.  
   - **`SymbolRecord`:** drop `method` / `path_pattern`; `symbol_content` = non-route arm only.  
   - Do **not** keep SQL behind a feature flag in v1 (F36 hard-require 0163).

3. **Ingest cap (Phase 3 / F37 / F43):**  
   - Remove bare `.take(500)`.  
   - Per-root ingest max = `AI_BRAINS_NIGHTLY_MAX_SYMBOLS` (default **5000**), aligned with CLI `--limit`.  
   - Metrics: `symbols_returned`, `symbols_ingested`, `symbols_truncated_inventory` (0163 `truncated`), `symbols_truncated_by_ingest_cap` (if env max &lt; returned).

4. **0163 invoke + indexStatus (Phase 3):**  
   ```text
   ledgerful symbols --pub --json --limit <N> --auto-index [--path <prefix>]
   ```  
   with `.current_dir(root)`. Non-zero / spawn err → warn + skip root.  
   If JSON has `indexStatus` and state is not usable → skip + warn (even with auto-index failure).

5. **Multi-pass (F37):** If `truncated: true` after root pass → `read_dir(root)` top-level dirs (skip dotfiles), re-run `--path <name>` per dir, concatenate + dedupe by `(path, qualifiedName, kind)`. Max recursion depth **2**. Still truncated at depth 2 → warn + accept partial.

6. **MADR (Phase 4):** `ingest_madr_from_ledgerful(ctx, alias_project_id, root)` → `Command::new("ledgerful").current_dir(root).args(["bridge","export",…])`. Empty `BridgeRecord.project_id` → **alias owner**, not Phase-1 env project.

7. **Docs:** OPERATIONS: System32 independence; `register-path` ≠ `set-alias`; each root needs **`ledgerful init` once** before first index/symbols.

---

## Preflight (done at plan time — 2026-08-11)

| Check | Result |
|-------|--------|
| `ai-brains preflight --summary` | OK (pinned + constraints present) |
| `ai-brains recall` multi-root / 0163 | Pipeline DECISION: 0163 then T233; Option B path aliases |
| `ledgerful doctor` | ready (ambient warns: legacy `.changeguard`, sig-pin, timings — OOS) |
| `ledgerful ledger status --compact` | 0 pending, 0 unaudited drift |
| **AC0 live:** `ledgerful symbols --help` | Present: `--path` `--changed` `--kind` `--pub` `-l/--limit` `--json` `--auto-index` |
| **AC0 live:** pure JSON smoke | `schemaVersion: 1`, `truncated`, `resultCount`, `totalMatching` (COUNT), `symbols[]` |
| AI-Brains `--pub --limit 5000` | **2699** public symbols, `truncated: false` (fits hard max) |
| `crates --pub --limit 5000` | **2536**, not truncated |
| Live vault `project list` **path** column | **All `—`** (zero path aliases registered) — labels like `C:\dev\dedupe` are **name aliases**, not filesystem path aliases |
| `C:\dev\` dirs with `.ledgerful` | AI-Brains, CrawlX, Dedupe, Degoo, Family, GIMP, homebrew-tap, KinLedger, ledgerful, ledgerful-action, ledgerful-frontend, ledgerful-web, STL, Wondermaker |
| WSL `\\wsl.localhost\Ubuntu\home\ryan\dev` | honcho, memory-os, Sneaky-Browse (reachable; index when ready) |
| Code SoT | `register_path_alias` CP-only; `symbol_bridge` uses **cwd** + raw SQL `project_symbols`; nightly single-shot MADR+symbols |
| Dep research | **No forced bumps** (see § Research) |

### Deferred roll-in

| Item | Source | Disposition |
|------|--------|-------------|
| Nightly Ledgerful bridge cwd=System32; multi-repo roots | deferred.md | **Absorb** — this track DoD |
| T229 multi-root bridge half | deferred / T229 | **Absorb** |
| Prefer `ledgerful symbols` over SQL | 0163 cross-repo | **Absorb** — primary path F36 |
| T212 soft hermetic path_alias seed / verbose raw name | deferred | **Soft residual** — not DoD |
| T229 soft F8–F12/F14 (doctor model ports, JSON status, embed sleep) | T229 | **Decline** — orthogonal |
| Unbounded dump-all / implement 0163 in AI-Brains | prior drafts | **Decline** |
| Forced clap/serde_json/rusqlite bumps | crates.io latest | **Decline** |
| `unregister-path` compensating event | draft F31 | **Soft residual** v1 |
| Bootstrap `--from-scan` | draft F15 | **Soft Phase 5** — not DoD v1 |
| Route join via `api_routes` / `endpoints` | T70 SQL | **Soft residual** — 0163 symbols lack method/path_pattern |
| Multi-kind OR / multi-path flags in Ledgerful | 0163 OOS | **Client multi-pass** if `truncated` |
| AI2 keep-500 ingest throttle | AI-review | **Declined as default** — raise to 5000 / env (F43) |
| SQL inventory feature-flag residual | AI-review M2 | **Declined** — delete SQL path in v1 |

---

## Research (2026-08-11)

### 0163 shipped contract (freeze — parse only these fields)

```text
ledgerful symbols --pub --json --limit <N> [--auto-index] [--path <prefix>]
```

| Field | Rule |
|-------|------|
| `schemaVersion` | **1** (reject / warn other) |
| `scope` | `{ path, changed, kind, pubOnly }` — nulls when unset |
| `limit` | request echo |
| `truncated` | `totalMatching > limit` |
| `resultCount` | `symbols.len()` |
| `totalMatching` | **COUNT** before limit (honest multi-pass) |
| `symbols[]` | `name`, `kind`, `path`, `line?`, `isPublic`, `qualifiedName` |
| `indexStatus` | optional; present when index missing/unusable |
| Identity for MemoryPinned key | prefer `qualifiedName` + project_id (stable); content uses kind/name/path/line |
| Defaults | limit **200**; hard max **5000** |
| Empty | envelope + `symbols: []`, exit **0** |

**Nightly invocation policy (locked — post AI fold-in):**

1. `Command::new("ledgerful").current_dir(root)` for **all** root-scoped cmds (F9) — PATH exe, then `current_dir` (Windows relative-exe caveat).  
2. **Primary:** `symbols --pub --json --limit <N> --auto-index` where `N = min(5000, AI_BRAINS_NIGHTLY_MAX_SYMBOLS)` (default 5000). **No** separate `ledgerful index` / no `refresh_ledgerful_index`.  
3. If `truncated: true` → multi-pass depth ≤2 via `read_dir` top-level dirs + `--path` (F37); never silent complete.  
4. `indexStatus` present/unusable → skip + warn (F6).  
5. Missing binary / non-zero → per-root warn + continue (F11).  
6. **SQL inventory path deleted** in v1 (not gated).  

### Dependency pins (no bumps)

| Surface | Workspace / live | Latest researched | T233 posture |
|---------|------------------|-------------------|--------------|
| clap | workspace `4.5` (derive+env) | crates.io / docs.rs **4.6.6** (2026-08-11) | **No bump** — add `ProjectCommands` variants only |
| serde_json | workspace `1.0` | **1.0.151** | **No bump** |
| rusqlite | workspace `0.39` (SQLCipher) | crates.io **0.40.2** | **No bump**; CLI still has rusqlite for other paths; primary symbol path leaves SQL |
| ledgerful CLI | installed PATH (0.2.7+ with 0163) | PR #159 | **Runtime contract** only — not a Cargo dep |
| reqwest | CLI has **none** | — | **Do not add** (T229 SOOT) |

### Windows / scheduler best practice

- Task Scheduler default cwd is often `System32` if “Start in” empty — **do not rely on scheduler Start in**; fix is **in-process** `current_dir(root)` per root (F9/F17).  
- Rust note: on Windows, relative exe paths resolve **before** `current_dir`; invoke `ledgerful` via PATH (absolute-from-PATH), then set `current_dir` for index discovery only.

### Architecture SoT (files on go)

| Area | Path | Today | T233 change |
|------|------|-------|-------------|
| CP register | `ai-brains-control-plane/src/grants.rs` `register_path_alias` | Event write | **Reuse**; CLI adapter only |
| Projection | `store/.../repository_identity.rs` | UPSERT by normalized_path | Conflict check in CLI before write (F21) |
| Path list (display) | `query_store` `list_projects_detail` | First path ASC only | Keep; add **`list_path_aliases`** for Phase 2 |
| Normalize | `ai_brains_path::normalize_for_location_compare` | Used by CP | Required for register + sort (F4/F28) |
| CLI project | `main.rs` `ProjectCommands` | list/resolve/detect/set-alias | **+ register-path** (+ soft list-paths) |
| Symbol bridge | `cli/.../symbol_bridge.rs` | cwd + raw SQL + `.take(500)` | **Root + 0163 JSON**; **delete** SQL path; cap **5000** (env) |
| Nightly | `cli/.../nightly.rs` | Single project_id MADR+symbols | Phase1 vault; Phase2 foreach alias |
| MADR | `ingest_madr_from_ledgerful` | no `current_dir` | `root: &Path` + `.current_dir(root)` (F27); empty record pid → alias owner |

---

## Frozen open questions (plan lock + fold-in)

| # | Question | Decision |
|---|----------|----------|
| 1 | Hard-require 0163 vs SQL fallback? | **Hard-require** `ledgerful symbols`. **Delete** SQL inventory path in v1. |
| 2 | Bootstrap `--from-scan` in v1? | **No DoD** — soft Phase 5. |
| 3 | `unregister-path` v1? | **Soft residual** (F31). |
| 4 | Exact flags | `symbols --pub --json --limit N --auto-index [--path …]` |
| 5 | Route enrichment | Soft residual; drop method/path_pattern fields |
| 6 | Max roots | Optional `AI_BRAINS_NIGHTLY_MAX_ROOTS` (F35) |
| 7 | `take(500)`? | **Raise** to 5000 / `AI_BRAINS_NIGHTLY_MAX_SYMBOLS` (F43) — not keep-500 default |
| 8 | F21 check location? | **QueryStore** `find_path_alias_owner` before CP write; exit 1 |
| 9 | Multi-pass depth? | **2** max; partial OK with warn |
| 10 | MADR empty project_id? | **Alias-owning** project (F12) |

---

## Phase 0 — Ledger + impact (on go)

- [ ] `ledgerful ledger status --compact`
- [ ] `ledgerful ledger start T233-path-alias-multiroot-nightly --category FEATURE --message "…"`
- [ ] `ledgerful scan --impact` (symbol_bridge, nightly, project CLI, query_store, grants)
- [ ] Reconfirm `ledgerful symbols --json` schemaVersion 1 on PATH binary

## Phase 1 — Red → Green: `register-path` CLI (F2/F4/F13/F21 / AC1 / AC13)

- [ ] Clap: `project register-path <project_id|alias> <path>`
- [ ] Resolve: UUID parse **or** alias lookup (L1)
- [ ] Normalize path; reject empty after normalize
- [ ] **F21:** `find_path_alias_owner` → other project → **exit 1** + fixed message (see pins); same project → ok idempotent
- [ ] Call `register_path_alias` (CP); do not rely on UPSERT alone
- [ ] Hermetic AC1 + conflict AC13
- [ ] Dual Win/WSL normalize same project (AC2)

## Phase 2 — Query: list all path aliases (F1/F14/F28)

- [ ] Store: `list_path_aliases() -> Vec<(ProjectId, String)>` ORDER BY normalized ASC
- [ ] Store: `find_path_alias_owner(normalized) -> Option<ProjectId>` (F21)
- [ ] Soft CLI `project list-paths` (O2) — **not DoD**
- [ ] Units: multi-alias same project; sort order; conflict owner lookup

## Phase 3 — Symbol bridge: explicit root + 0163 JSON (F9/F36–F39/F43 / AC3–AC4)

- [ ] API: `ingest_symbols_from_ledgerful(ctx, project_id, root: &Path) -> Result<usize, _>`
- [ ] **Delete:** `query_symbols_from_ledgerful`, `sqlite_table_exists`, `refresh_ledgerful_index`, direct rusqlite ledger.db open in this module
- [ ] **SymbolRecord:** drop `method`/`path_pattern`; simplify `symbol_content`
- [ ] Spawn: `ledgerful symbols --pub --json --limit N --auto-index` + `.current_dir(root)`; non-fatal arms
- [ ] Parse envelope: schemaVersion 1; `indexStatus` → skip+warn; map symbols; **no** `.take(500)` — cap via F43
- [ ] Multi-pass depth ≤2 on `truncated` (F37 pin)
- [ ] Keep `symbol_in_project` as safety net (L6); source_tag `ledgerful:symbol`
- [ ] **O1 DoD:** unit tests mock 0163 JSON (truncated true/false, missing line, indexStatus)
- [ ] Hermetic: fixture JSON only; process cwd must not affect root-scoped invoke (mock/spy Command if needed)

## Phase 4 — Nightly Phase 1 / 2 loop (F5–F12/F17–F18/F26–F29 / AC5–AC8)

- [ ] **Phase 1:** existing vault intelligence once — capture independence (F18)
- [ ] **Phase 2:** `list_path_aliases`; zero → no-op + `register-path` hint (AC7)
- [ ] For each root ASC: skip missing path; MADR + symbols with **alias `project_id` + `root`**
- [ ] MADR: `.current_dir(root)` at `Command::new("ledgerful")` export site; empty record pid → alias owner (L5)
- [ ] Per-root failure warn + continue (AC8); Phase1 OK if Phase2 all skip (AC6)
- [ ] Metrics F26 + `symbols_truncated_by_ingest_cap` when applicable
- [ ] Env: `AI_BRAINS_NIGHTLY_MAX_ROOTS` (F35), `AI_BRAINS_NIGHTLY_MAX_SYMBOLS` (F43)
- [ ] Units: System32 process cwd must not zero registered temp root (AC3)

## Phase 5 — Soft bootstrap scan (F15/F20 — not DoD)

- [ ] Optional: `--from-scan` dry-run for `.ledgerful` roots  
- [ ] Skip if timeboxed; document operator one-liners instead

## Phase 6 — Docs (F24 / AC9)

- [ ] `Docs/OPERATIONS.md` — multi-root; register-path vs set-alias; System32 independence; **`ledgerful init` once per root**; 0163 flags
- [ ] `Docs/CAPABILITIES.md` — path aliases + Phase2 bridge
- [ ] Root `CHANGELOG.md` — T233 row only
- [ ] Pointer to 0163 agent-output-contract field names

## Phase 7 — Review + gate + dogfood (AC10–AC12)

- [ ] Internal review vs spec (incl. fold-in pins)
- [ ] **Cross-model Codex required** (ARCHITECTURE/FEATURE)
- [ ] Full CI gate + `ledgerful verify --scope full`
- [ ] **Live dogfood AC12:** root has `.ledgerful` (AI-Brains does) → register-path → Phase2 → symbols &gt; 0
- [ ] Conductor **Completed**; deferred.md strike; pin DECISION
- [ ] Optional fleet register of other `C:\dev\*` roots (not gate)

---

## Explicit non-goals

- Implementing or re-shipping 0163
- Unbounded dump-all / raise Ledgerful hard max beyond 5000
- MSI / clap 5 / forced dep bumps
- Adding reqwest to CLI
- Auto-bridging all vault projects without path aliases
- Changing T229 router/probe/status surface
- `Option<ProjectId>` API expand for Phase1 (keep T229 nil SOOT)
- WSL-only roots without Windows-reachable path (prefer dual-checkout under `C:\dev`)
- Keeping SQL `project_symbols` path as feature-flagged residual in v1
- Default ingest throttle of 500 (declined)

---

## Evidence log (fill on implement)

| AC | Command / proof | Result |
|----|-----------------|--------|
| AC0 | `ledgerful symbols --help` + `--json` smoke | plan-time PASS |
| AC1 | hermetic register-path → list path | |
| AC2 | dual path normalize same project | |
| AC3 | bridge with System32 process cwd + explicit root | |
| AC4 | Phase2 uses 0163 inventory (not SQL) | |
| AC5–AC8 | skip/continue/zero-alias/phase1-ok | |
| AC9 | docs + CHANGELOG | |
| AC10–AC11 | full gate; no unwrap/expect prod | |
| AC12 | live register AI-Brains + symbols &gt; 0 | |
| AC13 | F21 conflict → exit 1 + message | |
| AC14 | no `.take(500)`; cap ≥5000 default; SQL funcs deleted | |

---

## Definition of Done

- [ ] F0–F44 + AC0–AC14 (F0/AC0 already satisfied by 0163 ship)
- [ ] AI fold-in pins honored
- [ ] Plan phases complete; review clean; full gate green
- [ ] Manual AC12 recorded
- [ ] conductor + deferred + pin updated
- [x] User **go** received (implement session)

---

**In progress — product + review loop.**
