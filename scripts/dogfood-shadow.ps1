#Requires -Version 5.1
<#
.SYNOPSIS
  Progressive shadow dogfood orchestrator (T170 / P9.4).

.DESCRIPTION
  Runs Stage A (evaluate governed), optional Stage B fixture vault rehearsal,
  and Stage C redacted shadow when -SourceVault is provided. Captures governed
  briefing + legacy preflight JSON via --vault-path only (D26). Never sets
  AI_BRAINS_VAULT_PATH to a shadow/migrated path. Never sets User-level env.
  Never runs Stage D (live enablement requires explicit user approval).

.PARAMETER WorkDir
  Non-live work directory for reports, shadow, migrate dest, and compare JSON.

.PARAMETER SourceVault
  Optional Stage C source vault (prefer operator test vault). When omitted,
  Stage B creates a fixture vault under WorkDir.

.PARAMETER ProjectId
  Optional project id for Stage C `briefing project --project-id`. Required for
  multi-project vaults (see SHADOW-DOGFOOD-GATE.md Stage C).

.PARAMETER SkipMigrate
  Skip optional migrate governed step.

.PARAMETER EvaluateFixtures
  Path to T169 scenario fixtures directory (default: fixtures/governed-memory/scenarios).

.PARAMETER DryRun
  Shadow create --dry-run only (no write); still runs evaluate (Stage A required).

.PARAMETER SkipEvaluate
  Debug only: skip Stage A. Requires -AllowIncomplete; exits non-zero (2).
  Stage A is required for a complete dogfood run.

.PARAMETER AllowIncomplete
  With -SkipEvaluate only: permit incomplete debug runs that still exit 2.

.PARAMETER SkipShadow
  Skip shadow create (compare only if inputs already exist).

.EXAMPLE
  .\scripts\dogfood-shadow.ps1 -WorkDir C:\temp\ai-brains-dogfood

.EXAMPLE
  .\scripts\dogfood-shadow.ps1 -WorkDir C:\temp\df -SourceVault C:\vaults\test.db -ProjectId <guid> -SkipMigrate
#>
[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)]
    [string]$WorkDir,

    [Parameter(Mandatory = $false)]
    [string]$SourceVault,

    [Parameter(Mandatory = $false)]
    [string]$ProjectId,

    [Parameter(Mandatory = $false)]
    [switch]$SkipMigrate,

    [Parameter(Mandatory = $false)]
    [string]$EvaluateFixtures = "fixtures\governed-memory\scenarios",

    [Parameter(Mandatory = $false)]
    [switch]$DryRun,

    [Parameter(Mandatory = $false)]
    [switch]$SkipEvaluate,

    [Parameter(Mandatory = $false)]
    [switch]$AllowIncomplete,

    [Parameter(Mandatory = $false)]
    [switch]$SkipShadow
)

$ErrorActionPreference = 'Stop'

function Write-Info([string]$Message) {
    Write-Host "[dogfood] $Message"
}

function Write-WarnMsg([string]$Message) {
    Write-Warning "[dogfood] $Message"
}

function Get-FileSha256Hex([string]$Path) {
    if (-not (Test-Path -LiteralPath $Path)) {
        return $null
    }
    try {
        return (Get-FileHash -LiteralPath $Path -Algorithm SHA256).Hash.ToLowerInvariant()
    }
    catch {
        # Live vault may be locked by daemon/another process — do NOT treat as N/A pass (D24).
        Write-WarnMsg "D24: could not hash live vault ($Path): $_"
        return $null
    }
}

# PS 5.1 Set-Content -Encoding utf8 emits BOM; serde_json rejects BOM-leading JSON.
$script:Utf8NoBom = New-Object System.Text.UTF8Encoding $false

function Write-JsonNoBom([string]$Path, [string]$Content) {
    [System.IO.File]::WriteAllText($Path, $Content, $script:Utf8NoBom)
}

