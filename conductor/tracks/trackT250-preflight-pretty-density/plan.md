# T250 Plan — Preflight pretty density (pass-2)

**Status:** ✅ **Completed** (2026-08-14 PR #165 `bf23f0e`)  
**Spec:** [spec.md](./spec.md) F1–F16 / AC1–AC16 + §14 AI fold-in  
**Category:** UX / FEATURE  
**Ledger TX (on go):** `ledgerful ledger start T250-preflight-pretty-density --category UX --message "Pretty line-cap on session/recent; --compact tighter caps; JSON/summary ignore compact"`

---

## AI fold-in (2026-08-14) — `C:\dev\AI-review.md` AI1 + AI2

No Highs. AI1 M1/L1–L4 are must-pin. AI2 remapped ACs declined (keep AC1–AC16).

### Disposition

| ID | Source | Verdict | Action |
|----|--------|---------|--------|
| **AI1 M1** | AI1 | **Agree hard** | F8 + AC7 governed chrome-only |
| **AI1 L1** | AI1 | **Agree hard** | Phase 1 extract `truncate_preview_chars` → `display_text` |
| **AI1 L2** | AI1 | **Agree hard** | Phase 4 hermetic `-m 3000` + seed present |
| **AI1 L3** | AI1 | **Agree hard** | Phase 2 `(999 mo ago)` + 33-char fail-closed |
| **AI1 L4** | AI1 | **Agree hard** | Phase 3 keep Recent recall hint |
| **AI1 L5/L6 + Safety note** | AI1 | **Agree** | Phase 5 OPERATIONS + after_help + Safety residual |
| **AI2 remapped ACs / `<= 34`** | AI2 | **Decline** | Keep AC1–AC16; inner char ≤32 |

### Pins locked by fold-in

1. Governed Other uncapped (no section parser).
2. Single truncate helper in `display_text`.
3. Hermetic `-m 3000` + seed-present.
4. Chrome 32-char inner bound + longest-timestamp unit.
5. Compact Recent keeps `(Use 'recall'…)`.

---

## Preflight (plan time — 2026-08-14)

| Check | Result |
|-------|--------|
| `preflight --summary` | Scope `test-alias`; 544 pinned; 2 sessions; in-context 1/43/6; 1076 words; harnesses grok/agy/opencode ok |
| `preflight --pretty -m 1500` | 66 lines / 840 words / 5 headers; 11 lines >160; longest **779** (session DECISION); Recent `(just now) ASSISTANT:`; index already ~64 chars; `+19 more via recall` |
| `preflight --compact` | clap unexpected argument; **exit 2** |
| `format_preflight_pretty_body` | Exists; T219 caps 8/6/3/15; Recent 3 local; no line-cap; no PrettyCaps |
| `strip_role_prefix` | Leading-only; timestamp-prefixed Recent chrome survives |
| T180 JSON | `{text, word_count}` compact; hermetic `preflight_pretty__json_format__two_keys_and_newlines_in_text` |
| clap / serde_json / is-terminal / chrono | lock 4.6.1 / 1.0.150 / 0.4.17 / 0.4.44 — **no bumps** (crates.io clap 4.6.6, serde_json 1.0.151, chrono 0.4.45) |
| rustc | 1.95.0 |
| Ledger | 0 pending, 0 unaudited drift |
| T243 / T245 / T246 / T247 / T248 / T249 | Completed — no rewrite |
| Embedding / completion :8083/:8081 | Unreachable — capture-independent; not blocking |
| Recall | T246/T248/T249 pins; no prior T250 pin |

---

## Absorbed deferred

| Item | Source | Plan action |
|------|--------|-------------|
| Preflight pretty density | deferred.md / audit 7/7 | **DoD** F1–F7 |
| T219 `--compact` / PrettyOpts | T219 F11/F30 | **F2/F4** |
| Placeholder F1 lower default caps | spec draft | **Decline** silent count change; **F1** line-cap instead |
| Placeholder F2 Scope always-on | spec draft | **F7** already T219 |
| Placeholder F3 JSON envelope | spec draft | **F3** |
| Retrieval JSON role strip | T219 F22 | **Not absorbed** |
| is-terminal → std | T214/T219/T249 | **Not absorbed** |
| T241 `--install-grants` / T255 / T251 | peers | **Not absorbed** |

---

## Phase 0 — Ledger + impact (on go)

- [x] `ledgerful ledger status --compact` — expect 0 pending, 0 unaudited drift
- [x] `ledgerful ledger start T250-preflight-pretty-density --category UX`
- [x] `ledgerful scan --impact`
- [x] Confirm no other agent is editing `preflight.rs` / `display_text.rs` / `main.rs` Preflight clap / `preflight_pretty_readability.rs`

---

## Phase 1 — Red → Green: PrettyCaps + line-cap (F1 / F4 / AC1–AC2 / AC6 / AC9)

- [x] Introduce `PrettyCaps` + `format_preflight_pretty_body_with`
- [x] Keep `format_preflight_pretty_body(text)` as `standard()` wrapper (T219 units compile)
- [x] Lift Recent `3` to `PRETTY_RECENT_MAX`
- [x] Apply `line_max` only to Session + Recent on `standard()` (`PRETTY_LINE_MAX = 140`)
- [x] Promote `memory::truncate_preview_chars` → `pub(crate) display_text::truncate_preview_chars`; `preview_line` + pretty line-cap both call it (AI1 L1 — **no** `truncate_pretty_line` in `preflight.rs`)
- [x] Never truncate headers or F31 notices
- [x] Units AC1 / AC2 / AC6 / AC9
- [x] Do **not** edit `word_budget.rs` / `truncate_turn` / `truncate_index_summary`

---

## Phase 2 — Pretty-only chrome strip (F5 / AC4–AC5)

- [x] `strip_pretty_chrome` in `preflight.rs` (do **not** change `strip_role_prefix`)
- [x] Inner paren **char count** ≤32 (not AI2 byte `<= 34`); fail-closed if over
- [x] Wire display lines through chrome strip **then** line-cap
- [x] Turn counting still uses `has_leading_role_prefix` on retrieval-emitted lines
- [x] Units AC4 / AC5 including `(999 mo ago)` and 33-char non-strip (AI1 L3)

---

## Phase 3 — Compact constructor (F2 / AC3 / AC7–AC8)

- [x] `PrettyCaps::compact()` 3/2/1/5/2 + `first_line_only` + line_max 100
- [x] Safety/Recent blocks: first line only when compact
- [x] Keep trailing `(Use 'recall'…)` hint (AI1 L4)
- [x] F31 notices use compact N
- [x] Units AC3 / AC7 / AC8 — AC7 includes 200-char governed body line **uncapped** (AI1 M1)

---

## Phase 4 — Clap + hermetics (F2 / F3 / F10 / AC10–AC13)

- [x] `--compact` on `Preflight` + `PreflightRunOptions.compact` + `main.rs` dispatch
- [x] `human_mode && compact` → `PrettyCaps::compact()`; else `standard()`
- [x] JSON / `--summary` ignore compact
- [x] after_help example `ai-brains preflight --pretty --compact`
- [x] Hermetic AC10–AC13 (extend `preflight_pretty_readability.rs`; **`-m 3000`** + assert seeded long line **present** — AI1 L2)
- [x] T219 AC3–AC8 still green
- [x] No TempEnv mandate (units pure; hermetics use `cmd.env` like T249 L4)

---

## Phase 5 — Docs (F11 / AC14)

- [x] CAPABILITIES pretty-density + `--compact` (JSON/summary ignore)
- [x] CHANGELOG Unreleased
- [x] OPERATIONS “Generating Preflight Context” `--compact` + Safety residual (AI1 L5)
- [x] New `after_help` on Preflight (AI1 L6 additive)
- [x] Do **not** rewrite T204 Start-here / Daily group labels

---

## Phase 6 — Gate + closeout (AC15–AC16)

- [x] Targeted nextest + clippy `-p ai-brains-cli`
- [x] Manual AC16 on this machine
- [x] Full gate
- [x] `review.md`; conductor → Completed only after PR
- [x] Soft leftovers → deferred.md F12
- [x] Pin DECISION

---

## Isolation (do not)

- Rewrite T249 scope/daemon/doctor, T248 retention, T246 graph
- Change `OutputFormat::parse`
- Grow `PreflightContextResponse`
- Strip JSON `text` roles
- Migrate is-terminal
- Bump clap / serde_json
- Start/stop/install the live daemon
- Print or commit `AI_BRAINS_KEY`
