# eval-companion

Evaluation pipeline for the DINOForge Desktop Companion.

This command orchestrates:

1. **Build the companion** (Release mode)
2. **Run the UiAutomation test suite** with FlaUI (Windows UI Automation)
3. **Validate each UI page** with VLM screenshot analysis
4. **Generate eval report** (JSON)

## Prerequisites

- .NET 8+ SDK installed
- Windows 10+
- Visual Studio 2022+ (or Build Tools)
- Game screenshot MCP tools available

## Usage

```bash
eval-companion
```

or with output directory:

```bash
eval-companion --output-dir ./docs/eval-reports/companion
```

## Steps

### 1. Build Companion (Release)

```powershell
dotnet build "C:\Users\koosh\Dino\src\Tools\DesktopCompanion\DesktopCompanion.csproj" -c Release --verbosity minimal
```

The build outputs:
```
C:\Users\koosh\Dino\src\Tools\DesktopCompanion\bin\Release\net11.0-windows10.0.26100.0\DINOForge.DesktopCompanion.exe
```

### 2. Run UiAutomation Tests

Set env var and run tests:

```powershell
$companionExe = "C:\Users\koosh\Dino\src\Tools\DesktopCompanion\bin\Release\net11.0-windows10.0.26100.0\DINOForge.DesktopCompanion.exe"
$env:COMPANION_EXE = $companionExe

dotnet test "C:\Users\koosh\Dino\src\Tests\UiAutomation\DINOForge.Tests.UiAutomation.csproj" `
  --verbosity normal `
  --logger "console;verbosity=minimal"
```

Capture test results: pass/fail counts, timing.

### 3. VLM Screenshot Validation

For each companion page (Dashboard, PackList, DebugPanel, Settings):

1. Launch the companion process
2. Wait 2 seconds for UI to render
3. Navigate to page via UI automation helpers in CompanionFixture
4. Take screenshot using `game_screenshot` MCP tool
5. Analyze screenshot with `game_analyze_screen` MCP tool (detect buttons, text, controls)
6. Store VLM response in report

**Navigation AutomationIds** (from CompanionFixture.cs):
- Dashboard: `NavDashboard`
- Pack List: `NavPackList`
- Debug Panel: `NavDebugPanel`
- Settings: `NavSettings`

### 4. Generate Report

Write `eval_companion_report.json` with:

```json
{
  "timestamp": "2026-03-29T12:34:56Z",
  "companion_exe": "<path>",
  "build_status": "success|failure",
  "test_results": {
    "total_tests": 0,
    "passed": 0,
    "failed": 0,
    "skipped": 0,
    "duration_ms": 0
  },
  "page_validation": [
    {
      "page_name": "Dashboard",
      "navigation_id": "NavDashboard",
      "screenshot_path": "<relative path>",
      "vlm_analysis": "<VLM response summary>",
      "validation_status": "pass|fail"
    }
  ],
  "overall_status": "pass|fail",
  "notes": ""
}
```

## Implementation Notes

- Use `scripts/game/eval-companion.ps1` helper script for robust build + test orchestration
- FlaUI tests use `UIA3Automation` for WinUI 3 companion introspection
- CompanionFixture handles process launch, window detection (15s timeout), navigation
- Screenshots stored in output directory with timestamp
- Exit code 0 = success, 1 = any failure

## Related

- `src/Tools/DesktopCompanion/` — Companion source (WinUI 3 MVVM)
- `src/Tests/UiAutomation/` — FlaUI test suite
- `scripts/game/eval-companion.ps1` — Helper script
- `docs/screenshots/` — Screenshot archive
