# T250 internal review r1 — Preflight pretty density

Reviewer: Grok (read-only). Scope: product files only (no conductor closeout). Authority: working-tree Planning spec `spec.md` F1–F16 / AC1–AC16 + §14 pins. Compared working tree + HEAD on `feature/T250-preflight-pretty-density`.

## Verdict: PASS

## Requirement matrix (AC1–AC16)

| ID | Result | Evidence |
|----|--------|----------|
| **AC1** | **Met** | `format_preflight_pretty_body_with__session_line_over_140__capped_ellipsis` and `__recent_line_over_140__capped_ellipsis`: 200-char bodies through `PrettyCaps::standard()` are ≤140 and end with `…`. `__session_line_80__unchanged` keeps the 80-char body with no ellipsis. |
| **AC2** | **Met** | `pretty_caps_standard__t219_item_caps` locks 8 / 6 / 3 / 15 / recent 3 / `line_max=140` / `first_line_only=false`. Existing T219 `format_preflight_pretty_body__over_cap_sections__f31_wording` still calls the `standard()` wrapper; fixtures stay short so F31 wording is unchanged. |
| **AC3** | **Met** | `pretty_caps_compact__item_caps_and_f31_and_recent_hint`: safety 3 (`+7 more safety…`), turns 2 (`+2 more turns…`), sessions 1 (`+1 more sessions`), index 5 (`+3 more via recall`), recent 2 (third item dropped), `first_line_only` drops Recent continuation, recall hint kept. |
| **AC4** | **Met** | `strip_pretty_chrome` units: `(just now) ASSISTANT: DECISION: x` → `DECISION: x`; `(10 hr ago) USER: hi` → `hi`; `(999 mo ago) ASSISTANT: x` → `x`; mid-line and lowercase unchanged; 33-char inner paren fail-closed (char count, not byte `<= 34`). |
| **AC5** | **Met** | `display_text::strip_role_prefix` still leading-only (`USER:` / `ASSISTANT:` / `SYSTEM:`). T219 unit `strip_role_prefix__leading_case_sensitive__strips_and_leaves_mid_lower` still green. Pretty chrome is a separate `strip_pretty_chrome` in `preflight.rs`. |
| **AC6** | **Met** | Headers emitted as `h.trim()` (never through `display_pretty_line`). F31 strings are `format!` pushes. `format_preflight_pretty_body__long_header_and_notice__not_line_capped` locks a 160-char header and exact `+2 more safety entries — ai-brains memory list`. |
| **AC7** | **Met** | `format_preflight_pretty_body_with__governed_hash_headers__full_body_both_caps`: `#` / `##` preserved on standard **and** compact; 200-char `##` body stays full length (Other, no section parser). T219 governed unit still present. |
| **AC8** | **Met** | T219 `format_preflight_pretty_body__orphan_header__omitted` still holds; compact uses the same omit-if-no-content path. |
| **AC9** | **Met** | `display_text::truncate_preview_chars` units: em-dash and CJK over-max → char count == max, ends with `…`, no mid-char slice; `max_chars==0` → `""`. Pretty line-cap calls this helper only. |
| **AC10** | **Met** | `preflight_pretty__long_session_recent__line_capped_140`: `--pretty -m 3000`; seed pinned last; `T250SEEDLONG` present; non-index seed line ≤140 and ends with `…`; full seed absent; `Scope:` present; no display line starts with `ASSISTANT:`. |
| **AC11** | **Met** | `preflight_pretty__compact__tighter_caps_and_f31`: `--compact --pretty -m 3000` exit 0; fewer lines than standard; F31 `more safety entries` and/or `more via recall`. Item math is locked in the AC3 unit. |
| **AC12** | **Met** | `preflight_pretty__compact_json__uncapped_text_two_keys`: `--compact --format json -m 3000` exit 0; exactly `text` + `word_count`; `text` contains the **full** seeded body; no Scope chrome. |
| **AC13** | **Met** | `preflight_pretty__summary_compact__dual_model_unchanged`: `--summary --compact` prints T214 banner + `Pinned memories` + `In context`; no pretty Safety header. Summary returns before compact is consulted. |
| **AC14** | **Met** | CAPABILITIES pretty-density + `--compact` + ignore-compact rows + Safety residual; CHANGELOG Unreleased; OPERATIONS “Generating Preflight Context” `--pretty --compact` + Safety residual; Preflight `after_help` includes `ai-brains preflight --pretty --compact` and the Safety note. |
| **AC15** | **Met (product)** | No new crates in `ai-brains-cli` / workspace pins (clap still `4.5`, no pager/`comfy-table`). Presentation-only. Orchestrator: fmt on changed files; `clippy -p ai-brains-cli --all-targets -- -D warnings` PASS; `nextest -p ai-brains-cli preflight` 66/66 PASS. Full workspace gate is closeout (out of this review). |
| **AC16** | **Out of scope** | Manual live dogfood is closeout; not reviewed here. |

## Findings

None.

## Completeness sweep

### Frozen decisions F1–F15

