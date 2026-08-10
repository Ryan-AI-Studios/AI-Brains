# T222 — Graph-on install path — Plan

**Status:** 📋 Planning + **AI fold-in** (plan-only until **go**)  
**Category:** INFRA (FEATURE if A2=yes)  
**Depends:** T200 SOOT + exit 2; T198 FEATURE_UNAVAILABLE; T213 density  
**Feeds:** T232 density remediation capability branch  
**Ledger on go:** `ledgerful ledger start T222-graph-on-install-path --category INFRA --message "Graph-on local install scripts + doctor graph_feature; keep Cargo default off unless F2"`

## Goal

Operators who rebuild/install via **local scripts** (or follow INSTALL SOOT) get a **graph-capable** PATH `ai-brains`. Doctor surfaces `graph_feature: available|unavailable`. Cargo `default = []` stays unless product go + T200 F2 size gate.

## Absorbed deferred

- Graph-off PATH usefulness **3** (deferred.md / series README)  
- Local Build-AIBrains / build.ps1 graph-off drift vs INSTALL  

**Not absorbed as DoD:** T232 density remediation branching; release.yml graph-on; Cargo default flip without F2; MSI/binstall; auto rebuild; clap/rusqlite bump; build.ps1 deprecation.

## Decision pins (hard)

| ID | Pin |
|----|-----|
| **B** | Scripts **always** `--features graph` for CLI (F1/F5/F6) |
| **A** | Cargo default flip **optional** — plan default **A2=no** (F2) |
| **C** | Doctor `graph_feature` Ok-severity info; unavailable + SOOT remediation (F9/F10) |
| **Release** | No flip without separate go (F3) |
| **T232** | Density remediations stay for peer track (F11) |

## AI fold-in pins (hard)

| ID | Pin |
|----|-----|
| **M1** | Code **12→13**; docs CAPABILITIES **11→13** (add `harness_wiring` + `graph_feature`) |
| **M2** | `Vec::with_capacity(12) → 13` |
| **M3/L2** | Probe env: known-missing `AI_BRAINS_VAULT_PATH`; fail only exit 2 **and** FEATURE_UNAVAILABLE on stub path |
| **M4** | `GRAPH_REINSTALL_SOOT` in `governed_common` — stubs + doctor + empty-lag substring; smoke guards constant |
| **M5** | Full 13-name order: … `zero_key_escape`, `graph_feature`, `graph_density`, `harness_wiring`, `integrity` |
| **M6** | rusqlite 0.40.2 available — **defer**; clap 4.6.6 — **defer** |
| **O2** | Primary script probe = `doctor --json` → `graph_feature` message `available` |
| **O3** | CAPABILITIES ordered list includes harness_wiring (was missing) |

## Research pins (2026-08-10)

| Fact | Pin |
|------|-----|
| PATH binary | graph-off; `graph update` exit 2 FEATURE_UNAVAILABLE |
| Doctor density | sparse warn; remediation `graph rebuild` (T232) |
| INSTALL SOOT | `cargo install --path crates/ai-brains-cli --locked --features graph` |
| Build-AIBrains / build.ps1 | missing `--features graph` |
| Doctor matrix code | **12** checks; CAPABILITIES text **11** (stale) |
| clap | pin 4.5; latest 4.6.6 — no bump |
| rusqlite | pin 0.39.0; latest 0.40.2 — no bump (M6) |
| Cargo features | default enables listed; `--no-default-features` for slim if A2 |

## Phased checklist

### Phase 0 — Preflight (on go)

- [ ] `ledgerful doctor` / `ledgerful ledger status --compact`
- [ ] `ledgerful scan --impact`
- [ ] `ledgerful ledger start T222-graph-on-install-path --category INFRA …`
- [ ] `ai-brains preflight --summary`
- [ ] Confirm A2: **no** unless user re-go after size measure

### Phase 1 — Red: SOOT + doctor matrix (AC7, AC16)

- [x] Add failing smoke/unit expecting `GRAPH_REINSTALL_SOOT` + matrix order F10 length 13
- [x] Confirm current CAPABILITIES “11 checks” listed for docs AC10 rewrite

### Phase 2 — Green: constant + `graph_feature` (AC4–AC6, M4)

- [x] `pub const GRAPH_REINSTALL_SOOT` in `governed_common.rs`
- [x] Wire both `main.rs` stubs to constant
- [x] `check_graph_feature()`; insert before `graph_density`; `with_capacity(13)`
- [x] `REMEDIATION_EMPTY_LAG` uses constant for install substring only
- [x] Update smoke grep for constant; matrix unit full order + len 13
- [x] cfg dual units: available / unavailable+remediation

