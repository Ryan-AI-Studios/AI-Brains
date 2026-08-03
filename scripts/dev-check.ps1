# AI-Brains CI Gate Verification Script (T71)
# Checks tool presence and versions, then runs the full CI gate.
# Usage: .\scripts\dev-check.ps1 [--check-only]
#   --check-only  Only verify tool presence; skip running the gate.

param([switch]$CheckOnly)

$ErrorActionPreference = "Stop"

# ---------------------------------------------------------------------------
# Required tool versions (update when intentionally upgrading)
# ---------------------------------------------------------------------------
$Required = @{
    "cargo-nextest" = @{ MinVersion = "0.9.140"; InstallCmd = "cargo install cargo-nextest --locked" }
    "cargo-deny"    = @{ MinVersion = "0.20.2";  InstallCmd = "cargo install cargo-deny --locked" }
    "cargo-audit"   = @{ MinVersion = "0.22.2";  InstallCmd = "cargo install cargo-audit --locked" }
}

function Get-ToolVersion([string]$Name) {
    try {
        $raw = & $Name --version 2>&1 | Select-Object -First 1
        if ($raw -match "(\d+\.\d+\.\d+)") { return $Matches[1] }
    } catch {}
    return $null
}

function Compare-Versions([string]$Installed, [string]$Min) {
    $i = [System.Version]$Installed
    $m = [System.Version]$Min
    return $i -ge $m
}

# ---------------------------------------------------------------------------
# Preflight: verify tools are present and meet minimum versions
# ---------------------------------------------------------------------------
Write-Host "=== AI-Brains CI Gate - Tool Preflight ===" -ForegroundColor Cyan
$allOk = $true

# T187: SQLCipher vendored OpenSSL needs Perl on Windows MSVC
$perl = Get-Command perl -ErrorAction SilentlyContinue
if (-not $perl) {
    Write-Host "  [MISSING] perl - required for bundled-sqlcipher-vendored-openssl (openssl-src). Install Strawberry Perl and add to PATH." -ForegroundColor Red
    $allOk = $false
} else {
    $perlVer = (& perl -v 2>&1 | Select-Object -First 2) -join " "
    Write-Host "  [OK] perl ($perlVer)" -ForegroundColor Green
}
$nasm = Get-Command nasm -ErrorAction SilentlyContinue
if (-not $nasm) {
    Write-Host "  [INFO] nasm not on PATH (optional; OpenSSL may build with no-asm)" -ForegroundColor DarkGray
} else {
    Write-Host "  [OK] nasm $((& nasm -v 2>&1 | Select-Object -First 1))" -ForegroundColor Green
}

foreach ($tool in $Required.Keys) {
    $info    = $Required[$tool]
    $version = Get-ToolVersion $tool
    if (-not $version) {
        Write-Host "  [MISSING] $tool - install with: $($info.InstallCmd)" -ForegroundColor Red
        $allOk = $false
    } elseif (-not (Compare-Versions $version $info.MinVersion)) {
        Write-Host "  [OUTDATED] $tool $version (need >= $($info.MinVersion)) - upgrade: $($info.InstallCmd)" -ForegroundColor Yellow
        $allOk = $false
    } else {
        Write-Host "  [OK] $tool $version" -ForegroundColor Green
    }
}

if (-not $allOk) {
    Write-Host "`n[FAIL] One or more tools are missing or outdated. Install them and re-run." -ForegroundColor Red
    exit 1
}

Write-Host ""

if ($CheckOnly) {
    Write-Host "[OK] All tools present. Skipping gate (--check-only)." -ForegroundColor Green
    exit 0
}

# ---------------------------------------------------------------------------
# Run the full CI gate
# ---------------------------------------------------------------------------
function Run-Step([string]$Label, [scriptblock]$Block) {
    Write-Host "--- $Label ---" -ForegroundColor Cyan
    & $Block
    if ($LASTEXITCODE -ne 0) {
        Write-Host "$Label FAILED" -ForegroundColor Red
        exit $LASTEXITCODE
    }
    Write-Host ""
}

# T187: tests that open historical all-zero vault keys need the escape hatch.
# Production CLI still refuses zero keys when this is unset.
if (-not $env:AI_BRAINS_ALLOW_ZERO_KEY) {
    $env:AI_BRAINS_ALLOW_ZERO_KEY = "1"
    Write-Host "  [env] AI_BRAINS_ALLOW_ZERO_KEY=1 (test hermetic default; unset for production refuse tests)" -ForegroundColor DarkGray
}

Run-Step "cargo fmt --check" { cargo fmt --check }
Run-Step "cargo clippy"      { cargo clippy --workspace --all-targets -- -D warnings }
Run-Step "cargo nextest"     { cargo nextest run --workspace }
# T200: default nextest is graph-off. Graph health smoke (CI F14 required):
#   cargo nextest run -p ai-brains-cli --features graph
# See CONTRIBUTING.md build matrix + Docs/INSTALL.md.
Run-Step "cargo deny check"  { cargo deny check }
Run-Step "cargo audit"       { cargo audit }

Write-Host "[SUCCESS] CI Gate passed!" -ForegroundColor Green
