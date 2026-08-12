# T240 Plan — Project identity convergence

**Status:** 🔧 **Implementing** (code + hermetics green; cross-model / full gate / dogfood for close)  
**Spec:** [spec.md](./spec.md) § AI fold-in / F0–F22  
**Category:** FEATURE / UX / OPS  
**Ledger TX:** `0ee32f70-2565-448a-b39e-10ae87f36095` (T240-project-identity-convergence FEATURE) — left open for coordinator

---

## AI fold-in (2026-08-11) — `C:\dev\AI-review.md` AI1 + AI2

Both AIs re-verified code + live triangle; **no Highs**. Verdict: fold M1–M3 + pins below, then go.

### Disposition

| ID | Source | Verdict | Action |
|----|--------|---------|--------|
| **M1** git toplevel discarded by `get_git_repo_slug` | AI1+AI2 | **Agree** | Refactor → `collect_git_identity(cwd) -> GitIdentity { slug, toplevel }` (single `rev-parse --show-toplevel` + remote); path lookup uses **toplevel** then cwd fallback |
| **M2** `--global` not global clap; pre-parse skip | AI2 | **Agree** | At mismatch-warn site: `args.iter().any(|a| a == "--global")` same pattern as `--no-project-context` |
| **M3** path-conflict memory edge case | AI2 | **Agree** | **Path owner always wins** (no memory heuristic). Extra stderr if path owner 0 mem and slug hit >0 mem: verify path alias via `project list` |
| **M3/M4 AI1** whoami JSON + mismatch warn text | AI1 | **Agree** | Structured fields + SOOT warn line |
| **L1** is-terminal crate | AI2 | **Agree for new code** | `std::io::IsTerminal` for whoami TTY; leave existing is-terminal OOS |
| **L2** whoami JSON names | AI2 | **Agree** | Verbose `effective_project_id` etc. (not bare `project_id`) |
| **L3** detect `--json` schema | AI2 | **Defer soft** | F13 not DoD; if shipped: schemaVersion 1 + source enum |
| **L4** whoami + `--no-project-context` | AI2 | **Agree** | `env_project_id` null; still path/detect |
| **L5** once-per-process warn | AI2 | **Agree (refined)** | Needs vault for `find_path_alias_owner` → fire **once** via `Once`/`OnceLock` on first vault-open with project context (not pure pre-dotenv; not every subcommand re-fire) |
| **L6** `project use` | AI2 | **Defer** | F14 not DoD |
| **L7** detect `--export` source comment | AI2 | **Agree** | `# from path_alias` / `git_slug` / `env` |
| **L8** detect step 3 post-dotenv env | AI2 | **Agree** | Intentional daily Scope SOOT |
| **L9** shell vs .env in whoami | AI2 | **Agree** | Capture `shell_project_id` **before** `apply_local_project_context_env` force-set |
| **L10** doctor uses `vault_conn` | AI2 | **Agree** | No AppContext; soft check only |
| **O11** cross-model | AI2 | **Agree required** | FEATURE high UX risk |
| **O12/O13** IsTerminal + docs triangle | AI2 | **Agree** | Phase 2/4 |
| Memory-count prefer path only when empty | — | **Decline** | Heuristic; path is SOOT |
| Make `--global` clap global=true | AI2 opt 3 | **Decline** | Too invasive |
| Heuristic memory comparison for conflict | AI2 opt 2 | **Decline** | |

### Pins locked by fold-in

1. **Git identity (M1):**  
   ```rust
   struct GitIdentity { slug: Option<String>, toplevel: Option<PathBuf> }
   fn collect_git_identity(cwd: &Path) -> Result<GitIdentity, …>
   ```  
   Path alias: normalize(toplevel) → `find_path_alias_owner`; else normalize(cwd).  
   Keep `GIT_TERMINAL_PROMPT=0`. Update detect + tests.

2. **Conflict F6 (M3):** Path B always wins over slug A. Stderr: note A. If B.mem==0 && A.mem>0: extra “verify path alias (`project list`)” note.

3. **whoami JSON (M3/L2/L9):**  
   `effective_project_id`, `env_project_id` (post-dotenv), `shell_project_id` (pre-dotenv, if set/differs), `path_alias_project_id`, `detect_project_id`, `git_slug`, `git_toplevel`, `mismatch`, `remediations[]`.  
   Human default on TTY via `std::io::IsTerminal`; JSON when piped or `--format json`.  
   Subcommand name `whoami` under `project` — **no** collision with Windows `whoami.exe`.

4. **Mismatch warn (M4/M2/L5):**  
   ```text
   Warning: project identity mismatch: daily Scope is '{env}', but path is registered to '{path}'. Run 'ai-brains project whoami'.
   ```  
   Skip: `--no-project-context`, `--global` (argv pre-scan), no path alias, no env project.  
   **Once per process** (`Once`/`OnceLock`) at first vault-using command path that can query aliases — **never** mutates PROJECT_ID.

5. **detect `--export`:** comment includes source (`path_alias` / `git_slug` / `env`).

