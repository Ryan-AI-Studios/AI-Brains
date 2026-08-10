# T224 Review Log — Search/display role-prefix strip

**Track:** T224-SearchRolePrefixStrip  
**Branch:** `feat/T224-search-role-prefix-strip`  
**Commits:** `3cb7d4b` (feat) + follow-up fix for Codex R1  
**Ledger TX:** `aefb8e8a-919e-4e72-840d-1bda5a761f9f`

## Rounds

| Round | Reviewer | Verdict | Notes |
|-------|----------|---------|-------|
| R1 | Internal (explore) | **CLEAN** | No Critical/High/Medium; AC1–AC13 product path pass |
| R2 | Codex (`review.codex.md`) | **FAIL** (process + 2×P2) | See dispositions below |
| R3 | Codex re-review | *pending* | After P2 fixes + evidence update |

## Manual dogfood (2026-08-10)

```text
ai-brains recall "DECISION" --format pretty --limit 2
→ DECISION: … (no leading ASSISTANT:)

ai-brains recall "DECISION" --limit 1
→ JSON content still "ASSISTANT: DECISION: …"

ai-brains forget --match "DECISION" --dry-run
→ previews "DECISION: …" with … truncation; no ASSISTANT:
```

## Full local gate

- `cargo fmt --check` clean
- `cargo clippy --workspace --all-targets -- -D warnings` clean
- `cargo nextest run --workspace` **2487 passed** (1 skipped)
- `cargo deny check` + `cargo audit` (allowed warnings only)
- Result: **FULL_GATE_OK**

## Codex R1 findings disposition

| ID | Sev | Title | Disposition |
|----|-----|-------|-------------|
| C1 | P1 | Plan/closeout incomplete | **Partly valid (process)** — plan Phase 6 checkboxes + review.md updated; conductor Completed + deferred strike deferred until squash-merge closeout (same as T219) |
| C2 | P2 | AC4–AC6 no forget-site regression tests | **Validated → fixed** — `forget_match_preview` / `forget_multi_preview` pure helpers used at all human sites; units lock strip + max 100/80 + `…` |
| C3 | P2 | Dual token list in preflight `is_session_turn_start` | **Validated → fixed** — `ROLE_PREFIXES` SOOT + `has_leading_role_prefix`; preflight detects via helper (AC10) |

## Internal R1 findings

None open (Medium+).

## Soft residuals (not DoD)

- Converge `truncate_preview` triplication (ingest/pin) onto shared helper
- Optional JSON `preview` field / `--strip-roles`
- Promote `strip_role_prefix` to core for retrieval converge
- clap pin → 4.6; T228 Scope; T231 search unify
