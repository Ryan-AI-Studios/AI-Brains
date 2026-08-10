# T219 Internal Review

## Verdict: CLEAN

Core F1/F2b word-budget fix, F6 Scope wire, F7/F8 dual strip converge, F9/F10/F14/F31 pretty formatter, F5 JSON isolation, F12 summary isolation, and docs are in place with solid pure units. Round-1 Mediums **M1/M2** and **L1** re-verified fixed (2026-08-09 re-review). No new Medium/High findings. Open residuals are Low process only (L2 soft honesty, L3 AC12/AC13 gate/dogfood). Process residuals remain for orchestrator before track close.

## Scope

- **Branch / intent:** `feat/T219-preflight-pretty-readability` — preflight pretty readability (newline word budget + F2b + human Scope/caps/role-strip; JSON 2-key with structured `text`).
- **Audited production:**
  - `C:\dev\AI-Brains\crates\ai-brains-retrieval\src\word_budget.rs`
  - `C:\dev\AI-Brains\crates\ai-brains-retrieval\src\preflight.rs` (word_count / F2b / `truncate_turn`)
  - `C:\dev\AI-Brains\crates\ai-brains-cli\src\commands\display_text.rs`
  - `C:\dev\AI-Brains\crates\ai-brains-cli\src\commands\memory.rs` (dual strip callers)
  - `C:\dev\AI-Brains\crates\ai-brains-cli\src\commands\preflight.rs` (pretty formatter + human/JSON wire)
- **Tests:** pure units in `word_budget.rs`, `display_text.rs`, `memory.rs`, `preflight.rs`; hermetic `crates/ai-brains-cli/tests/preflight_pretty_readability.rs`
- **Docs:** `Docs/CAPABILITIES.md`, `CHANGELOG.md`
- **Spec/plan:** `conductor/tracks/trackT219-preflight-pretty-readability/spec.md`, `plan.md`
- **Re-review (2026-08-09):** read-only verification of M1/M2/L1 fixes in Session arm + units; no code changes in this pass.

## Requirement matrix (AC + key F)

| ID | Status | Evidence / notes |
|----|--------|------------------|
| **AC1** | Met | `trim_to_word_budget__multiline_under_budget__preserves_newlines` |
| **AC2** | Met | `trim_to_word_budget__multiline_over_budget__keeps_structure_until_cut` |
| **AC3** | Met | Hermetic multi-line + blank after `---` header |
| **AC4** | Met | Hermetic `Scope: project=` / not `Repository:` |
| **AC5** | Met (unit strong; hermetic OK if pin→turn stores `ASSISTANT:`) | Pure `format_preflight_pretty_body__role_strip_*`; hermetic asserts no leading `ASSISTANT:` |
| **AC6** | Met | Pure F31 wording unit + M1 multi-line turn unit + M2 notice-before-index unit; L1 non-tautology assert |
| **AC7** | Met | Hermetic compact 2 keys + deserialized `text` has `\n`; no Scope chrome |
| **AC8** | Met | Hermetic summary smoke; dual banner/Scope; no full-body safety dump |
| **AC9** | Met | `display_text` unit + `preview_line` / `content_has_tag` both call shared helper + existing memory units |
| **AC10** | Met | `content_word_count` on `PreflightContext.word_count`; under-1500 still `word_count <= 1500` |
| **AC11** | Met | CAPABILITIES preflight T219 rows; CHANGELOG T219 entry (F2b + governed F1) |
| **AC12** | Residual | Plan Phase 6 full CI gate unchecked; not re-run in this review |
| **AC13** | Met | Manual dogfood on `target\debug\ai-brains.exe`: multi-line Scope, no ASSISTANT index, `+N more via recall`, JSON 2-key ~88 newlines; plan checkbox marked |
| **AC14** | Met (body) | Pure unit preserves `#`/`##`; Scope prepend is same `human_mode` wire (no governed hermetic) |
| **AC15** | Met | Over/under budget sentinel units |
| **AC16** | Met | Invariant unit `content_word_count == 3` + `\n` |
| **AC17** | Met | CRLF unit |
| **AC18** | Met | Orphan header unit |
| **F1 / F2 / F2b / F32** | Honored | Line walk, CRLF strip, trailing `…` own line, content count excludes trailing sentinel |
| **F4 / F5** | Honored | Pretty polish only on `human_mode`; JSON raw `context.text` + `word_count` |
| **F6 / F6b** | Honored | CLI `get_project_by_id` + `format_scope_line`; no `Repository:` |
| **F7 / F8 / F39** | Honored | `&str` helper; both memory callers; mid-line/lowercase left |
| **F9 / F10 / F29 / F31 / F37** | Honored | Caps constants; F31 strings exact; orphan omit; **logical turn count (M1)**; **sessions notice on last session part (M2)** |
| **F12 / F40** | Honored | Summary early-return unchanged |
| **F13** | Honored | No marker/selection SQL change |
| **F14** | Honored | Only `---` headers; `##` not re-bucketed |
| **F16** | Honored | No new crates |
| **F38** | Soft done | `truncate_turn` avoids double chrome when F2b already present |

