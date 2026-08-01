# T179 — Multi-Platform Compatibility Matrix (P12.1)

- **Track ID:** T179-CompatibilityMatrix
- **Phase:** P12 — Release hardening and adoption (Task 1)
- **Status:** ✅ **Completed** (2026-08-01) — GHA PR #51 all gates green (run 30683807812; docs follow-up 30684575740); residuals deferred to T183/T185/T186
- **Depends on:** Core CLI/daemon stable on Windows (live primary); P10 desktop Complete on Windows; P11 sync Complete (wire crypto platform-agnostic)
- **Blocks / feeds:** T183 install docs (Windows-first, then others); T185 release gate “platform smoke per T179”; honest marketing claims
- **Category:** INFRA / RELEASE
- **Deferred absorbed:** T174 §49 multi-OS / WDIO matrix residual; T174 soft WDIO embedded-provider note; product PRD secondary Ubuntu/WSL; vision “Windows 11 + WSL first production support.” **Not** #34.2 DataKey rotation; **not** T180 N-1 wire protocol; **not** App Store / notarization ship.
- **Review fold-in:** AI1 BS1–4 + ergonomics; AI2 findings #1–#11 + improvements A–H (agreed items → F23–F32). See §14 disposition table.

## 1. Objective

Publish and enforce an **honest compatibility matrix** for AI-Brains across:

| Surface | Platforms in scope |
|---------|-------------------|
| OS | Windows 11 (primary), Ubuntu/WSL (secondary), Linux native (tiered), macOS (tiered) |
| Arch | x86_64 primary; arm64 documented as best-effort until green |
| Product binaries | `ai-brains` CLI, `ai-brainsd` daemon, optional desktop (Tauri) |

Without overstating support: **Windows-first remains normative**. Linux/macOS become tier-1 only where CI or recorded smoke proves them.

After T179:

| Capability | Present |
|------------|---------|
| `Docs/COMPATIBILITY.md` with frozen tier table | Yes |
| Grep-complete `cfg(windows)` inventory | Yes |
| CI matrix plan (PR vs release) for agreed tiers | Yes (implement on go-ahead) |
| Cross-platform smoke checklist + evidence template | Yes |
| Capture independence proven on every tier that runs tests | Yes |
| Full feature parity (Windows service, DPAPI, WebView2 Isolation) on Linux/macOS | **No** — documented Windows-only |
| App Store / notarized Tauri / MSI installers as DoD | **No** (T183/T185 packaging residuals) |
| Nested WSL2 e2e as mandatory PR gate | **No** (path unit + optional host WSL smoke) |

## 2. Live baseline (re-scan 2026-07-31 + review fold-in)

| Area | Live state |
|------|------------|
| Primary platform | Windows 11, PowerShell-first (PRD / Implementation-Plan) |
| Secondary platform | Ubuntu / WSL after Windows reliability proven |
| CI workflows | **No `.github/` workflows in tree** — gate is local `scripts/dev-check.ps1` + AGENTS.md full gate |
| Toolchain | `rust-toolchain.toml`: **channel 1.95.0**, components rustfmt+clippy, targets **`x86_64-pc-windows-msvc` only** |
| Workspace edition | **2024** (requires rustc ≥ 1.85; pin 1.95.0) |
| Vault store | `rusqlite` **0.39.0** features `bundled`, `backup`, `fallible_uint` — **not** `bundled-sqlcipher` (Deviations.md §1). crates.io also has **0.40.1** — hold 0.39 for T179 |
| Path | `ai-brains-path`: drive-case, UNC, WSL `/mnt/c` → Windows, reparse/symlink refuse; tests include `wsl_mnt_c_maps_to_windows` |
| Crypto | DPAPI wrap/unwrap **`cfg(windows)`**; passphrase recovery cross-platform |
| Daemon IPC (live) | **Windows:** named pipe (`\\.\pipe\ledgerful-bridge` via `DaemonClient`). **Unix:** **Unix domain socket** `/tmp/ledgerful-bridge.sock` (`request_unix` / `UnixStream`) — **not** HTTP by default |
| Daemon HTTP (portable) | T161 loopback HTTP + bearer (`ai-brains-api-server`) — product-portable control plane for multi-OS; desktop uses HTTP |
| Scheduler | Task Scheduler / `schtasks` / ProgramData ACL — **Windows product path** (T145) |
| Desktop | Tauri v2 + **WebView2** (Windows) shipped P10; macOS **WKWebView** / Linux **WebKitGTK** = different engines; no Isolation claim off Windows |
| Sync wire | WRAP/sign pure Rust (dalek/aes-gcm/hkdf) — **no OS crypto** for wire |
| Sync device seed storage | `private_blob.rs`: Windows seals with **`PROTECTION_DATAKEY_DPAPI`**; non-Windows **`PROTECTION_DATAKEY` only**. DPAPI-sealed seeds **not portable** to non-Windows |
| Capture independence | Mandate: capture path must work without models/graph/sync — **must remain true on every tier** |
| Git askpass | Windows: `git-askpass-noop.cmd`; Unix: **`/bin/true`** (scratch images may lack it) |
| Workspace `sqlx` | Pinned **0.8** in root `Cargo.toml` (`runtime-tokio-rustls`, `sqlite`); **no crate currently depends on it** — hold; not a T179 consumer |
| `windows` crate consumers | Target-gated in: `ai-brains-api-server`, `ai-brains-path`, `ai-brains-crypto`, `ai-brainsd`, `ai-brains-cli`, `apps/desktop/src-tauri` |

