# AI-Brains Compatibility Matrix

**Track:** T179 (P12.1)  
**Status:** Normative product matrix (Windows-first)  
**Toolchain pin:** `rust-toolchain.toml` channel **1.95.0**  
**Related:** [ci-tooling.md](ci-tooling.md) · [OPERATIONS.md](OPERATIONS.md) · [CAPABILITIES.md](CAPABILITIES.md) · [evidence/](../conductor/tracks/trackT179-compatibility-matrix/evidence/) · [CFG inventory](../conductor/tracks/trackT179-compatibility-matrix/evidence/CFG-INVENTORY.md)

This document is the **honest** platform support surface for AI-Brains. Windows 11 x64 remains the primary product environment. Other tiers are claimed only where CI or recorded smoke proves them. Do **not** market equal primary platforms without matching evidence.

---

## 1. Support tiers

| Tier | Meaning | Evidence bar |
|------|---------|--------------|
| **T1 Supported** | Documented; CI green; smoke recorded | Native runner job + smoke checklist under `evidence/SMOKE-*.md` |
| **T2 Best-effort** | May work; issues accepted; no equality claim | Compile and/or partial tests; residual listed |
| **T3 Unsupported** | Not claimed; clear error or refuse | Document only |

### Matrix invariant — capture independence

On **every T1 tier**, the capture path (CLI → daemon → event log) **must** remain functional **without** dependencies on models, embeddings, graph databases, or sync. This is an AGENTS.md mandate and a T179 matrix invariant (F12). Optional: `cargo tree -p ai-brains-capture` must not pull `ai-brains-sync` as a dependency on T1 jobs.

---

## 2. Product × platform

| Product surface | Win11 x64 | WSL2 path interop¹ | Ubuntu 24.04 x64 | macOS arm64 | Linux/Win arm64 |
|-----------------|-----------|--------------------|------------------|-------------|-----------------|
| CLI capture + recall (FTS) | **T1** | **T1** (Linux bin + `/mnt/c`) | **T1** (WSL + required GHA) | **T2** (soft CI) | **T3** |
| Store / migrations / projections | **T1** | via `/mnt/c` vault | **T1** (WSL + required GHA) | **T2** | **T3** |
| Daemon Windows Service + named pipe | **T1** | N/A (Windows host) | **T3** | **T3** | **T3** |
| Daemon UDS (live Unix CLI) | N/A | **T1** if Linux daemon | **T1** (UDS unit + compile) | **T2** | **T3** |
| Daemon loopback HTTP + bearer | **T1** | **T1** if daemon reachable | **T1** (hermetic HTTP tests) | **T2** | **T3** |
| DPAPI unlock | **T1** | N/A | **T3** (error) | **T3** (error) | **T3** |
| Device seed seal portability | Win DPAPI | — | DataKey-only; **cannot open** Win DPAPI blobs | same | same |
| Passphrase recovery kit | **T1** | **T1** | **T1** (WSL + required GHA) | **T2** | **T3** |
| Nightly Task Scheduler | **T1** | N/A | **T3** | **T3** | **T3** |
| Sync wire + fake-relay tests | **T1** | N/A | **T1** (WSL nextest) | **T2** | **T3** |
| Desktop Tauri | **T1** WebView2 + Isolation | N/A | **T2** WebKitGTK (excluded from required CI) | **T2** WKWebView | **T3** |
| SQLCipher page-level vault | **T1 live** (T187 `bundled-sqlcipher-vendored-openssl`) | **T1 live** | same build | same build | same build |

**Evidence bar:** Windows T1 = local full gate + required `windows-2025` job. Ubuntu T1 core = WSL dry-run recorded in `evidence/SMOKE-linux.md` / `UNIX-BUILD.md` + required `ubuntu-24.04` job (desktop excluded). First GHA greens are recorded on the PR; do not claim GHA label green from WSL alone.

¹ **WSL2 column (F4):** Tier applies to running the **Linux** `ai-brains` binary against vaults/paths under `/mnt/c/...`. Running the **Windows** binary is the Win11 column. Path unit tests validate string normalization on any OS. Nested WSL2 e2e is **not** a PR gate.

**arm64:** Without a soft job, arm64 is **T3 Unsupported** (F14). Optional soft `ubuntu-24.04-arm` evidence may promote to T2 later — not claimed here.

---

## 3. Daemon transport matrix (F23)

| OS | Live `DaemonClient` transport | Portable product control plane |
|----|------------------------------|--------------------------------|
| **Windows** | Named pipe `\\.\pipe\ledgerful-bridge` | Loopback HTTP + bearer (T161) also available |
| **Unix (Linux/macOS)** | **Unix domain socket** (T195 shared resolver) | Loopback HTTP + bearer (T161) — preferred multi-OS product surface |

**Unix UDS path order (daemon bind + CLI connect share the same helper):**
1. Absolute `AI_BRAINS_DAEMON_SOCKET` (relative → fail closed)
2. Else valid `$XDG_RUNTIME_DIR/ledgerful-bridge.sock` (dir must exist, mode `0700`, uid == euid; AI-Brains does not create XDG)
3. Else `/tmp/ledgerful-bridge.sock` + runtime warning (fallback residual, common on macOS)

