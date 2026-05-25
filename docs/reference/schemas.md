# Schema Reference

DINOForge uses JSON Schema and YAML Schema definitions to validate all pack content before runtime. Schemas live in the `schemas/` directory at the repository root.

## Available Schemas

| Schema | File | Description |
|--------|------|-------------|
| Pack Manifest | `pack-manifest.schema.yaml` / `.json` | Required metadata for every pack |
| Unit | `unit.schema.yaml` / `.json` | Unit definitions with stats, weapons, behavior |
| Faction | `faction.schema.yaml` / `.json` | Faction identity, roster, economy, visuals |
| Building | `building.schema.json` | Structure definitions |
| Weapon | `weapon.schema.json` | Weapon class definitions |
| Projectile | `projectile.schema.json` | Projectile behavior and visuals |
| Doctrine | `doctrine.schema.json` | Combat doctrine modifiers |
| Skill | `skill.schema.json` | Unit abilities and special actions |
| Wave | `wave.schema.json` | Enemy wave composition templates |
| Squad | `squad.schema.json` | Squad formation definitions |
| Stat Override | `stat-override.schema.json` | Balance mod stat modifications |
| Scenario | `scenario.schema.json` | Scenario scripting, conditions, victory/loss |
| Economy Profile | `economy-profile.schema.json` | Resource rates, trade, economy balance |
| Universe Bible | `universe-bible.json` | Total conversion lore, naming, style guides |
| Asset Manifest | `asset-manifest.schema.json` | Intake manifest with technical and IP provenance states |

## Pack Manifest

Every pack must include a `pack.yaml` conforming to this schema.

### Required Fields

| Field | Type | Pattern | Description |
|-------|------|---------|-------------|
| `id` | string | `^[a-z][a-z0-9-]*$` | Unique pack identifier |
| `name` | string | — | Human-readable name |
| `version` | string | semver | Semantic version (`0.1.0`) |
| `framework_version` | string | range | Compatible DINOForge range |
| `author` | string | — | Author name |
| `type` | enum | — | `content`, `balance`, `ruleset`, `total_conversion`, `utility` |

### Optional Fields

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `description` | string | — | Brief description |
| `depends_on` | string[] | `[]` | Required pack IDs |
| `conflicts_with` | string[] | `[]` | Incompatible pack IDs |
| `load_order` | integer | `100` | Load priority (lower = earlier) |
| `game_version` | string | — | Compatible DINO version range |
| `loads` | object | — | Content declarations |
| `overrides` | object | — | Registry entries this pack overrides |
| `asset_policy` | object | — | Asset sourcing metadata |

### Content Declarations (`loads`)

```yaml
loads:
  factions: []
  units: []
  buildings: []
  weapons: []
  projectiles: []
  effects: []
  doctrines: []
  audio: []
  visuals: []
  localization: []
  wave_templates: []
  tech_nodes: []
  scenarios: []
```

## Unit Schema

Units define combat entities with stats, weapons, defense tags, and behavior.

### Required Fields

| Field | Type | Description |
|-------|------|-------------|
| `id` | string (`^[a-z][a-z0-9_]*$`) | Unique unit identifier |
| `display_name` | string | Shown in-game |
| `unit_class` | enum | Combat classification |
| `faction_id` | string | Owning faction |
| `stats` | object | HP, damage, range, speed, cost |

### Unit Classes

`MilitiaLight`, `CoreLineInfantry`, `EliteLineInfantry`, `HeavyInfantry`, `Skirmisher`, `AntiArmor`, `ShockMelee`, `SwarmFodder`, `FastVehicle`, `MainBattleVehicle`, `HeavySiege`, `Artillery`, `WalkerHeavy`, `StaticMG`, `StaticAT`, `StaticArtillery`, `SupportEngineer`, `Recon`, `HeroCommander`, `AirstrikeProxy`, `ShieldedElite`

### Stats Block

```yaml
stats:
  hp: 100          # minimum: 1
  damage: 15       # minimum: 0
  armor: 0         # default: 0
  range: 6         # minimum: 0
  speed: 3.5       # minimum: 0
  cost:
    resource_1: 30
    resource_2: 10
    resource_3: 0
    resource_4: 0
    population: 1
  accuracy: 0.7    # 0.0 - 1.0
  fire_rate: 1.0   # minimum: 0
  morale: 100
```

### Defense Tags

`Unarmored`, `InfantryArmor`, `HeavyArmor`, `Fortified`, `Shielded`, `Mechanical`, `Biological`, `Heroic`

### Behavior Tags

`HoldLine`, `AdvanceFire`, `Charge`, `Kite`, `Swarm`, `SiegePriority`, `AntiStructure`, `AntiMass`, `AntiArmor`, `MoralePressure`

## Faction Schema

Factions define a complete army identity: roster, economy modifiers, visuals, and audio.

### Required Sections

| Section | Description |
|---------|-------------|
| `faction` | Identity: id, theme, archetype, display name |
| `economy` | Gather bonus, upkeep, research/build speed |
| `army` | Morale style, unit cap, elite cost, spawn rate |
| `roster` | Maps abstract roles to concrete unit IDs |
| `buildings` | Maps abstract building roles to concrete IDs |
| `visuals` | Colors, projectile pack, UI skin |
| `audio` | Weapon, structure, ambient, music packs |

### Themes

`starwars`, `modern`, `futuristic`, `fantasy`, `custom`

### Archetypes

`order`, `industrial_swarm`, `asymmetric`, `custom`

### Roster Slots

`cheap_infantry`, `line_infantry`, `elite_infantry`, `anti_armor`, `support_weapon`, `recon`, `light_vehicle`, `heavy_vehicle`, `artillery`, `hero_commander`, `spike_unit`

## Weapon Schema

Weapon classes used in the `weapon` field of unit definitions:

`BallisticLight`, `BallisticHeavy`, `BlasterLight`, `BlasterHeavy`, `ExplosiveAT`, `ExplosiveHE`, `BeamPrecision`, `FlameArea`, `MissileGuided`, `SuppressionWeapon`, `MeleeLight`, `MeleeHeavy`
