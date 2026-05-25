# DINO Wiki Data Scrape - Summary & Integration Guide

**Date**: 2026-03-13
**Status**: COMPLETE

## Overview

Comprehensive game data compiled from the official Diplomacy is Not an Option Fandom Wiki and community sources. Due to Fandom 403 blocking, data was gathered via:
- TechRaptor guides (detailed research trees)
- NamuWiki mirror documentation
- Steam Community guides
- GamePretty, Gameplay.tips guides
- Recent news on Undead faction update

## Output Files

### 1. DINO_WIKI_DATA_COMPILATION.md (12 KB)
Human-readable markdown documentation with:
- Complete building directory (resource, military, defense, economy, special)
- Full unit roster with stats (HP, DPS, armor, costs)
- Economy system details (resources, population, housing, trading)
- Complete technology trees by building (54 techs standard, 64 Undead)
- Faction descriptions and mechanics
- Special mechanics (combat, magic, undead corpse recycling)
- Data gaps and unknowns documented

### 2. DINO_GAME_DATA.json (21 KB)
Structured JSON format with:
- Metadata and source references
- Resource definitions (types, storage, sources)
- Buildings organized by category
- Units with full stat blocks
- Research trees indexed by building type
- Faction definitions with unique mechanics
- Special mechanics organized by system
- Game modes and difficulty settings

## Data Coverage

### Buildings: 30+ Types Documented
- Resource gathering (7 types)
- Military production (8 tiers)
- Defense structures (6+ types)
- Economy buildings (8 types)
- Special buildings (Hospitals, Universities, Cabins, Undead facilities)

### Units: 20+ Types with Stats
- Infantry (5 types) - HP, DPS, armor, special effects
- Ranged (3 types) - damage, range, role
- Cavalry (1 type) - mounted knight
- Support (1 type) - healer
- Siege (3 types) - catapult, ballista, trebuchet
- Summoned (Dark Knights)
- Undead (6 exclusive types)

### Research: 54+ Technologies
- Town Hall tiers (10 techs)
- Barracks tiers (8 techs)
- Engineer Guild (5 techs)
- Hospital (3 techs)
- Market/University (5 techs)
- Plus 64 exclusive Undead techs

### Economy System
- 7 resource types (Food, Wood, Stone, Iron, Gold, Soul Crystal, Death Metal)
- Housing system (3 tiers: 6/12/18 capacity)
- Storage mechanics (100 food base, shared Wood/Stone/Iron pool)
- Trading system (Market-based gold acquisition)
- Population mechanics (daily growth, worker assignment)

### Factions: 4 Playable + 4 Enemy
- **Playable**: Lords Troops, Rebels, Sarranga, Undead
- **Enemy**: Royal Army, Rebels (enemy), Warlike Tribe, Undead (enemy)
- Each with unique playstyle, defense strategy, units, and tech tree

## Key Metrics Compiled

### Costs (Example)
- Keep upgrade L2: 150 Wood + 100 Stone
- House I: 25 Wood
- House II upgrade: 14 Stone
- Research range: Free to 6 Gold
- Unit production: 1 citizen each

### Stats (Example)
- Swordsman: 55 HP, 5 DPS
- Spearman: 100 HP, 6 DPS, +300% vs cavalry, -50% arrows, +3 HP/sec regen
- Foot Knight: 300 HP, 9.75 DPS
- Mounted Knight: 400 HP, 13 DPS
- Archer double-damage: 20% chance
- Hammerguy area damage with -15% move penalty

### Timings (Example)
- Research: 1:00 to 4:00 (typical 1-3 minutes)
- Food consumption: 1 per citizen per day
- Fisherman Hut: 3x slower than berry picker
- Zombie duration: 120 seconds (Dark Knights)
- Cooldowns: 300 seconds (Infected Mushroom)

## Data Gaps Identified

### Missing Detailed Stats
1. Complete building construction costs (only a few known)
2. Unit production times (not documented)
3. Resource gathering rates per building
4. Exact storage capacities beyond shared pool
5. Siege weapon damage values
6. Disease spread mechanics (exact percentages)

### Incomplete Mechanics
1. Trade route detailed mechanics
2. Morale system (no morale mechanic found in sources)
3. Terrain bonuses and effects
4. Neutral structure types and locations
5. Environmental weather effects
6. Multiplayer mechanics (if any exist)
7. Sarranga detailed unit roster
8. Complete Undead unit stats

### Faction-Specific Unknowns
- Sarranga building variants beyond "Ancient" upgrade tier
- Exact Sarranga unit roster and stats
- Undead unit costs and production mechanics
- Sarranga vs other faction balance differences

## Integration Recommendations

### For DINOForge SDK
1. **Use JSON directly** in ContentLoader validation schemas
2. **Reference markdown** for human-readable documentation
3. **Create domain models** mapping to this structure:
   - BuildingRegistry entries
   - UnitRegistry entries
   - ResearchRegistry entries
   - FactionRegistry entries

