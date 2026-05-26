# DINOForge Mod Coverage Analysis
**Date**: 2026-03-13
**Scope**: Star Wars & Aviation subsystems vs vanilla-dino baseline

---

## Executive Summary

| Pack | Status | Coverage | Blocker | Priority |
|------|--------|----------|---------|----------|
| **Star Wars** | 65% complete | 26/77 units (33.8%), 20/40 buildings, 6 doctrines | No aerial units defined | HIGH |
| **Aviation** | 40% complete | 2 systems, 2 components, 0 gameplay examples | No targeting system | HIGH |

Both mods need immediate fixes before shipping. Focus on:
1. **Star Wars**: Add aerial unit definitions (V-19 Torrent, Tri-fighter)
2. **Aviation**: Implement AerialTargetingSystem + add anti-air buildings

---

## Star Wars Mod Deep Dive

### Unit Coverage

**Total Units**: 26 (13 Republic, 13 CIS)
**Vanilla Available**: 77 units across 21 unit_classes
**Coverage**: 33.8%

**Tier Distribution** (Balanced):
- Tier 1: 9 units
- Tier 2: 9 units
- Tier 3: 9 units

### Unit Class Utilization

**Used (13 of 21)**:
- MilitiaLight (Clone Militia, B1 Droids)
- CoreLineInfantry (Clone Troopers, Battle Droids)
- HeavyInfantry (Heavy Clones, Super Battle Droids)
- Skirmisher (ARF Troopers, Droideka)
- FastVehicle (Speeder Pilots, AAT Pilots)
- MainBattleVehicle (AT-TE, AAT)
- EliteLineInfantry (Arc Trooper, Commando, Grievous)
- StaticMG (Clone Sniper, Droid Sniper)
- StaticAT (Clone AT, Droid AT)
- SupportEngineer (Clone Engineer, Droid Engineer)
- Recon (Scout Trooper, Battle Droid Scout)
- HeroCommander (Jedi Knight, General Grievous)
- ShieldedElite (Clone Commander, Battle Droid Commander)

**Unused (8 of 21)** - CRITICAL GAPS:
| Class | Purpose | Star Wars Equivalent | Impact |
|-------|---------|---------------------|--------|
| **AirstrikeProxy** | Air support strikes | V-19 Torrent, Tri-fighter | **BLOCKS Aviation integration** |
| **AntiArmor** | Specialized armor pen | AT-RT, Droid Tank Killer | Anti-tank support missing |
| **Artillery** | Siege/ranged support | Clone Artillery, DSD1 Cannon | Limited ranged options |
| **HeavySiege** | Fortress breakers | None in Clone Wars | Unlikely needed |
| **ShockMelee** | Charge-focused melee | Jedi Charge Knights | Charge tactics not emphasized |
| **WalkerHeavy** | Specialized walkers | AT-TE (currently MainBattleVehicle) | Walker tactics limited |
| **SwarmFodder** | Expendable cannon fodder | B1 swarms (currently CoreLineInfantry) | CIS swarm mechanics weak |
| **StaticArtillery** | Stationary artillery | Cannon emplacements | Fortified position support missing |

### Faction Architecture

#### Galactic Republic
- **Archetype**: `order` (disciplined, elite-centric)
- **Economy**:
  - Gather: 1.0x (baseline)
  - Upkeep: 1.1x (expensive to maintain)
  - Research: 1.1x (tech advantage)
  - Build: 1.0x (standard speed)
- **Army**:
  - Morale: disciplined (high baseline)
  - Unit Cap: 1.0x (standard)
  - Elite Cost: 0.9x (cheaper elites, encourages quality)
- **Faction Roster**: 11/11 slots filled

**Assessment**: Strong archetype alignment with historical Clone Wars strategy (discipline, elite training, technological edge). Upkeep penalty correctly reflects expensive clone maintenance.

#### Confederacy of Independent Systems
- **Archetype**: `industrial_swarm` (mass production, expendable)
- **Economy**:
  - Gather: 0.9x (slower collection)
  - Upkeep: 0.6x (cheap droid maintenance)
  - Research: 0.9x (slower tech)
  - Build: 1.4x (rapid droid manufacturing)
