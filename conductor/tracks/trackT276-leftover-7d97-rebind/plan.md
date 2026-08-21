# T276 Plan — Leftover `7d97a456` must not starve `--global`

**Status:** **Pending** (Planned; F0 until **go**)
**Spec:** [spec.md](./spec.md) F0–F37 / AC1–AC16
**Category:** FEATURE / UX / RETRIEVAL
**Ledger TX (planning):** (this pass, DOCS)
**Ledger TX (implement):** start **FEATURE** on **go**

---

## Preflight (plan time — 2026-08-21)

| Check | Result |
|-------|--------|
| HEAD / tree | `a5562cc` T275 `#190`. CLEAN; `main` = `origin/main` |
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

- [ ] `ledgerful doctor` ; `ledgerful ledger status --compact` ; `ledgerful scan --impact` — work root `C:\dev\AI-Brains`; 0 pending / 0 drift (before FEATURE TX)
- [ ] Re-read `main.rs` T112 `--global` clear (`:4320–4326`), `RecallOptions` / `RecallHit`, lexical SELECT (`:259`), `format_pretty_hit_line` (`:407`), `rebind_path_alias`
- [ ] Confirm leftover still 11 shared roots (`list-paths --shared-only --format json`); do **not** `--write --yes`
- [ ] Rescan `conductor/deferred.md` — T276 rows already absorbed; no new overlapping open rows
- [ ] Confirm #190 comments/reviews still empty (N/A); #188 Mediums stay T284; no mint
- [ ] Re-dogfood `cargo run -p ai-brains-cli -- recall "what did we decide" --global --limit 5 --format pretty --no-bridge` (source has T274). PATH-behind noted
- [ ] Re-check clap lock **4.6.1** (crates.io 4.6.6), rusqlite **0.39.0** (0.40.2), chrono **0.4.44** (0.4.45) — **no bump**
- [ ] FEATURE TX on go
- [ ] Did **not** `cargo install`; did **not** edit `ranking.rs` / `project.rs` / `POLICY_DENIED_HINT`; did **not** live-rebind leftover

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

---

## Phase 1 — Red (TDD)

- [ ] Unit `merge_preferred_then_global__preferred_first_no_dupes` (AC1) — red until helper exists
- [ ] Retrieval hermetic `recall_full__global_prefer__owner_pin_beats_leftover_chrome` (AC2) — **required red**
- [ ] Retrieval `recall_full__global_prefer__leftover_still_in_candidates` (AC3)
- [ ] CLI hermetic `recall__global_pretty__tags_project` (AC4)
- [ ] CLI `recall__global_json__no_project_id_key` (AC5)
- [ ] CLI `recall__scoped_pretty__no_global_tag` (AC9)

---

## Phase 2 — Green

- [ ] `RetrievalMemory` + lexical SELECT `COALESCE(mp.project_id, sp.project_id)`
- [ ] `RecallHit.project_id` (constructors default `None`)
- [ ] `RecallOptions.preferred_project_id` (Default `None`)
- [ ] `prefer_project.rs` merge; `recall_full` second `lexical_search` when preferred is Some
- [ ] CLI `main.rs`: `--global` threads pre-clear id as preferred; scoped `project_id` stays `None`
- [ ] Pretty `--global` leading tag (reuse T264 peel/upgrade + `display_label`)
- [ ] Sync `recall_full` one field (AC15)
- [ ] No production `unwrap`/`expect`/`panic`
- [ ] Do not edit `ranking.rs` / `project.rs` / leftover live aliases

---

## Phase 3 — Docs + registry

- [ ] CAPABILITIES: `--global` prefer-fill + labels; leftover split still `rebind-path`
- [ ] OPERATIONS: one sentence (memories stay; `--global` labeled)
- [ ] CHANGELOG minor
- [ ] `conductor.md` **In Progress** during implement; Completed only after gate + review + publish
- [ ] `deferred.md` absorb notes (planning pass already)

---

## Phase 4 — Verify (implement, not plan)

- [ ] Targeted: `cargo nextest run -p ai-brains-retrieval --test recall_pin_rank --test recall_global_prefer`
- [ ] `cargo nextest run -p ai-brains-cli --test recall_global_prefer --test project_rebind_path --test preflight_global_isolation`
- [ ] `cargo clippy -p ai-brains-retrieval -p ai-brains-cli --all-targets -- -D warnings`
- [ ] Manual hermetic AC2/AC4; live `list-paths --shared-only` print-only (AC12)
- [ ] Review log `review.md`; FEATURE `codex-review` (F27)
- [ ] Full gate at closeout only (`.\scripts\dev-check.ps1` + `ledgerful verify --scope full`)

---

## DoD

- [ ] AC1–AC16 green (AC12 recorded; live write only if owner confirmed)
- [ ] No silent leftover exclude (AC3)
- [ ] No T240 F2 `.env`; no hardcoded leftover UUID in SQL
- [ ] T274 / T259 / T264 suites stay green
- [ ] Conductor T276 **Completed** only after go + gate + review + publish (Phase 6)
- [ ] Pin decisions after implement
