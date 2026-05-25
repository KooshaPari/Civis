---
title: Aviation System
description: How aerial units work in DINOForge - flight mechanics, altitude, and air combat
---

# Aviation System

This guide explains how aerial units work in DINOForge, including flight mechanics, altitude management, and air-to-air combat.

## Overview

Aerial units are aircraft that operate in 3D space with controllable altitude (Y-axis elevation). They use a separate movement system from ground units and have specialized combat rules.

**Key concepts**:
- **Cruise Altitude**: Target elevation during normal flight
- **Ascend/Descend Speeds**: Rate of altitude change
- **Anti-Air**: Can engage other aerial targets
- **Behavior Tags**: Must include `Aerial` tag

## Defining an Aerial Unit

### Unit YAML Definition

All aerial units must:
1. Include `Aerial` in `behavior_tags`
2. Define an `aerial` block with flight parameters

**Example**:
```yaml
id: clone_starfighter
display_name: Clone Starfighter
unit_class: FastVehicle
faction_id: republic
tier: 2

stats:
  hp: 80
  damage: 14
  armor: 3
  range: 10
  speed: 7
  accuracy: 0.85
  fire_rate: 1.5
  morale: 95
  cost:
    food: 0
    wood: 40
    stone: 20
    iron: 60
    gold: 30
    population: 1

weapon: fighter_cannon

defense_tags:
  - Mechanical

behavior_tags:
  - AdvanceFire
  - Aerial
  - AntiAir

aerial:
  cruise_altitude: 20
  ascend_speed: 6
  descend_speed: 4
  anti_air: true
```

### Aerial Properties Explained

#### cruise_altitude
- **Type**: `float`
- **Units**: World units (same as ground movement)
- **Description**: Target altitude during normal flight

**Typical values**:
- **10-15**: Low flight (helicopters, bombers doing low runs)
- **15-25**: Medium altitude (fighters, standard aircraft)
- **25+**: High altitude (interceptors, strategic bombers)

```yaml
aerial:
  cruise_altitude: 20  # Medium altitude
```

**Why it matters**:
- Prevents collision with tall buildings (e.g., towers are ~10 units tall)
- Affects visual presence (higher = appears smaller)
- Influences engagement ranges with ground units
- Determines vulnerability to grounded anti-air

#### ascend_speed
- **Type**: `float`
- **Units**: World units per second
- **Range**: Typically 3-8
- **Description**: Rate at which unit climbs to cruise altitude

```yaml
ascend_speed: 6
```

**Examples**:
- **3-4**: Slow climbers (bombers, transports, helicopters)
- **5-6**: Standard aircraft (fighters, gunships)
- **7-8**: Fast climbers (interceptors, light fighters)

**Gameplay effect**: Slow ascent = vulnerable during takeoff, fast = quick escape

#### descend_speed
- **Type**: `float`
- **Units**: World units per second
- **Range**: Typically 2-5
- **Description**: Rate at which unit descends when attacking or returning to ground

```yaml
descend_speed: 4
```

**Usually slower than ascent** for more interesting tactical play:
- Forces commitment to attack runs
- Prevents instant escape
- Creates vulnerable landing phase

#### anti_air
- **Type**: `boolean`
- **Description**: Whether this unit can engage aerial targets

```yaml
aerial:
  anti_air: true   # Can shoot down other aircraft
```

**Examples**:
- `anti_air: true` — Interceptor, air superiority fighter, AA gun
- `anti_air: false` — Bomber, transport, air support (ground-attack only)

## Flight Mechanics System

### How Altitude Works

Units operate in 3D space with ground elevation (Y-axis):

```
Y-axis (altitude)
↑
│
25 ├─ Interceptor (high altitude, fast)
   │
20 ├─ Fighter (medium altitude)
   │
15 ├─ Helicopter (low altitude, slow)
   │
10 ├─ Building tops (vulnerable)
   │
5  ├─ Ground level
   │
0  └─────────────────────→ Movement (X, Z axes)
```

### Movement States

