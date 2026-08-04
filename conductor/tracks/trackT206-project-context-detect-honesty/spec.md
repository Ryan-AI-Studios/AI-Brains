# T206 — Project context + detect honesty

- **Track ID:** T206-ProjectContextDetectHonesty
- **Phase:** Post-T204 / post-T205 skill·CLI audit follow-ups (P1)
- **Status:** 📋 **Proposed / Expanded + AI fold-in** (plan-only until go)
- **Depends on:** T89 set-alias; T93 env fallback; T198 soft miss copy; **T205** global dotenv (PR #88); T204 closed
- **Blocks / feeds:** Honest cold-start project scoping; T207 recall empty/scope; T212 list labels separate
- **Category:** FEATURE / DOCS
- **Source:** Audit 2026-08-04 — detect **4/4**; **test-alias `.env` hijack**
- **Deferred absorbed:** Detect/env confusion; T93 `--json` soft residual; CAPABILITIES detect vs context honesty
- **Not absorbed:** T212 list redesign; IdP; T205 dotenv changes; MSI; auto `.env` rewrite; inventing ledgerful detect; git2
- **Research date:** 2026-08-04
- **AI fold-in:** 2026-08-04 — AI1 **M1–M6** accepted; **L1–L5** notes (L1 elevated). AI2 affirms F3–F5/F12. Disposition **§14**.
- **Ledger:** plan-only until go

## 1. Objective

1. Honest **`project detect`**: source clarity (`git` vs `env` vs miss) and **git vs env disagreement**.
2. Fix **silent exit 0** on ambiguous git matches.
3. When env wins but git slug known and unmatched → **stderr warn** + `set-alias` hint (exit 0).
4. Docs: detect chain truthful; **context** keeps real `.ledgerful` claim separately.
5. Soft: `--json` `source`; soft `context --show` mismatch (file-scoped — L3).

## 2. Live baseline (re-scan + AI verify 2026-08-04)

### 2.1 Audit reproduction — confirmed

| Fact | Value |
|------|--------|
| Git toplevel / origin | `AI-Brains` / `…/AI-Brains.git` |
| `.env` | `PROJECT_ID=441837f6-…` (**test-alias**) |
| Main vault project | `7d97a456-…` empty alias, 8389 memories |
| `project detect` | Exit 0 from **.env** / test-alias — **no** mismatch warn |
| `--export` | Export id, **no** `#` warning comment |

### 2.2 Code map — confirmed

| Issue | Location | Gap |
|-------|----------|-----|
| Ambiguous human path silent `Ok(())` | project.rs ~102–114 | F5 |
| Exact+contains same filter | ~77–85 | F3 |
| Env fallback no mismatch warn | ~118–139 | F4 |
| Slug = **directory name first**, remote only if empty | ~217–222 | **M1** — fork false-positive risk |
| No `--json` | Detect enum | F8 soft |
| CAPABILITIES detect claims `.ledgerful` | :147 | F12 — **context** uses ledgerful, detect does not |

### 2.3 Touch map

| File | Role |
|------|------|
| `project.rs` | detect, `get_git_repo_slug` (M1), pure helpers, GIT_TERMINAL_PROMPT |
| `main.rs` | Detect flags; soft `json: bool` if F8 ships |
| `context.rs` | Soft F10 show (file-only PROJECT_ID) |
| `tests/project_detect_honesty.rs` (preferred) | AC suite |
| `empty_states_exit_hygiene.rs` | Miss regression guard |
| CAPABILITIES / OPERATIONS / CHANGELOG | F12 exact text |
| T205 `isolate_empty_home` pattern | Hermetic PROJECT_ID control |

### 2.4 Deps

No git2/gix. dirs 6.0.0 latest. Zero new crates.

## 3. Research summary

| Finding | Application |
|---------|-------------|
| Live test-alias hijack | F4 |
| Silent ambiguous Ok | F5 exit **1** only (M6) |
| Directory-first slug | **F31** remote-origin preferred (M1) |
| CAPABILITIES conflates detect/context | **F12** exact replacement (M3) |
| GIT hang risk | **F7** required `GIT_TERMINAL_PROMPT=0` (L1) |
| T205 home isolation | Hermetic tests use same pattern |

## 4. Frozen decisions (F1–F36)

| ID | Decision |
|----|----------|
| **F1 — Scope** | Detect honesty + docs. No renames, no schema, no auto `.env` write. |
| **F2 — Detect order** | (1) Resolve **git identity slug** (F31). (2) Vault match. (3) Process `AI_BRAINS_PROJECT_ID` if in vault. (4) Miss exit **1**. |
| **F3 — Exact-first match** | Exact name/alias (case-insensitive) first; **contains** only if zero exact and **exactly one** contains hit. |
| **F4 — Env + git mismatch warn** | Env hit **and** slug non-empty **and** slug does not exact-match name/alias → stderr warning (template §10.2) + set-alias hint. Exit **0**. Label clearly as **git/env mismatch** (distinct from T205 “local .env overrides shell” warn — L5). `--export`: same text as `#` comments. Do not auto-write `.env`. |
| **F5 — Ambiguous git** | Print candidates to **stderr** (sorted by project_id — F21), exit **1**. No silent Ok. Human and `--export` both fail-closed. |
| **F6 — Source labels** | `from git` / `from env` (or `from .env`). Soft: `from env (AI_BRAINS_PROJECT_ID)`. |
| **F7 — Git tooling (L1 elevated)** | Subprocess git only. **Required:** set `GIT_TERMINAL_PROMPT=0` (and soft `GIT_ASKPASS`/non-interactive discipline if free) on **every** git spawn in detect path. No git2. |
| **F8 — `--json` soft** | Soft preferred, **not** DoD. If shipped: `ProjectCommands::Detect { export, json }` clap change + dispatch; fields `{ project_id, name, alias, source: "git_slug"\|"env", git_slug, warnings: [] }`. If deferred: explicit residual — no half-wired flag. |
| **F9 — `--export`** | Keep; F4 warnings as `#` lines; miss/ambiguous non-zero. |
| **F10 — context --show soft (L3)** | Soft only. `--show` is **file-based** (cwd `.env` lines) today — **not** process env. Mismatch warn on show only if project `.env` has PROJECT_ID (not global-only gap-fill). Do not expand show to full process env as DoD. Prefer detect F4 as SOOT honesty. |
| **F11 — set-alias UX** | Soft suggest command on miss/mismatch; never invent project. |
| **F12 — Docs (M3 exact)** | CAPABILITIES auto-detect row **replacement** (normative): **`project detect` (git slug → vault match → env `PROJECT_ID`); `context` init discovery (`.ledgerful` / `.env`)** — do **not** delete context’s real `.ledgerful` claim; only uncouple it from detect. OPERATIONS/skill one-line. |
| **F13 — No auto .env rewrite** | — |
| **F14 — Capture independence** | — |
| **F15 — Zero new crates** | — |
| **F16 — Hermetic locks** | AC1–AC5 + T198 miss regression; HOME isolation via T205 pattern when controlling global PROJECT_ID. |
| **F17 — High findings** | Silent ambiguous; env hijack no warn; fork false mismatch (M1); docs wipe ledgerful from context; auto .env. |
| **F18 — Exit codes (M6)** | Miss **and** ambiguous: **exit 1 only**. Drop “or 2”. Mismatch warn: exit 0. |
| **F19 — Review** | FEATURE; primary required. |
| **F20 — Series** | After T205; feeds T207. |
| **F21 — Determinism** | Sort ambiguous by project_id; stable warn template; no timestamps. |
| **F22 — Tests** | Pure helpers unit (no git); hermetic CLI + temp git when available; guard `project_detect__miss__mentions_context_exit_1`. |
| **F23 — T212 out** | — |
| **F24 — resolve soft** | Soft reuse exact-first helper; not DoD. |
| **F25 — Privacy** | No new secrets. |
| **F26 — Global PROJECT_ID** | Still labeled env source after T205 merge; F4 applies when slug known. |
| **F27 — after_help soft** | Soft detect examples + set-alias. |
| **F28 — Ledger** | On go. |
| **F29 — AI fold-in** | §14. |
| **F30 — Soft decline** | `--strict` fail mismatch; ledgerful detect; auto set-alias; git2. |
| **F31 — Git slug identity (M1)** | **`get_git_repo_slug` must prefer remote `origin` repo name** (`git remote get-url origin` → `extract_repo_name`) when available and non-empty. **Fallback** to toplevel directory name only if remote missing/fails/empty. Rationale: checkout dir names (`my-fork`) must not drive false F4 warnings when origin is still `AI-Brains`. Soft alternative declined as DoD: warn only if **both** dir and remote disagree (more complex; prefer single SOOT slug = remote-first). |
| **F32 — extract_repo_name tests (M2/L2)** | Preferred unit cases: HTTPS, SSH scp `git@host:user/repo`, `.git` suffix, ssh port path. Soft: guard Windows `C:\…` local remote (avoid `C` slug) if free. |
| **F33 — Pure helpers required** | `match_projects_for_slug` → Unique \| Ambiguous \| None; `env_fallback_warning(…) -> Option<String>`; unit-tested without vault spawn where possible. |
| **F34 — Hermetic HOME** | Tests controlling ambient/global PROJECT_ID use T205 `isolate_empty_home` / USERPROFILE+HOME tempdir pattern (`global_dotenv_key_gapfill` precedent). |
| **F35 — Dual-warning UX (L5)** | F4 warning prefix must read as git/env **project identity** mismatch, not shell/dotenv override. Manual check both can appear without confusion. |
| **F36 — `--json` plan boundary (M4)** | Soft only; if not implemented, plan marks **deferred residual** — do not leave unused clap flag. |

## 5. Acceptance criteria

| AC | Criterion | Proof |
|----|-----------|-------|
| **AC1** | Unique git match wins over wrong env PROJECT_ID; prints from git | Unit/hermetic |
| **AC2** | Env-only hit; from env; exit 0 | Hermetic |
| **AC3** | Env hit + remote slug mismatch → stderr warn + set-alias; exit 0 | Unit + hermetic |
| **AC4** | Ambiguous ≥2 matches → exit **1** + listed candidates (no silent Ok) | Unit/hermetic |
| **AC5** | Miss exit 1 + context/set-alias guidance; T198 regression green | Existing + suite |
| **AC6** | CAPABILITIES uses F12 exact split (detect vs context ledgerful) | Doc |
| **AC7** | CHANGELOG minor | Doc |
| **AC8** (soft) | `--json` `source` if F8 shipped | Test |
| **AC9** (soft) | context --show file-scoped mismatch if F10 | Manual |
| **AC10** | Slug prefers origin remote over directory name (F31) | Unit |
| **AC11** | Git spawns set `GIT_TERMINAL_PROMPT=0` | Code/unit |

## 6. Non-goals

Auto `.env` rewrite · auto set-alias · git2 · invent ledgerful detect · T212 list · `--strict` DoD · daemon

## 7. Risk & verification

| Risk | Mitigation |
|------|------------|
| Fork false F4 warn | F31 remote-first |
| Script stdout parse | Warn stderr; exit 0 on env hit |
| Test hang on git auth | F7 GIT_TERMINAL_PROMPT=0 |
| Docs delete context ledgerful | F12 split wording |

**Implement order:** B1 pure helpers + F31 slug + F7 → C1–C3 CLI → D1 hermetic + miss regression → D2 docs F12 text → gate.

## 8. Residual after ship

T212 · soft JSON · soft context show · optional Windows local-remote guard · monorepo multi-project

## 9. Series

After **T205** closed. Next suggested: **T207**.

## 10. Implementation notes

### 10.1 Pure helpers

```text
match_projects_for_slug(projects, slug) -> Unique(row) | Ambiguous(Vec) | None
env_fallback_warning(git_slug, project_id, name, alias) -> Option<String>
// F31:
resolve_git_slug(path) -> Option<String>  // origin first, else dir name
```

### 10.2 Warning template (F4 / F35)

```text
Warning: git/env project mismatch: AI_BRAINS_PROJECT_ID points to project {id} (alias={alias}) but git repo slug is '{slug}' which does not match this project's name/alias.
Hint: ai-brains project set-alias {id} {slug}
```

### 10.3 CAPABILITIES row (F12 / M3) — normative replacement

Replace detect-only false claim with either:

| Auto-detect | `project detect` (git slug → vault match → env `PROJECT_ID`); `context` discovery (`.ledgerful` / `.env`) |

or two rows (detect vs context). **Do not** claim ledgerful for `project detect`.

### 10.4 get_git_repo_slug target order (F31)

```text
1. git rev-parse --show-toplevel (fail → None)
2. git remote get-url origin → extract_repo_name (success → Some)
3. else toplevel directory file_name
```

Both git commands: `GIT_TERMINAL_PROMPT=0`.

## 14. AI fold-in disposition (2026-08-04)

| ID | Source | Disposition |
|----|--------|-------------|
| AI2 §1–4 | F3/F4/F5/F12 | **Accept** (already core) |
| **M1** | Remote origin slug > directory name | **Accept** → F31 / AC10 |
| **M2** | extract_repo_name edges | **Soft** → F32 unit tests |
| **M3** | CAPABILITIES exact split detect/context | **Accept** → F12 / §10.3 |
| **M4** | --json clap wiring boundary | **Accept** → F8/F36 soft |
| **M5** | resolve exact-first reuse | **Soft** → F24 |
| **M6** | Exit 1 only for ambiguous | **Accept** → F5/F18 |
| **L1** | GIT_TERMINAL_PROMPT=0 | **Accept required** → F7 / AC11 |
| **L2** | Pure helpers + URL unit tests | **Accept** → F32/F33 |
| **L3** | context --show file vs process | **Accept** → F10 clarify |
| **L4** | Sort ambiguous | **Affirm** F21 |
| **L5** | Distinct mismatch vs override warn | **Accept** → F35 / §10.2 |
| T198 miss test | Regression guard | **Accept** → F22 / AC5 |
| T205 home isolation | Hermetic pattern | **Accept** → F34 |

**Not folded:** inventing ledgerful detect; directory-first slug keep; `--strict` as DoD; git2.
