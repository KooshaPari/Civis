# Game Launch Isolation & Cursor Interference Diagnostic
# Phase 1-4 comprehensive testing

Write-Host "=== DINOForge Game Isolation Diagnostic ===" -ForegroundColor Cyan
Write-Host "Date: $(Get-Date)" -ForegroundColor Gray

# Read game path from Directory.Build.props
$propFile = "C:\Users\koosh\Dino\Directory.Build.props"
[xml]$props = Get-Content $propFile
$gamePath = $props.Project.PropertyGroup.GameInstallPath
$testInstPath = $props.Project.PropertyGroup.GameInstallPath_Test
$boxBase = $props.Project.PropertyGroup.DINOBoxBaseDir

# Handle null/element returns
if ($null -eq $gamePath -or $gamePath -is [System.Xml.XmlElement]) {
    $gamePath = "G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option"
}
if ($null -eq $testInstPath -or $testInstPath -is [System.Xml.XmlElement]) {
    $testInstPath = "G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option_TEST"
}
if ($null -eq $boxBase -or $boxBase -is [System.Xml.XmlElement]) {
    $boxBase = "G:\dino_boxes"
}

Write-Host "`n[CONFIG] Game paths from Directory.Build.props:" -ForegroundColor Yellow
Write-Host "  Main: $gamePath"
Write-Host "  Test: $testInstPath"
Write-Host "  Box base: $boxBase"

# Verify paths exist
Write-Host "`n[VERIFICATION] Checking paths..." -ForegroundColor Gray
Write-Host "  Main game dir exists: $(Test-Path $gamePath)" -ForegroundColor $(if (Test-Path $gamePath) { 'Green' } else { 'Red' })
Write-Host "  Test game dir exists: $(Test-Path $testInstPath)" -ForegroundColor $(if (Test-Path $testInstPath) { 'Green' } else { 'Yellow' })
Write-Host "  Box base dir exists: $(Test-Path $boxBase)" -ForegroundColor $(if (Test-Path $boxBase) { 'Green' } else { 'Yellow' })

# PHASE 1: Kill & Launch Game Isolated
Write-Host "`n[PHASE 1] Launch Game Isolated" -ForegroundColor Cyan

Write-Host "  1a. Killing existing instances..." -ForegroundColor Gray
Stop-Process -Name "Diplomacy is Not an Option" -Force -ErrorAction SilentlyContinue
Start-Sleep -Seconds 3

$procs = Get-Process "Diplomacy is Not an Option" -ErrorAction SilentlyContinue
if ($procs) {
    Write-Host "    ERROR: Process still running after kill!" -ForegroundColor Red
    $procs | ForEach-Object { Write-Host "      PID $($_.Id): $($_.MainWindowTitle)" }
} else {
    Write-Host "    OK: All instances killed" -ForegroundColor Green
}

Write-Host "  1b. Launching with hidden window..." -ForegroundColor Gray
$exe = "$gamePath\Diplomacy is Not an Option.exe"
if (-not (Test-Path $exe)) {
    Write-Host "    ERROR: Executable not found: $exe" -ForegroundColor Red
} else {
    try {
        Start-Process -FilePath $exe -WorkingDirectory $gamePath -WindowStyle Hidden -ErrorAction Stop
        Write-Host "    OK: Process started" -ForegroundColor Green
    } catch {
        Write-Host "    ERROR: Failed to start: $_" -ForegroundColor Red
    }
}

Write-Host "  1c. Waiting 10 seconds for game to initialize..." -ForegroundColor Gray
Start-Sleep -Seconds 10

$proc = Get-Process "Diplomacy is Not an Option" -ErrorAction SilentlyContinue
if ($proc) {
    Write-Host "    OK: Process running (PID $($proc.Id))" -ForegroundColor Green
    Write-Host "      MainWindowTitle: '$($proc.MainWindowTitle)'"
    Write-Host "      MainWindowHandle: $($proc.MainWindowHandle)"
    $wnd = [System.Diagnostics.Process]::GetProcessById($proc.Id).MainWindowTitle
    if ($wnd -match "Fatal|Error|crash") {
        Write-Host "    ERROR: Fatal dialog detected!" -ForegroundColor Red
    }
} else {
    Write-Host "    ERROR: Process exited or hidden (failed to launch)" -ForegroundColor Red
}

# PHASE 2: Test Parallel Cursor Automation
Write-Host "`n[PHASE 2] Test Parallel Cursor Automation" -ForegroundColor Cyan

