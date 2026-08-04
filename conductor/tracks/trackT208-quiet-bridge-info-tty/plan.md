# T208 Plan — Quiet Cozo / bridge INFO

Status: **Completed** (PR #91 `9985ab4`). Spec: [spec.md](./spec.md).

## Absorbed

| Residual | Disposition |
|----------|-------------|
| T200 Cozo INFO residual | **F2** demote + **F8** filter; closes residual |
| deferred.md Cozo → T208 | This track |
| Audit recall/sync noise | AC1 + soft AC6 |
| EnvFilter prefix leak (M1) | Documented + F8 required |
| Hermetic RUST_LOG flaky (M3/L2) | F22 env_remove + **F29** denylist |
| T81 quiet bridge warnings | Regression AC4 only |
| T207 empty pretty Cozo first | F25 + AC1 |
| T118 smoke RUST_LOG="" (M4) | Residual — not this track |

## Research (2026-08-04)

| Source | Takeaway |
|--------|----------|
| Live repro | INFO Cozo on every graph-on recall; quiet/no-bridge irrelevant |
| EnvFilter prefix | `ai_brains=info` matches `ai_brains_graph` |
| empty vs unset RUST_LOG | `""` → ERROR-only; unset → default_filter |
| `cozo_proxy.rs` | Sole graph info = init |
| tracing / subscriber | **0.1.44** / **0.3.23** — no bump |
| clig.dev | Lifecycle chatter → verbose/debug |

## AI fold-in (2026-08-04)

| ID | Source | Action |
|----|--------|--------|
| **AI1 #1** | Demote info→debug | **Affirm** → F2 |
| **AI1 #2** | RUST_LOG escape | **Affirm + M2** → F4 =debug only |
| **AI1 #3** | T81 boundary | **Affirm** → F11/AC4 |
| **AI1 #4** | Hermetic + docs | **Affirm** → F22/F23 |
| **M1** | Prefix match + F8 | **Accept** → §2.2 + **F8 required** |
| **M2** | F4 =info wording | **Accept** → F4 strip re-elevate |
| **M3** | AC1 env_remove | **Accept** → F22; never `RUST_LOG=""` |
| **M4** | T118 test weakness | **Residual** — not DoD |
| **L1** | Elevate F8 | **Accept required** |
| **L2** | RUST_LOG denylist | **Accept required** → F29 |
| **L3** | hermetic not tracing-test | **Affirm** F15 |
| **L4** | log-format pre-scan | Out |
| **L5** | graph-off cfg | **Affirm** F16 |

## Phases

### A0 — Expand + fold-in (done)

- [x] Live repro + prefix-match root cause (M1)  
- [x] Spec F1–F30 + AC1–AC8  
- [x] AI fold-in disposition  
- [x] Conductor/deferred roll  
- [ ] On **go**: `ledgerful ledger start T208-quiet-bridge-cozo-info --category FEATURE --message "Demote Cozo init to debug; default filter ai_brains_graph=warn; RUST_LOG hermetic denylist"`  
- [ ] On go: `ledgerful scan --impact`  

### A1 — Red

- [x] **B1** Hermetic graph-on AC1: `env_remove("RUST_LOG")` + recall → no `CozoProxyBackend initialized`  
- [x] **B2** AC2: `.env("RUST_LOG","ai_brains_graph=debug")` → line present  
- [x] **B3** Soft AC6: sync query same quiet under unset RUST_LOG  
- [x] **B4** AC4 T81 quiet bridge warning regression  
- [x] **B5** AC7 filter string contains `ai_brains_graph=warn`  
- [x] **B6** AC8 denylist contains `RUST_LOG`  

### B — Green

- [x] **C1** F2: `cozo_proxy.rs` `info!` → `debug!`  
- [x] **C2** F8: default filter `…,ai_brains_graph=warn` in `main.rs`  
- [x] **C3** F29: `AMBIENT_DENYLIST` += `RUST_LOG`  
- [x] **C4** Soft F10: skip — residual  

### C — Docs + closeout

- [x] **D1** CAPABILITIES §9 + OPERATIONS: escape **=debug only** (AC5)  
- [x] **D2** CHANGELOG minor  
- [x] **D3** Review + gate with `--features graph` targeted tests  
- [x] **D4** PR; conductor Completed; deferred strike T200 Cozo residual; note M4 residual  

## Test plan

| Lock | Assert |
|------|--------|
| AC1 | unset RUST_LOG, no Cozo string |
| AC2 | =debug shows Cozo |
| AC4 | T81 quiet |
| AC7–AC8 | filter + denylist |
| Soft AC6 | sync query |

Suite: `quiet_cozo_info` under `#[cfg(feature = "graph")]`. After F29, hermetic_bin strips RUST_LOG; AC1 relies on that + no re-set; AC2 re-sets after strip.

## Manual

- [x] Unset RUST_LOG; graph-on recall pretty empty — Scope/hint only  
- [ ] `RUST_LOG=ai_brains_graph=debug` — init returns  
- [ ] Graph-off binary sanity  

## Stop-before

- Removing Cozo from multiplex  
- Blanket default `warn` only  
- Changing T81  
- AC1 with `RUST_LOG=""`  
- T209 scope  

## Done when

AC1–AC2 + AC5 + AC7–AC8 green; AC3/4/6 soft-or-required per checks; review clear; gate green; T200 Cozo residual closed.

## Implement notes

1. Production: one-line demote + one-line filter suffix + denylist entry.  
2. **Never** prove AC1 with empty `RUST_LOG` — that is ERROR-only, not product default.  
3. Do not assert timestamps.  
4. Keep debug fields (`available`, path) for support.  
