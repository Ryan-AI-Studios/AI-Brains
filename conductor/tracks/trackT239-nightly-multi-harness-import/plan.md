# T239 Plan — Nightly multi-harness import

**Status:** Engineering complete — awaiting PR CI + squash-merge + closeout  
**Category:** FEATURE / OPS  
**Depends:** T234 ✅, T236 ✅, T237 ✅, T238 ✅  

See `spec.md` and `review.md` for full DoD / review matrix.

## Phase checklist

### Phase 0 — Preflight
- [x] ledgerful doctor + ledger status
- [x] ledgerful ledger start T239 (tx 8d35f716-53d4-48b4-8f9d-f313923b7797)
- [x] scan --impact
- [x] No dep bumps

### Phase 1 — Report + hermetic orchestrator
- [x] AC1–AC4, AC13 hermetic tests
- [x] MultiImportReport / SourceImportReport
- [x] run_multi_harness_import; make_sink; D20 overrides; F22 soft-skip

### Phase 2 — Nightly wire + status
- [x] AC6/AC11/AC12 status
- [x] Wire nightly + persist
- [x] Clap flags + SYSTEM skip-import

### Phase 3 — Smoke + docs
- [x] Smoke skip flags + status never/unreadable
- [x] CAPABILITIES / OPERATIONS / WORKFLOWS / antigravity-rule / CHANGELOG

### Phase 4 — Verify / close
- [x] Full gate 2385 nextest
- [x] Internal + Codex (r1 FAIL → fix → r2 **PASS**)
- [ ] PR + CI green + squash-merge
- [ ] Conductor Completed + deferred S9 closed + coordinated

## Soft residuals
S-SYS, S-JSON, S-DOC, S-SESSION (partially addressed via soft-skip), S-HOME, S-CAP, S-CLAUDE (T239+), S-FORCE, S-BRAINLOG, S-BUDGET
