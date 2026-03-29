# /eval-advanced

Evaluate DINOForge advanced feature infrastructure: hidden desktop isolation, dual-instance testing, and virtual display driver support.

This command tests **agent-automation readiness** for autonomous game testing without desktop contamination or interference. Three features are evaluated:

- **D-1: Hidden Desktop** — isolated background game launch via Win32 CreateDesktop
- **D-2: Dual-Instance** — concurrent main + test instances for parallel testing
- **D-3: VDD** — planned virtual display driver for future headless deployment

Results are written to `$env:TEMP\DINOForge\eval_advanced_report.json` and reported with PASS/FAIL/SKIP clarity.

---

## Requirements

- PowerShell 5.1+
- MCP server running on `http://127.0.0.1:8765` (start with: `python -m dinoforge_mcp.server` in `src/Tools/DinoforgeMcp`)
- DINOForge repository with game installation

---

## Execution

```powershell
# Run evaluation from repo root
pwsh -ExecutionPolicy Bypass -File scripts/game/eval-advanced.ps1

# With verbose output
pwsh -ExecutionPolicy Bypass -File scripts/game/eval-advanced.ps1 -Verbose

# Custom output directory (default: $env:TEMP\DINOForge)
pwsh -ExecutionPolicy Bypass -File scripts/game/eval-advanced.ps1 -OutputDir C:\custom\path
```

---

## Feature Details

### D-1: Hidden Desktop Launch

**Status**: Available

Tests whether the game can run in an isolated Win32 hidden desktop without interfering with the user's desktop environment.

**What it checks**:
- `scripts/game/hidden_desktop_test.ps1` exists and is executable
- Script contains Win32 P/Invoke definitions (`CreateDesktop`, `SetThreadDesktop`)
- Infrastructure is ready for background game automation

**Why it matters**:
- Agents can run tests without stealing desktop focus
- Automation does not interrupt user workflows
- Game rendering is isolated to a separate virtual desktop
- Critical for CI/CD pipelines and unattended testing

**Limitations**:
- Rendering quality may be reduced on systems with GPU passthrough limits
- Some older graphics drivers may not support hidden desktop context switching
- If GPU is integrated (Intel/AMD APU), performance degrades vs. native desktop

**Result interpretation**:
- ✓ PASS: Script exists and has full P/Invoke infrastructure
- ⚠ SKIP: Script exists but P/Invoke definitions incomplete (partial support)
- ✗ FAIL: Script not found or cannot be executed

---

### D-2: Dual-Instance Testing

**Status**: Ready or Incomplete (depends on test instance deployment)

Tests whether both main and test game instances are prepared for concurrent execution. Enables running two independent game processes simultaneously on the same machine for A/B testing or parallel feature validation.

**What it checks**:
- Configuration file `.dino_test_instance_path` exists
- Path points to a valid directory: `G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option_TEST\`
- Game executable exists at the test instance location
- BepInEx plugin infrastructure is deployed (checked indirectly)

**Why it matters**:
- Agents can run two tests in parallel (main vs. test branch)
- Balance testing: compare stat overrides without manual game restart
- Regression testing: old version vs. new version side-by-side
- Reduces test turnaround time from sequential to parallel

**Limitations**:
- Requires manual duplication of game directory to `_TEST` variant
- Both instances share the same Steam account (single Diplomacy license)
- Both instances cannot have the exact same working directory (Windows mutex on UnityPlayer.dll)
- VDD (D-3) would eliminate this limitation in future

**Result interpretation**:
- ✓ PASS: Both main and test instances are configured and game.exe exists at test location
- ⚠ SKIP: Test directory exists but game.exe not deployed (incomplete setup)
- ✗ FAIL: Configuration missing or test instance path does not exist

**Setup steps (if FAIL)**:
```powershell
# Copy game directory to test instance
Copy-Item -Path "G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option\" `
          -Destination "G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option_TEST\" `
          -Recurse -Force

# Deploy DINOForge to test instance
dotnet build src/Runtime/DINOForge.Runtime.csproj -c Release `
  -p:GameInstallPath="G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option_TEST"
```

---

### D-3: Virtual Display Driver

**Status**: Future (planned for v0.9)

Virtual Display Driver (VDD) is a planned feature for completely isolated headless game launches. A custom IDD/WDDM driver would allow the game to render to a virtual display that doesn't require a hidden Win32 desktop or physical GPU memory.

