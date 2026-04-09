# Interactive API Reference

This guide provides practical, copy-paste ready code examples for DINOForge's core APIs. Whether you're building a new pack or extending the platform, these examples demonstrate real-world usage patterns.

## Registry API Usage

The Registry system is the foundation of DINOForge. All game entities (units, buildings, weapons, factions) are registered through typed registries that provide type-safe, conflict-aware management.

### Registering a Custom Unit

```csharp
using DINOForge.SDK.Registry;
using DINOForge.SDK.Models;

// Create a new unit definition
var unitRegistry = new UnitRegistry();

var customUnit = new UnitDefinition
{
    Id = "my-heavy-tank",
    Name = "Custom Heavy Tank",
    DisplayName = "Heavy Tank",
    Description = "A heavily armored tank with excellent firepower",
    Health = 150,
    Cost = 500,
    BuildTime = 45,
    FactionId = "my-faction",
    UnitType = UnitType.Tank,
    MovementSpeed = 4.5f,
    IsElite = false,
    RequiredTechs = new[] { "advanced-armor", "tank-engineering" }
};

// Register the unit (with conflict detection)
try
{
    unitRegistry.Register(customUnit);
    Console.WriteLine($"Unit '{customUnit.Name}' registered successfully!");
}
catch (RegistryConflictException ex)
{
    Console.WriteLine($"Conflict detected: {ex.Message}");
    // Handle duplicate IDs, version conflicts, etc.
}

// Query the registry
var registeredUnit = unitRegistry.Get("my-heavy-tank");
Console.WriteLine($"Unit cost: {registeredUnit.Cost}");
```

### Building a Faction Registry

```csharp
using DINOForge.SDK.Registry;

var factionRegistry = new FactionRegistry();

var customFaction = new FactionDefinition
{
    Id = "my-faction",
    Name = "Custom Faction",
    DisplayName = "The Custom Empire",
    Description = "A rising power in the world",
    Color = "#FF6B35",  // Hex color code
    IsPlayable = true,
    IsNeutral = false,
    StartingResources = new ResourceAllocation
    {
        Gold = 1000,
        Wood = 500,
        Stone = 300
    },
    Traits = new[] { "economically-advanced", "militarily-strong" }
};

factionRegistry.Register(customFaction);

// List all registered factions
foreach (var faction in factionRegistry.GetAll())
{
    Console.WriteLine($"- {faction.DisplayName} ({faction.Id})");
}
```

### Adding Building Definitions

```csharp
var buildingRegistry = new BuildingRegistry();

var customBuilding = new BuildingDefinition
{
    Id = "my-armory",
    Name = "Advanced Armory",
    DisplayName = "Advanced Armory",
    Description = "Produces advanced weapons and armor upgrades",
    Health = 200,
    BuildCost = new ResourceCost { Gold = 800, Wood = 400 },
    BuildTime = 60,
    ProductionRate = 1.2f,
    FactionId = "my-faction",
    RequiredTechs = new[] { "metalworking", "engineering" },
    UnitsProduced = new[] { "heavy-sword", "plate-armor" },
    IsOffensive = false
};

buildingRegistry.Register(customBuilding);

// Verify registration with conflict detection
if (buildingRegistry.Contains("my-armory"))
{
    Console.WriteLine("Building registered and ready for use!");
}
```

## Pack Manifest Structure

Every DINOForge mod is a **pack** with a declarative YAML manifest. This manifest defines metadata, dependencies, version constraints, and content declarations.

### Basic Pack Manifest

```yaml
# pack.yaml - Complete pack manifest example
id: my-custom-mod
version: 0.1.0
name: My Custom Mod
display_name: "My Custom Mod v0.1"
author: "Your Name"
description: "A custom mod that adds new units and factions to DINO"
repository: "https://github.com/yourusername/my-custom-mod"

# Framework compatibility
framework_version: ">=0.14.0 <1.0.0"

# Pack type: content, balance, ruleset, total_conversion, utility
type: content

# Optional: pack icon and preview image
icon: assets/icon.png
preview: assets/preview.jpg

# Dependencies on other packs
depends_on:
  - id: example-balance
    version: ">=0.1.0"
    optional: false
  - id: warfare-modern
    version: "^1.0.0"
    optional: true

# Packs this mod conflicts with
conflicts_with:
  - warfare-starwars    # Incompatible asset palettes
  - total-conversion-aliens

# Content declarations (which definitions this pack loads)
loads:
  factions:
    - my-faction
    - my-ally-faction
  
  units:
    - my-heavy-tank
    - my-advanced-soldier
    - my-support-unit
  
  buildings:
    - my-armory
    - my-research-center
  
  weapons:
    - my-laser-rifle
    - my-plasma-cannon
  
  doctrines:
    - my-combat-doctrine
    - my-economic-doctrine
  
  technologies:
    - advanced-armor
    - tank-engineering
    - energy-weapons

# Custom metadata (optional)
metadata:
  tags:
    - gameplay
    - balance
    - new-content
  difficulty_level: intermediate
  estimated_playtime_hours: 20
  supports_multiplayer: true
```

