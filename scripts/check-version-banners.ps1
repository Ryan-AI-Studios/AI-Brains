# T185 — Soft version-banner consistency (Cargo.toml vs CHANGELOG)
#
# Reads workspace package version from Cargo.toml [workspace.package].
# If CHANGELOG latest section is [Unreleased], soft-warns when no
# ## [<version>] section exists for the cargo version (AI2 F-8 / L13 soft).
#
# Usage:
#   .\scripts\check-version-banners.ps1
#   .\scripts\check-version-banners.ps1 -Strict   # hard fail on missing versioned section
#
# Default: soft warn, exit 0. -Strict: exit 1 on hard problems.

param(
    [switch]$Strict,
    [string]$Path = ""
)

$ErrorActionPreference = "Stop"

if (-not $Path) {
    $Path = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
}
$root = (Resolve-Path $Path).Path

Write-Host "=== check-version-banners.ps1 ===" -ForegroundColor Cyan

$cargoToml = Get-Content (Join-Path $root "Cargo.toml") -Raw
if ($cargoToml -notmatch '(?ms)\[workspace\.package\].*?^version\s*=\s*"([^"]+)"') {
    Write-Error "Could not parse [workspace.package] version from Cargo.toml"
    exit 1
}
$version = $Matches[1]
Write-Host "  Cargo workspace version: $version"

$changelogPath = Join-Path $root "CHANGELOG.md"
if (-not (Test-Path $changelogPath)) {
    $msg = "CHANGELOG.md missing"
    if ($Strict) {
        Write-Error $msg
        exit 1
    }
    Write-Host "  [WARN] $msg" -ForegroundColor Yellow
    exit 0
}

$cl = Get-Content $changelogPath -Raw
$hasUnreleased = $cl -match '(?m)^##\s*\[Unreleased\]'
$hasVersioned = $cl -match "(?m)^##\s*\[$([regex]::Escape($version))\]"

Write-Host "  CHANGELOG [Unreleased]: $hasUnreleased"
Write-Host "  CHANGELOG ## [$version]: $hasVersioned"

$warnings = @()
if ($hasUnreleased -and -not $hasVersioned) {
    $warnings += "CHANGELOG has [Unreleased] but no ## [$version] section for workspace version (expected at public tag time)"
}
if (-not $hasUnreleased -and -not $hasVersioned) {
    $warnings += "CHANGELOG has neither [Unreleased] nor ## [$version] for workspace version"
}

# Soft consistency: README product version mention is optional (do not hard fail)
$readme = Join-Path $root "README.md"
if (Test-Path $readme) {
    $rm = Get-Content $readme -Raw
    if ($rm -match '(?i)version\s*[:=]?\s*0\.\d+\.\d+' -or $rm -match '(?i)\b0\.\d+\.\d+\b') {
        # informational only
        Write-Host "  README mentions a 0.x version string (manual check at release)"
    }
}

if ($warnings.Count -eq 0) {
    Write-Host "[OK] Version banners look consistent (or Unreleased with versioned section present)" -ForegroundColor Green
    exit 0
}

foreach ($w in $warnings) {
    Write-Host "  [WARN] $w" -ForegroundColor Yellow
}

if ($Strict) {
    Write-Host "[FAIL] Strict mode: version-banner warnings treated as errors" -ForegroundColor Red
    exit 1
}

Write-Host "[OK soft] Exiting 0 with warnings (use -Strict for hard fail)" -ForegroundColor Green
exit 0