### 2.1 Windows-only / OS-bifurcated surfaces (**illustrative highlight** — not the full inventory)

> **AC2 / Phase A1:** §2.1 is a **highlight**, not exhaustive. Committed `evidence/CFG-INVENTORY.md` MUST be **grep-complete** (`cfg(windows)` / `cfg(not(windows))` / `target.'cfg(windows)'`) across the workspace (~75 sites / 18 files / 11 crates as of AI2 scan — re-count at implement).

| Surface | Crate / path | Non-Windows behavior |
|---------|--------------|----------------------|
| DPAPI key wrap | `ai-brains-crypto::dpapi` | Explicit error: “DPAPI is only available on Windows”; passphrase path remains |
| Device private seed seal | `ai-brains-sync::private_blob` | DataKey-only seal; cannot open Windows `datakey_dpapi` blobs |
| Windows Service host | `ai-brainsd::windows_service` (`#![cfg(windows)]`) | Foreground daemon; systemd residual (doc only) |
| Named pipe + SDDL | `ai-brainsd::pipe_security`, CLI pipe path | Unix uses **UDS** (live); product-portable path = **HTTP+bearer** |
| Task Scheduler / elevation / nightly | `elevation`, `commands/nightly`, T145 ACL | Document Windows-only; no cron/systemd T1 claim |
| Artifact ACL / reparse | `artifact_security`, path reparse | Unix mode / symlink refuse via `ai-brains-path` |
| Desktop WebView2 Isolation | `apps/desktop` webview2 | **No Isolation claim** on WKWebView/WebKitGTK |
| Git askpass noop | `ai-brains-git::command` | `/bin/true` required |
| Device CLI private-key write | `commands/device` (DPAPI paths) | Clear Windows-only / fail-closed errors |
| `windows` crate | 6 consumers (target-gated) | Must not leak into Unix dependency graph |

### 2.1.1 Daemon client transport matrix (**F23** — AI1 BS1 folded with live-code correction)

| OS | Live `DaemonClient` transport | Portable product control plane |
|----|------------------------------|--------------------------------|
| Windows | Named pipe `\\.\pipe\ledgerful-bridge` | Loopback HTTP + bearer (T161) also available |
| Unix (Linux/macOS) | **Unix domain socket** `/tmp/ledgerful-bridge.sock` | Loopback HTTP + bearer (T161) — **preferred multi-OS product surface** |

**Do not claim** “Unix already defaults to HTTP” — live code uses UDS. T179 must:

1. Document the live bifurcation honestly in COMPATIBILITY.md.  
2. Ensure Unix UDS path **compiles and fail-closed** when socket missing.  
3. Smoke **HTTP** on every T1 tier that claims portable daemon IPC (health/bearer).  
4. Add/keep a test that Unix `DaemonClient` uses UDS path construction (not Windows pipe APIs).  
5. **Optional residual (not T179 DoD):** later unify CLI→daemon on Unix to prefer HTTP; out of scope unless free.

### 2.2 Cross-platform surfaces (must compile + smoke on agreed tiers)

