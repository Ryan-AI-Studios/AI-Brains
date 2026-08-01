# T185 Phase B — Tooling spike notes

**Date:** 2026-08-01  
**Product version:** workspace `0.1.1`  
**Host:** Windows (`x86_64-pc-windows-msvc`)  
**Repo visibility:** public GitHub (Artifact Attestations allowed)

## Pins (release-gate tools only — not product deps)

| Tool | Version | License | Install |
|------|---------|---------|---------|
| `cargo-cyclonedx` | **0.5.9** | **Apache-2.0** (not MIT) | `cargo install --locked cargo-cyclonedx` |
| `cargo-about` | **0.9.1** | MIT OR Apache-2.0 | `cargo install --locked --features cli cargo-about` |
| Fallback: `cargo-sbom` | **0.10.0** | MIT | `cargo install --locked cargo-sbom` (only if cyclonedx fails) |

**No AGPL/GPL tools** in the release gate (L7).

## B1–B9 results

### B1 — Install cargo-cyclonedx 0.5.9

```text
cargo install --locked cargo-cyclonedx
→ cargo-cyclonedx-cyclonedx 0.5.9
```

### B2 — Spike command (binaries + target)

```powershell
cargo cyclonedx --format json --spec-version 1.5 --describe binaries --target x86_64-pc-windows-msvc --target-in-filename
```

**Do not** pass blind `--all-features` (over-includes optional graph/desktop edges).

Observed crate-local outputs (next to crates):

| Binary | Path pattern |
|--------|----------------|
| `ai-brains` (CLI) | `crates/ai-brains-cli/ai-brains_bin_x86_64-pc-windows-msvc.cdx.json` |
| `ai-brainsd` | `crates/ai-brainsd/ai-brainsd_bin_x86_64-pc-windows-msvc.cdx.json` |
| desktop (optional) | under `apps/desktop/…` when present |
| check_db / other | may emit; **shipped** set is **ai-brains + ai-brainsd** by default |

Release layout after script copy:

```text
dist/sbom/ai-brains-0.1.1.cdx.json
dist/sbom/ai-brainsd-0.1.1.cdx.json
```

### B3 — specVersion

JSON carries `"specVersion": "1.5"`. Tool max is **1.3–1.5**; CycloneDX 1.6/1.7 / ECMA-424 lag is documented — **do not claim 1.6/1.7**.

### B4 — Fallback

Primary path works with cargo-cyclonedx 0.5.9. Fallback if needed:

| Tool | Version | License | Note |
|------|---------|---------|------|
| `cargo-sbom` | 0.10.0 | MIT | Alternate SPDX/CycloneDX generator; record command in ci-tooling if promoted |

### B5 — Install cargo-about

```text
cargo install --locked --features cli cargo-about
→ cargo-about 0.9.1
```

**CLI feature is required** (AI2 F-3). Without `--features cli`, generate may be unavailable.

### B6 — about config

Committed at repo root:

- `about.toml` — simple SPDX ids aligned with `deny.toml` (cargo-about rejects compound `MIT OR Apache-2.0` strings in `accepted`; dual-licensed crates match if any component is listed)
- `about.md.hbs` — markdown template → `dist/THIRD-PARTY.md`
- Release NOTICE graph: `ignore-dev-dependencies = true`, `ignore-build-dependencies = true`, `private = { ignore = true }`
- Extra accepted: `BSD-2-Clause`, `0BSD`, `CC0-1.0` (rare transitive / public-domain)

Decision: ship **markdown** `THIRD-PARTY.md` (not HTML).

Generate (PowerShell cannot redirect stdout reliably for this tool — use `-o`):

```powershell
cargo about generate about.md.hbs -o dist/THIRD-PARTY.md
# via script:
.\scripts\generate-notices.ps1
```

Dry-run: `dist/THIRD-PARTY.md` ~361 KB; overview includes CDLA-Permissive-2.0 (1 crate).

### B7 — CDLA note

- `deny.toml` allows **CDLA-Permissive-2.0**.
- `about.toml` `accepted` list includes **CDLA-Permissive-2.0**.
- Generate succeeded with CDLA present in overview (no unknown-license failure).

### B8 — License + ci-tooling

Neither tool is AGPL. Pins recorded in `Docs/ci-tooling.md` (release tools section).

### B9 — SOURCE_DATE_EPOCH

```powershell
$env:SOURCE_DATE_EPOCH = git log -1 --format=%ct
# e.g. 1785601844
```

`scripts/generate-sbom.ps1` sets this soft if unset, for more reproducible timestamps/serials.

## Shipped binary policy

| Binary | Package | Default ship BOM |
|--------|---------|------------------|
| `ai-brains` | `ai-brains-cli` | **Yes** |
| `ai-brainsd` | `ai-brainsd` | **Yes** |
| desktop | `ai-brains-desktop` / Tauri | **No** unless `-IncludeDesktop` |
| check_db / helpers | various | **No** |

Crate-local `*.cdx.json` files are cleaned after copy to avoid dirty trees (also gitignored).

## Attestations (R-SLSA soft)

- Public repo → GitHub Artifact Attestations allowed.
- Prefer SHA-pinned `actions/attest` for greenfield `release.yml`.
- Honest language only: Build L1-oriented; **no** SLSA L3 / certified claims.