- **Army**:
  - Morale: mechanical (unbreakable, no morale hits)
  - Unit Cap: 1.6x (massive armies)
  - Elite Cost: 1.3x (expensive superbattledroids, discourages overspecialization)
- **Faction Roster**: 11/11 slots filled

**Assessment**: Excellent archetype contrast with Republic. Mechanized morale system correctly reflects droid nature (emotionless, unbreakable). Build speed bonus supports swarm doctrine.

### Building Coverage

**Total Buildings**: 20 (10 per faction)

| Type | Count | Details |
|------|-------|---------|
| Command Centers | 2 | 1 Republic, 1 CIS |
| Barracks (T1-T3) | 6 | 3 per faction |
| Defense Towers | 4 | Shields/towers per faction |
| Resource Facilities | 4 | Factory/refinery per faction |
| Research Labs | 2 | 1 per faction |
| **Total** | **20** | Sufficient for themed conversion |

**Gap Analysis**:
- ❌ No anti-air defense buildings (blocks aerial unit counters)
- ❌ No wall segment variants (limited fortification depth)
- ❌ No unique command/hero buildings per faction (flavor opportunity)
- ✅ Barracks T1-T3 ladder supports unit progression
- ✅ Faction-specific economy buildings (tibanna refinery, clone facility)

### Doctrine Coverage

**Total**: 6 (3 per faction) — **EXCELLENT diversity**

| Faction | Doctrine | Effects | Strategy |
|---------|----------|---------|----------|
| **Republic** | Elite Discipline | Acc +15%, morale +20%, fire_rate +5%, dmg +5% | Bonus unit quality |
| **Republic** | Jedi Leadership | Morale +30%, speed +15%, dmg +10%, cost +10% | Mobility + aggression |
| **Republic** | Defensive Formation | Armor +20%, range +10%, HP +10%, speed -10% | Turtle tactics |
| **CIS** | Mechanized Attrition | Spawn +50%, cost -30%, HP -15%, dmg -10% | Mass over quality |
| **CIS** | Rolling Thunder | Armor +20%, dmg +15%, speed -10%, cost +15% | Heavy assault |
| **CIS** | Swarm Protocol | Fire_rate +30%, speed +15%, acc -20%, spawn +30% | Speed swarms, lower accuracy |

**Assessment**: Each doctrine enforces a distinct playstyle with trade-offs. Republic focuses on elite quality, CIS on mass production. Excellent archetype support.

### Weapon Coverage

**Total**: 18 (9 Republic, 9 CIS)

**Weapon Categories**:
- Rifles (11): DC-15A, DC-15S, DC-15X, E-5, E-5S, Wrist Blaster, STAP Twin, Z-6 Rotary, Droideka Twin
- Cannons (3): AT-TE Mass Driver, AAT Cannon, DSD1 Laser
- Melee (2): Lightsaber, Electrostaff
- Missiles (1): DC-17m Anti-Armor
- Pistols (1): DC-17 Hand

**Gap Analysis**:
- ❌ No dedicated anti-air weapons (missile/cannon variants)
- ❌ No AoE/splash weapons beyond cannons
- ✅ Each unit has appropriate weapon
- ✅ Damage types reflect Clone Wars theme (energy, explosive)

---

## Aviation Subsystem Deep Dive

### Architecture

**Framework**: ECS-based aerial mechanics integrated into Runtime domain

### Components

#### AerialUnitComponent
```csharp
public struct AerialUnitComponent : IComponentData
{
    public float CruiseAltitude;      // Target altitude (world units)
    public float AscendSpeed;        // Climb rate (units/sec)
    public float DescendSpeed;       // Descent rate (units/sec)
    public bool IsAttacking;         // Descend for attack flag
}
```

**Usage**: Attached via AerialUnitMapper when unit has "Aerial" behavior_tag

#### AntiAirComponent
```csharp
public struct AntiAirComponent : IComponentData
{
    public float AntiAirRange;           // Range vs aerial targets
    public float AntiAirDamageBonus;    // Damage multiplier (e.g., 1.5x)
}
```

