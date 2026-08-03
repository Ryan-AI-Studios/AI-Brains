# T200 — Graph Feature Install Honesty

- **Track ID:** T200-GraphFeatureInstallHonesty
- **Phase:** Post-T199 CLI UX series (P1/P3)
- **Status:** ✅ **Completed** (2026-08-03) — PR #83 squash-merged `84f4a23`; docs-only A; Codex R2 PASS WITH DEFERRED P3
- **Depends on:** T66–T69 graph; **T198** FEATURE_UNAVAILABLE + exit 2 (shipped); T196 CONTRIBUTING/INSTALL; capture-independence CI edge check
- **Blocks / feeds:** Operator install path honesty; **required** CI coverage for graph-on **or** feature-off (A2-gated); T201 may refine feature-unavailable envelopes
- **Category:** FEATURE / DOCS / INFRA (light)
- **Source:** CLI audit 2026-08-02 P3 + graph E≈2; T198 handoff “install policy”
- **Deferred absorbed:** Graph default install / feature honesty residual; CAPABILITIES stub wording lag after T198; missing CI for graph-on smoke
- **Not absorbed:** Cozo remote product; graph schema change; desktop graph UI; MSI packaging; flip release.yml graph-on without gate; full exit matrix (T201); help IA (T204); cargo-binstall metadata
- **Research date:** 2026-08-03 (expand + live re-scan)
- **AI fold-in:** AI1 affirms INSTALL primary / cost gate / hint / exit 2 / docs sync. AI2 **M1–M7** accepted; **L1/L3/L8/L10** notes; **O1/O2** declined; **O3** soft residual. Disposition §14.
- **Ledger:** TX `718ff569` (DOCS); shipped PR #83

## 1. Objective

1. Ship **one clear product install policy** for graph so operators are not surprised by help listing `graph` while a bare install / release binary cannot run it.  
2. **Preserve T198 honesty:** feature-off remains **exit 2** + `FEATURE_UNAVAILABLE` (not silent success).  
3. Align **INSTALL / CONTRIBUTING / CAPABILITIES / CHANGELOG** with that policy; document **GitHub Release = graph-off** honesty (M1).  
4. Keep **capture independence**. Prefer **docs-only A** (no Cargo default flip) unless implement explicitly measures and opts into Cargo default after F2 (AI2 strong recommendation).  
5. Ensure **CI covers both** feature-on smoke and feature-off exit 2 (which side is “extra” depends on A2 — F13/F14 hard, not soft).

## 2. Live baseline (re-scan 2026-08-03; AI2 11/11 confirmed)

| Observation | Live (post-T198) | Gap for T200 |
|-------------|------------------|--------------|
| Feature-off `graph *` | Both stubs (`main.rs` ~1810 + ~2825): exit **2** + `FEATURE_UNAVAILABLE:` + reinstall hint | Exit honesty **closed T198** — regression-only |
| Reinstall hint strings | Already: `cargo install --path crates/ai-brains-cli --locked --features graph` (both stubs) | F9 = **grep guard**, not planned edit (M4) |
| `graph --help` feature-off | Exit **0** | OK |
| Cargo features | `default = []`; `graph = [dep:ai-brains-graph, ai-brains-retrieval/graph]` | Bare install = no graph |
| INSTALL.md | Primary **without** graph; graph is comment | Primary must recommend `--features graph` |
| CAPABILITIES | §9 “stub with install hint”; table/needs lines omit exit 2 | Update **all** graph honesty refs (M6) |
| CONTRIBUTING | Capture independence only | Build matrix missing |
| Feature-on smoke | `test_graph_health_smoke` `#[cfg(feature = "graph")]` | **Not in CI** today (M7) |
| Feature-off smoke | `graph__default_build__prints_hint` `#[cfg(not(feature = "graph"))]` | Runs only when graph off (default CI today) |
| Graph weight | SQLite-native; Cozo IPC only; no crates.io cozo | Default-on not “heavy Cozo” |
| Capture tree CI | Forbids capture→graph/models/sync | Keep green |
| Binary sample | ~22 MB release local; **`release.yml` builds graph-off** (no `--features graph`) | Baseline = graph-off (L10) |
| Dep pins | clap 4.5, rusqlite 0.39.0 | No bumps |

### 2.1 Routing / files

