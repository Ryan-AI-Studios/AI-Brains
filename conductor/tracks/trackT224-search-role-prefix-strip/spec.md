# T224 — Search/display role-prefix strip

- **Track ID:** T224-SearchRolePrefixStrip
- **Phase:** T217–T232 post-audit CLI quality (P2)
- **Status:** ✅ **Completed** 2026-08-10 — PR #120 squash-merged `a18fae6`
- **Depends on:** T216 `preview_line` SOOT; **T219** `commands/display_text.rs` `strip_role_prefix` (landed)
- **Blocks / feeds:** Operator trust that search/match previews match `memory list` cleanliness; soft residual for T231 unified search UX
- **Category:** UX / BUGFIX (display-only)
- **Source:** Non-destructive CLI audit 2026-08-05 — `ASSISTANT:` in recall/sync/forget dry-run vs memory-list strip
- **Deferred absorbed:** deferred.md “ASSISTANT: in search paths”; “Display ASSISTANT: strip”; T219 soft residual “T224 consumers / search-path role strip”
- **Not absorbed:** T228 non-empty recall Scope; retrieval-side strip for preflight JSON `text`; move helper to core/retrieval; clap 5 / workspace clap pin bump; T231 unified search defaults; MSI; vault mutation / event rewrite; contracts DTO growth; **ingest/pin dry-run previews** (M1 — keep raw write-intent honesty); **truncate_preview triplication SOOT** across ingest/pin/forget (M2 — soft residual)
- **Research date:** 2026-08-10 (live dogfood + T219 SOOT + clap 4.6.6 pin check)
- **AI fold-in:** 2026-08-10 — AI1 affirms F1–F12/AC1–AC12 (no new criticals). AI2 **M1–M5 hard**; **L1–L3/L5** elevated or noted; **L4** affirm; **O1/O2/O4** folded. Disposition **§11**.
- **Ledger:** plan-only until go (`ledgerful ledger start T224-search-role-prefix-strip --category UX`)

## 1. Objective

1. **Display parity:** Human-facing search/match previews must strip leading capture role tokens (`USER:` / `ASSISTANT:` / `SYSTEM:`) the same way `memory list` and preflight pretty already do.  
2. **Single SOOT helper:** Reuse T219 `strip_role_prefix` — **no second copy** of the token list.  
3. **Storage integrity:** Vault rows, FTS bodies, `MemoryPinned` event payloads, and machine JSON `content` remain **raw** (role prefix preserved).  
4. **Capture independence:** Pure string ops in CLI display paths only — no models, embeddings, or graph.  
5. **Zero new crates.**

## 2. Live baseline (re-scan 2026-08-10)

### 2.1 Confirmed dumps (dogfood)

| Surface | Live behavior |
|---------|----------------|
| `ai-brains recall "DECISION" --format pretty` | Hit lines show `…: ASSISTANT: DECISION: …` |
| `ai-brains recall …` default JSON | `"content":"ASSISTANT: …"` (expected machine raw) |
| `ai-brains forget --match "DECISION" --dry-run` | Previews `— ASSISTANT: DECISION: …` |
| `memory list` preview | Already strips (T216 + T219 converge) |
| Preflight pretty index/session | Already strips (T219) |

### 2.2 Call sites (code truth)

| Path | File | Today | T224 action |
|------|------|-------|-------------|
| Pretty hit line (recall + `sync query`) | `recall.rs` `format_pretty_hit_line` | Prints `h.content` raw (500-char truncate) | **Strip before truncate** (F3: `trim_start` then strip) |
| `print_pretty_hits` | `recall.rs` | Passes raw content | Inherits via formatter |
| Forget dry-run match list | `forget.rs` `truncate_preview(first_line)` | Raw first line | **`memory::preview_line(content, 100)`** |
| Forget single-match confirm `Found:` | `forget.rs` ~L122 | Raw first line | **`preview_line(..., 100)`** |
| Forget multi-match list | `forget.rs` ~L145–154 | Raw first **80** chars, **no** ellipsis | **`preview_line(..., 80)`** — strip + `…` on cut (M4/M5; small ellipsis behavior change) |
| Forget UUID dry-run / confirm preview | `forget.rs` `truncate_preview` | Raw | **`preview_line(..., 100)`** |
| `forget --list-forgotten` | via `memory` inventory | Already stripped | No change |
| JSON `RecallResult.content` | `recall.rs` response map | Raw | **Keep raw** (document) |
| Bridge Insight `content` (`sync` JSONL) | `sync.rs` | Raw vault text | **Keep raw** (export fidelity) |
| `MemoryPinned` append | `recall.rs` | Raw `hit.content` | **Must not strip** |
| Retrieval `classify_pin_kind` / preflight safety | `ai-brains-retrieval` | Own `ASSISTANT: ` strip | **Out of scope** (crate boundary; different SOOT) |
| **ingest `--dry-run` preview** | `ingest.rs` `truncate_preview(&req.content)` | Raw input | **Keep raw** (M1) — pre-storage write-intent honesty |
| **pin `--dry-run` preview** | `pin.rs` `truncate_preview(&content)` | Raw input | **Keep raw** (M1) — shows exactly what will be written |

