# T285 Plan — recall rank v2 (envelope + detector + chrome-seed skip)

**Status:** **Completed** 2026-08-22. Full F-list in spec.md.
**Spec:** [spec.md](./spec.md) F0–F37 / AC1–AC17 + §13 AI fold-in
**Category:** FEATURE / UX / RETRIEVAL
**Ledger TX (planning):** `515b984b-7f5e-4386-9566-a292efd3afe1` (DOCS)
**Ledger TX (fold-in Agy+OpenCode):** `3a598eff-b7e5-4158-970b-be5e331006a7` (DOCS)
**Ledger TX (implement):** FEATURE `ac3da53e-1eea-4539-aaa7-054808fb35a3`

---

## AI fold-in (2026-08-22) — `agy-review.md` + `opencode-review.md`

Agy **B 0 / M 0**. OpenCode **B 0 / M 3** (all three **folded**). Disposition in spec **§13**.

### Pins locked by fold-in

1. **AC4/AC5:** dump **bodies contain the unique needle** (MATCH + high BM25); first line stays live chrome. Needle-absent dumps are not red (OpenCode M1; Agy O2 declined).
2. **F10/M3:** skip from `blended.content`, not `(id, score, kind)`.
3. **F36/AC6:** graph-off unit helper `parent_seeds_graph_neighbors`.
4. **F37/AC17:** CLI `test(graph)` hermetic; existing CI filter. No `ci.yml` retrieval extra line (O1 declined).
5. **F7:** post-retain gate (live `:180` is pre-retain today).
6. **F2/AC1:** `strip_prefix` + `trim_start`; empty → Other.
7. **F34:** recency-retry `NOT IN` bound (Agy m1).
8. **Affirm:** #200 N/A; T286/T287/T293 not stolen; O3 core ROLE_PREFIXES declined.

---

## Preflight (plan time — 2026-08-22)

| Check | Result |
|-------|--------|
| HEAD / tree | **Plan dogfood:** `76c4db9` mint T285–T300. CLEAN. Ahead of `origin/main` `ae5f6fd` `#200` by **1** (docs). Product `src/` = 0.1.2 |
| PATH `ai-brains` | **0.1.2** mtime 2026-08-22 19:41, 25 139 712 bytes. **Has T274.** Hole is in **source**. **Do not `cargo install`.** |
| `preflight --summary` | Pinned **3648**; in-context 5/0/0; grants **3 of 3**; Scope `3581317d` |
| PATH `recall "capture independence event log" --no-bridge --limit 5` | #1 `# AI-Brains Session Onboarding Complete` (−12.222); #2 `# Review of Track 254`; no leading `DECISION:` |
| PATH `search "DECISION:" --no-bridge --limit 5` | Chat crumbs mentioning DECISION; JSON `ASSISTANT:` not `ASSISTANT: DECISION:` |
| PATH `--semantic "SQLCipher page encryption"` | Honesty `no semantic hits above threshold; showing lexical` then T254 / `## Objective` |
| Last PR comments | #200 version bump — **empty** (N/A). **No T301.** #188 closed by T284 |
| Open PR on HEAD | none (Dependabot remotes only: rusqlite 0.40.2 `#61`, chrono 0.4.45 `#62`, …) |
| Pins | clap lock **4.6.1** (crates.io 4.6.6; **no clap 5**); serde_json **1.0.150** (1.0.151); chrono **0.4.44** (0.4.45); rusqlite **0.39.0** (0.40.2); uuid lock **1.23.1**; tokio **1.52.3** — **no bumps** |
| rustc / nextest / workspace | 1.95.0 / 0.9.140 / **0.1.2** |
| Hotspots | `project.rs` **#1** — do not touch. `sync.rs` #2 — do not grow. `session_chrome.rs` **#10** — extend. |
| Ledger | 0 pending / 0 drift at scan; planning TX `515b984b` |
| `ISSUES.md` | **Does not exist** (F28) |
| ledgerful search | `classify_pin_kind` `ranking.rs:101`; `is_session_chrome` `session_chrome.rs`; `sync.rs:529` inherits |
| Online | SQLite FTS5 bm25 column weights need a split (decline); Elastic Labs multiplicative (decline; +16 additive); APSW position_rank → first-line bonus; T260 stub-seed analog |
| Skill | `.agents` / `.claude` recall recipes unchanged except CAPABILITIES pin-type + graph-seed sentence (F31) |