function Write-CliStdoutNoBom([string]$Path, [scriptblock]$Command) {
    # Capture CLI stdout only (not stderr) and write BOM-less UTF-8 for serde_json.
    $out = & $Command
    $exit = $LASTEXITCODE
    if ($exit -ne 0) {
        throw "CLI command failed exit $exit while writing $Path"
    }
    $text = if ($null -eq $out) {
        ""
    }
    elseif ($out -is [System.Array]) {
        ($out | ForEach-Object { "$_" }) -join [Environment]::NewLine
    }
    else {
        "$out"
    }
    if (-not [string]::IsNullOrEmpty($text) -and -not $text.EndsWith("`n")) {
        $text = $text + [Environment]::NewLine
    }
    Write-JsonNoBom -Path $Path -Content $text
}

function Resolve-LiveVaultPathForIntegrity {
    <#
      D24 resolve WITHOUT mutating env to shadow.
      Prefer process AI_BRAINS_VAULT_PATH if set and file exists and is not under WorkDir.
      Else ~/.ai-brains/.env, else common default vault.db.
    #>
    param([string]$WorkDirResolved)

    $workFull = [System.IO.Path]::GetFullPath($WorkDirResolved)

    if ($env:AI_BRAINS_VAULT_PATH) {
        $candidate = $env:AI_BRAINS_VAULT_PATH.Trim()
        if ($candidate -and (Test-Path -LiteralPath $candidate)) {
            $candFull = [System.IO.Path]::GetFullPath($candidate)
            if ($candFull.StartsWith($workFull, [System.StringComparison]::OrdinalIgnoreCase)) {
                Write-WarnMsg "AI_BRAINS_VAULT_PATH points under WorkDir ($candFull); ignoring for live integrity (D24)."
            }
            else {
                return $candFull
            }
        }
    }

    # Do not use $HOME/$home — PowerShell automatic variable is read-only.
    $userHome = $env:USERPROFILE
    if (-not $userHome) {
        $userHome = [Environment]::GetEnvironmentVariable("HOME")
    }
    if ($userHome) {
        $envFile = Join-Path $userHome ".ai-brains\.env"
        if (Test-Path -LiteralPath $envFile) {
            $lines = Get-Content -LiteralPath $envFile -ErrorAction SilentlyContinue
            foreach ($line in $lines) {
                if ($line -match '^\s*AI_BRAINS_VAULT_PATH\s*=\s*(.+)\s*$') {
                    $val = $Matches[1].Trim().Trim('"').Trim("'")
                    if ($val -and (Test-Path -LiteralPath $val)) {
                        $candFull = [System.IO.Path]::GetFullPath($val)
                        if (-not $candFull.StartsWith($workFull, [System.StringComparison]::OrdinalIgnoreCase)) {
                            return $candFull
                        }
                    }
                }
            }
        }
        $defaultVault = Join-Path $userHome ".ai-brains\vault.db"
        if (Test-Path -LiteralPath $defaultVault) {
            return [System.IO.Path]::GetFullPath($defaultVault)
        }
    }

    return $null
}

function Assert-NeverVaultPathEnvIsShadow {
    param(
        [string]$ShadowPath,
        [string]$MigratedPath
    )
    # D26 enforcement: refuse if process env currently equals shadow/migrated.
    if (-not $env:AI_BRAINS_VAULT_PATH) {
        return
    }
    $current = $env:AI_BRAINS_VAULT_PATH.Trim()
    if (-not $current) {
        return
    }
    $curFull = [System.IO.Path]::GetFullPath($current)
    foreach ($p in @($ShadowPath, $MigratedPath)) {
        if (-not $p) { continue }
        if (-not (Test-Path -LiteralPath $p)) { continue }
        $pFull = [System.IO.Path]::GetFullPath($p)
        if ($curFull.Equals($pFull, [System.StringComparison]::OrdinalIgnoreCase)) {
            throw "D26 refuse: AI_BRAINS_VAULT_PATH points at shadow/migrated path ($curFull). Unset it and use --vault-path only."
        }
    }
}

function Get-D24Status {
    param(
        [string]$LiveVault,
        [string]$ShaPre,
        [string]$ShaPost
    )
    if (-not $LiveVault) {
        return "na"
    }
    if ($ShaPre -and $ShaPost) {
        if ($ShaPre -eq $ShaPost) {
            return "verified"
        }
        return "mismatch"
    }
    if ($ShaPre -or $ShaPost) {
        return "partial"
    }
    return "unreadable"
}