### 2.3 Helper already landed (T219)

```rust
// crates/ai-brains-cli/src/commands/display_text.rs
pub(crate) fn strip_role_prefix(line: &str) -> &str
// case-sensitive leading USER:|ASSISTANT:|SYSTEM: only; mid-line + lowercase unchanged; borrow, no alloc
```

**Consumers after T224 (O4 — list in helper doc):** `memory::preview_line`, `content_has_tag`, preflight pretty, `format_pretty_hit_line`, forget human previews (via `preview_line`).

## 3. Frozen decisions

| ID | Decision |
|----|----------|
| **F1 — Display only** | Never rewrite stored content, projections, FTS, or append-only events. Strip only when formatting human stdout. |
| **F2 — Helper reuse (hard)** | Call `crate::commands::display_text::strip_role_prefix` only. **Forbid** a second prefix array in `recall.rs` / `forget.rs`. |
| **F3 — Pretty SOOT + trim (M3 hard)** | Inside `format_pretty_hit_line`, **before** 500-char truncation: `let content = display_text::strip_role_prefix(content.trim_start());` then existing `chars().take(500)`. Covers recall + `sync query` (T211 F37). **Must not** strip after truncate (L3 — role token could straddle cut). |
| **F4 — Strip unit** | Leading prefix only on the display body (pretty: whole content after `trim_start`; forget: first non-empty line via `preview_line`). Multibyte-safe via `chars()`. Do **not** strip mid-body or every line. CRLF: `trim_start` / `.lines()` handle leading `\r`/`\n` (L5 unit lock). |
| **F5 — Forget previews via `preview_line` (M5 hard)** | **Reuse** `memory::preview_line(content, max_chars)` for all forget human previews — **no** new `match_preview` and **no** second first-line loop. Budgets (M4 hard): **100** dry-run match / single `Found:` / UUID dry-run+confirm; **80** multi-match list. Truncation marker becomes **`…`** (Unicode ellipsis, T216 SOOT) instead of forget’s local `...` — intentional (L1). Multi-match gains ellipsis-on-cut where it previously had none — cosmetic, accepted. |
| **F5b — ingest/pin dry-run (M1 hard)** | **Explicitly out of DoD.** `ingest --dry-run` and `pin --dry-run` previews show **raw write intent** (exactly what will be stored, including role chrome the user/agent is about to commit). Stripping would misrepresent the write. Document in §5 / CAPABILITIES if needed. |
| **F6 — JSON / bridge keep raw (hard)** | `RecallResult.content`, default JSON recall, and bridge Insight payload **keep** leading role tokens. Rationale: machine fidelity, re-display elsewhere, classification/heuristic consumers, no contract shape change. Document in CAPABILITIES. Soft residual: optional future `preview` field or `--strip-roles` — **not** DoD. |
| **F7 — MemoryPinned raw (hard)** | Event `content` remains vault text with role prefix. Stripping here would corrupt graph/session provenance. AC8: no strip call in append loop. |
| **F8 — Case + token SOOT** | Identical to T216/T219: case-sensitive `USER:` / `ASSISTANT:` / `SYSTEM:`; leave `assistant:` and mid-line tokens. |
| **F9 — Capture independence** | CLI string ops only. No retrieval ranking change. |
| **F10 — Zero new crates / no clap bump** | clap workspace `4.5` (lock may resolve 4.6.x e.g. 4.6.1); crates.io **4.6.6** (2026-08-06) — soft residual only, not this track. |
| **F11 — Exit codes** | Unchanged (0 success). |
| **F12 — Contracts** | No DTO change. Document only. |
| **F13 — High findings if…** | Dual strip SOOT; stripping JSON content without docs; mutating vault/events; mid-line strip; forget multi-match still dumps `ASSISTANT:`; strip **after** 500 truncate; stripping ingest/pin dry-run “by accident”; forgetting `trim_start` so `"  ASSISTANT:"` leaks (M3). |
| **F14 — Soft residuals** | JSON/bridge strip or `preview` field; promote `strip_role_prefix` to core for retrieval converge; clap pin → 4.6; T228 Scope; T231 search unify; is-terminal migrate; **converge `truncate_preview` triplication** (forget/ingest/pin identical 100+`...`) onto shared `display_text` helper — **not** T224 DoD (M2/O1); optional hermetic e2e for pretty strip. |
| **F15 — Parallel-friendly** | Touches `recall.rs` formatter + `forget.rs` preview call sites only; low conflict with T222/T223 (L4 verified). Coordinate if T228 also edits non-empty pretty. |
| **F16 — Series order** | Next after T219 close. Peers: graph/install (T222/T232), env quiet (T223). |
| **F17 — Plan-only** | No production code until user **go**. |
| **F18 — Ledger** | On go: `ledgerful ledger start T224-search-role-prefix-strip --category UX`. |
| **F19 — Review** | UX primary review required. Cross-model soft (small display track). |
| **F20 — Implement order (O2 hard)** | Red **pure unit** `format_pretty_hit_line` strip (AC1–AC3, L3 long-prefix, L5 CRLF, AC9 multibyte) → Green F3 → Red/Green forget sites → `preview_line` (AC4–AC6) → AC7/AC8 code freeze (JSON path untouched; no hermetic required unless easy) → O4 helper doc consumers → docs → gate. Prefer pure unit over hermetic (L2: no prior regression guard — Red-first is correct). |
| **F21 — Determinism** | Pure formatters; no timestamps. |
| **F22 — Docs** | CAPABILITIES: human recall/sync/forget previews strip; JSON/bridge/ingest-pin dry-run keep raw; CHANGELOG; optional WORKFLOWS. Skill optional soft. |
| **F23 — Helper doc consumers (O4)** | Update `strip_role_prefix` rustdoc to list consumers after T224 (prevents accidental dual SOOT). |

