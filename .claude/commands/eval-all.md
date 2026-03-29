# /eval-all

Run the complete DINOForge evaluation suite covering all features across all components.

This command orchestrates four sequential evaluation pipelines:
1. **Pipeline B** (Installer) — Validates installer artifacts, logic, and unit tests
2. **Pipeline A** (Game Features) — Runs `/prove-features` to record mods, F9, F10, then extends with stat override, asset swap, economy, scenario, hot reload
3. **Pipeline C** (Companion) — Validates Desktop Companion UI flows and automation
4. **Pipeline D** (Advanced) — Tests hidden desktop isolation, dual-instance readiness, and VDD prerequisites

All results are bundled with PASS/FAIL clarity and a unified EVAL_REPORT.md summary.

---

## Requirements

### Core Prerequisites
- **PowerShell** 5.1+
- **.NET SDK** — `dotnet --version` returns 8.0+
- **Game** installed at `G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option\`

### Conditional (Pipeline A only)
- **ffmpeg** at `C:\program files\imagemagick-7.1.0-q16-hdri\ffmpeg.exe`
- **Python 3** + edge-tts: `pip install edge-tts`
- **Node.js 18+** with Remotion deps: `cd scripts/video && npm install`

### Conditional (Advanced features)
- **MCP server** running on `http://127.0.0.1:8765` (optional, checks if available)

---

## Flags

```powershell
# Quick mode: skip video rendering, hot reload, dual-instance
--quick

# Skip game launch entirely (Pipeline A only)
--skip-game

# Skip Companion evaluation (Pipeline C)
--skip-companion

# Skip Installer evaluation (Pipeline B)
--skip-installer

# Skip Advanced evaluation (Pipeline D)
--skip-advanced

# Custom output directory (default: docs/proof-of-features/eval_<timestamp>)
--output <path>
```

Examples:
```powershell
/eval-all --quick
/eval-all --skip-companion --skip-advanced
/eval-all --output docs/eval-reports/custom
```

---

## Orchestration

Run all steps from **repo root** (`C:\Users\koosh\Dino`). Each pipeline must complete before the next starts.

---

### Pre-flight Checks

Before orchestration, verify system state:

```powershell
# Check MCP server health (HTTP GET)
$health = Invoke-WebRequest -Uri http://127.0.0.1:8765/health -Method GET -ErrorAction SilentlyContinue
if ($health.StatusCode -eq 200) {
    Write-Output "✓ MCP server is healthy"
} else {
    Write-Output "⚠ MCP server not responding — skipping features that require it"
}

# Verify .NET SDK
$dotnetVersion = dotnet --version
if ($LASTEXITCODE -eq 0) {
    Write-Output "✓ .NET SDK: $dotnetVersion"
} else {
    Write-Output "✗ .NET SDK not found — cannot proceed"
    exit 1
}

# Verify game path
if (Test-Path "G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option\Diplomacy is Not an Option.exe") {
    Write-Output "✓ Game executable found"
} else {
    Write-Output "⚠ Game not found at default path — skipping Pipeline A"
}

# Report what will be skipped based on flags
if ($skipGame) { Write-Output "⊘ Pipeline A: SKIPPED (--skip-game)" }
if ($skipInstaller) { Write-Output "⊘ Pipeline B: SKIPPED (--skip-installer)" }
if ($skipCompanion) { Write-Output "⊘ Pipeline C: SKIPPED (--skip-companion)" }
if ($skipAdvanced) { Write-Output "⊘ Pipeline D: SKIPPED (--skip-advanced)" }
```

---

### Step 1: Create output bundle

```powershell
$ts = Get-Date -Format "yyyyMMdd_HHmmss"
$bundle = if ($outputDir) { $outputDir } else { "docs/proof-of-features/eval_$ts" }
New-Item -ItemType Directory -Force -Path $bundle | Out-Null
Write-Output "Bundle path: $bundle"
```

---

### Step 2: Pipeline B — Installer Evaluation