## Findings

### [M1] Session cap counts physical lines, not logical turns
- **severity:** Medium
- **description:** `PrettySectionKind::Session` increments `turn_count` / `turn_total` for every non-empty physical line. Retrieval emits multi-line turns via `truncate_turn` (up to 3 lines) then `format!("{ROLE}: {truncated}")` and joins with `\n`, so one turn becomes multiple body lines. That inflates the cap, can cut mid-turn after `PRETTY_TURNS_PER_SESSION` lines, and can emit false `+N more turns in session` even when SQL already returns ≤5 turns. Unit AC6 only uses single-line fixtures, so it would pass without correct multi-line turn handling.
- **files:** `crates/ai-brains-cli/src/commands/preflight.rs` (`PrettySectionKind::Session` arm); producer `crates/ai-brains-retrieval/src/preflight.rs` (`truncate_turn` + session line join)
- **required_fix:** Count a turn only on role-leading lines (`USER:` / `ASSISTANT:` / `SYSTEM:` after trim) and keep following non-role lines as part of the current turn until the next role line or blank separator; only emit F31 turn overflow from true turn totals. Add a pure unit with a multi-line turn that stays under 6 turns but would exceed 6 physical lines.
- **status:** verified_fixed
- **fix evidence:** `is_session_turn_start` + continuation lines (`in_open_turn`); unit `format_preflight_pretty_body__multiline_turns__cap_by_role_starts`
- **re-verify (2026-08-09):** Session arm only increments `turn_total`/`turn_count` on `is_session_turn_start` (leading `USER:`/`ASSISTANT:`/`SYSTEM:` after `trim_start`); non-role lines append while `in_open_turn && turn_count <= PRETTY_TURNS_PER_SESSION`; past-cap role starts set `in_open_turn = false` so continuations drop; overflow notice uses `turn_total`. Matches producer shape: `format!("{}: {}", role.to_uppercase(), truncate_turn(...))` puts role only on first line of multi-line body. Unit covers 3×3 multi-line (no false overflow) and 7 single-line role-starts (`+1 more turns`, keeps turn 6, drops turn 7).

### [M2] `+N more sessions` notice attaches to last out_part, not last session
- **severity:** Medium
- **description:** After the section loop, session overflow notice is appended to `out_parts.last()` (comment claims “after last session section content”). Legacy assembly order is safety → sessions → Memory Index / Most Recent, so the notice often lands under Index/Recent chrome, which is misleading for F31 session wording.
- **files:** `crates/ai-brains-cli/src/commands/preflight.rs` (`sessions_skipped` block after section loop)
- **required_fix:** Emit `+N more sessions` immediately after the last *emitted* session section (or as its own part inserted at the session boundary), before subsequent Index/Recent sections. Unit-lock: with 4 sessions + following Memory Index, notice appears after the 3rd session block / before index content (or clearly between), not only as a trailing Index line.
- **status:** verified_fixed
- **fix evidence:** `last_session_part_idx` set to `out_parts.len()` before push on Session parts; post-loop appends notice to that index (fallback standalone part if none); unit `format_preflight_pretty_body__sessions_notice__before_index` + AC6 order assert
- **re-verify (2026-08-09):** `last_session_part_idx` tracks last *emitted* session section only (skipped overflow sessions never push). Notice append uses `\n` onto that session part, then `out_parts.join("\n\n")` places Index after — byte-order asserts `notice < index` in both dedicated unit and AC6 over-cap unit.

