# Civis Art Closeout — #4 Warm-Key/Cool-Fill Lighting + #5 Per-Biome Surface PBR

> **Status:** Binding design spec (2026-05-30). PLANNER artifact — specs, values, and
> pseudocode only; no implementation. Authority for the Rendering Lead and Voxel/Material Lead.
>
> **Scope:** Closes gaps **#4 (warm-key / cool-fill lighting + golden-hour preset)** and
> **#5 (per-biome terrain surface PBR)** from
> [`docs/research/art-direction.md`](../research/art-direction.md) §7. Companion to the
> already-landed voxel-material PBR profile (gaps #1/#2) and post-FX stack (gap #4 post side).
>
> **Requirements:** `FR-CIV-RENDER-LIGHT-KEYFILL`, `FR-CIV-RENDER-LIGHT-GOLDENHOUR`,
> `FR-CIV-RENDER-DAYNIGHT-CURVE`, `FR-CIV-RENDER-BIOME-PBR`, `FR-CIV-RENDER-ATMOS-FOG`,
> `FR-CIV-RENDER-GRADE-COHESION`.
>
> **Source files read (read-only):** `clients/bevy-ref/src/atmosphere.rs`, `post_fx.rs`,
> `materials.rs`; `docs/research/art-direction.md`.
>
> **Verification stance (project memory — vision-verify):** none of these values are "done"
> on telemetry alone. After wiring, capture a screenshot at noon **and** golden-hour and read
> the pixels: warm-lit faces + cool shadows, lava blooming, water reading wet, snow shinier
> than forest floor.

---

## 0. Design intent (one paragraph)

Civis should read like a cinematic god-game: a **warm key** (sun, ≈5400K) rakes across voxel
stacks while a **cool sky fill** (HDRI environment, ≈7500K) lifts the shadows toward navy.
That warm/cool split is the single cue that makes the gold/cyan brand palette "sing" — gold
lives in the lit highlights, cyan lives in the shadows, exactly the mood of `logo.svg`. The
already-landed voxel PBR profile supplies *surface response* (roughness/metallic/emissive);
this spec supplies the *light that reveals it* (§1–§3) and the *per-biome ground surface* it
plays across (§4). §6 is the composition contract that proves the three layers stack into one
coherent grade rather than fighting each other.

---

## 1. Light rig — warm key / cool fill (`FR-CIV-RENDER-LIGHT-KEYFILL`)

### 1.1 Three-light model

| Role | Bevy component | Source file | Purpose |
|------|----------------|-------------|---------|
| **Key** (warm sun) | `DirectionalLight` (`SunLight`) | `atmosphere.rs` | primary modelling light, casts CSM shadows |
| **Fill** (cool sky) | `EnvironmentMapLight` from the loaded HDRI | `lighting_gi.rs` / `skybox.rs` | lifts shadow side toward cool navy; specular IBL on wet/metal |
| **Ambient floor** | `GlobalAmbientLight` (low) | `atmosphere.rs` | tiny global floor so deep crevices never crush to pure black |

> **Key change vs current code:** today `atmosphere.rs` uses a neutral/white sun
> (`Color::WHITE` above `daylight > 0.55`) and a `GlobalAmbientLight { color: WHITE,
> brightness: 500 }`. That neutral-on-neutral rig is *why the world reads flat*. The fill
> must come from the **HDRI environment map** (cool, directional-ish sky bounce), not a flat
> white ambient. Drop `GlobalAmbientLight.brightness` to a low floor and let the
> `EnvironmentMapLight` do the fill.

### 1.2 Noon (neutral daylight) reference values

| Parameter | Value | Hex / note |
|-----------|-------|-----------|
| **Key color** (sun) | `srgb(1.000, 0.957, 0.878)` | `#FFF4E0`, ≈5400K warm-white |
| **Key illuminance** | `32_000` lx | art-direction §3 noon target (was 15 000 — too dim for filmic ACES) |
| **Key elevation** | sun.y ≈ `0.82` (≈55° above horizon) | high but never straight down — keep raking shadows |
| **Fill: `EnvironmentMapLight.diffuse_intensity`** | `900.0` | cool sky bounce from `kloofendal_43d_clear_puresky_1k.hdr` |
| **Fill: `EnvironmentMapLight.specular_intensity`** | `1.0` | drives IBL specular on water/ice/metal (rides §4) |
| **Fill tint (perceived)** | `srgb(0.498, 0.659, 0.847)` | `#7FA8D8` cool — comes free from the HDRI; do **not** hand-tint |
| **Ambient floor brightness** | `120.0` (was 500) | low; just lifts black crevices |
| **Ambient floor color** | `srgb(0.051, 0.086, 0.157)` | `#0D1628` base-navy — biases shadow toward brand-cool, not gray |
| **Key:fill ratio (perceived)** | ≈ **4:1** at noon | the cinematic contrast; never 1:1 (flat) or 8:1 (harsh) |

**Pseudocode (replaces the white-sun branch in `update_lighting`):**

```
KEY_WARM   = srgb(1.000, 0.957, 0.878)   // #FFF4E0
KEY_GOLDEN = srgb(1.000, 0.851, 0.627)   // #FFD9A0
KEY_DUSK   = srgb(0.961, 0.408, 0.227)   // #F56839 (low-sun ember)

// daylight in 0..1 from sun elevation (existing var)
sun.color       = key_color_for_phase(time_of_day, golden_hour_preset)
sun.illuminance = lerp(200.0, 32_000.0, smoothstep(0.0, 0.25, daylight))

ambient.color      = srgb(0.051, 0.086, 0.157)   // #0D1628
ambient.brightness = lerp(40.0, 120.0, daylight)  // night floor → day floor
env_map.diffuse_intensity  = lerp(120.0, 900.0, daylight)
env_map.specular_intensity = 1.0
```

### 1.3 Cascade shadows (keep, verify)

`post_fx.rs::tune_sun_shadows` already builds 4-cascade CSM at 800 m — **keep as-is**. Add one
verification: with the higher 32 000 lx key, confirm near-voxel **contact shadows** stay crisp
(cascade 0 first-split). The contact-shadow darkening is what sells voxel stacks as solid 3D.
No code change unless cascade 0 split exceeds ~30 m (then lower `first_cascade_far_bound`).

---

## 2. Golden-hour preset (`FR-CIV-RENDER-LIGHT-GOLDENHOUR`)

A **named, toggleable lighting mood** for hero shots / marketing / screenshots, on top of the
live day/night curve. It is a *preset that pins `time_of_day` low and warms the key*, not a
separate rig.

| Parameter | Golden-hour value | Hex / note |
|-----------|-------------------|-----------|
| **Sun elevation** | sun.y ≈ `0.18` (≈10° above horizon) | low, long shadows |
| **Sun azimuth** | side-on to camera (≈ −60° from view) | rim-lights voxel silhouettes |
| **Key color** | `srgb(1.000, 0.851, 0.627)` | `#FFD9A0` pushed-warm gold |
| **Key illuminance** | `20_000` lx | lower than noon; ACES rolls the warm highlights to gold, not white |
| **Fill diffuse intensity** | `650.0` | dimmer cool fill → richer warm/cool separation |
| **Sky horizon haze** | `srgb(0.961, 0.612, 0.341)` | `#F59C57` warm band, tied to sun (see §5) |
| **Bloom intensity** | bump to `0.22` (from 0.15) | sun-glow halo; makes gold accents glow like the logo |
| **Grade highlight push** | toward `#FFD9A0` | §6 grade; unifies into the marketing mood |

**Expose as:** a `GoldenHourPreset(bool)` resource (or a `LightingPreset` enum
`{ Live, Noon, GoldenHour }`). When active, `update_lighting` **pins** `time_of_day = 0.62`
(low evening sun) and substitutes the table above instead of the live curve. Default = `Live`.
Toggle is a debug/marketing affordance — never forced on the live sim.

---

## 3. Day/night color curve (`FR-CIV-RENDER-DAYNIGHT-CURVE`)

Replaces the ad-hoc branch ladder in `update_lighting` with one documented keyframe table.
`time_of_day` is normalized `0..1` (existing convention: **noon ≈ 0.75, midnight ≈ 0.25**).
Values interpolate with `smoothstep` between adjacent keyframes (existing helper).

| Phase | `time_of_day` | Sun color (hex) | Illuminance (lx) | Sky zenith → horizon | Ambient brightness |
|-------|:-------------:|-----------------|:----------------:|----------------------|:------------------:|
| Midnight | 0.25 | `#3A4D80` (moonlight, cool) | 200 | `#070C18` → `#0E1A33` | 40 |
| Pre-dawn | 0.40 | `#6E5A8C` (violet) | 1 500 | `#101A33` → `#2A2F5C` | 55 |
| Dawn | 0.50 | `#FFD9A0` (golden) | 8 000 | `#2A4A78` → `#F59C57` | 75 |
| Morning | 0.62 | `#FFE6C2` (warm) | 20 000 | `#2A5A9A` → `#BCD0E8` | 105 |
| **Noon** | **0.75** | **`#FFF4E0`** (warm-white key) | **32 000** | **`#2A5A9A` → `#BCD0E8`** | **120** |
| Afternoon | 0.85 | `#FFE6C2` (warm) | 24 000 | `#2A5A9A` → `#C8D4E0` | 110 |
| Dusk | 0.92 | `#F56839` (ember) | 6 000 | `#3A4A82` → `#D86E3C` | 70 |
| Twilight | 0.05 | `#5A4A8C` (violet-blue) | 1 200 | `#141E3C` → `#3A2F5C` | 50 |

**Curve rules:**
- Sun **color** lerps in sRGB between keyframes (use existing `lerp_color`).
- **Illuminance** uses `smoothstep` so dawn/dusk ramp is soft, not linear-harsh.
- The **horizon** value at each phase is the sun-tied haze color (§5) — dawn/dusk horizons are
  warm because the sun is warm; this is what couples sky to sun.
- Moon (`MoonLight`) stays as-is but its color is locked to `#3A4D80` (current `0.35,0.45,0.75`
  is close — nudge to the table value for palette coherence).
- **Brand coherence:** every shadow-side/ambient value biases toward the cyan-navy family
  (`#0D1628`/`#070C18`); every key/highlight biases warm. The world's own day/night cycle thus
  *is* the cyan-shadow / gold-highlight brand mood at every hour.

---

## 4. Per-biome terrain surface PBR (`FR-CIV-RENDER-BIOME-PBR`)

### 4.1 Mapping note — 7 families vs the shipped 6-band enum

The task names **7 surface families** (desert / grass / tundra / forest / wetland / rock /
snow). The code's `Biome` enum (`materials.rs`) ships **6 height-bands** (`SandBeach`,
`DirtGround`, `GrassField`, `ForestFloor`, `RockCliff`, `SnowPure`). The table below gives a
value row for **all 7 families** plus the **direct mapping** onto the 6 shipped bands so an
implementer can paste today and extend later. `Desert` and `Wetland` are the two new families;
until the enum grows, **Desert ≈ SandBeach (dry variant)** and **Wetland ≈ DirtGround (wet
variant)** with the alpha/roughness deltas noted.

