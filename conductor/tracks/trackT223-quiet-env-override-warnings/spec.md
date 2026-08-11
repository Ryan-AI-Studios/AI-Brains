# T223 — Quiet env override warnings

- **Track ID:** T223-QuietEnvOverrideWarnings
- **Phase:** T217–T232 post-audit CLI quality (P2)
- **Status:** 📋 **Planning** (plan-only until go)
- **Depends on:** T113 / T139 / T205 dotenv layers; T80 project-context force; T206 git/env mismatch (must stay distinct)
- **Blocks / feeds:** Agent/operator stderr signal-to-noise; soft residual for optional clap flag
- **Category:** UX
- **Source:** Non-destructive CLI audit 2026-08-05 — dual “local .env overrides inherited shell value” spam on nearly every scoped command
- **Deferred absorbed:** deferred.md “`.env` override double-warn spam” → this track DoD
- **Not absorbed:** T206 git/env project mismatch wording or behavior; dotenv precedence (shell vs project for non-ID keys); T225 backup quiet; clap 5; daemon dotenv path; MSI
- **Research date:** 2026-08-10 (live dogfood + main.rs truth + dotenvy 0.15.7 pin check + T113/T205 history)
- **AI fold-in:** 2026-08-10 — AI1 affirms F1–F24/AC1–AC12 (no new criticals; blind spot 3 on global quiet **corrected by AI2 M1**). AI2 **M1–M4 hard**; **L3/L4/L6** folded; **O3/O4** hard-ish; **O1** reject as DoD (F1); **O2/O5** soft. Disposition **§13**.
- **Ledger:** plan-only until go (`ledgerful ledger start T223-quiet-env-override-warnings --category UX`)

## 1. Objective

1. **Reduce stderr spam** when local `.env` intentionally overrides inherited `AI_BRAINS_PROJECT_ID` / `AI_BRAINS_SESSION_ID` without hiding **real** project-identity conflicts.  
2. **Collapse** multi-key overrides into **one** warning line.  
3. **Opt-out suppress** for agents/CI via `AI_BRAINS_QUIET_ENV_WARN`.  
4. **Demote session-only** overrides (high-churn, low-signal) to `tracing::debug`.  
5. **Keep precedence frozen** — only presentation / emit policy changes.  
6. **Capture independence / zero new crates.**

## 2. Live baseline (re-scan 2026-08-10)

### 2.1 Dogfood (this workspace)

| Case | Behavior today |
|------|----------------|
| Shell IDs **≠** local `.env` IDs | **Two** stderr lines every command in `should_warn` list (e.g. `preflight`, `recall`) |
| Shell IDs **=** local `.env` | **Silent** (already `existing != value`) |
| Agent multi-command loop | Warning pair reappears **per process** (spawn-per-command) — not once-per-shell |
| Equal-path recheck after aligning env to `.env` | No `Warning: local` lines |

Example (current):

```text
Warning: local .env AI_BRAINS_PROJECT_ID overrides inherited shell value aaaaaaaa-….
Warning: local .env AI_BRAINS_SESSION_ID overrides inherited shell value bbbbbbbb-….
```

### 2.2 Code truth

| Item | Location | Notes |
|------|----------|--------|
| Emit site | `crates/ai-brains-cli/src/main.rs` `apply_local_project_context_env` ~1899–1945 | Per-key `eprintln!` when `warn_on_override && existing != value` |
| Command gate | `should_warn_project_context_override` | `preflight\|recall\|sync\|pin\|forget\|nightly\|context\|project\|safety\|antigravity-import\|briefing\|query` |
| Non-warn path | same fn, `warn_on_override == false` | `tracing::debug!` only |
| Load order (code truth) | `main_inner` ~2001–2038 | (1) `dotenvy::dotenv()` project gap-fill (all keys) → (2) `apply_local_project_context_env` force IDs + **emit** → (3) global `~/.ai-brains/.env` gap-fill. **Quiet is only visible at emit if already in process env** (shell or project `.env` at step 1). Global quiet at step 3 is **too late** (M1). Do **not** cite stale T113 `dotenv_override()` text — code uses gap-fill `dotenv()` (L5). |
| Hermetic lock | `tests/smoke.rs` `preflight__local_env_project_context_overrides_inherited_shell_ids` | Asserts **both** substring warnings today |
| Quiet env today | **None** for this warn | No `AI_BRAINS_QUIET_ENV_WARN` |
| T206 distinct | `commands/project.rs` `env_fallback_warning` | Prefix `git/env project mismatch` + set-alias hint — **must not** merge |

