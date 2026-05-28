# `dinoforge pack diff` Command

Compares two packs and shows overlap/conflicts in units, buildings, factions, weapons, and doctrines.

## Usage

```bash
dinoforge pack diff <packA> <packB> [OPTIONS]
```

### Arguments

- `packA`: First pack ID to compare (e.g., `warfare-starwars`)
- `packB`: Second pack ID to compare (e.g., `warfare-modern`)

### Options

- `--format <format>`: Output format: `table` (default) or `json`
- `--show-stats`: Show stat-level differences for entities that exist in both packs

## Examples

### Basic comparison (table format)
```bash
dinoforge pack diff warfare-starwars warfare-modern
```

Output: A three-column table showing:
- Entities only in Pack A (green)
- Entities only in Pack B (blue)
- Entities in both packs (yellow)

### With detailed stat differences
```bash
dinoforge pack diff warfare-starwars warfare-modern --show-stats
```

Adds a section showing stat-by-stat differences for entities that appear in both packs.

### Machine-readable output
```bash
dinoforge pack diff warfare-starwars warfare-modern --format json
```

Returns JSON with structure:
```json
{
  "packA": "warfare-starwars",
  "packB": "warfare-modern",
  "units": {
    "onlyInA": ["rep_clone_militia", "rep_clone_trooper", ...],
    "onlyInB": ["western_rifleman", "western_squad", ...],
    "inBoth": ["rep_v19_torrent", "cis_tri_fighter"],
    "statDiffs": {
      "rep_v19_torrent": {
        "hp": [110.0, 125.0],
        "damage": [18.0, 20.0]
      }
    }
  },
  "buildings": { ... },
  "factions": { ... },
  "weapons": { ... },
  "doctrines": { ... }
}
```

## Use Cases

### Content Planning
Identify which units/buildings are unique to each faction pack to avoid accidental duplication when creating new packs.

### Conflict Detection
Find overlapping IDs that would cause registry conflicts if both packs are enabled.

### Balance Comparison
Use `--show-stats` to compare stat distributions between warfare packs (e.g., medieval vs modern vs Star Wars).

### Pack Merging
When combining content from multiple packs, use the diff to understand what IDs are already defined.

## Categories Compared

1. **Units** - Combat units (militia, infantry, specialists, vehicles)
2. **Buildings** - Structures (barracks, towers, factories, labs)
3. **Factions** - Faction definitions (republic, enemy, western alliance, etc.)
4. **Weapons** - Weapon definitions used by units
5. **Doctrines** - Military doctrines and bonuses

## Output Colors (Table Format)

- **Green**: Entities only in the first pack
- **Blue**: Entities only in the second pack
- **Yellow**: Entities in both packs (potential conflicts)

## Technical Details

The command:
1. Reads `pack.yaml` manifest from both packs
2. Loads YAML files referenced in the `loads:` section
3. Parses each entity by its `id:` field
4. Categorizes entities into three groups: A-only, B-only, in-both
5. Optionally computes stat-level diffs using reflection on the loaded objects
6. Formats output as table or JSON

## Notes

- Pack paths are resolved relative to the repository root (via `.git` detection)
- Both pack directories must exist in `packs/<pack-id>/`
- All YAML files are parsed using YamlDotNet for consistent handling
- Stat diffs only display fields that differ between the two entities
