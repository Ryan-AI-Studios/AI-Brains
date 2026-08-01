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

Local default nextest profile keeps retries=1 and fail-fast (fast feedback). To mirror **GHA CI** (no-fail-fast + retries=3):

```powershell
cargo nextest run --workspace --profile ci
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

CI-mirror (profile.ci):

```bash
cargo nextest run --workspace --exclude ai-brains-desktop --profile ci
```

Or the POSIX mirror (T179 / F30), which applies the same exclude on Linux/Darwin:

```bash
./scripts/dev-check.sh
./scripts/dev-check.sh --check-only
```

`scripts/dev-check.sh` uses the **same minimum versions** as `dev-check.ps1` (nextest 0.9.140, deny 0.20.2, audit 0.22.2), runs with `set -euo pipefail`, and exits non-zero on any step failure.

## Nextest configuration (T186)

Config path (auto-discovered from workspace root): **[`.config/nextest.toml`](../.config/nextest.toml)**  
Root `nextest.toml` is **not** auto-discovered by cargo-nextest 0.9.x — do not reintroduce it.

| Profile | retries | fail-fast | slow-timeout |
|---------|---------|-----------|--------------|
| `default` | 1 | true (built-in) | period 30s, terminate-after 4 (**kill after 120s**) |
| `ci` | 3 | **false** | inherits default unless overridden |

- GHA jobs run `cargo nextest run … --profile ci` so OS-specific failures are not hidden by fail-fast.
- Tests are marked **slow** after 30s and **terminated** after 4 periods (120s). Mark multi-minute suites with the `__slow` suffix / nextest overrides if needed.
- Optional env escapes (not used in committed GHA): `NEXTEST_RETRIES`, `NEXTEST_FAIL_FAST`.
- Wall-clock budget: multi-OS GHA Win+Linux ≈ **15–20 min** (retries can stretch the tail).

Prove profile loads:

```powershell
cargo nextest show-config test-groups --profile ci
cargo nextest list --profile ci -p ai-brains-path
```

## Hermetic CLI integration tests (T186)

CLI integration tests must not depend on developer ambient `AI_BRAINS_*` for pass (L1).

| Rule | Detail |
|------|--------|
| Shared helper | `crates/ai-brains-cli/tests/common/mod.rs` — `hermetic_bin`, `hermetic_vault`, `hermetic_cmd` |
| Ambient strip | Prefer `env_remove` denylist over `env_clear` |
| Denylist | Elevation keys (`AI_BRAINS_VAULT_PATH`, `KEY`, `VAULT_KEY`, model/embed URLs, `PROJECT_ID`, `SESSION_ID`) + `AI_BRAINS_SCOPE` + `AI_BRAINS_PREFLIGHT_PRINCIPAL_ID` |
| Fixtures | `tempfile::tempdir()` only; never write vaults outside temp |
| Pollution proof | `tests/hermetic_smoke.rs` (AC2) |
| `env_clear` | Prefer never; if used, restore full OS allowlist (PATH, SystemRoot, …) |

Each `tests/*.rs` is a separate crate: `mod common;` (or `#[path]`) + `#[allow(dead_code)]` on shared helpers.

Soft-canonicalize (`resolve_best_effort`) is **not** openat/cap-std TOCTOU closure (#12 residual).

## GitHub Actions matrix pins (T179 + T186)

Workflow: [`.github/workflows/ci.yml`](../.github/workflows/ci.yml)

| Job | Runner label | Required? | Steps |
|-----|--------------|-----------|-------|
| `gate-windows` | **`windows-2025`** | Yes | fmt, clippy workspace all-targets, nextest **`--profile ci`** (includes desktop) |
| `gate-linux` | **`ubuntu-24.04`** | Yes | toolchain 1.95.0 → check/clippy/nextest **`--exclude ai-brains-desktop --profile ci`** → deny → **audit (exit code)** + capture-independence tree spot |
| `gate-macos` | **`macos-15`** | Soft (`continue-on-error`) | cargo check + nextest **`--exclude ai-brains-desktop --profile ci`** |

- Toolchain channel **1.95.0** (matches `rust-toolchain.toml`). Image-default Rust may be newer; pin wins.
- **R-CI-PIN (T186):** all third-party `uses:` in `ci.yml` are full 40-char SHA pins with version comments, aligned with `release.yml` (checkout v4, rust-toolchain v1, rust-cache v2).
- Dependabot `github-actions` remains enabled for pin bumps.
- **Do not** use archived `actions-rs/*`.
- Prefer explicit labels over floating `-latest` for release evidence.
- Soft macOS pin is **`macos-15`** — never claim macOS 15 support from `macos-latest` if it resolves to 26 (F25).
- Optional `Swatinem/rust-cache` is soft (may `continue-on-error`).
- No AGPL CI tools.
- Windows job shell: PowerShell with `;` not `&&`. Linux/macOS: `set -euo pipefail`.

Local `dev-check.ps1` / `dev-check.sh` remain the developer full gate (default nextest profile); GHA uses `--profile ci` for multi-OS honesty.

## Windows App Control Notes

- `cargo-deny` and `cargo-nextest` must be installed via `cargo install` (MSVC or GNU toolchain). Pre-built binaries from third-party sources may be blocked by Windows Application Control.
- If `cargo-deny` is blocked (OS error 4551), uninstall it and reinstall via `cargo install cargo-deny --locked`.
- No special execution policy changes are required for `scripts/dev-check.ps1` if run within the project shell.

## Release-gate tools (T185 / P12.7)

Dev/CI only — **not** workspace product dependencies. **No AGPL/GPL** tools.

| Tool | Pin | License | Install |
|------|-----|---------|---------|
| `cargo-cyclonedx` | **0.5.9** | **Apache-2.0** | `cargo install --locked cargo-cyclonedx` (pin: `--version 0.5.9`) |
| `cargo-about` | **0.9.1** | MIT OR Apache-2.0 | `cargo install --locked --features cli cargo-about` (pin: `--version 0.9.1`) |
| Fallback `cargo-sbom` | **0.10.0** | MIT | `cargo install --locked cargo-sbom` — only if cyclonedx path fails |

### SBOM (CycloneDX 1.5, per shipped binary)

```powershell
# Windows — package defaults, not --all-features; target matches release build
.\scripts\generate-sbom.ps1
# → dist/sbom/ai-brains-<ver>.cdx.json, ai-brainsd-<ver>.cdx.json (specVersion 1.5)
```

```bash
# Linux runners
./scripts/generate-sbom.sh
# TARGET=x86_64-unknown-linux-gnu VERSION=0.1.1 ./scripts/generate-sbom.sh
```

Underlying generator:

```text
cargo cyclonedx --format json --spec-version 1.5 --describe binaries --target <triple> --target-in-filename
```

Tool supports CycloneDX **1.3–1.5** only; do not claim 1.6/1.7 until the generator catches up. Soft `SOURCE_DATE_EPOCH` from `git log -1 --format=%ct` improves timestamp reproducibility.

### THIRD-PARTY notices

Committed config: repo-root `about.toml` + markdown template `about.md.hbs`.

```powershell
# Must use -o (PowerShell cannot reliably redirect cargo-about stdout)
.\scripts\generate-notices.ps1
# → dist/THIRD-PARTY.md
```

```bash
./scripts/generate-notices.sh
```

`deny.toml` allows **CDLA-Permissive-2.0**; `about.toml` accepted list includes the same SPDX id.

### Claims scan + soft wrappers

```powershell
.\scripts\check-release-claims.ps1
.\scripts\check-version-banners.ps1
.\scripts\generate-checksums.ps1
.\scripts\dev-release-check.ps1          # gate + SBOM + NOTICE + claims + checksums
.\scripts\dev-release-check.ps1 -SkipGate
```

Human order and sign-off: [RELEASE-CHECKLIST.md](RELEASE-CHECKLIST.md). Normative claims: [RELEASE-CLAIMS.md](RELEASE-CLAIMS.md). Soft release workflow: `.github/workflows/release.yml` (SHA-pinned `uses:`). PR `ci.yml` is also SHA-pinned (T186 / R-CI-PIN).

### R-SLSA language

Optional GitHub Artifact Attestations (public repo or Enterprise Cloud private) are **Build L1-oriented** only. Forbidden: SLSA Build L3, “SLSA certified,” tamper-proof supply chain.

## Upgrading a Tool

1. Run `cargo install <tool> --locked` with the new version.
2. Verify the full gate still passes: `.\scripts\dev-check.ps1` (Windows) or `./scripts/dev-check.sh` (POSIX).
3. Update the version pin table above, in `scripts/dev-check.ps1` (`$Required` hash), in `scripts/dev-check.sh`, and in `.github/workflows/ci.yml` install steps.
4. For release tools (`cargo-cyclonedx`, `cargo-about`), also update this section, `scripts/generate-*.ps1` headers, and `.github/workflows/release.yml` install pins.

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