### 2.3 Dependency pin research

| Crate | Workspace / lock | crates.io (2026-08-10) | Action |
|-------|------------------|------------------------|--------|
| `dotenvy` | workspace `"0.15"` → lock **0.15.7** | **0.15.7** latest stable; 0.16 unpublished on main | **No bump** this track |

Truthy suppress must match product convention elsewhere: `1` / `true` / `yes` (case-insensitive, trim) — same set as HTTP / zero-key / retention flags.

## 3. Problem analysis (why dual spam exists)

1. **Intentional project bind (keep):** Local `.env` **must** win for `AI_BRAINS_PROJECT_ID` / `AI_BRAINS_SESSION_ID` so cwd project context beats a stale shell (T80 / post-T113 special case). General keys still follow shell > project > global gap-fill (T113/T205).  
2. **Honest warn (keep in principle):** When shell had a **different** value, tell the operator once that local `.env` took over.  
3. **UX failure:** Two nearly identical lines on every scoped command; agent sessions often inherit both IDs from a prior session → constant dual noise. Session ID churn is **expected**; project ID override is the high-signal half.  
4. **Draft placeholder gap:** “Only warn when values differ and both set” is **already** true. This track is **emit policy + format**, not a new differ-gate.

## 4. Frozen decisions

| ID | Decision |
|----|----------|
| **F1 — Precedence frozen (hard)** | Do **not** change load order, force-set of PROJECT_ID/SESSION_ID, gap-fill of other keys, global merge, or `--no-project-context`. Only **whether / how** we print override notices. |
| **F2 — Collapse (hard)** | When ≥1 keys will stderr-warn, emit **exactly one** line listing all stderr-worthy keys (and their previous shell values). Never two independent `Warning:` lines for the same apply pass. |
| **F3 — Message shape (hard)** | **Stderr SOOT** (exact for pure `assert_eq!` — O4): `Warning: local .env overrides inherited shell: AI_BRAINS_PROJECT_ID (was {old})[, AI_BRAINS_SESSION_ID (was {old})].` Key order stable: PROJECT_ID then SESSION_ID when both present. Distinct from T206 `git/env project mismatch`. **Debug SOOT (M3 hard):** same body **without** the `Warning: ` prefix — one collapsed line listing the keys that were collected (session-only → session key only; quiet with both → both keys). Do **not** keep dual per-key debug lines after collapse. Wording: `local .env overrides inherited shell: AI_BRAINS_PROJECT_ID (was {old})[, …].` |
| **F4 — Quiet suppress (hard)** | If `AI_BRAINS_QUIET_ENV_WARN` is truthy (`1`/`true`/`yes`, case-insensitive, trim) → **no** `eprintln`; use collapsed `tracing::debug!` (F3 debug SOOT). Invalid/empty/unset → normal policy. **Source visibility (M1 hard):** Quiet is honored only when the flag is already in the **process environment at apply time** — i.e. **shell env** or **project-local `.env`** (gap-filled by `dotenvy::dotenv()` **before** apply). **Global `~/.ai-brains/.env` loads after apply**, so quiet set **only** there will **not** suppress this warning — document explicitly in OPERATIONS env table + CAPABILITIES (no silent operator trap). **Do not** reorder global merge before project (would violate F1 / T113–T205 — O1 soft residual only). **Do not** special-case pre-read of global quiet in DoD. Elevation handoff does **not** forward this flag (`ELEVATE_ENV_KEYS` excludes it) — elevated child may still warn; **out of scope** (L3). |
| **F5 — Session-only demote (hard)** | If **only** `AI_BRAINS_SESSION_ID` would override (PROJECT_ID not overriding: missing shell, equal, or not in `.env`) → **no** stderr; one collapsed `tracing::debug!` with F3 debug SOOT for SESSION only. Rationale: session rotation is normal; dual-key agent noise is mostly session + project. Note: default `DEFAULT_ENV_FILTER` hides `debug!` — session demote is **effectively silent** unless operator raises `RUST_LOG`. |
| **F6 — Project still warns (hard)** | If `AI_BRAINS_PROJECT_ID` differs → stderr (unless quiet). If both differ → one collapsed line including **both** keys (session rides along for honesty). |
| **F7 — Differ gate unchanged** | Still only consider override when shell var is set **and** value ≠ `.env` value. Equal → silent (existing). |
| **F8 — Command gate unchanged** | Keep `should_warn_project_context_override` list as-is unless a review finds an obvious miss; expanding the list is **soft residual**, not DoD. |
| **F9 — Pure helpers (hard)** | Extract pure, unit-testable helpers (**CLI-local**, e.g. `env_warn.rs` or main-adjacent) for: truthy quiet parse; classify/format from collected `(key, old)` pairs. No env mutation inside pure formatters. Prefer pure Red units + `assert_eq!` on full SOOT (O4). **Truthy 7th copy (M2):** CLI-local `quiet_env_warn_truthy` is intentional this track; product already has 6+ independent `1/true/yes` parsers — consolidation to `ai-brains-core` is **F18 soft residual**, not DoD. |
| **F10 — Smoke migration (hard)** | Update `preflight__local_env_project_context_overrides_inherited_shell_ids`: assert **full prefix** `Warning: local .env overrides inherited shell:` (L6 — not bare `local .env`); both keys + old values when both differ; **exactly one** match of that prefix (O3 count guard); **not** legacy dual template `local .env AI_BRAINS_* overrides inherited shell value`. Quiet + session-only: pure units sufficient if hermetic cost high. |
| **F11 — No clap flag in DoD** | Soft residual only: global `--quiet-env-warn`. Env quiet sources: **shell** or **project `.env`** (M1). Do **not** document global `.env` as a working quiet source without also shipping a load-order/pre-read change (out). |
| **F12 — T206 separate (hard)** | Do not touch `env_fallback_warning` / project detect. Distinct prefixes. **AC12 / Phase 5:** manual `project detect` with shell/`.env` PROJECT_ID mismatch **and** git slug mismatch → **both** warnings co-occur, no merged wording (M4). |
| **F13 — Secrets / values** | Continue printing previous shell **UUID values** in the warn line (already public-ish IDs, not vault keys). Do **not** print KEY material (this path never loads KEY into the warn). |
| **F14 — Exit codes** | Unchanged (warnings never flip exit). |
| **F15 — Contracts** | No DTO change. |
| **F16 — dotenvy** | Stay on **0.15.7**; no API migration. |
| **F17 — High findings if…** | Dual stderr lines for one apply; quiet ignored when truthy; session-only still eprintln; PROJECT_ID override silenced without quiet; T206 text/path edited “by accident”; precedence / force-set broken; smoke still requires two independent lines. |
| **F18 — Soft residuals** | Clap `--quiet-env-warn`; once-per-TTY rate limit; expand `should_warn` list; wire suppress into command-local `--quiet` (**out**); **promote truthy parser to `ai-brains-core` and consolidate 6+ copies** (`is_http_env_truthy`, `env_truthy`, zero-key, etc.) — M2/O2; **spike: reorder global merge before project apply** so quiet works from global `.env` (O1 — needs T113/T205 regression; may conflict F1); special-case pre-read of quiet from global file without full reorder; elevation handoff add quiet to `ELEVATE_ENV_KEYS` (L3); drive-by comment at `dotenvy::dotenv()` gap-fill evolution (O5). |
| **F19 — Parallel-friendly** | Touches `main.rs` apply path + smoke + docs; low conflict with T225–T231 if they avoid dotenv startup. |
| **F20 — Plan-only** | No production code until user **go**. |
| **F21 — Ledger** | On go: `ledgerful ledger start T223-quiet-env-override-warnings --category UX`. |
| **F22 — Review** | UX primary. Cross-model soft (small presentation track). |
| **F23 — Docs (L4 hard pin)** | **CAPABILITIES §5** (Project, session & context) — primary home for override-warn UX (collapse + session-only demote + quiet). Cross-link **§14** hierarchy only if needed (do not bury quiet solely under §14). **OPERATIONS** env table: `AI_BRAINS_QUIET_ENV_WARN` + **M1 source limitation** (shell / project `.env` only; not global alone). CHANGELOG T223 entry. Optional skill one-liner soft. |
| **F24 — Determinism** | Pure formatters; fixed key order; no timestamps. |
| **F25 — Helper shape (L1 soft)** | Prefer short name e.g. `EnvOverrideEmit { None, Debug(String), Stderr(String) }` or `Option`+kind; long `ProjectContextOverrideEmit` optional. Non-blocking. |

