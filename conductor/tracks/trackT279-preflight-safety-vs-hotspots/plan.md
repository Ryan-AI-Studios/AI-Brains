# T279 Plan — Preflight Safety vs live hotspots

**Status:** **Completed** (implement 2026-08-22)
**Spec:** [spec.md](./spec.md) F0–F37 / AC1–AC14 + §13 AI fold-in
**Category:** FEATURE / UX / HONESTY
**Ledger TX (planning):** `4d4dd4b0-1884-4bfc-a0dd-8543aa5de1a5` (DOCS)
**Ledger TX (fold-in Agy+OpenCode):** `2b834a4e-ea61-4142-a6c5-a03a9a7eb108` (DOCS)
**Ledger TX (implement):** FEATURE `a9c3fdbd-a516-4b8e-b1a9-f3534f057ab4`

---

## AI fold-in (2026-08-22) — `agy-review.md` + `opencode-review.md`

No Blockers / Majors either harness. Disposition in spec **§13**.

### Pins locked by fold-in

1. **F36 / AC9:** JSON array finder is first line whose `trim_start` starts with `[` (`safety.rs` `:116–118`). Extra `displayScore` ignored.
2. **F37:** always-emit still `trim_to_word_budget_no_sentinel(..., onboarding_budget)`.
3. **AC6:** stay-green is `preflight_pretty__summary_smoke__dual_model_unchanged` + `preflight_pretty__summary_compact__dual_model_unchanged`.
4. **§2.1:** pin counts / hotspot `displayScore` volatile; F2 uses raw `score={:.2}`.
5. **Already:** F3/AC14; AC2/AC9 pure units.
6. **Decline:** OpenCode O1 split `--global` empty copy.

---

## Preflight (plan time — 2026-08-22)

| Check | Result |
|-------|--------|
| HEAD / tree | **Plan dogfood:** `631a8f8` T278 `#194`. **This fold-in:** `448ef47` (docs-only; product crates identical). CLEAN |
| PATH `ai-brains` | **0.1.1** mtime 2026-08-21 05:55. **T270** on PATH. Safety SQL unchanged since T274 F23. **Do not `cargo install`.** |
| `preflight --summary` | Pinned **volatile** (plan 3516; fold-in **3546**); in-context 0/0/0; grants **0 of 3**; Scope `3581317d` |
| `safety sync --dry-run` | **5** paths: `project.rs`, `sync.rs`, `forget.rs`, `context.rs`, `governed_common.rs` |
| `preflight --pretty --compact -m 400` | Safety = **`## Objective`**. No Intelligence. No dry-run paths |
| `preflight --pretty -m 800` | Safety = T272 review-track Objective dump |
| Last PR comments | #194 T278 — **empty** (N/A). #188 closed by T284. No T285 |
| Open PR on HEAD | none (Dependabot remotes only: rusqlite 0.40.2 `#61`, chrono 0.4.45 `#62`) |
| Pins | clap lock **4.6.1** (crates.io 4.6.6; **no clap 5**); serde_json **1.0.150** (1.0.151); chrono **0.4.44** (0.4.45); rusqlite **0.39.0** (0.40.2); uuid lock **1.23.1** (1.25.0) — **no bumps** |
| rustc / nextest / workspace | 1.95.0 / 0.9.140 / **0.1.1** |
| Hotspots | `project.rs` **#1** (displayScore volatile; F2 raw `score={:.2}`) — do not grow. CLI `preflight.rs` #7 (2027) — do not grow. Retrieval `preflight.rs` **1087**. `doctor.rs` / `sync.rs` — do not grow |
| Ledger | 0 pending / 0 drift at scan; planning TX `4d4dd4b0` |
| `ISSUES.md` | **Does not exist** (F24) |
| ledgerful search | `safety_sql` `preflight.rs:290`; `query_ledgerful` `:695`; CLI `fetch_hotspots_json` `safety.rs:102` |
| Online | SQLite GLOB vs LIKE; clig.dev human-first empty; clap 4.6.6; rusqlite 0.40.2 **not** bumped |

---

## Phase 0 — on go (re-verify)

- [x] `ledgerful doctor` ; `ledgerful ledger status --compact` ; `ledgerful scan --impact` — work root `C:\dev\AI-Brains`; 0 pending / 0 drift (before FEATURE TX)
- [x] Re-read Safety SQL (`retrieval/src/preflight.rs` ~`:290–305`), skip-set ~`:345–349`, emit ~`:351–376`, HOTSPOT suppress ~`:325–327`
- [x] Re-read `query_ledgerful` ~`:695` — **do not edit** (F11)
- [x] Re-read CLI `safety.rs` `fetch_hotspots_json` ~`:102–128` + clap `SafetyCommands::Sync` default **5**
- [x] Re-read `index_marker_glob_sql` (`session_chrome.rs` ~`:73`) — **do not reuse** (includes DECISION)
- [x] Confirm T272 AC2/AC3 still in `preflight_global_isolation.rs`
- [x] Confirm T219 `preflight_pretty__summary_smoke__dual_model_unchanged` (`:266`) and `preflight_pretty__summary_compact__dual_model_unchanged` (`:532`) still assert no Bearings on `--summary`
- [x] Rescan `conductor/deferred.md` — T279 rows absorbed; no new overlapping open rows
- [x] Confirm #194 comments/reviews still empty (N/A); no mint; Dependabot `#61` still not this track
- [x] Re-dogfood `safety sync --dry-run` + `preflight --pretty --compact -m 400` **read-only**. **Did not** pin. **Did not** `safety sync` without `--dry-run`
- [x] Re-check clap lock **4.6.1**, rusqlite **0.39.0**, chrono **0.4.44** — **no bump**
- [x] FEATURE TX `a9c3fdbd-a516-4b8e-b1a9-f3534f057ab4`
- [x] Did **not** `cargo install`; did **not** grow `project.rs` / CLI `preflight.rs` / `doctor.rs` / `sync.rs` / `safety.rs`