### Phase 3 — Scripts (AC1–AC3, AC17, O2)

- [x] `Build-AIBrains.ps1`: `--features graph` + F7 probe (missing vault env; doctor JSON primary; stub secondary)
- [x] `build.ps1`: same (L1: keep parallel minimal script)
- [x] Soft: Install-AIBrains.ps1 only if still useful
- [x] Comment cites GRAPH_REINSTALL_SOOT / INSTALL F27 text

### Phase 4 — Docs (AC10, AC15, O3)

- [x] `Docs/INSTALL.md` — local script path graph-on note (keep Release honesty)
- [x] `Docs/OPERATIONS.md` — graph-on rebuild via script; doctor `graph_feature`
- [x] `Docs/CAPABILITIES.md` — **13** checks; ordered list with `graph_feature` **and** `harness_wiring`
- [x] `CONTRIBUTING.md` — script matrix = graph-on
- [x] `CHANGELOG.md` — T222

### Phase 5 — Optional A2 (only if go)

- [ ] Size measure → `evidence/size-measure.md` (Δ ≤ 8 MB)
- [ ] If pass + go: `default = ["graph"]`; F13 CI; INSTALL slim `--no-default-features`
- [x] Else: skip; AC12 docs-only+scripts

### Phase 6 — Manual + gate (AC8–AC9, AC13–AC14, AC18)

- [ ] Manual: run Build-AIBrains; PATH graph not FEATURE_UNAVAILABLE; doctor `graph_feature=available`
- [ ] Regression: feature-off smoke still exit 2 (CI default job)
- [ ] Confirm lockfile: no clap/rusqlite bump
- [ ] `cargo nextest run -p ai-brains-cli` (+ `--features graph` for available-side units)
- [ ] `cargo clippy -p ai-brains-cli --all-targets -- -D warnings`
- [ ] Full gate: fmt / clippy workspace / nextest / deny / audit
- [ ] `ledgerful verify` / commit / pin / conductor Completed / deferred strike PATH usefulness
- [ ] `review.md`; cross-model if A2=yes

## File touch map

| File | Change |
|------|--------|
| `scripts/Build-AIBrains.ps1` | `--features graph` + F7 probe |
| `scripts/build.ps1` | `--features graph` + F7 probe |
| `crates/ai-brains-cli/src/commands/governed_common.rs` | `GRAPH_REINSTALL_SOOT` |
| `crates/ai-brains-cli/src/main.rs` | stubs use constant |
| `crates/ai-brains-cli/src/commands/doctor.rs` | `graph_feature` + matrix + capacity |
| `crates/ai-brains-cli/src/graph_density.rs` | empty-lag install substring → constant |
| `crates/ai-brains-cli/tests/smoke.rs` | SOOT constant guard |
| `Docs/INSTALL.md`, `OPERATIONS.md`, `CAPABILITIES.md` | Honesty 11→13 + harness_wiring |
| `CONTRIBUTING.md`, `CHANGELOG.md` | Matrix / entry |
| `conductor/*` | Status / deferred |
| `Cargo.toml` features | **Only if A2=yes** |
| `.github/workflows/ci.yml` | **Only if A2=yes** (F13) |
| `release.yml` | **No** |
| Density remediation **branching** | **No** (T232) |

## Non-goals (reminder)

Release flip · density remediation branch · auto rebuild · MSI · default Cargo flip without F2 · dep bumps · capture coupling · build.ps1 deprecation DoD

## Verification matrix

| AC | Proof |
|----|-------|
| AC1–3, AC17 | scripts |
| AC4–7, AC16 | doctor units + smoke + matrix |
| AC8–9 | smoke + tree |
| AC10,15 | docs (11→13 + harness) |
| AC11–12 | A2 branch |
| AC13–14 | gate + manual |
| AC18 | research + lock |

## Out of scope checklist

- [ ] release.yml graph-on  
- [ ] T232 remediation rewrite  
- [ ] dual artifact / binstall  
- [ ] clap 5 / rusqlite 0.40  
- [ ] build.ps1 deprecation  

## Implement notes

1. **Default path:** SOOT constant → doctor → scripts → docs; **A2=no**.  
2. **High findings:** scripts still off; probe touches real vault; CAPABILITIES still 11 or missing harness; fourth SOOT copy; Cargo flip without F13.  
3. **Stop-before:** release flip; density rewrite; Cargo default without go.  
4. **After ship:** plan **T232** for rebuild vs reinstall remediations using `graph_feature`.  
5. **Category:** `INFRA` (scripts+doctor); `FEATURE` only if A2.
