# T306 — PATH install: SQLCipher 4.14.0 community

- **Track ID:** T306-PathInstallSqlcipher414
- **Status:** **Planned** (Pending until **go**)
- **Category:** CHORE / OPS
- **Owner:** Grok
- **Source:** T305 R3 — gate used track-built `target\debug\ai-brains.exe`; PATH `ai-brains` still rusqlite **0.39** / SQLCipher **4.10**. Owner asked leftover placeholders 2026-08-26; this pass upgrades the stub to a full plan.
- **Depends on:** T305 `#222` lock rusqlite **0.40.2**; COMPATIBILITY F8 **observed** source `4.14.0 community`; T200/T222 `GRAPH_REINSTALL_SOOT`; T187 `bundled-sqlcipher-vendored-openssl`.
- **Blocks / feeds:** PATH doctor `cipher_page` honesty. Unblocks operators from running the WAL-reset-fixed SQLCipher 4.14 that source already has. Does **not** unblock T307/T308/T309.
- **Absorbs:** T305 R3; T305 Codex “PATH still points at the older binary”; mint stub problem text.
- **Not absorbed (DoD):** T307 dual tower-http; T308 sparse remediator; T309 `table_exists`; PATH `ai-brainsd` 4.10 WAL writer + T84 `run_update` graph-off (**T310**); clap 5; floor retune; live `vault encrypt` / `graph rebuild` / `daemon stop`.
- **Research date:** 2026-08-26 (HEAD `cb5aa49`; product `src/` = T305 `#222` `a49acbd`).
- **Ledger:** planning DOCS TX `2b0a2dec-7921-4e84-a964-b37cb703457c`. Series mint DOCS `c62396f6-4532-4335-b10b-f31b3fa02ec2`. Implement starts a **CHORE** TX on **go**.
- **Isolation:** Do **not** `cargo install` as planning. Do **not** live `vault encrypt` / `graph rebuild`. Do **not** `daemon stop` / `daemon start`. Do **not** print or commit `AI_BRAINS_KEY`. Do **not** edit `GRAPH_REINSTALL_SOOT`. Do **not** bump workspace `0.1.3`. Do **not** merge Dependabot remotes. Never `git push origin main`.

---

## 1. Objective

1. **PATH matches source SQLCipher.** After go, `C:\Users\RyanB\.cargo\bin\ai-brains.exe` is a `--locked --features graph` install of the current workspace, so live `PRAGMA cipher_version` is **4.14.x** (source already records **`4.14.0 community`**).
2. **Keep T222 graph-on.** Install command is exactly `GRAPH_REINSTALL_SOOT`. Do not drop `--features graph` (would reinstall graph-off). Do not drop `--locked` (would ignore `Cargo.lock` rusqlite **0.40.2**).
3. **Prove it with doctor JSON.** `cipher_page` **ok** is not enough — T187-V-01 only requires a non-empty `4.` string, so **4.10.0 community also passes**. DoD is the **message token `4.14`**. `--summary` hides ok checks; use `--json`.
4. **Capture independence.** Operator install only. No new events, no contracts DTO, no crate edits, no models. The append-only event log stays the SoT; this track only puts the already-shipped 4.14 reader/writer **CLI** on PATH.

This unblocks daily ops honesty: T305 upgraded the lock; T306 is the operator binary. PATH still serving SQLCipher **4.10** misses Zetetic’s 4.14 WAL-reset fix on the CLI that operators actually run.

---

## 2. Live baseline (re-scan 2026-08-26)

### 2.1 Operator dogfood (this machine)

