---
title: Quick Start
description: Create your first DINOForge pack in 5 minutes
---

# Quick Start — 5-Minute Pack Creation

Create a minimal balance pack in under 5 minutes.

## Prerequisites

- DINOForge installed ([Getting Started](/guide/getting-started))
- `.NET 8 SDK` installed
- A text editor or IDE

## 1. Create the Pack Directory

```bash
mkdir -p packs/my-balance-mod
cd packs/my-balance-mod
```

## 2. Write the Manifest

Create `pack.yaml`:

```yaml
id: my-balance-mod
name: My Balance Mod
version: 0.1.0
framework_version: ">=0.1.0 <1.0.0"
author: Your Name
type: balance
description: Doubles archer damage and increases wall HP

depends_on: []
conflicts_with: []

loads:
  units:
    - archer_buffed
  buildings: []
```

::: info Pack Metadata
- `id`: Must match `^[a-z][a-z0-9-]*$` (lowercase, hyphens OK)
- `version`: Semantic versioning (`MAJOR.MINOR.PATCH`)
- `type`: One of `content`, `balance`, `ruleset`, `total_conversion`, `utility`
- `framework_version`: Supported DINOForge versions (uses semver ranges)
:::

## 3. Create Content Directories

```bash
mkdir units
mkdir buildings
```

## 4. Define a Unit Override

Create `units/archer_buffed.yaml`:

```yaml
id: archer_buffed
display_name: Elite Archer
unit_class: CoreLineInfantry
faction_id: vanilla
tier: 1

stats:
  hp: 100
  damage: 25          # doubled from vanilla ~12
  armor: 0
  range: 8
  speed: 3.5
  accuracy: 0.8
  fire_rate: 1.2
  morale: 85
  cost:
    food: 40
    wood: 10

weapon: BallisticLight

defense_tags:
  - Unarmored
  - Biological

behavior_tags:
  - HoldLine
  - AdvanceFire

vanilla_mapping: archer
```

::: tip Stats Fields
All stats are optional and default to reasonable values if omitted:
- `hp`: Hit points (default: 1)
- `damage`: Base damage per attack (default: 0)
- `armor`: Damage reduction (default: 0)
- `range`: Attack range in world units (default: 0 = melee)
- `speed`: Movement speed (default: 0)
- `accuracy`: Hit chance 0.0-1.0 (default: 0.7)
- `fire_rate`: Attacks per second (default: 1.0)
- `morale`: Base morale (default: 100)
:::

## 5. Validate the Pack

```bash
dotnet run --project src/Tools/PackCompiler -- validate packs/my-balance-mod
```

Expected output:
```
✓ Pack validation passed
  Schema: OK
  References: OK
  Dependencies: OK
```

The validator checks:
- Pack manifest schema conformance
- Unit YAML schema conformance
- Required fields present and correct types
- ID format (`^[a-z][a-z0-9-]*$` for packs, `^[a-z][a-z0-9_]*$` for content IDs)
- Version format (semver)
- Dependency resolution (no cycles, no missing packs)

## 6. Build the Pack

```bash
dotnet run --project src/Tools/PackCompiler -- build packs/my-balance-mod
```

Your pack is now ready. The Runtime will discover and load it automatically at game boot.

## Test in Game

1. Launch **Diplomacy is Not an Option**
2. Press **F10** to open the DINOForge mod menu
3. You should see "My Balance Mod" listed with status "Loaded"
4. Your unit override is now active — test it in the game!

## What You Just Built

- **Pack manifest** with explicit metadata (id, version, author, dependencies)
- **Unit definition** with complete stats (HP, damage, armor, cost, behavior)
- **Validated** against DINOForge's JSON schema system
- **Built** as a distributable artifact

**No C# code. No Harmony patches. No reverse engineering. Just YAML.**

## Next Steps

- [Creating Packs](/guide/creating-packs) — Full guide to all pack types
- [Unit Schema Reference](/reference/unit-schema) — All unit fields documented
- [Building Schema Reference](/reference/building-schema) — Building definitions
- [Warfare Overview](/warfare/overview) — Learn factions and unit archetypes