## 4. Acceptance criteria

| AC | Criterion | Proof |
|----|-----------|-------|
| **AC1** | Pretty hit with leading `ASSISTANT: DECISION: …` does **not** show `ASSISTANT:` in the content body after id/badge | Pure unit on `format_pretty_hit_line` |
| **AC1b** | Strip **before** 500 truncate: content whose role prefix sits near a long body still strips (ordering lock L3) | Unit |
| **AC2** | Leading `USER:` / `SYSTEM:` stripped; mid-line `text ASSISTANT: x` and lowercase `assistant:` unchanged | Unit |
| **AC2b** | Leading whitespace before role token still strips (`"  ASSISTANT: body"` → body) via `trim_start` (M3) | Unit |
| **AC3** | `sync query` pretty inherits strip (same `format_pretty_hit_line`) | Unit SOOT + optional manual dogfood |
| **AC4** | Forget dry-run match preview has no leading role token | Unit: `preview_line` / wired path |
| **AC5** | Forget single + multi match human lists strip; multi uses **max 80** + `…` when truncated (M4) | Unit |
| **AC6** | Forget UUID dry-run + confirm preview strip (max **100**) | Unit |
| **AC7** | JSON recall `content` still **includes** `ASSISTANT:` when vault has it (raw freeze) | Path untouched + review; hermetic optional |
| **AC8** | `MemoryPinned` uses raw `hit.content` (no strip at append site) | Code review / no strip in loop |
| **AC9** | Multibyte + optional CRLF after strip truncates without panic (L5) | Unit |
| **AC10** | No second prefix token list outside `display_text.rs` | Review / grep |
| **AC11** | CAPABILITIES + CHANGELOG updated (incl. ingest/pin dry-run raw honesty if mentioned) | Docs gate |
| **AC12** | Full CI gate green | fmt/clippy/nextest/deny/audit |
| **AC13** | `strip_role_prefix` rustdoc lists consumers (O4) | Doc review |

## 5. Out of scope

- Changing retrieval ranking, FTS MATCH, or rescue ladder  
- Preflight JSON `text` role strip inside retrieval (T219 soft residual)  
- Non-empty recall Scope header (T228)  
- Unified search product defaults (T231)  
- Stripping bridge export JSONL Insight content  
- **`ingest --dry-run` / `pin --dry-run` role strip** (M1 — write-intent honesty)  
- **Converging `truncate_preview` triplication** forget/ingest/pin → shared helper (M2 soft residual)  
- Tag schema / auto-forget / CE wipe  
- clap 5 / MSI / App Store  