| Signal | Observation |
|--------|-------------|
| HEAD | `cb5aa49` mint T306–T309. Tree **CLEAN**. `origin/main` = `a49acbd` T305 `#222`. Ahead **1** (conductor mint only). Branch `main`. |
| PATH `ai-brains` | `C:\Users\RyanB\.cargo\bin\ai-brains.exe` — **25,216,512** bytes; LastWriteTime **2026-08-25 14:47:44**; `ai-brains 0.1.3`. Workspace version is also **0.1.3** — **`--version` does not prove rusqlite 0.40.2**. |
| PATH `doctor --json` `cipher_page` | **`ok` / `cipher_version=4.10.0 community`**. Smoking gun: PATH is still **SQLCipher 4.10** (rusqlite 0.39 era). |
| PATH `graph_feature` | **`available`**. T300 already installed graph-on. T306 must **keep** `--features graph`. |
| PATH `doctor --summary` | `status=degraded` `ok=11 warn=2 fail=0 skip=2`. Attention: `recovery_kit_event`; **`graph_density` sparse E/N=0.409** (nodes≈62974 edges≈25753 pinned=49288 memory_nodes=39329) remediator `ai-brains graph rebuild`. **`cipher_page` is absent from attention** because it is ok — **`--summary` cannot prove 4.14**. |
| PATH `ai-brainsd` | `C:\Users\RyanB\.cargo\bin\ai-brainsd.exe` — **21,045,248** bytes; LastWriteTime **2026-08-22 14:48:10**. Older than CLI. Likely still 4.10 WAL **writer**. **Not this DoD → T310.** |
| `preflight --summary` | Scope `C:\dev\ai-brains` (`3581317d`). Pinned **4296** (volatile). Sessions **4**. Capture independence holds. |
| Perl | **v5.42.2** MSWin32-x64 (Strawberry) on PATH — openssl-src Configure can run. |
| rustc / cargo | **1.95.0** / **1.95.0** (`rust-toolchain.toml`). |
| Last GitHub PR | [#222](https://github.com/Ryan-AI-Studios/AI-Brains/pull/222) T305 (merged 2026-08-26T03:30:04Z). `pulls/222/comments`, `/reviews`, `issues/222/comments` all **empty**. **last-PR Cursor: N/A.** Open PRs: **none**. **No leftover from Cursor.** T310 is from **this** live baseline (daemon + T84), not Cursor. |
| Ledger | **0 pending / 0 drift** at scan (before this DOCS TX). |
| `ISSUES.md` | **Does not exist.** |
| Planning install | **Not run.** |

### 2.2 Why this residual still matters

| Residual | Why it is still a product hole / why decline |
|----------|----------------------------------------------|
| PATH `cipher_version=4.10.0 community` | Operators run PATH, not `target\debug`. T305 AC8 used the **track-built** binary. Source lock is 0.40.2 / 4.14; PATH is not. **DoD.** |
| `--version` 0.1.3 | Same string as workspace. Cannot distinguish 4.10 vs 4.14. Do **not** bump 0.1.4 just to distinguish — `cipher_page` message is the distinguisher. |
| `doctor --summary` | Hides ok `cipher_page`. A green skim after install would still omit the proof token. **DoD uses `--json`.** |
| T187-V-01 `4.` | Both 4.10 and 4.14 pass. Do **not** freeze the unit on `4.14` this track (no product src). |
| SQLCipher 4.14 WAL-reset | [Zetetic 4.14.0](https://www.zetetic.net/blog/2026/03/17/sqlcipher-4.14.0-release/) (2026-03-17): SQLite **3.51.3** WAL-reset corruption fix; **WAL users strongly advised to upgrade.** We set `journal_mode = WAL` (`pragmas.rs:25`). PATH CLI still 4.10. **Reason to install**, not a live re-encrypt. |
| PATH `ai-brainsd` 2026-08-22 | Daemon is the single-writer; leaving it on 4.10 leaves the WAL-reset bug on the writer. Replacing a **Running** `ai-brainsd.exe` on Windows needs a stop. Mint F4 / this F4: **no daemon stop as DoD.** **T310.** |
| T84 `run_update` graph-off | `daemon.rs:1069–1072` `cargo install --path crates/ai-brains-cli --locked` **without** `--features graph`. After T306, `ai-brains update` would **undo graph-on**. Product src. **T310.** |

### 2.3 Code truth

| Item | Location | Notes |
|------|----------|-------|
| Install SOOT | `governed_common.rs:45–46` `GRAPH_REINSTALL_SOOT` | Exact: `cargo install --path crates/ai-brains-cli --locked --features graph`. Smoke `tests/smoke.rs` pins the constant to INSTALL. **Do not edit.** |
| INSTALL how-to | `Docs/INSTALL.md:41` | Same SOOT. Header still says product **0.1.2** (stale docs — **not this hole**). |
| `cipher_page` | `doctor.rs:166–185` | `HealthCheck::ok_msg("cipher_page", format!("cipher_version={ver}"))` when non-empty. Fail if empty / probe err. |
| `cipher_version` | `pragmas.rs:50–52` | `PRAGMA cipher_version`; errors propagated (T305 Codex P2-02). |
| T187-V-01 | `connection.rs:397–412` | Non-empty **and** `contains("4.")`. Comment records observed **`4.14.0 community`** under 0.40.2. Does **not** fail 4.10. |
| `cipher_compatibility` | `pragmas.rs:22` / `:42` | `= 4`. Unchanged. |
| `graph_feature` | `doctor.rs:854–865` | `cfg!(feature = "graph")` → message `available` else `unavailable` + SOOT remediation. Always Ok severity. |
| Doctor JSON DTO | `ai-brains-contracts/src/doctor.rs:62–71` | `HealthCheck { name, severity, ok, message, remediation }`. `ok_msg` sets `message: Some(...)`. schema_version **1**. **No DTO change.** |
| T84 update | `daemon.rs:1030–1099` `run_update` | CLI install **omits** `--features graph`; also installs `ai-brainsd`. **Do not call. T310.** |
| Local scripts | `scripts/Build-AIBrains.ps1` / `scripts/build.ps1` | T222 graph-on + `graph_feature` probe. Alternative PATH path; **not** `--locked` SOOT. Primary remains F1 SOOT. |
| Features | `ai-brains-cli` `default = []`; `graph = [...]` | Unchanged (T200 A2=no). |
| Workspace rusqlite | `Cargo.toml:57` | Exact **`0.40.2`** + `bundled-sqlcipher-vendored-openssl` / `backup` / `fallible_uint` / `trace`. |
| Lock | `Cargo.lock` | rusqlite **0.40.2**; libsqlite3-sys **0.38.2**; hashlink **0.12.1**. |
| T305 evidence | `trackT305-rusqlite-0-40/cipher_version.txt` | Source probe **`4.14.0 community`**. |
| Hotspots | `project.rs` #1 | **Do not touch.** This track has **zero** crate edits. |

### 2.4 Dependency / standards research (2026-08-26) — snapshot, re-verify at execute

| Pin | Workspace / lock | crates.io / docs | Action |
|-----|------------------|------------------|--------|
| rusqlite | exact **0.40.2** / lock **0.40.2** | **0.40.2** (`cargo search` 2026-08-26) | **No bump.** Install `--locked`. |
| libsqlite3-sys / hashlink | **0.38.2** / **0.12.1** | via rusqlite 0.40.2 | Unchanged. |
| clap | workspace **4.5** / lock **4.6.1** | clap **5 declined** | **No bump.** |
| tokio | **1.53** / **1.53.1** | — | **No bump.** |
| reqwest | **0.13** / **0.13.4** | still 0.13.4 | **No bump** (T307). |
| tower-http | **0.7** / **0.7.0 and 0.6.11** | — | **No steal** (T307). |
| thiserror | **2.0** / **2.0.20** + **1.0.69** | — | **No bump.** |
| rustc / edition | **1.95.0** / **2024** | — | Unchanged. |
| workspace version | **0.1.3** | — | **No bump** (F3). |
| New crates | — | — | **Zero.** |

**Cargo install (verified [Cargo Book](https://doc.rust-lang.org/cargo/commands/cargo-install.html) 2026-08-26):**

- `--path` **always builds and installs**, unless conflicting binaries from another package. Same `0.1.3` does **not** skip a path reinstall. `--force` is optional insurance, not required for `--path`.
- `--locked` uses workspace `Cargo.lock` (required so rusqlite stays **0.40.2**).
- `--features graph` is **not** implied by `default = []`.
- Cwd: workspace root `C:\dev\AI-Brains`. PowerShell `;` not `&&`.
- Windows: replacing a running `ai-brains.exe` can hit a sharing violation. Daemon is a **different** exe — do **not** stop it to replace CLI (F4). Stop-Before if install fails on file lock.

**SQLCipher / rusqlite (re-verified):**

- Source observed `4.14.0 community` (T305 / COMPATIBILITY F8 `:80`).
- Zetetic: format compatible within major 4.x; `cipher_compatibility = 4`. **Do not** live-encrypt. Existing vault already opens on the T305 debug binary.
- SQLCipher community pragma test expects `4.14.0 community`. DoD token is **`4.14`** (not frozen `.0`) so a future 4.14.x patch still passes.
- Windows MSVC vendored OpenSSL needs **Perl** on PATH (`Docs/ci-tooling.md` / INSTALL). Live: Perl **v5.42.2**. Stop-Before if `openssl-src` Configure fails.

**N/A (external implementation research):** This is operator `cargo install --path`, not a new API. Pattern is T222 F27 / T200 INSTALL / T300 PATH graph-on install. No third-party installer.

---

## 3. Frozen decisions

| ID | Decision |
|----|----------|
| **F0 — Go gate** | Plan-only until user **go**. Planning is DOCS. Implement starts a CHORE TX. **Do not** `cargo install` as planning. |
| **F1 — Install command** | From `C:\dev\AI-Brains`, exact `GRAPH_REINSTALL_SOOT`: `cargo install --path crates/ai-brains-cli --locked --features graph`. Optional trailing `--force` only if cargo refuses. Do **not** change the Rust constant. |
| **F2 — DoD probe** | PATH `ai-brains doctor --json`: check `name=cipher_page`, `ok==true`, `message` **contains `4.14`**. `--summary` is **not** sufficient (hides ok). Filter JSON; never paste key material. |
| **F3 — No product src** | No crate / `Cargo.toml` / `Cargo.lock` / `GRAPH_REINSTALL_SOOT` / INSTALL 0.1.2 header / CHANGELOG edits as DoD. No workspace version bump. `git diff -- crates/ Cargo.toml Cargo.lock` empty after go. |
| **F4 — No live mutate** | No `vault encrypt`, no `graph rebuild`, no `daemon stop`/`start`, no `retention apply --confirm`, no leftover `--write --yes`, no `harness install` as DoD. |
| **F5 — Git** | Never `git push origin main`. Do not merge Dependabot remotes. Conductor closeout may PR on `track/T306-*` (docs only). |
| **F6 — No key leak** | Captured doctor/install logs must not contain `AI_BRAINS_KEY` or `x'<64 hex>'` material. |
| **F7 — Graph-on** | After install, `graph_feature` message **`available`**. Slim install without `--features graph` is a **High** regression of T222. |
| **F8 — Daemon out** | PATH `ai-brainsd` 4.10 / mtime 2026-08-22 is **T310**. Mixed CLI 4.14 + daemon 4.10 is an accepted T306 residual (Zetetic same-major). |
| **F9 — Do not `ai-brains update`** | T84 `run_update` omits `--features graph`. Calling it after/instead of F1 is a T222 regression. **T310.** |
| **F10 — T187-V-01 freeze** | Unit stays non-empty + `4.`. Do not require `4.14` in the unit this track. |
| **F11 — Contracts** | `DoctorReport` / `HealthCheck` unchanged. Only the observational `cipher_page.message` value changes on PATH. |
| **F12 — Perl / openssl** | Perl must stay on PATH. Stop-Before if vendored OpenSSL build fails. |
| **F13 — File lock** | If `cargo install` fails because `ai-brains.exe` is locked, halt and ask. Do **not** `daemon stop` to clear it (wrong binary). |
| **F14 — Cross-model** | CHORE install-only, no src. `codex-review` **not** required unless a crate edit sneaks in. |
| **F15 — TDD** | No new tests (no src). Proof is PATH `--json` (AC2–AC5). Existing T187-V-01 / doctor matrix stay green without re-run as a plan gate. |
| **F16 — Scripts not primary** | `build.ps1` / `Build-AIBrains.ps1` are T222 alternatives. Primary is F1 SOOT (`--locked`). |
| **F17 — Version string** | `ai-brains --version` may remain `0.1.3`. Do not treat it as 4.14 proof. |
| **F18 — clap 5** | **Still declined.** |
| **F19 — Density** | Floors frozen. Sparse remediator is **T308**. Post-install `graph_density` warn is **expected**. |
| **F20 — `table_exists`** | **T309.** |
| **F21 — tower-http dual** | **T307.** |
| **F22 — Pragmas** | `cipher_compatibility = 4`; no `cipher_plaintext_header_size`. No live rekey. |
| **F23 — Capture independence** | Install only. No events. No models on the install path. |
| **F24 — Harness** | OPERATIONS “re-run `harness install` after cargo install” is a **soft residual**, not DoD (would mutate harness config). PATH spawn is enough. |
| **F25 — mtime** | After install, PATH `ai-brains.exe` LastWriteTime **newer than 2026-08-25 14:47:44**. |
| **F26 — Wrong binary** | If after F1 `cipher_page` is still `4.10`, **halt** (install root / PATH shadow / failed copy). Do not “fix” by editing doctor. |
| **F27 — Matrix** | Doctor stays **15** checks. Do not add a 16th “sqlcipher_patch” check. |
| **F28 — Pins** | No clap / rusqlite / tokio / reqwest / tower-http steal. Re-verify lock rusqlite **0.40.2** in Phase 0 on go. |
| **F29 — Full gate** | Product tree unchanged → gate should match T305. Run `ledgerful verify --scope fast` on conductor closeout; full workspace gate **not** required to prove PATH 4.14. Implement-track Phase 6 still applies to the docs PR. |
| **F30 — Debt file** | `conductor/ISSUES.md` does **not** exist. Residuals → `deferred.md`. |

---

## 4. Acceptance criteria

| AC | Proof |
|----|-------|
| **AC1** | Operator ran F1 from repo root. PATH `where.exe ai-brains` is still `C:\Users\RyanB\.cargo\bin\ai-brains.exe`. LastWriteTime **after** 2026-08-25 14:47:44 (F25). |
| **AC2** | `ai-brains doctor --json` (PATH): `checks[]` entry `name=="cipher_page"` has `ok==true` and `message` contains **`4.14`**. Prefer recording the full message (expect `cipher_version=4.14.0 community` shape). |
| **AC3** | Same JSON: `name=="graph_feature"` message **`available`**; no `FEATURE_UNAVAILABLE` on `ai-brains graph update --format json` (exit 0 or non-2). |
| **AC4** | Same JSON: `name=="vault_open"` ok (`opened read-only` class). Doctor did not migrate. |
| **AC5** | Captured `--json` / `--summary` / cargo install stdout+stderr contain **no** `AI_BRAINS_KEY` and no `x'` + 64 hex key token. |
| **AC6** | `git diff -- crates/ Cargo.toml Cargo.lock` empty. `GRAPH_REINSTALL_SOOT` byte-identical. |
| **AC7** | `--summary` may still be `degraded` (`graph_density` sparse, `recovery_kit_event`). **Not a fail.** Do not rebuild/export to clear them. |
| **AC8** | `conductor.md` T306 **Completed** with PATH `cipher_page` evidence; deferred.md T305 R3 struck / absorbed as done. |
| **AC9** | Phase 0: lock rusqlite **0.40.2**; Perl on PATH; cwd repo root. If any false, **Stop-Before** (do not install a wrong tree). |

No hermetic unit AC — there is no code change to unit-test. CI does not install PATH.

---

## 5. Design notes

### 5.1 Why `--json` not `--summary`

`doctor --summary` prints `attention:` for warn/fail/skip-of-interest only. Live 2026-08-26 skim had `cipher_page` **ok** and therefore **omitted**. After a successful 4.14 install the check is still ok, so `--summary` would still omit it. Proof command:

```powershell
ai-brains doctor --json |
  python -c "import sys,json; r=json.load(sys.stdin); c=[x for x in r['checks'] if x['name'] in ('cipher_page','graph_feature','vault_open')];
[print(x['name'], x.get('ok'), (x.get('message') or '')[:80]) for x in c]"
```

Do **not** dump the full JSON into chat/ledger (F6).

### 5.2 Why not bump 0.1.4

`--path` always rebuilds (Cargo book). `cipher_page` message is the distinguisher. A version bump is product src (every crate `version.workspace`) and a CHANGELOG event this track does not need.

### 5.3 Mixed CLI 4.14 / daemon 4.10

T305 F9 KATs already opened 4.10-era vaults with 4.14. `cipher_compatibility = 4`. Accept mixed versions until **T310**. Do not Stop-Before solely because daemon mtime is 2026-08-22.

### 5.4 Capture independence

`cargo install` rebuilds the CLI with the same store crate T305 shipped. No new events. Doctor stays `open_read_intent` only (`vault_open`).

---

## 6. Non-goals

- Editing any crate, lock, INSTALL header `0.1.2`, or `GRAPH_REINSTALL_SOOT`
- PATH `ai-brainsd` reinstall / `daemon stop` (**T310**)
- `ai-brains update` / T84 `run_update` graph-on (**T310**)
- T307 / T308 / T309
- Floor retune (`MIN_EDGE_NODE_RATIO = 0.50`)
- clap 5
- Live `vault encrypt` / live `graph rebuild` / recovery export
- Doctor 16th check for SQLCipher patch level
- Flipping `release.yml` / Cargo `default = ["graph"]`
- MSI / binstall / GitHub Release graph-on
- `[patch.crates-io]`
- Re-running `harness install` as DoD
- Dependabot remote merge / close hygiene (standing)

---

## 7. Verification plan

TDD: **no red tests** (F15). On go:

```powershell
# Phase 0 — re-verify (do not install yet)
Select-String -Path Cargo.lock -Pattern 'name = "rusqlite"' -Context 0,1
perl -v
where.exe ai-brains
ai-brains --version
# PATH still 4.10 expected until F1:
ai-brains doctor --json |
  python -c "import sys,json; r=json.load(sys.stdin); print([c['message'] for c in r['checks'] if c['name']=='cipher_page'][0])"

# F1 — only after go
cargo install --path crates/ai-brains-cli --locked --features graph

# Proof (filter; no key)
ai-brains doctor --json |
  python -c "import sys,json; r=json.load(sys.stdin); 
[print(c['name'], c.get('ok'), c.get('message')) for c in r['checks'] if c['name'] in ('cipher_page','graph_feature','vault_open')]"
ai-brains doctor --summary
Get-Item (Get-Command ai-brains).Source | Select-Object FullName, Length, LastWriteTime

# Confirm no product diff
git diff -- crates/ Cargo.toml Cargo.lock
```

Do **not** require full workspace nextest to finish the install proof. Conductor closeout: `ledgerful verify --scope fast`.

---

## 8. Risk

| Risk | Mitigation |
|------|------------|
| `--summary` false pass (ok 4.10 looks like ok 4.14) | **F2 / AC2** `--json` + token `4.14` |
| `--version` 0.1.3 false pass | **F17** |
| Slim install drops graph | **F1 / F7 / AC3** keep `--features graph` |
| `--locked` omitted → resolver drift | **F1** SOOT includes `--locked` |
| openssl-src / Perl missing | **F12** Phase 0 `perl -v`; Stop-Before |
| `ai-brains.exe` file lock | **F13** halt; do not stop daemon |
| `ai-brains update` undoes graph-on | **F9** / **T310** |
| Live encrypt/rebuild “to complete 4.14” | **F4** vault already opens on 4.14 debug bin |
| Daemon stays 4.10 WAL writer | **F8** accepted; **T310** |
| Sharing PATH proof in logs leaks key | **F6 / AC5** filter checks only |
| Conductor-only closeout skipped | **AC8** + implement-track Phase 6 docs PR |
| Wrong cwd / nested tree | **AC9** `C:\dev\AI-Brains` |

---

## 9. Deferred absorb / decline

Entire `conductor/deferred.md` scanned 2026-08-26.

| Item | Disposition |
|------|-------------|
| T305 R3 PATH pre-0.40.2 / 4.10 | **Absorb** F1–F7 / AC1–AC5 / AC8 |
| T305 Codex “PATH still older binary” | **Absorb** (same as R3) |
| T305 R1 Dependabot `#61` close hygiene | **Decline** — standing; not install |
| T305 R2 / T213 L4 `table_exists` | **Decline steal → T309** |
| T305 R4 lock extra variance | **Decline** — do not hand-edit |
| T305 R5 clap 5 | **Decline** F18 |
| T304 R2 dual tower-http 0.6.11 | **Decline steal → T307** |
| T304 R1/R3/R4 csrf / extras / hygiene | **Decline** — standing / T307 |
| T300 still sparse E/N 0.409; remediator rebuild | **Decline steal → T308**; AC7 expected |
| T278 floor retune | **Decline** F19 |
| T300/T299/… “PATH until cargo install” (CLI quality era) | **Partial** — T300 installed 0.1.3 graph-on; this track is **4.14** only |
| `recovery_kit_event` doctor warn | **Not this track** |
| INSTALL.md still says **0.1.2** | **Decline** — docs header drift; not T305 R3 |
| PATH `ai-brainsd` 2026-08-22 / T84 `run_update` graph-off | **Mint T310** (fits no Pending placeholder: T306 F3/F4 forbid src + daemon stop) |
| last-PR Cursor `#222` | **N/A empty** — comments/reviews/issue comments `[]`. **No T311 from Cursor.** |
| T240 F2 silent Scope / leftover `--write` / T263 H2 | **Decline** — standing |

---

## 10. Implement order (on go)

1. Phase 0: AC9 re-verify lock / Perl / cwd / PATH still 4.10. CHORE TX. **Do not install until this table is green.**
2. F1 `cargo install --path crates/ai-brains-cli --locked --features graph` from repo root.
3. AC1–AC5 PATH `--json` proof (filter). AC6 empty crate diff. AC7 degraded-ok.
4. AC8 conductor Completed + deferred R3 done. `ledgerful verify --scope fast`.
5. Phase 6: `track/T306-*` docs PR if there is a closeout commit; watch CI; squash-merge. Never `git push origin main`.

---

## 11. Soft residuals (post-close)

| Residual | Note |
|----------|------|
| PATH `ai-brainsd` still 4.10 WAL writer | **T310** |
| T84 `run_update` omits `--features graph` | **T310** |
| Mixed CLI 4.14 / daemon 4.10 | Accepted F8 until T310 |
| `graph_density` sparse E/N ~0.409 | **T308**; AC7 |
| `recovery_kit_event` | Not this track |
| INSTALL header 0.1.2 | Docs drift; not DoD |
| Harness reinstall after cargo install | F24 soft |
| `--force` never needed on this host | F1 optional |
| Workspace still 0.1.3 | F17 |

---

## 12. Touch map

| Path | Role |
|------|------|
| `C:\Users\RyanB\.cargo\bin\ai-brains.exe` | **Hard (ops):** replace via F1. Not in git. |
| `conductor/tracks/trackT306-path-install-sqlcipher-414/spec.md` | This plan |
| `conductor/tracks/trackT306-path-install-sqlcipher-414/plan.md` | Checklist |
| `conductor/conductor.md` | Pending → Completed **on go** |
| `conductor/deferred.md` | R3 absorb **on go** |
| `crates/**` | **Do not touch** |
| `Cargo.toml` / `Cargo.lock` | **Do not touch** |
| `Docs/COMPATIBILITY.md` | Already records source 4.14 — **do not rewrite** as PATH proof |
| `GRAPH_REINSTALL_SOOT` | **Do not touch** |
| `ai-brainsd.exe` | **T310** |
