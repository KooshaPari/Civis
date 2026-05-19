# Cleans lingering testhost processes that hold dll locks after closure-gate runs.
# Usage: pwsh scripts/dev/clean-testhost.ps1

param()

Write-Host "Cleaning testhost.exe processes..." -ForegroundColor Cyan
$killed = Get-Process -Name "testhost*" -ErrorAction SilentlyContinue |
    Measure-Object -ErrorAction SilentlyContinue |
    Select-Object -ExpandProperty Count

if ($killed -gt 0) {
    Get-Process -Name "testhost*" -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue
    Start-Sleep -Seconds 3
    Write-Host "Killed $killed testhost process(es)." -ForegroundColor Green
} else {
    Write-Host "No testhost processes found." -ForegroundColor Yellow
}

# Verify cleanup
$remaining = Get-Process -Name "testhost*" -ErrorAction SilentlyContinue | Measure-Object | Select-Object -ExpandProperty Count
if ($remaining -eq 0) {
    Write-Host "Cleanup verified: 0 testhost processes remain." -ForegroundColor Green
    exit 0
} else {
    Write-Host "WARNING: $remaining testhost process(es) still running after cleanup!" -ForegroundColor Red
    exit 1
}
