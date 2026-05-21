# PACK_NAME

> A DINOForge mod pack for [Diplomacy is Not an Option](https://store.steampowered.com/app/1175020/).

**Author**: PACK_AUTHOR
**Type**: PACK_TYPE
**DINOForge**: `>=0.24.0 <0.26.0`

## Installation

1. Download and run [DINOForge Installer](https://github.com/KooshaPari/Dino/releases/latest)
2. Copy this pack folder to `BepInEx/dinoforge_packs/PACK_ID/`
3. Launch Diplomacy is Not an Option
4. Press F10 to open the DINOForge mod menu and enable PACK_NAME

## Development

```bash
# Install DINOForge PackCompiler
dotnet tool install -g dinoforge

# Validate pack
dinoforge validate .

# Build pack (creates .dinopack zip)
dinoforge build .
```

## Pack Structure

```
PACK_ID/
  pack.yaml            # Manifest: id, version, dependencies
  asset_pipeline.yaml  # Optional asset import/optimize/generate config
  units/               # Unit definitions
  buildings/           # Building definitions
  factions/            # Faction definitions
  stats/               # Stat override files (work immediately)
  weapons/             # Weapon definitions
  doctrines/           # Doctrine definitions
  scenarios/           # Scenario definitions (victory/defeat/events)
  economy/             # Economy profiles, resources, trade routes
  ui/                  # UI overlays and menu definitions
  hud_elements/        # HUD element definitions
  themes/              # Theme/palette definitions
  assets/              # Asset bundles, models, textures
```

## License

MIT