| Surface | Notes |
|---------|-------|
| Event log + projections | `bundled` SQLite; same migrations; **not** page-level SQLCipher until feature+CI |
| Capture CLI path | No model/graph/sync required; T178-style capture↔sync absence check on all T1 tiers |
| Path normalize + WSL aliases | Unit tests for string forms; WSL smoke = Linux binary + `/mnt/c` (see §5.2) |
| Control-plane + contracts | Pure logic |
| Loopback HTTP API | axum 0.8 + tower-http; **portable** daemon surface |
| Sync **wire** crypto + fake-relay | Hermetic; pure Rust |
| Sync **device seed** storage | OS-bifurcated (DPAPI vs DataKey-only) — document, test both cfgs |
| Content envelopes / CE | AES-256-GCM under DataKey — pure Rust (application-level) |

## 3. Research summary (online + standards, 2026-07-31; AI2 refresh)

### 3.1 GitHub-hosted runners (current labels)

| Label | Typical image (2026-07) | Arch | T179 use |
|-------|-------------------------|------|----------|
| `windows-2025` | Windows Server 2025 | x64 | **Preferred pin** for required Windows job |
| `windows-latest` | → 2025 (floating) | x64 | Avoid for release; OK only if documented drift risk |
| `windows-2022` | Windows Server 2022 | x64 | Optional pin if 2025 regression |
| `ubuntu-24.04` | Ubuntu 24.04 | x64 | **Preferred pin** first non-Windows tier |
| `ubuntu-latest` | → 24.04 (floating) | x64 | Prefer explicit `ubuntu-24.04` |
| `ubuntu-26.04` | Ubuntu 26.04 **preview** | x64 | **Not** T179 pin; revisit T185 cycle |
| `macos-latest` | **macOS 26 arm64** (migrated June 2026) | arm64 | Soft job OK; smoke evidence must say **26** if used |
| `macos-15` | macOS 15 arm64 | arm64 | **Preferred soft pin** for stability (still GA) |
| `macos-26` | macOS 26 arm64 | arm64 | Explicit pin if claiming latest |
| `macos-15-intel` / `macos-26-intel` | Intel | x64 | Optional cost-sensitive |
| `macos-14` | macOS 14 | — | **Deprecated** — do **not** use |
| `ubuntu-24.04-arm` | Ubuntu 24.04 | arm64 | Optional soft T2 evidence only |
| `windows-11-arm` | Windows 11 | arm64 | Residual / not T1 |

**Practices applied:**

- **Required PR/release jobs pin explicit labels:** `windows-2025`, `ubuntu-24.04` (F24). Soft macOS: prefer **`macos-15`** or explicit **`macos-26`** — never claim “macOS 15” while running `macos-latest` (now 26) (F25).  
- Prefer **native runners** over `cross` for T1 claims (F19).  
- **Public repos:** standard runners free/unlimited. **Private:** minutes apply — lean PR matrix.  
- Do **not** require nested WSL2 Docker e2e on GHA PR (F5).  
- Footnote: Ubuntu 26.04 preview exists; T179 stays on 24.04 GA.

### 3.2 Rust CI toolchain actions

| Tool | Status | T179 recommendation |
|------|--------|---------------------|
| `actions-rs/*` | **Archived** | **Do not use** |
| `dtolnay/rust-toolchain` | Active; `@v1` is a **floating** major tag (no fine-grained releases) | Preferred installer; pass `toolchain: 1.95.0`. **Release jobs: pin action to commit SHA**; PR may use `@v1` (F26) |
| `actions-rust-lang/setup-rust-toolchain` | Active | Acceptable; same SHA-pin rule for release |
| `Swatinem/rust-cache` | Optional | Soft; SHA-pin if used on release jobs |
| `cargo-nextest` | Project **≥0.9.140** | Install per OS job |
| `cargo-deny` / `cargo-audit` | **≥0.20.2 / ≥0.22.2** | Prefer Linux job; **use exit code only** for audit (0.22.x no final summary — see `Docs/ci-tooling.md`) (F27) |

### 3.3 Dependency / library posture (relevant to multi-OS)

