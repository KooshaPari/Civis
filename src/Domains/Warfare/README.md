# DINOForge.Domains.Warfare

NuGet package for combat simulation, faction archetypes, doctrines, and combat roles in DINOForge.

## Overview

The Warfare domain provides:
- **Archetypes**: Define faction unit compositions (West, ClassicEnemy, etc.)
- **Doctrines**: Combat strategies with synergy bonuses and unit restrictions
- **Roles**: Unit combat classifications (Tank, DPS, Support, Ranged, etc.)
- **Waves**: Enemy wave composition and difficulty scaling
- **Combat Balance**: Stat modifiers, synergy calculations, damage calculations

## Installation

```bash
dotnet add package DINOForge.Domains.Warfare --version 0.18.0
```

## Key Classes

- **ArchetypeRegistry**: Manages faction unit archetypes
- **DoctrineRegistry**: Stores combat doctrines with synergy definitions
- **RoleRegistry**: Maps units to combat roles
- **WaveRegistry**: Defines enemy wave patterns
- **CombatBalanceCalculator**: Computes stat modifiers and damage

## Usage Example

```csharp
using DINOForge.Domains.Warfare;
using DINOForge.SDK.Registry;

// Resolve the archetype registry
var archetypeRegistry = serviceProvider.GetRequiredService<IRegistry<Archetype>>();

// Query factions
var westArchetype = archetypeRegistry.Get("west");
Console.WriteLine($"Units: {westArchetype.Units.Count}");

// Apply doctrine bonuses
var doctrine = doctrineRegistry.Get("blitzkrieg");
var synergy = doctrine.CalculateSynergy(unitIds);
```

## Testing

All Warfare functionality is covered by 31+ unit tests in `src/Tests/WarfareTests/`:
- ArchetypeTests
- DoctrineTests
- RoleTests
- WaveTests
- BalanceCalculatorTests

Run tests:
```bash
dotnet test src/Tests/ --filter "Category=Warfare"
```

## Dependencies

- **DINOForge.SDK** (0.18.0+) - Registry, validation, models

## License

MIT

## Contributing

See [CONTRIBUTING.md](https://github.com/KooshaPari/Dino/blob/main/CONTRIBUTING.md) in the main repository.
