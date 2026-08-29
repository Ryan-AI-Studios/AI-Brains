# T316 Plan — memory list preview + forget nudge

**Status:** **Planned** (Pending until **go**). Spec [spec.md](./spec.md).
**Category:** UX
**Ledger (planning):** DOCS `66b597f7-faf9-4f3e-bb06-6af72811bdc6`
**Ledger (fold-in):** DOCS `69e50ba1-5c35-49d4-abb3-56f1ff6419c6`

---

## Preflight (plan time — 2026-08-29)

| Check | Result |
|-------|--------|
| HEAD / tree | Fold-in `120bbfa` plan commit CLEAN; `origin/main` = `d1c3bd3` (ahead **1**). Plan-write was `d1c3bd3` / ahead **0** (Agy m1). Branch `track/T316-memory-list-preview`. Product `src/` = T320 `#238`. |
| PATH `ai-brains` | **0.1.3** graph-on; **26,897,408** B; mtime **2026-08-27 8:21:55 PM**. T287/T316 **not** on PATH. Hole **is** (chrome first lines + F36 stderr). |
| `preflight --summary` (PATH) | Pinned **4568→4569**; in-context **0/0/0**; `Total Word Count: 728` (PATH-behind T315) |
| PATH `memory list --limit 5` | `## Objective` / review / ` ```json ` / dump prose; F36 stderr interleaves after Scope |
| Source `cargo run … memory list --limit 5` | **Also recency** (T287 R1-1 live): `## Objective` + session dumps. F36 stderr present |
| Source `memory list --format json --limit 1` | `items[0].preview` = `"## Objective"`; nine keys frozen |
| `preview_line` | `memory.rs:53–66` envelope only |
| F36 | `memory.rs:556–559` eprintln |
| rustc | **1.95.0** |
| Pins | clap `"4.5"` / lock **4.6.1** / crates.io **4.6.6**; rusqlite **0.40.2**; serde_json **1.0.150**; uuid ws `"1.13"` / lock **1.23.1**; workspace **0.1.3** — no bump |
| Last PR Cursor | `#238` empty. `#237` Bugbot medium `PinnedCountFailed` → **T326**. `#230` → **T325** already. |
| Open PRs | **none** |
| Ledger | 0 pending / 0 drift at scan; this TX `66b597f7` |
| Hotspots | `project.rs` #1 **3.681** — do not touch. `forget.rs` #5 — do not grow production. `session_chrome.rs` #6 — import only. |
| Line counts | `memory.rs` **721** nonblank; `forget.rs` **269**. F25 = go-HEAD diff. |
| `ISSUES.md` | **Does not exist** |
| Planning install / live pin | **Not run** |

---

## Absorbed deferred

| Item | Plan action |
|------|-------------|
| Audit memory list 6/6 preview + F36 | **DoD** F1–F10 / AC1–AC9 / AC17 |
| T216 F36 stderr | **Supersede runtime** F9 |
| T287 ORDER / JSON recency | **Freeze** F8 / F7 |
| T287 R1-1 GLOB empty | **Partial** F27 |
| T299 empty remediator | **Freeze** F11; update nonempty F36 AC9 |
| T319 `memory show` | **Decline** F13 |
| last-PR `#237` Cursor pin-count | **T326** — not stolen |
| last-PR `#230` F8 recency | **T325** — not stolen |
| T318 / T321 / T322–T324 / clap 5 | **Not stolen** / **Decline** |
| OpenCode m1 walk-stop | **F1/F3 / AC19** first-non-chrome |
| OpenCode m2 after_help hermetic | **AC14** named test |
| OpenCode O1 empty classify | **F3** |
| OpenCode O2 inherit smoke | **Partial** F6 helper units; decline extra briefing/graph hermetics |
| Agy m1 HEAD | **§2.1** `120bbfa` / ahead **1** |
| Agy m2 all-chrome | **Already** F5 / AC5 |

---

## Phase 0 — on go (re-verify + deferred rescan)

- [ ] `ledgerful doctor` ; `ledgerful ledger status --compact`
- [ ] Confirm cwd `C:\dev\AI-Brains`
- [ ] Re-read `preview_line` `:53–66` + F36 `:556–559` + `emit_list_json` `:441`
- [ ] Re-read `first_contentful_line` `ranking.rs:102` (do **not** edit) + `is_session_chrome` `:24–58` (import only)
- [ ] Re-read T287 mix `run_inventory` `:228–265` + `prefer_fill_authority` `:99`
- [ ] Re-read T299 `forgotten_empty_remediator` `:31` + empty arm `:488–508`
- [ ] Re-read hermetic F36 `:239–243` + T299 AC4 `:1410–1450` + T287 help `:1237`
- [ ] Re-read callers: `forget.rs:19–25`, `graph.rs:279`, `briefing.rs:77` (inherit-only)
- [ ] Re-dogfood `memory list --limit 5 --format human` + `--format json --limit 1` (source)
- [ ] Confirm clap lock still **4.6.1**; JSON keys still T216; Daily string already has `status`
- [ ] Rescan `deferred.md` open overlapping rows
- [ ] Confirm T325 / T326 / T318 still Pending (do not steal)
- [ ] `ledgerful ledger start T316-memory-list-preview --category FEATURE`
- [ ] **Do not** `cargo install` / live production `pin` / `.env` rewrite / clap 5 / `forget.rs` production growth