| Path | Role |
|------|------|
| `Docs/INSTALL.md` | Primary graph install + slim + **release binary honesty** (M1) |
| `Docs/CAPABILITIES.md` | §9 + command table + needs note (M6) |
| `CONTRIBUTING.md` | Build matrix; how to run graph smoke |
| `Docs/README.md` + `CHANGELOG.md` | Align |
| `crates/ai-brains-cli/Cargo.toml` | Cargo default only if A2=yes after F2 |
| `crates/ai-brains-cli/src/main.rs` | Stubs: regression only (F3/F9); no churn unless INSTALL SOOT changes |
| `crates/ai-brains-cli/tests/smoke.rs` | Feature-on/off smokes |
| `.github/workflows/ci.yml` | **Required** graph-on **or** no-default-features job (A2-gated) — hard DoD |
| Soft: `scripts/dev-check.ps1` | Note gap if free (L5) |
| Soft: `release.yml` | **Do not** flip graph-on without F2 + product go; document only (M1) |

## 3. Research summary (2026-08-03)

| Finding | Application |
|---------|-------------|
| Cargo default-for-binaries | Defensible if cost low; **prefer docs-only A** to avoid CI dual-matrix + release divergence (AI2) |
| Capture independence | Capture crate edge, not CLI omit graph |
| Graph weight | Workspace SQLite; compile-time delta for CLI codegen likely small (L1) |
| T198 B-min closed | Exit 2; T200 = install + docs + CI coverage |
| Release vs source | release.yml graph-off ≠ INSTALL source `--features graph` — must document (M1) |
| Dep bumps | Out |

## 4. Frozen decisions (F1–F35)

| ID | Decision |
|----|----------|
| **F1 — Policy + preferred path (M5/AI2)** | **Always:** INSTALL primary recommends `cargo install --path crates/ai-brains-cli --locked --features graph`. **Preferred implement path: docs-only A** (`default = []` stays) unless product go explicitly requests Cargo flip after F2. Cargo `default = ["graph"]` remains **optional** behind F2 + hard F13 CI — not the default recommendation. |
| **F2 — Cost gate if Cargo flip (M2)** | Flip only if **all** of: (a) **absolute** release binary size delta **≤ 8 MB** (drop % clause); (b) feature-on nextest green; (c) capture tree clean; (d) `cargo deny check` still green with graph on (O4 soft→required if flip). Measure with **same shape as release** when comparing to artifacts: `cargo build --release -p ai-brains-cli --target x86_64-pc-windows-msvc` ± `--features graph` (or local release profile documented if target unavailable). Record numbers in `evidence/size-measure.md`. If any fail or prefer docs-only → **no Cargo flip**. |
| **F3 — Feature-off honesty (T198)** | Exit **2**, `FEATURE_UNAVAILABLE` prefix, reinstall hint, help exit **0**. No regression. |
| **F4 — Capture independence** | Unchanged; no capture→graph edge. |
| **F5 — INSTALL paths + release honesty (M1)** | (1) Primary: source `cargo install --path crates/ai-brains-cli --locked --features graph`. (2) Slim/secondary: branch on A2 — **A2=no:** bare `cargo install --path crates/ai-brains-cli --locked` (= no graph, exit 2 on `graph *`); **A2=yes:** `--no-default-features` for slim. (3) **Required:** state that **GitHub Release `ai-brains.exe` is graph-off** (release.yml has no `--features graph`); for graph use source install with features, or a future graph-enabled artifact (out of DoD). Do **not** flip release.yml without separate go + F2. |
| **F6 — CONTRIBUTING** | Matrix: workspace default tests vs `--features graph` for graph smoke; point to INSTALL + release honesty. |
| **F7 — CAPABILITIES full graph honesty (M6)** | Update **all** graph operator-facing refs: §9 exit 2 + FEATURE_UNAVAILABLE; graph command table footnote (feature-off exits 2); any “needs graph feature” line. Capture independence notes unchanged. |
| **F8 — Docs README** | Align one-liner with recommended source install (graph-on). |
| **F9 — Reinstall hint SOOT (M4)** | Both stubs already match INSTALL primary string. **Do not edit unless INSTALL SOOT changes.** Add **grep regression** (unit or hermetic) that both stub sites contain the INSTALL primary install command. |
| **F10 — No new crates / no dep bumps** | Zero. |
| **F11 — Feature-on smoke** | Keep `test_graph_health_smoke` (nodes/edges/live). |
| **F12 — Feature-off smoke** | Keep `graph__default_build__prints_hint`. With A2=yes only runs under `--no-default-features`. |
| **F13 — Feature-off CI hard if A2=yes (M3)** | If `default = ["graph"]`: **required** CI step (not soft, not continue-on-error): `cargo nextest run -p ai-brains-cli --no-default-features` (or equiv) so exit-2 smoke still runs. **DoD.** |
| **F14 — Feature-on CI hard if A2=no (M7)** | If docs-only A (`default = []`): **required** CI step (not soft): `cargo nextest run -p ai-brains-cli --features graph` covering graph health smoke (full package or `-E 'test(graph)'` + known smoke). **DoD.** If A2=yes, workspace nextest already graph-on → feature-on CI satisfied by default workspace job; F13 covers off. **Post-T200: both on and off must have CI coverage.** |
| **F15 — JSON envelope** | → **T201**. Note: `exit_code_for_api_error` already maps FEATURE_UNAVAILABLE→2 (T198); T201 = fail_api call, not new map. |
| **F16 — Skills** | Soft one-line only. |
| **F17 — Size evidence** | Track `evidence/size-measure.md` if A1 runs; skip file if docs-only and no flip (optional A1 still recommended once). |
| **F18 — Claims** | No “graph always in every install”; release graph-off honesty; no multi-OS graph product overclaim. |
| **F19 — High findings** | INSTALL omits graph primary; CAPABILITIES stale; release/source divergence undocumented; Cargo flip without F13; feature-on never in CI (A2=no); exit 2 regression; capture tree break. |
| **F20 — Series** | After T198/T199; docs-only parallel-safe with T201. |
| **F21 — Determinism** | Stable INSTALL SOOT string. |
| **F22 — Review** | Docs-only A → cross-model optional. Cargo flip + CI → FEATURE/INFRA, cross-model **required**. |
| **F23 — `--locked`** | In primary INSTALL SOOT. |
| **F24 — No rename** | Keep `graph` subcommand. |
| **F25 — retrieval/graph** | Unchanged wiring; if A2=yes, note retrieval graph feature enables with CLI default (L3 — internal only). |
| **F26 — Keep graph in help** | Decline hide-when-off. |
| **F27 — INSTALL SOOT string pin** | Exact primary: `cargo install --path crates/ai-brains-cli --locked --features graph` |
| **F28 — release.yml scope (L8)** | Honesty note applies to **`ai-brains.exe` only**; `ai-brainsd` has no graph feature. |
| **F29 — No second release artifact DoD** | Soft/out: dual `ai-brains-graph.exe` not required. |
| **F30 — Stub dedupe** | Soft residual → later (O3); not T200 DoD. |
| **F31 — binstall (O1)** | Out (packaging debt). |
| **F32 — slim feature alias (O2)** | Decline; use bare install or `--no-default-features`. |
| **F33 — Plan B1 branch (M5)** | Docs tasks branch on A2 yes/no for slim install wording. |
| **F34 — High finding add** | Feature-on or feature-off missing from CI post-ship. |
| **F35 — AI1 affirm** | Primary install, F2 measure, hint parity, exit 2 preserve, docs sync — all above. |

