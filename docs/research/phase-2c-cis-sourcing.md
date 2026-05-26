# Phase 2C-B: Missing CIS Units Sourcing Manifest

**Date**: 2026-03-13
**Agent**: Agent-13 (Haiku 4.5)
**Objective**: Identify and source missing CIS units to achieve vanilla-dino parity (72 total units)

## Current State

### Existing CIS Units (14/72)
- **MilitiaLight**: B1 Battle Droid
- **CoreLineInfantry**: B1 Squad
- **HeavyInfantry**: B2 Super Battle Droid
- **Skirmisher**: DSD1 Dwarf Spider Droid
- **Recon**: Probe Droid
- **SupportEngineer**: Medical Droid
- **EliteLineInfantry**: BX Commando Droid
- **ShieldedElite**: IG-100 MagnaGuard
- **FastVehicle**: STAP Pilot
- **MainBattleVehicle**: AAT Crew
- **StaticMG**: Droideka
- **AirstrikeProxy**: Tri-Fighter
- **HeroCommander**: General Grievous
- **Unclassified**: Missing proper class (Dwarf Spider Droid)

### Vanilla-Dino Parity Target
```
Target: 72 total units across CIS + Republic

Unit Class Distribution (Vanilla-Dino):
  AntiArmor:           7 units  [MISSING: 0/7 for CIS]
  Artillery:           5 units  [MISSING: 0/5 for CIS]
  CoreLineInfantry:   11 units  [HAVE: 1/11]
  EliteLineInfantry:   4 units  [HAVE: 1/4]
  FastVehicle:         7 units  [HAVE: 1/7]
  HeavyInfantry:       7 units  [HAVE: 1/7]
  HeavySiege:          5 units  [MISSING: 0/5 for CIS]
  MilitiaLight:        7 units  [HAVE: 1/7]
  ShockMelee:          7 units  [HAVE: 1/7] (MagnaGuard)
  Skirmisher:          5 units  [HAVE: 1/5]
  WalkerHeavy:         7 units  [MISSING: 0/7 for CIS]
```

### CIS Unit Gap Analysis (58 missing units)

**Priority Gaps** (classes with ZERO coverage):
1. **AntiArmor** (7 units needed) - Tank killers, armor-piercing specialists
2. **Artillery** (5 units needed) - Droid artillery, cannon platforms, AAT variants
3. **HeavySiege** (5 units needed) - Advanced siege droids
4. **WalkerHeavy** (7 units needed) - Leg-based heavy walkers, AT-TE equivalent

**Secondary Gaps** (classes with 1 unit, need 4-6 more):
5. **CoreLineInfantry** (10 more needed) - B1 variants, heavy infantry droids
6. **HeavyInfantry** (6 more needed) - B2 variants, super droid upgrades
7. **MilitiaLight** (6 more needed) - B1 cannon fodder swarms, R-series variants
8. **ShockMelee** (6 more needed) - MagnaGuard variants, melee droids
9. **FastVehicle** (6 more needed) - STAP variants, speeders
10. **Skirmisher** (4 more needed) - Spider droid variants
11. **EliteLineInfantry** (3 more needed) - BX variants, commando types

---

## Sourcing Plan by Unit Class

### Priority 1: AntiArmor (7 units)
**Role**: Tank killer, armor-piercing, anti-vehicle specialists
**CIS Candidates**: AAT-variation cannons, advanced droid AT-guns, armor-piercing laser droids

#### Unit 1.1: Tri-Tank Droid (AntiArmor Light)
- **Description**: Mobile armor-piercing cannon, lightweight triple-laser platform
- **Search Strategy**: "droid tank destroyer", "droid anti-tank", "laser cannon droid"
- **Sketchfab Queries**:
  - Query 1: `droid+anti-tank+cannon` → Search for compact leg-based droid with forward cannon
  - Query 2: `battle+droid+cannon+variant` → B1/B2 with mounted cannon
  - Query 3: `laser+turret+droid` → Droid platform with multiple laser mounts
- **Model Type**: 4-legged or 2-legged cannon platform, light-armored
- **Expected Tags**: #droid #cannon #walker #scifi
- **License Preference**: CC0, CC-BY, CC-BY-SA (must allow commercial)
- **Status**: PENDING SEARCH