# --- Preconditions ---
$cli = Get-Command ai-brains -ErrorAction SilentlyContinue
if (-not $cli) {
    Write-Error "ai-brains CLI not found on PATH. Build/install crates/ai-brains-cli first."
    exit 1
}

if ($SkipEvaluate -and -not $AllowIncomplete) {
    throw "Stage A is required for a complete dogfood run. Remove -SkipEvaluate, or pass -AllowIncomplete (debug only; exits 2)."
}

if (-not (Test-Path -LiteralPath $WorkDir)) {
    New-Item -ItemType Directory -Force -Path $WorkDir | Out-Null
}
$WorkDir = [System.IO.Path]::GetFullPath($WorkDir)

$shadowDb = Join-Path $WorkDir "shadow.db"
$migratedDb = Join-Path $WorkDir "migrated.db"
$evaluateReport = Join-Path $WorkDir "evaluate-report.json"
$governedPacket = Join-Path $WorkDir "governed-packet.json"
$legacyPreflight = Join-Path $WorkDir "legacy-preflight.json"
$compareOut = Join-Path $WorkDir "dogfood-compare.json"
$migrateReport = Join-Path $WorkDir "migrate-report.json"
$fixtureDb = Join-Path $WorkDir "fixture.db"
$fixtureProjectIdFile = Join-Path $WorkDir "fixture-project-id.txt"
$stageAReportHashFile = Join-Path $WorkDir "stage-a-report-hash.txt"
$fixtureProjectId = $null
$scriptIncomplete = $false
$exitCode = 0

Write-Info "WorkDir=$WorkDir"
Write-Info "D26: will pass --vault-path for compare; will NEVER set AI_BRAINS_VAULT_PATH to shadow/migrated."
Write-Info "Stage D is REFUSED by this script (requires explicit user approval)."

# --- D24 pre ---
$liveVault = Resolve-LiveVaultPathForIntegrity -WorkDirResolved $WorkDir
$shaPre = $null
if ($liveVault) {
    $shaPre = Get-FileSha256Hex -Path $liveVault
    if ($shaPre) {
        Write-Info "D24 live vault pre-hash: $shaPre (path basename only in evidence)"
    }
    else {
        Write-WarnMsg "D24 live vault resolved but SHA-256 unreadable (locked); will fail-closed unless both pre/post hashes are obtained"
    }
}
else {
    Write-Info "D24: no live vault resolved (N/A)"
}

# --- Stage A: evaluate ---
$t169Exit = $null
$t169ReportHash = $null
$t169Hard = $null

