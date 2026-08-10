# T224 — Search/display role-prefix strip — Plan

**Status:** ✅ Completed 2026-08-10 — PR #120 `a18fae6`  
**Category:** UX  
**Depends:** T219 `display_text::strip_role_prefix` + T216 `memory::preview_line`  
**Ledger on go:** `ledgerful ledger start T224-search-role-prefix-strip --category UX --message "Strip role prefixes on recall/sync/forget human previews"`

## Goal

Remove leading `USER:` / `ASSISTANT:` / `SYSTEM:` from **human** search and forget-match previews by reusing the T219 helper. Do not mutate storage or JSON `content`.

## Absorbed deferred

- ASSISTANT: in search paths (deferred.md)  
- Display ASSISTANT: strip (harness residual row)  
- T219 soft residual: T224 search-path consumers  

**Not absorbed as DoD:** T228 Scope; retrieval preflight JSON strip; core promotion of helper; clap pin; T231; bridge Insight strip; contracts `preview` field; **ingest/pin dry-run strip** (M1 write-intent); **truncate_preview triplication SOOT** (M2 soft residual).

## AI fold-in pins (hard)

| ID | Pin |
|----|-----|
| **M1** | ingest/pin dry-run stay **raw** (write-intent honesty) |
| **M2** | Do not converge forget/ingest/pin `truncate_preview` this track |
| **M3** | Pretty: `strip_role_prefix(content.trim_start())` before 500 truncate |
| **M4** | Forget budgets: **100** dry-run/single/UUID; **80** multi-match |
| **M5** | Forget uses **`memory::preview_line`** only (no new `match_preview`) |
| **O2** | Pure unit first; hermetic optional for JSON only |
| **O4** | List consumers on `strip_role_prefix` rustdoc |

## Phased checklist

### Phase 0 — Preflight (on go)

- [x] `ledgerful doctor` / `ledgerful ledger status --compact`
- [x] `ledgerful scan --impact` (expect `recall.rs`, `forget.rs`, maybe `display_text.rs` doc, docs)
- [x] `ledgerful ledger start T224-search-role-prefix-strip --category UX …` (TX aefb8e8a-919e-4e72-840d-1bda5a761f9f)
- [x] `ai-brains preflight --summary`

### Phase 1 — Red: pretty strip units (AC1–AC3, AC1b, AC2b, AC9)

- [x] `format_pretty_hit_line__role_prefix_stripped__ac1` — `ASSISTANT: DECISION: …` → body without leading ASSISTANT after id
- [x] `format_pretty_hit_line__trim_start_before_strip__ac2b` — `"  ASSISTANT: body"`
- [x] mid-line + lowercase leave (AC2)
- [x] USER:/SYSTEM: strip
- [x] `format_pretty_hit_line__strip_before_truncate__ac1b` — long content; strip ordering
- [x] multibyte / optional CRLF (AC9)
- [x] Existing T218 score-kind units still pass (verify gate)

### Phase 2 — Green: `format_pretty_hit_line` (F3)

```rust
// Exact sketch (M3) — strip BEFORE 500-char truncate:
let content = crate::commands::display_text::strip_role_prefix(content.trim_start());
let content = if content.chars().count() > 500 {
    format!("{}...", content.chars().take(500).collect::<String>())
} else {
    content.to_string()
};
```

- [x] No second token list
- [x] Do not touch JSON map or MemoryPinned loop

### Phase 3 — Red/Green: forget via `preview_line` (F5, AC4–AC6)

- [x] Replace raw first-line + local `truncate_preview` at human preview sites with:
  - dry-run match / single Found / UUID: `preview_line(&hit.content, 100)`
  - multi-match list: `preview_line(&hit.content, 80)`
- [x] Remove unused local `truncate_preview` from `forget.rs` **if** no longer referenced (leave ingest/pin alone)
- [x] Units or path tests: strip + max 80/100; `…` on over-budget (intentional L1) — `forget_match_preview` / `forget_multi_preview` pure units + call-site wiring
- [x] **Do not** edit ingest.rs / pin.rs dry-run (F5b)

### Phase 4 — JSON / events freeze (AC7–AC8)

- [x] Confirm no strip on `RecallResult { content: h.content.clone() }`
- [x] Confirm no strip on `MemoryPinnedPayload { content: hit.content.clone() }`
- [x] Confirm bridge Insight path untouched
- [x] Optional one-line comments at append/JSON sites (F6/F7)

### Phase 5 — Docs + helper rustdoc (AC11, AC13)

- [x] `display_text::strip_role_prefix` rustdoc consumer list (O4)
- [x] `Docs/CAPABILITIES.md` — human recall/sync/forget strip; JSON/bridge raw; ingest/pin dry-run raw (write intent)
- [x] `CHANGELOG.md` — T224 entry
- [ ] Soft: WORKFLOWS forget dry-run example

### Phase 6 — Manual + gate (AC12)

- [x] Manual: pretty recall + forget dry-run clean; JSON still has prefix when stored
- [x] Optional: pin/ingest dry-run stay raw by design (code freeze F5b; no dry-run edit)
- [x] `cargo nextest run -p ai-brains-cli` (bin package; no `--lib`) — **761+** passed
- [x] `cargo clippy -p ai-brains-cli --all-targets -- -D warnings` — clean
- [x] `cargo fmt --check` — clean
- [x] Full gate: fmt / clippy workspace / nextest **2487** / deny / audit — **FULL_GATE_OK**
- [x] `ledgerful verify` per practice (pre-push / ship)
- [x] `review.md` (internal CLEAN + Codex R2 PASS WITH DEFERRED P3); conductor Completed + deferred closed
- [x] ledger commit + pin + PR #120 squash-merged `a18fae6`

### Codex R1 fix follow-up

- [x] P2 forget AC4–AC6 pure units via `forget_match_preview` / `forget_multi_preview`
- [x] P2 AC10 SOOT: `ROLE_PREFIXES` + `has_leading_role_prefix`; preflight turn-start uses helper
## File touch map

| File | Change |
|------|--------|
| `crates/ai-brains-cli/src/commands/recall.rs` | F3 strip+trim in `format_pretty_hit_line`; pure units |
| `crates/ai-brains-cli/src/commands/forget.rs` | Wire `forget_*_preview` → `preview_line` at human sites; pure units AC4–AC6 |
| `crates/ai-brains-cli/src/commands/display_text.rs` | O4 rustdoc; `ROLE_PREFIXES` SOOT; `has_leading_role_prefix` |
| `crates/ai-brains-cli/src/commands/preflight.rs` | Turn-start detection via `has_leading_role_prefix` (AC10) |
| `crates/ai-brains-cli/src/commands/memory.rs` | **No change** unless `preview_line` already `pub(crate)` (it is) |
| `ingest.rs` / `pin.rs` | **No change** (M1/F5b) |
| `Docs/CAPABILITIES.md`, `CHANGELOG.md` | Honesty |
| `conductor/*` | Status / deferred closeout |

## Non-goals (reminder)

Vault rewrite · JSON content strip · MemoryPinned strip · ingest/pin dry-run strip · truncate triplication SOOT · retrieval classify merge · T228 · clap bump

## Success

Human search/match previews clean; JSON/storage/write-intent dry-runs raw; one strip helper; gate green.
