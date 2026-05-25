# Warfare Domain Plugin Specification

**Version**: 0.1.0
**Status**: Draft
**Date**: 2026-03-09

---

## 1. Overview

The Warfare domain plugin extends DINOForge with combat-oriented modding capabilities: factions, doctrines, unit taxonomies, weapon systems, wave composition, and themed content packs.

This is the first domain plugin and serves as the reference implementation for the domain plugin architecture.

---

## 2. Faction Archetypes

Three mechanical families underpin all factions:

### A. Order / Conventional Quality Army
**Used by**: Republic, West

| Trait | Value |
|-------|-------|
| Line infantry | Strong |
| Ranged DPS | Reliable |
| Static defenses | Stronger |
| Artillery/vehicle access | Better |
| Command/support buffs | Better |
| Individual unit cost | Higher |
| Raw swarm count | Lower |

### B. Industrial Swarm / Mechanical Attrition Army
**Used by**: CIS, Classic West Enemy

| Trait | Value |
|-------|-------|
| Unit numbers | Larger |
| Core unit cost | Cheaper |
| Expendability | High |
| Siege pressure | Strong |
| Per-unit durability | Weaker (unless elite) |
| Emphasis | Mechanical/armor, massed fire |

### C. Asymmetric Harassment / Insurgent Pressure Army
**Used by**: Guerrilla West Enemy

| Trait | Value |
|-------|-------|
| Unit weight | Light |
| Mobility | High |
| Ambush capability | Strong |
| Frontline durability | Weak |
| Raid pressure | Strong |
| Anti-structure | Explosives/rockets |
| Battlefield presence | "Annoying" harassment |

---

## 3. Faction Definitions

### Republic (Star Wars)
- Theme: `starwars`, Archetype: `order`, Doctrine: `elite_discipline`
- Fantasy: Clone infantry line, specialist troopers, walkers, artillery, Jedi command
- Gameplay: High precision, strong fortified defense, expensive losses hurt, strong late game

### CIS (Star Wars)
- Theme: `starwars`, Archetype: `industrial_swarm`, Doctrine: `mechanized_attrition`
- Fantasy: B1 flood, B2 heavy line, droideka shields, spider/tank droids
- Gameplay: Flood and pressure, disposable front, oppressive economy, shield spikes

### West (Modern)
- Theme: `modern`, Archetype: `order`, Doctrine: `combined_arms_precision`
- Fantasy: Rifle squads, marksmen, AT teams, IFVs/MBTs, drones/airstrikes
- Gameplay: Flexible, strong anti-armor, premium units, high burst damage

### Classic West Enemy (Modern)
- Theme: `modern`, Archetype: `industrial_swarm`, Doctrine: `massed_peer_assault`
- Fantasy: Conscript mass, mechanized waves, rocket saturation, heavy fortifications
- Gameplay: Attritional, stronger mass, lower finesse, brutal siege, dangerous late push

### Guerrilla West Enemy (Modern)
- Theme: `modern`, Archetype: `asymmetric`, Doctrine: `raid_and_attrit`
- Fantasy: Raiders, technicals, RPG teams, mortars, tunnel/cache networks
- Gameplay: Weak head-on, strong vs exposed economy, high harassment

---

## 4. Unit Role Matrix

Every faction fills the same slots with theme-appropriate units:

| Slot | Republic | CIS | West | Classic Enemy | Guerrilla |
|------|----------|-----|------|---------------|-----------|
| T1 Cheap | Militia clone | B1 | Militia | Conscript | Irregular |
| T1 Core | Clone trooper | B1 improved/B2 | Rifle squad | Rifle infantry | Raider rifle |
| T2 Elite | ARC trooper | Commando droid | Spec ops | Guards/assault | Veteran cell |
| T2 Anti-armor | Clone AT | Heavy droid AT | ATGM team | AT team | RPG team |
| T2 Support | Repeater team | Heavy blaster | MG team | MG team | HMG/recoilless |
| T2 Recon | Scout trooper | Commando scout | Drone recon | Scout/mech recon | Infiltrator |
| T3 Light Vehicle | AT-RT/speeder | Dwarf/spider | Technical/IFV | IFV | Technical |
| T3 Heavy Vehicle | Walker/tank | AAT/heavy droid | MBT | MBT/assault gun | (N/A) |
| T3 Artillery | SP battery | Shelling droid | MLRS/howitzer | Tube+rocket arty | Mortar/rocket |
| Defense 1 | Blaster tower | Droid tower | MG nest | Bunker | Fortified nest |
| Defense 2 | Heavy laser | Shielded turret | AT/missile | Heavy missile | Rocket emplace |
| Commander | Jedi/clone cmdr | Tactical relay | Officer/UAV | Commissar/cmd | Field emir |
| Spike Unit | Shield elite | Droideka | Drone strike | Thermobaric | Ambush wave |