if ($SkipEvaluate) {
    Write-WarnMsg "SkipEvaluate + AllowIncomplete: Stage A skipped — run is INCOMPLETE (will exit 2)"
    $scriptIncomplete = $true
    $exitCode = 2
}
else {
    Write-Info "Stage A: evaluate governed --fixtures $EvaluateFixtures"
    if (-not (Test-Path -LiteralPath $EvaluateFixtures)) {
        throw "Evaluate fixtures path not found: $EvaluateFixtures"
    }
    # D20 idempotency: remove existing report (or use --allow-report-overwrite).
    if (Test-Path -LiteralPath $evaluateReport) {
        Write-WarnMsg "Removing existing evaluate-report.json for idempotent re-run (D20)"
        Remove-Item -LiteralPath $evaluateReport -Force
    }
    & ai-brains --no-project-context evaluate governed `
        --fixtures $EvaluateFixtures `
        --report $evaluateReport `
        --allow-report-overwrite
    $t169Exit = $LASTEXITCODE
    if ($t169Exit -ne 0) {
        Write-Error "Stage A evaluate failed with exit $t169Exit (0=pass, 1=tool, 7=trust fail). Aborting dogfood."
        exit $t169Exit
    }
    if (-not (Test-Path -LiteralPath $evaluateReport)) {
        throw "Stage A evaluate-report.json missing after exit 0: $evaluateReport"
    }
    try {
        $evalJson = Get-Content -LiteralPath $evaluateReport -Raw -Encoding UTF8 | ConvertFrom-Json
    }
    catch {
        throw "Stage A evaluate-report.json unparseable: $_"
    }
    $t169ReportHash = $evalJson.report_hash
    if (-not $t169ReportHash) {
        throw "Stage A evaluate-report.json missing report_hash"
    }
    # Missing hard_gates_passed → fail (do not default true).
    if ($null -eq $evalJson.PSObject.Properties['hard_gates_passed'] -or $null -eq $evalJson.hard_gates_passed) {
        throw "Stage A evaluate-report.json missing hard_gates_passed"
    }
    $t169Hard = [bool]$evalJson.hard_gates_passed
    if (-not $t169Hard) {
        throw "Stage A hard_gates_passed=false; aborting dogfood"
    }
    Write-Info "Stage A report_hash=$t169ReportHash hard_gates_passed=$t169Hard"
    # Compare to prior baseline (if any) BEFORE overwriting — Stage C re-runs use this.
    if (Test-Path -LiteralPath $stageAReportHashFile) {
        $baseline = (Get-Content -LiteralPath $stageAReportHashFile -Raw -Encoding UTF8).Trim()
        if ($baseline -and ($baseline -ne $t169ReportHash)) {
            Write-WarnMsg "Evaluate report_hash DRIFT vs WorkDir Stage A baseline: baseline=$baseline current=$t169ReportHash (document intentional fixture drift for Stage C)"
        }
        elseif ($baseline) {
            Write-Info "Evaluate report_hash matches WorkDir Stage A baseline"
        }
    }
    # Persist Stage A baseline for Stage C drift detection on later runs in same WorkDir.
    Write-JsonNoBom -Path $stageAReportHashFile -Content $t169ReportHash
}

# --- Stage B/C source ---
$stage = "B"
$sourceForShadow = $null
$briefingProjectId = $null

if ($SourceVault) {
    if (-not (Test-Path -LiteralPath $SourceVault)) {
        throw "SourceVault not found: $SourceVault"
    }
    $sourceForShadow = [System.IO.Path]::GetFullPath($SourceVault)
    $stage = "C"
    Write-Info "Stage C source vault provided (prefer operator test vault): using for shadow"
    if ($ProjectId) {
        $briefingProjectId = $ProjectId.Trim()
        Write-Info "Stage C ProjectId=$briefingProjectId"
    }
    else {
        Write-WarnMsg "Stage C without -ProjectId: briefing project may be empty on multi-project vaults (see runbook)"
    }
}
else {
    # Stage B fixture vault
    Write-Info "Stage B: ensuring fixture vault at $fixtureDb"
    if (-not (Test-Path -LiteralPath $fixtureDb)) {
        & ai-brains --no-project-context --vault-path $fixtureDb init
        if ($LASTEXITCODE -ne 0) {
            throw "fixture init failed exit $LASTEXITCODE"
        }
        # Pin requires project/session; set process-scoped only for fixture (not shadow).
        # Use ephemeral IDs so pin works without loading project .env.
        $prevProject = $env:AI_BRAINS_PROJECT_ID
        $prevSession = $env:AI_BRAINS_SESSION_ID
        try {
            $fixtureProjectId = [guid]::NewGuid().ToString()
            $env:AI_BRAINS_PROJECT_ID = $fixtureProjectId
            $env:AI_BRAINS_SESSION_ID = [guid]::NewGuid().ToString()
            # Persist for re-run / briefing --project-id (R1-01).
            Write-JsonNoBom -Path $fixtureProjectIdFile -Content $fixtureProjectId
            & ai-brains --no-project-context --vault-path $fixtureDb pin "DECISION: T170 Stage B fixture decision"
            if ($LASTEXITCODE -ne 0) {
                throw "Stage B pin decision failed exit $LASTEXITCODE (refusing empty fixture)"
            }
            & ai-brains --no-project-context --vault-path $fixtureDb pin "CONSTRAINT: T170 fixture vault is not live"
            if ($LASTEXITCODE -ne 0) {
                throw "Stage B pin constraint failed exit $LASTEXITCODE (refusing incomplete fixture)"
            }
            Write-Info "Stage B fixture project_id=$fixtureProjectId (saved to fixture-project-id.txt)"
        }
        finally {
            if ($null -eq $prevProject) { Remove-Item Env:AI_BRAINS_PROJECT_ID -ErrorAction SilentlyContinue }
            else { $env:AI_BRAINS_PROJECT_ID = $prevProject }
            if ($null -eq $prevSession) { Remove-Item Env:AI_BRAINS_SESSION_ID -ErrorAction SilentlyContinue }
            else { $env:AI_BRAINS_SESSION_ID = $prevSession }
        }
    }
    else {
        # Reuse existing fixture; load persisted project_id when present.
        if (Test-Path -LiteralPath $fixtureProjectIdFile) {
            $fixtureProjectId = (Get-Content -LiteralPath $fixtureProjectIdFile -Raw -Encoding UTF8).Trim()
            if ($fixtureProjectId) {
                Write-Info "Stage B reusing fixture project_id=$fixtureProjectId"
            }
        }
        if (-not $fixtureProjectId) {
            Write-WarnMsg "Stage B fixture.db exists but fixture-project-id.txt missing; briefing may be empty without --project-id"
        }
    }
    $sourceForShadow = $fixtureDb
    $briefingProjectId = $fixtureProjectId
    if ($ProjectId) {
        # Explicit override even on Stage B (rare).
        $briefingProjectId = $ProjectId.Trim()
    }
}

# --- Shadow ---
if (-not $SkipShadow) {
    Assert-NeverVaultPathEnvIsShadow -ShadowPath $shadowDb -MigratedPath $migratedDb

    if ($DryRun) {
        Write-Info "Shadow dry-run: source=$sourceForShadow dest=$shadowDb"
        & ai-brains --no-project-context shadow create `
            --source $sourceForShadow `
            --destination $shadowDb `
            --dry-run
        if ($LASTEXITCODE -ne 0) {
            throw "shadow create --dry-run failed exit $LASTEXITCODE"
        }
        Write-Info "DryRun complete after shadow plan. Skipping migrate/compare materialization."
        # Still do D24 post check
    }
    else {
        if (Test-Path -LiteralPath $shadowDb) {
            Write-WarnMsg "Removing existing shadow.db for idempotent re-run"
            Remove-Item -LiteralPath $shadowDb -Force
            Get-ChildItem -LiteralPath $WorkDir -Filter "shadow.db-*" -ErrorAction SilentlyContinue | Remove-Item -Force
            $manifest = Join-Path $WorkDir "shadow-manifest.json"
            if (Test-Path -LiteralPath $manifest) { Remove-Item -LiteralPath $manifest -Force }
        }
        Write-Info "Shadow create (default redact): $sourceForShadow -> $shadowDb"
        & ai-brains --no-project-context shadow create `
            --source $sourceForShadow `
            --destination $shadowDb
        if ($LASTEXITCODE -ne 0) {
            throw "shadow create failed exit $LASTEXITCODE"
        }
    }
}

