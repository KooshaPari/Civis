# DINOForge Debug Log Size Audit (iter-142)

## File Metrics
- **Size**: 3.08 GB
- **Last Write**: 2026-05-18 03:16:52 UTC
- **Age**: 15.7 hours (since ~11:30 PM yesterday)
- **Estimated Lines**: ~73+ million (avg 42 chars/line)
- **Disk Headroom**: 128.98 GB free on G: (no immediate pressure)

## Log Content Analysis
**Dominant Pattern (90%+ of file):**
```
[5/18/2026 3:13:12 AM] [KeyInputSystem] [KeyInputSystem.OnUpdate] frame=203400 enabled=True overlayEnsured=True PersistentRoot=alive
```

Every ~8-11 seconds, ONE line emitted per KeyInputSystem.OnUpdate call. At 60 FPS × 15.7 hours = 3.4M frames. Each line ~85 bytes. Result: **3.08 GB of pure per-frame telemetry spam**.

## Root Cause
File `src/Runtime/Bridge/KeyInputSystem.cs` line 354 writes to `dinoforge_debug.log` during **every OnUpdate call** (ECS frame), appending via `File.AppendAllText()` with 100% no rotation logic.

See:
- `src/Runtime/Bridge/GameBridgeServer.cs:2275` — `WriteDebug()` appends without size check
- `src/Runtime/Bridge/KeyInputSystem.cs` — logs frame state per ECS tick
- 20+ other systems also log to same file (AssetSwap, Aerial*, Faction, LOD, etc.)

## Recommendations (Priority Order)

| Option | Effort | Impact | Status |
|--------|--------|--------|--------|
| **Remove per-frame KeyInputSystem spam** | 5 min (1 line delete) | 90% reduction, unblocks usage | ✅ Recommended |
| Implement log rotation (max-size + rollover) | 30 min | Clean long-running sessions | P2 (do after spam removal) |
| Conditional debug logging (config gate) | 20 min | Runtime control | P3 |
| Accept + archive (manual cleanup) | <1 min | Band-aid, recurring problem | Not recommended |

## Next Action
1. Comment out or remove line 354 in `KeyInputSystem.cs` (the per-frame WriteDebug call)
2. Rebuild DINOForge.Runtime.csproj, redeploy
3. Verify new log size growth rate (should drop to ~10MB/session vs. 3GB/15hrs)
4. File a follow-up task for log rotation infrastructure (P2)

