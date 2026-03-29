#Requires -Version 5.1
<#
.SYNOPSIS
    Install DINOForge git hooks via Lefthook.
.DESCRIPTION
    Installs Lefthook (Go binary, no runtime deps) and wires up:
    - pre-commit (parallel): dotnet format, check-yaml, check-json, check-merge-conflicts
    - pre-push   (serial):   dotnet build → unit tests → integration tests

    NO STASHING: Lefthook never stashes unstaged changes (no stage_fixed in config).
    Hooks always run against the full working tree.

    Bypass all hooks:       LEFTHOOK=0 git commit
    Skip one command:       LEFTHOOK_EXCLUDE=format-check git commit
    Run manually:           lefthook run pre-commit
                            lefthook run pre-push

.EXAMPLE
    pwsh -File scripts/install-hooks.ps1
#>

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"
$RepoRoot = Split-Path -Parent $PSScriptRoot

Write-Host "Installing DINOForge git hooks via Lefthook..." -ForegroundColor Cyan

# ── Install Lefthook if missing ────────────────────────────────────────────────
$lhAvailable = Get-Command lefthook -ErrorAction SilentlyContinue
if (-not $lhAvailable) {
    Write-Host "Lefthook not found. Installing via winget..." -ForegroundColor Yellow
    winget install evilmartians.lefthook --accept-source-agreements --accept-package-agreements
    # Refresh PATH for current session
    $env:PATH = [System.Environment]::GetEnvironmentVariable("PATH","Machine") + ";" +
                [System.Environment]::GetEnvironmentVariable("PATH","User")
    # Winget installs to a long path that git's sh.exe can't find.
    # Copy binary to ~/.local/bin which is already in PATH (prek installs there too).
    $localBin = "$env:USERPROFILE\.local\bin"
    New-Item -ItemType Directory -Force -Path $localBin | Out-Null
    $wingetLh = Get-ChildItem "$env:LOCALAPPDATA\Microsoft\WinGet\Packages\evilmartians.lefthook*\lefthook.exe" -ErrorAction SilentlyContinue | Select-Object -First 1
    if ($wingetLh) {
        Copy-Item $wingetLh.FullName "$localBin\lefthook.exe" -Force
        Write-Host "Copied lefthook to $localBin\lefthook.exe" -ForegroundColor Green
    }
    $env:PATH = "$localBin;" + $env:PATH
    $lhAvailable = Get-Command lefthook -ErrorAction SilentlyContinue
    if (-not $lhAvailable) {
        Write-Host "Lefthook installed but not in PATH yet — restart shell and re-run." -ForegroundColor Yellow
        exit 1
    }
}

Write-Host "lefthook $(lefthook version)" -ForegroundColor Green

# ── Ensure pyyaml available for check-yaml hook ───────────────────────────────
$pyYaml = python3 -c "import yaml; print('ok')" 2>&1
if ($pyYaml -ne "ok") {
    Write-Host "Installing pyyaml for check-yaml hook..." -ForegroundColor Yellow
    pip install pyyaml -q
}

# ── Install hooks ──────────────────────────────────────────────────────────────
Push-Location $RepoRoot
lefthook install
Pop-Location

Write-Host ""
Write-Host "Done. Hooks active:" -ForegroundColor Cyan
Write-Host "  pre-commit (parallel):"
Write-Host "    format-check         — dotnet format --verify-no-changes"
Write-Host "    check-yaml           — validate all YAML files"
Write-Host "    check-json           — validate all JSON files"
Write-Host "    check-merge-conflicts— detect real conflict markers"
Write-Host "  pre-push (serial):"
Write-Host "    build                — dotnet build"
Write-Host "    test-unit            — 1,222 unit tests"
Write-Host "    test-integration     — 18 integration tests"
Write-Host ""
Write-Host "Verify:    lefthook run pre-commit --all-files"
Write-Host "Run tests: pwsh -File scripts/test-local.ps1"