---

## Phase 0 — on go (re-verify)

- [x] `git fetch --all --prune` ; if `origin/main` moved, reconcile (no rebase over user work; never `git push origin main`)
- [x] `ledgerful doctor` ; `ledgerful ledger status --compact` ; `ledgerful scan --impact` — work root `C:\dev\AI-Brains`; 0 pending / 0 drift (before FEATURE TX)
- [x] Re-read `ranking.rs` `first_contentful_line` / `classify_pin_kind` / `rerank_hits`
- [x] Re-read `session_chrome.rs` detector + `authority_glob_sql`
- [x] Re-read `lexical.rs` two-pass + F35 `NOT IN`
- [x] Re-read `recall.rs` graph loop `:492–552` + snapshot `:498–501` (no content) + T260 stub-seed `:484`
- [x] Confirm CI graph job is CLI-only (`ci.yml` `:109` / `:173`) — **do not** edit ci.yml
- [x] Confirm AC4 dumps **contain** the needle in bodies (OpenCode M1)
- [x] Confirm `pin.rs` still prepends `TAGS:` — **do not rewrite**
- [x] Confirm clap Recall `graph_hop_depth` default **1** — **do not zero**
- [x] Confirm T274 `tests/recall_pin_rank.rs` still uses `graph_hop_depth: 0` and needle-in-dump-bodies — **stay green**, do not “fix” into AC4
- [x] Rescan `conductor/deferred.md` — T285 rows absorbed; T286/T287/T293/T294 not stolen
- [x] Confirm #200 comments/reviews still empty (N/A); no mint; Dependabot `#61` still not this track
- [x] Re-dogfood `recall` / `search` / `--semantic` **read-only**. **Did not** pin production decisions; **did not** write `.env`
- [x] Re-check clap lock **4.6.1**, rusqlite **0.39.0**, chrono **0.4.44** — **no bump**
- [x] FEATURE TX (new) — category FEATURE
- [x] Did **not** `cargo install`; did **not** grow `sync.rs` / `project.rs` / CLI `preflight.rs` / `pin.rs` write

---

## Absorbed deferred

| Item | Plan action |
|------|-------------|
| Audit recall/search/semantic/sync-vault Q=4 | **DoD** F1–F12 / AC1–AC6 / AC12–AC14 |
| Placeholder Manual DoD + `--tag` canary | **DoD** F2 envelope; AC1/AC4/AC12; Manual below |
| T274 live dumps after install | **DoD** — PATH already 0.1.2 |
| T274 detector not rstest | F26 / AC2 new prefixes |
| T274 I1 ASSISTANT: + TAGS envelope | F2 |

## Declined (written)

| Item | Why |
|------|-----|
| Preflight Index / summary | **T286** |
| `memory list` ORDER | **T287** / T216 |
| `graph neighbors` CLI | **T293** |
| Leftover dest upsert / T276 F39 | **T294** / F23 |
| T263 H2 / T240 F2 / clap 5 / rusqlite 0.40 / raise depth | Standing |
| last-PR #200 Cursor | N/A empty — no T301 |

---

## Phase 1 — Red (required)

- [x] AC1 unit: TAGS envelope → Decision / Objective-after-TAGS → Other
- [x] AC2 rstest: onboarding Complete + `# Review of Track` true; `# Heading` false
- [x] AC3 `rerank_hits_with_query` pin first vs live onboarding chrome
- [x] AC4 retrieval hermetic: tagged pin vs 15 `# Review of Track` dumps **with needle in dump bodies** (chrome first line); pin #1
- [x] AC6 unit: `parent_seeds_graph_neighbors` false for chrome, true for DECISION
- [x] AC12 CLI hermetic pretty `recall` + `search` pin #1, EXIT 0
- [x] Commit red (allowed)

