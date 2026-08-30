# T327 review log — Preflight authority-first sections

**Track:** T327-PreflightAuthorityFirst  
**FEATURE TX:** `cc2212aa-001a-4a68-ae4a-f547f06afce3`  
**Product HEAD at start:** `5a6dffc` (T326 `#248`)  
**Branch:** `track/T327-preflight-authority-first`

## Scope

Retrieval Index drain never full-body-`break`s; non-global `INDEX_SLOT_CAP=15`; `ORDER BY updated_at DESC, memory_id ASC`; Session Other cap 3 + `+K more session turns via recall`; Recent prefer-authority two-pass; Index F4 honesty line; CLI pretty F56 overflow recognizer + F57 F4 prologue. Docs: CAPABILITIES / PROTOCOL-COMPAT / CHANGELOG. No `list_authority_memories`. No capture filter. No clap/rusqlite/workspace bump.

## Internal review (implementer)

| AC | Status | Evidence |
|----|--------|----------|
| AC1 | met | Hermetic whale + small pin: Index 1 is `DECISION:` (not Objective); needle still in window. Red on body-`break`. |
| AC2 / AC8 / AC15 | stay-green | T274/T286 Index hermetics + session Paris in `test(preflight)` 153/153. |
| AC3 | met | F4 SOOT exactly once when no authority. |
| AC4 | met | Pin first; numbered Index ≤ 15. |
| AC5 | met | `ORDER BY m.updated_at DESC, m.memory_id ASC`. |
| AC6 | met | Same-tick lex-smaller `memory_id` is Index 1. |
| AC7 | met | Session LIMIT-5 window: USER kept, Other ≤ 3, `+K` notice. |
| AC9 | met | `max_words=1500`; needle after `--- Most Recent Memories ---`. |
| AC10 | met | Dumps-only Recent; F4 not in Recent slice. |
| AC11 | met | Sealed needle absent (privacy path unchanged). |
| AC12 / AC13 | met | CLI JSON keys frozen; Index item `DECISION:`; summary T220. |
| AC14 / AC24 | met | Pretty overflow own line (standard+compact); F4 before `1.`. |
| AC16 / AC18 | met | CAPABILITIES compact-after-retrieval; PROTOCOL-COMPAT item-string; CHANGELOG. |
| AC17 | met (observed) | `cargo run --pretty -m 1500`: F4 prologue + recency fill (no fitting in-scope leading DECISION). Session `+K more session turns via recall`. Summary still `in_context_decisions: 0` + T315 next (F12). |
| AC19 | met | `git diff --stat` — no `project.rs` / `sync.rs` / `forget.rs` / `governed_common.rs` / `session_chrome.rs` / `ranking.rs` / `query_store.rs`. CLI `preflight.rs` is F56/F57 only. |
| AC20 | met | No new env / clap flags. |
| AC21 | stay-green | empty_repo / T315 scope-none in preflight suite. |
| AC23 | met | `INDEX_SLOT_CAP=15` after global arm; `preflight_global_isolation__three_a_one_b__b_appears_a_capped` green. |

### Findings

| id | severity | description | status | evidence |
|----|----------|-------------|--------|----------|
| R1 | low-info | Live vault Index still recency-fills after F4 (no fitting leading-line DECISION in `3581317d` window). | deferred | AC17 pass-with-observed-data; hermetic AC1 SoT. Do not H2. |
| R2 | low-info | PATH `ai-brains` still 0.1.3 until owner `cargo install`. | deferred | F28. |
| R3 | low-info | Session fetch `LIMIT 5` (`sessions.rs`) — AC7 adapted to that window; do not raise the cap (F19). | deferred | Spec wanted 8 Other; live loader is 5. F2 proven on loaded turns + companion authority-assistant hermetic. CAPABILITIES names the window. |
| R4 | low-info | AC1 line-1 needle vs F44 no body-skip: whale (newer) is Index 1; small pin still enters. Body-size skip not used. | deferred | F44 freeze (OpenCode m4); still red-on-Objective / green-on-DECISION. |

No critical/high. No mediums. No placeholders.

## Targeted gates (observed)

- `cargo fmt --check` exit 0
- `cargo clippy -p ai-brains-retrieval -p ai-brains-cli --all-targets -- -D warnings` exit 0
- `cargo nextest run -p ai-brains-retrieval -p ai-brains-cli -E "test(preflight)"` **153 passed** (pre-companion); companion + tightened AC4/AC13 later **pass**
- last-PR `#248` Cursor comments **0** / issue comments **0**
- clap lock **4.6.1**; crates.io **4.6.6**; no bump
- `ledgerful scan --impact` HIGH (CLI `preflight.rs` hotspot #7 expected); F19 files untouched

## Full gate AC22

- `.\scripts\dev-check.ps1` **[SUCCESS]** — nextest **3716 passed** (1 skipped), deny, audit
- `ledgerful verify --scope full` exit **0** (`overallPass: true`)
- AC17: `cargo run --pretty -m 1500` Index F4 prologue then recency; `--summary` `in_context_decisions: 0` + T315 next
- Hermetic ×2: `preflight__index_same_tick` PASS then PASS

## Cross-model (Codex FAIL → dispositions)

`review.codex.md` FAIL at in-progress tree.

| Finding | Disposition |
|---------|-------------|
| P1 AC7 LIMIT 5 | **out_of_scope** — F19 forbids `sessions.rs`. F2 applies to loaded turns. Companion hermetic locks authority-assistant uncapped. CAPABILITIES documents latest-5. |
| P1 provenance/publish | **fixed_pending_verification** — closeout commits + Phase 6 publish |
| P2 AC1 vs F44 | **out_of_scope / already F44** — OpenCode m4; whale is Index 1; small pin in window |
| P2 weak AC4 | **verified_fixed** — `numbered.len() == 15` |
| P2 weak AC13 | **verified_fixed** — require `in_context_decisions >= 1` |
| P2 AC17 ×2 | **verified_fixed** — AC6 hermetic run twice, both PASS |

Internal explore review: **PASS WITH DEFERRED P3** (same LIMIT-5 / F44 notes). Easy P3s tightened.

## Completion

Engineering DoD met. Residuals in `conductor/deferred.md`. Publish Phase 6 next.
