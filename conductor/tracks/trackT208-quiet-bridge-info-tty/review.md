# Track Completion Audit — T208

- **Reviewer:** Internal completion reviewer (read-only R1)
- **Date:** 2026-08-04
- **Branch:** `agent/T208-quiet-bridge-info-tty`
- **Commit:** `ffdb494` (vs origin/main / `fbe5328`)
- **Scope:** Quiet Cozo / bridge INFO on human CLI paths (F2 demote, F8 default filter, F29 denylist, hermetic AC1–AC2/AC7–AC8, docs)

## Verdict: PASS

No P0/P1/P2 findings. Implementation matches frozen F1–F30 and required AC1–AC2/AC5/AC7–AC8. Soft AC3/AC4/AC6 covered or explicitly soft. Incidental touchpoints reviewed as non-regressions.

## Scope Reviewed

| Area | Paths / evidence |
|------|------------------|
| Spec / plan | `conductor/tracks/trackT208-quiet-bridge-info-tty/spec.md`, `plan.md` |
| F2 demote | `crates/ai-brains-graph/src/cozo_proxy.rs` (`tracing::debug!` init) |
| F8 default filter + AC7 | `crates/ai-brains-cli/src/main.rs` `DEFAULT_ENV_FILTER` + unit test |
| F29 denylist + AC8 | `crates/ai-brains-cli/tests/common/mod.rs` `AMBIENT_DENYLIST` |
| Hermetic ACs | `crates/ai-brains-cli/tests/quiet_cozo_info.rs` |
| Wiring / construct path | `live_graph.rs` `CozoProxyBackend::new`; `recall.rs` / `sync.rs` graph-on store/hook |
| T81 quiet path | `crates/ai-brains-retrieval/src/recall.rs` `options.quiet` eprintln gate; soft AC4 + smoke |
| Docs | `Docs/CAPABILITIES.md` §9, `Docs/OPERATIONS.md`, `CHANGELOG.md` |
| Incidental | retrieval graph-hop let-chain style; `FEATURE_UNAVAILABLE` still live under graph-on via `exit_code_for_api_error` |
| Graph crate INFO audit | `rg tracing::info!` under `ai-brains-graph` → **zero** matches |

## Requirement and DoD Matrix

### Frozen decisions F1–F30

| ID | Status | Evidence |
|----|--------|----------|
| **F1** Scope = quiet Cozo lifecycle INFO | **Met** | No ranking/backup/T81 redesign; demote + filter only |
| **F2** `info!` → `debug!` init | **Met** | `cozo_proxy.rs` L120–127 `tracing::debug!(…, "CozoProxyBackend initialized")` |
| **F3** Single INFO site; no re-raise debug | **Met** | No `info!` in graph crate; mutations already `debug!` |
| **F4** Escape `=debug` only (not `=info`) | **Met** | CAPABILITIES/OPERATIONS/CHANGELOG/main comments; AC2 uses `ai_brains_graph=debug` |
| **F5** Do not require `--quiet` for Cozo quiet | **Met** | Cozo is tracing, not `options.quiet`; quiet only gates bridge eprintln (T81) |
| **F6** No graph behavior change | **Met** | `LiveGraphHook::new` still builds `CozoProxyBackend`; failures still non-fatal `warn!` |
| **F7** Empty + non-empty quiet | **Met** | Recall always constructs `GraphAwareEventStore` before pin loop; AC1 empty query |
| **F8** Default filter `…,ai_brains_graph=warn` | **Met** | `DEFAULT_ENV_FILTER` exact SOOT string; AC7 unit assert |
| **F9** JSON + pretty benefit | **Met** | Shared subscriber filter; pretty locked by AC1/AC2 |
| **F10** Lazy store residual | **Out / soft** | Not DoD; not implemented (correct) |
| **F11** Bridge warnings out of scope | **Met** | T81 path unchanged; soft AC4 regression |
| **F12** Closes T200 Cozo residual | **Met** (code) | Product fix lands; conductor deferred strike is D4 ship residual |
| **F13** Capture independence | **Met** | Capture path / store append unchanged |
| **F14** No daemon-only work | **Met** | Daemon does not construct Cozo / install this filter |
| **F15** Zero new crates | **Met** | Hermetic CLI capture only |
| **F16** Hermetic locks + M3 | **Met** | `#[cfg(feature = "graph")]` AC1/AC2/AC6; AC1 never sets `RUST_LOG=""` |
| **F17** High-finding avoidance | **Met** | Init demoted; Cozo kept; not blanket warn-only; T81 untouched; docs =debug |
| **F18** Exit codes unchanged | **Met** | No exit-code edits |
| **F19** FEATURE primary review | **Met** | This R1 |
| **F20** Series position | **N/A** | Process |
| **F21** Determinism / strip ambient | **Met** | F29 denylist + absence asserts |
| **F22** AC1 env_remove / AC2 debug | **Met** | Denylist strip + no re-set (AC1); `.env("RUST_LOG","ai_brains_graph=debug")` (AC2) |
| **F23** Docs | **Met** | CAPABILITIES §9 + OPERATIONS + CHANGELOG minor |
| **F24** Privacy (path debug-only) | **Met** | Fields remain on `debug!` only |
| **F25** T207 interaction | **Met** | Default filter has no Cozo preamble (AC1) |
| **F26** Soft declines | **Met** | Lazy store / T209 / M4 not in DoD |
| **F27** Ledger start | **Process residual** | Plan A0 unchecked; not a product correctness fail |
| **F28** Implement order | **Met** | Demote + filter + denylist + tests + docs |
| **F29** `RUST_LOG` on denylist | **Met** | `AMBIENT_DENYLIST` includes `"RUST_LOG"`; AC8 |
| **F30** Fold-in applied | **Met** | M1–M3/L1/L2 reflected in code + docs |

