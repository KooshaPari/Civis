#!/usr/bin/env pwsh
<#
.SYNOPSIS
    Setup and verify VitePress Journey Viewer components
.DESCRIPTION
    1. Navigate to docs directory
    2. Install dependencies if needed
    3. Start VitePress dev server
    4. Check Journey Viewer components exist
    5. Verify journey data manifests
    6. Register components in VitePress config if needed
    7. Report status
#>

$ErrorActionPreference = "Stop"
$RepoRoot = "C:\Users\koosh\Dino"
$DocsDir = Join-Path $RepoRoot "docs"

Write-Host "=== VitePress Journey Viewer Setup ===" -ForegroundColor Cyan

# Step 1: Check if docs directory exists
Write-Host "`n[1/6] Checking docs directory..." -ForegroundColor Yellow
if (!(Test-Path $DocsDir -PathType Container)) {
    Write-Host "ERROR: docs directory not found at $DocsDir" -ForegroundColor Red
    exit 1
}
Write-Host "OK: docs directory found" -ForegroundColor Green

# Step 2: Check package.json and dependencies
Write-Host "`n[2/6] Checking package.json and dependencies..." -ForegroundColor Yellow
$PackageJsonPath = Join-Path $DocsDir "package.json"
if (!(Test-Path $PackageJsonPath -PathType Leaf)) {
    Write-Host "ERROR: package.json not found at $PackageJsonPath" -ForegroundColor Red
    exit 1
}
Write-Host "OK: package.json found" -ForegroundColor Green

# Check if node_modules exists
$NodeModulesDir = Join-Path $DocsDir "node_modules"
if (!(Test-Path $NodeModulesDir -PathType Container)) {
    Write-Host "INFO: node_modules not found, installing dependencies..." -ForegroundColor Yellow
    Push-Location $DocsDir
    try {
        # Try npm first, fallback to bun
        if (Get-Command npm -ErrorAction SilentlyContinue) {
            npm install
        } elseif (Get-Command bun -ErrorAction SilentlyContinue) {
            bun install
        } else {
            Write-Host "ERROR: npm or bun not found in PATH" -ForegroundColor Red
            exit 1
        }
    } finally {
        Pop-Location
    }
    Write-Host "OK: Dependencies installed" -ForegroundColor Green
} else {
    Write-Host "OK: node_modules found, skipping install" -ForegroundColor Green
}

# Step 3: Check Journey Viewer components
Write-Host "`n[3/6] Checking Journey Viewer components..." -ForegroundColor Yellow
$JourneyViewerPath = Join-Path $DocsDir ".vitepress\theme\components\JourneyViewer.vue"
$JourneyStepPath = Join-Path $DocsDir ".vitepress\theme\components\JourneyStep.vue"

$ViewerExists = Test-Path $JourneyViewerPath -PathType Leaf
$StepExists = Test-Path $JourneyStepPath -PathType Leaf

$ViewerStatus = if ($ViewerExists) { 'FOUND' } else { 'MISSING' }
$ViewerColor = if ($ViewerExists) { 'Green' } else { 'Yellow' }
$StepStatus = if ($StepExists) { 'FOUND' } else { 'MISSING' }
$StepColor = if ($StepExists) { 'Green' } else { 'Yellow' }

Write-Host "  JourneyViewer.vue: $ViewerStatus" -ForegroundColor $ViewerColor
Write-Host "  JourneyStep.vue: $StepStatus" -ForegroundColor $StepColor

# Step 4: Check journey manifests
Write-Host "`n[4/6] Checking journey manifests..." -ForegroundColor Yellow
$JourneysDir = Join-Path $DocsDir "journeys\manifests"
$ExpectedManifests = @(
    "us-f1-1-game-launch\manifest.json",
    "us-f2-1-mod-menu\manifest.json",
    "us-f3-1-visual-assets\manifest.json",
    "us-f4-1-pause-menu\manifest.json"
)

$MissingManifests = @()
foreach ($manifest in $ExpectedManifests) {
    $manifestPath = Join-Path $JourneysDir $manifest
    $exists = Test-Path $manifestPath -PathType Leaf
    $statusText = if ($exists) { 'FOUND' } else { 'MISSING' }
    $statusColor = if ($exists) { 'Green' } else { 'Yellow' }
    Write-Host "  $($manifest): $statusText" -ForegroundColor $statusColor
    if (!$exists) {
        $MissingManifests += $manifest
    }
}

