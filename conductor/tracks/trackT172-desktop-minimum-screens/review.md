# T172 Review Log — Desktop Minimum Screens

**Track:** T172-DesktopMinimumScreens (P10.1)  
**Branch:** `feat/t172-desktop-minimum-screens`  
**Base:** `2918ae4` (T171)

## Round 1 — Internal (2026-07-30)

Verdict: **CLEAN** (no critical/high/medium). Lows R1-01–R1-06 opened.

| id | severity | description | status |
|----|----------|-------------|--------|
| R1-01 | low_info | Mutation errors message-only (not StatePanel) | fixed → R2 verified_fixed |
| R1-02 | low_info | Source not in nav | fixed → R2 verified_fixed |
| R1-03 | low_info | Claim id used as evidence inspect CTA | fixed → R2 verified_fixed |
| R1-04 | low_info | Bearer as plain String (not Zeroizing) | fixed → R2 verified_fixed |
| R1-05 | low_info | Claim detail route-state only (no re-fetch) | deferred (T173/T174) |
| R1-06 | low_info | Review resolve path id not percent-encoded | fixed → R2 verified_fixed |

## Round 2 — Internal re-review after easy lows (2026-07-30)

Commit: `db47ac2` + follow-up param/link polish.

Verdict: **CLEAN**

| id | severity | description | status |
|----|----------|-------------|--------|
| R1-01..04, R1-06 | low_info | See R1 | **verified_fixed** |
| R1-05 | low_info | Claim re-fetch by id | **deferred** |
| R2-01 | low_info | Evidence/Source route param not synced | **fixed** (useEffect sync) |
| R2-02 | low_info | Home evidence ids not linked | **fixed** (Link to /evidence/:id) |

## Codex reviews

- R1: see `review.codex.md` — **FAIL** → fixer pass (scope wiring, claims plural, adapter tests, zeroize)
- Fresh R2: see `review.codex.r2.md` (pending before gate clear)

### Codex R1 findings (fix pass)

| id | severity | description | status |
|----|----------|-------------|--------|
| P1-01 | critical | Scope not propagated to live screens | fixed_pending_verification |
| P1-02 | high | Claim route singular vs `#/claims/:kind/:id` | fixed_pending_verification |
| P2-01 | medium | httpmock tests bypassed adapter (`post_json`/`get_json`) | fixed_pending_verification |
| P2-02 | low | Token read path Zeroizing hygiene | fixed_pending_verification |

## Gate evidence (package-scoped)

- `cargo test -p ai-brains-desktop` — see fixer commit
- `cargo clippy -p ai-brains-desktop --all-targets -- -D warnings` — see fixer commit
- `npm run typecheck` / `license:check` — see fixer commit
