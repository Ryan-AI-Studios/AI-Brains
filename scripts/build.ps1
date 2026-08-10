$ErrorActionPreference = "Stop"
Set-Location "C:\dev\AI-Brains"

# T222: local PATH install builds CLI with --features graph (matches INSTALL primary SOOT).
# SOOT: cargo install --path crates/ai-brains-cli --locked --features graph
Write-Host "Building ai-brains (CLI --features graph)..." -ForegroundColor Cyan
cargo build --release -p ai-brains-cli --features graph

if ($LASTEXITCODE -eq 0) {
    $OutputBin = "C:\Users\RyanB\.cargo\bin\ai-brains.exe"
    Copy-Item "target\release\ai-brains.exe" $OutputBin -Force
    Write-Host "BUILD SUCCESS" -ForegroundColor Green

    # Verify
    $ver = & $OutputBin --version 2>$null
    Write-Host "Version: $ver"

    # T222 F7: fail-closed graph capability probe (never touch operator vault)
    # Primary: doctor --json graph_feature=available (stdout only; --log-format off).
    # Secondary: graph update fail-closed only on exit 2 AND FEATURE_UNAVAILABLE.
    # Unique owned probe dir: assert vault path absent before use; remove only this dir.
    $probeDir = Join-Path $env:TEMP ("ai-brains-graph-probe-" + [guid]::NewGuid().ToString("N"))
    New-Item -ItemType Directory -Path $probeDir -Force | Out-Null
    $probeVault = Join-Path $probeDir "missing.db"
    if (Test-Path $probeVault) {
        Write-Error "Probe vault path unexpectedly exists: $probeVault"
        Remove-Item $probeDir -Recurse -Force -ErrorAction SilentlyContinue
        exit 1
    }
    $prevVault = $env:AI_BRAINS_VAULT_PATH
    $env:AI_BRAINS_VAULT_PATH = $probeVault
    try {
        $docOut = & $OutputBin --log-format off doctor --json 2>$null | Out-String
        $gfOk = $false
        try {
            $report = $docOut | ConvertFrom-Json
            $gf = $report.checks | Where-Object { $_.name -eq 'graph_feature' } | Select-Object -First 1
            if ($gf -and $gf.message -eq 'available') { $gfOk = $true }
        } catch { }

        $probe = & $OutputBin --log-format off graph update 2>&1 | Out-String
        $featureOff = ($LASTEXITCODE -eq 2 -and $probe -match 'FEATURE_UNAVAILABLE')
        if ($featureOff -or -not $gfOk) {
            Write-Error "Installed binary is graph-off or graph_feature not available; expected --features graph build"
            exit 1
        }
        Write-Host "Graph feature probe: available" -ForegroundColor Green
    } finally {
        if ($null -eq $prevVault) { Remove-Item Env:AI_BRAINS_VAULT_PATH -ErrorAction SilentlyContinue }
        else { $env:AI_BRAINS_VAULT_PATH = $prevVault }
        if (Test-Path $probeDir) { Remove-Item $probeDir -Recurse -Force -ErrorAction SilentlyContinue }
    }
} else {
    Write-Host "BUILD FAILED" -ForegroundColor Red
    exit 1
}
