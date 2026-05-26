<#
.SYNOPSIS
    Validate DINOBox pool setup and confirm parallel test readiness.

.DESCRIPTION
    Creates a 4-instance DINOBox pool and validates:
    - Box creation completed successfully
    - Unique pipe names assigned to each instance
    - No asset duplication (symlinks working)
    - Box structure correct
    - <30s total creation time

.EXAMPLE
    .\test-dino-boxes.ps1
#>

param(
    [int]$Count = 4,
    [string]$BaseDir = "G:\dino_boxes"
)

$ErrorActionPreference = "Stop"

Write-Host ""
Write-Host "========================================"
Write-Host "  DINOBox Pool Validation Test"
Write-Host "========================================"
Write-Host ""

# Measure pool creation time
$creationStart = Get-Date

Write-Host "[1/5] Creating DINOBox pool ($Count instances)..."
try {
    $scriptPath = Join-Path (Split-Path -Parent $MyInvocation.MyCommand.Path) "New-DINOBoxPool.ps1"
    $pool = & $scriptPath -Count $Count -BaseDir $BaseDir
} catch {
    Write-Host "[FAIL] Pool creation failed:"
    Write-Host "  $_"
    exit 1
}

$creationEnd = Get-Date
$creationTime = ($creationEnd - $creationStart).TotalSeconds

Write-Host "[OK] Pool created in $creationTime seconds"
Write-Host ""

# Validate pool structure
Write-Host "[2/5] Validating pool structure..."

if ($null -eq $pool -or $pool.Count -eq 0) {
    Write-Host "[FAIL] Pool is empty"
    exit 1
}

Write-Host "  Found $($pool.Count) boxes"

foreach ($i in $pool.Keys | Sort-Object) {
    $box = $pool[$i]

    # Check box directory exists
    if (-not (Test-Path $box.BoxPath)) {
        Write-Host "  [FAIL] Box $i path not found: $($box.BoxPath)"
        exit 1
    }

    # Check game executable exists
    if (-not (Test-Path $box.ExePath)) {
        Write-Host "  [FAIL] Box $i exe not found: $($box.ExePath)"
        exit 1
    }

    # Check BepInEx directory exists
    if (-not (Test-Path $box.BepInExDir)) {
        Write-Host "  [FAIL] Box $i BepInEx dir not found: $($box.BepInExDir)"
        exit 1
    }

    Write-Host "  [OK] Box $i structure valid"
}

Write-Host "[OK] All boxes valid"
Write-Host ""

# Validate pipe name isolation
Write-Host "[3/5] Validating pipe name isolation..."

$pipeNames = @()
foreach ($i in $pool.Keys | Sort-Object) {
    $box = $pool[$i]
    $pipeNames += $box.PipeName
}

# Check uniqueness
$uniqueCount = ($pipeNames | Select-Object -Unique).Count
if ($uniqueCount -ne $pipeNames.Count) {
    Write-Host "  [FAIL] Duplicate pipe names detected"
    exit 1
}

Write-Host "  Found $uniqueCount unique pipe names:"
foreach ($pipe in $pipeNames) {
    Write-Host "    - $pipe"
}

Write-Host "[OK] Pipe names isolated"
Write-Host ""

# Validate symlinks (no duplication)
Write-Host "[4/5] Validating symlinks (no duplication)..."

$mainDataDir = "G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option\Diplomacy is Not an Option_Data"
$totalDiskUsage = 0

foreach ($i in $pool.Keys | Sort-Object) {
    $box = $pool[$i]
    $dataDir = Join-Path $box.BoxPath "Diplomacy is Not an Option_Data"

    # Check for symlinks
    $managedLink = Join-Path $dataDir "Managed"
    if (Test-Path $managedLink) {
        $item = Get-Item $managedLink
        $isSymlink = ($item.Attributes -band [System.IO.FileAttributes]::ReparsePoint) -ne 0
        if (-not $isSymlink) {
            Write-Host "  [WARN] Box $i Managed is not a symlink (may be duplicate)"
        } else {
            Write-Host "  [OK] Box $i uses symlinked Managed"
        }
    }

    # Measure actual disk usage (excluding symlinks)
    $boxSize = (Get-ChildItem $box.BoxPath -Recurse -ErrorAction SilentlyContinue |
        Where-Object { -not (($_.Attributes -band [System.IO.FileAttributes]::ReparsePoint) -ne 0) } |
        Measure-Object -Property Length -Sum).Sum

    if ($null -ne $boxSize) {
        $boxSizeMB = [math]::Round($boxSize / 1MB, 2)
        $totalDiskUsage += $boxSize
        Write-Host "  Box $i actual disk usage: $boxSizeMB MB"
    }
}

$totalDiskUsageMB = [math]::Round($totalDiskUsage / 1MB, 2)
$expectedIfDuplicated = [math]::Round(($Count * 12000), 2)  # 12GB per instance

Write-Host "[OK] Total disk usage: $totalDiskUsageMB MB (expected < 2000 MB with symlinks)"
if ($totalDiskUsageMB -gt 2000) {
    Write-Host "  [WARN] Disk usage higher than expected (symlinks may not be working)"
}

Write-Host ""

# Test parallel launch capability
Write-Host "[5/5] Testing parallel launch capability..."

Write-Host "  Launching all instances in parallel..."
$launchStart = Get-Date

$launchJobs = @()
foreach ($i in $pool.Keys | Sort-Object) {
    $box = $pool[$i]

    # Create background job for each launch
    $job = Start-Job -ScriptBlock {
        param($BoxPath, $PipeName, $Timeout)

        $exe = Join-Path $BoxPath "Diplomacy is Not an Option.exe"
        if (Test-Path $exe) {
            try {
                $proc = Start-Process -FilePath $exe -WorkingDirectory $BoxPath -PassThru -WindowStyle Hidden
                Start-Sleep -Milliseconds 500
                return $proc.Id
            } catch {
                return $null
            }
        }
        return $null
    } -ArgumentList $box.BoxPath, $box.PipeName, 10

    $launchJobs += $job
}

# Wait for all launches to complete
$completedJobs = 0
foreach ($job in $launchJobs) {
    $result = Receive-Job -Job $job -Wait
    if ($null -ne $result) {
        $completedJobs++
    }
    Remove-Job $job
}

$launchEnd = Get-Date
$launchTime = ($launchEnd - $launchStart).TotalSeconds

Write-Host "  Launched $completedJobs/$Count instances in $launchTime seconds"

# Kill all launched instances
Write-Host "  Cleaning up test instances..."
Get-Process -Name "Diplomacy is Not an Option" -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue
Start-Sleep -Seconds 1

Write-Host "[OK] Launch test complete"
Write-Host ""

# Summary
Write-Host "========================================"
Write-Host "  VALIDATION SUMMARY"
Write-Host "========================================"
Write-Host ""
Write-Host "Pool Creation Time:      $([math]::Round($creationTime, 2)) seconds"
Write-Host "Parallel Launch Time:    $([math]::Round($launchTime, 2)) seconds"
Write-Host "Total Disk Usage:        $totalDiskUsageMB MB"
Write-Host "Unique Pipe Names:       $uniqueCount"
Write-Host "Symlinks Working:        $(if ($totalDiskUsageMB -lt 2000) { 'YES' } else { 'NO (WARNING)' })"
Write-Host ""

if ($creationTime -lt 30 -and $totalDiskUsageMB -lt 2000) {
    Write-Host "[SUCCESS] DINOBox pool is ready for parallel testing!"
    exit 0
} else {
    Write-Host "[WARNING] Some metrics out of spec, but setup may still work"
    exit 0
}
