# T237 Review Log — Grok Build seamless ingest

## Scope

- **Track:** T237 Grok Build seamless ingest
- **Branch:** `feat/T237-grok-build-seamless-ingest`
- **Category:** FEATURE / capture surface
- **Depends:** T234 ✅, T235 ✅, T236 lessons ✅

## Reviewers / rounds

| Round | Source | Verdict | Notes |
|-------|--------|---------|-------|
| IR1 | Internal subagent | **FAIL** | Missing hermetic import suite (AC5–AC8/AC18); AC3 proof weak |
| IR2 | Internal re-review | **PASS WITH DEFERRED P3** | Hermetic suite solid; AC3 helper-level residual |
| CX1 | Codex gpt-5.4 high | **FAIL** | P1 dry-run missing; P1 AC3 not vault-proven; P2 preflight AGY decline suppresses Grok |
| CX2 | Codex re-review | **FAIL** | dry-run/preflight fixed; AC3 still not wired through live hook |
| CX3 | Codex re-review | **FAIL** | AC3 wire OK; **HIGH** unbound env stamps `grok-unbound` onto env project |
| CX4 | Codex re-review | **PASS WITH DEFERRED P3** | AC6 anti-hijack fixed; soft-install labels residual |
| CX5 | Codex final | **PASS** | Display labels fixed; no open findings |

## DoD matrix (final)

| AC | Status | Evidence |
|----|--------|----------|
| AC1–AC2 | MET | F11 matrix unit tests in `message_only.rs` |
| AC3 | MET | `append_grok_turns` (thinking:None); `grok-hook` calls it; vault hermetic `hook_path__vault_ingest__thinking_none` |
| AC4 | MET | Shared `generate_grok_turn_id` / `append_grok_turns` live+batch |
| AC5–AC8 | MET | `grok_import_t237` hermetic suite |
| AC9–AC12 | MET | install empty stdout, dry-run, foreign keep, timeout 120 |
| AC13–AC20 | MET | normalize, updates.jsonl structural, subagent skip, percent encode, resolve fallbacks |
| AC14–AC17 | MET | discovery SOOT, docs, corrupt refuse, capture independence |

## Findings disposition

| ID | Sev | Status | Disposition |
|----|-----|--------|-------------|
| IR1-P1 hermetic import | P1 | verified_fixed | `grok_import_t237.rs` 9 tests |
| CX1 dry-run | P1 | verified_fixed | `--dry-run` + dry path in importer |
| CX1 AC3 | P1 | verified_fixed | `append_grok_turns` + hook wire + vault proof |
| CX1 preflight decline | P2 | verified_fixed | per-harness decline filter |
| CX3 unbound env hijack | High | verified_fixed | env routes session; does **not** stamp unbound alias |
| CX4 soft labels | P3 | verified_fixed | `display_name()` on soft paths |
| F12 was DoD | — | fixed | dry-run shipped |
| AdapterKind omits Grok | P3 | deferred | registry unused for Grok capability; `grok_capability()` exported |

## Gates (orchestrator)

```
cargo fmt --check                          OK
cargo clippy --workspace --all-targets -D warnings  OK
cargo nextest run --workspace              2338 passed (1 skipped)
cargo deny check                           OK
cargo audit                                OK (pre-existing allowed glib warnings only)
```

Codex final: **PASS** (`review.codex.final.md`).

## Soft residuals (not blocking)

- UserPromptSubmit (S1), opt-in subagent include (S2), watermark (S3), fingerprint turn-ids (S8)
- Claude/Codex install_ready still follow-up / T238+
- AdapterKind::Grok optional registry wire
- OpenCode / multi-harness nightly T238/T239

## Completion decision

Engineering DoD met. Final Codex **PASS**. Ready for PR + CI + squash-merge + conductor closeout.
