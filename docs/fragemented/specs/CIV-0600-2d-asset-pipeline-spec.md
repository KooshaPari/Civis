# CIV-0600: 2D Asset Pipeline Specification

**Version:** 1.0
**Status:** SPECIFICATION
**Date:** 2026-02-21
**Authors:** CIV Asset & Rendering Team
**References:** CIV-0300-rts-ui-ux-spec.md, CIV-0001-core-simulation-loop.md, CIV-0500-performance-optimization-spec.md, PRD.md

---

## Table of Contents

1. [Overview and Design Principles](#1-overview-and-design-principles)
   - 1.1 [Pipeline Philosophy](#11-pipeline-philosophy)
   - 1.2 [Pipeline Stages at a Glance](#12-pipeline-stages-at-a-glance)
   - 1.3 [Zoom-Level Output Contract](#13-zoom-level-output-contract)
   - 1.4 [Build-Time vs Runtime Contract](#14-build-time-vs-runtime-contract)
   - 1.5 [Directory Structure](#15-directory-structure)
2. [SVG Template System](#2-svg-template-system)
   - 2.1 [Templating Engine](#21-templating-engine)
   - 2.2 [Template Variables Reference](#22-template-variables-reference)
   - 2.3 [Template Catalog](#23-template-catalog)
   - 2.4 [Template Layer Architecture](#24-template-layer-architecture)
   - 2.5 [Example Templates](#25-example-templates)
   - 2.6 [Template Validation Rules](#26-template-validation-rules)
3. [resvg Renderer](#3-resvg-renderer)
   - 3.1 [Library Configuration](#31-library-configuration)
   - 3.2 [Rasterization Parameters](#32-rasterization-parameters)
   - 3.3 [Font Handling](#33-font-handling)
   - 3.4 [Color Profile Policy](#34-color-profile-policy)
   - 3.5 [Rust Implementation](#35-rust-implementation)
   - 3.6 [Performance Characteristics](#36-performance-characteristics)
   - 3.7 [Error Handling](#37-error-handling)
4. [SD XL Agentic Enhancement (Optional Pass)](#4-sd-xl-agentic-enhancement-optional-pass)
   - 4.1 [Trigger Conditions](#41-trigger-conditions)
   - 4.2 [Model Configuration](#42-model-configuration)
   - 4.3 [ControlNet Integration](#43-controlnet-integration)
   - 4.4 [Prompt Template System](#44-prompt-template-system)
   - 4.5 [Reproducibility Contract](#45-reproducibility-contract)
   - 4.6 [Cost Management](#46-cost-management)
   - 4.7 [Post-Pass Integration](#47-post-pass-integration)
5. [rembg Background Removal](#5-rembg-background-removal)
   - 5.1 [Library and Model Selection](#51-library-and-model-selection)
   - 5.2 [Processing Pipeline](#52-processing-pipeline)
   - 5.3 [Batch Processing Architecture](#53-batch-processing-architecture)
   - 5.4 [Quality Gate](#54-quality-gate)
   - 5.5 [Error Handling](#55-error-handling)
6. [Palette Quantization](#6-palette-quantization)
   - 6.1 [Library Selection](#61-library-selection)
   - 6.2 [Quantization Parameters](#62-quantization-parameters)
   - 6.3 [Nation Color Binding](#63-nation-color-binding)
   - 6.4 [Output Format](#64-output-format)
   - 6.5 [Rust Implementation](#65-rust-implementation)
7. [Atlas Packing](#7-atlas-packing)
   - 7.1 [Library and Algorithm](#71-library-and-algorithm)
   - 7.2 [Atlas Size Contracts](#72-atlas-size-contracts)
   - 7.3 [Metadata Format](#73-metadata-format)
   - 7.4 [Output File Contract](#74-output-file-contract)
   - 7.5 [Rust Implementation](#75-rust-implementation)
   - 7.6 [Runtime Loading (Pixi.js)](#76-runtime-loading-pixijs)
8. [Asset Manifest Schema](#8-asset-manifest-schema)
   - 8.1 [Schema Definition](#81-schema-definition)
   - 8.2 [Field Specifications](#82-field-specifications)
   - 8.3 [Versioning Policy](#83-versioning-policy)
   - 8.4 [Manifest Generation](#84-manifest-generation)
9. [Build Pipeline (Taskfile)](#9-build-pipeline-taskfile)
   - 9.1 [Taskfile Definition](#91-taskfile-definition)
   - 9.2 [Stage Dependencies](#92-stage-dependencies)
   - 9.3 [Incremental Build Strategy](#93-incremental-build-strategy)
   - 9.4 [Environment Requirements](#94-environment-requirements)
   - 9.5 [CI Integration](#95-ci-integration)
10. [Runtime Sprite System (Pixi.js)](#10-runtime-sprite-system-pixijs)
    - 10.1 [SpriteManager Architecture](#101-spritemanager-architecture)
    - 10.2 [Zoom-Level Atlas Switching](#102-zoom-level-atlas-switching)
    - 10.3 [Nation Recoloring via Shader](#103-nation-recoloring-via-shader)
    - 10.4 [LOD Policy](#104-lod-policy)
    - 10.5 [Sprite Pooling](#105-sprite-pooling)
    - 10.6 [TypeScript Implementation](#106-typescript-implementation)
11. [Functional Requirements](#11-functional-requirements)
    - 11.1 [FR-CIV-ASSET-001 through FR-CIV-ASSET-010](#111-fr-civ-asset-001-through-fr-civ-asset-010)
    - 11.2 [FR-CIV-ASSET-011 through FR-CIV-ASSET-020](#112-fr-civ-asset-011-through-fr-civ-asset-020)
12. [Test Harness](#12-test-harness)
    - 12.1 [Unit Tests](#121-unit-tests)
    - 12.2 [Visual Regression Tests](#122-visual-regression-tests)
    - 12.3 [Performance Tests](#123-performance-tests)
    - 12.4 [Integration Tests](#124-integration-tests)
    - 12.5 [CI Gate Policy](#125-ci-gate-policy)
13. [Determinism Rules](#13-determinism-rules)
    - 13.1 [Determinism Contract](#131-determinism-contract)
    - 13.2 [Non-Determinism Sources and Mitigations](#132-non-determinism-sources-and-mitigations)
    - 13.3 [BLAKE3 Verification Protocol](#133-blake3-verification-protocol)
    - 13.4 [Seed Management](#134-seed-management)
14. [FR Traceability](#14-fr-traceability)

---

## 1. Overview and Design Principles

### 1.1 Pipeline Philosophy

CivLab's 2D asset pipeline is built around a single governing principle: **author once as vector, bake once at build time, serve deterministically at runtime.** No SVG parsing, no runtime rasterization, no dynamic image generation happens in the browser or on the game server. All visual assets are fully resolved to packed PNG atlases before any binary ships.

This principle derives from three hard constraints inherited from the CivLab architecture:

**Constraint 1 — Determinism (from CIV-0001):** The simulation core is a deterministic, fixed-timestep tick engine. The visual representation of that simulation must be equally deterministic. A given set of template parameters must always produce the same sprite, byte-for-byte. This is required for replay validation, CI regression detection, and cross-platform consistency.

**Constraint 2 — Performance (from CIV-0500):** The RTS client targets 60 FPS rendering of hex grids with potentially thousands of visible tiles. Runtime SVG parsing and rasterization would consume 30–60 ms per frame on complex maps — an unacceptable frame budget violation. Pre-baked atlases eliminate this cost entirely.

**Constraint 3 — Zoom Architecture (from CIV-0300):** CivLab's three-zoom UI architecture (Strategic/Nation, Tactical/City, Citizen/Research) requires distinct sprite sets at each zoom level. These cannot be generated lazily without introducing visible pop-in. Pre-baked atlases loaded per zoom level eliminate pop-in while keeping VRAM usage bounded.

The SVG-first authoring approach delivers resolution independence, easy parameterization (color injection, tier markers, ideology icons), and a clean separation between art direction and engine concerns. Artist tools (Inkscape, Figma export, procedural generators) target `.svg.j2` templates; the pipeline handles all downstream rasterization.

### 1.2 Pipeline Stages at a Glance

The full pipeline from source SVG template to runtime sprite consists of six sequential stages, with one optional enhancement pass:

```
┌─────────────────────────────────────────────────────────────────────┐
│                    2D ASSET PIPELINE OVERVIEW                       │
└─────────────────────────────────────────────────────────────────────┘

 [1] SVG Templates (.svg.j2)
      │  Tera/Jinja2 templating engine
      │  Injects terrain_type, nation_color_hex, population_tier, etc.
      ▼
 [2] Rendered SVGs (.svg)
      │  Parameter-resolved static SVG files
      │  One file per unique parameter combination
      ▼
 [3] resvg Rasterizer (Rust)
      │  4× supersampling → downscale to target size
      │  Bundled fonts; sRGB output
      │  ~2ms per 128×128 sprite (M2 Mac, parallelized via rayon)
      ▼
 [4] RGBA PNGs (pre-background-removal)
      │
      ├─── [4a] SD XL Enhancement (OPTIONAL — --quality=high only)
      │         ControlNet conditioning on resvg output
      │         Prompt-guided texture/detail enhancement
      │         Seed-captured for reproducibility
      │
      ▼
 [5] rembg Background Removal (Python, U2Net)
      │  Transparent background; alpha coverage QA gate
      │  Async batch; max 4 parallel GPU tasks
      ▼
 [6] Clean RGBA PNGs (transparent background)
      │
      ▼
 [7] imagequant Palette Quantization (Rust)
      │  16-color (sprites) or 32-color (buildings)
      │  Floyd-Steinberg dithering @ 0.4 strength
      │  Nation primary/secondary colors as forced palette entries
      ▼
 [8] Indexed PNGs + Palette JSONs
      │
      ▼
 [9] texture_packer Atlas Packing (Rust, MaxRects)
      │  2048×2048 terrain | 1024×1024 buildings | 512×512 citizens
      │  Power-of-two constraint (WebGL compatibility)
      │  UV rect + pivot + trim rect metadata per sprite
      ▼
[10] Atlas PNGs + Atlas JSONs + asset_manifest.json
      │
      ▼
[11] Runtime (Pixi.js v8)
      SpriteManager loads atlases on init
      Zoom-level swap | Nation recoloring shader | Sprite pooling
```

### 1.3 Zoom-Level Output Contract

The pipeline produces distinct sprite sets for each of CivLab's three zoom levels. These match the zoom architecture defined in CIV-0300.

| Zoom Level | Name | Primary Use | Sprite Size | Atlas Target |
|------------|------|-------------|-------------|--------------|
| Zoom 1 | Strategic / Nation | Nation icons, territory markers, strategic overlays | 64×64 px | `terrain_atlas` (2048×2048) |
| Zoom 2 | Tactical / City | City tiles, building sprites, terrain detail tiles | 128×128 px | `buildings_atlas` (1024×1024) |
| Zoom 3 | Citizen / Research | Citizen portrait icons, unit close-ups, research nodes | 64×64 px | `citizens_atlas` (512×512) |

All zoom levels are pre-baked. The runtime `SpriteManager` holds all three atlas sets in memory simultaneously; zoom transitions are atlas swaps, not reloads.

**Size Rationale:**

- Zoom 1 (64×64): Nation icons must be readable at high map zoom-out; 64×64 at 1× device pixel ratio (DPR) displays cleanly on 1080p and retina. Larger sizes waste VRAM at strategic scale.
- Zoom 2 (128×128): City and building tiles need sufficient detail for tacticians to distinguish production buildings, barracks, markets, etc. 128×128 provides this at Zoom 2 camera scale.
- Zoom 3 (64×64): Citizen portraits are rendered as part of the Citizen Micro-View panel (CIV-0300 §9.5). 64×64 is sufficient for portrait clarity in the panel grid.

### 1.4 Build-Time vs Runtime Contract

**CRITICAL RULE: No SVG parsing at runtime. No rasterization at runtime. No dynamic image generation at runtime.**

This contract is enforced at the architecture level:

| Operation | Build Time | Runtime | Enforcement |
|-----------|-----------|---------|-------------|
| SVG template injection | YES | NO | No SVG parser in web bundle |
| SVG rasterization | YES | NO | No resvg in web bundle |
| Background removal | YES | NO | No rembg in web bundle |
| Palette quantization | YES | NO | No imagequant in web bundle |
| Atlas packing | YES | NO | No texture_packer in web bundle |
| Atlas loading | NO | YES | `Assets.load()` on init |
| Sprite UV lookup | NO | YES | Constant-time hash lookup |
| Nation recoloring | NO | YES | Fragment shader (GPU) |
| Zoom-level switching | NO | YES | Atlas swap in SpriteManager |

Any PR that introduces runtime SVG parsing, rasterization, or dynamic atlas generation MUST be rejected in code review. This rule exists in the CI lint configuration as a bundle-size and import-pattern check.

### 1.5 Directory Structure

```
civ/
├── assets/
│   └── templates/               # Source SVG templates (.svg.j2)
│       ├── terrain/
│       │   ├── plains.svg.j2
│       │   ├── desert.svg.j2
│       │   ├── forest.svg.j2
│       │   ├── tundra.svg.j2
│       │   ├── mountain.svg.j2
│       │   ├── ocean.svg.j2
│       │   ├── river.svg.j2
│       │   └── volcanic.svg.j2
│       ├── buildings/
│       │   ├── city_center.svg.j2
│       │   ├── farm.svg.j2
│       │   ├── mine.svg.j2
│       │   ├── barracks.svg.j2
│       │   ├── market.svg.j2
│       │   ├── library.svg.j2
│       │   ├── temple.svg.j2
│       │   ├── harbor.svg.j2
│       │   ├── workshop.svg.j2
│       │   ├── granary.svg.j2
│       │   ├── palace.svg.j2
│       │   └── wall.svg.j2
│       └── citizens/
│           ├── farmer.svg.j2
│           ├── artisan.svg.j2
│           ├── merchant.svg.j2
│           ├── scholar.svg.j2
│           └── soldier.svg.j2
├── scripts/
│   ├── svg_inject.py            # Stage 1: Jinja2 template injection
│   ├── rembg_batch.py           # Stage 3: Background removal
│   └── gen_manifest.py          # Stage 6: Manifest generation
├── src/
│   └── bin/
│       ├── resvg_batch.rs       # Stage 2: Batch rasterization
│       ├── quantize.rs          # Stage 4: Palette quantization
│       └── atlas_pack.rs        # Stage 5: Atlas packing
├── build/                       # Intermediate build artifacts (gitignored)
│   ├── svg/                     # Injected SVGs
│   ├── png/                     # Raw rasterized PNGs
│   ├── sdxl/                    # SD XL enhanced PNGs (optional)
│   ├── clean/                   # Background-removed PNGs
│   └── quantized/               # Quantized indexed PNGs
└── web/
    └── public/
        └── atlases/             # Final shipped artifacts
            ├── terrain_atlas.png
            ├── terrain_atlas.json
            ├── buildings_atlas.png
            ├── buildings_atlas.json
            ├── citizens_atlas.png
            ├── citizens_atlas.json
            └── asset_manifest.json
```

---

## 2. SVG Template System

### 2.1 Templating Engine

**Primary Engine:** Tera 1.x (Rust) for the full Rust pipeline path.
**Preprocessing Fallback:** Jinja2 (Python 3.12+) for the Python preprocessing step when artists iterate on templates independently of the full Rust build.

Both engines consume the same `.svg.j2` template syntax. Tera implements a Jinja2-compatible syntax as its design goal, so templates are valid for both engines. Any Tera-specific extensions (e.g., `{% set_global %}`) are prohibited to preserve Jinja2 compatibility.

**Template File Extension:** `.svg.j2` — the double extension makes the file type unambiguous: SVG content, Jinja2/Tera syntax.

**Rendering Context:** Each template receives a single JSON object as its context. This object is constructed by `svg_inject.py` from the asset parameter matrix (see Section 2.2).

```python
# scripts/svg_inject.py — context construction example
context = {
    "terrain_type": "plains",
    "terrain_color": "#7ec850",
    "nation_color_hex": "#c8303c",
    "nation_color_secondary_hex": "#f0c040",
    "population_tier": 3,
    "ideology_index": 2,
    "building_level": 1,
    "zoom_level": 2,
    "asset_id": "terrain_plains_z2_n3",
}
```

**Output:** One `.svg` file per unique parameter combination. These files are ephemeral build artifacts, not committed to source control.

### 2.2 Template Variables Reference

The following variables are defined in the pipeline context schema. All templates MUST only reference variables from this table. Referencing undefined variables causes a build failure (Tera strict mode is enabled).

| Variable | Type | Description | Example Values |
|----------|------|-------------|----------------|
| `terrain_type` | string | Terrain type identifier | `"plains"`, `"desert"`, `"forest"`, `"tundra"`, `"mountain"`, `"ocean"`, `"river"`, `"volcanic"` |
| `terrain_color` | hex string | Primary terrain fill color | `"#7ec850"` (plains), `"#d4a040"` (desert) |
| `terrain_color_dark` | hex string | Terrain shadow/edge color (darkened variant) | `"#5aa030"` |
| `terrain_color_light` | hex string | Terrain highlight color (lightened variant) | `"#a0e070"` |
| `nation_color_hex` | hex string | Nation primary color | `"#c8303c"` |
| `nation_color_secondary_hex` | hex string | Nation secondary/accent color | `"#f0c040"` |
| `nation_color_dark_hex` | hex string | Nation primary darkened (borders, outlines) | `"#8a1020"` |
| `population_tier` | integer [1–5] | Population density tier | `1` (sparse) … `5` (megalopolis) |
| `ideology_index` | integer [0–5] | Ideology variant selector | `0` (Neutral), `1` (Liberty), `2` (Authority), `3` (Theocracy), `4` (Republic), `5` (Collectivist) |
| `building_level` | integer [1–4] | Building upgrade tier | `1` (village) … `4` (metropolis) |
| `zoom_level` | integer [1–3] | Target zoom level for this render | `1`, `2`, `3` |
| `citizen_class` | string | Citizen profession class | `"farmer"`, `"artisan"`, `"merchant"`, `"scholar"`, `"soldier"` |
| `asset_id` | string | Unique asset identifier (injected for debugging) | `"terrain_plains_z2"` |
| `icon_codepoint` | string | Fontello icon codepoint (hex) | `"e800"` |
| `show_icon` | boolean | Whether to render the icon layer | `true`, `false` |
| `opacity_overlay` | float [0.0–1.0] | Overlay layer opacity | `0.4` |

**Computed Variants:** The `terrain_color_dark` and `terrain_color_light` variants are computed by `svg_inject.py` from `terrain_color` using HSL manipulation (darken by 20%, lighten by 20%). Templates MUST NOT recompute these inline.

### 2.3 Template Catalog

**Terrain Templates (8 types × 3 zoom levels = 24 renders per nation)**

| Template File | Terrain Type | Dominant Color | Notes |
|---------------|-------------|----------------|-------|
| `terrain/plains.svg.j2` | Plains | `#7ec850` | Base grassland; most common tile |
| `terrain/desert.svg.j2` | Desert | `#d4a040` | Sand texture via SVG pattern fill |
| `terrain/forest.svg.j2` | Forest | `#2d7a3c` | Tree canopy silhouette overlay |
| `terrain/tundra.svg.j2` | Tundra | `#b8c8d0` | Ice/snow texture; reduced saturation |
| `terrain/mountain.svg.j2` | Mountain | `#8090a0` | Elevation contour paths |
| `terrain/ocean.svg.j2` | Ocean | `#2050a0` | Wave pattern animation (baked as static frame) |
| `terrain/river.svg.j2` | River | `#4080c0` | River path through terrain base |
| `terrain/volcanic.svg.j2` | Volcanic | `#402020` | Lava glow effect via radial gradient |

**Building Templates (12 types × 4 levels × zoom 2 only = 48 variants)**

| Template File | Building Type | Level Range | Zoom |
|---------------|--------------|-------------|------|
| `buildings/city_center.svg.j2` | City Center | 1–4 | 2 |
| `buildings/farm.svg.j2` | Farm | 1–4 | 2 |
| `buildings/mine.svg.j2` | Mine | 1–4 | 2 |
| `buildings/barracks.svg.j2` | Barracks | 1–4 | 2 |
| `buildings/market.svg.j2` | Market | 1–4 | 2 |
| `buildings/library.svg.j2` | Library | 1–4 | 2 |
| `buildings/temple.svg.j2` | Temple | 1–4 | 2 |
| `buildings/harbor.svg.j2` | Harbor | 1–4 | 2 |
| `buildings/workshop.svg.j2` | Workshop | 1–4 | 2 |
| `buildings/granary.svg.j2` | Granary | 1–4 | 2 |
| `buildings/palace.svg.j2` | Palace | 1–4 | 2 |
| `buildings/wall.svg.j2` | Wall | 1–4 | 2 |

**Citizen Templates (5 classes × 6 ideology variants × zoom 3 = 30 variants)**

| Template File | Citizen Class | Notes |
|---------------|--------------|-------|
| `citizens/farmer.svg.j2` | Farmer | Agricultural tools icon overlay |
| `citizens/artisan.svg.j2` | Artisan | Craft/workshop tools overlay |
| `citizens/merchant.svg.j2` | Merchant | Currency/trade icon overlay |
| `citizens/scholar.svg.j2` | Scholar | Book/torch icon overlay |
| `citizens/soldier.svg.j2` | Soldier | Weapon/shield overlay |

**Nation Icon Overlays (Zoom 1):** Nation-level icons are generated from terrain templates with a nation color overlay applied. Each nation sees its primary color injected into the terrain tile's nation zone. 6 ideology variants create visual differentiation in national icon framing.

**Total Unique Renders (baseline, no SDXL):**
- Terrain: 8 types × 3 zoom levels = 24 base renders; nation-specific color injections multiply by number of active nations (up to 16). Color injection uses shader at runtime; base renders do not multiply.
- Buildings: 12 types × 4 levels = 48 renders
- Citizens: 5 classes × 6 ideology variants = 30 renders
- **Baseline total: ~102 unique sprite renders**

### 2.4 Template Layer Architecture

Every SVG template follows a consistent layer structure using SVG `<g>` groups with explicit IDs. This structure enables:
- Predictable rendering order (painter's algorithm, bottom-to-top)
- Layer-targeted modifications (the injector can toggle layers by boolean variable)
- Consistent UV pivot point placement (always center of `base` layer bounding box)

**Layer Stack (bottom to top):**

```
<svg viewBox="0 0 128 128" ...>
  <!-- Layer 0: Base terrain/building silhouette -->
  <g id="base">
    <path ... fill="{{terrain_color}}" />
  </g>

  <!-- Layer 1: Texture/detail overlays (SVG patterns or paths) -->
  <g id="texture" opacity="{{opacity_overlay}}">
    <path ... fill="{{terrain_color_dark}}" />
  </g>

  <!-- Layer 2: Nation color zone (borders, banners, flags) -->
  <g id="nation_zone">
    <path ... fill="{{nation_color_hex}}" />
    <path ... fill="{{nation_color_secondary_hex}}" />
  </g>

  <!-- Layer 3: Icon overlay (Fontello codepoint) -->
  <g id="icon_layer" display="{% if show_icon %}inline{% else %}none{% endif %}">
    <text font-family="civlab-icons" font-size="24"
          x="64" y="88" text-anchor="middle"
          fill="{{nation_color_secondary_hex}}">&#x{{icon_codepoint}};</text>
  </g>

  <!-- Layer 4: Population tier marker (dots or rings) -->
  <g id="population_marker">
    {% for i in range(end=population_tier) %}
    <circle cx="{{loop.index * 10 + 8}}" cy="120" r="3"
            fill="{{nation_color_hex}}" />
    {% endfor %}
  </g>
</svg>
```

**Pivot Point Convention:** The sprite pivot point (for Pixi.js anchor) is always `(0.5, 0.5)` — the center of the `base` layer's bounding box, which is always `(64, 64)` for 128×128 output. This is enforced by the atlas packer which records pivot in metadata.

### 2.5 Example Templates

**Terrain: Plains (terrain/plains.svg.j2)**

```xml
<?xml version="1.0" encoding="UTF-8"?>
<!--
  CivLab SVG Template: Plains Terrain Tile
  Template vars: terrain_color, terrain_color_dark, terrain_color_light,
                 nation_color_hex, nation_color_secondary_hex,
                 population_tier, show_icon, icon_codepoint, opacity_overlay
-->
<svg viewBox="0 0 128 128" width="128" height="128"
     xmlns="http://www.w3.org/2000/svg"
     xmlns:xlink="http://www.w3.org/1999/xlink">

  <defs>
    <!-- Grass texture pattern -->
    <pattern id="grass_pattern" x="0" y="0" width="16" height="16"
             patternUnits="userSpaceOnUse">
      <rect width="16" height="16" fill="{{terrain_color}}" />
      <line x1="4" y1="12" x2="6" y2="4" stroke="{{terrain_color_dark}}"
            stroke-width="1" opacity="0.6" />
      <line x1="9" y1="14" x2="11" y2="6" stroke="{{terrain_color_dark}}"
            stroke-width="1" opacity="0.4" />
      <line x1="13" y1="11" x2="14" y2="5" stroke="{{terrain_color_dark}}"
            stroke-width="1" opacity="0.5" />
    </pattern>

    <!-- Hex clip shape -->
    <clipPath id="hex_clip">
      <polygon points="64,4 124,34 124,94 64,124 4,94 4,34" />
    </clipPath>
  </defs>

  <!-- Layer 0: Base hex fill -->
  <g id="base" clip-path="url(#hex_clip)">
    <rect width="128" height="128" fill="url(#grass_pattern)" />
  </g>

  <!-- Layer 1: Terrain edge shading -->
  <g id="texture" opacity="{{opacity_overlay}}">
    <polygon points="64,4 124,34 124,94 64,124 4,94 4,34"
             fill="none" stroke="{{terrain_color_dark}}" stroke-width="3" />
  </g>

  <!-- Layer 2: Nation color zone (corner banner) -->
  <g id="nation_zone">
    <polygon points="100,4 124,4 124,28"
             fill="{{nation_color_hex}}" opacity="0.85" />
    <polygon points="108,4 124,4 124,16"
             fill="{{nation_color_secondary_hex}}" opacity="0.9" />
  </g>

  <!-- Layer 3: Icon overlay -->
  <g id="icon_layer" display="{% if show_icon %}inline{% else %}none{% endif %}">
    <text font-family="civlab-icons" font-size="20"
          x="64" y="84" text-anchor="middle" dominant-baseline="middle"
          fill="{{nation_color_secondary_hex}}" opacity="0.9">
      &#x{{icon_codepoint}};
    </text>
  </g>

  <!-- Layer 4: Population tier dots -->
  <g id="population_marker">
    {% for i in range(end=population_tier) %}
    <circle cx="{{8 + loop.index0 * 11}}" cy="119" r="3.5"
            fill="{{nation_color_hex}}" stroke="{{nation_color_secondary_hex}}"
            stroke-width="1" />
    {% endfor %}
  </g>
</svg>
```

**Building: City Center (buildings/city_center.svg.j2)**

```xml
<?xml version="1.0" encoding="UTF-8"?>
<!--
  CivLab SVG Template: City Center Building
  Template vars: nation_color_hex, nation_color_secondary_hex,
                 nation_color_dark_hex, building_level, ideology_index
-->
<svg viewBox="0 0 128 128" width="128" height="128"
     xmlns="http://www.w3.org/2000/svg">

  <defs>
    <!-- Building color driven by nation -->
    <!-- Level-scaled tower heights: level 1=40px, 2=56px, 3=72px, 4=88px -->
    {% set tower_height = 40 + (building_level - 1) * 16 %}
    {% set tower_y = 88 - tower_height %}
  </defs>

  <!-- Layer 0: Ground base plate -->
  <g id="base">
    <rect x="20" y="90" width="88" height="14" rx="2"
          fill="{{nation_color_dark_hex}}" />
  </g>

  <!-- Layer 1: Main tower body (level-scaled) -->
  <g id="texture">
    <rect x="44" y="{{tower_y}}" width="40" height="{{tower_height}}"
          fill="{{nation_color_hex}}" />
    <!-- Tower windows -->
    {% if building_level >= 2 %}
    <rect x="52" y="{{tower_y + 8}}" width="8" height="10" rx="1"
          fill="{{nation_color_secondary_hex}}" opacity="0.8" />
    <rect x="68" y="{{tower_y + 8}}" width="8" height="10" rx="1"
          fill="{{nation_color_secondary_hex}}" opacity="0.8" />
    {% endif %}
    {% if building_level >= 3 %}
    <rect x="52" y="{{tower_y + 24}}" width="8" height="10" rx="1"
          fill="{{nation_color_secondary_hex}}" opacity="0.8" />
    <rect x="68" y="{{tower_y + 24}}" width="8" height="10" rx="1"
          fill="{{nation_color_secondary_hex}}" opacity="0.8" />
    {% endif %}
    <!-- Level 4: Flag pole -->
    {% if building_level == 4 %}
    <line x1="64" y1="{{tower_y}}" x2="64" y2="{{tower_y - 16}}"
          stroke="{{nation_color_dark_hex}}" stroke-width="2" />
    <polygon points="64,{{tower_y - 16}} 80,{{tower_y - 10}} 64,{{tower_y - 4}}"
             fill="{{nation_color_secondary_hex}}" />
    {% endif %}
  </g>

  <!-- Layer 2: Nation color accent on roofline -->
  <g id="nation_zone">
    <rect x="40" y="{{tower_y - 4}}" width="48" height="8"
          fill="{{nation_color_secondary_hex}}" />
  </g>

  <!-- Layer 3: Ideology icon on building face -->
  <g id="icon_layer" display="{% if show_icon %}inline{% else %}none{% endif %}">
    <text font-family="civlab-icons" font-size="16"
          x="64" y="{{tower_y + tower_height // 2 + 6}}"
          text-anchor="middle" fill="white" opacity="0.7">
      &#x{{icon_codepoint}};
    </text>
  </g>

  <!-- Layer 4: Population marker (building level indicator dots) -->
  <g id="population_marker">
    {% for i in range(end=building_level) %}
    <circle cx="{{52 + loop.index0 * 9}}" cy="110" r="2.5"
            fill="{{nation_color_secondary_hex}}" />
    {% endfor %}
  </g>
</svg>
```

### 2.6 Template Validation Rules

Templates MUST pass the following validation checks before being accepted into the build:

1. **Valid SVG:** Parses without error as SVG 1.1 (validated by `svg_inject.py` using `lxml`).
2. **ViewBox present:** `viewBox` attribute MUST be present on the root `<svg>` element.
3. **Layer IDs present:** All five layer `<g>` groups (`base`, `texture`, `nation_zone`, `icon_layer`, `population_marker`) MUST be present. Additional layers are permitted.
4. **No embedded raster images:** `<image>` elements with base64 data URIs are forbidden. External image refs are forbidden. All imagery must be vector paths, patterns, or text.
5. **No JavaScript:** `<script>` elements are forbidden.
6. **No external references:** `xlink:href` references to external files are forbidden; only internal `#id` references are permitted.
7. **Variable coverage:** Every variable referenced with `{{var}}` or `{% if var %}` in the template MUST be present in the context schema (Section 2.2). Tera strict mode enforces this at render time.
8. **Font references:** The only permitted `font-family` values are `"civlab-icons"`, `"Noto Sans"`, and `"Noto Sans Mono"`. No system font fallbacks.

Validation is run as a pre-commit hook (`hooks/pre-commit-svg-validate.sh`) and in the CI pipeline before Stage 1.

---

## 3. resvg Renderer

### 3.1 Library Configuration

**Library:** `resvg` version `0.39.0`
**Backend:** pixman (software rendering; no GPU dependency in build pipeline)
**Crate:** `resvg = "0.39"` in `Cargo.toml`

```toml
[dependencies]
resvg = "0.39"
tiny-skia = "0.11"
fontdb = "0.16"
rayon = "1.8"
anyhow = "1.0"
```

`resvg` was selected over alternatives for the following reasons:

| Criterion | resvg | librsvg | Inkscape CLI | Browser headless |
|-----------|-------|---------|-------------|-----------------|
| Deterministic output | YES | Partial | NO (platform-dependent) | NO |
| Rust-native | YES | No (C + GObject) | No | No |
| No X11/display dependency | YES | Requires GTK | Requires Xvfb | Requires display |
| CI compatibility | YES | Fragile | Fragile | Heavy |
| SVG 1.1 coverage | ~95% | ~97% | ~98% | ~99% |
| Custom font loading | YES | YES | NO | YES |

The 2–4% SVG 1.1 coverage gap between resvg and full browser rendering is acceptable because templates are validated against resvg's supported feature set. Templates MUST NOT use unsupported SVG features (see Section 2.6 validation rules).

### 3.2 Rasterization Parameters

**Supersampling Strategy:** 4× linear supersampling. For a target output of 128×128 pixels, resvg renders at 512×512 then downscales using a Lanczos3 filter to produce the final 128×128 output. This eliminates aliasing on diagonal paths and curves common in terrain tile outlines.

```rust
pub struct RasterizationConfig {
    /// Target output dimensions in pixels
    pub output_size: (u32, u32),
    /// Supersampling multiplier (4 = render 4x, then downscale)
    pub supersample_factor: u32,
    /// Downscale filter (Lanczos3 for quality; Bilinear for speed)
    pub downscale_filter: DownscaleFilter,
}

impl Default for RasterizationConfig {
    fn default() -> Self {
        Self {
            output_size: (128, 128),
            supersample_factor: 4,
            downscale_filter: DownscaleFilter::Lanczos3,
        }
    }
}
```

**Zoom-Level Configs:**

| Zoom | Output Size | Render Size (4×) | Filter |
|------|-------------|------------------|--------|
| 1 (Nation icons) | 64×64 | 256×256 | Lanczos3 |
| 2 (City tiles) | 128×128 | 512×512 | Lanczos3 |
| 3 (Citizen portraits) | 64×64 | 256×256 | Lanczos3 |

### 3.3 Font Handling

All fonts are bundled in the repository under `assets/fonts/`. The `fontdb::Database` is populated at process startup from these bundled files. No system fonts are consulted. This is mandatory for cross-platform determinism.

**Bundled Fonts:**

| Font | File | Usage |
|------|------|-------|
| Noto Sans Regular | `assets/fonts/NotoSans-Regular.ttf` | UI labels in templates |
| Noto Sans Bold | `assets/fonts/NotoSans-Bold.ttf` | Title/header labels |
| Noto Sans Mono | `assets/fonts/NotoSansMono-Regular.ttf` | Code/numeric labels |
| civlab-icons | `assets/fonts/civlab-icons.ttf` | Game icon glyphs (Fontello-generated) |

**civlab-icons Glyph Map:**

| Icon | Codepoint | Use |
|------|-----------|-----|
| Plains leaf | `e800` | Plains terrain |
| Desert sun | `e801` | Desert terrain |
| Forest tree | `e802` | Forest terrain |
| Snowflake | `e803` | Tundra terrain |
| Mountain peak | `e804` | Mountain terrain |
| Wave | `e805` | Ocean terrain |
| River fork | `e806` | River terrain |
| Flame | `e807` | Volcanic terrain |
| Hammer | `e810` | Workshop/Mine |
| Wheat | `e811` | Farm/Granary |
| Sword | `e812` | Barracks |
| Book | `e813` | Library |
| Temple | `e814` | Temple |
| Anchor | `e815` | Harbor |
| Crown | `e816` | Palace |
| Shield | `e817` | Wall |
| Coin | `e818` | Market |
| Flag | `e819` | City Center |
| Person (farmer) | `e820` | Farmer citizen |
| Tools (artisan) | `e821` | Artisan citizen |
| Scales (merchant) | `e822` | Merchant citizen |
| Torch (scholar) | `e823` | Scholar citizen |
| Spear (soldier) | `e824` | Soldier citizen |

### 3.4 Color Profile Policy

**Output Color Space:** sRGB, 8-bit per channel.
**No ICC profiles** are embedded in output PNGs. The game client assumes sRGB throughout.
**Alpha:** Straight (un-premultiplied) alpha. Pixi.js handles premultiplication internally.

resvg renders to tiny-skia's `Pixmap` type, which is RGBA8 with premultiplied alpha internally. The pipeline post-processes to straight alpha before writing PNG:

```rust
fn premul_to_straight(rgba: &mut [u8]) {
    for chunk in rgba.chunks_exact_mut(4) {
        let a = chunk[3];
        if a > 0 {
            chunk[0] = (chunk[0] as u16 * 255 / a as u16) as u8;
            chunk[1] = (chunk[1] as u16 * 255 / a as u16) as u8;
            chunk[2] = (chunk[2] as u16 * 255 / a as u16) as u8;
        }
    }
}
```

### 3.5 Rust Implementation

**Binary:** `src/bin/resvg_batch.rs`

```rust
use anyhow::{Context, Result};
use fontdb::Database;
use rayon::prelude::*;
use resvg::usvg::{self, Options, Tree};
use std::path::{Path, PathBuf};
use tiny_skia::{Pixmap, Transform};

pub struct SvgRenderer {
    /// Scale factor: output_size * supersample_factor
    pub scale_factor: f32,
    /// Final output dimensions after downscale
    pub output_size: (u32, u32),
    /// Loaded font database (bundled fonts only)
    pub font_db: Database,
    /// Downscale filter for supersampling
    pub downscale_filter: image::imageops::FilterType,
}

impl SvgRenderer {
    pub fn new(output_size: (u32, u32), supersample_factor: u32) -> Result<Self> {
        let mut font_db = Database::new();
        font_db.load_fonts_dir("assets/fonts/");

        if font_db.is_empty() {
            anyhow::bail!("Font database is empty — bundled fonts not found at assets/fonts/");
        }

        Ok(Self {
            scale_factor: supersample_factor as f32,
            output_size,
            font_db,
            downscale_filter: image::imageops::FilterType::Lanczos3,
        })
    }

    pub fn render(&self, svg_path: &Path) -> Result<image::RgbaImage> {
        let svg_data = std::fs::read(svg_path)
            .with_context(|| format!("Failed to read SVG: {}", svg_path.display()))?;

        let options = Options {
            fontdb: std::sync::Arc::new(self.font_db.clone()),
            ..Options::default()
        };

        let tree = Tree::from_data(&svg_data, &options)
            .with_context(|| format!("Failed to parse SVG: {}", svg_path.display()))?;

        let render_w = (self.output_size.0 as f32 * self.scale_factor) as u32;
        let render_h = (self.output_size.1 as f32 * self.scale_factor) as u32;

        let mut pixmap = Pixmap::new(render_w, render_h)
            .context("Failed to allocate pixmap")?;

        let scale = Transform::from_scale(self.scale_factor, self.scale_factor);
        resvg::render(&tree, scale, &mut pixmap.as_mut());

        // Convert tiny-skia RGBA (premultiplied) to straight alpha
        let mut rgba_data = pixmap.take();
        premul_to_straight(&mut rgba_data);

        let render_img = image::RgbaImage::from_raw(render_w, render_h, rgba_data)
            .context("Failed to create image from pixmap data")?;

        // Downscale from supersampled size to target output size
        let output_img = image::imageops::resize(
            &render_img,
            self.output_size.0,
            self.output_size.1,
            self.downscale_filter,
        );

        Ok(output_img)
    }
}

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 3 {
        anyhow::bail!("Usage: resvg_batch <input_dir> <output_dir>");
    }

    let input_dir = PathBuf::from(&args[1]);
    let output_dir = PathBuf::from(&args[2]);
    std::fs::create_dir_all(&output_dir)?;

    // Collect all .svg files in input_dir
    let svg_files: Vec<PathBuf> = std::fs::read_dir(&input_dir)?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().map_or(false, |e| e == "svg"))
        .collect();

    if svg_files.is_empty() {
        anyhow::bail!("No .svg files found in {}", input_dir.display());
    }

    println!("Rendering {} SVGs...", svg_files.len());

    // Parallel render via rayon
    let renderer = SvgRenderer::new((128, 128), 4)?;
    let errors: Vec<_> = svg_files
        .par_iter()
        .filter_map(|svg_path| {
            let stem = svg_path.file_stem()?.to_str()?.to_owned();
            let out_path = output_dir.join(format!("{}.png", stem));
            match renderer.render(svg_path) {
                Ok(img) => {
                    img.save(&out_path).err()
                        .map(|e| format!("{}: {}", svg_path.display(), e))
                }
                Err(e) => Some(format!("{}: {}", svg_path.display(), e)),
            }
        })
        .collect();

    if !errors.is_empty() {
        for err in &errors {
            eprintln!("ERROR: {}", err);
        }
        anyhow::bail!("{} render errors — aborting", errors.len());
    }

    println!("Done. {} sprites rendered to {}", svg_files.len(), output_dir.display());
    Ok(())
}
```

### 3.6 Performance Characteristics

**Benchmark Environment:** Apple M2 Pro, 16 GB RAM, macOS 15.

| Operation | Single-threaded | Parallel (rayon, 10 threads) |
|-----------|----------------|------------------------------|
| 128×128 sprite render (4× SS) | ~2 ms | ~0.3 ms (effective/sprite) |
| 64×64 sprite render (4× SS) | ~0.8 ms | ~0.12 ms (effective/sprite) |
| 200-sprite batch | ~400 ms | ~35 ms |
| Font database load | ~50 ms (once) | N/A (shared) |
| SVG parse + tree build | ~0.5 ms/file | Parallel |

**CI Performance Target:** Full 200-sprite render batch (all baseline sprites) MUST complete in under 30 seconds on the CI runner (4-core x86_64 Linux). This is validated by the performance test suite (Section 12.3).

### 3.7 Error Handling

The `resvg_batch` binary follows the fail-fast philosophy. Any error in any sprite causes the entire batch to fail with a non-zero exit code and a descriptive error message. Silent skipping of failed sprites is forbidden.

**Error Categories:**

| Error | Behavior |
|-------|----------|
| SVG file not found | Fatal — log path, exit 1 |
| SVG parse failure (malformed XML) | Fatal — log path + parse error, exit 1 |
| SVG feature not supported by resvg | Fatal — log path + unsupported feature, exit 1 |
| Font not found in fontdb | Fatal — log missing font family, exit 1 |
| Pixmap allocation failure (OOM) | Fatal — log dimensions + available memory, exit 1 |
| PNG write failure (disk full) | Fatal — log path + OS error, exit 1 |

---

## 4. SD XL Agentic Enhancement (Optional Pass)

### 4.1 Trigger Conditions

The SD XL enhancement pass is **not part of the default build pipeline**. It MUST only execute when one of the following conditions is met:

1. **Explicit flag:** `--quality=high` is passed to the `assets:generate` task.
2. **Hero asset flag:** The asset's entry in `asset_parameters.yaml` has `hero: true` set.
3. **Manual override:** The `CIVLAB_SDXL_ENABLE=1` environment variable is set.

In CI, the default pipeline (`--quality=standard`) NEVER triggers the SDXL pass. This is enforced by the Taskfile condition guard:

```yaml
assets:generate:sdxl:
  cmds:
    - python3 scripts/sdxl_enhance.py build/png/ build/sdxl/
  preconditions:
    - sh: '[ "$CIVLAB_QUALITY" = "high" ] || [ -n "$CIVLAB_SDXL_ENABLE" ]'
      msg: "SDXL pass requires --quality=high or CIVLAB_SDXL_ENABLE=1"
```

### 4.2 Model Configuration

**Model:** Stable Diffusion XL 1.0 (SDXL base + refiner)
**Inference Backend:** SDXL API (via Replicate or equivalent hosted inference endpoint; configurable via `CIVLAB_SDXL_API_ENDPOINT` env var)
**Resolution:** 1024×1024 (SDXL native resolution) → downscaled to target sprite size
**Steps:** 30 (balanced quality/cost)
**CFG Scale:** 7.5

```python
SDXL_CONFIG = {
    "model": "stability-ai/sdxl",
    "version": "sdxl-1.0",
    "steps": 30,
    "cfg_scale": 7.5,
    "scheduler": "DPMSolverMultistep",
    "output_format": "png",
    "output_quality": 100,
}
```

### 4.3 ControlNet Integration

**ControlNet Type:** Depth + Canny (dual conditioning for structure preservation)

The resvg output PNG serves as the ControlNet conditioning image. This preserves the structural geometry (building silhouette, terrain hex shape, citizen portrait framing) while allowing SDXL to add photorealistic texture, lighting, and detail.

**ControlNet Parameters:**

| Parameter | Value | Rationale |
|-----------|-------|-----------|
| `controlnet_strength` | 0.65 (depth) + 0.70 (canny) | Preserves structure; allows texture freedom |
| `conditioning_scale` | 0.8 | Moderate conditioning; full conditioning causes over-rigidity |
| `control_guidance_start` | 0.0 | Structure enforced from step 0 |
| `control_guidance_end` | 0.6 | Structure released after 60% of steps for natural detail |

The dual-ControlNet approach (depth + canny) is used because:
- **Depth alone:** Adds 3D relief but can distort outlines (hex edge becomes rounded)
- **Canny alone:** Preserves hard edges but loses volumetric depth impression
- **Combined:** Hard edges preserved (canny) + volumetric depth (depth) + SDXL texture freedom

### 4.4 Prompt Template System

Prompts are constructed from a template per asset type. Prompts MUST specify the game art style constraints to prevent SDXL from generating photorealistic renders (which would look out of place in the 2D RTS context).

**Prompt Template:**

```python
PROMPT_TEMPLATES = {
    "terrain": (
        "top-down 2D game tile sprite, {terrain_type} terrain, pixel art adjacent style, "
        "detailed hand-painted texture, {style_descriptor}, isometric-friendly flat perspective, "
        "clean edges, game asset, white background, ultra detailed, 4k"
    ),
    "building": (
        "2D RTS game building sprite, {building_type}, level {building_level} tier, "
        "stylized game art, detailed architecture, {nation_style_descriptor}, "
        "top-down isometric view, clean silhouette, game asset, white background, 4k detail"
    ),
    "citizen": (
        "2D RTS game character portrait, {citizen_class} worker, small icon size, "
        "stylized game art portrait, {ideology_style_descriptor}, "
        "clean round portrait, expressive face, game sprite, white background, 4k"
    ),
}

NEGATIVE_PROMPT = (
    "photorealistic, 3D render, blurry, out of focus, watermark, text overlay, "
    "JPEG artifacts, noisy, grainy, multiple characters, complex background, "
    "dark background, black background"
)
```

**Style Descriptor Map:**

```python
STYLE_DESCRIPTORS = {
    "terrain": {
        "plains": "lush green grasslands, golden hour lighting",
        "desert": "arid sandy dunes, harsh sunlight, ochre tones",
        "forest": "dense tree canopy, dappled light, deep greens",
        "tundra": "icy permafrost, pale blues and whites, frozen",
        "mountain": "rocky peaks, grey stone, dramatic shadows",
        "ocean": "blue waves, foam crests, reflective water",
        "river": "flowing blue water, riverbank vegetation",
        "volcanic": "molten lava, black rock, orange glow",
    },
    "ideology": {
        0: "neutral earth tones, balanced composition",
        1: "bright colors, open composition, warm lighting",
        2: "strong contrasts, bold geometry, authoritarian symbols",
        3: "ornate religious motifs, warm golden accents",
        4: "civic symbols, classical architecture hints",
        5: "collective imagery, shared symbols, muted colors",
    },
}
```

### 4.5 Reproducibility Contract

Every SDXL-enhanced asset MUST have its generation seed captured in `asset_manifest.json`. This ensures:
1. The exact same image can be regenerated on demand.
2. CI can verify that re-running the pipeline with the same seed produces the same output.
3. Artists can request a specific seed for aesthetic preference.

**Seed Policy:**
- Default: seed is generated as `hash(asset_id + pipeline_hash) % (2^32)` — deterministic from asset ID.
- Override: `asset_parameters.yaml` can specify `sdxl_seed: 12345` for individual assets.
- Capture: seed is written to `asset_manifest.json` under the asset's `sdxl_seed` field (null for non-SDXL assets).

```python
import hashlib

def compute_default_seed(asset_id: str, pipeline_hash: str) -> int:
    combined = f"{asset_id}:{pipeline_hash}".encode()
    digest = hashlib.blake2b(combined, digest_size=4).digest()
    return int.from_bytes(digest, "big")
```

### 4.6 Cost Management

**Per-Sprite Cost:** ~$0.003 USD (SDXL API, 30 steps, 1024×1024)
**Baseline Sprite Count:** 102 unique sprites
**Full SDXL Pass Cost:** ~$0.31 USD (acceptable for releases)
**Budget Gate:** Production builds MUST NOT exceed $5.00 USD in SDXL costs per pipeline run. This is enforced by checking the sprite count before invoking SDXL:

```python
MAX_SDXL_BUDGET_USD = 5.00
COST_PER_SPRITE_USD = 0.003

def check_sdxl_budget(sprite_count: int) -> None:
    estimated_cost = sprite_count * COST_PER_SPRITE_USD
    if estimated_cost > MAX_SDXL_BUDGET_USD:
        raise RuntimeError(
            f"SDXL budget exceeded: {sprite_count} sprites × ${COST_PER_SPRITE_USD} "
            f"= ${estimated_cost:.2f} > budget ${MAX_SDXL_BUDGET_USD:.2f}. "
            f"Use --quality=standard or reduce sprite count."
        )
```

### 4.7 Post-Pass Integration

After SDXL generation, the enhanced PNG replaces the resvg output PNG in `build/sdxl/`. The subsequent rembg pass (Section 5) treats SDXL output identically to resvg output — it consumes whatever PNG is in the configured input directory.

The manifest records whether a sprite was SDXL-enhanced via the `sdxl_seed` field: `null` = resvg only; non-null integer = SDXL enhanced with that seed.

---

## 5. rembg Background Removal

### 5.1 Library and Model Selection

**Library:** `rembg` 2.0.x (Python, MIT license)
**Runtime:** Python 3.12+; ONNX Runtime for model inference
**GPU:** CUDA optional (falls back to CPU without code change; GPU gives ~10× speedup)

**Model Selection by Asset Type:**

| Asset Type | Model | Rationale |
|------------|-------|-----------|
| Game sprites (terrain, buildings, citizens) | `u2netp` | Fast inference; acceptable accuracy for game art with clean edges |
| Promotional art / hero assets | `u2net` | Higher accuracy; slower (500ms vs 150ms per image) |
| CI pipeline | `u2netp` | Speed priority in automated builds |

**Model Files:** ONNX models are downloaded on first use and cached in `~/.u2net/`. The CI environment pre-caches models to avoid download latency.

### 5.2 Processing Pipeline

```python
# scripts/rembg_batch.py

from rembg import remove, new_session
from PIL import Image
import asyncio
import aiofiles
from pathlib import Path

async def process_sprite(
    input_path: Path,
    output_path: Path,
    session: any,
    semaphore: asyncio.Semaphore,
) -> None:
    async with semaphore:
        # Read input PNG
        async with aiofiles.open(input_path, "rb") as f:
            input_bytes = await f.read()

        # Run rembg in thread pool (CPU-bound)
        loop = asyncio.get_event_loop()
        output_bytes = await loop.run_in_executor(
            None,
            lambda: remove(input_bytes, session=session)
        )

        # Quality check: alpha coverage
        from io import BytesIO
        img = Image.open(BytesIO(output_bytes)).convert("RGBA")
        alpha = img.split()[3]
        total_pixels = img.width * img.height
        transparent_pixels = sum(1 for p in alpha.getdata() if p < 10)
        opaque_pixels = total_pixels - transparent_pixels
        coverage = opaque_pixels / total_pixels

        if coverage < 0.60:
            raise ValueError(
                f"Alpha coverage {coverage:.1%} < 60% threshold for {input_path.name}. "
                f"Background removal failed — sprite may be mostly transparent."
            )

        # Write output
        async with aiofiles.open(output_path, "wb") as f:
            await f.write(output_bytes)
```

### 5.3 Batch Processing Architecture

Batch processing uses `asyncio` for I/O concurrency with a semaphore limiting concurrent GPU inference tasks. The semaphore limit is 4 by default (tuned for 8 GB VRAM; reduce to 2 for 4 GB VRAM).

```python
async def main(input_dir: Path, output_dir: Path, max_concurrent: int = 4) -> None:
    output_dir.mkdir(parents=True, exist_ok=True)
    session = new_session("u2netp")

    png_files = list(input_dir.glob("*.png"))
    if not png_files:
        raise RuntimeError(f"No PNG files found in {input_dir}")

    semaphore = asyncio.Semaphore(max_concurrent)
    tasks = [
        process_sprite(
            p, output_dir / p.name, session, semaphore
        )
        for p in png_files
    ]

    results = await asyncio.gather(*tasks, return_exceptions=True)
    errors = [
        (png_files[i].name, str(r))
        for i, r in enumerate(results)
        if isinstance(r, Exception)
    ]

    if errors:
        for name, err in errors:
            print(f"ERROR [{name}]: {err}", file=sys.stderr)
        raise SystemExit(f"{len(errors)} background removal errors — pipeline aborted")

    print(f"Background removal complete: {len(png_files)} sprites processed")
```

### 5.4 Quality Gate

The alpha coverage check (≥ 60% opaque pixels) is the primary quality gate. This detects:
- Failed background removal (entire image transparent)
- Partial removal failures (large background regions incorrectly retained or removed)
- Corrupted inputs (empty PNGs, single-color images)

**Secondary Quality Check — Edge Fringing:** After background removal, any pixel with alpha between 10 and 240 that borders a fully transparent pixel is classified as a "fringe pixel." If fringe pixel count exceeds 5% of total sprite perimeter length, a warning is emitted (non-fatal; logged to `build/rembg_warnings.log`).

### 5.5 Error Handling

Any sprite failing the alpha coverage gate causes the entire batch to fail with exit code 1. This matches the fail-fast policy. The error message includes:
- Sprite filename
- Measured alpha coverage
- Minimum required coverage threshold
- Suggestion to check the input PNG for correctness

---

## 6. Palette Quantization

### 6.1 Library Selection

**Primary:** `imagequant` (Rust bindings via `imagequant` crate 4.x)
**Secondary:** `exoquant` (Rust, pure-Rust implementation, used for palette JSON generation)

`imagequant` (libimagequant) is the industry-standard lossy PNG quantizer, originally created for `pngquant`. It produces perceptually optimal 8-bit indexed PNGs with minimal visible quality loss.

```toml
[dependencies]
imagequant = "4.3"
exoquant = "0.2"
png = "0.17"
```

### 6.2 Quantization Parameters

**Palette Size by Asset Type:**

| Asset Type | Palette Size | Rationale |
|------------|-------------|-----------|
| Game sprites (terrain, Zoom 1) | 16 colors | Small memory footprint; strategic view sprites small enough |
| Building tiles (Zoom 2) | 32 colors | More detail at city scale warrants larger palette |
| Citizen portraits (Zoom 3) | 16 colors | Portrait icons small; 16 colors sufficient |
| Hero/promotional assets | 64 colors | Maximum quality for featured art |

**Dithering Configuration:**

```rust
pub struct QuantizationConfig {
    pub palette_size: u8,
    pub dithering_strength: f32,   // 0.0 = none, 1.0 = full Floyd-Steinberg
    pub quality_min: u8,           // 0–100; fail if quality drops below this
    pub quality_max: u8,           // 0–100; target quality
}

impl QuantizationConfig {
    pub fn for_sprite() -> Self {
        Self {
            palette_size: 16,
            dithering_strength: 0.4,
            quality_min: 60,
            quality_max: 85,
        }
    }

    pub fn for_building() -> Self {
        Self {
            palette_size: 32,
            dithering_strength: 0.4,
            quality_min: 65,
            quality_max: 90,
        }
    }
}
```

**Floyd-Steinberg at 0.4 Strength:** Full Floyd-Steinberg (1.0) can produce visible "pepper noise" in game sprites where flat color areas are desirable. 0.4 provides enough dithering to smooth gradients in terrain textures without introducing noise in flat building walls.

### 6.3 Nation Color Binding

The palette quantization step MUST preserve two nation-specific colors as forced palette entries. This ensures that:
1. Nation colors are never quantization-approximated (which would break visual consistency)
2. The runtime shader palette-swap (Section 10.3) operates on exact palette indices

**Forced Palette Entries:**

```rust
fn add_forced_palette_entries(
    liq: &mut imagequant::Attributes,
    nation_primary: [u8; 4],
    nation_secondary: [u8; 4],
) -> Result<()> {
    // Force nation primary color as palette entry 0
    liq.add_fixed_color(imagequant::RGBA {
        r: nation_primary[0],
        g: nation_primary[1],
        b: nation_primary[2],
        a: nation_primary[3],
    })?;

    // Force nation secondary color as palette entry 1
    liq.add_fixed_color(imagequant::RGBA {
        r: nation_secondary[0],
        g: nation_secondary[1],
        b: nation_secondary[2],
        a: nation_secondary[3],
    })?;

    Ok(())
}
```

Palette entries 0 and 1 are reserved for nation primary and secondary colors respectively, across all quantized sprites. The runtime shader knows to replace indices 0 and 1 with the active nation's current colors.

### 6.4 Output Format

For each quantized sprite, two files are produced:

1. **Indexed PNG:** 8-bit indexed PNG with the quantized palette embedded in the `PLTE` chunk.
2. **Palette JSON:** JSON file recording the full palette for the atlas packer and runtime shader.

**Palette JSON Format:**

```json
{
  "asset_id": "terrain_plains_z2",
  "palette_size": 16,
  "nation_primary_index": 0,
  "nation_secondary_index": 1,
  "entries": [
    { "index": 0, "rgba": [200, 48, 60, 255], "role": "nation_primary" },
    { "index": 1, "rgba": [240, 192, 64, 255], "role": "nation_secondary" },
    { "index": 2, "rgba": [126, 200, 80, 255], "role": "terrain" },
    ...
  ]
}
```

### 6.5 Rust Implementation

```rust
// src/bin/quantize.rs

use imagequant::{Attributes, Image, RGBA};
use std::path::{Path, PathBuf};

pub fn quantize_sprite(
    input_png: &Path,
    output_png: &Path,
    output_palette_json: &Path,
    config: &QuantizationConfig,
    nation_primary: [u8; 4],
    nation_secondary: [u8; 4],
) -> anyhow::Result<()> {
    // Load RGBA PNG
    let img = image::open(input_png)?.to_rgba8();
    let (width, height) = img.dimensions();

    let pixels: Vec<RGBA> = img
        .pixels()
        .map(|p| RGBA { r: p[0], g: p[1], b: p[2], a: p[3] })
        .collect();

    let mut liq = Attributes::new();
    liq.set_max_colors(config.palette_size as u32)?;
    liq.set_quality(config.quality_min, config.quality_max)?;

    add_forced_palette_entries(&mut liq, nation_primary, nation_secondary)?;

    let mut liq_image = liq.new_image(pixels.as_slice(), width as usize, height as usize, 0.0)?;
    let mut result = liq.quantize(&mut liq_image)?;
    result.set_dithering_level(config.dithering_strength)?;

    let (palette, pixels) = result.remapped(&mut liq_image)?;

    // Write indexed PNG
    write_indexed_png(output_png, width, height, &palette, &pixels)?;

    // Write palette JSON
    write_palette_json(output_palette_json, &palette, &config)?;

    Ok(())
}
```

---

## 7. Atlas Packing

### 7.1 Library and Algorithm

**Library:** `texture_packer` 0.7.x (Rust)
**Algorithm:** MaxRects (rectangle packing algorithm — optimal bin packing via maximum rectangles heuristic)
**Crate:** `texture_packer = "0.7"` in `Cargo.toml`

```toml
[dependencies]
texture_packer = "0.7"
image = "0.24"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

**Algorithm Selection — MaxRects vs Alternatives:**

| Algorithm | Packing Efficiency | Speed | Implementation |
|-----------|-------------------|-------|----------------|
| MaxRects | 95–98% | O(n²) | texture_packer |
| Guillotine | 85–92% | O(n log n) | Simpler |
| Shelf | 75–85% | O(n) | Too inefficient |
| Optimal (ILP) | 100% | Exponential | Impractical |

MaxRects is the correct choice here. Packing efficiency directly translates to atlas size — higher efficiency means fewer atlases and smaller GPU memory footprint.

**Heuristic:** `BestShortSideFit` (BSSF) — places each rectangle in the position that minimizes the shorter side of the remaining free space. This heuristic produces tightly packed atlases for uniform sprite sizes (all 128×128 or 64×64).

### 7.2 Atlas Size Contracts

Atlas dimensions are fixed by asset category. These are power-of-two sizes required for WebGL texture compatibility.

| Atlas | Dimensions | Sprite Size | Capacity | Asset Category |
|-------|-----------|-------------|----------|----------------|
| `terrain_atlas` | 2048×2048 | 64×64 (Z1) + 128×128 (Z2) | ~512 sprites (Z1) / ~256 sprites (Z2) | Terrain tiles |
| `buildings_atlas` | 1024×1024 | 128×128 (Z2) | ~64 sprites | Building sprites |
| `citizens_atlas` | 512×512 | 64×64 (Z3) | ~64 sprites | Citizen portraits |

**Capacity Headroom:** Current baseline is 102 sprites. Atlas capacities are sized for 4–8× growth without requiring atlas splitting. When any atlas exceeds 75% fill, an automated alert is emitted in the CI log.

**Power-of-Two Enforcement:** The atlas packer MUST reject non-power-of-two atlas dimensions. This is enforced by:

```rust
fn assert_power_of_two(size: u32, name: &str) {
    assert!(
        size.is_power_of_two(),
        "{} must be power-of-two, got {}",
        name,
        size
    );
}
```

### 7.3 Metadata Format

Each atlas produces a companion JSON metadata file. The format is compatible with the Pixi.js `Assets.load()` spritesheet format (PixiJS Spritesheet v4.4+).

**Atlas JSON Schema (terrain_atlas.json example):**

```json
{
  "meta": {
    "app": "civlab-atlas-packer",
    "version": "1.0",
    "image": "terrain_atlas.png",
    "format": "RGBA8888",
    "size": { "w": 2048, "h": 2048 },
    "scale": "1",
    "smartupdate": "$TexturePacker:SmartUpdate:civlab:1.0$"
  },
  "frames": {
    "terrain_plains_z1": {
      "frame": { "x": 0, "y": 0, "w": 64, "h": 64 },
      "rotated": false,
      "trimmed": true,
      "spriteSourceSize": { "x": 0, "y": 0, "w": 64, "h": 64 },
      "sourceSize": { "w": 64, "h": 64 },
      "pivot": { "x": 0.5, "y": 0.5 },
      "palette_json": "terrain_plains_z1_palette.json"
    },
    "terrain_plains_z2": {
      "frame": { "x": 64, "y": 0, "w": 128, "h": 128 },
      "rotated": false,
      "trimmed": true,
      "spriteSourceSize": { "x": 0, "y": 0, "w": 128, "h": 128 },
      "sourceSize": { "w": 128, "h": 128 },
      "pivot": { "x": 0.5, "y": 0.5 },
      "palette_json": "terrain_plains_z2_palette.json"
    }
  },
  "animations": {}
}
```

**Extended Fields (CivLab additions beyond PixiJS standard):**

| Field | Description |
|-------|-------------|
| `pivot` | Sprite anchor point (always `{x: 0.5, y: 0.5}` for CivLab sprites) |
| `palette_json` | Path to the palette JSON for this sprite (relative to atlases dir) |
| `trim_rect` | Tight bounding box of non-transparent pixels within the frame |
| `asset_type` | `"terrain"` / `"building"` / `"citizen"` |
| `zoom_level` | 1, 2, or 3 |

### 7.4 Output File Contract

For each atlas, two files are written to `web/public/atlases/`:

```
web/public/atlases/
├── terrain_atlas.png        # 2048×2048 RGBA PNG
├── terrain_atlas.json       # PixiJS spritesheet metadata
├── buildings_atlas.png      # 1024×1024 RGBA PNG
├── buildings_atlas.json     # PixiJS spritesheet metadata
├── citizens_atlas.png       # 512×512 RGBA PNG
├── citizens_atlas.json      # PixiJS spritesheet metadata
└── asset_manifest.json      # Full pipeline manifest (Section 8)
```

Atlas PNGs are written as 32-bit RGBA (straight alpha). The quantization step produces indexed PNGs, but the atlas packer composites them back to RGBA for the final atlas (indexed PNG atlases are not compatible with WebGL texture upload in all browsers).

**File Size Expectations:**

| Atlas | Raw RGBA Size | Compressed (PNG) |
|-------|--------------|------------------|
| terrain_atlas (2048×2048) | 16 MB | ~2–4 MB |
| buildings_atlas (1024×1024) | 4 MB | ~0.8–1.5 MB |
| citizens_atlas (512×512) | 1 MB | ~200–400 KB |

Compression uses `oxipng` with level 4 (balanced speed/size) as a post-processing step in the Taskfile.

### 7.5 Rust Implementation

```rust
// src/bin/atlas_pack.rs

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use texture_packer::exporter::ImageExporter;
use texture_packer::{TexturePacker, TexturePackerConfig};

#[derive(Debug, Serialize, Deserialize)]
pub struct AtlasFrame {
    pub frame: Rect,
    pub rotated: bool,
    pub trimmed: bool,
    #[serde(rename = "spriteSourceSize")]
    pub sprite_source_size: Rect,
    #[serde(rename = "sourceSize")]
    pub source_size: Size,
    pub pivot: Pivot,
    pub palette_json: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Rect { pub x: u32, pub y: u32, pub w: u32, pub h: u32 }

#[derive(Debug, Serialize, Deserialize)]
pub struct Size { pub w: u32, pub h: u32 }

#[derive(Debug, Serialize, Deserialize)]
pub struct Pivot { pub x: f32, pub y: f32 }

pub struct AtlasPackerConfig {
    pub atlas_name: String,
    pub output_size: (u32, u32),
    pub padding: u32,
    pub allow_rotation: bool,
}

impl AtlasPackerConfig {
    pub fn terrain() -> Self {
        Self {
            atlas_name: "terrain_atlas".into(),
            output_size: (2048, 2048),
            padding: 2,
            allow_rotation: false,
        }
    }

    pub fn buildings() -> Self {
        Self {
            atlas_name: "buildings_atlas".into(),
            output_size: (1024, 1024),
            padding: 2,
            allow_rotation: false,
        }
    }

    pub fn citizens() -> Self {
        Self {
            atlas_name: "citizens_atlas".into(),
            output_size: (512, 512),
            padding: 2,
            allow_rotation: false,
        }
    }
}

pub fn pack_atlas(
    input_dir: &Path,
    output_dir: &Path,
    config: &AtlasPackerConfig,
) -> anyhow::Result<()> {
    assert_power_of_two(config.output_size.0, "atlas width");
    assert_power_of_two(config.output_size.1, "atlas height");

    let packer_config = TexturePackerConfig {
        max_width: config.output_size.0,
        max_height: config.output_size.1,
        allow_rotation: config.allow_rotation,
        texture_padding: config.padding,
        ..Default::default()
    };

    let mut packer = TexturePacker::new_skyline(packer_config);
    let mut frames: HashMap<String, AtlasFrame> = HashMap::new();

    let png_files: Vec<PathBuf> = std::fs::read_dir(input_dir)?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().map_or(false, |e| e == "png"))
        .collect();

    if png_files.is_empty() {
        anyhow::bail!("No PNG files found in {}", input_dir.display());
    }

    for png_path in &png_files {
        let asset_id = png_path
            .file_stem()
            .and_then(|s| s.to_str())
            .ok_or_else(|| anyhow::anyhow!("Invalid filename: {}", png_path.display()))?;

        let img = image::open(png_path)
            .with_context(|| format!("Failed to open {}", png_path.display()))?;

        packer.pack_own(asset_id.to_owned(), img)
            .map_err(|e| anyhow::anyhow!("Pack failed for {}: {:?}", asset_id, e))?;
    }

    // Export atlas PNG
    let output_png = output_dir.join(format!("{}.png", config.atlas_name));
    let atlas_img = ImageExporter::export(&packer)
        .context("Atlas export failed")?;
    atlas_img.save(&output_png)
        .with_context(|| format!("Failed to save atlas PNG: {}", output_png.display()))?;

    // Build frame metadata
    for (name, frame) in packer.get_frames() {
        let r = &frame.frame;
        let palette_json = format!("{}_palette.json", name);
        frames.insert(name.clone(), AtlasFrame {
            frame: Rect { x: r.x, y: r.y, w: r.w, h: r.h },
            rotated: frame.rotated,
            trimmed: true,
            sprite_source_size: Rect { x: 0, y: 0, w: r.w, h: r.h },
            source_size: Size { w: r.w, h: r.h },
            pivot: Pivot { x: 0.5, y: 0.5 },
            palette_json,
        });
    }

    // Write atlas JSON
    let atlas_json = serde_json::json!({
        "meta": {
            "app": "civlab-atlas-packer",
            "version": "1.0",
            "image": format!("{}.png", config.atlas_name),
            "format": "RGBA8888",
            "size": { "w": config.output_size.0, "h": config.output_size.1 },
            "scale": "1"
        },
        "frames": frames,
        "animations": {}
    });

    let output_json = output_dir.join(format!("{}.json", config.atlas_name));
    let json_str = serde_json::to_string_pretty(&atlas_json)?;
    std::fs::write(&output_json, json_str)
        .with_context(|| format!("Failed to write atlas JSON: {}", output_json.display()))?;

    // Fill check — warn if over 75%
    let total_area = config.output_size.0 * config.output_size.1;
    let used_area: u32 = packer.get_frames().values()
        .map(|f| f.frame.w * f.frame.h)
        .sum();
    let fill_pct = used_area as f32 / total_area as f32 * 100.0;
    if fill_pct > 75.0 {
        eprintln!(
            "WARNING: {} is {:.1}% full — consider splitting atlas if sprite count grows",
            config.atlas_name, fill_pct
        );
    }

    println!(
        "Packed {} — {} sprites, {:.1}% fill",
        config.atlas_name,
        frames.len(),
        fill_pct
    );

    Ok(())
}
```

### 7.6 Runtime Loading (Pixi.js)

The atlas JSON format is directly compatible with Pixi.js v8's `Assets` system and `Spritesheet` class.

```typescript
// web/src/assets/atlasLoader.ts

import { Assets, Spritesheet, Texture } from "pixi.js";

export interface AtlasSet {
  terrain: Spritesheet;
  buildings: Spritesheet;
  citizens: Spritesheet;
}

export async function loadAllAtlases(baseUrl: string): Promise<AtlasSet> {
  const [terrain, buildings, citizens] = await Promise.all([
    Assets.load<Spritesheet>(`${baseUrl}/terrain_atlas.json`),
    Assets.load<Spritesheet>(`${baseUrl}/buildings_atlas.json`),
    Assets.load<Spritesheet>(`${baseUrl}/citizens_atlas.json`),
  ]);

  if (!terrain || !buildings || !citizens) {
    throw new Error(
      "Atlas load failed: one or more atlases returned null. " +
      "Ensure web/public/atlases/ contains all three atlas files."
    );
  }

  return { terrain, buildings, citizens };
}

export function getSpriteTexture(
  atlases: AtlasSet,
  assetId: string
): Texture {
  const atlas = resolveAtlas(atlases, assetId);
  const texture = atlas.textures[assetId];

  if (!texture) {
    throw new Error(
      `Texture not found in atlas: "${assetId}". ` +
      `Check asset_manifest.json and atlas JSON for this asset_id.`
    );
  }

  return texture;
}

function resolveAtlas(atlases: AtlasSet, assetId: string): Spritesheet {
  if (assetId.startsWith("terrain_")) return atlases.terrain;
  if (
    assetId.startsWith("building_") || assetId.startsWith("city_") ||
    assetId.startsWith("farm_") || assetId.startsWith("mine_") ||
    assetId.startsWith("barracks_") || assetId.startsWith("market_") ||
    assetId.startsWith("library_") || assetId.startsWith("temple_") ||
    assetId.startsWith("harbor_") || assetId.startsWith("workshop_") ||
    assetId.startsWith("granary_") || assetId.startsWith("palace_") ||
    assetId.startsWith("wall_")
  ) return atlases.buildings;
  if (assetId.startsWith("citizen_")) return atlases.citizens;

  throw new Error(
    `Cannot resolve atlas for asset_id "${assetId}". ` +
    `asset_id prefix must be terrain_, building_, or citizen_ (or a known building type prefix).`
  );
}
```

**Load Performance Target:** All three atlases MUST load and parse within 500 ms on a 100 Mbps connection (see FR-CIV-ASSET-015). Atlas files are served with `Cache-Control: immutable` headers since they are content-addressed by pipeline hash.

---

## 8. Asset Manifest Schema

### 8.1 Schema Definition

The `asset_manifest.json` is the authoritative record of every asset produced by a pipeline run. It is generated by `scripts/gen_manifest.py` as the final step of the build pipeline.

**Full Schema (JSON Schema Draft-07):**

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "$id": "https://civlab.io/schemas/asset_manifest_v1.json",
  "title": "CivLab Asset Manifest",
  "type": "object",
  "required": ["manifest_version", "build_timestamp", "pipeline_hash", "assets"],
  "properties": {
    "manifest_version": {
      "type": "string",
      "const": "1.0"
    },
    "build_timestamp": {
      "type": "string",
      "format": "date-time"
    },
    "pipeline_hash": {
      "type": "string",
      "pattern": "^blake3:[a-f0-9]{64}$"
    },
    "civlab_version": { "type": "string" },
    "quality_tier": {
      "type": "string",
      "enum": ["standard", "high"]
    },
    "assets": {
      "type": "array",
      "items": { "$ref": "#/definitions/AssetEntry" },
      "minItems": 1
    }
  },
  "definitions": {
    "AssetEntry": {
      "type": "object",
      "required": [
        "asset_id", "asset_type", "zoom_level", "svg_template",
        "parameters", "output_atlas", "uv_rect", "content_hash"
      ],
      "properties": {
        "asset_id": { "type": "string", "pattern": "^[a-z][a-z0-9_]*$" },
        "asset_type": { "type": "string", "enum": ["terrain", "building", "citizen"] },
        "zoom_level": { "type": "integer", "enum": [1, 2, 3] },
        "svg_template": { "type": "string" },
        "parameters": { "type": "object", "additionalProperties": true },
        "sdxl_seed": { "type": ["integer", "null"] },
        "sdxl_controlnet_strength": { "type": ["number", "null"] },
        "output_atlas": {
          "type": "string",
          "enum": ["terrain_atlas", "buildings_atlas", "citizens_atlas"]
        },
        "uv_rect": {
          "type": "array",
          "items": { "type": "integer" },
          "minItems": 4,
          "maxItems": 4
        },
        "pivot": {
          "type": "array",
          "items": { "type": "number" },
          "minItems": 2,
          "maxItems": 2
        },
        "content_hash": { "type": "string", "pattern": "^blake3:[a-f0-9]{64}$" },
        "palette_size": { "type": "integer" },
        "nation_primary_palette_index": { "type": "integer" },
        "nation_secondary_palette_index": { "type": "integer" }
      }
    }
  }
}
```

### 8.2 Field Specifications

**asset_id Naming Convention:**

```
{asset_type}_{subtype}_{variant}_z{zoom_level}

Examples:
  terrain_plains_z1          — Plains terrain, Zoom 1
  terrain_plains_z2          — Plains terrain, Zoom 2
  building_city_center_l1_z2 — City Center level 1, Zoom 2
  building_farm_l3_z2        — Farm level 3, Zoom 2
  citizen_farmer_ideo2_z3    — Farmer, ideology 2, Zoom 3
```

**pipeline_hash Computation:**

```python
import blake3
from pathlib import Path

def compute_pipeline_hash(templates_dir: Path, scripts_dir: Path) -> str:
    hasher = blake3.blake3()
    for template in sorted(templates_dir.rglob("*.svg.j2")):
        hasher.update(template.read_bytes())
    for script in sorted(scripts_dir.glob("*.py")):
        hasher.update(script.read_bytes())
    return f"blake3:{hasher.hexdigest()}"
```

**content_hash Computation:**

```python
def compute_content_hash(png_path: Path) -> str:
    return f"blake3:{blake3.blake3(png_path.read_bytes()).hexdigest()}"
```

### 8.3 Versioning Policy

The manifest schema version is incremented when the schema adds required fields or changes existing field types. The game client MUST reject manifests with an unknown `manifest_version` — no silent fallback handling.

```typescript
const SUPPORTED_MANIFEST_VERSIONS = ["1.0"] as const;

export function validateManifestVersion(version: string): void {
  if (!SUPPORTED_MANIFEST_VERSIONS.includes(version as any)) {
    throw new Error(
      `Unsupported asset manifest version: "${version}". ` +
      `Supported versions: ${SUPPORTED_MANIFEST_VERSIONS.join(", ")}. ` +
      `Rebuild assets with the current pipeline.`
    );
  }
}
```

### 8.4 Canonical Asset Manifest Example

```json
{
  "manifest_version": "1.0",
  "build_timestamp": "2026-02-21T00:00:00Z",
  "pipeline_hash": "blake3:a3f7c9e2d1b5084f6a2c3e4d5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4",
  "civlab_version": "0.4.0-rc1",
  "quality_tier": "standard",
  "assets": [
    {
      "asset_id": "terrain_plains_z2",
      "asset_type": "terrain",
      "zoom_level": 2,
      "svg_template": "terrain/plains.svg.j2",
      "parameters": {
        "terrain_type": "plains",
        "terrain_color": "#7ec850",
        "terrain_color_dark": "#5aa030",
        "terrain_color_light": "#a0e070",
        "nation_color_hex": "#c8303c",
        "nation_color_secondary_hex": "#f0c040",
        "population_tier": 1,
        "ideology_index": 0,
        "show_icon": false,
        "opacity_overlay": 0.4
      },
      "sdxl_seed": null,
      "sdxl_controlnet_strength": null,
      "output_atlas": "terrain_atlas",
      "uv_rect": [0, 0, 128, 128],
      "pivot": [0.5, 0.5],
      "content_hash": "blake3:b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0a1b2c3d4e5f6a7b8c9d0e1f2a3b4",
      "palette_size": 16,
      "nation_primary_palette_index": 0,
      "nation_secondary_palette_index": 1
    }
  ]
}
```

---

## 9. Build Pipeline (Taskfile)

### 9.1 Taskfile Definition

The entire asset pipeline is orchestrated via [Taskfile](https://taskfile.dev) (`Taskfile.yml` in the project root). Each stage is a discrete task with explicit `deps` and `sources`/`generates` for incremental build support.

```yaml
# Taskfile.yml (asset pipeline section)

version: "3"

vars:
  BUILD_DIR: build
  ASSETS_DIR: assets
  WEB_ATLASES_DIR: web/public/atlases
  CIVLAB_QUALITY: '{{.CIVLAB_QUALITY | default "standard"}}'
  CIVLAB_VERSION:
    sh: git describe --tags --always --dirty 2>/dev/null || echo "dev"

tasks:
  assets:generate:
    desc: "Run the full 2D asset pipeline (all stages)"
    deps: [assets:pack:atlas]
    cmds:
      - task: assets:manifest

  assets:inject:
    desc: "Stage 1 — Inject parameters into SVG templates"
    sources:
      - assets/templates/**/*.svg.j2
      - assets/asset_parameters.yaml
      - scripts/svg_inject.py
    generates:
      - "{{.BUILD_DIR}}/svg/*.svg"
      - "{{.BUILD_DIR}}/asset_params.json"
    cmds:
      - mkdir -p {{.BUILD_DIR}}/svg
      - >
        python3 scripts/svg_inject.py
        --templates {{.ASSETS_DIR}}/templates/
        --params {{.ASSETS_DIR}}/asset_parameters.yaml
        --out {{.BUILD_DIR}}/svg/
        --params-out {{.BUILD_DIR}}/asset_params.json

  assets:rasterize:
    desc: "Stage 2 — Rasterize SVGs to PNG via resvg (Rust, parallel)"
    deps: [assets:inject]
    sources:
      - "{{.BUILD_DIR}}/svg/*.svg"
    generates:
      - "{{.BUILD_DIR}}/png/*.png"
    cmds:
      - mkdir -p {{.BUILD_DIR}}/png
      - cargo run --release --bin resvg_batch -- {{.BUILD_DIR}}/svg/ {{.BUILD_DIR}}/png/

  assets:sdxl:
    desc: "Stage 2a (optional) — SDXL agentic enhancement pass"
    deps: [assets:rasterize]
    preconditions:
      - sh: '[ "{{.CIVLAB_QUALITY}}" = "high" ] || [ -n "$CIVLAB_SDXL_ENABLE" ]'
        msg: "SDXL pass skipped. Set CIVLAB_QUALITY=high to enable."
    sources:
      - "{{.BUILD_DIR}}/png/*.png"
      - scripts/sdxl_enhance.py
    generates:
      - "{{.BUILD_DIR}}/sdxl/*.png"
    cmds:
      - mkdir -p {{.BUILD_DIR}}/sdxl
      - >
        python3 scripts/sdxl_enhance.py
        --input {{.BUILD_DIR}}/png/
        --output {{.BUILD_DIR}}/sdxl/
        --quality {{.CIVLAB_QUALITY}}

  assets:rembg:
    desc: "Stage 3 — Remove backgrounds with rembg (Python, async)"
    deps: [assets:rasterize]
    sources:
      - "{{.BUILD_DIR}}/png/*.png"
      - scripts/rembg_batch.py
    generates:
      - "{{.BUILD_DIR}}/clean/*.png"
    cmds:
      - mkdir -p {{.BUILD_DIR}}/clean
      - >
        python3 scripts/rembg_batch.py
        {{.BUILD_DIR}}/png/
        {{.BUILD_DIR}}/clean/
        --max-concurrent 4

  assets:quantize:
    desc: "Stage 4 — Quantize palettes with imagequant (Rust)"
    deps: [assets:rembg]
    sources:
      - "{{.BUILD_DIR}}/clean/*.png"
      - "{{.BUILD_DIR}}/asset_params.json"
    generates:
      - "{{.BUILD_DIR}}/quantized/*.png"
      - "{{.BUILD_DIR}}/quantized/*_palette.json"
    cmds:
      - mkdir -p {{.BUILD_DIR}}/quantized
      - >
        cargo run --release --bin quantize --
        {{.BUILD_DIR}}/clean/
        {{.BUILD_DIR}}/quantized/
        --params {{.BUILD_DIR}}/asset_params.json

  assets:pack:atlas:
    desc: "Stage 5 — Pack sprites into atlases (Rust, MaxRects)"
    deps: [assets:quantize]
    sources:
      - "{{.BUILD_DIR}}/quantized/*.png"
    generates:
      - "{{.WEB_ATLASES_DIR}}/terrain_atlas.png"
      - "{{.WEB_ATLASES_DIR}}/terrain_atlas.json"
      - "{{.WEB_ATLASES_DIR}}/buildings_atlas.png"
      - "{{.WEB_ATLASES_DIR}}/buildings_atlas.json"
      - "{{.WEB_ATLASES_DIR}}/citizens_atlas.png"
      - "{{.WEB_ATLASES_DIR}}/citizens_atlas.json"
    cmds:
      - mkdir -p {{.WEB_ATLASES_DIR}}
      - >
        cargo run --release --bin atlas_pack --
        {{.BUILD_DIR}}/quantized/
        {{.WEB_ATLASES_DIR}}/
      - >
        oxipng --opt 4 --strip safe
        {{.WEB_ATLASES_DIR}}/terrain_atlas.png
        {{.WEB_ATLASES_DIR}}/buildings_atlas.png
        {{.WEB_ATLASES_DIR}}/citizens_atlas.png

  assets:manifest:
    desc: "Stage 6 — Generate asset_manifest.json"
    deps: [assets:pack:atlas]
    sources:
      - "{{.WEB_ATLASES_DIR}}/*_atlas.json"
      - "{{.BUILD_DIR}}/asset_params.json"
      - "{{.BUILD_DIR}}/quantized/*.png"
      - scripts/gen_manifest.py
    generates:
      - "{{.WEB_ATLASES_DIR}}/asset_manifest.json"
    cmds:
      - >
        python3 scripts/gen_manifest.py
        {{.BUILD_DIR}}/
        {{.WEB_ATLASES_DIR}}/asset_manifest.json
        {{.CIVLAB_QUALITY}}
        {{.CIVLAB_VERSION}}

  assets:validate:
    desc: "Validate asset_manifest.json against JSON schema"
    deps: [assets:manifest]
    cmds:
      - >
        python3 scripts/validate_manifest.py
        {{.WEB_ATLASES_DIR}}/asset_manifest.json
        assets/schemas/asset_manifest_v1.json

  assets:clean:build:
    desc: "Clean intermediate build artifacts"
    cmds:
      - rm -rf {{.BUILD_DIR}}/svg {{.BUILD_DIR}}/png {{.BUILD_DIR}}/sdxl
      - rm -rf {{.BUILD_DIR}}/clean {{.BUILD_DIR}}/quantized

  assets:clean:all:
    desc: "Clean all build artifacts and shipped atlases"
    deps: [assets:clean:build]
    cmds:
      - rm -rf {{.WEB_ATLASES_DIR}}/*.png {{.WEB_ATLASES_DIR}}/*.json
```

### 9.2 Stage Dependencies DAG

```
assets:inject
     │
     ▼
assets:rasterize
     │
     ├─────────────────────── (optional, quality=high)
     │                       │
     ▼                       ▼
assets:rembg          assets:sdxl
     │                       │
     └──────────┬────────────┘
                ▼
        assets:quantize
                │
                ▼
       assets:pack:atlas
                │
                ▼
       assets:manifest
                │
                ▼
       assets:validate
```

### 9.3 Incremental Build Strategy

Taskfile's `sources`/`generates` enable incremental builds. If no source files have changed since the last run, a task is skipped.

**Incremental Invalidation Rules:**
1. Changing a single `.svg.j2` template invalidates Stage 1 and all downstream stages.
2. Changing `asset_parameters.yaml` invalidates Stage 1 for all assets.
3. Changing a pipeline script invalidates its stage and all downstream stages.
4. Changing any font file in `assets/fonts/` invalidates Stage 2 (rasterization) and all downstream stages.
5. Force-rebuild: `task --force assets:generate`

### 9.4 Environment Requirements

**Required Tools:**

| Tool | Version | Purpose |
|------|---------|---------|
| `cargo` | stable 1.80+ | Rust binary builds |
| `python3` | 3.12+ | Python scripts |
| `uv` | 0.4+ | Python dependency management |
| `task` | 3.x | Taskfile runner |
| `oxipng` | 9.x | PNG compression |
| `git` | 2.x | Version tag for manifest |

**Python Dependencies** (`scripts/requirements.txt`):

```
jinja2==3.1.4
rembg==2.0.59
aiofiles==24.1.0
pillow==10.4.0
blake3==0.4.1
pydantic==2.9.2
jsonschema==4.23.0
```

### 9.5 CI Integration

The asset pipeline runs in CI on every PR touching `assets/**`, `scripts/**`, `src/bin/**`, `Cargo.toml`, or `Cargo.lock`.

**GitHub Actions Workflow excerpt:**

```yaml
name: Asset Pipeline CI
on:
  pull_request:
    paths:
      - 'assets/**'
      - 'scripts/**'
      - 'src/bin/**'
      - 'Cargo.toml'
      - 'Cargo.lock'

jobs:
  asset-pipeline:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - name: Install uv
        run: curl -LsSf https://astral.sh/uv/install.sh | sh
      - name: Install Python deps
        run: uv pip install -r scripts/requirements.txt --system
      - name: Cache rembg models
        uses: actions/cache@v4
        with:
          path: ~/.u2net
          key: rembg-models-u2netp-v1
      - uses: arduino/setup-task@v2
      - name: Install oxipng
        run: cargo install oxipng --version "9.*"
      - name: Run asset pipeline
        run: task assets:generate assets:validate
        env:
          CIVLAB_QUALITY: standard
      - name: Performance gate
        run: python3 tests/perf/test_render_time.py --max-seconds 30
      - name: Upload atlas artifacts
        uses: actions/upload-artifact@v4
        with:
          name: atlases-${{ github.sha }}
          path: web/public/atlases/
          retention-days: 7
```

---

## 10. Runtime Sprite System (Pixi.js)

### 10.1 SpriteManager Architecture

The `SpriteManager` class is the single point of access for all game sprites at runtime. It owns:
- The three loaded `Spritesheet` objects (terrain, buildings, citizens)
- The asset manifest (for metadata lookups)
- The sprite object pool (reusable `Sprite` instances)
- The nation recoloring shader filters (one per active nation)

**Architecture Invariants:**
1. `SpriteManager` is a singleton — one instance per game client.
2. All atlas loading is done in `SpriteManager.init()` before the first game frame renders.
3. No sprite texture is loaded after initialization; all textures come from pre-loaded atlases.
4. The sprite pool is pre-warmed with 256 `Sprite` instances at startup.

### 10.2 Zoom-Level Atlas Switching

When the player zooms between levels, `SpriteManager.setZoomLevel()` swaps the active texture for all visible sprites. This is a pointer swap (no GPU upload), since all three atlas sets are already in VRAM.

**Zoom Transition Sequence:**

```
Player triggers zoom (scroll wheel / keyboard shortcut)
    │
    ▼
Camera system begins zoom tween (CIV-0300 §7.1 — 200ms duration)
    │
    ▼
At 50% of tween (100ms in):
    SpriteManager.setZoomLevel(newZoom)
    → For each active sprite handle:
        sprite.texture = getTextureForZoom(handle.assetId, newZoom)
    │
    ▼
Camera tween completes
    │
    ▼
Render frame — all sprites using new zoom level textures
```

The swap at 50% of the tween coincides with maximum camera distortion, masking texture-pop.

### 10.3 Nation Recoloring via Shader

Nation colors are not baked into atlas sprites. A Pixi.js fragment shader replaces palette indices 0 (nation primary) and 1 (nation secondary) at render time with the actual nation's current colors.

**Shader Design:**

```glsl
// web/src/shaders/nation_recolor.frag
precision mediump float;

varying vec2 vTextureCoord;
uniform sampler2D uSampler;

uniform vec4 uNationPrimary;    // Active nation primary RGBA [0–1]
uniform vec4 uNationSecondary;  // Active nation secondary RGBA [0–1]
uniform vec4 uBakedPrimary;     // Template baked primary (#c8303c default)
uniform vec4 uBakedSecondary;   // Template baked secondary (#f0c040 default)

const float TOLERANCE = 0.08;   // Handles dithering artifacts

bool colorMatches(vec4 a, vec4 b) {
    return distance(a.rgb, b.rgb) < TOLERANCE;
}

void main(void) {
    vec4 color = texture2D(uSampler, vTextureCoord);
    if (colorMatches(color, uBakedPrimary)) {
        gl_FragColor = vec4(uNationPrimary.rgb, color.a);
    } else if (colorMatches(color, uBakedSecondary)) {
        gl_FragColor = vec4(uNationSecondary.rgb, color.a);
    } else {
        gl_FragColor = color;
    }
}
```

The filter is applied at the `Container` level per nation (not per sprite). Each nation's tile container receives one `NationRecolorFilter` instance. This means one GPU uniform update per nation per frame when nation colors change — highly efficient.

### 10.4 LOD Policy

| Camera Zoom | Active Atlas Frames | Sprite Size | VRAM (active) |
|-------------|--------------------|-----------|--------------------|
| Zoom 1 (strategic) | terrain_atlas Z1 frames | 64×64 | ~2–4 MB |
| Zoom 2 (tactical) | terrain_atlas Z2 + buildings_atlas | 128×128 | ~5–8 MB |
| Zoom 3 (citizen) | citizens_atlas | 64×64 | ~1–2 MB |

All three atlases are always loaded in VRAM (~8–14 MB total). LOD switching is a texture reference swap, not a GPU upload.

**LOD Hysteresis:** A 200 ms hysteresis timer prevents rapid LOD switching at zoom boundaries.

### 10.5 Sprite Pooling

```typescript
// web/src/assets/SpritePool.ts

import { Sprite, Texture } from "pixi.js";

export class SpritePool {
  private readonly pool: Sprite[] = [];

  constructor(prewarmSize = 256) {
    for (let i = 0; i < prewarmSize; i++) {
      const s = new Sprite();
      s.visible = false;
      this.pool.push(s);
    }
  }

  public acquire(texture: Texture): Sprite {
    const sprite = this.pool.pop();
    if (!sprite) {
      // Pool exhausted — allocate new (warn; not fatal)
      console.warn("SpritePool exhausted — allocating new Sprite. Increase prewarm size.");
      return new Sprite(texture);
    }
    sprite.texture = texture;
    sprite.visible = true;
    return sprite;
  }

  public release(sprite: Sprite): void {
    sprite.visible = false;
    sprite.texture = Texture.EMPTY;
    sprite.filters = null;
    this.pool.push(sprite);
  }
}
```

Pool exhaustion emits a console warning but does not crash — a new `Sprite` is allocated. Repeated pool exhaustion in production indicates the prewarm size should be increased (current target: sufficient for 20×20 tile viewport at Zoom 2).

### 10.6 TypeScript Implementation

**SpriteManager (core class):**

```typescript
// web/src/assets/SpriteManager.ts

import { Sprite, Spritesheet, Texture } from "pixi.js";
import { SpritePool } from "./SpritePool";
import { NationRecolorFilter, NationColors } from "./NationRecolorFilter";
import { loadAllAtlases, getSpriteTexture, AtlasSet } from "./atlasLoader";
import type { AssetManifest } from "./manifest";

export enum ZoomLevel { Strategic = 1, Tactical = 2, Citizen = 3 }

export class SpriteManager {
  private static instance: SpriteManager | null = null;

  private atlases!: AtlasSet;
  private manifest!: AssetManifest;
  private pool!: SpritePool;
  private currentZoom: ZoomLevel = ZoomLevel.Tactical;
  private activeHandles: Map<string, { sprite: Sprite; assetId: string }> = new Map();
  private nationFilters: Map<string, NationRecolorFilter> = new Map();

  public static getInstance(): SpriteManager {
    if (!SpriteManager.instance) {
      SpriteManager.instance = new SpriteManager();
    }
    return SpriteManager.instance;
  }

  public async init(atlasBaseUrl: string): Promise<void> {
    const resp = await fetch(`${atlasBaseUrl}/asset_manifest.json`);
    if (!resp.ok) {
      throw new Error(
        `Failed to load asset_manifest.json: HTTP ${resp.status}. ` +
        `Ensure the asset pipeline has run successfully.`
      );
    }
    this.manifest = await resp.json();

    if (this.manifest.manifest_version !== "1.0") {
      throw new Error(
        `Unsupported manifest version: "${this.manifest.manifest_version}". ` +
        `Expected "1.0". Rebuild assets.`
      );
    }

    this.atlases = await loadAllAtlases(atlasBaseUrl);
    this.pool = new SpritePool(256);
  }

  public acquireSprite(assetId: string, nationId?: string): Sprite {
    const texture = this.resolveTexture(assetId);
    const sprite = this.pool.acquire(texture);
    sprite.anchor.set(0.5, 0.5);

    if (nationId) {
      sprite.filters = [this.getOrCreateNationFilter(nationId)];
    }

    this.activeHandles.set(`${assetId}:${nationId ?? ""}`, { sprite, assetId });
    return sprite;
  }

  public releaseSprite(sprite: Sprite, assetId: string, nationId?: string): void {
    this.activeHandles.delete(`${assetId}:${nationId ?? ""}`);
    this.pool.release(sprite);
  }

  public setZoomLevel(zoom: ZoomLevel): void {
    if (zoom === this.currentZoom) return;
    this.currentZoom = zoom;
    for (const handle of this.activeHandles.values()) {
      handle.sprite.texture = this.resolveTexture(handle.assetId);
    }
  }

  public updateNationColors(nationId: string, colors: NationColors): void {
    this.nationFilters.get(nationId)?.updateNationColors(colors);
  }

  private resolveTexture(assetId: string): Texture {
    const zoomedId = assetId.endsWith(`_z${this.currentZoom}`)
      ? assetId
      : `${assetId}_z${this.currentZoom}`;
    return getSpriteTexture(this.atlases, zoomedId);
  }

  private getOrCreateNationFilter(nationId: string): NationRecolorFilter {
    if (!this.nationFilters.has(nationId)) {
      this.nationFilters.set(
        nationId,
        new NationRecolorFilter({
          primary: [0.784, 0.188, 0.235, 1.0],
          secondary: [0.941, 0.753, 0.251, 1.0],
        })
      );
    }
    return this.nationFilters.get(nationId)!;
  }
}
```

---

## 11. Functional Requirements

### 11.1 FR-CIV-ASSET-001 through FR-CIV-ASSET-010

**FR-CIV-ASSET-001 — SVG Template Rendering**

> The pipeline SHALL render every `.svg.j2` template in `assets/templates/` for every parameter combination defined in `asset_parameters.yaml`, producing one unique SVG file per combination.

- **Acceptance Criteria:** Given a template catalog of 25 templates and 102 parameter combinations, `svg_inject.py` produces exactly 102 `.svg` files in `build/svg/`. Any template variable referenced in a template but absent from the parameter record causes an immediate build failure.
- **Test Reference:** `tests/unit/test_svg_inject.py::test_all_templates_rendered`
- **Priority:** P0 (blocking)

---

**FR-CIV-ASSET-002 — resvg Rasterization Correctness**

> The resvg renderer SHALL produce pixel-identical output for identical SVG inputs across all supported CI platforms (Linux x86_64, macOS arm64).

- **Acceptance Criteria:** For a reference set of 10 canonical SVGs, the BLAKE3 hash of the resvg output PNG matches the reference hash on all CI platforms. Hash comparison is performed in the CI workflow as an explicit step.
- **Test Reference:** `tests/cross_platform/test_resvg_determinism.rs`
- **Priority:** P0 (blocking)

---

**FR-CIV-ASSET-003 — Supersampling Quality**

> The resvg renderer SHALL apply 4× supersampling (render at 4× target resolution, downscale with Lanczos3) to all sprites. Direct rendering to target size without supersampling is NOT permitted.

- **Acceptance Criteria:** The `RasterizationConfig.supersample_factor` field MUST be `4` at all code paths. A lint check (`scripts/check_supersample.py`) confirms this in CI.
- **Test Reference:** `tests/unit/test_resvg_config.rs::test_supersample_factor`
- **Priority:** P0 (blocking)

---

**FR-CIV-ASSET-004 — Background Removal Quality Gate**

> The rembg background removal step SHALL reject any output sprite with alpha channel coverage below 60% (fewer than 60% of pixels are opaque).

- **Acceptance Criteria:** `rembg_batch.py` raises a fatal error and exits with code 1 for any sprite where `opaque_pixels / total_pixels < 0.60`. The build pipeline stops. No fallback to the un-removed sprite is permitted.
- **Test Reference:** `tests/unit/test_rembg_batch.py::test_alpha_coverage_gate`
- **Priority:** P0 (blocking)

---

**FR-CIV-ASSET-005 — Nation Color Preservation in Quantization**

> The palette quantization step SHALL preserve the nation primary color (palette index 0) and nation secondary color (palette index 1) as exact forced palette entries. These colors MUST NOT be approximated by nearest-palette-entry substitution.

- **Acceptance Criteria:** After quantization, extracting palette entries 0 and 1 from the indexed PNG MUST return exactly the input `nation_primary_rgba` and `nation_secondary_rgba` values (within 1 LSB for each channel to account for integer rounding).
- **Test Reference:** `tests/unit/test_quantize.rs::test_nation_colors_preserved`
- **Priority:** P0 (blocking)

---

**FR-CIV-ASSET-006 — Atlas Power-of-Two Dimensions**

> All output atlas PNGs SHALL have dimensions that are powers of two on both width and height. Non-power-of-two atlas dimensions are forbidden.

- **Acceptance Criteria:** After `atlas_pack` runs, each output PNG passes: `width.is_power_of_two() && height.is_power_of_two()`. The `assert_power_of_two` check in the packer binary enforces this at runtime with a panic.
- **Test Reference:** `tests/unit/test_atlas_pack.rs::test_atlas_dimensions_pow2`
- **Priority:** P0 (blocking)

---

**FR-CIV-ASSET-007 — Atlas UV Coordinate Validity**

> Every sprite entry in an atlas JSON SHALL have a `frame` rect that is fully contained within the atlas dimensions. No sprite frame SHALL overflow the atlas boundary or overlap another sprite frame.

- **Acceptance Criteria:** For each atlas, a post-pack validation script checks: `frame.x + frame.w <= atlas_width` and `frame.y + frame.h <= atlas_height` for all frames. Overlap detection uses a 2D interval sweep. Failures cause the pipeline to abort.
- **Test Reference:** `tests/unit/test_atlas_pack.rs::test_uv_validity`, `tests/unit/test_atlas_pack.rs::test_no_overlap`
- **Priority:** P0 (blocking)

---

**FR-CIV-ASSET-008 — Manifest Completeness**

> The `asset_manifest.json` SHALL contain an entry for every sprite packed into every atlas. No sprite that exists in an atlas JSON SHALL be absent from the manifest.

- **Acceptance Criteria:** After `gen_manifest.py` runs, the count of manifest `assets` entries equals the sum of frame counts across all three atlas JSONs. Any discrepancy causes the manifest script to exit with error code 1.
- **Test Reference:** `tests/unit/test_gen_manifest.py::test_manifest_completeness`
- **Priority:** P0 (blocking)

---

**FR-CIV-ASSET-009 — Manifest JSON Schema Conformance**

> The `asset_manifest.json` SHALL conform to the JSON Schema defined in `assets/schemas/asset_manifest_v1.json`. Validation SHALL run as the final step of every pipeline run.

- **Acceptance Criteria:** `python3 scripts/validate_manifest.py` exits with code 0. Any schema validation error (missing required field, wrong type, pattern mismatch) causes exit code 1 and blocks CI merge.
- **Test Reference:** `tests/unit/test_validate_manifest.py::test_schema_conformance`
- **Priority:** P0 (blocking)

---

**FR-CIV-ASSET-010 — Build-Time Reproducibility (Content Hash Stability)**

> Running the full pipeline twice with identical inputs (same templates, same parameters, same fonts, same pipeline scripts) SHALL produce identical `content_hash` values for every asset in the manifest.

- **Acceptance Criteria:** A CI job runs the pipeline twice in sequence and diffs the two generated `asset_manifest.json` files. The only field that may differ between runs is `build_timestamp`. All `content_hash` and `pipeline_hash` fields MUST be identical. Any difference causes CI failure.
- **Test Reference:** `tests/integration/test_pipeline_reproducibility.sh`
- **Priority:** P0 (blocking)

---

### 11.2 FR-CIV-ASSET-011 through FR-CIV-ASSET-020

**FR-CIV-ASSET-011 — Build Performance: Render Time**

> The full baseline sprite render batch (all 102 sprites) SHALL complete in under 30 seconds on a 4-core x86_64 Linux CI runner (Ubuntu 22.04, 8 GB RAM).

- **Acceptance Criteria:** `tests/perf/test_render_time.py` measures wall-clock time of `cargo run --release --bin resvg_batch`. If elapsed time exceeds 30 seconds, the test fails and blocks CI.
- **Test Reference:** `tests/perf/test_render_time.py::test_batch_render_under_30s`
- **Priority:** P1 (performance gate)

---

**FR-CIV-ASSET-012 — Atlas Load Time**

> All three atlas files (terrain, buildings, citizens) SHALL load and be available for texture lookup within 500 ms of `SpriteManager.init()` being called, on a connection with 100 Mbps bandwidth and ≤10 ms latency.

- **Acceptance Criteria:** Playwright end-to-end test measures `performance.now()` from `SpriteManager.init()` call to the `init()` resolved promise. On a throttled network (100 Mbps, 10 ms RTT), this MUST be ≤500 ms.
- **Test Reference:** `tests/e2e/test_atlas_load_time.spec.ts`
- **Priority:** P1 (performance gate)

---

**FR-CIV-ASSET-013 — VRAM Budget**

> The total VRAM consumed by all three loaded atlas textures SHALL NOT exceed 20 MB. This is measured as the sum of `width × height × 4` bytes for each atlas PNG.

- **Acceptance Criteria:** The atlas pack script computes and logs total VRAM usage after packing. If computed usage exceeds 20 MB, the build emits a CI warning. If it exceeds 32 MB, the build fails.
- **Test Reference:** `tests/unit/test_atlas_pack.rs::test_vram_budget`
- **Priority:** P1

---

**FR-CIV-ASSET-014 — No Runtime SVG Parsing**

> The web bundle (Vite output in `web/dist/`) SHALL NOT contain `resvg`, `svg.js`, or any SVG parser/renderer library. Background removal, palette quantization, and atlas packing libraries SHALL also be absent from the web bundle.

- **Acceptance Criteria:** A CI step runs `grep -r "resvg\|rembg\|imagequant\|texture_packer\|svg\.js" web/dist/` (or equivalent bundle analysis). Any match causes a CI failure. Bundle analysis is also run with `vite-bundle-visualizer` to confirm.
- **Test Reference:** `tests/ci/check_bundle_no_svg_runtime.sh`
- **Priority:** P0 (blocking — architectural invariant)

---

**FR-CIV-ASSET-015 — Atlas Cache-Control Headers**

> Atlas files served from `web/public/atlases/` SHALL be served with `Cache-Control: public, max-age=31536000, immutable` HTTP headers. The `asset_manifest.json` SHALL be served with `Cache-Control: no-cache` (to allow version detection without hard-coding TTLs).

- **Acceptance Criteria:** The Vite/Nginx/Vercel serving configuration includes explicit header rules for `*.png` and `*_atlas.json` files under `/atlases/`. A CI test fetches a test atlas and validates the response headers.
- **Test Reference:** `tests/e2e/test_cache_headers.spec.ts`
- **Priority:** P1

---

**FR-CIV-ASSET-016 — Nation Recoloring Shader Correctness**

> The nation recoloring shader SHALL replace all pixels whose RGB distance from `uBakedPrimary` is less than `TOLERANCE` (0.08) with `uNationPrimary`, and all pixels within tolerance of `uBakedSecondary` with `uNationSecondary`. Non-matching pixels SHALL be rendered unchanged.

- **Acceptance Criteria:** A pixel-exact test renders a reference sprite through the shader with known input and output nation colors, then compares against a pre-computed reference image. SSIM deviation MUST be < 0.5% from reference.
- **Test Reference:** `tests/visual/test_nation_recolor_shader.spec.ts`
- **Priority:** P1

---

**FR-CIV-ASSET-017 — Sprite Pool Pre-Warm**

> The `SpritePool` SHALL pre-allocate exactly 256 `Sprite` instances during `SpriteManager.init()`. No Sprite objects SHALL be created after initialization in normal operation (non-pool-exhaustion paths).

- **Acceptance Criteria:** After `SpriteManager.init()`, `pool.poolSize === 256`. During a simulated 20×20 tile render (400 tiles), no new Sprite allocations occur (verified by monkey-patching the `Sprite` constructor and counting calls).
- **Test Reference:** `tests/unit/test_sprite_pool.spec.ts::test_no_allocation_during_render`
- **Priority:** P1

---

**FR-CIV-ASSET-018 — Zoom Transition Texture Swap**

> When `SpriteManager.setZoomLevel()` is called, all active sprite handles SHALL have their textures swapped to the new zoom level's variant within the same JavaScript event loop tick (synchronous swap, no deferred/async).

- **Acceptance Criteria:** A test acquires 20 sprite handles, calls `setZoomLevel(ZoomLevel.Citizen)`, then immediately (synchronously) checks that each sprite's `texture.key` ends with `_z3`. No `await` between the `setZoomLevel` call and the assertion.
- **Test Reference:** `tests/unit/test_sprite_manager.spec.ts::test_zoom_swap_synchronous`
- **Priority:** P1

---

**FR-CIV-ASSET-019 — SDXL Seed Determinism**

> When the SDXL enhancement pass is enabled, running the pipeline twice with the same `asset_parameters.yaml` and same `pipeline_hash` SHALL produce the same `sdxl_seed` for every asset. The generated image for a given seed MUST be byte-identical across runs (assuming the same model version and inference endpoint).

- **Acceptance Criteria:** Two pipeline runs with `CIVLAB_QUALITY=high` and identical inputs produce identical `sdxl_seed` values in `asset_manifest.json`. Image byte-identity is verified for a subset of 5 reference sprites.
- **Test Reference:** `tests/integration/test_sdxl_reproducibility.py`
- **Priority:** P2 (SDXL-only)

---

**FR-CIV-ASSET-020 — Template Validation Pre-Commit**

> All `.svg.j2` template files MUST pass SVG validity, layer structure, font reference, and variable coverage checks before being committed. The pre-commit hook SHALL block commits containing invalid templates.

- **Acceptance Criteria:** The pre-commit hook `hooks/pre-commit-svg-validate.sh` runs `python3 scripts/validate_templates.py assets/templates/` and blocks the commit if exit code is non-zero. The validation script checks: SVG 1.1 validity, presence of all 5 layer groups, only permitted font families, no embedded raster images, no `<script>` elements.
- **Test Reference:** `tests/unit/test_validate_templates.py::test_all_template_rules`
- **Priority:** P0 (blocking)

---

## 12. Test Harness

### 12.1 Unit Tests

Unit tests cover each pipeline stage independently with controlled inputs and exact output assertions.

**Test Organization:**

```
tests/
├── unit/
│   ├── test_svg_inject.py              # Stage 1: Jinja2 template injection
│   ├── test_validate_templates.py      # Template validation rules
│   ├── test_resvg_config.rs            # resvg configuration correctness
│   ├── test_rembg_batch.py             # Background removal + quality gate
│   ├── test_quantize.rs                # Palette quantization + forced colors
│   ├── test_atlas_pack.rs              # Atlas packing: dimensions, UVs, fill
│   ├── test_gen_manifest.py            # Manifest generation + completeness
│   ├── test_validate_manifest.py       # Manifest schema validation
│   ├── test_sprite_pool.spec.ts        # Sprite pool behavior
│   └── test_sprite_manager.spec.ts     # SpriteManager: zoom swap, init
├── visual/
│   ├── test_nation_recolor_shader.spec.ts   # Shader pixel comparison
│   └── test_sprite_visual_regression.py     # SSIM comparison vs references
├── perf/
│   ├── test_render_time.py             # Batch render under 30s gate
│   └── test_atlas_load_time.spec.ts    # Atlas load under 500ms gate
├── integration/
│   ├── test_pipeline_reproducibility.sh    # Two-run hash identity check
│   └── test_sdxl_reproducibility.py       # SDXL seed + image identity
├── cross_platform/
│   └── test_resvg_determinism.rs       # Cross-platform hash comparison
└── ci/
    └── check_bundle_no_svg_runtime.sh  # Bundle SVG-free check
```

**Key Unit Test Patterns:**

```python
# tests/unit/test_svg_inject.py

import pytest
from pathlib import Path
from scripts.svg_inject import inject_template, load_parameters

class TestSvgInject:

    def test_all_variables_injected(self, tmp_path):
        """Every {{var}} in a template must be replaced with its value."""
        template_content = """<svg><path fill="{{terrain_color}}" /></svg>"""
        template_path = tmp_path / "test.svg.j2"
        template_path.write_text(template_content)

        context = {"terrain_color": "#7ec850"}
        result = inject_template(template_path, context)

        assert "#7ec850" in result
        assert "{{terrain_color}}" not in result

    def test_undefined_variable_raises(self, tmp_path):
        """Referencing an undefined variable must raise, not silently blank."""
        template_content = """<svg><path fill="{{undefined_var}}" /></svg>"""
        template_path = tmp_path / "test.svg.j2"
        template_path.write_text(template_content)

        with pytest.raises(Exception, match="undefined_var"):
            inject_template(template_path, {})

    def test_output_count_matches_parameter_combinations(self):
        """Number of output SVGs must equal number of parameter records."""
        params = load_parameters(Path("assets/asset_parameters.yaml"))
        assert len(params) == 102  # Update when catalog changes

    def test_no_template_variables_remain_in_output(self, tmp_path):
        """No {{...}} or {%...%} syntax must remain in injected SVGs."""
        import re
        output_dir = tmp_path / "svg"
        # Run injection (mocked)
        for svg_file in output_dir.glob("*.svg"):
            content = svg_file.read_text()
            assert not re.search(r"\{\{.*?\}\}", content), \
                f"Uninjected variable found in {svg_file.name}"
```

```rust
// tests/unit/test_quantize.rs

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nation_colors_preserved() {
        let nation_primary: [u8; 4] = [200, 48, 60, 255];
        let nation_secondary: [u8; 4] = [240, 192, 64, 255];

        // Create a test RGBA image with nation colors present
        let test_img = create_test_image_with_nation_colors(nation_primary, nation_secondary);

        let config = QuantizationConfig::for_sprite();
        let result = quantize_image(&test_img, &config, nation_primary, nation_secondary)
            .expect("Quantization should succeed");

        // Extract palette entries 0 and 1
        let palette_entry_0 = result.palette[0];
        let palette_entry_1 = result.palette[1];

        // Nation colors must be exact (within 1 LSB)
        assert_approx_eq!(palette_entry_0.r, nation_primary[0], 1);
        assert_approx_eq!(palette_entry_0.g, nation_primary[1], 1);
        assert_approx_eq!(palette_entry_0.b, nation_primary[2], 1);

        assert_approx_eq!(palette_entry_1.r, nation_secondary[0], 1);
        assert_approx_eq!(palette_entry_1.g, nation_secondary[1], 1);
        assert_approx_eq!(palette_entry_1.b, nation_secondary[2], 1);
    }

    #[test]
    fn test_atlas_dimensions_pow2() {
        for config in [
            AtlasPackerConfig::terrain(),
            AtlasPackerConfig::buildings(),
            AtlasPackerConfig::citizens(),
        ] {
            assert!(
                config.output_size.0.is_power_of_two(),
                "Atlas width {} is not power-of-two for {}",
                config.output_size.0,
                config.atlas_name
            );
            assert!(
                config.output_size.1.is_power_of_two(),
                "Atlas height {} is not power-of-two for {}",
                config.output_size.1,
                config.atlas_name
            );
        }
    }

    #[test]
    fn test_no_uv_overlap() {
        // Pack a set of test sprites and verify no frame overlaps
        let test_sprites = generate_test_sprites(20, (64, 64));
        let config = AtlasPackerConfig::citizens();
        let frames = pack_and_get_frames(&test_sprites, &config)
            .expect("Packing should succeed");

        // Check all pairs for overlap
        let frame_list: Vec<_> = frames.values().collect();
        for i in 0..frame_list.len() {
            for j in (i + 1)..frame_list.len() {
                assert!(
                    !rects_overlap(&frame_list[i].frame, &frame_list[j].frame),
                    "Sprite frames overlap: {} and {}",
                    i, j
                );
            }
        }
    }
}
```

### 12.2 Visual Regression Tests

Visual regression tests detect unintended visual changes to sprites between pipeline versions. They use structural similarity (SSIM) comparison against reference sprites stored in `tests/visual/references/`.

**Reference Sprite Policy:**
- References are generated from the current known-good pipeline run.
- References are committed to git under `tests/visual/references/` (small set — 20 representative sprites).
- When an intentional visual change is made (e.g., new texture detail), references MUST be regenerated and committed as part of the same PR.

**SSIM Threshold:** Maximum allowed SSIM deviation is 1% (SSIM score ≥ 0.99). Deviations above 1% cause the test to fail and output a side-by-side diff image to `tests/visual/diffs/`.

```python
# tests/visual/test_sprite_visual_regression.py

import pytest
from pathlib import Path
from PIL import Image
from skimage.metrics import structural_similarity as ssim
import numpy as np

REFERENCES_DIR = Path("tests/visual/references")
PIPELINE_OUTPUT_DIR = Path("build/clean")
SSIM_THRESHOLD = 0.99

REFERENCE_SPRITES = [
    "terrain_plains_z1.png",
    "terrain_plains_z2.png",
    "terrain_desert_z2.png",
    "terrain_forest_z2.png",
    "terrain_mountain_z2.png",
    "building_city_center_l1_z2.png",
    "building_city_center_l4_z2.png",
    "building_farm_l2_z2.png",
    "building_barracks_l1_z2.png",
    "citizen_farmer_ideo0_z3.png",
    "citizen_soldier_ideo2_z3.png",
    # ... 20 total
]

@pytest.mark.parametrize("sprite_name", REFERENCE_SPRITES)
def test_sprite_ssim(sprite_name: str, tmp_path):
    reference_path = REFERENCES_DIR / sprite_name
    pipeline_path = PIPELINE_OUTPUT_DIR / sprite_name

    if not reference_path.exists():
        pytest.skip(f"Reference not found: {reference_path}. Generate with: task assets:update-references")

    if not pipeline_path.exists():
        pytest.fail(
            f"Pipeline output not found: {pipeline_path}. "
            f"Run 'task assets:generate' before running visual regression tests."
        )

    ref_img = np.array(Image.open(reference_path).convert("RGBA"), dtype=np.float32) / 255.0
    pipeline_img = np.array(Image.open(pipeline_path).convert("RGBA"), dtype=np.float32) / 255.0

    if ref_img.shape != pipeline_img.shape:
        pytest.fail(
            f"Size mismatch for {sprite_name}: "
            f"reference={ref_img.shape}, pipeline={pipeline_img.shape}"
        )

    score = ssim(ref_img, pipeline_img, data_range=1.0, channel_axis=2)

    if score < SSIM_THRESHOLD:
        # Save diff image for inspection
        diff = np.abs(ref_img - pipeline_img)
        diff_img = Image.fromarray((diff * 255).astype(np.uint8))
        diff_path = tmp_path / f"diff_{sprite_name}"
        diff_img.save(diff_path)
        pytest.fail(
            f"Visual regression for {sprite_name}: SSIM={score:.4f} < {SSIM_THRESHOLD}. "
            f"Diff image saved to {diff_path}. "
            f"If this change is intentional, update the reference with: task assets:update-references"
        )
```

### 12.3 Performance Tests

**Batch Render Time Gate (FR-CIV-ASSET-011):**

```python
# tests/perf/test_render_time.py

import subprocess
import time
import argparse
import sys

def test_batch_render_time(max_seconds: float = 30.0) -> None:
    """Measure wall-clock time for full baseline sprite render batch."""
    start = time.monotonic()

    result = subprocess.run(
        ["cargo", "run", "--release", "--bin", "resvg_batch",
         "--", "build/svg/", "build/png/"],
        capture_output=True,
        text=True,
    )

    elapsed = time.monotonic() - start

    if result.returncode != 0:
        print(f"FAIL: resvg_batch exited with code {result.returncode}", file=sys.stderr)
        print(result.stderr, file=sys.stderr)
        sys.exit(1)

    print(f"Render time: {elapsed:.1f}s (limit: {max_seconds}s)")

    if elapsed > max_seconds:
        print(
            f"FAIL: Render time {elapsed:.1f}s exceeds limit {max_seconds}s. "
            f"Performance regression detected. Check for parallelism regressions or template complexity increase.",
            file=sys.stderr
        )
        sys.exit(1)

    print(f"PASS: Render time within budget ({elapsed:.1f}s ≤ {max_seconds}s)")

if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--max-seconds", type=float, default=30.0)
    args = parser.parse_args()
    test_batch_render_time(args.max_seconds)
```

**Atlas Load Time Gate (FR-CIV-ASSET-012) — Playwright:**

```typescript
// tests/perf/test_atlas_load_time.spec.ts

import { test, expect } from "@playwright/test";

test("atlas load time under 500ms on 100Mbps", async ({ page, context }) => {
  // Throttle network to 100 Mbps, 10ms RTT
  const cdpSession = await context.newCDPSession(page);
  await cdpSession.send("Network.emulateNetworkConditions", {
    offline: false,
    downloadThroughput: (100 * 1024 * 1024) / 8,  // 100 Mbps in bytes/s
    uploadThroughput: (100 * 1024 * 1024) / 8,
    latency: 10,
  });

  await page.goto("http://localhost:5173");

  const loadTime = await page.evaluate(async () => {
    const { SpriteManager } = await import("/src/assets/SpriteManager.ts");
    const manager = SpriteManager.getInstance();
    const start = performance.now();
    await manager.init("/atlases");
    return performance.now() - start;
  });

  console.log(`Atlas load time: ${loadTime.toFixed(0)}ms`);
  expect(loadTime).toBeLessThanOrEqual(500);
});
```

### 12.4 Integration Tests

**Pipeline Reproducibility Test (FR-CIV-ASSET-010):**

```bash
#!/usr/bin/env bash
# tests/integration/test_pipeline_reproducibility.sh

set -euo pipefail

echo "=== Pipeline Reproducibility Test ==="

# Run pipeline once
echo "--- Run 1 ---"
task assets:clean:all
task assets:generate
cp web/public/atlases/asset_manifest.json /tmp/manifest_run1.json

# Extract content hashes (exclude build_timestamp)
python3 -c "
import json
m = json.load(open('/tmp/manifest_run1.json'))
hashes = {a['asset_id']: a['content_hash'] for a in m['assets']}
json.dump({'pipeline_hash': m['pipeline_hash'], 'asset_hashes': hashes}, open('/tmp/hashes_run1.json', 'w'), indent=2)
"

# Run pipeline a second time
echo "--- Run 2 ---"
task assets:clean:all
task assets:generate
cp web/public/atlases/asset_manifest.json /tmp/manifest_run2.json

python3 -c "
import json
m = json.load(open('/tmp/manifest_run2.json'))
hashes = {a['asset_id']: a['content_hash'] for a in m['assets']}
json.dump({'pipeline_hash': m['pipeline_hash'], 'asset_hashes': hashes}, open('/tmp/hashes_run2.json', 'w'), indent=2)
"

# Compare
echo "--- Comparing hashes ---"
python3 - <<'PYEOF'
import json, sys

run1 = json.load(open('/tmp/hashes_run1.json'))
run2 = json.load(open('/tmp/hashes_run2.json'))

errors = []

if run1['pipeline_hash'] != run2['pipeline_hash']:
    errors.append(f"pipeline_hash mismatch: {run1['pipeline_hash']} != {run2['pipeline_hash']}")

for asset_id, hash1 in run1['asset_hashes'].items():
    hash2 = run2['asset_hashes'].get(asset_id)
    if hash2 is None:
        errors.append(f"Asset {asset_id} missing from run 2")
    elif hash1 != hash2:
        errors.append(f"content_hash mismatch for {asset_id}: {hash1} != {hash2}")

if errors:
    print(f"FAIL: {len(errors)} reproducibility errors:")
    for e in errors:
        print(f"  - {e}")
    sys.exit(1)

print(f"PASS: All {len(run1['asset_hashes'])} asset hashes are identical across both runs")
PYEOF
```

### 12.5 CI Gate Policy

**Merge Blocking Rules (all must pass to merge):**

| Test Category | Gate | Block on Failure |
|---------------|------|-----------------|
| Template validation | Pre-commit hook | YES — blocks local commit |
| Unit tests (all) | CI job | YES — blocks PR merge |
| Visual regression (SSIM) | CI job | YES — blocks PR merge |
| Pipeline reproducibility | CI job (nightly or on main) | YES — blocks release |
| Render time (≤30s) | CI job | YES — blocks PR merge |
| Atlas load time (≤500ms) | CI job | YES — blocks PR merge |
| Bundle SVG-free check | CI job | YES — blocks PR merge |
| Manifest schema validation | Part of pipeline run | YES — blocks pipeline |
| UV coordinate validity | Part of pipeline run | YES — blocks pipeline |

**Test Runner Commands:**

```bash
# Run all unit tests (Rust + Python)
cargo test && python3 -m pytest tests/unit/ -v

# Run visual regression tests
python3 -m pytest tests/visual/ -v

# Run performance tests
python3 tests/perf/test_render_time.py --max-seconds 30
npx playwright test tests/perf/test_atlas_load_time.spec.ts

# Run integration tests
bash tests/integration/test_pipeline_reproducibility.sh

# Run full CI suite locally
task test:all
```

---

## 13. Determinism Rules

### 13.1 Determinism Contract

**CRITICAL INVARIANT:** Given identical inputs, the asset pipeline MUST produce byte-identical output.

"Identical inputs" means:
1. All `.svg.j2` template files are byte-identical.
2. `asset_parameters.yaml` is byte-identical.
3. All font files in `assets/fonts/` are byte-identical.
4. All pipeline scripts (`*.py`, Rust binaries compiled from identical source) are functionally identical.
5. The rembg model version (`u2netp` or `u2net`) is pinned.
6. The imagequant version is pinned.
7. For SDXL-enhanced assets: the model version, inference endpoint, and seed are identical.

"Byte-identical output" means:
1. Each atlas PNG's pixel data is identical.
2. Each atlas JSON's frame metadata is identical (UV coordinates, sizes).
3. Each `content_hash` in `asset_manifest.json` is identical.
4. The `pipeline_hash` in `asset_manifest.json` is identical.
5. Palette JSON files are identical.

**What is NOT required to be identical:**
- `build_timestamp` in the manifest (wall-clock time is inherently non-deterministic).
- File modification timestamps.
- Intermediate file ordering (if parallel, ordering may vary, but final atlas packing is deterministic because sprites are sorted by `asset_id` before packing).

### 13.2 Non-Determinism Sources and Mitigations

| Non-Determinism Source | Risk Level | Mitigation |
|----------------------|-----------|-----------|
| **System fonts** — OS-provided fonts vary by platform | HIGH | Bundled fonts only; `fontdb` populated exclusively from `assets/fonts/`. System font lookup disabled. |
| **Float ordering in rayon** — parallel sum ordering varies | MEDIUM | No floating-point accumulation in atlas packing. PNG pixel data is integer. Palette quantization uses deterministic integer paths. |
| **File iteration order** — `read_dir` order varies by OS | MEDIUM | All file lists are sorted by filename before processing. Enforced in `resvg_batch`, `quantize`, and `atlas_pack`. |
| **SDXL inference** — same prompt + seed may produce slight variation across runs if model loaded with different precision | MEDIUM | Seed captured in manifest. Byte-identity of SDXL output is verified only within the same inference run; cross-run SDXL identity requires pinned model checkpoint hash. |
| **rembg U2Net** — model inference is deterministic given same input + same ONNX Runtime version | LOW | rembg version pinned. ONNX Runtime version pinned. Determinism verified in integration tests. |
| **PNG encoder byte-order** — `png` crate encoding is deterministic across versions | LOW | `png` crate version pinned in `Cargo.lock`. |
| **imagequant internal state** — quantization algorithm is deterministic given same inputs | LOW | `imagequant` version pinned. Forced palette entries are inserted in consistent order (primary first, secondary second). |
| **Time-dependent SVG features** — SVG `<animate>` or `<set>` elements | LOW | Templates are validated against a feature allowlist. Animated SVG features are rejected by `validate_templates.py`. |
| **`oxipng` compression** — PNG lossless optimization is deterministic for same input | LOW | `oxipng` version pinned. |

### 13.3 BLAKE3 Verification Protocol

Every pipeline run computes and verifies BLAKE3 hashes at two levels:

**Level 1 — Per-Asset Content Hash:**
After the quantization step, each quantized PNG's pixel data is hashed. The hash is stored in `asset_manifest.json` under `content_hash`.

```python
def compute_and_verify_content_hash(
    png_path: Path,
    expected_hash: str | None = None
) -> str:
    data = png_path.read_bytes()
    actual_hash = f"blake3:{blake3.blake3(data).hexdigest()}"

    if expected_hash is not None and actual_hash != expected_hash:
        raise ValueError(
            f"Content hash mismatch for {png_path.name}: "
            f"expected={expected_hash}, actual={actual_hash}. "
            f"Pipeline output is not reproducible — investigate non-determinism sources."
        )

    return actual_hash
```

**Level 2 — Pipeline Hash (All Inputs):**
The `pipeline_hash` in the manifest covers all template files and pipeline scripts. If templates or scripts change, the pipeline hash changes, causing the CI reproducibility check to detect input changes and invalidate the cached reference hashes.

**Level 3 — Atlas Integrity Verification:**
A separate script (`scripts/verify_manifest.py`) re-reads all quantized PNGs and re-computes their BLAKE3 hashes, comparing against `asset_manifest.json`. This runs in CI after every pipeline execution:

```python
# scripts/verify_manifest.py

import json
import sys
import blake3
from pathlib import Path

def verify_manifest(manifest_path: Path, quantized_dir: Path) -> None:
    manifest = json.loads(manifest_path.read_text())
    errors = []

    for asset in manifest["assets"]:
        asset_id = asset["asset_id"]
        expected_hash = asset["content_hash"]
        png_path = quantized_dir / f"{asset_id}.png"

        if not png_path.exists():
            errors.append(f"{asset_id}: PNG not found at {png_path}")
            continue

        actual_hash = f"blake3:{blake3.blake3(png_path.read_bytes()).hexdigest()}"
        if actual_hash != expected_hash:
            errors.append(
                f"{asset_id}: content_hash mismatch "
                f"(expected={expected_hash}, actual={actual_hash})"
            )

    if errors:
        print(f"FAIL: {len(errors)} hash verification errors:", file=sys.stderr)
        for err in errors:
            print(f"  {err}", file=sys.stderr)
        sys.exit(1)

    print(f"PASS: All {len(manifest['assets'])} asset content hashes verified")

if __name__ == "__main__":
    verify_manifest(
        manifest_path=Path(sys.argv[1]),
        quantized_dir=Path(sys.argv[2]) if len(sys.argv) > 2 else Path("build/quantized"),
    )
```

### 13.4 Seed Management

For SDXL-enhanced assets, seed management is the primary mechanism for ensuring reproducibility across runs.

**Seed Derivation (deterministic):**

```python
import hashlib

def derive_sdxl_seed(asset_id: str, pipeline_hash: str) -> int:
    """
    Derive a deterministic SDXL seed from the asset ID and pipeline hash.
    Changing either the asset_id or pipeline_hash produces a different seed.
    This intentionally invalidates cached SDXL outputs when pipeline inputs change.
    """
    combined = f"{asset_id}:{pipeline_hash}".encode("utf-8")
    digest = hashlib.blake2b(combined, digest_size=4).digest()
    return int.from_bytes(digest, "big")
```

**Seed Override:** Artists can pin a specific seed for an asset by adding `sdxl_seed: <integer>` to the asset's entry in `asset_parameters.yaml`. When a seed override is present, `derive_sdxl_seed()` is NOT called for that asset; the override seed is used directly.

**Seed Storage:** Seeds are stored in `asset_manifest.json` alongside the asset entry. The seed for non-SDXL assets is `null`. Seeds are used by the CI SDXL reproducibility test to re-run SDXL inference and verify image identity.

**Seed Exhaustion Policy:** The derived seed space is 32-bit (0 to 2^32 - 1). With 102 baseline sprites, the probability of seed collision is negligible (~(102²) / 2^32 ≈ 0.00024%). No deduplication of seeds is required.

---

## 14. FR Traceability

This section maps every Functional Requirement in this spec to its parent requirements in the project-level FR registry, its test coverage, and the pipeline stage it governs.

| FR ID | Title | Parent FR | Pipeline Stage | Test Coverage | Priority |
|-------|-------|-----------|---------------|---------------|----------|
| FR-CIV-ASSET-001 | SVG Template Rendering | FR-CIV-RTS-RENDER-001 | Stage 1 | `test_svg_inject.py` | P0 |
| FR-CIV-ASSET-002 | resvg Cross-Platform Determinism | FR-CIV-CORE-DET-001 | Stage 2 | `test_resvg_determinism.rs` | P0 |
| FR-CIV-ASSET-003 | 4× Supersampling Required | FR-CIV-RTS-RENDER-002 | Stage 2 | `test_resvg_config.rs` | P0 |
| FR-CIV-ASSET-004 | Background Removal Quality Gate | FR-CIV-RTS-RENDER-003 | Stage 3 | `test_rembg_batch.py` | P0 |
| FR-CIV-ASSET-005 | Nation Color Preservation | FR-CIV-RTS-NATION-001 | Stage 4 | `test_quantize.rs` | P0 |
| FR-CIV-ASSET-006 | Power-of-Two Atlas Dimensions | FR-CIV-RTS-RENDER-004 | Stage 5 | `test_atlas_pack.rs` | P0 |
| FR-CIV-ASSET-007 | UV Coordinate Validity | FR-CIV-RTS-RENDER-005 | Stage 5 | `test_atlas_pack.rs` | P0 |
| FR-CIV-ASSET-008 | Manifest Completeness | FR-CIV-ASSET-MANI-001 | Stage 6 | `test_gen_manifest.py` | P0 |
| FR-CIV-ASSET-009 | Manifest Schema Conformance | FR-CIV-ASSET-MANI-002 | Stage 6 | `test_validate_manifest.py` | P0 |
| FR-CIV-ASSET-010 | Build-Time Reproducibility | FR-CIV-CORE-DET-002 | All stages | `test_pipeline_reproducibility.sh` | P0 |
| FR-CIV-ASSET-011 | Render Batch Performance (≤30s) | FR-CIV-PERF-BUILD-001 | Stage 2 | `test_render_time.py` | P1 |
| FR-CIV-ASSET-012 | Atlas Load Time (≤500ms) | FR-CIV-PERF-RT-001 | Runtime | `test_atlas_load_time.spec.ts` | P1 |
| FR-CIV-ASSET-013 | VRAM Budget (≤20MB) | FR-CIV-PERF-RT-002 | Runtime | `test_atlas_pack.rs` | P1 |
| FR-CIV-ASSET-014 | No Runtime SVG Parsing | FR-CIV-ARCH-NOSVG-001 | Runtime / CI | `check_bundle_no_svg_runtime.sh` | P0 |
| FR-CIV-ASSET-015 | Atlas Cache-Control Headers | FR-CIV-PERF-WEB-001 | Deployment | `test_cache_headers.spec.ts` | P1 |
| FR-CIV-ASSET-016 | Nation Recoloring Shader Correctness | FR-CIV-RTS-NATION-002 | Runtime | `test_nation_recolor_shader.spec.ts` | P1 |
| FR-CIV-ASSET-017 | Sprite Pool Pre-Warm | FR-CIV-PERF-RT-003 | Runtime | `test_sprite_pool.spec.ts` | P1 |
| FR-CIV-ASSET-018 | Zoom Transition Synchronous Swap | FR-CIV-RTS-ZOOM-001 | Runtime | `test_sprite_manager.spec.ts` | P1 |
| FR-CIV-ASSET-019 | SDXL Seed Determinism | FR-CIV-CORE-DET-003 | Stage 2a | `test_sdxl_reproducibility.py` | P2 |
| FR-CIV-ASSET-020 | Template Validation Pre-Commit | FR-CIV-ASSET-QUAL-001 | Pre-Stage 1 | `test_validate_templates.py` | P0 |

**FR Prefix Key:**
- `FR-CIV-RTS-*` — RTS rendering and UI requirements (CIV-0300)
- `FR-CIV-CORE-DET-*` — Core determinism requirements (CIV-0001)
- `FR-CIV-PERF-*` — Performance requirements (CIV-0500)
- `FR-CIV-ARCH-*` — Architecture invariant requirements
- `FR-CIV-ASSET-MANI-*` — Asset manifest integrity requirements
- `FR-CIV-ASSET-QUAL-*` — Asset quality control requirements

**Coverage Summary:**

| Priority | Count | Test Coverage | Blocking |
|----------|-------|--------------|---------|
| P0 | 10 | 100% | YES — blocks merge |
| P1 | 9 | 100% | YES — blocks merge |
| P2 | 1 | 100% (SDXL-only path) | NO — advisory |

---

*End of CIV-0600: 2D Asset Pipeline Specification*

---

**Document Control**

| Field | Value |
|-------|-------|
| Spec ID | CIV-0600 |
| Version | 1.0 |
| Status | SPECIFICATION |
| Date | 2026-02-21 |
| Authors | CIV Asset & Rendering Team |
| Reviewers | CIV Architecture Team, CIV Client Team |
| Next Review | 2026-05-21 |
| Supersedes | N/A (new spec) |
| Superseded By | N/A |

**Change Log**

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0 | 2026-02-21 | Asset & Rendering Team | Initial specification. Covers all 6 pipeline stages, SDXL optional pass, runtime SpriteManager, 20 FRs, full test harness, and determinism rules. |