### Manifest with Asset References

```yaml
id: my-visual-mod
version: 1.0.0
name: Custom Unit Visuals
type: content

# Asset bundle references
assets:
  unit_models:
    - id: my-heavy-tank
      bundle: my-heavy-tank-model  # References: packs/my-visual-mod/assets/bundles/my-heavy-tank-model
      format: prefab
      lod_levels: 3
    - id: my-soldier
      bundle: my-soldier-model
      format: prefab
      lod_levels: 2

# Visual asset assignments (in unit definitions)
loads:
  units:
    - my-heavy-tank          # Definition includes: visual_asset: my-heavy-tank-model
    - my-soldier             # Definition includes: visual_asset: my-soldier-model
```

### Pack with Doctrine Definitions

```yaml
id: my-doctrines
version: 0.5.0
name: Custom Combat Doctrines
type: ruleset
description: Advanced combat doctrines for strategic gameplay

depends_on:
  - id: example-balance
    version: ">=0.1.0"

loads:
  doctrines:
    - aggressive-tactics
    - defensive-formation
    - economic-focus
  
  # Doctrines may also declare stat modifiers
  stat_modifiers:
    - unit_attack_bonus_aggressive
    - building_defense_bonus_defensive
    - resource_production_bonus_economic
```

## Domain Plugin Integration

Domain plugins extend DINOForge with new gameplay systems. Here's how to build and integrate one programmatically.

### Loading a Domain Plugin

```csharp
using DINOForge.SDK;
using DINOForge.Domains.Warfare;
using DINOForge.Runtime;

public class ModInitializer
{
    public static void Initialize()
    {
        // Create the content loader
        var contentLoader = new ContentLoader();

        // Initialize the Warfare domain plugin
        var warfarePlugin = new WarfareDomainPlugin();
        warfarePlugin.Initialize(contentLoader);

        Console.WriteLine("Warfare domain initialized with:");
        Console.WriteLine($"  - {warfarePlugin.RegisteredFactions} factions");
        Console.WriteLine($"  - {warfarePlugin.RegisteredUnits} unit types");
        Console.WriteLine($"  - {warfarePlugin.RegisteredDoctrines} combat doctrines");
    }
}
```

### Creating a Custom Domain Plugin

```csharp
using DINOForge.SDK;
using DINOForge.SDK.Registry;

public class CustomDomainPlugin : IDomainPlugin
{
    private UnitRegistry unitRegistry;
    private BuildingRegistry buildingRegistry;
    private FactoryRegistry<WeaponDefinition> weaponRegistry;

    public string PluginId => "custom-domain";
    public string DisplayName => "Custom Domain Plugin";
    public string Version => "1.0.0";

    public void Initialize(IContentLoader contentLoader)
    {
        // Initialize registries
        unitRegistry = new UnitRegistry();
        buildingRegistry = new BuildingRegistry();
        weaponRegistry = new FactoryRegistry<WeaponDefinition>();

        // Load pack content
        var packs = contentLoader.LoadPacks(new[] { "my-custom-mod", "my-visual-mod" });
        
        foreach (var pack in packs)
        {
            LoadPackContent(pack);
        }

        Console.WriteLine($"Custom domain initialized with {packs.Length} packs");
    }

    private void LoadPackContent(ModPack pack)
    {
        // Load units from pack manifests
        foreach (var unitId in pack.Manifest.Loads.Units)
        {
            var unitDef = pack.GetUnitDefinition(unitId);
            unitRegistry.Register(unitDef);
        }

        // Load buildings
        foreach (var buildingId in pack.Manifest.Loads.Buildings)
        {
            var buildingDef = pack.GetBuildingDefinition(buildingId);
            buildingRegistry.Register(buildingDef);
        }
    }

    public void Dispose()
    {
        unitRegistry?.Dispose();
        buildingRegistry?.Dispose();
        weaponRegistry?.Dispose();
    }
}
```

### Accessing Domain-Specific Services

```csharp
using DINOForge.Domains.Warfare;
using DINOForge.Domains.Economy;

// Warfare domain example
var warfareService = contentLoader.GetDomainService<IWarfareService>();
var doctrines = warfareService.GetAllDoctrines();
var roles = warfareService.GetUnitRoles("my-faction");

// Economy domain example
var economyService = contentLoader.GetDomainService<IEconomyService>();
var tradeRoutes = economyService.CalculateProfitableRoutes();
var productionRates = economyService.GetProductionRateByResource("wood");

// Scenario domain example
var scenarioService = contentLoader.GetDomainService<IScenarioService>();
var victoryConditions = scenarioService.GetVictoryConditions();
var difficulty = scenarioService.GetScenarioDifficulty("tutorial-scenario");
```

## MCP Bridge Tool Examples

