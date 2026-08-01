# T179 CFG Inventory — Windows / non-Windows Bifurcation

**Track:** T179 Compatibility Matrix (P12.1)  
**Scan date:** 2026-07-31  
**Method:** Workspace-wide search of `cfg(windows)`, `cfg(not(windows))`, and `target.'cfg(windows)'` across `*.rs` / `*.toml` (excluding `.git/`, `target/`, `node_modules/`).  
**Authority:** Spec F28 / AC2 — this document is **grep-complete for the three primary patterns**. Spec §2.1 is **illustrative only** and must not be treated as exhaustive.

> **Method note (R1-L1 / Codex P3):** Phase B also uses `cfg(any(windows, test))` for pure parsers exercised in tests on non-Windows. Those sites are **Windows-or-test** gates (not product Unix paths) and may not appear in the primary three-pattern site count. Re-scan with `cfg(any(windows` if auditing test-only gates.

## Summary counts (re-count at implement)

| Metric | Count |
|--------|------:|
| **Match sites** (line occurrences of the three patterns) | **123** |
| **Files** containing at least one match | **34** |
| **Crates / packages** with bifurcation | **9** |
| **`windows` crate consumers** (target-gated) | **6** |

> Spec AI2 scan noted ~75 sites / 18 files / 11 crates as of an earlier pass. This implement re-count is authoritative for T179 closeout evidence.

### Pattern breakdown (approximate)

| Pattern | Role |
|---------|------|
| `#[cfg(windows)]` / `#![cfg(windows)]` | Windows-only code paths, modules, tests |
| `#[cfg(not(windows))]` | Unix/macOS stubs, UDS, DataKey-only, mode-based ACL |
| `[target.'cfg(windows)'.dependencies]` | Cargo target-gated deps (`windows`, `windows-service`) |

---

## `windows` crate consumers (all six — target-gated)

These packages declare `windows` (and in one case `windows-service`) under `[target.'cfg(windows)'.dependencies]` so the Windows SDK crate **must not** enter the Unix dependency graph.

| # | Package | Manifest | Version / notes | Primary surface |
|---|---------|----------|-----------------|-----------------|
| 1 | `ai-brains-api-server` | `crates/ai-brains-api-server/Cargo.toml` | `windows.workspace = true` (0.62) | HTTP token owner-only ACL |
| 2 | `ai-brains-path` | `crates/ai-brains-path/Cargo.toml` | `windows` workspace | Reparse / symlink detection |
| 3 | `ai-brains-crypto` | `crates/ai-brains-crypto/Cargo.toml` | `windows` workspace | DPAPI wrap/unwrap |
| 4 | `ai-brainsd` | `crates/ai-brainsd/Cargo.toml` | `windows` + `windows-service = "0.8"` | SCM service host, pipe SDDL |
| 5 | `ai-brains-cli` | `crates/ai-brains-cli/Cargo.toml` | `windows` workspace | Elevation, artifact ACL, schtasks helpers |
| 6 | `apps/desktop/src-tauri` | `apps/desktop/src-tauri/Cargo.toml` | `windows = "0.62"` (Registry, messaging) | WebView2 Isolation / registry |

**Not consumers of the `windows` crate** but still cfg-bifurcated: `ai-brains-git`, `ai-brains-sources`, `ai-brains-sync` (pure `cfg` without Win32 bindings).

Workspace pin: root `Cargo.toml` `windows = { version = "0.62", features = [...] }`.

---

## Surfaces grouped by product concern

### DPAPI key wrap (`ai-brains-crypto`)

| File | Sites | Behavior |
|------|------:|----------|
| `crates/ai-brains-crypto/src/dpapi.rs` | 5 | Win: `CryptProtectData` / `CryptUnprotectData`. Non-Win: `Err("DPAPI is only available on Windows")`. |
| `crates/ai-brains-crypto/src/recovery_kit.rs` | 1 | Optional DPAPI path in recovery kit on Windows. |
| `crates/ai-brains-crypto/tests/crypto_recovery.rs` | 2 | Windows-only recovery assertions. |
| `crates/ai-brains-crypto/Cargo.toml` | 1 | Target-gated `windows` dep. |

### Device private seed seal (`private_blob`)

