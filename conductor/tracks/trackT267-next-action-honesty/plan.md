# T267 Plan — Next-action honesty

**Status:** **Planned** (plan-only until go)
**Spec:** [spec.md](./spec.md) F0–F22 / AC1–AC14
**Category:** UX / FEATURE
**Ledger TX (planning):** `50c39329-176a-4075-95c1-7638bb6885c0` (DOCS)
**Ledger TX (on go):** `ledgerful ledger start T267-next-action-honesty --category FEATURE --message "Harness ok next is none; list footer never cwd-slug unless target is cwd path-owner"`

---

## Preflight (plan time — 2026-08-18)

| Check | Result |
|-------|--------|
| HEAD / tree | `fa90981` T266 `#180`. `main` = `origin/main`. CLEAN. |
| T267 stub | Placeholder upgraded in place to **Planned** |
| PATH `ai-brains` | **0.1.1**. Same holes (status self-next; leftover+AI-Brains footer). **Do not `cargo install`.** |
| Live hole | `harness status`: 5× `next: ai-brains harness status` + five install trailers. `project list`: `set-alias 7d97a456 … AI-Brains`. `whoami` remediations **(none)** / T258 already adopt-path. Preflight harness block already omits ok `next:`. |
| SoT | `next_action_for` `:286`; `harness.rs` `:58`; `print_unaliased_footer` `:104`; `footer_alias_suggestion` `:125`. |
| clap / serde_json | lock clap **4.6.1** / crates.io **4.6.6**; serde_json lock **1.0.150** / crates.io **1.0.151**. rustc **1.95.0**. **No clap 5.** Snapshot — re-verify at execute. |
| Last PR Cursor | #180 comments/reviews **empty**. #179 Bugbot Medium still true → **T272** already minted. No open PR on `main`. |
| `deferred.md` | Full scan. Overlap: audit T267 **absorb**; T259 footer **absorb**; T258 F2 **affirm**; T212 chrome **partial**; T235 F40 **affirm**; T265–T272 except T267 **decline**; T240 F2 / T255 **decline**. |
| ai-brains | `preflight --summary` 3581317d / 3046 pins / grants 0 of 3 (T241). Recall: T259 review notes; no T267 pin. |
| ledgerful | doctor ready (hygiene warns). 0 pending 0 drift. Hotspot **#1** `project.rs` — extract footer. Index incremental completed (0 file delta). |
| Research | clig.dev suggest-next + saying-just-enough + future-proof human; git status is the reference (suggests add/restore, not status). |
| `ISSUES.md` | **Does not exist** |
| Live `.env` / bootstrap / nightly mutate / pin | **Not written** / **not run** / **not scheduled** / **not pinned** this pass. |

---

## Absorbed deferred

| Item | Source | Plan action |
|------|--------|-------------|
| harness/whoami self-next; list leftover-as-AI-Brains | audit T267 | **DoD** F1 / F3 / F3b / F6. Whoami **affirm** F2 |
| T259 footer algorithm | T259 F3 / closeout | **Absorb** F3 / F3b / F9 |
| T212 set-alias footer chrome | T212 F8 | **Partial** F22 — keep stderr example; change pick |
| T258 whoami remediations | T258 F10 | **Affirm** F2 / AC5 |
| T235 install next = status | T235 F40 | **Affirm** F7 / AC10 |
| Shared helper (stub F4) | placeholder | **Decline** F4 |
| T265 / T268 / T269 / T270 / T271 / T272 | series | **Decline** F11 |
| T240 F2 / T255 bag | standing | **Decline** F12 |
| last-PR Cursor #180 | empty | **N/A** |
| #179 safety_ids | Bugbot | **T272** already exists |

---

## Phase 0 — on go (re-verify)