- Do **not** claim that Unix always lives under `/tmp` — XDG-first when valid.
- Do **not** claim that Unix already defaults to HTTP — live CLI code uses **UDS**.
- Portable multi-OS smoke should exercise **HTTP health/bearer** when claiming portable daemon IPC.
- Prior `/tmp`-hardcoded external clients need `AI_BRAINS_DAEMON_SOCKET` on daemon **and** client when the daemon uses XDG (see CHANGELOG / OPERATIONS).
- Optional residual (not T179 DoD): unify Unix CLI→daemon to prefer HTTP.

---

## 4. Vault encryption honesty (F8 — exact wording)

Vault storage uses **SQLCipher page-level encryption** (T187: workspace `rusqlite` features `bundled-sqlcipher-vendored-openssl` + `backup` + `fallible_uint`) combined with **application-level Content Envelope AES-256-GCM** (P8) and OS filesystem permissions.

- New vaults under a correct key workflow do **not** have a plain `SQLite format 3` header.
- Wrong key on open / backup verify fails closed (`VaultLocked` / key-verification class).
- Legacy plain SQLite vaults fail with `LegacyPlaintextVault` and a migrate hint; operators convert with `ai-brains vault encrypt` (`sqlcipher_export`, not Online Backup).
- All-zero keys are refused unless `AI_BRAINS_ALLOW_ZERO_KEY=1` (tests/legacy only).
- `PRAGMA cipher_compatibility = 4`; do **not** set `cipher_plaintext_header_size` (full header encrypted).
- Observed `PRAGMA cipher_version` (2026-08-02 Windows MSVC / `bundled-sqlcipher-vendored-openssl`): **`4.10.0 community`** (unit smoke T187-V-01; track evidence `conductor/tracks/trackT187-sqlcipher-page-encryption/cipher_version.txt`). Re-probe after toolchain upgrades.
- **Not** FIPS validated; **not** NIST SP 800-88 Purge/Destroy. Page key bound to DataKey via `SqlCipherKey::from_data_key`; ceremony rotation is **T189 / ADR-0020** (`vault rotate-datakey`).

See `Docs/Deviations.md` §1 (resolved by T187).

---

## 5. Device private seed portability (F29)

| Platform | Seal label | Notes |
|----------|------------|-------|
| Windows | `datakey_dpapi` (`PROTECTION_DATAKEY_DPAPI`) | Outer DPAPI on ct‖tag after AES-GCM under vault DataKey |
| Non-Windows | `datakey` (`PROTECTION_DATAKEY`) | DataKey-only (weaker OS binding) |

**Windows DPAPI-sealed device private seeds cannot be opened on non-Windows.** Opening a `datakey_dpapi` blob off Windows fails closed with a clear DPAPI-related error. Multi-OS unlock of vault material uses passphrase / recovery kit paths, not DPAPI.

---

## 6. Desktop rendering engines (F6 / F21)

| OS | Engine | Isolation claim |
|----|--------|-----------------|
| Windows | **WebView2** | **Isolation supported** (Windows-only product claim) |
| macOS | **WKWebView** | **No Isolation claim** |
| Linux | **WebKitGTK** | **No Isolation claim** |

Desktop multi-OS is **not** a T1 DoD. Windows desktop = T1; Linux/macOS desktop = T2. No WDIO release plugins required for T179. T174 multi-OS / WDIO residual is absorbed as T2 desktop + engine honesty.

---

## 7. Git askpass (F32)

| OS | Program |
|----|---------|
| Windows | `git-askpass-noop.cmd` (dev tree / packaged `scripts/`, or fail-closed `cmd.exe` residual) |
| Unix | **`/bin/true`** |

Some minimal/scratch containers lack `/bin/true`. Git automation on Unix requires a present no-op askpass binary.

---

## 8. Known limitations

1. **Windows Service / named pipes** are Windows-only; Unix live CLI uses **UDS**; portable multi-OS IPC is **loopback HTTP + bearer**.
2. **DPAPI** is Windows-only; multi-OS unlock = passphrase / recovery kit.
3. **Device private seeds** sealed with `PROTECTION_DATAKEY_DPAPI` on Windows **cannot be opened on non-Windows**. Non-Windows uses DataKey-only sealing (weaker OS binding).
4. **SYSTEM scheduled task ACL model** is Windows-only (T145).
5. **Desktop Isolation** is **WebView2-only (Windows)**. macOS WKWebView / Linux WebKitGTK: no Isolation claim.
6. **Vault encryption (F8 / T187):** SQLCipher page-level live (`bundled-sqlcipher-vendored-openssl`) + application-level CE AES-256-GCM + OS permissions; wrong-key fail-closed; zero-key refuse unless escape hatch.
7. **Models / VRAM / Ollama / llama-server** are environment-specific; not OS-tier guarantees.
8. **Git automation** on Unix requires **`/bin/true`** (absent in some minimal containers).
9. **Capture independence** holds; brain/nightly features may require local models on any OS.
10. **arm64** (Win/Linux) is **T3** unless a soft job is added later.
11. **Reference systemd and launchd unit templates** are provided under `packaging/reference/`; automated installer management on Unix is not claimed; not T1 multi-OS service parity.
12. **App Store / notarization / MSI** packaging are T183/T185 residuals — not T179 DoD.

