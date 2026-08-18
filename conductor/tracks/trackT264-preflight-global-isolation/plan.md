# T264 Plan — Preflight global isolation

**Status:** **In Progress** (FEATURE TX `02ee555e-b659-4999-87b2-8477f23169f9`)
**Spec:** [spec.md](./spec.md) F0–F30 / AC1–AC14 + §13 fold-in
**Category:** UX / FEATURE
**Ledger TX (planning):** `a0500604-b8ff-47b9-b24d-9c0923b8855e` (DOCS)
**Ledger TX (fold-in):** `7d6ad8f5-0caf-4506-9903-ab3b0f062f2c` (DOCS)

---

## AI fold-in (2026-08-18) — `agy-review.md` + `opencode-review.md`

No Blockers. OpenCode **M1** folded as **F30** / **AC5** (item-first-line + two-line pin). F24 wrap steps and leading-only upgrade locked. Disposition in spec **§13**.

### Pins locked by fold-in

1. **F30 / AC5:** first line + Session header only; two-line continuation pin; no per-line retag.
2. **F4 / AC3 / AC4:** peel + upgrade the leading tag only; body-internal `[8hex]` stays.
3. **F24:** `truncate_chars(32)` + `]`→`·` in `preflight_pretty.rs`; do not edit `display_label`.
4. **AC14:** pass-with-observed-data if no foreign Session in window.
5. **§2.1:** `d8be361` product vs `bc10f3e` plan.
6. **O1:** still F10.

---

## Preflight (plan time — 2026-08-18)

| Check | Result |
|-------|--------|
| HEAD / tree | Plan dogfood `d8be361`. Plan commit `bc10f3e`. Fold-in docs on that product src. |
| T264 stub | Placeholder upgraded in place to **Planned** |
| PATH `ai-brains` | `C:\Users\RyanB\.cargo\bin\ai-brains.exe` **0.1.1**. Compact/summary-json present. **Do not `cargo install`.** |
| Live hole | `--global --pretty`: unlabeled hip-hierarchy Session 0098/0085 + conductor 0011 next to this-repo T260 Safety. `--global --summary`: 53 projects / 22 in-context decisions / **no span**. Project-scoped pretty is this-repo only. |
| SoT | `build_legacy_preflight` unscoped Safety LIMIT 10 + no `project_id`; `active_sessions` no project field + `format!` SQL; pretty leading chrome; T220 `PreflightSummaryJson`; T230 `display_label` in hotspot `project.rs`. |
| clap / serde_json | lock clap **4.6.1** / crates.io **4.6.6**; serde_json lock **1.0.150** / crates.io **1.0.151**. rustc **1.95.0**. **No clap 5.** Snapshot — re-verify at execute. |
| Last PR Cursor | #178 comments/reviews/inline **0**. **N/A.** |
| `deferred.md` | Full scan. Overlap: audit T264 **absorb**; T214 label residual **absorb**; T214 format! **partial**; T219 F13 **partial**; leftover recall drop **decline**; T265–T271 / T240 F2 / T255 **decline**. |
| ai-brains | `preflight --summary` 3581317d / 2995 pins / grants 0 of 3 (T241). Recall: T260 leftover pointer + `build_preflight`. |
| ledgerful | doctor ready (hygiene warns). 0 pending 0 drift at scan. Hotspot **#1** `project.rs` — do not edit. `#9` `preflight.rs` — sibling file. Index incremental completed. |
| Research | clig.dev human-first + Heroku `--all` grouped tenants; T180 2-key; T214 F9; T219 F13; clap 4.6.6 current. |
| `ISSUES.md` | **Does not exist** |
| Live `.env` / bootstrap / nightly / pin | **Not written** / **not run** / **not scheduled** / **not pinned** this pass. |

---

## Absorbed deferred

| Item | Source | Plan action |
|------|--------|-------------|
| `--global` pretty blender + summary mix | audit T264 | **DoD** F1–F8 / AC5–AC8 / AC14 |
| Honest multi-project body label | T214 residual | **Absorb** F2–F3 |
| `active_sessions` `format!` | T214 | **Partial** F10 |
| T219 F13 selection freeze | T219 | **Partial** — scoped stands; global caps F5 |
| Leftover-project `--global` | T259–T262 closeout | **Partial** preflight; **decline drop** F11 |
| T214 F9 ledgerful-on-global | T214 | **Decline** F14 |
| T265 envelope | series | **Decline** F12 |
| T266 / T267 / T268+ | series | **Decline** F27 |
| T240 F2 / T255 | standing | **Decline** F28 |
| last-PR Cursor | #178 | **N/A** — no leftover to mint |
| OpenCode M1 AC5 over-guarantee | review | **Absorb** F30 / AC5 |
| OpenCode m1 / Agy m2 F24 wrap | review | **Absorb** Phase 2 checklist |
| OpenCode m2 leading-only upgrade | review | **Absorb** F4 / AC4 |
| OpenCode m3 AC14 age-out | review | **Absorb** AC14 |
| Agy O1 `params![]` | review | **Already** F10 |

