# DINOForge — Comprehensive Specification

**Document ID:** PHENOTYPE_DINO_SPEC_001  
**Status:** Active  
**Last Updated:** 2026-04-04  
**Author:** Phenotype Architecture Team

---

## Table of Contents

1. [Project Overview](#1-project-overview)
2. [Vision & Principles](#2-vion--principles)
3. [User Personas](#3-user-personas)
4. [Architecture](#4-architecture)
5. [Runtime Layer Specification](#5-runtime-layer-specification)
6. [SDK Layer Specification](#6-sdk-layer-specification)
7. [Domain Plugin Specification](#7-domain-plugin-specification)
8. [Pack System Specification](#8-pack-system-specification)
9. [Registry System Specification](#9-registry-system-specification)
10. [Schema Validation Specification](#10-schema-validation-specification)
11. [ECS Bridge Specification](#11-ecs-bridge-specification)
12. [Asset Pipeline Specification](#12-asset-pipeline-specification)
13. [Hot Reload Specification](#13-hot-reload-specification)
14. [MCP Server Specification](#14-mcp-server-specification)
15. [Desktop Companion Specification](#15-desktop-companion-specification)
16. [CLI Tool Specification](#16-cli-tool-specification)
17. [API Reference](#17-api-reference)
18. [Error Handling](#18-error-handling)
19. [Security](#19-security)
20. [Performance Requirements](#20-performance-requirements)
21. [Testing Strategy](#21-testing-strategy)
22. [CI/CD Pipeline](#22-cicd-pipeline)
23. [Deployment](#23-deployment)
24. [Migration & Versioning](#24-migration--versioning)
25. [Glossary](#25-glossary)

---

## 1. Project Overview

### 1.1 Product Name

**DINOForge**

### 1.2 Product Definition

DINOForge is a **general-purpose mod operating system** for *Diplomacy is Not an Option* (DINO), a Unity ECS-based real-time strategy game. It is not a single mod — it is a **framework, SDK, pack system, and tooling platform** that enables the creation, distribution, and management of any type of mod for DINO.

### 1.3 Target Game

| Property | Value |
|----------|-------|
| **Game** | Diplomacy is Not an Option |
| **Engine** | Unity (ECS/DOTS) |
| **Entities** | 45K+ dumped entities |
| **Mod Loader** | BepInEx 5.4.x (ECS variant) |
| **Steam App ID** | 1272320 |

### 1.4 Key Capabilities

| Capability | Description |
|------------|-------------|
| **Pack System** | YAML-first declarative content packs with dependency resolution, conflict detection, and schema validation |
| **Typed Registries** | Units, buildings, factions, weapons, projectiles, doctrines, skills, waves, squads with layered override priority |
| **ECS Bridge** | Maps mod content to DINO's actual Unity ECS components at runtime (30+ component mappings) |
| **Asset Pipeline** | Full import → validate → optimize → LOD → prefab → Addressables pipeline; 38 catalog entries with 3-level LOD |
| **Pack Submodule Management** | Add/list/update/lock git submodule packs via CLI and Desktop Companion |
| **Asset Browser & Mod Manager** | Desktop Companion with visual asset browser, mod conflict detection, and update management |
| **Asset Library & Catalog** | SQLite asset catalog with source adapters and CLI asset-library commands |
| **Warfare Domain** | Faction archetypes (Order, Industrial Swarm, Asymmetric), doctrines, unit role validation, wave composition, balance calculation |
| **Dev Tooling** | PackCompiler CLI, DumpTools, in-game debug overlay, entity dumper |
| **MCP Server** | Game automation and analysis tools (16+ tools for Claude Code integration) |
| **Schema Validation** | 17 JSON schemas catch errors before runtime |
| **Fuzzing** | FsCheck property-based testing (30+ properties) + SharpFuzz coverage-guided fuzzing |
| **Hot Reload** | File watcher + manual F10 trigger for YAML pack changes without game restart |
| **Multi-Instance** | Parallel game instances with automated orchestration (planned) |

### 1.5 Milestone Status

| # | Milestone | Description | Status |
|---|-----------|-------------|--------|
| M0 | Reverse-Engineering Harness | Entity dumps, 45K entities | ✅ Done |
| M1 | Runtime Scaffold | BepInEx plugin, ECS systems | ✅ Done |
| M2 | Generic Mod SDK | Registries, schemas, ContentLoader | ✅ Done |
| M3 | Dev Tooling | PackCompiler, DumpTools, DebugOverlay | ✅ Done |
| M4 | Warfare Domain | Archetypes, doctrines, roles, waves, balance | ✅ Done |
| M5 | Example Packs | warfare-starwars, warfare-aerial, warfare-guerrilla, warfare-modern | ✅ Done |
| M6 | In-Game Mod Menu + HMR | F9/F10, hot reload | ✅ Done |
| M7 | Installer + Universe Bible | PowerShell/Bash installer, Universe Bible | ✅ Done |
| M8 | Runtime Integration | ModPlatform, ECS bridge, asset swap | ✅ Done |
| M9 | Desktop Companion | WinUI 3, Mica, pack manager | ✅ Done |
| M10 | Fuzzing | FsCheck 30+ props, SharpFuzz, corpus, nightly CI | ✅ Done |
| M11 | Test Coverage + Code Completion | 1017+ tests | ✅ Done |
| M12 | Pack Submodule Management | PackSubmoduleManager, CLI pack add/list/update/lock | ✅ Done |
| M13 | Asset Browser + Mod Manager | Asset Browser page, Browse/Update/Conflict views | ✅ Done |
| M14 | Asset Library & Catalog | SQLite AssetCatalogStore, asset-library CLI, LocalSourceAdapter | ✅ Done |

**Current test count: 1,017+ passing**

### 1.6 Project Structure

```
DINOForge/
├── src/
│   ├── Runtime/                    # Product A — BepInEx plugin
│   │   ├── Plugin.cs               # BepInEx entry point
│   │   ├── ModPlatform.cs          # Game lifecycle hooks
│   │   ├── Bridge/
│   │   │   ├── ComponentMap.cs     # Vanilla ↔ mod component mapping
│   │   │   ├── EntityQueries.cs    # ECS query helpers
│   │   │   ├── StatModifierSystem.cs # Runtime stat override system
│   │   │   └── VanillaCatalog.cs   # Vanilla unit/building data
│   │   ├── HotReload/
│   │   │   ├── HotReloadBridge.cs  # File watcher and reload trigger
│   │   │   └── ModuleState.cs      # Per-pack reload state
│   │   └── UI/
│   │       └── DebugOverlay.cs     # In-game F10 menu
│   │
│   ├── SDK/                        # Product B — Public mod API
│   │   ├── Registry/
│   │   │   ├── TypedRegistry.cs    # Generic registry base
│   │   │   ├── UnitRegistry.cs
│   │   │   ├── BuildingRegistry.cs
│   │   │   └── FactionRegistry.cs
│   │   ├── Models/
│   │   │   ├── Unit.cs
│   │   │   ├── Building.cs
│   │   │   ├── Faction.cs
│   │   │   └── *.cs                # Data models
│   │   ├── Validation/
│   │   │   ├── SchemaValidator.cs  # JSON Schema validation
│   │   │   └── PackValidator.cs    # Pack integrity
│   │   ├── Assets/
│   │   │   ├── AddressablesCatalog.cs
│   │   │   └── AssetSwapService.cs
│   │   ├── Dependencies/
│   │   │   └── DependencyResolver.cs # Topological sort
│   │   ├── Universe/               # Universe Bible system
│   │   ├── HotReload/              # Pack file watcher
│   │   └── ContentLoader.cs        # Pack loading orchestration
│   │
│   ├── Bridge/
│   │   ├── Protocol/               # JSON-RPC types, IGameBridge
│   │   └── Client/                 # Out-of-process game client
│   │
│   ├── Domains/
│   │   ├── Warfare/                # Warfare domain plugin
│   │   │   ├── Archetypes/         # Unit archetypes
│   │   │   ├── Doctrines/          # Combat doctrines
│   │   │   ├── Roles/              # Unit role system
│   │   │   ├── Waves/              # Wave scripting
│   │   │   └── Balance/            # Balance parameters
│   │   ├── Economy/                # Economy domain (planned)
│   │   ├── Scenario/               # Scenario domain (planned)
│   │   └── UI/                     # UI domain (planned)
│   │
│   ├── Tools/
│   │   ├── PackCompiler/           # CLI: validate, build
│   │   ├── DumpTools/              # Entity dump analysis
│   │   ├── McpServer/              # MCP protocol server
│   │   │   ├── McpServer.cs
│   │   │   ├── GameBridge.cs
│   │   │   └── Tools/              # Tool implementations
│   │   ├── Cli/                    # CLI commands
│   │   ├── Installer/              # BepInEx + DINOForge installer
│   │   └── Templates/              # Pack templates
│   │
│   └── Tests/                      # xUnit + FluentAssertions
│       ├── Unit/                   # Unit tests
│       ├── Integration/            # Integration tests
│       └── Fixtures/               # Test data and mocks
│
├── packs/                          # Product C — Content packs
│   ├── warfare-starwars/           # 28 units, 10 buildings
│   ├── warfare-modern/             # Modern military
│   ├── warfare-guerrilla/          # Asymmetric warfare
│   └── example-balance/            # Balance tweaks
│
├── schemas/                        # JSON Schema definitions (17 schemas)
├── docs/                           # VitePress documentation site
├── scripts/                        # PowerShell/bash automation
├── manifests/                      # Ownership map, extension points
└── .claude/                        # Claude Code configuration
```

---

## 2. Vision & Principles

### 2.1 Vision Statement

Create the canonical mod platform for DINO that transforms brittle one-off reverse-engineered hacks into a structured, extensible, testable, agent-operable ecosystem.

### 2.2 Product Principles

| # | Principle | Description |
|---|-----------|-------------|
| P1 | **Wrap, don't handroll** | Use established libraries and wrap them thinly. Every handrolled component is a liability; every wrapped dependency is borrowed reliability. |
| P2 | **Framework before content** | The first product is the platform, not the themed mod. |
| P3 | **Declarative before imperative** | Prefer pack manifests, schemas, mappings, and registries over custom patch code. |
| P4 | **Stable abstraction over unstable internals** | Low-level engine glue must be isolated from mod authoring surfaces. |
| P5 | **Agent-first repository design** | The codebase and docs must optimize for autonomous agent development. |
| P6 | **Observability is first-class** | Runtime must explain itself through logs, overlays, reports, validators. |
| P7 | **Domain extensibility** | Warfare is the first domain plugin, not the only one. |
| P8 | **Compatibility-aware packaging** | Mods must be packs with explicit dependencies, conflicts, versions. |
| P9 | **Graceful degradation** | Missing assets/broken mappings fail loudly with fallbacks where safe. |

### 2.3 Development Methodologies

| Methodology | Description |
|-------------|-------------|
| **SDD** (Spec-Driven Development) | Specifications drive the pipeline |
| **BDD** (Behavior-Driven Development) | Acceptance criteria before implementation |
| **TDD** (Test-Driven Development) | Unit tests for all public APIs |
| **DDD** (Domain-Driven Design) | Bounded contexts (Warfare, Economy, Scenario) |
| **ADD** (Agent-Driven Development) | Fully agent-authored codebase |
| **CDD** (Contract-Driven Development) | Schemas as contracts between packs and engine |

---

## 3. User Personas

### 3.1 Primary: Product Owner / Mod Director

**Needs:**
- Request features in natural language
- Avoid reading source code
- Receive clear diagnostics when things fail
- Iterate on gameplay, balance, and theming quickly
- Add new mod concepts without fresh reverse engineering each time

**Pain Points:**
- Traditional modding requires deep engine knowledge
- Debugging mod issues is time-consuming
- Coordinating multiple mod components is complex

### 3.2 Secondary: Autonomous Coding Agents

**Needs:**
- Clear public APIs with XML doc comments
- Typed schemas for all data shapes
- Examples and templates for common patterns
- Deterministic build/test flows
- Bounded ownership areas (agent roster)
- Machine-readable contracts
- Debugging tools and reports

**Pain Points:**
- Ambiguous API contracts
- Missing test fixtures
- Unclear file ownership boundaries

### 3.3 Tertiary: End Users / Players

**Needs:**
- Install packs safely
- Understand compatibility and conflicts
- Get stable gameplay behavior
- Receive understandable errors when packs fail

**Pain Points:**
- Manual mod installation is error-prone
- Conflicting mods cause crashes
- No clear indication of which packs are active

---

## 4. Architecture

### 4.1 Three-Product Architecture

DINOForge is structured as three distinct products with clear layering boundaries:

```
┌─────────────────────────────────────────────────────────────────────┐
│                     DINOForge Architecture Stack                     │
│                                                                      │
│  ┌──────────────────────────────────────────────────────────────┐   │
│  │  Product C: Content Packs                                    │   │
│  │  ┌──────────────────────────────────────────────────────┐   │   │
│  │  │  warfare-starwars/  # 28 units, 10 buildings         │   │   │
│  │  │  warfare-modern/    # Modern military units           │   │   │
│  │  │  warfare-guerrilla/ # Asymmetric warfare              │   │   │
│  │  │  example-balance/   # Simple balance tweaks           │   │   │
│  │  │                                                         │   │   │
│  │  │  Each pack:                                             │   │   │
│  │  │  ├── pack.yaml (manifest)                              │   │   │
│  │  │  ├── units/ (unit definitions)                         │   │   │
│  │  │  ├── buildings/ (building definitions)                 │   │   │
│  │  │  └── assets/ (textures, models, audio)                 │   │   │
│  │  └──────────────────────────────────────────────────────┘   │   │
│  └──────────────────────────────────────────────────────────────┘   │
│                                  │                                   │
│                                  ▼                                   │
│  ┌──────────────────────────────────────────────────────────────┐   │
│  │  Domain Plugins (Extensible)                                 │   │
│  │  ┌──────────────────────────────────────────────────────┐   │   │
│  │  │  Warfare/                                              │   │   │
│  │  │  ├── Archetypes/ (infantry, ranged, cavalry, artillery) │   │   │
│  │  │  ├── Doctrines/ (combat bonuses, tech trees)           │   │   │
│  │  │  ├── Roles/ (tank, dps, support, scout)                │   │   │
│  │  │  ├── Waves/ (spawn patterns, difficulty scaling)       │   │   │
│  │  │  └── Balance/ (formulas, resource costs)               │   │   │
│  │  │                                                         │   │   │
│  │  │  Economy/ (planned)                                     │   │   │
│  │  │  Scenario/ (planned)                                    │   │   │
│  │  │  UI/ (planned)                                          │   │   │
│  │  └──────────────────────────────────────────────────────┘   │   │
│  └──────────────────────────────────────────────────────────────┘   │
│                                  │                                   │
│                                  ▼                                   │
│  ┌──────────────────────────────────────────────────────────────┐   │
│  │  Product B: SDK Layer (Public API)                           │   │
│  │  ┌──────────────────────────────────────────────────────┐   │   │
│  │  │  Registry/ (TypedRegistry<T>, UnitRegistry, etc.)     │   │   │
│  │  │  Models/ (Unit, Building, Faction, StatBlock, etc.)   │   │   │
│  │  │  Validation/ (SchemaValidator, PackValidator)         │   │   │
│  │  │  Assets/ (AddressablesCatalog, AssetSwapService)      │   │   │
│  │  │  Dependencies/ (DependencyResolver)                   │   │   │
│  │  │  Universe/ (UniverseBible, UniverseLoader)            │   │   │
│  │  │  ContentLoader.cs (pack loading orchestration)        │   │   │
│  │  └──────────────────────────────────────────────────────┘   │   │
│  └──────────────────────────────────────────────────────────────┘   │
│                                  │                                   │
│                                  ▼                                   │
│  ┌──────────────────────────────────────────────────────────────┐   │
│  │  Product A: Runtime Layer (BepInEx Plugin)                   │   │
│  │  ┌──────────────────────────────────────────────────────┐   │   │
│  │  │  Plugin.cs (BepInEx entry point)                      │   │   │
│  │  │  ModPlatform.cs (game lifecycle hooks)                │   │   │
│  │  │                                                         │   │   │
│  │  │  Bridge/                                                │   │   │
│  │  │  ├── ComponentMap.cs (vanilla ↔ mod mapping)           │   │   │
│  │  │  ├── EntityQueries.cs (ECS query helpers)              │   │   │
│  │  │  ├── StatModifierSystem.cs (runtime stat overrides)    │   │   │
│  │  │  └── VanillaCatalog.cs (vanilla data mirror)           │   │   │
│  │  │                                                         │   │   │
│  │  │  HotReload/                                             │   │   │
│  │  │  ├── HotReloadBridge.cs (file watcher, reload trigger)  │   │   │
│  │  │  └── ModuleState.cs (per-pack reload state)            │   │   │
│  │  │                                                         │   │   │
│  │  │  UI/                                                    │   │   │
│  │  │  └── DebugOverlay.cs (F10 debug menu)                  │   │   │
│  │  └──────────────────────────────────────────────────────┘   │   │
│  └──────────────────────────────────────────────────────────────┘   │
│                                  │                                   │
│                                  ▼                                   │
│  ┌──────────────────────────────────────────────────────────────┐   │
│  │  Game: Diplomacy is Not an Option                            │   │
│  │  └── Unity ECS (Entities 1.0.16)                             │   │
│  │      ├── Unit entities (Health, Position, UnitId)            │   │
│  │      ├── Building entities (Health, Position, BuildingId)    │   │
│  │      └── Game systems (combat, economy, AI)                  │   │
│  └──────────────────────────────────────────────────────────────┘   │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

### 4.2 Layer Communication

```
Content Packs ──YAML──► ContentLoader ──Registry──► ECS Bridge ──Components──► Game
     │                      │                       │
     │                      ▼                       ▼
     │               SchemaValidator          StatModifierSystem
     │                      │                       │
     │                      ▼                       ▼
     │               ValidationResult        Component Updates
     │                                              │
     ▼                                              ▼
  PackCompiler                              DebugOverlay (F10)
  (validate/build)                          (observability)
```

### 4.3 Content Layering Model

DINOForge applies a 5-layer content model:

| Layer | Description | Frequency | Target User |
|-------|-------------|-----------|-------------|
| **Content Packs** | YAML/JSON data — factions, units, stats, waves, localization | Most mods | Pack authors |
| **Asset Packs** | Bundled art/audio/prefab with manifests | Visual/audio mods | Asset creators |
| **Code Plugins** | C# plugin API through SDK interfaces | Advanced mods | Developers |
| **Patch Layer** | Controlled Harmony patches (marked unsafe) | Rare | Runtime specialists |
| **Tooling** | CLI tools, validators, inspectors | Development | All users |

### 4.4 Registry Priority Layers

```
Priority 4000+
┌─────────────────────────────────────────────────────────────┐
│  User Override (runtime edits, debug tweaks)                 │
└─────────────────────────────────────────────────────────────┘

Priority 3000+
┌─────────────────────────────────────────────────────────────┐
│  Content Packs (mod content overrides)                       │
│  ├── warfare-starwars: priority 3000                         │
│  ├── warfare-modern: priority 3001                           │
│  └── example-balance: priority 3500 (explicit)               │
└─────────────────────────────────────────────────────────────┘

Priority 2000+
┌─────────────────────────────────────────────────────────────┐
│  Domain Plugins (Warfare/Economy defaults)                   │
│  ├── Archetype defaults (infantry: 100 HP base)              │
│  └── Doctrine modifiers (+20% damage vs specific types)      │
└─────────────────────────────────────────────────────────────┘

Priority 1000+
┌─────────────────────────────────────────────────────────────┐
│  Framework Defaults (DINOForge baseline)                     │
│  └── Default stat blocks, fallback values                    │
└─────────────────────────────────────────────────────────────┘

Priority 0+
┌─────────────────────────────────────────────────────────────┐
│  Base Game (vanilla DINO values)                             │
│  └── Original unit/building stats from game files            │
└─────────────────────────────────────────────────────────────┘

Resolution Rule: Higher priority wins. Same priority = conflict error.
```

### 4.5 The Two-Boot Cycle

DINO's game flow causes the Doorstop pre-loader to initialize **twice** per playthrough:

1. **Boot 1**: Game launcher → load BepInEx → load Runtime plugin
2. **Intermediate**: Scene loads, ECS world initializes
3. **Boot 2**: Scene transition (or new game → continue) → Doorstop re-runs → must NOT double-initialize

**Mechanism: HideAndDontSave + DontDestroyOnLoad**

1. **HideAndDontSave flag** — Mark runtime root as not player-saveable
2. **DontDestroyOnLoad marker** — Persist from Boot 1 → Boot 2
3. **RuntimeDriver.OnDestroy resurrection** — If root destroyed by accident, create new one from marker
4. **ModPlatform singleton pattern** — Only one instance ever exists; subsequent boots detect via static reference

---

## 5. Runtime Layer Specification

### 5.1 Plugin Bootstrap

**Entry Point:** `src/Runtime/Plugin.cs`

```csharp
[BepInPlugin(PluginGuid, PluginName, PluginVersion)]
[BepInProcess("DINO.exe")]
public class Plugin : BaseUnityPlugin
{
    public const string PluginGuid = "com.dinoforge.runtime";
    public const string PluginName = "DINOForge Runtime";
    public const string PluginVersion = "0.5.0";

    private void Awake()
    {
        // Forward Unity callbacks to RuntimeDriver
        RuntimeDriver.Initialize(this);
    }
}
```

**Responsibilities:**
- BepInEx plugin registration
- Unity callback forwarding to RuntimeDriver
- Version detection and logging
- Plugin lifecycle management

### 5.2 ModPlatform Orchestrator

**Location:** `src/Runtime/ModPlatform.cs`

**Responsibilities:**
- Game lifecycle hooks (world ready, scene changes)
- Pack loading orchestration
- Hot reload enablement
- UI initialization (F9/F10 overlays)
- ECS bridge synchronization

**Lifecycle Flow:**

```
BepInEx Initializes (Plugin.OnEnable)
  │
  ▼
Plugin.cs forwards Unity callbacks to RuntimeDriver
  │
  ▼
RuntimeDriver.Update — Frame 0: Detect ECS World
  │
  ▼
RuntimeDriver.OnDestroy — called at frame ~1
  │
  ▼
ModPlatform.Initialize — Create root + load SDK
  │
  ▼
Root marked: HideAndDontSave + DontDestroyOnLoad
  │
  ▼
RuntimeDriver resurrected in OnDestroy via DontDestroyOnLoad marker
  │
  ▼
ModPlatform.OnWorldReady — Load packs, enable HMR, start UI
  │
  ▼
F9/F10 overlays active across scene reloads
```

### 5.3 ECS Bridge

**Location:** `src/Runtime/Bridge/`

#### ComponentMap

Maps vanilla ECS components to mod-aware equivalents:

| Vanilla Component | DINOForge Bridge Component | Sync Direction | Frequency |
|-------------------|---------------------------|----------------|-----------|
| Health | StatOverrideComponent | Mod → Game | Per-frame |
| UnitId | ModUnitDefinitionComponent | Game → Mod | On-spawn |
| Position | (read-only query) | Game → Mod | On-demand |
| MeshRenderer | AssetSwapComponent | Mod → Game | On-load |
| Material | AssetSwapComponent | Mod → Game | On-load |

#### EntityQueries

ECS query helpers for mod content:

```csharp
public static class EntityQueries
{
    public static IEnumerable<Entity> GetUnitsByFaction(
        EntityManager em, string factionId)
    {
        var query = em.CreateEntityQuery(
            typeof(UnitIdComponent),
            typeof(HealthComponent),
            typeof(PositionComponent));

        var entities = query.ToEntityArray(Allocator.Temp);

        foreach (var entity in entities)
        {
            var unitId = em.GetComponentData<UnitIdComponent>(entity);
            if (unitId.FactionId == factionId)
            {
                yield return entity;
            }
        }

        entities.Dispose();
    }
}
```

#### StatModifierSystem

Runtime stat override system:

```csharp
[BurstCompile]
public partial struct StatModifierSystem : ISystem
{
    public void OnUpdate(ref SystemState state)
    {
        foreach (var (health, modDef, overrides) in SystemAPI
            .Query<RefRW<HealthComponent>,
                   RefRO<ModUnitDefinitionComponent>,
                   RefRO<StatOverrideComponent>>())
        {
            var baseHealth = modDef.ValueRO.Definition.Stats.Health;
            var overrideValue = overrides.ValueRO.GetOverride("health");

            health.ValueRW.Max = overrideValue ?? baseHealth;
        }
    }
}
```

#### VanillaCatalog

Mirror of vanilla game data for reference and fallback:

```csharp
public static class VanillaCatalog
{
    public static IReadOnlyDictionary<string, UnitDefinition> Units { get; private set; }
    public static IReadOnlyDictionary<string, BuildingDefinition> Buildings { get; private set; }

    public static void Initialize(EntityManager entityManager)
    {
        // Dump all vanilla entities and cache their definitions
        Units = DumpVanillaUnits(entityManager);
        Buildings = DumpVanillaBuildings(entityManager);
    }
}
```

### 5.4 Hot Reload Bridge

**Location:** `src/Runtime/HotReload/`

**Mechanisms:**
- **FileSystemWatcher**: Detects YAML changes in packs directory
- **Debouncer**: 500ms debounce to batch rapid changes
- **Manual Trigger**: F10 key for explicit reload
- **ModuleState**: Per-pack reload state tracking

**Reload Flow:**

```
File Change Detected
  │
  ▼
Debouncer (500ms)
  │
  ▼
ContentLoader.ReloadPack(packPath)
  │
  ├── Schema Validation
  ├── Dependency Check
  ├── Registry Update
  └── ECS Sync
  │
  ▼
Result: Success / Failure (with error details)
```

### 5.5 Debug Overlay

**Location:** `src/Runtime/UI/DebugOverlay.cs`

**Activation:** F10 key in-game

**Features:**
- Entity counts by type
- System state information
- Loaded pack list
- Error/warning log
- Performance metrics

---

## 6. SDK Layer Specification

### 6.1 Public API Surface

The SDK (`DINOForge.SDK`, target: `netstandard2.0`) provides the public mod API:

| Namespace | Purpose |
|-----------|---------|
| `DINOForge.SDK.Registry` | Typed registries for all content types |
| `DINOForge.SDK.Models` | Data models (Unit, Building, Faction, etc.) |
| `DINOForge.SDK.Validation` | Schema validation utilities |
| `DINOForge.SDK.Assets` | Asset management services |
| `DINOForge.SDK.Dependencies` | Dependency resolution |
| `DINOForge.SDK.Universe` | Universe Bible system |
| `DINOForge.SDK.HotReload` | Pack file watcher |

### 6.2 ContentLoader

**Location:** `src/SDK/ContentLoader.cs`

**Responsibilities:**
- Discover pack.yaml files in packs directory
- Parse YAML manifests
- Resolve dependencies (topological sort)
- Validate content against schemas
- Register content to typed registries
- Return load results with errors/warnings

**Load Pipeline:**

```
1. DISCOVERY
   └── Scan packs/ directory for pack.yaml files

2. PARSING
   └── Parse YAML manifests (id, version, dependencies, loads)

3. DEPENDENCY RESOLUTION
   └── Topological sort based on dependency graph
   └── Detect circular dependencies (error)
   └── Detect version conflicts (error)

4. VALIDATION
   └── Schema validation per content type
   └── Cross-reference validation (IDs exist)
   └── Asset existence check

5. REGISTRATION
   └── Register to TypedRegistry<T> with priority
   └── Apply overrides to existing entries
   └── Store source pack ID for conflict detection

6. BRIDGE SYNC
   └── Sync registered content to ECS components
   └── Spawn/refresh entity components
```

**Error Handling:**
- Validation failure → Skip pack, log error, continue
- Dependency missing → Error, halt load
- Conflict detected → Error, show in debug overlay

### 6.3 Data Models

#### UnitDefinition

```csharp
public record UnitDefinition
{
    public string Id { get; init; } = "";
    public string Name { get; init; } = "";
    public string Description { get; init; } = "";
    public string Archetype { get; init; } = "";
    public string Role { get; init; } = "";
    public string Faction { get; init; } = "";
    public VisualDefinition Visual { get; init; } = new();
    public StatBlock Stats { get; init; } = new();
    public CombatBlock Combat { get; init; } = new();
    public CostBlock Cost { get; init; } = new();
    public List<AbilityDefinition> Abilities { get; init; } = new();
    public List<UnitVariantDefinition> Variants { get; init; } = new();
}
```

#### BuildingDefinition

```csharp
public record BuildingDefinition
{
    public string Id { get; init; } = "";
    public string Name { get; init; } = "";
    public string Description { get; init; } = "";
    public string Faction { get; init; } = "";
    public VisualDefinition Visual { get; init; } = new();
    public StatBlock Stats { get; init; } = new();
    public CostBlock Cost { get; init; } = new();
    public ProductionBlock Production { get; init; } = new();
}
```

#### FactionDefinition

```csharp
public record FactionDefinition
{
    public string Id { get; init; } = "";
    public string Name { get; init; } = "";
    public string Description { get; init; } = "";
    public string Archetype { get; init; } = "";
    public List<string> Units { get; init; } = new();
    public List<string> Buildings { get; init; } = new();
    public List<string> Doctrines { get; init; } = new();
    public FactionColors Colors { get; init; } = new();
}
```

#### PackManifest

```csharp
public record PackManifest
{
    public string Id { get; init; } = "";
    public string Name { get; init; } = "";
    public string Version { get; init; } = "";
    public string Author { get; init; } = "";
    public string Type { get; init; } = "";
    public string FrameworkVersion { get; init; } = "";
    public string Description { get; init; } = "";
    public string Homepage { get; init; } = "";
    public string License { get; init; } = "";
    public int Priority { get; init; } = 3000;
    public PackLoadConfig Loads { get; init; } = new();
    public List<DependencyDefinition> Depends { get; init; } = new();
    public List<ConflictDefinition> Conflicts { get; init; } = new();
    public AssetConfig Assets { get; init; } = new();
}
```

---

## 7. Domain Plugin Specification

### 7.1 Domain Plugin Architecture

Domain plugins extend DINOForge with game-specific logic. Each domain plugin:

- References the SDK (`DINOForge.SDK`)
- Registers domain-specific content to registries
- Provides domain-specific schemas
- Implements domain-specific validation rules

**Current Domains:**

| Domain | Status | Content Types |
|--------|--------|---------------|
| **Warfare** | ✅ Production | Archetypes, Doctrines, Roles, Waves, Balance |
| **Economy** | 🔄 In Progress | Resources, Trade, Production |
| **Scenario** | 🔄 In Progress | Events, Conditions, Win/Loss |
| **UI** | 🔄 In Progress | Themes, HUD elements, Menus |

### 7.2 Warfare Domain

**Location:** `src/Domains/Warfare/`

#### Faction Archetypes

| Archetype | Traits | Used By |
|-----------|--------|---------|
| **Order** | Strong line infantry, reliable DPS, better defenses, higher unit cost | Republic, West |
| **Industrial Swarm** | Larger numbers, cheaper core, expendable, strong siege | CIS, Classic West Enemy |
| **Asymmetric** | Light units, mobility, ambush, raid pressure, structure harassment | Guerrilla West Enemy |

#### Doctrines

Doctrines provide combat bonuses and strategic modifiers:

```yaml
# doctrines/elite_discipline.doctrine.yaml
id: warfare:elite_discipline
name: "Elite Discipline"
description: "Units fight with superior coordination"

effects:
  - type: stat_modifier
    target: archetype:infantry
    stat: accuracy
    value: +15
    condition: formation == line

  - type: stat_modifier
    target: archetype:ranged
    stat: fire_rate
    value: +10
    condition: has_support_unit
```

#### Unit Roles

| Role | Description | Typical Stats |
|------|-------------|---------------|
| **Frontline** | Absorbs damage, holds the line | High HP, high armor, moderate DPS |
| **DPS** | Primary damage dealer | Low HP, high damage, moderate range |
| **Support** | Buffs allies, debuffs enemies | Low HP, aura effects, utility |
| **Scout** | Fast, low-cost reconnaissance | High speed, low HP, low cost |
| **Siege** | Structure destruction specialist | Very high damage vs buildings, slow |

#### Wave System

Waves define enemy composition and spawn patterns:

```yaml
# waves/wave_01.wave.yaml
id: starwars:wave_01
name: "CIS First Assault"
difficulty: easy

composition:
  - unit: cis:battle_droid
    count: 20
    spawn_delay: 0.5
  - unit: cis:droid_tank
    count: 3
    spawn_delay: 2.0
  - unit: cis:droideka
    count: 5
    spawn_delay: 1.0

spawn_pattern:
  type: staggered
  interval: 3.0
  location: enemy_base

objectives:
  - type: destroy_all
    timeout: 300
```

### 7.3 Domain Plugin Contract

```csharp
public interface IDomainPlugin
{
    string DomainId { get; }
    string DomainName { get; }
    string Version { get; }

    void RegisterRegistries(IRegistryCollection registries);
    void RegisterSchemas(ISchemaRegistry schemas);
    void RegisterValidators(IValidatorCollection validators);
    void Initialize(IContentLoadResult content);
}
```

---

## 8. Pack System Specification

### 8.1 Pack Manifest Schema

```yaml
# Required fields
id: warfare-starwars                    # Unique pack identifier
name: "Star Wars: Clone Wars"           # Display name
version: 1.0.0                          # Semantic version
author: DINOForge Team                  # Author name
type: total-conversion                  # Pack type
framework_version: ">=0.1.0"            # DINOForge compatibility

# Optional fields
description: "Republic vs CIS faction warfare"
homepage: https://github.com/KooshaPari/Dino
license: MIT

# Load configuration
loads:
  units:
    - units/           # Relative to pack root
  buildings:
    - buildings/
  factions:
    - factions/
  waves:
    - waves/

# Dependencies
depends:
  - id: dinoforge-core
    version: ">=0.1.0"
    optional: false
  - id: warfare-assets-base
    version: "~1.0.0"
    optional: true

# Conflicts (mutually exclusive packs)
conflicts:
  - id: warfare-fantasy
    reason: "Different theme setting"

# Priority for registry resolution (higher = wins)
priority: 3000

# Asset bundle configuration
assets:
  bundle_name: "warfare_starwars_bundle"
  addressables_group: "warfare-starwars"
```

### 8.2 Pack Types

| Type | Description | Example |
|------|-------------|---------|
| **content** | New units, buildings, factions | warfare-modern |
| **balance** | Stat adjustments, cost changes | example-balance |
| **ruleset** | Research, wave, victory condition changes | (planned) |
| **total-conversion** | Complete faction/theme replacement | warfare-starwars |
| **utility** | Debug tools, QoL improvements | (planned) |

### 8.3 Pack Loading Order

```
1. Discover all pack.yaml files
2. Parse manifests into PackManifest objects
3. Build dependency graph
4. Topological sort → load order
5. For each pack in order:
   a. Validate content against schemas
   b. Check dependencies are satisfied
   c. Check no conflicts with already-loaded packs
   d. Load content files (units/, buildings/, etc.)
   e. Register to TypedRegistry<T> with pack priority
   f. Track source pack for conflict detection
6. Sync registries to ECS bridge
7. Enable hot reload watcher
```

### 8.4 Pack Submodule Management

**CLI Commands:**
- `dinoforge pack add <url>` — Add git submodule pack
- `dinoforge pack list` — List all packs with status
- `dinoforge pack update <id>` — Update pack to latest
- `dinoforge pack lock <id> <commit>` — Lock pack to specific commit

**Lock File:** `packs.lock`

```yaml
# packs.lock
lockfile_version: 1
packs:
  - id: warfare-starwars
    path: packs/warfare-starwars
    url: https://github.com/KooshaPari/dinoforge-packs
    commit: abc123def456
    branch: main
    locked_at: 2026-04-04T12:00:00Z
```

---

## 9. Registry System Specification

### 9.1 Registry Architecture

```
TypedRegistry<T> (generic base)
├── UnitRegistry (IRegistry<UnitDefinition>)
├── BuildingRegistry (IRegistry<BuildingDefinition>)
├── FactionRegistry (IRegistry<FactionDefinition>)
├── WeaponRegistry (IRegistry<WeaponDefinition>)
├── ProjectileRegistry (IRegistry<ProjectileDefinition>)
├── DoctrineRegistry (IRegistry<DoctrineDefinition>)
├── SkillRegistry (IRegistry<SkillDefinition>)
├── WaveRegistry (IRegistry<WaveDefinition>)
└── SquadRegistry (IRegistry<SquadDefinition>)
```

### 9.2 Registry Interface

```csharp
public interface IRegistry<T> where T : class, IRegisteredContent
{
    void Register(string id, T content, int priority, string sourcePack);
    T? Get(string id);
    IReadOnlyDictionary<string, T> GetAll();
    IReadOnlyList<string> GetConflicts();
    bool Contains(string id);
    int Count { get; }
}
```

### 9.3 Priority Resolution

| Priority Range | Source | Example |
|---------------|--------|---------|
| 0-999 | Base game (vanilla DINO) | Original unit stats |
| 1000-1999 | Framework defaults | DINOForge baseline values |
| 2000-2999 | Domain plugins | Warfare archetype defaults |
| 3000-3999 | Content packs | Mod content overrides |
| 4000+ | User overrides | Runtime debug tweaks |

**Resolution Rule:** Higher priority wins. Same priority = conflict detected and reported.

### 9.4 Conflict Detection

```csharp
public IReadOnlyList<string> GetConflicts()
{
    return _entries
        .Where(kvp => kvp.Value.Count > 1
            && kvp.Value[0].Priority == kvp.Value[1].Priority)
        .Select(kvp => kvp.Key)
        .ToList();
}
```

Conflicts are reported in:
- Debug overlay (F10)
- PackCompiler validation output
- Desktop Companion conflict view
- CI/CD pipeline (blocking)

---

## 10. Schema Validation Specification

### 10.1 Schema Inventory

| Schema File | Content Type | Draft | Required Fields |
|-------------|-------------|-------|-----------------|
| `pack.schema.json` | Pack manifest | Draft 7 | id, name, version, author, type |
| `unit.schema.json` | Unit definition | Draft 7 | id, name, stats |
| `building.schema.json` | Building definition | Draft 7 | id, name, stats |
| `faction.schema.json` | Faction definition | Draft 7 | id, name, archetype |
| `weapon.schema.json` | Weapon definition | Draft 7 | id, name, damage |
| `projectile.schema.json` | Projectile definition | Draft 7 | id, name, speed |
| `doctrine.schema.json` | Doctrine definition | Draft 7 | id, name, effects |
| `skill.schema.json` | Skill definition | Draft 7 | id, name, effect |
| `wave.schema.json` | Wave definition | Draft 7 | id, name, composition |
| `squad.schema.json` | Squad definition | Draft 7 | id, name, units |
| `archetype.schema.json` | Unit archetype | Draft 7 | id, name, traits |
| `role.schema.json` | Unit role | Draft 7 | id, name, description |
| `ability.schema.json` | Ability definition | Draft 7 | id, name, effect |
| `variant.schema.json` | Unit variant | Draft 7 | id, name, stat_modifiers |
| `economy.schema.json` | Economy profile | Draft 7 | id, name, resources |
| `scenario.schema.json` | Scenario definition | Draft 7 | id, name, conditions |
| `universe.schema.json` | Universe Bible | Draft 7 | id, name, factions |

### 10.2 Validation Rules

| Rule | Severity | Description |
|------|----------|-------------|
| **id_format** | Error | Must match `^[a-z0-9_]+:[a-z0-9_]+$` |
| **required_fields** | Error | Required fields must be present |
| **reference_exists** | Error | Referenced IDs must exist in registry |
| **asset_exists** | Warning | Referenced asset files must exist |
| **stat_range** | Warning | Stats must be within reasonable bounds |
| **cost_balance** | Warning | Costs should follow archetype guidelines |
| **unique_ids** | Error | No duplicate IDs within a pack |
| **circular_deps** | Error | No circular dependencies between packs |
| **version_compat** | Error | framework_version must match current DINOForge |
| **conflict_check** | Error | No conflicts with loaded packs at same priority |

### 10.3 Validation Pipeline

```
Input: pack.yaml + content files
  │
  ▼
┌─────────────────────┐
│  YAML Parsing       │  YamlDotNet
│  → C# Objects       │
└────────┬────────────┘
         │
         ▼
┌─────────────────────┐
│  Schema Validation  │  NJsonSchema
│  (per content type) │  pack.schema.json, unit.schema.json, etc.
└────────┬────────────┘
         │
         ▼
┌─────────────────────┐
│  Cross-Reference    │  Check referenced IDs exist
│  Validation         │  Check asset files exist
└────────┬────────────┘
         │
         ▼
┌─────────────────────┐
│  Dependency         │  Resolve dependency graph
│  Validation         │  Detect circular deps
└────────┬────────────┘
         │
         ▼
┌─────────────────────┐
│  Conflict           │  Check priority conflicts
│  Detection          │  Check version compatibility
└────────┬────────────┘
         │
         ▼
  ValidationResult: Pass / Fail (with errors)
```

---

## 11. ECS Bridge Specification

### 11.1 Component Mapping

The ECS bridge maps DINOForge's mod content definitions to DINO's actual Unity ECS components:

| Mod Concept | Vanilla ECS Component | Bridge Component | Notes |
|-------------|----------------------|------------------|-------|
| Unit health | HealthComponent | StatOverrideComponent | Override at runtime |
| Unit identity | UnitIdComponent | ModUnitDefinitionComponent | Link to mod definition |
| Unit visuals | MeshRenderer | AssetSwapComponent | Replace prefab |
| Unit materials | MaterialProperty | AssetSwapComponent | Replace materials |
| Building health | HealthComponent | StatOverrideComponent | Same as units |
| Building identity | BuildingIdComponent | ModBuildingDefinitionComponent | Link to mod definition |
| Faction affiliation | FactionComponent | (read-only) | Query only |
| Position | LocalTransform | (read-only) | Query only |

### 11.2 Bridge Systems

| System | Purpose | Execution |
|--------|---------|-----------|
| **ModComponentInjectionSystem** | Adds mod components to entities | On entity creation |
| **StatOverrideSystem** | Applies stat overrides per frame | Every frame (Burst) |
| **AssetSwapSystem** | Replaces visuals on load | On pack load |
| **FactionSystem** | Manages faction-specific behavior | On faction change |
| **WaveInjectorSystem** | Injects mod waves into game | On wave trigger |
| **VanillaArchetypeMapper** | Maps vanilla units to mod archetypes | On pack load |

### 11.3 Performance Guarantees

| Metric | Target | Current | Measurement |
|--------|--------|---------|-------------|
| **ECS sync overhead** | <1ms/frame | ~0.3ms | 1000 entities |
| **Component query** | <1μs | ~0.5μs | Single lookup |
| **Stat application** | <0.5ms/1000 units | ~0.2ms | Per-frame batch |
| **Asset swap** | <100ms | ~50ms | Per prefab |

---

## 12. Asset Pipeline Specification

### 12.1 Pipeline Stages

```
Stage 1: Import
  Input: Source assets (FBX, PNG, WAV)
  Process: Format validation, metadata extraction
  Output: Validated source files
  Tool: assetctl import

Stage 2: Validate
  Input: Validated source files
  Process: Technical checks (poly count, UVs, texture size)
  Output: Validation report
  Tool: assetctl validate

Stage 3: Optimize
  Input: Validated source files
  Process: Compression, LOD generation, texture atlasing
  Output: Optimized assets
  Tool: assetctl optimize

Stage 4: Build
  Input: Optimized assets
  Process: Prefab creation, Addressables catalog
  Output: Addressables bundle
  Tool: assetctl build

Stage 5: Catalog
  Input: Addressables bundle
  Process: SQLite catalog entry, source adapter
  Output: AssetCatalogStore entry
  Tool: assetctl catalog
```

### 12.2 LOD Generation

| Level | Quality | Distance | Use Case |
|-------|---------|----------|----------|
| **LOD 0** | 100% | 0-30 units | Close-up, hero units |
| **LOD 1** | 60% | 30-60 units | Mid-range, standard view |
| **LOD 2** | 30% | 60+ units | Far view, mass battles |

### 12.3 Addressables Catalog

```
Content Packs
├── warfare-starwars (Group)
│   ├── Units/
│   │   ├── clone_trooper.prefab → bundle_units_abc123
│   │   ├── battle_droid.prefab → bundle_units_abc123
│   │   └── ...
│   ├── Buildings/
│   │   ├── republic_barracks.prefab → bundle_buildings_def456
│   │   └── ...
│   └── Assets/
│       ├── Materials/
│       ├── Textures/
│       ├── Models/
│       └── Animations/
│
├── warfare-modern (Group)
├── example-balance (Group - data only)
└── ...

Catalog.json (runtime asset lookup)
└── Maps Addressables keys → bundle hashes → download URLs
```

### 12.4 SQLite Asset Catalog

The `AssetCatalogStore` maintains a SQLite database of all assets:

```sql
CREATE TABLE assets (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    type TEXT NOT NULL,  -- 'model', 'texture', 'audio', 'prefab'
    pack_id TEXT NOT NULL,
    source_path TEXT NOT NULL,
    addressable_key TEXT,
    bundle_hash TEXT,
    lod_levels INTEGER DEFAULT 1,
    file_size INTEGER,
    created_at TEXT,
    updated_at TEXT
);

CREATE TABLE sources (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    type TEXT NOT NULL,  -- 'local', 'sketchfab', 'url'
    config TEXT  -- JSON configuration
);
```

---

## 13. Hot Reload Specification

### 13.1 Reload Triggers

| Trigger | Latency | Use Case |
|---------|---------|----------|
| **FileSystemWatcher** | ~300ms | Automatic on YAML save |
| **Manual F10** | ~100ms | Explicit reload request |
| **MCP reload_packs** | ~200ms | AI agent-triggered |
| **WebSocket** | ~50ms | External tool trigger (planned) |

### 13.2 Reload Process

```
1. Detect file change (FileSystemWatcher or manual trigger)
2. Debounce (500ms) to batch rapid changes
3. Identify affected pack(s)
4. Validate changed content against schemas
5. Update registry entries (add/modify/remove)
6. Sync changes to ECS bridge
7. Update debug overlay
8. Log result (success/failure with details)
```

### 13.3 State Preservation

| State Type | Preserved? | Notes |
|------------|-----------|-------|
| **Registry entries** | ✅ Yes | Updated in-place |
| **ECS components** | ✅ Yes | Updated on existing entities |
| **Spawned entities** | ✅ Yes | Stats updated, visuals swapped |
| **Game progress** | ✅ Yes | No game state lost |
| **User settings** | ✅ Yes | Stored separately |

---

## 14. MCP Server Specification

### 14.1 Server Architecture

```
Claude Code / AI Agent
  │
  ▼
HTTP Transport (JSON-RPC 2.0) — Port 8765
  │
  ▼
MCP Server (FastMCP C# / Python)
  │
  ├── Game Query Tools
  ├── Game Control Tools
  ├── Asset Pipeline Tools
  ├── Pack Management Tools
  ├── Log Analysis Tools
  └── UI Automation Tools
  │
  ▼
Game Process (via Named Pipes Bridge)
  ├── ECS World (entities, components)
  ├── BepInEx Plugin (DINOForge Runtime)
  └── Named Pipes Bridge
```

### 14.2 Tool Inventory

| Tool | Category | Purpose | Latency |
|------|----------|---------|---------|
| `game_status` | Query | Check if game is running and mods loaded | ~50ms |
| `game_query_entities` | Query | Query ECS entities by component type | ~100ms |
| `game_get_stat` | Query | Read a stat value on an entity | ~50ms |
| `game_apply_override` | Control | Apply a stat override | ~100ms |
| `game_reload_packs` | Control | Hot-reload packs without restarting | ~200ms |
| `game_dump_state` | Control | Trigger entity dump to file | ~500ms |
| `game_screenshot` | Automation | Capture game window screenshot | ~500ms |
| `game_verify_mod` | Query | Verify mod is loaded and active | ~50ms |
| `game_wait_for_world` | Query | Wait until ECS world is ready | ~100ms |
| `game_ui_automation` | Automation | Automate game UI interactions | ~500ms |
| `game_launch_test` | Control | Launch TEST instance (second concurrent DINO) | ~5s |
| `game_analyze_screen` | Automation | Detect UI elements via OmniParser | ~1s |
| `game_input` | Automation | Inject keyboard/mouse input (Win32 SendInput) | ~1ms |
| `game_wait_and_screenshot` | Automation | Poll for visual change then capture | ~1s |
| `game_navigate_to` | Automation | Navigate to game state via input sequences | ~2s |
| `asset_validate` | Asset | Validate asset against technical requirements | ~200ms |
| `asset_import` | Asset | Import asset into pipeline | ~500ms |
| `asset_optimize` | Asset | Optimize asset (LOD, compression) | ~1s |
| `asset_build` | Asset | Build Addressables bundle | ~2s |
| `pack_validate` | Pack | Validate pack against schemas | ~200ms |
| `pack_build` | Pack | Build pack artifact | ~1s |
| `pack_list` | Pack | List all installed packs | ~50ms |
| `log_tail` | Log | Read recent log entries | ~50ms |
| `dump_state` | Log | Get current entity dump status | ~100ms |
| `swap_status` | Log | Check asset swap operation status | ~50ms |

### 14.3 Transport Configuration

```json
{
  "mcpServers": {
    "dinoforge": {
      "url": "http://127.0.0.1:8765"
    }
  }
}
```

### 14.4 Multi-Instance Orchestration (ADR-022)

| Range | Purpose | Allocation |
|-------|---------|------------|
| 8765 | Primary instance | Fixed |
| 8766-8799 | Test instances | Dynamic |
| 8800-8899 | Scenario instances | Dynamic |
| 8900+ | Reserved | Future |

**Heartbeat Protocol:**
- MCP Server sends heartbeat every 30s to registry
- Missing 3 heartbeats → Mark unhealthy
- Missing 5 heartbeats → Auto-deregister

---

## 15. Desktop Companion Specification

### 15.1 Architecture

```
Desktop Companion (WinUI 3)
├── NavigationView Shell
│   ├── Pack Manager
│   │   ├── Pack List (Virtualized GridView)
│   │   ├── Pack Details Panel
│   │   ├── Enable/Disable Toggle
│   │   └── Conflict Indicators
│   ├── Asset Browser
│   │   ├── Asset Grid (SQLite-backed)
│   │   ├── Asset Preview
│   │   └── Search & Filter
│   ├── Mod Manager
│   │   ├── Installed Mods
│   │   ├── Available Updates
│   │   └── Conflict Detection
│   └── Debug Panel
│       ├── Entity Counts
│       ├── System State
│       └── Error Log
│
├── State Management
│   ├── disabled_packs.json (shared with game)
│   ├── PackFileWatcher (500ms debounce)
│   └── SDK direct reference (netstandard2.0)
│
└── UI Framework
    ├── WinUI 3 + Windows App SDK
    ├── Mica material background
    └── Dark color tokens matching DinoForgeStyle.cs
```

### 15.2 State Parity

The Desktop Companion reads/writes `disabled_packs.json` which is shared with the game runtime:

```json
{
  "disabled_packs": [
    "warfare-guerrilla",
    "example-balance"
  ],
  "last_updated": "2026-04-04T12:00:00Z"
}
```

### 15.3 Pack File Watcher

- Watches packs directory for YAML changes
- 500ms debounce to batch rapid changes
- Reloads companion state on change
- Uses SDK `PackFileWatcher` directly

---

## 16. CLI Tool Specification

### 16.1 PackCompiler CLI

| Command | Purpose | Example |
|---------|---------|---------|
| `validate` | Validate pack against schemas | `packcompiler validate packs/warfare-starwars` |
| `build` | Build pack with Addressables | `packcompiler build packs/warfare-starwars` |
| `package` | Create distributable zip | `packcompiler package packs/warfare-starwars` |
| `deps` | Analyze dependencies | `packcompiler deps packs/warfare-starwars` |
| `conflicts` | Detect content conflicts | `packcompiler conflicts` |

### 16.2 dinoforge CLI

| Command | Purpose | Example |
|---------|---------|---------|
| `pack add` | Add git submodule pack | `dinoforge pack add <url>` |
| `pack list` | List all packs with status | `dinoforge pack list` |
| `pack update` | Update pack to latest | `dinoforge pack update warfare-starwars` |
| `pack lock` | Lock pack to specific commit | `dinoforge pack lock warfare-starwars abc123` |
| `asset import` | Import asset into pipeline | `dinoforge asset import model.fbx` |
| `asset validate` | Validate asset | `dinoforge asset validate model.fbx` |
| `asset optimize` | Optimize asset | `dinoforge asset optimize model.fbx` |
| `asset build` | Build Addressables bundle | `dinoforge asset build warfare-starwars` |
| `asset library` | Query asset catalog | `dinoforge asset library search clone` |
| `install` | Install DINOForge | `dinoforge install` |
| `status` | Project health summary | `dinoforge status` |

---

## 17. API Reference

### 17.1 SDK Public API

#### Registry API

```csharp
// Register content
registry.Register("starwars:clone_trooper", unit, priority: 3000, sourcePack: "warfare-starwars");

// Get content
var unit = registry.Get<UnitDefinition>("starwars:clone_trooper");

// Get all content
var allUnits = registry.GetAll<UnitDefinition>();

// Check conflicts
var conflicts = registry.GetConflicts<UnitDefinition>();

// Check existence
bool exists = registry.Contains("starwars:clone_trooper");

// Count
int count = registry.Count<UnitDefinition>();
```

#### ContentLoader API

```csharp
// Load all packs
var result = await ContentLoader.LoadPacksAsync(packsDir);

// Reload single pack
var reloadResult = await ContentLoader.ReloadPackAsync(packPath);

// Get load errors
var errors = result.Errors;

// Get loaded packs
var packs = result.LoadedPacks;
```

#### SchemaValidator API

```csharp
// Validate content
var result = await validator.ValidateAsync(contentPath, "unit");

// Check result
if (result.IsValid)
{
    // Content is valid
}
else
{
    foreach (var error in result.Errors)
    {
        Console.WriteLine($"{error.Path}: {error.Message}");
    }
}
```

#### DependencyResolver API

```csharp
// Resolve dependencies
var result = resolver.Resolve(manifests);

if (result.IsSuccess)
{
    var loadOrder = result.LoadOrder; // Topologically sorted
}
else
{
    Console.WriteLine(result.Error); // Circular dependency, missing dep, etc.
}
```

### 17.2 Runtime API

#### ModPlatform API

```csharp
// Initialize (called by RuntimeDriver)
ModPlatform.Initialize();

// On world ready (called by ModPlatform)
ModPlatform.OnWorldReady(entityManager);

// Get loaded packs
var packs = ModPlatform.LoadedPacks;

// Get registry
var registry = ModPlatform.Registry;
```

#### ECS Bridge API

```csharp
// Apply stat override
StatModifierSystem.ApplyOverride(entityId, "health", 200f);

// Swap asset
AssetSwapService.SwapUnitPrefab("starwars:clone_trooper", "new_prefab_key");

// Query entities
var units = EntityQueries.GetUnitsByFaction(entityManager, "starwars:galactic_republic");
```

### 17.3 MCP API

All MCP tools follow the JSON-RPC 2.0 protocol:

```json
// Request
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "game_status",
    "arguments": {}
  }
}

// Response
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "content": [
      {
        "type": "text",
        "text": "{\"running\": true, \"entity_count\": 45000, \"loaded_packs\": [\"warfare-starwars\"]}"
      }
    ]
  }
}
```

---

## 18. Error Handling

### 18.1 Error Categories

| Category | Severity | Action | Example |
|----------|----------|--------|---------|
| **Schema Validation Error** | Blocking | Skip pack, log error | Missing required field in unit.yaml |
| **Dependency Missing** | Blocking | Halt load, report error | Pack depends on missing pack |
| **Circular Dependency** | Blocking | Halt load, report cycle | A→B→C→A |
| **Version Conflict** | Blocking | Halt load, report incompatibility | framework_version mismatch |
| **Content Conflict** | Warning | Log conflict, higher priority wins | Two packs define same unit at same priority |
| **Asset Missing** | Warning | Log warning, use fallback | Referenced prefab not found |
| **Runtime Error** | Critical | Log error, attempt recovery | ECS query failure |
| **Hot Reload Failure** | Warning | Log error, keep previous state | Invalid YAML on save |

### 18.2 Error Reporting

Errors are reported through multiple surfaces:

| Surface | Content | Audience |
|---------|---------|----------|
| **Console/Log** | Full error details with stack traces | Developers |
| **Debug Overlay (F10)** | Summary of errors and warnings | Developers, mod authors |
| **PackCompiler Output** | Validation errors with file/line | Mod authors, CI |
| **Desktop Companion** | User-friendly error messages | End users |
| **MCP Tool Response** | Structured error with context | AI agents |

### 18.3 Error Format

```csharp
public record ValidationError
{
    public string Path { get; init; } = "";       // File path
    public string Message { get; init; } = "";     // Human-readable message
    public string Code { get; init; } = "";        // Error code (e.g., "id_format")
    public int? LineNumber { get; init; }         // Line number (if available)
    public int? LinePosition { get; init; }       // Column number (if available)
    public ErrorSeverity Severity { get; init; }   // Error, Warning, Info
}

public enum ErrorSeverity
{
    Info,
    Warning,
    Error,
    Critical
}
```

### 18.4 Recovery Strategies

| Error Type | Recovery Strategy |
|------------|------------------|
| **Schema validation failure** | Skip invalid content, continue loading rest of pack |
| **Missing dependency** | Halt load, report to user, suggest installation |
| **Circular dependency** | Halt load, report cycle path to user |
| **Content conflict** | Use higher priority, log warning |
| **Asset missing** | Use fallback asset, log warning |
| **Runtime crash** | Attempt graceful degradation, log crash report |
| **Hot reload failure** | Keep previous state, log error |

---

## 19. Security

### 19.1 Threat Model

| Threat | Likelihood | Impact | Mitigation |
|--------|-----------|--------|------------|
| **Malicious pack content** | Low | Medium | Schema validation, YAML-only content |
| **Dependency confusion** | Medium | High | Git submodule verification, checksums |
| **Save file corruption** | Low | High | Pack validation before load, save backups |
| **Privilege escalation** | Low | Critical | BepInEx plugin isolation, no network access |
| **Data exfiltration** | Low | High | No network access for packs, sandboxed runtime |

### 19.2 Content Security Pipeline

```
Pack Submission
  │
  ▼
Schema Validation → Reject invalid content
  │
  ▼
Source Verification → Verify git submodule origin, check commit signatures
  │
  ▼
Dependency Validation → Verify all dependencies exist, check version compatibility
  │
  ▼
Conflict Detection → Detect content conflicts, check priority overlaps
  │
  ▼
Asset Validation → Verify asset file integrity, check file types/sizes
  │
  ▼
Approved Pack → Install
```

### 19.3 Security Controls

| Control | Implementation | Scope |
|---------|---------------|-------|
| **Schema Validation** | NJsonSchema against 17 schemas | All content |
| **Source Verification** | Git submodule URL + commit hash | Pack submodules |
| **Dependency Locking** | packs.lock with commit hashes | All dependencies |
| **No Network Access** | Packs have no network capabilities | Runtime |
| **BepInEx Isolation** | Plugin runs in BepInEx sandbox | Runtime |
| **Checksum Verification** | SHA-256 for pack artifacts | Distribution |
| **Version Gating** | framework_version compatibility check | Pack loading |

### 19.4 Supported Versions

| Version | Status | Support End |
|---------|--------|-------------|
| **0.5.x** | Current | Until 0.6.0 release |
| **0.4.x** | Maintenance | Until 0.5.x EOL |
| **<0.4.0** | End of Life | No support |

### 19.5 Vulnerability Reporting

Private vulnerability reporting via [SECURITY.md](SECURITY.md).

Response timelines:
- **Critical**: 24 hours
- **High**: 72 hours
- **Medium**: 1 week
- **Low**: 2 weeks

---

## 20. Performance Requirements

### 20.1 Runtime Performance

| Metric | Target | Current | Measurement |
|--------|--------|---------|-------------|
| **Pack load time** | <100ms/pack | ~50ms | 100 unit pack |
| **Hot reload latency** | <500ms | ~300ms | F10 press to refresh |
| **Registry query** | <1μs | ~0.5μs | Single lookup |
| **ECS sync overhead** | <1ms/frame | ~0.3ms | 1000 entities |
| **MCP tool response** | <100ms | ~50ms | game_status query |
| **Schema validation** | <10ms/file | ~5ms | Average unit YAML |
| **Frame overhead** | <2ms | ~0.5ms | Full mod stack |
| **Memory per entity** | <64 bytes | ~32 bytes | Mod components |

### 20.2 Build Performance

| Metric | Target | Current |
|--------|--------|---------|
| **Full solution build** | <60s | ~45s |
| **Test execution** | <30s | ~20s |
| **Pack validation** | <5s | ~2s |
| **Asset optimization** | <30s/asset | ~15s |

### 20.3 Scalability Targets

| Metric | Target | Notes |
|--------|--------|-------|
| **Max packs loaded** | 50+ | With dependency resolution |
| **Max units per pack** | 100+ | With schema validation |
| **Max entities tracked** | 10K+ | ECS bridge capacity |
| **Max concurrent MCP clients** | 5+ | HTTP transport |
| **Max multi-instance games** | 10+ | Planned (ADR-020) |

---

## 21. Testing Strategy

### 21.1 Testing Pyramid

```
┌─────────────────────────────────────────────────────────────┐
│  E2E Tests (5%)                                             │
│  ├── Game launch + mod load                                 │
│  ├── Unit spawn via MCP                                     │
│  └── Screenshot capture validation                          │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│  Integration Tests (25%)                                    │
│  ├── Pack loading + validation                              │
│  ├── Registry conflict detection                            │
│  ├── ECS Bridge sync                                        │
│  └── MCP server round-trip                                  │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│  Unit Tests (70%)                                           │
│  ├── Registry operations                                    │
│  ├── Schema validation                                      │
│  ├── Dependency resolution                                  │
│  ├── Model serialization                                    │
│  └── Utility functions                                      │
└─────────────────────────────────────────────────────────────┘

Property Tests (FsCheck):
├── Roundtrip serialization (30+ properties)
├── Registry consistency (20+ properties)
└── Dependency graph validation (15+ properties)

Fuzzing (SharpFuzz):
├── PackCompiler input fuzzing
└── YAML parser fuzzing

Mutation Testing (Stryker.NET):
├── SDK models and domain code
└── Threshold: 85% high / 70% low / 60% break
```

### 21.2 Test Categories

| Category | Count | Framework | Purpose |
|----------|-------|-----------|---------|
| **Unit Tests** | 700+ | xUnit + FluentAssertions | API correctness |
| **Integration Tests** | 200+ | xUnit + Mocks | Component interaction |
| **Property Tests** | 65+ | FsCheck | Invariant validation |
| **Snapshot Tests** | 10 | ApprovalTests | Serialization stability |
| **Performance Tests** | 7 | xUnit + Stopwatch | Regression detection |
| **Mutation Tests** | N/A | Stryker.NET | Test quality |
| **Fuzz Tests** | N/A | SharpFuzz | Crash detection |
| **MCP Tests** | 186 | pytest | Server tool coverage |

### 21.3 Coverage Targets

| Package | Target | Current |
|---------|--------|---------|
| **Bridge.Protocol** | 100% | 100% |
| **Warfare** | 95% | 95.6% |
| **Scenario** | 90% | 93.1% |
| **UI** | 90% | 89.2% |
| **Economy** | 85% | 87.9% |
| **Installer** | 85% | 88.3% |
| **SDK** | 80% | 76.4% |
| **Bridge.Client** | 80% | 82.4% |
| **Total** | 81% | 83.5% |

---

## 22. CI/CD Pipeline

### 22.1 GitHub Actions Workflows

| Workflow | Trigger | Purpose |
|----------|---------|---------|
| **ci.yml** | Push/PR | Build, test, coverage |
| **fuzz.yml** | Nightly | SharpFuzz targets |
| **release.yml** | Tag | Versioned release |
| **mcp-pytest.yml** | Push/PR | MCP server tests |
| **mutation.yml** | Weekly | Stryker.NET mutation testing |

### 22.2 Quality Gates

| Gate | Threshold | Action on Failure |
|------|-----------|-------------------|
| **Build** | 0 errors | Block merge |
| **Tests** | 0 failures | Block merge |
| **Coverage** | 81% line | Block merge |
| **Format** | No changes | Block merge |
| **Mutation** | 60% break | Warning |
| **Fuzzing** | 0 crashes | Block merge |

### 22.3 Versioning

- **SemVer** tags (`vX.Y.Z`)
- **MinVer** for automatic version calculation
- **VERSION** file tracks latest released version
- **CHANGELOG.md** follows Keep a Changelog format

---

## 23. Deployment

### 23.1 Game Plugin (BepInEx)

**Installation:**
```powershell
irm https://raw.githubusercontent.com/KooshaPari/Dino/main/src/Tools/Installer/Install-DINOForge.ps1 | iex
```

**Output:** `BepInEx/ecs_plugins/DINOForge.Runtime.dll`

### 23.2 Desktop Companion

**Installation:**
```powershell
irm https://raw.githubusercontent.com/KooshaPari/Dino/main/scripts/install-companion.ps1 | iex
```

**Output:** `DINOForge.Companion-vX.Y.Z-win-x64.zip`

### 23.3 SDK (NuGet)

**Package:** `DINOForge.SDK` on nuget.org

```bash
dotnet add package DINOForge.SDK
```

### 23.4 Bridge Protocol (NuGet)

**Package:** `DINOForge.Bridge.Protocol` on nuget.org

```bash
dotnet add package DINOForge.Bridge.Protocol
```

---

## 24. Migration & Versioning

### 24.1 Version Compatibility

| DINOForge Version | Game Version | Breaking Changes | Migration Required |
|-------------------|-------------|-----------------|-------------------|
| 0.5.x | Any | No | No |
| 0.4.x | Any | Minor | Update pack.yaml framework_version |
| 0.3.x | Any | Yes | Update schemas, regenerate packs |
| <0.3.0 | Any | Major | Full migration |

### 24.2 Migration Guide

When upgrading DINOForge:

1. Update `framework_version` in pack.yaml
2. Re-validate packs with `packcompiler validate`
3. Check for deprecated schema fields
4. Update SDK reference if using as library
5. Rebuild Desktop Companion if using locally

### 24.3 Deprecation Policy

- Deprecation warnings issued 1 release before removal
- Deprecated fields documented in CHANGELOG.md
- Migration scripts provided when possible
- Breaking changes only in minor version bumps (pre-1.0)

---

## 25. Glossary

| Term | Definition |
|------|-----------|
| **Pack** | A unit of mod content with a manifest (pack.yaml), content files, and optional assets |
| **Registry** | A typed collection of content entries with priority-based conflict resolution |
| **ECS** | Entity Component System — Unity's Data-Oriented Technology Stack (DOTS) |
| **BepInEx** | Unity mod loader that DINOForge uses as its bootstrap |
| **Harmony** | IL-level method patching library (avoided by DINOForge in favor of ECS-native) |
| **Schema** | JSON Schema definition that validates pack content structure |
| **Domain Plugin** | A game-specific extension (Warfare, Economy, etc.) that adds registries and logic |
| **Hot Reload** | The ability to update pack content without restarting the game |
| **MCP** | Model Context Protocol — AI agent integration for game automation |
| **Addressables** | Unity's asset loading system for runtime content delivery |
| **LOD** | Level of Detail — multiple quality versions of 3D assets |
| **Beed** | A work item in the Kilo Gastown agent coordination system |
| **Convoy** | A cross-repo methodology propagation train |
| **SDD** | Spec-Driven Development |
| **BDD** | Behavior-Driven Development |
| **TDD** | Test-Driven Development |
| **DDD** | Domain-Driven Design |
| **ADD** | Agent-Driven Development |
| **CDD** | Contract-Driven Development |

---

## Appendix A: Agent Roster & File Ownership

| Agent Role | Domain | Key Files | Can Modify |
|-----------|--------|-----------|-----------|
| runtime-specialist | ECS bridge, BepInEx | src/Runtime/ | Plugin.cs, Bridge/*, HotReload/*, DebugOverlay, ModPlatform.cs |
| sdk-architect | Registry, SDK, schemas | src/SDK/ | Registry/*, Models/*, Validation/*, Assets/*, Dependencies/*, Universe/* |
| warfare-designer | Warfare domain, balance | src/Domains/Warfare/ | Archetypes/*, Doctrines/*, Roles/*, Waves/*, Balance/* |
| pack-builder | Content packs, YAML | packs/ | packs/**/*, any pack.yaml |
| toolsmith | CLI tools, PackCompiler | src/Tools/ | PackCompiler/*, DumpTools/*, Cli/*, McpServer/* |
| qa-engineer | Tests, CI/CD | src/Tests/, .github/ | Tests/**, workflows/*, test fixtures |
| docs-curator | Documentation, VitePress | docs/ | docs/**, CHANGELOG.md, README.md, AGENTS.md |

## Appendix B: Key Invariants

1. **All tests must pass before any commit to main**
2. **Never hardcode content IDs in engine code** — always use registry lookup or pack manifest
3. **Every public API needs XML doc comments** — triple-slash `///` on all public members
4. **Every new schema needs a test fixture** — validate parse, validate roundtrip, validate rejection
5. **Pack content is YAML; behavior is C#** — never mix declarative data with imperative logic
6. **Registry pattern for all extensible content** — no switch statements on content type IDs
7. **Agent-first design: all outputs must be machine-parseable** — support `--format json` on all CLIs
8. **Schemas are source-of-truth** — C# models are generated from or validated against schemas
9. **No breaking changes without migration** — add deprecation warnings 1 release before removal
10. **Commit message must reference domain/feature** — e.g., "feat(warfare): add wave scripting system"

## Appendix C: Architecture Decision Records Index

| ADR | Title | Status |
|-----|-------|--------|
| [ADR-001](docs/adr/ADR-001-agent-driven-development.md) | Agent-Driven Development Model | Accepted |
| [ADR-002](docs/adr/ADR-002-declarative-first-architecture.md) | Declarative-First Architecture | Accepted |
| [ADR-003](docs/adr/ADR-003-pack-system-design.md) | Pack System Design | Accepted |
| [ADR-004](docs/adr/ADR-004-registry-model.md) | Registry Model | Accepted |
| [ADR-005](docs/adr/ADR-005-ecs-integration-strategy.md) | ECS Integration Strategy | Accepted |
| [ADR-006](docs/adr/ADR-006-domain-plugin-architecture.md) | Domain Plugin Architecture | Accepted |
| [ADR-007](docs/adr/ADR-007-observability-first.md) | Observability First | Accepted |
| [ADR-008](docs/adr/ADR-008-wrap-dont-handroll.md) | Wrap, Don't Handroll | Accepted |
| [ADR-009](docs/adr/ADR-009-runtime-orchestration.md) | Runtime Orchestration via ModPlatform | Accepted |
| [ADR-010](docs/adr/ADR-010-asset-intake-pipeline.md) | Deterministic Star-Wars Asset Intake Pipeline | Proposed |
| [ADR-011](docs/adr/ADR-011-desktop-companion.md) | WinUI 3 Desktop Companion App | Accepted |
| [ADR-012](docs/adr/ADR-012-fuzzing-strategy.md) | Fuzzing and Property-Based Testing Strategy | Accepted |
| [ADR-013](docs/adr/ADR-013-duplicate-instance-detection-bypass.md) | Duplicate Instance Detection Bypass | Accepted |
| [ADR-014](docs/adr/ADR-014-runtime-execution-model.md) | Runtime Execution Model | Accepted |
| [ADR-015](docs/adr/ADR-015-native-menu-injector.md) | Native Menu Injector | Accepted |
| [ADR-016](docs/adr/ADR-016-no-harmony-patches-on-dino-systems.md) | No Harmony Patches on DINO Systems | Accepted |
| [ADR-017](docs/adr/ADR-017-neural-tts-for-proof-videos.md) | Neural TTS for Proof Videos | Accepted |
| [ADR-018](docs/adr/ADR-018-second-instance-bypass.md) | Second Instance Bypass | Accepted |
| [ADR-019](docs/adr/ADR-019-mod-manager-client.md) | Local Mod Manager Client (M12) | Accepted |
| [ADR-020](docs/adr/ADR-020-multi-instance-concurrency.md) | Multi-Instance Concurrency Architecture | Proposed |
| [ADR-021](docs/adr/ADR-021-neural-asset-pipeline.md) | Neural Asset Pipeline | Proposed |
| [ADR-022](docs/adr/ADR-022-mcp-orchestration.md) | MCP Orchestration Model | Proposed |

---

*This specification is part of the DINOForge documentation suite. For questions or contributions, refer to the [CONTRIBUTING.md](CONTRIBUTING.md) guide.*