## 6. Dependency / ecosystem research (2026-08-10)

| Item | Finding | Action |
|------|---------|--------|
| **strip primitive** | `str::strip_prefix` + suffix `trim_start` (already T219) | Keep; no new crate |
| **clap** | crates.io **4.6.6** (docs.rs 2026-08-06); workspace pin `4.5` | Soft residual only |
| **CLI display practice** | Display-layer sanitization; keep wire/storage raw (sunshowers CLI recs: separate presentation) | Affirms F1/F6 |
| **Zero new deps** | — | Hard |

## 7. Deferred roll-in

| Source | Item | Disposition |
|--------|------|-------------|
| deferred.md search paths | ASSISTANT: in recall/sync/forget | **This track DoD** |
| deferred.md harness residual | Display ASSISTANT: strip | **This track** (orthogonal display) |
| T219 soft F22 / closeout | T224 consumers | **This track** |
| T219 soft | retrieval JSON role strip | **Not** this track (preflight assembly) |
| T228 | Non-empty recall Scope | Separate |

After plan freeze: mark deferred rows as **Planning → T224** (close on ship).

## 8. Risk

| Risk | Mitigation |
|------|------------|
| Agents parse pretty lines with `ASSISTANT:` literal | Breaking for scrapers of pretty only; JSON unchanged; document |
| Dual first-line logic in forget | **Reuse `preview_line`** (M5) — no local match_preview |
| Strip after 500 truncate | F3/AC1b: strip **first** |
| Leading whitespace leaks role | F3/AC2b: `trim_start` before strip |
| Accidental event strip | AC8 + code review of MemoryPinned loop |
| Accidental ingest/pin strip | F5b / §5 explicit exclude |
| Multi-match ellipsis behavior change | Accepted cosmetic (M4); document |

## 9. Manual test plan (on go)

```powershell
ai-brains recall "DECISION" --format pretty --limit 3
# expect: no "ASSISTANT:" in content body after memory_id:

ai-brains forget --match "DECISION" --dry-run
# expect: previews without leading ASSISTANT:

ai-brains recall "DECISION" --limit 1
# JSON content still has ASSISTANT: when stored that way

# Optional honesty check (must still show role if user typed it):
# ai-brains pin --dry-run "ASSISTANT: DECISION: x"   # raw intentional
```

## 10. Success definition

Pretty recall/sync and all forget human previews match `memory list` role-prefix cleanliness; vault and JSON remain raw; single strip helper SOOT; ingest/pin dry-run remain write-intent raw; gate green; deferred search-path strip closed.

## 11. AI fold-in disposition (2026-08-10)

| ID | Source | Disposition |
|----|--------|-------------|
| **AI1** overall | Affirms architecture, AC table, call-site plan | **Accept** — no new criticals; hermetic-for-everything **softened** by O2 (pure unit preferred) |
| **M1** ingest/pin dry-run missing | Medium | **Hard (b):** exclude with rationale — write-intent honesty (F5b, §2.2, §5) |
| **M2** truncate_preview ×3 | Medium | **Hard (a):** acknowledge + **soft residual** F14 (not DoD) |
| **M3** trim_start before strip | Medium | **Hard fold** F3/F4/AC2b — correctness pin |
| **M4** multi 80 / ellipsis policy | Medium | **Hard fold** F5: max 100 vs 80 per site; multi gains `…` via `preview_line` |
| **M5** match_preview vs preview_line | Medium | **Hard (a):** reuse `memory::preview_line` only |
| **L1** `…` vs `...` | Low | **Document** intentional convergence on `…` |
| **L2** no existing regression guard | Low | **Affirm** F20 Red-first |
| **L3** strip-before-truncate | Low | **Elevate** AC1b unit lock |
| **L4** parallel claim | Low | **Affirm** F15 |
| **L5** CRLF | Low | **Unit lock** AC9 |
| **O1** triplication residual | Opp | **Fold** F14 |
| **O2** pure unit over hermetic | Opp | **Hard** F20 |
| **O3** clig +N | Opp | **N/A** — no action |
| **O4** consumer list in doc | Opp | **Hard** F23/AC13 |

**Rejected / not absorbed:** treating ingest/pin dry-run as DoD strip targets; promoting full truncate SOOT into this track; requiring hermetic for AC1–AC6; inventing a second `match_preview` when `preview_line` already exists.