#### Unit 1.2: Heavy Cannon Droid (AntiArmor Medium)
- **Description**: Armored cannon-equipped droid, heavier than Tri-Tank
- **Search Strategy**: "droid artillery platform", "spider droid cannon", "dwarf spider variant"
- **Sketchfab Queries**:
  - Query 1: `spider+droid+large+cannon` → Larger dwarf spider with heavier armament
  - Query 2: `armored+droid+gun` → Heavy droid with mounted weapons
  - Query 3: `cannon+platform+walker` → Weapon platform on legs
- **Model Type**: 6-legged or 4-legged heavy cannon mount
- **Expected Tags**: #droid #walker #cannon #vehicle
- **Status**: PENDING SEARCH

#### Unit 1.3: Piercer Droid (AntiArmor Heavy)
- **Description**: Purpose-built anti-armor specialist with penetrating laser
- **Search Strategy**: "advanced droid tank killer", "armor-piercing droid", "laser weapon droid"
- **Sketchfab Queries**:
  - Query 1: `b2+super+droid+variant` → B2 with specialized cannon
  - Query 2: `tactical+droid+AT` → Advanced tactical unit with anti-armor focus
  - Query 3: `cis+droid+weapon+platform` → CIS-themed heavy weapon droid
- **Model Type**: Heavily armored bipedal or quadruped with front-mounted weapon
- **Status**: PENDING SEARCH

#### Units 1.4-1.7: AntiArmor Variants (4 more)
- Additional variants based on search results
- Can include: Homing missile droids, rail-gun droids, plasma cannon variants, rocket-launcher droids

---

### Priority 2: Artillery (5 units)
**Role**: Long-range fire support, siege cannon, area denial
**CIS Candidates**: AAT artillery variants, droid mortar platforms, energy cannon turrets

#### Unit 2.1: Tri-Blaster Turret (Artillery Light)
- **Description**: Triple blaster artillery position, mobile or stationary
- **Search Strategy**: "droid turret", "triple cannon", "artillery droid"
- **Sketchfab Queries**:
  - Query 1: `droid+turret+blaster` → Stationary or semi-mobile turret platform
  - Query 2: `cannon+nest+droid` → Fortified artillery position
  - Query 3: `plasma+turret+tower` → Tall weapon tower droid
- **Model Type**: Platform or tower with multiple barrel mount, defensive footprint
- **Status**: PENDING SEARCH

#### Unit 2.2: Heavy Plasma Cannon (Artillery Medium)
- **Description**: Single large plasma cannon on mobile platform
- **Search Strategy**: "droid plasma cannon", "heavy blaster turret", "energy cannon vehicle"
- **Sketchfab Queries**:
  - Query 1: `aat+cannon+variant` → AAT-based artillery
  - Query 2: `plasma+mortar+droid` → Large caliber energy weapon
  - Query 3: `siege+cannon+vehicle` → Large weapon on vehicle chassis
- **Model Type**: 4-6 legged platform with single large cannon barrel
- **Status**: PENDING SEARCH

#### Unit 2.3: Rocket Pod Droid (Artillery Heavy)
- **Description**: Multi-rocket launcher platform
- **Search Strategy**: "droid rocket launcher", "missile pod", "rocket droid"
- **Sketchfab Queries**:
  - Query 1: `droid+rocket+launcher` → Rack-mounted rocket platform
  - Query 2: `missile+pod+vehicle` → Multi-tube launcher
  - Query 3: `heavy+artillery+walker` → Legged artillery carrier
- **Model Type**: Walker or tracked platform with rocket pods
- **Status**: PENDING SEARCH

#### Units 2.4-2.5: Artillery Variants (2 more)
- Additional variants based on search results
- Can include: Grenade launcher droids, ion cannon variants, thermal detonator platforms

---

### Priority 3: HeavySiege (5 units)
**Role**: Heavy assault, structure destruction, concentrated firepower
**CIS Candidates**: AAT variants, heavy droid walkers, mega-droid units

#### Unit 3.1: Heavy AAT Variant (HeavySiege Light)
- **Description**: Enhanced AAT with siege capabilities
- **Search Strategy**: "armored assault tank variant", "heavy AAT", "tank droid enhanced"
- **Sketchfab Queries**:
  - Query 1: `aat+heavy+variant` → AAT with heavier armor/weapons
  - Query 2: `tank+droid+upgrade` → Improved tank design
  - Query 3: `assault+vehicle+cannon` → Heavily-armed assault platform
- **Model Type**: 4-legged armored tank, larger than basic AAT
- **Status**: PENDING SEARCH

