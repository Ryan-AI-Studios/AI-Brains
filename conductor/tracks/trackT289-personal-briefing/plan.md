# T289 Plan — Personal deny must not print `_None_` preferences

**Status:** **Pending** (Planned). Full F-list in spec.md.
**Spec:** [spec.md](./spec.md) F0–F24 / AC1–AC12 + §13 AI fold-in
**Category:** FEATURE / UX / HONESTY
**Ledger TX (planning):** `25bbc580-99a6-4969-8ea5-d0e1902d374e` (DOCS)
**Ledger TX (fold-in Agy+OpenCode):** `45277700-a110-4f91-911b-8f921173dfdb` (DOCS)
**Ledger TX (implement):** FEATURE on **go**

---

## AI fold-in (2026-08-23) — `agy-review.md` + `opencode-review.md`

Agy **B 0 / M 0**. OpenCode **B 0 / M 0**. Disposition in spec **§13**.

### Pins locked by fold-in

1. **F11 (Agy m1):** helper is private `fn` — not `pub` / not re-exported.
2. **AC5 (Agy O1):** allowed-empty `## Preferences\n_None_`.
3. **§2.3 (OpenCode m1):** CP test is `tests/personal_briefing.rs:154`.
4. **AC3/F5 (OpenCode m2):** T288 overlay project-path only.
5. **AC4 (OpenCode m3):** const-level `_None_` / bootstrap guards.
6. **AC8/F20 (OpenCode O2):** CAPABILITIES *extend* row `:322`.
7. **Affirm:** Agy m2 exact AC4; Agy O2 after_help; OpenCode O1 `empty_personal`; #204 N/A; T290/H2 not stolen.

---

## Preflight (plan time — 2026-08-23)

| Check | Result |
|-------|--------|
| HEAD / tree | `05d7ac0` T288 `#204`. CLEAN. `origin/main` = HEAD |
| PATH `ai-brains` | **0.1.2** mtime 2026-08-22 19:41. **No T285–T288.** Hole in **source**. **Do not `cargo install`.** |
| `briefing personal --format human` | Denied + T263 recall next + **`## Preferences` `_None_`** + **`## Continuity` `_None_`** |
| `briefing personal --format json` | `denied: true`, empty arrays, `denial_hint` recall, no bootstrap |
| Last PR comments | #204 T288 — **empty** (N/A). **No T301.** |
| Open PR on HEAD | none (Dependabot remotes only) |
| Pins | clap lock **4.6.1** (crates.io 4.6.6; **no clap 5**); rusqlite **0.39.0**; serde_json **1.0.150** — **no bumps** |
| rustc / nextest / workspace | 1.95.0 / 0.9.140 / **0.1.2** |
| Hotspots | `project.rs` #1 — do not touch. `personal.rs` **#7** — **do not grow**. Extend `renderer.rs` |
| Ledger | 0 pending / 0 drift at scan; planning TX `25bbc580` |
| `ISSUES.md` | **Does not exist** (F17) |
| ledgerful search | `render_personal_markdown` `renderer.rs:243` |
| Online | clig.dev no-access vs empty; T275 analog without copying HIDDEN; T180 no new JSON keys |

---

## Phase 0 — on go (re-verify)

- [ ] `git fetch --all --prune` ; if `origin/main` moved, reconcile (never `git push origin main`)
- [ ] `ledgerful doctor` ; `ledgerful ledger status --compact` ; `ledgerful scan --impact` — 0 pending / 0 drift before FEATURE TX
- [ ] Re-read `render_personal_markdown` `:243` + `_None_` `:262–273`
- [ ] Confirm `BRIEFING_PERSONAL_DENIED_NEXT_STEP` unchanged — **do not edit**
- [ ] Confirm `personal.rs:121` still PERSONAL hint — **do not edit `personal.rs`**
- [ ] Confirm T275 AC16 unit still forbids GRANT_WALL/HIDDEN on personal deny
- [ ] Confirm T227 AC8 allowed-empty `_None_` + empty_continuity
- [ ] Rescan `conductor/deferred.md` — T289 absorbed; T290 not stolen
- [ ] Confirm #204 comments/reviews still empty (N/A); no mint
- [ ] Re-dogfood `briefing personal --format human` + json **read-only**. **Did not** Personal bootstrap; **did not** write `.env`
- [ ] Re-check clap **4.6.1**, rusqlite **0.39.0** — **no bump**
- [ ] FEATURE TX (new)
- [ ] Did **not** `cargo install`; did **not** grow `personal.rs` / `project.rs` / `preflight.rs`

