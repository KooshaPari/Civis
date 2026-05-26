# Creating Packs

Every mod in DINOForge is a **pack** — a self-contained, versioned, validated bundle of content definitions.

## Pack Types

| Type | Description | Example |
|------|-------------|---------|
| `content` | Units, buildings, projectiles, effects, icons, audio | New unit pack |
| `balance` | Stats, costs, spawn sizes, economy rates | Archer damage buff |
| `ruleset` | Research requirements, wave timings, victory conditions | Hard mode rules |
| `total_conversion` | Full faction swap with theme | Star Wars Clone Wars |
| `utility` | Debug tools, profilers, inspectors | Entity inspector |

## Directory Structure

```
packs/my-pack/
  pack.yaml              # Required — pack metadata
  units/                 # Unit definitions
    trooper.yaml
    sniper.yaml
  buildings/             # Building definitions
    barracks.yaml
  weapons/               # Weapon definitions
    rifle.yaml
  factions/              # Faction definitions
    my_faction.yaml
  doctrines/             # Doctrine definitions
  waves/                 # Wave template definitions
  assets/                # Icons, audio, VFX references
    icons/
    audio/
  localization/          # Text/string bundles
    en.yaml
```

## Manifest Format

Every pack requires a `pack.yaml`:

```yaml
id: my-awesome-pack
name: My Awesome Pack
version: 0.1.0
framework_version: ">=0.1.0 <1.0.0"
author: Your Name
type: content
description: Adds new units and buildings

depends_on:
  - dino-warfare-domain    # requires warfare plugin

conflicts_with:
  - some-incompatible-pack

load_order: 100            # lower = loads earlier

loads:
  factions:
    - my_faction
  units:
    - trooper
    - sniper
  buildings:
    - barracks
  weapons:
    - rifle

overrides:
  units:
    - vanilla_archer       # this pack overrides the vanilla archer

asset_policy:
  allow_generated: true
  allow_public_assets: true
  allow_borrowed_assets: only_with_permission
  credits_manifest: assets/CREDITS.md
```

### Required Fields

| Field | Format | Description |
|-------|--------|-------------|
| `id` | `^[a-z][a-z0-9-]*$` | Unique identifier |
| `name` | string | Human-readable name |
| `version` | semver | `0.1.0`, `1.2.3-beta.1` |
| `framework_version` | range | `>=0.1.0 <1.0.0` |
| `author` | string | Author name |
| `type` | enum | `content`, `balance`, `ruleset`, `total_conversion`, `utility` |

## Content Layering

DINOForge applies content in layers with clear priority:

```
Base Game (vanilla DINO)
  └── Framework Defaults
        └── Domain Plugin (e.g. Warfare)
              └── Pack Overrides (your mod)
```

Later layers override earlier ones. If two packs override the same unit, load order determines precedence.

## Dependencies and Conflicts

Packs declare explicit relationships:

- **`depends_on`** — Required packs that must be loaded first
- **`conflicts_with`** — Packs that cannot coexist
- **`framework_version`** — Compatible DINOForge version range

The pack compiler resolves the dependency graph, detects cycles, and flags conflicts before anything loads.

## Validation

Always validate before building:

```bash
# Validate a single pack
dotnet run --project src/Tools/PackCompiler -- validate packs/my-pack

# Validate all packs
dotnet run --project src/Tools/PackCompiler -- validate packs/
```

The validator checks:
- Schema conformance for all YAML files
- Required fields and correct types
- ID format and uniqueness
- Dependency resolution (no cycles, no missing deps)
- Asset reference integrity
- ECS registration conflict detection

## Building

```bash
dotnet run --project src/Tools/PackCompiler -- build packs/my-pack
```

The build pipeline:
1. Validates all schemas
2. Resolves references across files
3. Checks for missing assets
4. Checks for circular dependencies
5. Builds the pack artifact
6. Emits compatibility metadata

## Next Steps

- [Schema Reference](/reference/schemas) — Detailed schema docs for every content type
- [Registry System](/concepts/registry-system) — How registries merge pack content
- [Warfare Overview](/warfare/overview) — Build themed faction packs