| Crate / component | Workspace / live | crates.io notes (2026-07-31) | License | T179 action |
|-------------------|------------------|------------------------------|---------|-------------|
| `rusqlite` | **0.39.0** `bundled` | **0.40.1** exists — hold 0.39 | MIT | **Hold**; no bump (F9) |
| SQLCipher CE (product intent) | Deferred | Cross-platform intent; Windows OpenSSL friction | BSD-style | Honesty only (F8) |
| `sqlx` | **0.8** workspace pin (`runtime-tokio-rustls`) | **0.9.0** stable (MSRV ~1.94) | MIT/Apache | **Hold 0.8**; no crate consumers today; rustls portable |
| `windows` | **0.62** (→0.62.2 available) | Hold minor | MIT/Apache | Target-gated only |
| `windows-service` | **0.8** (→0.8.1) | MIT/Apache | Windows-only; no required bump |
| `tokio` | **1.52** floor | **1.53.1** latest resolves under floor | MIT | Portable; label floor not “current” |
| `axum` | **0.8.9** | Current | MIT | Portable HTTP |
| `reqwest` | **0.13** (→0.13.4) | Hold | MIT/Apache | Portable |
| `ed25519-dalek` / `x25519-dalek` / `aes-gcm` / `hkdf` | As T176–T178 | Classical ECC | BSD / MIT-Apache | Wire tests OS-agnostic |
| Desktop `@tauri-apps/*` | **2.11.x** | Tauri v2 | Apache/MIT | Windows T1; others T2 |
| Playwright / Vitest | T174 pins | Apache/MIT | Desktop web tests; binary WebView residual |

**Verdict:** Zero new **production** crate dependencies. CI Actions are not Cargo deps. Do not bump rusqlite/sqlx/tokio for T179.

### 3.4 Platform product best practices (applied)

| Practice | Application in T179 |
|----------|---------------------|
| **Honest tiers** | T1 / T2 / T3 with evidence bar |
| **Windows-first** | Full gate + service + DPAPI + Isolation only on Windows |
| **Transport honesty** | Pipe (Win) / UDS (Unix live) / HTTP (portable) — all documented |
| **Path reality** | WSL = Linux binary + `/mnt/c` path interop, not separate distro |
| **Capture independence** | Smoke + optional `cargo tree -p ai-brains-capture` sync-absence on every T1 |
| **cfg hygiene** | Grep-complete inventory; Linux `cargo check --workspace` before clippy |
| **Cost control** | PR: Win + Linux required; macOS soft; arm optional soft only |
| **No AGPL CI** | Required |
| **SQLCipher honesty** | Exact COMPATIBILITY wording (F8 / §5.3) |
| **Desktop ≠ CLI** | Separate matrix rows; engine names honest |

### 3.5 Standards / non-claims

| Topic | Position |
|-------|----------|
| FIPS / validated modules | **Non-claim** |
| Equal primary platforms | **Forbidden** without evidence |
| SQLCipher page-level encryption on all tiers | **Non-claim** while `bundled` |
| WebView2 Isolation on macOS/Linux | **Non-claim** |
| NIST media Purge | **Non-claim** |

## 4. Frozen design decisions (F1–F32)

