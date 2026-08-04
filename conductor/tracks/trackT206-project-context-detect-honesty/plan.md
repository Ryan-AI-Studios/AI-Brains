# T206 Plan — Project context + detect honesty

Status: **Completed** (PR #89 squash-merged `d727fc5`). Spec: [spec.md](./spec.md).

## Absorbed

| Residual | Disposition |
|----------|-------------|
| test-alias `.env` hijack | F4 |
| Ambiguous silent Ok | F5 exit **1** |
| CAPABILITIES false detect ledgerful | F12 exact split |
| Directory-first slug fork warn | **F31** remote-first (M1) |
| T93 `--json` | Soft F8/F36 |
| T212 list | Out |

## AI fold-in (2026-08-04)

| ID | Action |
|----|--------|
| **M1** | B0: flip `get_git_repo_slug` to origin-first (F31); AC10 |
| **M3** | D2: CAPABILITIES exact replacement §10.3 (detect vs context) |
| **M6** | C1: exit **1** only — no exit 2 option |
| **L1** | B0: `GIT_TERMINAL_PROMPT=0` on all detect git spawns (required) |
| **M2/L2** | B1: unit `extract_repo_name` + match helpers |
| **M4** | C4 soft `--json` only if full clap+tests; else residual |
| **M5** | Soft resolve reuse F24 |
| **L3** | F10: show mismatch file-scoped only |
| **L5** | F4 template “git/env project mismatch” |
| T198 | D1 regression: `project_detect__miss__mentions_context_exit_1` |
| T205 | D1: `isolate_empty_home` when controlling PROJECT_ID |

## Phases

### A0 — Expand + fold-in (done)

- [x] Live repro + F1–F36 + AC1–AC11  
- [x] On go: ledger start + scan --impact  

### A1 — Pure + slug fix (Red → Green)

- [x] **B0** F31 remote-first slug + F7 `GIT_TERMINAL_PROMPT=0` (AC10/AC11)  
- [x] **B1** Unit: `match_projects_for_slug` Unique/Ambiguous/None (exact-first)  
- [x] **B2** Unit: `env_fallback_warning` text + F35 label  
- [x] **B3** Unit: `extract_repo_name` URL matrix (F32)  
- [x] **B4** Green: wire helpers into `detect`  

### B — CLI

- [x] **C1** Ambiguous → stderr list + exit **1** (F5/F18)  
- [x] **C2** Env + slug mismatch → stderr warn + set-alias; exit 0; export `#` comments (F4/F9)  
- [x] **C3** Git unique wins wrong env (AC1)  
- [x] **C4** Soft: `--json` full or skip (F8/F36) — **SKIPPED residual** (no half-wired flag)  
- [x] **C5** Soft: context --show file-only mismatch (F10) — **SKIPPED residual**  

### C — Hermetic + docs

- [x] **D1** Hermetic AC1–AC5; miss regression; T205 home isolation if needed  
- [x] **D2** CAPABILITIES F12 exact text; OPERATIONS/skill  
- [x] **D3** CHANGELOG minor  
- [x] **D4** Review + full gate + PR #89 squash-merged  

## Test plan

| Lock | Assert |
|------|--------|
| AC1–AC5 | match/warn/ambiguous/miss |
| AC6 | docs split |
| AC10–AC11 | remote-first + GIT_TERMINAL_PROMPT |
| Soft AC8–9 | json / show |

## Manual

- [x] AI-Brains + test-alias `.env` → mismatch warn (not only override warn) — post-merge global install  
- [ ] `set-alias … AI-Brains` → detect from git (optional operator)  
- [x] Checkout remote-first covered by hermetic AC10

## Stop-before

- Auto `.env` rewrite  
- git2  
- Deleting context `.ledgerful` claim  
- Shipping unused `--json` flag  

## Done when

AC1–AC7 + AC10–AC11 green; review clear; PR #89 merged `d727fc5`.