6. **F13/F14:** soft deferred with schemas reserved; not DoD.

---

## Preflight (plan time — 2026-08-11)

| Check | Result |
|-------|--------|
| `ai-brains preflight --summary` | Scope **test-alias** (441837f6…) — wrong for repo work |
| Live `project detect` | `ai-brains` **80f638d5…** alias=`C:\dev\ai-brains` **0 memories** (git slug exact name) |
| Live main vault project | **7d97a456…** (no alias) **9305** mem; **path** `C:\dev\ai-brains` |
| Live active `*` | **test-alias** from **repo `.env`** force PROJECT_ID |
| Shell vs local .env | Warning: local overrides shell PROJECT_ID (was 7d97a456…) |
| Global `~/.ai-brains/.env` | VAULT_PATH only (no PROJECT_ID) — correct |
| cwd | `C:\dev\AI-Brains` (case differs from path alias string) |
| origin | `https://github.com/Ryan-AI-Studios/AI-Brains.git` → slug **AI-Brains** / extract **AI-Brains** or **ai-brains** |
| `ledgerful ledger status` | 0 pending, 0 unaudited drift |
| Deps | **No bumps** — clap lock **4.6.1** (crates.io 4.6.6); dotenvy existing; no git2 |

### Live identity triangle (root cause)

```text
                    git slug "ai-brains"
                           │
                           ▼
              80f638d5  name=ai-brains  alias=C:\dev\ai-brains  mem=0
                           ▲ mis-used set-alias as path label

  repo .env PROJECT_ID ──► 441837f6  test-alias  mem=683  (daily Scope *)

  path_alias normalize ──► 7d97a456  (no alias)  path=C:\dev\ai-brains  mem=9305
                           ▲ real work; register-path only
```

**Detect does not consult path aliases.** Daily commands do not call detect; they use force-set local `.env` PROJECT_ID (intentional since T205/T223).

---

## Deferred roll-in

| Item | Source | Disposition |
|------|--------|-------------|
| Default project identity (env/detect/path) | deferred T240 | **Absorb** — this track |
| CLI audit P0 identity | 2026-08-11 audit | **Absorb** |
| T206 soft: detect `--json` source | deferred T206 | **Soft absorb** if cheap (F14); not hard DoD |
| T212 soft: path_alias hermetic seed / verbose raw | deferred T212 | **Soft residual** — not DoD |
| T233 list-paths / unregister | T254 | **Decline** (other track) |
| Policy grants empty | T241 | **Decline** |
| Env warn spam once-per-process | T242 | **Decline** (orthogonal; may share whoami warn site carefully) |
| Auto-merge / delete test projects | — | **Decline** |
| Auto-rewrite `.env` without consent | T206 F1 | **Decline** — operator rebind only |
| Forced clap bump | crates.io 4.6.6 | **Decline** |
| Memory-count heuristic for path vs slug | AI2 | **Decline** — path always wins |
| Make `--global` clap `global=true` | AI2 | **Decline** — pre-scan only |

---

## Research (2026-08-11 + fold-in)

### Code SOOT

| Site | Role |
|------|------|
| `main.rs` `apply_local_project_context_env` | Local `.env` **force-sets** PROJECT_ID/SESSION_ID over shell |
| `main.rs` dotenv order | gap-fill dotenv → force project context → global gap-fill KEY/VAULT |
| `project.rs` `detect` | (1) git slug (2) vault name/alias match (3) env (4) miss exit 1 — **no path** |
| `get_git_repo_slug` | Computes toplevel then **discards** it — returns slug only (**M1**) |
| `find_path_alias_owner` | T233; available on QueryStore; detect unused |
| `--global` | Per-subcommand bool — **not** clap global (**M2**) |
| Doctor | 13 checks; no `project_identity` yet |

### Precedence freeze (post fold-in)

**Daily Scope:** `--project-id` / `--global` → effective env PROJECT_ID (post-dotenv). **Never** auto path.  
**Mismatch warn:** once/process when path owner ≠ env (see pins).

**Detect order:**

| Step | Signal | Behavior |
|------|--------|----------|
| 1 | Path alias of **git toplevel** else cwd | Prefer; `source=path_alias` |
| 2 | Git slug exact-first (T206) | If differs from step-1 → note A on stderr; **path wins** |
| 3 | Env PROJECT_ID (post-dotenv) if in vault | T206 mismatch warn with git if applicable |
| 4 | Miss | exit 1 |

### Conflict rule F6 — **locked**

Path owner **always** wins vs unique slug hit (regardless of memory counts).  
Stderr always notes slug project.  
Extra note if path owner 0 mem and slug hit >0 mem → verify `project list` / re-register-path.

### Deps

| Surface | Pin | Posture |
|---------|-----|---------|
| clap | lock 4.6.1 / crates.io 4.6.6 | **No bump** |
| dotenvy | current | **No bump** |
| git subprocess | `GIT_TERMINAL_PROMPT=0` | Keep |
| TTY | `std::io::IsTerminal` for **new** whoami | No new crate |

---

## Frozen open questions (post fold-in)

