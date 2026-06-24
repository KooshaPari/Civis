# CIV-0300: RTS UI/UX Specification

**Version:** 1.0
**Status:** SPECIFICATION
**Date:** 2026-02-21
**Authors:** CIV Client & UI Team
**References:** CIVLAB_GAME_DESIGN.md, CIV-0200-client-protocol.md, FUNCTIONAL_REQUIREMENTS.md, PRD.md

---

## Table of Contents

1. [UI Philosophy & Design Principles](#1-ui-philosophy--design-principles)
2. [Screen Layout — Full ASCII Wireframes](#2-screen-layout--full-ascii-wireframes)
   - 2.1 [Zoom 1: Strategic / Nation Level](#21-zoom-1-strategic--nation-level)
   - 2.2 [Zoom 2: Tactical / City Level](#22-zoom-2-tactical--city-level)
   - 2.3 [Zoom 3: Citizen Level / Research Mode](#23-zoom-3-citizen-level--research-mode)
3. [Map Rendering System](#3-map-rendering-system)
   - 3.1 [Hex Grid & Coordinate System](#31-hex-grid--coordinate-system)
   - 3.2 [Tile Layer Stack](#32-tile-layer-stack)
   - 3.3 [Camera System](#33-camera-system)
   - 3.4 [Pixi.js Render Pipeline](#34-pixijs-render-pipeline)
   - 3.5 [Fog of War](#35-fog-of-war)
4. [HUD Components](#4-hud-components)
   - 4.1 [Top Bar](#41-top-bar)
   - 4.2 [Resource Bar](#42-resource-bar)
   - 4.3 [Minimap](#43-minimap)
   - 4.4 [Nation Panel (Right)](#44-nation-panel-right)
   - 4.5 [Relations Panel](#45-relations-panel)
   - 4.6 [Action Panel](#46-action-panel)
   - 4.7 [Alert Feed](#47-alert-feed)
   - 4.8 [Research Tree Panel](#48-research-tree-panel)
   - 4.9 [Bottom Timeline](#49-bottom-timeline)
5. [Overlay System](#5-overlay-system)
   - 5.1 [Economy Overlay](#51-economy-overlay)
   - 5.2 [Military Overlay](#52-military-overlay)
   - 5.3 [Climate Overlay](#53-climate-overlay)
   - 5.4 [Social Overlay](#54-social-overlay)
   - 5.5 [Diplomacy Overlay](#55-diplomacy-overlay)
6. [Unit & Entity Rendering](#6-unit--entity-rendering)
   - 6.1 [Sprite Specifications](#61-sprite-specifications)
   - 6.2 [Selection System](#62-selection-system)
   - 6.3 [Formation Display](#63-formation-display)
   - 6.4 [Path Visualization](#64-path-visualization)
   - 6.5 [Combat Visualization](#65-combat-visualization)
7. [Zoom Transition System](#7-zoom-transition-system)
   - 7.1 [Zoom 1 to Zoom 2](#71-zoom-1-to-zoom-2)
   - 7.2 [Zoom 2 to Zoom 3](#72-zoom-2-to-zoom-3)
   - 7.3 [LOD Switching Rules](#73-lod-switching-rules)
   - 7.4 [Performance Targets](#74-performance-targets)
8. [Input & Hotkey System](#8-input--hotkey-system)
9. [Research Mode UI](#9-research-mode-ui)
   - 9.1 [Time-Series Chart Panel](#91-time-series-chart-panel)
   - 9.2 [Parameter Sweep Configuration](#92-parameter-sweep-configuration)
   - 9.3 [Divergence Viewer](#93-divergence-viewer)
   - 9.4 [Export Controls](#94-export-controls)
   - 9.5 [Citizen Micro-View](#95-citizen-micro-view)
10. [Web Client Stack](#10-web-client-stack)
    - 10.1 [Framework & Dependencies](#101-framework--dependencies)
    - 10.2 [State Management (Zustand)](#102-state-management-zustand)
    - 10.3 [WebSocket Integration](#103-websocket-integration)
    - 10.4 [Bundle Targets & Performance](#104-bundle-targets--performance)
11. [2D to 3D Transition Readiness](#11-2d-to-3d-transition-readiness)
    - 11.1 [IRenderer Abstraction](#111-irenderer-abstraction)
    - 11.2 [Asset Contract](#112-asset-contract)
    - 11.3 [Camera Abstraction](#113-camera-abstraction)
    - 11.4 [Spatial Math Policy](#114-spatial-math-policy)
12. [FR Traceability](#12-fr-traceability)
    - 12.1 [FR-CIV-RTS-* Mapping](#121-fr-civ-rts--mapping)
    - 12.2 [FR-CIV-GEO-* Mapping](#122-fr-civ-geo--mapping)

---

## 1. UI Philosophy & Design Principles

### 1.1 Core Philosophy

CivLab's UI must serve three distinct audiences simultaneously: the **strategist** who evaluates national-scale decisions, the **tactician** who manages city-level production chains and army formations, and the **researcher** who needs dense data access and reproducible export pipelines. No single UI paradigm satisfies all three; CivLab solves this through a three-zoom architecture where each zoom level delivers a specialized interface optimized for its purpose while sharing a coherent visual language.

The overriding design goal is **information density with legibility**. Every pixel of the HUD should encode meaningful simulation state. A veteran player should be able to read stability trends, supply line health, and ideological drift from a single glance at the HUD — without opening any panel. This is the Dwarf Fortress principle applied to a readable UI.

### 1.2 2D-First with 3D Transition Readiness

CivLab launches in 2D. The rendering layer uses Pixi.js (WebGL) for the web client and Bevy 2D for the native client. All spatial data, UI interaction geometry, and asset naming follow contracts that guarantee a clean swap to 3D in Phase 2.

**2D-first policy rules:**
- All UI chrome (panels, buttons, toolbars) is SVG-based — resolution-independent, theme-swappable, scalable to any DPI
- All game entities (units, buildings, terrain features) use PNG sprite sheets at defined atlas sizes
- Sprite atlas filenames follow the Phase 2 swap contract (see Section 11.2): `{type}_{direction}_{frame}.png` becomes `{type}_{direction}_{frame}.glb`
- No 2D-specific geometry is embedded in game logic; all spatial math routes through `Vec2` (upgradeable to `Vec3`)
- Camera operations (pan, zoom, focus) are abstracted behind `ICamera` (see Section 11.3)

### 1.3 Inspirations

| Inspiration | UI Lesson Applied |
|-------------|-------------------|
| **Victoria 3** | Clean choropleth map; political and economic overlays that communicate aggregate state without individual-unit clutter at strategic zoom |
| **Dwarf Fortress** | Dense information density; every column of text is meaningful; no wasted whitespace |
| **Factorio** | Production graph visibility directly on the HUD; resource flow rates shown as delta per tick next to every counter |
| **Anno 1800** | Beautiful city view at tactical zoom; buildings read as purposeful structures, not colored blobs |

### 1.4 Information Hierarchy

Information priority at each zoom level must match the decisions available at that zoom:

| Zoom | Primary Decision | Must-Be-Visible Without Drilling |
|------|------------------|----------------------------------|
| 1 (Strategic) | Declare war, set taxes, sign treaties, allocate research budget | Stability index, GDP delta, army strength ratio, diplomatic status of all neighbors |
| 2 (Tactical) | Build structures, recruit units, assign workers, manage supply | Production per district, construction queue, unit health bars, supply line status |
| 3 (Citizen) | Observe only (no direct control) | Individual happiness, ideology, job, family graph, information propagation |

### 1.5 Visual Design

**Theme:** Dark by default (space/night aesthetic). The simulation engine is a model of civilizational forces — vast, impersonal, long-horizon. The dark theme reinforces this. Map colors are saturated (biome colors pop on dark background). HUD chrome is near-black with subtle blue-grey highlights.

**Light theme (Research Mode):** High-contrast white background. Recharts and D3.js plots are readable on monitors at daylight. Activated via `?theme=light` URL param or keyboard shortcut `Shift+T`.

**Color palette (Okabe-Ito colorblind-safe):**

| Role | Hex | Usage |
|------|-----|-------|
| Primary action | `#E69F00` | Buttons, selected units, construction markers |
| Friendly | `#009E73` | Allied units, positive deltas, stable regions |
| Warning | `#F0E442` | Low supply, approaching threshold events |
| Hostile | `#D55E00` | Enemy units, war declarations, critical instability |
| Info | `#56B4E9` | Research mode, neutral highlights, tutorial callouts |
| Neutral | `#CC79A7` | Shadow network indicators, espionage events |
| Background | `#1A1A2E` | Primary dark background |
| Surface | `#16213E` | Panel backgrounds |
| Border | `#0F3460` | Panel borders, tile outlines at zoom 2 |

All faction colors in multi-player are drawn from a palette with sufficient contrast ratio (>= 4.5:1 against background), validated at scenario load.

### 1.6 Accessibility

- **WCAG 2.1 AA compliance** for all static UI chrome
- **Keyboard navigation**: every panel, button, and map interaction has a keyboard equivalent (no mouse-only actions)
- **Colorblind-safe**: all data visualizations use Okabe-Ito palette; no red/green information encoding without a shape or texture backup signal
- **Font sizing**: minimum 12px for body text; 14px for interactive labels; 18px for primary HUD counters. All sizes scale with browser font zoom.
- **Focus indicators**: 2px solid `#E69F00` outline on all focused interactive elements
- **Screen reader support**: all alert feed items have `aria-live="polite"` regions; all buttons have descriptive `aria-label` attributes
- **Reduced motion**: all zoom transitions and unit animations respect `prefers-reduced-motion`; falling back to instant cuts

### 1.7 Design Non-Negotiables

- No modal dialogs that block the simulation view; all panels use side-panels or bottom drawers
- No tooltip-only information — critical data is always in the main HUD; tooltips are supplementary
- Simulation speed controls are always reachable without hovering; Space bar always toggles pause
- Alert badges degrade gracefully: if > 99 alerts, display "99+" not a truncated number
- All HUD transitions are animated at <= 150ms (fast enough to feel responsive, not so fast as to be missed)

---

## 2. Screen Layout — Full ASCII Wireframes

### 2.1 Zoom 1: Strategic / Nation Level

The strategic view presents the entire map at nation scale. The player sees diplomatic relationships, army strengths, resource totals, and macro-economic indicators. Individual units and buildings are not rendered; regions are colored blocs.

```
┌─────────────────────────────────────────────────────────────────────────────┐
│ [⚑ Flag][Nation Name]    [Tick: 1247] [Year: 49 Q3]  [▶▶ 2×] [⏸]    [☰] │
├──────────────────┬──────────────────────────────────┬────────────────────────┤
│ STRATEGIC        │                                  │  NATION PANEL          │
│ OVERLAYS         │                                  │  ─────────────────     │
│ ────────────     │                                  │  GDP:  ██████████ 150M │
│ ○ Economy        │                                  │  Δ/tick: +120K  ↑      │
│ ○ Military       │      MAP VIEWPORT                │  Stability: 73/100     │
│ ○ Diplomacy      │      (Hex Grid, Panned)          │  ████████░░░░░         │
│ ○ Climate        │                                  │  Population:    2.43M  │
│ ○ Social         │      Nation Regions              │  Army Str:  ████░░ 68% │
│                  │      rendered as                 │  Legitmacy: ███░░░ 61% │
│ ACTIVE ALERTS    │      colored biome polygons      │  Treasury: 45,230g     │
│ ────────────     │      with nation color borders   │  Credit: ★★★★☆ AA     │
│ ⚠ Drought: N.    │                                  │  ─────────────────     │
│   Province       │                                  │  TECH PROGRESS         │
│ ⚠ Insurgency:    │                                  │  Steam Engine: ██░░    │
│   W. District    │                                  │  [Research Panel →]    │
│ ⚠ Low Supply:    │                                  │  ─────────────────     │
│   3rd Army       │                                  │  RELATIONS             │
│                  │                                  │  France:  ★★★★☆ +72   │
│ QUICK ACTIONS    │                                  │  China:   ★★☆☆☆ +34   │
│ ────────────     │                                  │  USA:     ★★★★★ +91   │
│ [Set Policy]     │                                  │  Russia:  ★☆☆☆☆ +18   │
│ [Declare War]    │                                  │  ─────────────────     │
│ [Sign Treaty]    │                                  │  ACTIONS               │
│ [Hire Official]  │                                  │  [Declare War]         │
│ [Adjust Tax]     │                                  │  [Sign Treaty]         │
│ [Set Research]   │                                  │  [Propose Alliance]    │
│                  │                                  │  [Set National Policy] │
│                  │                                  │  [Manage Officials]    │
│                  │                                  │  [Espionage Budget]    │
├──────────────────┴──────────────────────────────────┴────────────────────────┤
│ ⚡ Joules: 4.2PJ [↑+120TJ/t]  🌾 Food: 8.2M [↑+12K/t]  ⚙ Mat: 1.1M [-3K/t] │
│ [Events Log ▼]  [Timeline ─────●────]  [Research Tree]  [Replay: ▶ Export] │
└─────────────────────────────────────────────────────────────────────────────┘
```

**Component positions at Zoom 1:**

| Zone | Component | Width | Height |
|------|-----------|-------|--------|
| Top bar | Tick counter, speed, pause | 100% | 36px |
| Left sidebar | Overlay toggles, alerts, quick actions | 200px | calc(100% - 36px - 80px) |
| Center | Map viewport | calc(100% - 200px - 320px) | calc(100% - 36px - 80px) |
| Right sidebar | Nation panel, relations, action panel | 320px | calc(100% - 36px - 80px) |
| Bottom bar | Resource bar, timeline, controls | 100% | 80px |

### 2.2 Zoom 2: Tactical / City Level

The tactical view zooms into a specific region, showing individual hex tiles with buildings, units, and district boundaries. The player can directly select units, queue construction, and manage production chains.

```
┌─────────────────────────────────────────────────────────────────────────────┐
│ [⚑] [Nation] ← [Paris Region]    [Tick: 1247]  [▶▶ 1×] [⏸]     [☰] [?] │
├────────────────────────────┬───────────────────────────────┬────────────────┤
│ DISTRICT PANEL             │                               │ UNIT PANEL     │
│ ────────────────────       │                               │ ─────────────  │
│ [North Quarter  ▼]         │                               │ SELECTED: 3    │
│  Pop:    12,400            │                               │ 3rd Infantry   │
│  Food:   ████░░ 68%        │                               │ HP: ████░ 80%  │
│  Metal:  ██░░░░ 40%        │   MAP VIEWPORT (HEX GRID)     │ Morale: 72%    │
│  Wood:   █████░ 85%        │   64×64px tiles               │ Level: 3       │
│  Energy: ███░░░ 55%        │                               │ XP: ███░ 75/100│
│  Health: ██████ 94%        │   Individual buildings        │ Supply: ██░ 30%│
│  Crime:  ██░░░░ 22%        │   and units visible           │ ⚠ LOW SUPPLY   │
│                            │   at this zoom                │ ─────────────  │
│ CONSTRUCTION QUEUE         │                               │ COMMANDS       │
│ ────────────────────       │   Fog-of-war active           │ [▶ Move]  (Q)  │
│ 1. Barracks    [████░] 80% │                               │ [⚔ Attack] (W) │
│ 2. Farm        [░░░░░]  0% │   Selection box drag          │ ⛏ Fortify (E)  │
│    (queued)               │   active                      │ [↩ Rally]  (R) │
│ [+ Add Build]              │                               │ [📦 Resupply]  │
│                            │                               │ [⏺ Queue Cmds] │
│ PRODUCTION CHAINS          │                               │ ─────────────  │
│ ────────────────────       │                               │ FORMATION      │
│ Wheat Farm → Mill → Bread  │                               │ [Wedge] [Line] │
│   20/tick    15/tick  12/t │                               │ [Column][Circle│
│                            │                               │ ─────────────  │
│ Coal Mine → Factory        │                               │ MULTI-SELECT   │
│   8/tick  → Weapons 3/t    │                               │ Shift+Click    │
│                            │                               │ Drag to select │
│ [Trade Routes →]           │                               │ Dbl-click type │
│ [Local Policy]             │                               │                │
│ [City Budget]              │                               │ PATH PREVIEW   │
│                            │                               │ A* route shown │
├────────────────────────────┴───────────────────────────────┴────────────────┤
│ ⚡ 4.2PJ[+120/t] 🌾8.2M[+12/t] ⚙1.1M[-3/t] 💰45K[+800/t]  | Minimap [200px]│
│ [Events ▼] [Timeline ──●──] [City Budget] [Workers] [Diplomacy] [Research] │
└─────────────────────────────────────────────────────────────────────────────┘
```

**District detail side panel (expanded on tile click):**

```
┌────────────────────────────────────────────────┐
│ NORTH QUARTER DISTRICT                   [✕]   │
│ ─────────────────────────────────────────────  │
│ Population:  12,400  (density: 87/hex)         │
│ Happiness:   ██████░░░░  63/100                │
│ Stability:   ████████░░  78/100                │
│                                                │
│ EMPLOYMENT                                     │
│ Farmers:     ████████  4,200  (34%)            │
│ Craftsmen:   ██████    3,100  (25%)            │
│ Merchants:   ████      1,800  (15%)            │
│ Military:    ███       1,500  (12%)            │
│ Unemployed:  ██        1,100  (9%)  ⚠          │
│ Other:       █           700  (6%)             │
│                                                │
│ TOP INSTITUTIONS                               │
│  Merchant Guild (power: 72)                    │
│  City Guard     (power: 45)                    │
│  Temple         (power: 38)                    │
│                                                │
│ STRUCTURES (12 visible)                        │
│  3× Farm, 2× Mine, 1× Barracks, 1× Temple     │
│  1× Market, 1× Housing Block (×4)             │
│                                                │
│ [Manage Workers]  [Build]  [Policies]          │
└────────────────────────────────────────────────┘
```

**Unit command bar (appears at bottom when unit selected):**

```
┌─────────────────────────────────────────────────────────────────┐
│ 3rd Infantry Rgt ─ HP:████░80% ─ Morale:72% ─ Supply:██░30% ⚠  │
│ [▶ Move] [⚔ Attack] [⛏ Fortify] [↩ Rally] [📦 Resupply] [☠ Disband]│
│ Formation: [Wedge▼]   Stance: [Aggressive▼]  [Queue Commands +] │
└─────────────────────────────────────────────────────────────────┘
```

### 2.3 Zoom 3: Citizen Level / Research Mode

The citizen view is reached by double-clicking a citizen entity at Zoom 2 or from the Research Mode dashboard. It is an observer mode — no direct control actions are available. The entire interface is optimized for data exploration.

```
┌─────────────────────────────────────────────────────────────────────────────┐
│ [⚑] [Nation] ← [Paris] ← [North Quarter] ← Citizen #48291    [Tick: 1247] │
├──────────────────────────┬──────────────────────────┬───────────────────────┤
│ CITIZEN PROFILE          │                          │ INFORMATION NETWORK   │
│ ────────────────────     │                          │ ──────────────────    │
│ ID:       #48291         │                          │ Rumor Network         │
│ Name:     Jean Moreau    │  CITIZEN LOCATION MAP    │ (social graph)        │
│ Age:      34             │  (D3.js force graph)     │                       │
│ Class:    Craftsman      │                          │  ●──●──●              │
│ Job:      Blacksmith     │  Citizen dot in          │  │  │                 │
│ Location: North Quarter  │  district context        │  ●  ●──●             │
│                          │                          │  │     │              │
│ HAPPINESS: 58/100        │  Social graph shows      │  ●─────●             │
│ █████████░░░░ 58%        │  family + work           │                       │
│ Breakdown:               │  connections as edges    │ Rumors believed:      │
│  + Consumption:   +18    │                          │  "King is corrupt"    │
│  + Job Satisf:    +12    │  Highlighted beliefs     │   (believed: 2 mo)    │
│  - Crime victim:   -0    │  and propaganda          │  "Drought coming"     │
│  - Ideology mis:   -8    │  exposures               │   (believed: 5 mo)    │
│  - Health (sick): -12    │                          │                       │
│  + Ration level:  +11    │                          │ Info received tick:   │
│  + Social status: +10    │                          │  1240, 1231, 1219...  │
│                          │                          │                       │
│ IDEOLOGY: 43/100         │                          │ FAMILY TREE           │
│  [Autocracy]◄──────►[Democracy]                    │ ──────────────────    │
│             43           │                          │ Parent: #21044        │
│ vs Nation: 52 → -9 unhap │                          │ Spouse: #51002        │
│                          │                          │ Child:  #67891        │
│ HEALTH: Sickly (-12)     │                          │ Child:  #67902        │
│ SKILLS:                  │                          │                       │
│  Farming:   ████░   4    │                          │ WEALTH                │
│  Combat:    ██░░░   2    │                          │ Savings:  234g        │
│  Trade:     ███░░   3    │                          │ Property: 0           │
│  Govern:    █░░░░   1    │                          │ Debt:     0           │
│                          │                          │ Tax paid: 18g/yr      │
│ ALLEGIANCES              │                          │                       │
│  Nation:    ████░░ 65%   │                          │ TICK HISTORY          │
│  Merchant G:███░░░ 50%   │                          │ ────────────────      │
│  Church:    ██░░░░ 35%   │                          │ Happiness sparkline:  │
│                          │                          │ ▁▃▅▄▆▅▃▂▂▄▅▆▇▆▅▄▃  │
│ [Watch Neighbors]        │                          │ (last 50 ticks)       │
│ [Export Profile]         │                          │ [Full History →]      │
├──────────────────────────┴──────────────────────────┴───────────────────────┤
│ IDEOLOGY DISTRIBUTION (North Quarter):  [Scatter plot — 1000 citizens]      │
│  Autocracy ◄──[■■■■█████████████████████████████████░░░░░]──► Democracy 52 │
│ [Export CSV] [Run Parameter Sweep] [Compare Scenarios] [Replay This Tick]   │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## 3. Map Rendering System

### 3.1 Hex Grid & Coordinate System

CivLab uses a **flat-top hexagonal grid** with axial coordinates `(q, r)`. This system provides a clean, cache-friendly representation and maps naturally to screen space.

**Axial to pixel conversion (flat-top hex):**

```
hex_width  = 64   // pixels at zoom level 2
hex_height = hex_width * sqrt(3) / 2  &asymp; 55.4 pixels

pixel_x = hex_width * (q + r / 2)
pixel_y = hex_height * r * (3/4) * (2/sqrt(3))
```

Or using the standard flat-top matrix form:

```
[pixel_x]   [  hex_width   hex_width/2  ] [q]
[pixel_y] = [      0       hex_height   ] [r]
```

**Pixel to axial (inverse, for mouse hit-testing):**

```
q = (2/3 * pixel_x) / hex_width
r = (-1/3 * pixel_x + sqrt(3)/3 * pixel_y) / hex_width
// Round to nearest hex using cube-coordinate rounding
```

**Six neighbor directions (flat-top):**

```typescript
const HEX_DIRECTIONS: [number, number][] = [
  [+1,  0],  // East
  [+1, -1],  // North-East
  [ 0, -1],  // North-West
  [-1,  0],  // West
  [-1, +1],  // South-West
  [ 0, +1],  // South-East
];

function hex_neighbors(q: number, r: number): [number, number][] {
  return HEX_DIRECTIONS.map(([dq, dr]) => [q + dq, r + dr]);
}
```

**Hex distance:**

```typescript
function hex_distance(q1: number, r1: number, q2: number, r2: number): number {
  return Math.max(Math.abs(q1 - q2), Math.abs(r1 - r2), Math.abs((-q1 - r1) - (-q2 - r2)));
}
```

**Tile sizes by zoom level:**

| Zoom Level | Tile Size | Usage |
|------------|-----------|-------|
| 1 (Strategic) | 16×14px | Nation-scale overview; regions not tiles |
| 2 (Tactical) | 64×55px | Primary gameplay; building sprites visible |
| Minimap | 4×3px | Always-on 200×150px minimap |

### 3.2 Tile Layer Stack

Layers are rendered bottom-to-top in a single Pixi.js scene graph. Each layer is a separate `PIXI.Container` child of the main stage.

```
Layer 8 (top): UI overlays (selection box, path arrows, formation lines)
Layer 7:        Fog of war (WebGL shader mask)
Layer 6:        Data overlays (economy choropleth, climate gradient)
Layer 5:        Units (PIXI.ParticleContainer for 10k+ units)
Layer 4:        Infrastructure (roads, walls, aqueducts)
Layer 3:        Buildings / structures
Layer 2:        Resources (trees, iron deposits, visible surface features)
Layer 1 (base): Terrain (biome tiles, rivers, coastlines)
```

**Layer management rules:**
- Each layer has a `visible` flag toggled by overlay controls
- Fog of war (Layer 7) uses a custom GLSL fragment shader; never a canvas-based implementation
- Unit layer uses `PIXI.ParticleContainer` with `autoResize: true`; supports batched 10,000+ sprite renders at 60fps
- Overlay layers (Layer 6) use `PIXI.Graphics` with `alpha` tweened per overlay toggle animation

### 3.3 Camera System

The camera tracks a `viewport` state: `{ x: number, y: number, zoom: number }` where `(x, y)` is the world-space center of the viewport and `zoom` is the linear scale factor.

**Pan behavior:**
- **Mouse drag**: left button drag updates viewport center directly
- **Inertia damping**: release adds velocity vector; velocity decays exponentially at rate `0.92/frame` (approximately 500ms to stop)
- **Arrow keys**: discrete 32px/frame pan; acceleration after 10 frames of held key (max 128px/frame)
- **Edge scroll**: when mouse within 40px of window edge in fullscreen mode, pan in that direction at 2px/frame

**Zoom behavior:**
- **Scroll wheel**: logarithmic zoom; `zoom_factor = Math.exp(delta * 0.001)`
- **Pinch gesture**: direct `scale` from `TouchEvent.touches` distance ratio
- **Zoom-to-cursor**: calculate world position under cursor before and after zoom; adjust `(x, y)` so that world position stays under cursor
- **Keyboard `+`/`-`**: discrete zoom steps at `1.25×` per press
- **Zoom limits**: minimum `0.25` (entire world visible), maximum `8.0` (individual citizen dots visible)

**Camera animation (for zoom transitions):**

```typescript
interface CameraTransition {
  fromZoom: number;
  toZoom: number;
  fromCenter: [number, number];
  toCenter: [number, number];
  durationMs: number;
  easing: 'ease-in-out-cubic';
}

function animateCamera(transition: CameraTransition, onFrame: (state: CameraState) => void): void {
  // Cubic ease-in-out: f(t) = t < 0.5 ? 4t³ : 1 - (-2t+2)³/2
  // Interpolate zoom and center independently
  // Call onFrame on each animation frame until complete
}
```

### 3.4 Pixi.js Render Pipeline

The web client uses **Pixi.js v8** with a WebGL renderer. The render pipeline is:

```
1. WebSocket tick_broadcast received
2. Zustand store updated (game state slices)
3. React reconciler triggers re-render of HUD components (React DOM)
4. Pixi.js render loop (requestAnimationFrame) reads from Zustand store
5. Pixi.js updates sprite positions/textures from store diff
6. Pixi.js renders frame to WebGL canvas
7. HUD React components render over canvas via absolute positioning
```

**Texture atlas loading sequence:**

```typescript
// Load terrain atlas at startup (lazy via Pixi.js loader)
const TERRAIN_ATLAS = 'assets/terrain_atlas_64.json';    // ~256KB
const UNIT_ATLAS    = 'assets/units_32x32_atlas.json';   // ~192KB
const BUILDING_ATLAS = 'assets/buildings_64x64.json';   // ~384KB

// All atlases loaded before first render
await PIXI.Assets.load([TERRAIN_ATLAS, UNIT_ATLAS, BUILDING_ATLAS]);
```

**Terrain tiles (PIXI.TilingSprite):**
- One `PIXI.TilingSprite` per terrain type (plains, forest, hill, mountain, water, city)
- Tiles share texture atlas; each tile rendered at `(pixel_x, pixel_y)` from axial coordinate
- Dirty region tracking: only re-render tiles whose state changed this tick

**Unit sprites (PIXI.ParticleContainer):**
- All units of same type in one `PIXI.ParticleContainer`
- Container has `properties: { position: true, scale: true, rotation: true, tint: true }`
- Tint used for faction color overlay (applied to grayscale base sprite)
- Particle containers are pre-sized: `new PIXI.ParticleContainer(10000, properties)` at init

**Selection highlight:**
- `PIXI.Graphics` circle drawn over selected unit; animated pulsing ring using `alpha` tween
- Multi-selection: convex hull outline drawn with `PIXI.Graphics.moveTo/lineTo`

### 3.5 Fog of War

Fog of war is rendered as a WebGL fragment shader pass on top of the scene.

**Shader behavior:**
- `fogAlpha = 1.0` (fully fogged): black overlay
- `fogAlpha = 0.5` (scouted, not currently visible): desaturate + darken (historical fog)
- `fogAlpha = 0.0` (in vision range): no overlay

**Fog state storage:**
- `Uint8Array` sized `(map_width * map_height)` where values are `0` (clear), `128` (historical), `255` (fogged)
- Uploaded as a WebGL texture; fragment shader samples from this texture per-pixel
- Updated every tick (full redraw of fog texture from unit vision ranges)

**Fragment shader (simplified):**

```glsl
uniform sampler2D fogTexture;
uniform vec2 mapSize;
varying vec2 vUv;

void main() {
  float fog = texture2D(fogTexture, vUv).r;

  if (fog > 0.75) {
    // Fully fogged: black
    gl_FragColor = vec4(0.0, 0.0, 0.0, 0.85);
  } else if (fog > 0.25) {
    // Historical: desaturate
    vec4 sceneColor = texture2D(sceneTexture, vUv);
    float gray = dot(sceneColor.rgb, vec3(0.299, 0.587, 0.114));
    gl_FragColor = vec4(gray * 0.5, gray * 0.5, gray * 0.55, 0.7);
  } else {
    // Visible
    discard;
  }
}
```

**Vision range update algorithm:**

```typescript
function updateFogTexture(units: Unit[], fogArray: Uint8Array, mapWidth: number): void {
  // First pass: age all scouted (clear) cells to historical if no unit sees them
  // Second pass: for each unit, mark hexes within vision_range as clear
  // Hexes never seen remain at 255 (fully fogged)
  for (const unit of units) {
    const visibleHexes = hexesInRange(unit.q, unit.r, unit.visionRange);
    for (const [q, r] of visibleHexes) {
      fogArray[r * mapWidth + q] = 0;
    }
  }
}
```

---

## 4. HUD Components

### 4.1 Top Bar

**Purpose:** Persistent simulation state and control. Always visible regardless of zoom or active panel.

**Layout (left to right):**

```
┌──────────────────────────────────────────────────────────────────────────────┐
│ [⚑ flag 24px] [Nation Name 16px]  │  Tick: 1247  │  Year 49, Quarter 3      │
│                                   │  [0.5×] [1×] [▶2×] [5×] [⏭]  [⏸ PAUSE] │
│ [Alert badge ● 3]                 │              [Speed slider 120px]        │
└──────────────────────────────────────────────────────────────────────────────┘
Height: 36px. Background: var(--surface-0). Border-bottom: 1px var(--border).
```

**Components:**

| Component | Data Source | Update Frequency | Interaction |
|-----------|-------------|-----------------|-------------|
| Nation flag | `snapshot.nation.flag_url` | On load | None |
| Nation name | `snapshot.nation.name` | Static | Click → Nation panel focus |
| Tick counter | `snapshot.header.tick` | Every tick | None |
| Year/Quarter | `tick / 25` integer division | Every 25 ticks | Tooltip: exact tick |
| Speed buttons | `simState.speed` | On change | Click to set speed |
| Speed slider | `simState.speed` | On change | Drag to set speed (0.1× to max) |
| Pause/Resume | `simState.paused` | On change | Click, Space bar |
| Alert badge | `alerts.unacknowledged.length` | Every tick | Click → Alert Feed |

**Speed control states:**

| Label | Multiplier | Icon |
|-------|-----------|------|
| `0.5×` | 0.5 | `⏪` |
| `1×`   | 1.0 | `▶` |
| `2×`   | 2.0 | `▶▶` |
| `5×`   | 5.0 | `▶▶▶` |
| `MAX`  | unlimited | `⏭` |
| `PAUSE` | 0 | `⏸` |

**Accessibility:** All speed buttons have `role="radio"` in a `radiogroup`. Pause button has `aria-pressed` state. Speed slider has `aria-valuenow` and `aria-valuemin/max`.

### 4.2 Resource Bar

**Purpose:** Global resource snapshot with per-tick delta indicators. Drives at-a-glance economic health.

**Layout:**

```
┌──────────────────────────────────────────────────────────────────────────────┐
│ ⚡ Joules: 4.24 PJ  ↑+120 TJ/t  │ 🌾 Food: 8.2M  ↑+12K/t  │               │
│ ⚙ Materials: 1.1M  ↓-3K/t   ⚠  │ 💰 Treasury: 45.2K  ↑+800/t │            │
│ 🏭 CO₂: 425 ppm  ↑+2.3/t    ⚠  │ 👥 Pop: 2.43M  ↑+1.2K/t    │            │
└──────────────────────────────────────────────────────────────────────────────┘
Height: 44px (two rows of resources). Bottom section of main bar.
```

**Delta indicator rules:**
- `↑` green if delta > 0; `↓` red if delta \< 0; `→` grey if delta == 0
- Warning `⚠` shown if:
  - Joules balance negative (energy shortage)
  - Food delta < -5% of stock per tick (famine risk)
  - CO₂ > 450ppm (climate event zone)
  - Treasury \< 0 (deficit)
- Delta is a rolling 5-tick average (not instantaneous per-tick noise)

**Data source:** `snapshot.metrics` from `sim.tick_broadcast`

**Update frequency:** Every tick (animated counter tick-up on change)

**Accessibility:** Each resource counter has `aria-label="Joules: 4.24 petajoules, increasing 120 terajoules per tick"`.

### 4.3 Minimap

**Purpose:** World overview, alert visualization, click-to-pan navigation.

**Specification:**
- **Size:** 200×150px (fixed), rendered in bottom-right corner
- **Renderer:** Separate Pixi.js `PIXI.RenderTexture`, re-rendered every 5 ticks
- **Scale:** Maps entire world to 200×150px; each minimap pixel &asymp; multiple world tiles
- **Viewport indicator:** White rectangle outline showing current viewport area; draggable
- **Click-to-pan:** Click anywhere on minimap → smooth camera transition to that world position (300ms)

**Overlay toggles on minimap:**
- Nation territories (default on): each nation's territory colored by nation color
- Alert markers: red pulsing dots at alert locations
- Army positions: small colored triangles per army

**Alert visualization:**
- Critical alerts (stability \< 20, energy shortage, war started): red pulsing ring on minimap
- Warning alerts (supply low, drought, insurgency): yellow static dot
- Info alerts: no minimap indicator (only in Alert Feed)

**Performance:** Minimap re-render is batched to every 5 ticks; viewport rectangle updates every frame (cheap `PIXI.Graphics` draw).

### 4.4 Nation Panel (Right)

**Purpose:** Macro-economic and military health at a glance. Visible at Zoom 1; collapses to icon row at Zoom 2.

**Layout:**

```
┌──────────────────────────────────────────┐
│ NATION PANEL                      [–][✕] │
│ ──────────────────────────────────────── │
│ GDP (50-tick sparkline)                  │
│ ▁▂▃▄▅▆▇█▇▆▅▄▃▄▅▆▇█ → 150M (+0.8%/t)   │
│                                          │
│ Stability          73/100                │
│ ████████████░░░░░  [threshold: 10 ▼]    │
│                                          │
│ Population         2.43M                 │
│ by class:  [Farmer: 38%][Craft: 25%]... │
│                                          │
│ Army Strength      68% (vs. avg)         │
│ ████████████░░░░░  12,400 troops         │
│                                          │
│ Happiness (Histogram)                    │
│  80+ ████                               │
│  60-80 ████████████                    │
│  40-60 ████████                        │
│  20-40 ███                             │
│  0-20  █                               │
│  avg: 63   std: 14                      │
│                                          │
│ Legitimacy         61/100               │
│ Credit Rating      AA (★★★★☆)           │
│ Treasury           45,230g  (+800/t)    │
│ Energy Balance     +120 TJ/t  ✓         │
│ CO₂                425 ppm   ⚠          │
└──────────────────────────────────────────┘
Width: 320px. Scrollable if content overflows.
```

**GDP sparkline:** `\<canvas\>` element (50px tall, 280px wide) rendered via Recharts `\<SparkLine\>`. 50-tick rolling window. Y-axis auto-scales to data range.

**Stability meter:** Horizontal progress bar. Color transitions: green (70-100) → yellow (40-70) → orange (20-40) → red (0-20). Threshold line at 10 (collapse threshold from game design).

**Happiness histogram:** Recharts `\<BarChart\>` with 5 buckets. Updated every 10 ticks (not per-tick; histogram computation is O(population)).

**Update frequency:**
- GDP sparkline: appends new point every tick (sparkline auto-scrolls)
- All other meters: every tick
- Histogram: every 10 ticks

### 4.5 Relations Panel

**Purpose:** Diplomatic relationship summary, trust levels, active war/alliance states.

**Layout:**

```
┌──────────────────────────────────────────┐
│ DIPLOMATIC RELATIONS             [expand] │
│ ──────────────────────────────────────── │
│ France    ★★★★☆  trust: 72  [Allied]    │
│ China     ★★☆☆☆  trust: 34  [Neutral]   │
│ USA       ★★★★★  trust: 91  [Alliance]  │
│ Russia    ★☆☆☆☆  trust: 18  [Hostile]   │
│ Ottoman   ★★★☆☆  trust: 55  [Neutral]   │
│                                          │
│ ACTIVE WARS: 0                           │
│ ALLIANCES: 1 (USA)                       │
│                                          │
│ [Propose Treaty]  [View All Nations]     │
└──────────────────────────────────────────┘
```

**Trust encoding:**
- Stars: `Math.round(trust / 20)` stars filled (0-5)
- Color: >= 70 green background; 30-70 neutral; < 30 red background
- Text: `[Allied]` / `[Neutral]` / `[Hostile]` / `[War]`

**Interaction:** Click any nation row → opens Diplomacy Modal with full relationship history, proposed actions (propose alliance, declare war, offer trade, request peace).

**War indicator:** If at war, row flashes at 0.5Hz; `[WAR]` label in `#D55E00`.

### 4.6 Action Panel

**Purpose:** Context-sensitive action buttons. Changes content based on current selection (nothing selected, unit selected, city selected, nation-level).

**States:**

**State: Nothing selected (Nation-level actions)**

```
┌──────────────────────────────────────────┐
│ NATIONAL ACTIONS                         │
│ [Declare War]       [Sign Treaty]        │
│ [Propose Alliance]  [Set Policy]         │
│ [Hire Official]     [Adjust Tax Rate]    │
│ [Allocate Research] [Espionage Budget]   │
└──────────────────────────────────────────┘
```

**State: Unit selected**

```
┌──────────────────────────────────────────┐
│ UNIT ACTIONS — 3rd Infantry              │
│ [▶ Move (Q)]    [⚔ Attack (W)]          │
│ [⛏ Fortify (E)] [↩ Rally (R)]           │
│ [📦 Resupply]   [⏺ Queue Commands]      │
│ [☠ Disband]     [⬆ Upgrade]             │
└──────────────────────────────────────────┘
```

**State: City/District selected**

```
┌──────────────────────────────────────────┐
│ DISTRICT ACTIONS — North Quarter         │
│ [🏗 Build (B)]  [👷 Workers]             │
│ [📋 Policy]     [💰 Local Tax]           │
│ [🛡 Recruit]    [🏛 Institutions]        │
│ [📊 Statistics] [🔄 Trade Routes]       │
└──────────────────────────────────────────┘
```

**All action buttons:**
- Have keyboard shortcut label in `( )` where applicable
- Disabled (greyed out with `opacity: 0.4`) if action not available (e.g., insufficient resources)
- Disabled buttons show tooltip with reason: "Insufficient treasury (need 500g, have 45g)"
- Emit `sim.command` WebSocket message on click

### 4.7 Alert Feed

**Purpose:** Live scrolling list of simulation events. Player's primary notification channel.

**Layout:**

```
┌──────────────────────────────────────────────────────────────────────────┐
│ EVENTS  Filter: [All▼] [Economy] [Military] [Climate] [Diplomacy]  [✕×3]│
│ ─────────────────────────────────────────────────────────────────────── │
│ [⚠ T:1247] LOW SUPPLY: 3rd Army in N.Province — 30% remaining   [→ Map] │
│ [⚡ T:1245] ENERGY SHORTAGE: W.District offline — 12% penalty    [→ Map] │
│ [🌾 T:1244] DROUGHT BEGAN: N.Province food output -40% for 200t  [→ Map] │
│ [⚔ T:1240] BATTLE RESULT: vs. France at Calais — Victory (+120 XP)       │
│ [📜 T:1238] TREATY OFFERED: France proposes trade agreement       [Accept]│
│ [👥 T:1235] MIGRATION: 1,200 citizens fled W.District to Capital          │
│ [🔬 T:1230] TECH: Steam Engine research +15% this tick                    │
│ [💰 T:1228] MARKET: Wheat price +12% (shortage in S.Province)            │
└──────────────────────────────────────────────────────────────────────────┘
Height: 200px (collapsible to 44px single-line summary). Overflowing content scrollable.
```

**Alert categories and icons:**

| Category | Icon | Color | Example Events |
|----------|------|-------|----------------|
| Economy | 💰 | `#E69F00` | Price changes, trade events, treasury |
| Military | ⚔ | `#D55E00` | Battle results, unit deaths, siege |
| Climate | 🌡 | `#56B4E9` | Drought, flood, CO₂ thresholds |
| Social | 👥 | `#CC79A7` | Migration, ideology shift, insurgency |
| Diplomacy | 📜 | `#009E73` | Treaty offers, war declarations |
| Research | 🔬 | `#F0E442` | Tech breakthroughs, research progress |
| Critical | ⚠ | `#D55E00` blink | Stability \< 20, collapse imminent |

**Interaction:**
- Click `[→ Map]`: smooth camera pan to event location, highlight affected unit/district
- Click `[Accept]` / `[Reject]` for actionable events (treaty offers): fires `sim.command`
- Click alert row: select the entity involved (unit, district, nation)
- Filter dropdown: show only selected category

**Performance:** Alert feed is a virtualized list (react-virtualized or `@tanstack/virtual`). Only visible rows are DOM nodes. Max 1000 alerts in memory; older alerts paginated.

**Accessibility:** New alerts announced via `aria-live="polite"` region. Critical alerts use `aria-live="assertive"`.

### 4.8 Research Tree Panel

**Purpose:** Technology dependency graph. Player allocates research budget to progress tech.

**Layout:**

```
┌──────────────────────────────────────────────────────────────────────────┐
│ RESEARCH TREE                                          Budget: 5,000g/yr │
│ [Filter: All ▼]   [Era: Industrial]              [Search: ____________] │
│                                                                          │
│  Bronze Age              Iron Age              Steam Age                 │
│  ─────────────           ─────────────         ─────────────────         │
│  [Iron Working ✓]──────► [Steel Forging ░░░──] ──► [Bessemer Process ░] │
│                            72% complete             (locked)             │
│                            6 ticks remain                                │
│                                                                          │
│  [Agriculture ✓]──────────────────────────────► [Crop Rotation ✓]       │
│                                                                          │
│  [Writing ✓]───► [Mathematics ░░░░░░░░░────] ──► [Scientific Method]    │
│                   14% complete                      (locked)             │
│                                                                          │
│  SELECTED: Steel Forging                                                 │
│  Cost: 500g/tick × 8 ticks = 4,000g total                               │
│  Unlocks: [Bessemer Process] [Improved Cannon] [Steam Locomotive]        │
│  [Increase Budget ▲]  [Decrease Budget ▼]   [Set as Priority]           │
└──────────────────────────────────────────────────────────────────────────┘
Width: full screen overlay (800px max-width, centered). Opened via toolbar button or F6.
```

**Graph rendering:** D3.js DAG layout (`d3-dag` library). Nodes are tech items. Edges are prerequisites. Pan/zoom on the graph uses D3 zoom behavior.

**Node states:**

| State | Visual |
|-------|--------|
| Completed | Solid green fill, white text, checkmark |
| In Progress | Partial fill (progress %), orange border |
| Available | Hollow, grey border, clickable |
| Locked | Greyed out, lock icon, not clickable |

**Budget allocation:** Player sets research budget as `g/tick`. Tech in progress receives budget proportional to priority. Multiple techs can be in progress simultaneously (budget split).

### 4.9 Bottom Timeline

**Purpose:** Tick scrubber (replay mode), event markers, playback controls.

**Layout:**

```
┌──────────────────────────────────────────────────────────────────────────┐
│ [◄◄ Start] [◄ Back]  [──────────────●──────────────────────────]  [Export│
│ [⏸ Pause]  [► Play]   Tick 847/1247   ▲ battle  ▲ treaty  ▲ disaster    │
│                        Year 34, Q2                                        │
└──────────────────────────────────────────────────────────────────────────┘
Height: 44px.
```

**Timeline scrubber:**
- `input[type=range]` with `min=0 max={totalTicks} value={currentTick}`
- Event markers rendered as colored triangles above scrubber track:
  - Battle: `⚔` red marker
  - Treaty: `📜` green marker
  - Climate event: `🌡` blue marker
  - Tech breakthrough: `🔬` yellow marker
- Click on marker: jump to that tick
- Drag scrubber: jumps to tick in replay mode; disabled in live mode (shows current tick as read-only position)

**Replay mode:** Activated by `sim.snapshot` requests. In replay mode, timeline is interactive; in live mode, scrubber auto-advances with ticks.

**Export button:** Downloads `.civreplay` file (complete event log from session start to current tick).

---

## 5. Overlay System

Overlays are toggled via the Strategic Overlay panel (Zoom 1 sidebar) or `F1`–`F5` hotkeys. Multiple overlays can be active simultaneously (they blend via `PIXI.Graphics` `blendMode`). All overlay renders are separate `PIXI.Container` layers above the base tile layer.

### 5.1 Economy Overlay

**Toggle:** `F1` key or `○ Economy` sidebar button

**Visualizations:**

**GDP choropleth:**
- Each district/region colored by GDP per capita
- Color scale: `#16213E` (low) → `#009E73` (median) → `#E69F00` (high)
- Computed via D3.js `scaleQuantize` with 7 buckets
- Legend rendered in bottom-left corner of overlay

**Trade route arrows:**
- SVG arrows overlaid on the Pixi.js canvas via an absolutely-positioned `\<svg\>` element
- Arrow thickness proportional to trade volume (cubic bezier paths)
- Color: `#E69F00` for active; `#56B4E9` for proposed; `#D55E00` for embargoed
- Animated: dashed stroke-dashoffset animation shows flow direction

**Price heat map:**
- Triggered by selecting a specific good (wheat, metal, etc.) from overlay controls
- Districts colored by price of selected good
- Clicking a trade arrow shows detailed breakdown tooltip

**Data source:** `snapshot.world.districts[*].gdp`, `snapshot.world.trade_routes`

**Update frequency:** Every 5 ticks (choropleth is expensive to recompute)

### 5.2 Military Overlay

**Toggle:** `F2` key or `○ Military` sidebar button

**Visualizations:**

**Unit positions:**
- All units (own + visible enemy + allied) shown as faction-colored dots
- Own units: solid fill; enemy: hollow ring; allied: triangle
- Cluster indicator: if > 10 units in same tile, show count badge

**Supply line chains:**
- Lines from supply depots to each connected army
- Color encodes supply status:
  - `#009E73` (green): supply >= 75%
  - `#E69F00` (yellow): supply 25-75%
  - `#D55E00` (red): supply \< 25% or severed
- Dashed line if supply route is contested

**Territory control borders:**
- Bold outline of each nation's territory; fill color at 15% opacity
- Border flashes at disputed hexes (contested by multiple nations)

**Data source:** `snapshot.world.units`, `snapshot.world.supply_lines`, `snapshot.world.territory`

### 5.3 Climate Overlay

**Toggle:** `F3` key or `○ Climate` sidebar button

**Visualizations:**

**Temperature gradient:**
- Full-map gradient from pole (blue) to equator (orange-red)
- Modulated by current climate state (drought = hotter; flood = cooler)
- D3.js diverging color scale centered on global average temperature

**CO₂ emission sources:**
- Bubble chart overlaid on map: bubble center = district, bubble radius = CO₂/tick emission
- Bubble color: red for coal/oil, orange for gas, white for nuclear, green for renewables
- Animated breathing effect at 2-second period for active sources

**Renewable vs. fossil split:**
- Pie chart in corner showing global energy mix (solar %, wind %, coal %, etc.)
- Per-district bar visible on hover

**Carbon budget indicator:**
- Horizontal bar across top of overlay showing global CO₂ ppm
- Zones: safe (< 350), warning (350-450), danger (450-550), catastrophe (> 550)
- Current value marker animated tick-by-tick

**Data source:** `snapshot.metrics.co2_ppm`, `snapshot.world.districts[*].energy_mix`

### 5.4 Social Overlay

**Toggle:** `F4` key or `○ Social` sidebar button

**Visualizations:**

**Ideology distribution:**
- Each district colored by dominant faction ideology
- Color scale: `#D55E00` (full autocracy) → `#CC79A7` (mixed) → `#56B4E9` (full democracy)
- Pie chart on hover showing exact faction split

**Insurgency hotspots:**
- Districts with insurgency risk > 20%: pulsing orange ring
- Districts in active insurgency: red fill with animated unrest icon

**Migration flow arrows:**
- Animated arrows showing net migration between districts (size = net migrants/tick)
- Only shown for flows > 50 citizens/tick
- Arrow head direction = destination

**Happiness heat map:**
- Alternative mode (toggle in overlay): district colored by average happiness
- Scale: `#D55E00` (0-20 happiness) → `#E69F00` (40-60) → `#009E73` (80-100)

**Data source:** `snapshot.world.districts[*].ideology_distribution`, `snapshot.world.districts[*].insurgency_level`, `snapshot.world.migration_flows`

### 5.5 Diplomacy Overlay

**Toggle:** `F5` key or `○ Diplomacy` sidebar button

**Visualizations:**

**Alliance blocs:**
- Nations in same alliance share a color band (2px colored outline + 5% territory fill tint)
- Color bands use the Okabe-Ito palette cycling through available colors
- Alliance name shown in center of alliance territory cluster

**War theaters:**
- Active war zones: red translucent fill over contested territory
- Front lines: animated red pulsing border

**Shadow network edges (dashed lines):**
- Known espionage relationships: dashed grey lines between nations
- Successfully uncovered operations only (player has intelligence)
- Opacity reflects recency: recent = 80%, older = 20%

**Trust heatmap:**
- Toggle within diplomacy overlay: nations colored by bilateral trust to player's nation
- Green = high trust; red = low trust; grey = no contact

**Data source:** `snapshot.world.alliances`, `snapshot.world.wars`, `snapshot.world.diplomacy_relations`

---

## 6. Unit & Entity Rendering

### 6.1 Sprite Specifications

**Unit base sprite:** 32×32px at Zoom 2. 16×16px at Zoom 1 (strategic; mostly icons not sprites).

**Unit types:**

| Type | Sprite Set | Animation Frames | Directions |
|------|-----------|-----------------|------------|
| Infantry | `infantry_*` | 4 (idle), 8 (walk), 4 (attack) | 8 |
| Cavalry | `cavalry_*` | 4 (idle), 8 (walk/charge), 4 (attack) | 8 |
| Archer | `archer_*` | 4 (idle), 8 (walk), 4 (shoot) | 8 |
| Siege | `siege_*` | 4 (idle), 4 (deploy), 4 (fire) | 4 |
| Worker | `worker_*` | 4 (idle), 4 (work), 4 (walk) | 4 |
| Merchant | `merchant_*` | 4 (idle), 4 (walk) | 4 |
| Spy | `spy_*` | 4 (idle), 4 (walk) | 4 |
| Scout | `scout_*` | 4 (idle), 4 (walk), 4 (reveal) | 8 |

**Total atlas size calculation:**
- 8 unit types × 8 directions × max(16 frames per type) = 1024 frames maximum
- At 32×32px per frame: 32,768px² per type, target atlas: 2048×2048px (4 types per atlas)
- Two atlases: `units_atlas_01.png` (infantry, cavalry, archer, siege) and `units_atlas_02.png` (worker, merchant, spy, scout)

**Building sprites:** 64×64px base. Single-frame (no animation for buildings). Separate atlas.

**Nation flag badge:** 16×16px. Rendered as `PIXI.Sprite` overlay on unit sprite. One flag atlas per scenario loaded at start.

**Health bar:** Rendered as `PIXI.Graphics` thin rectangle (32px wide × 3px tall) below unit sprite. Color: green if HP > 60%, yellow if 30-60%, red if \< 30%.

**Supply bar:** Same dimensions as health bar, 2px below health bar. Color: `#56B4E9` (blue). Only shown for military units.

**Level badge:** Small `★` badge in top-right corner of sprite for units level 3+. Filled stars = level / 2 (max 5 stars for level 10).

### 6.2 Selection System

**Single click:** Select unit under cursor. Opens Action Panel for that unit. Previous selection deselected.

**Shift + click:** Toggle unit in/out of multi-selection set. Action Panel shows shared actions.

**Drag-box select:**
- Mouse down at `(x1, y1)`, drag to `(x2, y2)`: draw selection rectangle in Layer 8
- On mouse up: select all units with center within bounding box
- `PIXI.Graphics` rectangle with `#56B4E9` border, 10% fill opacity

**Double click:** Select all units of same type currently visible in viewport. Triggers viewport pan to show all selected if spread across viewport.

**Selection state rendering:**
- Selected unit: pulsing `#E69F00` ring around sprite (1.5px thick, scale: 1.0→1.15→1.0 at 1Hz)
- Multi-selected units: same ring plus a thin line connecting to selection centroid
- Max visual selection: first 100 units show individual indicators; `+N more` badge for overflow

**Keyboard:**
- `Ctrl+A`: select all visible units of current player
- `Escape`: deselect all
- `Delete` (with units selected): opens confirm dialog → `sim.command` type `disband`

### 6.3 Formation Display

When multiple units are selected and a formation type is chosen:

**Formation visualization:**
- Lines drawn between adjacent units in formation (dashed `#56B4E9` lines)
- Formation shape outline (ghost polygon showing target shape at destination)
- Leader unit highlighted with crown badge

**Formation types (from FR-CIV-RTS-003):**

| Formation | Shape | Use Case |
|-----------|-------|---------|
| Wedge | Triangle, point forward | Offensive charge |
| Line | Horizontal row | Frontal engagement |
| Column | Vertical file | Movement through narrow terrain |
| Circle | Defensive ring | Surrounded defense |

**Drag-to-reorder:** Within a selected formation group, units can be drag-reassigned to different formation slots. Slot highlight on hover; swap on drop.

### 6.4 Path Visualization

When a move command is issued (unit selected + right-click target or `Q` hotkey + click):

**Path rendering:**
- A* pathfinding result displayed as animated dashed polyline on tile layer
- Dashes animate along path direction (stroke-dashoffset CSS animation equivalent in PIXI)
- Color: `#56B4E9` for friendly move; `#D55E00` for attack move
- Obstacle hexes along path highlighted with orange outline
- ETA badge at path endpoint: "~6 ticks"

**Move range indicator:**
- When unit selected but no move command yet: hex tiles within move range highlighted with `#56B4E9` at 20% fill opacity
- Tiles reachable this tick: bright highlight; tiles reachable in 2 ticks: dimmer

**Queued commands display:**
- If unit has queued commands (FR-CIV-RTS-004): numbered waypoints shown along path (◉1 ◉2 ◉3)
- Click waypoint to cancel commands from that point onward

### 6.5 Combat Visualization

**Hit effect:** When a unit takes damage — white flash overlay on sprite for 3 frames (50ms).

**Damage number:** Floating `−N` text (red, bold, 14px) rises 30px above target unit over 800ms then fades. Critical hits (> 2× average damage): larger text, `#E69F00` color with exclamation.

**Death animation:**
1. Unit sprite: scale to 1.3× (impact) over 100ms
2. Scale back to 0 over 200ms (collapse)
3. Particle burst: 8 particles in faction color fly outward, fade over 400ms
4. Remove from scene graph

**Morale break (routing):**
- Unit sprite gets a "!" badge overlay
- Movement path changes to retreat direction (automatic pathfinding away from enemy)
- Speed multiplied by 1.5 (routing units run)
- Alert in Alert Feed: "3rd Infantry routing!"

**Battle result popup:**
- After combat resolves: floating combat summary appears at battle location
- Shows: attacker casualties, defender casualties, outcome text
- Auto-dismisses after 3 seconds

---

## 7. Zoom Transition System

### 7.1 Zoom 1 to Zoom 2

**Trigger:** Player scrolls into zoom factor 3.0 or clicks a specific city/region.

**Transition sequence:**

```
Frame 0 (tick T):
  - Player clicks on "Paris Region" or scrolls to zoom factor >= 3.0
  - Begin transition animation

Frames 1-30 (500ms total, ease-in-out-cubic):
  - Camera zooms from zoom_factor 1.0 → 4.0 centered on clicked location
  - Zoom 1 strategic overlays (nation colors, diplomatic lines): alpha 1.0 → 0.0
  - Zoom 2 district tiles: alpha 0.0 → 1.0 (fade in as zoom increases)
  - Left sidebar transitions: Overlay panel slides out; District panel slides in
  - Right sidebar: Nation panel collapses to 160px; Unit panel appears

Frame 31 (transition complete):
  - Zoom level locked to 2
  - Fog of war activates (unit vision ranges used)
  - Individual building sprites visible
```

**UI panel transition:**
- Left sidebar content cross-fades (150ms fade out old, 150ms fade in new)
- Right sidebar content cross-fades
- Bottom bar: resource counters remain; Timeline tick continues

### 7.2 Zoom 2 to Zoom 3

**Trigger:** Double-click on a citizen entity (citizen dots appear when zoom factor >= 6.0) or from Research Mode button.

**Transition sequence:**

```
Frame 0:
  - Player double-clicks citizen entity #48291
  - Camera zoom continues to 8.0 (citizen level)
  - Begin citizen card slide-in

Frames 1-20 (300ms total):
  - Map zooms to citizen's tile (fills viewport)
  - Citizen dot expands to full 32×32px portrait placeholder
  - Citizen card panel slides in from right edge (translateX: 100% → 0)

Frame 21 (transition complete):
  - Full citizen profile visible
  - Social graph renders (D3.js force layout, 500ms settle animation)
  - Happiness sparkline loads historical data (sim.query request)
```

**Back to Zoom 2:**
- `Escape` key or back breadcrumb → reverse transition (300ms)
- Map zooms out, citizen card slides out right

### 7.3 LOD Switching Rules

The level-of-detail system governs what is rendered at each zoom factor (continuous zoom, not discrete levels):

| Zoom Factor | What Renders |
|-------------|-------------|
| 0.25 – 1.5 | Nation territory blocs (colored regions), no tile detail |
| 1.5 – 3.0 | Transition zone: territories fade, biome tiles emerge |
| 3.0 – 5.0 | Individual hex tiles, district outlines, building footprints (no sprites) |
| 5.0 – 6.5 | Full building sprites (64×64px), unit sprites (32×32px), health bars visible |
| 6.5 – 8.0 | Citizen dots appear (4×4px per citizen cluster), building detail textures |
| > 8.0 | Individual citizen portrait icons (zoom 3 mode) |

**LOD transitions are smooth:** Each LOD layer has opacity that ramps linearly between the zoom factor thresholds (&plusmn;0.5 zoom factor crossfade range).

**Pixi.js implementation:**

```typescript
function updateLODVisibility(zoomFactor: number, layers: LODLayers): void {
  layers.nationBlocs.alpha = clampedLerp(1.0, 0.0, zoomFactor, 1.0, 2.0);
  layers.biomeTiles.alpha  = clampedLerp(0.0, 1.0, zoomFactor, 1.5, 3.0);
  layers.buildingSprites.alpha = clampedLerp(0.0, 1.0, zoomFactor, 4.5, 5.5);
  layers.citizenDots.alpha = clampedLerp(0.0, 1.0, zoomFactor, 6.0, 7.0);
}

function clampedLerp(fromAlpha: number, toAlpha: number, z: number, zStart: number, zEnd: number): number {
  const t = Math.max(0, Math.min(1, (z - zStart) / (zEnd - zStart)));
  return fromAlpha + (toAlpha - fromAlpha) * t;
}
```

### 7.4 Performance Targets

| Metric | Target | Constraint |
|--------|--------|------------|
| Steady-state render | 60fps | No frame drop during normal play |
| Zoom transition | 60fps | No drop during 500ms transition |
| Minimum acceptable | 45fps | Below this: reduce particle count |
| Unit count at 60fps | 10,000 | Via PIXI.ParticleContainer |
| Tile count at 60fps | 50,000 | Via dirty-region tile update |
| Fog texture update | < 5ms | Per tick |
| A* pathfinding | < 10ms | Per unit movement command |

**Adaptive quality:**
- If frame time > 22ms (45fps), disable non-critical effects in order: particle effects, fog gradients, trade route animations, minimap overlay
- `window.devicePixelRatio` capped at 2.0 for Pixi.js canvas to control GPU load on high-DPI screens

---

## 8. Input & Hotkey System

### 8.1 Hotkey Table

**Simulation Control:**

| Key | Action | Notes |
|-----|--------|-------|
| `Space` | Pause / Resume simulation | Always active |
| `1` | Zoom preset: Zoom 1 (Strategic) | Camera transition 500ms |
| `2` | Zoom preset: Zoom 2 (Tactical) | Camera transition 500ms |
| `3` | Zoom preset: Zoom 3 (Citizen) | Requires citizen selected |
| `+` / `=` | Zoom in one step (1.25×) | |
| `-` | Zoom out one step (0.8×) | |
| `Arrow Up/Down/Left/Right` | Pan map | Acceleration after 10 frames |
| `Home` | Pan to player capital | Smooth transition 400ms |

**Overlay Toggles:**

| Key | Action | Notes |
|-----|--------|-------|
| `F1` | Toggle Economy overlay | |
| `F2` | Toggle Military overlay | |
| `F3` | Toggle Climate overlay | |
| `F4` | Toggle Social overlay | |
| `F5` | Toggle Diplomacy overlay | |
| `F6` | Open / Close Research Tree panel | |
| `F7` | Toggle Minimap | |
| `F8` | Toggle Alert Feed expanded / collapsed | |
| `F9` | Open / Close Research Mode dashboard | |

**Unit Commands (when unit(s) selected):**

| Key | Action | FR Reference |
|-----|--------|-------------|
| `Q` | Move command (click target to confirm) | FR-CIV-RTS-001 |
| `W` | Attack command (click target) | FR-CIV-RTS-002 |
| `E` | Fortify (unit digs in, +defense bonus) | FR-CIV-RTS-002 |
| `R` | Rally (return to nearest friendly city) | FR-CIV-RTS-001 |
| `T` | Queue next command (toggle queue mode) | FR-CIV-RTS-004 |
| `Ctrl+A` | Select all visible friendly units | FR-CIV-RTS-003 |
| `Delete` | Disband selected units (with confirm dialog) | |
| `G` | Go to selected unit location (pan camera) | |

**Alert & Navigation:**

| Key | Action | Notes |
|-----|--------|-------|
| `Tab` | Cycle through active critical alerts | Pans camera to each |
| `G` | Go to most recent alert location | |
| `N` | Pan to next unresolved alert | |
| `Escape` | Deselect all / Close active panel / Zoom out one level | Priority: deselect > close panel > zoom out |

**File & Session:**

| Key | Action | Notes |
|-----|--------|-------|
| `Ctrl+S` | Quick save (.civreplay download) | Serializes event log |
| `Ctrl+Z` | Undo last action (sandbox mode only) | Not available in real-time mode |
| `Ctrl+Shift+C` | Open cheats console (sandbox only) | `cheat: \<command\>` |
| `Shift+T` | Toggle light/dark theme | |

**Formation Hotkeys (when multiple units selected):**

| Key | Action | FR Reference |
|-----|--------|-------------|
| `Shift+1` | Wedge formation | FR-CIV-RTS-003 |
| `Shift+2` | Line formation | FR-CIV-RTS-003 |
| `Shift+3` | Column formation | FR-CIV-RTS-003 |
| `Shift+4` | Circle (defensive) formation | FR-CIV-RTS-003 |

### 8.2 Custom Hotkey Binding System

Players can rebind any hotkey via a JSON configuration file or the in-game settings panel.

**Config file location:** `~/.civlab/hotkeys.json` (or `%APPDATA%\civlab\hotkeys.json` on Windows)

**Schema:**

```typescript
interface HotkeyConfig {
  version: "1";
  profile: string;           // "default" | "custom-profile-name"
  bindings: {
    [actionId: string]: {
      key: string;            // "Space", "F1", "q", "Ctrl+S"
      description: string;   // Human-readable
    };
  };
}
```

**Example:**

```json
{
  "version": "1",
  "profile": "vi-style",
  "bindings": {
    "unit_move":    { "key": "m", "description": "Move unit" },
    "unit_attack":  { "key": "a", "description": "Attack" },
    "unit_fortify": { "key": "f", "description": "Fortify" },
    "unit_rally":   { "key": "r", "description": "Rally" },
    "pause":        { "key": "p", "description": "Pause/Resume" }
  }
}
```

**Live reload:** The client watches `hotkeys.json` via `BroadcastChannel` (service worker) or `window.focus` event re-read. Changes take effect without page reload.

**Conflict detection:** If two actions share the same key in the same context, the settings panel shows a red warning. Conflicts do not prevent gameplay — later binding wins.

**Binding contexts:**
- Global (always active): pause, zoom, pan, overlay toggles
- Unit-selected: move, attack, fortify, rally, formation
- Panel-focused: panel-internal navigation (Tab, arrow keys within panel)

Context priority: Panel-focused > Unit-selected > Global

---

## 9. Research Mode UI

Research Mode is a separate UI layer activated by `?mode=research` URL parameter, in-game `F9` key, or the Python `civlab` headless API. It provides time-series analysis, parameter sweeps, and comparative run visualization.

### 9.1 Time-Series Chart Panel

**Purpose:** Plot simulation metrics over time from a completed or in-progress run.

**Layout:**

```
┌──────────────────────────────────────────────────────────────────────────┐
│ TIME-SERIES CHARTS                    Run: modern_era_001  Seed: 54321   │
│ [+ Add Chart]  [Export All]  [Share Link]    Tick range: [0] to [1247]   │
│ ─────────────────────────────────────────────────────────────────────── │
│ CHART 1: GDP + Stability                     [Edit] [Remove] [↕ Resize]  │
│ │                                                                        │
│ │  150M ┤                                     ╭────╮                    │
│ │  100M ┤           ╭──╮    ╭────────────────╯    ╰──────               │
│ │   50M ┤──────────╯  ╰────╯                                            │
│ │     0 ┼──────────────────────────────────────────────────── Tick      │
│ │       0          250         500          750        1000  1247        │
│ │  Stab: ──── (right axis, 0-100)                                        │
│ │  GDP:  ──── (left axis, USD)                                           │
│                                                                          │
│ CHART 2: CO₂ ppm vs. Renewable Energy %   [Edit] [Remove]               │
│ │  600 ┤                     ╭──────────────────────────────            │
│ │  450 ┤          ╭──────────╯                                          │
│ │  350 ┤──────────╯                                                     │
│ │  350 ┼─────────────────────────────────────────────────── Tick        │
│                                                                          │
│ [+ Add Metric]  Available: GDP, Stability, CO₂, Population, Happiness,  │
│                  Army Strength, Legitimacy, Treasury, Energy, Trade Vol   │
└──────────────────────────────────────────────────────────────────────────┘
```

**Chart library:** Recharts `\<LineChart\>` with responsive container. Each chart is independently configurable.

**Chart configuration modal:**

```
Metric A: [GDP ▼]         Color: [████]  Axis: [Left ▼]
Metric B: [Stability ▼]   Color: [████]  Axis: [Right ▼]
Smoothing: [5-tick rolling average ▼]
Y-axis range: [Auto ▼]
Reference lines: [+ Add threshold line]
```

**Interaction:**
- Hover on chart: crosshair cursor shows exact values at that tick for all series
- Click on tick in chart: jump simulation to that tick (replay mode) or show event log at that tick
- Brush selection: drag to select tick range; all other charts zoom to same range (linked brushing)

### 9.2 Parameter Sweep Configuration

**Purpose:** Define multi-run sweeps (run same scenario N times with varied parameters).

**Layout:**

```
┌──────────────────────────────────────────────────────────────────────────┐
│ PARAMETER SWEEP                                                          │
│ Scenario: [modern_era.yaml ▼]    Base Seed: [54321]                     │
│ ─────────────────────────────────────────────────────────────────────── │
│ SWEEP VARIABLES                                                          │
│ [+ Add Variable]                                                         │
│                                                                          │
│ 1. carbon_budget_limit     Range: [300] to [600]  Steps: [4]            │
│    Values: 300, 400, 500, 600                                            │
│                                                                          │
│ 2. ai_difficulty            Values: [easy, medium, hard]  (categorical)  │
│                                                                          │
│ RUNS PER COMBINATION: [10]                                               │
│ TOTAL RUNS: 4 × 3 × 10 = 120                                            │
│ ESTIMATED TIME: ~120 × 0.8s/run = 96 seconds (headless mode)            │
│                                                                          │
│ VICTORY CONDITIONS TO TRACK: [All ▼]                                    │
│ EXPORT FORMAT: [CSV ▼]  [JSON]  [Parquet]                               │
│                                                                          │
│ [▶ Run Sweep (headless)]  [Preview Config YAML]  [Load Saved Config]    │
└──────────────────────────────────────────────────────────────────────────┘
```

**Output:** Each sweep generates a dataset where each row = one run result:

```csv
carbon_budget,ai_difficulty,seed,ticks_to_victory,final_stability,final_gdp,final_co2,outcome
300,easy,0,750,82.3,145000000,320,stability_victory
300,easy,1,890,78.1,132000000,315,stability_victory
...
```

**Progress display:** During sweep, a progress bar shows `run X of 120`. Streaming results update charts in real-time.

### 9.3 Divergence Viewer

**Purpose:** Compare two runs side by side; identify the tick where outcomes first diverged.

**Layout:**

```
┌──────────────────────────────────────────────────────────────────────────┐
│ DIVERGENCE VIEWER                                                        │
│ Run A: modern_era_001 (carbon=300)     Run B: modern_era_002 (carbon=600)│
│ ─────────────────────────────────────────────────────────────────────── │
│ FIRST DIVERGENCE: Tick 423 — Climate: Drought in N.Province             │
│                   [← 50 ticks before] [Jump to divergence →]            │
│ ─────────────────────────────────────────────────────────────────────── │
│ SIDE BY SIDE (Tick 500)           ▲ Divergence marker                   │
│ ┌───────────────────┐  │  ┌───────────────────┐                         │
│ │ Run A             │  │  │ Run B             │                         │
│ │ GDP: 89M          │  ←  │ GDP: 84M  (-5.6%) │                         │
│ │ Stability: 76     │  ←  │ Stability: 62     │                         │
│ │ CO₂: 310ppm       │  ←  │ CO₂: 445ppm  ⚠   │                         │
│ │ Population: 2.1M  │  =  │ Population: 2.1M  │                         │
│ └───────────────────┘  │  └───────────────────┘                         │
│                                                                          │
│ DIFF LOG (events only in one run):                                       │
│  T:423  A: no drought      B: drought.N.Province (co2 trigger)          │
│  T:450  A: stable          B: stability -8 (food shortage)               │
│  T:480  A: treaty.France   B: no treaty (France hostile, war risk)       │
└──────────────────────────────────────────────────────────────────────────┘
```

**Divergence detection algorithm:**
1. Load event logs from two runs
2. Walk through ticks in parallel
3. At each tick, compare state hash (`SHA256(state_serialized)`)
4. First tick where hashes differ = divergence point
5. Diff events from that tick onward to show what caused the divergence

### 9.4 Export Controls

**Formats supported:**

| Format | Content | Use Case |
|--------|---------|---------|
| `.civreplay` | Full event log (JSON, gzip) | Replay/share runs |
| `.csv` | Metrics time-series | Excel/Python analysis |
| `.json` | Full snapshot at any tick | Custom analysis |
| `.parquet` | Columnar metrics (sweep output) | Pandas/R/Spark |
| `.png` | Current chart export | Papers/presentations |
| `.svg` | Overlay map export | Publication-quality |

**Export UI:**

```
┌──────────────────────────────────────────────────────────────────────────┐
│ EXPORT                                                                   │
│ Scope: [Full run ▼]  Tick range: [0] to [1247]                          │
│                                                                          │
│ ○ .civreplay (full event log, 2.3 MB estimated)                          │
│ ● .csv (metrics only, 145 KB estimated)                                  │
│   Metrics: [✓ GDP] [✓ Stability] [✓ CO₂] [✓ Pop] [✓ Happiness] [All]  │
│   Tick resolution: [Every tick ▼]                                        │
│                                                                          │
│ [Download]  [Copy API URL]  [Upload to CivLab Hub]                      │
└──────────────────────────────────────────────────────────────────────────┘
```

### 9.5 Citizen Micro-View

**Purpose:** Statistical view of a random citizen sample; tracks ideology shift and happiness evolution.

**Layout:**

```
┌──────────────────────────────────────────────────────────────────────────┐
│ CITIZEN MICRO-VIEW              Sample: 1000 random citizens             │
│ Nation: Great Britain  District: All  Tick: 1247   [Re-sample]          │
│ ─────────────────────────────────────────────────────────────────────── │
│ IDEOLOGY SCATTER (tick 1247)    HAPPINESS OVER TIME                     │
│                                                                          │
│  Democracy                        80 ┤      ╭────────╮                  │
│  100 ┤  ·  ·           · ·          ┤   ╭──╯        ╰──╮               │
│   80 ┤ ·  ·····      ·· ·       60 ┤─╭╯               ╰──             │
│   60 ┤ ·  · ··██████·· ·           ┤                                   │
│   40 ┤ · ··  ·█████·· ··       40 ┼─────────────────────── Tick       │
│   20 ┤  ·   ·  ··                 0       250     500    750  1000      │
│    0 ┼──────────────── Happiness                                        │
│       0  20  40  60  80  100                                            │
│    Autocracy                      Blue: mean  Band: &plusmn;1 std              │
│                                                                          │
│ SELECTED CITIZEN CLUSTERS        TOP BELIEF THEMES (by count)           │
│ ● Cluster A: 342 (autocrat, poor)  1. "King is corrupt"      (34%)      │
│ ● Cluster B: 289 (mixed, moderate) 2. "Trade is good"        (52%)      │
│ ● Cluster C: 178 (democrat, rich)  3. "Army is weak"         (18%)      │
│ ● Unclustered: 191                 4. "Drought is coming"     (9%)      │
│                                                                          │
│ [Run K-Means]  [Export Scatter Data]  [Animate Over Time ▶]             │
└──────────────────────────────────────────────────────────────────────────┘
```

**Scatter plot:** D3.js `\<svg\>` scatter with `circle` elements per citizen. Radius 3px. Color by class. Hover shows citizen ID and stats.

**Animation:** "Animate Over Time" plays scatter plot as 1-tick-per-frame animation from run start to current tick.

---

## 10. Web Client Stack

### 10.1 Framework & Dependencies

**Core:**

| Library | Version | Purpose |
|---------|---------|---------|
| React | 19.x | Component framework, HUD rendering |
| TypeScript | 5.x | Type safety, strict mode |
| Pixi.js | 8.x | WebGL game canvas rendering |
| Vite | 6.x | Build tool, HMR |

**State Management:**

| Library | Version | Purpose |
|---------|---------|---------|
| Zustand | 5.x | Game state slices |
| Immer | 10.x | Immutable state updates (via Zustand middleware) |

**UI Components:**

| Library | Version | Purpose |
|---------|---------|---------|
| Radix UI | 2.x | Accessible primitives (Dialog, DropdownMenu, Tooltip, etc.) |
| Tailwind CSS | 4.x | Utility CSS; CSS custom properties for theme tokens |

**Charts & Visualization:**

| Library | Version | Purpose |
|---------|---------|---------|
| Recharts | 2.x | Time-series charts, sparklines, histograms in Research Mode |
| D3.js | 7.x | Custom overlays, DAG layout (tech tree), force graph (social), scatter plots |

**Networking:**

| Library | Version | Purpose |
|---------|---------|---------|
| Native WebSocket | - | Protocol layer |
| `@civlab/client` | internal | CivLab JSON-RPC client wrapper (exponential backoff reconnect) |

**Utilities:**

| Library | Version | Purpose |
|---------|---------|---------|
| `react-virtualized-auto-sizer` | 1.x | Virtualized alert feed list |
| `@tanstack/virtual` | 3.x | Alert list virtualization |
| `date-fns` | 3.x | Tick → date display formatting |
| `zstd-wasm` | - | Binary frame decompression (zstd) |

### 10.2 State Management (Zustand)

**Store slices:**

```typescript
// Game state stores (Zustand slices)

interface MapState {
  tiles: Map<string, Tile>;          // key: "q,r"
  fogArray: Uint8Array;              // fog-of-war state
  cameraState: CameraState;         // {x, y, zoom}
  activeOverlays: Set<OverlayType>; // Economy, Military, etc.
}

interface SelectionState {
  selectedUnits: number[];           // unit entity IDs
  selectedDistrict: number | null;
  selectedCitizen: number | null;
  formationType: FormationType | null;
}

interface SimState {
  tick: number;
  speed: number;
  paused: boolean;
  snapshot: Snapshot | null;
}

interface AlertState {
  alerts: Alert[];                   // capped at 1000
  unacknowledged: number[];          // alert IDs
  activeFilter: AlertCategory | 'all';
}

interface OverlayDataState {
  economyData: EconomyOverlayData | null;
  militaryData: MilitaryOverlayData | null;
  climateData: ClimateOverlayData | null;
  socialData: SocialOverlayData | null;
  diplomacyData: DiplomacyOverlayData | null;
  lastUpdatedTick: number;
}

interface ResearchState {
  techTree: TechNode[];
  inProgressTechs: TechProgress[];
  budget: number; // g/tick
}
```

**Update flow:**

```typescript
// WebSocket message handler
function handleTickBroadcast(params: TickBroadcastParams): void {
  const { snapshot, events } = params;

  // Batch all state updates in one Zustand transaction
  useSimStore.setState(state => ({
    tick: snapshot.header.tick,
    snapshot,
  }));

  useMapStore.setState(state => ({
    tiles: mergeTiles(state.tiles, snapshot.world.cells),
    fogArray: computeFogArray(snapshot.world.agents, state.fogArray),
  }));

  useAlertStore.setState(state => ({
    alerts: [...state.alerts, ...events.map(eventToAlert)].slice(-1000),
    unacknowledged: [...state.unacknowledged, ...events.filter(e => e.priority === 'critical').map(e => e.id)],
  }));
}
```

### 10.3 WebSocket Integration

**Connection management:**

```typescript
class CivLabWebSocketClient {
  private ws: WebSocket | null = null;
  private reconnectDelay = 1000;  // ms, doubles on each failure
  private maxReconnectDelay = 30000;
  private subscriptionId: string | null = null;

  connect(url: string): void {
    this.ws = new WebSocket(url);
    this.ws.onopen = () => { this.reconnectDelay = 1000; this.handshake(); };
    this.ws.onmessage = (e) => this.handleMessage(e);
    this.ws.onclose = () => this.scheduleReconnect();
    this.ws.onerror = (e) => console.error('WS error', e);
  }

  private scheduleReconnect(): void {
    setTimeout(() => {
      this.reconnectDelay = Math.min(this.reconnectDelay * 2, this.maxReconnectDelay);
      this.connect(this.url);
    }, this.reconnectDelay);
  }

  private async handshake(): Promise<void> {
    const result = await this.rpc('sim.handshake', {
      client_id: `web_${Date.now()}`,
      client_type: 'game',
      client_version: '1.0.0',
      desired_framerate: 60,
    });
    await this.subscribe(['entities.agents', 'entities.buildings', 'events.all', 'metrics.all']);
  }
}
```

**Binary frame support:** When `use_binary_frames: true` in subscribe, incoming WebSocket binary messages (ArrayBuffer) are decoded using the CIV-0200 frame format via a `zstd-wasm` decoder.

**Command queue:** All `sim.command` calls are queued client-side (FIFO). If WebSocket is not connected, commands are held and replayed on reconnect (up to 100 commands buffered).

### 10.4 Bundle Targets & Performance

**Bundle size targets:**

| Bundle | Target | Notes |
|--------|--------|-------|
| Initial JS (critical path) | < 500 KB gzipped | React + Zustand + Radix UI + minimal Pixi.js |
| Game assets (lazy) | On-demand | Atlases loaded as needed per zoom level |
| Pixi.js | ~250 KB gzipped | Loaded in initial bundle (needed for first frame) |
| Recharts + D3 | ~180 KB gzipped | Lazy-loaded on Research Mode entry |

**Loading targets:**

| Metric | Target |
|--------|--------|
| TTI (Time to Interactive) on 10 Mbps | < 2 seconds |
| First meaningful paint | < 1 second |
| Game canvas first render | < 500ms after TTI |
| Terrain atlas load | < 300ms on 10 Mbps |

**Lazy loading strategy:**

```typescript
// Lazy imports for Research Mode
const ResearchMode = lazy(() => import('./components/ResearchMode'));
const TechTreePanel = lazy(() => import('./components/TechTreePanel'));

// Preload overlays on mouseover (before user clicks)
function onOverlayHover(type: OverlayType): void {
  import('./overlays/' + type).then(module => {
    overlayRegistry.register(type, module.default);
  });
}
```

**Production build config:**

```typescript
// vite.config.ts
export default defineConfig({
  build: {
    rollupOptions: {
      output: {
        manualChunks: {
          'vendor-react': ['react', 'react-dom'],
          'vendor-pixi': ['pixi.js'],
          'vendor-zustand': ['zustand'],
          'vendor-radix': ['@radix-ui/react-dialog', /* ... */],
          'research-mode': ['recharts', 'd3'],
        },
      },
    },
    target: 'es2022',
    minify: 'esbuild',
  },
});
```

---

## 11. 2D to 3D Transition Readiness

### 11.1 IRenderer Abstraction

All game rendering is mediated through an `IRenderer` interface. The Pixi.js web renderer and the Bevy 2D native renderer both implement this contract. When Phase 2 transitions to 3D (Bevy 3D or Three.js), only the concrete implementation changes; game logic and HUD logic remain untouched.

**TypeScript interface (web client):**

```typescript
interface IRenderer {
  // Map rendering
  renderTile(q: number, r: number, terrain: TerrainType, overlayColor?: RGBA): void;
  clearTile(q: number, r: number): void;

  // Entity rendering
  renderUnit(entityId: number, q: number, r: number, sprite: UnitSprite, tint: RGBA): void;
  renderBuilding(entityId: number, q: number, r: number, sprite: BuildingSprite): void;
  removeEntity(entityId: number): void;

  // UI overlays (rendered above tiles, below HUD)
  renderOverlayPolygon(hexes: [number, number][], color: RGBA, alpha: number): void;
  renderOverlayArrow(from: [number, number], to: [number, number], thickness: number, color: RGBA): void;
  renderSelectionRing(entityId: number, color: RGBA, pulseHz: number): void;
  renderPathLine(path: [number, number][], color: RGBA, animated: boolean): void;

  // Camera
  getCamera(): ICamera;

  // Fog of war
  updateFog(fogArray: Uint8Array): void;

  // Lifecycle
  resize(width: number, height: number): void;
  destroy(): void;
}
```

**Bevy (Rust) equivalent trait:**

```rust
pub trait IRenderer {
    fn render_tile(&mut self, q: i32, r: i32, terrain: TerrainType, overlay: Option<Color>);
    fn clear_tile(&mut self, q: i32, r: i32);
    fn render_unit(&mut self, entity_id: u64, q: i32, r: i32, sprite: UnitSprite, tint: Color);
    fn render_building(&mut self, entity_id: u64, q: i32, r: i32, sprite: BuildingSprite);
    fn remove_entity(&mut self, entity_id: u64);
    fn render_overlay_polygon(&mut self, hexes: &[(i32, i32)], color: Color, alpha: f32);
    fn render_overlay_arrow(&mut self, from: (i32, i32), to: (i32, i32), thickness: f32, color: Color);
    fn render_selection_ring(&mut self, entity_id: u64, color: Color, pulse_hz: f32);
    fn render_path_line(&mut self, path: &[(i32, i32)], color: Color, animated: bool);
    fn get_camera(&mut self) -> &mut dyn ICamera;
    fn update_fog(&mut self, fog_array: &[u8]);
    fn resize(&mut self, width: u32, height: u32);
}
```

**Phase 2 swap:** Replace `PixiJsRenderer` with `ThreeJsRenderer` (or Bevy 3D renderer). No callers change.

### 11.2 Asset Contract

All game entity assets follow a naming convention that enables direct Phase 2 3D asset substitution:

**Naming convention:**

```
{unit_type}_{direction}_{frame}.{ext}

Where:
  unit_type: infantry | cavalry | archer | siege | worker | merchant | spy | scout
  direction: n | ne | e | se | s | sw | w | nw  (or: 0-7 for numeric)
  frame:      0-padded integer (00, 01, 02, ..., 15)
  ext:        png (Phase 1) | glb (Phase 2)

Examples (Phase 1 2D):
  infantry_e_00.png   — Infantry facing east, frame 0 (idle)
  cavalry_sw_04.png   — Cavalry facing south-west, frame 4 (walk)

Phase 2 3D equivalent:
  infantry_e_00.glb   — Same naming, 3D model (direction encoded as rotation)
  cavalry_sw_04.glb   — (or single .glb with animation clips; direction via rotation)
```

**Asset registry (TypeScript):**

```typescript
const AssetRegistry = {
  unit: (type: UnitType, dir: Direction, frame: number) =>
    `assets/units/${type}_${dir}_${String(frame).padStart(2, '0')}.png`,

  building: (type: BuildingType) =>
    `assets/buildings/${type}.png`,

  terrain: (type: TerrainType) =>
    `assets/terrain/${type}.png`,

  flag: (nationId: string) =>
    `assets/flags/${nationId}.svg`,
};
```

**Phase 2 swap:** Change `.png` extension to `.glb` in AssetRegistry. Loader switches from `PIXI.Assets` to Three.js `GLTFLoader`. No entity code changes.

### 11.3 Camera Abstraction

**ICamera interface:**

```typescript
interface ICamera {
  // Current state
  getPosition(): [number, number];    // world-space center (x, y)
  getZoom(): number;                  // scale factor
  getViewBounds(): Rect;              // visible world rectangle

  // Mutations
  panTo(x: number, y: number, durationMs?: number): void;
  zoomTo(factor: number, centerX?: number, centerY?: number, durationMs?: number): void;
  panBy(dx: number, dy: number): void;
  zoomBy(delta: number): void;

  // Coordinate transforms
  worldToScreen(x: number, y: number): [number, number];
  screenToWorld(sx: number, sy: number): [number, number];
  hexToScreen(q: number, r: number): [number, number];
  screenToHex(sx: number, sy: number): [number, number];
}
```

**Phase 1 implementation:** `PixiCamera` uses Pixi.js `stage.scale` and `stage.position` for orthographic-equivalent zoom/pan.

**Phase 2 implementation:** `ThreePerspectiveCamera` implements the same `ICamera` interface with `THREE.PerspectiveCamera`. `panTo` changes camera `position.x/z`; `zoomTo` changes `position.y` (camera height). `worldToScreen` uses `THREE.Vector3.project`.

The `hexToScreen` and `screenToHex` conversions are identical in both phases (hex math is 2D regardless of render mode).

### 11.4 Spatial Math Policy

All spatial computations within the simulation core and client use 2D vector types that have a direct 3D upgrade path:

**Web client (TypeScript):**

```typescript
// Phase 1: Vec2 only
type Vec2 = { x: number; y: number };

function hexToWorld(q: number, r: number): Vec2 {
  return {
    x: HEX_SIZE * (q + r / 2),
    y: HEX_SIZE * r * SQRT3_2,
  };
}
```

**Bevy (Rust) core:**

```rust
use glam::Vec2;

fn hex_to_world(q: i32, r: i32, hex_size: f32) -> Vec2 {
    Vec2::new(
        hex_size * (q as f32 + r as f32 / 2.0),
        hex_size * r as f32 * SQRT_3_2,
    )
}
```

**Phase 2 upgrade:** `Vec2` → `Vec3` in Bevy (add `z: 0.0` for ground plane). `glam::Vec2` → `glam::Vec3`. The function signatures change; callers update from `Vec2` to `Vec3` but the spatial logic is identical.

**Hardcoded 2D rules:**
- No `canvas.getContext('2d')` used for game entities (only for potential debug overlays)
- No CSS `perspective` or `transform-style: preserve-3d` on game canvas
- No angle calculations assuming `z = 0` embedded in game logic (use `atan2(y, x)` which works identically in 2D/3D projection)

---

## 12. FR Traceability

### 12.1 FR-CIV-RTS-* Mapping

| FR ID | Title | UI Component(s) | Notes |
|-------|-------|-----------------|-------|
| FR-CIV-RTS-001 | Unit Movement Command | Action Panel → Move button (`Q`); Path Visualization (Section 6.4); Unit command bar at Zoom 2 | Move order issued via click after `Q` hotkey; path preview shown immediately |
| FR-CIV-RTS-002 | Unit Combat & Attack Orders | Action Panel → Attack button (`W`); Combat Visualization (Section 6.5); Alert Feed (battle result) | Attack cursor mode activated on `W`; damage numbers float on hit |
| FR-CIV-RTS-003 | Unit Group & Formation Control | Multi-select via drag-box (Section 6.2); Formation Display (Section 6.3); Formation hotkeys `Shift+1-4` | Formation lines visible when 2+ units selected; formation preview at destination |
| FR-CIV-RTS-004 | Command Queuing & Auto-Execute | Action Panel → Queue Commands button; path waypoint numbers (◉1 ◉2 ◉3) | `T` hotkey toggles queue mode; queued commands shown as numbered waypoints on path |
| FR-CIV-RTS-005 | Supply Line & Logistics | Supply bar under unit sprite (Section 6.1); Military Overlay supply chain lines (Section 5.2); Alert Feed "LOW SUPPLY" event | Supply bar color-coded; `⚠ LOW SUPPLY` badge on unit at 25%; zero supply unit has red unit tint |
| FR-CIV-RTS-006 | Structure Construction & Management | Construction Queue panel in Zoom 2 left sidebar; Building sprite progress indicator (% complete shown as overlay bar) | Build command via District Actions panel; progress shown as partially-built sprite |
| FR-CIV-RTS-007 | Structure Damage & Repair | Combat Visualization: structure hit flash; building HP bar; Alert Feed "STRUCTURE DAMAGED" | Damaged buildings show HP bar (same as units); repair action in District Actions panel |
| FR-CIV-RTS-008 | Vision & Fog of War | Fog of War renderer (Section 3.5); WebGL shader; "?" stale marker on fogged-but-scouted tiles | Fog updates every tick; historical fog shows desaturated last-seen state |
| FR-CIV-RTS-009 | Diplomacy & Treaties | Relations Panel (Section 4.5); Action Panel → Declare War / Sign Treaty; Alert Feed treaty notifications | Diplomatic actions available at Zoom 1 from Relations Panel; AI response shown in Alert Feed |
| FR-CIV-RTS-010 | Siege Mechanics | Alert Feed "SIEGE BEGAN" event; Military Overlay siege ring visualization; District Panel shows siege status | Besieged city shows red pulsing ring on map and minimap; garrison health countdown in district panel |
| FR-CIV-RTS-011 | Espionage & Shadow Operations | Nation Panel → Espionage Budget section; Diplomacy Overlay shadow network edges (Section 5.5); Alert Feed espionage events | Espionage results surface as Alert Feed items; captured spy → alert "SPY EXPOSED" |
| FR-CIV-RTS-012 | Turn-Based vs Real-Time | Speed controls in Top Bar (Section 4.1); hotkeys `Space` pause, `1×`/`2×`/`5×`/`MAX` | Turn-based mode: speed set to 0; Space advances one tick per press |
| FR-CIV-RTS-013 | Unit Experience & Leveling | Unit Panel: XP bar, Level badge (Section 6.1); Level star badges on unit sprite; Alert Feed "LEVEL UP" | XP bar shown in Unit Panel; star badge visible on sprite at level 3+ |
| FR-CIV-RTS-014 | Faction AI Behavior | Alert Feed AI actions; Relations Panel faction state updates; Military Overlay enemy positions | AI actions surface as events in Alert Feed; enemy moves visible through fog via scouted positions |
| FR-CIV-RTS-015 | Client-Side Prediction & Replay Correction | Smooth unit movement interpolation (frame-rate independent); snap correction \< 100ms | Unit positions extrapolated per-frame; server authoritative state corrects if drift > 1 hex |

### 12.2 FR-CIV-GEO-* Mapping

| FR ID | Title | UI Component(s) | Notes |
|-------|-------|-----------------|-------|
| FR-CIV-GEO-001 | Terrain Types & Properties | Map Rendering (Section 3); Tile Layer Stack (Section 3.2); terrain atlas sprites | Each terrain type has distinct sprite; movement costs shown in tile tooltip |
| FR-CIV-GEO-002 | Map Generation & Biome Systems | Map viewport at Zoom 1 (biome territory blocs); Zoom 2 (biome tile textures); Climate Overlay (Section 5.3) | Biome visual distinct from nation color; temperature gradient in climate overlay |
| FR-CIV-GEO-003 | District & Region Subdivision | District Panel in Zoom 2 left sidebar; breadcrumb navigation (Nation → Region → District → Citizen); drill-down click | Zoom transition from Zoom 1 region to Zoom 2 district is the core LOD transition |
| FR-CIV-GEO-004 | Neighbor Queries & Pathfinding | Path Visualization (Section 6.4); A* path preview; hex coordinate system (Section 3.1) | Path preview generated on move command issue; displayed as animated dashed line |
| FR-CIV-GEO-005 | Resource Distribution & Renewal | Resource Bar (Section 4.2) with delta indicators; District Panel resource bars; Economy Overlay choropleth | Delta indicators show per-tick extraction vs. renewal; depletion shown as red delta |
| FR-CIV-GEO-006 | Climate Events & Modulation | Climate Overlay (Section 5.3); Alert Feed climate events; Resource Bar CO₂ counter | Drought → food delta warning in Resource Bar; climate event → alert + map visual |
| FR-CIV-GEO-007 | District Connectivity & Trade Routes | Economy Overlay trade route arrows (Section 5.1); District Panel trade routes button | Trade routes visible in Economy Overlay; thickness = volume; color = status |
| FR-CIV-GEO-008 | Population Density & Urban Growth | District Panel population density display; Social Overlay migration flows (Section 5.4) | High-density districts shown with urbanization icon; migration arrows in social overlay |
| FR-CIV-GEO-009 | Disaster Zones & Recovery | Alert Feed disaster events; Map: disaster zone red overlay polygon; District Panel recovery progress bar | Recovery progress shown as percentage in district panel; disaster zone has distinctive map shading |
| FR-CIV-GEO-010 | LOD Rendering Contract & Data Schema | Zoom Transition System (Section 7); LOD Switching Rules (Section 7.3); Pixi.js LOD container visibility | Each zoom level loads schema-appropriate data; zoom 1 gets region aggregates, zoom 2 gets district detail |

---

## Appendix A: Component Dependency Graph

```
WebSocket (CIV-0200)
    │
    ▼
Zustand Stores ──────────────────────────────────────┐
    │                                                 │
    ├── SimState ──► Top Bar, Speed Controls          │
    ├── MapState ──► Pixi.js Renderer ──────────────► │
    │                    │                            │
    │                    ├── Terrain Tiles            │
    │                    ├── Unit Sprites             │
    │                    ├── Building Sprites         │
    │                    ├── Fog of War Shader        │
    │                    └── Overlay Layers           │
    │                                                 │
    ├── SelectionState ──► Action Panel               │
    │                    ► Unit Command Bar           │
    │                    ► Formation Display          │
    │                                                 │
    ├── AlertState ──► Alert Feed                     │
    │               ► Alert Badge (Top Bar)           │
    │               ► Minimap alert dots              │
    │                                                 │
    ├── OverlayDataState ──► 5 Overlay renders        │
    │                                                 │
    └── ResearchState ──► Research Tree Panel         │
                        ► Research Mode Dashboard     │
```

---

## Appendix B: Theme Token Reference

```css
/* Dark theme (default) */
:root {
  --bg-0:          #1A1A2E;  /* Page background */
  --surface-0:     #16213E;  /* Panel backgrounds */
  --surface-1:     #0F3460;  /* Panel borders, tile grid lines */
  --text-primary:  #E8E8F0;  /* Primary text */
  --text-secondary:#8899BB;  /* Secondary labels */

  /* Okabe-Ito game palette */
  --color-action:  #E69F00;  /* Primary actions, selected units */
  --color-friendly:#009E73;  /* Allied, positive, safe */
  --color-warning: #F0E442;  /* Near-threshold warnings */
  --color-hostile: #D55E00;  /* Enemy, critical, war */
  --color-info:    #56B4E9;  /* Info, neutral highlights */
  --color-shadow:  #CC79A7;  /* Espionage, shadow networks */
  --color-research:#0072B2;  /* Tech, science, research */

  /* Semantic colors */
  --stability-high:   #009E73;
  --stability-mid:    #E69F00;
  --stability-low:    #D55E00;
  --hp-full:          #009E73;
  --hp-mid:           #E69F00;
  --hp-critical:      #D55E00;
}

/* Light theme (research mode) */
[data-theme="light"] {
  --bg-0:          #FAFAFA;
  --surface-0:     #FFFFFF;
  --surface-1:     #E8E8E8;
  --text-primary:  #1A1A2E;
  --text-secondary:#666688;
}
```

---

## Appendix C: Animation Timing Reference

| Animation | Duration | Easing | Notes |
|-----------|----------|--------|-------|
| Zoom 1 → Zoom 2 transition | 500ms | ease-in-out-cubic | Camera + LOD crossfade |
| Zoom 2 → Zoom 3 transition | 300ms | ease-in-out-cubic | Citizen card slide-in |
| Panel open/close | 150ms | ease-out | Sidebar translate |
| Alert item entry | 200ms | ease-out | Fade + slide-down |
| Unit selection ring pulse | 1000ms | sine | Continuous, 1Hz |
| Damage number float | 800ms | ease-in | Upward + fade |
| Death animation | 300ms total | linear | Scale-up 100ms, scale-down 200ms |
| Speed control switch | 100ms | linear | Immediate feel |
| Overlay toggle | 300ms | ease-out | Fade in/out |
| Hotkey feedback flash | 80ms | linear | Brief button highlight |
| Chart data update | 200ms | ease | Recharts internal animation |

---

## Appendix D: Accessibility Checklist

- [ ] All interactive elements have visible focus indicators (2px `#E69F00` outline)
- [ ] Focus trap implemented in all modal dialogs (Radix UI `Dialog` handles this)
- [ ] Keyboard navigation: Tab order matches visual reading order (left-to-right, top-to-bottom)
- [ ] Alert Feed announces new alerts via `aria-live="polite"` (critical: `assertive`)
- [ ] All icon-only buttons have `aria-label` (e.g., `aria-label="Pause simulation"`)
- [ ] Color is never the sole information carrier: shapes, labels, patterns supplement
- [ ] Fog of war dark overlay maintains > 4.5:1 contrast for visible tile text
- [ ] Speed control buttons: `role="radio"` with `aria-checked`, grouped with `aria-label="Simulation speed"`
- [ ] Chart tooltips accessible via keyboard (arrow keys navigate data points in Recharts)
- [ ] `prefers-reduced-motion`: transitions reduced to instant cuts; particle effects disabled
- [ ] Font sizes: minimum 12px; all user-adjustable via browser zoom without layout breaks
- [ ] Overlay toggle buttons: `aria-pressed` state reflects active/inactive

---

## Appendix E: Component Acceptance Criteria

Every UI component listed in this specification has explicit acceptance criteria. These criteria form the test plan for the web client implementation sprint.

### E.1 Top Bar

- [ ] Tick counter increments every tick broadcast received
- [ ] Speed button active state matches `simState.speed` at all times
- [ ] Pressing `Space` toggles pause; button visual state updates within 1 frame
- [ ] Alert badge shows correct count of unacknowledged alerts (0 suppressed)
- [ ] When alert count > 99, badge shows "99+" not truncated number
- [ ] Year/Quarter display: tick 0 = Year 1, Quarter 1; tick 25 = Year 2, Quarter 1
- [ ] Top bar visible at all zoom levels and all panel states
- [ ] Tab-focusable in correct order: flag/nation → tick display → speed controls → pause → menu

### E.2 Resource Bar

- [ ] All 5 primary resources displayed: Joules, Food, Materials, Treasury, Population
- [ ] CO₂ ppm displayed with color threshold (> 450ppm = orange, > 550ppm = red)
- [ ] Delta indicators use rolling 5-tick average (not instantaneous)
- [ ] Warning badge appears for: negative Joules balance, food delta < -5%/tick, CO₂ > 450ppm, treasury \< 0
- [ ] Joules display uses SI prefix scaling: TJ, PJ (not raw integer)
- [ ] Counter animation: when value changes, digit rolls from old to new (100ms)

### E.3 Map Viewport

- [ ] Hex tiles render at correct axial coordinates (q, r) matching server snapshot
- [ ] Terrain type visually distinct for all 7 terrain types (plains, forest, hill, mountain, water, city, shallow_water)
- [ ] Zoom wheel: zoom-to-cursor behavior (world point under cursor stays fixed)
- [ ] Pan inertia: velocity decays to 0 within 500ms of mouse release
- [ ] Arrow key pan: accelerates after 10 frames; stops immediately on key release
- [ ] Multi-select drag box: visible as `#56B4E9` rectangle; selects all units within on release
- [ ] Right-click on map (unit selected): issues move command, shows path preview
- [ ] Double-click on unit type: selects all visible units of same type
- [ ] Tile hover: tooltip shows terrain type, resource stocks, current occupants

### E.4 Unit Rendering

- [ ] Unit sprites load from `units_atlas_01.png` / `units_atlas_02.png`
- [ ] Faction tint applied via PIXI `tint` property (not separate sprite)
- [ ] Health bar renders below unit: green > 60%, yellow 30-60%, red \< 30%
- [ ] Supply bar renders below health bar (military units only)
- [ ] Level badge: star icon in sprite corner for units level >= 3
- [ ] Selected unit: pulsing `#E69F00` ring at 1Hz
- [ ] 10,000 simultaneous unit sprites render at >= 60fps on target hardware
- [ ] Unit death: scale-up → collapse → particle burst sequence completes within 300ms

### E.5 Fog of War

- [ ] Unseen hexes: opaque black overlay (fogAlpha = 1.0)
- [ ] Scouted-but-not-visible hexes: desaturated + 70% dim (fogAlpha = 0.5)
- [ ] Visible hexes: no overlay (fogAlpha = 0.0)
- [ ] Fog updates each tick within the same animation frame as snapshot render
- [ ] Fog texture update cost \< 5ms for 50,000 hex map
- [ ] Historical fog shows "?" stale marker on units last seen > 10 ticks ago

### E.6 Overlays

- [ ] Economy overlay: choropleth recomputes on overlay toggle (not on every tick)
- [ ] Trade route arrows: visible when economy overlay active; thickness proportional to volume
- [ ] Military overlay: supply line color matches supply percentage thresholds
- [ ] Climate overlay: CO₂ bubble radius proportional to district emissions
- [ ] Social overlay: ideology colors use Okabe-Ito palette, not red/green only
- [ ] Diplomacy overlay: alliance bands use distinct non-conflicting colors
- [ ] Multiple overlays can be simultaneously active (visual blend is legible)
- [ ] Overlay toggle animation: 300ms fade in/out; no layout shift

### E.7 Alert Feed

- [ ] New alerts appear at top of feed within 1 frame of receiving tick broadcast
- [ ] Filter dropdown: selecting a category shows only matching alert types
- [ ] `[→ Map]` link: camera smoothly pans to alert location (400ms transition)
- [ ] Actionable alerts (treaty offers): Accept/Reject buttons issue `sim.command`
- [ ] Feed virtualized: > 500 alerts in feed has no visible scroll performance degradation
- [ ] Critical alerts (stability \< 20) use `aria-live="assertive"` announcement
- [ ] Tab key cycles through critical alerts (panning camera to each)

### E.8 Research Tree

- [ ] Tech nodes rendered as D3.js DAG; no edges cross (or minimize crossings)
- [ ] Completed techs: filled green background; locked techs: greyed out + lock icon
- [ ] In-progress tech: partial fill progress bar showing percentage
- [ ] Click tech: details panel shows cost, prerequisites, what it unlocks
- [ ] Research budget slider: changing budget issues `sim.command` type `policy_set`
- [ ] Tech tree zoom/pan: independent from map zoom (D3 zoom behavior on SVG)

### E.9 Zoom Transitions

- [ ] Zoom 1 → Zoom 2: strategic region blocs fade out by zoom factor 3.0; hex tiles fade in
- [ ] Zoom 2 → Zoom 3: requires citizen entity selected; transitions in 300ms
- [ ] Zoom transition runs at >= 60fps (no jank)
- [ ] LOD layers cross-fade smoothly (no pop-in)
- [ ] Escape key at Zoom 3 zooms back to Zoom 2 in reverse transition

### E.10 Citizen Profile

- [ ] All fields populated from `sim.query` response (not snapshot broadcast)
- [ ] Happiness breakdown shows all contributing factors with sign and magnitude
- [ ] Social graph renders with D3.js force layout; nodes draggable for exploration
- [ ] Sparkline shows 50-tick history; loads historical data from `sim.query` on open
- [ ] Export Profile button: downloads JSON with all citizen fields
- [ ] Family tree links: clicking parent/child citizen navigates to their profile

---

## Appendix F: Error States & Edge Cases

The UI must handle the following error conditions gracefully. In all cases, the error is shown to the player via the Alert Feed (not a blocking modal). The simulation does not stop.

### F.1 WebSocket Disconnection

**Display:**

```
┌─────────────────────────────────────────────────────────────────────────┐
│ ⚠ CONNECTION LOST — Attempting to reconnect (retry 2/10)...             │
│ Last received: Tick 1247  Delay: 3.2s  [Retry Now]  [Work Offline]     │
└─────────────────────────────────────────────────────────────────────────┘
```

- Reconnection banner appears at top of viewport (below top bar), pushing content down 36px
- Simulation display freezes at last received tick (not cleared)
- Reconnect attempts with exponential backoff: 1s → 2s → 4s → 8s → 16s → 30s (max)
- `[Retry Now]` forces immediate reconnect attempt
- `[Work Offline]` dismisses banner; player can inspect last snapshot but not issue commands
- On reconnect: fast-forward display to current server tick using snapshot difference

### F.2 Server-Rejected Command

**Display:**

```
ACTION REJECTED: Insufficient resources
  Required: 500g  Available: 45g
  [View Treasury]  [Dismiss]
```

- Toast notification appears bottom-right, auto-dismisses after 5s
- Action button that was clicked shows red flash (80ms)
- No state change on map (predictive state rolls back immediately)

### F.3 Schema Version Mismatch

If `lod_snapshots.schema_version` from server differs from client's expected version:

```
┌─────────────────────────────────────────────────────────────────────────┐
│ SERVER VERSION MISMATCH                                                  │
│ Client expects schema v1; Server sent schema v2.                         │
│ [Reload Page]  [Continue (may show incorrect data)]                     │
└─────────────────────────────────────────────────────────────────────────┘
```

- Hard reload is the recommended action (re-downloads client from server)
- "Continue" mode: render what can be parsed; log unknown fields to console

### F.4 Extremely High Alert Volume

If > 50 critical alerts arrive in a single tick (e.g., mass collapse scenario):

- Alert Feed shows: "50+ critical events this tick — [View Summary]"
- Summary modal groups by category with counts
- Individual events still in feed but oldest ones paginated
- `aria-live` uses debounce: one announcement per 2-second window max (prevents screen reader spam)

### F.5 Low-Performance Device

When frame time > 22ms (< 45fps) for 5 consecutive seconds:

- Adaptive quality system activates (see Section 7.4)
- Notification toast: "Performance mode: reduced visual effects"
- Player can manually override: Settings → Performance → [High / Medium / Low / Auto]
- Performance mode persists in `localStorage` across sessions

### F.6 Very Large Maps (> 100,000 hexes)

- Terrain tiles use chunked loading: 32×32 hex chunks loaded on demand as viewport pans
- Each chunk is one `PIXI.RenderTexture` (cached offscreen render)
- Chunk LRU cache: keep 25 most recent chunks in GPU memory; evict LRU
- Minimap always uses pre-rendered low-resolution texture (not live Pixi render) for > 100K hex maps

---

## Appendix G: Modding & Custom Theme API

CivLab's UI is designed to be skinnable by modders. The following extension points are available:

### G.1 Custom Themes

Modders can override the CSS custom property theme by providing a `theme.css` file in their mod bundle. The file is injected after the base theme, allowing selective overrides.

**Example mod theme:**

```css
/* my-mod/theme.css */
:root {
  --bg-0: #0D1117;          /* GitHub dark background */
  --surface-0: #161B22;
  --color-action: #58A6FF;  /* GitHub blue instead of amber */
  --color-friendly: #3FB950; /* GitHub green */
}
```

**How it loads:**

```typescript
// Mod loader injects theme after base stylesheet
function loadModTheme(modId: string, themeCssContent: string): void {
  const styleEl = document.createElement('style');
  styleEl.id = `mod-theme-${modId}`;
  styleEl.setAttribute('data-mod', modId);
  styleEl.textContent = themeCssContent;
  document.head.appendChild(styleEl);
}
```

### G.2 Custom HUD Panels

Mods can register additional HUD panels that appear as new tabs in the right sidebar:

```typescript
// mod API (exposed as window.CivLabMod in browser context)
interface ModPanelDescriptor {
  id: string;
  label: string;
  icon: string;              // SVG string (24×24px)
  component: React.ComponentType<{ snapshot: Snapshot }>;
  zoom_levels: (1 | 2 | 3)[];   // which zoom levels show this panel
}

window.CivLabMod.registerPanel({
  id: 'my-mod-economics',
  label: 'Advanced Economics',
  icon: '<svg>...</svg>',
  component: MyEconomicsPanel,
  zoom_levels: [1, 2],
});
```

### G.3 Custom Overlay Layers

Mods can add new named overlay layers (beyond the 5 built-in overlays):

```typescript
interface ModOverlayDescriptor {
  id: string;
  label: string;
  hotkey?: string;           // additional F-key or combo
  render: (
    graphics: PIXI.Graphics,
    snapshot: Snapshot,
    camera: ICamera
  ) => void;
}

window.CivLabMod.registerOverlay({
  id: 'my-mod-trade-flow',
  label: 'Advanced Trade Flow',
  render: (graphics, snapshot, camera) => {
    // Custom overlay drawing using PIXI.Graphics
  },
});
```

### G.4 Custom Unit Sprite Packs

Mods can replace unit sprites by providing an alternate atlas at the same filename convention:

```
my-mod/assets/units/
  infantry_e_00.png    ← replaces default infantry east idle frame 0
  cavalry_nw_02.png    ← replaces cavalry north-west walk frame 2
  ...
```

The mod loader registers alternate atlases at higher priority than the base atlas. The `AssetRegistry` checks mod atlases first before falling back to base.

### G.5 Custom Scenario Data Visualization

Research Mode panels expose a data API for mods to add custom charts:

```typescript
interface ModChartDescriptor {
  id: string;
  label: string;
  data_source: (snapshot: Snapshot) => number;  // extract scalar per tick
  chart_type: 'line' | 'bar' | 'scatter';
  y_label: string;
  color: string;
}

window.CivLabMod.registerResearchChart({
  id: 'my-mod-corruption-index',
  label: 'Corruption Index',
  data_source: (snap) => snap.metrics.corruption_index ?? 0,
  chart_type: 'line',
  y_label: 'Corruption (0-100)',
  color: '#CC79A7',
});
```

---

## Appendix H: Screen Reader & Keyboard Navigation Flow

This appendix documents the keyboard-only navigation path through the full UI, ensuring WCAG 2.1 AA compliance.

### H.1 Tab Order (Top Level)

```
1. Nation flag/name (landmark: banner)
2. Tick counter (informational, not interactive)
3. Speed controls group (role=radiogroup)
   3a. 0.5× button
   3b. 1× button
   3c. 2× button
   3d. 5× button
   3e. MAX button
4. Pause/Resume button
5. Main menu button
6. Resource bar items (informational; Tab moves to next item)
7. Left sidebar (landmark: complementary, aria-label="Strategic Overlays")
   7a. Overlay toggle buttons (F1-F5 shortcuts labeled)
   7b. Alert list items
   7c. Quick action buttons
8. Map viewport canvas (landmark: main, aria-label="Game Map")
   8a. Receives arrow key pan; + /- zoom
   8b. Enter/Space on focused tile: select entity at cursor
9. Right sidebar (landmark: complementary, aria-label="Nation Panel")
   9a. Nation stats (informational)
   9b. Relations list items
   9c. Action buttons
10. Bottom bar (landmark: contentinfo)
    10a. Resource counters
    10b. Timeline scrubber
    10c. Export/replay controls
```

### H.2 Map Keyboard Navigation

When focus is on the map canvas:

| Key | Action |
|-----|--------|
| Arrow keys | Pan map (32px/frame, accelerates) |
| `+` / `=` | Zoom in |
| `-` | Zoom out |
| `Enter` / `Space` | Select entity at current cursor hex |
| `Escape` | Deselect / zoom out level |
| `Tab` | Move focus to unit panel (when unit selected) |
| `Shift+Tab` | Move focus back to sidebar |

### H.3 Panel Keyboard Navigation

Within the Action Panel when unit is selected:

| Key | Action |
|-----|--------|
| `Q` | Move command (same as Move button) |
| `W` | Attack command |
| `E` | Fortify |
| `R` | Rally |
| Arrow keys | Move button focus between action buttons |
| `Enter` | Activate focused button |
| `Escape` | Deselect unit, return focus to map |

### H.4 Alert Feed Keyboard Navigation

| Key | Action |
|-----|--------|
| `Tab` (into feed) | Focus first alert item |
| Arrow keys | Move between alert items |
| `Enter` | Activate alert action ([→ Map], [Accept]) |
| `Escape` | Return focus to map |
| `N` | Jump to next unread alert (without focusing feed) |
| `Tab` (while on critical alert) | Jump to next critical alert on map |

### H.5 Screen Reader Landmark Regions

```html
<header role="banner" aria-label="CivLab Top Bar">
  <!-- Top bar content -->
</header>

<aside role="complementary" aria-label="Strategic Overlays and Alerts">
  <!-- Left sidebar -->
</aside>

<main role="main" aria-label="Game Map Viewport">
  <canvas id="game-canvas" aria-label="CivLab game world. Use arrow keys to pan, + and - to zoom.">
  </canvas>
</main>

<aside role="complementary" aria-label="Nation Stats and Actions">
  <!-- Right sidebar -->
</aside>

<footer role="contentinfo" aria-label="Resources and Timeline">
  <!-- Bottom bar -->
</footer>

<!-- Alert feed is an ARIA live region -->
<section aria-live="polite" aria-label="Event Log">
  <!-- Alert items -->
</section>
```

---

## Appendix I: Bevy 2D Client Specification

The secondary renderer target is **Bevy 2D** (Rust). This appendix specifies the Bevy-specific implementation notes, system schedule, and protocol integration that parallel the Pixi.js web client.

### I.1 Bevy Plugin Architecture

The CivLab Bevy client is organized as a set of Bevy plugins, each responsible for a domain:

```rust
pub fn build_civlab_app() -> App {
    let mut app = App::new();
    app
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "CivLab".to_string(),
                resolution: (1920.0, 1080.0).into(),
                ..default()
            }),
            ..default()
        }))
        // CivLab domain plugins
        .add_plugins(CivLabNetworkPlugin { server_url: "ws://localhost:9876".to_string() })
        .add_plugins(CivLabMapPlugin)
        .add_plugins(CivLabUnitPlugin)
        .add_plugins(CivLabHUDPlugin)
        .add_plugins(CivLabOverlayPlugin)
        .add_plugins(CivLabInputPlugin)
        .add_plugins(CivLabCameraPlugin);
    app
}
```

### I.2 Bevy System Schedule

Systems run in the following Bevy schedule stages per frame:

```
PreUpdate:
  network_receive_system   — Poll WebSocket; push tick_broadcasts to event queue
  input_system             — Keyboard/mouse input → game commands

Update:
  snapshot_apply_system    — Apply snapshot diffs to ECS entities
  fog_update_system        — Recompute fog array from unit vision
  overlay_compute_system   — Compute overlay data (economy choropleth, etc.)
  alert_process_system     — Parse events → alerts in alert store resource
  camera_system            — Camera pan/zoom/transitions

PostUpdate:
  sprite_sync_system       — Sync ECS Transform from game position
  hud_update_system        — Update HUD text, bars, counters
  command_flush_system     — Send queued sim.command payloads via WebSocket
```

### I.3 ECS Components for Units

```rust
#[derive(Component)]
pub struct CivUnit {
    pub entity_id: u64,
    pub unit_type: UnitType,
    pub faction_id: String,
    pub hp: f32,         // 0.0 – 1.0
    pub supply: f32,     // 0.0 – 1.0
    pub morale: f32,     // 0.0 – 1.0
    pub level: u8,
    pub experience: u32,
}

#[derive(Component)]
pub struct CivHexPosition {
    pub q: i32,
    pub r: i32,
}

#[derive(Component)]
pub struct Selected;   // Tag component; attached/removed by selection system

#[derive(Component)]
pub struct Routing;    // Tag: unit is currently routing (flee behavior)
```

### I.4 HUD via Bevy UI

The Bevy client uses `bevy_ui` for HUD elements (not HTML). Layout mirrors the wireframes in Section 2 using Bevy's flexbox-equivalent node system.

```rust
fn spawn_top_bar(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Px(36.0),
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                ..default()
            },
            background_color: BackgroundColor(Color::srgb(0.086, 0.129, 0.243)), // --surface-0
            ..default()
        })
        .with_children(|parent| {
            // Nation name text
            parent.spawn((
                TextBundle::from_section(
                    "Nation Name",
                    TextStyle {
                        font: asset_server.load("fonts/NotoSans-Regular.ttf"),
                        font_size: 14.0,
                        color: Color::srgb(0.91, 0.91, 0.94),
                    },
                ),
                TopBarNationName,
            ));
            // ... speed controls, tick counter, pause button
        });
}
```

### I.5 Bevy Input Mapping

```rust
fn input_system(
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut commands: EventWriter<GameCommand>,
    selection: Res<SelectionState>,
) {
    // Pause toggle
    if keys.just_pressed(KeyCode::Space) {
        commands.send(GameCommand::TogglePause);
    }

    // Overlay toggles
    if keys.just_pressed(KeyCode::F1) { commands.send(GameCommand::ToggleOverlay(OverlayType::Economy)); }
    if keys.just_pressed(KeyCode::F2) { commands.send(GameCommand::ToggleOverlay(OverlayType::Military)); }

    // Unit commands (when units selected)
    if !selection.selected_units.is_empty() {
        if keys.just_pressed(KeyCode::KeyQ) { commands.send(GameCommand::BeginMoveOrder); }
        if keys.just_pressed(KeyCode::KeyW) { commands.send(GameCommand::BeginAttackOrder); }
        if keys.just_pressed(KeyCode::KeyE) { commands.send(GameCommand::FortifySelected); }
        if keys.just_pressed(KeyCode::KeyR) { commands.send(GameCommand::RallySelected); }
    }

    // Escape: deselect > close panel > zoom out
    if keys.just_pressed(KeyCode::Escape) {
        commands.send(GameCommand::EscapePressed);
    }
}
```

---

*End of CIV-0300 UI/UX Specification*

**Document Status:** SPECIFICATION (Draft 1.0)
**Next Review:** After CIV-0301 (Asset Production Spec) and CIV-0302 (Web Client Implementation Plan)
**Related Documents:** CIV-0200 (Client Protocol), CIV-0101 (Two-Zoom LOD), CIV-0105 (War & Diplomacy)