### 4.2 The locked per-biome surface table

All values target Bevy `StandardMaterial`. `roughness` = `perceptual_roughness`,
`reflectance` is Bevy's 0–1 dielectric reflectance (0.5 ≈ 4% F0, the physical default; raise
for wet/polished), `metallic` is 0 for all natural ground. `base_color` is the sRGB **tint**
multiplied over the albedo texture (keep near-white-warm so textures dominate; tint only nudges
mood).

| Family | Maps to enum band | `base_color` tint (hex) | roughness | reflectance | metallic | Specular read |
|--------|-------------------|-------------------------|:---------:|:-----------:|:--------:|---------------|
| **Desert** (dry sand) | `SandBeach` | `#DBC78A` | **0.92** | 0.30 | 0.0 | matte, faint sheen on dune crests |
| **Grass** (plains) | `GrassField` | `#4A9A3D` | **0.88** | 0.25 | 0.0 | matte, soft |
| **Tundra** (cold scrub) | *(new; ≈ DirtGround cold)* | `#8A8C70` | **0.85** | 0.30 | 0.0 | dry-matte, slight frost glint |
| **Forest** (leaf litter) | `ForestFloor` | `#1F571F` | **0.95** | 0.18 | 0.0 | flattest — darkest, most matte |
| **Wetland** (mud/marsh) | *(new; ≈ DirtGround wet)* | `#6E5A38` | **0.55** | **0.45** | 0.0 | **wet** — low roughness reads damp |
| **Rock** (cliff) | `RockCliff` | `#6C7074` | **0.78** | 0.35 | 0.0 | mid; faceted speculars catch the key |
| **Snow** (alpine) | `SnowPure` | `#F0F6FF` | **0.45** | **0.55** | 0.0 | **shiny + bright**; high reflectance sparkle |