---

## Absorbed deferred

| Item | Plan action |
|------|-------------|
| Audit Safety = `## Objective` / review-track | **DoD** F1–F3 / AC3–AC4 / AC10 |
| T274 F23 Safety SQL | **Lift** F1 GLOB |
| T274 AC6 buried CONSTRAINT steal | **AC3** dump not in Safety |
| T250 F12 float | **Partial** F15 live `score={:.2}` |

## Declined (written)

| Item | Why |
|------|-----|
| T272 skip / T264 caps | F5/F6 |
| Intelligence rewrite | F11 |
| T280–T283 / leftover rebind / T240 F2 / clap 5 / rusqlite 0.40 | F12/F17 |
| last-PR #194 Cursor | N/A empty |
| Dependabot rusqlite `#61` | F12 — no T285 |
| Live `safety sync` pin | F21 |
| OpenCode O1 `--global` empty wording | F3 one remediator |

---

## Phase 1 — Red (TDD)

- [x] `safety_marker_glob_sql__includes_constraint_not_decision` — AC1
- [x] `format_safety_hotspot_line__path_and_score__hotspot_prefix` — AC2
- [x] `parse_hotspots_json__log_then_array__one_path` — AC9
- [x] `preflight__buried_constraint_dump__not_in_safety` — AC3
- [x] `preflight__no_bearings__emits_safety_sync_remediator` — AC4
- [x] `safety_empty_const__no_hotspot_marker` — AC14
- [x] `skip_live_hotspots_env__truthy__no_spawn` — AC8
- [x] Commit red allowed (`df173a7`)

## Phase 2 — Green

- [x] F1 `safety_marker_glob_sql` in `session_chrome.rs`; Safety SQL uses it (not LIKE, not `index_marker_glob_sql`)
- [x] F14 `preflight_safety.rs`: parse (F36 line-finder), `format_safety_hotspot_line`, `SAFETY_EMPTY`, skip-env, fail-open fetch
- [x] F2 prepend live lines (project-scoped, limit 5) **without** memory_ids (F5)
- [x] F3 always-emit header + empty or body; F37 still `trim_to_word_budget_no_sentinel(..., onboarding_budget)`
- [x] F7 suppress vault HOTSPOT when live inject non-empty (leading `PinKind::Hotspot`; Intelligence substring kept)
- [x] F13 `hermetic_bin` sets `AI_BRAINS_PREFLIGHT_SKIP_LIVE_HOTSPOTS=1`; denylist includes the key
- [x] F30 preflight `after_help` additive (`main.rs`)
- [x] AC5/AC6/AC7/AC11/AC12 stay green
- [x] Commit green (`0766239`); fixture skip (`4e4cc5a`); F7/cap (`8fd6eab`)

## Phase 3 — Docs

- [x] CAPABILITIES: Safety = live hotspots (project-scoped) + leading GLOB; empty names `safety sync --dry-run`
- [x] OPERATIONS one sentence (preflight vs `safety sync --dry-run`; pin is mutating)
- [x] PROTOCOL-COMPAT: no new required keys
- [x] CHANGELOG T279
- [x] Skill one-liner if preflight section exists (tracked `.claude`; untracked `.agents` copy)
- [x] conductor Completed on implement closeout

## Phase 4 — Verify

- [x] Targeted nextest: `-p ai-brains-retrieval` safety units; `--test preflight_global_isolation`; new AC3/AC4 hermetic; T219 summary test
- [x] `cargo clippy -p ai-brains-retrieval --all-targets -- -D warnings` ; `cargo clippy -p ai-brains-cli --all-targets -- -D warnings`
- [x] `cargo fmt --check`
- [x] Primary review → `review.md`; mediums not silently dropped
- [x] Cross-model `codex-review` (F23) CX1 FAIL → fixes; CX2 **PASS WITH DEFERRED P3**
- [x] Full workspace gate: `dev-check.ps1` nextest **3294** passed / 1 skipped; `ledgerful verify --scope full` exit 0
- [x] Classify-only live `cargo run -p ai-brains-cli -- preflight --pretty --compact -m 400` (AC10). **No** live pin. **No** `safety sync` without `--dry-run`

## DoD (checkable)

- [x] Hermetic: buried CONSTRAINT dump **not** in Safety; leading CONSTRAINT **is** (AC3)
- [x] Hermetic: no bearings → Safety header + `safety sync --dry-run` (AC4)
- [x] GLOB helper excludes DECISION (AC1)
- [x] Live line unit `HOTSPOT:` + `{:.2}` (AC2)
- [x] JSON parse fail-open (AC9)
- [x] Skip-env no spawn (AC8)
- [x] T272 AC2 green (AC5)
- [x] `--summary` still no Bearings header (AC6)
- [x] Live classify-only AC10 (cargo run, not PATH)
- [x] No live `safety sync` without `--dry-run`
- [x] No `cargo install`
- [x] Diff omits `project.rs` / CLI `preflight.rs` / `doctor.rs` / `sync.rs` / `safety.rs` (AC13)
- [ ] implement-track Phase 6: push `track/T279-*` → PR → watch GHA `CI` green → squash-merge → prune (never `git push origin main`)

## Stop-before

- Live pin / `safety sync` without `--dry-run` / `.env` rewrite / schtasks mutate / `cargo install` / leftover rebind / grant bootstrap
- Scope exceeds T279 (do not steal T280–T283, T274 Index, T272 skip, T214 Intelligence, T213 floors)
- Ambiguous spec vs src after Phase 0 — halt and ask
