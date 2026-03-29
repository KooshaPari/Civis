#Requires -Version 5.1
<#
.SYNOPSIS
    Install DINOForge git hooks via prek.
.DESCRIPTION
    Installs prek (Rust-based pre-commit replacement) and wires up:
    - pre-commit: trailing-whitespace, YAML/JSON check, dotnet format
    - pre-push:   dotnet test (unit + integration, ~6s)
    Run once after cloning.
.EXAMPLE
    pwsh -File scripts/install-hooks.ps1
#>

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"
$RepoRoot = Split-Path -Parent $PSScriptRoot

Write-Host "Installing DINOForge git hooks via prek..." -ForegroundColor Cyan

# ── Install prek if missing ────────────────────────────────────────────────────
$prekAvailable = Get-Command prek -ErrorAction SilentlyContinue
if (-not $prekAvailable) {
    Write-Host "prek not found. Installing..." -ForegroundColor Yellow
    irm https://github.com/j178/prek/releases/latest/download/prek-installer.ps1 | iex
    # Refresh PATH
    $env:PATH = [System.Environment]::GetEnvironmentVariable("PATH", "User") + ";" + $env:PATH
    $prekAvailable = Get-Command prek -ErrorAction SilentlyContinue
    if (-not $prekAvailable) {
        Write-Host "prek install succeeded but binary not in PATH yet." -ForegroundColor Yellow
        Write-Host "Restart your shell and re-run this script." -ForegroundColor Yellow
        exit 1
    }
}

Write-Host "prek $(prek --version)" -ForegroundColor Green

# ── Install hooks ──────────────────────────────────────────────────────────────
Push-Location $RepoRoot
prek install --hook-type pre-commit --overwrite
prek install --hook-type pre-push --overwrite
Pop-Location

Write-Host ""
Write-Host "Done. Hooks active:" -ForegroundColor Cyan
Write-Host "  pre-commit: trailing-whitespace, end-of-file-fixer, check-yaml, check-json, yamllint, dotnet-format"
Write-Host "  pre-push:   dotnet test (unit + integration)"
Write-Host ""
Write-Host "Verify:    prek run --all-files"
Write-Host "Run tests: pwsh -File scripts/test-local.ps1"
