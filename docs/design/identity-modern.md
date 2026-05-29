# DINOForge — Warfare Modern: Brand Identity

> **Mod**: `warfare-modern`
> **Theme**: Western Alliance vs Classic Enemy — Cold-War-to-Near-Future ground conflict
> **Design Language**: Tactical HUD · Military Stencil · NATO-vs-OPFOR · Classified Briefing

---

## 1. Logo & Wordmark

### Concept

The primary mark pairs a stencilled wordmark with an iconographic emblem. All letterforms are
uppercase, tracking +200, with deliberate spray-bleed or cut-mask artifacts at stroke terminals
to evoke physical stencil manufacture. The emblem draws from open military heraldry: a compass
rose (NATO-style), crossed optics crosshairs, or a silhouetted tactical helmet — never copyrighted
insignia, only generic military grammar.

---

### Option A — "IRONLINE" (Preferred)

**Wordmark**: `WARFARE: MODERN` set in **Bebas Neue** (SIL OFL, Google Fonts), weight Bold,
tracking +220. The colon and word `MODERN` are 60% of the primary height, right-aligned flush.
A 2px rule sits between the two words, extending 40px past the right edge — a brevity line from
military briefing formats.

**Emblem**: Octagonal kill-zone crosshair ring (8-spoke, inner circle 40% of outer) cut from a
solid rectangle. The ring is hollow (knockout). Top arc reads `ALLIANCE` in 7pt stencil caps;
bottom arc reads `THEATER 1` (fictional, non-infringing). Ring colour: Western Alliance Navy
`#1C2B3A`. Background fill: Olive Drab `#4A5240`.

**Lockup**: Emblem left, wordmark right. Minimum size: 220 x 48 px. Monochrome version: full
knockout white on `#1C2B3A`.

---

### Option B — "STAMPED" (Impact Alternate)

**Wordmark**: `WARFARE · MODERN` set in **Oswald** (SIL OFL, Google Fonts) ExtraBold,
all-caps, tracking +150. Each glyph carries a visible ink-bleed drop shadow at 1px offset
(`#000000` 40% opacity) — simulating rubber-stamp ink spread.

**Emblem**: Dog-tag silhouette (rounded-corner rectangle, 2:1 aspect, notch bottom-center).
Inside: a bold `Omega` shape (or NATO-phonetic initials `WM`) at 50% emblem height. Below
the emblem, a chain of 3 flat ellipses rendered as a necklace chain.

**Lockup**: Stacked (emblem top, wordmark bottom). Minimum size: 180 x 120 px.

---

### Option C — "REDACTED" (Intel Dossier)

**Wordmark**: `[WARFARE: MODERN]` — literal square-bracket glyphs flank the text, referencing
declassified document formatting. Font: **Share Tech Mono** (SIL OFL, Google Fonts), tracking
+80, uppercase only.

**Emblem**: A rectangle with 3 horizontal "redaction bars" — solid `#0A0A0A` bars of varying
width (70%, 55%, 85%) left-aligned, evoking a blacked-out intelligence report. The top bar
reveals the Western Alliance faction icon (see Section 6) as a negative-space cutout.

**Lockup**: Horizontal; emblem left at 0.6x wordmark height. Ideal for loading screen header.

---

## 2. Color System

### 2.1 Western Alliance Palette

| Role | Name | Hex | Usage |
|---|---|---|---|
| Primary Surface | Olive Drab | `#4A5240` | Panel fills, unit card backgrounds |
| Secondary Surface | NATO Navy | `#1C2B3A` | Header bars, deep UI chrome |
| Tertiary Surface | Desert Tan | `#C9A96E` | Map overlays, terrain fills |
| Accent — Primary | Hi-Vis Amber | `#F5A623` | Selected unit outlines, button hover |
| Accent — Secondary | Comms Green | `#39D353` | Status indicators, health-full |
| Danger | Casualty Red | `#CC2929` | Health-critical, enemy ping |
| Text Primary | Buff White | `#EDE8DC` | Body text on dark surfaces |
| Text Secondary | Faded Field | `#A0A08A` | Timestamps, secondary labels |
| UI Chrome | Camo Edge | `#2E3528` | Panel borders, dividers |

