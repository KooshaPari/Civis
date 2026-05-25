# VFX System Design: Star Wars Clone Wars Pack
## Projectiles, Effects, UI Feedback & Implementation Roadmap

**Document Version**: 1.0
**Pack**: `warfare-starwars` (v0.1.0)
**Framework Target**: DINOForge ≥0.1.0
**Status**: Design Phase (v1.0 roadmap)

---

## Table of Contents

1. [Design Philosophy](#design-philosophy)
2. [Projectile VFX System](#projectile-vfx-system)
3. [Impact Effects](#impact-effects)
4. [Ability & Interaction VFX](#ability--interaction-vfx)
5. [UI Effects & Feedback](#ui-effects--feedback)
6. [Asset Integration & Addressables](#asset-integration--addressables)
7. [YAML Schema & Pack Integration](#yaml-schema--pack-integration)
8. [Implementation Roadmap](#implementation-roadmap)
9. [Community Contribution Guide](#community-contribution-guide)

---

## Design Philosophy

### Guiding Principles

**1. Low-Poly VFX Aesthetic**
All effects must match the TABS-style low-poly aesthetic of units and buildings. This means:
- Simple geometric shapes (planes, spheres, cylinders)
- Flat emissive colors with minimal shading
- High visual impact relative to triangle budget
- Particle effects using 2D sprites rather than 3D meshes where possible

**2. Faction Visual Identity**
Visual effects reinforce faction personality:
- **Republic**: Clean blue glows, organized particle patterns, high-tech aesthetic
- **CIS**: Orange-red heat signatures, chaotic swarms, industrial/mechanical feel

**3. Color Fidelity**
Projectile and effect colors match the established palette from ASSET_PIPELINE.md:
- **Republic bolts**: `#4488FF` bright blue
- **CIS bolts**: `#FF4400` red-orange
- **Lightsabers (blue)**: `#4488FF`, (green): `#44FF44`, (purple/Grievous): `#FF44FF`
- **Electrostaff**: `#FFFF44` yellow emissive
- **Explosions**: orange-yellow burst, smoke trail
- **Shield effects**: faction-color transparent glow

**4. Performance Constraints**
DINO must render hundreds of entities simultaneously. VFX must:
- Use particle systems efficiently (max 200 particles per effect)
- Reduce emission rates in distance/LOD states
- Favor 2D sprites over 3D meshes
- Support dynamic pooling and reuse

**5. Clarity Over Spectacle**
In a battle with 200+ units on screen, effects must:
- Communicate game state clearly (hit/miss, unit death, ability trigger)
- Not obscure player view of terrain or unit positions
- Use consistent timing and scale across all effects

---

## Projectile VFX System

### Overview

Projectile effects are defined in `weapons/*.yaml` by referencing projectile IDs. Each projectile has:
- **visual_prefab**: The 3D mesh or particle system in flight
- **impact_effect**: The VFX spawned on hit

Projectiles are grouped by weapon class and faction.

### Projectile Types & Definitions

#### Republic Blaster Bolts

**ID**: `rep_blaster_bolt`
**Weapon Users**: DC-15A, DC-15S, DC-17, Z-6 Rotary, DC-17m Sniper

```yaml
- id: rep_blaster_bolt
  display_name: Republic Blaster Bolt
  speed: 40.0  # units per second (fast)
  damage: 14.0  # matched to weapon, unused here (weapon damage value used)
  aoe_radius: 0.0  # no AoE for standard bolts
  visual_prefab: warfare-starwars/projectiles/rep_blaster_bolt
  impact_effect: rep_blaster_impact
```

**Visual Description**:
- **In Flight**: Thin glowing blue cylinder, 0.1 units diameter × 0.3 units length
  - Emissive material: `#4488FF` (bright blue)
  - Additive blend mode (glow effect)
  - Simple quad or cylinder mesh (8-12 tris)
  - Optional: slight blue trail particle (5-10 particles, short lifespan)

- **Mesh Spec** (Kenney or custom):
  - Quad plane with round-end UV
  - Base color: white with gradient falloff
  - Emissive map: solid blue channel
  - Smoothness: 0.0 (matte)

**Color Reference**: `#4488FF` RGB(68, 136, 255)

---

**ID**: `rep_blaster_sniper_bolt`
**Weapon Users**: DC-15x Sniper, DC-17m Sniper Attachment

```yaml
- id: rep_blaster_sniper_bolt
  display_name: Republic Sniper Blaster Bolt
  speed: 50.0  # faster than standard
  damage: 40.0
  aoe_radius: 0.0
  visual_prefab: warfare-starwars/projectiles/rep_blaster_sniper_bolt
  impact_effect: rep_blaster_sniper_impact
```

**Visual Description**:
- **In Flight**: Slightly thicker glowing cylinder, brighter blue
  - Diameter: 0.15 units
  - Trail particles: 8-12, 0.1s lifetime, spreads slightly
  - Brighter emissive: `#6688FF` (lighter blue)

---

#### CIS Blaster Bolts

**ID**: `cis_blaster_bolt`
**Weapon Users**: E-5 Rifle, Wrist Blaster, STAP Twin Blasters, Droideka Twin Blasters

```yaml
- id: cis_blaster_bolt
  display_name: CIS Blaster Bolt
  speed: 38.0  # slightly slower than Republic (mass production aesthetic)
  damage: 9.0
  aoe_radius: 0.0
  visual_prefab: warfare-starwars/projectiles/cis_blaster_bolt
  impact_effect: cis_blaster_impact
```

**Visual Description**:
- **In Flight**: Red-orange glowing cylinder
  - Emissive color: `#FF4400` (red-orange)
  - Diameter: 0.1 units
  - Trail particles: 6-8 trailing sparks, 0.08s lifetime (fewer/faster than Republic = less organized)
  - Optional: slight distortion/jitter on cylinder to suggest instability

---

**ID**: `cis_blaster_sniper_bolt`
**Weapon Users**: E-5s Sniper Rifle

```yaml
- id: cis_blaster_sniper_bolt
  display_name: CIS Sniper Blaster Bolt
  speed: 48.0
  damage: 30.0
  aoe_radius: 0.0
  visual_prefab: warfare-starwars/projectiles/cis_blaster_sniper_bolt
  impact_effect: cis_blaster_sniper_impact
```

**Visual Description**:
- **In Flight**: Thicker red-orange bolt, more intense glow
  - Diameter: 0.15 units
  - Emissive: `#FF6600` (brighter orange)
  - Trail: denser spark spray (12-15 particles)

---

#### Explosive Projectiles

**ID**: `atte_mass_driver_round`
**Weapon Users**: AT-TE Mass Driver Cannon

```yaml
- id: atte_mass_driver_round
  display_name: AT-TE Mass Driver Round
  speed: 35.0
  damage: 65.0  # high damage
  aoe_radius: 5.0  # large AoE on impact
  visual_prefab: warfare-starwars/projectiles/mass_driver_round
  impact_effect: explosion_large_blue
```

**Visual Description**:
- **In Flight**: Larger sphere, dark blue with emissive glow
  - Diameter: 0.3 units
  - Color: dark blue body `#1A3A6B` with bright blue glow `#4488FF`
  - Trail: blue smoke/energy trail (15-20 particles, medium density)
  - Mesh: sphere (96 tris - acceptable for 1 or 2 rounds in flight)

---

**ID**: `aat_laser_round`
**Weapon Users**: AAT Heavy Laser Cannon

```yaml
- id: aat_laser_round
  display_name: AAT Heavy Laser Round
  speed: 32.0
  damage: 55.0
  aoe_radius: 4.0
  visual_prefab: warfare-starwars/projectiles/aat_laser_round
  impact_effect: explosion_large_orange
```

**Visual Description**:
- **In Flight**: Sphere similar to mass driver but warmer tone
  - Color: tan/sand body `#C8A87A` with orange glow `#FF6600`
  - Trail: orange-yellow energy (15-20 particles)

---

**ID**: `dsd1_laser_bolt`
**Weapon Users**: DSD1 Laser Cannon (Dwarf Spider Droid)

```yaml
- id: dsd1_laser_bolt
  display_name: DSD1 Laser Bolt
  speed: 40.0
  damage: 35.0
  aoe_radius: 2.0  # medium AoE
  visual_prefab: warfare-starwars/projectiles/dsd1_laser_bolt
  impact_effect: explosion_medium_orange
```

**Visual Description**:
- **In Flight**: Thin orange beam-like bolt
  - Appearance: stretched hexagon or cylinder, orange core
  - Color: bright orange `#FF6600`
  - Trail: sparse spark spray (4-6 particles)

---

#### Lightsaber & Melee Projectiles

Lightsabers and electrostaff are melee weapons, so no projectile visuals needed. However, if implemented as ranged or energy-throw mechanics:

**ID**: `lightsaber_blue_beam`
**Weapon Users**: Lightsaber (Jedi)

```yaml
- id: lightsaber_blue_beam
  display_name: Lightsaber Blue Beam
  speed: 60.0  # very fast
  damage: 50.0
  aoe_radius: 0.0
  visual_prefab: warfare-starwars/projectiles/lightsaber_blue_beam
  impact_effect: lightsaber_impact_blue
```

**Visual Description**:
- **In Flight**: Glowing blue blade or energy wave
  - Appearance: elongated hexagon or thin blade shape
  - Color: bright blue `#4488FF` with white core
  - Trail: blue energy distortion (8-10 particles, concentrated)
  - Emissive intensity: high (2.0+)

---

**ID**: `lightsaber_green_beam`
**Weapon Users**: Yoda variant (if implemented)

```yaml
- id: lightsaber_green_beam
  display_name: Lightsaber Green Beam
  speed: 60.0
  damage: 50.0
  aoe_radius: 0.0
  visual_prefab: warfare-starwars/projectiles/lightsaber_green_beam
  impact_effect: lightsaber_impact_green
```

**Visual Description**:
- **In Flight**: Green glowing blade
  - Color: bright green `#44FF44` with white core

---

**ID**: `grievous_purple_beam`
**Weapon Users**: General Grievous Lightsabers

```yaml
- id: grievous_purple_beam
  display_name: Grievous Purple Lightsaber
  speed: 55.0  # slightly slower to differentiate
  damage: 60.0
  aoe_radius: 0.5  # slight AoE
  visual_prefab: warfare-starwars/projectiles/lightsaber_purple_beam
  impact_effect: lightsaber_impact_purple
```

**Visual Description**:
- **In Flight**: Purple glowing blade (Grievous trophy color)
  - Color: purple `#FF44FF` with white core

---

**ID**: `electrostaff_bolt`
**Weapon Users**: Electrostaff (Magna Guard, Grievous Guards)

```yaml
- id: electrostaff_bolt
  display_name: Electrostaff Bolt
  speed: 42.0
  damage: 30.0
  aoe_radius: 0.0
  visual_prefab: warfare-starwars/projectiles/electrostaff_bolt
  impact_effect: electrostaff_impact
```

**Visual Description**:
- **In Flight**: Yellow-white crackling energy bolt
  - Color: yellow `#FFFF44` with white corona
  - Trail: electric arc particles (10-12, animated crackle effect)
  - Effect: slight jitter/distortion to simulate electrical instability

---

### Projectile Mesh Specifications

All projectile meshes follow these constraints:

| Type | Budget | Material | Texture |
|------|--------|----------|---------|
| Blaster bolt (plane) | 8-12 tris | Emissive unlit | White + emissive channel |
| Blaster bolt (cylinder) | 20-30 tris | Emissive unlit | White gradient + emissive |
| Explosive round (sphere) | 96 tris | Lit + emissive | Tan/blue base + emissive glow |
| Lightsaber blade | 16-24 tris | Emissive unlit | White core + emissive edge |
| Energy effect (sprite) | 2 tris | Additive particle | 2D sprite PNG (power-of-2) |

**Material Properties**:
- **Albedo/Base Color**: faction-appropriate, matte (`#FFFFFF` for bolts, faction color for rounds)
- **Emissive**: faction color (blue/red/yellow), intensity 1.5-2.5 (visible glow)
- **Metallic**: 0.0 (energy effects are not metallic)
- **Smoothness**: 0.0-0.1 (very matte)
- **Blend Mode**: Opaque for meshes, Additive for particles (glowing trail effect)

---

## Impact Effects

### Impact Effect Categories

Impact effects are triggered when projectiles hit targets or terrain. They must:
1. Communicate damage (visual feedback)
2. Differentiate hit type (unit vs building)
3. Match projectile color/faction aesthetic
4. NOT obscure target unit or block vision

### Blaster Impact Effects

**ID**: `rep_blaster_impact`
**Triggered By**: `rep_blaster_bolt`

```yaml
impact_effect: rep_blaster_impact
vfx_spec:
  type: particle_burst
  particle_system:
    prefab: warfare-starwars/vfx/rep_blaster_impact
    duration: 0.3  # Effect plays for 0.3 seconds then despawns
    emission_rate: 100  # particles per second
    max_particles: 50
    particle_lifetime: 0.25
    particle_size: 0.05-0.15 (random range)
  colors:
    primary: "#4488FF"  # bright blue
    secondary: "#6688FF"  # lighter blue
  trail: spark_burst
    direction: outward from impact point
    spread: 45 degrees cone
  audio: blaster_hit_light  # optional, see Audio Effects section
```

**Particle Description**:
- **Burst Type**: Radial spark burst
- **Particle Count**: 30-50 small glowing spheres
- **Colors**: Blue to lighter blue gradient
- **Spread**: 45-90 degree cone outward from impact
- **Lifespan**: 0.15-0.25 seconds, fade quickly
- **Movement**: Initial velocity outward (0.5-2.0 m/s), gravity drops them slightly
- **Mesh**: Small sphere or quad sprite (1-2 tris per particle)

**Visual Effect**:
Small, tight blue spark burst. Suggests energy discharge without being distracting.

---

**ID**: `cis_blaster_impact`
**Triggered By**: `cis_blaster_bolt`

```yaml
impact_effect: cis_blaster_impact
vfx_spec:
  type: particle_burst
  particle_system:
    prefab: warfare-starwars/vfx/cis_blaster_impact
    duration: 0.35
    emission_rate: 120  # slightly more aggressive
    max_particles: 50
    particle_lifetime: 0.2
  colors:
    primary: "#FF4400"  # red-orange
    secondary: "#FF6600"  # brighter orange
  trail: spark_burst_chaotic
    direction: slightly chaotic/scattered
    spread: 60 degrees (less organized than Republic)
```

**Visual Effect**:
Larger, messier orange spark burst with chaotic spread. Suggests industrial/unstable energy discharge.

---

**ID**: `rep_blaster_sniper_impact`
**Triggered By**: `rep_blaster_sniper_bolt`

```yaml
impact_effect: rep_blaster_sniper_impact
vfx_spec:
  type: particle_burst
  particle_system:
    prefab: warfare-starwars/vfx/rep_blaster_sniper_impact
    duration: 0.4
    emission_rate: 150  # larger burst
    max_particles: 75
    particle_lifetime: 0.3
  colors:
    primary: "#4488FF"
    secondary: "#6688FF"
  burst_radius: 0.8  # wider spread
```

**Visual Effect**:
Larger blue burst than standard blaster. Communicates higher damage without being overwhelming.

---

**ID**: `cis_blaster_sniper_impact`
**Triggered By**: `cis_blaster_sniper_bolt`

```yaml
impact_effect: cis_blaster_sniper_impact
vfx_spec:
  type: particle_burst
  particle_system:
    prefab: warfare-starwars/vfx/cis_blaster_sniper_impact
    duration: 0.4
    emission_rate: 160
    max_particles: 80
    particle_lifetime: 0.3
  colors:
    primary: "#FF4400"
    secondary: "#FF6600"
  burst_radius: 0.9
```

---

### Explosive Impact Effects

**ID**: `explosion_large_blue`
**Triggered By**: `atte_mass_driver_round`

```yaml
impact_effect: explosion_large_blue
vfx_spec:
  type: explosion_complex
  particle_systems:
    - name: flash
      prefab: warfare-starwars/vfx/explosion_flash
      duration: 0.05
      emission_rate: 200
      max_particles: 100
      particle_lifetime: 0.08
      colors:
        primary: "#FFFFFF"  # white hot
        secondary: "#AADDFF"  # blue-white
    - name: smoke_ring
      prefab: warfare-starwars/vfx/explosion_smoke
      duration: 0.6
      emission_rate: 50
      max_particles: 60
      particle_lifetime: 0.5
      colors:
        primary: "#CCCCCC"  # light gray
        secondary: "#4488FF"  # blue tint
    - name: debris
      prefab: warfare-starwars/vfx/explosion_debris
      duration: 0.3
      emission_rate: 30
      max_particles: 40
      particle_lifetime: 0.4
      colors:
        primary: "#AADDFF"  # blue sparks
  shockwave:
    radius_start: 0.5
    radius_end: 8.0
    duration: 0.2
    color: "#4488FF" with 50% alpha
    strength: pushes nearby particles/entities (optional physics integration)
  audio: explosion_large
```

**Effect Breakdown**:
1. **Flash** (0-0.05s): Bright white/blue flash burst, obscures initial impact area momentarily
2. **Smoke Ring** (0-0.6s): Rising smoke ring with blue tint, expands outward and up
3. **Debris** (0-0.3s): Blue sparks ejected in all directions, falls under gravity
4. **Shockwave** (0-0.2s): Optional expanding ring that pushes particles outward

**Total Duration**: 0.6s (smoke lingers longest)

---

**ID**: `explosion_large_orange`
**Triggered By**: `aat_laser_round`

```yaml
impact_effect: explosion_large_orange
vfx_spec:
  type: explosion_complex
  particle_systems:
    - name: flash
      colors:
        primary: "#FFFFFF"
        secondary: "#FFDD88"  # yellow-white
    - name: smoke_ring
      colors:
        primary: "#CCCCCC"
        secondary: "#FF6600"  # orange tint
    - name: debris
      colors:
        primary: "#FFDD88"  # yellow sparks
  shockwave:
    color: "#FF6600" with 50% alpha
```

**Visual Effect**: Same structure as blue explosion but with orange/yellow coloration.

---

**ID**: `explosion_medium_orange`
**Triggered By**: `dsd1_laser_bolt`

```yaml
impact_effect: explosion_medium_orange
vfx_spec:
  type: explosion_simple
  particle_systems:
    - name: flash
      duration: 0.08
      emission_rate: 150
      max_particles: 60
    - name: smoke
      duration: 0.4
      emission_rate: 40
      max_particles: 40
      particle_lifetime: 0.35
    - name: debris
      duration: 0.25
      emission_rate: 20
      max_particles: 25
  colors:
    flash: "#FFDD88"
    smoke: "#FF6600"
    debris: "#FF8800"
```

**Visual Effect**: Smaller explosion than AAT/mass driver. Suitable for medium-damage weapons.

---

### Lightsaber Impact Effects

**ID**: `lightsaber_impact_blue`
**Triggered By**: `lightsaber_blue_beam`

```yaml
impact_effect: lightsaber_impact_blue
vfx_spec:
  type: energy_impact_ring
  particle_systems:
    - name: impact_ring
      prefab: warfare-starwars/vfx/lightsaber_impact_ring
      duration: 0.2
      emission_rate: 100
      max_particles: 60
      particle_lifetime: 0.2
      colors:
        primary: "#4488FF"  # bright blue
        secondary: "#AADDFF"  # light blue
    - name: slash_sparks
      duration: 0.15
      emission_rate: 80
      max_particles: 40
      particle_lifetime: 0.15
      colors:
        primary: "#4488FF"
  audio: lightsaber_hit
```

**Visual Effect**:
- **Impact Ring**: Expanding circle of blue energy, suggests energy discharge
- **Sparks**: Short-lived blue sparks fanning outward
- **No explosion**: Melee hit is precise and concentrated, not wide-area

---

**ID**: `lightsaber_impact_green`
**Triggered By**: `lightsaber_green_beam`

```yaml
impact_effect: lightsaber_impact_green
vfx_spec:
  type: energy_impact_ring
  colors:
    primary: "#44FF44"  # bright green
    secondary: "#88FF88"  # light green
```

---

**ID**: `lightsaber_impact_purple`
**Triggered By**: `grievous_purple_beam`

```yaml
impact_effect: lightsaber_impact_purple
vfx_spec:
  type: energy_impact_ring
  particle_systems:
    - name: impact_ring
      duration: 0.25  # slightly longer
      max_particles: 80
      colors:
        primary: "#FF44FF"  # purple
        secondary: "#FF88FF"  # light purple
    - name: slash_sparks
      duration: 0.2
      max_particles: 50
  audio: lightsaber_hit_heavy
```

---

**ID**: `electrostaff_impact`
**Triggered By**: `electrostaff_bolt`

```yaml
impact_effect: electrostaff_impact
vfx_spec:
  type: energy_impact_electric
  particle_systems:
    - name: electrical_discharge
      prefab: warfare-starwars/vfx/electrostaff_discharge
      duration: 0.3
      emission_rate: 120
      max_particles: 70
      colors:
        primary: "#FFFF44"  # yellow
        secondary: "#FFFFFF"  # white
    - name: electric_arcs
      duration: 0.2
      emission_rate: 50
      max_particles: 30
      colors:
        primary: "#FFFF44"
  audio: electrostaff_hit
```

**Visual Effect**:
- **Electrical Discharge**: Yellow-white spark burst with crackling appearance
- **Electric Arcs**: Thin glowing arcs extending outward (chain-like effect)
- Suggests stun/electrical damage

---

## Ability & Interaction VFX

### Jedi Force Abilities (Future Implementation)

These effects are placeholders for v1.1+ when ability systems are implemented.

**ID**: `jedi_force_push`
**Triggered By**: Jedi Knight's Force Push ability

```yaml
ability_vfx:
  id: jedi_force_push
  type: directional_wave
  description: Blue-white energy wave emanating from Jedi
  particle_system:
    prefab: warfare-starwars/vfx/force_push_wave
    duration: 0.4
    emission_rate: 200 (at wave origin)
    max_particles: 100
    particle_lifetime: 0.3
    colors:
      primary: "#AADDFF"  # light blue
      secondary: "#FFFFFF"  # white core
    shape: expanding hemisphere
    radius_start: 0.5
    radius_end: 5.0
    height: 3.0
  audio: force_push_whoosh
```

**Visual Effect**:
Expanding blue hemisphere of energy bursting outward from Jedi unit. Particles stream radially outward. High impact without obscuring units behind.

---

**ID**: `jedi_force_pull`
**Triggered By**: Jedi Knight's Force Pull ability

```yaml
ability_vfx:
  id: jedi_force_pull
  type: convergent_field
  description: Blue energy field drawing units inward
  particle_system:
    prefab: warfare-starwars/vfx/force_pull_field
    duration: 0.5
    emission_rate: 150
    max_particles: 80
    colors:
      primary: "#4488FF"  # bright blue
      secondary: "#AADDFF"
    shape: converging spirals
    origin_radius: 4.0
    target_radius: 1.0
    height: 2.5
  audio: force_pull_hum
```

**Visual Effect**:
Spiral of blue particles converging toward Jedi unit. Suggests pulling force without being overwhelming.

---

**ID**: `jedi_lightsaber_whirl`
**Triggered By**: Jedi's Lightsaber Whirl ability (if implemented)

```yaml
ability_vfx:
  id: jedi_lightsaber_whirl
  type: spinning_ring
  description: Rotating lightsaber arc
  particle_system:
    prefab: warfare-starwars/vfx/lightsaber_whirl_ring
    duration: 0.6
    emission_rate: 250
    max_particles: 120
    particle_lifetime: 0.4
    colors:
      primary: "#4488FF"  # blue blade
      secondary: "#FFFFFF"  # white core
    radius: 2.0
    rotation_speed: 360 degrees/second
  audio: lightsaber_whirl_hum
```

---

### Droideka Shield (Future Implementation)

**ID**: `droideka_shield_deploy`
**Triggered By**: Droideka entering shield mode

```yaml
ability_vfx:
  id: droideka_shield_deploy
  type: dome_expansion
  description: Blue shield bubble expanding around Droideka
  particle_system:
    prefab: warfare-starwars/vfx/droideka_shield_ring
    duration: 0.3
    emission_rate: 180
    max_particles: 100
    particle_lifetime: 0.25
    colors:
      primary: "#4488FF"  # blue
      secondary: "#AADDFF"  # lighter blue
    radius_start: 0.5
    radius_end: 3.0
  persistent_dome:
    prefab: warfare-starwars/vfx/droideka_shield_bubble
    material: transparent blue @ 20% alpha
    radius: 3.0
    color: "#4488FF"
    duration: ability_active
  audio: shield_activate_hum
```

**Visual Components**:
1. **Deployment Ring** (0-0.3s): Expanding circle of blue particles marking shield activation
2. **Persistent Dome** (0.3s-end): Semi-transparent blue sphere around Droideka, visible while active
3. **Audio**: Humming sound while shield is active

---

## UI Effects & Feedback

### Faction Color Scheme in UI

**Republic Elements**:
- Primary color: `#1A3A6B` (deep navy blue)
- Accent color: `#4488FF` (bright blue)
- Health bar: blue gradient
- Selection highlight: blue glow
- Damage numbers: blue text
- Buff icons: blue border/glow

**CIS Elements**:
- Primary color: `#5C3D1E` (dark brown/tan)
- Accent color: `#FF4400` (red-orange)
- Health bar: orange-red gradient
- Selection highlight: orange glow
- Damage numbers: orange text
- Debuff icons: orange/red border

### Health Bar Feedback

**Mechanic**: Health bars change color as units take damage, with optional visual "crack" effect.

```yaml
ui_effect: health_bar_feedback
type: progressive_color_shift
colors:
  healthy: "#00FF00"  # green (100%)
  damaged: "#FFFF00"  # yellow (50-99%)
  critical: "#FF0000"  # red (0-49%)
animation:
  duration: 0.3  # smooth color transition
  ease_function: linear
optional_crack_effect:
  type: screen_space_overlay
  trigger: damage_taken > 20
  duration: 0.15
  intensity: damage_amount / max_health
  audio: crack_sound (optional)
```

---

### Damage Numbers

**Mechanic**: Floating damage text pops up above hit unit, with faction color.

```yaml
ui_effect: damage_number_popup
type: floating_text
prefab: warfare-starwars/ui/damage_number
text_content: "{damage_amount}"
colors:
  republic: "#4488FF"  # blue
  cis: "#FF4400"  # orange
position:
  spawn: above hit unit center + random offset (±0.3 units)
  target: move upward 1.5 units over duration
animation:
  duration: 1.0
  ease_out: exponential (slows fade)
  fade_start: 0.5s (halfway through duration)
optional_critical_hit:
  multiplier_text: "CRITICAL!"
  color: "#FFFF44"  # yellow
  scale: 1.5x normal
  spin: 360 degrees rotation
```

---

### Selection Highlight

**Mechanic**: Selected unit is outlined or glowing with faction color.

```yaml
ui_effect: unit_selection_highlight
type: outline_and_glow
prefab: warfare-starwars/ui/selection_highlight
colors:
  republic: "#4488FF"  # blue
  cis: "#FF4400"  # orange
outline_style:
  thickness: 0.02 units
  glow_radius: 0.1 units
animation:
  pulse_speed: 1 Hz (pulse once per second)
  pulse_intensity: ±30% brightness
behavior:
  activate_on_click: unit becomes selected
  deactivate_on_unclick: unit loses selection
```

---

### Ability Ready / Cooldown Indicator

**Mechanic**: Glowing aura around unit when ability is ready; dimmed/grayed out during cooldown.

```yaml
ui_effect: ability_readiness_indicator
type: aura_and_icon_feedback
prefab: warfare-starwars/ui/ability_indicator
states:
  ready:
    aura: glowing (faction color)
    intensity: full brightness
    animation: slow pulse (0.5 Hz)
    audio: ability_ready_chime (optional)
  cooldown:
    aura: dimmed (50% brightness)
    icon_tint: grayscale or faction color @ 30% alpha
    progress_ring: circular timer arc (0-100%)
    duration: cooldown_remaining_seconds
  charging:
    aura: flashing (high frequency, 3 Hz)
    color: shifts toward white during charge buildup
    progress_bar: linear meter (0-100% ability power)
```

---

### Building Construction / Destruction

**Construction Complete** (v1.1+):
```yaml
ui_effect: building_constructed
type: sparkle_and_pulse
particle_system:
  prefab: warfare-starwars/vfx/construction_sparkles
  duration: 0.5
  emission_rate: 200
  max_particles: 100
  colors:
    republic: "#4488FF"
    cis: "#FF4400"
  spawn_points: around building perimeter
audio: construction_complete_chime
```

**Building Destroyed**:
```yaml
ui_effect: building_destroyed
type: explosion_and_collapse
particle_systems:
  - explosion_burst (primary faction color)
  - smoke_cloud (gray, slow rise)
  - secondary_sparks (2-3 small bursts at structure weak points)
  - dust_settle (fine particles drifting to ground over 1-2 seconds)
animation:
  structural_collapse: building mesh falls/crumbles (shader-based or mesh swap)
  duration: 1.0
audio: building_collapse_rumble, secondary_impact_cracks
```

---

## Asset Integration & Addressables

### Addressables Manifest Entry

All VFX assets must be registered in `packs/warfare-starwars/assets/manifest.yaml`:

```yaml
vfx_effects:
  # Projectile meshes
  - id: warfare-starwars/projectiles/rep_blaster_bolt
    type: projectile_mesh
    source: assets/meshes/projectiles/rep_blaster_bolt.fbx
    materials:
      - assets/materials/rep_blaster_material.mat
    shader: Universal Render Pipeline/Lit
    placeholder: true  # until real asset committed

  # Particle system prefabs
  - id: warfare-starwars/vfx/rep_blaster_impact
    type: particle_system
    source: assets/prefabs/vfx/rep_blaster_impact.prefab
    particle_count_max: 50
    placeholder: true

  # UI effect prefabs
  - id: warfare-starwars/ui/damage_number
    type: ui_prefab
    source: assets/prefabs/ui/damage_number.prefab
    placeholder: true

  # Lightsaber mesh variants
  - id: warfare-starwars/projectiles/lightsaber_blue_beam
    type: projectile_mesh
    source: assets/meshes/projectiles/lightsaber_blue_beam.fbx
    placeholder: true

  # Shield dome (persistent effect)
  - id: warfare-starwars/vfx/droideka_shield_bubble
    type: persistent_effect
    source: assets/prefabs/vfx/droideka_shield_bubble.prefab
    placeholder: true
```

### Addressables Address Naming Convention

All VFX Addressables addresses must follow this pattern:

```
warfare-starwars/projectiles/{projectile_id}
warfare-starwars/vfx/{effect_id}
warfare-starwars/ui/{ui_effect_id}
```

Examples:
- `warfare-starwars/projectiles/rep_blaster_bolt`
- `warfare-starwars/vfx/rep_blaster_impact`
- `warfare-starwars/ui/damage_number`

### Loading VFX at Runtime

The `AddressablesCatalog` service in the SDK loads these dynamically:

```csharp
// Example pseudocode
var projectilePrefab = await Addressables.LoadAssetAsync<GameObject>(
    "warfare-starwars/projectiles/rep_blaster_bolt"
).Task;
var impactPrefabTask = Addressables.LoadAssetAsync<ParticleSystem>(
    "warfare-starwars/vfx/rep_blaster_impact"
);
```

---

## YAML Schema & Pack Integration

### Projectile YAML File Structure

**File**: `packs/warfare-starwars/projectiles/blasters.yaml`

```yaml
# Star Wars - Blaster & Energy Projectiles

# Republic Blaster Bolts
- id: rep_blaster_bolt
  display_name: Republic Blaster Bolt
  speed: 40.0
  damage: 14.0
  aoe_radius: 0.0
  visual_prefab: warfare-starwars/projectiles/rep_blaster_bolt
  impact_effect: rep_blaster_impact

- id: rep_blaster_sniper_bolt
  display_name: Republic Sniper Blaster Bolt
  speed: 50.0
  damage: 40.0
  aoe_radius: 0.0
  visual_prefab: warfare-starwars/projectiles/rep_blaster_sniper_bolt
  impact_effect: rep_blaster_sniper_impact

# CIS Blaster Bolts
- id: cis_blaster_bolt
  display_name: CIS Blaster Bolt
  speed: 38.0
  damage: 9.0
  aoe_radius: 0.0
  visual_prefab: warfare-starwars/projectiles/cis_blaster_bolt
  impact_effect: cis_blaster_impact

- id: cis_blaster_sniper_bolt
  display_name: CIS Sniper Blaster Bolt
  speed: 48.0
  damage: 30.0
  aoe_radius: 0.0
  visual_prefab: warfare-starwars/projectiles/cis_blaster_sniper_bolt
  impact_effect: cis_blaster_sniper_impact

# Lightsaber Projectiles
- id: lightsaber_blue_beam
  display_name: Lightsaber Blue Beam
  speed: 60.0
  damage: 50.0
  aoe_radius: 0.0
  visual_prefab: warfare-starwars/projectiles/lightsaber_blue_beam
  impact_effect: lightsaber_impact_blue

- id: lightsaber_green_beam
  display_name: Lightsaber Green Beam
  speed: 60.0
  damage: 50.0
  aoe_radius: 0.0
  visual_prefab: warfare-starwars/projectiles/lightsaber_green_beam
  impact_effect: lightsaber_impact_green

- id: grievous_purple_beam
  display_name: Grievous Purple Lightsaber
  speed: 55.0
  damage: 60.0
  aoe_radius: 0.5
  visual_prefab: warfare-starwars/projectiles/lightsaber_purple_beam
  impact_effect: lightsaber_impact_purple

- id: electrostaff_bolt
  display_name: Electrostaff Bolt
  speed: 42.0
  damage: 30.0
  aoe_radius: 0.0
  visual_prefab: warfare-starwars/projectiles/electrostaff_bolt
  impact_effect: electrostaff_impact

# Explosive Projectiles
- id: atte_mass_driver_round
  display_name: AT-TE Mass Driver Round
  speed: 35.0
  damage: 65.0
  aoe_radius: 5.0
  visual_prefab: warfare-starwars/projectiles/mass_driver_round
  impact_effect: explosion_large_blue

- id: aat_laser_round
  display_name: AAT Heavy Laser Round
  speed: 32.0
  damage: 55.0
  aoe_radius: 4.0
  visual_prefab: warfare-starwars/projectiles/aat_laser_round
  impact_effect: explosion_large_orange

- id: dsd1_laser_bolt
  display_name: DSD1 Laser Bolt
  speed: 40.0
  damage: 35.0
  aoe_radius: 2.0
  visual_prefab: warfare-starwars/projectiles/dsd1_laser_bolt
  impact_effect: explosion_medium_orange
```

### Weapon YAML Integration

**File**: `packs/warfare-starwars/weapons/blasters.yaml` (updated)

Each weapon references a projectile ID:

```yaml
- id: dc15a_blaster
  display_name: DC-15A Blaster Rifle
  weapon_class: rifle
  damage_type: energy
  base_damage: 14.0
  range: 20.0
  rate_of_fire: 2.5
  aoe_radius: 0.0
  projectile_id: rep_blaster_bolt  # <-- references projectile VFX

- id: dc15x_sniper
  display_name: DC-15x Sniper Rifle
  weapon_class: rifle
  damage_type: energy
  base_damage: 40.0
  range: 40.0
  rate_of_fire: 0.5
  aoe_radius: 0.0
  projectile_id: rep_blaster_sniper_bolt  # <-- projectile with sniper glow

- id: lightsaber
  display_name: Lightsaber
  weapon_class: melee
  damage_type: energy
  base_damage: 50.0
  range: 2.0
  rate_of_fire: 2.5
  aoe_radius: 0.0
  projectile_id: lightsaber_blue_beam  # <-- ranged beam variant for Force throw
```

### VFX Effects Definition File (New)

**File**: `packs/warfare-starwars/vfx/effects.yaml` (new, v1.1+)

```yaml
# VFX Effects Registry - Reusable effect definitions

impact_effects:
  - id: rep_blaster_impact
    display_name: Republic Blaster Impact
    prefab: warfare-starwars/vfx/rep_blaster_impact
    duration: 0.3
    particle_count: 50
    primary_color: "#4488FF"

  - id: cis_blaster_impact
    display_name: CIS Blaster Impact
    prefab: warfare-starwars/vfx/cis_blaster_impact
    duration: 0.35
    particle_count: 50
    primary_color: "#FF4400"

  - id: explosion_large_blue
    display_name: Large Blue Explosion
    prefab: warfare-starwars/vfx/explosion_large_blue
    duration: 0.6
    particle_count: 200
    primary_color: "#4488FF"
    has_shockwave: true

  - id: explosion_large_orange
    display_name: Large Orange Explosion
    prefab: warfare-starwars/vfx/explosion_large_orange
    duration: 0.6
    particle_count: 200
    primary_color: "#FF6600"
    has_shockwave: true

  - id: lightsaber_impact_blue
    display_name: Blue Lightsaber Impact
    prefab: warfare-starwars/vfx/lightsaber_impact_blue
    duration: 0.2
    particle_count: 100
    primary_color: "#4488FF"
    audio_effect: lightsaber_hit

ui_effects:
  - id: damage_number
    display_name: Damage Number Popup
    prefab: warfare-starwars/ui/damage_number
    duration: 1.0
    colors:
      republic: "#4488FF"
      cis: "#FF4400"

  - id: health_bar
    display_name: Unit Health Bar
    prefab: warfare-starwars/ui/health_bar
    colors:
      healthy: "#00FF00"
      damaged: "#FFFF00"
      critical: "#FF0000"

ability_effects:
  - id: jedi_force_push
    display_name: Jedi Force Push
    prefab: warfare-starwars/vfx/force_push_wave
    duration: 0.4
    particle_count: 100
    primary_color: "#AADDFF"
    requires_ability_system: true

  - id: droideka_shield_deploy
    display_name: Droideka Shield Deploy
    prefab: warfare-starwars/vfx/droideka_shield_ring
    duration: 0.3
    persistent_duration: ability_active
    primary_color: "#4488FF"
    requires_ability_system: true
```

---

## Implementation Roadmap

### Version 1.0 (Current - Placeholder Phase)

**Scope**: Schema definition, concept art, asset sourcing list
**Status**: COMPLETE

**Deliverables**:
- [x] VFX_SYSTEM_DESIGN.md (this document)
- [x] Projectile YAML schema (projectile.schema.json)
- [x] Weapon-to-projectile linkage (weapons/blasters.yaml)
- [x] Color palette reference (ASSET_PIPELINE.md)
- [x] Impact effect specifications (text descriptions)
- [x] UI effect framework (mockups)

**Testing**: Validation against schemas via PackCompiler

---

### Version 1.1 (VFX Implementation Phase - v1.1.0+)

**Estimated Timeline**: 2-4 weeks (community contribution-dependent)

**Scope**: Create, import, test actual VFX assets

#### 1.1.1 Projectile Meshes (Week 1)

**Tasks**:
- [ ] Create base blaster bolt mesh (plane + cylinder variants)
  - Use Blender with Kenney assets as base
  - Target: 8-12 tris, emissive material
  - Test in Unity 2021.3 with Addressables

- [ ] Create lightsaber blade meshes (blue, green, purple)
  - Target: 16-24 tris per variant
  - White core + emissive edge material

- [ ] Create explosive round meshes (sphere variants for mass driver, AAT, DSD1)
  - Target: 96 tris (single shared sphere)
  - Tan/blue/orange colorization via material

- [ ] Create electrostaff bolt mesh
  - Target: 12-16 tris
  - Crackling energy appearance (shader or trail)

**Assets to Create**: 8 projectile prefabs

**Validation**:
```bash
dotnet run --project src/Tools/DumpTools -- \
  verify-asset-manifest packs/warfare-starwars/assets/manifest.yaml
```

---

#### 1.1.2 Particle Systems (Week 2)

**Tasks**:
- [ ] Create blaster impact particle systems (rep + cis variants)
  - Radial spark burst, faction colors
  - Test emission rates, particle lifetime

- [ ] Create explosive impact particle systems (large blue, large orange, medium orange)
  - Flash + smoke ring + debris combo
  - Test shockwave effect (if physics integration available)

- [ ] Create lightsaber impact particle systems (blue, green, purple)
  - Impact ring + slash sparks
  - High visual polish for hero weapons

- [ ] Create electrostaff impact particle system
  - Electrical discharge + arc effects

**Assets to Create**: 10 particle system prefabs

**Validation**: Test in DINO sandbox to ensure visual clarity at battle scale

---

#### 1.1.3 UI Effect Prefabs (Week 2-3)

**Tasks**:
- [ ] Create damage number popup prefab
  - Floating text with faction color, fade-out animation
  - Support critical hit scale/spin variant

- [ ] Create health bar UI panel
  - Color shift from green → yellow → red
  - Optional "crack" damage visual

- [ ] Create selection highlight (outline + glow)
  - Pulse animation
  - Faction color variants

- [ ] Create ability readiness indicator
  - Aura glow + cooldown progress ring

**Assets to Create**: 4 UI prefab variants

---

#### 1.1.4 Bundle Assembly & Testing (Week 3-4)

**Tasks**:
- [ ] Add all projectile & VFX prefabs to Addressables group `warfare-starwars`

- [ ] Build bundle: `dotnet run --project src/Tools/PackCompiler -- build packs/warfare-starwars`

- [ ] Update manifest.yaml: change `placeholder: false` for all committed assets

- [ ] Test in DINO:
  - Fire clone trooper blaster → verify blue bolt + impact glow
  - Fire battle droid blaster → verify orange bolt + spark burst
  - Fire AT-TE cannon → verify explosion with shockwave
  - Select unit → verify blue/orange highlight
  - Take damage → verify health bar color shift + damage number

- [ ] Screenshot gameplay: each weapon firing, explosions, selection UI

- [ ] Create CONTRIBUTE.md: step-by-step guide for community VFX artists

**Validation**:
```bash
# Validate pack structure
dotnet run --project src/Tools/PackCompiler -- validate packs/warfare-starwars

# Build and test
dotnet test src/DINOForge.sln --filter "Category=VFX"
```

---

### Version 1.2 (Ability VFX Phase - v1.2.0+)

**Estimated Timeline**: 2-3 weeks (depends on ability system implementation in Warfare domain)

**Scope**: Ability-triggered VFX (Jedi Force powers, Droideka shield, special attacks)

#### 1.2.1 Jedi Ability Effects

**Tasks**:
- [ ] Implement Force Push wave VFX
  - Expanding hemisphere of blue particles
  - Integrate with Jedi unit ability trigger

- [ ] Implement Force Pull field VFX
  - Converging spiral particles
  - Test with unit pull simulation

- [ ] Implement Lightsaber Whirl AoE VFX
  - Rotating blade ring around Jedi
  - High visual polish for hero abilities

**Assets to Create**: 3 ability effect prefabs

---

#### 1.2.2 CIS Ability Effects

**Tasks**:
- [ ] Implement Droideka Shield Deploy animation
  - Expanding ring → persistent dome effect
  - Integrate with Droideka unit toggle

- [ ] Implement B2 Super Battle Droid self-heal glow
  - Green energy aura

- [ ] Implement AAT Plasma Blast charge effect
  - Buildup glow, charge aura

**Assets to Create**: 3 ability effect prefabs

---

#### 1.2.3 Audio Integration (Optional v1.2+)

**Tasks** (if audio is in scope):
- [ ] Create or source impact sounds: blaster hit, explosion, lightsaber clash
- [ ] Integrate into impact_effect definitions
- [ ] Test audio timing with particle effect duration

---

### Version 1.3+ (Polish & Expansion)

**Scope**: Stretch goals, community contributions, cosmetic variants

- [ ] Additional Jedi lightsaber colors (blue, green, orange, yellow)
- [ ] Droideka shield hit feedback (shield cracks, electrical damage overlay)
- [ ] Unit death effects (clone trooper electrocution, droid disintegration)
- [ ] Building construction/destruction effects
- [ ] Faction-specific ambient effects (battlefield atmosphere)
- [ ] Cinematic slow-motion hit effect (optional)

---

## Community Contribution Guide

### For VFX Artists: How to Contribute

#### Prerequisites

1. **Software** (all free):
   - Blender 3.x (mesh creation): https://blender.org
   - Unity 2021.3.45f2 (prefab assembly): https://unity3d.com
   - Git (version control): https://git-scm.com

2. **Knowledge** (optional but helpful):
   - Basic Blender modeling (humanoid proportions, simple geometry)
   - Unity particle system (emission, lifetime, material properties)
   - Addressables workflow in Unity
   - YAML syntax

#### Workflow: Creating a Blaster Bolt VFX

**Step 1: Create Projectile Mesh (15 minutes)**

```bash
# 1. Download Kenney Space Kit or create a simple quad plane
#    File > New > General > UV Sphere (for rounds)
#                or Plane (for bolts)

# 2. Scale to 0.1 units diameter (test in DINO viewport)
# 3. Create white + blue emissive material
#    Shader: Unlit
#    Base Color: #FFFFFF
#    Emission: #4488FF, Intensity 2.0
# 4. UV unwrap (Smart UV Project)
# 5. Export as FBX to: assets/meshes/projectiles/rep_blaster_bolt.fbx
# 6. File > Export > FBX > [check "Selected Objects"]
```

**Step 2: Import into Unity (30 minutes)**

```bash
# 1. Create fresh Unity 2021.3.45f2 project (3D URP)
# 2. Window > Asset Management > Addressables > Settings
#    Create default settings (if not present)
# 3. Copy FBX into Assets/StarWars/Meshes/Projectiles/
# 4. Select FBX Inspector > Import Settings:
#    - Scale Factor: 1.0
#    - Mesh Compression: Low
#    - Read/Write: OFF
# 5. Create URP/Lit material with:
#    - Base Map: white texture
#    - Emission: blue @ 2.0 intensity
# 6. Drag FBX into scene, assign material
# 7. Export as prefab: Assets/StarWars/Prefabs/Projectiles/rep_blaster_bolt.prefab
```

**Step 3: Test in DINO (15 minutes)**

```bash
# 1. Copy prefab to: packs/warfare-starwars/assets/prefabs/
# 2. Update manifest.yaml:
#    - id: warfare-starwars/projectiles/rep_blaster_bolt
#      placeholder: false
# 3. Build bundle:
#    dotnet run --project src/Tools/PackCompiler -- build packs/warfare-starwars
# 4. Launch DINO, fire clone trooper weapon, verify bolt appearance
```

---

#### Workflow: Creating Impact Particle System (45 minutes)

**Step 1: Set Up Particle System Prefab (30 minutes)**

```bash
# 1. In Unity, right-click > Create > GameObject (empty)
# 2. Add component > Particle System
# 3. Configure:
#    - Duration: 0.3
#    - Looping: OFF
#    - Start Lifetime: 0.25
#    - Start Speed: 2.0
#    - Start Size: random 0.05-0.15
#    - Max Particles: 50
# 4. Emission:
#    - Rate over Time: 100
# 5. Color over Lifetime:
#    - Blue (#4488FF) → Transparent blue (#4488FF @ 0% alpha)
# 6. Renderer:
#    - Material: default particle material (white)
#    - Render Mode: Billboard
# 7. Save as prefab: Assets/StarWars/Prefabs/VFX/rep_blaster_impact.prefab
```

**Step 2: Test & Iterate (15 minutes)**

```bash
# 1. Play scene, spawn instance of particle prefab at cursor
# 2. Adjust:
#    - Spread angle if too narrow/wide
#    - Emission rate if too sparse/dense
#    - Lifetime if effect lingers too long
# 3. Save changes to prefab
```

---

#### Checklist: Before Submitting PR

- [ ] Asset is CC0 licensed (or CC-BY with attribution in ATTRIBUTION.md)
- [ ] Mesh poly count is within budget (see ASSET_PIPELINE.md table)
- [ ] Material uses correct faction color from palette
- [ ] Prefab is named matching the ID (e.g., `rep_blaster_bolt.prefab`)
- [ ] Addressables address matches prefab name
- [ ] `placeholder: true` changed to `false` in manifest.yaml
- [ ] Bundle builds successfully: `dotnet run --project src/Tools/PackCompiler -- build packs/warfare-starwars`
- [ ] Pack validates: `dotnet run --project src/Tools/PackCompiler -- validate packs/warfare-starwars`
- [ ] Tested in DINO (screenshot provided in PR description)
- [ ] No binary files committed to git (only FBX sources and metadata)

---

#### Community Priority List (Ranked by Impact)

**Tier 1 - Highest Visibility** (Start here for max impact):
1. Republic blaster bolt + impact (most-fired projectile)
2. CIS blaster bolt + impact
3. Large explosion (AT-TE & AAT weapons)
4. Lightsaber blue beam + impact
5. Lightsaber purple beam (Grievous)

**Tier 2 - Medium Visibility**:
6. Sniper blaster variants (blue, red)
7. Electrostaff bolt + impact
8. Droideka shield visual (persistent effect)

**Tier 3 - UI & Polish**:
9. Damage number popup
10. Selection highlight (outline + glow)
11. Health bar color shift animation

---

## Appendix A: Color Palette Quick Reference

### Emissive Colors (for glowing effects)

| Faction | Element | Hex | RGB | Use Case |
|---------|---------|-----|-----|----------|
| Republic | Standard Blaster | `#4488FF` | (68, 136, 255) | Bolt + impact glow |
| Republic | Sniper Blaster | `#6688FF` | (102, 136, 255) | Brighter, premium feel |
| Republic | Jedi Glow | `#AADDFF` | (170, 221, 255) | Light blue, ability effects |
| CIS | Standard Blaster | `#FF4400` | (255, 68, 0) | Bolt + impact spark |
| CIS | Heavy Blaster | `#FF6600` | (255, 102, 0) | Brighter orange, heavier |
| Lightsaber | Blue | `#4488FF` | (68, 136, 255) | Jedi blade |
| Lightsaber | Green | `#44FF44` | (68, 255, 68) | Yoda, Jedi variant |
| Lightsaber | Purple | `#FF44FF` | (255, 68, 255) | Grievous trophy colors |
| Electrostaff | Yellow | `#FFFF44` | (255, 255, 68) | Stun weapon |
| Explosion | Flash White | `#FFFFFF` | (255, 255, 255) | Initial impact |
| Explosion | Smoke Gray | `#CCCCCC` | (204, 204, 204) | Lingering cloud |
| Shield | Blue Dome | `#4488FF` @ 20% alpha | Transparent | Droideka shield |
| Shield | Red Dome | `#FF4400` @ 20% alpha | Transparent | CIS defensive bubble |

---

## Appendix B: Particle System Template (Copy & Paste)

For consistency, use this template when creating new particle systems:

```yaml
particle_system_template:
  name: "{effect_name}"
  duration: 0.3  # seconds until system stops emitting
  looping: false  # one-shot effect

  emission:
    rate_over_time: 100  # particles per second
    rate_over_distance: 0  # not distance-based

  shape:
    type: cone  # or sphere for omnidirectional
    angle: 45  # degrees, 0 = straight line
    radius: 0.1  # units

  initial_conditions:
    lifetime: 0.25  # seconds until particle disappears
    speed: 2.0  # units per second
    size: random(0.05, 0.15)  # random range
    rotation: random(0, 360)  # degrees

  forces:
    gravity: 0.5  # or 0 for weightless effects
    damping: 0.2  # velocity reduction over time

  color_over_lifetime:
    start: "#4488FF" (faction color)
    end: "#4488FF" @ 0% alpha (fade to transparent)

  size_over_lifetime:
    start: 1.0 (100% initial size)
    end: 0.8 (80% final size, slight shrink)

  renderer:
    material: Particles/Standard Unlit
    render_mode: Billboard  # always face camera
    max_particles: 50  # cap on active particles
    sorting_order: 0  # render on top of world geometry
```

---

## Appendix C: Troubleshooting Common VFX Issues

### Problem: Projectile bolt is too faint / hard to see

**Solutions**:
1. Increase emissive intensity (2.0 → 3.0)
2. Add glow post-process to camera (Bloom effect)
3. Make bolt thicker (0.1 → 0.15 units diameter)
4. Add trailing particles for motion blur effect

---

### Problem: Explosion effect obscures units underneath

**Solutions**:
1. Reduce max particle count (200 → 100)
2. Increase particle spawn height (stay above ground)
3. Reduce smoke opacity (additive blend, lower alpha)
4. Shorten effect duration (0.6s → 0.4s)

---

### Problem: Impact particles drift wrong direction (upward instead of outward)

**Solutions**:
1. Check particle system shape: should be **cone** (not sphere) for directional burst
2. Cone angle: 45-90 degrees (widen if needed)
3. Gravity: set to 0.5 (not -1.0) so particles still fall slightly

---

### Problem: Addressables address doesn't match manifest

**Symptom**: `[ERROR] warfare-starwars/projectiles/rep_blaster_bolt not found in catalog`

**Solution**:
1. In Unity Addressables window, verify address matches exactly:
   - `warfare-starwars/projectiles/rep_blaster_bolt`
2. Rebuild bundle: `Build > New Build > Default Build Script`
3. Re-export catalog JSON
4. Verify bundle file exists: `packs/warfare-starwars/assets/warfare-starwars-assets.bundle`

---

## References & External Resources

### Particle System Documentation
- Unity Particle System Manual: https://docs.unity3d.com/Manual/class-ParticleSystem.html
- Addressables Setup: https://docs.unity3d.com/Packages/com.unity.addressables@latest

### Free Asset Sources (All CC0/CC-BY)
- Kenney.nl Asset Packs: https://kenney.nl/assets
- PolyPizza: https://poly.pizza
- OpenGameArt VFX: https://opengameart.org
- Blender Documentation: https://docs.blender.org

### Color Reference Tools
- Color Picker: https://colorpicker.com
- Hex to RGB Converter: https://www.rapidtables.com/convert/color/hex-to-rgb.html

---

## Document History

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0 | 2026-03-12 | Claude Code (Haiku Agent) | Initial design document, all sections complete |

---

**End of VFX_SYSTEM_DESIGN.md**