| ID | Decision |
|----|----------|
| **F1** | **Windows 11 x64 is tier-1 primary** for CLI, daemon (service+pipe+HTTP), capture, store, sync tests, desktop. |
| **F2** | **Ubuntu 24.04 x64** is the first non-Windows CI tier (explicit label `ubuntu-24.04`). |
| **F3** | **macOS arm64 is best-effort** until green. Soft job: pin **`macos-15`** (stable) or **`macos-26`** / document if using `macos-latest` (**= macOS 26** as of mid-2026). Smoke OS string must match runner. |
| **F4** | **WSL2** = path/vault **interop** for a **Linux binary** using `/mnt/c` (and unit path tests). Windows binary via WSL interop is covered by the **Windows** column — not a third binary distro. |
| **F5** | **No nested WSL2 e2e required on GHA PR.** |
| **F6** | Windows-only features fail closed (DPAPI, SCM, pipe SDDL, schtasks ACL, WebView2 Isolation). **Isolation claim is WebView2-only.** |
| **F7** | **Portable daemon product path** = foreground `ai-brainsd` + **loopback HTTP bearer**. Live Unix CLI still uses **UDS** (document). |
| **F8** | **Vault encryption honesty (normative COMPATIBILITY text):** “Vault storage uses **bundled SQLite** combined with **application-level Content Envelope AES-256-GCM** (P8) and OS filesystem permissions. **SQLCipher page-level encryption** remains architectural / feature-gated until CI verification.” |
| **F9** | **Zero new production Cargo dependencies.** |
| **F10** | Toolchain pin **1.95.0** authoritative; CI installs same channel/components; expand `targets` when multi-OS CI lands. |
| **F11** | deny + audit ≥ once per PR (Linux preferred); full nextest per OS tier. |
| **F12** | Capture independence is a **matrix invariant** on every T1 tier (include capture↔sync absence check). |
| **F13** | Desktop multi-OS is **not** T1 DoD; Windows desktop T1; Linux/macOS desktop T2. |
| **F14** | arm64 Win/Linux: optional soft evidence only; no T1. Without a soft job, claim **T3 unsupported** honestly rather than aspirational T2. |
| **F15** | Introduce `.github/workflows/` only on implement go-ahead. |
| **F16** | Smoke evidence under `conductor/tracks/trackT179-compatibility-matrix/evidence/`. |
| **F17** | Fail-closed Unix stubs; structured errors not panics. |
| **F18** | No AGPL/GPL CI tools as required gates. |
| **F19** | **T1 claims require native runners** (windows-2025, ubuntu-24.04, macos-15/26). `cross`/QEMU only for secondary targets (e.g. musl packaging residual). |
| **F20** | T183 consumes COMPATIBILITY.md; T185 checks platform smoke **and** that smoke runner OS matches claimed tier. |
| **F21** | Absorb T174 multi-OS residual as T2 desktop + engine honesty (WebView2 vs WKWebView vs WebKitGTK). |
| **F22** | Edition 2024 / no `unwrap`/`expect` in production for any T179 hygiene fixes. |
| **F23** | **Daemon transport matrix** §2.1.1 is normative (pipe / UDS live / HTTP portable). |
| **F24** | **Required CI jobs pin** `windows-2025` + `ubuntu-24.04` (not floating `-latest` for release). |
| **F25** | **macOS label honesty:** never publish “macOS 15 supported” evidence from `macos-latest` after it points at 26. |
| **F26** | **Release workflows pin GHA actions to commit SHA**; PR may use floating major tags. |
| **F27** | **cargo audit:** gate on **exit code** (and optional `--json`); never grep for a summary line (0.22.x). |
| **F28** | **§2.1 is illustrative;** CFG-INVENTORY must be grep-complete + list `windows` crate consumers. |
| **F29** | **Device private seed portability:** DPAPI-sealed (`datakey_dpapi`) Windows blobs are **not** openable on non-Windows; document in limitations + COMPATIBILITY. |
| **F30** | **POSIX `scripts/dev-check.sh`** (or shared gate script) mirrors `dev-check.ps1` for Linux/macOS developers — **Should**, not optional fluff. |
| **F31** | Linux CI: **`cargo check --workspace` (and/or `--all-targets`) before clippy** to fail-fast on cfg leakage. |
| **F32** | **Git askpass:** document Unix dependency on `/bin/true` (missing in some scratch containers). |

## 5. Compatibility matrix (draft normative table)

### 5.1 Support tiers

| Tier | Meaning | Evidence bar |
|------|---------|--------------|
| **T1 Supported** | Documented; CI green; smoke recorded | Native runner job + smoke checklist |
| **T2 Best-effort** | May work; issues accepted; no equality claim | Compile and/or partial tests; residual listed |
| **T3 Unsupported** | Not claimed; clear error or refuse | Document only |

### 5.2 Product × platform (proposed freeze)

| Product surface | Win11 x64 | WSL2 path interop¹ | Ubuntu 24.04 x64 | macOS arm64 (15 or 26 pin) | Linux/Win arm64 |
|-----------------|-----------|--------------------|------------------|----------------------------|-----------------|
| CLI capture + recall (FTS) | **T1** | **T1** (Linux bin + `/mnt/c`) | **T1** (after CI) | **T2→T1** if CI green | **T3** unless soft job |
| Store / migrations / projections | **T1** | via `/mnt/c` vault | **T1** | **T2→T1** | **T3** unless soft |
| Daemon Windows Service + pipe | **T1** | N/A (Windows host) | **T3** | **T3** | **T3** |
| Daemon UDS (live Unix CLI) | N/A | **T1** if Linux daemon | **T1** after compile | **T2** | **T3** |
| Daemon loopback HTTP | **T1** | **T1** if daemon reachable | **T1** | **T2→T1** | **T3** unless soft |
| DPAPI unlock | **T1** | N/A | **T3** (error) | **T3** (error) | **T3** |
| Device seed seal portability | Win DPAPI | — | DataKey-only; **no** open of Win DPAPI blobs | same | same |
| Passphrase recovery kit | **T1** | **T1** | **T1** | **T2→T1** | **T3** unless soft |
| Nightly Task Scheduler | **T1** | N/A | **T3** | **T3** | **T3** |
| Sync wire + fake-relay tests | **T1** | N/A | **T1** | **T2→T1** | **T3** unless soft |
| Desktop Tauri | **T1** WebView2+Isolation | N/A | **T2** WebKitGTK | **T2** WKWebView | **T3** |
| SQLCipher page-level CE | **Honesty T2** (bundled today) | same | same | same | same |

