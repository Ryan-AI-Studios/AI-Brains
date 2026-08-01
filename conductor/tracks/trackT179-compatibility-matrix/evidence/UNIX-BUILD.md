# UNIX-BUILD — First Linux dry-run (T179 Phase B0)

**Date:** 2026-07-31  
**Host:** WSL2 Ubuntu (`Linux … microsoft-standard-WSL2`, x86_64)  
**Toolchain:** rustc 1.95.0 (override via `rust-toolchain.toml`)  
**Command cwd:** `/mnt/c/dev/AI-Brains`

## Attempts

| # | Command | Result |
|---|---------|--------|
| 1 | `cargo check --workspace` | **FAIL** — Tauri/GTK: missing `gio-2.0`, `gdk-3.0`, `glib-2.0`, `gobject-2.0` (desktop `ai-brains-desktop`) |
| 2 | `cargo check --workspace --exclude ai-brains-desktop` | **PASS** (exit 0) |
| 3 | `cargo clippy --workspace --all-targets --exclude ai-brains-desktop -- -D warnings` | **FAIL** initially — `clippy::collapsible_if` in `ai-brains-api-server/src/token.rs` Unix `refuse_hardlink` path |
| 4 | Same after collapse `if let … && nlink` fix | **FAIL** — `dead_code` / `needless_return` / unused imports on Unix (see Phase B1 below) |
| 5 | Same after Windows-only `cfg` gates + stub needless_return fixes | **PASS** (exit 0) |
| 6 | `cargo check --workspace --exclude ai-brains-desktop` | **PASS** (exit 0) |

## Phase B1 — clippy dead_code / needless_return (Unix)

**Root cause:** Windows-only helpers/constants were compiled on Unix without callers (named pipe probe, ACL/SDDL, UAC elevation), and Unix fail-closed stubs used `return Err(...)` as the sole expression in blocks (`clippy::needless_return`).

**Fixes (prefer `#[cfg(windows)]` / `#[cfg(any(windows, test))]`; keep fail-closed stubs; no fake success):**

| Crate / file | Change |
|--------------|--------|
| `ai-brainsd/src/main.rs` | `DaemonRequest` import + `PIPE_NAME` gated `#[cfg(windows)]` (`DaemonResponse` stays shared for `handle_client`) |
| `ai-brains-cli/src/artifact_security.rs` | `RESTRICTIVE_FILE_SDDL`, pure ACL parsers, `verify_restrictive_acl` → `#[cfg(any(windows, test))]`; `ensure_parent_protected` / `apply_restrictive_acl` → `#[cfg(windows)]`; Unix stubs drop bare `return` |
| `ai-brains-cli/src/elevation.rs` | `ELEVATE_ENV_KEYS` / `write_elevate_env_handoff` → `#[cfg(windows)]`; `quote_windows_arg` / `build_relaunch_params` → `#[cfg(any(windows, test))]`; `Relaunched` kept for exhaustive matches with `#[cfg_attr(not(windows), allow(dead_code))]` |
| `ai-brains-cli/src/commands/device.rs` | `zeroize::Zeroizing` import gated `#[cfg(windows)]` (DPAPI private-key path only) |

**Windows re-check:** `cargo clippy -p ai-brains-cli -p ai-brainsd -p ai-brains-api-server --all-targets -- -D warnings` → **PASS** (exit 0).

## Disposition

1. **Desktop on Linux is T2** (F13). Required Linux CI **excludes** `ai-brains-desktop` rather than installing WebKitGTK on every PR (cost + honesty: no false T1 desktop claim).
2. **Core CLI/daemon/store/sync** compile on Linux without Windows API leakage once desktop is excluded.
3. **Clippy hygiene** for Unix-only hardlink check + Windows-only dead_code / needless_return is in-tree (T179 Phase B / B1).
4. Full desktop Linux residual: install system packages (e.g. `libwebkit2gtk-4.1-dev`, `libgtk-3-dev`, related pkg-config) — **not** T179 T1 DoD.

