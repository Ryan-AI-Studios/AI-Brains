# T326 — `status` / `graph update` must not fake `pinned=0` on COUNT fail + workspace 0.1.4

- **Track ID:** T326-StatusPinnedCountFailOpen
- **Status:** **Planned** (Pending until **go**)
- **Category:** BUGFIX / UX / CHORE
- **Owner:** Grok
- **Source:** Last-PR Cursor Bugbot on [#237](https://github.com/Ryan-AI-Studios/AI-Brains/pull/237) (T320, `mergedAt` **2026-08-29T03:17:43Z**). Medium `3885361601`: when pinned-memory COUNT fails, glance invents `pinned=0` and still runs density assessment. Empty graph on a pin-rich vault can then show `live`/`skip` + `pinned=0` instead of failing that section open. Doctor already skips. Owner also asked this track to **bump the workspace version** (0.1.3 → **0.1.4**).
- **Depends on:** T320 ✅ unified `status` (`status.rs`); T213/T308 `GatherResult::PinnedCountFailed`; T300 `graph_health_report`; T183 CHANGELOG / version banners; `#217` 0.1.3 analog
- **Blocks / feeds:** Glance + `graph update` honesty when `memory_projection` COUNT fails. `ai-brains --version` reports **0.1.4** after PATH install.
- **Absorbs:** `#237` Cursor `pulls/comments/3885361601` (still true on HEAD `9119c74`); `graph.rs:445–458` same fake 0; T304 residual R6 Docs banners still **0.1.2**; owner version-bump request
- **Not absorbed (DoD):** T307 Blocked; clap 5; T308 floors; T320 four-section compose / envelope keys (except this error path); desktop **0.1.2**; path-dep `"0.1.0"`; git tag; cargo-release; crates.io; T310 F15 `ai-brainsd --version`; protocol `schema_version`
- **Research date:** 2026-08-29 (plan-write product HEAD `9119c74` T325 `#247`; `origin/main` = same). Snapshot — **re-verify at execute**.
- **Ledger:** planning DOCS TX `5fd70b52-1a16-4971-ab0f-684c553a4c17`. Minted with T316 planning DOCS `66b597f7-faf9-4f3e-bb06-6af72811bdc6`. Implement starts a **BUGFIX** TX on **go**.
- **Isolation:** Do **not** implement until **go**. Do **not** `cargo install`. Do **not** retune floors. Do **not** grow `doctor.rs` (skip arm is SOOT). Do **not** grow `project.rs` / `sync.rs` / `governed_common.rs` / `session_chrome.rs`. Do **not** print or commit `AI_BRAINS_KEY`. Do **not** git-tag or `cargo publish`.

---

## 1. Objective

1. **Pinned COUNT fail is fail-open, not fake zero.** `GatherResult::PinnedCountFailed` must not feed `assess_graph_density` with `pinned_memories: 0`. Glance graph section emits T320 F4 `error` (other sections still emit; exit **0**). `graph update` must not report `live`/`skip` from a synthetic empty-lag — fail-closed `Err` like `TablesMissing`.
2. **Copy doctor SOOT.** Doctor skip message already exists. Do not add a 16th doctor check. Do not change skip → fail.
3. **Workspace 0.1.4.** Cut Keep a Changelog Unreleased into `[0.1.4]` (0.1.3 analog `#217`). `workspace.package.version` **0.1.3 → 0.1.4**. Catch up Docs banners still stuck at **0.1.2**.
4. **North star.** Capture independence: density gather honesty + release hygiene. No new events. Floors stay 0.50.

---

## 2. Live baseline (re-scan 2026-08-29)

### 2.1 Operator dogfood (this machine)

| Signal | Observation |
|--------|-------------|
| HEAD | `9119c74` T325 `#247` squash. Tree **DIRTY** conductor T325 Completed notes (absorb into this plan DOCS). Branch `track/T326-status-pinned-count-failopen`. `origin/main` = `9119c74` (ahead **0** at plan start). |
| PATH `ai-brains.exe` | `C:\Users\RyanB\.cargo\bin\ai-brains.exe` **26,897,408** B; LastWriteTime **2026-08-27 8:21:55 PM**; `ai-brains 0.1.3`. T263/T293/T311 **on PATH**. T312–T325 **not**. `status` clap **unrecognized subcommand** exit **1** (PATH-behind T320). **Do not `cargo install`.** |
| `preflight --summary` (PATH) | Pinned **4674**. In-context **0/0/0**. `Total Word Count: 717` (PATH-behind T315 `Budget window words:`). **Not this DoD.** |
| Source `cargo run -q -p ai-brains-cli -- status --format json` | Exit **0**. Envelope `schema_version: 1`. daemon `Running`. doctor `degraded` graph_density sparse `E/N=0.423 pinned=51768` (COUNT **succeeds** on live vault — hole is the fail path). graph `status=sparse` `nodes=64766` `edges=27370`. nightly last **2026-08-29T07:09:01Z**. Graph-off remediator is `GRAPH_REINSTALL_SOOT` (this `cargo run` omitted `--features graph`) — **not** the hole. |
| Live COUNT | Succeeds. `PinnedCountFailed` is **not** reproduced on this vault. Proof is hermetic inject + gather unit `gather_density_snapshot__pinned_table_missing__pinned_count_failed`. |
| Last GitHub PR | [#247](https://github.com/Ryan-AI-Studios/AI-Brains/pull/247) T325. `mergedAt` **2026-08-30T00:23:50Z**. Issue/review/inline comments **[]**. Open PRs: **none**. `#237` Bugbot **still true** (`status.rs:329–340`). **No T327 from Cursor.** |
| rustc | **1.95.0** |
| Ledger | 0 pending / 0 drift at scan (before this DOCS TX). Impact **LOW** (conductor-only dirty). |
| Hotspots | `project.rs` **#1** (3.615) — **do not touch.** `sync.rs` **#2**. `governed_common.rs` **#3**. `session_chrome.rs` **#6**. `status.rs` / `graph.rs` / `graph_density.rs` **not** top-10. |
| Line counts (physical) | `status.rs` **808**; `commands/graph.rs` **1731**; `graph_density.rs` **733**; `doctor.rs` **1855**. |
| `ISSUES.md` | **Does not exist.** |
| git tags `v0.*` | **None.** Do not mint a tag this track. |

### 2.2 Why fake `pinned=0` is still the hole

| Layer | Truth |
|-------|--------|
| Gather already distinguishes COUNT fail | `graph_density.rs:63–68` `PinnedCountFailed { nodes, edges, memory_nodes }`. Gather `:321–333` returns that variant when `memory_projection` COUNT `Err`. Unit `:659–681` proves the variant. |
| Doctor is SOOT | `doctor.rs:901–904` `HealthCheck::skip("graph_density", "pinned memory count failed (cannot assess empty_lag without pins)")`. 15-check matrix frozen. |
| Glance ignores the variant | `status.rs:329–340` builds `GraphDensitySnapshot { pinned_memories: 0, .. }` then `assess_graph_density`. Outer T320 F4 `Err → graph.error` (`:197–207`) **never fires** for this variant. |
| `graph update` same fake | `commands/graph.rs:445–458` (feature-gated `#[cfg(feature = "graph")]`) invents snap with `pinned_memories: 0` then assesses. `TablesMissing` already `return Err(...)` (`:439–443`). |
| Assessor skip arm is the Bugbot case | `graph_density.rs:260–270`: `pinned < MIN_PINNED` **and** `nodes==0` **and** `edges==0` → `DensityVerdict::Skip`, `density: "skip"`, **`status: "live"`**. Fake `pinned=0` on an empty graph **cannot** fire empty_lag (`pinned >= 100` required at `:188`). Result: pin-rich vault whose COUNT failed looks like a small empty vault that is `live`. |
| Sparse coincidence is not honesty | Sparse (`:218`) does **not** use pin count. A live sparse vault with COUNT fail would still report `sparse` **and** human `pinned=0`. JSON glance omits `pinned` (`GraphSection.pinned` is `#[serde(skip)]`) but still **assesses**. `graph update` JSON **includes** `pinned_memories: 0`. |
| T320 F4 already has the envelope | Section `error` replaces other keys. Human `format_status_graph_line` already prints `graph: error={err}` (`:479–481`). Missing is routing `PinnedCountFailed` into that path. |

### 2.3 Code truth (opened)

| Item | Location | Notes |
|------|----------|-------|
| Glance match | `status.rs:324–347` `build_graph_section` | `TablesMissing → Err("graph tables missing")`. `PinnedCountFailed → Ok(assess fake 0)`. `Ok(snap) → assess`. |
| Glance envelope | `status.rs:195–207` | `build_graph_section` `Err` → `GraphSection { error: Some(e), ..None }`. **Reuse.** |
| Human graph line | `status.rs:479–491` | Error arm exists. Happy path `pinned={pinned}`. |
| Glance tests | `status.rs:650–668` | `status_envelope__graph_err__error_keeps_others` covers **tables-missing** string, not COUNT fail. |
| Graph health | `graph.rs:35–48` `GraphHealthOutput` | Required i64 `pinned_memories`. T300 AC5 keys freeze (`:1118–1128`). |
| Graph health builder | `graph.rs:429–479` | Private. Callers `:520` / `:539` / `:769` + tests `:1083` / `:1092` / `:1116`. |
| Gather | `graph_density.rs:305–342` | **Do not change** success path / variant shape. |
| Assessor | `graph_density.rs:175–286` | **Do not change** floors or skip arm. |
| Doctor skip | `doctor.rs:901–904` | **Do not edit** (copy-not-share the skip phrase into a const). |
| `mod graph` | `commands/mod.rs:25–26` | `#[cfg(feature = "graph")]`. Glance + `graph_density` always compile. |
| Workspace version | `Cargo.toml:31` | `"0.1.3"`. Members `version.workspace = true` except desktop. |
| Lock workspace pkgs | `Cargo.lock` e.g. `ai-brains-cli` **0.1.3** | Refresh via cargo after toml bump (analog `#217` 44-line lock). |
| clap `--version` | `env!("CARGO_PKG_VERSION")` | No hardcoded `"0.1.3"` in CLI src. |
| Desktop | `apps/desktop/src-tauri/Cargo.toml:3` + `package.json` + `tauri.conf.json` | Independent **0.1.2**. `#217` did **not** bump. **Freeze.** |
| Path deps | e.g. `ai-brains-store/Cargo.toml` `version = "0.1.0"` | Path+caret; `#217` did not bump. **Freeze.** |
| Docs banners | CAPABILITIES / INSTALL / README / RELEASE-CHECKLIST / RELEASE-CLAIMS / SECURITY-LIMITS still **0.1.2** | T304 R6; T183 L6 manual banners. `#217` skipped them. |
| Banner script | `scripts/check-version-banners.ps1` | Cargo.toml vs CHANGELOG `## [<version>]`. Soft warn default; `-Strict` opt-in (T185). |

### 2.4 Dependency / standards research (2026-08-29)

| Pin | Workspace / lock | Action |
|-----|------------------|--------|
| clap | ws `"4.5"` / lock **4.6.1** / crates.io **4.6.6** (2026-08-06) | **No bump.** clap 5 **forbidden**. |
| rusqlite | exact **0.40.2** (crates.io **0.40.2**) | **No bump.** |
| serde_json | lock **1.0.150** | **No bump.** |
| tokio | ws `"1.53"` / lock **1.53.1** | **No bump.** |
| workspace version | **0.1.3** | **Bump to 0.1.4** (this track). |
| rustc / edition | **1.95.0** / **2024** | Unchanged. |
| New crates | — | **Zero.** No `cargo-release` / `cargo-workspaces`. |

**Fail-open / metric honesty:**

| Source | What we take | What we decline |
|--------|----------------|-----------------|
| Live doctor skip + T320 F4 | COUNT fail ≠ zero; section `error` already exists | Invent `pinned=0` then assess |
| [clig.dev](https://clig.dev/) (prior T320 fetch) | Catch/rewrite errors; don't pretend success; JSON keys stable on the happy path | New required JSON keys; bitwise exits |
| Assessor `:260–270` | Empty graph + `pinned=0` is `live`/`skip` — **the Bugbot scenario** | Changing skip-arm thresholds |
| `graph.rs` `TablesMissing` | Command `Err` when the report cannot be truthful | Adding `error` to frozen `GraphHealthOutput` |

**Version bump:**

| Source | What we take | What we decline |
|--------|----------------|-----------------|
| [Cargo Book workspaces](https://doc.rust-lang.org/cargo/reference/workspaces.html) (fetched 2026-08-29) | `[workspace.package] version` inherited via `version.workspace = true` (MSRV 1.64+) | Editing every member `Cargo.toml` |
| [Cargo Book SemVer](https://doc.rust-lang.org/cargo/reference/semver.html) (fetched 2026-08-29) | 0.y.z: **y** is major, **z** is minor. Patch **0.1.4** can include the Unreleased feature list at 0.x | 0.2.0 (owner asked “a version bump”; series analog is 0.1.2 / 0.1.3 patch) |
| T183 / [Keep a Changelog 1.1.0](https://keepachangelog.com/en/1.1.0/) | Unreleased at top; cut into a dated version; `Fixed` for this bug; human prose | Common Changelog; git-log dump |
| `#217` `2ed5b06` | `Cargo.toml` + lock workspace pkg versions + insert `## [0.1.3]` after Unreleased | git tag; crates.io; cargo-release CLI |
| T185 `check-version-banners.ps1` | After bump, CHANGELOG **must** have `## [0.1.4]` (else soft warn) | `-Strict` as a required gate; rewriting historical 0.1.1 claims |

**N/A:** SQLCipher page encrypt, schtasks, Windows service, llama.cpp `/health`, T307 reqwest/tower-http, T180 new required keys, clap 5.

**Could not verify:** Forcing live `memory_projection` COUNT fail without dropping the table (do not mutate the operator vault). Hermetic inject is the proof.

**ledgerful / ai-brains:** `preflight --summary` 0/0/0 vs **4674** pins; PATH `status` unrecognized; source glance COUNT succeeds; `ledgerful search "PinnedCountFailed"` → `doctor.rs:901`, `graph_density.rs:64`, `status.rs:329`, `graph.rs:445`; `graph_health_report` private in `graph.rs:429` with callers `:520/:539/:769`; `scan --impact` LOW (dirty conductor); hotspots `project.rs` #1 — **do not grow**. Semantic recall of this leftover is T320/T213 plan-audit chatter, not a DECISION pin.

---

## 3. Frozen decisions

| ID | Decision |
|----|----------|
| **F0 — Go gate** | Plan-only until user **go**. Planning is this DOCS TX. Implement starts a **BUGFIX** TX (version bump is in-scope hygiene, not a second CHORE track). |
| **F1 — Glance COUNT fail → error** | `PinnedCountFailed` returns `Err` from the graph-section builder (T320 F4). Do **not** construct `GraphDensitySnapshot { pinned_memories: 0 }` or call `assess_graph_density`. Human `graph: error=…`. JSON `graph.error` nonempty; other graph keys omitted. Exit **0**. Other sections still emit. |
| **F2 — graph update COUNT fail → Err** | Same variant: `return Err` analog `TablesMissing` (`graph.rs:439–443`). Do **not** emit `GraphHealthOutput` with `pinned_memories: 0`. Do **not** add an `error` key to frozen T213/T300 JSON. Feature-off `graph *` stays clap exit **2**. |
| **F3 — Doctor skip freeze** | `doctor.rs:901–904` skip arm **untouched**. 15-check matrix freeze. Do **not** grow `doctor.rs`. |
| **F4 — Message SOOT** | Skip phrase exact: `pinned memory count failed (cannot assess empty_lag without pins)`. `pub(crate)` const on `graph_density.rs`. Glance `error` **is** that string. `graph update` may prefix `Failed to count pinned memories: ` (TablesMissing analog). Doctor keeps the **literal** (copy-not-share — do not edit `doctor.rs`). |
| **F5 — Gather / assessor freeze** | Do **not** change `gather_density_snapshot` success path, variant fields, or `assess_graph_density` floors/priority. `MIN_EDGE_NODE_RATIO=0.50` frozen. |
| **F6 — T320 envelope freeze** | `schema_version: 1`; keys in T320 §5.1 frozen. This track only routes an existing variant onto the existing `error` path. No new required keys. `pinned` stays `#[serde(skip)]`. |
| **F7 — T320 F5 exit 0** | Glance still exit **0** when graph is `error`. Do **not** call `doctor::exit_code_for`. |
| **F8 — Helpers copy-not-share** | Extract `graph_section_from_gather` in `status.rs` and `graph_health_from_gather` in `graph.rs`. Do **not** import `commands/graph.rs` from `status.rs`. Do **not** genericize into `graph_density.rs` beyond the message const. |
| **F9 — No new CLI flag** | No `--fail-on-pinned-count`. |
| **F10 — No DTO / contracts** | No `ai-brains-contracts` change. Glance stays CLI-local. |
| **F11 — File growth** | Production: `status.rs` F8 arm + `graph.rs` PinnedCountFailed arm + `graph_density.rs` const. Tests next to existing units. **Do not** edit `doctor.rs`, `project.rs`, `sync.rs`, `governed_common.rs`, `session_chrome.rs`, `ranking.rs`, `hybrid.rs`, `pin.rs`, `forget.rs` production, `.github/workflows/ci.yml`. |
| **F12 — Capture independence** | Read-only honesty. No events. No models. No migrate. |
| **F13 — No unwrap/expect/panic** | Production. |
| **F14 — Test names** | `function_or_feature__condition__expected_result`. |
| **F15 — Implement TX is BUGFIX** | Planning is DOCS. |
| **F16 — Debt file** | `conductor/ISSUES.md` does **not** exist. |
| **F17 — PowerShell** | `;` not `&&`. |
| **F18 — last-PR Cursor** | `#247` empty → **N/A**. `#237` → **this**. **No T327.** |
| **F19 — Decline peers** | T307 Blocked; clap 5; H2; T240 F2; floor retune; T325 (Completed); Index SQL; csrf. |
| **F20 — PATH** | Do not `cargo install`. Hermetic / `cargo run` SoT. PATH `status` missing is T320 lag, not Complete-blocking. PATH `--version` stays **0.1.3** until owner install. |
| **F21 — Live vault** | Do **not** drop `memory_projection` or pin production DECISIONs. Hermetic inject is SoT. |
| **F22 — 80-net** | Two small match-arm helpers + const + tests. Do **not** grow `main.rs` test blocks. |
| **F23 — Version = 0.1.4** | `workspace.package.version` `"0.1.3"` → `"0.1.4"`. Cargo lock workspace package versions refresh via cargo (not hand-edited except as cargo rewrites). Analog `#217`. |
| **F24 — CHANGELOG cut** | Insert `## [0.1.4] — <go date>` immediately after `## [Unreleased]` (same `#217` trick so current Unreleased bullets become 0.1.4). Add a **Fixed** T326 bullet (either under Unreleased before the cut, or as the first 0.1.4 Fixed). Keep Unreleased heading. `scripts/check-version-banners.ps1` then sees `## [0.1.4]`. |
| **F25 — Docs banners** | Current product-version **headers** still **0.1.2** → **0.1.4**: `Docs/CAPABILITIES.md`, `INSTALL.md`, `README.md`, `RELEASE-CHECKLIST.md` (the “currently” line, not historical 0.1.1 rows), `RELEASE-CLAIMS.md` header, `SECURITY-LIMITS.md`. Example `VERSION=0.1.2` in `Docs/ci-tooling.md` + `scripts/generate-sbom.ps1` comment. Do **not** rewrite RELEASE-CLAIMS historical **0.1.1** sections. |
| **F26 — Desktop freeze** | `apps/desktop/**` stays **0.1.2**. Independent of workspace.package. |
| **F27 — Path-dep freeze** | Intra-workspace `version = "0.1.0"` on path deps stays. `#217` did not bump them. |
| **F28 — No tag / no publish / no cargo-release** | No `v0.1.4` git tag. No crates.io. No new release-tool crate. T185 `-Strict` not a required gate. |
| **F29 — No clap 5 / no rusqlite bump** | Standing. |
| **F30 — Isolation** | No `.env` rewrite; no live `nightly` without `--status`; no daemon stop; no `graph rebuild`; no live `backup create`. |
| **F31 — graph.rs feature gate** | `graph_health_from_gather` tests compile only with `--features graph`. Glance tests always compile. |
| **F32 — Stay-green T320 AC5** | `status_envelope__graph_err__error_keeps_others` (tables-missing) stays. New AC is COUNT-fail, not a rewrite of AC5. |
| **F33 — Stay-green T300 AC5** | `rebuild_with_daemon_state__format_json__health_keys` still requires the frozen keys on the **Ok** path. |
| **F34 — Stay-green gather unit** | `gather_density_snapshot__pinned_table_missing__pinned_count_failed` stays. |
| **F35 — Do not fake skip JSON** | Do **not** emit `GraphHealthOutput { density: "skip", status: "live", pinned_memories: 0, note: skip-msg }`. That is still a fake zero. Command `Err` is the truthful analog of `TablesMissing`. |
| **F36 — CAPABILITIES** | `status` format-matrix row: pin-count fail is section `error` (not `pinned=0`). `graph update` row: pinned COUNT fail is command error (not fake `pinned_memories: 0`). |

---

## 4. Acceptance criteria

| AC | Proof |
|----|-------|
| **AC1** | Unit `graph_section_from_gather__pinned_count_failed__error_not_fake_zero` (status.rs): inject `GatherResult::PinnedCountFailed { nodes: 0, edges: 0, memory_nodes: Some(0) }` → `Err` whose display **equals** `PINNED_COUNT_FAILED_MSG`. Must **not** be `Ok` with `pinned == Some(0)` / `status == Some("live")`. **Required red** (today that fixture assesses skip/`live`). Second case: `nodes: 100, edges: 10` (would be sparse if assessed) still `Err` — not `Ok` sparse with `pinned=0`. |
| **AC2** | Unit `graph_health_from_gather__pinned_count_failed__err` (`graph.rs`, `--features graph`): same variant → `Err`; display contains `cannot assess empty_lag without pins`. Must **not** produce `GraphHealthOutput { pinned_memories: 0, status: "live", .. }`. |
| **AC3** | T320 `status_envelope__graph_err__error_keeps_others` **stay-green** (tables-missing). |
| **AC4** | `gather_density_snapshot__pinned_table_missing__pinned_count_failed` **stay-green**. |
| **AC5** | `health_check_order_names__fixed_matrix` len **15** **stay-green**. Doctor skip arm file-level: `git diff` `doctor.rs` production **empty**. |
| **AC6** | T300 `rebuild_with_daemon_state__format_json__health_keys` **stay-green**. |
| **AC7** | Glance JSON fixture for COUNT fail: `graph.error` nonempty; `graph.status` / `nodes` / `edges` **omitted**; `daemon` still present; `schema_version==1`. Covered by AC1 + existing `apply_graph_error` serialize, or an additive serialize assert in AC1. |
| **AC8** | Human: `format_status_graph_line` on COUNT-fail section contains `graph: error=` and **not** `pinned=0`. |
| **AC9** | Workspace: `Cargo.toml` `[workspace.package] version = "0.1.4"`. `env!("CARGO_PKG_VERSION")` for `ai-brains-cli` is `"0.1.4"` (unit or hermetic `ai-brains --version` on the **source** bin contains `0.1.4`). PATH `--version` **0.1.3** is honesty, not fail. |
| **AC10** | CHANGELOG has `## [0.1.4]` and a **Fixed** T326 pin-count sentence. `scripts/check-version-banners.ps1` (non-Strict) exit **0** with `CHANGELOG ## [0.1.4]: True`. |
| **AC11** | Docs: F25 headers say **0.1.4**. CAPABILITIES `status` + `graph update` rows name COUNT-fail honesty (F36). Do **not** rewrite historical 0.1.1 claims. |
| **AC12** | `cargo clippy -p ai-brains-cli --all-targets -- -D warnings`; nextest `-p ai-brains-cli` (include `--features graph` for AC2). |
| **AC13** | Isolation: `git diff` product — `doctor.rs` / `project.rs` / `sync.rs` / `governed_common.rs` / contracts **empty** of behavior. Desktop version files **untouched**. |
| **AC14** | No new clap flags. `status --help` / `graph update --help` stay-green (no `--fail-on-pinned-count`). |
| **AC15** | **Manual:** PATH `status` may still be unrecognized (T320 not installed) — honesty. Source SoT is AC1/AC7/AC8. **No** live table drop. **No** `cargo install`. **No** git tag. |
| **AC16** | Floors freeze unit (T213/T308 `MIN_EDGE_NODE_RATIO == 0.50`) **stay-green** if present; else assert const in existing density tests. |

---

## 5. Design notes

### 5.1 Glance (`status.rs`)

Today (`:327–345`):

```rust
GatherResult::PinnedCountFailed { nodes, edges, memory_nodes } => {
    let snap = GraphDensitySnapshot { nodes, edges, pinned_memories: 0, memory_nodes };
    Ok(graph_from_assessment(&snap, &assess_graph_density(&snap)))
}
```

On go, extract:

```rust
fn graph_section_from_gather(gather: GatherResult) -> Result<GraphSection, String> {
    match gather {
        GatherResult::TablesMissing => Err("graph tables missing".into()),
        GatherResult::PinnedCountFailed { .. } => Err(PINNED_COUNT_FAILED_MSG.into()),
        GatherResult::Ok(snap) => Ok(graph_from_assessment(&snap, &assess_graph_density(&snap))),
    }
}
```

`build_graph_section` stays the vault/lock wrapper and calls the helper. Existing `:197–207` Err arm does the envelope.

### 5.2 `graph update` (`graph.rs`)

Today (`:445–458`) invents snap + `pinned_for_json = 0` then assesses. On go, `PinnedCountFailed` **returns Err** before `assess_graph_density`, analog `TablesMissing`. Extract `graph_health_from_gather` so AC2 does not need `AppContext`.

Do **not** add `error` to `GraphHealthOutput` (T213/T300 key freeze / F35).

### 5.3 Why empty graph + fake 0 is `live`

`assess_graph_density_with` `:260–270`: small/empty skip requires `pinned < MIN_PINNED` (100) **and** empty graph. Fake 0 satisfies the pin side. empty_lag (`:188`) requires `pinned >= 100`, so COUNT fail **hides** empty_lag — exactly the Bugbot text.

### 5.4 Version bump (0.1.3 analog)

`#217` `2ed5b06` (2026-08-25): `Cargo.toml` 0.1.2→0.1.3, lock workspace pkg versions, CHANGELOG insert `## [0.1.3]` **between** Unreleased and the then-current `### Added`. Docs banners were **not** updated (still 0.1.2). This track repeats the cargo/CHANGELOG move **and** catches F25 banners up to **0.1.4** (T304 R6 / T183 L6). Desktop stays 0.1.2.

Implementer: edit `Cargo.toml`; run a cargo command that rewrites lock package versions (e.g. `cargo metadata --format-version 1` / `cargo check -p ai-brains-cli`); do **not** `cargo update` of third-party crates.

### 5.5 Const

```rust
/// Doctor skip / glance error / graph-update error body (T326 F4).
pub(crate) const PINNED_COUNT_FAILED_MSG: &str =
    "pinned memory count failed (cannot assess empty_lag without pins)";
```

Doctor literal must remain **byte-identical** (copy-not-share). If go wants doctor to use the const, that is a **one-line** `doctor.rs` switch — **Stop-Before** vs F3; default is do not edit doctor.

---

## 6. Non-goals

- Floor retune / auto-rebuild / 16th doctor check / HTTP or TCP on glance
- Growing `doctor.rs` / `project.rs` / `sync.rs` / `governed_common.rs`
- New `GraphHealthOutput.error` key / `pinned_memories: null`
- Fake skip JSON (F35)
- clap 5 / rusqlite bump / T307 / H2 / T240 F2
- Desktop 0.1.2 / path-dep 0.1.0 / git tag / crates.io / cargo-release
- T310 F15 `ai-brainsd --version` / protocol schema bump / T320 `schema_version: 2`
- T185 `-Strict` as a required CI gate
- Rewriting historical 0.1.1 claims
- `cargo install` / live vault mutate / dropping `memory_projection`

---

## 7. Verification plan (TDD)

1. **Red:** AC1 unit fails (today Ok + `live`/`pinned=0`). AC2 fails (today Ok `pinned_memories: 0`).
2. **Green:** F1/F2 match arms + F4 const. AC1/AC2 pass. AC3–AC8 / AC12–AC14 / AC16 stay-green.
3. **Version:** F23–F25 / AC9–AC11 (after the fail-open is green; same BUGFIX TX, later commit OK).
4. **Manual AC15:** PATH honesty only.

---

## 8. Risk

| Risk | Mitigation |
|------|------------|
| Changing `graph update` from fake report to `Err` is user-visible | Analog `TablesMissing` already Err. COUNT fail is rarer than missing tables. F35 forbids skip-JSON. |
| Doctor/const string drift | Copy-not-share; AC1 asserts const; doctor tests stay-green |
| Version bump lock churn | Only workspace package `version` fields; no `cargo update` of crates.io deps |
| Docs historical 0.1.1 rewrite | F25 header-only |
| Live COUNT never fails | Hermetic inject is SoT (F21) |
| `graph.rs` tests need `--features graph` | F31; AC12 names it |

---

## 9. Deferred absorb / decline

| Item | Disposition |
|------|-------------|
| `#237` Bugbot `PinnedCountFailed` fake 0 (`status.rs:329–340`) | **Absorb** F1 / AC1 |
| `graph.rs:445–458` same fake 0 | **Absorb** F2 / AC2 |
| T320 F4 error envelope | **Reuse** (already shipped) |
| T304 R6 INSTALL.md / Docs banners still 0.1.2 | **Absorb** F25 / AC11 |
| T183 L6 version banners manual | **Absorb** F24/F25 |
| Owner “also have it do a version bump” | **Absorb** F23–F28 / AC9–AC11 |
| T325 implement residuals (LIMIT, PATH, pretty BM25) | **Not stolen** |
| T307 Blocked | **Not stolen** |
| clap 5 / H2 / T240 F2 / floors | **Decline** F19 |
| Desktop 0.1.2 / path-dep 0.1.0 / git tag | **Decline** F26–F28 |
| last-PR Cursor `#247` | **N/A empty** (no defect) |
| last-PR `#237` | **this** — **no T327** |
| `ISSUES.md` | **Does not exist** |
| T325 Completed conductor notes (dirty on main) | **Plan-write DOCS commit** |

---

## 10. Implement order (on go)

1. Phase 0: re-read `status.rs:329–340` vs doctor `:901–904` vs `graph.rs:445–458`; lock rusqlite **0.40.2**; workspace still **0.1.3**; T307 still Blocked; BUGFIX TX. **Do not install.**
2. Red AC1 + AC2.
3. Green F1/F2/F4/F8. Stay-green AC3–AC8 / AC12–AC16.
4. Version F23–F25 / AC9–AC11.
5. CAPABILITIES F36 + CHANGELOG Fixed + cut 0.1.4.
6. Conductor Completed only after implement-track Phase 6 (push / PR / GHA / squash). **Never** `git push origin main`.

---

## 11. Soft residuals

| Residual | Note |
|----------|------|
| PATH until owner `cargo install --features graph` | T312–T325 + this `--version` 0.1.4 |
| COUNT fail is rare on a healthy vault | Hermetic SoT |
| Doctor literal vs const | Copy-not-share (F3/F4) |
| No git tag `v0.1.4` | F28; T185 public-tag path |
| Docs example scripts still mention 0.1.x in comments we miss | F25 closed set; extra mentions → this residual |
| T307 / clap 5 | Standing |

---

## 12. Touch map

| File | Change |
|------|--------|
| `crates/ai-brains-cli/src/commands/status.rs` | `graph_section_from_gather`; PinnedCountFailed → Err; AC1 unit |
| `crates/ai-brains-cli/src/commands/graph.rs` | `graph_health_from_gather`; PinnedCountFailed → Err; AC2 unit (`cfg graph`) |
| `crates/ai-brains-cli/src/graph_density.rs` | `PINNED_COUNT_FAILED_MSG` const only |
| `Cargo.toml` | `workspace.package.version` 0.1.4 |
| `Cargo.lock` | workspace package version fields (cargo rewrite) |
| `CHANGELOG.md` | T326 Fixed + `## [0.1.4]` cut |
| `Docs/CAPABILITIES.md` | Version banner + F36 rows |
| `Docs/INSTALL.md` / `README.md` / `RELEASE-CHECKLIST.md` / `RELEASE-CLAIMS.md` (header) / `SECURITY-LIMITS.md` / `ci-tooling.md` example | 0.1.4 banners |
| `scripts/generate-sbom.ps1` | comment example version |
| `conductor/*` | registry / deferred / this spec+plan |

**Do not touch:** `doctor.rs` production, `project.rs`, `sync.rs`, `governed_common.rs`, `session_chrome.rs`, desktop version files, path-dep `0.1.0`, contracts DTOs.

---

## 13. Fold-in pins (plan-write)

- AC1 red is **empty-graph PinnedCountFailed → live/skip today**.
- AC2 is **graph update Err**, not skip JSON.
- AC11 Docs banners **0.1.2 → 0.1.4** (they never caught 0.1.3).
- last-PR `#247` empty / `#237` → this / **no T327**.
- Still **plan-only until go.**