# --- Optional migrate ---
$dbForCompare = $shadowDb
if (-not $SkipMigrate -and -not $DryRun -and -not $SkipShadow -and (Test-Path -LiteralPath $shadowDb)) {
    Write-Info "Migrate governed (confirm) shadow -> migrated under WorkDir"
    if (Test-Path -LiteralPath $migratedDb) {
        Write-WarnMsg "Removing existing migrated.db for re-run"
        Remove-Item -LiteralPath $migratedDb -Force -ErrorAction SilentlyContinue
        Get-ChildItem -LiteralPath $WorkDir -Filter "migrated.db-*" -ErrorAction SilentlyContinue | Remove-Item -Force
        $mm = Join-Path $WorkDir "migrate-manifest.json"
        if (Test-Path -LiteralPath $mm) { Remove-Item -LiteralPath $mm -Force }
    }
    & ai-brains --no-project-context migrate governed `
        --source $shadowDb `
        --destination $migratedDb `
        --report $migrateReport `
        --confirm
    if ($LASTEXITCODE -ne 0) {
        Write-WarnMsg "migrate governed failed exit $LASTEXITCODE; falling back to shadow for compare"
    }
    elseif (Test-Path -LiteralPath $migratedDb) {
        $dbForCompare = $migratedDb
    }
}
elseif ($SkipMigrate) {
    Write-Info "SkipMigrate set"
}

