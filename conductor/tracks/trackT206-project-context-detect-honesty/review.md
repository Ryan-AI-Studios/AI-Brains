# T206 Review Log — Project context + detect honesty

**Track:** T206-ProjectContextDetectHonesty  
**Ledger TX:** `649ed8e6-b74a-46a3-9b1f-8ccf59c92329`  
**Branch:** `feature/t206-project-detect-honesty`

## Scope shipped

- F31 remote-first `get_git_repo_slug` + F7 `GIT_TERMINAL_PROMPT=0` on all detect git spawns
- F3 exact-first `match_projects_for_slug` (Unique | Ambiguous | None)
- F5/F18 ambiguous → stderr + exit 1 (human + `--export`)
- F4/F35 env fallback mismatch warn + set-alias hint (exit 0; export `#` comments)
- F12 CAPABILITIES detect vs context split; OPERATIONS/skill one-liners; CHANGELOG minor
- Unit helpers + hermetic `tests/project_detect_honesty.rs`

## Residuals (soft)

| ID | Item | Disposition |
|----|------|-------------|
| F8 / AC8 | `--json` with `source` field | Deferred — no half-wired clap flag |
| F10 / AC9 | `context --show` file-scoped mismatch | Deferred — detect F4 is SOOT honesty |
| F24 | resolve exact-first reuse | Soft not done |

## Findings

### Internal R1 (2026-08-04) — CLEAN

| ID | Severity | Status | Note |
|----|----------|--------|------|
| — | — | — | No P0–P2. Soft F8/F10/F24 deferred as planned. |

**Gate (orchestrator):** focused nextest 24/24 (unit + hermetic + miss regression); `clippy -p ai-brains-cli --all-targets -D warnings` pass; `fmt --check` pass.

### Cross-model R1 (2026-08-04) — Claude (Codex rate-limited)

| Source | Verdict |
|--------|---------|
| Codex primary | **Unavailable** — usage limit until ~2026-08-07 |
| Claude Sonnet 5 / high (fallback) | **PASS** — no P0–P2; raw `review.claude.md` |

Claude P3 notes (fixed post-review, test-only):
| ID | Severity | Status | Note |
|----|----------|--------|------|
| F34 hermetic home | P3 | **verified_fixed** | Suite now uses `isolate_empty_home` via `hermetic_detect()` |
| export+miss coverage | P3 | **verified_fixed** | `project_detect__export_miss__exit_1_hash_comment` |

Production code unchanged after Claude PASS. Focused nextest post-fix: 10/10 project_detect hermetic+regression green.

### Final gate evidence

| Gate | Result |
|------|--------|
| Full workspace gate (pre test-only P3 fix) | FULL_GATE_OK — fmt, clippy workspace, nextest 2046, deny, audit |
| Focused post F34 fix | project_detect hermetic 10 passed |
| Internal R1 | CLEAN |
| Cross-model | Claude **PASS** (Codex rate-limited) |

**Completion decision:** Engineering DoD met (AC1–AC7, AC10–AC11). Soft residual F8/F10/F24 only.

**Shipped:** PR #89 squash-merged `d727fc5` (2026-08-04). CI gate-windows / gate-linux / gate-macos all SUCCESS.