## CI mapping

See `.github/workflows/ci.yml` job `gate-linux`:

- `cargo check|clippy|nextest --workspace … --exclude ai-brains-desktop`
- then `cargo deny check` + `cargo audit` (exit code only)

## Capture independence (spot)

`cargo tree -p ai-brains-capture` on Linux should not pull `ai-brains-sync` (same invariant as Windows). Recorded in smoke template.

## Phase B2 — nextest (Linux residual → green)

**Command:** `cargo nextest run --workspace --exclude ai-brains-desktop --no-fail-fast`  
**Host:** WSL2 Ubuntu, cwd `/mnt/c/dev/AI-Brains`  
**Baseline:** 24 failed / 1565 passed (Windows full suite was green).

### Root causes (by class)

| Class | Root cause | Fix |
|-------|------------|-----|
| Windows ACL / protected artifact unit tests | Unix fail-closed stubs return `"only supported on Windows"`; tests expected icacls/reparse/hardlink messages | `#[cfg(windows)]` on ACL/write-path tests; Unix fail-closed unit tests added |
| Daemon/nightly `.bat` wrapper generators | `Path::parent` treats `\` as non-separator on Unix → missing `cd /d` lines | Host-independent `windows_path_parent` (normalize `/`→`\`, drive root `C:\`) |
| `ledgerful_dir_discovery` | Incomplete prior edit (missing `}` on `nested_start`); `join(".")` unreliable on some Unix paths | Fix `nested_start` → `create_dir_all(&child)` |
| Obsidian / vault_fs reparse refuse | `path_is_same_or_inside` follows symlinks via `resolve_best_effort` → `PathEscape` before reparse check | Lexical containment in `resolve_under_root` (no follow); reparse refuse still hard-fail |
| Smoke: schema / graph stub / daemon status | `AppContext::from_cli` required vault for all commands; clean Linux has no ambient `AI_BRAINS_VAULT_PATH` | Vault-path-free early exit for schema printers + non-graph stub; smoke tests supply temp vault; `HOME`+`USERPROFILE` for global `.env` |

### Files touched

- `crates/ai-brains-cli/src/artifact_security.rs`
- `crates/ai-brains-cli/src/commands/daemon.rs`
- `crates/ai-brains-cli/src/commands/nightly.rs`
- `crates/ai-brains-cli/src/main.rs`
- `crates/ai-brains-cli/tests/smoke.rs`
- `crates/ai-brains-cli/tests/cli_capture_smoke.rs` (stdin EOF + log isolation; was intermittent under full suite I/O load)
- `crates/ai-brains-path/tests/ledgerful_dir_discovery.rs`
- `crates/ai-brains-sources/src/vault_fs.rs`

### Security notes

- Symlink/reparse refuse remains fail-closed on Linux (now correctly returns `ReparseRefused` instead of accidental `PathEscape`).
- Protected artifact write/ACL APIs remain Windows-only fail-closed stubs on Unix (no silent success).

### Residual test notes

- **Desktop** still excluded (T2 / F13).
- **`cli_capture_smoke`:** one intermittent fail under full-suite load on `/mnt/c` (empty/non-JSON stdout); passes alone and after stdin-EOF + `--log-format off` harden. Severity: low/medium flake, not a product regression.
- Prefer native Linux FS (ext4) over `/mnt/c` for full nextest when measuring CI latency.

### Verification (post-fix)

| Host | Command | Result |
|------|---------|--------|
| WSL Linux | `cargo nextest run --workspace --exclude ai-brains-desktop --no-fail-fast` | **1587 passed**, 0 failed, 1 skipped (~316s) |
| Windows | `cargo nextest run -p ai-brains-cli -p ai-brains-path -p ai-brains-sources --tests` | **558 passed**, 0 failed |