---

## Phase 0 — on go (re-verify)

- [x] Re-read `build_legacy_preflight` Safety/Index SQL and `sessions.rs`.
- [x] Re-read `format_preflight_pretty_body_with` + `strip_pretty_chrome` + T230 `display_label`.
- [x] Classify-only dogfood: `--global --pretty --compact -m 400` still unlabeled foreign Session. **Do not** pin. **Do not** `cargo install`.
- [x] Re-check lock clap **4.6.1** / crates.io **4.6.6**. rustc **1.95.0**. No clap 5.
- [x] Rescan **entire** `conductor/deferred.md`.
- [x] Last merged PR #178 comments/reviews/inline **0**. N/A.
- [x] `ledgerful ledger start T264-preflight-global-isolation --category FEATURE` → `02ee555e-b659-4999-87b2-8477f23169f9`

---

## Phase 1 — Red

- [x] `take_round_robin__leftover_then_other__interleaves_per_project` (AC1) — failed on blended take, then green
- [x] `take_round_robin__empty_and_unknown__respects_max` (AC2)
- [x] `peel_global_tag__tagged_timestamp_role__chrome_still_strips` (AC3) — remainder `[8hex]` not re-peeled
- [x] `upgrade_global_tag__alias_missing_and_bracket` (AC4) — F24 sanitize + body-internal `[aaaaaaaa]` unchanged
- [x] Hermetic `preflight_global_isolation.rs`: AC5–AC8 / AC10 / AC11 — AC5 includes two-line continuation pin

---

## Phase 2 — Green

- [x] `preflight_global.rs`: `take_round_robin` + `[8hex]` / `[unknown]` prefix + span count
- [x] `build_legacy_preflight`: SELECT `COALESCE(m.project_id, s.project_id)`; Safety LIMIT **40** when global; apply F5 caps; write tags into text
- [x] `SessionContext.project_id`; `active_sessions` both arms `params![]`
- [x] `PreflightContext.in_context_project_span`
- [x] Summary line F7 + JSON F8
- [x] `preflight_pretty.rs`: peel **leading** tag → chrome → `display_label` → `truncate_chars(..., 32)` → replace `]` with `·` → reattach (no `project.rs` edit; do not regex-replace `[8hex]` over the whole line)
- [x] Wire pretty dispatch; do not grow `preflight.rs` except summary/JSON/dispatch
- [x] `lib.rs` / `mod.rs` modules

---

## Phase 3 — Verify

- [x] Targeted: retrieval AC1/AC2 units + CLI peel/upgrade/chrome + hermetic isolation + T214/T220/T250/T180
- [x] `cargo clippy -p ai-brains-retrieval -p ai-brains-cli --all-targets -- -D warnings` exit 0
- [x] Confirm T214 / T220 / T250 hermetics + `t180_c_preflight_json_keys` green
- [x] Manual AC14 source bin (classify-only). See evidence below.
- [x] Docs: CAPABILITIES + PROTOCOL-COMPAT + CHANGELOG
- [x] `ledgerful scan --impact` ; `ledgerful verify --scope fast` (fmt dirtied then cleaned; clippy/nextest/deny/audit ok)

---

## Phase 4 — Review + publish (implement-track)

- [x] `review.md` Phase-1 clean (medium+ not dropped)
- [x] `codex-review` (FEATURE) — CX1 P2 identity → CX2 P2 pretty collision → **CX3 PASS WITH DEFERRED P3**
- [x] Full gate: `dev-check.ps1` SUCCESS nextest 3100 (1 skipped); `ledgerful verify --scope full` passed
- [x] conductor **Completed**; deferred closeout row
- [ ] Push `track/T264-*` (never `main`); PR; watch GHA; squash-merge; prune

---

## Definition of done

- [ ] AC1–AC14 green or waived in review with evidence
- [ ] F0–F30 honored (especially F11 drop-decline, F12 T265, F23 no `project.rs`, F30 first-line only)
- [ ] T180 2-key + T220 required keys unchanged
- [ ] Project-scoped pretty unlabeled / unchanged
- [ ] No `unwrap`/`expect`/`panic` in production
- [ ] No clap 5 / new crates / live `.env` / `cargo install` unless owner asked
- [ ] FEATURE TX committed; conductor Completed only after merge hygiene

---

## Stop-before (even after go)

- Scope exceeds F1–F10 (recall leftover drop, T265 envelope, T266 maze)
- T240 F2 / T255 reopen
- Missing secrets / live migrate / live bootstrap as DoD
- Broad unrelated failures (triage; do not clean up)
- Push to `main` / force-push