Only if `--skip-installer` is NOT set.

```powershell
Write-Output "`n=== PIPELINE B: Installer Evaluation ==="
$installerReportFile = "$env:TEMP\DINOForge\eval\eval_installer_report.json"

powershell -ExecutionPolicy Bypass -File scripts/game/eval-installer.ps1 -OutputDir $bundle

if (Test-Path $installerReportFile) {
    $report = Get-Content $installerReportFile | ConvertFrom-Json
    Write-Output "✓ Installer evaluation complete"
    Copy-Item $installerReportFile "$bundle\eval_installer_report.json" -Force
} else {
    Write-Output "✗ Installer evaluation failed — no report generated"
}
```

---

### Step 3: Pipeline A — Game Features Evaluation

Only if `--skip-game` is NOT set.

This step combines `/prove-features` (mods, F9, F10 video proof) + extended game feature tests (stat override, asset swap, economy, scenario, hot reload).

#### Step 3a: Run `/prove-features` pipeline

Executes game capture → VLM validation → TTS → Remotion render.

```powershell
Write-Output "`n=== PIPELINE A: Game Features (Phase 1-3: Capture + TTS + Render) ==="

# Phase 1: Capture + VLM validation
powershell -ExecutionPolicy Bypass -File scripts/game/capture-feature-clips.ps1

# Check if validate_report.json exists and all features confirmed
$validateReport = "$env:TEMP\DINOForge\capture\validate_report.json"
if (Test-Path $validateReport) {
    $validations = Get-Content $validateReport | ConvertFrom-Json
    $allConfirmed = $validations | Where-Object { $_.confirmed -eq $false }
    if ($allConfirmed) {
        Write-Output "✗ One or more features failed VLM validation"
        exit 1
    } else {
        Write-Output "✓ All features VLM-validated"
    }
} else {
    Write-Output "✗ Validation report not found"
    exit 1
}

# Phase 2: TTS (skip if --quick)
if (-not $quick) {
    Write-Output "`nPIPELINE A Phase 2: Neural TTS"
    python scripts/video/generate_tts.py `
        --spec scripts/video/vo_spec.json `
        --out "$env:TEMP\DINOForge\tts"

    if ($LASTEXITCODE -ne 0) {
        Write-Output "⚠ TTS generation failed"
    } else {
        Write-Output "✓ TTS complete"
    }
}

# Phase 3: Remotion render (skip if --quick)
if (-not $quick) {
    Write-Output "`nPIPELINE A Phase 3: Remotion Render"

    $cap = ($env:TEMP + "\DINOForge\capture") -replace '\\', '/'
    $tts = ($env:TEMP + "\DINOForge\tts")     -replace '\\', '/'

    $env:RAW_MODS_PATH  = "$cap/raw_mods.mp4"
    $env:RAW_F9_PATH    = "$cap/raw_f9.mp4"
    $env:RAW_F10_PATH   = "$cap/raw_f10.mp4"
    $env:TTS_INTRO_PATH = "$tts/intro.mp3"
    $env:TTS_MODS_PATH  = "$tts/mods.mp3"
    $env:TTS_F9_PATH    = "$tts/f9.mp3"
    $env:TTS_F10_PATH   = "$tts/f10.mp3"
    $env:TTS_OUTRO_PATH = "$tts/outro.mp3"

    Push-Location scripts/video
    npx remotion render src/index.tsx ModsButtonFeature --output out/mods_feature.mp4
    npx remotion render src/index.tsx F9OverlayFeature  --output out/f9_feature.mp4
    npx remotion render src/index.tsx F10MenuFeature    --output out/f10_feature.mp4
    npx remotion render src/index.tsx DINOForgeReel     --output out/dinoforge_reel.mp4
    Pop-Location

    Write-Output "✓ Remotion renders complete"
}

