# T196 Review Log — Service Units + Ops Docs Hygiene

## Scope
Reference systemd/launchd packaging under `packaging/reference/`, root `CONTRIBUTING.md`, docs/claims residual reword, soft `scripts/check-reference-units.sh`, soft Unix SIGTERM graceful shutdown in `ai-brainsd`.

Ledger TX: `19c21fe3-cc52-49d5-a5b2-51938c756128` (DOCS).

## Reviewers / rounds

| Round | Source | Verdict |
|-------|--------|---------|
| Internal R1 | subagent | **FAIL** — P3 `mapfile` non-portable in check script |
| Fix | orchestrator | Replaced `mapfile` with bash 3.2-safe `while read` |
| Internal R2 | subagent | **PASS** |
| Codex R1 | gpt-5.6-luna high | **FAIL** — P1 wrapper permissions; P2 process closeout (mid-ship); P3 SIGTERM delivery test |
| Fix | orchestrator | Wrapper owner-only 0600/0400 fail-closed (644/640 fail, 600/400 pass) |
| Codex R2 | gpt-5.6-luna high | **PASS WITH DEFERRED P3** |

## DoD / AC matrix (engineering)

| AC | Status |
|----|--------|
| AC1–AC7, AC9–AC14 | **Met** |
| AC8 deferred strike / conductor Completed | **Ship process** (post-merge closeout) |

## Findings dispositions

| ID | Sev | Disposition |
|----|-----|-------------|
| mapfile bash-4 | P3 | **verified_fixed** |
| wrapper 0640/0644 allowed | P1 | **verified_fixed** (Codex R2) |
| process closeout mid-implement | P2 | **out_of_scope** until ship (orchestrator owns) |
| SIGTERM child-process delivery test | P3 | **deferred** — F36 soft; wiring + spawn/abort test present; real delivery test residual |

## Gates (observed)

- `bash scripts/check-reference-units.sh` — PASS
- `cargo clippy -p ai-brainsd --all-targets -- -D warnings` — PASS
- `cargo nextest run -p ai-brainsd --lib` — 43/43 PASS
- Wrapper mode dry-check — 644/640 FAIL, 600/400 PASS

## Deferred (append ISSUES / deferred as appropriate)

- **T196-P3 SIGTERM delivery test:** unit tests do not send SIGTERM to a live daemon child and assert graceful exit. Soft F36; code wired.

## Completion decision (engineering)

Engineering DoD met with one deferred P3. Process closeout (PR, CI, conductor Completed, deferred strike, ledger commit, final Codex after ship) follows PR merge.

## Ship record

- **PR:** #79 squash-merged `3f16648` (2026-08-02)
- **CI:** gate-windows / gate-linux / gate-macos all SUCCESS
- **conductor / deferred:** Completed + residual strikes on closeout commit
- **Deferred P3 retained:** SIGTERM child-process delivery test (F36 soft)
- **Final Codex:** after closeout commit (gate for finished)

## Final Codex gate
- **review.codex-final.md**: **PASS WITH DEFERRED P3** (2026-08-03)
- Only residual engineering: SIGTERM child-process delivery test P3

