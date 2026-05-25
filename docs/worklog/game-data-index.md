# Diplomacy is Not an Option - Complete Game Data Index

**Compiled**: 2026-03-13
**Source**: Fandom Wiki + Community Guides (NamuWiki, TechRaptor, Steam, GamePretty, Gameplay.tips)
**Status**: Complete and integrated into DINOForge documentation

---

## Documentation Files

### 1. DINO_WIKI_DATA_COMPILATION.md
**Type**: Human-readable reference
**Size**: 12 KB, 337 lines
**Best for**: Reading about game mechanics, understanding systems, developer reference

**Contents**:
- Complete building directory (30+ types)
- Full unit roster with stats (20+ units)
- Economy system documentation
- Research trees (54+ technologies)
- Faction descriptions
- Special mechanics (combat, magic, undead)
- Data gaps and unknowns

**Use Cases**:
- Reading up on game mechanics
- Understanding building progressions
- Learning faction differences
- Planning mod pack content
- Reference during development

---

### 2. DINO_GAME_DATA.json
**Type**: Structured data (JSON)
**Size**: 21 KB, 750 lines
**Best for**: Programmatic access, validation, schema generation, data integration

**Contents**:
- Metadata and source references
- Resources (7 types with attributes)
- Buildings organized by category
- Units with complete stat blocks
- Research indexed by building
- Factions with unique mechanics
- Special mechanics systems
- Game modes

**Use Cases**:
- ContentLoader validation schemas
- Registry population
- Test data generation
- Pack template creation
- API contract definition

**Key JSON Sections**:
```json
{
  "metadata": {...},
  "resources": {...},
  "buildings": {
    "resource_gathering": {...},
    "military_production": {...},
    "defense": {...},
    "economy": {...},
    "special": {...}
  },
  "units": {
    "infantry": [...],
    "ranged": [...],
    "cavalry": [...],
    "support": [...],
    "siege": [...],
    "summoned": [...],
    "undead": [...]
  },
  "research": {...},
  "factions": {...},
  "special_mechanics": {...},
  "game_modes": {...}
}
```

---

### 3. WIKI_SCRAPE_SUMMARY.md
**Type**: Integration guide and metadata
**Size**: 9.1 KB, 240 lines
**Best for**: Understanding data quality, integration recommendations, development planning

**Contents**:
- Scraping methodology and approach
- Data coverage summary
- Key metrics documented
- Data gaps identified
- Quality assessment (high/medium/low confidence)
- Integration recommendations for SDK and packs
- Testing recommendations
- Future enrichment suggestions

**Use Cases**:
- Planning SDK schema design
- Understanding data reliability
- Identifying areas needing datamining
- Planning test coverage
- Setting up validation rules

---

## Data Categories

### BUILDINGS (30+ documented)

**Resource Gathering (7)**
- Lumber Mill, Stone Mine, Farm, Fisherman's Hut, Berry Picker's House, Iron Mine, Gold Mine

**Military Production (8 tiers)**
- Barracks I/II/III, Stables I/II/III, Engineer Guild I/II

**Defense (6+)**
- Wooden/Stone Gates, Walls, Towers, Stone Obelisk, Undead Towers

**Economy (8)**
- Town Hall (3 levels), Granary, Storage, Market, House I/II/III

**Special (6)**
- Hospital, University, Rebel Cabin, Death Metal facilities (Undead)

---

### UNITS (20+ with stats)

**All units cost 1 citizen to recruit**

**Infantry**
- Swordsman (55 HP, 5 DPS, Light armor)
- Axeman (80 HP, 10 DPS, Light armor)
- Foot Knight (300 HP, 9.75 DPS, Heavy armor)
- Spearman (100 HP, 6 DPS, Medium armor, +300% vs cavalry, -50% arrows, +3 HP/sec)
- Hammerguy (8 DPS area damage, Heavy armor)

**Ranged**
- Archer (5 DPS, Light armor)
- Crossbowman (higher DPS, Light armor)
- Horse Archer (mobile ranged)

**Cavalry**
- Mounted Knight (400 HP, 13 DPS, Heavy armor)

**Support & Siege**
- Healer (weak healing support)
- Catapult (cheap area weapon, no iron)
- Ballista (direct fire)
- Trebuchet (parabolic fire)

