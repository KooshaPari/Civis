# Civis Art Direction — DINOForge/FocalPoint-grade target + gap closeout

> **Status:** Binding art-direction guide (2026-05-30). Authority for the UI/UX Lead,
> Voxel/Material Lead, and Rendering Lead. Companion to
> [`docs/specs/tool-design-directives.md`](../specs/tool-design-directives.md) and
> [`docs/guides/emergence-charter.md`](../guides/emergence-charter.md).
>
> **Why this exists:** the user judged the art on sibling projects **FocalPoint** and
> **DINOForge** to be "solid" and wants Civis to hit the same bar. This guide distills
> *what makes those projects read as polished*, measures Civis against them, and gives a
> prioritized, file-anchored closeout so every gap maps to a system to change.

---

## 1. What makes the sibling art strong (the bar to match)

### DINOForge — studied source of truth
Files studied: `C:/Users/koosh/Dino/assets/branding/svg/{dinoforge-badge,mods-active-chip,window-icon}.svg`.

DINOForge's art language is **disciplined and deliberate**, and that discipline is the
reason it reads as "designed" rather than "generated":

1. **A locked, documented 4-role palette.** Every SVG header states the palette inline
   (anvil steel `#6B7280`/`#9CA3AF`/`#D1D5DB`, forge ember `#E8743B`, brand teal `#4ECDC4`,
   dark base `#1A1F2E`/`#0F1219`). Color is assigned by *role* (structure / accent / heat /
   ground), never picked ad-hoc. Exactly **one** hot accent (ember) per composition.
2. **Layered depth on flat vector.** Steel uses a 3-stop top→bottom gradient
   (`anvilFace`: `#D1D5DB → #9CA3AF → #4B5563`) to fake a lit metal face; a separate darker
   `anvilSide` gradient sells the bevel. Nothing is a single flat fill where a surface should
   catch light.
3. **Glow as a first-class tool, used sparingly.** A reusable `emberGlow`/`ringGlow`
   `feGaussianBlur`+`feMerge` filter makes the ember sparks and brand ring *emit* light. The
   glow is reserved for the energy/accent element — it earns attention because it is the only
   thing glowing.
4. **Readability budget per scale.** `window-icon.svg` is explicitly authored to survive
   16×16 ("only the glow dot matters at that size") — bold high-contrast fills, no thin
   strokes at icon scale. `mods-active-chip.svg` is a glass HUD pill: dark translucent
   gradient bg (`rgba(15,18,30,0.82)`), thin teal border at 0.65 opacity, a 0.12-opacity
   white top highlight line for a glass-pane feel.
5. **Consistent semantic accents.** Teal = brand/active/structure; ember = heat/energy/alert;
   white-at-low-opacity = glass highlight; the same eye motif (dark socket + ember pupil +
   tiny warm highlight dot) recurs at every scale.

### FocalPoint — corroborating signal
Files studied: `C:/Users/koosh/Dev/FocalPoint/README.md`, `apps/ios/` (SwiftUI + Rust core).
FocalPoint is a native iOS app (no shipped SVG asset library on disk; visual identity lives
in SwiftUI + an `xcassets`-driven mascot state machine that is referenced but not yet
populated). Its transferable lessons are **structural, not pixel-level**:

- **Native, platform-idiomatic surfaces** over custom-drawn chrome (SwiftUI system materials /
  vibrancy as the "glass" base). Civis's egui HUD should likewise lean on a *single coherent
  material system*, not per-widget bespoke art.