¹ **WSL2 column meaning (F4):** Tier applies to running the **Linux** `ai-brains` binary against vaults/paths under `/mnt/c/...`. Running the **Windows** binary is the Win11 column. Path unit tests validate string normalization on any OS.

### 5.3 Known limitations (must appear in COMPATIBILITY.md)

1. **Windows Service / named pipes** are Windows-only; Unix live CLI uses **UDS**; portable multi-OS IPC is **loopback HTTP + bearer**.  
2. **DPAPI** is Windows-only; multi-OS unlock = passphrase / recovery kit.  
3. **Device private seeds** sealed with `PROTECTION_DATAKEY_DPAPI` on Windows **cannot be opened on non-Windows**. Non-Windows uses DataKey-only sealing (weaker OS binding).  
4. **SYSTEM scheduled task ACL model** is Windows-only (T145).  
5. **Desktop Isolation** is **WebView2-only (Windows)**. macOS WKWebView / Linux WebKitGTK: no Isolation claim.  
6. **Vault encryption (F8 normative):** bundled SQLite + application-level CE AES-256-GCM + OS permissions; SQLCipher page-level is not live until feature+CI.  
7. **Models / VRAM / Ollama / llama-server** are environment-specific; not OS-tier guarantees.  
8. **Git automation** on Unix requires **`/bin/true`** (absent in some minimal containers).  
9. **Capture independence** holds; brain/nightly features may require local models on any OS.

## 6. CI matrix plan (design freeze; implement on go-ahead)

### 6.1 PR jobs (lean) — **pinned labels**

```yaml
# Illustrative — implement on go-ahead
strategy:
  matrix:
    include:
      - os: windows-2025
        target: x86_64-pc-windows-msvc
        required: true
      - os: ubuntu-24.04
        target: x86_64-unknown-linux-gnu
        required: true
      - os: macos-15   # or macos-26; document choice in smoke
        target: aarch64-apple-darwin
        required: false
```

| Job ID | Runner | Steps | Required? |
|--------|--------|-------|-----------|
| `gate-windows` | **`windows-2025`** | fmt, clippy workspace, nextest workspace | **Yes** |
| `gate-linux` | **`ubuntu-24.04`** | toolchain 1.95.0 → **`cargo check --workspace`** → clippy → nextest → **deny** → **audit (exit code)** | **Yes** (after first green) |
| `gate-macos` | **`macos-15`** or **`macos-26`** | compile + nextest core or full | **Soft** |

### 6.2 Release / scheduled jobs

| Job ID | Runner | Notes |
|--------|--------|-------|
| `release-windows` | `windows-2025` | Full gate + smoke; **SHA-pinned actions** |
| `release-linux` | `ubuntu-24.04` | Full gate + smoke; SHA-pinned |
| `release-macos` | explicit `macos-15` or `macos-26` | Full or CLI-focused; smoke OS matches label |
| `wsl-smoke` | windows + optional setup-wsl | `workflow_dispatch` only |
| `arm-linux-soft` | `ubuntu-24.04-arm` optional | Soft T2 evidence only |

### 6.3 Smoke checklist (all T1 OS tiers)

Record in `evidence/SMOKE-<os>.md` (include **exact runner label** + OS version):

