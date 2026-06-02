# Build-AIBrains.ps1 - Build AI-Brains Windows binary with all features
# Run from C:\dev\AI-Brains

$ErrorActionPreference = "Stop"
$RepoPath = "C:\\dev\\AI-Brains"
$OutputBin = "C:\\Users\\RyanB\\.cargo\\bin\\ai-brains.exe"
$DaemonBin = "C:\\Users\\RyanB\\.cargo\\bin\\ai-brainsd.exe"

Write-Host "Building AI-Brains Windows binary..." -ForegroundColor Cyan
Write-Host "Repo:    $RepoPath"
Write-Host "Output:  $OutputBin"
Write-Host "=" * 60

Set-Location $RepoPath

# Verify cargo is available
$cargo = Get-Command cargo -ErrorAction SilentlyContinue
if (-not $cargo) {
    Write-Error "cargo not found in PATH. Is Rust installed?"
    exit 1
}

Write-Host "Rust toolchain: $(cargo --version)"

# T84: Stop the daemon before replacing binaries to avoid "Access Denied" on Windows
Write-Host ""
Write-Host "Checking for running daemon..." -ForegroundColor Cyan
$daemonProc = Get-Process ai-brainsd -ErrorAction SilentlyContinue
$daemonWasRunning = $false
if ($daemonProc) {
    $daemonWasRunning = $true
    Write-Host "  Found ai-brainsd (PID $($daemonProc.Id)) — stopping gracefully..." -ForegroundColor Yellow
    # Attempt graceful shutdown via CLI; ignore errors if CLI is not yet on PATH
    $aiBrains = Get-Command ai-brains -ErrorAction SilentlyContinue
    if ($aiBrains) {
        ai-brains daemon stop 2>$null
        Start-Sleep -Milliseconds 800
    }
    # Force-kill if still running
    $stillRunning = Get-Process ai-brainsd -ErrorAction SilentlyContinue
    if ($stillRunning) {
        Write-Host "  Graceful shutdown timed out — force-killing..." -ForegroundColor Yellow
        taskkill /F /IM ai-brainsd.exe 2>$null
        Start-Sleep -Milliseconds 400
    }
    Write-Host "  Daemon stopped." -ForegroundColor Green
} else {
    Write-Host "  No running daemon found."
}

# Build release binary
Write-Host ""
Write-Host "Building release binaries..." -ForegroundColor Cyan
cargo build --release -p ai-brains-cli -p ai-brainsd
if ($LASTEXITCODE -ne 0) {
    Write-Error "Build failed with exit code $LASTEXITCODE"
    exit 1
}

# Copy ai-brains.exe to cargo bin directory
$builtBin = "$RepoPath\\target\\release\\ai-brains.exe"
if (Test-Path $builtBin) {
    Copy-Item $builtBin $OutputBin -Force
    Write-Host "Installed: $OutputBin" -ForegroundColor Green
} else {
    # Check if it's named ai-brains-new.exe
    $builtBin = "$RepoPath\\target\\release\\ai-brains-new.exe"
    if (Test-Path $builtBin) {
        Copy-Item $builtBin $OutputBin -Force
        Write-Host "Installed: $OutputBin (from ai-brains-new.exe)" -ForegroundColor Green
    } else {
        Write-Error "Build output not found at expected paths"
        exit 1
    }
}

# Copy ai-brainsd.exe to cargo bin directory
$builtDaemon = "$RepoPath\\target\\release\\ai-brainsd.exe"
if (Test-Path $builtDaemon) {
    Copy-Item $builtDaemon $DaemonBin -Force
    Write-Host "Installed: $DaemonBin" -ForegroundColor Green
}

# Verify the binary works
Write-Host ""
Write-Host "Verifying build..."
& $OutputBin --version 2>$null
if ($LASTEXITCODE -eq 0) {
    Write-Host "Binary responds to --version" -ForegroundColor Green
} else {
    Write-Warning "Binary may have issues"
}

# T84: Restart the daemon if it was running before the update
if ($daemonWasRunning) {
    Write-Host ""
    Write-Host "Restarting AI-Brains daemon..." -ForegroundColor Cyan
    & $OutputBin daemon start 2>$null
    if ($LASTEXITCODE -eq 0) {
        Write-Host "  Daemon restarted." -ForegroundColor Green
    } else {
        Write-Warning "  Daemon did not start automatically. Run: ai-brains daemon start"
    }
}

Write-Host ""
Write-Host "Done! The binary now supports:"
Write-Host "  --semantic flag for embedding search"
Write-Host "  --global flag for cross-project preflight"
Write-Host "  All current codebase features"
