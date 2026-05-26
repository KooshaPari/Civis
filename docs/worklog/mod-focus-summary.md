# DINOForge Mod Development Focus — Spring 2026

## Current Status

### Vanilla-DINO Baseline ✅ COMPLETE
- **23 YAML files, 200+ game entities**
- 6 factions, 77 units, 40+ buildings, 30+ weapons, 25 technologies, 12 doctrines
- Serves as the mutable baseline all mods adjust to
- Ready for production use

### Packs in Focus

| Pack | Status | Files | Units | Buildings | Focus | Blocker |
|------|--------|-------|-------|-----------|-------|---------|
| **Star Wars** | 65% | 20+ | 26 | 20 | Republic vs CIS | ❌ No aerial units |
| **Aviation** | 40% | Runtime | 0 (untested) | 0 | Altitude systems | ❌ No targeting |
| **Guerrilla** | Archived | — | — | — | — | Deferred |
| **Modern** | Archived | — | — | — | — | Deferred |

---

## Star Wars Pack Analysis

### What's Working (65% complete)

#### Factions ✅
- **Republic**: Order archetype with elite discipline doctrine
  - Economy: Standard gathering, 1.1x upkeep (expensive maintenance), tech advantage
  - Army: Disciplined morale, 0.9x elite cost (encourages quality)
  - 13 units across T1-T3 balanced tiers

- **CIS**: Industrial swarm with mechanized morale
  - Economy: 0.6x upkeep (cheap droids), 1.4x build speed (rapid manufacturing)
  - Army: Mechanical morale (unbreakable), 1.6x unit cap (massive armies), 1.3x elite cost (discourages overspecialization)
  - 13 units with swarm mechanics

#### Unit Tiers ✅
- **T1** (9 units): Basic infantry/droids (Clone Militia, B1 Droids)
- **T2** (9 units): Specialized/vehicles (Heavy Clones, Super Battle Droids, ATEs)
- **T3** (9 units): Elite/commanders (Jedi, Grievous, Arc Troopers)
- Distribution: Perfectly balanced (9-9-9)

#### Doctrines ✅ (EXCELLENT)
1. **Elite Discipline** (Republic): +15% accuracy, +20% morale, +5% fire_rate/damage
2. **Jedi Leadership** (Republic): +30% morale, +15% speed, +10% damage (at +10% cost)
3. **Defensive Formation** (Republic): +20% armor, +10% range, +10% HP (speed -10% trade)
4. **Mechanized Attrition** (CIS): +50% spawn rate, -30% cost, -15% HP/-10% damage (swarm over quality)
5. **Rolling Thunder** (CIS): +20% armor, +15% damage, cost +15% (heavy assault)
6. **Swarm Protocol** (CIS): +30% fire_rate, +15% speed, +30% spawn (accuracy -20% trade)

Each enforces distinct playstyle. Republic = elite quality, CIS = mass attrition. ✅

#### Buildings ✅ (20 total, sufficient)
- **Command Centers** (2): Faction HQs
- **Barracks T1-T3** (6): Unit progression ladder
- **Defense** (4): Shields/towers per faction
- **Resource** (4): Tibanna refinery, clone facility
- **Labs** (2): Research per faction

#### Weapons ✅ (18 total, appropriate)
- Rifles (11): DC-15A, E-5, etc.
- Cannons (3): AT-TE, AAT, DSD1
- Melee (2): Lightsaber, Electrostaff
- Missiles (1): Anti-armor
- Pistols (1): DC-17

#### Unit Class Coverage ✅ (13/21 used)
Used: MilitiaLight, CoreLineInfantry, HeavyInfantry, Skirmisher, FastVehicle, MainBattleVehicle, EliteLineInfantry, StaticMG, StaticAT, SupportEngineer, Recon, HeroCommander, ShieldedElite

### Critical Gaps (Blocking Launch)

#### ❌ NO AERIAL UNITS
- **Impact**: Aviation subsystem is untested, dead code
- **Missing**: V-19 Torrent (Republic), Tri-fighter (CIS)
- **Fix Required**: Add 2 units with AirstrikeProxy class + "Aerial" behavior_tag
- **Severity**: CRITICAL

#### ❌ NO ANTI-AIR BUILDINGS
- **Impact**: Aerial units can't be countered once added
- **Missing**: Skyshield generator, Vulture nest (defensive structures)
- **Fix Required**: Add 2 buildings/faction with AntiAirComponent
- **Severity**: CRITICAL

### High-Priority Gaps

#### 🟡 Missing Unit Classes (8/21 unused)
- **AirstrikeProxy** — Air support strikes (CRITICAL for Aviation)
- **Artillery** — Siege/ranged support (missing siege options)
- **AntiArmor** — Specialized armor pen (generalist heavy units only)
- **WalkerHeavy** — Specialized walkers (AT-TE is MainBattleVehicle)
- **SwarmFodder** — Expendable cannon fodder (CIS swarm underdeveloped)
- **HeavySiege, ShockMelee, StaticArtillery** — Unused niche classes

