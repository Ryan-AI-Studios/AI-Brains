#Requires -Version 5.1
<#
.SYNOPSIS
  Thin wrapper around `ai-brains shadow create` for governed-memory dogfood copies.

.DESCRIPTION
  Forwards arguments to the ai-brains CLI. Does not open or mutate the live vault
  beyond what the CLI itself resolves for safety checks.

.EXAMPLE
  .\scripts\shadow-vault.ps1 create --source $env:AI_BRAINS_VAULT_PATH --destination C:\temp\shadow.db --dry-run
#>
[CmdletBinding()]
param(
    [Parameter(ValueFromRemainingArguments = $true)]
    [string[]]$RemainingArgs
)

$ErrorActionPreference = "Stop"

$cli = Get-Command ai-brains -ErrorAction SilentlyContinue
if (-not $cli) {
    Write-Error "ai-brains CLI not found on PATH. Build/install crates/ai-brains-cli first."
    exit 1
}

& ai-brains shadow @RemainingArgs
exit $LASTEXITCODE
