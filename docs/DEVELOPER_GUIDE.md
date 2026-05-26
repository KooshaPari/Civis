# DINOForge Developer Guide

This guide walks through common development tasks in DINOForge, from creating simple content packs to building complex domain plugins. Start with the basics and progress to advanced topics.

## Table of Contents

1. [Architecture Overview](#architecture-overview)
2. [Your First Pack](#your-first-pack)
3. [Adding New Content to Existing Packs](#adding-new-content)
4. [Building a Domain Plugin](#building-a-domain-plugin)
5. [Testing Game Logic](#testing-game-logic)
6. [Common Development Tasks](#common-development-tasks)

## Architecture Overview

### Layered Architecture

DINOForge uses a **hexagonal (ports & adapters)** layered design where each layer depends inward only:

```
┌─────────────────────────────────────────┐
│  Packs (User-Created Content)           │
│  ├─ warfare-starwars/                   │
│  ├─ economy-balanced/                   │
│  └─ scenario-tutorial/                  │
└─────────────────────────────────────────┘
                    ↓
┌─────────────────────────────────────────┐
│  Domain Plugins (Gameplay Systems)      │
│  ├─ Warfare                             │
│  ├─ Economy                             │
│  ├─ Scenario                            │
│  └─ UI                                  │
└─────────────────────────────────────────┘
                    ↓
┌─────────────────────────────────────────┐
│  SDK Layer (Registries & Services)      │
│  ├─ Registries (Unit, Building, etc.)   │
│  ├─ Schema Validation (NJsonSchema)     │
│  ├─ Content Loader                      │
│  ├─ Dependency Resolver                 │
│  └─ Asset Services                      │
└─────────────────────────────────────────┘
                    ↓
┌─────────────────────────────────────────┐
│  Runtime (BepInEx Plugin)               │
│  ├─ ECS Bridge (Component Mapping)      │
│  ├─ Entity Queries                      │
│  ├─ Stat Modifiers                      │
│  └─ Hot Reload                          │
└─────────────────────────────────────────┘
                    ↓
┌─────────────────────────────────────────┐
│  Game (DINO Unity ECS)                  │
└─────────────────────────────────────────┘
```

### Registry Priority Layers

Content from multiple sources is merged using explicit priority layers:

```
┌─────────────────────────────────┐
│  Pack (priority 3000+)          │  ← Mod content (highest priority)
├─────────────────────────────────┤
│  Domain Plugin (priority 2000+) │  ← Warfare/Economy/etc. defaults
├─────────────────────────────────┤
│  Framework (priority 1000+)     │  ← DINOForge defaults
├─────────────────────────────────┤
│  Base Game (priority 0+)        │  ← Vanilla DINO (lowest priority)
└─────────────────────────────────┘
```

Higher priority wins. Conflicts at same priority are detected and reported.

### Key Concepts

#### Registries

Typed registries store extensible domain data. Examples:

- `IUnitRegistry` — All unit definitions (warriors, archers, etc.)
- `IBuildingRegistry` — Building definitions
- `IFactionRegistry` — Faction definitions
- `IDoctineRegistry` — Combat doctrines

Registries are **NOT hardcoded**. Every entry is registered dynamically at load time.

#### Schemas

JSON/YAML schemas (in `schemas/`) define the structure of content **before** it's loaded:

- `unit.schema.json` — Structure of a unit definition
- `faction.schema.json` — Structure of a faction definition
- `pack.schema.json` — Structure of a pack.yaml manifest

Schemas catch errors at load time, preventing silent failures.

#### Packs

Packs are **YAML-first** content bundles with:

- `pack.yaml` — Manifest describing the pack (version, dependencies, loads)
- Content files — YAML/JSON files for units, buildings, factions, etc.
- Assets (optional) — 3D models, textures, prefabs for visual content
- Dependencies — Explicit list of other packs required to load

#### Domain Plugins

Domain plugins extend the engine with new gameplay systems:

- **Warfare** — Factions, doctrines, unit roles, combat balance
- **Economy** — Production rates, trade, resources
- **Scenario** — Story missions, victory conditions, scripted events
- **UI** — HUD elements, menus, themes

Each plugin registers registries and content with the SDK.

## Your First Pack

A pack is a self-contained bundle of YAML content. Let's create a simple pack with one new unit.

### Step 1: Create the Pack Directory

```bash
# Create directory structure
mkdir -p packs/my-first-pack/{units,factions,buildings}
cd packs/my-first-pack
```

### Step 2: Create pack.yaml

The manifest file tells DINOForge what content is in this pack:

```yaml
# packs/my-first-pack/pack.yaml

id: my-first-pack
name: My First Pack
version: 0.1.0
author: Your Name
description: A simple pack with one new unit

# Minimum framework version required to load this pack
framework_version: ">=0.1.0 <1.0.0"

# Other packs this pack depends on
# Example: depends_on: ["warfare-base"]
depends_on: []

# Other packs this pack is incompatible with
conflicts_with: []

# Content to load from this pack
loads:
  units:
    - units/my-unit.yaml
  factions: []
  buildings: []
  doctrines: []
  waves: []
```

### Step 3: Create a Unit Definition

Units are defined in YAML and validated against `schemas/unit.schema.json`:

```yaml
# packs/my-first-pack/units/my-unit.yaml

id: my-warrior
name: My Warrior
description: A custom warrior unit for testing

# Base stats
stats:
  max_health: 60
  attack_power: 15
  armor: 2
  movement_speed: 5

# Unit classification
role: warrior
faction: neutral
cost: 100

# Abilities (optional)
abilities:
  - slash
  - charge
```

### Step 4: Validate the Pack

Run the PackCompiler to validate against schemas:

```bash
# From repo root
dotnet run --project src/Tools/PackCompiler -- validate packs/my-first-pack
```

Expected output:
```
✓ pack.yaml: valid
✓ units/my-unit.yaml: valid
✓ All dependencies resolved
✓ Pack is valid
```

### Step 5: Test in Game (Optional)

```bash
# 1. Copy the pack to your DINO installation
cp -r packs/my-first-pack "G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option\BepInEx\dinoforge_packs\"

# 2. Launch game and open mod menu (F10)
# 3. Check the pack is loaded without errors
```

### Pack Structure Reference

```
my-first-pack/
  pack.yaml              # Manifest (required)
  units/
    my-unit.yaml         # Unit definitions
  buildings/
    my-building.yaml     # Building definitions (optional)
  factions/
    my-faction.yaml      # Faction definitions (optional)
  doctrines/
    my-doctrine.yaml     # Combat doctrines (optional)
  assets/
    bundles/
      my-unit-model      # 3D model bundle (optional)
```

## Adding New Content

Now that you've created a pack, let's add more content types.

### Adding a Building

Buildings work similarly to units:

```yaml
# packs/my-first-pack/buildings/watchtower.yaml

id: watchtower
name: Watchtower
description: Defensive structure with range advantage

stats:
  max_health: 100
  attack_power: 8
  armor: 5
  production_rate: 0

cost:
  gold: 300
  wood: 150

construction_time: 30  # seconds
```

Add to `pack.yaml`:

```yaml
loads:
  buildings:
    - buildings/watchtower.yaml
```

### Adding a Faction

Factions define the different sides in gameplay:

```yaml
# packs/my-first-pack/factions/my-faction.yaml

id: my-faction
name: My Faction
description: A new playable faction
color: "#FF6B6B"
icon: null  # Placeholder

# Units available to this faction
units:
  - my-warrior
  - archer
  - knight

# Buildings available to this faction
buildings:
  - watchtower
  - barracks

# Available doctrines/strategies
doctrines: []
```

### Validating All Changes

After adding content, validate again:

```bash
dotnet run --project src/Tools/PackCompiler -- validate packs/my-first-pack

# Validate all packs
dotnet run --project src/Tools/PackCompiler -- validate packs/
```

## Building a Domain Plugin

Domain plugins extend DINOForge with new gameplay systems. Let's create a simple "Crafting" domain as an example.

### Step 1: Create Project Structure

```bash
mkdir -p src/Domains/Crafting
cd src/Domains/Crafting

# Create C# class library
dotnet new classlib -f net8.0 -n DINOForge.Domains.Crafting

# Add to solution
cd ../../../
dotnet sln src/DINOForge.sln add src/Domains/Crafting/DINOForge.Domains.Crafting.csproj
```

### Step 2: Define Data Models

Create the core domain models:

```csharp
// src/Domains/Crafting/Models/Recipe.cs

using DINOForge.SDK.Models;

namespace DINOForge.Domains.Crafting.Models;

/// <summary>A crafting recipe that converts resources into items.</summary>
public record Recipe(
    string Id,
    string Name,
    string Description,
    Dictionary<string, int> InputResources,   // e.g., { "wood": 50, "stone": 25 }
    Dictionary<string, int> OutputResources,  // e.g., { "sword": 1 }
    int CraftingTime                          // seconds
);

/// <summary>Crafting station where recipes can be executed.</summary>
public record CraftingStation(
    string Id,
    string Name,
    List<string> AvailableRecipes
);
```

### Step 3: Create Registries

Registries enable other packs to extend the domain:

```csharp
// src/Domains/Crafting/Registry/IRecipeRegistry.cs

namespace DINOForge.Domains.Crafting.Registry;

public interface IRecipeRegistry
{
    /// <summary>Register a new crafting recipe.</summary>
    bool Register(Recipe recipe);

    /// <summary>Get a recipe by ID.</summary>
    Recipe? Get(string id);

    /// <summary>Get all registered recipes.</summary>
    IReadOnlyList<Recipe> GetAll();
}
```

```csharp
// src/Domains/Crafting/Registry/RecipeRegistry.cs

using System.Collections.Generic;
using System.Diagnostics.CodeAnalysis;

namespace DINOForge.Domains.Crafting.Registry;

public class RecipeRegistry : IRecipeRegistry
{
    private readonly Dictionary<string, Recipe> _recipes = new();

    public bool Register(Recipe recipe)
    {
        ArgumentNullException.ThrowIfNull(recipe);

        if (_recipes.ContainsKey(recipe.Id))
            return false;

        _recipes[recipe.Id] = recipe;
        return true;
    }

    public Recipe? Get(string id) =>
        _recipes.TryGetValue(id, out var recipe) ? recipe : null;

    public IReadOnlyList<Recipe> GetAll() =>
        _recipes.Values.ToList();
}
```

### Step 4: Create the Domain Plugin Class

```csharp
// src/Domains/Crafting/CraftingDomainPlugin.cs

using DINOForge.SDK;
using DINOForge.Domains.Crafting.Registry;
using DINOForge.SDK.Registry;

namespace DINOForge.Domains.Crafting;

/// <summary>Domain plugin for crafting system.</summary>
public class CraftingDomainPlugin : IDomainPlugin
{
    private IRecipeRegistry? _recipeRegistry;

    public string Id => "crafting";
    public string Name => "Crafting System";
    public string Version => "0.1.0";

    /// <summary>Register schemas and services.</summary>
    public void RegisterSchemas(ISchemaRegistry schemas)
    {
        // In a real implementation, load JSON schemas
        // For now, schemas are validated separately
    }

    /// <summary>Initialize registries when plugin loads.</summary>
    public void Initialize(IServiceProvider services)
    {
        _recipeRegistry = new RecipeRegistry();

        // Register default recipes
        _recipeRegistry.Register(new Recipe(
            Id: "iron-sword",
            Name: "Iron Sword",
            Description: "Craft an iron sword from ore",
            InputResources: new() { { "iron-ore", 10 }, { "wood", 5 } },
            OutputResources: new() { { "sword", 1 } },
            CraftingTime: 20
        ));
    }

    /// <summary>Called when a pack is loaded with content for this domain.</summary>
    public void OnPackLoaded(PackManifest pack, IServiceProvider services)
    {
        // Load pack-specific recipes if provided
        // This would be called by the content loader when a pack is activated
    }
}
```

### Step 5: Add Tests

Create comprehensive tests:

```csharp
// src/Tests/Domains/CraftingTests.cs

using Xunit;
using FluentAssertions;
using DINOForge.Domains.Crafting.Registry;
using DINOForge.Domains.Crafting.Models;

public class RecipeRegistryTests
{
    private readonly RecipeRegistry _registry = new();

    [Fact]
    [Category("Unit")]
    public void Register_WithUniqueId_Succeeds()
    {
        // Arrange
        var recipe = new Recipe(
            Id: "iron-sword",
            Name: "Iron Sword",
            Description: "Craft an iron sword",
            InputResources: new() { { "ore", 10 } },
            OutputResources: new() { { "sword", 1 } },
            CraftingTime: 20
        );

        // Act
        bool registered = _registry.Register(recipe);

        // Assert
        registered.Should().BeTrue();
        _registry.Get("iron-sword").Should().NotBeNull();
    }

    [Fact]
    [Category("Unit")]
    public void Register_WithDuplicateId_FailsSilently()
    {
        // Arrange
        var recipe1 = new Recipe("sword", "Sword", "...", new(), new(), 10);
        var recipe2 = new Recipe("sword", "Different", "...", new(), new(), 20);

        _registry.Register(recipe1);

        // Act
        bool registered = _registry.Register(recipe2);

        // Assert
        registered.Should().BeFalse();
        _registry.Get("sword")!.CraftingTime.Should().Be(10);  // Original unchanged
    }
}
```

### Step 6: Register with Runtime

Register the plugin in the runtime loader:

```csharp
// src/Runtime/Plugins/PluginLoader.cs (simplified)

public static void LoadDomainPlugins(IServiceCollection services)
{
    services.AddSingleton<IDomainPlugin>(new CraftingDomainPlugin());
    // ... other plugins
}
```

## Testing Game Logic

Testing without launching the game is critical for rapid development.

### Using MockGameBridgeServer

The `MockGameBridgeServer` simulates a DINO game instance for offline testing:

```csharp
// src/Tests/Integration/GameLogicTests.cs

using Xunit;
using DINOForge.Bridge.Protocol;
using DINOForge.SDK;

public class WarfareIntegrationTests
{
    private readonly MockGameBridgeServer _gameServer = new();

    [Fact]
    [Category("Integration")]
    public async Task LoadPack_Registers_UnitsInGame()
    {
        // Arrange
        var contentLoader = new ContentLoader();
        var packResult = await contentLoader.LoadPackAsync("packs/warfare-modern");

        await _gameServer.StartAsync();

        // Act
        var units = packResult.Units;

        // Assert
        units.Should().NotBeEmpty();
        units.Count.Should().BeGreaterThan(10);
    }

    [Fact]
    [Category("Integration")]
    public async Task BalanceModifier_Applies_CorrectStatChanges()
    {
        // Arrange
        var originalHealth = 50;
        var modifier = new BalanceModifier("warrior", 1.2f);  // 20% health boost

        // Act
        var modifiedHealth = modifier.ApplyTo(originalHealth);

        // Assert
        modifiedHealth.Should().Be(60);
    }
}
```

### Running Game Automation Tests

For tests that need a real game instance, use the MCP server:

```bash
# Start MCP server
./scripts/start-mcp.ps1 -Detached

# Run game automation tests
dotnet test src/Tests/GameAutomationTests.csproj --filter "Category=GameAutomation"
```

These tests use tools like:
- `game_launch` — Launch a game instance
- `game_query_entities` — Query ECS entities
- `game_apply_override` — Apply stat modifiers
- `game_screenshot` — Verify visual results

## Common Development Tasks

### Task: Add a New Unit Type

**Goal**: Add a new cavalry unit to the `warfare-modern` pack.

1. **Define the unit YAML**:
   ```yaml
   # packs/warfare-modern/units/cavalry.yaml
   id: cavalry
   name: Cavalry
   role: cavalry
   stats:
     max_health: 55
     attack_power: 16
     armor: 1
     movement_speed: 8
   cost: 200
   ```

2. **Add to pack.yaml**:
   ```yaml
   loads:
     units:
       - units/cavalry.yaml
   ```

3. **Create a unit test**:
   ```csharp
   [Fact]
   public void LoadPack_IncludesNewCavalryUnit()
   {
       var pack = await contentLoader.LoadPackAsync("packs/warfare-modern");
       var cavalry = pack.Units.FirstOrDefault(u => u.Id == "cavalry");
       cavalry.Should().NotBeNull();
       cavalry!.Role.Should().Be("cavalry");
   }
   ```

4. **Validate and test**:
   ```bash
   dotnet run --project src/Tools/PackCompiler -- validate packs/warfare-modern
   dotnet test src/Tests/ --filter "LoadPack_IncludesNewCavalryUnit"
   ```

### Task: Balance a Unit's Stats

**Goal**: Increase warrior health from 50 to 60.

1. **Edit the unit definition**:
   ```yaml
   # In the unit's YAML file
   stats:
     max_health: 60  # Changed from 50
   ```

2. **Update balance test**:
   ```csharp
   [Fact]
   public void Warrior_HasCorrectBaseHealth()
   {
       var warrior = _unitRegistry.Get("warrior");
       warrior!.MaxHealth.Should().Be(60);
   }
   ```

3. **Test in game** (optional):
   ```bash
   dotnet build -p:DeployToGame=true
   # Launch game and verify unit stats in-game menu
   ```

### Task: Create a Doctrine (Strategy)

**Goal**: Add a new combat doctrine for the Warfare domain.

1. **Define the doctrine**:
   ```yaml
   # packs/warfare-modern/doctrines/aggressive.yaml
   id: aggressive
   name: Aggressive Doctrine
   description: Prioritize offense over defense
   modifiers:
     attack_power: 1.15  # +15% attack
     armor: 0.85         # -15% armor
   ```

2. **Add tests**:
   ```csharp
   [Fact]
   public void AggressiveDoctrine_BoostsAttackPower()
   {
       var doctrine = _doctineRegistry.Get("aggressive");
       doctrine!.Modifiers["attack_power"].Should().Be(1.15f);
   }
   ```

3. **Validate**:
   ```bash
   dotnet run --project src/Tools/PackCompiler -- validate packs/warfare-modern
   ```

### Task: Add a Global Balance Configuration

**Goal**: Tweak economy settings globally without touching individual units.

1. **Create a balance pack**:
   ```yaml
   # packs/balance-tweaks/pack.yaml
   id: balance-tweaks
   name: Balance Tweaks
   version: 0.1.0
   type: balance
   ```

2. **Define modifiers**:
   ```yaml
   # packs/balance-tweaks/global-modifiers.yaml
   apply_to: all
   modifiers:
     gold_generation: 1.1  # +10% gold
     wood_generation: 0.9  # -10% wood
   ```

3. **Test the changes**:
   ```bash
   dotnet test --filter "BalanceModifiers"
   ```

### Task: Debugging Content Loading

**Goal**: A pack isn't loading. Find the issue.

1. **Check pack validity**:
   ```bash
   dotnet run --project src/Tools/PackCompiler -- validate packs/my-pack
   ```

2. **Read DINOForge debug logs**:
   ```bash
   tail -50 "G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option\BepInEx\dinoforge_debug.log"
   ```

3. **Check schema compliance**:
   ```bash
   # Manually validate against schemas/unit.schema.json
   dotnet run --project src/Tools/PackCompiler -- validate packs/my-pack --verbose
   ```

4. **Test with MockGameBridgeServer**:
   ```csharp
   [Fact]
   public async Task MyPack_LoadsWithoutErrors()
   {
       var loader = new ContentLoader();
       var result = await loader.LoadPackAsync("packs/my-pack");
       result.Errors.Should().BeEmpty();
   }
   ```

## Next Steps

- **Explore Existing Packs**: Study `packs/warfare-starwars/` and `packs/economy-balanced/` for real examples
- **Read Schemas**: Each content type has a JSON schema in `schemas/` describing all valid fields
- **Review Domain Code**: `src/Domains/Warfare/` shows the full domain plugin pattern
- **Run Tests**: Execute `dotnet test src/DINOForge.sln` to see tests in action
- **Check the Docs**: Visit [https://kooshapari.github.io/Dino](https://kooshapari.github.io/Dino) for comprehensive documentation

Happy developing!