**Plus the two shipped bands not in the 7-family list (keep current behavior, refined):**

| Enum band | `base_color` tint (hex) | roughness | reflectance | metallic | Note |
|-----------|-------------------------|:---------:|:-----------:|:--------:|------|
| `SandBeach` (wet shore) | `#C9B87A` | **0.65** | **0.42** | 0.0 | wetter than Desert — it's at the waterline |
| `DirtGround` (packed) | `#7A5E38` | **0.90** | 0.18 | 0.0 | dry matte; Wetland is its wet sibling |

### 4.3 Design rules behind the numbers

1. **Contrast budget:** the two *shiny* ground families (Snow `0.45`, Wetland `0.55`) exist so
   the matte families (Forest `0.95`, Grass `0.88`, Desert `0.92`) have something to read
   against. If everything is `0.95` (today's bug) the world is flat. Keep at least a `0.40`
   roughness spread across the biome set.
2. **Reflectance is the wet/dry knob:** wet surfaces (Wetland, wet shore, Snow) get
   reflectance `0.42–0.55`; dry powders (Forest, Dirt) get `0.18`. Reflectance — not a darker
   base color — is what makes a surface read damp under the cool fill's specular IBL.
3. **Tint stays subtle:** `base_color` multiplies the albedo texture, so keep tints close to
   the texture's own hue and lightly warm. Forest is the one deep tint (it should feel like
   shadowed canopy floor).
