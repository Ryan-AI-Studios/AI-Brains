# T276 Plan — Leftover `7d97a456` must not starve `--global`

**Status:** **Completed** (FEATURE TX `6846ad81-4892-41fd-935c-82030dcaf0ac`)
**Spec:** [spec.md](./spec.md) F0–F41 / AC1–AC16 + §13 AI fold-in
**Category:** FEATURE / UX / RETRIEVAL
**Ledger TX (planning):** `d5b9a9cc-fa83-4ce9-a74f-aaf77eb591fe` (DOCS)
**Ledger TX (fold-in):** `30332efc-0716-4f22-ab89-5879cde7aa2e` (DOCS)
**Ledger TX (implement):** start **FEATURE** on **go**

---

## AI fold-in (2026-08-21) — `agy-review.md` + `opencode-review.md`

No Blockers / Majors. Disposition in spec **§13**.

### Pins locked by fold-in

1. **F4 / AC4:** leading project tag, one space, then `[score=` / `[rank=#`.
2. **F38 / AC1:** merge `HashSet<String>` seen ids; overlap once.
3. **F39:** skip global scan when preferred fills `depth`; AC3 only when remainder &gt; 0.
4. **F15:** COALESCE SELECT stays for tags; prefer-fill is two `lexical_search` (F1). Do not drop the column.
5. **F40:** `prefer_authority: true` on both arms; bridge uses `project_id` None.
6. **F41:** AC3 is pre-`rerank_hits`.
7. **F18:** `format_pretty_hit_line(..., project_tag: Option<&str>)` (Agy O1 already).
8. **Decline:** OpenCode O1 empty-hint; CP `display_label` citation; leftover UUID `7d97a51a` typo.

---

## Preflight (plan time — 2026-08-21)

| Check | Result |
|-------|--------|
| HEAD / tree | **Plan dogfood:** `a5562cc` T275 `#190`. **This fold-in:** `61fd3cb` (plan docs; product crates identical). CLEAN; `main` ahead of `origin/main` by planning docs |
| PATH `ai-brains` | **0.1.1** mtime 2026-08-21 05:55. **T270** on PATH. **Do not `cargo install`.** |
| Source debug | 2026-08-21 18:34 (T275). Tests/manual use `cargo run` / hermetic |
| `preflight --summary` | Pinned **3352**; in-context 0/0/0; grants **0 of 3**; Scope `3581317d` |
| `project whoami` | `mismatch: false`; shell leftover `7d97a456` (**T282**) |
| `memory list --summary --global` | **38833** pinned; leftover **18038**; this repo **3352**; `fcb8a40f` **4875** |
| `list-paths --shared-only` | **11** leftover `C:\dev\*` roots, all exist — T259 tools unused |
| `recall --global "T270 memory_legacy"` | Unique DECISION pins win; **unlabeled** |
| `recall --global "what did we decide"` | T263 DECISION then unlabeled dumps |
| Last PR comments | #190 T275 — **empty** (N/A). #188 Mediums stay **T284**. No T285 |
| Open PR on HEAD | none (Dependabot remotes only) |
| Pins | clap lock **4.6.1** (crates.io 4.6.6; **no clap 5**); serde_json **1.0.150** (crates.io **1.0.151**); chrono **0.4.44** (crates.io 0.4.45); rusqlite **0.39.0** (crates.io 0.40.2) — **no bumps** |
| rustc / nextest / workspace | 1.95.0 / 0.9.140 / **0.1.1** |
| Hotspots | `project.rs` **#1** (3.981, 1332) — do not grow. `sync.rs` #2 — one field only. CLI `preflight.rs` #7 **2027** — do not grow. `ranking.rs` **939** — do not edit. CLI `recall.rs` **1438**. `lexical.rs` **443**. retrieval `recall.rs` **866** |
| Ledger | 0 pending / 0 drift at scan |
| `ISSUES.md` | **Does not exist** (F28) |
| ledgerful search | `rebind_path_alias` `grants.rs:287`; `prefer_authority` `lexical.rs:167`; `RecallHit` `recall.rs` |
| Online | clig.dev default-right + explicit mutate; Elastic/Mongo tenant **filter** ≠ dump identity; ES boost analog; Azure event-sourcing memories stay; clap 4.6.6; rusqlite 0.40.2 **not** bumped |

---

## Phase 0 — on go (re-verify)

- [x] `ledgerful doctor` ; `ledgerful ledger status --compact` ; `ledgerful scan --impact` — work root `C:\dev\AI-Brains`; 0 pending / 0 drift (before FEATURE TX)
- [x] Re-read `main.rs` force-set `:3268`, `Cli::parse()` `:3356`, clap `env = AI_BRAINS_PROJECT_ID` `:1017`, T112 clear `:4322` (preferred = pre-clear id)
- [x] Re-read `recall_full` lexical_search (`prefer_authority: true`) and bridge (`project_id` — stays None when global, F40)
- [x] Re-read lexical SELECT (COALESCE is F15 tags, not fill); `format_pretty_hit_line` (F4/F18 tag arg)
- [x] Confirm leftover still 11 shared roots (`list-paths --shared-only --format json`); do **not** `--write --yes`
- [x] Rescan `conductor/deferred.md` — T276 rows already absorbed; no new overlapping open rows
- [x] Confirm #190 comments/reviews still empty (N/A); #188 Mediums stay T284; no mint
- [x] Re-dogfood PATH `--global` unlabeled; hermetic/source bin for ACs. PATH-behind noted
- [x] Re-check clap lock **4.6.1**, rusqlite **0.39.0**, chrono **0.4.44** — **no bump**
- [x] FEATURE TX on go `6846ad81-4892-41fd-935c-82030dcaf0ac`
- [x] Did **not** `cargo install`; did **not** edit ranking product logic / `project.rs` / `POLICY_DENIED_HINT`; did **not** live-rebind leftover

