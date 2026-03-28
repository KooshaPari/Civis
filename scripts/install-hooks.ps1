#Requires -Version 5.1
<#
.SYNOPSIS
    Install DINOForge git hooks (pre-commit framework + pre-push test runner).
.DESCRIPTION
    Installs:
    - pre-commit hooks (trailing whitespace, YAML/JSON check, dotnet format)
    - pre-push hook: dotnet test integration
    Run once after cloning. Requires: pre-commit (pip install pre-commit)
.EXAMPLE
    pwsh -File scripts/install-hooks.ps1
#>

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"
$RepoRoot = Split-Path -Parent $PSScriptRoot

Write-Host "Installing DINOForge git hooks..." -ForegroundColor Cyan

# ── pre-commit framework ───────────────────────────────────────────────────────
$pcAvailable = Get-Command pre-commit -ErrorAction SilentlyContinue
if (-not $pcAvailable) {
    Write-Host "pre-commit not found. Install with: pip install pre-commit" -ForegroundColor Yellow
    Write-Host "Skipping pre-commit install." -ForegroundColor Yellow
} else {
    Push-Location $RepoRoot
    pre-commit install --hook-type pre-commit
    pre-commit install --hook-type pre-push
    Pop-Location
    Write-Host "pre-commit hooks installed." -ForegroundColor Green
}

# ── pre-push hook: dotnet test integration ─────────────────────────────────────
$prePushPath = Join-Path $RepoRoot ".git\hooks\pre-push"
$prePushContent = @'
#!/bin/sh
# DINOForge pre-push: run integration tests before push
echo "[pre-push] Running integration tests..."
dotnet test src/Tests/Integration/DINOForge.Tests.Integration.csproj --no-build --verbosity quiet
if [ $? -ne 0 ]; then
  echo "[pre-push] FAIL: integration tests failed. Push blocked."
  exit 1
fi
echo "[pre-push] PASS: integration tests OK."
exit 0
'@

# Write with Unix line endings (required for sh scripts on Windows/Git)
[System.IO.File]::WriteAllText($prePushPath, $prePushContent.Replace("`r`n", "`n"))

# Make executable (git on Windows respects this via core.fileMode=false, but set it anyway)
$gitConfigResult = & git -C $RepoRoot config core.hooksPath
if ($gitConfigResult) {
    Write-Host "Note: core.hooksPath is set to '$gitConfigResult'. Hooks written to .git/hooks/ may not fire." -ForegroundColor Yellow
}

Write-Host "pre-push hook installed at .git/hooks/pre-push" -ForegroundColor Green
Write-Host ""
Write-Host "Done. Hooks active:" -ForegroundColor Cyan
Write-Host "  pre-commit: trailing-whitespace, end-of-file-fixer, check-yaml, check-json, dotnet-format"
Write-Host "  pre-push:   dotnet test (integration, 18 tests)"
Write-Host ""
Write-Host "Run now to verify: pre-commit run --all-files"
Write-Host "Run tests:         pwsh -File scripts/test-local.ps1"
