# T222 — Graph-on install path

- **Track ID:** T222-GraphOnInstallPath
- **Phase:** T217–T232 post-audit CLI quality (P2)
- **Status:** ✅ **Completed** (PR #122 `c1ac594`, 2026-08-10)
- **Depends on:** **T200** install honesty (docs SOOT + CI graph-on smoke; A2=no Cargo default); **T198** FEATURE_UNAVAILABLE exit 2; **T213** doctor `graph_density` (SQL-only)
- **Blocks / feeds:** **T232** density remediation branching (capability-aware rebuild vs reinstall); operator PATH usefulness for `graph *`
- **Category:** INFRA / DOCS / FEATURE (light doctor check)
- **Source:** CLI audit 2026-08-05 — Graph-off install usefulness **3** / honesty 9; series README row T222
- **Deferred absorbed:** deferred.md “Graph-off PATH usefulness 3”; residual “operators follow INSTALL but PATH binary still graph-off via local build scripts”
- **Not absorbed:** Density remediation text branching (**T232**); auto `graph rebuild`; Cozo INFO regression (T208 closed); MSI / dual release artifact / binstall; projector edge rewrite; clap pin bump; release.yml graph-on flip without separate product go + F2 size gate
- **Research date:** 2026-08-10 (live PATH dogfood + T200 re-read + Cargo features docs + script audit)
- **AI fold-in:** 2026-08-10 — AI1 affirms F1–F28 / AC1–AC15 (probe discrimination, matrix count, T232 handoff). AI2 **M1–M6 hard**; **L1–L2 elevated**; **L3 soft**; **L4–L6 notes**; **O2/O3 hard**; **O1/O4 soft**. Disposition **§11**.
- **Ledger:** plan-only until go (`ledgerful ledger start T222-graph-on-install-path --category INFRA`)

## 1. Objective

1. **Close the PATH gap:** Common local install/rebuild paths that land `ai-brains` on PATH must produce a **graph-capable** binary (or an unmistakable one-command upgrade), so `ai-brains graph *` is useful without hunting docs.  
2. **Preserve T198/T200 honesty:** Feature-off still exits **2** + `FEATURE_UNAVAILABLE` + INSTALL SOOT reinstall hint; capture path remains independent of graph.  
3. **Prefer scripts over Cargo default flip:** Keep workspace `default = []` unless product **go** re-runs T200 **F2** size gate (Δ ≤ **8 MB** absolute) and explicitly opts into `default = ["graph"]`.  
4. **Doctor discoverability:** Soft `graph_feature` check reports `available|unavailable` so agents/ops see binary capability without guessing (feeds T232).  
5. **Zero new crates; no dep bumps.**

## 2. Live baseline (re-scan 2026-08-10)

### 2.1 Dogfood (this machine)

| Probe | Result |
|-------|--------|
| `where.exe ai-brains` | `C:\Users\RyanB\.cargo\bin\ai-brains.exe` |
| Binary size / date | **~22.9 MB** release; last write **2026-08-05** |
| `ai-brains graph update` | **`FEATURE_UNAVAILABLE`** exit **2** + SOOT reinstall hint (graph-off) |
| `ai-brains doctor` `graph_density` | **warn** sparse (`nodes=1326 edges=110 E/N=0.083 pinned=16788`); remediation **`ai-brains graph rebuild`** (dead-end on this binary — **T232**) |
| INSTALL primary SOOT | Already `cargo install --path crates/ai-brains-cli --locked --features graph` (**T200**) |
| Cargo `ai-brains-cli` features | `default = []`; `graph = ["dep:ai-brains-graph", "ai-brains-retrieval/graph"]` |
| `scripts/Build-AIBrains.ps1` | `cargo build --release -p ai-brains-cli -p ai-brainsd` — **no `--features graph`** → installs graph-off to cargo bin |
| `scripts/build.ps1` | Same gap (release build + copy to cargo bin, graph-off) |
| `release.yml` | Builds `ai-brains-cli` **without** `--features graph` (T200 honesty: Release graph-off) |
| CI | Workspace default graph-off; required `--features graph` nextest `-E 'test(graph)'` (T200 F14) |

### 2.2 Root cause (honesty)

| Layer | Reality | Gap |
|-------|---------|-----|
| Docs (T200) | Recommend `--features graph` | Operators who use **local scripts** never hit SOOT |
| Cargo default | `default = []` (A2=no by design) | Bare `cargo build -p ai-brains-cli` = graph-off |
| Local “install to PATH” scripts | Copy release binary without feature | **PATH usefulness 3** despite honest docs |
| GitHub Release | Intentional graph-off | Documented; not this track’s flip |
| Doctor density | SQL works graph-off; remediation says `rebuild` | Capability mismatch → **T232** |

### 2.3 Code / file touch map

| Path | Role in T222 |
|------|----------------|
| `scripts/Build-AIBrains.ps1` | **Hard:** pass `--features graph` on CLI build; post-install probe |
| `scripts/build.ps1` | **Hard:** same feature + probe (or delegate comment to Build-AIBrains) |
| Optional `scripts/Install-AIBrains.ps1` | **Soft:** thin wrapper around F27 SOOT `cargo install … --features graph` if useful; not required if Build-* + INSTALL suffice |
| `crates/ai-brains-cli/src/commands/doctor.rs` | Soft check **`graph_feature`** (compile-time `cfg!(feature = "graph")`) |
| Doctor matrix unit (order list) | Code **12 → 13**; `Vec::with_capacity(12) → 13`; full order F10 |
| `Docs/INSTALL.md` | Point operators at fixed scripts; keep F27 SOOT; Release honesty unchanged |
| `Docs/OPERATIONS.md` | Graph-on local rebuild path; doctor `graph_feature` note |
| `Docs/CAPABILITIES.md` | Docs delta **11 → 13** (was stale: missing `harness_wiring` + new `graph_feature`); ordered list F10 |
| `CONTRIBUTING.md` | Script matrix row: local PATH scripts = graph-on |
| `CHANGELOG.md` | T222 entry |
| `governed_common.rs` | **Hard:** `GRAPH_REINSTALL_SOOT` constant (M4) |
| `main.rs` stubs | Use constant (smoke guard updates) |
| `graph_density.rs` | Use constant inside `REMEDIATION_EMPTY_LAG` install substring only — **no** remediation branching (T232) |
| `crates/ai-brains-cli/Cargo.toml` | **Only if A2=yes** after F2 measure |
| `.github/workflows/ci.yml` | **Only if A2=yes:** required `--no-default-features` (T200 F13) |
| `.github/workflows/release.yml` | **Out of DoD** unless separate product go |

### 2.4 Dep pins (research 2026-08-10; AI2 re-verify)

| Item | Pin / note |
|------|------------|
| Rust | Workspace **1.95.0** (`rust-toolchain.toml`) |
| clap | Workspace **4.5** (lock may resolve 4.6.x); crates.io **4.6.6** (2026-08) — **no bump** (soft residual) |
| rusqlite | Workspace **0.39.0** SQLCipher; crates.io **0.40.2** available — **explicitly deferred** (M6; no SQLCipher surface in T222) |
| cargo-nextest / deny / audit | Tooling pins in `Docs/ci-tooling.md`; not bumped this track |
| Graph backend | Workspace SQLite-native + Cozo proxy in `ai-brains-graph`; no crates.io `cozo` dep on CLI path — graph-on compile cost is local crate, not remote Cozo product |
| Cargo features | [Cargo Book](https://doc.rust-lang.org/cargo/reference/features.html): `default` enables listed features; `--no-default-features` / `default-features = false` for slim; features must stay additive |
| Zero new crates | Required |

## 3. Research summary

| Finding | T222 application |
|---------|------------------|
| T200 closed **docs-only A** (A2=no) with strong AI2 preference | Do **not** flip Cargo default by default; re-open only behind F2 + product go |
| T200 F2 absolute size gate Δ ≤ **8 MB** | If measuring A2, same release-shaped compare; record `evidence/size-measure.md` |
| PATH binary is graph-off while density warns rebuild | Install scripts are the high-leverage fix for usefulness **3** |
| Cargo features additive / default behavior | Flip to `default = ["graph"]` is SemVer-safe for apps but **CI dual matrix** + slim path wording cost (T200 F13) |
| Doctor has no `Info` severity | Use **Ok** with message (+ optional remediation) so graph-off does **not** alone force `degraded` (parity with harness_wiring “info”); density still warns |
| T232 owns capability-aware density remediation | T222 only adds capability **signal**; does not rewrite sparse/empty remediations |

## 4. Frozen decisions (F1–F28)

| ID | Decision |
|----|----------|
| **F1 — Preferred product path = B (scripts)** | Local install/rebuild scripts that put `ai-brains` on PATH **must** build CLI with `--features graph`. Docs already recommend F27 SOOT; scripts must match. |
| **F2 — Cargo default = A optional gated** | Keep `default = []` unless product **go** after size measure: absolute release binary Δ **≤ 8 MB** (T200 F2), capture tree clean, deny green, nextest graph-on green. If flip: INSTALL slim uses `--no-default-features`; CI adds required `--no-default-features` nextest (T200 F13). **Plan default: A2=no.** |
| **F3 — Release.yml out of DoD** | Do **not** flip GitHub Release `ai-brains.exe` to graph-on without separate product go + F2. Keep INSTALL Release honesty. |
| **F4 — INSTALL SOOT pin (T200 F27)** | Primary remains exact string: `cargo install --path crates/ai-brains-cli --locked --features graph`. Implemented as Rust constant **F27**; INSTALL/docs quote the same text. |
| **F5 — Build-AIBrains.ps1 hard** | `cargo build --release -p ai-brains-cli --features graph -p ai-brainsd` (daemon has no graph feature; only CLI needs it). Copy to cargo bin as today. |
| **F6 — build.ps1 hard** | Same: CLI release build with `--features graph` before PATH copy. **L1:** `build.ps1` is a minimal subset of `Build-AIBrains.ps1` (no daemon stop/restart); keep both graph-on this track; deprecation/fold soft residual. |
| **F7 — Post-install probe (scripts) — M3/L2/O2 hard** | Fail closed if binary is graph-off. **Env preconditions (hard):** set `AI_BRAINS_VAULT_PATH` to a **known-missing temp path** (never operator vault); clear/ignore project context as needed so probe cannot migrate or rebuild a real vault. **Primary probe (O2 preferred):** `ai-brains doctor --json` → find check `name=graph_feature` with `message=available` (side-effect-free; lands same PR as F9). **Secondary / fail-closed stub probe:** `graph update` against the missing vault path — fail only if exit **2** **and** output matches `FEATURE_UNAVAILABLE` (AI1 blind spot + M3). Accept non-2 or non-FEATURE_UNAVAILABLE as “feature present” on secondary. Document in script comments. |
| **F8 — Optional Install-AIBrains.ps1** | Soft: thin PowerShell wrapper around F27 SOOT; not required if F5/F6 + INSTALL suffice. **No** `Install-AIBrains.ps1` exists today (AI2 verified). |
| **F9 — Doctor `graph_feature` (option C minimal)** | New soft check name **`graph_feature`**. Message **`available`** \| **`unavailable`**. Severity **Ok** always (info-level — never alone fail/degraded). When unavailable, set `remediation` to **`GRAPH_REINSTALL_SOOT`** (F27). Pure `cfg!(feature = "graph")` — no vault, no graph crate link in feature-off builds. |
| **F10 — Doctor matrix order + counts (M1/M2/M5 hard)** | Insert `graph_feature` **immediately before** `graph_density`. **Code delta 12 → 13:** (1) `Vec::with_capacity(12) → 13` (`doctor.rs` ~83); (2) matrix unit `expected` array length 12 → 13; (3) `assert_eq!(report.checks.len(), 12, …)` → 13. **Full post-insertion order (pin):** `vault_exists`, `vault_open`, `schema_readable`, `cipher_page`, `daemon_reachable`, `backup_recent`, `recovery_kit_event`, `recovery_kit_file`, `zero_key_escape`, **`graph_feature`**, `graph_density`, `harness_wiring`, `integrity`. **Docs delta 11 → 13 (M1/O3):** CAPABILITIES currently says **11** and **omits `harness_wiring`**; ship **13** with full ordered list including **both** `harness_wiring` and `graph_feature` (not only the new check). |
| **F11 — Density remediations not rewritten** | Branching rebuild vs reinstall stays **T232**. T222 may only substitute the **install SOOT substring** in `REMEDIATION_EMPTY_LAG` via F27 constant (no logic change). |
| **F12 — Capture independence** | `graph_feature` uses compile-time cfg only; no `ai-brains-graph` dep in feature-off binary; capture tree CI unchanged. |
| **F13 — Exit codes** | Unchanged: feature-off graph * → 2; doctor ok\|degraded → 0 default. |
| **F14 — Contracts** | No new DTO field required — reuse `HealthCheck` row. schema_version stays **1**. |
| **F15 — Zero new crates / no clap/rusqlite bump** | Required. M6: rusqlite **0.40.2** available, **deferred**; clap **4.6.6** available, **deferred**. |
| **F16 — Claims honesty** | Do not claim “every binary always has graph.” Document: source recommended + local scripts graph-on; slim + Release may be graph-off. |
| **F17 — High findings if…** | Scripts still graph-off after ship; probe not enforced / probe touches real vault (L2); Cargo flip without F13 CI; release flipped without go; capture→graph edge; doctor `graph_feature` alone hard-fails; density branching rewritten here; fourth divergent SOOT string (M4); CAPABILITIES left at 11 or still missing harness_wiring (M1/O3). |
| **F18 — Soft residuals** | MSI / dual artifact / binstall; release graph-on; Cargo default flip deferred; T232 remediation branch; build.ps1 deprecation into Build-AIBrains (L1); human-format checks=N unit assert (L3); skill one-liner; optional doctor JSON probe-only without secondary graph update (O4 partial). |
| **F19 — Parallel-friendly** | Scripts + doctor check + docs; low conflict with T223 env-warn / T225 backup. Coordinate if T232 lands same week on `graph_density.rs` remediations. |
| **F20 — Series order** | After T224 close. Peer **T232** next for density remediation honesty. |
| **F21 — Plan-only** | No production code until user **go**. |
| **F22 — Ledger** | On go: `ledgerful ledger start T222-graph-on-install-path --category INFRA` (or `FEATURE` if A2=yes). |
| **F23 — Review** | INFRA primary. Cross-model **required** if A2=yes (Cargo + CI); optional if scripts+doctor+docs only. |
| **F24 — Implement order** | (0) Preflight + ledger. (1) Red: SOOT constant smoke + doctor matrix **13** order (M1/M5). (2) Green: `GRAPH_REINSTALL_SOOT` + `check_graph_feature` + wire stubs/density substring + matrix (M4). (3) Scripts F5/F6 + F7 probe (doctor JSON primary, controlled vault env). (4) Docs AC10/O3 (11→13 + full list). (5) Optional A2 only if go. (6) Manual AC14. (7) Full gate. |
| **F25 — Determinism** | Doctor check pure cfg; no timestamps in check name/message beyond existing report `generated_at`. |
| **F26 — Size measure (A2 only)** | Same shape as T200: `cargo build --release -p ai-brains-cli` ± `--features graph`; record absolute Δ in `conductor/tracks/trackT222-graph-on-install-path/evidence/size-measure.md`. |
| **F27 — Install SOOT constant (M4 hard — option a)** | Extract `pub const GRAPH_REINSTALL_SOOT: &str = "cargo install --path crates/ai-brains-cli --locked --features graph";` in `governed_common.rs` (alongside `FEATURE_UNAVAILABLE`). **Consumers this track:** both `main.rs` feature-off stubs; `check_graph_feature` remediation; `REMEDIATION_EMPTY_LAG` install substring in `graph_density.rs`. Update smoke grep guard to pin the **constant** (not only two stub line contexts). Script comments cite the same text. **Do not** leave a fourth hard-coded copy. |
| **F28 — Daemon** | `ai-brainsd` has no graph feature; scripts may build daemon without graph flags. |
| **F29 — AI1 affirm** | Scripts + probe discrimination + matrix 13 + T232 handoff + A2=no default — all above. |
| **F30 — AI2 M1–M6** | Folded hard into F7/F10/F15/F27 and AC7/AC10/AC16–AC18. |

## 5. Residual disposition

| Residual | Disposition |
|----------|-------------|
| Graph-off PATH usefulness 3 | **Absorb** (scripts + doctor feature signal) |
| INSTALL already recommends graph | Keep; align scripts |
| Cargo default flip | **Optional A2** behind F2 + go; default **no** |
| Release graph-on | **Out** (F3) |
| Density remediation rebuild vs reinstall | **T232** |
| Cozo INFO | Closed T208 |
| Dual release artifact / binstall / MSI | Out / packaging residual |

## 6. Acceptance criteria

| AC | Criterion | Proof |
|----|-----------|-------|
| **AC1** | `Build-AIBrains.ps1` builds CLI with `--features graph` | Diff + script review |
| **AC2** | `build.ps1` builds CLI with `--features graph` | Diff |
| **AC3** | Scripts fail closed when probe detects graph-off (F7: doctor JSON not available **and/or** exit 2 + FEATURE_UNAVAILABLE); probe env uses known-missing vault path | Diff + manual |
| **AC4** | Doctor JSON includes check `name=graph_feature` with message `available` or `unavailable` | Unit / hermetic doctor |
| **AC5** | Feature-on build: `graph_feature` severity Ok, message available, no remediation | Unit cfg or hermetic with features |
| **AC6** | Feature-off build: `graph_feature` severity Ok, message unavailable, remediation **equals** `GRAPH_REINSTALL_SOOT` | Unit cfg(not feature) |
| **AC7** | Matrix unit: full **13**-name order per F10; `with_capacity(13)`; `checks.len()==13` | Unit update |
| **AC8** | Feature-off `graph *` still exit 2 + FEATURE_UNAVAILABLE (T198 regression) | Existing smoke |
| **AC9** | Capture tree still forbids capture→graph | CI / local tree |
| **AC10** | INSTALL/OPERATIONS/CAPABILITIES/CONTRIBUTING/CHANGELOG honesty; CAPABILITIES **13** checks lists **`graph_feature` + `harness_wiring`** (docs 11→13) | Diff |
| **AC11** | If A2=yes: size Δ ≤8 MB + F13 CI + slim `--no-default-features` docs | evidence + CI |
| **AC12** | If A2=no: plan/spec note docs+scripts path; no Cargo default change | Spec |
| **AC13** | Full gate green | Process |
| **AC14** | Manual: after script rebuild, PATH not FEATURE_UNAVAILABLE on graph; doctor `graph_feature=available` | Manual evidence |
| **AC15** | Claims: no “always graph”; Release graph-off note preserved | Review |
| **AC16** | `GRAPH_REINSTALL_SOOT` is single SOOT; stubs + doctor remediation + empty-lag install substring consume it; smoke guards constant | Diff + smoke |
| **AC17** | Probe never opens operator vault (temp missing path precondition) | Script review |
| **AC18** | Dep pins: clap/rusqlite not bumped; rusqlite 0.40.2 explicitly deferred in plan/spec research | Spec + lock unchanged |

## 7. Non-goals

- Auto `graph rebuild`  
- Rewriting `graph_density` remediations (T232)  
- Flipping `release.yml` without separate go  
- Dual `ai-brains-graph.exe` artifact  
- MSI / notarization / binstall  
- Cargo default flip without F2  
- Dep upgrades (clap, rusqlite, …)  
- Making graph required for capture  
- Hiding `graph` from clap help  

## 8. Handoffs

| To | What |
|----|------|
| **T232** | Use `graph_feature` / `cfg!(feature = "graph")` to choose rebuild vs reinstall remediations |
| deferred.md | Strike PATH usefulness 3 on ship; leave density remediation row for T232 |
| T200 | SOOT + exit 2 + CI graph-on unchanged unless A2 |
| Packaging residual | Release graph-on / MSI later |

## 9. Implementation sketch

### 9.1 Scripts (always)

```powershell
# Build-AIBrains.ps1 / build.ps1
cargo build --release -p ai-brains-cli --features graph -p ai-brainsd
# … copy to cargo bin …

# F7: never touch real vault (M3/L2) — unique owned probe dir; assert absent; cleanup only this dir
$probeDir = Join-Path $env:TEMP ("ai-brains-graph-probe-" + [guid]::NewGuid().ToString("N"))
New-Item -ItemType Directory -Path $probeDir -Force | Out-Null
$probeVault = Join-Path $probeDir "missing.db"
# assert -not (Test-Path $probeVault); set AI_BRAINS_VAULT_PATH = $probeVault; restore + Remove-Item $probeDir in finally

# Primary: doctor --json graph_feature (O2) — stdout only + --log-format off
$docOut = & $OutputBin --log-format off doctor --json 2>$null | Out-String
$gfOk = $false
try {
  $report = $docOut | ConvertFrom-Json
  $gf = $report.checks | Where-Object { $_.name -eq 'graph_feature' } | Select-Object -First 1
  if ($gf -and $gf.message -eq 'available') { $gfOk = $true }
} catch { }

# Secondary fail-closed: feature-off stub only (exit 2 AND FEATURE_UNAVAILABLE)
$probe = & $OutputBin --log-format off graph update 2>&1 | Out-String
$featureOff = ($LASTEXITCODE -eq 2 -and $probe -match 'FEATURE_UNAVAILABLE')
if ($featureOff -or -not $gfOk) {
    Write-Error "Installed binary is graph-off or graph_feature not available; expected --features graph build"
    exit 1
}
```

### 9.2 Doctor check + SOOT (always)

```rust
// governed_common.rs
pub const GRAPH_REINSTALL_SOOT: &str =
    "cargo install --path crates/ai-brains-cli --locked --features graph";

fn check_graph_feature() -> HealthCheck {
    if cfg!(feature = "graph") {
        HealthCheck::ok_msg("graph_feature", "available")
    } else {
        HealthCheck::new(
            "graph_feature",
            CheckSeverity::Ok,
            Some("unavailable".into()),
            Some(GRAPH_REINSTALL_SOOT.into()),
        )
    }
}
// Insert before graph_density; with_capacity(13); matrix order F10.
```

### 9.3 Optional A2 (only after go + F2)

```toml
[features]
default = ["graph"]
graph = ["dep:ai-brains-graph", "ai-brains-retrieval/graph"]
```

+ CI `--no-default-features` nextest + INSTALL slim wording.

## 10. Verification plan

| Layer | Commands |
|-------|----------|
| Units | Doctor matrix order; `graph_feature` available/unavailable via cfg dual compile (existing smoke pattern) |
| Regression | `graph__default_build__prints_hint`; exit contract FEATURE_UNAVAILABLE |
| Script | Manual rebuild via Build-AIBrains; probe |
| Docs | INSTALL SOOT still F27; Release honesty; CAPABILITIES 13-check matrix |
| Gate | `cargo fmt --check`; clippy workspace `-D warnings`; nextest workspace; deny; audit; ledgerful verify |

## 11. AI fold-in disposition (2026-08-10)

| ID | Source | Severity | Disposition |
|----|--------|----------|-------------|
| AI1 affirm | AI1 T222 | — | **Accept** — scripts + probe + doctor signal + A2=no + T232 handoff match plan |
| AI1 probe exit discrimination | AI1 §3.1 | Medium | **Accept** → **F7** (exit 2 **and** FEATURE_UNAVAILABLE only) |
| AI1 matrix 12→13 | AI1 §3.2 | Medium | **Accept** → **F10/AC7** |
| AI1 T232 handoff | AI1 §3.3 | Medium | **Accept** → **F11** already; reaffirmed |
| AI1 T224 block | AI-review.md preamble | — | **Ignore** — T224 closed PR #120; not this track |
| **M1** docs 11→13 + code 12→13 | AI2 | Medium | **Hard** → F10, AC7, AC10, O3 |
| **M2** `with_capacity(12)→13` | AI2 | Medium | **Hard** → F10 |
| **M3** probe false-negative / env | AI2 | Medium | **Hard** → F7, AC3, AC17 |
| **M4** SOOT constant (option a) | AI2 | Medium | **Hard** → F27, AC16 (not soft “if free”) |
| **M5** full 13-check order | AI2 | Medium | **Hard** → F10 order array |
| **M6** rusqlite 0.40.2 defer | AI2 | Medium | **Hard** → §2.4, F15, AC18 |
| **L1** build.ps1 vs Build-AIBrains | AI2 | Low | **Elevated note** → F6; deprecation soft residual |
| **L2** probe vault side effect | AI2 | Low | **Elevated hard** → F7 known-missing path (with M3) |
| **L3** human checks=N assert | AI2 | Low | Soft residual F18 |
| **L4** parallel F19 | AI2 | Low | Note only — verified |
| **L5** F25 pure cfg | AI2 | Low | Note only — verified |
| **L6** graph_feature new surface | AI2 | Low | Note only |
| **O1** human format works | AI2 | Opp | Note — no action |
| **O2** doctor --json probe | AI2 | Opp | **Hard preferred** primary probe → F7 |
| **O3** CAPABILITIES + harness_wiring | AI2 | Opp | **Hard** → F10/AC10 |
| **O4** script verifies doctor after probe | AI2 | Opp | Soft — covered by O2 primary probe |

## 12. Stop-before

- Cargo `default = ["graph"]` without F2 measure + F13 CI + user go  
- `release.yml` graph-on without product go  
- Rewriting density remediations in this track (belongs T232)  
- Capture independence regression  

## 13. Suggested ledger message (on go)

```text
ledgerful ledger start T222-graph-on-install-path --category INFRA --message "Graph-on local install scripts + doctor graph_feature; keep Cargo default off unless F2"
```