**Fix**: Add 4-6 units to expand variety:
- Clone Artillery, DSD1 Cannon (Artillery)
- Clone AT-RT (AntiArmor)
- B1 Fodder Swarm (SwarmFodder)
- Commando Charge Squad (ShockMelee)

#### 🟡 Missing Anti-Air Weapons
- **Current**: Generic weapons used
- **Missing**: Missile launcher with AntiAirBonus, anti-air cannon variants
- **Impact**: Anti-air relies only on component multiplier, not weapon specialization
- **Fix**: Add 2-3 anti-air weapon variants

#### 🟡 Building Variety
- **Gap**: Only 2 types per faction (barracks vs assembly)
- **Missing**: Unique command/hero buildings, faction-specific tech labs
- **Fix**: Add 4-5 faction-specific buildings (Tipoca City, Geonosis Lab)

---

## Aviation Subsystem Analysis

### What's Working (40% complete)

#### ECS Components ✅
1. **AerialUnitComponent**
   ```
   - CruiseAltitude (float)
   - AscendSpeed, DescendSpeed
   - IsAttacking (bool)
   ```
   Attached when unit has "Aerial" behavior_tag

2. **AntiAirComponent**
   ```
   - AntiAirRange (float)
   - AntiAirDamageBonus (float)
   ```
   Attached when unit/building has "AntiAir" tag

#### ECS Systems ✅
1. **AerialMovementSystem** (SimulationSystemGroup)
   - Maintains altitude
   - Straight-line movement (no NavMesh)
   - Attack descent/re-ascent logic

2. **AerialSpawnSystem** (SimulationSystemGroup)
   - Initializes spawned aerial units to cruise altitude

#### Integration Points ✅
- **AerialUnitMapper**: Reads unit YAML `aerial:` block
- **Tag System**: "Aerial" and "AntiAir" behavior_tags
- **Framework**: Works within existing ECS, no invasive changes

### Critical Gaps (Blocking Gameplay)

#### ❌ NO TARGETING SYSTEM
- **Current State**: AerialMovementSystem only handles altitude
- **Missing**: Logic to select targets, trigger attacks, apply AntiAirDamageBonus
- **Impact**: Aerial units move but can't fight
- **Fix Required**: Implement AerialTargetingSystem
  - Scan for ground targets within weapon range
  - Pick target (nearest/weakest)
  - Set IsAttacking=true → Descend
  - Apply damage with aerial weapon stats
  - Ascend after cooldown
- **Severity**: CRITICAL

#### ❌ NO GAMEPLAY EXAMPLES
- **Current**: 0 aerial units in Star Wars pack
- **Missing**: V-19 Torrent, Tri-fighter
- **Impact**: Subsystem untested in gameplay
- **Fix Required**: Add units with "Aerial" tag
- **Severity**: CRITICAL

#### ❌ NO ANTI-AIR BUILDINGS
- **Current**: 0 anti-air buildings in Star Wars
- **Missing**: Skyshield, Vulture nest
- **Impact**: Aerial units can't be countered
- **Fix Required**: Add buildings with AntiAirComponent
- **Severity**: CRITICAL (dependent on aerial units being added first)

### Medium-Priority Gaps

#### 🟡 NO AERIAL WEAPONS
- **Current**: Generic weapons used by aerial units
- **Missing**: Air-to-ground missile variants, anti-air weapons
- **Impact**: Aerial attacks feel generic, not specialized
- **Fix**: Add 2-3 aerial weapon variants

#### 🟡 NO COLLISION/PATHFINDING
- **Current**: Straight-line movement
- **Missing**: Obstacle avoidance, aerial pathfinding layer
- **Impact**: Units pass through terrain/buildings
- **Fix**: Implement aerial collision detection

#### 🟡 NO ANIMATIONS
- **Current**: Instant altitude changes
- **Missing**: Takeoff/landing animations, descent/ascent sequences
- **Impact**: Visual feedback missing
- **Fix**: Add animation hooks (nice-to-have, post-launch)

---

## Launch Blockers (Critical Path)

### PHASE 1: IMMEDIATE (Blocking Ship)

**Milestone 1**: Add Aerial Units (1 sprint)
```
Task: Star Wars pack
- Add rep_v19_torrent (AirstrikeProxy + Aerial tag)
- Add cis_tri_fighter (AirstrikeProxy + Aerial tag)
- Add building: rep_skyshield_generator (AntiAir)
- Add building: cis_vulture_nest (AntiAir)
Impact: Aviation subsystem now testable in gameplay
```