```text
1. rustc -V / cargo -V match pin 1.95.0
2. cargo check --workspace (Linux/macOS)
3. cargo nextest (workspace or agreed packages)
4. Capture independence + capture has no ai-brains-sync dep (tree/toml gate)
5. Path unit tests (incl. WSL string forms where applicable)
6. Daemon: document which transport smoked (pipe / UDS / HTTP)
7. HTTP health/bearer if portable IPC claimed
8. Sync hermetic tests if claimed
9. Explicit non-claims for this OS (DPAPI, service, Isolation, SQLCipher page-level)
10. Device seed: note DPAPI vs DataKey-only for this OS
```

### 6.4 Local gate parity

| Script | Role |
|--------|------|
| `scripts/dev-check.ps1` | Windows developer full gate (existing) |
| `scripts/dev-check.sh` | **POSIX mirror** (F30) — fmt, clippy, nextest, deny, audit exit codes |
| Optional shared | Extract common cargo sequence to avoid drift (AI2 improvement A — soft) |

### 6.5 Desktop (soft)

- Windows: T174 L2–L4 remain desktop gate.  
- Linux/macOS desktop: not PR-required.  
- Document rendering engines; no WDIO release plugins.

## 7. Acceptance criteria

| ID | Criterion |
|----|-----------|
| **AC1** | `Docs/COMPATIBILITY.md` with §5 table + §5.3 limitations + F8 vault wording |
| **AC2** | **Grep-complete** `evidence/CFG-INVENTORY.md` (not just §2.1) + `windows` consumers list |
| **AC3** | CI workflows: required `windows-2025` + `ubuntu-24.04` (or residual with human multi-OS evidence — prefer CI) |
| **AC4** | Smoke evidence per T1 tier; **runner label matches claimed OS** |
| **AC5** | Capture independence on each T1 tier |
| **AC6** | Unix build: fail-closed Windows APIs; no production unwrap/expect introduced |
| **AC7** | deny + audit green; audit gated by exit code |
| **AC8** | T174 multi-OS residual closed or restated T2 + engine honesty |
| **AC9** | Conductor + deferred updated; no overstated platform claims |
| **AC10** | Zero new production Cargo dependencies |
| **AC11** | Transport matrix (pipe/UDS/HTTP) documented; Unix UDS path verified |
| **AC12** | Device seed DPAPI non-portability documented |
| **AC13** | `scripts/dev-check.sh` exists **or** justified residual with Linux CI-only gate |

## 8. Deferred.md absorption

| Deferred / residual | Disposition |
|---------------------|-------------|
| **T174 §49 / #49** multi-OS visual / WDIO | **Absorb** T2 desktop + engine honesty |
| **PRD / Implementation-Plan** secondary Ubuntu/WSL | **Absorb** F2/F4 |
| **Vision** Windows 11 + WSL first production | **Absorb** F1/F4 |
| **Deviations §1** SQLCipher deferred | **Absorb** F8 exact wording |
| **#34.2** DataKey rotation | **Out of scope** |
| **T178** crypto residuals (except OS matrix for private_blob) | Wire OS-agnostic; **private_blob DPAPI** folded (F29) |
| **T180** protocol N-1 | **Out of scope** |
| **arm64 matrix** | Soft job or **T3** honesty (F14) |
| **systemd / launchd** | Residual |
| **Unify Unix CLI to HTTP-only** | Residual (F23 optional) |

## 9. Non-goals

| Out of scope | Owner |
|--------------|--------|
| Equal primary platforms without evidence | Forbidden |
| Full Windows Service parity on Linux/macOS | Residual |
| Re-enable `bundled-sqlcipher` as DoD | Encryption hygiene track |
| App Store / notarization / MSI | Packaging / T185 |
| Nested WSL2 Docker e2e PR gate | F5 |
| Electron | Forbidden |
| AGPL CI agents | Forbidden |
| Claiming Unix DaemonClient already uses HTTP | Forbidden (live = UDS) |
| #34.2 DataKey rotation | Separate |
| Protocol wire N-1 goldens | **T180** |
| Claims/SBOM packaging | **T185** |

## 10. License / commercial constraints

- Prefer GitHub-hosted runners + existing Rust toolchain — no AGPL CI agents.  
- `cross` / Docker base images: document licenses if used; not required for T1.  
- Crates MIT/Apache/BSD allowlist (`deny.toml`).  
- Product: PolyForm NC + Small-Entity Commercial Exception unchanged.  
- Desktop npm license:check remains Windows-primary evidence.

## 11. Risks