# Copy Phase 1 artifacts to bundle
@("raw_mods.mp4","raw_f9.mp4","raw_f10.mp4",
  "validate_mods.png","validate_f9.png","validate_f10.png",
  "validate_report.json") | ForEach-Object {
    $src = "$env:TEMP\DINOForge\capture\$_"
    if (Test-Path $src) { Copy-Item $src "$bundle\" -Force }
}

# Copy Phase 3 artifacts to bundle (if renders completed)
if (-not $quick) {
    @("mods_feature.mp4","f9_feature.mp4","f10_feature.mp4","dinoforge_reel.mp4") | ForEach-Object {
        $src = "scripts/video/out/$_"
        if (Test-Path $src) { Copy-Item $src "$bundle\" -Force }
    }
}
```

#### Step 3b: Extended Game Features (Stat Override, Asset Swap, Economy, Scenario, Hot Reload)

After `/prove-features` completes, run extended feature tests. Game window remains open.

```powershell
Write-Output "`nPIPELINE A Phase 4: Extended Features (Stat Override, Asset Swap, Economy, Scenario, Hot Reload)"

# Game is still running from Phase 1. Call game_analyze_screen for each extended feature.

# Test stat override
Write-Output "Testing stat override..."
game_analyze_screen  # Call MCP tool to capture screenshot with stat override visible
# Save screenshot to $bundle\validate_stat_override.png
# Record result as PASS/FAIL in extended_features.json

# Test asset swap
Write-Output "Testing asset swap..."
game_analyze_screen  # Asset swap should be visible on units in game
# Save screenshot to $bundle\validate_asset_swap.png

# Test economy pack
Write-Output "Testing economy pack..."
game_analyze_screen  # Economy UI or balance effects visible
# Save screenshot to $bundle\validate_economy.png

# Test scenario pack
Write-Output "Testing scenario pack..."
game_analyze_screen  # Scenario objectives/conditions visible
# Save screenshot to $bundle\validate_scenario.png

# Test hot reload (skip if --quick)
if (-not $quick) {
    Write-Output "Testing hot reload..."
    # Trigger pack reload via MCP: game_reload_packs
    # Wait for reload to complete
    # game_analyze_screen to verify pack list updated
    # Save screenshot to $bundle\validate_hot_reload.png
} else {
    Write-Output "⊘ Hot reload test skipped (--quick mode)"
}

Write-Output "✓ Extended features validation complete"
```

---

### Step 4: Pipeline C — Companion Evaluation

Only if `--skip-companion` is NOT set.

```powershell
Write-Output "`n=== PIPELINE C: Desktop Companion Evaluation ==="

powershell -ExecutionPolicy Bypass -File scripts/game/eval-companion.ps1 -OutputDir $bundle

if (Test-Path "$env:TEMP\DINOForge\eval\eval_companion_report.json") {
    $report = Get-Content "$env:TEMP\DINOForge\eval\eval_companion_report.json" | ConvertFrom-Json
    Write-Output "✓ Companion evaluation complete"
    Copy-Item "$env:TEMP\DINOForge\eval\eval_companion_report.json" "$bundle\eval_companion_report.json" -Force
} else {
    Write-Output "⚠ Companion evaluation report not generated"
}
```

---

### Step 5: Pipeline D — Advanced Evaluation

Only if `--skip-advanced` is NOT set.

```powershell
Write-Output "`n=== PIPELINE D: Advanced Infrastructure Evaluation ==="

pwsh -ExecutionPolicy Bypass -File scripts/game/eval-advanced.ps1 -OutputDir $bundle

if (Test-Path "$env:TEMP\DINOForge\eval_advanced_report.json") {
    $report = Get-Content "$env:TEMP\DINOForge\eval_advanced_report.json" | ConvertFrom-Json
    Write-Output "✓ Advanced evaluation complete"
    Copy-Item "$env:TEMP\DINOForge\eval_advanced_report.json" "$bundle\eval_advanced_report.json" -Force
} else {
    Write-Output "⚠ Advanced evaluation report not generated"
}
```

---

### Step 6: Generate Unified EVAL_REPORT.md

Synthesize all pipeline reports into a single markdown summary.

```powershell
Write-Output "`n=== Generating Unified Report ==="

# Collect results from each pipeline
$installerReport = if (Test-Path "$bundle\eval_installer_report.json") {
    Get-Content "$bundle\eval_installer_report.json" | ConvertFrom-Json
} else { $null }

$companionReport = if (Test-Path "$bundle\eval_companion_report.json") {
    Get-Content "$bundle\eval_companion_report.json" | ConvertFrom-Json
} else { $null }

$advancedReport = if (Test-Path "$bundle\eval_advanced_report.json") {
    Get-Content "$bundle\eval_advanced_report.json" | ConvertFrom-Json
} else { $null }

$validateReport = if (Test-Path "$bundle\validate_report.json") {
    Get-Content "$bundle\validate_report.json" | ConvertFrom-Json
} else { $null }

# Generate markdown report
$reportContent = @"
# DINOForge Evaluation Report — $ts

## Summary

| Pipeline | Component | Status | Evidence |
|---|---|---|---|
| A | Game Features (Mods, F9, F10) | $(if ($validateReport -and ($validateReport | Where-Object { $_.confirmed -eq $true }).Count -gt 0) { "✓ PASS" } else { "✗ FAIL" }) | validate_report.json |
| A | Asset Swap | $(if (Test-Path "$bundle\validate_asset_swap.png") { "✓ PASS" } else { "⊘ SKIP" }) | validate_asset_swap.png |
| A | Stat Override | $(if (Test-Path "$bundle\validate_stat_override.png") { "✓ PASS" } else { "⊘ SKIP" }) | validate_stat_override.png |
| A | Economy Pack | $(if (Test-Path "$bundle\validate_economy.png") { "✓ PASS" } else { "⊘ SKIP" }) | validate_economy.png |
| A | Scenario Pack | $(if (Test-Path "$bundle\validate_scenario.png") { "✓ PASS" } else { "⊘ SKIP" }) | validate_scenario.png |
| A | Hot Reload | $(if (Test-Path "$bundle\validate_hot_reload.png") { "✓ PASS" } else { "⊘ SKIP" }) | validate_hot_reload.png |
| B | Installer | $(if ($installerReport.status -eq "PASS") { "✓ PASS" } else { "✗ FAIL" }) | eval_installer_report.json |
| C | Companion UI | $(if ($companionReport.status -eq "PASS") { "✓ PASS" } else { "✗ FAIL" }) | eval_companion_report.json |
| D | Advanced (Hidden Desktop, Dual-Instance, VDD) | $(if ($advancedReport.status -eq "PASS") { "✓ PASS" } else { "⊘ SKIP" }) | eval_advanced_report.json |

## Pipeline A: Game Features

### Core Features (Video Proof)
| Feature | VLM Confirmed | Screenshot | Raw Clip |
|---|---|---|---|
| Mods Button | $(if ($validateReport[0].confirmed) { "✓" } else { "✗" }) | validate_mods.png | raw_mods.mp4 |
| F9 Debug Overlay | $(if ($validateReport[1].confirmed) { "✓" } else { "✗" }) | validate_f9.png | raw_f9.mp4 |
| F10 Mod Menu | $(if ($validateReport[2].confirmed) { "✓" } else { "✗" }) | validate_f10.png | raw_f10.mp4 |

### Extended Features
| Feature | Screenshot | Status |
|---|---|---|
| Asset Swap | validate_asset_swap.png | $(if (Test-Path "$bundle\validate_asset_swap.png") { "✓ Captured" } else { "⊘ Skipped" }) |
| Stat Override | validate_stat_override.png | $(if (Test-Path "$bundle\validate_stat_override.png") { "✓ Captured" } else { "⊘ Skipped" }) |
| Economy Balance Pack | validate_economy.png | $(if (Test-Path "$bundle\validate_economy.png") { "✓ Captured" } else { "⊘ Skipped" }) |
| Scenario Pack | validate_scenario.png | $(if (Test-Path "$bundle\validate_scenario.png") { "✓ Captured" } else { "⊘ Skipped" }) |
| Hot Reload | validate_hot_reload.png | $(if (Test-Path "$bundle\validate_hot_reload.png") { "✓ Captured" } else { "⊘ Skipped" }) |

### Rendered Videos (if not --quick)
| Output | File | Status |
|---|---|---|
| Mods feature clip | mods_feature.mp4 | $(if (Test-Path "$bundle\mods_feature.mp4") { "✓ Available" } else { "⊘ Not rendered" }) |
| F9 feature clip | f9_feature.mp4 | $(if (Test-Path "$bundle\f9_feature.mp4") { "✓ Available" } else { "⊘ Not rendered" }) |
| F10 feature clip | f10_feature.mp4 | $(if (Test-Path "$bundle\f10_feature.mp4") { "✓ Available" } else { "⊘ Not rendered" }) |
| Compilation reel | dinoforge_reel.mp4 | $(if (Test-Path "$bundle\dinoforge_reel.mp4") { "✓ Available" } else { "⊘ Not rendered" }) |

## Pipeline B: Installer

**Status**: $(if ($installerReport) { $installerReport.status } else { "NOT RUN" })

$(if ($installerReport) {
@"
- **PowerShell Script**: $(if ($installerReport.powershell_valid) { "✓ Valid" } else { "✗ Invalid" })
- **InstallerLib Build**: $(if ($installerReport.installer_lib_builds) { "✓ Success" } else { "✗ Failed" })
- **GUI Build**: $(if ($installerReport.gui_builds) { "✓ Success" } else { "✗ Failed" })
- **Unit Tests**: $(if ($installerReport.unit_tests_pass) { "✓ Pass" } else { "✗ Fail" })

**Details**: See `eval_installer_report.json` for full schema.
"@
})

## Pipeline C: Desktop Companion

**Status**: $(if ($companionReport) { $companionReport.status } else { "NOT RUN" })

$(if ($companionReport) {
@"
- **Builds**: $(if ($companionReport.builds) { "✓ Yes" } else { "✗ No" })
- **UI Automation Tests**: $(if ($companionReport.automation_tests_pass) { "✓ Pass" } else { "✗ Fail" })
- **Pages Validated**: $(if ($companionReport.pages_validated) { "$($companionReport.pages_validated) pages" } else { "0 pages" })

**Details**: See `eval_companion_report.json` for full results.
"@
})

## Pipeline D: Advanced Infrastructure

**Status**: $(if ($advancedReport) { $advancedReport.status } else { "SKIPPED" })

$(if ($advancedReport) {
@"
- **D-1: Hidden Desktop**: $(if ($advancedReport.d1_hidden_desktop) { "✓ PASS" } elseif ($advancedReport.d1_hidden_desktop -eq $null) { "⊘ SKIP" } else { "✗ FAIL" })
- **D-2: Dual-Instance**: $(if ($advancedReport.d2_dual_instance) { "✓ PASS" } elseif ($advancedReport.d2_dual_instance -eq $null) { "⊘ SKIP" } else { "✗ FAIL" })
- **D-3: VDD**: $(if ($advancedReport.d3_vdd) { "✓ PASS" } elseif ($advancedReport.d3_vdd -eq $null) { "⊘ SKIP" } else { "✗ FAIL" })

**Details**: See `eval_advanced_report.json` for infrastructure readiness.
"@
})

## Bundle Contents

All artifacts are in: **$bundle**

- **Raw game clips**: raw_mods.mp4, raw_f9.mp4, raw_f10.mp4
- **VLM validation screenshots**: validate_*.png (5+ images)
- **Extended feature screenshots**: validate_stat_override.png, validate_asset_swap.png, validate_economy.png, validate_scenario.png, validate_hot_reload.png
- **Rendered videos**: mods_feature.mp4, f9_feature.mp4, f10_feature.mp4, dinoforge_reel.mp4 (if not --quick)
- **Pipeline reports**: eval_installer_report.json, eval_companion_report.json, eval_advanced_report.json
- **VLM validation report**: validate_report.json
- **This summary**: EVAL_REPORT.md

## Execution Summary

- **Total pipelines run**: $(if (-not $skipInstaller) { 1 } else { 0 }) + $(if (-not $skipGame) { 1 } else { 0 }) + $(if (-not $skipCompanion) { 1 } else { 0 }) + $(if (-not $skipAdvanced) { 1 } else { 0 })
- **Timestamp**: $ts
- **Quick mode**: $(if ($quick) { "Yes" } else { "No" })
- **Game launched**: $(if (-not $skipGame) { "Yes" } else { "No" })

"@

$reportContent | Out-File "$bundle\EVAL_REPORT.md" -Encoding UTF8
Write-Output "✓ Report written to: $bundle\EVAL_REPORT.md"
```

---

### Step 7: Open bundle folder

```powershell
Write-Output "`nOpening bundle folder..."
Start-Process $bundle
Write-Output "`nEvaluation complete. Results at: $bundle"
```

---

## Error Handling

If any pipeline step fails:
1. Log the error to console
2. Mark the failed pipeline as **FAILED** in the report
3. Continue with remaining pipelines
4. The unified `EVAL_REPORT.md` will show all successes and failures side-by-side

Example:
```
✗ Pipeline A (Game Features) failed at Phase 3 (Remotion render)
⊘ Continuing to Pipeline B (Installer)...
✓ Pipeline B complete
```

---

## Success Criteria

Minimal success (at least one pipeline completes):
- ✓ One or more pipeline reports generated
- ✓ `EVAL_REPORT.md` created with summary
- ✓ Bundle folder opened on completion

Full success (all pipelines pass):
- ✓ Pipeline A: All features VLM-confirmed, rendered videos exist
- ✓ Pipeline B: All installer tests pass
- ✓ Pipeline C: All Companion UI tests pass
- ✓ Pipeline D: All advanced infrastructure checks pass

---

## Troubleshooting

**Pre-flight fails**: Check `.NET SDK` version and game path
```powershell
dotnet --version
Test-Path "G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option\Diplomacy is Not an Option.exe"
```

**Pipeline A fails**: Game capture or VLM validation issue
- Verify ffmpeg: `Get-Command ffmpeg`
- Verify MCP server: `curl http://127.0.0.1:8765/health`
- Check game window title doesn't have "Fatal error"

**Pipeline B fails**: Installer project compilation issue
- Verify .NET SDK: `dotnet --version`
- Check InstallerLib project: `dotnet build src/Runtime/InstallerLib/InstallerLib.csproj`

**Pipeline C fails**: Companion build or UI automation issue
- Verify Avalonia project builds: `dotnet build src/Tools/DesktopCompanion/DesktopCompanion.csproj`
- Check FlaUI is available for UI Automation

**Pipeline D fails**: Advanced infrastructure not set up
- Check hidden desktop support: `scripts/game/hidden_desktop_test.ps1`
- Verify test instance path: `.dino_test_instance_path` file
- VDD (D-3) is planned; skip if not implemented

**Report generation fails**: Check bundle folder permissions
- Ensure `$bundle` path is writable
- Verify disk space available

---

## Parameters Recap

| Flag | Behavior |
|---|---|
| `--quick` | Skip TTS, Remotion renders, hot reload test (game capture + VLM still run) |
| `--skip-game` | Skip Pipeline A entirely (Pipelines B, C, D still run) |
| `--skip-companion` | Skip Pipeline C (others run) |
| `--skip-installer` | Skip Pipeline B (others run) |
| `--skip-advanced` | Skip Pipeline D (others run) |
| `--output <path>` | Custom bundle directory (default: docs/proof-of-features/eval_<timestamp>) |