### Acceptance criteria AC1–AC8

| AC | Status | Evidence |
|----|--------|----------|
| **AC1** Unset `RUST_LOG`, no Cozo init | **Met** | `quiet_cozo__recall_unset_rust_log__no_cozo_init_line`; hermetic strip; **not** `RUST_LOG=""` |
| **AC2** `=debug` shows init | **Met** | `quiet_cozo__recall_graph_debug__shows_cozo_init_line` re-sets after strip |
| **AC3** `--log-format off` quiet tracing | **Soft Met** | Existing `off` arm forces filter `"off"`; soft AC4 uses it; no dedicated new lock required |
| **AC4** T81 `--quiet` bridge-failed | **Soft Met** | `quiet_cozo__recall_quiet__no_bridge_failed_human_warning` + existing smoke `test_recall_quiet_silences_bridge_warning` |
| **AC5** Docs =debug only + CHANGELOG | **Met** | CAPABILITIES L297–304; OPERATIONS L608–614; CHANGELOG L46 |
| **AC6** Soft sync query quiet | **Soft Met** | `quiet_cozo__sync_query_unset_rust_log__no_cozo_init_line` |
| **AC7** Filter contains `ai_brains_graph=warn` | **Met** | `default_env_filter__contains_ai_brains_graph_warn` |
| **AC8** Denylist includes `RUST_LOG` | **Met** | `ambient_denylist__includes_rust_log` (always, not graph-gated) |

### Default filter SOOT (spec §10.3)

```text
warn,ai_brains=info,ai_brains_cli=info,ai_brains_brain=info,ai_brains_graph=warn
```

**Exact match** in `DEFAULT_ENV_FILTER` (`main.rs` L42–43).

## Findings (P0-P3 format if any)

**None blocking.**

### Optional observations (not DoD failures)

| ID | Sev | Note |
|----|-----|------|
| T208-O1 | P3 | AC1/AC6 pass `--quiet`. Cozo is tracing, not quiet-gated, so this does **not** false-green F2/F8. A no-`--quiet` AC1 would slightly strengthen F5 optics only. |
| T208-O2 | P3 | No hermetic lock that `RUST_LOG=ai_brains_graph=info` **hides** demoted init (F4/M2 docs + code demote prove this; AC2 proves escape at debug). |
| T208-O3 | P3 | Global `~/.ai-brains/.env` can still gap-fill `RUST_LOG` after denylist strip (same class as T205 KEY). Spec F29 only required ambient denylist. Risk is false-**fail**, not false-green. |

None of O1–O3 require deferral tracking as open defects for clearance.

## Completeness Sweep

| Check | Result |
|-------|--------|
| Placeholders / TODO in T208 product code | **None** |
| AC1 uses empty `RUST_LOG=""` | **No** — denylist strip only; comments cite M3 |
| F8 missing | **No** — constant + AC7 |
| Denylist missing | **No** — F29 + AC8 |
| Escape documented as `=info` | **No** — docs explicitly say `=info` is not enough |
| Graph behavior removed | **No** — Cozo still constructed; multiplex intact |
| T81 changed | **No** — `if !options.quiet { eprintln!(…) }` unchanged |
| Capture independence | **Preserved** |
| Zero new crates | **Yes** |
| Graph-off still builds AC8 / AC7 | **Yes** (cfg-free) |
| Graph-on ACs cfg-gated | **Yes** |

## Wiring and Regression Review

### Demote hits default CLI path

1. Graph-on `recall` builds `GraphAwareEventStore::new` → `LiveGraphHook::new` → `CozoProxyBackend::new(None)` (`live_graph.rs` L24–28; `recall.rs` L187–188).
2. Pretty `sync query` delegates to `recall::run` (`sync.rs` L471–488) → same construct path (soft AC6 meaningful).
3. Init log is `debug!` only; default EnvFilter enables `ai_brains=info` but **`ai_brains_graph=warn` overrides** prefix match (M1/F8).
4. Subscriber install: `try_from_default_env().unwrap_or(DEFAULT_ENV_FILTER)` on default `compact` and other non-`off` formats (`main.rs` L1816–1852). Unset `RUST_LOG` → product default.