## 5. Residual disposition

| Residual | Disposition |
|----------|-------------|
| Graph install honesty | **Absorb** |
| Feature-off exit 0 | **Closed T198** |
| CAPABILITIES lag (all graph refs) | **Absorb** F7 |
| Release vs source graph | **Absorb** F5 honesty (no release.yml flip DoD) |
| Graph-on / feature-off CI | **Absorb** F13/F14 hard |
| JSON feature-unavailable | **T201** |
| Help grouping | **T204** soft |
| Stub dedupe | Soft residual |
| binstall / MSI | Out |

## 6. Acceptance criteria

| AC | Criterion | Proof |
|----|-----------|-------|
| **AC1** | INSTALL primary = F27 SOOT (`--locked --features graph`) | Diff |
| **AC2** | INSTALL slim path correct for A2 outcome; feature-off exit 2 documented | Diff |
| **AC3** | Feature-off: exit 2 + FEATURE_UNAVAILABLE (regression) | Smoke + **CI** (default or F13) |
| **AC4** | Feature-on: graph health smoke; **runs in CI** (default workspace if A2=yes, or F14 if A2=no) | Smoke + CI |
| **AC5** | CAPABILITIES graph sections (§9 + command table + needs) + CONTRIBUTING + CHANGELOG | Diff |
| **AC6** | Grep guard: both main.rs stubs contain F27 SOOT string | Test / review |
| **AC7** | Capture tree forbid graph still green | CI / local |
| **AC8** | If A2=yes: size evidence ≤8 MB delta + F13 feature-off CI required | evidence + CI |
| **AC9** | If A2=no: plan notes “docs-only A”; F14 graph-on CI required | Plan + CI |
| **AC10** | Full gate | Process |
| **AC11** | Claims: no false always-graph; release graph-off honesty present | Review |
| **AC12** | INSTALL documents GitHub Release `ai-brains.exe` is graph-off (M1) | Diff |
| **AC13** | Both feature-on and feature-off have CI coverage post-ship (F13∨F14) | CI config review |