4. **ORM map wins within a surface:** once the Phase-2 `orm.ktx2` is wired
   (`metallic_roughness_texture` + `occlusion_texture`), the **per-biome roughness above
   becomes the *default*** and the ORM's green/blue channels modulate it *within* the surface
   (wet rock pools, dry dune ridges). The scalar table is the floor, not the ceiling.
5. **No metallic on ground:** all natural terrain is dielectric. Metallic > 0 is reserved for
   the voxel ore/molten-metal materials (already landed), never terrain.

### 4.4 Pseudocode — replace the uniform block in `load_biome_materials`

```
// today every biome shares perceptual_roughness:0.95, reflectance:0.18 — the flat-RGB bug.
fn surface_pbr(biome) -> (roughness, reflectance, metallic) = match biome {
    SandBeach  => (0.65, 0.42, 0.0),   // wet shore
    DirtGround => (0.90, 0.18, 0.0),
    GrassField => (0.88, 0.25, 0.0),
    ForestFloor=> (0.95, 0.18, 0.0),
    RockCliff  => (0.78, 0.35, 0.0),
    SnowPure   => (0.45, 0.55, 0.0),
}
// when Desert/Tundra/Wetland enum variants land, add their rows from §4.2.

materials.add(StandardMaterial {
    base_color: srgb(biome.tint()),          // §4.2 tint, not the old fallback_srgb
    base_color_texture: Some(albedo),
    normal_map_texture: Some(normal),
    // Phase 2: metallic_roughness_texture + occlusion_texture from orm.ktx2
    perceptual_roughness: r,
    reflectance: refl,
    metallic: m,
    ..default()
})
```

---

## 5. Atmospheric scattering / fog + sky (`FR-CIV-RENDER-ATMOS-FOG`)

Distance fog + a sun-coupled sky band give depth and tie the scene together. Values target a
Bevy `DistanceFog` component on the camera and the sky gradient in `skybox.rs`.

| Parameter | Noon value | Golden-hour value | Note |
|-----------|------------|-------------------|------|
| Fog mode | `FogFalloff::Atmospheric` (or exponential-squared) | same | physically-plausible haze |
| Fog start / "visibility" | ~600 m | ~400 m (more haze, mood) | 20 mi² world → far fog, near city crisp |
| Fog color (base) | `srgb(0.737, 0.816, 0.910)` `#BCD0E8` | `srgb(0.847, 0.612, 0.341)` `#D89C57` | **= horizon haze color; tie to sun** |
| Fog inscattering (sun-side glow) | sun color @ low directional weight | `#FFD9A0` strong | atmospheric sun-glow toward the light |
| Sky zenith | `#2A5A9A` | `#3A4A82` | art-direction §3 |
| Sky horizon | `#BCD0E8` | `#D89C57` | **same hex as fog color — unified** |
| Warm band at sun | soft, sun color | strong, `#FFD9A0` | the logo's sun-glow mood |

