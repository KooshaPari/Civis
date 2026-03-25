# Architecture

DINOForge is structured as a three-product architecture with clear layering boundaries.

## System Overview

```mermaid
graph TD
    A[DINO Game<br/>Unity ECS / DOTS] --> B[Runtime Layer<br/>BepInEx Bootstrap]
    B --> C[SDK Layer<br/>Registries + Schemas + Pack Loader]
    C --> D[Domain Plugins<br/>Warfare, Economy, Scenario, UI]
    D --> E[Content Packs<br/>YAML Manifests + Assets]

    style A fill:#1a1a2e,stroke:#e94560,color:#fff
    style B fill:#1a1a2e,stroke:#0f3460,color:#fff
    style C fill:#1a1a2e,stroke:#16213e,color:#fff
    style D fill:#1a1a2e,stroke:#533483,color:#fff
    style E fill:#1a1a2e,stroke:#2b9348,color:#fff
```

## Runtime Execution Flow

The runtime bootstrap follows a precise sequence to initialize the mod platform:

```mermaid
graph TD
    Start["BepInEx Initializes<br/>Plugin.OnEnable"] --> Step1["Plugin.cs forwards<br/>Unity callbacks to RuntimeDriver"]
    Step1 --> Step2["RuntimeDriver.Update<br/>Frame 0: Detect ECS World"]
    Step2 --> Step3["RuntimeDriver.OnDestroy<br/>called at frame ~1"]
    Step3 --> Step4["ModPlatform.Initialize<br/>Create root + load SDK"]
    Step4 --> Step5["Root marked: HideAndDontSave<br/>+ DontDestroyOnLoad"]
    Step5 --> Step6["RuntimeDriver resurrected in OnDestroy<br/>via DontDestroyOnLoad marker"]
    Step6 --> Step7["ModPlatform.OnWorldReady<br/>Load packs, enable HMR, start UI"]
    Step7 --> Step8["F9/F10 overlays active<br/>across scene reloads"]

    style Start fill:#e94560,stroke:#fff,color:#fff
    style Step3 fill:#ff6b6b,stroke:#fff,color:#fff
    style Step5 fill:#2b9348,stroke:#fff,color:#fff
    style Step6 fill:#0f3460,stroke:#fff,color:#fff
    style Step8 fill:#533483,stroke:#fff,color:#fff
```

## Three Products

### Product A — Runtime / Hook Layer

The lowest level. Most brittle, fewest agents should touch.

- Boots into DINO via BepInEx + modified Doorstop pre-loader
- Locates ECS systems, entities, components, assets
- Exposes safe patch points
- Handles version checks and rollback/fallback
- Provides debug surfaces (entity dumper, system enumerator, debug overlay)

**Output**: `BepInEx/ecs_plugins/DINOForge.Runtime.dll`

### Product B — Mod API / Domain SDK

The real scaffold. Where trivial modding becomes possible.

- High-level mod definition interfaces
- Hides engine internals behind stable abstractions
- Provides schemas, registries, validators, pack loaders
- Supports multiple mod classes (content, balance, ruleset, total conversion, utility)

**Key components**: Pack manifest model, YAML loader (YamlDotNet), dependency resolver, schema validation (JsonSchema.Net)

### Product C — Mod Packs / Content Packs

Where actual mods live. Mostly declarative and content-driven.

- Warfare packs (Modern, Star Wars)
- Balance packs, economy packs
- QoL/UI packs, debug packs
- Each pack has explicit metadata: id, version, dependencies, conflicts

## Content Layering Model

DINOForge applies a 5-layer content model:

```mermaid
graph LR
    L1[Layer 1<br/>Content Packs<br/>YAML/JSON data] --> L2[Layer 2<br/>Asset Packs<br/>Icons, audio, VFX]
    L2 --> L3[Layer 3<br/>Code Plugins<br/>C# via SDK API]
    L3 --> L4[Layer 4<br/>Patch Layer<br/>Harmony / unsafe]
    L4 --> L5[Layer 5<br/>Tooling<br/>CLI, validators]

    style L1 fill:#2b9348,stroke:#fff,color:#fff
    style L2 fill:#1b7340,stroke:#fff,color:#fff
    style L3 fill:#533483,stroke:#fff,color:#fff
    style L4 fill:#e94560,stroke:#fff,color:#fff
    style L5 fill:#0f3460,stroke:#fff,color:#fff
```

| Layer | Description | Frequency |
|-------|-------------|-----------|
| Content Packs | YAML/JSON data — factions, units, stats, waves, localization | Most mods |
| Asset Packs | Bundled art/audio/prefab with manifests | Visual/audio mods |
| Code Plugins | C# plugin API through SDK interfaces | Advanced mods |
| Patch Layer | Controlled Harmony patches (marked unsafe) | Rare |
| Tooling | CLI tools, validators, inspectors | Development |

