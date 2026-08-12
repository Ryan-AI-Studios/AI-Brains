# T240 — Project identity convergence

- **Track ID:** T240-ProjectIdentityConvergence
- **Status:** 🔄 **Implementing** (product on PR #144; close after CI + final Codex)
- **Category:** FEATURE / UX / OPS
- **Owner:** Grok
- **Source:** CLI audit 2026-08-11 P0; live identity triangle; T206 detect; T233 path aliases; **AI fold-in** `C:\dev\AI-review.md` AI1+AI2
- **Depends on:** T206 detect honesty; T212 list path column; T233 `register-path` / `find_path_alias_owner`; T205 local PROJECT_ID force-set
- **Blocks / feeds:** Honest daily Scope; unblocks audit scores for recall/preflight; informs T241 scope choice
- **Absorbs:** deferred identity row; detect path-blindness; whoami; mismatch warn; AI M1–M3 pins
- **Not absorbed:** T241 policy; T242 env quiet; T254 list-paths; auto-merge; auto `.env` write; memory heuristics

**Plan:** [plan.md](./plan.md) § AI fold-in

---

## 1. Objective

1. Make **all identity signals visible** (`project whoami`).
2. Fix **`project detect`** so path-alias (T233) beats empty name-collision projects.
3. **Warn** when daily env Scope ≠ path-alias of cwd (no silent auto-switch).
4. Document set-alias vs register-path vs `.env` SOOT.
5. Never auto-merge projects.

## 2. Problem (frozen live 2026-08-11)

| Signal | Project | Notes |
|--------|---------|--------|
| Repo `.env` PROJECT_ID | **441837f6** test-alias | Daily Scope `*`; ~683 pins |
| `project detect` (git slug) | **80f638d5** name=`ai-brains` | **0** mem; alias string looks like a path (set-alias misuse) |
| path alias `C:\dev\ai-brains` | **7d97a456** | **9305** mem; real work; no set-alias |
| Shell PROJECT_ID | was 7d97… | Overridden by local `.env` force-set |

cwd `C:\dev\AI-Brains` vs path `C:\dev\ai-brains` — normalization must treat as same location.

## 3. Frozen decisions (F0–F22)

| ID | Decision |
|----|----------|
| **F0** | Plan-only until go. |
| **F1** | Daily Scope still from effective env PROJECT_ID after dotenv (CLI flags / `--global` unchanged). |
| **F2** | **No** silent rewrite of daily Scope to path-alias. |
| **F3** | Mismatch warn when path-alias(toplevel/cwd) owner ≠ env project (non-fatal; **once/process**; SOOT message + whoami). |
| **F3b** | Skip warn: `--no-project-context`, argv `--global` pre-scan, no path alias, empty env. |
| **F4** | `project whoami` human (TTY/`IsTerminal`) + json: `effective_project_id`, `env_project_id` (post-dotenv), `shell_project_id` (pre-dotenv if differs), `path_alias_project_id`, `detect_project_id`, `git_slug`, `git_toplevel`, `mismatch`, `remediations[]`. |
| **F5** | Detect order: (1) path alias of toplevel/cwd (2) git slug + conflict rule (3) env post-dotenv (4) miss exit 1. |
| **F5b** | `collect_git_identity` returns **slug + toplevel** (do not discard toplevel). |
| **F6** | Path conflict: unique slug A + unique path B → **always prefer B** (no memory heuristic); stderr note A; extra note if B 0 mem and A >0 mem. |
| **F7** | T206 exact-first / ambiguous exit 1 / `GIT_TERMINAL_PROMPT=0` when no path. |
| **F8** | set-alias = label; register-path = filesystem; docs never conflate. |
| **F9** | No auto-merge; no auto-delete thin projects. |
| **F10** | No auto `.env` write; F14 `project use` **deferred** not DoD. |
| **F11** | Adapter-only CLI; pure helpers unit-tested. |
| **F12** | Soft: doctor `project_identity` via `vault_conn` (no AppContext). |
| **F13** | Soft deferred: detect `--json` schemaVersion 1 + `source` (not DoD). |
| **F14** | Soft deferred: `project use` (not DoD). |
| **F15** | Zero new crates; no clap forced bump. |
| **F16** | Capture independence: identity never requires models/graph. |
| **F17** | `--no-project-context`: skip env force-set/warn for IDs; whoami still path/detect with env null. |
| **F18** | Hermetic tests: path case, conflict rule, T206 regressions, whoami fields. |
| **F19** | CAPABILITIES + WORKFLOWS + OPERATIONS triangle diagram. |
| **F20** | Root CHANGELOG only for this track. |
| **F21** | detect `--export` comments include source (`path_alias` / `git_slug` / `env`). |
| **F22** | Cross-model review **required** before close. |

## 4. Acceptance criteria

| ID | Criterion |
|----|-----------|
| **AC0** | Plan-time identity triangle documented (done) |
| **AC1** | Docs precedence matches code |
| **AC2** | `whoami` shows shell/env/path/detect/effective + remediations (human + json) |
| **AC3** | Detect prefers path owner over unique slug collision (any mem counts) |
| **AC4** | Env≠path → single non-fatal warn + whoami hint |
| **AC5** | T206 hermetics still pass (ambiguous exit 1, exact-first) |
| **AC6** | Live dogfood after **operator** rebinds `.env`: daily Scope can hit 7d97… |
| **AC7** | Full CI gate; no production unwrap/expect |
| **AC8** | No silent daily Scope change without operator config change |
| **AC9** | `collect_git_identity` exposes toplevel for path lookup (subdir cwd still works) |

## 5. Implementation plan

See [plan.md](./plan.md) Phases 0–5.

## 6. Risks

| Risk | Mitigation |
|------|------------|
| Changing detect surprises export scripts | Stable when path unset; export includes comments |
| Warn spam | One clear line; coordinate later with T242 |
| Case/WSL path miss | `normalize_for_location_compare` |
| Operator doesn’t fix `.env` | AC6 requires operator step; product only unblocks honesty |

## 7. Operator runbook (after ship)

```powershell
cd C:\dev\AI-Brains
ai-brains project whoami
# Prefer main vault project for this repo:
ai-brains project set-alias 7d97a456-f2f4-43ea-1f13-211af684ad37 AI-Brains
# Edit .env:
# AI_BRAINS_PROJECT_ID=7d97a456-f2f4-43ea-1f13-211af684ad37
# Path alias already on 7d97… — keep
ai-brains project detect
ai-brains preflight --summary   # Scope should match main
```

## 8. Definition of Done

- [x] F0–F22 + AC0–AC9 (product implemented; F13/F14 soft deferred)
- [x] AI fold-in pins honored
- [ ] Cross-model final PASS + CI green + squash (process remaining)
- [ ] conductor / deferred / pin closeout

## 9. AI fold-in summary

| Absorb | Decline |
|--------|---------|
| M1 `collect_git_identity` slug+toplevel | Memory-count heuristic |
| M2 `--global` argv pre-scan | clap promote `--global` to global=true |
| M3 path-always-wins + 0-mem note | Auto Scope rewrite |
| whoami JSON fields + shell_project_id | is-terminal crate removal (workspace) |
| Once/process mismatch warn | F13/F14 as hard DoD |
| doctor vault_conn soft check | |
| export source comments; IsTerminal for whoami | |
| Cross-model required | |

Full disposition: [plan.md](./plan.md) § AI fold-in.

---

**Product shipped on branch `agent/T240-project-identity-convergence` (PR #144). Closeout after CI + final Codex.**
