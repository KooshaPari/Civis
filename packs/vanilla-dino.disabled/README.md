# Vanilla DINO - Canonical Reference Pack

The **vanilla-dino** pack is the authoritative reference for all 100+ vanilla Diplomacy is Not an Option game units, factions, buildings, weapons, technologies, and doctrines. It serves as the foundational baseline for all mods to extend, map to, and inherit from.

## Purpose

- **Canonical Baseline**: Codifies all vanilla game mechanics in structured YAML format
- **Mod Foundation**: Enables mods to reference vanilla units via `vanilla_mapping` field for stat inheritance
- **CRUD Operations**: Facilitates efficient creation, modification, and deletion of units/factions without duplicating stats
- **Naming Conventions**: Establishes consistent ID naming, display names, and wiki references across the ecosystem
- **Load Order**: Marked with `load_order: 10` to load before all mods

## File Structure

```
vanilla-dino/
├── pack.yaml                              # Master manifest
├── units/
│   ├── lords-troops-units.yaml           # 14 units
│   ├── rebels-units.yaml                 # 13 units
│   ├── royal-army-units.yaml             # 15 units
│   ├── sarranga-units.yaml               # 7 units
│   ├── undead-units.yaml                 # 23 units
│   └── bugs-units.yaml                   # 5 units
├── factions/
│   ├── lords-troops.yaml
│   ├── rebels.yaml
│   ├── royal-army.yaml
│   ├── sarranga.yaml
│   ├── undead.yaml
│   └── bugs.yaml
├── buildings/
│   ├── lords-troops-buildings.yaml       # Faction-specific
│   ├── rebels-buildings.yaml
│   ├── royal-army-buildings.yaml
│   ├── sarranga-buildings.yaml
│   ├── undead-buildings.yaml
│   ├── bugs-buildings.yaml
│   └── economy-buildings.yaml            # Shared resource/defense/housing
├── weapons/
│   └── vanilla-weapons.yaml              # 30+ weapons
├── doctrines/
│   └── vanilla-doctrines.yaml            # 12 faction doctrines
├── technologies/
│   └── vanilla-technologies.yaml         # 25 research techs
└── README.md
```

## Content Overview

### Factions (6)

| Faction | Archetype | Units | Theme | Key Mechanic |
|---------|-----------|-------|-------|--------------|
| **Lords Troops** | Order | 14 | Medieval | Combined-arms balance, elite cavalry |
| **Rebels** | Chaos | 13 | Peasant uprising | Mass assault, volatile morale |
| **Royal Army** | Defense | 15 | Professional military | Disciplined formations, defensive strength |
| **Sarranga** | Magic | 7 | Mystical | Elemental specialists, magic-focused |
| **Undead** | Swarm | 23 | Necromantic | Corpse mastery, 1.3x unit cap, no morale |
| **Bugs** | Swarm | 5 | Insectoid | Hive coordination, 1.5x spawn rate, instinctive |

### Units (77 Total)

Each unit includes:
- **IDs**: vanilla-{faction}_{unit_name} (e.g., `vanilla_swordsman`, `vanilla_rebel_pitchfork`)
- **Stats**: HP, damage, armor, range, speed, accuracy, fire_rate, morale
- **Costs**: Food, wood, stone, iron, gold, population
- **Tags**: Defense tags (InfantryArmor, Biological, Mechanical, Exotic, Magical) + Behavior tags (HoldLine, Charge, Kite, SiegePriority, etc.)
- **Metadata**: vanilla_dino_name, wiki_reference, unit_class, tier (1-3)

**Example**: Vanilla Swordsman
```yaml
id: vanilla_swordsman
display_name: Swordsman
unit_class: MilitiaLight
stats:
  hp: 50.0
  damage: 8.0
  armor: 2.0
  speed: 5.5
  cost:
    food: 20
    iron: 5
    gold: 3
```

### Buildings (40+)