### Denylist strips

`hermetic_bin` → `strip_ambient` → `env_remove` for every `AMBIENT_DENYLIST` entry including `RUST_LOG`. AC2 re-sets after strip; AC1 does not.

### Tests would fail on old behavior

| Old behavior | Failing lock |
|--------------|--------------|
| Init still `info!` + no `ai_brains_graph=warn` | AC1 (line present under default) |
| Init removed entirely | AC2 (line absent under debug) |
| F8 filter suffix omitted | AC7 |
| Denylist omits `RUST_LOG` | AC8 |
| Escape only documented/implemented as `=info` after demote | Docs/AC2 mismatch (AC2 correctly uses debug) |

AC1 alone is satisfied by **either** F2 **or** F8; DoD requires both. F8 is independently locked by AC7; F2 is visible in source and is the level AC2/debug escape depends on for “lifecycle at debug.”

### Incidental diffs

| Item | Assessment |
|------|------------|
| Retrieval graph-hop `if cond && let Some(…)` let-chain | Behavior-preserving style/clippy shape; quiet / blend / hop logic intact |
| `FEATURE_UNAVAILABLE` under graph-on | Still referenced from `exit_code_for_api_error` / tests; graph stubs remain `#[cfg(not(feature = "graph"))]`. No honesty regression |

### Capture / graph safety

- Capture path does not depend on tracing level of Cozo init.
- Live graph apply/flush failures remain `tracing::warn!` (not demoted).
- No Cozo removal from multiplex.

## Verification Evidence

**Read-only audit.** Orchestrator-reported (not re-executed in this review):

- nextest quiet_cozo graph-on: **6 passed**
- clippy graph-on targeted: **pass**
- AC1 does **not** use `RUST_LOG=""`

Static verification performed here:

- F2 demote present; zero graph-crate `info!`
- `DEFAULT_ENV_FILTER` SOOT exact
- Denylist includes `RUST_LOG`; strip wired
- Hermetic suite: AC1, AC2, soft AC6, soft AC4, AC8; AC7 unit in `main.rs`
- Docs: escape `=debug` only; CHANGELOG minor T208 entry

## Deferred Candidates

**None required for clearance.**

Pre-existing / already-spec residuals (not new findings):

- **M4:** T118 smoke `tracing_filter__external_deps_stay_quiet` still uses `RUST_LOG=""` (ERROR-only, not product default) — explicitly out of T208 DoD
- **F10:** Lazy `GraphAwareEventStore` — soft residual
- **Process D4:** PR, conductor Completed, deferred.md strike for T200 Cozo residual, ledger commit — ship closeout, not product fail

Optional hygiene (O1–O3 above) may be absorbed later without blocking T208.

## Completion Decision

**PASS** — implementer may proceed to final review / gate / ship closeout.

| Required for green DoD | Status |
|------------------------|--------|
| F2 demote | Done |
| F8 filter | Done |
| F29 denylist | Done |
| AC1 (unset, not empty) | Locked hermetically |
| AC2 (`=debug` present) | Locked hermetically |
| AC5 docs | Done (`=debug` only) |
| AC7 / AC8 | Locked |
| Soft AC3/4/6 | Soft Met |
| No high-severity regressions | Confirmed |

**Ship residuals (process, not FAIL):** ledger start/commit if used; conductor status → Completed; deferred.md strike “Graph-on Cozo INFO → T208”; note M4 remains residual.

---

## Cross-model R1 (Claude fallback — Codex rate-limited)

- **Reviewer:** Claude Sonnet (cross-model fallback)
- **Date:** 2026-08-04
- **Artifact:** `review.claude.md`
- **HEAD reviewed:** `7ed2bf0` (product `ffdb494` + F5 AC1/AC6 strengthen)
- **Codex:** usage limit until ~2026-08-07; same fallback path as T207

### Verdict: PASS

No P0/P1/P2 findings. F5 proven without `--quiet` on AC1/AC6. Required DoD met.

### Disposition of optional notes
| Note | Disposition |
|------|-------------|
| M4 T118 `RUST_LOG=""` | Pre-existing residual — not T208 DoD; remain on deferred |
| F10 lazy GraphAwareEventStore | Soft residual — not DoD |
| Global dotenv can re-inject RUST_LOG (false-fail) | False-fail only; denylist is ambient strip SOOT |

### Final gate decision
Cross-model **PASS** with zero open findings greater than low. Fresh clean review on HEAD after F5 strengthen. Engineering clearance for PR/CI.

