<#
.SYNOPSIS
Single-command release readiness probe for v0.25.0.

.DESCRIPTION
Probes git state, VERSION file, deployed DLL, latest test results, key pattern
detectors, and NuGet artifacts. Each section wrapped in try/catch so partial
failures don't kill the rest. Mirrors style of health-summary.ps1 and
git-state-probe.ps1.

.PARAMETER Json
Output as compact JSON when $true. Default $false = human-readable.

.EXAMPLE
pwsh scripts/diag/release-readiness-check.ps1
pwsh scripts/diag/release-readiness-check.ps1 -Json $true
#>
[CmdletBinding()]
param([bool]$Json = $false)

$ErrorActionPreference = 'SilentlyContinue'
$RepoRoot     = 'C:\Users\koosh\Dino'
$GamePath     = 'G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option'
$DeployedDll  = "$GamePath\BepInEx\plugins\DINOForge.Runtime.dll"
$VersionFile  = "$RepoRoot\VERSION"
$TestResults  = "$RepoRoot\docs\test-results"
$ArtifactsDir = "$RepoRoot\artifacts"

$H = [ordered]@{
    timestamp_utc = (Get-Date).ToUniversalTime().ToString('o')
    git=@{}; version=@{}; deploy=@{}; tests=@{}; detectors=@{}; nuget=@{}
}
$AnyFail = $false; $AnyWarn = $false

# [GIT]
try {
    $H.git.repo_root   = (git -C $RepoRoot rev-parse --show-toplevel 2>$null)
    $H.git.branch      = (git -C $RepoRoot branch --show-current 2>$null)
    $H.git.head        = (git -C $RepoRoot rev-parse --short HEAD 2>$null)
    $H.git.dirty_count = ((git -C $RepoRoot status --porcelain 2>$null | Measure-Object).Count)
    $H.git.clean       = ($H.git.dirty_count -eq 0)
    $base = git -C $RepoRoot merge-base HEAD main 2>$null
    if ($base) {
        $H.git.ahead_main  = ((git -C $RepoRoot rev-list "$base..HEAD" 2>$null | Measure-Object).Count)
        $H.git.behind_main = ((git -C $RepoRoot rev-list "HEAD..main" 2>$null | Measure-Object).Count)
    }
    $H.git.status = if ($H.git.branch) { 'OK' } else { 'FAIL' }
    if ($H.git.status -eq 'FAIL') { $AnyFail = $true }
} catch { $H.git.status='FAIL'; $H.git.error=$_.Exception.Message; $AnyFail=$true }

# [VERSION]
try {
    if (Test-Path $VersionFile) {
        $raw = (Get-Content $VersionFile -Raw).Trim()
        $H.version.raw = $raw
        $H.version.matches_expected = ($raw -match '^0\.25\.0(-dev|-rc[0-9]+)?$' -or $raw -match '^0\.25\.[0-9]+$')
        $H.version.status = if ($H.version.matches_expected) { 'OK' } else { 'WARN' }
        if ($H.version.status -eq 'WARN') { $AnyWarn = $true }
    } else {
        $H.version.status = 'FAIL'; $H.version.error = 'VERSION file missing'; $AnyFail = $true
    }
} catch { $H.version.status='FAIL'; $H.version.error=$_.Exception.Message; $AnyFail=$true }

# [DEPLOY]
try {
    if (Test-Path $DeployedDll) {
        $info = Get-Item $DeployedDll
        $H.deploy.present = $true
        $H.deploy.mtime   = $info.LastWriteTime.ToString('o')
        $H.deploy.age_min = [math]::Round(((Get-Date) - $info.LastWriteTime).TotalMinutes, 1)
        $hash = (Get-FileHash $DeployedDll -Algorithm SHA256).Hash
        $H.deploy.sha8    = $hash.Substring(0, 8)
        $H.deploy.status  = 'OK'
    } else {
        $H.deploy.present=$false; $H.deploy.status='WARN'; $AnyWarn=$true
    }
} catch { $H.deploy.status='FAIL'; $H.deploy.error=$_.Exception.Message; $AnyFail=$true }

# [TESTS]
try {
    if (Test-Path $TestResults) {
        $latest = Get-ChildItem -Path $TestResults -Filter *.json -ErrorAction SilentlyContinue |
                  Sort-Object LastWriteTime -Descending | Select-Object -First 1
        if ($latest) {
            $H.tests.file = $latest.Name
            $H.tests.mtime = $latest.LastWriteTime.ToString('o')
            $obj = Get-Content $latest.FullName -Raw | ConvertFrom-Json
            $passed = $null; $failed = $null; $skipped = $null
            if ($obj.summary) {
                $passed = $obj.summary.passed; $failed = $obj.summary.failed; $skipped = $obj.summary.skipped
            } elseif ($obj.tests) {
                $passed = $obj.tests.passed; $failed = $obj.tests.failed; $skipped = $obj.tests.skipped
            } elseif ($null -ne $obj.passed) {
                $passed = $obj.passed; $failed = $obj.failed; $skipped = $obj.skipped
            }
            $H.tests.passed = $passed; $H.tests.failed = $failed; $H.tests.skipped = $skipped
            if ($null -ne $failed -and [int]$failed -gt 0) { $H.tests.status='FAIL'; $AnyFail=$true }
            else { $H.tests.status='OK' }
        } else {
            $H.tests.status = 'WARN'; $H.tests.note = 'NO_TEST_REPORT'; $AnyWarn = $true
        }
    } else {
        $H.tests.status = 'WARN'; $H.tests.note = 'NO_TEST_REPORT'; $AnyWarn = $true
    }
} catch { $H.tests.status='FAIL'; $H.tests.error=$_.Exception.Message; $AnyFail=$true }