**Faction-Specific Barracks/Production**:
- Barracks I-III, Stables I-III, Engineer Guilds, Special structures (Lord's Hall, Mystical Circle, Crypt, etc.)

**Shared Economy & Defense** (15 buildings):
- **Resource Gathering**: Lumber Mill, Stone Mine, Farm, Fisherman's Hut, Berry Picker, Iron Mine, Gold Mine
- **Defense**: Wooden Gate, Stone Gate, Walls, Guard Tower, Stone Obelisk
- **Housing**: House Tier 1-3 (6/12/18 capacity)
- **Storage**: Granary, Storage Building, Market
- **Government**: Town Hall Tier 1-3 (unlocks iron mining, advanced buildings)
- **Special**: Hospital (health), University (research)

### Weapons (30+)

**Melee** (14): sword, axe, pike, hammer, lance, club, pitchfork, scythe, dagger, claws, enchanted variants, staffs, mandibles

**Ranged** (8): bow, crossbow, mounted bow, enchanted bow, catapult, ballista, trebuchet, magic projectile, firebomb

**Support** (2): magic staff, none

Each weapon includes:
- Damage type (slashing, piercing, cleaving, crushing, magical, fire, poison, acid)
- Base stats (base_damage, armor_penetration, knockback, attack_range)
- Special effects (mounted_bonus, structure_bonus, area_damage, poison_damage, magic_damage, etc.)

### Technologies (25)

Organized by research building & category:

| Category | Count | Examples |
|----------|-------|----------|
| Barracks Training | 8 | Mongoose Reflexes, Sharpshooter, Harsh Training, Blacksmith Guild |
| Siege Engineering | 5 | Conveyor Method, Big Rocks, Manufacturing Production |
| Economy | 5 | Hygiene, Dietetics, Urban Planning I-II |
| Cavalry | 2 | Horse Tactics, Heavy Cavalry |
| Undead-Specific | 2 | Corpse Reanimation, Plague Mastery |
| Magic Spells | 3 | Astral Ray, Mass Healing, Meteor |

Each tech includes:
- Building required, cost (60-160 gold), research_time (60-180s)
- Doctrinal effects (numeric modifiers on unit/building/spell stats)
- Unlock chains (e.g., Urban Planning I unlocks Urban Planning II)

### Doctrines (12)

**Lords Troops** (3): Combined Arms, Heavy Cavalry, Siege Mastery
**Rebels** (2): Mass Assault, Guerrilla Tactics
**Royal Army** (2): Defensive Formations, Discipline
**Sarranga** (2): Elemental Mastery, Mystical Binding
**Undead** (2): Corpse Mastery, Plague Spreading
**Bugs** (2): Hive Coordination, Reproductive Surge

Each doctrine provides faction-specific bonuses:
- Unit cost modifiers, damage bonuses, morale effects
- Armor/speed bonuses, spawn rate increases, storage capacity boosts

## Usage in Mods

### Extending Units

Mods inherit vanilla unit stats and extend/override them:

```yaml
# In warfare-starwars pack
- id: sw_clone_trooper_b1
  display_name: Clone Trooper B1
  vanilla_mapping: vanilla_swordsman      # Inherit from Lords Troops Swordsman
  stats:
    hp: 50.0                              # Inherited
    damage: 9.0                           # Override to 9 (from 8)
    armor: 2.5                            # Override to 2.5 (from 2.0)
  faction_id: cis
```

### Creating Mod Factions

Mods reference vanilla faction archetypes:

```yaml
# In custom-mod pack
faction:
  id: my_custom_faction
  archetype: order                        # Use same archetype as Lords Troops
  doctrine: combined_arms
  roster:
    cheap_infantry: custom_militia        # Custom unit, not vanilla
    elite_infantry: vanilla_foot_knight   # Can mix vanilla references
```

### Technology Extension

Mods add technologies that require vanilla buildings:

```yaml
- id: mod_advanced_training
  display_name: Advanced Training
  building_required: barracks_tier_3      # Requires vanilla building
  unlocks_unit: custom_elite_warrior
```

## Integration Checklist

When creating a new mod:

- [ ] Define factions with reference to vanilla archetypes (order, defense, chaos, magic, swarm)
- [ ] Create custom units but map to vanilla unit_class archetypes (MilitiaLight, HeavyInfantry, etc.)
- [ ] Use `vanilla_mapping` field to inherit base stats from vanilla units
- [ ] Reference vanilla buildings in production rosters (barracks, stables, etc.)
- [ ] Add doctrines extending vanilla faction doctrines, not replacing them
- [ ] Document all new unit/faction/building IDs with wiki references
- [ ] Test load order: vanilla-dino should load before mod (load_order: 10)

## Metadata

- **Pack ID**: vanilla-dino
- **Version**: 1.0.0
- **Load Order**: 10 (loads before all mods)
- **Canonical**: true
- **Exportable**: true
- **Mods Extend This**: true
- **Authors**: DINOForge Team
- **Wiki Reference**: https://dino.fandom.com/wiki/Units

## Statistics

| Metric | Count |
|--------|-------|
| Total YAML Files | 23 |
| Unit Definitions | 77 |
| Factions | 6 |
| Buildings | 40+ |
| Weapons | 30+ |
| Technologies | 25 |
| Doctrines | 12 |
| **Total Entries** | **200+** |

## Notes

- All units include `vanilla_dino_name` and `wiki_reference` for canonical mapping
- Undead faction includes 8 reanimated variants of Lords Troops units (Swordsman, Axe Warrior, Spearman, Crossbowman, Foot Knight, Mounted Knight, etc.)
- Buildings organized by faction-specific production (barracks, stables) + shared economy (resource gathering, housing, defense)
- Town Hall tier system (I→II→III) unlocks progressively more advanced buildings and unit tiers
- All stats derived from wiki community guides and Fandom documentation compiled via web scraping (March 2026)

## Data Sources

- Primary: https://diplomacy-is-not-an-option.fandom.com/wiki/
- Community Guides: TechRaptor, NamuWiki, Steam Community, GamePretty, Gameplay.tips
- Updates: Undead faction overhaul (November 2025)