**Usage**: Attached to units/buildings with "AntiAir" tag. No buildings in Star Wars currently tagged.

### Systems

#### AerialMovementSystem (SimulationSystemGroup)
**Responsibility**: Altitude maintenance, movement, attack descent

**Logic**:
1. Read AerialUnitComponent, Translation, Velocity
2. If attacking: Descend to ground level
3. If not attacking: Maintain CruiseAltitude
4. Move in straight line (bypasses NavMesh pathfinding)

**Limitation**: **No targeting logic** — system doesn't select targets or decide when to attack

#### AerialSpawnSystem (SimulationSystemGroup)
**Responsibility**: Initialize spawned aerial units to cruise altitude

**Logic**:
1. Detect newly spawned unit with AerialUnitComponent
2. Set Translation.y = CruiseAltitude
3. One-time initialization

### Behavior Tag Integration

**Framework Tags**:
- `"Aerial"` → AerialUnitComponent attached
- `"AntiAir"` → AntiAirComponent attached

**Star Wars Pack Status**: **Neither tag used**. Zero aerial units defined.

### Asset Pipeline

**Status**: Placeholder

**Integration Point**: Units can optionally define:
```yaml
aerial:
  cruise_altitude: 15.0
  ascend_speed: 5.0
  descend_speed: 3.0
  anti_air: false
```

**Default Values**: If not specified, CruiseAltitude=15, AscendSpeed=5, DescendSpeed=3

### Gameplay Gaps

| Gap | Severity | Impact | Fix |
|-----|----------|--------|-----|
| **No Targeting System** | CRITICAL | Aerial units move but don't attack | Implement AerialTargetingSystem |
| **No Anti-Air Buildings** | CRITICAL | Can't counter aerial units | Add 2 buildings/faction with AntiAirComponent |
| **No Aerial Units** | CRITICAL | Subsystem untested in practice | Add V-19 Torrent, Tri-fighter to SW pack |
| **No Aerial Weapons** | HIGH | Generic weapons used, no air-to-ground specialization | Add anti-air/aerial variants |
| **No Collision Detection** | MEDIUM | Straight-line movement passes through obstacles | Implement aerial pathfinding layer |
| **No Takeoff/Landing** | LOW | Units teleport to/from altitude | Add animation hooks (nice-to-have) |

---

## Integration Assessment

### Star Wars Readiness

**Feature Completeness**: **65%**

**What Works**:
- ✅ Faction architectures (Republic order, CIS swarm)
- ✅ Unit tier progression (T1-T3 balanced)
- ✅ Doctrine diversity (6 distinct playstyles)
- ✅ Weapon set (18 thematic weapons)
- ✅ Building production chains
- ✅ Roster mappings (11/11 slots filled per faction)

**What's Missing**:
- ❌ Aerial units (V-19, Tri-fighter) — **BLOCKS Aviation demo**
- ❌ Anti-air defenses — Can't counter aerial when added
- ❌ Artillery units — Limited siege options
- ❌ AntiArmor specialists — Generic heavy units
- ❌ SwarmFodder — CIS swarm mechanics underdeveloped

**Launch Blocking**:
- ❌ **NO AERIAL UNITS** = Aviation subsystem is untested, dead code

### Aviation Readiness

**Feature Completeness**: **40%**

**What Works**:
- ✅ Altitude system (ECS components + movement)
- ✅ Spawn initialization
- ✅ Integration hooks (AerialUnitMapper)
- ✅ Tag-based attachment

**What's Missing**:
- ❌ **NO TARGETING SYSTEM** — can't attack
- ❌ **NO EXAMPLE UNITS** — untested
- ❌ **NO ANTI-AIR BUILDINGS** — can't be countered
- ❌ **NO AERIAL WEAPONS** — generic weapons feel weak
- ❌ Collision/pathfinding

**Launch Blocking**:
- ❌ **NO GAMEPLAY LOOP** = Aerial units exist but can't fight

---

## Critical Path to Launch

### Phase 1: Immediate (Blocking)