### 2.2 Classic Enemy Palette

| Role | Name | Hex | Usage |
|---|---|---|---|
| Primary Surface | OPFOR Red | `#8B1A1A` | Enemy panel fills, alert overlays |
| Secondary Surface | Gunmetal | `#3D4145` | Enemy header bars, dark UI |
| Tertiary Surface | Woodland Green | `#3A4A2E` | Enemy terrain, camouflage fills |
| Accent — Primary | Soviet Crimson | `#D42B2B` | Enemy selection ring, hover state |
| Accent — Secondary | Steel Ice | `#6AACCC` | Enemy comms, radar blips |
| Danger | Black Smoke | `#1A1A1A` | Critical enemy state overlay |
| Text Primary | Cold Paper | `#E0DDD5` | Enemy-side body text |
| Text Secondary | Rusted Field | `#887060` | Enemy secondary labels |
| UI Chrome | Blast Edge | `#252829` | Enemy panel borders |

### 2.3 Neutral / Shared UI Chrome

| Role | Hex | Notes |
|---|---|---|
| UI Background (global) | `#141A18` | Near-black base behind all panels |
| Panel Translucency | `#1C2318` @ 88% | Glassmorphism for floating HUD panels |
| Minimap Border | `#2E3528` | 1px solid frame |
| Tooltip Background | `#0E1410` @ 95% | |
| Selection Cursor | `#F5A623` | 2px animated dash-ring (Alliance) |
| Selection Cursor (enemy) | `#D42B2B` | 2px animated dash-ring (OPFOR) |
| Damage Flash | `#FF4422` @ 60% | Full-screen vignette on player damage |

---

## 3. Typography

All fonts are freely licensable under SIL Open Font License (OFL) and available on Google Fonts.

### 3.1 Font Stack

| Role | Font | Weight | Source |
|---|---|---|---|
| Display / Logo | **Bebas Neue** | Regular (400) | fonts.google.com/specimen/Bebas+Neue |
| UI Headings | **Oswald** | SemiBold (600), Bold (700) | fonts.google.com/specimen/Oswald |
| Body / Tooltips | **Barlow Condensed** | Regular (400), Medium (500) | fonts.google.com/specimen/Barlow+Condensed |
| Monospace / HUD data | **Share Tech Mono** | Regular (400) | fonts.google.com/specimen/Share+Tech+Mono |
| Mission Briefing copy | **Special Elite** | Regular (400) | fonts.google.com/specimen/Special+Elite |

### 3.2 Type Scale (HUD context — 1080p base)

| Level | Font | Size | Color | Usage |
|---|---|---|---|---|
| H1 Loading Headline | Bebas Neue | 72pt | `#EDE8DC` | Loading screen title |
| H2 Mission Title | Oswald Bold | 36pt | `#F5A623` | Mission briefing header |
| H3 Panel Title | Oswald SemiBold | 20pt | `#EDE8DC` | Unit card section headers |
| Body | Barlow Condensed Medium | 14pt | `#A0A08A` | Tooltip text, descriptions |
| HUD Numerals | Share Tech Mono | 18pt | `#39D353` / `#CC2929` | Resource counters, cooldown timers |
| Briefing Flavor | Special Elite | 13pt | `#C9A96E` | Loading tips, lore text |
| Fine Print | Barlow Condensed Regular | 11pt | `#606050` | Copyright, version strings |

### 3.3 Stencil Treatment Rules

- All text at H1-H2 scale uses a 2px drop shadow in `#000000` at 50% opacity, offset (1, 2).
- Heading glyphs may display a "spray bleed" texture overlay at 8% opacity using a noise mask — optional art-agent post-process.
- Never use anti-aliasing suppression in code; simulate stencil feel via texture, not pixelation.

