# Example Total Conversion Pack

A minimal example demonstrating the DINOForge mod system for mod authors.

## What This Pack Demonstrates

**example-total-conversion** is designed to teach new mod authors how DINOForge works by providing:

1. **Pack Manifest** (`pack.yaml`)
   - Total conversion pack type (replaces game factions/units entirely)
   - Framework version constraint (`>=0.24.0 <1.0.0`)
   - Declarative content loading paths

2. **Faction Definition** (`factions/sentinels.yaml`)
   - Custom faction with economy, army, and roster entries
   - Theme, archetype, and visual configuration
   - Minimal but complete faction structure

3. **Unit Definition** (`units/sentinel-trooper.yaml`)
   - Unit registered to the Sentinels faction via `faction_id`
   - Visual asset reference (`visual_asset: sentinel-trooper-bundle`)
   - Stat profile with cost structure
   - Vanilla game mapping (vanilla_mapping: line_infantry)

## How to Use This Pack

### 1. Install
Copy this pack to your DINOForge mods directory, then load:
```bash
dotnet run --project src/Tools/PackCompiler -- validate packs/example-total-conversion
```

### 2. Customize
Edit the YAML files to create your own:
- Modify faction colors, economy multipliers, and roster entries
- Add more units by copying sentinel-trooper.yaml
- Add buildings, doctrines, and waves by extending pack.yaml

### 3. Deploy to Game
```bash
dotnet build src/DINOForge.sln -c Release -p:DeployToGame=true
```

The Runtime will automatically load and deserialize your pack into the mod registries.

## Key Patterns Demonstrated

- **Universe Bible System**: Factions and units registered to a global registry
- **Schema Validation**: All YAML files validated against schemas/ definitions
- **Faction References**: Units use `faction_id` to link to parent factions
- **Bundle References**: Units reference visual assets via `visual_asset` key (Phase 1 abstract, Phase 2 runtime swap)
- **Framework Versioning**: Compatible with DINOForge v0.24.0+

## Next Steps

After understanding this pack:
1. Add more units with unique stats
2. Create custom factions with different archetypes (chaos, ranged, etc.)
3. Add doctrines via `doctrines/` directory
4. Create waves for campaign sequences

See CLAUDE.md for full API and pattern guidance.