**Why it matters**:
- Eliminates desktop isolation limitations of D-1
- Supports true headless deployment on remote/cloud machines
- GPU memory is virtualized; no risk of conflict with host graphics
- Future: Game automation in GitHub Actions or Docker containers without X11/Xvfb

**Timeline**:
- v0.9: Design document for custom IDD driver
- v1.0: Integration with PackCompiler and MCP server
- v1.1: CI/CD templates for GitHub Actions headless runners

**Current workaround**:
- Use D-1 (hidden desktop) for agent automation
- For cloud/CI, use GitHub Actions Windows runners (native D3D11 GPU access via cloud provider)

**Result interpretation**:
- ⚠ SKIP: D-3 is future work; D-1 (hidden desktop) currently provides adequate isolation

---

## Output Report

Report file: `$env:TEMP\DINOForge\eval_advanced_report.json`

```json
{
  "timestamp": "2026-03-29T12:34:56.0000000",
  "mcp_server": {
    "status": "healthy",
    "healthy": true,
    "reason": ""
  },
  "d1_hidden_desktop": {
    "status": "available",
    "available": true,
    "reason": "Full hidden desktop infrastructure available; supports isolated game launches",
    "script_path": "C:\\Users\\koosh\\Dino\\scripts\\game\\hidden_desktop_test.ps1"
  },
  "d2_dual_instance": {
    "status": "ready",
    "available": true,
    "reason": "Both main and test instances are prepared; concurrent game launches possible",
    "test_instance_path": "G:\\SteamLibrary\\steamapps\\common\\Diplomacy is Not an Option_TEST"
  },
  "d3_vdd": {
    "status": "future",
    "available": false,
    "reason": "VDD is planned; not yet implemented. Win32 hidden desktop (D-1) currently provides isolation."
  },
  "summary": {
    "passed": 3,
    "failed": 0,
    "skipped": 1
  }
}
```

---

## Troubleshooting

### MCP server shows as offline
```powershell
# Start the MCP server (from src/Tools/DinoforgeMcp directory)
python -m dinoforge_mcp.server

# Verify it's running
curl http://127.0.0.1:8765/health
```

### D-1 shows as unavailable
The hidden desktop launcher script should exist at `scripts/game/hidden_desktop_test.ps1`. If missing, it was likely deleted. Check git history:
```bash
git log --all -- scripts/game/hidden_desktop_test.ps1
git show <commit>:scripts/game/hidden_desktop_test.ps1 > scripts/game/hidden_desktop_test.ps1
```

### D-2 shows as incomplete or failed
**If directory doesn't exist**:
```powershell
# Copy the main game directory to the test location
Copy-Item -Path "G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option\" `
          -Destination "G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option_TEST\" `
          -Recurse -Force
```

**If directory exists but game.exe is missing**:
```powershell
# Deploy game files (must have Steam sync or manual copy)
# Then deploy DINOForge:
dotnet build src/Runtime/DINOForge.Runtime.csproj -c Release `
  -p:GameInstallPath="G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option_TEST"
```

**If config file missing**:
```powershell
# Create the config file and add path
"G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option_TEST" | `
  Out-File -FilePath ".dino_test_instance_path" -Encoding UTF8
```

---

## Next Steps

Once evaluation is complete:

1. **If all PASS**: Advanced features are ready for use in automated testing pipelines
2. **If SKIP (D-2 incomplete)**: Deploy test instance using steps above, then re-run eval-advanced
3. **If FAIL**: See troubleshooting section above or check `eval_advanced_report.json` for detailed reason

---

## Integration with CI/CD

The report can be consumed by GitHub Actions or other CI tools:

```yaml
# In a GitHub Actions workflow
- name: Evaluate advanced features
  run: |
    pwsh -ExecutionPolicy Bypass -File scripts/game/eval-advanced.ps1 `
      -OutputDir ${{ runner.temp }}\DINOForge

- name: Check results
  run: |
    $report = Get-Content ${{ runner.temp }}\DINOForge\eval_advanced_report.json | ConvertFrom-Json
    if ($report.summary.failed -gt 0) {
      Write-Error "Advanced feature evaluation failed"
      exit 1
    }
```

---

## Related Commands

- `/prove-features` — Full feature proof with video evidence
- `/check-game` — Game status and debug logs
- `/launch-game` — Launch second game instance manually
- `/game-coverage` — Advanced automation coverage metrics

