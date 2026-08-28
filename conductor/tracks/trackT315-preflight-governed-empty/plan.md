# T315 Plan — preflight empty-decisions next-step + word-count label

**Status:** **Pending** (plan-only until **go**). Spec [spec.md](./spec.md).
**Category:** UX / HONESTY
**Ledger (planning):** DOCS `ca5b1614-6849-416d-ad27-1d44a23198d7`
**Ledger (fold-in):** DOCS `c90c1c71-aa57-40b4-8ee6-7b068837b4bc`

---

## Preflight (plan time — 2026-08-28)

| Check | Result |
|-------|--------|
| HEAD / tree | Fold-in `2b6919c` plan commit CLEAN; `origin/main` = `44520d8` (ahead **1**). Plan-write was `44520d8` / ahead **0** (Agy m1 / OpenCode O4). Branch `track/T315-preflight-governed-empty`. Product `src/` = T312 `#230`. |
| PATH `ai-brains` | **0.1.3** graph-on; **26,897,408** B; mtime **2026-08-27 8:21:55 PM**. T312 **not** on PATH. |
| `preflight --summary` | Pinned **4520**; in-context **0/0/0**; `Total Word Count: 688`; no next-step; grants omitted (3 of 3) |
| `--summary --format json` | `next_step` **omitted**; `word_count` 792 (later invocation) |
| `--pretty -m 1500` | Index **1. `## Objective -- just now`** (T286 R1-1 still true) |
| Formatter | `preflight.rs` `:772–800` 9-arg; word-count `:796`; JSON DTO `:41–64` |
| T241 | optional `next_step` only when grants incomplete (`:908–912`); `probe_discovery_active_count` `:138–162` returns `Some(0)` on project-scoped empty grants |
| rustc | **1.95.0** |
| Pins | clap `"4.5"` / lock **4.6.1** / crates.io **4.6.6**; rusqlite **0.40.2**; workspace **0.1.3** — no bump |
| Last PR Cursor | `#230` `mergedAt` **2026-08-28T02:35:31Z**; Bugbot medium F8 OR-fill no PreferRecency → **T325 minted** |
| Open PRs | **none** |
| Ledger | 0 pending / 0 drift at scan; this TX `ca5b1614` |
| `ISSUES.md` | **Does not exist** |
| Planning install / live pin | **Not run** |

---

## Absorbed deferred

| Item | Plan action |
|------|-------------|
| Audit 0/0/0 + opaque word count | **DoD** F1/F2/F7 / AC1–AC6 / AC11 |
| T220 F30 human label | **F7** `Budget window words:` |
| T241 optional `next_step` | **Reuse**; F5 grants win |
| T286 Index Objective | **Not stolen** F11 |
| T288 / T290 overlay | **Needle only** F3 |
| last-PR `#230` Cursor | **Mint T325** — not this DoD |
| T307 / T308 floors / H2 / clap 5 | **Not stolen** / **Decline** |
| OpenCode m1 AC7 omitted-key | **F5 / F30 / AC7** — not T315 SOOT |
| OpenCode O1 AC5/AC6 fixture | **F38** scope-none |

---

## Phase 0 — on go (re-verify + deferred rescan)

- [ ] `ledgerful doctor` ; `ledgerful ledger status --compact`
- [ ] Confirm cwd `C:\dev\AI-Brains` (not Helping Hands)
- [ ] Re-read `format_preflight_summary_lines` `:772–800` and `print_summary` `:854–957`
- [ ] Re-read `PreflightSummaryJson` `:41–64` and T241 JSON assignment `:893–918`
- [ ] Re-read `probe_discovery_active_count` `:138–162` (global/none → `None`; project id → `Some(n)` including 0)
- [ ] Confirm `LIST_RECALL_QUERY` still `"what did we decide"` in `governed_common.rs` (not yet imported by `preflight.rs`)
- [ ] Confirm T214 arity-9 unit still present
- [ ] Rescan `deferred.md` open overlapping rows
- [ ] Confirm T325 placeholder still Pending (do not steal F8 recency)
- [ ] `ledgerful ledger start T315-preflight-governed-empty --category FEATURE`
- [ ] **Do not** `cargo install` / live production `pin` / `policy bootstrap` / `.env` rewrite in Phase 0

