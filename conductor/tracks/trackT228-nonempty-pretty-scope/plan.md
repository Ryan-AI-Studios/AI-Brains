# T228 — Non-empty pretty Scope — Plan

**Status:** ✅ **Completed** (PR #134 `e51d5e4`, 2026-08-11)  
**Category:** UX / FEATURE (light)  
**Depends:** T207 · T214 · T219 · T211 (shared `print_pretty_hits`)

## Goal

Print T207/T214 `Scope:` on **non-empty** pretty `recall` (and **sync query** vault section), Scope-first before Session/hits, via shared `resolve_active_scope_line` SOOT. JSON frozen. Close AC10 residual.

## Absorbed deferred

| Item | Disposition |
|------|-------------|
| deferred.md Non-empty pretty Scope (T207 soft) | **DoD** |
| T207 AC10 / M3 | **Elevate to hard** |
| CAPABILITIES deferred sentence | **Close** |
| Series README T228 | **DoD** |
| T219 / T224 soft “T228” callouts | **Absorb** |

**Not absorbed:** JSON `scope`; auto-global; T230 labels; T231 merge; OutputFormat F34; omit generated Session on non-empty; **sync random-UUID fallback fix (DoD)**; **sync TTY default alignment**.

## Research pins (2026-08-11)

| Pin | Evidence |
|-----|----------|
| Live non-empty pretty | Session + hits only — **no Scope** |
| Live empty pretty | `Scope: project=test-alias (…)` OK |
| Code | non-empty branch “AC10 deferred”; sync non-empty → `print_pretty_hits` only |
| SOOT | `format_scope_line` shared with preflight + memory list |
| Non-empty Session always | `effective_session_id` always Some → always print Session |
| Empty blank | Scope then Session/hint with **no** blank (F26) |
| Sync fallback | missing project → random UUID (pre-existing; F32 residual) |
| Sync default | always pretty (F34 residual) |
| Affected tests | isolation:86, ranking×4, smoke:86, smoke:459 — safe substring; need AC8 lock |
| Deps | clap 4.6.1 / is-terminal 0.4.17 — **no bump** |

## AI fold-in pins (hard)

| ID | Pin |
|----|-----|
| **AI1 M1 / F29** | Extract `resolve_active_scope_line` — empty/non-empty recall + empty/non-empty sync |
| **AI1 M2 / F4** | Strict order Scope → Session → Embedding? → hits |
| **AI1 M3 / F26** | No blank between Scope and Session |
| **AI1 M4 / AC10** | Hermetic asserts Scope on non-empty |
| **AI1 L1 / F30** | Global skips `get_project_by_id` |
| **AI1 L2 / F19–F20** | CAPABILITIES + **new** CHANGELOG only |
| **AI2 M1 / F31/AC8** | Enumerate verify-green tests; lock `Scope: global` on isolation global |
| **AI2 M2 / F32** | Document sync random-UUID residual; soft fix only with proof |
| **AI2 M3 / F33/AC11** | lines[0]=Scope, lines[1]=Session on non-empty |
| **AI2 L1 / F19** | Do **not** edit T207 CHANGELOG historical row |
| **AI2 O2/O3/O5** | Hard AC8/AC2/AC6 named tests |
| **AI2 F12 ledger-first** | Scope only inside vault block after ledger section |

**Soft:** F34 sync TTY default residual; O4 fallback fix; L6 smoke Scope; O6 cross-model focus.

See `spec.md` §15 full disposition.

## Frozen decision index

See `spec.md` §3 **F1–F36**. Hard summary:

1. Always Scope on pretty empty+non-empty (F2).  
2. Shared resolver + global short-circuit (F29/F30).  
3. Order Scope → Session → Embedding? → hits; no blank (F4/F26/F33).  
4. Session omit-generated stays empty-only (F5).  
5. JSON frozen (F7).  
6. Quiet keeps Scope (F9).  
7. Sync vault Scope + AC8 + verify-green list (F12/F31).  
8. New CHANGELOG only — never rewrite T207 row (F19).  
9. Sync random UUID + TTY default = residuals (F32/F34).  

## Task checklist

### 0. Preflight (on go)

- [x] `ledgerful doctor` + `ledgerful ledger status --compact`
- [x] `ledgerful ledger start T228-nonempty-pretty-scope --category FEATURE --message "non-empty pretty Scope on recall/sync"`
- [x] `ledgerful scan --impact` (include `recall.rs`, `sync.rs`, listed tests)
- [x] Confirm clean ledger / tree (or reconcile first)

### 1. Red — AC10 / AC11 order (AI1 M4, AI2 M3)

- [x] Update/rename non-empty hermetic → `recall_nonempty__pretty__prints_scope_before_hits`
- [x] Assert `Scope:` present + hit content + no `No results`
- [x] Assert non-empty lines: `[0].starts_with("Scope:")` && `[1].starts_with("Session:")` (AC11)
- [x] Confirm **fails** on current main (red)

### 2. Green — resolver + non-empty recall (AI1 M1–M3, L1)

- [x] Implement `pub(crate) resolve_active_scope_line(conn, global, project_id)` (F29/F30)
- [x] Wire empty recall + non-empty recall through resolver
- [x] Non-empty: print Scope first, then Session, Embedding?, hits — **no blank** (F26)
- [x] Remove “AC10 deferred” comment; point to T228
- [x] Unit: global → `Scope: global` without project row (AC13)

### 3. Red/green — AC2 global + AC6 quiet (AI2 O3/O5)

- [x] **Hard:** `recall_nonempty__pretty_global__scope_global` → `Scope: global`
- [x] **Hard:** `recall_nonempty__pretty_quiet__keeps_scope`
- [x] Regression: full `recall_empty_pretty_scope` suite green
- [x] Soft: project+alias AC3 if hermetic vault has alias

### 4. Green — sync non-empty Scope + F31 sweep (AI2 M1/O2)

- [x] After `--- AI-Brains Recall ---`, non-empty prints Scope via resolver then hits
- [x] Refactor `print_pretty_empty_sync` to use resolver
- [x] **AC8 hard:** assert `Scope: global` on `sync_query_pretty_global_flag_returns_cross_project_results`
- [x] **Verify still green (enumerate):**
  - [x] `sync_query_isolation.rs` empty-path isolation (`…default_scoped…`)
  - [x] `sync_query_ranking.rs` (all 4)
  - [x] `smoke.rs` `sync_query__no_bridge__skips_ledgerful_section`
  - [x] `smoke.rs` `recall_pretty__shows_session_prefix` (session= bracket substring)
- [x] Ledger-first path (if exercised): Scope only under vault header
- [x] **F32:** do **not** silently fix random-UUID fallback unless proven safe; else note residual in CHANGELOG/review

### 5. Docs (AI1 L2, AI2 L1)

- [x] CAPABILITIES: always-on Scope; remove “deferred (T228)”; sync vault parity
- [x] CHANGELOG: **new** T228 minor UX entry only — **do not edit** T207 historical row
- [x] Soft: OPERATIONS / skill if Session-only samples
- [x] Series README + deferred.md strike on complete

### 6. Gate + closeout (on go)

- [x] `cargo fmt --check`
- [x] `cargo clippy --workspace --all-targets -- -D warnings`
- [x] `cargo nextest run --workspace` (or `-p ai-brains-cli` + listed packages during work)
- [x] `cargo deny check` + `cargo audit`
- [x] Manual dogfood (spec §9)
- [x] Primary review → fix → verified_fixed
- [x] Soft cross-model (F23/O6): order regressions + sync sweep + smoke:459
- [x] `ledgerful verify --scope full` + ledger commit
- [x] Conductor → Completed; pin DECISION
- [x] Document F29 helper + F32 residual status in `review.md`

## Out of scope reminders

- Do **not** add JSON `scope`.
- Do **not** change ranking, role strip, score brackets.
- Do **not** auto-widen scope.
- Do **not** “fix” T227 OutputFormat residual here.
- Do **not** omit generated Session on non-empty (F5 empty-only).
- Do **not** rewrite T207 CHANGELOG history.
- Do **not** force sync `ProjectId::new()` → `None` without proof (F32).
- Do **not** align sync TTY default in this track (F34).

## Manual dogfood (copy)

```powershell
ai-brains recall "DECISION" --limit 2 --format pretty
ai-brains recall "DECISION" --limit 2 --global --format pretty
ai-brains recall "zzzznomatchesT228xyz" --format pretty
ai-brains sync query "DECISION" --no-bridge --format pretty
```

## Expected residual after ship

JSON optional `scope` · T230 labels · T231 unified search · omit-Session-on-nonempty soft · T227 F34 · T229 ops · **sync random-UUID project fallback (F32)** · **sync always-pretty default (F34)**.
