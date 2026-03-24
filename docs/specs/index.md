# Specifications

Welcome to the DINOForge specifications documentation. This section contains comprehensive user and technical documentation for the DINOForge platform.

---

## Overview

DINOForge specifications are divided into two primary documents:

### [User Specification](./user-spec.md)

**For**: Modders, game enthusiasts, content creators

**Contains**:
- User personas and target audiences
- Core user workflows (install, create pack, balance tune, hot reload)
- Feature specifications for each DINOForge capability
- Implementation status and known issues
- Performance expectations
- Installation and pack creation workflows
- Acceptance testing checklists

**Use This If**:
- You want to create a mod for DINO
- You need to understand what DINOForge can do
- You're testing a pack and need acceptance criteria
- You want to know about known limitations

---

### [Technical Specification](./technical-spec.md)

**For**: Plugin developers, runtime maintainers, advanced modders

**Contains**:
- Architecture overview (three-layer design)
- Core component responsibilities (Plugin, RuntimeDriver, KeyInputSystem, ModPlatform, DebugOverlay, HudStrip)
- ECS Bridge layer (ComponentMap, EntityQueries, AssetSwapSystem)
- Key design decisions and their rationale
- Critical facts about DINO's ECS (prefab entities, scene transitions, two-boot cycle)
- Component interactions and data flow diagrams
- F9/F10 key input architecture
- Pack system and registry pattern
- Schema validation pipeline
- Technical constraints and workarounds
- Extension points for custom plugins

**Use This If**:
- You want to extend DINOForge with custom domains
- You need to understand internal architecture
- You're debugging integration issues
- You're contributing to the runtime or SDK

---

## Quick Navigation

### For Modders

