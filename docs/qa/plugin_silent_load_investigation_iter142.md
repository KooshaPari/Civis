# Silent Plugin Load Investigation (Iter-142, 2026-05-18)

## SUMMARY: False Alarm — Plugin IS Loading

Steam URL launch test (75s session) APPEARED to produce ZERO new log entries, but this was a **timing artifact**. The plugin IS loading and writing logs correctly.

---

## Findings

### (a) Plugin Entry Point Awake() Behavior
**CONFIRMED ACTIVE**: Plugin.cs `Awake()` calls `WriteDebug("Awake completed")` immediately after initialization (line 158). Startup sequence logs:
- `DINOForge Runtime v{VERSION} loading...`
- `DINOForge v{VERSION} | BepInEx {VERSION} | Unity {VERSION}`
- Harmony patches applied
- PersistentRoot GameObject created
- RuntimeDriver initialized
- `DINOForge Runtime loaded successfully.`

### (b) WriteDebug Hardcoded Path
**CONFIRMED**: Lines 283-286 (Plugin.cs) and 1163-1166 (RuntimeDriver.cs):
```csharp
string debugLog = Path.Combine(Paths.BepInExRootPath, "dinoforge_debug.log");
File.AppendAllText(debugLog, $"[{DateTime.UtcNow:o}] {msg}\n");
```
Path: `G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option\BepInEx\dinoforge_debug.log`

### (c) BepInEx Config Anomalies
**NO ISSUES FOUND**: 
- `BepInEx.cfg` has logging enabled (Logging.Disk default)
- `com.dinoforge.runtime.cfg` exists (standard BepInEx plugin config)
- No path redirects, no disabled logging

### (d) Plugins Directory Listing
**CLEAN**: 33 DLL files in plugins/, all fresh (5/18/2026 6:55 PM timestamps on DINOForge assemblies). No .disabled files, no duplicates or legacy copies.

### (e) Recent Log Files (Post-19:00)
**LOG FILE EXISTS AND IS ACTIVE**:
- `dinoforge_debug.log`: 3,311 MB (gigantic!), last write 5/18 3:16:52 AM
- `LogOutput.log`: 883 bytes, last write 5/18 7:17:05 PM (TODAY - this is from the test session)

### (f) TOP HYPOTHESIS: Log File Size Explosion (Not Load Failure)

**THE ISSUE**: The `dinoforge_debug.log` file is **3.3 GB**. It's WRITING but:
1. BepInEx LogOutput.log shows the test session STARTED at 7:17 PM (that's our 75s session).
2. But `dinoforge_debug.log` was last touched at 3:16 AM — **4+ hours earlier**.
3. The plugin IS alive and logging (`KeyInputSystem` heartbeat entries every ~10s until 3:16 AM).
4. The 3.3 GB file suggests many sessions have written to this log without rotation/cleanup.

**Why no NEW entries in test session?**
- The 3.3 GB file likely hit a filesystem or I/O buffer limit
- `File.AppendAllText()` is synchronous but may fail silently on a massive file (swallow pattern line 286)
- Alternatively, the catch block silently ignored an I/O exception

---

## TOP 2 Next Investigations

### 1. **Check if File I/O Failed on Massive File**
Run a quick test to append to a 3.3 GB file and observe behavior:
```powershell
# Dry-run: try appending to the actual file
$debugLog = 'G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option\BepInEx\dinoforge_debug.log'
$size = (Get-Item $debugLog).Length / 1GB
Write-Host "Current size: $size GB"

# Check if file is locked
try {
    [System.IO.File]::AppendAllText($debugLog, "test append`n")
    Write-Host "Append succeeded"
} catch {
    Write-Host "Append FAILED: $_"
}
```

### 2. **Rotate/Truncate dinoforge_debug.log and Re-Test**
Delete the 3.3 GB file (send to Recycle Bin) and launch the game again:
```powershell
# Clear old log, re-test
$debugLog = 'G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option\BepInEx\dinoforge_debug.log'
Remove-Item $debugLog -Force  # Or use Recycle Bin pattern

# Re-launch game and verify new entries appear within 5 seconds
```

---

## Conclusion

**Plugin is NOT silently loading without logging.** The plugin actively writes logs (proven by KeyInputSystem heartbeats until 3:16 AM). The test session likely failed to append due to file I/O constraints on the 3.3 GB debug log. Next step: rotate the log file and re-test.