### [L1] AC6 secondary assert is nearly tautological
- **severity:** Low
- **description:** `!out.contains("+2 more via recall") || out.contains("+3 more via recall")` is true whenever the stronger `+3 more via recall` assert already passed. Does not add failure modes.
- **files:** `crates/ai-brains-cli/src/commands/preflight.rs` (test `format_preflight_pretty_body__over_cap_sections__f31_wording`)
- **required_fix:** Optional: assert `!out.contains("+2 more via recall")` and/or exact unique safety vs index phrases without OR tautology.
- **status:** verified_fixed
- **fix evidence:** replaced with `!out.contains("+2 more via recall")` (18 index items → cap 15 → must be +3 only)
- **re-verify (2026-08-09):** assert is non-tautological: fails if wrong overflow N is emitted alongside or instead of correct +3 wording.

### [L2] Intermediate section trims can leave mid-body F2b `…` counted as content words
- **severity:** Low
- **description:** `trim_to_word_budget` is used on safety/index *sub*-assemblies before the final join. Those intermediate sentinels sit mid-document; `content_word_count` only strips a *trailing* sentinel. Remaining-budget math also uses raw `word_count` on joined sections (sentinel can cost 1 word of remaining budget). Final `word_count` field still ≤ `max_words` (AC10 holds). Soft honesty/noise residual, not DoD-breaking.
- **files:** `crates/ai-brains-retrieval/src/preflight.rs` (safety/index partial trims + `remaining_budget`); `word_budget.rs` (`content_word_count`)
- **required_fix:** Optional follow-up: use `content_word_count` for remaining-budget arithmetic; avoid intermediate F2b chrome or strip mid-body section sentinels when recomposing (track residual if deferred).
- **status:** open

### [L3] AC12 / AC13 process residuals
- **severity:** Low
- **description:** Plan Phase 6 still lists full CI gate + manual dogfood as unchecked. Not a code defect; clearance should not claim AC12/AC13 without evidence.
- **files:** `conductor/tracks/trackT219-preflight-pretty-readability/plan.md`
- **required_fix:** Run full gate; record AC13 dogfood outcomes before track close.
- **status:** open

## Completeness

| Area | Assessment |
|------|------------|
| Root single-line wall (F1) | Fixed at SOOT `trim_to_word_budget`; units lock preserve/truncate/CRLF/invariant |
| F2b JSON honesty | Trailing `…`; `PreflightContext.word_count` uses `content_word_count` |
| Pretty human path | Scope + pretty body wired; caps constants; no PrettyOpts over-engineering |
| JSON path | Compact `PreflightContextResponse` only; no Scope/caps chrome |
| Role strip dual SOOT | `display_text::strip_role_prefix`; `preview_line` + `content_has_tag` both call it |
| Governed `##` | Pure unit; only `---` recognized as section headers |
| Summary | Unchanged early path |
| Marker selection | Untouched |
| Docs | CAPABILITIES + CHANGELOG cover pretty, F2b, governed F1, F31, Scope, JSON |
| Gaps | **None open for M1/M2/L1.** Remaining: L2 soft mid-body sentinel; L3 process gate/dogfood |

No production `unwrap()` / `expect()` introduced in the T219 production paths reviewed (`word_budget`, `display_text`, pretty wire). Existing `Option::unwrap_or` / test-only unwraps are out of scope.

## Wiring

| Path | Behavior | OK? |
|------|----------|-----|
| `human_mode` (`pretty` \|\| format human/pretty \|\| TTY default human) | `format_scope_line` (CLI alias via `get_project_by_id`) + blank + `format_preflight_pretty_body(&context.text)` | Yes |
| JSON non-summary | `serde_json::to_string` of `{text, word_count}` from raw post-F1 context | Yes |
| Summary | Early `print_summary` return; hermetic AC8 smoke | Yes |
| Retrieval assembly | Final + governed re-budget via newline-preserving trim; content word_count excludes trailing sentinel | Yes |
| T180 compact envelope | Outer JSON remains single-line (`to_string`); newlines live inside escaped `text` | Yes (compat test still valid) |