---

## 4. Loading Screen

### 4.1 Layout Concept — "SATELLITE PASS"

**Background layer**: A desaturated satellite-imagery texture (procedurally generated or CC0
aerial photograph composite) at `#141A18` tint, 70% overlay. Grid lines at 5% opacity — 1px
`#EDE8DC` lines every 128px horizontal, every 96px vertical — evoking a tactical map overlay.

**Central panel** (860 x 480 px, centered): Semi-transparent `#1C2318` @ 92% with 1px border
`#4A5240`. Inside, three zones:

1. **Top strip** (860 x 56 px): Red `#8B1A1A` fill left half, Navy `#1C2B3A` right half, a
   centered 1px vertical divider. Left side: "WESTERN ALLIANCE" in Oswald Bold 14pt `#EDE8DC`;
   right side: "CLASSIC ENEMY" same treatment. Faction emblems (32 x 32) flank the label.

2. **Center zone** (860 x 360 px): Mission map thumbnail (256 x 256 px) left-anchored with
   6px olive-drab border. Right 560 px: mission designation in Oswald Bold 32pt `#F5A623`,
   below it a 200pt Special Elite `#C9A96E` mission briefing paragraph (2-3 sentences lore).
   Below that: a Barlow Condensed 13pt "CLASSIFICATION: [REDACTED]" line with a yellow warning glyph.

3. **Bottom strip** (860 x 64 px): Progress bar (full width, 6px height, rounded) filled
   `#39D353` to empty `#2E3528`, animated. Below bar: Share Tech Mono 12pt loading status text
   in `#A0A08A`: e.g. `LOADING THEATER ASSETS... 64%`. Right side: pulsing dot in `#F5A623`.

**Tip strip** (full width, pinned bottom, 48 px tall): `#0E1410` @ 98%. Text:
`"TACTICAL TIP >> [tip text]"` in Barlow Condensed 14pt `#C9A96E`. Tips are styled as
decoded field messages; example: `"TACTICAL TIP >> Anti-armor units reduce armored column advance
speed by 35% — deploy forward of choke points."` Cycle on load progress milestones.

### 4.2 Loading Tips Voice / Style Guide

- 1-2 sentences; active-voice military brevity.
- Reference in-game mechanics without being a tutorial.
- Pattern: `[Unit type] [action verb] [effect] — [strategic implication].`
- No exclamation marks; no second-person "you".
- Use unit/faction names from the pack (not real-world unit names).

---

## 5. Menu Skin — Tactical HUD Panels

### 5.1 Design Language

All menu panels are **angular, chamfered** (45-degree corner cut 12 px) rather than rounded. This
military-industrial aesthetic appears in both the main menu and in-game overlay menus.

Panel anatomy:
- **Top-left chamfer cut**: 12 x 12 px notch, always.
- **Bottom-right chamfer cut**: 12 x 12 px notch, always.
- **Left accent strip**: 3px solid `#F5A623` (Alliance) or `#D42B2B` (Enemy) — color
  contextual to active faction.
- **Inner padding**: 16 px all sides.
- **Header rule**: 1px `#4A5240` at 24 px from top inner edge, full panel width.

### 5.2 Button System

| State | Fill | Border | Text |
|---|---|---|---|
| Default | `#1C2B3A` | `#4A5240` 1px | `#EDE8DC` Oswald SemiBold 14pt |
| Hover | `#243347` | `#F5A623` 1px | `#F5A623` |
| Active/Pressed | `#0D1720` | `#F5A623` 2px | `#FFFFFF` |
| Disabled | `#141A18` | `#2E3528` 1px | `#606050` |
| Danger (delete/abort) | `#2A1010` | `#CC2929` 1px | `#CC2929` |

Buttons have a 4px left-pad `>>` glyph (Share Tech Mono) on hover — slides in 80ms linear.

### 5.3 Ammo-Counter Frame (Resource Display)

