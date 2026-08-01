# CI Tooling — Installation & Version Pins (T71 + T179)

This document records the supported installation paths and version pins for the tools required by the AI-Brains CI gate. Updated whenever a tool is intentionally upgraded.

**Platform matrix:** see **[COMPATIBILITY.md](COMPATIBILITY.md)** for support tiers, runner labels, transport honesty, and vault encryption wording.

## Required Tools

| Tool | Minimum Version | Install Command |
|------|----------------|-----------------|
| `cargo-nextest` | 0.9.140 | `cargo install cargo-nextest --locked` |
| `cargo-deny` | 0.20.2 | `cargo install cargo-deny --locked` |
| `cargo-audit` | 0.22.2 | `cargo install cargo-audit --locked` |

All three tools install to `~/.cargo/bin/` via standard `cargo install`. No project-local binaries or generated caches are used.

## Full CI Gate

### Windows (PowerShell)

```powershell
cargo fmt --check ; cargo clippy --workspace --all-targets -- -D warnings ; cargo nextest run --workspace ; cargo deny check ; cargo audit
```

Or use the verification script, which checks tool presence and versions before running the gate:

```powershell
.\scripts\dev-check.ps1
```

Pass `--check-only` to verify tool presence without running the full gate:

```powershell
.\scripts\dev-check.ps1 --check-only
```

### Linux / macOS (POSIX)

Desktop (Tauri / `ai-brains-desktop`) is **T2** on Linux/macOS and needs WebKitGTK / WKWebView system packages. The required core gate **excludes** desktop:

```bash
cargo fmt --check
cargo clippy --workspace --all-targets --exclude ai-brains-desktop -- -D warnings
cargo nextest run --workspace --exclude ai-brains-desktop
cargo deny check
cargo audit   # gate on exit code only (F27)
```

Or the POSIX mirror (T179 / F30), which applies the same exclude on Linux/Darwin:

```bash
./scripts/dev-check.sh
./scripts/dev-check.sh --check-only
```

`scripts/dev-check.sh` uses the **same minimum versions** as `dev-check.ps1` (nextest 0.9.140, deny 0.20.2, audit 0.22.2), runs with `set -euo pipefail`, and exits non-zero on any step failure.

## GitHub Actions matrix pins (T179)

Workflow: [`.github/workflows/ci.yml`](../.github/workflows/ci.yml)

| Job | Runner label | Required? | Steps |
|-----|--------------|-----------|-------|
| `gate-windows` | **`windows-2025`** | Yes | fmt, clippy workspace all-targets, nextest workspace (includes desktop) |
| `gate-linux` | **`ubuntu-24.04`** | Yes | toolchain 1.95.0 → check/clippy/nextest **`--exclude ai-brains-desktop`** → deny → **audit (exit code)** + capture-independence tree spot |
| `gate-macos` | **`macos-15`** | Soft (`continue-on-error`) | cargo check + nextest **`--exclude ai-brains-desktop`** |

- Toolchain channel **1.95.0** (matches `rust-toolchain.toml`).
- Installer: `dtolnay/rust-toolchain@v1` on PR (floating major OK). Release jobs should **SHA-pin** actions (F26).
- **Do not** use archived `actions-rs/*`.
- Prefer explicit labels over floating `-latest` for release evidence.
- Soft macOS pin is **`macos-15`** — never claim macOS 15 support from `macos-latest` if it resolves to 26 (F25).
- Optional `Swatinem/rust-cache` is soft (may `continue-on-error`).
- No AGPL CI tools.
- Windows job shell: PowerShell with `;` not `&&`. Linux/macOS: `set -euo pipefail`.

Local `dev-check.ps1` / `dev-check.sh` remain the developer full gate; GHA mirrors the gate with OS-specific tooling friction in mind (deny/audit preferred on Linux job).

## Windows App Control Notes

- `cargo-deny` and `cargo-nextest` must be installed via `cargo install` (MSVC or GNU toolchain). Pre-built binaries from third-party sources may be blocked by Windows Application Control.
- If `cargo-deny` is blocked (OS error 4551), uninstall it and reinstall via `cargo install cargo-deny --locked`.
- No special execution policy changes are required for `scripts/dev-check.ps1` if run within the project shell.

## Upgrading a Tool

1. Run `cargo install <tool> --locked` with the new version.
2. Verify the full gate still passes: `.\scripts\dev-check.ps1` (Windows) or `./scripts/dev-check.sh` (POSIX).
3. Update the version pin table above, in `scripts/dev-check.ps1` (`$Required` hash), in `scripts/dev-check.sh`, and in `.github/workflows/ci.yml` install steps.

## Behavior Notes

### `cargo audit` exits 0 with no final summary on a clean run (F27)

`cargo-audit` 0.22.x changed its CLI output — a clean run now exits 0 but
emits **no final summary line**. The visible output ends with
`Scanning Cargo.lock for vulnerabilities (N crate dependencies)`. To a casual
reader, that looks like a hang that exited 0.

**CI and scripts MUST gate on exit code only.** Never `grep` for a success summary line.

How to interpret:

- Exit 0 + tail `Scanning …` → success, no vulnerabilities found.
- Exit 0 + any text after `Scanning …` (a `warning` or `error:` block) →
  success with informational warnings.
- Exit non-zero → real failure; the message before exit code is the cause.

To get an explicit confirmation in scripts or CI logs, use the JSON output:

```powershell
cargo audit --json
# => {"database":{...},"lockfile":{"dependency-count":N},"vulnerabilities":{"found":false,"count":0,"list":[]},"warnings":{}}
```

The JSON envelope's `vulnerabilities.count` is the authoritative answer.

This quirk is what made the early T71 verification confusing. The
`scripts\dev-check.ps1` and `scripts/dev-check.sh` scripts treat exit 0 as success, which is correct —
just be aware that the human-readable form gives no positive confirmation.