---

## 5. Combat Model

### Unit Classes
`MilitiaLight`, `CoreLineInfantry`, `EliteLineInfantry`, `HeavyInfantry`, `Skirmisher`, `AntiArmor`, `ShockMelee`, `SwarmFodder`, `FastVehicle`, `MainBattleVehicle`, `HeavySiege`, `Artillery`, `WalkerHeavy`, `StaticMG`, `StaticAT`, `StaticArtillery`, `SupportEngineer`, `Recon`, `HeroCommander`, `AirstrikeProxy`, `ShieldedElite`

### Weapon Classes
`BallisticLight`, `BallisticHeavy`, `BlasterLight`, `BlasterHeavy`, `ExplosiveAT`, `ExplosiveHE`, `BeamPrecision`, `FlameArea`, `MissileGuided`, `SuppressionWeapon`, `MeleeLight`, `MeleeHeavy`

### Defense Tags
`Unarmored`, `InfantryArmor`, `HeavyArmor`, `Fortified`, `Shielded`, `Mechanical`, `Biological`, `Heroic`

### Behavior Tags
`HoldLine`, `AdvanceFire`, `Charge`, `Kite`, `Swarm`, `SiegePriority`, `AntiStructure`, `AntiMass`, `AntiArmor`, `MoralePressure`

---

## 6. Faction Schema

```yaml
faction:
  id: republic
  theme: starwars
  archetype: order
  display_name: Galactic Republic

economy:
  gather_bonus: 1.0
  upkeep_modifier: 1.1
  research_speed: 1.05

army:
  morale_style: disciplined
  unit_cap_modifier: 0.9
  elite_cost_modifier: 1.2

roster:
  line_infantry: clone_trooper
  elite_infantry: arc_trooper
  anti_armor: clone_at
  shock_unit: republic_guard
  recon: clone_scout
  heavy_vehicle: at_rt_proxy
  artillery: sp_artillery_proxy
  hero_commander: jedi_commander_proxy

buildings:
  barracks: republic_barracks
  workshop: republic_motor_pool
  artillery_foundry: republic_field_command
  tower_mg: republic_laser_tower
  heavy_defense: republic_turbolaser_proxy

visuals:
  primary_color: "#d8d8d8"
  accent_color: "#8b0000"
  projectile_pack: blaster_red_republic
  ui_skin: republic_clean

audio:
  weapon_pack: republic_blaster
  structure_pack: republic_ui
```

---

## 7. Content Production Strategy

### Level 1 - Zero-New-Model Build (MVP)
Only: renamed units, new icons, recolors, projectile/VFX swaps, stat changes, wave logic, SFX swaps, text/UI replacement. Gets a playable framework fast.

### Level 2 - Selective Model Swaps
Add: hero units, key vehicles, signature defenses, faction command buildings. Only highest-visibility pieces.

### Level 3 - Full Asset Conversion
Only after framework is proven fun. This is the difference between "released mod" and "dead dream repo."

---

## 8. Theme Requirements

### Star Wars Theme - Required Illusion Package
- Blaster projectiles
- Brighter energy muzzle VFX
- Droid-vs-clone sound identity
- Shield visuals for select units
- Command-post style UI text
- Walker/hover/repulsor approximations

### Modern Theme - Required Illusion Package
- Ballistic tracers
- Rockets/missiles
- Modern infantry names and icons
- Modern fortification names
- Drone/airstrike proxies
- Tanks/artillery identity
- Different enemy doctrine packs

---

## 9. Module Split

| Module | Responsibility |
|--------|---------------|
| `DinoWarfare.Core` | Schemas, faction definitions, balance tables, shared enums, logging, config |
| `DinoWarfare.ECS` | Entity queries, ECS patching, component mutation, spawn overrides, system hooks |
| `DinoWarfare.Assets` | Icon registry, localization, audio routing, VFX registry, prefab mapping |
| `DinoWarfare.Campaign` | Wave composition, faction matchup rules, mission-level overrides |
| `DinoWarfare.Theme.StarWars` | Republic/CIS content pack |
| `DinoWarfare.Theme.Modern` | West/Classic Enemy/Guerrilla content pack |
| `DinoWarfare.Debug` | Overlay, entity inspector, hot reload, test-spawn menus, component dumps |