| # | Q | Decision |
|---|---|----------|
| 1 | Auto daily Scope → path? | **No** |
| 2 | Path vs slug prefer? | **Path always** (+ 0-mem note) |
| 3 | Auto `.env`? | **No** |
| 4 | `project use`? | **Deferred** not DoD |
| 5 | detect `--json`? | **Soft deferred** (schema reserved) |
| 6 | Doctor check? | Soft preferred F12 |
| 7 | Git API? | `collect_git_identity` returns slug+toplevel |
| 8 | `--global` skip warn? | Argv pre-scan |

---

## Phase 0 — Ledger + impact (on go)

- [x] `ledgerful ledger status --compact` (coordinator)
- [x] `ledgerful ledger start T240-… --category FEATURE --message "…"` → TX `0ee32f70-…`
- [ ] `ledgerful scan --impact` (project.rs, main env, doctor) — coordinator
- [ ] Reconfirm live triangle — operator dogfood

## Phase 1 — Red → Green: identity helpers (M1/M3/O1)

- [x] `GitIdentity` + `collect_git_identity` (replace/wrap `get_git_repo_slug` for detect path)
- [x] `resolve_path_alias_project(path) -> Option<ProjectId>` via normalize + store lookup
- [x] `resolve_detect` path → slug → env + F6 conflict notes (incl. 0-mem extra)
- [x] Units: path wins over slug; 0-mem path note; collect_git_identity non-git None/None; skip-warn pure

## Phase 2 — whoami + detect rewire (M3/L2/L4/L7/L9)

- [x] Capture `shell_project_id` before force-set (`record_shell_project_id` OnceLock)
- [x] Clap `Whoami { format }`; JSON fields pinned; TTY via `IsTerminal`
- [x] whoami under `--no-project-context`: env null, path/detect still run
- [x] Wire detect + `--export` source comments
- [x] Hermetic CLI whoami + detect path prefer (`tests/project_identity_convergence.rs`)

## Phase 3 — Mismatch warn once (M4/M2/L5)

- [x] Once warn after vault can query path alias (`maybe_warn_identity_mismatch` + `Once`)
- [x] Pre-scan skip `--no-project-context` / `--global`
- [x] Never mutate PROJECT_ID; SOOT message + whoami hint

## Phase 4 — Doctor soft + docs (L1/L10/O13)

- [x] Soft `project_identity` via doctor `vault_conn` + env + path (14-check matrix)
- [x] CAPABILITIES + WORKFLOWS + OPERATIONS: triangle diagram + whoami
- [x] Root CHANGELOG
- [x] F13/F14 remain deferred (not shipped)

## Phase 5 — Review + gate + dogfood

- [ ] Internal review vs fold-in pins
- [ ] **Cross-model Codex required** (FEATURE / high UX)
- [ ] Full gate
- [ ] Live: whoami full triangle; detect → 7d97…; operator `.env` rebind → Scope main (AC6)
- [ ] Conductor Completed; deferred; pin DECISION

---

## Explicit non-goals

- Auto-merge / auto-delete projects / auto-write `.env`  
- Memory-count heuristics  
- clap `--global` promote to global arg  
- T241 / T242 / T254  
- Remove is-terminal crate workspace-wide  
- Forced dep bumps  

---

## Evidence log (fill on implement)

| AC | Proof | Result |
|----|-------|--------|
| AC0 triangle | plan-time | PASS |
| AC1 docs = code | CAPABILITIES whoami + detect order; WORKFLOWS §0; OPERATIONS identity SOOT | PASS (docs updated) |
| AC2 whoami fields | `project_whoami__json__fields_present` + `project_whoami__env_differs_path__mismatch_true` | PASS |
| AC3 detect path prefer | `project_detect__path_alias_wins_over_unique_slug` + case normalize | PASS (hermetic); live pending |
| AC4 mismatch once | `project_list__identity_mismatch__warn_on_stderr` + skip under `--no-project-context` | PASS |
| AC5 T206 green | `cargo nextest run -p ai-brains-cli --test project_detect_honesty` (9/9) | PASS |
| AC6 operator rebind | live dogfood | pending operator |
| AC7–AC8 gate + no silent Scope | clippy `-D warnings` + hermetics; no PROJECT_ID mutation in warn | PASS clippy + hermetics; full workspace gate pending coordinator |

### Commands proved (implementer)

```
cargo clippy -p ai-brains-cli --all-targets -- -D warnings   # PASS
cargo nextest run -p ai-brains-cli --test project_detect_honesty --test project_register_path --test project_identity_convergence  # 21 PASS
cargo nextest run -p ai-brains-cli project_   # 96 PASS
cargo nextest run -p ai-brains-cli --test doctor_cli  # 22 PASS
```

---

## Definition of Done

- [ ] F0–F22 + AC0–AC8  
- [x] AI fold-in pins honored (M1–M3, whoami, once warn, doctor soft, export source)  
- [ ] Cross-model clean; full gate; dogfood  
- [ ] conductor / deferred / pin  

---

**Implementation complete pending review / cross-model / operator dogfood.**