## 5. Acceptance criteria

| AC | Criterion | Proof |
|----|-----------|-------|
| **AC1** | Both PROJECT_ID and SESSION_ID differ → **exactly one** stderr `Warning:` line containing both keys and both old values | Pure unit on formatter + migrated smoke |
| **AC2** | Only PROJECT_ID differs → one stderr line with PROJECT_ID only (no dual-line SESSION noise) | Unit |
| **AC3** | Only SESSION_ID differs → **no** stderr `Warning: local .env`; debug path only | Unit (+ optional hermetic) |
| **AC4** | Equal shell vs `.env` → no override warn (regression) | Unit / existing equal path |
| **AC5** | `AI_BRAINS_QUIET_ENV_WARN=1` (and true/yes case-insensitive) in process env at apply → no stderr override warn even when PROJECT_ID differs | Unit on truthy + apply policy |
| **AC6** | Quiet unset/empty/0/no → normal policy | Unit |
| **AC7** | Precedence / force-set unchanged: local `.env` still wins PROJECT_ID/SESSION_ID; preflight Scope still local project | Existing smoke Scope asserts + code freeze on set_var path |
| **AC8** | T206 mismatch string still `git/env project mismatch` (untouched) | Grep / no edit project.rs warn |
| **AC9** | Message never uses two separate legacy lines (`local .env AI_BRAINS_* overrides inherited shell value`); smoke: exactly **one** `Warning: local .env overrides inherited shell:` when both differ | Unit + smoke (O3) |
| **AC10** | CAPABILITIES **§5** + OPERATIONS (quiet row + **M1 shell/project-only** honesty) + CHANGELOG document collapse, quiet, session-only demote | Docs gate |
| **AC11** | Full CI gate green | fmt/clippy/nextest/deny/audit |
| **AC12** | Manual dogfood: dual-differ → one line; session-only silent; quiet (shell) silent; equal silent; **`project detect` with shell/`.env` PROJECT mismatch + git slug mismatch → T223 + T206 both present, distinct prefixes** (M4) | plan.md evidence |
| **AC13** | Debug path emits **one** collapsed F3 debug line (not dual per-key) for quiet / session-only / non-warn-command with collected overrides | Pure unit on debug body formatter |