A recurring motif in the HUD is the "ammo-counter frame": a rectangular frame with:
- Top label: resource name in Oswald Bold 10pt `#A0A08A` all-caps.
- Main numeral: Share Tech Mono 28pt `#EDE8DC` (green `#39D353` if >50%, red `#CC2929` if <20%).
- Bottom rule: 2px progress bar using resource fill color.
- Left accent: 2px `#F5A623` strip.
- Outer frame: 1px `#2E3528`, chamfered 6px top-left and bottom-right.

Used for: Gold, Population, Mana (retextured as "Supply"), and wave countdown.

### 5.4 Main Menu Layout (ASCII wireframe)

    +-----------------------------------------------------------+
    |  [SATELLITE BG]  ########## WARFARE: MODERN               |
    |                  ## LOGO ## [classified stamp]             |
    |                  ##########                                |
    |  +------------------------------+                          |
    |  | >> START CAMPAIGN            | <- chamfered panel       |
    |  | >> CUSTOM BATTLE             |                          |
    |  | >> LOAD SAVE                 |                          |
    |  | >> OPTIONS                   |                          |
    |  | >> EXIT                      |                          |
    |  +------------------------------+                          |
    |  [ALLIANCE EMBLEM]         [ENEMY EMBLEM]                  |
    |  WESTERN ALLIANCE          CLASSIC ENEMY                   |
    |  ---------------------------------------------------------  |
    |  v1.0.0 | DINOFORGE WARFARE-MODERN                         |
    +-----------------------------------------------------------+

Background: SATELLITE PASS texture, full bleed. Menu panel: 360 x 320 px, center-right.
Logo: top-center, 480 x 96 px. Faction emblems: bottom strip, 64 x 64 px each.

---

## 6. Iconography & Motifs

### 6.1 Recurring Visual Motifs

| Motif | Description | Uses |
|---|---|---|
| **Chevron strip** | 3 right-pointing chevrons `>>>`, angled 20 degrees, 2px stroke, 8px gap | Unit speed indicators, button prefix, tab dividers |
| **Crosshair ring** | Thin-ring circle with 4 cardinal ticks, center dot. Scales 16-64 px. | Selection cursor, targeting overlay, map ping |
| **Dog-tag shape** | Rounded-rect 2:1 aspect, notch bottom 10%. 1px border. | Unit portrait frame for infantry |
| **Hex armor plate** | Regular hexagon, flat-top, used as tile/badge background | Building/vehicle portrait frame |
| **NATO compass rose** | Simplified 8-point star, no text, pure geometric | Faction emblem base for Western Alliance |
| **Redaction bar** | Solid black rectangle 70-90% width, 8px tall | Flavor text decorators, dossier style |
| **Signal wave** | 3 concentric arc segments (radio symbol), 2px stroke | Comms building, reinforcement calldown |
| **Stencil overspray** | Noise vignette (grain, 4-8 px radius) at 6% opacity on text masks | Logo, unit card headers |

### 6.2 Western Alliance Faction Icon

Primary shape: NATO compass rose (8 equal-length spokes, no arrowheads) inscribed in a circle.
Outer ring: 2px `#EDE8DC`. Inner fill: `#1C2B3A`. Spoke color: `#F5A623`.
Minimum rendered size: 24 x 24 px.

### 6.3 Classic Enemy Faction Icon

Primary shape: A 5-pointed star (Communist heraldry grammar, but non-literal: slightly irregular
proportions, avoiding direct national-flag replication). Inscribed in a rounded square (4px radius).
Outer fill: `#8B1A1A`. Star: `#EDE8DC` silhouette.
Minimum rendered size: 24 x 24 px.

### 6.4 Unit Role Badges (16 x 16 px)

| Role | Icon Description |
|---|---|
| Infantry | Single silhouette, upright, rifle at 45 degrees |
| Armor | Side-view tank silhouette, one track visible |
| Artillery | Long-barrel side view, elevated barrel |
| Air Defense | Upward-pointing missile with arc |
| Engineer | Crossed wrench + pickaxe |
| Command | Star with 3 radiating lines (signal) |
| Support | Medic cross (white on green) |

