# AI-Brains Developer Check Script
# Runs the full CI gate for local verification.

Write-Host "--- Running cargo fmt ---" -ForegroundColor Cyan
cargo fmt --all -- --check
if ($LASTEXITCODE -ne 0) { Write-Host "Format check failed!" -ForegroundColor Red; exit $LASTEXITCODE }

Write-Host "`n--- Running cargo clippy ---" -ForegroundColor Cyan
cargo clippy --workspace --all-targets -- -D warnings
if ($LASTEXITCODE -ne 0) { Write-Host "Clippy failed!" -ForegroundColor Red; exit $LASTEXITCODE }

Write-Host "`n--- Running cargo nextest ---" -ForegroundColor Cyan
cargo nextest run --workspace
if ($LASTEXITCODE -ne 0) { Write-Host "Tests failed!" -ForegroundColor Red; exit $LASTEXITCODE }

Write-Host "`n--- Running cargo deny ---" -ForegroundColor Cyan
cargo deny check
if ($LASTEXITCODE -ne 0) { Write-Host "Cargo deny failed!" -ForegroundColor Red; exit $LASTEXITCODE }

Write-Host "`n--- Running cargo audit ---" -ForegroundColor Cyan
cargo audit
if ($LASTEXITCODE -ne 0) { Write-Host "Cargo audit failed!" -ForegroundColor Red; exit $LASTEXITCODE }

Write-Host "`n[SUCCESS] CI Gate passed!" -ForegroundColor Green