## Phase 1 — Red

- [ ] `preview_line__session_chrome_heading__skips_to_body` (AC1)
- [ ] `preview_line__let_me_verify__skips_to_next` (AC2)
- [ ] `preview_line__all_chrome__fallback_first_contentful` (AC5)
- [ ] `preview_line__authority_line__never_skipped` (AC6)
- [ ] `preview_line__fence_then_decision__keeps_decision` (AC19)
- [ ] rstest `preview_line__walk_cap__eight` (AC7)
- [ ] Hermetic `memory_list__nonempty__omits_f36_stderr` (AC8)
- [ ] Hermetic `forget_list_forgotten__nonempty__omits_f36_stderr` (AC9)
- [ ] Hermetic `memory_list__format_json__preview_skips_chrome` (AC11)
- [ ] Hermetic `memory_list_help__after_help__names_chrome_skip_and_no_forget_hint` (AC14)
- [ ] Confirm red tests **fail** on current tree (preview still `## Objective` / F36 stderr present)
- [ ] Confirm TAGS envelope + TAGS-only units **still pass** (AC3/AC4)

## Phase 2 — Green

- [ ] `PREVIEW_CHROME_WALK = 8` + `PREVIEW_AGENT_CHROME_PREFIXES` in `memory.rs` (F33)
- [ ] `skip_leading_preview_chrome` / `preview_line_is_chrome` (F1–F5)
- [ ] Wire `preview_line` after `first_contentful_line`
- [ ] Delete F36 `eprintln!` (F9)
- [ ] after_help one sentence (F26)
- [ ] Production net: `memory.rs` + after_help; do not grow `forget.rs` / `session_chrome.rs` / `ranking.rs`

## Phase 3 — Stay-green + docs

- [ ] AC3/AC4 TAGS units
- [ ] AC10 T299 empty remediator
- [ ] AC12 JSON recency order
- [ ] AC13 `prefer_fill_authority` rstest
- [ ] T287 help `:1237` stay-green (AC14 is Phase 1 red, additive after_help)
- [ ] AC15 CAPABILITIES / OPERATIONS / CHANGELOG
- [ ] AC16 empty diff: `forget.rs` / `graph.rs` / `briefing.rs` / `ranking.rs` / `session_chrome.rs` / `project.rs` / `sync.rs` / `query_store.rs`
- [ ] AC18 exit 2 / clap InvalidValue stay-green

## Phase 4 — Manual + gate

- [ ] AC17 `cargo run -p ai-brains-cli -- memory list --limit 5 --format human` (no F36 stderr; previews pass-with-observed-data)
- [ ] AC17 `cargo run -p ai-brains-cli -- memory list --format json --limit 1` keys frozen
- [ ] `cargo fmt --check` ; `clippy -p ai-brains-cli --all-targets -- -D warnings` ; targeted nextest `-p ai-brains-cli`
- [ ] Phase-1 review → `review.md` ; FEATURE cross-model `codex-review` (F22)
- [ ] Full gate before Complete; `ledgerful verify --scope full`
- [ ] conductor **Completed** only after squash-merge hygiene (implement-track Phase 6). Never `git push origin main`.

---

## Definition of Done (checkable)

- [ ] AC1–AC9 / AC11 / AC14 / AC19 red-then-green
- [ ] AC3/AC4/AC10/AC12/AC13/AC16/AC18 stay-green
- [ ] AC14–AC15 docs + after_help
- [ ] AC17 manual recorded (PATH-behind not a fail)
- [ ] No F36 stderr on nonempty human list
- [ ] No fake `next: forget --memory-id <id>`
- [ ] T287 ORDER / T216 JSON keys / T299 empty remediator unchanged
- [ ] No `memory show` / no new clap flag / no clap 5
- [ ] T326 not stolen
- [ ] Status stays **Pending** until implement Complete + PR merge

---

## Isolation (on go)

No `cargo install`. No live production pin. No `forget.rs` / `project.rs` / `sync.rs` / `session_chrome.rs` / `ranking.rs` hotspot growth. No `git push origin main`.
