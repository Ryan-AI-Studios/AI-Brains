# Track Completion Audit — T327-PreflightAuthorityFirst

**Reviewer:** Codex (`gpt-5.6-sol`, read-only, 2026-08-30)  
**Source:** `codex exec -s read-only` (this file captured from the exec transcript; `-o` path was not written by the CLI)

## Verdict: FAIL (at time of review)

Two P1 and two P2 findings. Full workspace gate was already green. Orchestrator dispositions below in `review.md`.

## Findings (as filed)

### P1 — AC7 Session narrowed to five-turn window
`sessions.rs` LIMIT 5. Spec AC7 fixture 1 USER + 1 DECISION + 8 Other cannot all be loaded. Test used USER + 4 Other.

### P1 — Completion/provenance/publish unfinished
Uncommitted working tree; FEATURE TX pending; no PR yet.

### P2 — AC1 contradictory vs F44
Spec AC1 wants needle in Index 1; F44 forbids body-skip so whale (newer) is Index 1.

### P2 — Weak proofs
AC4 `<= 15`; AC13 T315 fallback; AC17 hermetic ×2 not recorded.

## Orchestrator note

This FAIL was against an in-progress tree (expected for P1 provenance). Product P1 LIMIT-5 is **out of scope** (F19). Proof P2s were tightened after this review. See `review.md`.