All badges: white silhouette on transparent background. Faction colorization applied in-engine.

---

## 7. Asset Manifest

### 7.1 Branding / Identity

| Asset | Dimensions | Format | Notes |
|---|---|---|---|
| `logo-primary.png` | 480 x 96 px | PNG-32 | Option A lockup, full color |
| `logo-monochrome.png` | 480 x 96 px | PNG-32 | White knockout on transparent |
| `logo-emblem-only.png` | 128 x 128 px | PNG-32 | Octagonal crosshair ring |
| `logo-emblem-small.png` | 48 x 48 px | PNG-32 | Same, aliased for HUD |
| `wordmark-horizontal.png` | 360 x 48 px | PNG-32 | Text only, no emblem |
| `classified-stamp.png` | 200 x 80 px | PNG-32 | Red ink "CLASSIFIED" stencil stamp |

### 7.2 Loading Screens

| Asset | Dimensions | Format | Notes |
|---|---|---|---|
| `loading-bg-satellite.png` | 1920 x 1080 px | PNG-32 | Desaturated aerial texture, dark tinted |
| `loading-bg-grid-overlay.png` | 1920 x 1080 px | PNG-32 | Transparent grid, 1% opacity white |
| `loading-panel.png` | 860 x 480 px | PNG-32 | Semi-transparent panel, chamfered |
| `loading-progressbar-bg.png` | 840 x 6 px | PNG-32 | Empty progress track |
| `loading-progressbar-fill.png` | 840 x 6 px | PNG-32 | Filled state, #39D353 |
| `loading-screen-briefing.png` | 1920 x 1080 px | PNG-32 | Alternate: dossier/folder desk layout |

### 7.3 Main Menu

| Asset | Dimensions | Format | Notes |
|---|---|---|---|
| `menu-bg-main.png` | 1920 x 1080 px | PNG-32 | SATELLITE PASS background |
| `menu-panel-bg.png` | 360 x 320 px | PNG-32 | Chamfered panel, semi-transparent fill |
| `menu-btn-default.9.png` | 240 x 40 px | 9-slice PNG | Default button state |
| `menu-btn-hover.9.png` | 240 x 40 px | 9-slice PNG | Hover button state, amber border |
| `menu-btn-active.9.png` | 240 x 40 px | 9-slice PNG | Pressed state |
| `menu-btn-disabled.9.png` | 240 x 40 px | 9-slice PNG | Disabled state |
| `menu-divider.png` | 320 x 1 px | PNG-32 | Horizontal rule #4A5240 |
| `menu-bottom-strip.png` | 1920 x 48 px | PNG-32 | Version/credit bar, dark fill |

### 7.4 In-Game HUD Panels

| Asset | Dimensions | Format | Notes |
|---|---|---|---|
| `hud-panel-bg.9.png` | 200 x 120 px | 9-slice PNG | Generic HUD panel, chamfered |
| `hud-ammo-frame.png` | 80 x 80 px | PNG-32 | Ammo-counter frame (single resource) |
| `hud-resource-bar-bg.png` | 180 x 6 px | PNG-32 | Resource bar empty track |
| `hud-resource-bar-fill.png` | 180 x 6 px | PNG-32 | Fill slice (tinted in-engine) |
| `hud-minimap-border.png` | 200 x 200 px | PNG-32 | Square map frame, 2px #2E3528 |
| `hud-selection-ring-alliance.png` | 64 x 64 px | PNG-32 | Animated crosshair ring, amber |
| `hud-selection-ring-enemy.png` | 64 x 64 px | PNG-32 | Same, crimson |
| `hud-ping-marker.png` | 32 x 32 px | PNG-32 | Map ping indicator |
| `hud-chevron-strip.png` | 48 x 12 px | PNG-32 | >>> motif, decorative |

### 7.5 Faction Icons & Emblems