## Phase 1 — Red

- [ ] `format_summary_empty_decisions_next__zero__exact_soot` (AC1)
- [ ] `format_preflight_summary_lines__global__…` asserts `Budget window words: 100` (AC2)
- [ ] `insert_after_budget_window_line__zero_decisions__after_word_count_before_footer` (AC4; F8 dual prefix)
- [ ] Trigger unit hotspots=5 / decisions=0 still inserts (AC14)
- [ ] CLI hermetic AC5/AC6: **scope-none empty vault** (F38 — no `register_project`; `--no-project-context --vault-path`)
- [ ] Confirm those tests **fail** on current tree (`Total Word Count`; no T315 `next:`)
- [ ] AC3 arity-9 + AC7 T286 pin (not T315 SOOT) + AC8 T241 bootstrap + AC9 T180 are **stay-green** (not Phase-1 red)

## Phase 2 — Green

- [ ] F7 rename label inside 9-arg formatter
- [ ] F2/F35 helper using `LIST_RECALL_QUERY` (**new** import — F23); F8 insert after budget-window line (dual prefix)
- [ ] F5 JSON: after T241, fill `next_step` only if `None` and `decision_count == 0`
- [ ] Human T315 line even when grants incomplete; do **not** relocate T241 append
- [ ] Do **not** call `format_authorized_empty_next`
- [ ] Do **not** edit retrieval `preflight.rs` / `project.rs` / `sync.rs` / `governed_common.rs` (import only)

## Phase 3 — Stay-green + docs

- [ ] AC3 9-arg
- [ ] AC7 T286 tagged/legacy pin: `in_context_decisions >= 1` **and** `next_step` is **not** the T315 SOOT (bootstrap presence is expected)
- [ ] AC8 T241 bootstrap wins JSON
- [ ] AC9 T180 compact keys
- [ ] AC12 retrieval `preflight.rs` diff empty
- [ ] AC13 SOOT ≤140
- [ ] AC15 no new required JSON keys
- [ ] CAPABILITIES + CHANGELOG + PROTOCOL-COMPAT (F28 / AC10)
- [ ] Optional `after_help` one sentence

## Phase 4 — Gate + review

- [ ] `cargo fmt --check` ; clippy workspace `-D warnings` ; nextest workspace ; `cargo deny check` ; `cargo audit`
- [ ] `ledgerful verify --scope full`
- [ ] Phase-1 review log `review.md` until clean
- [ ] `codex-review` (FEATURE) until clean
- [ ] Manual AC11 `cargo run -- preflight --summary` (source bin)

## Phase 5 — Closeout

- [ ] Conductor T315 **Completed** with evidence
- [ ] deferred.md T315 row struck; residuals appended (Index Objective stays T286 R1-1)
- [ ] FEATURE TX commit
- [ ] Phase 6: push `track/T315-*` → PR → watch GHA `CI` green → `gh pr merge --squash --delete-branch`. Never `git push origin main`.

---

## DoD (checkable)

- [ ] AC1/AC2/AC4/AC5/AC6 red-then-green
- [ ] `In context decisions: 0` still prints 0 (honest); next-step present
- [ ] `Budget window words:` replaces `Total Word Count:`
- [ ] JSON `word_count` key unchanged; optional `next_step` = T315 SOOT when grants complete
- [ ] T241 incomplete grants still bootstrap
- [ ] T286 pin hermetics stay green and `next_step` is **not** the T315 SOOT (T241 bootstrap may be present)
- [ ] Retrieval Index SQL untouched
- [ ] No `decision propose` / no H2
- [ ] CAPABILITIES + CHANGELOG + PROTOCOL-COMPAT
- [ ] Full gate + Codex
- [ ] PATH install **not** required for Completed (F18)
- [ ] T325 still Pending (not stolen)

---

## Isolation

No `cargo install`. No live vault production pins as implement SoT. No extra `policy bootstrap`. Never `git push origin main`. T307 / T313–T324 / T325 / H2 / clap 5 / floor retune / Index SQL not stolen.