### For Warfare Domain Pack
1. **Buildings**: Map game building types to mod pack building definitions
2. **Units**: Use unit stats for archetypes and balance calculations
3. **Research**: Link to research dependencies and unlocks
4. **Factions**: Implement faction-specific tweaks (Undead corpse system, Rebel magic)

### For Example Packs
1. **Modern Pack**: Could adapt existing buildings/units to modern warfare
2. **Star Wars Pack**: Could reuse Undead resurrection mechanic for "cloning" or "droid armies"
3. **Guerrilla Pack**: Could leverage Sarranga hit-and-run tactics

### For Testing
1. **Unit stat tests**: Validate HP, DPS, armor calculations
2. **Cost tests**: Verify resource costs match compiled data
3. **Research tests**: Validate dependency chains (Urban Planning I → II)
4. **Faction tests**: Verify faction-specific mechanics (Undead corpse recycling, Rebel magic)

## Data Quality Notes

### High Confidence (Verified Multiple Sources)
- Unit base stats (HP, DPS values)
- Building costs (few examples verified)
- Research trees and costs
- Faction definitions and playstyles
- Special mechanics (Spearman bonuses, Undead necromancy)

### Medium Confidence (Single Source or Partial)
- Complete building costs (only 2 examples fully documented)
- Unit production times
- Resource gathering rates
- Storage capacity details

### Low Confidence (Not Found in Sources)
- Morale system (possibly non-existent or hidden)
- Terrain bonuses
- Complete balance ratios
- Exact AI behavior patterns

## Future Enrichment

When official game stats become available (via datamining or developer release):
1. Add production time values
2. Complete all building costs
3. Add missing unit stats for Sarranga/Undead
4. Clarify morale system (if it exists)
5. Document terrain effects
6. Provide exact resource gathering rates

## Source References

**Primary Wiki Sources** (blocked by Fandom 403):
- Buildings page: https://diplomacy-is-not-an-option.fandom.com/wiki/Buildings
- Units page: https://diplomacy-is-not-an-option.fandom.com/wiki/Units
- Research page: https://diplomacy-is-not-an-option.fandom.com/wiki/Research

**Alternative Sources Used**:
- TechRaptor Tech Tree Guide: https://techraptor.net/gaming/guides/diplomacy-is-not-option-research-tech-tree-guide
- TechRaptor Beginners Guide: https://techraptor.net/gaming/guides/diplomacy-is-not-option-guide-for-beginners
- NamuWiki: https://en.namu.wiki/w/Diplomacy%20is%20Not%20an%20Option
- Steam Community Guide: https://steamcommunity.com/sharedfiles/filedetails/?id=2748204153
- NoobFeed Beginners Guide: https://www.noobfeed.com/articles/diplomacy-is-not-an-option-beginners-guide
- GamePretty Guide: https://gamepretty.com/diplomacy-is-not-an-option-beginners-guide-build-order-combat-units-tech-and-spells/
- Gameplay.tips Guide: https://gameplay.tips/guides/diplomacy-is-not-an-option-how-to-start-guide-build-order-combat-units-tech-and-spells.html
- COGconnected Undead Update: https://cogconnected.com/2025/11/diplomacy-is-not-an-option-overhauls-undead-faction-in-a-new-major-update/
- PCGamesN Undead Update: https://www.pcgamesn.com/diplomacy-is-not-an-option/undead-update

## Methodology Notes

### Web Scraping Approach
1. Direct Fandom access failed (403 Fandom blocking)
2. Pivoted to community guides and mirrors
3. Aggregated data from multiple sources
4. Cross-referenced for consistency
5. Noted conflicts/uncertainties explicitly

### Data Validation
- High-confidence values appear in 2+ sources
- Single-source values noted as such
- Obvious gaps documented clearly
- No values fabricated or guessed
- Conservative approach: documented unknowns rather than extrapolating

## File Locations

```
/c/Users/koosh/Dino/docs/
├── DINO_WIKI_DATA_COMPILATION.md    (Human-readable, 12 KB)
├── DINO_GAME_DATA.json              (Structured data, 21 KB)
└── WIKI_SCRAPE_SUMMARY.md           (This file)
```

## Next Steps for Integration

1. **Schema Creation**: Build JSON schemas for buildings, units, research, factions
2. **Registry Tests**: Validate against compiled data
3. **Pack Templates**: Create starter packs using this data
4. **Documentation**: Link from VitePress docs to compiled data
5. **Maintenance**: Update when new game patches/data becomes available

---

**Compiled by**: Claude Code (Haiku 4.5)
**Data Accuracy**: Cross-referenced multiple sources; gaps documented
**Recommendation**: Use as foundation; validate critical stats with in-game verification