| Asset | Dimensions | Format | Notes |
|---|---|---|---|
| `faction-alliance-icon.png` | 128 x 128 px | PNG-32 | NATO compass rose, full color |
| `faction-alliance-icon-sm.png` | 32 x 32 px | PNG-32 | Minimap/HUD size |
| `faction-enemy-icon.png` | 128 x 128 px | PNG-32 | 5-point star in rounded square |
| `faction-enemy-icon-sm.png` | 32 x 32 px | PNG-32 | Minimap/HUD size |
| `faction-alliance-banner.png` | 64 x 192 px | PNG-32 | Tall banner for faction panel |
| `faction-enemy-banner.png` | 64 x 192 px | PNG-32 | Same, enemy |

### 7.6 Unit Portrait Frames

| Asset | Dimensions | Format | Notes |
|---|---|---|---|
| `portrait-frame-infantry.png` | 96 x 96 px | PNG-32 | Dog-tag silhouette frame |
| `portrait-frame-vehicle.png` | 96 x 96 px | PNG-32 | Hex armor plate frame |
| `portrait-frame-building.png` | 96 x 96 px | PNG-32 | Angular panel frame |
| `portrait-frame-selected.png` | 96 x 96 px | PNG-32 | Amber-border active state |
| `portrait-placeholder-alliance.png` | 80 x 80 px | PNG-32 | Silhouette placeholder, alliance tint |
| `portrait-placeholder-enemy.png` | 80 x 80 px | PNG-32 | Same, enemy tint |

### 7.7 Unit Role Badges

| Asset | Dimensions | Format | Notes |
|---|---|---|---|
| `badge-infantry.png` | 16 x 16 px | PNG-32 | White silhouette |
| `badge-armor.png` | 16 x 16 px | PNG-32 | Tank side view |
| `badge-artillery.png` | 16 x 16 px | PNG-32 | Long-barrel profile |
| `badge-air-defense.png` | 16 x 16 px | PNG-32 | Upward missile + arc |
| `badge-engineer.png` | 16 x 16 px | PNG-32 | Crossed tools |
| `badge-command.png` | 16 x 16 px | PNG-32 | Star + 3 signal lines |
| `badge-support.png` | 16 x 16 px | PNG-32 | Medic cross |

### 7.8 Icon Motifs (Reusable)

| Asset | Dimensions | Format | Notes |
|---|---|---|---|
| `icon-crosshair.png` | 32 x 32 px | PNG-32 | Thin-ring + cardinal ticks |
| `icon-crosshair-large.png` | 64 x 64 px | PNG-32 | Selection/targeting version |
| `icon-signal-wave.png` | 24 x 24 px | PNG-32 | 3-arc radio symbol |
| `icon-dogtag.png` | 24 x 40 px | PNG-32 | Tag shape outline |
| `icon-redaction-bar.png` | 200 x 8 px | PNG-32 | Single solid bar |
| `icon-stencil-spray.png` | 256 x 256 px | PNG-32 | Noise mask for overspray FX |

---

## 8. Design Rules Summary

1. **Chamfer everything** — no rounded corners except dog-tags and faction icon backgrounds.
2. **Stencil typefaces only for display** — Bebas Neue / Oswald for headings, Share Tech Mono for
   numbers. Never decorative script.
3. **Faction color isolation** — Alliance amber `#F5A623` and Enemy crimson `#D42B2B` must never
   appear together in the same UI panel; swap the full panel skin based on active faction context.
4. **Monochrome deployability** — every logo and faction icon must work in single-color (white
   knockout on dark, or black on light) without loss of legibility.
5. **Military brevity voice** — all UI copy is imperative or declarative, short, no punctuation
   trails, no exclamation marks.
6. **CC0 / OFL assets only** — all typefaces on Google Fonts with SIL OFL; all textures must be
   procedural, hand-crafted, or explicitly CC0. No ripped game assets.

---

*Generated: 2026-05-28 | DINOForge Brand Identity — Warfare Modern | For art-agent production use.*
