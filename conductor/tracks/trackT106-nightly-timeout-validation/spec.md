# Track T106: Nightly End-to-End Timeout Validation

**Status:** Pending
**Started:** —
**Owner:** —
**Priority:** P3 — T100 added timeouts but they haven't been validated end-to-end.
**Source:** Systematic command test 2026-06-22.

---

## Problem Statement

T100 added per-request timeouts (120s/30s/10s) to `LlamaCppProvider` to prevent `ai-brains nightly` from hanging indefinitely. The unit tests verify that a single request times out correctly. However, the nightly pipeline has not been tested end-to-end to confirm that:
1. A timed-out session is logged as an error and skipped (not retried or hung).
2. The nightly run completes in bounded time when multiple sessions time out.
3. The error message is informative (includes session ID and timeout duration).
4. Sessions that succeed are not affected by the timeout configuration.

The nightly status shows 2 unsummarized sessions remaining. A real `nightly --skip-import` run would validate the timeout behavior under production conditions.

## Acceptance Criteria

**AC1:** `ai-brains nightly --skip-import` completes in bounded time (<10 minutes) even if some sessions time out. No hang.

**AC2:** Sessions that time out are logged with `tracing::error!` including the session ID and the error message. The error is added to the `errors` vector in the nightly service.

**AC3:** The nightly summary at the end of the run reports: `N sessions summarized, M errors, K sessions remaining.` where errors include timeout-related failures.

**AC4:** A timeout to a dead LLM backend (set `AI_BRAINS_MODEL_URL=http://127.0.0.1:1`) causes the nightly to skip all sessions with timeout errors within the configured timeout, not hang.

**AC5:** Normal nightly operation (LLM backend available) is not affected — sessions are summarized correctly, no spurious timeouts.

## Design Notes

- This is primarily a validation/observability track, not a code change track. The T100 implementation should already satisfy these criteria.
- If any criteria fail, create a follow-up track to fix the specific issue.
- Test with a dead backend by setting `AI_BRAINS_MODEL_URL=http://127.0.0.1:1` and running `ai-brains nightly --skip-import`. All sessions should time out within ~2 minutes (120s per session * N sessions, but with the per-request timeout the connection refused error fires immediately).
- Test with a live backend by running `ai-brains nightly --skip-import` with the normal config. Verify sessions are summarized.
- Check the nightly status after each run: `ai-brains nightly --status`.

## Files

- No code changes expected. If criteria fail, identify the specific file and create a fix track.
- `conductor/tracks/trackT106-nightly-timeout-validation/report.md` — write the validation results here.

## Tests (TDD)

No new tests. This is a manual validation track.

## Verification

1. `ai-brains nightly --skip-import` with dead backend → completes in <2 min, all errors logged.
2. `ai-brains nightly --skip-import` with live backend → completes normally, sessions summarized.
3. `ai-brains nightly --status` → shows updated last-run time, correct error/session counts.

## Out of Scope

- Changing timeout values (T100 already made them configurable via env vars).
- Adding retry logic (separate track if needed).
- Testing with a real large-context session (the nightly already has chunking).