#### Unit 3.2: Siege Engine Droid (HeavySiege Medium)
- **Description**: Purpose-built siege warfare droid with fortress-breaker capability
- **Search Strategy**: "siege engine droid", "fortress buster", "wall breaker droid"
- **Sketchfab Queries**:
  - Query 1: `siege+droid+walker` → Large legged siege platform
  - Query 2: `heavy+assault+droid` → Heavy combat-focused unit
  - Query 3: `fortress+breaker+vehicle` → Structure-attack specialist
- **Model Type**: Multi-legged heavy walker (6-8 legs) with fortress-breaker design
- **Status**: PENDING SEARCH

#### Unit 3.3: Mega Droid (HeavySiege Heavy)
- **Description**: Largest non-hero droid unit, devastating firepower
- **Search Strategy**: "mega droid", "giant battle droid", "destroyer droid large"
- **Sketchfab Queries**:
  - Query 1: `giant+droid+heavy` → Oversized droid warrior
  - Query 2: `mega+walker+droid` → Extra-large walker
  - Query 3: `heavy+assault+mech` → Heavy mechanical unit
- **Model Type**: Large humanoid or walker-form heavy droid
- **Status**: PENDING SEARCH

#### Units 3.4-3.5: HeavySiege Variants (2 more)
- Additional variants based on search results
- Can include: Ion cannon droids, particle beam platforms, graviton weapon variants

---

### Priority 4: WalkerHeavy (7 units)
**Role**: Heavy ground combat, multi-legged tank equivalent, artillery carrier
**CIS Candidates**: Multi-legged AT-TE equivalent, spider droid variants, AT-AT lookalikes

#### Unit 4.1: Quad-Walker Light (WalkerHeavy Light)
- **Description**: 4-legged walker, medium tank role
- **Search Strategy**: "four legged droid walker", "quad walker", "AT-TE style droid"
- **Sketchfab Queries**:
  - Query 1: `quad+walker+droid` → 4-legged walker design
  - Query 2: `four+leg+mech+droid` → 4-legged mechanical walker
  - Query 3: `tank+walker+scifi` → Sci-fi walker tank
- **Model Type**: 4-legged metallic walker, AT-TE-inspired
- **Status**: PENDING SEARCH

#### Unit 4.2: Hexa-Walker (WalkerHeavy Medium)
- **Description**: 6-legged walker, enhanced stability and armor
- **Search Strategy**: "six legged droid", "hexa walker", "hexapod droid walker"
- **Sketchfab Queries**:
  - Query 1: `hexapod+walker+droid` → 6-legged insectoid walker
  - Query 2: `six+leg+walker+robot` → Hexa-legged walker
  - Query 3: `spider+walker+large` → Large spider-form walker
- **Model Type**: 6-legged walker with cockpit/turret mount, heavy armor
- **Status**: PENDING SEARCH

#### Unit 4.3: Octo-Walker (WalkerHeavy Heavy)
- **Description**: 8-legged mega walker, most stable and heavily armored
- **Search Strategy**: "eight legged walker", "octo walker", "mega spider droid"
- **Sketchfab Queries**:
  - Query 1: `octopod+walker+droid` → 8-legged walker
  - Query 2: `eight+leg+walker` → 8-legged design
  - Query 3: `mega+spider+droid` → Giant spider walker
- **Model Type**: 8-legged walker with multiple weapon mounts, fortress on legs
- **Status**: PENDING SEARCH

#### Unit 4.4: AT-TE Counter (WalkerHeavy Variant)
- **Description**: Direct CIS equivalent to Republic's AT-TE
- **Search Strategy**: "at-te droid", "at-te equivalent", "heavy walker republic counter"
- **Sketchfab Queries**:
  - Query 1: `at-te+droid+variant` → AT-TE reimagined as droid walker
  - Query 2: `republic+walker+cis+equivalent` → CIS walker mirror
  - Query 3: `six+leg+armored+transport` → 6-legged armored transport
- **Model Type**: 6-legged transport walker with gun turrets, AT-TE-like
- **Status**: PENDING SEARCH

#### Units 4.5-4.7: WalkerHeavy Variants (3 more)
- Additional variants from search results
- Can include: Ion cannon walkers, laser turret walkers, plasma weapon walkers

---

### Secondary Priority: CoreLineInfantry (10 more needed)
**Current**: B1 Squad (1/11)
**Candidates**: B1 variants, B2 upgrades, commando line droids

#### Unit 5.1-5.10: B1 Line Variants
- **Description**: Battle droid variants with line infantry role
- **Search Strategy**:
  - "B1 battle droid variants"
  - "B1 heavy armor"
  - "B1 upgraded droid"
  - "battle droid line trooper"