## Phase 2 — Green

- [x] F2 envelope in `ranking.rs` (`first_contentful_line` after role + TAGS skip)
- [x] F5 detector prefixes in `session_chrome.rs`
- [x] F6 `LEADING_QUERY_BONUS` inside single `rerank_hits` sort; `recall_full` passes query
- [x] F7 GLOB-or-TAGS + in-memory retain (post-retain `len`)
- [x] F8 recency retry when retain empty; F34 bound params
- [x] F10 skip chrome parents using `blended` content (not 3-tuple); F36 helper
- [x] AC5 untagged pin still #1 vs **same body-MATCH dumps**
- [x] AC17 CLI `test(graph)`: chrome parent does not seed a non-MATCH neighbor
- [x] AC13 `sync query` vault top = pin (hermetic; do not edit `sync.rs`)
- [x] AC14 semantic fallback top-3 pin (no HTTP required)
- [x] AC7–AC11, AC15, AC16 stay green
- [x] Commit green (allowed)

## Phase 3 — Docs + gate

- [x] CAPABILITIES pin-type row + graph-seed sentence (F31)
- [x] CHANGELOG T285
- [x] PROTOCOL-COMPAT: no new required keys
- [x] `cargo fmt --check`
- [x] `cargo clippy -p ai-brains-retrieval -p ai-brains-cli --all-targets -- -D warnings`
- [x] Targeted nextest (spec §7) including `cargo nextest run -p ai-brains-cli --features graph -E "test(graph)"`
- [x] Review log `review.md`; Phase-1 clean then `codex-review` (F27)
- [x] Full gate at closeout: `cargo fmt --check ; cargo clippy --workspace --all-targets -- -D warnings ; cargo nextest run --workspace ; cargo deny check ; cargo audit ; ledgerful verify --scope full`
- [x] Manual DoD (below) on `cargo run` / hermetic — **not** PATH until owner asks `cargo install`
- [x] conductor.md T285 **Completed** only after implement-track Phase 6 publish
- [x] Append residuals to `conductor/deferred.md`

---

## Manual DoD (on go — command-level proof)

Hermetic AC12/AC13 are SoT. After green, also run this unique canary (allowed F22 — uuid in the string; **not** a production decision pin):

```powershell
$needle = "T285 rank-v2 unique canary $(New-Guid)"
ai-brains pin "DECISION: $needle" --tag t285-canary
ai-brains recall $needle --limit 5 --format pretty --no-bridge
ai-brains search $needle --limit 5 --format pretty --no-bridge
ai-brains recall $needle --semantic --limit 5 --format pretty --no-bridge
ai-brains sync query $needle --no-bridge --limit 5
```

**Pass:** `recall` and `search` hit **#1 or top-3** contain `$needle` and do **not** start with `## Objective` / `# AI-Brains Session Onboarding` / `# Review of Track`. `--semantic` either surfaces the pin **or** honest `no semantic hits` **plus** the lexical pin in the fallback list. `sync query` **vault** half same pin proof; ledger pane unchanged (may be empty). Exit **0**.

If the owner declines a live-vault canary, hermetic AC12/AC13 **alone** close Manual — record that in `review.md`.

---

## DoD (track)

- [x] Rank-1/top-3 is the canary/hermetic pin, not session chrome
- [x] Tagged `ASSISTANT: TAGS:` pins classify and enter pass-1
- [x] Chrome parents do not seed graph neighbors
- [x] JSON keys unchanged
- [x] T274 / T260 / T207 / T261 / T216 tests stay green
- [x] Full gate green; FEATURE TX committed; implement-track Phase 6 published