---

## 9. CI runner pins (F24 / F25)

| Job | Runner label | Required? | Role |
|-----|--------------|-----------|------|
| `gate-windows` | **`windows-2025`** | **Yes** | fmt, clippy workspace all-targets, nextest workspace |
| `gate-linux` | **`ubuntu-24.04`** | **Yes** | toolchain 1.95.0 → cargo check/clippy/nextest **`--exclude ai-brains-desktop`** → deny → audit (exit code) |
| `gate-macos` | **`macos-15`** | **Soft** (`continue-on-error: true`) | cargo check + nextest **exclude desktop**; residual T2 until green |

**Desktop exclusion (F13 honesty):** Required Linux/macOS jobs exclude the Tauri `ai-brains-desktop` crate. Full desktop build on those OS needs WebKitGTK / WKWebView system packages and remains **T2** (Windows desktop stays T1 in the Windows job).

- Prefer explicit labels over floating `-latest` for release evidence.
- **Never** claim “macOS 15 supported” from `macos-latest` after it points at macOS 26 (F25). Soft pin for T179 is **`macos-15`**.
- **Do not** use deprecated `macos-14`.
- **Do not** use archived `actions-rs/*`.
- PR `ci.yml` and release jobs **SHA-pin** third-party actions (T186 / R-CI-PIN + F26). Dependabot `github-actions` bumps pins.
- **cargo audit:** gate on **exit code** only (F27); never grep for a final summary line (`cargo-audit` 0.22.x). See [ci-tooling.md](ci-tooling.md).
- No AGPL CI tooling.

Workflow: [`.github/workflows/ci.yml`](../.github/workflows/ci.yml).

Local gates:

| Script | Role |
|--------|------|
| `scripts/dev-check.ps1` | Windows full gate |
| `scripts/dev-check.sh` | POSIX mirror (Linux/macOS developers) |

---

## 10. Handoffs

### T183 — Install docs

- Windows-first install order; document PowerShell and MSVC toolchain as primary.
- Unix: document UDS live path + HTTP portable path; do not force CLI→HTTP-only.
- Call out DPAPI device seed non-portability and passphrase recovery for multi-OS.
- Git askpass: ship `git-askpass-noop.cmd` on Windows; ensure `/bin/true` on Unix images.
- Desktop: document WebView2 / WKWebView / WebKitGTK; Isolation Windows-only.
- See `evidence/HANDOFF-T183-T185.md`.

### T185 — Release gate

- Platform smoke checkbox must use a **runner label that matches** the claimed COMPATIBILITY tier (F20).
- Reaffirm **F8** SQLCipher honesty — page-level is live (T187); still forbid FIPS / NIST Purge / perfect-deletion claims.
- macOS smoke OS string must match pin (`macos-15` vs `macos-26`).
- See `evidence/HANDOFF-T183-T185.md`.

---

## 11. Evidence index

| Artifact | Purpose |
|----------|---------|
| [CFG-INVENTORY.md](../conductor/tracks/trackT179-compatibility-matrix/evidence/CFG-INVENTORY.md) | Grep-complete Windows/Unix surfaces |
| [SMOKE-windows.md](../conductor/tracks/trackT179-compatibility-matrix/evidence/SMOKE-windows.md) | Windows smoke (`windows-2025`) |
| [SMOKE-linux.md](../conductor/tracks/trackT179-compatibility-matrix/evidence/SMOKE-linux.md) | Linux smoke (`ubuntu-24.04`) |
| [SMOKE-macos.md](../conductor/tracks/trackT179-compatibility-matrix/evidence/SMOKE-macos.md) | macOS soft residual (`macos-15`) |
| [SMOKE-wsl.md](../conductor/tracks/trackT179-compatibility-matrix/evidence/SMOKE-wsl.md) | Optional WSL path interop |
| [UNIX-BUILD.md](../conductor/tracks/trackT179-compatibility-matrix/evidence/UNIX-BUILD.md) | First Linux breakages + fixes |
| [HANDOFF-T183-T185.md](../conductor/tracks/trackT179-compatibility-matrix/evidence/HANDOFF-T183-T185.md) | Install + release handoffs |

---

## 12. Non-claims

| Topic | Position |
|-------|----------|
| FIPS / validated modules | Non-claim |
| Equal primary platforms without evidence | Forbidden |
| FIPS / NIST Purge page encryption | Non-claim (T187 ships community SQLCipher + vendored OpenSSL, not FIPS) |
| WebView2 Isolation on macOS/Linux | Non-claim |
| Nested WSL2 Docker e2e as PR gate | Non-claim (F5) |
| Unix DaemonClient already uses HTTP | Forbidden (live = UDS) |
