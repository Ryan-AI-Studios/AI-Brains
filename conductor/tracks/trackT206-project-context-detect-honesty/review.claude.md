## T206 Independent Completion Review ΓÇö Project context + detect honesty

**Scope reviewed:** `crates/ai-brains-cli/src/commands/project.rs` (working diff), `crates/ai-brains-cli/tests/project_detect_honesty.rs` (new), `Docs/CAPABILITIES.md`, `Docs/OPERATIONS.md`, `CHANGELOG.md`, `.claude/skills/ai-brains/SKILL.md`, `conductor/conductor.md`, `conductor/deferred.md`, `crates/ai-brains-cli/src/main.rs` (dispatch), `crates/ai-brains-cli/tests/empty_states_exit_hygiene.rs` (regression guard), `crates/ai-brains-cli/tests/common/mod.rs`.

Note: sandbox permissions blocked me from re-running `cargo nextest`/`clippy`/`fmt` directly in this session (Bash denied for non-git commands). I relied on static code/test reading plus the reported gate results in the task brief (24 passed, clippy clean, fmt clean, internal R1 clean).

### 1. AC / F requirement audit

| ID | Status | Evidence |
|----|--------|----------|
| AC1 | Γ£à | `project_detect__unique_git_wins_over_wrong_env` ΓÇö git match returns before env check reached |
| AC2 | Γ£à | `project_detect__env_only__from_env_exit_0` |
| AC3 | Γ£à | `project_detect__env_hit_git_mismatch__warn_exit_0` + `ΓÇªexport_mismatch__hash_comments_exit_0` |
| AC4 | Γ£à | `project_detect__ambiguous__exit_1_lists_candidates` + `ΓÇªexport_ambiguous__exit_1` |
| AC5 | Γ£à | `project_detect__miss__exit_1_mentions_context` + pre-existing T198 regression `project_detect__miss__mentions_context_exit_1` still matches wording |
| AC6 | Γ£à | `CAPABILITIES.md:147` exact F12 split text; `context`'s `.ledgerful` claim retained separately |
| AC7 | Γ£à | `CHANGELOG.md` Unreleased/Changed entry |
| AC10 | Γ£à | `project_detect__remote_first_slug__dir_name_ignored`; code order matches spec ┬º10.4 (toplevel ΓåÆ remote origin ΓåÆ dir fallback) |
| AC11 | Γ£à | `git_command()` sets `GIT_TERMINAL_PROMPT=0`; unit test `git_command__sets_git_terminal_prompt_zero`; both git spawns in `get_git_repo_slug` route through it exclusively (verified no other `Command::new("git")` in the crate) |
| F1ΓÇôF3, F5, F6, F9, F18, F21 | Γ£à | Ambiguous ΓåÆ stderr + `exit(1)` unconditionally (no `exit(2)` anywhere in `detect`); silent `Ok(())` on ambiguous confirmed removed (old code returned `Ok(())` after only `tracing::info!` in non-export mode ΓÇö now gone); exact-first/contains-fallback with stable `project_id` sort |
| F4/F35 | Γ£à | Warning text matches spec ┬º10.2 verbatim, `set-alias` hint included, labeled "git/env project mismatch" (distinct from T205 override warning) |
| F7 | Γ£à | Every detect-path git spawn hermetic |
| F12 | Γ£à | Exact replacement text, no ledgerful claim on `detect` |
| F31 | Γ£à | Remote-first, dir-name fallback only |
| F32 | Γ£à | HTTPS, scp-style SSH, ssh:// with port, `.git` suffix, Windows drive-letter guard ΓÇö all unit tested |
| F33 | Γ£à | `match_projects_for_slug`, `env_fallback_warning` pure, unit-tested without vault spawn |
| F8/F36 (soft) | Γ£à correctly residual | No `--json` flag anywhere in `main.rs`/`project.rs`; CHANGELOG explicitly notes residual |
| F10 (soft) | Γ£à correctly residual | `context.rs` untouched, no half-wired flag |

### 2. Findings

**P3 (non-blocking, informational ΓÇö not proposing for deferred.md, trivial rather than "difficult")**

- `tests/project_detect_honesty.rs` uses `common::hermetic_bin()` (which strips ambient `AI_BRAINS_PROJECT_ID` from the *parent-supplied* child env) but does **not** call `isolate_empty_home` per F34, so the child process's own always-on global-dotenv gap-fill (`main.rs:1781`, `dotenvy::from_path` against the real `USERPROFILE`/`HOME`) is not redirected. In practice this doesn't cause flakiness for any of the current tests: the only test relying on a truly absent `AI_BRAINS_PROJECT_ID` (`project_detect__miss__exit_1_mentions_context`) uses a freshly `init_vault`'d vault with zero registered projects, so even a leaked global `PROJECT_ID` value could never resolve to a vault match ΓÇö the miss path is unaffected. All ambiguous/unique-git tests short-circuit before the env-var branch is reached. Still, this is a literal gap vs. F34's stated pattern; worth an `isolate_empty_home` pass if the suite is touched again, but not worth blocking or deferring.
- No test explicitly exercises `--export` + miss (only human-mode miss and `--export` + ambiguous are tested). The code path is shared/unconditional (`exit(1)` regardless of `export_shell`), so this is coverage-completeness only, not a functional gap.

No P0/P1/P2 findings.

### 3. Extra-focus checklist (from review brief)

- Silent `Ok` on ambiguous: **confirmed removed** ΓÇö old code's non-export ambiguous branch returned `Ok(())` after only a `tracing::info!` (invisible without `RUST_LOG`); now both branches print to stderr and `exit(1)` unconditionally.
- Directory-first slug: **confirmed not primary** ΓÇö `get_git_repo_slug` tries toplevel (existence check only) ΓåÆ remote origin ΓåÆ dir-name fallback, matching spec ┬º10.4 order exactly.
- CAPABILITIES ledgerful claim: **confirmed removed for `project detect`**, retained correctly for `context`.
- No auto `.env` rewrite: **confirmed** ΓÇö no `fs::write`/`.env` mutation anywhere in `detect`.
- Production `unwrap`/`expect`: **confirmed zero** in non-test code (`project.rs`); the one `.expect(...)` found is inside `#[cfg(test)] mod tests`.

### 4. Docs honesty

CAPABILITIES/OPERATIONS/skill/CHANGELOG diffs all match F12/┬º10.3 wording precisely; no stray references to old "git / `.ledgerful` / `.env`" detect claim remain anywhere in `Docs/`.

### Verdict: **PASS**

No P0ΓÇôP2 findings. The two P3 observations above are informational and not proposed for `deferred.md` (both trivial, neither blocking correctness given current test design).