| File | Sites | Behavior |
|------|------:|----------|
| `crates/ai-brains-sync/src/private_blob.rs` | 4 | Win seal → `PROTECTION_DATAKEY_DPAPI` (DPAPI outer). Non-Win → `PROTECTION_DATAKEY` only. Open of `datakey_dpapi` calls `dpapi::unwrap_key` (fails closed off Windows / on junk). **F29:** DPAPI-sealed seeds not portable to non-Windows. |
| `crates/ai-brains-sync/src/relay.rs` | 2 | Path/temp helpers differ by OS. |
| `crates/ai-brains-cli/src/commands/device.rs` | 2 | Device CLI private-key write / DPAPI paths; clear fail-closed off Windows. |
| `crates/ai-brains-cli/tests/device_replicate_cli.rs` | 1 | Windows-gated device CLI coverage. |

### Windows Service + named pipe SDDL (`ai-brainsd`)

| File | Sites | Behavior |
|------|------:|----------|
| `crates/ai-brainsd/src/windows_service.rs` | 1 | Entire module `#![cfg(windows)]` — SCM host. |
| `crates/ai-brainsd/src/pipe_security.rs` | 1 | Entire module `#![cfg(windows)]` — pipe SDDL. |
| `crates/ai-brainsd/src/lib.rs` | 2 | Conditional module exports. |
| `crates/ai-brainsd/src/main.rs` | 9 | Service dispatch vs foreground; Unix foreground-only stubs. |
| `crates/ai-brainsd/Cargo.toml` | 1 | `windows` + `windows-service`. |

### Named pipe vs UDS (`daemon_client` + daemon CLI)

| File | Sites | Behavior |
|------|------:|----------|
| `crates/ai-brains-cli/src/daemon_client.rs` | 9 | **F23:** Win → named pipe `\\.\pipe\ledgerful-bridge`. Non-Win → UDS `/tmp/ledgerful-bridge.sock`. Detached spawn flags Windows-only. |
| `crates/ai-brains-cli/src/commands/daemon.rs` | 7 | `install`/`uninstall` service ops Windows-only; non-Win messages fail closed. |

### Task Scheduler / nightly / elevation

| File | Sites | Behavior |
|------|------:|----------|
| `crates/ai-brains-cli/src/commands/nightly.rs` | 3 | `schtasks` schedule state Windows-only; non-Win: `"Scheduled: (unknown on non-Windows)"`. |
| `crates/ai-brains-cli/src/elevation.rs` | 7 | UAC re-launch / elevated process helpers Windows-only; non-Win stubs. |

### Artifact ACL / ProgramData security

| File | Sites | Behavior |
|------|------:|----------|
| `crates/ai-brains-cli/src/artifact_security.rs` | 18 | Largest surface: SDDL/ACL, reparse refuse, ProgramData hardening (T145). Unix: mode-based / stub paths. |

### HTTP token ACL (`ai-brains-api-server`)

| File | Sites | Behavior |
|------|------:|----------|
| `crates/ai-brains-api-server/src/token.rs` | 9 | Owner-only ACL via Win32; Unix `0600`. |
| `crates/ai-brains-api-server/Cargo.toml` | 1 | Target-gated `windows`. |

### Path reparse / location / WSL string forms

| File | Sites | Behavior |
|------|------:|----------|
| `crates/ai-brains-path/src/reparse.rs` | 3 | Win reparse-point detection; non-Win symlink/best-effort. |
| `crates/ai-brains-path/src/location.rs` | 2 | Drive-case lowercasing Windows; preserve case non-Win. |
| `crates/ai-brains-path/tests/symlink_resolution_best_effort.rs` | 1 | Windows-focused reparse/symlink tests. |
| `crates/ai-brains-path/Cargo.toml` | 1 | Target-gated `windows`. |

### Desktop WebView2 Isolation

| File | Sites | Behavior |
|------|------:|----------|
| `apps/desktop/src-tauri/src/webview2.rs` | 8 | Isolation / WebView2 Windows-only; non-Win returns unsupported / no Isolation claim. |
| `apps/desktop/src-tauri/Cargo.toml` | 1 | Target-gated `windows` (Registry, messaging). |

### Git askpass (`ai-brains-git`)

| File | Sites | Behavior |
|------|------:|----------|
| `crates/ai-brains-git/src/command.rs` | 5 | Win: `git-askpass-noop.cmd` resolution. Non-Win: **`/bin/true`** (**F32** — may be missing in scratch containers). |

### Vault filesystem / connectors

| File | Sites | Behavior |
|------|------:|----------|
| `crates/ai-brains-sources/src/vault_fs.rs` | 1 | Windows-specific FS edge (reparse / path). |
| `crates/ai-brains-sources/tests/markdown_obsidian_connector.rs` | 2 | Windows path cases. |

### Shadow vault / migrate / CLI tests