| Risk | Mitigation |
|------|------------|
| No GHA today → first Linux build finds cfg bugs | Phase B inventory + check-before-clippy (F31); optional WSL/cross dry-run |
| macOS label drift (15 vs 26) | F3/F25 explicit pins; T185 asserts smoke matches label |
| Overclaim SQLCipher | F8 exact text |
| DPAPI device seed cross-OS open fails | F29 documented limitation |
| UDS vs HTTP confusion | F23 transport matrix |
| Floating GHA action tags | F26 SHA pin on release |
| audit stdout parse foot-gun | F27 exit code only |
| Desktop engine confusion | F21 engine names in COMPATIBILITY |
| `/bin/true` missing in scratch | F32 documented |

## 12. Definition of Done

- [ ] F1–F32 reflected in COMPATIBILITY.md + plan evidence  
- [ ] AC1–AC13 satisfied or residual-listed with owner  
- [ ] CI jobs for windows-2025 + ubuntu-24.04 required (prefer)  
- [ ] macOS status explicit (T1 or T2) with matching runner label  
- [ ] Grep-complete CFG inventory  
- [ ] T174 multi-OS residual absorbed  
- [ ] deny + audit green; no new prod deps  
- [ ] Conductor → Completed after review; deferred updated  

## 13. Implementation priority (see plan.md)

1. **Phase A** — Grep-complete inventory + COMPATIBILITY.md (F8/F23/F29 wording)  
2. **Phase B** — Unix compile hygiene + private_blob/UDS/DPAPI fail-closed proofs  
3. **Phase C** — GHA pinned labels + check-before-clippy + audit exit code + optional dev-check.sh  
4. **Phase D** — Smoke evidence + T183/T185 handoffs  
5. **Phase E** — Gate, review, closeout  

**Still design-only until human go-ahead.**

## 14. AI review disposition (fold-in record)

| Source | Item | Disposition |
|--------|------|-------------|
| AI1 BS1 | Unix daemon_client → HTTP | **Partial agree** — live is **UDS**, not HTTP. Folded as **F23** transport matrix (pipe/UDS/HTTP). Tests for UDS path; HTTP as portable product surface |
| AI1 BS2 | F8 vault wording | **Agree** — normative text in F8 / §5.3 |
| AI1 BS3 | Pin GHA labels | **Agree** — **F24**; **reject macos-14** (deprecated per AI2) |
| AI1 BS4 | Native vs cross / F19 | **Agree** — reaffirmed F19 |
| AI1 Opp1 | `dev-check.sh` | **Agree** — **F30** / AC13 |
| AI1 Opp2 | Linux `cargo check` leakage | **Agree** — **F31** |
| AI2 #1 | macos-latest = 26 | **Agree** — **F3/F25** |
| AI2 #2 | rusqlite 0.40 hold | **Agree** — §3.3 corrected |
| AI2 #3 | sqlx row | **Agree with nuance** — workspace pin exists; **no crate consumers**; hold 0.8 |
| AI2 #4 | §2.1 incomplete inventory | **Agree** — **F28** / AC2 |
| AI2 #5 | private_blob DPAPI | **Agree** — **F29** high honesty fix |
| AI2 #6 | SHA-pin actions | **Agree** — **F26** |
| AI2 #7 | cargo audit exit code | **Agree** — **F27** |
| AI2 #8 | tokio floor labeling | **Agree** — §3.3 |
| AI2 #9 | ubuntu-26.04 preview footnote | **Agree** — §3.1 |
| AI2 #10 | windows consumers list | **Agree** — inventory subsection |
| AI2 #11 | dev-check.ps1 alignment | **Agree** — reuse versions |
| AI2 A | Shared gate script | **Soft agree** — prefer dev-check.sh; shared extract optional |
| AI2 B | check before clippy | **Agree** — F31 |
| AI2 C | git askpass `/bin/true` | **Agree** — **F32** |
| AI2 D | arm soft job vs T3 | **Agree** — F14 honesty |
| AI2 E | rust-cache SHA | **Agree** if used on release |
| AI2 F | first Linux dry-run | **Agree** — plan B0 |
| AI2 G | WSL column clarify | **Agree** — §5.2 note ¹ |
| AI2 H | Desktop engine honesty | **Agree** — F6/F21 |
| AI2 | Unify Unix CLI to HTTP as DoD | **Reject as DoD** — residual only (F23 optional) |