- **A mascot/character as an emotional anchor** (FocalPoint's mascot state machine). Civis has
  the equivalent hook in `assets/ui/map2d/agent.svg` and the civilian/creature glb set — the
  art direction should give the agent/citizen a recognizable, expressive silhouette so the
  living world has a "face."

### Where Civis already meets the bar (do not regress this)
Civis's **2D/HUD SVG layer is already DINOForge-grade** and should be treated as the locked
house style, not reworked. Evidence:
- `clients/bevy-ref/assets/ui/logo.svg` — radial globe gradient, dual cyan/gold glow filters,
  drop-shadowed serif wordmark, corner registration marks. This is a strong, intentional mark.
- `clients/bevy-ref/assets/ui/hud/panel-frame.svg` — proper 9-slice glass panel: translucent
  navy gradient, glowing 2px cyan border, low-opacity top sheen, decorative corner dots +
  edge ticks. Directly parallels DINOForge's chip.
- `clients/bevy-ref/assets/ui/material-icons/water.svg` — clipped drop with interior wave +
  glint; clean iconography.

**The gap is almost entirely in the 3D voxel world**, not the 2D UI.

---

## 2. The Civis palette (locked hex)

Derived from the existing `logo.svg` / `panel-frame.svg` house style and harmonized into
DINOForge-style **named roles**. Use these everywhere — UI, gizmos, world accents, doc art.

### Brand / UI core
| Role | Hex | Use |
|------|-----|-----|
| Ground / deep base | `#060E1A` | darkest backdrop, panel bottom |
| Base navy | `#0D1628` | panel bg, viewport letterbox |
| Surface navy | `#1A1F2E` | raised surfaces, tooltips |
| Brand cyan (primary accent) | `#50C8F0` | borders, active state, selection, brand glow |
| Cyan deep | `#1A3A5C` | cyan fills in shadow |
| Brand gold (secondary accent) | `#E8B84B` | sun, highlights, "confirm/positive", thin accent lines |
| Gold warm-core | `#FFF7D4` | hottest highlight of a gold glow |
| Text high | `#E8EEF5` | primary text/wordmark |
| Text mid | `#9CA3AF` | secondary text, inactive |
| Text low | `#6B7280` | tertiary, captions |

**Rule (from DINOForge):** cyan and gold are the *only* two accents. Cyan = structure /
brand / selection. Gold = energy / sun / positive confirmation. Never introduce a third UI
accent hue; semantic colors below live in the **world**, not the chrome.

### World material accents (semantic, world-only)
Pulled from `crates/voxel/src/material.rs` and re-roled so the palette stays coherent:
| Family | Anchor hex | Notes |
|--------|-----------|-------|
| Heat / lava / ember | `#F56F18` → `#FFAA4A` (hot core `#FFF7D4`) | **must emit** (see §4) |
| Water / coolant | `#3670CC` (deep `#0D2A4A`) | wet specular, not flat blue |
| Toxic / acid | `#5ECC38` | reserved — used *only* for hazard, like ember in DINOForge |
| Earth / dirt / clay | `#705030`–`#9A7054` | high roughness, matte |
| Stone / rock | `#6C7074`–`#7E7C7A` | mid roughness, faint spec |
| Snow / ice | `#E0F0FF` / `#B0E0FF` | ice = translucent + high spec |
| Metal / ore | `#C4A454` (molten) `#B08848` (ore) | metallic > 0 |

---

## 3. Lighting & atmosphere targets

Civis already spawns a sun + atmosphere (`clients/bevy-ref/src/atmosphere.rs`,
`skybox.rs`, `lighting_gi.rs`) and an HDRI (`assets/sky/kloofendal_43d_clear_puresky_1k.hdr`).
The targets below give the *values* that turn "lit" into "lit well."

- **Key light (sun):** warm, not white. `DirectionalLight` color ≈ `#FFF4E0`
  (≈ 5200–5500K), illuminance ~32,000 lx at noon. Drives DINOForge-style warm-key / cool-fill
  contrast.
- **Ambient / fill:** cool sky bounce ≈ `#7FA8D8` from the HDRI environment map (already
  loaded — ensure it is wired as the `EnvironmentMapLight`, not just a skybox texture). Low
  intensity so shadows stay readable.
- **Golden-hour default for hero shots/marketing:** sun azimuth low, color pushed to
  `#FFD9A0`; this is the lighting that makes the gold/cyan brand palette sing and matches the
  logo's sun-glow mood.
- **Shadows:** keep the existing 4-cascade CSM at ~800 m (`post_fx.rs::tune_sun_shadows`).
  Verify cascade blend so near voxels get crisp contact shadows (the cheap, high-impact cue
  that makes voxel stacks read as solid 3D, not a flat heightfield).
- **Sky:** physically-plausible gradient zenith `#2A5A9A` → horizon `#Bcd0E8`; keep a soft
  warm band at the sun. Tie horizon haze color to the sun color so atmosphere feels unified.

---

## 4. Material / PBR treatment — the #1 visual gap

**Problem (confirmed in source):** every voxel material in
`crates/voxel/src/material.rs` carries a single flat `color: [u8;4]` and nothing else — no
roughness, no metallic, no emissive. `clients/bevy-ref/src/materials.rs` then builds terrain
`StandardMaterial`s with a *uniform* `perceptual_roughness: 0.95, metallic: 0.0,
reflectance: 0.18` for **all** biomes. Result: the world reads as flat RGB — wet water,
molten lava, polished metal, and dry dirt all share the same surface response. This is the
single biggest thing standing between Civis and the sibling bar.

**Target: every material carries a small PBR profile, not just a color.** Extend
`MaterialDef` with four fields and map them into the Bevy `StandardMaterial`:

| Field | Type | Meaning |
|-------|------|---------|
| `perceptual_roughness` | `u8` (0–255 → 0.0–1.0) | dirt/stone high, water/ice/metal low |
| `metallic` | `u8` | only ore/molten-metal > 0 |
| `emissive` | `[u8;3]` | lava/ember/plasma/fire glow; black for inert |
| `reflectance` | `u8` | water/ice/glass higher; matte powders low |

**Per-family PBR targets (the rich-not-flat recipe):**
- **Water / salt water / coolant:** roughness ~0.05, reflectance ~0.5, slight base-color
  alpha for depth tint. Reads *wet* via specular, not via a darker blue.
- **Lava / molten metal / ember / fire / plasma:** **emissive** keyed to temperature —
  e.g. lava emissive `#FF5A14` at ~4–8× HDR intensity so it blooms (the bloom pass already
  exists and is doing nothing for these today). This is the highest-wow, lowest-cost change.
- **Ice / glass / crystal:** low roughness + high reflectance + partial transmission/alpha;
  cool tint.
- **Metal / ore:** metallic ~0.8, mid roughness — catches the warm key as a hot highlight.
- **Dirt / clay / sand / ash:** roughness 0.9–1.0, reflectance ~0.1 — stay matte so the
  shiny materials have contrast to read against.
- **Snow:** high roughness *but* high base brightness + subtle subsurface tint.

**Terrain biomes:** replace the single shared roughness with per-biome roughness/reflectance
(snow shinier than forest floor; wet sand vs dry sand) in `materials.rs`. The texture maps
already exist (`assets/textures/<biome>/{albedo,normal}.jpg`); wire the **ORM** (Phase 2,
already stubbed in `materials.rs`) so roughness varies *within* a surface.

---

## 5. Iconography & UI style rules (DINOForge SVG language)

Civis's SVG layer is already on-style; codify it so new assets stay consistent:

1. **Palette by role only** — §2 hexes. One cyan + one gold accent; semantic world colors
   never leak into chrome.
2. **Depth via gradients, never flat fills on a lit surface.** Match DINOForge's 3-stop
   steel and Civis's own `globeGrad`/`panelBg`. Panels: top→bottom dark navy gradient + a
   low-opacity top sheen.
3. **Glow is reserved for energy/accent/active.** Use the existing `cyanGlow`/`goldGlow`/
   `borderGlow` `feGaussianBlur`+`feMerge` filters (in `logo.svg`/`panel-frame.svg`).
   Default state does not glow; selection/active/heat does.
4. **Scale-aware authoring.** Tool icons & material icons (48×48 viewBox) must read at
   ~24px: bold fills, ≥2px strokes, the meaning carried by silhouette + one accent. Like
   DINOForge's 16px icon test.
5. **House framing furniture:** corner registration marks (logo) and corner dots + edge
   ticks (panel) are the Civis "console" motif — reuse on new panels/dialogs for cohesion.
6. **Glass HUD recipe (locked):** bg `url(#panelBg)` navy gradient @ ~0.92 alpha → 2px
   `#50C8F0` border via `borderGlow` → 0.5px inner accent border @ 0.25 → corner dots @ 0.5.
7. **Type:** serif (`Georgia`) for the brand wordmark only; sans (`Trebuchet/Arial`) for all
   functional/UI text, wide letter-spacing for labels (mirrors logo subtitle).
8. **Agent/citizen identity (FocalPoint mascot lesson):** give the agent a single
   recognizable silhouette + one accent, consistent between `map2d/agent.svg` and the 3D
   `civilian.glb`, so the living world has a "face."

---

## 6. Post-processing recipe (concrete values)

The stack exists and is correctly assembled in `clients/bevy-ref/src/post_fx.rs`
(`Hdr` + `Tonemapping::AcesFitted` + `Bloom` + `ScreenSpaceAmbientOcclusion` + `TAA`, MSAA
off). What is missing is *tuned values* and *emissive content for bloom to act on*.

| Effect | Target | Rationale |
|--------|--------|-----------|
| Tonemap | `AcesFitted` (keep) | filmic rolloff; keeps the warm/cool palette from clipping |
| Bloom | `intensity ≈ 0.15`, low threshold | soft halo on emissive lava/ember/cyan UI-in-world; **needs §4 emissive materials to matter** |
| SSAO | keep enabled, mid strength | contact darkening in voxel crevices — huge for solidity, cheap |
| TAA | keep (MSAA off) | stable edges on dense voxel geometry |
| Color grade | slight S-curve contrast; lift shadows toward cool `#0D1628`, push highlights toward warm `#FFF4E0` | unifies everything into the cyan-shadow/gold-highlight brand mood (the logo's mood) |
| Vignette (optional) | subtle, ~0.1 | focuses the city-builder camera, frames marketing shots |

Expose all of the above through the existing `PostFxSettings` resource so it stays tunable
and testable; add the grade/vignette as new fields rather than hard-coding.

---

## 7. Prioritized art gap closeout (gap → system/file)

Ranked by **visual-bar-raised ÷ effort**. Each row names the file/system to change.

| # | Gap | Fix | File / system | Effort | Impact |
|---|-----|-----|---------------|:------:|:------:|
| 1 | Emissive materials are inert → lava/ember don't glow; bloom does nothing for the world | Add `emissive` to `MaterialDef`; key heat materials to HDR emissive | `crates/voxel/src/material.rs` + map in `clients/bevy-ref/src/materials.rs` | S | ★★★★★ |
| 2 | All materials share one roughness/metallic → flat RGB world | Add `perceptual_roughness`/`metallic`/`reflectance` to `MaterialDef`; map per-material into `StandardMaterial` | `crates/voxel/src/material.rs`, `clients/bevy-ref/src/materials.rs` | M | ★★★★★ |
| 3 | Per-biome terrain surface is uniform | Per-biome roughness/reflectance; wire ORM map (Phase 2 stub) | `clients/bevy-ref/src/materials.rs` | M | ★★★★ |
| 4 | Post-FX has no tuned values; bloom/grade unset | Set bloom intensity, add color-grade + vignette to `PostFxSettings` | `clients/bevy-ref/src/post_fx.rs` | S | ★★★★ |
| 5 | Sun/ambient are neutral; no warm-key/cool-fill | Warm sun `#FFF4E0` + cool HDRI `EnvironmentMapLight` fill; golden-hour preset | `clients/bevy-ref/src/atmosphere.rs`, `lighting_gi.rs` | S | ★★★★ |
| 6 | Water/ice read as flat fills | Low-roughness + reflectance + alpha depth tint (rides #2) | `material.rs` + `materials.rs` | S | ★★★ |
| 7 | No locked palette doc for new asset authors | This file is the source of truth; reference it in `CONTRIBUTING`/asset PRs | docs + asset-review checklist | S | ★★★ |
| 8 | Material icons not coverage-complete vs the new rich taxonomy (tool-design-directives wants dozens) | Author missing element icons in the locked SVG style (§5) | `assets/ui/material-icons/*.svg` | M | ★★ |
| 9 | Agent/citizen lacks a consistent identity across 2D/3D | Align `map2d/agent.svg` silhouette + accent with `civilian.glb` | assets + `bevy_render.rs` | M | ★★ |
| 10 | Sky/atmosphere colors not tied to sun | Couple horizon haze to sun color; zenith/horizon gradient values §3 | `skybox.rs`, `atmosphere.rs` | S | ★★ |

---

## 8. Reference index (files Civis should emulate / change)

**Emulate (sibling exemplars):**
- `C:/Users/koosh/Dino/assets/branding/svg/dinoforge-badge.svg` — palette-by-role, layered
  gradients, reserved glow.
- `C:/Users/koosh/Dino/assets/branding/svg/window-icon.svg` — scale-aware (16px) icon
  authoring.
- `C:/Users/koosh/Dino/assets/branding/svg/mods-active-chip.svg` — glass HUD pill recipe.
- `C:/Users/koosh/Dev/FocalPoint/README.md` — mascot-as-anchor + native-material discipline.

**Already on-bar in Civis (lock as house style):**
- `clients/bevy-ref/assets/ui/logo.svg`, `assets/ui/hud/panel-frame.svg`,
  `assets/ui/material-icons/water.svg`.

**Change (where the bar is raised):**
- `crates/voxel/src/material.rs` — add PBR profile fields.
- `clients/bevy-ref/src/materials.rs` — map per-material PBR + per-biome surface.
- `clients/bevy-ref/src/post_fx.rs` — tuned bloom/grade/vignette.
- `clients/bevy-ref/src/{atmosphere,lighting_gi,skybox}.rs` — warm-key/cool-fill + sky.

> **Verification stance (per project memory — vision-verify):** none of these changes are
> "done" on telemetry alone. After each, capture a screenshot and read the pixels — the world
> must *look* rich, with lava that glows, water that reads wet, and warm-key/cool-shadow
> separation, before the gap is closed.
