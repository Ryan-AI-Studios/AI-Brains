# T183 relative link checker (soft)
# Resolve repo root: evidence -> track -> tracks -> conductor -> repo (4 parents)
param(
  [string[]]$Files = @(
    'README.md',
    'Docs/README.md',
    'Docs/INSTALL.md',
    'Docs/SECURITY-LIMITS.md',
    'SECURITY.md'
  )
)

$root = $PSScriptRoot
for ($i = 0; $i -lt 4; $i++) {
  $root = Split-Path $root -Parent
}
if (-not (Test-Path (Join-Path $root 'Cargo.toml'))) {
  $probe = $PSScriptRoot
  while ($probe -and -not (Test-Path (Join-Path $probe 'Cargo.toml'))) {
    $probe = Split-Path $probe -Parent
  }
  if ($probe) { $root = $probe }
}

$fail = 0
$ok = 0
foreach ($f in $Files) {
  $path = Join-Path $root $f
  if (-not (Test-Path $path)) {
    Write-Output "MISSING $f"
    $fail++
    continue
  }
  $dir = Split-Path $path -Parent
  foreach ($m in [regex]::Matches((Get-Content $path -Raw), '\[([^\]]+)\]\(([^)]+)\)')) {
    $t = $m.Groups[2].Value
    if ($t -match '^(https?://|mailto:|#)') { continue }
    $t = $t -replace '#.*$', ''
    if ([string]::IsNullOrWhiteSpace($t)) { continue }
    $resolved = [IO.Path]::GetFullPath((Join-Path $dir $t))
    if (Test-Path -LiteralPath $resolved) {
      $ok++
    }
    else {
      Write-Output "FAIL $f -> $t"
      $fail++
    }
  }
}
Write-Output "root=$root ok=$ok fail=$fail"
if ($fail -gt 0) { exit 1 }