---

## Absorbed deferred

| Item | Plan action |
|------|-------------|
| Audit leftover ~18k / `--global` junk | **DoD** F1–F6 / AC1–AC5 |
| T264 leftover-first recall (not silent exclude) | **DoD** prefer-fill + label; **not** drop |
| T259/T270 leftover still owns pins | **DoD** hermetic; live rebind F9 |
| Identity `7d97` vs `fcb8a40f` leftover data | **Partial** F2 preferred = cwd project |

---

## Declined (written)

| Item | Why |
|------|-----|
| Silent exclude leftover from `--global` | F6 — T264 F11 |
| `--exclude-project` flag | F20 — not DoD |
| Memory reclassify / `MemoryMoved` | F7 — T259 F5 |
| Live leftover `--write --yes` | F9 — owner confirm |
| whoami mismatch:false | F10 — already T258 |
| Shell leftover on `context --show` | T282 |
| `project list` cwd-first | T283 |
| last-PR #190 Cursor | N/A empty |
| #188 Work/samples | T284 |
| clap 5 / rusqlite 0.40 / DTO / `cargo install` | F21 / F22 |
| T274 chrome / `ranking.rs` / depth raise | F12 / F14 / F13 |
| Empty-hint `Try --global` when already global | Live `build_recall_hint_core` `:675` already honest (OpenCode O1) |

---

## Phase 1 — Red (TDD)

- [x] Unit `merge_preferred_then_global__preferred_first_no_dupes` (AC1) — red until helper exists
- [x] Unit `merge_preferred_then_global__overlap_id__once` (F38)
- [x] Unit `merge_preferred_then_global__preferred_fills_depth__skips_global` (F39)
- [x] Retrieval hermetic `recall_full__global_prefer__owner_pin_beats_leftover_chrome` (AC2) — **required red**
- [x] Retrieval `recall_full__global_prefer__leftover_still_in_candidates` (AC3)
- [x] CLI hermetic `recall__global_pretty__tags_project` (AC4 — tag then space then `[score=` / `[rank=#`)
- [x] CLI `recall__global_json__no_project_id_key` (AC5)
- [x] CLI `recall__scoped_pretty__no_global_tag` (AC9)

---

## Phase 2 — Green

- [x] `RetrievalMemory` + lexical SELECT `COALESCE(mp.project_id, sp.project_id)`
- [x] `RecallHit.project_id` (constructors default `None`)
- [x] `RecallOptions.preferred_project_id` (Default `None`)
- [x] `prefer_project.rs` merge (F38 HashSet, F39 skip); `recall_full` second `lexical_search` when preferred is Some (`prefer_authority: true` both, F40)
- [x] Comment at merge: AC3 is pre-rerank (F41)
- [x] CLI `main.rs`: `--global` threads pre-clear id as preferred; scoped `project_id` stays `None`
- [x] Pretty `--global` leading tag (reuse T264 peel/upgrade + `display_label`)
- [x] Sync `recall_full` one field (AC15)
- [x] No production `unwrap`/`expect`/`panic`
- [x] Do not edit `ranking.rs` product logic / `project.rs` / leftover live aliases

---

## Phase 3 — Docs + registry

- [x] CAPABILITIES: `--global` prefer-fill + labels; leftover split still `rebind-path`
- [x] OPERATIONS: one sentence (memories stay; `--global` labeled)
- [x] CHANGELOG minor
- [x] `conductor.md` **In Progress** during implement; Completed only after gate + review + publish
- [x] `deferred.md` absorb notes (planning pass already)

---

## Phase 4 — Verify (implement, not plan)

- [x] Targeted: `cargo nextest run -p ai-brains-retrieval --test recall_pin_rank --test recall_global_prefer`
- [x] `cargo nextest run -p ai-brains-cli --test recall_global_prefer --test project_rebind_path --test preflight_global_isolation`
- [x] `cargo clippy -p ai-brains-retrieval -p ai-brains-cli --all-targets -- -D warnings`
- [x] Manual hermetic AC2/AC4; live `list-paths --shared-only` print-only (AC12)
- [x] Review log `review.md`; FEATURE `codex-review` (F27) CX2 PASS WITH P3-1
- [x] Full gate at closeout only (`.\scripts\dev-check.ps1` + `ledgerful verify --scope full`)

---

## DoD

- [x] AC1–AC16 green (AC12 recorded; live write only if owner confirmed)
- [x] No silent leftover exclude (AC3)
- [x] No T240 F2 `.env`; no hardcoded leftover UUID in SQL
- [x] T274 / T259 / T264 suites stay green
- [ ] Conductor T276 **Completed** only after go + gate + review + publish (Phase 6)
- [ ] Pin decisions after implement