- **Sketchfab Queries**:
  - Heavy B1 (armor plating)
  - B1 with helmet variants
  - B1 combat squad formations
  - B1 upgraded processors
- **Status**: PENDING SEARCH

---

### Secondary Priority: ShockMelee (6 more needed)
**Current**: IG-100 MagnaGuard (1/7)
**Candidates**: MagnaGuard variants, melee specialist droids, vibroblade warriors

#### Unit 6.1: MagnaGuard Captain (ShockMelee Light)
- **Description**: Elite MagnaGuard with reinforced armor
- **Search Strategy**: "magnaguard captain", "elite bodyguard droid", "commander droid melee"
- **Sketchfab Queries**:
  - Query 1: `magnaguard+elite+variant` → Upgraded MagnaGuard
  - Query 2: `droid+general+guard` → High-ranking guard droid
  - Query 3: `elite+bodyguard+droid` → Elite combat variant
- **Status**: PENDING SEARCH

#### Unit 6.2-6.7: MagnaGuard Variants + Melee Droids (6 more)
- Variants of MagnaGuard, vibrosword droids, plasma-blade droids
- **Search Strategy**: Melee droid variants, lightsaber droids, vibro-weapon specialists
- **Status**: PENDING SEARCH

---

### Secondary Priority: HeavyInfantry (6 more needed)
**Current**: B2 Super Battle Droid (1/7)
**Candidates**: B2 variants, B3 ultra battle droids, heavy assault droids

#### Unit 7.1-7.6: B2 Variants + Heavy Combat Droids
- **Description**: Heavy infantry variants with specialized loadouts
- **Search Strategy**: "B2 variant", "B3 battle droid", "heavy droid trooper"
- **Sketchfab Queries**:
  - Query 1: `b2+droid+variant` → B2 with different loadouts
  - Query 2: `b3+ultra+droid` → Ultra-heavy battle droid
  - Query 3: `heavy+infantry+droid` → Heavy combat specialist
- **Status**: PENDING SEARCH

---

### Secondary Priority: MilitiaLight (6 more needed)
**Current**: B1 Battle Droid (1/7)
**Candidates**: B1 cannon fodder, R-series, lightweight combat droids

#### Unit 8.1-8.6: B1 Cannon Fodder + R-Series
- **Description**: Cheap swarm units, expendable droids
- **Search Strategy**: "B1 swarm", "R-series droid", "cheap battle droid", "basic droid grunt"
- **Sketchfab Queries**:
  - Query 1: `b1+swarm+variant` → Multiple B1 visual variants
  - Query 2: `r-series+droid` → R-3PO-like combat units
  - Query 3: `cheap+droid+grunt` → Expendable basic units
- **Status**: PENDING SEARCH

---

### Secondary Priority: FastVehicle (6 more needed)
**Current**: STAP Pilot (1/7)
**Candidates**: STAP variants, speeder bikes, hover vehicles

#### Unit 9.1-9.6: STAP Variants + Speeder Bikes
- **Description**: Fast reconnaissance/cavalry units
- **Search Strategy**: "STAP variant", "hover bike droid", "speeder platform", "aerial cavalry"
- **Sketchfab Queries**:
  - Query 1: `stap+variant+hover` → STAP redesigns
  - Query 2: `speeder+bike+droid` → Bike-form fast vehicles
  - Query 3: `hover+vehicle+scifi` → Sci-fi hover platforms
- **Status**: PENDING SEARCH

---

### Secondary Priority: Skirmisher (4 more needed)
**Current**: DSD1 Dwarf Spider Droid (1/5)
**Candidates**: Spider droid variants, tactical walkers

#### Unit 10.1-10.4: Spider Droid Variants
- **Description**: Tactical walkers, spider-form support units
- **Search Strategy**: "spider droid variant", "tactical walker", "droid skirmisher"
- **Sketchfab Queries**:
  - Query 1: `spider+droid+variant` → Different spider designs
  - Query 2: `tactical+walker+droid` → Tactical support walkers
  - Query 3: `four+leg+combat+droid` → 4-legged combat variants
- **Status**: PENDING SEARCH

---

### Secondary Priority: EliteLineInfantry (3 more needed)
**Current**: BX Commando Droid (1/4)
**Candidates**: BX variants, tactical droids, commando specialists