# [DETECTORS]
$detectorNames = @(
    'scripts/ci/detect_logerror_no_stack.py',
    'scripts/ci/detect_silent_catch.py',
    'scripts/ci/detect_test_pack_leak.py'
)
$raycaster = Get-ChildItem -Path "$RepoRoot\scripts\ci" -Filter 'detect_graphicraycaster*.py' -ErrorAction SilentlyContinue | Select-Object -First 1
if ($raycaster) { $detectorNames += "scripts/ci/$($raycaster.Name)" }

foreach ($d in $detectorNames) {
    $full = Join-Path $RepoRoot $d
    $name = [IO.Path]::GetFileNameWithoutExtension($d)
    try {
        if (-not (Test-Path $full)) { $H.detectors[$name] = 'N/A'; continue }
        $out = & python $full 2>&1 | Out-String
        $high = $null
        $m1 = [regex]::Match($out, '(?im)HIGH:\s*(\d+)')
        if ($m1.Success) { $high = [int]$m1.Groups[1].Value }
        if ($null -eq $high) {
            $m2 = [regex]::Match($out, '(?im)HIGH\s+violations?:\s*(\d+)')
            if ($m2.Success) { $high = [int]$m2.Groups[1].Value }
        }
        if ($null -eq $high) {
            $m3 = [regex]::Match($out, '(?im)(\d+)\s+HIGH\b')
            if ($m3.Success) { $high = [int]$m3.Groups[1].Value }
        }
        if ($null -eq $high) { $high = 0 }
        $H.detectors[$name] = $high
    } catch { $H.detectors[$name] = 'FAIL'; $AnyFail=$true }
}

# [NUGET]
try {
    if (Test-Path $ArtifactsDir) {
        $pkgs = Get-ChildItem -Path $ArtifactsDir -Filter *.nupkg -ErrorAction SilentlyContinue
        $H.nuget.count = $pkgs.Count
        $H.nuget.files = @($pkgs | ForEach-Object { $_.Name })
        $H.nuget.status = if ($pkgs.Count -gt 0) { 'OK' } else { 'WARN' }
        if ($H.nuget.status -eq 'WARN') { $AnyWarn = $true }
    } else {
        $H.nuget.status = 'WARN'; $H.nuget.note = 'NO_ARTIFACTS'; $AnyWarn = $true
    }
} catch { $H.nuget.status='FAIL'; $H.nuget.error=$_.Exception.Message; $AnyFail=$true }

# Verdict
$verdict = 'GREEN'
if ($AnyFail) { $verdict = 'RED' }
elseif ($AnyWarn) { $verdict = 'YELLOW' }
$H.verdict = $verdict

function Get-Icon($s) { switch ($s) { 'OK' {'[OK]'} 'WARN' {'[WARN]'} 'FAIL' {'[FAIL]'} default {"[$s]"} } }

if ($Json) {
    $H | ConvertTo-Json -Depth 4 -Compress
} else {
    Write-Host "=== DINOForge Release Readiness (v0.25.0) ===" -ForegroundColor Cyan
    Write-Host "$(Get-Icon $H.git.status)   [GIT]       branch=$($H.git.branch) head=$($H.git.head) clean=$($H.git.clean) ahead/behind=$($H.git.ahead_main)/$($H.git.behind_main)"
    Write-Host "$(Get-Icon $H.version.status)   [VERSION]   raw='$($H.version.raw)' expected=v0.25.x"
    if ($H.deploy.present) {
        Write-Host "$(Get-Icon $H.deploy.status)   [DEPLOY]    sha8=$($H.deploy.sha8) age=$($H.deploy.age_min)min mtime=$($H.deploy.mtime)"
    } else {
        Write-Host "$(Get-Icon $H.deploy.status) [DEPLOY]    DLL not present at $DeployedDll"
    }
    if ($H.tests.note -eq 'NO_TEST_REPORT') {
        Write-Host "$(Get-Icon $H.tests.status) [TESTS]     NO_TEST_REPORT"
    } else {
        Write-Host "$(Get-Icon $H.tests.status)   [TESTS]     passed=$($H.tests.passed) failed=$($H.tests.failed) skipped=$($H.tests.skipped) file=$($H.tests.file)"
    }
    Write-Host "     [DETECTORS]"
    foreach ($k in $H.detectors.Keys) { Write-Host "              $k HIGH=$($H.detectors[$k])" }
    if ($H.nuget.note -eq 'NO_ARTIFACTS') {
        Write-Host "$(Get-Icon $H.nuget.status) [NUGET]     NO_ARTIFACTS"
    } else {
        Write-Host "$(Get-Icon $H.nuget.status)   [NUGET]     count=$($H.nuget.count) files=$(($H.nuget.files -join ', '))"
    }
    $color = switch ($verdict) { 'GREEN' { 'Green' } 'YELLOW' { 'Yellow' } 'RED' { 'Red' } }
    Write-Host "VERDICT: $verdict" -ForegroundColor $color
}
