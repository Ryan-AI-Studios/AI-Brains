# T185 — Generate third-party NOTICE / THIRD-PARTY.md via cargo-about
#
# Requires: cargo-about 0.9.1+ with --features cli
#   cargo install --locked --features cli cargo-about
# Config: about.toml + about.md.hbs (repo root)
#
# Usage:
#   .\scripts\generate-notices.ps1
#   .\scripts\generate-notices.ps1 -Output dist/THIRD-PARTY.md
#
# Note: Must use cargo about generate … -o path (PowerShell cannot reliably
# redirect stdout for this tool).

param(
    [string]$Output = "dist/THIRD-PARTY.md",
    [string]$RepoRoot = ""
)

$ErrorActionPreference = "Stop"

if (-not $RepoRoot) {
    $RepoRoot = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
}
Set-Location $RepoRoot

Write-Host "=== generate-notices.ps1 ===" -ForegroundColor Cyan

$toolVer = $null
try {
    $raw = cargo about --version 2>&1 | Select-Object -First 1
    if ("$raw" -match '(\d+\.\d+\.\d+)') { $toolVer = $Matches[1] }
} catch {}
if (-not $toolVer) {
    Write-Error "cargo-about not found. Install: cargo install --locked --features cli cargo-about"
    exit 1
}
Write-Host "  cargo-about: $toolVer"

$template = Join-Path $RepoRoot "about.md.hbs"
$config = Join-Path $RepoRoot "about.toml"
if (-not (Test-Path $template)) {
    Write-Error "Missing template: about.md.hbs"
    exit 1
}
if (-not (Test-Path $config)) {
    Write-Error "Missing config: about.toml"
    exit 1
}

$outPath = if ([System.IO.Path]::IsPathRooted($Output)) { $Output } else { Join-Path $RepoRoot $Output }
$outDir = Split-Path -Parent $outPath
if ($outDir) {
    New-Item -ItemType Directory -Force -Path $outDir | Out-Null
}

cargo about generate $template -o $outPath
if ($LASTEXITCODE -ne 0) {
    Write-Error "cargo about generate failed with exit $LASTEXITCODE"
    exit $LASTEXITCODE
}

if (-not (Test-Path $outPath)) {
    Write-Error "NOTICE output missing: $outPath"
    exit 1
}
$info = Get-Item $outPath
if ($info.Length -lt 32) {
    Write-Error "NOTICE output empty or too small: $outPath ($($info.Length) bytes)"
    exit 1
}

Write-Host "  [OK] $outPath ($($info.Length) bytes)" -ForegroundColor Green
Write-Host "[SUCCESS] THIRD-PARTY notices generated" -ForegroundColor Green
exit 0