## 7. Non-goals

- Graph schema / Cozo product / desktop UI  
- Making graph required for capture  
- Flipping release.yml to graph-on without explicit go  
- Dual release artifact DoD  
- JSON feature envelope (T201)  
- Help IA (T204)  
- cargo-binstall metadata  
- rusqlite / clap upgrades  
- Hide graph from clap help  

## 8. Handoffs

| To | What |
|----|------|
| deferred.md | Strike graph install residual on ship |
| T198 | Exit 2 normative |
| T201 | JSON FEATURE_UNAVAILABLE via fail_api |
| T204 | Soft optional-feature help group |
| Future packaging | release graph-on artifact / binstall (F29/F31) |

## 9. Implementation sketch

### 9.1 Preferred path (docs-only A — A2=no)

1. No Cargo.toml default change.  
2. INSTALL: primary F27; slim = bare locked install; **Release binary graph-off** note.  
3. CAPABILITIES full graph honesty; CONTRIBUTING matrix.  
4. CI: add **required** `cargo nextest run -p ai-brains-cli --features graph` (F14).  
5. F9 grep test on both stubs.  
6. CHANGELOG + gate.

### 9.2 Optional Cargo default (A2=yes — only after F2)

```toml
[features]
default = ["graph"]
graph = ["dep:ai-brains-graph", "ai-brains-retrieval/graph"]
```

+ F13 required `--no-default-features` CI  
+ INSTALL slim uses `--no-default-features`  
+ evidence/size-measure.md  

### 9.3 INSTALL honesty block (sketch)

```markdown
### Recommended (graph enabled)
cargo install --path crates/ai-brains-cli --locked --features graph

### Slim / capture-focused (graph exits 2)
cargo install --path crates/ai-brains-cli --locked
# (if Cargo default=graph: add --no-default-features)

### GitHub Release binary
`ai-brains.exe` from GitHub Releases is currently a **graph-off** build.
Use source install with `--features graph` for graph CLI.
```

## 10. Verification plan

| Layer | What |
|-------|------|
| Docs | AC1–2, AC5, AC12 |
| Hermetic | AC3–4, AC6 |
| CI | AC13 (F13 or F14) |
| Tree / deny | AC7; deny if A2=yes |
| Gate | AC10 |
| Claims | AC11 |

## 11. Stop-before

- Cargo default flip without F2 + F13  
- Release.yml graph flip without product go  
- Losing feature-off or feature-on CI coverage  
- Weakening exit 2  
- Capture tree break  

## 12. Suggested implement order

1. Confirm A2 preference (default: **docs-only A**).  
2. Optional A1 size measure (required only if considering flip).  
3. INSTALL + CAPABILITIES + CONTRIBUTING + README.  
4. F9 grep test; optional A2 Cargo + F13.  
5. F14 or F13 CI step.  
6. CHANGELOG + full gate + deferred strike.

## 14. AI fold-in disposition (2026-08-03)

| ID | Source | Disposition |
|----|--------|-------------|
| AI1 §1–5 | INSTALL primary, F2 gate, hint, exit 2, docs | **Accept** — core freezes |
| **M1** | Release binary graph-off honesty | **Accept** → F5, AC12; no release.yml flip DoD |
| **M2** | Absolute ≤8 MB; drop %; measure release-shaped | **Accept** → F2 |
| **M3** | Feature-off CI hard if default=graph | **Accept** → F13 hard DoD |
| **M4** | Stubs already match; grep guard | **Accept** → F9 |
| **M5** | Plan B1 branch on A2 | **Accept** → F33 |
| **M6** | CAPABILITIES all graph refs | **Accept** → F7 |
| **M7** | Feature-on CI hard if A2=no | **Accept** → F14 hard DoD |
| AI2 strong rec | Prefer docs-only A | **Accept as preferred** → F1 |
| L1 | Compile time note | Affirm F2 note |
| L3 | retrieval/graph with default | Soft F25 |
| L8 | Only ai-brains.exe | Affirm F28 |
| L10 | 22 MB = graph-off | Baseline fixed |
| L5 | dev-check gap | Soft |
| L2/L4/L6/L7/L9 | Affirm / no change | Affirm |
| **O1** binstall | Out | F31 |
| **O2** slim feature | Decline | F32 |
| **O3** stub dedupe | Soft residual | F30 |
| **O4** deny if flip | Soft→with F2 | F2(d) |

**Baseline:** AI2 11/11 confirmed. **Verdict target:** M1–M7 folded; preferred path = docs-only A + graph-on CI + release honesty.
)