## 6. Out of scope

- Changing shell vs `.env` precedence for models/KEY/VAULT_PATH  
- Removing project-context force for IDs  
- T206 detect / set-alias **implementation** (manual co-occur check only)  
- Daemon / service dotenv (`ai-brainsd`)  
- Rate-limiting across processes  
- Clap global quiet flag (soft residual)  
- Reordering global `.env` before project apply (O1 / F1)  
- Global-only quiet pre-read special case  
- Forwarding `AI_BRAINS_QUIET_ENV_WARN` via elevation handoff (L3)  
- Consolidating product-wide truthy parsers into core (M2/O2 soft)  
- Editing legacy T113 pending status text (historical; code already evolved)  

## 7. Implementation sketch (on go)

```rust
// Pure — truthy SOOT (1/true/yes); CLI-local 7th copy OK (M2 → F18)
fn quiet_env_warn_truthy(raw: Option<&str>) -> bool { … }

// Pure body shared by stderr + debug (keys ordered PROJECT then SESSION)
// "local .env overrides inherited shell: AI_BRAINS_PROJECT_ID (was {old})[, …]."
fn format_override_body(overrides: &[(&str, &str)]) -> String { … }

// Classify: empty → None; session-only → Debug(body); else Stderr("Warning: " + body)
// Quiet / !warn_on_override applied at call site → always Debug(body) when non-empty
enum EnvOverrideEmit {
    Debug(String),
    Stderr(String),
}

fn classify_env_overrides(overrides: &[(&str, &str)]) -> Option<EnvOverrideEmit> { … }
```

`apply_local_project_context_env`:

1. Collect differing PROJECT_ID / SESSION_ID overrides (same as today).  
2. Always `set_var` (precedence frozen).  
3. Read quiet: `quiet_env_warn_truthy(std::env::var("AI_BRAINS_QUIET_ENV_WARN").ok().as_deref())`.  
4. Classify; if `!warn_on_override` or quiet → force Debug path.  
5. Emit 0 or 1 `eprintln!` / one `tracing::debug!`.

## 8. Verification plan

