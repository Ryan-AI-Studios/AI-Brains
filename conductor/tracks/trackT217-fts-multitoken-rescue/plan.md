# T217 Plan — FTS multi-token / natural-phrase rescue

**Status:** 🚧 Implementing (phases 1–4 code complete; verify/review pending coordinator)
**Category:** FEATURE / UX
**Depends:** T90 ✅, T105 ✅, T111 ✅, T112 ✅, T140 ✅
**Spec:** [spec.md](./spec.md) — includes AI fold-in §14 (2026-08-09)

## Goal

1. Stop natural-language multi-token FTS from false-empty when contentful keywords exist.
2. SOOT ladder (stopword AND → OR) **opt-in** via `rescue`; **`recall_full` only**.
3. **SQL LIMIT** on all MATCH paths (hard cap 200 / `candidate_depth`).
4. **forget stays strict R0** (no ladder — no destructive match widening).
5. Empty-hint “try fewer keywords” via **core** token helpers (contentful ≥ 1).
6. Align token split with unicode61 (**split `_`**).
7. Hermetic proof + docs; no auto-semantic; no dep bumps.

## Absorbed deferred / audit / AI fold-in

| Source | Item | Handling |
|--------|------|----------|
| deferred.md T217 | FTS natural-phrase empty (quality 4) | Hard DoD |
| Series README | P1 honesty — empty FTS trap | This track |
| Placeholder F1–F5 | Expanded D1–D14 + F1–F22 + AC1–AC17 | spec.md |
| T105/T111 order | Rescue **before** substring + hint | F11 |
| Double-sanitize | Raw query into lexical; match path no re-sanitize | F9/F10 |
| Dogfood 2026-08-09 | multi-token empty / two-token hits | AC1 freeze |
| **AI1 M1** | OR unbounded → LIMIT | **D13/F19/AC15** hard |
| **AI1 M2** | forget destructive inherit | **D9/F16/AC14** rescue opt-in |
| **AI1 M3** | negators + literal stopwords | **§4.1/AC6b** |
| **AI1 M4** | `_` vs unicode61 | **D14/AC16** split `_` |
| **AI1 M5** | hint SOOT + contentful≥1 | **D7/AC7b** |
| **AI1 M6** | non-stopword 3+ OR | §2 + manual dogfood |
| **AI1 M7** | UUID forget ladder | free with M2 |
| **AI1 M8** | NEAR / tokenchars / fts5vocab | soft residual |
| **AI2** dedupe contentful | redundant OR clauses | **D3/AC17** |
| **AI2** emoji hint | 💡 | **Rejected** (plain text) |

**Not absorbed:** T218 semantic v2; T224 role strip; T231 unified UX; bridge multi-round; control-plane FTS; clap5/rusqlite 0.40; stemmer/trigram migration; FTS rebuild.

## Live dogfood freeze (2026-08-09)

| Query (`--no-bridge`) | Observed |
|----------------------|----------|
| `what did we decide about forget list` | empty + T111 hint |
| `forget list` | FTS hits (T216 pins) |
| `forget` | FTS hits |

## Research freeze (2026-08-09)

| Topic | Note |
|-------|------|
| SQLite FTS5 | Implicit AND; explicit OR; quote safety; unicode61 `_` separator |
| LIMIT gap | lexical_search no LIMIT today → M1 |
| forget --match | 1 hit + --force deletes → M2 |
| Pins | rusqlite **0.39**, clap **4.5** — no bump |

## Rescue ladder (implement exactly)

```
opts.rescue, opts.limit   # recall: true + candidate_depth; forget: false + cap
tokens = extract(raw)     # split non-alnum including '_'
R0: AND(all) + LIMIT
if rescue && empty && |tokens| >= 3:
  c = contentful(tokens)  # stopwords §4.1, len>=2, dedupe, KEEP negators
  R1: if c non-empty && c ≠ tokens → AND(c) + LIMIT
  R2: if empty && |c| >= 2 → OR(select_or(c)) + LIMIT   # max 8 tokens
if still empty (recall path) → T105 substring(raw) if vault ≤10k
if still empty → T111 hint (+ fewer keywords if |tokens|≥3 && |c|≥1)
```