**Milestone 2**: AerialTargetingSystem (1 sprint)
```
Task: Runtime/Aviation
- Implement AerialTargetingSystem in SimulationSystemGroup
- Scan for targets, apply AntiAirDamageBonus
- Integrate with damage system
Impact: Aerial units can now engage enemies
```

**Milestone 3**: Anti-Air Weapons (1 sprint)
```
Task: Star Wars pack
- Add rep_missile_launcher (AntiAirBonus)
- Add cis_seismic_charge (AntiAirBonus)
Impact: Aerial attacks feel specialized, not generic
```

**Gate**: Can ship when:
- ✅ 2+ aerial units in Star Wars
- ✅ 4+ anti-air buildings (2 per faction)
- ✅ AerialTargetingSystem implemented
- ✅ Gameplay loop works (spawn aerial → attack ground → counter with anti-air)

### PHASE 2: HIGH PRIORITY (Pre-Launch)

**Milestone 4**: Unit Class Expansion (1 sprint)
```
Add 4-6 units to fill unused classes:
- Artillery (Clone Mortar, Droid Artillery)
- AntiArmor (Clone AT-RT, Droid Tank Killer)
- SwarmFodder (B1 Fodder, Cannon Droids)
- ShockMelee (Commando Corps, Elite B1)
Pushes coverage from 33.8% to ~50%
```

**Milestone 5**: Building Variety (1 sprint)
```
Add faction-specific buildings:
- Tipoca City (Republic tech lab)
- Geonosis Laboratory (CIS tech lab)
- Kaminoan Genetics (unique Republic)
- Lucrehulk Command (unique CIS)
Adds flavor, gameplay variety
```

### PHASE 3: LAUNCH QUALITY (Post-MVP)

**Milestone 6**: Asset Pipeline (2 sprints)
- Source V-19 Torrent, Tri-fighter models
- Texture with Star Wars colors
- Import via Addressables v1.21.18

**Milestone 7**: Advanced Aviation (1 sprint)
- Aerial collision/pathfinding
- Takeoff/landing animations
- Audio cues for aerial threats

---

## Success Metrics

### Star Wars Pack Target

| Metric | Target | Current | Gap | Priority |
|--------|--------|---------|-----|----------|
| Unit Coverage | 80%+ | 33.8% | +30 units | MEDIUM |
| Unit Classes | 18/21 | 13/21 | +5 classes | MEDIUM |
| Aerial Units | 4+ | 0 | +4 | **CRITICAL** |
| Anti-Air Buildings | 4+ | 0 | +4 | **CRITICAL** |
| Buildings | 25+ | 20 | +5 | HIGH |
| Weapons | 25+ | 18 | +7 | MEDIUM |
| Doctrines | 6+ | 6 | ✓ | DONE |
| Factions | 2 | 2 | ✓ | DONE |

### Aviation Subsystem Target

| Metric | Target | Current | Gap | Priority |
|--------|--------|---------|-----|----------|
| Targeting System | 1 | 0 | +1 | **CRITICAL** |
| Example Units | 4+ | 0 | +4 | **CRITICAL** |
| Anti-Air Buildings | 4+ | 0 | +4 | **CRITICAL** |
| Anti-Air Weapons | 2+ | 0 | +2 | HIGH |
| Pathfinding | Yes | No | TBD | MEDIUM |
| Animations | Yes | No | TBD | LOW |

---

## Sprint Plan

### Sprint 1 (Next 2 weeks)
- ✅ Vanilla-dino complete (DONE)
- 🟡 Add V-19 Torrent + Tri-fighter to Star Wars
- 🟡 Add Skyshield + Vulture nest buildings
- 🟡 Implement AerialTargetingSystem

### Sprint 2 (Following 2 weeks)
- 🟡 Add 4-6 units to fill unused classes
- 🟡 Add anti-air weapon variants
- 🟡 Add faction-specific buildings

### Sprint 3 (2 weeks after)
- 🟡 Asset pipeline (models, textures)
- 🟡 Aerial pathfinding
- 🟡 Polish & test

---

## Archive Status

**Archived for Future Work**:
- `packs/_archived/warfare-guerrilla/` (Modern combat)
- `packs/_archived/warfare-modern/` (Guerrilla warfare)

**Rationale**: Full focus on Star Wars + Aviation integration. Can revive when core subsystems stable.

---

## Key Takeaway

**Vanilla-DINO is a MUTABLE BASELINE**. Star Wars and Aviation are the test cases for how mods adjust to vanilla changes. Success = demonstrating:
1. Mod units reference vanilla archetypes (not copy them)
2. Mod buildings extend vanilla production chains
3. Mod doctrines build on vanilla faction archetypes
4. New subsystems (Aviation) integrate seamlessly with vanilla content

Current focus: **Close the critical gaps (aerial units + targeting system) to ship a working Star Wars MVP that proves the framework works.**