1. **Getting Started**: Read [User Specification - Installation Workflow](./user-spec.md#5-installation-workflow)
2. **Create Your First Pack**: See [User Specification - Pack Creation Workflow](./user-spec.md#6-pack-creation-workflow)
3. **Debug Issues**: Check [User Specification - Common Workflows](./user-spec.md#7-common-workflows)
4. **Known Issues**: Review [User Specification - Known Issues and Limitations](./user-spec.md#8-known-issues-and-limitations)

### For Plugin Developers

1. **Understand Architecture**: Read [Technical Specification - Architecture Overview](./technical-spec.md#1-architecture-overview)
2. **Learn Core Components**: Study [Technical Specification - Core Components](./technical-spec.md#2-core-components)
3. **Understand Design Decisions**: Review [Technical Specification - Key Design Decisions](./technical-spec.md#3-key-design-decisions)
4. **Learn Extension Points**: See [Technical Specification - Extension Points](./technical-spec.md#11-extension-points-for-plugin-developers)

---

## Feature Status Matrix

| Feature | User-Facing | Tech Details | Status | Version |
|---------|---|---|---|---|
| Pack loading & manifest system | [User](./user-spec.md#f1-pack-loading--manifest-system) | [Tech](./technical-spec.md#27-pack-system-architecture) | ✅ STABLE | 0.5.0+ |
| Debug overlay (F9) | [User](./user-spec.md#story-2-view-debug-overlay-f9-key) | [Tech](./technical-spec.md#25-debugoverlaycs-f9-display) | ✅ STABLE | 0.6.0+ |
| Mod menu (F10) | [User](./user-spec.md#story-3-access-in-game-mod-menu-f10-key) | [Tech](./technical-spec.md#26-hudstripcs-f10-display) | ✅ STABLE | 0.8.0+ |
| Hot reload (YAML) | [User](./user-spec.md#story-7-hot-reload-packs-without-restarting-game) | [Tech](./technical-spec.md#flow-3-f10-pressed--reload) | ✅ STABLE | 0.9.0+ |
| Asset swap system | [User](./user-spec.md#story-6-load-themed-asset-packs) | [Tech](./technical-spec.md#assetsswapsystem) | ✅ STABLE | 0.7.0+ |
| Desktop Companion | [User](./user-spec.md#story-9-use-desktop-companion-to-manage-packs) | [Tech](./technical-spec.md#extension-point-2-custom-asset-processor) | ✅ STABLE | 0.9.0+ |
| Warfare domain plugin | [User](./user-spec.md#story-10-create-a-total-conversion) | [Tech](./technical-spec.md#extension-point-1-custom-domain-plugin) | ✅ STABLE | 0.6.0+ |

---

## Architecture Diagram

```
┌─────────────────────────────────────────────────┐
│ CONTENT LAYER (Packs)                           │
│  YAML manifests, assets, registries             │
└─────────────────────────────────────────────────┘
           ↑
┌─────────────────────────────────────────────────┐
│ SDK LAYER                                       │
│  ContentLoader, Registry, Schemas, Validators  │
└─────────────────────────────────────────────────┘
           ↑
┌─────────────────────────────────────────────────┐
│ RUNTIME LAYER (BepInEx Plugin)                  │
│  Plugin.cs → RuntimeDriver → Systems            │
│  • KeyInputSystem (F9/F10)                      │
│  • ModPlatform (orchestration)                  │
│  • DebugOverlay & HudStrip (UI)                │
│  • ECS Bridge (ComponentMap, EntityQueries)    │
│  • AssetSwapSystem (visuals)                    │
└─────────────────────────────────────────────────┘
           ↑
┌─────────────────────────────────────────────────┐
│ GAME (DINO)                                     │
│  Unity 2021.3.45f2 ECS + BepInEx                │
└─────────────────────────────────────────────────┘
```

---

## Key Facts

### For Users

- **Two Overlays**: F9 (debug) and F10 (mod menu) are always available
- **Hot Reload**: Edit YAML, save, and see changes in-game without restart
- **Asset Swaps**: Pack authors can include custom 3D models for units/buildings
- **No Crashes on Errors**: Missing assets or broken configs fail gracefully with fallbacks
- **Performance**: <2ms overhead per overlay; full game at 60 FPS with mods

### For Developers

- **ECS First**: All core systems are ECS-based, not MonoBehaviour-based
- **Persistence**: RuntimeDriver persists across scene transitions via DontDestroyOnLoad
- **Win32 Input**: F9/F10 work globally via GetAsyncKeyState (not Unity Input)
- **Prefab Entities**: All DINO entities use `EntityQueryOptions.IncludePrefab`
- **600-Frame Stabilization**: Asset swaps wait 600 frames before executing

---

## Troubleshooting Checklist

### Pack Won't Load

1. **Validate manifest**: Run `dotnet run --project src/Tools/PackCompiler -- validate packs/my-pack`
2. **Check schema**: Ensure all YAML files match canonical schemas in `schemas/`
3. **Check dependencies**: Verify all packs in `depends_on` are installed
4. **Check conflicts**: Review `conflicts_with` list; remove if conflicting pack is installed

### Assets Not Showing

1. **Check bundle filename**: Must match exactly the `visual_asset` ID in pack YAML
2. **Check Unity version**: Bundles must be built with Unity 2021.3.45f2
3. **Check addressables**: Verify bundle is registered in addressables catalog
4. **Check logs**: Press F9 and look for "AssetSwap" warnings in error log

### Hot Reload Not Working

1. **Check system**: Only YAML-only content reloads; code changes require restart
2. **Check permissions**: Ensure file is saved and readable
3. **Check changes**: Edit stats (health, cost); role changes require restart

### F9/F10 Not Responding

1. **Check window focus**: F9/F10 should work even unfocused, but fullscreen exclusive mode may block
2. **Check debounce**: Wait 166ms between presses (10-frame cooldown)
3. **Check logs**: Check `BepInEx/LogOutput.log` for KeyInputSystem errors

---

## Version History

### v0.11.0 (Current)

- Desktop Companion WinUI 3 app
- F9/F10 key input refactored to RuntimeDriver
- Asset swap Phase 2 (live entity updates)
- 678 tests passing

### v0.10.0

- Audio hot reload support
- Companion file watcher integration
- Bug fixes and optimizations

### v0.9.0

- Desktop Companion preview
- Pack registry discovery improvements
- MCP server enhancements

### v0.8.0

- 4 additional Star Wars models imported
- F10 mod menu improvements
- Performance optimizations

### v0.7.0

- First visual asset imports
- Asset swap Phase 1 and Phase 2
- Hot reload for YAML content

See [CHANGELOG.md](/CHANGELOG.md) for full version history.

---

## Related Documentation

- **[Product Requirements Document](../product-requirements-document.md)**: High-level product goals and vision
- **[QA Matrix](../QA_MATRIX.md)**: Testing tiers and quality gates
- **[Project Status](../guide/project-status.md)**: Current release status and roadmap
- **[Contributing Guidelines](../../CONTRIBUTING.md)**: How to contribute to DINOForge

---

## Support

- **Discord**: [DINOForge Community](https://discord.gg/dino-forge)
- **GitHub Issues**: [Report bugs](https://github.com/KooshaPari/Dino/issues)
- **Discussions**: [Community Q&A](https://github.com/KooshaPari/Dino/discussions)

---

**Last Updated**: 2026-03-20
**Status**: Active Documentation
**Audience**: All Users and Developers