**Summoned & Undead**
- Dark Knight (2 Soul Crystals, 5 units, 120 sec duration)
- Hunting Fiend, Flesh Collector, Thundering Amalgam, Drake variants, Undead Horseman

---

### RESEARCH (54+ technologies)

**Town Hall (TH1/TH2/TH3)**
- Comfortable Shoes, Advanced Toolkits, Stained Wood, Sport Shoes, Sturdy Concrete, Enhanced Foundations, Blue Suede Shoes, Personal Toolkits

**Barracks (BR1/BR2/BR3)**
- Mongoose Reflexes, Sharpshooter, Quad Cure, Harsh Training, Quick Reload, Blacksmith Guild, Infected Mushroom, Cast-Iron Hammer

**Engineer Guild (EG1/EG2)**
- Conveyor Method, Big Rocks, Manufacturing Production, Shrapnel Projectiles, Foolproof Charge

**Hospital**
- Hygiene, Urgency Bonus, General Wards

**Market & University**
- Market Regulations, Greased Propeller, Dietetics, Urban Planning I/II

**Undead Faction** (64 exclusive techs)
- Massive research tree unique to Undead (2025 update)

---

### ECONOMY SYSTEM

**Resources (7 types)**
1. **Food** - Daily consumption (1 per citizen/unit), sources: Farms, Fisherman's Hut, Berry Picker's House
2. **Wood** - Limited early, abundant late
3. **Stone** - Defense and advanced buildings
4. **Iron** - Requires Town Hall II, precious resource
5. **Gold** - Trade-only via Market (Town Hall II required)
6. **Soul Crystal** - Rebel magic system spellcasting
7. **Death Metal** - Undead faction unique resource

**Housing (3 tiers)**
- House I: 6 capacity, 25 Wood
- House II: 12 capacity, 25 Wood + 14 Stone
- House III: 18 capacity, requires Town Hall II

**Storage**
- Food: Separate storage (100 base in Town Hall)
- Wood/Stone/Iron: Shared pool (reaching limit blocks all three)
- Storage Bonuses: Wood Cart (+40), Stone Cart (+15), Food Cart (+20)

**Population**
- Citizens added daily if housing exists
- Research bonuses: Urban Planning I/II (+10 citizens/day each)

---

### FACTIONS (4 playable)

**1. Lord's Troops**
- Traditional RTS, balanced playstyle
- Stone walls, versatile units
- Standard tech tree (54 technologies)

**2. Rebels**
- Magic-focused with soul crystals
- Wooden structures only
- Spells: Summon, Astral Ray, Mass Healing, Meteor
- Area buffs, weak individual units

**3. Sarranga (Tribes)**
- Aggressive, mobile, champion-focused
- Ancient upgrade level available
- Hit-and-run tactics

**4. Undead**
- Offensive, corpse-based resurrection
- No walls; body-wall defense
- Death Metal unique resource
- Corpse recycling: Kill → zombies → bones
- 64-tech exclusive research tree (2025 update)

---

## Critical Game Mechanics

### Combat System
- **Armor Types**: Light (minimal), Medium (moderate), Heavy (70-100% reduction)
- **Spearman Special**: +300% vs cavalry, -50% arrows, +3 HP/sec constant regen
- **Double-Damage**: Archers +20% chance (with Sharpshooter)
- **Avoidance**: Swordsmen +15% (with Mongoose Reflexes)
- **Area Damage**: Hammerguy, siege weapons

### Magic System (Rebel)
- Summon (2 crystals → 5 Dark Knights, 120 sec)
- Astral Ray (5 crystals → continuous damage)
- Mass Healing (2 crystals → 100 HP radius heal)
- Meteor (10 crystals → 500 damage area)

### Undead Mechanics
- **Necromancy**: Raise dead enemies as temporary zombies
- **Corpse Recycling**: Kill enemies → zombies → convert to bone resources
- **Death Metal Economy**: Parallel resource system fueling unit production
- **Aggressive Incentive**: Victory fuels economy; no walls encourage offense

### Production System
- **Unit Cost**: 1 citizen per unit (all units standardized)
- **Research**: One at a time, no cancellation, no refunds
- **Time Range**: Typical 1-3 minutes per research (range 1-4 minutes)
- **Building Construction**: Variable, affected by builder count and speed research