**Ground State** (Y = 0):
- Unit is grounded
- Cannot move until takeoff
- Can be attacked by ground units

**Takeoff** (0 < Y < cruise_altitude):
- Unit ascending at `ascend_speed`
- Partially vulnerable to anti-air
- Cannot attack until reaching cruise altitude

**Cruising** (Y = cruise_altitude):
- Normal flight altitude
- Full movement and attack capability
- Can see entire map

**Attack Run** (cruise_altitude > Y > target):
- Unit descending toward ground target
- At descend_speed
- Taking fire from ground units

**Landing** (0 < Y < cruise_altitude):
- Unit descending to ground
- Vulnerable during descent

### Combat Rules

#### Aerial vs Ground
- Aerial units can attack grounded units
- Grounded units can attack aerial units (reduced accuracy if unit is high)
- Anti-air ground units get accuracy bonus vs low-altitude targets

#### Aerial vs Aerial
- Only possible if **both units have `anti_air: true`**
- Occurs when units pass through same altitude band
- Full normal combat rules apply

#### Example: No Air-to-Air Without Anti-Air
```yaml
# Bomber (transport, no anti-air)
aerial:
  anti_air: false

# Interceptor (anti-air fighter)
aerial:
  anti_air: true

# Result: Interceptor can shoot bomber, bomber cannot fight back
```

## Behavior Tags for Aerial Units

### Aerial (Required)
```yaml
behavior_tags:
  - Aerial
```

Marks unit as aerial. Without this, the `aerial` block is ignored.

### Common Combinations

#### Fighter (Air Superiority)
```yaml
behavior_tags:
  - Aerial
  - AntiAir
  - AdvanceFire
```

Actively hunts other aircraft, fast, aggressive.

#### Ground Attack (CAS - Close Air Support)
```yaml
behavior_tags:
  - Aerial
  - AntiStructure
  - Charge
```

Focuses on ground targets and buildings, low and fast.

#### Bomber
```yaml
behavior_tags:
  - Aerial
  - SiegePriority
  - AntiStructure
```

Heavy damage to structures, slow, vulnerable to interception.

#### Reconnaissance
```yaml
behavior_tags:
  - Aerial
  - Kite
```

Fast, avoids combat, provides vision/scouting.

## Flight Path Examples

### Standard Attack Run

```
Altitude
    ↑
25  │         /
    │        /
20  ├───→ / ←─────────── Cruise altitude
    │    /
15  │   /
    │  /
10  │ /  Descending (descend_speed = 3)
    │/
 0  └────────────────────→ Ground
    Takeoff         Attack    Landing
    Time
```

Unit path:
1. Ascend from 0 to 20 at 6 units/sec = 3.3 seconds
2. Cruise to target at ground speed = variable
3. Descend from 20 to 5 at 3 units/sec = 5 seconds
4. Attack at low altitude, return to cruise
5. Ascend and retreat

### Interceptor vs Bomber

```
    25 │        I (Interceptor: fast, high)
       │       /│
    20 │      / │B (Bomber: slow, medium)
       │     /  │
    15 │    /   │
       │   /    │
    10 │  /     ↓
       │ /
     0 └────────→
```

- Bomber flying steady at Y=20
- Interceptor climbs from Y=10 to Y=25 to gain height advantage
- Intercepts from above (typical dogfighting advantage)
- Anti-air combat begins when passing through same altitude

## Practical Configuration Examples

### Republic Fighter (Star Wars Theme)

```yaml
id: arc170_fighter
display_name: ARC-170 Starfighter
unit_class: FastVehicle
faction_id: republic

stats:
  hp: 90
  damage: 16
  armor: 4
  range: 12
  speed: 8
  accuracy: 0.9
  fire_rate: 1.4
  morale: 100
  cost:
    food: 0
    wood: 40
    stone: 20
    iron: 80
    gold: 40
    population: 1

weapon: twin_blaster_cannon

defense_tags:
  - Mechanical

behavior_tags:
  - Aerial
  - AntiAir
  - AdvanceFire

aerial:
  cruise_altitude: 25
  ascend_speed: 7
  descend_speed: 4
  anti_air: true
```

