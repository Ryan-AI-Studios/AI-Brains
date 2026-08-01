# T185 — Soft unified release-gate wrapper (AI1 Opp1)
#
# Runs (by default):
#   1) scripts/dev-check.ps1 (full CI gate)  — skip with -SkipGate
#   2) generate-sbom.ps1
#   3) generate-notices.ps1
#   4) check-release-claims.ps1
#   5) check-version-banners.ps1 (soft)
#   6) generate-checksums.ps1 (if dist artifacts exist)
#
# Usage:
#   .\scripts\dev-release-check.ps1
#   .\scripts\dev-release-check.ps1 -SkipGate          # artifacts + claims only
#   .\scripts\dev-release-check.ps1 -CheckOnly         # tool preflight only (via dev-check)
#   .\scripts\dev-release-check.ps1 -StrictVersions    # hard-fail version banners
#
# Not a substitute for Docs/RELEASE-CHECKLIST.md human sign-off (L11).
# PowerShell: use ; not &&. See Docs/ci-tooling.md for tool pins.

param(
    [switch]$SkipGate,
    [switch]$CheckOnly,
    [switch]$StrictVersions,
    [string]$Target = "x86_64-pc-windows-msvc"
)

$ErrorActionPreference = "Stop"
$root = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
Set-Location $root

function Run-Step([string]$Label, [scriptblock]$Block) {
    Write-Host ""
    Write-Host "=== $Label ===" -ForegroundColor Cyan
    & $Block
    if ($null -ne $LASTEXITCODE -and $LASTEXITCODE -ne 0) {
        Write-Host "$Label FAILED (exit $LASTEXITCODE)" -ForegroundColor Red
        exit $LASTEXITCODE
    }
}

Write-Host "AI-Brains dev-release-check (T185)" -ForegroundColor Cyan
Write-Host "  Root: $root"
Write-Host "  Target: $Target"

if ($CheckOnly) {
    & (Join-Path $root "scripts\dev-check.ps1") -CheckOnly
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
    # Soft tool checks for release extras
    $extra = @("cargo-cyclonedx", "cargo-about")
    foreach ($t in $extra) {
        $v = $null
        try {
            if ($t -eq "cargo-cyclonedx") {
                $raw = cargo cyclonedx --version 2>&1 | Select-Object -First 1
            } else {
                $raw = cargo about --version 2>&1 | Select-Object -First 1
            }
            if ("$raw" -match '(\d+\.\d+\.\d+)') { $v = $Matches[1] }
        } catch {}
        if ($v) {
            Write-Host "  [OK] $t $v" -ForegroundColor Green
        } else {
            Write-Host "  [MISSING] $t" -ForegroundColor Yellow
        }
    }
    exit 0
}

if (-not $SkipGate) {
    Run-Step "dev-check.ps1 (full gate)" {
        & (Join-Path $root "scripts\dev-check.ps1")
    }
} else {
    Write-Host "Skipping full gate (-SkipGate)" -ForegroundColor Yellow
}

Run-Step "generate-sbom.ps1" {
    & (Join-Path $root "scripts\generate-sbom.ps1") -Target $Target
}

Run-Step "generate-notices.ps1" {
    & (Join-Path $root "scripts\generate-notices.ps1")
}

Run-Step "check-release-claims.ps1" {
    & (Join-Path $root "scripts\check-release-claims.ps1") -Path $root
}

Run-Step "check-version-banners.ps1" {
    if ($StrictVersions) {
        & (Join-Path $root "scripts\check-version-banners.ps1") -Path $root -Strict
    } else {
        & (Join-Path $root "scripts\check-version-banners.ps1") -Path $root
    }
}

Run-Step "generate-checksums.ps1" {
    & (Join-Path $root "scripts\generate-checksums.ps1") -RepoRoot $root
}

Write-Host ""
Write-Host "[SUCCESS] dev-release-check completed" -ForegroundColor Green
Write-Host "Remember: human sign-off on Docs/RELEASE-CHECKLIST.md is still required (L11)."
exit 0
