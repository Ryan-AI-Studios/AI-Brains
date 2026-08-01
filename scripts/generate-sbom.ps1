# T185 — Generate CycloneDX SBOMs for shipped binaries
#
# Feature set / target (document for release honesty):
#   - Default target: host Windows MSVC (x86_64-pc-windows-msvc)
#   - Default features: package defaults (NOT blind --all-features)
#   - Spec: CycloneDX JSON 1.5 via cargo-cyclonedx --describe binaries
#   - Shipped by default: ai-brains (CLI) + ai-brainsd
#   - Optional: desktop BOM via -IncludeDesktop
#
# Usage:
#   .\scripts\generate-sbom.ps1
#   .\scripts\generate-sbom.ps1 -Target x86_64-pc-windows-msvc -Version 0.1.1
#   .\scripts\generate-sbom.ps1 -IncludeDesktop
#
# Requires: cargo-cyclonedx 0.5.9+ (Apache-2.0). See Docs/ci-tooling.md.

param(
    [string]$Target = "x86_64-pc-windows-msvc",
    [string]$Version = "",
    [switch]$IncludeDesktop,
    [string]$RepoRoot = ""
)

$ErrorActionPreference = "Stop"

if (-not $RepoRoot) {
    $RepoRoot = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
}
Set-Location $RepoRoot

# Soft SOURCE_DATE_EPOCH from git for more reproducible SBOM timestamps
if (-not $env:SOURCE_DATE_EPOCH) {
    try {
        $epoch = (git log -1 --format=%ct 2>$null)
        if ($epoch -and $epoch -match '^\d+$') {
            $env:SOURCE_DATE_EPOCH = $epoch
            Write-Host "SOURCE_DATE_EPOCH set from git: $epoch"
        }
    } catch {
        Write-Host "SOURCE_DATE_EPOCH not set (git unavailable)"
    }
}

# Resolve product version from workspace Cargo.toml if not provided
if (-not $Version) {
    $cargoToml = Get-Content (Join-Path $RepoRoot "Cargo.toml") -Raw
    if ($cargoToml -match '(?ms)\[workspace\.package\].*?^version\s*=\s*"([^"]+)"') {
        $Version = $Matches[1]
    } else {
        Write-Error "Could not parse [workspace.package] version from Cargo.toml; pass -Version"
        exit 1
    }
}

Write-Host "=== generate-sbom.ps1 ===" -ForegroundColor Cyan
Write-Host "  Version: $Version"
Write-Host "  Target:  $Target"
Write-Host "  Features: package defaults (not --all-features)"
Write-Host "  Spec:    CycloneDX 1.5 / --describe binaries"

# Verify tool
$toolVer = $null
try {
    $raw = cargo cyclonedx --version 2>&1 | Select-Object -First 1
    if ($raw -match '(\d+\.\d+\.\d+)') { $toolVer = $Matches[1] }
} catch {}
if (-not $toolVer) {
    Write-Error "cargo-cyclonedx not found. Install: cargo install --locked cargo-cyclonedx"
    exit 1
}
Write-Host "  cargo-cyclonedx: $toolVer"

# Generate next to crates (tool default layout)
cargo cyclonedx --format json --spec-version 1.5 --describe binaries --target $Target --target-in-filename
if ($LASTEXITCODE -ne 0) {
    Write-Error "cargo cyclonedx failed with exit $LASTEXITCODE"
    exit $LASTEXITCODE
}

$distSbom = Join-Path $RepoRoot "dist\sbom"
New-Item -ItemType Directory -Force -Path $distSbom | Out-Null

# Pattern: {bin}_bin_{target}.cdx.json
function Find-BinBom {
    param([string]$BinName, [string]$Tgt)
    $pattern = "${BinName}_bin_${Tgt}.cdx.json"
    $found = Get-ChildItem -Path $RepoRoot -Recurse -Filter $pattern -ErrorAction SilentlyContinue |
        Where-Object { $_.FullName -notmatch '[\\/]target[\\/]' -and $_.FullName -notmatch '[\\/]dist[\\/]' } |
        Select-Object -First 1
    return $found
}

function Copy-And-Validate {
    param(
        [string]$BinName,
        [string]$OutName
    )
    $src = Find-BinBom -BinName $BinName -Tgt $Target
    if (-not $src) {
        Write-Error "Missing SBOM for shipped binary '$BinName' (expected *${BinName}_bin_${Target}.cdx.json)"
        exit 1
    }
    $dest = Join-Path $distSbom $OutName
    Copy-Item -Force $src.FullName $dest
    $json = Get-Content $dest -Raw | ConvertFrom-Json
    if ($json.specVersion -ne "1.5") {
        Write-Error "SBOM $OutName has specVersion='$($json.specVersion)' (expected 1.5)"
        exit 1
    }
    Write-Host "  [OK] $OutName (specVersion 1.5) from $($src.FullName)" -ForegroundColor Green
    return $src.FullName
}

$copiedSources = @()
$copiedSources += Copy-And-Validate -BinName "ai-brains" -OutName "ai-brains-$Version.cdx.json"
$copiedSources += Copy-And-Validate -BinName "ai-brainsd" -OutName "ai-brainsd-$Version.cdx.json"

if ($IncludeDesktop) {
    # Desktop BOM only under apps/ — never fall back to CLI ai-brains binary name
    $deskFiles = Get-ChildItem -Path (Join-Path $RepoRoot "apps") -Recurse -Filter "*_bin_${Target}.cdx.json" -ErrorAction SilentlyContinue |
        Where-Object { $_.FullName -notmatch '[\\/]target[\\/]' }
    if (-not $deskFiles) {
        $deskFiles = Get-ChildItem -Path (Join-Path $RepoRoot "apps") -Recurse -Filter "ai-brains-desktop_bin_${Target}.cdx.json" -ErrorAction SilentlyContinue
    }
    if ($deskFiles) {
        $src = $deskFiles | Select-Object -First 1
        $dest = Join-Path $distSbom "ai-brains-desktop-$Version.cdx.json"
        Copy-Item -Force $src.FullName $dest
        $json = Get-Content $dest -Raw | ConvertFrom-Json
        if ($json.specVersion -ne "1.5") {
            Write-Error "Desktop SBOM has specVersion='$($json.specVersion)' (expected 1.5)"
            exit 1
        }
        Write-Host "  [OK] ai-brains-desktop-$Version.cdx.json" -ForegroundColor Green
        $copiedSources += $src.FullName
    } else {
        Write-Error "IncludeDesktop set but no desktop *_bin_${Target}.cdx.json found under apps/"
        exit 1
    }
}

# Clean only cargo-cyclonedx crate-local generator outputs under crates/ and apps/
# (never conductor/, evidence/, dist/, target/). Includes bin + other target BOMs
# the tool may emit (e.g. staticlib/cdylib names).
$searchRoots = @(
    (Join-Path $RepoRoot "crates"),
    (Join-Path $RepoRoot "apps")
) | Where-Object { Test-Path $_ }
$genCdx = foreach ($root in $searchRoots) {
    Get-ChildItem -Path $root -Recurse -Filter "*.cdx.json" -ErrorAction SilentlyContinue
}
foreach ($f in $genCdx) {
    Remove-Item -Force $f.FullName
    Write-Host "  cleaned $($f.FullName)"
}

Write-Host "[SUCCESS] SBOMs written under dist/sbom/" -ForegroundColor Green
exit 0
