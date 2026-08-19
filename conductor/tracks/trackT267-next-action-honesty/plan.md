# T267 Plan — Next-action honesty

**Status:** **Planned** (plan-only until go)
**Spec:** [spec.md](./spec.md) F0–F22 / AC1–AC16 + §13 fold-in
**Category:** UX / FEATURE
**Ledger TX (planning):** `50c39329-176a-4075-95c1-7638bb6885c0` (DOCS)
**Ledger TX (fold-in):** `205fba7b-98aa-4823-93b2-e02d1c9cc353` (DOCS)
**Ledger TX (on go):** `ledgerful ledger start T267-next-action-honesty --category FEATURE --message "Harness ok next is none; list footer never cwd-slug unless target is cwd path-owner"`

---

## AI fold-in (2026-08-18) — `agy-review.md` + `opencode-review.md`

No Blockers / Majors. OpenCode **m3** folded as named additive AC10. Leftover live **11** paths folded as AC7/AC16 split. Agy **m2** folded as required AC15. Disposition in spec **§13**.

### Pins locked by fold-in

1. **AC10:** `harness_install__success__next_is_status` — no existing hermetic.
2. **AC15:** `format_harness_summary_lines__ok__omits_next`.
3. **AC7 / AC16:** multi-path leftover + orphan vs leftover-only basename.
4. **AC12:** total lines 1511 (not Measure-Object 1368).
5. **F10:** `collect_git_identity` already `pub(crate)`.
6. **F14 / AC11:** `after_help` optional.

---

## Preflight (plan time — 2026-08-18)

| Check | Result |
|-------|--------|
| HEAD / tree | Plan dogfood `fa90981`. Plan/fold-in docs `d4555f2`. Product `src/` unchanged. |
| T267 stub | Placeholder upgraded in place to **Planned** |
| PATH `ai-brains` | **0.1.1**. Same holes (status self-next; leftover+AI-Brains footer). **Do not `cargo install`.** |
| Live hole | `harness status`: 5× `next: ai-brains harness status` + five install trailers. `project list`: `set-alias 7d97a456 … AI-Brains`. `whoami` remediations **(none)** / T258 already adopt-path. Preflight harness block already omits ok `next:`. |
| SoT | `next_action_for` `:286`; `harness.rs` `:58`; `print_unaliased_footer` `:104`; `footer_alias_suggestion` `:125`. |
| clap / serde_json | lock clap **4.6.1** / crates.io **4.6.6**; serde_json lock **1.0.150** / crates.io **1.0.151**. rustc **1.95.0**. **No clap 5.** Snapshot — re-verify at execute. |
| Last PR Cursor | #180 comments/reviews **empty**. #179 Bugbot Medium still true → **T272** already minted. No open PR on `main`. |
| `deferred.md` | Full scan. Overlap: audit T267 **absorb**; T259 footer **absorb**; T258 F2 **affirm**; T212 chrome **partial**; T235 F40 **affirm**; T265–T272 except T267 **decline**; T240 F2 / T255 **decline**. |
| ai-brains | `preflight --summary` 3581317d / pin count **volatile** (3046→3058) / grants 0 of 3 (T241). Leftover `list-paths` **11** roots. |
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
| OpenCode m3 AC10 missing hermetic | review | **Absorb** AC10 named additive |
| Agy m2 Ok-row unit | review | **Absorb** AC15 |
| Leftover 11 paths (not 1) | fold-in verify | **Absorb** AC7/AC16 |
| OpenCode O2 ledgerful hygiene | review | **Decline** |

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
- [ ] `footer_alias_suggestion__non_owner__not_cwd_slug` (AC14)
- [ ] `format_harness_summary_lines__ok__omits_next` (AC15)
- [ ] Hermetic AC6 / AC7 (`project_list__footer__multipath_leftover_plus_orphan__picks_orphan`) / AC16 (`project_list__footer__leftover_only__basename_not_cwd_slug`)
- [ ] `harness_install__success__next_is_status` (AC10) — **additive; does not exist today**
- [ ] `cargo nextest run -p ai-brains-cli` — new tests **fail** (Ok still `harness status`; leftover still first)
- [ ] Commit red allowed

---

## Phase 2 — Green (harness)

- [ ] `next_action_for` Ok → `"none"` (F1)
- [ ] `run_status` omit `next:` when none/Ok (F1)
- [ ] F6 trailer: only present && !Ok
- [ ] Do **not** change install/uninstall success next (F7)
- [ ] Preflight: **required** AC15 Ok-row unit; do not rewrite summary (F8)
- [ ] Additive AC10 hermetic `harness_install__success__next_is_status`
- [ ] Targeted: `cargo clippy -p ai-brains-cli --all-targets -- -D warnings` ; AC1–AC4 / AC9 / AC10 / AC15 green
- [ ] Commit green allowed

---

## Phase 3 — Green (footer)

- [ ] Add `project_list_footer.rs` + `mod` (F10). Pure pick/suggestion. Slug via `collect_git_identity`
- [ ] Implement F3 pick + F3b suggestion (no leftover UUID). `path_count` from `list_path_aliases`, not first-path
- [ ] `project.rs` `list()` calls helper; delete old footer body
- [ ] T212 AC3/AC4/AC5 stay green (AC8)
- [ ] T258 AC7 stays green (AC5)
- [ ] AC6 / AC7 / AC14 / AC16 green
- [ ] `project.rs` **total** line count does not grow vs 1511 (AC12)
- [ ] Commit green allowed

---

## Phase 4 — Docs

- [ ] CAPABILITIES: harness ok omit/`none`; list footer F3/F3b (F14 / AC11)
- [ ] PROTOCOL-COMPAT additive `next_action: "none"` (F14)
- [ ] `project list` `after_help`: **optional** one-liner (F14). Current text is already neutral
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

- [ ] AC1–AC16 pass with evidence
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