The MCP (Model Context Protocol) server provides JSON-RPC tools for game automation and integration. Use these to interact with the running game instance, query entities, apply stat overrides, and trigger hot-reload.

### Querying Game Status

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "game_status",
  "params": {}
}
```

**Response:**
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "is_running": true,
    "current_scene": "gameplay",
    "entity_count": 45776,
    "active_worlds": 6,
    "loaded_packs": [
      "example-balance",
      "warfare-modern"
    ],
    "uptime_seconds": 3600
  }
}
```

### Querying Entities by Component Type

```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "method": "game_query_entities",
  "params": {
    "component_types": ["Health", "AttackCooldown"],
    "limit": 10
  }
}
```

**Response:**
```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "result": {
    "entities": [
      {
        "entity_id": 12345,
        "components": {
          "Health": { "current": 100, "max": 150 },
          "AttackCooldown": { "remaining": 2.5, "base": 5.0 }
        }
      },
      {
        "entity_id": 12346,
        "components": {
          "Health": { "current": 75, "max": 100 },
          "AttackCooldown": { "remaining": 0.0, "base": 3.0 }
        }
      }
    ],
    "total_matched": 247,
    "returned_count": 10
  }
}
```

### Getting a Stat Value

```json
{
  "jsonrpc": "2.0",
  "id": 3,
  "method": "game_get_stat",
  "params": {
    "entity_id": 12345,
    "stat_name": "Health"
  }
}
```

**Response:**
```json
{
  "jsonrpc": "2.0",
  "id": 3,
  "result": {
    "value": 100,
    "max_value": 150,
    "stat_name": "Health",
    "stat_type": "component",
    "source": "ArmorData"
  }
}
```

### Applying a Stat Override

```json
{
  "jsonrpc": "2.0",
  "id": 4,
  "method": "game_apply_override",
  "params": {
    "entity_id": 12345,
    "stat_name": "AttackDamage",
    "value": 50.0,
    "duration_seconds": 30.0,
    "reason": "Testing balance change"
  }
}
```

**Response:**
```json
{
  "jsonrpc": "2.0",
  "id": 4,
  "result": {
    "override_applied": true,
    "override_id": "override-abc123",
    "original_value": 40.0,
    "modified_value": 50.0,
    "expires_at": 1680000000,
    "entity_id": 12345
  }
}
```

### Reloading Packs (Hot Reload)

```json
{
  "jsonrpc": "2.0",
  "id": 5,
  "method": "game_reload_packs",
  "params": {
    "pack_ids": ["my-custom-mod"],
    "reload_assets": true,
    "preserve_entity_state": true
  }
}
```

**Response:**
```json
{
  "jsonrpc": "2.0",
  "id": 5,
  "result": {
    "packs_reloaded": 1,
    "reload_duration_ms": 1245,
    "assets_reloaded": 15,
    "errors": [],
    "warnings": ["Some entities will require respawn to see visual changes"],
    "status": "success"
  }
}
```

### Taking a Screenshot with Analysis

```json
{
  "jsonrpc": "2.0",
  "id": 6,
  "method": "game_screenshot",
  "params": {
    "filename": "gameplay-state.png",
    "analyze_ui": true
  }
}
```

**Response:**
```json
{
  "jsonrpc": "2.0",
  "id": 6,
  "result": {
    "filename": "gameplay-state.png",
    "file_path": "/path/to/screenshots/gameplay-state.png",
    "resolution": "1920x1080",
    "timestamp": 1680000000,
    "ui_elements_detected": [
      {
        "type": "health_bar",
        "position": [640, 720],
        "confidence": 0.95
      },
      {
        "type": "unit_portrait",
        "position": [50, 50],
        "confidence": 0.89
      }
    ]
  }
}
```

### Navigating to a Game State

```json
{
  "jsonrpc": "2.0",
  "id": 7,
  "method": "game_navigate_to",
  "params": {
    "target_state": "gameplay",
    "timeout_seconds": 10
  }
}
```

**Response:**
```json
{
  "jsonrpc": "2.0",
  "id": 7,
  "result": {
    "current_state": "gameplay",
    "navigation_successful": true,
    "steps_executed": 3,
    "time_elapsed_ms": 2500
  }
}
```

## Usage Tips

- **Registry Conflicts**: The registry system automatically detects duplicate IDs and version conflicts. Catch `RegistryConflictException` to handle these gracefully.
- **Pack Dependencies**: Always declare pack dependencies in your manifest. The system validates dependency graphs at load time.
- **Stat Overrides**: Stat overrides are temporary (duration-based) and useful for testing balance changes without restarting the game.
- **Hot Reload**: Use hot reload during development to iterate quickly. It preserves entity state and game progress while updating content.
- **JSON-RPC Calls**: The MCP server is HTTP-based on port 8765. All method calls are JSON-RPC 2.0 compliant.

For more detailed API documentation, see the [Registry API Reference](/api/registry), [Domain Plugins](/concepts/domain-plugins), and [MCP Tools](/guide/mcp-bridge).
