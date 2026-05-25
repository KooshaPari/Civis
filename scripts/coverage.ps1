# coverage.ps1 — run from repo root
# Generates code coverage report with detailed metrics

$root = Split-Path -Parent $PSScriptRoot

Write-Host "Building test projects..." -ForegroundColor Green
dotnet build src/DINOForge.sln -c Debug -v minimal | Out-Null

Write-Host "Running tests with coverage collection..." -ForegroundColor Green
dotnet test src/DINOForge.sln `
  --no-build `
  --collect:"XPlat Code Coverage" `
  --results-directory ./coverage/ `
  /p:CollectCoverage=true `
  /p:CoverletOutputFormat=cobertura

Write-Host ""
Write-Host "Coverage results saved to: ./coverage/" -ForegroundColor Yellow
Write-Host "Cobertura XML files:" -ForegroundColor Yellow
Get-ChildItem ./coverage/ -Recurse -Filter "*.cobertura.xml" | ForEach-Object {
    Write-Host "  $($_.FullName)"
}
