# AI-Brains Hook for OpenCode
# Handles session.created, message.updated, session.idle, and experimental.session.compacting events.

# Initialize UTF-8 encoding (BOM-less) for standard streams and file I/O
$utf8NoBom = New-Object System.Text.UTF8Encoding $false
$OutputEncoding = [Console]::InputEncoding = [Console]::OutputEncoding = $utf8NoBom

$logPrefix = '[ai-brains-opencode]'

function Write-Log($message) {
    [Console]::Error.WriteLine("$logPrefix $message")
}

function Write-HookResponse($response) {
    $response | ConvertTo-Json -Depth 10 -Compress
}

function Load-Env($path) {
    if (Test-Path -LiteralPath $path) {
        Write-Log "Loading env from $path"
        $content = Get-Content -LiteralPath $path -Raw
        $content -split "`r?`n" | ForEach-Object {
            if ($_ -match '^\s*([^#\s][^=]*)\s*=\s*(.*)$') {
                $name = $matches[1].Trim()
                $value = $matches[2].Trim().Trim('"').Trim("'")
                if (-not (Test-Path "Env:$name")) {
                    Set-Item "Env:$name" $value
                }
            }
        }
    }
}

function De-Noise($content) {
    if (-not $content) { return $null }
    $lines = $content -split "`r?`n"
    $filteredLines = [System.Collections.ArrayList]::new()
    $inCodeBlock = $false
    $currentBlock = [System.Collections.ArrayList]::new()

    foreach ($line in $lines) {
        if ($line -match '^```') {
            if ($inCodeBlock) {
                if ($currentBlock.Count -le 10) {
                    $filteredLines.Add('```') | Out-Null
                    foreach ($blockLine in $currentBlock) {
                        $filteredLines.Add($blockLine) | Out-Null
                    }
                    $filteredLines.Add('```') | Out-Null
                } else {
                    $filteredLines.Add('```... [Long block stripped] ...```') | Out-Null
                }
                $currentBlock = [System.Collections.ArrayList]::new()
                $inCodeBlock = $false
            } else {
                $inCodeBlock = $true
            }
            continue
        }

        if ($inCodeBlock) {
            $currentBlock.Add($line) | Out-Null
        } else {
            $filteredLines.Add($line) | Out-Null
        }
    }

    return ($filteredLines -join "`n")
}

function Invoke-Ingest($content, $inputJson, $projectDir, $label) {
    if (-not $content) { return }
    $content = De-Noise $content

    if ($projectDir) {
        $localScript = Join-Path $projectDir '.agents\skills\ai-brains\scripts\ingest.ps1'
        if (Test-Path -LiteralPath $localScript) {
            Write-Log "$label calling local ingest script"
            Push-Location $projectDir
            try {
                & $localScript -Content $content -Role 'assistant' | Out-Null
            } finally {
                Pop-Location
            }
            return
        }
    }

    Write-Log "$label falling back to direct CLI ingest"
    $harnessId = if ($env:AI_BRAINS_HARNESS_ID) { $env:AI_BRAINS_HARNESS_ID } else { 'opencode' }
    $sessionId = if ($env:AI_BRAINS_SESSION_ID) { $env:AI_BRAINS_SESSION_ID } else { $inputJson.sessionId }

    if (-not $env:AI_BRAINS_PROJECT_ID -or -not $sessionId) {
        Write-Log "$label missing project_id or session_id for direct ingest"
        return
    }

    $ingestPayload = @{
        session_id = $sessionId
        project_id = $env:AI_BRAINS_PROJECT_ID
        harness_id = $harnessId
        turn_id = [guid]::NewGuid().ToString()
        role = 'assistant'
        content = $content
        privacy = 'LocalOnly'
    } | ConvertTo-Json -Compress

    $tempFile = [System.IO.Path]::GetTempFileName()
    try {
        [System.IO.File]::WriteAllText($tempFile, $ingestPayload, $utf8NoBom)
        Get-Content -LiteralPath $tempFile -Raw | ai-brains ingest 2>$null | Out-Null
    } finally {
        if (Test-Path -LiteralPath $tempFile) { Remove-Item -LiteralPath $tempFile -Force }
    }
}

try {
    $stdin = [Console]::In.ReadToEnd()
    if (-not $stdin) { $stdin = $input | Out-String }
    $inputJson = $stdin | ConvertFrom-Json
} catch {
    Write-HookResponse @{ success = $true }
    exit 0
}

$projectDir = $inputJson.directory
if (-not $projectDir) { $projectDir = $inputJson.cwd }
if (-not $projectDir) { $projectDir = $PWD.Path }

if ($projectDir) { Load-Env (Join-Path $projectDir '.env') }
Load-Env (Join-Path $HOME '.ai-brains\.env')

$event = $inputJson.event_type
Write-Log "Event: $event | CWD: $projectDir"

switch ($event) {
    'session.created' {
        Write-Log 'Running preflight'
        $preflightRaw = ai-brains preflight --max-words 1500 2>$null
        if ($LASTEXITCODE -ne 0) {
            Write-Log "Preflight failed (exit $LASTEXITCODE)"
            Write-HookResponse @{ success = $true }
            return
        }
        $preflightText = ($preflightRaw -join "`n")
        Write-HookResponse @{
            success = $true
            additionalContext = $preflightText
        }
    }

    'message.updated' {
        if ($inputJson.message.role -eq 'assistant') {
            Invoke-Ingest $inputJson.message.content $inputJson $projectDir 'Message:'
        }
        Write-HookResponse @{ success = $true }
    }

    'session.idle' {
        # Final safety check if needed
        Write-HookResponse @{ success = $true }
    }

    'experimental.session.compacting' {
        # Could inject context here
        Write-HookResponse @{ success = $true }
    }

    default {
        Write-HookResponse @{ success = $true }
    }
}