| Phase | Check |
|-------|--------|
| Red | Pure units AC1–AC6, AC9 |
| Green | Wire apply path |
| Hermetic | Migrate smoke dual-assert → collapsed; optional quiet hermetic |
| Manual | Shell differ both / session-only / quiet / equal |
| Gate | `cargo fmt --check`; clippy `-D warnings`; nextest workspace; deny; audit; `ledgerful verify` |
| Review | `conductor/tracks/trackT223-quiet-env-override-warnings/review.md` |

## 9. Touch map

| Path | Change |
|------|--------|
| `crates/ai-brains-cli/src/main.rs` | Collect + pure emit policy; collapsed eprintln |
| Optional `crates/ai-brains-cli/src/env_warn.rs` (or similar) | Pure helpers + unit tests if main.rs units awkward |
| `crates/ai-brains-cli/tests/smoke.rs` | Migrate override assert |
| `Docs/CAPABILITIES.md` | **§5** override-warn UX; optional §14 cross-link |
| `Docs/OPERATIONS.md` | Env table: quiet + M1 shell/project-only (not global alone) |
| `CHANGELOG.md` | T223 entry |
| `conductor/conductor.md` | Planning → Completed on ship |
| `conductor/deferred.md` | Close double-warn row |
| `conductor/tracks/README-T217-T232-CLI-QUALITY.md` | Mark T223 closed on ship |

## 10. Absorbed deferred / series

| Item | Disposition |
|------|-------------|
| deferred.md “`.env` override double-warn spam” | **Absorbed** — close on ship |
| README series T223 P2 | This track |
| T206 dual-warning UX (F35) | **Not absorbed** — already closed; keep distinct |
| T225 backup quiet | Peer — do not fold |

## 11. Risks

| Risk | Mitigation |
|------|------------|
| Operators miss session override | Session still force-applied; debug under `RUST_LOG`; both listed when project also differs |
| Quiet env typo | Truthy SOOT documented; case-insensitive |
| Quiet only in global `.env` (M1) | Document limitation; do not claim global works |
| Smoke fragility | Full F3 prefix + count==1 + no legacy dual template |
| Over-collapse hides which value won | Line includes `was {old}`; new values visible via Scope / env |
| `project detect` dual warn confusion | M4 manual: distinct T223 vs T206 prefixes |

## 12. Stop-before

Halt and ask if go would require changing force-set semantics, silencing PROJECT_ID without quiet, merging T206 messages, or reordering global dotenv before project apply (F1).

## 13. AI fold-in disposition (2026-08-10)

| ID | Source | Disposition |
|----|--------|-------------|
| — | AI1 exec + AC table | **Affirm** — restates F1–F24 / AC1–AC12; no new criticals |
| AI1 §3.1–3.2 | smoke migrate + pure helpers | **Already plan** — F9/F10 |
| AI1 §3.3 | “global quiet works via dotenv before apply” | **Reject / correct** — global loads **after** apply (AI2 M1). Project `.env` quiet works; global alone does not |
| **M1** | AI2 quiet global timing | **Hard fold** — F4/F11/AC10/OPERATIONS: shell or project `.env` only; no reorder (F1); no pre-read DoD |
| **M2** | AI2 truthy 7th copy | **Hard fold** — F9 acknowledge; F18 soft residual core consolidate |
| **M3** | AI2 debug message shape | **Hard fold** — F3 debug SOOT collapsed one line; AC13 |
| **M4** | AI2 project detect dual warn | **Hard fold** — F12 + AC12 manual matrix |
| **L1** | enum name verbose | **Soft** — F25 shorter `EnvOverrideEmit` |
| **L2** | `Option<&str>` ergonomics | **Affirm** — keep pure signature; call site `.ok().as_deref()` |
| **L3** | elevation omits quiet | **Fold** — F4 out-of-scope; §6 |
| **L4** | CAPABILITIES §5 pin | **Hard fold** — F23 |
| **L5** | T113 spec stale | **Affirm** — §2.2 code truth only |
| **L6** | smoke full prefix | **Hard fold** — F10 |
| **O1** | reorder global merge | **Reject DoD** — F1; soft residual F18 spike only |
| **O2** | truthy → core | **Soft** — F18 (via M2) |
| **O3** | smoke count==1 | **Hard fold** — F10/AC9 |
| **O4** | `assert_eq!` full SOOT | **Hard fold** — F9/Phase 1 |
| **O5** | dotenv gap-fill comment | **Soft drive-by** if line touched |

**Go readiness:** after M1–M4 pins above (this revision). Still **plan-only** until user **go**.
