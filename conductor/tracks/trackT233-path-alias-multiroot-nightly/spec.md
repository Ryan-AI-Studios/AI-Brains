# T233 — Path-alias multi-root nightly (Option B)

- **Track ID:** T233-PathAliasMultiRootNightly
- **Phase:** Post-audit nightly / Ledgerful bridge honesty
- **Status:** 🔄 **In Progress** (user **go** 2026-08-11; upstream **0163 Completed**)
- **Category:** FEATURE / OPS / ARCHITECTURE
- **Owner (draft):** Grok
- **Execution repo:** `C:\dev\AI-Brains`
- **Upstream (done first):** Coordinated **[0163-SymbolsInventory](file:///C:/dev/coordinated/conductor/0163-SymbolsInventory/spec.md)** · Ledgerful PR #159 `3fe44367` (2026-08-09) · `ledgerful symbols` live on PATH
- **Depends on (AI-Brains):** T65/T206 detect; T70 symbol bridge; T132–T145 nightly wrappers; T212 path column; T229 router ops (schedule/env); control-plane `register_path_alias` (exists, **no CLI** until this track)
- **Blocks / feeds:** Multi-repo symbol + MADR ingest under Task Scheduler; closes System32 cwd bridge miss; Option B vault path SOOT
- **Source:** 2026-08-06 nightly log; operator Option B; roots `C:\dev\` + WSL `~/dev/`; plan 2026-08-11; **AI fold-in** `C:\dev\AI-review.md` AI1+AI2
- **Absorbs:** Multi-root bridge half of T229; deferred “System32 Ledgerful miss”
- **Not absorbed:** Implementing 0163; unbounded dump-all; tag schema; graph-on install; T229 soft F8–F12/F14; default ingest throttle of 500

**Plan checklist:** [plan.md](./plan.md) · fold-in pins in plan § AI fold-in

---

## 0. Cross-repo pointer — **0163 Completed** (contract freeze)

| Item | Location / status |
|------|-------------------|
| Spec | `C:\dev\coordinated\conductor\0163-SymbolsInventory\spec.md` |
| Registry | coordinated `conductor.md` → **Completed** 2026-08-09 |
| Ship | Ledgerful PR #159 squash `3fe44367` |
| Live probe (2026-08-11) | `ledgerful symbols --help` + pure JSON OK |

### Frozen CLI (T233 primary)

```text
ledgerful symbols --pub --json --limit 5000 [--auto-index] [--path <prefix>]
```

| Flag / field | Lock |
|--------------|------|
| `--pub` | Prefer public/entrypoint-class inventory (parity with old SQL public filter) |
| `--json` | Pure stdout; `schemaVersion` **1** |
| `--limit` | Default 200; hard max **5000**; nightly uses **5000** |
| `--auto-index` | Per-root freshness when index missing/stale (F38) |
| `--path` | Prefix filter for multi-pass when `truncated` |
| `truncated` + `totalMatching` | COUNT-backed; never silent “complete” (F37) |
| `symbols[]` | `name`, `kind`, `path`, `line` (integer when present), `isPublic`, `qualifiedName` — **no** method/path_pattern |
| `indexStatus` | Optional; when present and unusable → **skip root + warn** (F6); rare with `--auto-index` |

**Why not raw SQL (T70):** couples to Ledgerful SQLite schema; no honest `truncated`; System32 cwd opens wrong/missing DB. **T233 deletes** the SQL inventory path in `symbol_bridge` (not feature-gated residual).

**What 0163 is *not*:** unbounded dump-all. T233 multi-passes scoped inventory; never invent dump-all.

---

## 1. Objective

1. **Option B:** Vault **path aliases** = SOOT for which disk roots get a Ledgerful bridge pass.  
2. **CLI** `project register-path` → control-plane `register_path_alias`.  
3. **Nightly Phase 1 / 2:** vault intelligence once; **per registered root** bridge with **explicit root** (never Task Scheduler System32 cwd).  
4. **Windows + WSL:** dual aliases when both exist; normalize via path crate.  
5. **Symbol fetch:** **`ledgerful symbols --json` (0163)** per root; index via `--auto-index` and/or `index` with `current_dir(root)`.  
6. Capture independence: bridge failures non-fatal; Phase 1 needs no Ledgerful.

## 2. Problem (frozen)

### 2.1 Live failure

`AI-Brains-Nightly` → no guaranteed repo `cd` → cwd often `C:\Windows\System32` → open `System32\.ledgerful` → **0 symbols**. Summaries/embeddings can still succeed (T229).

### 2.2 Vault list alone insufficient

| Fact | Implication (live 2026-08-11) |
|------|--------------------------------|
| `project list` **path = —** for all rows | No Phase-2 roots until `register-path` |
| Labels like `C:\dev\dedupe` | **Human alias / name**, not filesystem path alias |
| `set-alias` is name only | Not a disk root |
| Bridge uses **cwd** | Path column alone does not fix nightly |
| Many noise UUIDs | Never auto-bridge all vault projects |

### 2.3 Operator roots (rescan 2026-08-11)

**`C:\dev\` with `.ledgerful`:** AI-Brains, CrawlX, Dedupe, Degoo, Family, GIMP, homebrew-tap, KinLedger, ledgerful, ledgerful-action, ledgerful-frontend, ledgerful-web, STL, Wondermaker.

**WSL `\\wsl.localhost\Ubuntu\home\ryan\dev`:** honcho, memory-os, Sneaky-Browse (register when indexed / dual-checkout preferred).

**Dogfood volume (AI-Brains):** `symbols --pub --limit 5000` → **2699** symbols, `truncated: false` (one-pass OK on this machine).

## 3. Ledgerful product boundary

| Need | Owner | Track |
|------|--------|--------|
| Scoped inventory CLI | **Ledgerful** | **0163 ✅** |
| Multi-root + path aliases + nightly loop + consume inventory | **AI-Brains** | **T233** |
| Human name lookup | Ledgerful | `search` / `ask` (unchanged) |
| Unbounded dump-all | **Neither** | Out of scope |

## 4. Frozen decisions (F0–F43)

| ID | Decision |
|----|----------|
| **F0 — Pipeline** | 0163 **Completed** → T233 unblocked. Plan-only until user go. |
| **F1 — Option B SOOT** | Vault path aliases (`repository_path_alias_projection`) list Phase-2 roots. |
| **F2 — CLI register-path** | `ai-brains project register-path <project_id\|alias> <path>` → `register_path_alias`. Resolve UUID **or** alias. |
| **F3 — Dual Win/WSL** | Register both forms when dual-checkout; `normalize_for_location_compare`. |
| **F4 — Normalize** | Always via path crate (already in register_path_alias). |
| **F5 — No invent paths** | Phase 2 only registered aliases. |
| **F6 — Eligibility** | Skip missing path; skip unusable index (`indexStatus` or empty unusable); attempt when root usable. |
| **F7 — Noise** | No path alias → ignored by Phase 2. |
| **F8 — Phase 1/2** | Phase 1 vault once; Phase 2 foreach root. |
| **F9 — Explicit root** | All Ledgerful cmds use `Command.current_dir(root)` (PATH-resolved `ledgerful` exe). Non-fatal on spawn/nonzero. |
| **F10 — Zero aliases** | Phase 2 no-op + hint `register-path`. |
| **F11 — Non-fatal** | Per-root warn + continue. |
| **F12 — project_id** | Symbols/MADR attach to alias-owning `project_id`. MADR empty `BridgeRecord.project_id` → **alias owner**, not Phase-1 env/nil alone. |
| **F13 — Alias vs path** | `set-alias` label; `register-path` filesystem. |
| **F14 — List paths** | Store lists all aliases for Phase 2; CLI `list-paths` optional soft. |
| **F15 — Bootstrap scan** | Soft (not DoD): `--from-scan` dry-run. |
| **F16 — WSL** | v1 Windows-reachable paths; UNC probe; dual-checkout preferred. |
| **F17 — Scheduler** | Loop **inside** `ai-brains nightly` (no multi-schtask). |
| **F18 — Capture independence** | Phase 1 needs no Ledgerful. |
| **F19 — Zero new crates** | — |
| **F20 — Scan roots** | Operator-supplied register list; document this machine’s defaults. |
| **F21 — Path conflict** | Same normalized path → one project. **Pre-check** via `QueryStore::find_path_alias_owner` before CP write. Other owner → **exit 1** + message `path alias '…' is already registered to project <id>…`. Same owner → idempotent OK. Projection UPSERT alone insufficient. Check-then-write **not** atomic — OK for single-operator CLI (not a concurrent API). |
| **F22 — Trust** | Local operator paths; soft warn outside known roots. |
| **F23 — Tests** | Hermetic register-path; Phase 2 fixture 0163 JSON (schemaVersion, truncated, indexStatus); no System32 dependence; no live ledger.db required for unit parse. |
| **F24 — Docs** | OPERATIONS multi-root; register-path vs set-alias; System32 independence; **`ledgerful init` once per root**; CAPABILITIES; root CHANGELOG. |
| **F25 — Out** | Unbounded dump-all; implementing 0163 in AI-Brains; MSI; forced dep bumps; SQL inventory feature-flag residual in v1. |
| **F26 — Metrics** | `bridge_roots_*`, `symbols_returned`, `symbols_ingested`, `symbols_truncated_inventory`, `symbols_truncated_by_ingest_cap` (when env max &lt; returned). |
| **F27 — MADR + symbols** | Same root loop; both `current_dir(root)`; pin MADR call site in `ingest_madr_from_ledgerful`. |
| **F28 — Order** | Roots sorted `normalized_path` ASC. |
| **F29 — Sequential** | One root at a time. |
| **F30 — Env** | Global dotenv vault/models unchanged (T229). |
| **F31 — unregister-path** | Soft residual / compensating event. |
| **F32 — detect** | Path alias continues to feed scope detect (existing CP). |
| **F33 — test-alias hygiene** | Separate; do not auto-register test projects. |
| **F34 — Governance** | Ship: deferred + coordinated note T233 consumes 0163. |
| **F35 — Perf** | Optional `AI_BRAINS_NIGHTLY_MAX_ROOTS`. |
| **F36 — Symbol source** | **Only** `ledgerful symbols --json` in v1. **Delete** `query_symbols_from_ledgerful` / `sqlite_table_exists` / direct ledger.db open in symbol_bridge. No SQL residual flag. |
| **F37 — Limit / multi-pass** | Nightly `--limit N` + `--pub`. If `truncated`, multi-pass: `read_dir` top-level dirs, `--path <dir>`, max recursion **depth 2**, concatenate+dedupe; still truncated → warn + partial. Never silent complete. |
| **F38 — Index freshness** | **Primary:** `--auto-index` on symbols. **No** separate `ledgerful index` / delete `refresh_ledgerful_index`. |
| **F39 — Pub filter** | `--pub` required for nightly default. |
| **F40 — Plan.md** | Expanded + AI fold-in. |
| **F41 — coordination** | On ship: note AI-Brains consumes 0163 JSON shape; schemaVersion 1. |
| **F42 — Go gate** | User **go** only (0163 already Completed). |
| **F43 — Ingest cap** | Remove `.take(500)`. Per-root max = `AI_BRAINS_NIGHTLY_MAX_SYMBOLS` default **5000** (match CLI hard max). Decline keep-500 default throttle. |
| **F44 — SymbolRecord** | Drop `method`/`path_pattern`; content non-route arm only; keep `symbol_in_project` as safety net for absolute out-of-root paths. |

## 5. Acceptance criteria

| ID | Criterion |
|----|-----------|
| **AC0** | 0163 Available: `ledgerful symbols --help` + pure JSON smoke (**plan-time PASS**) |
| **AC1** | `register-path` → `project list` path non-null |
| **AC2** | Win + WSL dual normalize to same project |
| **AC3** | Bridge uses explicit root; process cwd System32 alone does not zero registered roots |
| **AC4** | Phase 2 calls **0163 inventory** per root |
| **AC5** | Missing index/root → skip + warn, exit 0 |
| **AC6** | Phase 1 runs if Phase 2 all skip |
| **AC7** | Zero aliases → no-op + register-path hint |
| **AC8** | Per-root failure continues |
| **AC9** | Docs + CHANGELOG |
| **AC10** | Full CI green |
| **AC11** | No production unwrap/expect |
| **AC12** | Live dogfood: root has `ledgerful init` history; register `C:\dev\AI-Brains` + Phase 2 → symbols &gt; 0 |
| **AC13** | F21: second project registering same path → exit **1** + ownership message; same project re-register OK |
| **AC14** | No `.take(500)` silent drop; default cap 5000; SQL inventory helpers deleted from symbol_bridge |

## 6. Implementation plan

See [plan.md](./plan.md) Phases 0–7 (TDD) + AI fold-in pins. Summary:

0. Ledger + impact  
1. `register-path` CLI + F21 conflict  
2. `list_path_aliases` + `find_path_alias_owner`  
3. symbol_bridge: **delete SQL**, 0163 JSON, multi-pass, F43 cap  
4. nightly Phase1/2 + MADR `current_dir` + alias attribution  
5. soft bootstrap scan  
6. docs (`ledgerful init` note)  
7. review (incl. **cross-model ARCHITECTURE**) / gate / AC12 dogfood  

## 7. Risks

| Risk | Mitigation |
|------|------------|
| `truncated` on huge monorepos | F37 multi-pass depth 2; `--pub` first |
| WSL unreadable from Windows | Dual-checkout under `C:\dev`; F16 |
| Projection UPSERT steals path | F21 pre-check + exit 1 |
| Labels mistaken for paths | Docs: set-alias ≠ register-path |
| Route metadata lost vs SQL join | Soft residual; F44 |
| Scheduler Start-in empty | In-process roots only |
| Event-store growth at 5000/root | F43 env lower; idempotent `symbol_already_ingested` |
| New root without `ledgerful init` | Docs + skip+warn on indexStatus |

## 8. Operator runbook (after ship)

```powershell
# Once per root (examples)
ai-brains project register-path <id|alias> C:\dev\AI-Brains
ai-brains project register-path <id|alias> C:\dev\ledgerful
# dual-checkout optional second form:
# ai-brains project register-path <id|alias> \\wsl.localhost\Ubuntu\home\ryan\dev\...

# Manual inventory check (0163)
cd C:\dev\AI-Brains
ledgerful symbols --pub --json --limit 50

# Nightly (scheduler) runs Phase1+Phase2
ai-brains nightly
ai-brains nightly --status
```

## 9. Open questions — **closed at plan + AI fold-in**

1. Hard-require 0163? → **Yes; delete SQL inventory path.**  
2. Bootstrap `--from-scan` v1? → **Soft only.**  
3. unregister-path v1? → **Soft residual.**  
4. Flags? → **`symbols --pub --json --limit N --auto-index [--path …]`.**  
5. `take(500)`? → **Raise to 5000 / F43 env (not keep-500 default).**  
6. F21 check site? → **QueryStore `find_path_alias_owner`; exit 1.**  
7. Multi-pass depth? → **2.**  
8. MADR empty project_id? → **Alias owner (F12).**

## 10. Definition of Done

- [x] 0163 Completed (AC0)  
- [ ] F0–F44 + AC1–AC14  
- [ ] AI fold-in pins honored  
- [ ] Review (cross-model ARCHITECTURE) + full gate + manual AC12  
- [ ] conductor / deferred / pin updated  

## 11. AI fold-in summary (`C:\dev\AI-review.md`)

| Absorb | Decline |
|--------|---------|
| M1 F21 pin (location, exit 1, message, non-atomic note) | Default ingest throttle 500 (AI2 option b as default) |
| M2 delete SQL helpers + SymbolRecord route fields | SQL feature-flag residual |
| M3/F43 raise cap to 5000 + metrics | Forced clap/rusqlite bumps |
| M4/L4 MADR `current_dir(root)` | Concurrent atomic F21 (CLI-only OK) |
| L1 alias\|UUID; L1-init docs; L2 indexStatus; L3 multi-pass; L5 MADR pid; L6 safety net | — |
| O1 JSON unit tests DoD; O9 cross-model; O12 `--auto-index` only | O2 list-paths as hard DoD |

Full disposition: [plan.md](./plan.md) § AI fold-in.

---

**User go received — implementation in progress.**