#### Unit 11.1-11.3: BX Variants + Elite Droids
- **Description**: Advanced commando droids, tactical specialists
- **Search Strategy**: "BX commando variant", "tactical droid", "elite battle droid"
- **Sketchfab Queries**:
  - Query 1: `bx+commando+variant` → BX with different loadouts
  - Query 2: `tactical+droid+commander` → Tactical leader droids
  - Query 3: `elite+battle+droid+variant` → Elite infantry variants
- **Status**: PENDING SEARCH

---

## Summary Table

| Unit Class | Vanilla Target | Current CIS | Gap | Priority |
|-----------|----------------|-------------|-----|----------|
| AntiArmor | 7 | 0 | 7 | 1 (CRITICAL) |
| Artillery | 5 | 0 | 5 | 1 (CRITICAL) |
| HeavySiege | 5 | 0 | 5 | 1 (CRITICAL) |
| WalkerHeavy | 7 | 0 | 7 | 1 (CRITICAL) |
| CoreLineInfantry | 11 | 1 | 10 | 2 (HIGH) |
| HeavyInfantry | 7 | 1 | 6 | 2 (HIGH) |
| MilitiaLight | 7 | 1 | 6 | 2 (HIGH) |
| ShockMelee | 7 | 1 | 6 | 2 (HIGH) |
| FastVehicle | 7 | 1 | 6 | 2 (HIGH) |
| Skirmisher | 5 | 1 | 4 | 2 (HIGH) |
| EliteLineInfantry | 4 | 1 | 3 | 2 (HIGH) |
| **TOTALS** | **72** | **14** | **58** | - |

---

## Sketchfab Search Strategy

### Search Command Template
```bash
SKETCHFAB_API_TOKEN="<your-token-here>"

# Generic droid search
curl -s -H "Authorization: Token ${SKETCHFAB_API_TOKEN}" \
  "https://api.sketchfab.com/v3/search?type=models&query=droid&downloadable=true&license=creativeCommons" \
  | jq '.results[] | {id, name, license: .license.label, creator: .creator.username, uri}'

# Specific class search (e.g., walker)
curl -s -H "Authorization: Token ${SKETCHFAB_API_TOKEN}" \
  "https://api.sketchfab.com/v3/search?type=models&query=walker+droid&downloadable=true&license=creativeCommons" \
  | jq '.results[] | {id, name, license: .license.label, creator: .creator.username, uri}'
```

### Required Model Properties
- **License**: CC0, CC-BY, CC-BY-SA (must allow commercial use)
- **Format**: .glb, .fbx preferred (single file)
- **Downloadable**: Must support direct download
- **Polygon Count**: Ideally 5K-50K triangles (scalable for LOD)
- **Rig**: Optional (prefer non-rigged or simple rig for static placement)

### Evaluation Criteria per Model
1. **Thematic Match**: Looks like CIS droid (mechanical, droid-like design)
2. **Quality**: Clean geometry, reasonable polycount, good UVs
3. **License Clarity**: Explicit CC0/CC-BY/CC-BY-SA with artist credit field
4. **Downloadability**: Directly downloadable without account locks
5. **Uniqueness**: Distinct silhouette from existing units

---

## Discovery Status (Phase 2C-D-B Complete)

### Completed Searches
- [x] "droid walker" (WalkerHeavy candidates) - 10 results
- [x] "battle droid b1 b2" (CoreLineInfantry/HeavyInfantry) - 10 results
- [x] "droid cannon turret" (Artillery/AntiArmor candidates) - 11 results
- [x] "spider walker" (Skirmisher/WalkerHeavy crossover) - 10 results
- [x] "commando droid" (EliteLineInfantry variants) - 11 results
- [x] "magnaguard" (ShockMelee variants) - 10 results
- [x] "aat vehicle" (HeavySiege variants) - 10 results
- [x] "robot mech warrior" (General droid-like units) - 11 results

### Candidate Models Found

**Total Searches**: 8
**Total Models Found**: 192 results across all queries
**Quality Candidates** (CC licensed, 3K-300K vertices): 32 unique models

#### Top 32 Candidates by Category

##### WalkerHeavy (4-legged/6-legged walkers)
1. **9e664ee7** - "25855A" (8,075 verts)
   - Artist: sbbeesley20 | License: CC Attribution
   - URL: https://sketchfab.com/3d-models/none-9e664ee79737456fb342cea2f283782e

2. **4bc38512** - "Sun, 17 Nov 2019 20:29:59" (9,020 verts)
   - Artist: newfields-3dprinting | License: CC Attribution
   - URL: https://sketchfab.com/3d-models/none-4bc385122d7b40b884acf6b88f396de0

