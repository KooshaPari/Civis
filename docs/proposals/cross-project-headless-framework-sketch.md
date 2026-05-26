# Cross-Project Headless Game Framework Sketch

## Goal
A reusable framework: any game project (DINOForge, WorldBoxMod, future) plugs into a shared headless launch + sandbox + journey-record pipeline. Each game brings its own plugin shim; framework provides infrastructure.

## Layered Architecture

```
┌────────────────────────────────────────────┐
│ Claude Code Session (user)                 │
└─────────────┬────────────────────────────┘
              │
┌─────────────▼────────────────────────────┐
│ Cross-project CLI: gamectl (Rust)         │
│ ~/.claude/bin/gamectl                     │
│ Subcommands: launch, capture, input,      │
│ swap, record, navigate, verify            │
└─────────────┬────────────────────────────┘
              │
┌─────────────▼────────────────────────────┐
│ Per-game adapter plugin                   │
│ • BepInEx DLL (Mono games)                │
│ • Custom injector (IL2CPP future)         │
│ Implements IGameAdapter interface         │
└─────────────┬────────────────────────────┘
              │
┌─────────────▼────────────────────────────┐
│ Sandbox layer (pluggable tiers)           │
│ T1: Win32 CreateDesktop (Mono native)     │
│ T2: Windows Sandbox / Docker              │
│ T3: Hyper-V snapshot cloning              │
└────────────────────────────────────────────┘
```

## Per-Game Artifacts (Each Project Provides)

1. **Game adapter plugin** — Implements `IGameAdapter` interface
   - `Connect()` → game process, verify loaded
   - `Status()` → entity count, loaded mods, state
   - `SwapAsset(id, bundle)` → runtime asset replacement
   - `Screenshot()` → capture framebuffer
   - `Reload()` → hot-reload packs / reload signal

2. **Game manifest** — YAML declaring
   - Install path, exe name, DRM type (Steam/non-Steam)
   - Unpack required, BepInEx version, .NET runtime
   - Pack schema path (YAML locations), registry contracts

3. **Pack format** — Game-specific schemas
   - DINO: unit, building, faction, wave, doctrine, trade route
   - WorldBox: creature, biome, event, faction, skill
   - Future games add their own

## Framework Artifacts (Shared Across All Projects)

1. **gamectl Rust CLI** — Single ~8K LOC binary
   - Manifest-driven launch orchestration
   - Unified screenshot/input/recording pipeline
   - Headless session lifecycle (start → wait → capture → stop)

2. **Sandbox backends** — Tiered fallback
   - T1 (current): Win32 CreateDesktop (hardened against #152 hang)
   - T2 (planned): Windows Sandbox containers, Docker
   - T3 (future): Hyper-V snapshots, cloud agents

3. **Journey records** — Structured proof artifacts
   - Video, screenshot, manifest receipt, event log (JSON)
   - VitePress gallery component (shared UI)
   - Verifiable by external judges (Kimi, Claude)

4. **MCP bridge** — Single FastMCP server
   - Loads per-game plugin via manifest discovery
   - Exposes unified tool surface (launch, capture, input, verify)
   - Handles sandbox backend switching

## Integration Timeline

| Phase | Effort | Deliverable |
|-------|--------|-------------|
| Extract DINOForge MCP to framework | 1 sprint | Refactored plugin loader, example adapter |
| Build gamectl Rust CLI | 1 sprint | Launch + capture + input orchestration |
| Onboard WorldBox as 2nd game | 1 sprint | Proves abstraction, reveals schema gaps |
| Add journey records VitePress UI | 1 sprint | Gallery + metadata + judge-receipt linking |
| Sandbox T2/T3 backends | 2 sprints | Windows Sandbox, snapshot cloning |

## Out of Scope (Initial)

- Cross-Unity-version compatibility (Mono 2021 only)
- IL2CPP games (added after Mono proven)
- Console / mobile (PC Windows first)
- Network/cloud deployment (local-first)

## Open Questions for User

1. **Repository home**: New org (`kooshapari/gamectl-framework`) or under existing?
2. **Licensing**: Apache 2.0 (compat with DINO) or AGPL (viral)?
3. **Versioning**: Unified framework version (all games track together) or per-game?
4. **Plugin distribution**: NuGet (per game) or centralized registry?
