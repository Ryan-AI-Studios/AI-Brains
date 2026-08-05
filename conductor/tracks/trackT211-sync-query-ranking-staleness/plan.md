# T211 Plan — Sync query ranking + stale DECISION demotion

Status: **Completed** (PR #94 squash-merged `16990b1`). Spec: [spec.md](./spec.md).

## Phases

### Phase 0 — Plan

- [x] Live re-scan + online research
- [x] Spec F1–F42 + AC1–AC12 (incl. AI fold-in)
- [x] AI fold-in disposition §14 (M1–M6 accept; L1–L2 elevate; L3 soft)
- [x] User **go** → ledger start (`d9de5d2c-f15b-41d5-842f-d24ab55077b3`)

### Phase 1 — Red (TDD)

- [x] Pure `ranking` units: classify kind/staleness, manual track tokens (no regex), sibling demotion, None→0.0 composite, memory_id ties (AC1/AC2/AC4/AC5)
- [x] **AC1b:** Shipped base −0.5 beats Plan base −3.0 under F9
- [x] Hermetic scaffold: two Decision pins order + pretty badge (AC1/AC3)
- [x] Isolation regression baseline still green (AC7)

### Phase 2 — Green

- [x] `lexical.rs` + substring: `updated_at` on `RetrievalMemory`
- [x] `ranking.rs`: pure helpers + F9 consts + `extract_track_tokens` manual + `rerank_hits`
- [x] `RecallHit`: `updated_at`, plan/staleness flag; plumb **all** constructors + graph SQL (F38)
- [x] `recall_full`: after blend/graph, before truncate → `rerank_hits` (F8 single composite)
- [x] **F37:** extract shared pretty render helper; `recall::run` + `sync::run_query` use it; preserve T207 empty
- [x] **sync.rs:** direct `recall_full` (not `recall::run`); F12 `--json` probe + ledger-first + banner; **F27 `--limit` default 5**; badge via shared helper
- [x] Soft F26: additive contracts `staleness` if free

### Phase 3 — Docs + residuals

- [x] CAPABILITIES (re-rank + plan demotion + ledger-first + heuristic honesty + F40 entry point)
- [x] CHANGELOG minor
- [ ] Soft OPERATIONS / skill (not shipped)
- [x] deferred.md: strike T211 on ship; note F25 blend residual

### Phase 4 — Review + gate

- [x] Internal review vs spec; fix medium+ (R1 BM25 polarity, R2 F12 secondary, R3 F33 unit)
- [x] Cross-model Claude **PASS WITH DEFERRED P3** (Codex rate-limited)
- [x] Manual evidence (path TOCTOU — shipped first + `[plan/stale?]`)
- [x] Soft AC12 hermetic `--limit 1`
- [x] Full CI on PR green; squash-merge `16990b1`

## Absorbed audit / AI fold-in

| Item | Handling |
|------|----------|
| quality 5 / stale DECISIONs | F1–F14, F11 |
| Prefer ledger when contradicts | F12 + F37 |
| M1 regex vs F18 | Manual track scan F6 |
| M2 None score | F8 None→0.0 single composite |
| M3 inspect hits | F37 direct recall_full |
| M4 ledger detect | `--json` probe F12 |
| M5 updated_at sites | F16/F38 |
| M6 magnitude | AC1b/AC11 |
| L2 `--limit` | F27 DoD |
| Semantic | Out → T215 (F40) |

## Touch map (implement)

| File | Change |
|------|--------|
| `crates/ai-brains-retrieval/src/ranking.rs` | **New** pure ranking |
| `crates/ai-brains-retrieval/src/lexical.rs` | `updated_at` FTS + substring |
| `crates/ai-brains-retrieval/src/recall.rs` | plumb + `rerank_hits` + constructors |
| `crates/ai-brains-retrieval/src/lib.rs` | exports |
| `crates/ai-brains-cli/src/commands/sync.rs` | F37/F12/F27 |
| `crates/ai-brains-cli/src/commands/recall.rs` | shared pretty + badge; T207 empty keep |
| `crates/ai-brains-cli/src/main.rs` | SyncCommands::Query `--limit` |
| `crates/ai-brains-cli/tests/sync_query_ranking.rs` | hermetic AC |
| `crates/ai-brains-retrieval` unit/tests | pure AC + AC1b |
| Soft: `ai-brains-contracts/src/recall.rs` | F26 |
| `Docs/CAPABILITIES.md`, `CHANGELOG.md` | honesty |

## Structural note (M3) — implementer must not miss

Current pretty path:

```text
run_query → recall::run(...)  // prints internally; returns Ok; hits invisible
         → ledgerful ledger search  // always after vault
```

Required:

```text
run_query → recall_full(...) → hits in hand
         → if !no_bridge: ledger search --json → non_empty?
         → if non_empty && top_is_plan: print ledger then vault else vault then ledger
         → shared pretty render(hits) with [plan/stale?]
```

## Ledger (on go)

```powershell
ledgerful ledger start T211-sync-query-ranking-staleness --category FEATURE --message "Pin-type+recency re-rank; plan DECISION demotion; ledger-first; sync --limit; no regex dep"
```

## DoD checklist

- [x] AC1–AC9 + AC11 met (AC12 hermetic shipped; AC10 soft/manual)
- [x] No production unwrap/expect; no regex in retrieval
- [x] Isolation + T207 empty regression green
- [x] CAPABILITIES + CHANGELOG
- [x] Review clean for >low (Claude PASS WITH DEFERRED P3 → deferred.md)
- [x] Full gate green; conductor Completed; PR #94 `16990b1`

## Explicit non-work

- T215 semantic RRF
- Progressive query rewrite
- Auto-forget pins
- clap 5 / new crates / rusqlite 0.40
- T212–T214, T216
- Full ledger blend (F25)

## Residual after ship (expected)

- Soft full ledger↔vault blend (F25)
- Soft F26 JSON staleness if not shipped
- Semantic relevance → **T215**
- T207 AC10 non-empty Scope still open
