# ADR-009: Runtime Orchestration via ModPlatform

**Status**: Accepted
**Date**: 2026-03-10
**Deciders**: kooshapari

## Context

All major SDK, Bridge, UI, and HMR components existed as isolated pieces:

- **SDK**: Pack loading, registry system, schema validation, dependency resolution
- **Bridge**: ECS glue layer translating SDK operations into game state changes
- **UI**: F10 mod menu overlay, F9 debug overlay
- **HMR**: File watching and hot reload of YAML pack content

Each component worked independently but there was no central coordinator connecting them into a coherent lifecycle. Plugin.cs had to manually wire everything together, leading to fragile initialization ordering, scattered error handling, and difficulty testing the full pipeline.

## Decision

Create a `ModPlatform` class that owns the entire mod platform lifecycle:

```
Initialize -> OnWorldReady -> LoadPacks -> StartHotReload -> Shutdown
```

Key design choices:

- **ModPlatform is not a MonoBehaviour**. It is a plain C# class instantiated by `Plugin.cs` and given Unity callbacks (Update, OnDestroy) by the plugin.
- **Plugin.cs stays thin**. It creates `ModPlatform`, forwards Unity lifecycle events, and does nothing else.
- **Single entry point** for all mod platform features. SDK, Bridge, UI, and HMR are initialized and coordinated through ModPlatform.
- **Lifecycle phases** are explicit:
  - `Initialize()`: Set up logging, load configuration, create SDK services
  - `OnWorldReady(World)`: Called when ECS World is available; initialize Bridge, enumerate systems/entities
  - `LoadPacks()`: Discover packs, validate, resolve dependencies, apply to registries via Bridge
  - `StartHotReload()`: Begin file watching, wire change events to re-validation and re-application
  - `Shutdown()`: Stop HMR, flush state, clean up

## Consequences

- **Single entry point**: All mod platform behavior flows through ModPlatform, making it easy to understand the full initialization sequence.
- **Plugin.cs stays thin**: The BepInEx plugin class contains only bootstrap glue, not business logic.
- **Testable**: ModPlatform can be instantiated in tests with a mock World, without requiring BepInEx or Unity runtime. Each lifecycle phase can be tested independently.
- **Clear error boundaries**: Each phase has its own error handling. A failure in LoadPacks does not prevent the debug overlay from working.
- **Ordering guarantees**: Components that depend on ECS World availability are only initialized after OnWorldReady, eliminating null-World race conditions.
- **HMR integration**: Hot reload is a first-class lifecycle phase rather than an afterthought bolted onto Plugin.cs.