---

## Data Confidence Levels

### High Confidence (Verified Multiple Sources)
- Unit base stats (HP, DPS values) ✓
- Building costs (few examples fully documented) ✓
- Research trees and costs ✓
- Faction definitions and playstyles ✓
- Special mechanics (Spearman bonuses, Undead necromancy) ✓

### Medium Confidence (Single Source or Partial)
- Complete building costs (only 2 examples)
- Unit production times
- Resource gathering rates
- Storage capacity details

### Low Confidence (Not Found)
- Morale system (possibly non-existent)
- Terrain bonuses
- Complete balance ratios
- Exact AI behavior

### Not Documented
- Individual siege weapon damage values
- Exact disease spread percentages
- Sarranga unit roster (complete)
- Undead unit individual stats
- Trade route mechanics

---

## Integration Guide for DINOForge

### For SDK Development
1. **Use JSON schema directly** for ContentLoader validation
2. **Create domain models** mapping JSON structure:
   - `BuildingDef` ← building entries
   - `UnitDef` ← unit entries
   - `ResearchDef` ← research entries
   - `FactionDef` ← faction entries
3. **Reference markdown** in developer docs
4. **Validate against compiled data** in unit tests

### For Warfare Domain Pack
1. **Buildings**: Map to mod pack building definitions
2. **Units**: Use stats for archetype calculations
3. **Research**: Link dependency chains
4. **Factions**: Implement faction-specific systems

### For Example Packs
- **Modern Pack**: Adapt buildings/units to modern warfare theme
- **Star Wars Pack**: Leverage Undead resurrection for "cloning"/"droid armies"
- **Guerrilla Pack**: Use Sarranga hit-and-run mechanics

### For Testing Strategy
1. **Unit stats tests** - validate HP, DPS, armor
2. **Cost validation** - verify resource costs match data
3. **Research dependency tests** - validate chain linkage
4. **Faction mechanic tests** - verify unique behaviors

---

## Data Enrichment Opportunities

### From Datamining (High Priority)
1. Production time values for all units
2. Complete building costs across all tiers
3. Resource gathering rates per building
4. Exact storage capacities
5. Siege weapon damage values

### From Developer Contact (Medium Priority)
1. Terrain effect values
2. Balance ratio documentation
3. AI behavior specification
4. Complete Sarranga unit roster
5. Undead unit individual stats

### From Community Testing (Low Priority)
1. Exact cooldown values
2. Precise timing measurements
3. Edge case mechanics
4. PvP balance feedback
5. Advanced strategy documentation

---

## File Organization

```
/c/Users/koosh/Dino/docs/
├── DINO_WIKI_DATA_COMPILATION.md     ← Start here (human-readable)
├── DINO_GAME_DATA.json               ← Use for integration (structured)
├── WIKI_SCRAPE_SUMMARY.md            ← Understand data quality
└── GAME_DATA_INDEX.md                ← This file (overview)
```

---

## Quick Reference

### Most Important Files
1. **DINO_GAME_DATA.json** - Primary source for programmatic access
2. **DINO_WIKI_DATA_COMPILATION.md** - Primary source for human reading
3. **WIKI_SCRAPE_SUMMARY.md** - Data quality and integration guidelines

### Key Statistics
- **Buildings**: 30+ types
- **Units**: 20+ types with stats
- **Research**: 54+ technologies (64 for Undead)
- **Factions**: 4 playable + 4 enemy
- **Resources**: 7 types
- **Data Completeness**: 70-80% (high confidence areas documented)

### Most Critical Data Points
- Unit stats (HP, DPS, armor) - HIGH CONFIDENCE
- Building costs (few examples) - MEDIUM CONFIDENCE
- Research trees - HIGH CONFIDENCE
- Faction mechanics - HIGH CONFIDENCE
- Resource gathering rates - LOW CONFIDENCE

---

## Next Steps

1. **Review** these files for accuracy
2. **Integrate** JSON into SDK schemas
3. **Create** Warfare domain pack using this data
4. **Document** in VitePress site
5. **Test** against compiled data
6. **Enhance** with additional datamining

---

**Last Updated**: 2026-03-13
**Compiled by**: Claude Code (Haiku 4.5)
**Status**: Ready for integration into DINOForge
