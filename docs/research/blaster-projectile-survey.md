# Blaster Projectile Survey

## Scope

This note surveys how DINOForge currently handles projectiles in the runtime bridge, then proposes a modding design for themed projectile visuals and behavior:

- Star Wars: blaster bolts, faction-colored glow, fast straight-line travel, short impact bursts
- Modern: bullets, tracers, missiles, with a clear split between hitscan-like bullets, visible tracers, and homing ordnance

The goal is to let a pack replace or augment vanilla medieval projectile presentation without hardcoding one-off logic per theme.

## What DINOForge Already Exposes

### Runtime bridge

The current runtime bridge already knows about projectile data and projectile archetypes:

- [`src/Runtime/Bridge/ComponentMap.cs`](../../src/Runtime/Bridge/ComponentMap.cs) maps:
  - `Components.ProjectileDataBase` to `projectile.base`
  - `Components.RawComponents.ProjectileFlyData` to `projectile.damage`
  - `Components.RawComponents.ProjectileFlyData` to `projectile.gravity`
  - `unit.stats.damage` as an alias for `ProjectileFlyData.damage`
- [`src/Runtime/Bridge/VanillaCatalog.cs`](../../src/Runtime/Bridge/VanillaCatalog.cs) classifies projectile archetypes by component signature:
  - `Components.ProjectileDataBase`
  - `Components.RawComponents.ProjectileFlyData`
  - `Components.ProjectileMultiHitBuffer` for AoE or piercing-style projectiles
- [`src/Runtime/Bridge/ProjectileVFXSystem.cs`](../../src/Runtime/Bridge/ProjectileVFXSystem.cs) currently spawns impact VFX by scanning entities with `Components.ProjectileDataBase` and selecting a pool key by owner faction:
  - enemy owner -> `BlasterImpact_CIS`
  - player owner -> `BlasterImpact_Rep`

### Existing VFX asset layer

The runtime already has a descriptor-based VFX catalog:

- [`src/Runtime/VFX/VFXPrefabDescriptor.cs`](../../src/Runtime/VFX/VFXPrefabDescriptor.cs)
- It already contains named descriptors for:
  - `BlasterBolt_Rep`
  - `BlasterBolt_CIS`
  - `BlasterImpact_Rep`
  - `BlasterImpact_CIS`

That matters because it gives the mod a data-backed place to describe projectile presentation, instead of forcing all visual logic into code.

## What Is Missing

The current projectile VFX path is effectively impact-only and highly themed to the Star Wars prototype:

- no per-projectile trail or in-flight renderer selection
- no generic mapping from vanilla projectile archetype to themed projectile archetype
- no homing or guidance policy for missiles
- no pack-level override file that declares projectile replacement rules
- no source-of-truth for faction color, emissive intensity, trail length, or ballistic vs guided motion

So the next layer needs to separate:

1. projectile identity
2. projectile behavior
3. projectile visuals
4. projectile mapping from vanilla archetypes

## Proposed Design

### 1. Use a three-layer projectile model

#### A. Projectile archetype

Represents the gameplay semantics:

- straight ballistic bolt
- fast tracer round
- arcing shell
- homing missile
- area burst / explosive round

This layer owns:

- speed
- gravity
- damage model
- hit type
- turn rate or steering for guided ordnance
- lifetime
- collision radius

#### B. Projectile visual profile

Represents how the projectile is rendered:

- mesh or particle prefab
- emissive color
- glow strength
- trail style
- impact burst style
- scale by distance or LOD

This layer should be independent of damage numbers or faction rules.

#### C. Projectile mapping rule

Represents how a vanilla archetype gets remapped for a pack:

- `arrow` -> `blaster_bolt`
- `cannonball` -> `missile`
- `catapult_rock` -> `modern_shell`
- `siege_bolt` -> `guided_missile`

This layer is where the pack author declares the theme.

## Vanilla-To-Themed Mapping

The mapping should be explicit and pack-owned, not inferred from runtime heuristics alone.

### Medieval base game to Star Wars

| Vanilla archetype | Themed archetype | Behavior | Visual |
|---|---|---|---|
| arrow | blaster bolt | straight, very fast, no gravity | blue or red bolt, short glow trail |
| crossbow bolt | heavy blaster bolt | straight, fast, slightly larger radius | thicker bolt, stronger flash |
| cannonball | blaster impact projectile | arcing or straight depending on weapon | bright burst with heavier smoke/impact sparks |
| catapult rock | plasma lob / heavy energy orb | arcing, slower | larger glowing orb, brighter impact burst |
| siege projectile | heavy blaster or plasma cannon round | ballistic, larger lifetime | thicker trail, bigger flash |

