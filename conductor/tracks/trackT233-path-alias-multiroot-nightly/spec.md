# T233 — Path-alias multi-root nightly (Option B)

- **Track ID:** T233-PathAliasMultiRootNightly
- **Phase:** Post-audit nightly / Ledgerful bridge honesty
- **Status:** 📋 **Placeholder** (plan-only — **do not implement until coordinated 0163 ships + user go**)
- **Category:** FEATURE / OPS / ARCHITECTURE
- **Owner (draft):** Grok
- **Execution repo (this track):** `C:\dev\AI-Brains`
- **Upstream (do first):** Coordinated **[0163-SymbolsInventory](file:///C:/dev/coordinated/conductor/0163-SymbolsInventory/spec.md)** · execution `C:\dev\ledgerful` · registry row in `C:\dev\coordinated\conductor\conductor.md`
- **Depends on (AI-Brains):** T65/T206 detect; T70 symbol bridge; T132–T145 nightly wrappers; T212 path column; T229 router ops (schedule/env partial); control-plane `register_path_alias` (exists, **no CLI**)
- **Blocks / feeds:** Multi-repo symbol + MADR ingest under Task Scheduler; closes System32 cwd bridge miss; Option B vault path SOOT
- **Source:** 2026-08-06 nightly log; operator Option B; roots `C:\dev\` + WSL `~/dev/`
- **Operator pipeline (2026-08-06):** **Implement 0163 first (small)** → then **T233**. T233 points at 0163 as the preferred inventory contract.
- **Absorbs:** Multi-root bridge half of T229 (T229 keeps router `:8081`/`:8083` ops)
- **Not absorbed:** Implementing 0163; unbounded dump-all; tag schema; graph-on install

---

## 0. Cross-repo pointer — coordinated **0163-SymbolsInventory**

Governance (read on go):

| Item | Location |
|------|----------|
| Spec | `C:\dev\coordinated\conductor\0163-SymbolsInventory\spec.md` |
| Registry | `C:\dev\coordinated\conductor\conductor.md` → **0163** Placeholder 2026-08-06 |
| Execution | `C:\dev\ledgerful` |

**0163 objective (preliminary — not yet implemented):** first-class **scoped** symbol inventory for agents/planning:

```text
ledgerful symbols --path src/… --pub --limit 200
ledgerful symbols --changed --json
ledgerful symbols --kind fn --path src/cli --json
```

- Index-backed only (not a second search engine).
- **Bounded** by default (`--path` / `--changed` / hard `--limit` + `truncated`); **not** unlimited whole-repo dump.
- Pure `--json` (`schemaVersion` 1) for machine mode (soft-after 0149 patterns).
- Soft-after polish batch; independent of SCIP 0157.

**Why T233 waits on 0163**

| Today (T70) | After 0163 (preferred for T233) |
|-------------|----------------------------------|
| `ledgerful index` + **raw SQL** on `.ledgerful/state/ledger.db` (`project_symbols`) | `ledgerful symbols --json` with path/limit/pub filters **from repo root** |
| Couples AI-Brains to Ledgerful SQLite schema | Stable CLI contract; schema drift isolated in Ledgerful |
| No honest `truncated` / limit semantics | 0163 DoD requires limit + truncated honesty |

**Interim fallback (only if 0163 delayed):** keep SQL path as soft residual / feature-detect: if `symbols` subcommand missing, fall back to current SQL with warn. Prefer **hard-require 0163** when pipeline order holds.

**What 0163 is *not*:** unbounded “export entire monorepo.” That remains out of scope in 0163 and **must not** become T233’s ingest strategy. Nightly should call **scoped** inventory (e.g. high limit with `truncated` handling, or per-subdir if needed)—not invent dump-all.

---

## 1. Objective

1. **Option B:** Vault **path aliases** = SOOT for which disk roots get a Ledgerful bridge pass.  
2. **CLI** to register paths (`register_path_alias` is control-plane-only today).  
3. **Nightly Phase 1 / 2:** vault intelligence once; **per registered root** bridge with **explicit root** (never Task Scheduler System32 cwd).  
4. **Windows + WSL:** `C:\dev\…` and WSL `~/dev/…` dual aliases when both exist.  
5. **Symbol fetch:** Prefer **`ledgerful symbols` (0163)** per root; index refresh still `ledgerful index --incremental` (or `--auto-index` policy matching 0163/search).  
6. Capture independence: bridge failures non-fatal.

## 2. Problem (frozen)

### 2.1 Live failure

`AI-Brains-Nightly` → no `cd` → cwd `C:\Windows\System32` → open `System32\.ledgerful` → **0 symbols**. Summaries/embeddings OK.

### 2.2 Vault list alone insufficient

| Fact | Implication |
|------|-------------|
| Most `project list` **path = null** | No disk root |
| `set-alias` is name only | Not a filesystem root |
| Bridge uses **cwd** | Path column alone doesn’t fix nightly |
| Many noise UUIDs | Never auto-bridge all vault projects |

### 2.3 Operator roots (scan 2026-08-06)

**`C:\dev\` with `.ledgerful`:** AI-Brains, CrawlX, Dedupe, Degoo, Family, GIMP, homebrew-tap, KinLedger, ledgerful, ledgerful-action, ledgerful-frontend, ledgerful-web, STL, Wondermaker.

**WSL `\\wsl.localhost\Ubuntu\home\ryan\dev`:** Sneaky-Browse, honcho, memory-os (git yes; `.ledgerful` absent at scan — register when indexed).

## 3. Ledgerful product boundary

| Need | Owner | Track |
|------|--------|--------|
| Scoped inventory CLI for agents/bridge | **Ledgerful** | **0163** (do first) |
| Multi-root + path aliases + nightly loop + consume inventory | **AI-Brains** | **T233** (do second) |
| Human name lookup | Ledgerful | Existing `search` / `ask` (unchanged) |
| Unbounded dump-all | **Neither** | Explicitly out of scope both places |

**Corrected earlier draft note:** “No Ledgerful feature” applied to *unbounded dump-all*. The **right** Ledgerful feature for machine inventory is **0163**, already in the coordinated pipeline.

## 4. Frozen decisions (draft F1–F42)

| ID | Decision |
|----|----------|
| **F0 — Pipeline** | **0163 before T233.** Do not start T233 implementation until 0163 is **Completed** (or explicit go with SQL fallback). |
| **F1 — Option B SOOT** | Vault path aliases (`repository_path_alias_projection`) list Phase-2 roots. |
| **F2 — CLI register-path** | `ai-brains project register-path <project_id\|alias> <path>` → `register_path_alias`. |
| **F3 — Dual Win/WSL** | Register both forms when dual-checkout; `normalize_for_location_compare`. |
| **F4 — Normalize** | Always via path crate (already in register_path_alias). |
| **F5 — No invent paths** | Phase 2 only registered aliases. |
| **F6 — Eligibility** | Skip missing path; skip no index; attempt when root usable. |
| **F7 — Noise** | No path alias → ignored by Phase 2. |
| **F8 — Phase 1/2** | Phase 1 vault once; Phase 2 foreach root. |
| **F9 — Explicit root** | All Ledgerful cmds use `Command.current_dir(root)` (or `--root` if Ledgerful adds it later). |
| **F10 — Zero aliases** | Phase 2 no-op + hint `register-path`. |
| **F11 — Non-fatal** | Per-root warn + continue. |
| **F12 — project_id** | Symbols attach to alias-owning project_id. |
| **F13 — Alias vs path** | `set-alias` label; `register-path` filesystem. |
| **F14 — List paths** | Show all aliases for a project (CLI on go). |
| **F15 — Bootstrap scan** | Soft: `--from-scan C:\dev` / WSL root, dry-run first. |
| **F16 — WSL** | v1 Windows-reachable paths; UNC probe; dual-checkout preferred. |
| **F17 — Scheduler** | Loop **inside** `ai-brains nightly`. |
| **F18 — Capture independence** | Phase 1 needs no Ledgerful. |
| **F19 — Zero new crates** | — |
| **F20 — Scan roots** | Operator-supplied `--root` list; document this machine’s defaults. |
| **F21 — Path conflict** | Same normalized path → one project; second owner errors. |
| **F22 — Trust** | Local operator paths; soft warn outside known roots. |
| **F23 — Tests** | Hermetic register-path; Phase 2 mock/`symbols --json` fixture; no System32. |
| **F24 — Docs** | OPERATIONS multi-root; CAPABILITIES; point at 0163 contract. |
| **F25 — Out** | Unbounded dump-all; implementing 0163 in AI-Brains; MSI. |
| **F26 — Metrics** | bridge_roots_*, symbols_ingested, truncated counts from 0163. |
| **F27 — MADR + symbols** | Same root loop. |
| **F28 — Order** | Roots sorted normalized_path ASC. |
| **F29 — Sequential** | One root at a time. |
| **F30 — Env** | Global dotenv vault/models unchanged. |
| **F31 — unregister-path** | Soft / compensating event. |
| **F32 — detect** | Path alias should feed scope detect (existing CP tests). |
| **F33 — test-alias hygiene** | Separate from this track. |
| **F34 — Governance** | Ship: deferred + coordinated note 0163→T233. |
| **F35 — Perf** | Optional max-roots env. |
| **F36 — Symbol source preference** | **Primary:** `ledgerful symbols --json` (0163 flags as finalized). **Fallback:** SQL only if product explicitly allows pre-0163 go. |
| **F37 — Limit policy** | Use 0163 default/hard limit; if `truncated: true`, log + optional second page **or** raise limit within 0163 max — never silent truncate as “complete.” |
| **F38 — Index freshness** | Match 0163/search policy (`--auto-index` or pre-`index --incremental` per root). |
| **F39 — Pub filter** | Prefer 0163 `--pub` (or equivalent) for parity with today’s “public/entrypoint” SQL filter. |
| **F40 — Plan.md** | Expand on go after 0163 Complete. |
| **F41 — coordination.md** | On T233 go: note AI-Brains consumes 0163 JSON shape; flag schemaVersion. |
| **F42 — Go gate** | User go + 0163 Completed (or documented fallback). |

## 5. Acceptance criteria (draft)

| ID | Criterion |
|----|-----------|
| **AC0** | 0163 Available: `ledgerful symbols --help` / pure JSON smoke from a repo root |
| **AC1** | `register-path` → `project list` path non-null |
| **AC2** | Win + WSL dual normalize to same project |
| **AC3** | Bridge uses explicit root; System32 cwd alone does not zero all roots |
| **AC4** | Phase 2 calls **0163 inventory** (or documented fallback) per root |
| **AC5** | Missing index/root → skip + warn, exit 0 |
| **AC6** | Phase 1 runs if Phase 2 all skip |
| **AC7** | Zero aliases → no-op + register-path hint |
| **AC8** | Per-root failure continues |
| **AC9** | Docs + CHANGELOG |
| **AC10** | Full CI green |
| **AC11** | No production unwrap/expect |
| **AC12** | Live dogfood: register `C:\dev\AI-Brains` + Phase 2 → symbols &gt; 0 |

## 6. Implementation plan (on go — after 0163)

### Phase 0

- [ ] Confirm 0163 Complete; freeze JSON field names against 0163 shipped contract  
- [ ] Probe WSL UNC + `ledgerful symbols` from Windows  
- [ ] Operator list of paths to register  

### Phase 1 — register-path CLI  

### Phase 2 — list path aliases QueryStore  

### Phase 3 — symbol_bridge / MADR: explicit root + **0163 JSON ingest**  

### Phase 4 — nightly Phase 1/2 loop + metrics  

### Phase 5 — optional bootstrap scan  

### Phase 6 — docs + thin wrapper  

### Phase 7 — review / gate / close  

## 7. Risks

| Risk | Mitigation |
|------|------------|
| 0163 not shipped yet | Block T233 code; SQL fallback only with explicit go |
| 0163 bounded limit &lt; public symbol count | Honor truncated; multi-pass by path prefix if needed |
| WSL unreadable | Dual-checkout under `C:\dev`; F16 |
| Schema drift if SQL fallback | Prefer 0163 only |

## 8. Operator runbook (after both ship)

```powershell
# Ledgerful (per repo, or T233 does this in Phase 2)
cd C:\dev\AI-Brains
ledgerful index --incremental
ledgerful symbols --path . --pub --json --limit 500   # shape per 0163

# AI-Brains once per root
ai-brains project register-path <id|alias> C:\dev\AI-Brains

# Nightly (scheduler) runs Phase1+Phase2
ai-brains nightly --status
```

## 9. Open questions (go)

1. Hard-require 0163 vs SQL fallback? (**Default: hard-require.**)  
2. Bootstrap `--from-scan` in v1?  
3. unregister-path v1?  
4. Exact 0163 flag names when full plan locks (`symbols` vs `index symbols`).  

## 10. Definition of Done

- [ ] 0163 Completed (or explicit fallback disposition)  
- [ ] F0–F42 + AC0–AC12  
- [ ] Review + full gate + manual AC12  
- [ ] conductor/deferred/coordinated updated  

---

**Drafting note:** Placeholder only. Points at **0163**; no AI-Brains implementation in this session.