**1. Add Aerial Units to Star Wars Pack** (2 units)
```yaml
# Republic
- id: rep_v19_torrent
  display_name: V-19 Torrent Starfighter
  unit_class: AirstrikeProxy
  behavior_tags: [Aerial]
  aerial:
    cruise_altitude: 20.0
    ascend_speed: 8.0
    descend_speed: 5.0

# CIS
- id: cis_tri_fighter
  display_name: Tri-Fighter
  unit_class: AirstrikeProxy
  behavior_tags: [Aerial]
```

**Impact**: Aviation subsystem now has a playable test case

**2. Implement AerialTargetingSystem** (Runtime)
```
Hook: AerialMovementSystem.OnUpdate
Logic:
  - Scan for ground targets within aerial weapon range
  - Pick nearest/weakest target
  - If target exists: Set IsAttacking = true → Descend
  - Apply damage with aerial weapon stats
  - Ascend after attack cooldown
```

**Impact**: Aerial units can now engage enemies

**3. Add Anti-Air Buildings** (2 buildings per faction in Star Wars)
```yaml
# Republic - Skyshield Generator
- id: rep_skyshield_generator
  unit_cap: 1
  defense_tags: [AntiAir]
  anti_air:
    range: 40.0
    damage_bonus: 1.5

# CIS - Vulture Droid Nest
- id: cis_vulture_nest
  unit_cap: 1
  defense_tags: [AntiAir]
```

**Impact**: Players can counter aerial units with structures

### Phase 2: High Priority (Pre-Launch)

**4. Expand Unit Class Usage** (4-6 additional units)
- Add Artillery units (Clone Mortar, Droid Artillery)
- Add AntiArmor specialists (Clone AT-RT, Droid Tank Killer)
- Expand Shock tactics (Commando Corps, Battle Droid Elite)

**5. Add Anti-Air Weapons** (2-3 weapon variants)
- Missile launcher with AntiAirBonus
- Cannon with area splash damage

**6. Expand Building Variety**
- Unique faction research labs (Tipoca City, Geonosis Laboratory)
- Resource specialization (Food = Clone Nutrition, Resources = Metal Refineries)

### Phase 3: Quality (Post-Launch)

**7. Aviation Asset Pipeline**
- Source V-19 Torrent, Tri-fighter models (Sketchfab)
- Texture with Star Wars colors
- Import via Addressables

**8. Aerial Collision & Pathfinding**
- Add obstacle avoidance layer
- Implement takeoff/landing animation hooks

**9. Anti-Air Targeting**
- UI indicators for aerial threats
- Audio cues for aerial detection

---

## Success Metrics

| Metric | Target | Current | Gap |
|--------|--------|---------|-----|
| Unit Coverage | 80%+ | 33.8% | +46% (add 30+ units) |
| Unit Classes Used | 18/21 | 13/21 | +5 classes |
| Aerial Units | 4+ | 0 | +4 |
| Anti-Air Buildings | 2+ per faction | 0 | +4 |
| Targeting Systems | 1 | 0 | +1 |
| Doctrine Count | 6+ | 6 | ✓ Met |
| Building Count | 25+ | 20 | +5 |
| Weapon Coverage | 25+ | 18 | +7 |

---

## Recommendations

### Short-term (Next Sprint)
1. ✅ Vanilla-dino pack COMPLETE
2. 🟡 Add V-19 Torrent + Tri-fighter to Star Wars (critical for Aviation)
3. 🟡 Implement AerialTargetingSystem in Runtime
4. 🟡 Add 4 anti-air buildings (2 per faction)

### Medium-term (Sprint 2)
5. Add 6 additional Star Wars units (fill unused classes)
6. Add anti-air weapon variants
7. Expand building variety (faction-unique structures)

### Long-term (Sprint 3+)
8. Aviation asset pipeline (models, textures)
9. Aerial pathfinding & collision
10. Unit class expansion (SwarmFodder, Artillery variants)

---

## Data Sources

- Star Wars Pack: `packs/warfare-starwars/`
- Aviation Runtime: `src/Runtime/Aviation/`
- Vanilla Baseline: `packs/vanilla-dino/`
- Analysis Date: 2026-03-13
- Agent Analysis: a1f1111ce2e2d091d