### CIS Gunship (Transport/Support)

```yaml
id: droid_gunship
display_name: Droid Gunship
unit_class: MainBattleVehicle
faction_id: cis

stats:
  hp: 120
  damage: 20
  armor: 6
  range: 15
  speed: 5
  accuracy: 0.8
  fire_rate: 1.0
  morale: 80
  cost:
    food: 0
    wood: 50
    stone: 30
    iron: 100
    gold: 50
    population: 2

weapon: battle_cannon

defense_tags:
  - HeavyArmor
  - Mechanical

behavior_tags:
  - Aerial
  - AntiStructure
  - SiegePriority

aerial:
  cruise_altitude: 15
  ascend_speed: 5
  descend_speed: 5
  anti_air: false  # No air combat, support only
```

### Republic Bomber

```yaml
id: y_wing_bomber
display_name: Y-Wing Bomber
unit_class: Artillery
faction_id: republic

stats:
  hp: 100
  damage: 25
  armor: 5
  range: 18
  speed: 4
  accuracy: 0.75
  fire_rate: 0.8
  morale: 85
  cost:
    food: 0
    wood: 60
    stone: 40
    iron: 120
    gold: 60
    population: 2

weapon: proton_bomb_load

defense_tags:
  - HeavyArmor
  - Mechanical

behavior_tags:
  - Aerial
  - SiegePriority
  - AntiStructure

aerial:
  cruise_altitude: 18
  ascend_speed: 4
  descend_speed: 3
  anti_air: false  # Defenseless against fighters
```

## Validation Rules

When validating units with aerial properties:

1. **If `behavior_tags` contains `Aerial`**:
   - `aerial` block must exist and be non-empty
   - `cruise_altitude` must be &gt; 0
   - `ascend_speed` and `descend_speed` must be &gt; 0

2. **If `aerial` block exists**:
   - All numeric fields must be positive
   - `anti_air` must be boolean
   - Unit must have `behavior_tags: [Aerial]`

3. **Anti-air consistency**:
   - If `behavior_tags` contains `AntiAir`, unit should have `anti_air: true`
   - Non-aerial units can have `AntiAir` tag (ground-based AA)

## Design Principles

### Balance

- **Faster climb = shorter vulnerability window** (good for fighters)
- **Slower descent = longer attack commitment** (interesting tactics)
- **Limited anti-air = vulnerability to interception** (risk/reward)
- **High altitude = survives longer, less accurate from ground**

### Consistency

- Bombers: slow, high damage, no anti-air
- Fighters: fast, medium damage, full anti-air
- Support: medium speed, special effects, no combat
- Interceptors: fastest, anti-air focus, lower raw damage

### Playstyle

- Aerial combat requires **air superiority** (fighter advantage)
- Ground defense needs **AA units** (specialized counters)
- Bombers need **fighter cover** (support relationship)
- Transport units need **escort** (vulnerability creates interdependence)

## Common Issues

### "Unit Won't Fly"
- Check `behavior_tags` includes `Aerial`
- Check `aerial` block exists
- Check `cruise_altitude &gt; 0`

### "Aerial Unit Can't Hit Air Targets"
- Check both units have `anti_air: true`
- Check behavior tags include `AntiAir` on aerial combat units
- Check both units are within engagement altitude ranges

### "Unit Takes Too Long to Land"
- Increase `descend_speed` for faster landing
- Default is often conservative; many designs use `ascend_speed = descend_speed`

### "Bomber Gets Shot Down Too Fast"
- Increase `hp` and `armor` for tanking power
- Decrease `cruise_altitude` to be faster to target
- Add escort fighters with `anti_air: true`

## See Also

- [Unit Schema Reference](/reference/unit-schema) — Full aerial property docs
- [Warfare Overview](/warfare/overview) — Unit roles and archetypes
- [Creating Packs](/guide/creating-packs) — Full pack authoring guide