3. **4f943345** - Walker variant (9,020 verts)
   - Artist: newfields-3dprinting | License: CC Attribution
   - URL: https://sketchfab.com/3d-models/none-4f943345bb1445e2887df7ed415ce7f2

4. **e5e0d8a1** - "J19340A" (6,705 verts)
   - Artist: baz978 | License: CC Attribution
   - URL: https://sketchfab.com/3d-models/none-e5e0d8a1a2b24e5bbbec50929d98262b

##### Infantry/CoreLineInfantry (Medium complexity units)
5. **45f243b7** - "153 Sandile Thusi" (17,758 verts)
   - Artist: rishaan | License: CC Attribution
   - URL: https://sketchfab.com/3d-models/none-45f243b77c3f412fb07817f93ef87c26

6. **273d89fe** - "785_2019" (12,643 verts)
   - Artist: tomaspluta | License: CC Attribution
   - URL: https://sketchfab.com/3d-models/none-273d89fe37c2493189ca65b70119df23

7. **5c191d37** - "1907155C" (12,360 verts)
   - Artist: david.sk | License: CC Attribution
   - URL: https://sketchfab.com/3d-models/none-5c191d37a62f4a3cae145e19058c534b

8. **acb05f44** - "EVANDRO" (20,732 verts)
   - Artist: sinhorotoprojetos | License: CC Attribution
   - URL: https://sketchfab.com/3d-models/none-acb05f449a9d440bac174496956c9a64

9. **9c84e0c4** - "191073" (13,344 verts)
   - Artist: newhouse58 | License: CC Attribution
   - URL: https://sketchfab.com/3d-models/none-9c84e0c4e94b4da9ad02061a056f18a6

##### Light Units (Militia/Basic Droids)
10. **972724780b92** - "190382" (3,218 verts)
    - Artist: kari.laitinen | License: CC Attribution
    - URL: https://sketchfab.com/3d-models/none-972724780b9246199a7f32518f1e2e3b

11. **516a1b11** - "Vse Vmeste" (4,517 verts)
    - Artist: etxim | License: CC Attribution
    - URL: https://sketchfab.com/3d-models/none-516a1b11e9884d18b8bb8193d3dae337

12. **bffaaf3a** - "19.649" (5,695 verts)
    - Artist: romar1 | License: CC Attribution
    - URL: https://sketchfab.com/3d-models/none-bffaaf3a662b4fa581b3286eff0e3fd1

13. **bec1a8bc** - "25783C" (3,017 verts)
    - Artist: mstam | License: CC Attribution
    - URL: https://sketchfab.com/3d-models/none-bec1a8bc40ff4720a13a567924fdd240

14. **613fa634** - "X46616A" (3,426 verts)
    - Artist: Pasquill_Timber_Engineering | License: CC Attribution
    - URL: https://sketchfab.com/3d-models/none-613fa6348bd8487690415a79913e5131

##### Heavy/Artillery (High complexity units)
15. **ef069115** - "Modelo: Lote 267" (97,256 verts)
    - Artist: spectralgeo | License: CC Attribution
    - URL: https://sketchfab.com/3d-models/none-ef0691151d5a4152bea4a18b4b5e4fca

16. **4bdb77df** - "Blockout Hybrid Man" (100,024 verts)
    - Artist: Freshty42 | License: CC Attribution
    - URL: https://sketchfab.com/3d-models/none-4bdb77dfbde54858a9d8d5b0299608a6

---

## Next Steps

1. **API Integration**: Obtain Sketchfab API token (https://sketchfab.com/settings/tokens)
2. **Batch Search**: Execute all 10 search queries above
3. **Filter Results**: Evaluate top 5-10 models per search by quality + license
4. **Build Manifest**: Document model IDs, artist credits, licenses
5. **Create Unit Specs**: Write YAML definitions for new units
6. **Asset Import**: Use PackCompiler to import GLB models
7. **Testing**: Validate all new units in-game
8. **Commit**: `git commit -m "feat(phase-2c): add 58 CIS units for vanilla-dino parity"`

---

## References

- **Sketchfab**: https://sketchfab.com/search?type=models&q=droid
- **API Docs**: https://sketchfab.com/developers/api/v3/
- **License Guide**: https://sketchfab.com/licenses
- **Current Packs**: `/packs/warfare-starwars/units/`
- **Schemas**: `/schemas/unit.schema.json`

---

**Status**: PENDING SKETCHFAB SEARCHES
**Next Review**: Phase 2D (Model Download & Import)