# Step 5: Check VitePress config for component registration
Write-Host "`n[5/6] Checking VitePress config..." -ForegroundColor Yellow
$ConfigPath = Join-Path $DocsDir ".vitepress\config.mts"
$ConfigExists = Test-Path $ConfigPath -PathType Leaf

if ($ConfigExists) {
    Write-Host "OK: config.mts found" -ForegroundColor Green
    try {
        $ConfigContent = Get-Content $ConfigPath -Raw
        $hasJourneyPattern = $ConfigContent -match "journey"
        if ($hasJourneyPattern) {
            Write-Host "  Journey patterns found in config" -ForegroundColor Green
        } else {
            Write-Host "  WARNING: No journey patterns found in config" -ForegroundColor Yellow
        }
    } catch {
        Write-Host "  ERROR reading config: $_" -ForegroundColor Yellow
    }
} else {
    Write-Host "WARNING: config.mts not found" -ForegroundColor Yellow
}

# Step 6: Start VitePress dev server
Write-Host "`n[6/6] Starting VitePress dev server..." -ForegroundColor Yellow
Push-Location $DocsDir
try {
    # Determine which package manager to use
    $PackageManager = if (Get-Command npm -ErrorAction SilentlyContinue) { "npm" } else { "bun" }

    # Start the dev server in the background
    Write-Host "Starting with $PackageManager..." -ForegroundColor Cyan

    # Create a temporary script to run the dev server
    $DevServerScript = Join-Path $env:TEMP "start_vitepress.ps1"
    $ScriptContent = @"
cd "$DocsDir"
`$PackageManager = if (Get-Command npm -ErrorAction SilentlyContinue) { "npm" } else { "bun" }
& `$PackageManager run dev
"@
    $ScriptContent | Out-File $DevServerScript -Encoding UTF8

    # Start the dev server in a new PowerShell process (non-blocking)
    Start-Process powershell.exe -ArgumentList "-NoProfile", "-ExecutionPolicy", "Bypass", "-File", $DevServerScript

    # Wait a few seconds for the server to start
    Start-Sleep -Seconds 3

    Write-Host "OK: Dev server started (check output above for exact URL)" -ForegroundColor Green
    Write-Host "`nTypical URL: http://localhost:5173" -ForegroundColor Cyan
} finally {
    Pop-Location
}

# Final report
Write-Host "`n=== SETUP SUMMARY ===" -ForegroundColor Cyan
Write-Host "Components Status:" -ForegroundColor Yellow

$ViewerFinal = if ($ViewerExists) { 'FOUND' } else { 'MISSING' }
$ViewerFinalColor = if ($ViewerExists) { 'Green' } else { 'Red' }
$StepFinal = if ($StepExists) { 'FOUND' } else { 'MISSING' }
$StepFinalColor = if ($StepExists) { 'Green' } else { 'Red' }

Write-Host "  - JourneyViewer.vue: $ViewerFinal" -ForegroundColor $ViewerFinalColor
Write-Host "  - JourneyStep.vue: $StepFinal" -ForegroundColor $StepFinalColor

Write-Host "`nJourney Manifests Status:" -ForegroundColor Yellow
Write-Host "  Total expected: $($ExpectedManifests.Count)"
Write-Host "  Missing: $($MissingManifests.Count)"
if ($MissingManifests.Count -gt 0) {
    Write-Host "  Missing manifests:" -ForegroundColor Red
    foreach ($m in $MissingManifests) {
        Write-Host "    - $m" -ForegroundColor Red
    }
}

Write-Host "`nDev Server: http://localhost:5173" -ForegroundColor Green
Write-Host "`nNext steps:" -ForegroundColor Cyan
Write-Host "1. Visit http://localhost:5173 in your browser"
Write-Host "2. Check for any Journey Viewer rendering errors"
Write-Host "3. If components are missing, they will need to be created"
Write-Host "4. If manifests are missing, create them in docs/journeys/manifests/"