# --- D26 guard before compare ---
Assert-NeverVaultPathEnvIsShadow -ShadowPath $shadowDb -MigratedPath $migratedDb
# Explicit: never assign env to shadow
if ($env:AI_BRAINS_VAULT_PATH) {
    $probe = $env:AI_BRAINS_VAULT_PATH.Trim()
    if ($probe) {
        $probeFull = [System.IO.Path]::GetFullPath($probe)
        $shadowFull = if (Test-Path -LiteralPath $shadowDb) { [System.IO.Path]::GetFullPath($shadowDb) } else { $null }
        if ($shadowFull -and $probeFull.Equals($shadowFull, [System.StringComparison]::OrdinalIgnoreCase)) {
            throw "D26 refuse: refusing to continue with AI_BRAINS_VAULT_PATH=shadow"
        }
    }
}

# D24 post hash (always when live vault resolved)
$shaPost = $null
if ($liveVault) {
    $shaPost = Get-FileSha256Hex -Path $liveVault
    if ($shaPre -and $shaPost -and ($shaPre -ne $shaPost)) {
        Write-Error "D24 FAIL: live vault SHA-256 changed (pre=$shaPre post=$shaPost)"
        exit 1
    }
    if ($shaPost) {
        Write-Info "D24 live vault post-hash: $shaPost (match=$($shaPre -eq $shaPost))"
    }
    else {
        Write-WarnMsg "D24 live vault post-hash: unreadable (locked)"
    }
}

$d24Status = Get-D24Status -LiveVault $liveVault -ShaPre $shaPre -ShaPost $shaPost
Write-Info "D24 status=$d24Status"
if ($d24Status -eq "unreadable" -or $d24Status -eq "partial") {
    Write-WarnMsg "D24 not empirically verified (status=$d24Status); dogfood will exit non-zero (fail-closed)"
    $scriptIncomplete = $true
    if ($exitCode -eq 0) { $exitCode = 1 }
}
elseif ($d24Status -eq "mismatch") {
    # Already exited above when both hashes present and differ; defensive.
    $scriptIncomplete = $true
    if ($exitCode -eq 0) { $exitCode = 1 }
}

