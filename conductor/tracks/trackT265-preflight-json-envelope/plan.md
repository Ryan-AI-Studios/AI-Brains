# T265 Plan — Preflight JSON envelope

**Status:** **Completed** (2026-08-19)
**Spec:** [spec.md](./spec.md) F0–F26 / AC1–AC16 + §13 fold-in
**Category:** UX / CONTRACTS / FEATURE
**Ledger TX (planning):** `5fa57d64-9fac-4a8e-932a-d0f23c29f347` (DOCS)
**Ledger TX (fold-in):** `d6aa8b35-5970-4fa5-ba49-6168c11fe656` (DOCS)
**Ledger TX (on go):** `b5b7c4e8-a8a0-4465-be35-625afc6ead0b` (FEATURE)

---

## AI fold-in (2026-08-19) — `agy-review.md` + `opencode-review.md`

No Blockers / Majors. OpenCode F5 match rules + F6 session collapse folded as F5/F6/AC3/AC15. AC5 preamble-discard unit required. Agy `pub(crate) mod` is sibling `pub mod`. Disposition in spec **§13**.

### Pins locked by fold-in

1. **F5:** pretty `contains` / `starts_with` + Ledgerful `contains("Ledgerful Intelligence")`.
2. **F6 / AC3:** Session `\n`-joined turns → `items.len()==1`. No JSON turn-split.
3. **AC15:** live plain + Fallback Ledgerful header strings.
4. **AC5:** `split_preflight_sections__leading_preamble__discarded`.
5. **AC11:** never fabricate `empty_repo`.
6. **F12:** `pub mod` matching siblings; CLI `pub(crate) const` ids optional; no contracts const DoD.

---

## Preflight (plan time — 2026-08-19)

| Check | Result |
|-------|--------|
| HEAD / tree | Plan dogfood `2a00ce3`. Plan/fold-in docs `7192070`. Product `src/` unchanged. |
| T265 stub | Placeholder upgraded in place to **Planned** |
| PATH `ai-brains` | **0.1.1** PATH-behind (mtime 2026-08-18). JSON still 2-key. **Do not `cargo install`.** |
| Source JSON | `preflight --format json -m 200` → keys `text`,`word_count` len **2**; Bearings + Session; 22 newlines; `word_count=200`. **Live hole.** |
| SoT | contracts `PreflightContextResponse`; CLI emit `preflight.rs:279–286`; T180-C `len==2`; pretty classifier `:360`. |
| clap / serde_json | lock clap **4.6.1** / crates.io **4.6.6**; serde_json lock **1.0.150** / crates.io **1.0.151**. rustc **1.95.0**. **No clap 5.** Snapshot — re-verify at execute. |
| Last PR Cursor | #181 comments/reviews **empty**. #179 Bugbot Medium still true → **T272**. No open PR on `main`. |
| `deferred.md` | Full scan. Overlap: audit T265 **absorb**; T214 extra keys **absorb**; T220/T264/T266 freeze **absorb as lift**; T257 compact **affirm**; T272 / T268–T271 / T240 F2 / T255 **decline**. |
| ai-brains | `preflight --summary` 3581317d / pin count **volatile** (3089) / grants 0 of 3 (T241). |
| ledgerful | doctor ready (hygiene warns). 0 pending 0 drift. Hotspot **#1** `project.rs` — do not edit. **#7** CLI `preflight.rs` 2148 — sibling. Retrieval `preflight.rs` T272 — do not edit. Index incremental completed (0 file delta). |
| Research | clig.dev JSON structure + additive future-proofing; serde ignore-unknown; T180 extra-field policy. json-v2 declined. |
| `ISSUES.md` | **Does not exist** |
| Live `.env` / bootstrap / nightly mutate / pin | **Not written** / **not run** / **not scheduled** / **not pinned** this pass. |

---

## Absorbed deferred

| Item | Source | Plan action |
|------|--------|-------------|
| `{text, word_count}` blob (7/6) | audit T265 | **DoD** F1–F8 / AC1–AC4 |
| T214 `PreflightContextResponse` extra keys | T214 F11 residual | **Absorb** `sections` only |
| T220 / T264 / T266 “do not grow T180” | peers | **Absorb** as freeze this track lifts |
| T257 compact + `note_machine_stdout` | T257 | **Affirm** F15 |
| T220 summary JSON | T220 | **Decline** F9 / AC10 — leave |
| json-v2 / typed arrays | placeholder fork | **Decline** F10 |
| T272 `safety_ids` | Cursor #179 | **Decline** F11 — already minted |
| T268 / T269 / T270 / T271 | series | **Decline** F24 |
| T240 F2 / T255 bag | standing | **Decline** F23 |
| last-PR Cursor #181 | empty | **N/A** |
| OpenCode F6 session collapse | review | **Absorb** F6 / AC3 — one item; no turn-split DoD |
| OpenCode F5 match rules | review | **Absorb** F5 / AC15 live variants |
| OpenCode preamble / AC11 fabricate | review | **Absorb** F6 / AC5 / AC11 |
| Agy m2 `pub(crate) mod` | review | **Partial** F12 sibling `pub mod` |
| Agy O1 contracts consts | review | **Partial** CLI sibling only |