**Coupling rule (the key idea):** horizon-haze color, fog color, and fog inscattering are **all
driven from the current sun color** each frame. One source (sun `color`) → three consumers
(sky horizon, distance fog, inscatter). That is what makes the atmosphere feel *unified* with
the light instead of a static blue gradient pasted behind a warm scene.

```
horizon_haze = mix(sky_horizon_neutral, sun.color, 0.55)   // warmer as sun warms
fog.color           = horizon_haze
fog.inscatter_color = sun.color
sky.horizon_color   = horizon_haze
sky.zenith_color    = day_night_curve.zenith(time_of_day)   // §3 table
```

Keep the existing `ClearColor` darken/brighten with `time_of_day`; it becomes the fallback for
pixels the sky dome doesn't cover.

---

## 6. Composition contract — how the layers stack into one cinematic grade (`FR-CIV-RENDER-GRADE-COHESION`)

This is the proof that voxel-PBR (landed) + biome-surface (§4) + lighting (§1–§3) + post-FX
compose into **one** DINOForge-cyan/gold mood rather than four systems fighting. The render
order and each layer's job:

| Order | Layer | Owns | Reads from | Hands off |
|:-----:|-------|------|-----------|-----------|
| 1 | **Voxel material PBR** (landed) | roughness/metallic/**emissive** per voxel | `MaterialDef` | a physically-shaded surface |
| 2 | **Biome terrain surface** (§4) | ground roughness/reflectance/tint | per-biome table | matte vs wet ground response |
| 3 | **Light rig** (§1–§3) | warm key + cool fill + day/night | sun color, env map | warm-lit / cool-shadow linear-HDR scene |
| 4 | **Atmosphere/fog** (§5) | depth haze tied to sun | sun color | depth + unified horizon |
| 5 | **Post-FX grade** (`post_fx.rs`) | bloom + tonemap + **color grade** | linear-HDR scene | final cyan/gold film look |

### 6.1 The grade contract (extends `PostFxSettings`)

The post-FX stack already runs `Hdr` + `AcesFitted` + `Bloom{0.15}` + SSAO + TAA. To finish the
mood, **add a color-grade stage** (art-direction §6 already specifies this; here are the values):

| Grade control | Value | Effect |
|---------------|-------|--------|
| Tonemap | `AcesFitted` (keep) | filmic rolloff; warm highlights roll to gold not white-clip |
| Bloom intensity | `0.15` live / `0.22` golden-hour | halo on emissive lava/ember + bright snow + UI-in-world |
| Bloom threshold | low (~0.0 soft) | let emissive HDR voxels bloom; ground shouldn't |
| **Shadow lift (tint)** | toward `#0D1628` (cool navy) | shadows go cyan-family, not gray — brand cyan in the dark |
| **Highlight gain (tint)** | toward `#FFF4E0` (warm) | highlights go gold-family — brand gold in the light |
| **Contrast** | slight S-curve (~1.08) | crisp without crushing; keeps mid-tone biome detail |
| Saturation | +5% | richer biome greens/golds; not garish |
| Vignette (optional) | ~0.10 | frames the city-builder camera + marketing shots |

**Add these as fields on `PostFxSettings`** (keep it tunable/testable per the existing pattern):
`shadow_tint`, `highlight_tint`, `contrast`, `saturation`, `vignette`, `grade_enabled`. Do not
hard-code in the system body.

### 6.2 Why it's cohesive (the load-bearing logic)

- **One warm/cool axis everywhere.** Key = warm (`#FFF4E0`), fill = cool (`#7FA8D8`), grade
  pushes shadows cool + highlights warm. Every layer reinforces the *same* gold-highlight /
  cyan-shadow axis. Nothing introduces a competing hue (no green/magenta cast).
- **Bloom only fires on content that should glow.** Emissive voxels (lava/ember, landed) +
  bright snow specular + the gold sun band. Matte ground never blooms (low reflectance,
  no emissive) — so glow stays *meaningful*, exactly like DINOForge reserving glow for the
  ember accent.
- **Sun color is the single coupling source.** It drives the key, the horizon haze, the fog,
  and the inscatter (§5), and the grade's highlight tint tracks it. Change the time of day and
  the *entire frame* — light, sky, fog, grade — shifts together. That single-source coupling is
  what reads as "art-directed" rather than "assembled."
- **ACES is the safety net.** With a 32 000 lx warm key and HDR emissives, values exceed 1.0;
  `AcesFitted` rolls them filmically so the warm/cool palette never clips to white/black. This
  is why the grade can push contrast and tint without banding.

---

## 7. Implementation handoff (file → change, no code)

| # | File | Change | Effort | FR |
|---|------|--------|:------:|----|
| 4a | `clients/bevy-ref/src/atmosphere.rs` | warm key `#FFF4E0` @ 32 000 lx; drop `GlobalAmbientLight` to `#0D1628`@120; replace branch-ladder with §3 day/night curve table | S | `…KEYFILL`, `…DAYNIGHT-CURVE` |
| 4b | `clients/bevy-ref/src/lighting_gi.rs` | wire HDRI as `EnvironmentMapLight` (cool fill), diffuse 900 / specular 1.0 | S | `…KEYFILL` |
| 4c | `clients/bevy-ref/src/atmosphere.rs` (or new) | `LightingPreset { Live, Noon, GoldenHour }` resource; golden-hour pins `time_of_day=0.62` + §2 table | S | `…GOLDENHOUR` |
| 5a | `clients/bevy-ref/src/materials.rs` | replace uniform `0.95/0.18` block with §4.2 per-biome `(roughness,reflectance,metallic)` + §4.2 tints | M | `…BIOME-PBR` |
| 5b | `clients/bevy-ref/src/materials.rs` | when ORM lands: per-biome scalar becomes default, ORM modulates within surface (§4.3 rule 4) | M | `…BIOME-PBR` |
| 5c | `crates/voxel` enum + `materials.rs` | add `Desert`/`Tundra`/`Wetland` bands (§4.1) — extension, not required for paste | M | `…BIOME-PBR` |
| 6a | `clients/bevy-ref/src/skybox.rs` | couple horizon haze + zenith to sun color (§5); §3 zenith/horizon table | S | `…ATMOS-FOG` |
| 6b | `clients/bevy-ref/src/post_fx.rs` | add camera `DistanceFog` driven by sun color (§5) | S | `…ATMOS-FOG` |
| 6c | `clients/bevy-ref/src/post_fx.rs` | extend `PostFxSettings` with grade fields (§6.1); add color-grade stage | M | `…GRADE-COHESION` |

---

## 8. Acceptance (vision-verify, per project memory)

A screenshot at each checkpoint must show the stated pixels — telemetry PASS is not acceptance.

1. **Noon:** voxel faces toward the sun read warm/gold; the shadow side reads cool navy (not
   gray). Key:fill contrast visibly ≈4:1.
2. **Golden-hour preset:** low side-on sun rim-lights silhouettes; warm horizon band; gold
   accents bloom softly. Reads as a marketing hero shot.
3. **Biome spread:** snow visibly shinier/brighter than forest floor; wetland/wet-shore reads
   damp (specular sheen) while desert/forest read matte. No two adjacent bands look identical.
4. **Day/night sweep:** dawn→noon→dusk shifts sun, sky, fog, and grade *together* from one sun
   color; dusk horizon is warm-ember, midnight is cool-navy with moon + stars.
5. **Cohesion:** the whole frame sits on one gold-highlight / cyan-shadow axis — no competing
   hue cast; lava/ember + bright snow are the only things that bloom.

---

## 9. Reference index

- **Drives this spec:** [`docs/research/art-direction.md`](../research/art-direction.md) §3
  (lighting targets), §4 (PBR), §6 (post-FX), §7 (#3/#4/#5 gaps).
- **Source files (read-only, to be changed by implementers):**
  `clients/bevy-ref/src/atmosphere.rs`, `lighting_gi.rs`, `skybox.rs`, `materials.rs`,
  `post_fx.rs`.
- **Landed dependency:** voxel-material PBR profile (`crates/voxel/src/material.rs` emissive/
  roughness/metallic) — §6 layer 1; this spec assumes it is present.
- **HDRI asset:** `assets/sky/kloofendal_43d_clear_puresky_1k.hdr` — the cool fill source
  (§1.1); must be wired as `EnvironmentMapLight`, not skybox-texture-only.