## Phases

### Phase 0 — Plan freeze

- [x] Preflight / doctor / ledger status
- [x] Live dogfood multi-token empty vs short hits
- [x] Code map (sanitize, lexical, recall, hint, forget)
- [x] Online FTS5 boolean / AND research
- [x] Spec D1–D12 + F/AC (initial)
- [x] deferred.md + conductor → **Planning**
- [x] series README status note
- [x] `ai-brains pin` plan-start
- [x] **AI review fold-in** M1–M8 / AI2 → D13–D14, F16/F19–F22, AC6b/7b/14–17, §4.1 stopword list
- [x] `ai-brains pin` AI fold-in
- [x] User **go** (Implement track 217)

### Phase 1 — Ledger + red (after go)

- [x] `ledgerful doctor` ; `ledgerful ledger status --compact`
- [x] `ledgerful ledger start T217-fts-multitoken-rescue --category FEATURE --message "FTS multi-token rescue: stopword AND + OR ladder, SQL LIMIT, rescue opt-in (recall only; forget strict)"` — TX `03f22a3e-979c-4706-8cc5-278c316edd46`
- [x] `ledgerful scan --impact`
- [x] Red pure: extract `_` / stopword / negator / dedupe / match_and / match_or / select_or (AC5/6/6b/16/17)
- [x] Red hermetic: AC1 rescue=true; **AC14 rescue=false**
- [x] Red: AC15 LIMIT bound
- [x] Red CLI: AC7 + **AC7b** all-stopword

### Phase 2 — Core pure builders

- [x] extract (split `_`), stopword §4.1, contentful, match_*, select_or
- [x] `sanitize_fts_query` = match_and(extract(...)) — update existing tests for `_`
- [x] Pure tests green

### Phase 3 — Lexical ladder + recall wire

- [x] `LexicalSearchOptions { rescue, limit }` (or equiv)
- [x] `match_query` + **LIMIT** on all stages
- [x] ladder R0→R1→R2 only when rescue
- [x] `recall_full`: raw + rescue=true + candidate_depth
- [x] `forget`: rescue=false (explicit or default)
- [x] Debug tracing (F13)
- [x] Hermetic AC1/AC3/AC4/AC14/AC15 green
- [x] Privacy + scope (F18)

### Phase 4 — CLI hint + docs

- [x] Hint uses core extract/contentful (AC7/AC7b)
- [x] CAPABILITIES + CHANGELOG (AC12)
- [x] Optional FTS5-catch one-liner

### Phase 5 — Verify / close

- [x] Focused nextest + clippy packages
- [x] Manual dogfood (incl. M6 non-stopword 3+; forget dry-run no widen)
- [x] Full gate
- [x] Internal review.md clean
- [x] Cross-model FEATURE review (Codex R2; re-run after stage)
- [x] PR + CI + merge (#110)
- [x] Conductor **Completed**; deferred T217 closed; pin closeout

## Soft residuals (not DoD)

- Bridge multi-round rescue
- Control-plane evidence FTS
- JSON rescue-stage field
- Locale stopwords
- Porter/trigram/`tokenchars`/fts5vocab (rebuild)
- FTS5 `NEAR` R2.5
- T224 / T218 / T231

## Stop-before

- User **go** required before implement
- No force-push / main push
- No dep upgrades without explicit ask
- Do **not** enable rescue on forget
- Scope creep into T218 / T231

## Manual proof commands (after green)

```powershell
ai-brains recall "what did we decide about forget list" --no-bridge --limit 5
ai-brains recall "forget list" --no-bridge --limit 5
ai-brains recall "brittle hotspot fix" --no-bridge --limit 5
ai-brains recall "zzzz_no_such_token_aaa bbb ccc" --no-bridge
ai-brains forget --match "what did we decide about forget list" --dry-run
```

