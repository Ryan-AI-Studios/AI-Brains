# T313 — `sync query` ledger provenance (phrase vs rescue)

- **Track ID:** T313-SyncQueryProvenance
- **Status:** **Planned** (Pending until **go**) — **placeholder**. Full F-list on `/plan-track T313`.
- **Category:** UX / HONESTY
- **Owner:** Grok
- **Source:** Live CLI audit 2026-08-27 — `sync query` 8/**7**; ledger half silently downgrades phrase-miss → fuzzy token match.
- **Depends on:** T271 ✅ ledger pane + F6 token rescue + F7 banner; T273 ✅ POSIX `--` before QUERY
- **Blocks / feeds:** Unified recall+ledger daily. Vault rank remains **T312**.
- **Absorbs:** Audit “can’t tell which results came from where”; T271 F7 banner only-when-rescue — **reopen** if live output omits it or if rescue looks like a phrase hit
- **Not absorbed (DoD):** Vault ranking (T312); `sync.rs` hotspot growth beyond the ledger helper; clap 5; T92 pull/push
- **Research date:** 2026-08-27. Live `sync_query_ledger.rs`: `ledger_rescue_banner` exists; `LEDGER_RESCUE_TOKEN_CAP = 3`. Snapshot — re-verify at execute.
- **Ledger:** series DOCS TX `a6d3c404-1d64-4cba-a743-d75ac16c74cd`. Implement **FEATURE** TX on go.
- **Isolation:** Do **not** implement until go. Do **not** `cargo install`. Do **not** grow hotspot `sync.rs` (#2) — keep rescue copy in `sync_query_ledger.rs`. Do **not** print or commit `AI_BRAINS_KEY`.

---

## 1. Objective

1. **Phrase vs rescue is obvious.** When T271 token rescue produces hits, human (and JSON if a key already exists) must say the pane is a **rescued token**, not the user phrase.
2. **Vault vs ledger panes stay labeled.** Operator does not have to infer source by reading row text.
3. **Do not disable rescue.** Empty phrase → token rescue stays (T271 F6). Honesty, not a miss.
4. **North star.** Capture independence: CLI overlay on `ledgerful ledger search`. No new events.

---

## 2. Live baseline (mint 2026-08-27)

| Signal | Observation |
|--------|-------------|
| Code | `probe_ledger_search` sets `banner: Some(ledger_rescue_banner)` on token hit (`sync_query_ledger.rs`) |
| Audit | Live output still felt like a silent fuzzy `'graph'` match with 10 rows |
| `sync.rs` | Hotspot **#2** — do not grow |

---

## 3. Frozen until full plan

- **F0** plan-only until go.
- T273 `--` before QUERY stays.
- Do not FTS-quote ledger argv (T271 F5).

---

## 6. Non-goals

Changing Ledgerful search ranking. Vault BM25. Relay pull/push.

---

## 9. Deferred / last-PR

| Item | Disposition |
|------|-------------|
| T271 rescue residual | **Absorb** |
| last-PR `#229` | **N/A empty** |
| T307 | **Not stolen** |

---

## 12. Touch map (sketch)

`crates/ai-brains-cli/src/commands/sync_query_ledger.rs` (+ emit path in `sync.rs` only if banner is dropped). Tests in `sync_query_ledger.rs`.
