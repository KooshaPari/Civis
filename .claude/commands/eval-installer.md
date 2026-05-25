# /eval-installer

Evaluate DINOForge installer end-to-end — builds, validates, and tests.

---

## Overview

The installer pipeline consists of:

1. **PowerShell installer** (`Install-DINOForge.ps1`) — User-facing entry point
2. **InstallerLib** — C# core logic (SteamLocator, InstallVerifier)
3. **GUI Installer** — Avalonia 11 MVVM wizard for interactive install
4. **Unit tests** — InstallerTests in DINOForge.Tests

This command orchestrates all three and reports pass/fail status.

---

## What It Tests

- **File integrity** — All installer files exist
- **Parameter extraction** — PowerShell script declares correct params: `-GamePath`, `-Dev`, `-SkipBepInEx`
- **Script syntax** — No PowerShell parsing errors (dot-source validation)
- **InstallerLib build** — C# project compiles successfully
- **GUI Installer build** — Avalonia project compiles successfully
- **Unit tests** — All Installer-related xUnit tests pass (SteamLocator, InstallVerifier, install flow)

---

## Run

```powershell
powershell -ExecutionPolicy Bypass -File scripts/game/eval-installer.ps1
```

Optional custom output directory:

```powershell
powershell -ExecutionPolicy Bypass -File scripts/game/eval-installer.ps1 `
  -OutputDir "C:\Users\koosh\Desktop\reports"
```

---

## Output

Report file: `$env:TEMP\DINOForge\eval\eval_installer_report.json`

Schema:

```json
{
  "timestamp": "2026-03-29T12:34:56...",
  "overall_status": "passed|failed",
  "steps": [
    {
      "name": "file_verification|parameter_extraction|script_syntax_validation|build_installerlib|build_gui_installer|unit_tests|report_generation",
      "status": "passed|failed|warning",
      "details": { "...": "..." },
      "time": "2026-03-29T12:34:56..."
    }
  ],
  "installer_params": {
    "string_params": ["GamePath"],
    "switch_params": ["Dev", "SkipBepInEx"]
  },
  "test_results": {
    "summary": "15 passed, 0 failed",
    "output": [...]
  },
  "errors": ["..."]
}
```

---

## Failure Modes

| Failure | Cause | Fix |
|---------|-------|-----|
| File verification failed | Missing `Install-DINOForge.ps1` or `.csproj` files | Check file paths in script; run from repo root |
| Script syntax validation warning | Parameter dot-source test fails | Check `Install-DINOForge.ps1` syntax (Run Invoke-ScriptAnalyzer) |
| Build failed: InstallerLib | C# compilation error in `InstallerLib/` | Run `dotnet build src/Tools/Installer/InstallerLib/ -c Release` and inspect errors |
| Build failed: GUI Installer | Avalonia/WinUI dependency missing | Run `dotnet restore src/Tools/Installer/GUI/` |
| Unit tests failed | Installer test suite regression | Run `dotnet test src/Tests/ --filter "Installer"` for details |

---

## Related Commands

- `/launch-game` — Test the installer by launching a full game instance
- `/pack-deploy` — Deploy packs after installer validation
- `/check-game` — Verify DINOForge runtime is active post-install

---

## Integration with CI

The evaluation pipeline is designed to run in GitHub Actions and locally:

```yaml
# .github/workflows/installer-eval.yml
- name: Evaluate Installer
  run: |
    powershell -ExecutionPolicy Bypass -File scripts/game/eval-installer.ps1
```

Reports are persisted to `eval_installer_report.json` for CI artifacts.
