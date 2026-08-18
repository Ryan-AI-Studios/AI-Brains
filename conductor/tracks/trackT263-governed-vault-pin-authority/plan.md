# T263 Plan — Governed honesty (H1 only)

**Status:** **In Progress**
**Spec:** [spec.md](./spec.md) F0–F29 / AC1–AC14 + §13 fold-in
**Category:** FEATURE / UX
**Ledger TX (planning):** `bcc514c0-8f84-48d6-b8d7-779195d7c630` (DOCS)
**Ledger TX (fold-in):** `32e9608c-3317-4bfd-b168-44a9485c1123` (DOCS)
**Ledger TX (implement):** `6c40de3d-10ce-4524-a3e4-0c6488493446` (FEATURE)

---

## AI fold-in (2026-08-18) — `agy-review.md` + `opencode-review.md`

No Blockers / Majors. Agy **m2** folded as **F29** / **AC14** (≤140 / one line). OpenCode Personal deny live path folded as **F4** / **F23** / **AC3**. Disposition in spec **§13**.

### Pins locked by fold-in

1. **F29 / AC14:** `BRIEFING_EMPTY_AUTHORITY_NEXT_STEP` ≤140 chars, no newline.
2. **F4 / F23 / AC3:** Personal constants in `renderer.rs`; `personal.rs:121` one-line hint swap; update `briefing_personal__no_grants__soft_deny_denial_hint`.
3. **§2.1:** `b2aae2d` product vs `a8cf801` plan.
4. **O1:** still F8 helper.

---

## Preflight (plan time — 2026-08-18)

| Check | Result |
|-------|--------|
| HEAD / tree | Plan dogfood `b2aae2d`. Plan commit `a8cf801`. Fold-in docs on that product src. |
| T263 stub | Placeholder upgraded in place to **Planned** |
| PATH `ai-brains` | `C:\Users\RyanB\.cargo\bin\ai-brains.exe` **0.1.1**. **Do not `cargo install`.** |
| Live hole | Daily `3581317d`: **0 of 3** grants (T241). Leftover `441837f6` (`test-alias`): **3 grants**, briefing **empty_authority** + “seed an Approved decision”. Progressive authorized-empty already names `recall` (T243). Expand `Unknown` `preview: ""`. Trace `null`. Personal deny → bootstrap on `Personal:a1b2a1b2-…`. |
| SoT | `renderer.rs` empty/deny constants; `governed_query.rs` F31 + expand; `governed_common.rs` `PROGRESSIVE_RECALL_FALLBACK`; T167 pins → Evidence; `help_ia.rs` tip. |
| clap / serde_json | lock clap **4.6.1** / crates.io **4.6.6**; serde_json lock **1.0.150** / crates.io **1.0.151**. rustc **1.95.0**. **No clap 5.** Snapshot — re-verify at execute. |
| Last PR Cursor | #177 comments/reviews/inline **0**. **N/A.** |
| `deferred.md` | Full scan. Overlap: audit T263 **absorb**; T227 seed-Approved **absorb**; T227 F3 **affirm**; H2/T167 **decline**; T241 leftovers **partial/decline**; T264/T266/T267 **decline**; T240 F2 / T255 **decline**. |
| ai-brains | `preflight --summary` 3581317d / 2952 pins / grants 0 of 3. Recall no prior H1 pin. |
| ledgerful | doctor ready (hygiene warns). 0 pending 0 drift. Hotspot **#1** `project.rs` — do not touch. `#9` `personal.rs` — prefer renderer. Index incremental completed. |
| Research | clig.dev next-command after setup; T180 P-CLI; T167 under-promote; T170 stop-before; clap 4.6.6 current. |
| `ISSUES.md` | **Does not exist** |
| Live bootstrap / migrate / `.env` / nightly | **Not run** / **not written** / **not scheduled** this pass. |

---

## Absorbed deferred

| Item | Source | Plan action |
|------|--------|-------------|
| 3 grants / 0 authority | audit T263 | **DoD** H1 F1–F10 / AC1–AC14 |
| Briefing “seed Approved” | T227 F8 live | **Absorb** F2 / AC1 / AC4 |
| Dual-model no pin inject | T227 F3 | **Affirm** F3 |
| Personal unused vs bootstrap | placeholder F3 | **Absorb** F4 / F23 / AC3 (`personal.rs:121`) |
| Empty-authority length | Agy m2 | **Absorb** F29 / AC14 |
| Expand empty preview | audit expand 6/6 | **Absorb** F7 / AC5 |
| Trace literal `null` | placeholder F2 | **Decline wrap** F6 / AC6 document |
| List `items: []` no remediator | audit 3/5 | **Absorb** F8 / AC7 |
| Help/skill progressive oversell | placeholder F1 | **Absorb** F10 / AC9 / AC11 |
| Progressive empty next | T243 | **Leave** F9 / AC10 |
| H2 pin promotion | placeholder | **Decline** F11 |
| T170 GOVERNED_BRIEFING | placeholder F5 | **Affirm** F12 |
| T241 0-grant daily Scope | live | **Decline** F14 |
| T264 / T266 / T267 | series | **Decline** F28 |
| T240 F2 / T255 | standing | **Decline** |
| last-PR Cursor | #177 | **N/A** — no leftover to mint |

