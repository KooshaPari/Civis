# dinoforge-pack-new

Scaffold a new mod pack with guided prompts.

**Usage**: `/dinoforge-pack-new`

## Purpose

Creates a new pack directory with a complete `pack.yaml` manifest, directory structure, and example content stubs. Walks through a series of prompts to set up the pack metadata.

## Prompts

The skill will ask:

1. **Pack ID** (kebab-case, e.g., `my-warfare-mod`)
   - Must be unique (checked against existing packs/)
   - Used as the directory name and unique identifier

2. **Pack Name** (human-readable, e.g., "My Warfare Mod")
   - Displayed in game UI and mod managers

3. **Pack Type**
   - `content`: New units, buildings, factions (default)
   - `balance`: Stat tweaks, archetype rebalance
   - `ruleset`: New victory/defeat conditions, scenarios
   - `scenario`: Single-map story/campaign
   - `total_conversion`: Replaces vanilla factions + all units
   - `utility`: Tools, helpers, no gameplay changes

4. **Pack Version** (semver, e.g., `0.1.0`)
   - Default: `0.1.0`

5. **Author Name** (your name, e.g., "Jane Modder")
   - Stored in pack.yaml `author` field

6. **Pack Description** (one-liner explaining the mod)
   - Stored in `pack.yaml` `description`

7. **Dependencies** (if any; optional)
   - Comma-separated list of pack IDs this pack requires
   - Default: empty

8. **Conflicts** (if any; optional)
   - Comma-separated list of pack IDs this pack cannot coexist with
   - Default: empty

9. **Framework Version** (semver range, e.g., `>=0.1.0 <1.0.0`)
   - Minimum/maximum compatible DINOForge versions
   - Default: `>=0.1.0 <1.0.0`

## Output

Creates:
- `packs/<pack-id>/`
  - `pack.yaml` — manifest with all metadata
  - `README.md` — template for pack documentation
  - `units/` — empty directory for unit definitions
  - `buildings/` — empty directory for building definitions
  - `factions/` — empty directory for faction definitions
  - `assets/` — empty directory for visual assets (bundles, textures)

## Next Steps (after creation)

```bash
# Validate the pack structure
dotnet run --project src/Tools/PackCompiler -- validate packs/<pack-id>/

# Add a unit
/add-unit <pack-id> <unit-id> <unit-class>

# Deploy to game
/pack-deploy <pack-id>
```

## Use When

- Starting a new mod project
- Creating a content pack for a friend
- Prototyping a balance mod

## Time

~30 seconds (prompts + directory scaffold).