## Test quality

| Suite | Quality |
|-------|---------|
| `word_budget` units | Strong: AC1/2/15/16/17 + empty/max0; would fail pre-F1 space-join |
| `strip_role_prefix` unit | Strong leading/mid/lower |
| memory `preview_line` / `content_has_tag` | Strong dual-caller behavior (pre-existing + shared helper) |
| pretty pure units | Strong F31 wording, orphan, governed `##`, role strip; **M1 multi-line turn unit + overflow unit; M2 notice-before-index unit; L1 non-tautology** |
| Hermetic pretty | Good AC3/4/7/8 integration; AC5 depends on pin storage shape (turn projection prefixes `ASSISTANT:` — likely exercises strip) |
| Weak spots | No hermetic over-cap F31 (acceptable — pure unit preferred) |

## Residuals

- **Soft F22 / out of scope (as planned):** `--compact`, is-terminal migrate, clap pin bump, retrieval-side role strip for JSON, pager, T224 full search strip, T228 recall Scope, marker policy, PrettyOpts.
- **AC14 Scope on governed human path:** same `human_mode` prefix; no dedicated governed hermetic — residual only.
- **F38:** soft truncate_turn double-chrome guard present; no dedicated pathological `\n\n...` unit beyond that logic.
- **Soft (not a regression):** session non-role lines that themselves start with `USER:`/`ASSISTANT:`/`SYSTEM:` would count as new turns; producer only prefixes role on the first line of `truncate_turn` output — acceptable.
- **Clearance gate:** Mediums M1/M2 + L1 are `verified_fixed`. Do not mark track complete until AC12 full gate green and AC13 dogfood recorded (L3). L2 remains optional follow-up.

## Re-review notes (2026-08-09, M1/M2/L1)

| Finding | Verdict | Evidence |
|---------|---------|----------|
| M1 | verified_fixed | Session arm logical-turn walk + `format_preflight_pretty_body__multiline_turns__cap_by_role_starts` |
| M2 | verified_fixed | `last_session_part_idx` + `format_preflight_pretty_body__sessions_notice__before_index` + AC6 order assert |
| L1 | verified_fixed | `!out.contains("+2 more via recall")` in AC6 unit |
| New findings | **None** (no new Medium/High; no regression from the fix) |

- Code inspection: `format_preflight_pretty_body` Session arm + post-loop sessions notice block (`preflight.rs` ~256–485); producer multi-line turn join (`ai-brains-retrieval` `preflight.rs` ~337–365, `truncate_turn` ~787–805).
- Optional nextest filter `test(format_preflight_pretty_body)` was intended for local confirmation; review decision is grounded in code+unit correctness even if CI gate (AC12) is still orchestrator residual.
- No production code edited in this re-review pass; only this `review.md` updated.

## Codex cross-model (round 1) — disposition

Source: `review.codex.md` (gpt-5.4 high, read-only).

| Finding | Disposition |
|---------|-------------|
| P0 | none |
| P1 process DoD unchecked | **Process** — expected mid-track; plan checkboxes + AC13/review updated; full gate + ledger closeout still pending before Completed |
| P2 governance still Planning | **Fixed** — conductor/spec/deferred/series README → **In Progress** |
| P3 mid-body F2b budget | **Fixed** — `trim_to_word_budget_no_sentinel` for intermediate safety/index; remaining_budget via `content_word_count` |

Product wiring: Codex affirmed core behavior (F1/F2b/F5/F6/F7/F9/F12/F14/T180). No open product P0–P2 after fixes.

## Codex final product gate

Source: `review.codex.final.md` — **PASS WITH DEFERRED P3**
- P0/P2 none; product P1 none
- Prior R1 P2/P3 verified fixed
- Remaining: process closeout (CI merge, ledger commit, Completed status) + AC13 matrix row drift (fixed this commit)