---

## Phase 0 — on go (re-verify)

- [x] Re-read `PreflightContextResponse`, CLI JSON arm `:279–286`, T180-C, T219 AC7, T250 AC12, T264 AC9.
- [x] Confirm source `preflight --format json -m 200` is still 2-key.
- [x] Confirm T272 still at retrieval `preflight.rs:329` + `:467`.
- [x] Re-check lock clap **4.6.1** / crates.io clap. rustc **1.95.0**. No clap 5. serde_json lock vs crates.io.
- [x] Rescan **entire** `conductor/deferred.md`.
- [x] Last merged PR Cursor comments — #181 leftover none; T272 still the #179 mint.
- [x] `ledgerful ledger start T265-preflight-json-envelope --category FEATURE` (`b5b7c4e8-a8a0-4465-be35-625afc6ead0b`)

---

## Phase 1 — Red (failing tests first)

- [x] Unit `split_preflight_sections__legacy_headers__ids_in_order` (AC3)
- [x] Unit `split_preflight_sections__two_sessions__two_section_rows` (AC4)
- [x] Unit `split_preflight_sections__no_headers__empty` (AC5)
- [x] Unit `split_preflight_sections__leading_preamble__discarded` (AC5)
- [x] Unit `split_preflight_sections__governed_marker__one_section` (AC6)
- [x] Unit `preflight_context_response__n_minus_1_two_key__sections_default_empty` (AC7)
- [x] Unit `split_preflight_sections__ledgerful_and_other` (AC15)
- [x] Confirm T180-C still fails the new `sections` assert (or still `len==2` until green)

---

## Phase 2 — Green (DTO + split + emit)

- [x] Grow `PreflightContextResponse` + nested `{id,title,items}`; `#[serde(default)]` on `sections`; **no** `deny_unknown_fields`
- [x] Add `commands/preflight_json.rs` + `pub mod` matching siblings (F12); F5 match table; F6 session one-item
- [x] Wire JSON arm to sibling; keep `to_string` + `note_machine_stdout`
- [x] AC1 / AC2 / AC16
- [x] No retrieval `preflight.rs` edits

---

## Phase 3 — Existing `len==2` tests

- [x] T219 AC7: keep newlines + no Scope; drop `len==2`; assert `sections` array (AC8)
- [x] T250 AC12: keep uncapped seed; drop `len==2` (AC8)
- [x] T264 AC9: keep `[8hex]`; drop `len==2` (AC9)
- [x] T220 summary hermetics stay green (AC10)
- [x] Dogfood extra-key still scans `text` (AC14)
- [x] Empty-vault hermetic AC11 (no fabricated `empty_repo`)
- [x] `preflight_contextual_risk.rs` `test_preflight_json_output_with_scope` stays green (`"word_count":`)

---

## Phase 4 — Docs

- [x] CAPABILITIES full JSON row: required `text`/`word_count`; additive `sections`; E1 `[]`; compact; ids
- [x] PROTOCOL-COMPAT §5 + T180-C row
- [x] Root CHANGELOG T265
- [x] clap `--format` docstring + after_help one-liner (F26)
- [x] Rewrite T220 “Never grows…” comment in `preflight.rs` (F9)

---

## DoD (checkable)

- [x] F0–F26 honored (especially F5 match table, F6 session one-item, F10 json-v2 decline, F11 retrieval freeze, F12 sibling `pub mod`, F25 no version key)
- [x] AC1–AC16 green
- [x] T272 file untouched
- [x] `project.rs` / `governed_common.rs` untouched
- [x] No clap 5 / no lock bumps / no new crates
- [x] Phase-1 review clean; `codex-review` (F18)
- [x] Full gate; FEATURE TX committed; PR squash-merged (implement-track)

---

## Stop-before

- Scope exceeds F1–F8 (typed arrays, json-v2, retrieval/T272, summary envelope, pretty refactor)
- Missing secrets / live `.env` write
- T240 F2 / T255 reopen
- Push to `main`

---

## Not this track

T268 scan-roots; T269 nightly/Router; T270 classify; T271 ledger pane; T272 safety_ids; T241 grants; PATH `cargo install`.