| ID | Result |
|----|--------|
| **F1** | Standard keeps T219 counts. Session + Recent use `line_max=140` via `truncate_preview_chars`. Safety / Index / headers / F31 are not line-capped (`safety_cap` is `None` unless `first_line_only`). |
| **F2** | `#[arg(long)] compact: bool`. `human_mode && compact` → `PrettyCaps::compact()` 3/2/1/5/2 + `first_line_only` + `line_max=100`. Safety first-line-capped only when compact. F31 uses compact N. |
| **F3** | Summary returns before compact. Non-human path serializes raw `PreflightContextResponse`. `--compact` without `human_mode` is a no-op. No usage error. |
| **F4** | `format_preflight_pretty_body` wraps `PrettyCaps::standard()`. Recent 3 lifted to `PRETTY_RECENT_MAX`. `truncate_preview_chars` lives in `display_text`; `preview_line` and pretty both call it. No `truncate_pretty_line` in `preflight.rs`. `display_pretty_line` only composes chrome-then-optional-SOOT-truncate. |
| **F5** | `strip_pretty_chrome` pretty-only. Inner paren **char** count `<= 32`; fail-closed over. Does not change `strip_role_prefix` / `has_leading_role_prefix`. Turn counting still uses original retrieval lines. |
| **F6** | No `word_budget` / `truncate_turn` / `truncate_index_summary` / marker-selection edits in the T250 footprint. |
| **F7** | Human path still prints `format_scope_line` + blank line before the pretty body; compact does not drop Scope. |
| **F8** | `#` / `##` are not `---` headers (`is_legacy_section_header`). Other = chrome/role-strip only; no item/line caps; no governed parser. |
| **F9** | String ops + existing Scope SQL. No new crates. |
| **F10** | `--compact` is a bool (no case token). Unknown `--format` still follows pre-T249 preflight behavior (no `value_parser`). |
| **F11** | CAPABILITIES + CHANGELOG + OPERATIONS + additive Preflight `after_help`. Safety residual documented. `help_ia` Daily / Start-here labels untouched. |
| **F12** | Soft residuals not absorbed (is-terminal still used; no clap bump; no pager; no `--max-line`; no auto-compact). |
| **F13** | No T249/T248/T246 product rewrite in the T250 footprint. `OutputFormat::parse` not used for preflight. `PreflightContextResponse` still `{text, word_count}`. |
| **F14** | Pure formatter; chrome stripped, not rewritten. |
| **F15** | None of the high-finding anti-goals present (JSON text uncapped; leading-only SOOT intact; summary not TTY-switched; default item caps kept; `--format compact` is not density; no 16th doctor check; no DTO growth). |

### §14 / review pins

1. Session/Recent line-cap 140 on standard; Safety/Index/headers/F31 not capped on default — **held**.
2. `--compact` 3/2/1/5/2 + `first_line_only` + `line_max` 100; Safety first-line-capped only when compact — **held**.
3. JSON and `--summary` ignore `--compact` (T180 2-key uncapped text) — **held**.
4. `strip_pretty_chrome` pretty-only; inner paren char count ≤32; `strip_role_prefix` unchanged — **held**.
5. `truncate_preview_chars` in `display_text`; no third truncate helper in `preflight.rs` — **held**.
6. Governed Other / `#` / `##` uncapped; no governed section parser — **held**.
7. Hermetic `-m 3000` + seed-present; seed pinned last — **held**.
8. `after_help` includes `ai-brains preflight --pretty --compact` — **held**.
9. No new crates / no pin bumps / no T249–T248–T246 rewrite — **held** on reviewed files.

## Wiring

- **Branch / tree:** `feature/T250-preflight-pretty-density`. HEAD is `aa5c391` (branch point from `main`); T250 product edits are in the **working tree** (no T250 commit on the ref yet).
- **Pretty path:** `run()` → if `human_mode` and `options.compact` then `format_preflight_pretty_body_with(..., PrettyCaps::compact())` else `format_preflight_pretty_body` (`standard()`). Scope always prefixed.
- **Clap:** `Commands::Preflight { compact, ... }` → `PreflightRunOptions.compact`. Parse unit `preflight__compact_pretty__parses`. Not a `--format` token.
- **Truncate SOOT:** `display_text::truncate_preview_chars` (0-guard, keep `max-1` + `…`). Callers: `memory::preview_line`, `preflight::display_pretty_line`.
- **Chrome:** `strip_pretty_chrome` then optional line-cap. Index / Other / Safety-standard pass `line_cap=None`.
- **Docs:** `Docs/CAPABILITIES.md` density + compact + ignore-compact; `Docs/OPERATIONS.md` Generating Preflight Context; `CHANGELOG.md` Unreleased Added.
- **Tests:** units in `preflight.rs` (AC1–AC9) + `display_text.rs` (AC5/AC9); hermetics in `tests/preflight_pretty_readability.rs` (AC10–AC13). Orchestrator targeted gate 66/66.
- **Isolation:** T250 identifiers only in the scoped CLI/docs files (plus this track). `help_ia` Daily inventory unchanged. Workspace `clap = "4.5"` / `serde_json = "1.0"` / `is-terminal = "0.4"` unchanged.
