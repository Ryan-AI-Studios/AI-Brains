# T285–T300 — Post-T283 live CLI quality (placeholders)

**Source:** Non-destructive CLI audit **2026-08-22** on PATH graph-on `ai-brains` **0.1.2** (`ae5f6fd` `#200`). Live vault `C:\dev\ai-brains\vault.db`; Scope `3581317d`; grants **3 of 3**; leftover `7d97a456` still ~18k pins / **5** roots after partial rebind. Agent non-TTY.
**Status:** **T285 Completed** 2026-08-22 (`#201`). **T286 Completed** 2026-08-23 (`#202`). **T287 Completed** 2026-08-23 (`#203`). **T288 Completed** 2026-08-23 (`#204`). **T289 Completed** 2026-08-23 (`#205`). **T290 Completed** 2026-08-23 (`#206`). **T291 In Progress** 2026-08-23 (FEATURE TX `585cee1d-d763-4b87-a4e9-23ce2ff32526`). T292–T300 still **Placeholder / Pending**. Full F-list on `/plan-track TNN`. **Do not implement Placeholders.**
**Prior closed series:** T274–T284 (closer T283 `#199`). T274 ranking shipped but **live recall/Index still Q=4**.
**Ledger (registration):** DOCS TX `36224860-4861-4d1d-b0b0-058911971142`. **T285 plan:** DOCS TX `515b984b-7f5e-4386-9566-a292efd3afe1`. **T285 fold-in:** DOCS TX `3a598eff-b7e5-4158-970b-be5e331006a7`. **T286 plan:** DOCS TX `397f9c55-5953-402b-95fc-db431f5a037c`. **T286 fold-in:** DOCS TX `0eea671d-b8c3-4209-9e6b-31764707efdf`. **T287 plan:** DOCS TX `673e7322-b68f-40dd-bd34-6a91a83e7412`. **T287 fold-in:** DOCS TX `35a4042f-dd4a-40fc-b81a-6e34fdb7d903`. **T288 plan:** DOCS TX `6bf1d41c-a2c6-4b86-8b4b-2dee14690363`. **T288 fold-in:** DOCS TX `90e5e1d2-683d-4d62-baf1-4f821d423561`. **T289 plan:** DOCS TX `25bbc580-99a6-4969-8ea5-d0e1902d374e`. **T289 fold-in:** DOCS TX `45277700-a110-4f91-911b-8f921173dfdb`. **T290 plan:** DOCS TX `c66b1485-a4a7-4ca6-87d2-8b2e2d8b5865`. **T290 fold-in:** DOCS TX `8875a1cc-fba7-49a3-8026-dff1a033ddd6`. **T291 plan:** DOCS TX `c59e5bb6-adf1-40c5-9288-66403d208aca`. **T291 fold-in:** DOCS TX `627d3871-b5c6-4e03-8b11-9588a61777d1`.
**last-PR Cursor:** [#206](https://github.com/Ryan-AI-Studios/AI-Brains/pull/206) T290 Bugbot **1 Low** (`sanitize_recall_query` interpolator collapse) → **T291 F16**. **No T301.** (#205 / #204 / #203 / #202 / #201 / #200 empty.)

Scores below are **Usefulness / Quality** from that audit (1–10). Every command with **U&lt;8 or Q&lt;8**, plus every “doesn’t work,” friction, and significant-opportunity item, maps to **exactly one** track unless **declined**.

## Audit → track map

| Finding | U/Q or class | Track | Pri |
|---------|--------------|-------|-----|
| `recall` / `search` / `--semantic` rank review-track / `## Objective` over pins; unique DECISION pin not in top-3 | 10/**4**, 10/**4**, 8/**4** | **T285** | P0 |
| `sync query` vault half same dumps (ledger pane already found T187) | 9/**7** | **T285** (vault arm only) | P0 |
| `preflight --pretty` Session + Index still `## Objective`; `--summary` in-context decisions **0** next to 3647 pins | 7/**5**, 8/**7** | **T286** | P0 |
| `memory list --limit 5` is just-now ingest, not pins | 8/**6** | **T287** | P1 |
| `briefing project` granted-empty `_None_` feels like empty vault (H2 still declined) | 7/**7** | **T288** | P0 |
| Dual model: briefing/progressive = Approved; pins only via recall | friction | **T288** | P0 |
| `briefing personal` deny + `_None_` prefs | 4/**7** | **T289** | P2 |
| `evidence` / `source` / `review` list + `query progressive` granted-empty U=6 | 6/**8** | **T290** | P1 |
| `query trace` prints `null` | 3/**8**; friction | **T291** | P2 |
| `policy check` JSON-only | 7/**8** | **T292** | P2 |
| `graph neighbors` PREVIEW filled but neighbors are dump sessions | 7/**8** | **T293** | P1 |
| Leftover 5 roots dest-missing; `context` already-initialized skips vault upsert | not working / opp | **T294** | P1 |
| No usable encrypted backup (22/22 FAIL; doctor warn) | not working / opp | **T295** | P1 |
| Nightly Router `Ready` + `SCHED_S_TASK_TERMINATED` dual-truth | 8/8 nightly; friction / opp | **T296** | P2 |
| `daemon status` Stopped vs llama.cpp `:8081` Open | friction | **T297** | P2 |
| `device status` / `replicate status` optional empty | 5/**8**, 5/**8** | **T298** | P2 |
| `forget --list-forgotten` honest empty U=6 | 6/**8** | **T299** | P2 |
| Graph sparse E/N ~0.14 (floors honest; rebuild Stop-Before) | not working / opp | **T300** | P1 |

## Declined (written — not minted)

| Item | Why |
|------|-----|
| `doctor` / `daemon status` / `detect` / `whoami` / `list` cwd-first / `--show` leftover / `harness` / `retention plan` / `backup list\|verify` / `scope` / `adopt-path` / `scan-roots` / `bootstrap --dry-run` / `--help` | U≥8 **and** Q≥8 on 2026-08-22 |
| Raise 750 ms / unify daemon TCP with HTTP | **T255 F18** / **T269** / **T281** |
| T240 F2 silent Scope switch | Standing |
| T263 H2 pin → Approved | Standing — T288/T290 populate **without** promotion |
| T278 density floor retune | Honest sparse; **T300** is live rebuild owner-confirm, not floor change |
| clap 5 / rusqlite 0.40 / DTO new required keys | Standing |
| T274 two-pass as sufficient | **Reopened as T285** — live still Q=4 |
| last-PR #200 / #201 / #202 / #203 / #204 / #205 Cursor | **N/A** empty |
| last-PR #206 Cursor Bugbot Low sanitizer collapse | **T291 F16** (not a new T301) |

## Suggested implement order

1. **T285** (daily brain) then **T286** (Index/summary) then **T288** (briefing useful)
2. **T294** leftover dest upsert (unblocks 5 roots without `.env` rewrite)
3. **T287** / **T290** / **T293**
4. **T295** / **T300** live ops (owner-confirm)
5. **T289** / **T291** / **T292** / **T296** / **T297** / **T298** / **T299**

Do **not** `/implement-track` a Placeholder. Run `/plan-track TNN` first for T292–T300. T291 is **Planned** (Pending until go). T290 is **Completed**.

## Non-goals of this series

Live `retention apply --confirm`, CE, `migrate governed`, pin→Approved, clap 5, silent `.env` rewrite, schtasks mutate, graph default-on Cargo.