# --- Capture compare inputs (skip if DryRun with no shadow file) ---
if (-not $DryRun -and (Test-Path -LiteralPath $dbForCompare)) {
    # Idempotent re-run: clear regenerated JSON partials under WorkDir.
    foreach ($partial in @($governedPacket, $legacyPreflight, $compareOut)) {
        if (Test-Path -LiteralPath $partial) {
            Write-WarnMsg "Removing regenerated artifact for re-run: $(Split-Path -Leaf $partial)"
            Remove-Item -LiteralPath $partial -Force
        }
    }

    # Global --vault-path must precede the subcommand (not global=true in clap).
    $briefingArgs = @(
        "--no-project-context",
        "--vault-path", $dbForCompare,
        "briefing", "project",
        "--format", "json"
    )
    if ($briefingProjectId) {
        $briefingArgs += @("--project-id", $briefingProjectId)
        Write-Info "Capture governed: --vault-path $dbForCompare briefing project --project-id $briefingProjectId"
    }
    else {
        Write-Info "Capture governed: --vault-path $dbForCompare briefing project"
    }

    $prevFlag = $env:AI_BRAINS_GOVERNED_BRIEFING
    try {
        Write-CliStdoutNoBom -Path $governedPacket -Command {
            & ai-brains @briefingArgs
        }

        Write-Info "Capture legacy: preflight --vault-path with AI_BRAINS_GOVERNED_BRIEFING=0"
        $env:AI_BRAINS_GOVERNED_BRIEFING = "0"
        Write-CliStdoutNoBom -Path $legacyPreflight -Command {
            & ai-brains --no-project-context --vault-path $dbForCompare preflight --format json
        }
    }
    finally {
        if ($null -eq $prevFlag) {
            Remove-Item Env:AI_BRAINS_GOVERNED_BRIEFING -ErrorAction SilentlyContinue
        }
        else {
            $env:AI_BRAINS_GOVERNED_BRIEFING = $prevFlag
        }
    }

    # --- dogfood compare CLI ---
    Write-Info "Emit dogfood-compare.json via ai-brains dogfood compare"
    $compareArgs = @(
        "--no-project-context", "dogfood", "compare",
        "--governed", $governedPacket,
        "--legacy", $legacyPreflight,
        "--out", $compareOut,
        "--stage", $stage
    )
    if (Test-Path -LiteralPath $evaluateReport) {
        $compareArgs += @("--evaluate-report", $evaluateReport)
    }
    if (Test-Path -LiteralPath $migrateReport) {
        $compareArgs += @("--migrate-report", $migrateReport)
    }
    if (Test-Path -LiteralPath $shadowDb) {
        $compareArgs += @("--shadow", $shadowDb)
    }
    if (Test-Path -LiteralPath $migratedDb) {
        $compareArgs += @("--migrated", $migratedDb)
    }
    # Pass live path whenever resolved so CLI fail-closes if hashes incomplete (D24 honesty).
    if ($liveVault) {
        $compareArgs += @("--live-vault", $liveVault)
    }
    if ($shaPre) { $compareArgs += @("--sha256-pre", $shaPre) }
    if ($shaPost) { $compareArgs += @("--sha256-post", $shaPost) }
    if ($null -ne $t169Exit) {
        $compareArgs += @("--t169-exit", "$t169Exit")
    }
    if ($t169ReportHash) {
        $compareArgs += @("--t169-report-hash", $t169ReportHash)
    }
    if ($null -ne $t169Hard) {
        $compareArgs += @("--t169-hard-gates-passed", $(if ($t169Hard) { "true" } else { "false" }))
    }

    & ai-brains @compareArgs
    if ($LASTEXITCODE -ne 0) {
        throw "dogfood compare failed exit $LASTEXITCODE"
    }
    Write-Info "Wrote $compareOut"
}
else {
    Write-Info "Skipping compare capture (DryRun or missing db)"
}

# --- Rollback drill helper (print only; does not enable live) ---
Write-Host ""
Write-Host "=== Rollback drill (D21/D23) — run manually on work db ===" -ForegroundColor Cyan
$projectIdHint = if ($briefingProjectId) { " --project-id $briefingProjectId" } else { "" }
Write-Host @"
# Flag off → legacy (expect text does NOT match (governed))
`$env:AI_BRAINS_GOVERNED_BRIEFING = '0'
ai-brains --no-project-context --vault-path $dbForCompare preflight --format json

# Flag on → governed probe (expect (governed) in text OR briefing project OK)
`$env:AI_BRAINS_GOVERNED_BRIEFING = '1'
ai-brains --no-project-context --vault-path $dbForCompare preflight --format json
ai-brains --no-project-context --vault-path $dbForCompare briefing project$projectIdHint --format json

# Rollback primary
`$env:AI_BRAINS_GOVERNED_BRIEFING = '0'
# NEVER use: preflight --summary for governed observability

# Emergency User-env clear (MANUAL ONLY — this script never sets User scope):
# [Environment]::SetEnvironmentVariable('AI_BRAINS_GOVERNED_BRIEFING', `$null, 'User')
"@

Write-Host ""
Write-Host "=== Stage D (LIVE ENABLEMENT) — REFUSED ===" -ForegroundColor Yellow
Write-Host "Live enablement requires explicit user approval in-session."
Write-Host "This script will never set User-level env or enable production governed mode."
Write-Host "If approved later: session-only `$env:AI_BRAINS_GOVERNED_BRIEFING='1' + D25 observation."
Write-Host "Human checklist: Docs/EVALUATION/templates/dogfood-human-checklist.md"
Write-Host "Runbook: Docs/EVALUATION/SHADOW-DOGFOOD-GATE.md"

if ($scriptIncomplete) {
    Write-WarnMsg "Done (stage=$stage) INCOMPLETE: exit $exitCode (D24 status=$d24Status). Fix live vault readability or run without a locked live vault for honest N/A."
    exit $exitCode
}

Write-Info "Done (stage=$stage). Exit 0 (D24 status=$d24Status)."
exit 0
