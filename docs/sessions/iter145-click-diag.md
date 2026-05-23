# iter145 click diag

## Change set

- [src/Runtime/UI/DFCanvas.cs:124-125](C:\Users\koosh\Dino\src\Runtime\UI\DFCanvas.cs#L124) raises `sortingOrder` to `32767` and forces canvas-group raycasts during build.
- [src/Runtime/UI/DFCanvas.cs:177-204](C:\Users\koosh\Dino\src\Runtime\UI\DFCanvas.cs#L177) logs EventSystem snapshots in `Update()` and calls the shared reconcile path.
- [src/Runtime/UI/ModMenuPanel.cs:102-200](C:\Users\koosh\Dino\src\Runtime\UI\ModMenuPanel.cs#L102) forces parent CanvasGroups to allow raycasts and logs the hierarchy on `Show()`.
- [src/Runtime/Plugin.cs:287-379](C:\Users\koosh\Dino\src\Runtime\Plugin.cs#L287) reconciles EventSystems, prefers DINOForge-owned systems, disables others, and logs the active set.
- [src/Runtime/Plugin.cs:901-1200](C:\Users\koosh\Dino\src\Runtime\Plugin.cs#L901) wires UGUI on init/world-ready and logs the pack push path.
- [src/Runtime/ModPlatform.cs:111-114](C:\Users\koosh\Dino\src\Runtime\ModPlatform.cs#L111) exposes last-load diagnostics for pack handoff logging.

## Build

- Command: `dotnet build src/Runtime/DINOForge.Runtime.csproj -c Release -p:TargetFramework=netstandard2.0 --nologo`
- Exit: `0`

## Deploy

- Source hash: `C73260DE8775290F9E0A3631654525F9F75D9D44C6A7A67696C5A0D1B2A815EA`
- Destination hash: `C73260DE8775290F9E0A3631654525F9F75D9D44C6A7A67696C5A0D1B2A815EA`

## Live log

Captured after restart and 45s warmup from `G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option\BepInEx\dinoforge_debug.log`.

```text
[2026-05-23T02:54:43.2090524Z] [Plugin] [Plugin] ResurrectionFallback heartbeat #300 NeedsRes=False NeedsDefRes=False rootNull=False
[2026-05-23T02:54:45.2098284Z] [Plugin] [Plugin] ResurrectionFallback heartbeat #304 NeedsRes=False NeedsDefRes=False rootNull=False
[2026-05-23T02:54:47.2136947Z] [Plugin] [Plugin] ResurrectionFallback heartbeat #308 NeedsRes=False NeedsDefRes=False rootNull=False
[2026-05-23T02:54:59.2265672Z] [Plugin] [Plugin] ResurrectionFallback heartbeat #332 NeedsRes=False NeedsDefRes=False rootNull=False
[2026-05-23T02:54:59.9542811Z] [Plugin] [EventSystem] reconcile: preferred=DINOForge_EventSystem_Restored, current=DINOForge_EventSystem_Restored, total=2, enabled=1, systems=[DINOForge_EventSystem_Restored, EventSystem]
```

## Note

- The capture confirms the EventSystem fix path is active and the vanilla `EventSystem` is being deactivated.
- In this 45s window, no `PushLoadedPacksToUgui(...)` or `UGUI wired to ModPlatform` line appeared yet, so the pack-refresh path did not surface in the captured slice.
