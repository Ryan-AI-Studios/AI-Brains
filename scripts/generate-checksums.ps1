# T185 — SHA-256 checksums for release artifacts under dist/
#
# Hashes: dist/sbom/*, dist/THIRD-PARTY.md, dist/*.exe (if present)
# Output: dist/checksums/SHA256SUMS
#
# Usage:
#   .\scripts\generate-checksums.ps1
#   .\scripts\generate-checksums.ps1 -RepoRoot C:\dev\AI-Brains

param(
    [string]$RepoRoot = ""
)

$ErrorActionPreference = "Stop"

if (-not $RepoRoot) {
    $RepoRoot = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
}
$root = (Resolve-Path $RepoRoot).Path
$dist = Join-Path $root "dist"

Write-Host "=== generate-checksums.ps1 ===" -ForegroundColor Cyan

if (-not (Test-Path $dist)) {
    Write-Error "dist/ not found. Run generate-sbom / generate-notices first."
    exit 1
}

$files = @()
$sbomDir = Join-Path $dist "sbom"
if (Test-Path $sbomDir) {
    $files += Get-ChildItem -Path $sbomDir -File -ErrorAction SilentlyContinue
}
$notice = Join-Path $dist "THIRD-PARTY.md"
if (Test-Path $notice) {
    $files += Get-Item $notice
}
$files += Get-ChildItem -Path $dist -Filter "*.exe" -File -ErrorAction SilentlyContinue
# Also common Linux/mac binary names if present at dist root
foreach ($name in @("ai-brains", "ai-brainsd")) {
    $p = Join-Path $dist $name
    if (Test-Path $p -PathType Leaf) { $files += Get-Item $p }
}

# Deduplicate by full path
$files = $files | Sort-Object FullName -Unique

if ($files.Count -eq 0) {
    Write-Error "No artifacts to hash under dist/ (expected sbom/*, THIRD-PARTY.md, and/or binaries)"
    exit 1
}

$outDir = Join-Path $dist "checksums"
New-Item -ItemType Directory -Force -Path $outDir | Out-Null
$outFile = Join-Path $outDir "SHA256SUMS"

$lines = @()
foreach ($f in $files) {
    $hash = (Get-FileHash -Algorithm SHA256 -Path $f.FullName).Hash.ToLowerInvariant()
    # Paths relative to dist/ for portable SUMS
    $rel = $f.FullName.Substring($dist.Length).TrimStart('\', '/')
    $rel = $rel -replace '\\', '/'
    $lines += "$hash  $rel"
    Write-Host "  $hash  $rel"
}

# Deterministic order; UTF-8 without BOM when possible
$lines = $lines | Sort-Object
$text = ($lines -join "`n") + "`n"
$utf8NoBom = New-Object System.Text.UTF8Encoding $false
[System.IO.File]::WriteAllText($outFile, $text, $utf8NoBom)

Write-Host "  [OK] $outFile ($($lines.Count) entries)" -ForegroundColor Green
Write-Host "[SUCCESS] Checksums written" -ForegroundColor Green
exit 0