## Design Principles

1. **Wrap, don't handroll** — Use established libraries, wrap them thinly
2. **Framework before content** — Platform first, themed mods second
3. **Declarative before imperative** — YAML manifests over C# patches
4. **Stable abstraction over unstable internals** — Isolate ECS glue
5. **Agent-first repo design** — Optimize for autonomous agent development
6. **Observability is first-class** — Logs, overlays, reports, validators
7. **Domain extensibility** — Warfare is first plugin, not the only one
8. **Compatibility-aware packaging** — Explicit deps, conflicts, versions
9. **Graceful degradation** — Fail loudly with fallbacks

## Repository Layout

```
DINOForge/
  src/
    Runtime/             # Product A — BepInEx plugin
      Bridge/            #   ECS bridge (component mapping, stat modifiers, queries)
      HotReload/         #   Hot reload bridge
      UI/                #   Mod menu overlay (F10), settings panel
    SDK/                 # Product B — Public mod API
      Assets/            #   Asset service, addressables catalog
      Dependencies/      #   Dependency resolver
      HotReload/         #   Pack file watcher
      Models/            #   Content data models
      Registry/          #   Generic registry with conflict detection
      Universe/          #   Universe Bible system
      Validation/        #   Schema validation (NJsonSchema)
    Bridge/
      Protocol/          #   JSON-RPC types, IGameBridge interface
      Client/            #   Out-of-process game client
    Domains/
      Warfare/           #   Warfare domain (factions, doctrines, combat)
      Economy/           #   Economy domain (rates, trade, balance)
      Scenario/          #   Scenario domain (scripting, conditions)
      UI/                #   UI domain (HUD injection, menus)
    Tools/
      Cli/               #   dinoforge CLI (11 commands)
      McpServer/         #   MCP server for Claude Code (17 tools)
      PackCompiler/      #   Pack compiler (validate, build)
      DumpTools/         #   Dump analysis (Spectre.Console)
      Installer/         #   BepInEx + DINOForge installer
    Tests/               #   Unit tests (xUnit + FluentAssertions)
      Integration/       #   Integration tests
  packs/                 # Product C — Content packs (6 example packs)
  schemas/               # Canonical schema definitions (17 schemas)
  docs/                  # This documentation site
  manifests/             # Ownership map, extension points
```

## The Two-Boot Cycle

A critical runtime pattern ensures the mod platform survives scene reloads and maintains persistent state.

### Why It Exists

DINO's game flow causes the Doorstop pre-loader to initialize **twice** per playthrough:
1. **Boot 1**: Game launcher → load BepInEx → load Runtime plugin
2. **Intermediate**: Scene loads, ECS world initializes
3. **Boot 2**: Scene transition (or new game → continue) → Doorstop re-runs → must NOT double-initialize

### Mechanism: HideAndDontSave + DontDestroyOnLoad

The root GameObject persists across scene reloads via:

1. **HideAndDontSave flag** — Mark runtime root as not player-saveable
2. **DontDestroyOnLoad marker** — Persist from Boot 1 → Boot 2
3. **RuntimeDriver.OnDestroy resurrection** — If root destroyed by accident, create new one from marker
4. **ModPlatform singleton pattern** — Only one instance ever exists; subsequent boots detect via static reference

### Why Harmony Patches Failed

Earlier versions used Harmony patches to manipulate the lifecycle. This backfired because:
- **LazyPatch** intercepted object creation with custom logic
- **DeltaTimeResurrectionPatch** fought with the natural DontDestroyOnLoad flow
- Patch state leaked across scene boundaries
- Framework beat the patches; patches beat DontDestroyOnLoad

**Resolution** (commit df3b55e): Removed all patches, trusted the native HideAndDontSave + DontDestroyOnLoad mechanism.

## Reference Models

DINOForge draws from the best modding ecosystems:

| System | What We Take |
|--------|-------------|
| Factorio | API shape, manifests, dependency/version handling |
| RimWorld | Declarative content + imperative code escape hatch |
| Satisfactory/BepInEx | Mod loaders, plugin bootstrap |
| Minecraft Bedrock | Pack schemas, folder conventions |
| UEFN/Roblox | End-to-end creation pipeline concept |

## Products

### Product A: Desktop Companion (WinUI 3)

A standalone Windows desktop GUI that mirrors the in-game F9/F10 overlays. Allows pack configuration and debugging without launching the game.

- **UI Framework**: WinUI 3 + Mica background
- **Shell**: NavigationView with pack list and debug panel tabs
- **State Parity**: Reads/writes `disabled_packs.json` shared with game runtime
- **Use Case**: Configure packs, test pack compatibility, debug issues before launching game
- **Status**: In Progress (M9)

See ADR-011 for detailed design.
