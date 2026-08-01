# T185 — L13 automated forbidden-phrase scan over elevated release docs
#
# Scans elevated surfaces for high-confidence affirmative overclaims.
# Lines that also match a negation/allow pattern are skipped.
#
# Usage:
#   .\scripts\check-release-claims.ps1
#   .\scripts\check-release-claims.ps1 -Path C:\dev\AI-Brains
#   .\scripts\check-release-claims.ps1 -Strict  # same as default hard fail
#
# Exit 0 if clean, 1 if hits. Complements (does not replace) human RELEASE-CLAIMS review.

param(
    [string]$Path = "",
    [switch]$Strict
)

$ErrorActionPreference = "Stop"

if (-not $Path) {
    $Path = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
}
$root = (Resolve-Path $Path).Path

# Elevated set per RELEASE-CLAIMS / spec §6.4 (forward-slash paths; Join-Path is OS-safe)
$elevated = @(
    "README.md",
    "Docs/ARCHITECTURE.md",
    "Docs/CAPABILITIES.md",
    "Docs/OPERATIONS.md",
    "Docs/README.md",
    "Docs/INSTALL.md",
    "Docs/SECURITY-LIMITS.md",
    "SECURITY.md",
    "CHANGELOG.md",
    "Docs/RELEASE-CLAIMS.md",
    "Docs/RELEASE-CHECKLIST.md"
)

# High-confidence forbidden affirmative patterns (regex)
# Keep practical: fail on clear overclaims; allow non-claim / residual discussion.
$rules = @(
    @{ Name = "SOC2 certified"; Pattern = '\bSOC\s*2\s+certified\b' },
    @{ Name = "ISO 27001 certified"; Pattern = '\bISO\s*27001\s+certified\b' },
    @{ Name = "perfect deletion"; Pattern = '\bperfect\s+deletion\b' },
    @{ Name = "metadata-private (product property)"; Pattern = '\bmetadata-private\b' },
    @{ Name = "SLSA Build L3 achieved"; Pattern = '\bSLSA\s*(Build\s*)?L3\b' },
    @{ Name = "SLSA certified"; Pattern = '\bSLSA\s+certified\b' },
    @{ Name = "fully compliant (SSDF/OpenSSF)"; Pattern = '\bfully\s+compliant\b' },
    @{ Name = "tamper-proof supply chain"; Pattern = '\btamper-proof\s+supply\s+chain\b' },
    @{ Name = "full DB encryption (unqualified)"; Pattern = '\bfull\s+(DB|database)\s+encryption\b' },
    @{ Name = "SQLCipher encrypts the database (unqualified)"; Pattern = '\bSQLCipher\s+encrypts\s+the\s+database\b' },
    @{ Name = "plugin sandbox shipped"; Pattern = '\b(plugin|third-party)\s+sandbox\s+(is\s+)?(shipped|enabled|available)\b' },
    @{ Name = "invented doctor CLI as shipped"; Pattern = '\bai-brains\s+doctor\b(?!.*\b(not|no|non-|DTO|type|contract)\b)' },
    @{ Name = "invented recovery export CLI as shipped"; Pattern = '\bai-brains\s+recovery\s+export\b(?!.*\b(not|no|non-|not shipped|kit|drill)\b)' }
)

# If a matching line also hits this, treat as non-claim / residual honesty context.
# Tightened after R2: do NOT free-pass on residual IDs alone or bare "without"
# (those can appear on affirmative marketing lines). Require real negation /
# forbidden-inventory / non-claim framing.
$allowIf = '(?i)(\b(not|no|non-|never|forbidden|non-claim|must not|do not|don''t|out of scope|explicitly not|does not|isn''t|cannot|can''t|prohibited|denied|unclaimed|no claim|as a product property|without\s+F8|qualifier)\b|Do\s+\*\*not\*\*\s+claim|\bnot\s+(claimed|a claim|shipped|live|supported)\b)'

function Test-IsForbiddenInventoryLine {
    param([string]$Line)
    # List/inventory of forbidden phrases: bullets that only quote/backtick the term
    # e.g. - "Perfect deletion"  /  - `metadata-private` as a product property
    if ($Line -match '^\s*[-*]\s+[`"''].*[`"'']') { return $true }
    if ($Line -match '^\s*[-*]\s+`[^`]+`') { return $true }
    # Table cells documenting non-claim / forbidden class names
    if ($Line -match '(?i)\|\s*.*\b(non-claim|forbidden|residual|must not|do not)\b') { return $true }
    return $false
}

Write-Host "=== check-release-claims.ps1 ===" -ForegroundColor Cyan
Write-Host "  Root: $root"
Write-Host "  Elevated files: $($elevated.Count)"

$hits = @()
$scanned = 0
$missing = @()

foreach ($rel in $elevated) {
    $full = Join-Path $root $rel
    if (-not (Test-Path $full)) {
        $missing += $rel
        Write-Host "  [SKIP missing] $rel" -ForegroundColor Yellow
        continue
    }
    $scanned++
    $lines = Get-Content $full
    # Track recent "forbidden / non-claim inventory" section headers (soft window)
    $inForbiddenSection = $false
    for ($i = 0; $i -lt $lines.Count; $i++) {
        $line = $lines[$i]
        # Skip pure markdown table separators
        if ($line -match '^\s*\|[\s\-:|]+\|\s*$') { continue }

        if ($line -match '(?i)^#{1,4}\s+.*(forbidden|non-claim|what this release does not|illustrative forbidden)') {
            $inForbiddenSection = $true
        } elseif ($line -match '^#{1,4}\s+') {
            $inForbiddenSection = $false
        }

        foreach ($rule in $rules) {
            if ($line -notmatch $rule.Pattern) { continue }

            # Allow if line carries negation / residual / non-claim context
            if ($line -match $allowIf) { continue }

            # Extra guard: policy bullets / inventory of forbidden phrases
            if ($line -match '(?i)(non-claim|forbidden|must remain|must not appear|do not claim|do \*\*not\*\* claim)') { continue }
            if (Test-IsForbiddenInventoryLine $line) { continue }
            if ($inForbiddenSection) { continue }

            $hits += [pscustomobject]@{
                File    = $rel
                Line    = $i + 1
                Rule    = $rule.Name
                Text    = $line.Trim()
            }
        }
    }
}

if ($missing.Count -gt 0) {
    Write-Host "  Missing elevated files: $($missing -join ', ')" -ForegroundColor Yellow
}

if ($hits.Count -eq 0) {
    Write-Host "[OK] No forbidden affirmative claims in $scanned elevated file(s)" -ForegroundColor Green
    exit 0
}

Write-Host ""
Write-Host "[FAIL] $($hits.Count) elevated claim hit(s):" -ForegroundColor Red
foreach ($h in $hits) {
    Write-Host "  $($h.File):$($h.Line) [$($h.Rule)]" -ForegroundColor Red
    Write-Host "    $($h.Text)"
}

# Always hard-fail on hits (L13). -Strict reserved for future soft modes.
exit 1