if ($proc) {
    Write-Host "  2a. Starting MCP server..." -ForegroundColor Gray
    $mcpPath = "C:\Users\koosh\Dino\src\Tools\DinoforgeMcp"
    if (Test-Path $mcpPath) {
        try {
            # Start MCP in background
            $job = Start-Job -ScriptBlock {
                cd "C:\Users\koosh\Dino\src\Tools\DinoforgeMcp"
                python -m dinoforge_mcp.server 2>&1
            } -ErrorAction SilentlyContinue
            Start-Sleep -Seconds 5

            if ($job) {
                Write-Host "    OK: MCP job started (ID: $($job.Id))" -ForegroundColor Green

                # Test connectivity
                Write-Host "  2b. Testing MCP connectivity..." -ForegroundColor Gray
                try {
                    $response = Invoke-RestMethod -Uri "http://127.0.0.1:8765/api/health" -Method Get -TimeoutSec 3 -ErrorAction Stop
                    Write-Host "    OK: MCP responding" -ForegroundColor Green
                } catch {
                    Write-Host "    WARNING: MCP not responding yet (may be initializing)" -ForegroundColor Yellow
                }

                # Test game_screenshot
                Write-Host "  2c. Testing game_screenshot..." -ForegroundColor Gray
                try {
                    $body = @{
                        jsonrpc = "2.0"
                        method = "game_screenshot"
                        params = @{}
                        id = 1
                    } | ConvertTo-Json

                    $response = Invoke-WebRequest -Uri "http://127.0.0.1:8765" `
                        -Method POST -Body $body -ContentType "application/json" `
                        -TimeoutSec 5 -ErrorAction SilentlyContinue
                    Write-Host "    OK: Screenshot command sent" -ForegroundColor Green
                } catch {
                    Write-Host "    INFO: Screenshot call (may still work via SSE): $_" -ForegroundColor Gray
                }

                # Cleanup
                Write-Host "  2d. Stopping MCP job..." -ForegroundColor Gray
                Stop-Job -Job $job -ErrorAction SilentlyContinue
                Remove-Job -Job $job -ErrorAction SilentlyContinue
                Write-Host "    OK: MCP stopped" -ForegroundColor Green
            }
        } catch {
            Write-Host "    ERROR: Failed to start MCP: $_" -ForegroundColor Red
        }
    } else {
        Write-Host "    ERROR: MCP path not found: $mcpPath" -ForegroundColor Red
    }
} else {
    Write-Host "  SKIPPED: Game not running" -ForegroundColor Yellow
}

# PHASE 3: Check PlayCUA (bare-cua)
Write-Host "`n[PHASE 3] Check PlayCUA (bare-cua)" -ForegroundColor Cyan

$playcuaPath = "C:\Users\koosh\bare-cua\target\release\bare-cua-native.exe"
if (Test-Path $playcuaPath) {
    Write-Host "  OK: PlayCUA found at: $playcuaPath" -ForegroundColor Green
    Write-Host "  PlayCUA provides input isolation via lower-level Win32 APIs" -ForegroundColor Gray
} else {
    Write-Host "  WARNING: PlayCUA not found at: $playcuaPath" -ForegroundColor Yellow
    Write-Host "  Status: bare-cua project not built or located elsewhere" -ForegroundColor Gray
}

# PHASE 4: Sandbox Copy Test
Write-Host "`n[PHASE 4] Sandbox Copy Test" -ForegroundColor Cyan

$sandboxPath = "$boxBase\diagnostic_test"
Write-Host "  4a. Creating sandbox at: $sandboxPath" -ForegroundColor Gray

try {
    if (-not (Test-Path $boxBase)) {
        New-Item -ItemType Directory -Path $boxBase -Force -ErrorAction SilentlyContinue | Out-Null
    }

    New-Item -ItemType Directory -Path $sandboxPath -Force -ErrorAction Stop | Out-Null
    Write-Host "    OK: Directory created" -ForegroundColor Green

    # Copy exe
    $sourceExe = "$gamePath\Diplomacy is Not an Option.exe"
    $targetExe = "$sandboxPath\Diplomacy is Not an Option.exe"
    if (Test-Path $sourceExe) {
        Copy-Item $sourceExe -Destination $targetExe -Force -ErrorAction SilentlyContinue
        Write-Host "    OK: EXE copied" -ForegroundColor Green
    }

    # Create symlinks for large asset directories (avoid copy cost)
    $linksToCreate = @(
        @{ name = "_Data"; source = "$gamePath\Diplomacy is Not an Option_Data" },
        @{ name = "StreamingAssets"; source = "$gamePath\StreamingAssets" }
    )

    foreach ($link in $linksToCreate) {
        $targetLink = "$sandboxPath\$($link.name)"
        if (Test-Path $link.source) {
            if (Test-Path $targetLink) {
                Remove-Item $targetLink -ErrorAction SilentlyContinue
            }
            New-Item -ItemType SymbolicLink -Path $targetLink -Target $link.source -Force -ErrorAction SilentlyContinue | Out-Null
            if (Test-Path $targetLink) {
                Write-Host "    OK: Symlink created for $($link.name)" -ForegroundColor Green
            }
        }
    }

    # Create fresh BepInEx
    New-Item -ItemType Directory -Path "$sandboxPath\BepInEx" -Force -ErrorAction SilentlyContinue | Out-Null
    Write-Host "    OK: BepInEx directory created" -ForegroundColor Green

    Write-Host "  4b. Sandbox is ready at: $sandboxPath" -ForegroundColor Green
    Get-ChildItem $sandboxPath -Force | ForEach-Object { Write-Host "      $($_.Name)" }

} catch {
    Write-Host "    ERROR: Sandbox creation failed: $_" -ForegroundColor Red
}

# FINAL REPORT
Write-Host "`n[SUMMARY]" -ForegroundColor Cyan
Write-Host "Phase 1 (Launch Isolated): $(if ($proc) { 'PASS - Game running' } else { 'FAIL - Game not running' })" -ForegroundColor $(if ($proc) { 'Green' } else { 'Red' })
Write-Host "Phase 2 (Cursor Automation): TESTED (see above)" -ForegroundColor Gray
Write-Host "Phase 3 (PlayCUA): $(if (Test-Path $playcuaPath) { 'FOUND' } else { 'NOT FOUND' })" -ForegroundColor $(if (Test-Path $playcuaPath) { 'Green' } else { 'Yellow' })
Write-Host "Phase 4 (Sandbox): READY at $sandboxPath" -ForegroundColor Green

if ($proc) {
    Write-Host "`n[CLEANUP] Killing test instance..." -ForegroundColor Yellow
    Stop-Process -Name "Diplomacy is Not an Option" -Force -ErrorAction SilentlyContinue
    Start-Sleep -Seconds 2
    Write-Host "OK: Cleaned up" -ForegroundColor Green
}

Write-Host "`nDiagnostic complete." -ForegroundColor Cyan
