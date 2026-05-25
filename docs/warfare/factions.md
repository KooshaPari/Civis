# Factions

Every faction in the Warfare domain is built on one of three archetypes, themed to a setting, and governed by a doctrine.

## Faction Summary

| Faction | Theme | Archetype | Doctrine |
|---------|-------|-----------|----------|
| Republic | Star Wars | Order | `elite_discipline` |
| CIS | Star Wars | Industrial Swarm | `mechanized_attrition` |
| West | Modern | Order | `combined_arms_precision` |
| Classic West Enemy | Modern | Industrial Swarm | `massed_peer_assault` |
| Guerrilla West Enemy | Modern | Asymmetric | `raid_and_attrit` |

---

## Order Archetype

### Galactic Republic

- **Theme**: Star Wars Clone Wars
- **Fantasy**: Clone infantry line, specialist troopers, walkers, artillery, Jedi command
- **Gameplay**: High precision, strong fortified defense, expensive losses hurt, strong late game

**Economy modifiers:**
- Gather bonus: 1.0x (standard)
- Upkeep modifier: 1.1x (expensive army)
- Research speed: 1.05x (slight edge)

**Army traits:**
- Morale style: `disciplined`
- Unit cap modifier: 0.9x (fewer but stronger)
- Elite cost modifier: 1.2x (premium elites)

### West

- **Theme**: Modern Warfare
- **Fantasy**: Rifle squads, marksmen, AT teams, IFVs/MBTs, drones/airstrikes
- **Gameplay**: Flexible combined arms, strong anti-armor, premium units, high burst damage

**Doctrine**: `combined_arms_precision` — Balanced force with emphasis on combined arms coordination and precision fire.

---

## Industrial Swarm Archetype

### CIS (Confederacy of Independent Systems)

- **Theme**: Star Wars Clone Wars
- **Fantasy**: B1 flood, B2 heavy line, droideka shields, spider/tank droids
- **Gameplay**: Flood and pressure, disposable front line, oppressive economy, shield spikes

**Army traits:**
- Morale style: `mechanical` (no morale breaks)
- Higher unit cap
- Cheaper core units, expensive elites

### Classic West Enemy

- **Theme**: Modern Warfare
- **Fantasy**: Conscript mass, mechanized waves, rocket saturation, heavy fortifications
- **Gameplay**: Attritional, stronger mass, lower finesse, brutal siege, dangerous late push

**Doctrine**: `massed_peer_assault` — Overwhelm through volume. Cheaper units, larger waves, strong artillery support.

---

## Asymmetric Archetype

### Guerrilla West Enemy

- **Theme**: Modern Warfare
- **Fantasy**: Raiders, technicals, RPG teams, mortars, tunnel/cache networks
- **Gameplay**: Weak head-on, strong vs exposed economy, high harassment

**Doctrine**: `raid_and_attrit` — Avoid direct engagement, raid economy, harass with mobility, attrit through sustained pressure.

**Key traits:**
- Light units with high mobility
- Strong ambush capability
- Weak frontline durability
- Anti-structure specialists (explosives/rockets)
- "Annoying" harassment battlefield presence

---

## Faction Schema Example

```yaml
faction:
  id: republic
  theme: starwars
  archetype: order
  doctrine: elite_discipline
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

## Theme Requirements

### Star Wars — Required Illusion Package

- Blaster projectiles (red/blue energy bolts)
- Brighter energy muzzle VFX
- Droid-vs-clone sound identity
- Shield visuals for select units
- Command-post style UI text
- Walker/hover/repulsor approximations

### Modern — Required Illusion Package

- Ballistic tracers
- Rockets/missiles
- Modern infantry names and icons
- Modern fortification names
- Drone/airstrike proxies
- Tanks/artillery identity
- Different enemy doctrine packs