---

## Absorbed deferred

| Item | Plan action |
|------|-------------|
| Audit deny + `_None_` prefs | **DoD** F1–F4 / AC1–AC2 / AC10 |
| T275 F32 Personal `_None_` | **Promote** F1 |
| T288 closeout Personal `_None_` | **This track** |
| Placeholder Manual human | **DoD** AC2 / AC10 |

## Declined (written)

| Item | Why |
|------|-----|
| Auto Personal grant / bootstrap as required | F7 |
| Project GRANT_WALL / HIDDEN on Personal | F3 / T275 F35 |
| T288 vault-pin on Personal | F8 |
| Lists/progressive pin count | **T290** |
| #18 synthetic continuity | F24 |
| last-PR #204 Cursor | N/A empty — no T301 |
| clap 5 / rusqlite 0.40 / T240 F2 / H2 | Standing |

---

## Phase 1 — Red (required)

- [ ] `render_personal_markdown__denied__no_none_placeholder` (AC1)
- [ ] `briefing_personal_denied_body__exact_optional_one_line` (AC4)
- [ ] Confirm they **fail** (not compile-error-only) before green

## Phase 2 — Green

- [ ] `BRIEFING_PERSONAL_DENIED_BODY` exact F2 next to Personal deny consts
- [ ] **Private `fn`** `personal_empty_section_placeholder(denied)` — **not** `empty_section_placeholder`; **not** `pub`; **not** re-exported (Agy m1)
- [ ] Prefs + Continuity empty branches use the helper
- [ ] Reuse existing `empty_personal` fixture (`:383`) — do not mint a second one (OpenCode O1)
- [ ] No `unwrap`/`expect`/`panic` in production
- [ ] `personal.rs` diff empty

## Phase 3 — More ACs

- [ ] AC2 hermetic human no `_None_`
- [ ] AC3 JSON freeze (existing hermetic stays green)
- [ ] AC5 T227 allowed-empty `_None_` stays green (**both** Preferences and Continuity)
- [ ] AC6 T275 AC16 stays green
- [ ] AC7 unknown format exit 2
- [ ] AC11 `personal.rs` unchanged
- [ ] AC12 no new JSON keys

## Phase 4 — Docs + gates

- [ ] CAPABILITIES **extend** Denied packets row `:322` (not a new section)
- [ ] `briefing personal` after_help one sentence
- [ ] CHANGELOG T289
- [ ] `cargo fmt --check` ; `cargo clippy -p ai-brains-cli -p ai-brains-control-plane --all-targets -- -D warnings`
- [ ] `cargo nextest run --workspace` (or targeted then full)
- [ ] `cargo deny check` ; `cargo audit`
- [ ] `ledgerful verify --scope full`

## Phase 5 — Manual + closeout

- [ ] Manual AC10 `cargo run -p ai-brains-cli -- briefing personal --format human`
- [ ] Phase-1 `review.md` → clean
- [ ] `codex-review` (FEATURE)
- [ ] conductor.md T289 **Completed**; deferred closeout; README
- [ ] Publish: push `track/T289-*` → PR → `gh run watch --exit-status` CI green → `gh pr merge --squash --delete-branch` → fetch/prune. Never `git push origin main`. Never force-push.

## DoD

- [ ] Denied human has no `_None_`; next is recall; not Personal bootstrap
- [ ] JSON `denied: true` unchanged
- [ ] T288 / T290 / H2 not stolen
- [ ] CI green + squash-merged

## Isolation (every phase)

No `cargo install`. No live pin as implement. No `.env` write. No live `policy bootstrap`. No `retention apply --confirm`. No schtasks mutate.