### Medieval base game to Modern

| Vanilla archetype | Themed archetype | Behavior | Visual |
|---|---|---|---|
| arrow | bullet | straight, extremely fast, short lifetime | tracer line or small impact puff |
| crossbow bolt | rifle round | straight, very fast | brighter tracer, small muzzle-flash-style impact |
| cannonball | shell | ballistic, slower, high splash | shell tracer, dirt/smoke impact |
| catapult rock | grenade / mortar round | arcing, medium speed | visible arc, smoke trail, larger explosion |
| siege projectile | missile / rocket | guided or semi-guided | smoke plume, thrust flicker, terminal explosion |

The key design choice is that “vanilla arrow” should not map to one single modern visual. A pack must be able to choose between bullet, tracer, bolt, or missile depending on unit tier and weapon class.

## Proposed Component / System Surface

### Data components

Add a dedicated projectile visual/behavior descriptor component rather than stuffing everything into `ProjectileFlyData`:

- `ProjectileVisualProfile`
  - `VisualId`
  - `FactionColorMode`
  - `GlowIntensity`
  - `TrailPrefabId`
  - `ImpactPrefabId`
  - `TracerLength`
  - `TracerWidth`
  - `UseMotionBlur`
- `ProjectileMotionProfile`
  - `BaseSpeed`
  - `Gravity`
  - `TurnRate`
  - `HomingStrength`
  - `MaxLifetime`
  - `ArcHeightBias`
- `ProjectileMappingTag`
  - `SourceArchetypeId`
  - `ThemedArchetypeId`
  - `PackId`

If DINOForge prefers not to add new ECS components to vanilla entities, these can live as lookup records in the bridge and be resolved into runtime spawn parameters.

### Systems

#### `ProjectileOverrideResolveSystem`

Runs when projectile entities are created or when packs are loaded.

Responsibilities:

- inspect vanilla projectile archetype
- look up pack override rules
- attach resolved visual and motion profiles
- choose faction-colored impact/trail effects

#### `ProjectileTrailVFXSystem`

Spawns and updates in-flight visual trails.

Responsibilities:

- read projectile position and forward vector
- spawn/update trail prefab or line renderer
- keep glow aligned to motion
- handle per-faction tinting

#### `ProjectileHomingSystem`

Applies guided steering for missile-like projectiles.

Responsibilities:

- acquire target
- steer toward intercept point
- enforce max turn rate
- disable homing if line of sight or target validity is lost

#### `ProjectileImpactVFXSystem`

Generalizes the current `ProjectileVFXSystem`.

Responsibilities:

- choose impact prefab by projectile theme and faction
- emit sparks, burst, smoke, scorch, or energy flash
- support different impact profiles for armor, structure, and shield hits

#### `ProjectileDespawnCleanupSystem`

Ensures trails and pooled effects are reclaimed when the projectile dies.

## Recommended Behavior By Theme

### Star Wars blaster bolt

Blaster bolts should feel like short-lived luminous energy packets:

- very fast
- straight-line travel
- low or zero gravity
- strong emissive color
- narrow, bright trail
- impact burst chosen by faction

Recommended visual treatment:

- core color from faction palette
- emissive white-hot center with colored outer glow
- additive material
- brief trail ribbon or particle streak
- impact burst with 0.15 to 0.3 second lifetime

Recommended gameplay treatment:

- do not simulate long ballistic arcs
- keep travel deterministic and readable
- support optional “bolt speed” tiers for heavy weapons or snipers

### Modern bullets and tracers

Bullets should read as near-instant kinetic rounds:

- tiny projectile body or no visible body at all
- optional tracer line for readability
- minimal gravity unless it is a heavy round
- very short lifetime

Recommended visual treatment:

- tracer-only for small arms
- subtle muzzle flash on spawn
- tiny impact puff or spark
- shell ejection should be a separate weapon effect, not the projectile itself

### Modern missiles

Missiles should be the guided ordnance class:

- slower than bullets
- visible body with smoke plume
- modest steering
- target acquisition or soft homing
- large terminal explosion

Recommended gameplay treatment:

- steer toward a moving target, not directly snap to it
- cap turn rate to avoid unnatural curves
- allow target loss fallback to ballistic continuation
- optionally split into dumb-fire and guided variants

Recommended visual treatment:

- smoke trail with intermittent flame
- flashing exhaust or flicker
- larger explosion on impact
- visible body silhouette so the player can read the threat

## Homing Design Notes

Missile guidance should be built as a constrained steering problem, not a teleport-to-target behavior.

Use:

- forward vector
- desired intercept vector
- max turn rate per second
- acceleration or thrust value
- target validity checks

Avoid:

- instant angular snapping
- perfect tracking through obstacles
- infinite life if the target dies

Good RTS missile behavior usually follows one of these patterns:

1. dumb-fire projectile with smoke trail and big impact
2. soft homing that only adjusts enough to feel guided
3. hard homing only for slow, expensive, high-value ordnance

For DINOForge, soft homing is the best default because it stays readable in crowded battles.

## How A Pack Should Declare Overrides

The pack needs a declarative projectile override section.

### Suggested YAML shape

```yaml
projectiles:
  defaults:
    factionColorMode: sourceFaction
    trail:
      enabled: true
      prefab: projectile/trails/default
    impact:
      prefab: projectile/impacts/default

  overrides:
    - source: vanilla:archer_arrow
      themed: starwars:blaster_bolt
      behavior:
        type: straight
        speedScale: 2.5
        gravityScale: 0.0
      visuals:
        colorMode: faction
        glowIntensity: 2.5
        trailPrefab: warfare-starwars/projectiles/blaster_trail
        impactPrefab: warfare-starwars/projectiles/blaster_impact

    - source: vanilla:cannonball
      themed: modern:missile
      behavior:
        type: homing
        speedScale: 0.8
        turnRate: 180
        gravityScale: 0.05
      visuals:
        trailPrefab: warfare-modern/projectiles/missile_smoke
        impactPrefab: warfare-modern/projectiles/missile_explosion
```

### Suggested schema constraints

The pack validator should enforce:

- `source` must resolve to a known vanilla archetype or registry ID
- `themed` must resolve to a projectile archetype defined by the pack
- `behavior.type` must be one of:
  - `straight`
  - `ballistic`
  - `homing`
  - `hitscan_tracer`
  - `guided_arc`
- `speedScale`, `gravityScale`, and `turnRate` must be numeric and nonnegative where appropriate
- impact and trail prefabs must exist or have a descriptor fallback

### Suggested registry contract

If DINOForge continues the registry-first approach, projectile overrides should live alongside other pack content:

- `projectile.archetypes`
- `projectile.visuals`
- `projectile.mappings`
- `projectile.effects`

That keeps projectile theme changes data-driven and compatible with pack diffing and validation.

## How The Current Runtime Could Evolve

The shortest path is not to rewrite projectile physics from scratch. The most practical path is:

1. keep vanilla projectile spawn and collision behavior where possible
2. intercept the projectile archetype and resolve a themed profile
3. attach a trail/impact renderer based on the resolved profile
4. add a small guided-motion system only for missile-class overrides

This is lower risk than replacing all projectile simulation, and it preserves the current ECS hooks that already identify projectile entities.

## SOTA RTS Projectile / VFX Patterns

Across modern RTS and RTS-adjacent games, the strongest projectile systems tend to use the same design ideas:

- **Readability first**: the player should instantly know whether a projectile is a bullet, bolt, shell, or missile.
- **Separated simulation and presentation**: gameplay damage resolution should not depend on the exact visual effect.
- **Network-friendly / deterministic motion**: projectile motion is often simple enough to stay stable under load.
- **LOD-aware visual density**: trails, sparks, and impact debris scale down as distance or unit count rises.
- **Material-driven faction identity**: faction colors, emission, and trail shapes provide identity without bespoke meshes everywhere.
- **Pooled transient effects**: impact bursts and trail objects are reused aggressively.
- **Guided ordnance with constrained steering**: missiles usually curve, but not with perfect arcade homing unless the game is intentionally stylized.
- **Hitscan-like abstraction for small arms**: some RTS games fake bullets with tracers and instant damage to keep combat legible and cheap.

Practical takeaway for DINOForge:

- use visual differentiation to support theme
- keep simulation cheap and deterministic
- make homing an explicit projectile class, not an afterthought
- pool everything that is transient

## Recommended DINOForge Implementation Shape

### In the bridge

Extend the current bridge with:

- projectile archetype lookup
- projectile visual profile resolution
- projectile motion override resolution
- pack-aware mapping table

### In the runtime VFX layer

Extend the existing prefab descriptor pattern to include:

- trail prefabs
- impact prefabs
- optional glow-only effect descriptors
- faction-tint metadata

### In pack content

Add a projectile override declaration block to the pack manifest or a sibling YAML file, with schema validation and registry integration.

## Conclusion

DINO already exposes enough projectile identity to support themed projectiles, but it currently only uses that information for coarse impact VFX. The best modding design is a data-driven projectile override layer with three explicit parts:

1. vanilla archetype mapping
2. behavior profile
3. visual profile

That structure cleanly supports Star Wars blaster bolts, modern bullets/tracers, and guided missiles while staying consistent with DINOForge’s registry and ECS bridge model.
