# Concurrent Game Instances — Quick Start

**TL;DR**: Launch two independent game instances in <20 seconds, using ~100MB disk instead of 12GB.

## One-Liner

```powershell
$inst = & "scripts/game/Launch-ConcurrentInstances.ps1"; Write-Host "Main: $($inst.main.DebugLogPath)"; Write-Host "Temp: $($inst.temp.DebugLogPath)"
```

Both instances now running. Check logs anytime. When done:

```powershell
Remove-Item $inst.temp.RootDir -Recurse -Force
```

## What Just Happened

1. **Main instance** launched at `G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option\`
2. **Temp instance** created as lightweight symlink copy in `$env:TEMP\DINOForge\instances\<uuid>\`
3. Both share game files (no duplication via symlinks)
4. Both have independent logs, saves, and configs
5. Both initialized and ready for testing in <20 seconds

## Verify Both Are Running

```powershell
Get-Process | Where-Object { $_.Name -like "*Diplomacy*" } | Select-Object Name, Id, WorkingDirectory
```

Should show 2 processes.

## Check Logs

```powershell
# Main instance
Get-Content "G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option\BepInEx\dinoforge_debug.log" -Tail 20

# Temp instance (use path from $inst output)
Get-Content $inst.temp.DebugLogPath -Tail 20
```

## Performance

| Metric | Value |
|--------|-------|
| Cold-start time | <20 seconds |
| Disk per instance | ~100 MB |
| Total for 2 instances | <250 MB |
| Cleanup time | <1 second |

## FAQ

**Q: Can I test pack changes on both?**
A: Yes! Each instance has independent LocalAppData, so packs loaded in one don't affect the other.

**Q: What if I modify Runtime/plugins?**
A: Both instances symlink to the same BepInEx. Restart both to pick up changes. Or use full copy approach for plugin testing.

**Q: How do I kill just one instance?**
A: Both instances are independent processes. Kill by PID:
```powershell
Get-Process | Where-Object { $_.Id -eq $inst.main.ProcessId } | Stop-Process -Force
# or
Get-Process | Where-Object { $_.Id -eq $inst.temp.ProcessId } | Stop-Process -Force
```

**Q: Symlinks are confusing. Just copy the directory?**
A: Old `_TEST` approach still works:
```powershell
Copy-Item "G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option" `
          "G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option_TEST" `
          -Recurse -Force  # Takes 3-5 minutes
```

Symlink approach is 60x faster but both work.

## Real-World Example

```powershell
# 1. Launch both instances
$inst = & "scripts/game/Launch-ConcurrentInstances.ps1"

# 2. Deploy a pack to main instance
dotnet run --project src/Tools/PackCompiler -- build packs/warfare-starwars

# 3. Test in main instance
# ... wait for game to show updated content ...

# 4. Deploy different pack to temp instance (independent LocalAppData)
# ... manually copy different pack to temp instance BepInEx\dinoforge_packs\ ...

# 5. Compare behavior between instances
# ... side-by-side testing ...

# 6. Cleanup
Get-Process | Where-Object { $_.Name -like "*Diplomacy*" } | Stop-Process -Force
Remove-Item $inst.temp.RootDir -Recurse -Force
```

## Files

- **Launcher**: `scripts/game/Launch-ConcurrentInstances.ps1`
- **Temp factory**: `scripts/game/New-TempGameInstance.ps1`
- **Full docs**: `docs/CONCURRENT_INSTANCES.md`
- **Technical report**: `docs/sessions/HIDDEN_DESKTOP_CONCURRENT_INSTANCES_FINAL_REPORT.md`

## See Also

- `.claude/commands/launch-game.md` — Single instance launcher
- Boot config fix: `single-instance=0` in `Diplomacy is Not an Option_Data\boot.config`