| File | Sites | Behavior |
|------|------:|----------|
| `crates/ai-brains-cli/src/commands/shadow.rs` | 2 | Path / live-target refusal helpers. |
| `crates/ai-brains-cli/src/commands/migrate.rs` | 2 | OS path fixtures. |
| `crates/ai-brains-cli/tests/shadow_vault_refuses_live_target.rs` | 2 | Win vs non-Win path construction. |
| `crates/ai-brains-cli/tests/migrate_governed.rs` | 6 | Win vs non-Win temp/path fixtures. |

---

## Complete file list (34) with site counts

| Sites | Path |
|------:|------|
| 18 | `crates/ai-brains-cli/src/artifact_security.rs` |
| 9 | `crates/ai-brains-cli/src/daemon_client.rs` |
| 9 | `crates/ai-brains-api-server/src/token.rs` |
| 9 | `crates/ai-brainsd/src/main.rs` |
| 8 | `apps/desktop/src-tauri/src/webview2.rs` |
| 7 | `crates/ai-brains-cli/src/elevation.rs` |
| 7 | `crates/ai-brains-cli/src/commands/daemon.rs` |
| 6 | `crates/ai-brains-cli/tests/migrate_governed.rs` |
| 5 | `crates/ai-brains-crypto/src/dpapi.rs` |
| 5 | `crates/ai-brains-git/src/command.rs` |
| 4 | `crates/ai-brains-sync/src/private_blob.rs` |
| 3 | `crates/ai-brains-cli/src/commands/nightly.rs` |
| 3 | `crates/ai-brains-path/src/reparse.rs` |
| 2 | `crates/ai-brains-cli/src/commands/device.rs` |
| 2 | `crates/ai-brains-cli/src/commands/migrate.rs` |
| 2 | `crates/ai-brains-cli/src/commands/shadow.rs` |
| 2 | `crates/ai-brains-cli/tests/shadow_vault_refuses_live_target.rs` |
| 2 | `crates/ai-brains-crypto/tests/crypto_recovery.rs` |
| 2 | `crates/ai-brains-path/src/location.rs` |
| 2 | `crates/ai-brains-sources/tests/markdown_obsidian_connector.rs` |
| 2 | `crates/ai-brains-sync/src/relay.rs` |
| 2 | `crates/ai-brainsd/src/lib.rs` |
| 1 | `apps/desktop/src-tauri/Cargo.toml` |
| 1 | `crates/ai-brains-api-server/Cargo.toml` |
| 1 | `crates/ai-brains-cli/Cargo.toml` |
| 1 | `crates/ai-brains-cli/tests/device_replicate_cli.rs` |
| 1 | `crates/ai-brains-crypto/Cargo.toml` |
| 1 | `crates/ai-brains-crypto/src/recovery_kit.rs` |
| 1 | `crates/ai-brains-path/Cargo.toml` |
| 1 | `crates/ai-brains-path/tests/symlink_resolution_best_effort.rs` |
| 1 | `crates/ai-brains-sources/src/vault_fs.rs` |
| 1 | `crates/ai-brainsd/Cargo.toml` |
| 1 | `crates/ai-brainsd/src/pipe_security.rs` |
| 1 | `crates/ai-brainsd/src/windows_service.rs` |

**Sum:** 123 sites across 34 files.

---

## Crates with bifurcation (9)

| Crate / package | Role |
|-----------------|------|
| `ai-brains-api-server` | Token ACL |
| `ai-brains-cli` | Daemon client, elevation, artifact ACL, commands |
| `ai-brains-crypto` | DPAPI |
| `ai-brains-git` | Askpass |
| `ai-brains-path` | Reparse / location |
| `ai-brains-sources` | vault_fs / connector tests |
| `ai-brains-sync` | private_blob / relay |
| `ai-brainsd` | Service + pipe |
| `apps/desktop/src-tauri` | WebView2 |

---

## Notes for Phase B (Unix compile hygiene)

1. Windows-only modules already use `#![cfg(windows)]` (`windows_service`, `pipe_security`) — good isolation.
2. Fail-closed stubs exist for DPAPI, elevation, daemon install, Isolation — verify on first Linux `cargo check --workspace`.
3. Live Unix `DaemonClient` uses **UDS**, not HTTP (F23). Portable product IPC remains loopback HTTP + bearer.
4. First Linux breakages → record in `evidence/UNIX-BUILD.md`.

## Re-scan command (PowerShell)

```powershell
Get-ChildItem -Path . -Recurse -Include *.rs,*.toml -File |
  Where-Object { $_.FullName -notmatch '\\\.git\\|\\target\\|\\node_modules\\' } |
  Select-String -Pattern "cfg\(windows\)|cfg\(not\(windows\)\)|target\.'cfg\(windows\)'"
```
