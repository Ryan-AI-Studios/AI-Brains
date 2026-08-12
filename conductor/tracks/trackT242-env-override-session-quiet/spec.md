# T242 — Env override warning session quiet

- **Track ID:** T242-EnvOverrideSessionQuiet
- **Status:** 📋 **Planning** (plan-only until **go**)
- **Category:** UX / POLISH
- **Owner:** Grok
- **Source:** CLI audit 2026-08-11 P1 — warning on nearly every command; T223 residual F18 (once-per-TTY / clap quiet / truthy-core / global-reorder / elevation)
- **Depends on:** T223 collapse multi-key line + `AI_BRAINS_QUIET_ENV_WARN` (shipped PR #126 `7ff8f7f`)
- **Blocks / feeds:** Daily stderr scannability for all scoped commands; agents multi-spawn loops
- **Absorbs:** deferred.md “Env override warn spam (T223 residual)”; T223 F18 soft: once-per-TTY rate limit, clap quiet flag (optional), elevation quiet forward (optional)
- **Not absorbed:** T206 `git/env project mismatch` wording/path; T240 identity mismatch SOOT/behavior (may **share** marker infrastructure only if zero risk); dotenv force-set precedence (F1 frozen); global-before-project reorder as DoD (T223 O1/F1); truthy-parser consolidation to core as DoD; T241 policy; clap 5; daemon dotenv
- **Research date:** 2026-08-12 (live dogfood + main.rs/`env_warn.rs` truth + dotenvy pin + T223/T240 residuals + web)
- **AI fold-in:** 2026-08-12 — `C:\dev\AI-review.md` AI1 + AI2. No Highs. **Hard:** AI2 M1–M3 (smoke home redirect, atomic `create_new`, home resolve inside apply); AI1 M1–M4 shape (sha2+hex fingerprint, decide policy, fail-open, FORCE). **Hard lows:** L1 cwd=.env parent; L4 empty marker; L6 key-match fields; L8 marker only on Stderr; L9 full decision table. **Soft:** L2 manual cleanup docs; L3 single `env_warn_truthy`; L5 AtomicBool; L7 `--no-project-context` pin. **Decline:** AI1 chrono RFC3339 marker payload (F29 / empty file wins). Disposition **§13**.
- **Ledger:** plan-only until go (`ledgerful ledger start T242-env-override-session-quiet --category UX`)

---

## 1. Objective

1. **Stop per-command spam** when shell IDs still differ from local `.env` across **many CLI process spawns** (agent loops, operator multi-command).
2. Keep **first** notice honest: operator still learns local `.env` won for PROJECT (and SESSION when both differ).
3. **Re-warn** when the override *situation* changes (cwd / shell IDs / `.env` IDs fingerprint).
4. Preserve **T223** collapse, session-only demote, and absolute `AI_BRAINS_QUIET_ENV_WARN`.
5. Preserve **precedence / force-set** (presentation + rate limit only).
6. **Capture independence / zero new crates / no repo pollution.**

---

## 2. Live baseline (re-scan 2026-08-12)

### 2.1 Dogfood (this workspace)

| Case | Behavior today (post-T223) |
|------|----------------------------|
| Shell IDs **≠** local `.env` | **One** stderr line **per process** on warn-gated commands |
| 3 sequential `ai-brains project list` | **3** identical Warning lines (confirmed) |
| `AI_BRAINS_QUIET_ENV_WARN=1` in shell | **0** Warning lines (confirmed) |
| Quiet only in global `~/.ai-brains/.env` | Still warns (T223 M1 — global loads after apply) |
| Session-only differ | No stderr (debug deferred) |
| T240 identity mismatch | Separate once-per-**process** line after vault open |

Example (current):

```text
Warning: local .env overrides inherited shell: AI_BRAINS_PROJECT_ID (was 7d97a456-…), AI_BRAINS_SESSION_ID (was 1986908d-…).
```

### 2.2 Why “once-per-process” alone is insufficient

| Myth | Truth |
|------|--------|
| T242 = `std::sync::Once` on emit | `apply_local_project_context_env` already runs **once** per process. Once does not fix spawn-per-command agents. |
| T223 closed the audit finding | T223 closed **dual-line** spam. Audit residual is **cross-process** re-fire. |

T223 F18 already named this: *“once-per-TTY rate limit”* / *“Rate-limiting across processes”* was **out of T223 scope**.

### 2.3 Code truth

| Item | Location | Notes |
|------|----------|--------|
| Pure policy | `crates/ai-brains-cli/src/env_warn.rs` | `quiet_env_warn_truthy`, `format_override_body`, `classify_env_overrides` |
| Emit site | `main.rs` `apply_local_project_context_env` | Collect → force-set → quiet → classify → `eprintln!` or deferred debug |
| Command gate | `should_warn_project_context_override` | preflight/recall/sync/pin/forget/nightly/context/project/safety/antigravity-import/briefing/query |
| Load order | `main_inner` | project gap-fill → apply+emit → global gap-fill → tracing init → deferred debug |
| Smoke | `tests/smoke.rs` `preflight__local_env_project_context_overrides_inherited_shell_ids` | Exactly one Warning prefix per process |
| T240 parallel | `commands/project.rs` `MISMATCH_WARN_ONCE` | Once-per-process identity; **distinct** SOOT |

### 2.4 Dependency pin research (2026-08-12)

| Crate | Workspace / lock | crates.io (2026-08-12) | Action |
|-------|------------------|------------------------|--------|
| `dotenvy` | `"0.15"` → lock **0.15.7** | **0.15.7** latest stable; 0.16 still unpublished on main | **No bump** |
| clap | existing 4.x lock | minor drift only | **No forced bump** (series non-goal) |

### 2.5 External patterns (context)

- Python `warnings` default filter: suppress **repeats** of the same (message, category, …) — re-fire on change.
- CLI agents spawn **one process per tool call** → in-process Once is a no-op for multi-call UX.
- Product already has absolute quiet env (T223); missing is **sticky suppress across spawns with same fingerprint**.
- Marker/cache under **user storage or temp**, never the git worktree (AGENTS: no repo pollution).

---

## 3. Problem analysis

1. **Intentional force-set (keep):** Local `.env` PROJECT_ID/SESSION_ID must beat a stale shell (T80/T223 F1).
2. **Honest first warn (keep):** When PROJECT differs, operator should see one SOOT line **at least once** for a given situation.
3. **UX failure (fix):** Same situation re-prints on every `ai-brains` spawn → agent transcripts and operator loops become unreadable.
4. **Quiet env exists but is opt-in** — agents/operators rarely set it; session quiet should **default** for repeat same-fingerprint without requiring env churn.
5. **Must not hide new situations:** Changing shell PROJECT, `.env` PROJECT, SESSION pair, or cwd must re-warn.

---

## 4. Frozen decisions

| ID | Decision |
|----|----------|
| **F0 — Plan-only** | No production code until user **go**. |
| **F1 — Precedence frozen (hard)** | Do **not** change load order, ID force-set, gap-fill of other keys, global merge, or `--no-project-context`. Only **whether** stderr is emitted for an already-classified Stderr case. |
| **F2 — T223 preserve (hard)** | Collapse SOOT, session-only → Debug, `AI_BRAINS_QUIET_ENV_WARN` absolute suppress, deferred debug after tracing init, T206 distinct — all **unchanged**. |
| **F3 — Cross-process session quiet (hard DoD)** | Marker logic runs **only** when `classify` returns `EnvOverrideEmit::Stderr` (session-only `Debug` / empty `None` **never** consult or write markers — AI2 L8). Atomic claim via **`OpenOptions::create_new(true)`** (AI2 M2): existence = seen. **Reorder:** for Stderr path (after quiet/force pure decide): if not quiet and not force → `create_dir_all` best-effort then `create_new(marker)`; **Ok** → emit Stderr; **AlreadyExists** → demote Debug; other IO err → **fail-open Stderr** (no silent drop). Force → Stderr without requiring create success. Quiet → Debug, no marker write required. |
| **F4 — Fingerprint definition (hard)** | Stable SHA-256 over fixed concat (workspace **`sha2` 0.11** + **`hex` 0.4** — AI1 M1; F29 satisfied): `normalized_cwd\|shell_p\|shell_s\|env_p\|env_s` with empty string for missing. **cwd** = `normalize_for_location_compare` of **`.env` parent dir** (`path.parent()`, not process cwd alone — AI2 L1). **shell_p/shell_s** = pre-force differ-gate olds. **env_p/env_s** = post-force / `.env` values extracted **by key match** (PROJECT then SESSION), not `to_set` vector order (AI2 L6/F24). Hex digest: full 32-byte SHA-256 as lowercase hex (**64 chars**) for simplicity (truncation optional, not required). Marker **filename = hex**; **content = empty (0 bytes)** — existence is the signal (AI2 L4). **Decline** chrono/RFC3339 payload (would pull time into this path; F29). No vault KEY. |
| **F5 — Marker location + home resolve (hard)** | Path: `{home}/.ai-brains/cache/env-override-warn/<fingerprint_hex>`. Home via **`resolve_user_home_for_dotenv()` called inside `apply_local_project_context_env`** (AI2 M3) — same USERPROFILE → HOME → `dirs::home_dir` SOOT as global dotenv. Global dotenv still resolves home later (double call is cheap). **Never** write into the git repo. Fail-open: `home == None` or cache unusable → still Stderr. |
| **F6 — Re-warn on change (hard)** | Different fingerprint → new marker path → warn again. Same fingerprint → suppress after first successful `create_new`. Deleting markers / whole cache dir restores first-warn. **No auto-cleanup / TTL in T242** (AI2 L2); document manual `Remove-Item -Recurse …\env-override-warn`. Soft residual for future TTL sweep. |
| **F7 — Force always-warn (hard)** | `AI_BRAINS_FORCE_ENV_WARN=1`/`true`/`yes` via shared **`env_warn_truthy`** (rename/alias of `quiet_env_warn_truthy` — AI2 L3; not a third independent parser). Precedence: **quiet > force > atomic marker > first warn**. Quiet wins over force (CI silence). Force ignores marker presence. |
| **F8 — Absolute quiet still wins (hard)** | `AI_BRAINS_QUIET_ENV_WARN` truthy → Debug only; no marker write required. Source visibility unchanged (shell/project at apply; global alone too late). |
| **F9 — Once-per-process belt (defensive)** | `AtomicBool` (or `Once`) so a single process never double-`eprintln!` if apply were re-entered (AI2 L5). Current call graph is once; keep low-cost. |
| **F10 — Command gate unchanged (hard)** | Keep `should_warn_project_context_override` list as-is for DoD. Expanding list = soft residual. |
| **F11 — Message SOOT frozen (hard)** | No wording change to T223 stderr/debug templates. |
| **F12 — T206 / T240 separate (hard)** | Do not merge identity mismatch into this warn. Identity remains once-per-process. Optional shared cache dir later — not DoD. |
| **F13 — Pure helpers (hard)** | `env_warn.rs` stays pure: fingerprint + `decide_env_override_emit` + truthy. Marker IO in thin wrapper / `env_warn_session.rs` with injectable home root for hermetics. |
| **F14 — Hermetic isolation (hard)** | **All** tests that exercise override warn (including existing smoke) **must** `common::isolate_empty_home` (or equivalent temp home) so markers never touch operator `~/.ai-brains` (AI2 M1 — resolves F14/F15 contradiction). Multi-spawn under same temp home: second process 0 Warnings; fingerprint change re-warns. |
| **F15 — Smoke migration (hard)** | Update `preflight__local_env_project_context_overrides_inherited_shell_ids` to **redirect home** + assert exactly **one** Warning under **fresh** temp home. Optional assert marker file exists under temp home. Add multi-spawn suite for AC1–AC5. `hermetic_bin` alone is **not** enough (no home redirect today). |
| **F16 — Clap flag (soft residual)** | Global `--quiet-env-warn` not DoD. |
| **F17 — Elevation quiet (soft)** | Add `AI_BRAINS_QUIET_ENV_WARN` (+ optional FORCE) to `ELEVATE_ENV_KEYS` if trivial — soft residual (AI2 O12: low risk). |
| **F18 — Truthy → core (soft)** | Workspace-wide consolidate still soft residual; T242 only unifies quiet+force in `env_warn` via one function. |
| **F19 — Global quiet pre-read / reorder (soft / decline DoD)** | Spike only; document global-only quiet still too late. |
| **F20 — No default-quiet-all-non-TTY (hard decline)** | Agents still get one honest first line (AI2 O13). |
| **F21 — Exit codes / contracts (hard)** | Unchanged; no DTO change. |
| **F22 — Secrets (hard)** | UUIDs only as hash inputs; marker path hash-only; never KEY. |
| **F23 — Docs (hard)** | CAPABILITIES + OPERATIONS: session quiet, FORCE, atomic marker, manual cache reset, quiet wins, global-only quiet late. CHANGELOG T242. |
| **F24 — Determinism (hard)** | Fingerprint field order fixed: cwd, shell_p, shell_s, env_p, env_s; key-match extract; no wall-clock. |
| **F25 — High findings if…** | Same fingerprint still warns every **sequential** process after first claim; quiet broken; first-ever never warns when home+cache ok; force-set broken; T206/T240 SOOT edited by accident; markers in repo; production unwrap/expect; dual-line **legacy template** return. Concurrent race mitigated by `create_new` (not residual). |
| **F26 — Parallel-friendly** | Touches `env_warn` + `main.rs` apply + hermetic tests + docs. |
| **F27 — Ledger** | On go: `ledgerful ledger start T242-env-override-session-quiet --category UX`. |
| **F28 — Review** | UX primary. Cross-model **soft** (AI2 O11). |
| **F29 — Zero new crates (hard)** | Use workspace `sha2` + `hex` only. No chrono, no flock, no BLAKE3 add. |
| **F30 — `--no-project-context` (hard pin)** | Apply not called → no marker check/write/warn (AI2 L7). Unchanged. |
| **F31 — Body extraction on demote (hard)** | Quiet/seen demote Stderr→Debug uses body **without** `Warning: ` prefix (strip or keep body side-by-side in classify — AI2 L9). Debug SOOT remains T223 F3. |

---

## 5. Acceptance criteria

| AC | Criterion | Proof |
|----|-----------|-------|
| **AC1** | First process with PROJECT differ + empty marker → exactly one stderr Warning (T223 SOOT) + **0-byte** marker under redirected home (`create_new` won) | Hermetic multi-spawn |
| **AC2** | Second process same fingerprint + same home → **0** stderr Warning lines; force-set still applied (Scope still local project) | Hermetic |
| **AC3** | Change shell PROJECT (or `.env` PROJECT / `.env` parent location) → warn again | Hermetic |
| **AC4** | `AI_BRAINS_QUIET_ENV_WARN=1` → 0 stderr even with no marker; force does not override quiet | Unit + optional hermetic |
| **AC5** | `AI_BRAINS_FORCE_ENV_WARN=1` without quiet → stderr even if marker exists | Unit + hermetic |
| **AC6** | Session-only override still no stderr and **no marker file** written | Unit + hermetic optional |
| **AC7** | Both keys differ → still **one** collapsed line (not dual legacy) | Smoke + unit |
| **AC8** | Marker fail-open: home None / unwritable cache → still stderr (no silent drop) | Unit / hermetic |
| **AC9** | No marker files under repo worktree; tests never write operator real home (smoke uses `isolate_empty_home`) | Path assert / home redirect |
| **AC10** | T206 `git/env project mismatch` string untouched; T240 identity mismatch SOOT untouched | Grep / no edit |
| **AC11** | CAPABILITIES + OPERATIONS + CHANGELOG: session quiet, FORCE, manual cache reset, quiet wins, global-only quiet late | Docs gate |
| **AC12** | Full CI gate green; no production unwrap/expect | Gate |
| **AC13** | Manual dogfood: N sequential → **1** Warning; quiet → 0; force → N; after cache delete → 1 again | plan.md evidence |
| **AC14** | dotenvy **0.15.7**; sha2/hex only (no new crates); no clap forced bump | Lock / tree |
| **AC15** | Existing smoke updated with home redirect still passes on **second** nextest run (no sticky operator marker) | CI twice / local re-run |
| **AC16** | Pure units: quiet>force; force>seen; seen→Debug; unseen→Stderr; fingerprint stable/differs | `env_warn` units |

---

## 6. Out of scope

- Changing shell vs `.env` precedence for models/KEY/VAULT_PATH  
- Removing project-context force for IDs  
- T206 detect implementation  
- T240 whoami / mismatch once-per-process redesign (session marker for identity = soft residual later)  
- Daemon / `ai-brainsd` dotenv  
- Defaulting all non-TTY to quiet without first warn  
- Global-before-project dotenv reorder (DoD)  
- Truthy consolidation to `ai-brains-core` (DoD)  
- clap 5 / MSI / packaging  
- Auto-writing project `.env` or aligning shell env  

---

## 7. Implementation sketch (on go)

### 7.1 Decision table (AI2 L9 — hard pin)

| classified | quiet | force | marker claim | result |
|------------|-------|-------|--------------|--------|
| `None` | * | * | skip | Silent (no emit) |
| `Debug(body)` | * | * | skip | Debug(body) — session-only; **no marker** |
| `Stderr(line)` | true | * | skip | Debug(body) — quiet wins |
| `Stderr(line)` | false | true | skip claim | Stderr(line) — force |
| `Stderr(line)` | false | false | `create_new` Ok | Stderr(line) |
| `Stderr(line)` | false | false | AlreadyExists | Debug(body) |
| `Stderr(line)` | false | false | other IO / no home | Stderr(line) fail-open |

Body for Debug demote: strip `Warning: ` prefix or keep body from `format_override_body`.

### 7.2 Pure + IO

```rust
// env_warn.rs — pure
pub struct EnvOverrideFingerprint<'a> {
    pub normalized_cwd: &'a str,       // .env parent, location-normalized
    pub old_shell_project: Option<&'a str>,
    pub old_shell_session: Option<&'a str>,
    pub new_env_project: Option<&'a str>,
    pub new_env_session: Option<&'a str>,
}
pub fn compute_fingerprint_hex(fp: &EnvOverrideFingerprint<'_>) -> String; // sha2+hex

/// Shared truthy for QUIET and FORCE keys (rename quiet_env_warn_truthy → env_warn_truthy).
pub fn env_warn_truthy(raw: Option<&str>) -> bool;

pub struct EnvWarnPolicy { pub quiet: bool, pub force: bool }

/// Pure classify×policy without marker. Marker claim applied at call site for Stderr+!quiet+!force.
pub fn decide_env_override_emit(
    classified: Option<EnvOverrideEmit>,
    policy: EnvWarnPolicy,
) -> Option<EnvOverrideEmit>; // None→None; Debug→Debug; Stderr+quiet→Debug(body); force→Stderr; else Stderr (caller claims)

// env_warn_session.rs or main-adjacent — IO
fn marker_path(home: &Path, hex: &str) -> PathBuf;
/// create_dir_all + OpenOptions::new().create_new(true).write(true).open(path)
/// Ok → Claimed; AlreadyExists → Exists; other → IoFail (fail-open warn)
enum MarkerClaim { Claimed, Exists, IoFail }
fn try_claim_marker(home: &Path, hex: &str) -> MarkerClaim;
```

### 7.3 `apply_local_project_context_env` wire

1. Collect differ pairs + force-set (unchanged).  
2. Classify overrides (T223).  
3. If not Stderr path after quiet/force pure decide → emit Debug/None; **return** (no marker).  
4. `home = resolve_user_home_for_dotenv()` (inside apply — AI2 M3).  
5. Fingerprint from `.env` parent + olds + news (key match).  
6. If force → eprintln Stderr (no claim required).  
7. Else if home Some → `try_claim_marker`; Claimed → Stderr; Exists → Debug; IoFail → Stderr.  
8. Else (no home) → Stderr fail-open.  
9. Process-local AtomicBool belt after emit.

---

## 8. Verification plan

| Phase | Check |
|-------|--------|
| Red | Pure units: quiet/force/seen matrix; fingerprint stability; SOOT unchanged |
| Green | Wire apply + marker IO |
| Hermetic | Multi-spawn same home; fingerprint change; quiet; force |
| Smoke | Single-process count==1 preserved |
| Manual | Live multi-command + cache delete + quiet + force |
| Gate | fmt; clippy `-D warnings`; nextest; deny; audit; `ledgerful verify` |
| Review | `conductor/tracks/trackT242-env-override-session-quiet/review.md` |

---

## 9. Touch map

| Path | Change |
|------|--------|
| `crates/ai-brains-cli/src/env_warn.rs` | Fingerprint + decide policy + units |
| Optional `env_warn_session.rs` | Marker IO if env_warn should stay pure |
| `crates/ai-brains-cli/src/main.rs` | Wire decide + marker at apply |
| `crates/ai-brains-cli/tests/smoke.rs` and/or new `env_override_session_quiet.rs` | Multi-spawn hermetics |
| `crates/ai-brains-cli/tests/common/mod.rs` | Home redirect helpers if missing |
| `Docs/CAPABILITIES.md` | Session quiet + force rows |
| `Docs/OPERATIONS.md` | Env table FORCE + session marker |
| `CHANGELOG.md` | T242 entry |
| `conductor/conductor.md` | Planning → Completed on ship |
| `conductor/deferred.md` | Close T242 residual row |
| `conductor/tracks/README-T240-T255-CLI-EFFECTIVENESS.md` | Mark T242 closed on ship |

---

## 10. Absorbed deferred / series

| Item | Disposition |
|------|-------------|
| deferred.md “Env override warn spam (T223 residual)” | **Absorbed** — close on ship |
| T223 F18 once-per-TTY rate limit | **Absorbed** as F3 session fingerprint marker |
| T223 F18 clap `--quiet-env-warn` | **Soft residual** F16 |
| T223 F18 elevation quiet handoff | **Soft residual** F17 |
| T223 F18 truthy → core | **Soft residual** F18 |
| T223 F18 global reorder / pre-read | **Soft residual** F19 (not DoD) |
| T240 warn spam coordinate | **Not absorbed** — identity remains once/process; optional shared cache dir later |
| Series README T242 P1 | This track |

---

## 11. Risks

| Risk | Mitigation |
|------|------------|
| Operators miss first warn after cache pollution | Fingerprint change re-warns; force env; delete cache; quiet separate |
| Shared CI home causes cross-job suppress | Hermetic redirect; CI should set QUIET or isolated USERPROFILE (document) |
| Existing smoke sticky-breaks on re-run | F15 + AC15: `isolate_empty_home` on override smoke (AI2 M1) |
| Parallel agent double-warn | Atomic `create_new` claim (AI2 M2) |
| Home not available at apply | Resolve home **inside** apply (AI2 M3) |
| Marker becomes tracking surface | Hash-only filenames; user storage only; no KEY |
| Fail-closed hide | Fail-open Stderr on IoFail / no home |
| Scope regression | AC2 force-set still wins; no precedence edits |

---

## 12. Definition of Done

- [ ] F0–F31 decisions honored (soft F16–F19 not required)
- [ ] AC1–AC16 green
- [ ] Manual AC13 recorded in plan.md
- [ ] deferred.md residual closed; conductor Completed
- [ ] Full gate + review clean (or mediums justified ≤3)
- [ ] Pin: `ai-brains pin "DECISION: T242 …"`

---

## 13. AI fold-in disposition (`C:\dev\AI-review.md`)

| ID | Source | Verdict | Action |
|----|--------|---------|--------|
| **AI1 M1** sha2+hex fingerprint struct | AI1 | **Agree hard** | F4 `compute_fingerprint_hex` |
| **AI1 M2** `decide_env_override_emit` | AI1 | **Agree hard** | F13 + §7.1 table |
| **AI1 M3** fail-open marker + chrono RFC3339 | AI1 | **Partial** | Fail-open **yes**; chrono timestamp **decline** (AI2 L4 empty file) |
| **AI1 M4** FORCE_ENV_WARN | AI1 | **Agree hard** | F7 (already planned) |
| **AI1 L1 / O1** docs + unit names | AI1 | **Agree** | F23 + AC16 |
| **AI2 M1** smoke breaks without home redirect | AI2 | **Agree hard** | F14/F15/AC15 |
| **AI2 M2** concurrent race → `create_new` | AI2 | **Agree hard** | F3 atomic claim reorder |
| **AI2 M3** home after apply | AI2 | **Agree hard** | F5 resolve inside apply |
| **AI2 L1** cwd = `.env` parent | AI2 | **Agree hard** | F4 |
| **AI2 L2** no auto-cleanup | AI2 | **Agree** | F6 + ops docs |
| **AI2 L3** single `env_warn_truthy` | AI2 | **Agree** | F7 rename/alias |
| **AI2 L4** 0-byte marker | AI2 | **Agree hard** | F4 |
| **AI2 L5** AtomicBool belt | AI2 | **Agree soft** | F9 keep |
| **AI2 L6** key-match env fields | AI2 | **Agree hard** | F4/F24 |
| **AI2 L7** `--no-project-context` | AI2 | **Agree** | F30 pin |
| **AI2 L8** marker only on Stderr | AI2 | **Agree hard** | F3 |
| **AI2 L9** full decision table | AI2 | **Agree hard** | §7.1 + F31 |
| **AI2 L10** smoke home safe | AI2 | **Agree** | F15 note |
| **AI2 O11–O13** soft CX / F17 / F20 | AI2 | **Agree** | F28/F17/F20 |

**Pins locked by fold-in (implementer checklist):**

1. Fingerprint = SHA-256 hex of `norm_cwd|shell_p|shell_s|env_p|env_s`; cwd = normalize(`.env` parent).  
2. Atomic marker: `create_new` claim; Exists → Debug; IoFail/no home → Stderr.  
3. Home resolved **inside** apply via `resolve_user_home_for_dotenv()`.  
4. Existing override smoke **must** `isolate_empty_home`.  
5. Marker only when classify = Stderr; 0-byte files; no chrono.  
6. One `env_warn_truthy` for QUIET + FORCE; quiet > force > claim.

---

**Plan-only until go.** Say **go T242** to implement.