- [ ] Re-read `next_action_for`, `run_status`, `print_unaliased_footer`, `format_harness_summary_lines`.
- [ ] Confirm source `harness status` still prints `next: ai-brains harness status` on ok rows.
- [ ] Confirm source `project list` stderr still `set-alias 7d97a456 … AI-Brains`.
- [ ] Confirm whoami remediations still omit `` `ai-brains project whoami` ``.
- [ ] Re-check lock clap **4.6.1** / crates.io clap. rustc **1.95.0**. No clap 5.
- [ ] Rescan **entire** `conductor/deferred.md`.
- [ ] Last merged PR Cursor comments — #180 leftover none; T272 still the #179 mint.
- [ ] `ledgerful ledger start T267-next-action-honesty --category FEATURE`

---

## Phase 1 — Red

- [ ] `next_action_for__ok__none` (AC1)
- [ ] `harness_status__all_ok__omits_self_next` (AC2)
- [ ] `harness_status__all_ok__json_next_action_none` (AC3)
- [ ] `pick_unaliased_footer_target__cwd_owner_unaliased__wins` (AC14 / AC6)
- [ ] `footer_alias_suggestion__non_owner__not_cwd_slug` (AC14 / AC7)
- [ ] Hermetic AC6 / AC7 (`next_action_honesty.rs`)
- [ ] `cargo nextest run -p ai-brains-cli` — new tests **fail** (Ok still `harness status`; leftover still first)
- [ ] Commit red allowed

---

## Phase 2 — Green (harness)

- [ ] `next_action_for` Ok → `"none"` (F1)
- [ ] `run_status` omit `next:` when none/Ok (F1)
- [ ] F6 trailer: only present && !Ok
- [ ] Do **not** change install/uninstall success next (F7)
- [ ] Preflight: add Ok-row unit if missing; do not rewrite summary (F8)
- [ ] Targeted: `cargo clippy -p ai-brains-cli --all-targets -- -D warnings` ; AC1–AC4 / AC9 / AC10 green
- [ ] Commit green allowed

---

## Phase 3 — Green (footer)

- [ ] Add `project_list_footer.rs` + `mod` (F10)
- [ ] Implement F3 pick + F3b suggestion (no leftover UUID)
- [ ] `project.rs` `list()` calls helper; delete old footer body
- [ ] T212 AC3/AC4/AC5 stay green (AC8)
- [ ] T258 AC7 stays green (AC5)
- [ ] AC6 / AC7 / AC14 green
- [ ] `project.rs` line count does not grow (AC12)
- [ ] Commit green allowed

---

## Phase 4 — Docs

- [ ] CAPABILITIES: harness ok omit/`none`; list footer F3/F3b (F14 / AC11)
- [ ] PROTOCOL-COMPAT additive `next_action: "none"` (F14)
- [ ] `project list` `after_help`: example is cwd path-owner if unaliased; never cwd slug for a non-owner
- [ ] Root CHANGELOG T267 row
- [ ] Do **not** reorder T204 Start-here groups

---

## Phase 5 — Review + gate (on go)

- [ ] Internal review → `review.md`
- [ ] Medium+ not silently dropped
- [ ] `codex-review` (F16)
- [ ] Manual AC13 (source bin)
- [ ] Full gate: `cargo fmt --check` ; clippy workspace `-D warnings` ; nextest workspace ; `cargo deny check` ; `cargo audit` ; `ledgerful verify --scope full`
- [ ] Conductor T267 → Completed in the implement commit; publish is implement-track Phase 6

---

## Definition of done

- [ ] AC1–AC14 pass with evidence
- [ ] F0–F22 honored (declines written)
- [ ] No product commits under this planning TX
- [ ] T272 still the #179 leftover (not reminted)
- [ ] `conductor/ISSUES.md` not created

---

## Stop-before

- Scope exceeds F1/F3/F6 (T265 envelope, T268 scan, T272 retrieval, whoami remediations rewrite)
- Destructive git / push to `main`
- Hardcoding leftover UUID
- Broad unrelated failures