---

## Phase 0 — on go (re-verify)

- [ ] Re-read `renderer.rs` empty/deny constants and T227 units.
- [ ] Re-read `governed_query.rs` expand + `run_trace` F31.
- [ ] Re-read `legacy_import.rs` pin → Evidence (H2 still invalid).
- [ ] Classify-only dogfood: leftover granted briefing still `empty_authority`; daily deny still bootstrap. **Do not** `policy bootstrap`. **Do not** pin. **Do not** migrate.
- [ ] Re-check lock clap **4.6.1** / crates.io current. rustc **1.95.0**. No clap 5.
- [ ] Rescan **entire** `conductor/deferred.md`.
- [ ] Last merged PR comments/reviews/inline. Mint a placeholder only if a real leftover fits nowhere.
- [ ] `ledgerful ledger start T263-governed-vault-pin-authority --category FEATURE`

---

## Phase 1 — Red (failing tests first)

- [ ] `briefing_empty_authority_next_step__contains_recall_not_seed_approved`
- [ ] `render_project_markdown__allowed_empty__names_recall` (update T227 unit)
- [ ] `render_personal_markdown__denied__names_recall_not_personal_bootstrap`
- [ ] `briefing_empty_authority_next_step__one_line_at_most_140_chars` (AC14)
- [ ] `expand_unknown__preview_nonempty`
- [ ] `apply_authorized_empty_list_next__empty_items__sets_recall`
- [ ] `root_after_long_help__tip_names_recall_not_progressive`
- [ ] Update hermetic `briefing_personal__no_grants__soft_deny_denial_hint` (AC3 — must fail on current bootstrap hint)
- [ ] Hermetic granted-empty briefing + expand + lists (`governed_vault_pin_honesty` or extend existing)
- [ ] Confirm red: old seed-Approved string still in src so new asserts fail

---

## Phase 2 — Green

- [ ] Rewrite `BRIEFING_EMPTY_AUTHORITY_NEXT_STEP` (F2)
- [ ] Specialize Personal deny next / hint (F4): renderer constants + `personal.rs:121` only; Repository deny unchanged (F5)
- [ ] Enforce F29 ≤140 one-line empty-authority next
- [ ] Fill expand `Unknown` preview SOOT (F7)
- [ ] List overlay helper + evidence/source/review emit (F8)
- [ ] Help tip (F10)
- [ ] Docs: CAPABILITIES §15, WORKFLOWS, OPERATIONS, skill, CHANGELOG (AC11)
- [ ] Trace after_help honesty only (F6)
- [ ] No `project.rs`. No `legacy_import.rs`. No live bootstrap.

---

## Phase 3 — Verify

- [ ] `cargo clippy -p ai-brains-cli -p ai-brains-control-plane --all-targets -- -D warnings`
- [ ] Targeted nextest: renderer units + hermetics AC1–AC10
- [ ] T227 deny AC2 still green
- [ ] T243 progressive hint AC10 still green
- [ ] `ledgerful verify --scope fast`
- [ ] Manual AC13 on source/hermetic bin (not PATH)

---

## Phase 4 — Review + closeout (implement-track)

- [ ] `review.md` Phase 1 clean (mediums fixed or ≤3 deferred)
- [ ] FEATURE `codex-review`
- [ ] conductor Completed + deferred T263 closeout **only after** push → PR → GHA green → squash-merge → prune
- [ ] Pin `DECISION: T263 H1 only — vault pins are not governed authority`

---

## Definition of done

- [ ] AC1–AC14
- [ ] F0–F29 honored (H2 / live bootstrap / GOVERNED_BRIEFING / clap 5 / T240 F2 not done)
- [ ] No product commits from this **planning** pass
- [ ] Medium+ review findings not silently dropped

---

## Stop-before (even on go)

- Live `policy bootstrap`
- Live `migrate governed --confirm`
- `AI_BRAINS_GOVERNED_BRIEFING=1` on production preflight
- `cargo install` / `.env` rewrite / `nightly` schedule mutate
- Push to `main` / force-push
- Scope exceeds H1 (H2, T264–T271, T240 F2, T255)
