# T242 Plan — Env override session quiet

**Status:** ✅ **Completed** (PR #147 squash `9f3148b`)  
**Spec:** [spec.md](./spec.md) F0–F31 / AC1–AC16 + §13 AI fold-in  
**Category:** UX / POLISH  
**Ledger TX:** `1b39b40a-2b0f-446b-8763-388720ec106a` (+ Linux unit fix `89e1b859-156e-4c76-bf5a-8ebb68bc2be5`)

---

## AI fold-in (2026-08-12) — `C:\dev\AI-review.md` AI1 + AI2

No Highs. Spec design affirmed. Three AI2 mediums are **must-fold** before go; AI1 mediums mostly restated already-planned work with concrete API shape.

### Disposition

| ID | Source | Verdict | Action |
|----|--------|---------|--------|
| **AI1 M1** `EnvOverrideFingerprint` + sha2/hex | AI1 | **Agree hard** | Phase 1 pure units; workspace deps only |
| **AI1 M2** `decide_env_override_emit` quiet/force/seen | AI1 | **Agree hard** | Pure policy; marker claim at call site (AI2 M2 refine) |
| **AI1 M3** fail-open + chrono RFC3339 content | AI1 | **Partial** | Fail-open yes; **decline chrono** → 0-byte marker (AI2 L4) |
| **AI1 M4** `AI_BRAINS_FORCE_ENV_WARN` | AI1 | **Agree hard** | F7 |
| **AI1 L1/O1** docs + unit matrix | AI1 | **Agree** | Phase 1+3 |
| **AI2 M1** smoke no home redirect → sticky break | AI2 | **Agree hard** | Phase 1: `isolate_empty_home` on existing smoke |
| **AI2 M2** concurrent double-warn | AI2 | **Agree hard** | Phase 2: atomic `create_new` claim reorder |
| **AI2 M3** home resolve after apply | AI2 | **Agree hard** | Phase 2: resolve home **inside** apply |
| **AI2 L1** fingerprint cwd = `.env` parent | AI2 | **Agree hard** | F4 |
| **AI2 L2** no TTL cleanup | AI2 | **Agree** | Docs manual reset only |
| **AI2 L3** single truthy fn | AI2 | **Agree** | `env_warn_truthy` shared |
| **AI2 L4** empty marker | AI2 | **Agree hard** | 0 bytes |
| **AI2 L5** AtomicBool | AI2 | **Agree soft** | Keep defensive |
| **AI2 L6** key-match fields | AI2 | **Agree hard** | F24 |
| **AI2 L7** `--no-project-context` | AI2 | **Agree** | F30 pin only |
| **AI2 L8** marker only Stderr | AI2 | **Agree hard** | No session-only markers |
| **AI2 L9** decision table + body strip | AI2 | **Agree hard** | spec §7.1 / F31 |
| **AI2 L10** smoke KEY safe under empty home | AI2 | **Agree** | KEY already set by hermetic_bin |
| **AI2 O11–O13** CX soft / F17 / F20 | AI2 | **Agree** | No change to F20/F28 |

### Pins locked by fold-in

1. **Fingerprint (AI1 M1 + AI2 L1/L6):**  
   `compute_fingerprint_hex` over `norm_cwd|shell_p|shell_s|env_p|env_s`; cwd = `normalize_for_location_compare(.env parent)`; env fields by **key match**.

2. **Atomic claim (AI2 M2):**  
   `OpenOptions::create_new(true)` — Ok → Stderr; AlreadyExists → Debug; other IO / no home → Stderr fail-open. **Not** check-then-write.

3. **Home inside apply (AI2 M3):**  
   `resolve_user_home_for_dotenv()` called from `apply_local_project_context_env` (double call with global dotenv OK).

4. **Smoke home redirect (AI2 M1):**  
   `preflight__local_env_project_context_overrides_inherited_shell_ids` **must** call `common::isolate_empty_home` before spawn. Assertion remains count==1 under fresh temp home. AC15: second nextest run still passes.

5. **0-byte markers; no chrono** (AI2 L4; decline AI1 timestamp).

6. **Marker only when classify = Stderr** (AI2 L8).

7. **Single `env_warn_truthy`** for QUIET + FORCE (AI2 L3); quiet > force > claim.

---

## Preflight (plan time — 2026-08-12)

| Check | Result |
|-------|--------|
| Live multi-spawn | 3× → 3 Warnings (cross-process spam confirmed) |
| Quiet shell | Suppresses |
| `ledgerful ledger status` | 0 pending |
| dotenvy | **0.15.7** latest stable |
| sha2 / hex | Workspace **0.11** / **0.4** (AI1+AI2 verified) |
| smoke override test | Uses `hermetic_bin` **without** home redirect today → AI2 M1 gap |
| apply vs home resolve | apply ~2116, home for global ~2130 → AI2 M3 gap |

---

## Absorbed deferred

| Item | Source | Plan action |
|------|--------|-------------|
| Env override warn spam (T223 residual) | deferred.md | Close on ship |
| once-per-TTY rate limit | T223 F18 | **DoD** F3 atomic marker |
| clap quiet flag | T223 F18 | Soft F16 |
| elevation quiet handoff | T223 F18 | Soft F17 |
| truthy → core | T223 F18 | Soft F18 (T242 only unifies quiet+force locally) |
| global quiet pre-read | T223 F18 | Soft F19 |
| T240 identity session quiet | T240 | **Decline** DoD |

---

## Phases

### Phase 0 — Plan freeze

- [x] Full spec + plan
- [x] Live dogfood + dep research
- [x] AI fold-in AI1+AI2 → F3–F5, F14–F15, F30–F31, AC15–16, §13
- [x] User **go** before production code

### Phase 1 — Red (TDD)

- [x] Pure units (AI1 O1 names):
  - `decide_env_override_emit__quiet_wins_over_force`
  - `decide_env_override_emit__force_keeps_stderr` (force skips claim)
  - `decide_env_override_emit__unseen_keeps_stderr` (call-site claims)
  - `compute_fingerprint_hex__stable_across_identical_inputs`
  - `compute_fingerprint_hex__differs_on_cwd_or_project_change`
  - Session-only → no marker path exercised
- [x] **Update existing smoke** with `common::isolate_empty_home` (AI2 M1) — still assert 1 Warning
- [x] Hermetic multi-spawn suite (temp home): AC1–AC5, AC7, AC9
- [x] Optional: marker file exists under temp home after first spawn (0 bytes)

### Phase 2 — Green

- [x] `env_warn`: fingerprint + decide + `env_warn_truthy` rename/alias
- [x] Marker IO: `try_claim_marker` with `create_new` (AI2 M2) in `env_warn_session.rs`
- [x] Wire apply: home inside apply (AI2 M3); fingerprint from `.env` parent; Stderr-only claim
- [x] `AtomicBool` belt (F9)
- [x] No production unwrap/expect

### Phase 3 — Docs + registry

- [x] CAPABILITIES: session quiet, FORCE, re-warn, quiet wins, global late
- [x] OPERATIONS: FORCE env row + `Remove-Item` cache reset (AI2 L2)
- [x] CHANGELOG T242
- [x] deferred.md close residual; README series; conductor Completed (closeout after PR #147)

### Phase 4 — Review + gate

- [x] Internal review.md (R1 FAIL → fix → R2 PASS)
- [x] Soft cross-model (F28) Codex CX1 product clean; process closeout
- [x] Full CI gate; AC15 re-run smoke twice locally; GH CI Win/Linux/macOS green
- [x] Manual AC13 evidence
- [x] ledger commit + pin

---

## Manual evidence (recorded 2026-08-12)

```powershell
1..3 | ForEach-Object { ai-brains preflight --summary 2>&1 | Select-String 'Warning: local' }
$env:AI_BRAINS_QUIET_ENV_WARN='1'; ai-brains preflight --summary 2>&1 | Select-String 'Warning: local'
Remove-Item Env:AI_BRAINS_QUIET_ENV_WARN
$env:AI_BRAINS_FORCE_ENV_WARN='1'; ai-brains preflight --summary 2>&1 | Select-String 'Warning: local'
Remove-Item Env:AI_BRAINS_FORCE_ENV_WARN
Remove-Item -Recurse -Force "$env:USERPROFILE\.ai-brains\cache\env-override-warn" -ErrorAction SilentlyContinue
```

| Case | Expected | Actual | Pass? |
|------|----------|--------|-------|
| First of 3 | 1 Warning | 1 | **Pass** |
| 2nd–3rd | 0 Warning | 0,0 | **Pass** |
| Quiet | 0 | 0 | **Pass** |
| Force | 1 | 1 | **Pass** |
| After cache delete | 1 | 1 | **Pass** |

Evidence (2026-08-12): cleared `%USERPROFILE%\.ai-brains\cache\env-override-warn`, then `target\debug\ai-brains.exe preflight --summary` ×3 → warn counts **1,0,0**; `AI_BRAINS_QUIET_ENV_WARN=1` → **0**; `AI_BRAINS_FORCE_ENV_WARN=1` → **1**; cache delete → **1**.

---

## Targeted checks (executed)

```powershell
cargo nextest run -p ai-brains-cli --lib env_warn
cargo nextest run -p ai-brains-cli --test smoke -- preflight__local_env
cargo nextest run -p ai-brains-cli --test env_override_session_quiet
# AC15: run smoke test twice
cargo nextest run -p ai-brains-cli --test smoke -- preflight__local_env
cargo clippy -p ai-brains-cli --all-targets -- -D warnings
```

---

## Non-goals checklist

- [ ] Do not reorder global dotenv before project apply
- [ ] Do not silence T240 identity via this marker
- [ ] Do not write markers into the repo
- [ ] Do not add chrono/flock/BLAKE3
- [ ] Do not bump dotenvy / clap

---

**Completed** — PR #147 squash `9f3148b`; soft residual F16–F19 only.
