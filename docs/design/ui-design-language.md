# Civis UI/HUD Design Language — "Console Holo"

> **Status:** Binding UI/HUD design language (2026-05-31). Authority for the UI/UX Lead
> re-theming `clients/bevy-ref/src/ui_theme.rs` and every HUD module
> (`game_ui`, `tool_categories`, `event_feed`, `diplomacy_ui`, inspector, minimap, …).
> Supersedes the cyan/gold "AAA dark-glass" theme currently in `ui_theme.rs`, which the
> user judged **too generic / too color-dominated** (vibrant cyan + gold everywhere; the
> accent owns too much screen area, so the surface reads as "a generated game UI").
>
> **Companion to** `docs/research/art-direction.md` (world/material/PBR art bar — still
> authoritative for the 3D world; this doc governs the **2D HUD chrome** only).
>
> **Planner stance:** this is a token/recipe spec. It carries values, hex, and component
> recipes — **no Rust implementation**. The implementer maps these onto egui in
> `ui_theme.rs`; pseudocode appears only where a value is load-bearing.

---

## 0. The thesis — why the current theme reads "generic", and the fix

The current theme fails on **one axis: color area**. Cyan and gold are applied as *fills,
glows, borders, selection tints, hover tints* — accent color touches a large fraction of
on-screen pixels. When the accent owns area, intensity has to drop to stay tolerable, and a
desaturated-accent-everywhere surface is exactly the "default game UI" look.

**The fix is a single inversion:** make the surface **mostly neutral grayscale + glass**, and
let neon appear **only as line/edge/glow accents on active or live elements** — *less color
AREA, equal-or-higher color INTENSITY*. A confident, near-monochrome console punctuated by
sharp, bright neon reads as **designed**, not generated.

This language fuses five exact references:

1. **Xbox 2001 / pre-Frutiger Microsoft** — the original Xbox dashboard "blade" structure:
   dark base, beveled/extruded tech chrome, industrial precision, boot-sequence confidence.
   We take the **structure + retro-tech confidence** (beveled blades, extruded tabs, a
   green-tech signature) — *not* the literal green-orb skin.
2. **Geist (Vercel)** — typographic precision, monospace numerics, high-contrast neutral
   grayscale, exact 1px borders, restraint, generous spacing. We take the **type system +
   discipline**.
3. **Neo-glassmorphism** — frosted translucent layered panels, real depth (blur / elevation /
   inner-glow / drop-shadow), tasteful not bubbly.
4. **Restrained neon accent** — neon as thin lines, active-state glow, scanlines, edge
   highlights only. Mostly-monochrome surface, intense neon punctuation.
5. **Star Wars holograms** — holographic projection for key readouts (inspector, minimap,
   legends/overlays, special panels): scanline texture, cyan/blue holo-glow, slight chromatic
   aberration, subtle flicker/jitter, "projected light" translucency, wireframe/blueprint look.

### The two-tier surface model (the core idea)

Every pixel of the HUD belongs to one of two tiers:

- **CHROME (≈92% of UI pixels): the Console.** Neutral graphite glass, beveled blade
  structure, Geist type, 1px hairlines. **No accent fill.** Neon appears here only as a
  1px active edge or a focus glow. This is Xbox-blade × Geist × neo-glass.
- **HOLO (≈8% of UI pixels): the Projection.** Reserved for live, data-dense, "scanned"
  readouts — inspector, minimap, overlay legends, alerts. Cyan/blue projection with
  scanlines + flicker + aberration. This is where color *intensity* lives, concentrated in
  small area. This is the Star Wars layer.

Keeping these tiers strictly separated is what kills the generic feel: the eye reads a calm
metal console with a few glowing holographic instruments embedded in it.

---

## 1. Color tokens (locked hex)

Deliberately **NOT** the old cyan-everywhere + gold-everywhere. The palette is a **10-step
neutral graphite ramp** (the entire chrome tier is built from this — zero accent), **one
primary neon** (electric green — the Xbox-2001 signature, used as line/edge only), **one
restrained warm signal** (amber, for positive/confirm/treasury — sharp, tiny), and a
**hologram cyan family** (used ONLY inside the holo tier).

Color is assigned **by role**, never ad-hoc (the DINOForge discipline). The chrome tier may
use **only** the graphite ramp + green edge + amber signal + semantic statuses. The holo tier
may use **only** the holo-cyan family.

### 1.1 Neutral graphite ramp — the entire chrome surface

| Token | Hex | Premult. alpha (glass) | Role |
|-------|-----|------------------------|------|
| `INK_0` | `#05070A` | 244 | void / deepest scrim behind floating panels |
| `INK_1` | `#0A0D12` | 240 | panel bottom / blade base, viewport letterbox |
| `GRAPHITE_900` | `#0F131A` | 236 | **primary panel fill** (the console face) |
| `GRAPHITE_800` | `#161B23` | 236 | inset wells, list rows, sliders track |
| `GRAPHITE_700` | `#1E242E` | 238 | chips / inactive buttons (mid surface) |
| `GRAPHITE_600` | `#272E3A` | 240 | hover surface / raised nested cards |
| `GRAPHITE_500` | `#333C4A` | — | active-but-neutral surface, pressed blade |
| `STEEL_400` | `#4A5564` | — | **bevel highlight** (top/left lit edge) |
| `STEEL_300` | `#5E6A7A` | — | hairline borders, inactive widget stroke |
| `STEEL_200` | `#7C8898` | — | tick marks, disabled glyphs |

> **Bevel rule (Xbox-2001):** every raised blade/button gets a **2-tone edge** — a 1px
> `STEEL_400` highlight on its **top + left**, and a 1px `INK_1` shadow on its **bottom +
> right**. This single trick is what makes the chrome read "extruded tech panel" instead of
> "flat rounded rectangle". It costs two strokes per widget.

### 1.2 Text ramp (Geist neutral, high contrast)

| Token | Hex | Role |
|-------|-----|------|
| `TEXT_HI` | `#ECEFF4` | primary body, headings, live values |
| `TEXT_MID` | `#9AA4B2` | secondary text, field labels |
| `TEXT_LOW` | `#646F7E` | captions, units, inactive tabs, hints |
| `TEXT_DISABLED` | `#3C4450` | disabled |

### 1.3 Neon accent — the ONE signature (electric green)

The Xbox-2001 signature hue, modernized. **Used as line / edge / glow only — never as a fill,
never as a large tint.** This is the restraint that fixes the generic feel.

| Token | Hex | Role (line/edge/glow ONLY) |
|-------|-----|----------------------------|
| `NEON` | `#3DF07A` | active tab underline, focused-widget 1px edge, selection ring, "live" pulse |
| `NEON_HI` | `#8BFFB4` | hottest core of a neon glow (1px inner line) |
| `NEON_DIM` | `#1E7A45` | neon at rest / pre-glow trace lines, scanline base in chrome |

**Hard budget:** neon may touch **≤ 8% of any panel's pixel area**. If a mock looks like it
breaks this, it's wrong. Neon = the edge of the active thing + its glow halo, nothing more.

### 1.4 Warm signal — restrained amber (positive / confirm / treasury)

A *second* accent, used even more sparingly than neon, purely semantic. Sharp and tiny.

| Token | Hex | Role |
|-------|-----|------|
| `AMBER` | `#F2B33D` | treasury/era value, "confirm/positive", positive delta arrows |
| `AMBER_HI` | `#FFE3A0` | amber glow core |

### 1.5 Hologram cyan family — the PROJECTION tier ONLY

Never appears in chrome. This is the Death-Star-plans / Leia-message light.

| Token | Hex | Role |
|-------|-----|------|
| `HOLO_CORE` | `#7FE9FF` | holo line work, wireframe, primary projected text |
| `HOLO_GLOW` | `#2FBFE6` | holo glow / bloom color, scanline tint |
| `HOLO_DEEP` | `#0E3A4A` | holo panel translucent fill (very low alpha) |
| `HOLO_ABERR_R` | `#FF3B6B` | chromatic-aberration **red** ghost channel |
| `HOLO_ABERR_B` | `#3B7BFF` | chromatic-aberration **blue** ghost channel |

### 1.6 Semantic status (world/state, not chrome decoration)

Used as **small dot/badge/glyph + text**, never as panel fills.

| Token | Hex | Role |
|-------|-----|------|
| `OK` | `#46D67A` | healthy / positive (distinct from NEON: less saturated, fill-safe) |
| `WARN` | `#F2B33D` | caution (shares AMBER) |
| `DANGER` | `#F0556B` | alert / negative / war |
| `MANA` | `#9B7BF0` | disaster / magic / arcane category |

---

## 2. Glass parameters (neo-glassmorphism, tasteful)

Three elevation levels. Each is `fill + blur + bevel-edges + border + shadow`. Values are
egui-mappable (`Frame.fill`, `Shadow`, strokes; blur is faked via the premult-alpha fill +
the `INK_0` scrim behind floating panels, since egui has no live backdrop blur — see §2.1).

| Elevation | Use | Fill | Border (1px) | Bevel | Shadow (`offset / blur / spread / black-alpha`) | Corner |
|-----------|-----|------|--------------|-------|------------------------------------------------|--------|
| **E0 — docked** | top bar, toolbar, anchored panels | `GRAPHITE_900` @236 | `STEEL_300` @ 0.6 | hi `STEEL_400` T/L, lo `INK_1` B/R | `0,3 / 10 / 0 / 90` | `8px` |
| **E1 — floating** | flyouts, cards, dropdowns | `GRAPHITE_900` @236 | `STEEL_300` @ 0.7 + 0.5px `NEON_DIM` inner | bevel + 1px inner `STEEL_400`@0.25 sheen | `0,8 / 22 / 0 / 135` | `10px` |
| **E2 — modal** | menus, dialogs, loading | `INK_1` @244 over full `INK_0`@180 scrim | `STEEL_300` @ 0.8 | bevel + top sheen | `0,14 / 36 / 2 / 170` | `12px` |

### 2.1 Faking "frosted blur" in egui (no backdrop filter)

- **Scrim, don't blur.** Behind every floating/modal panel paint a full-rect `INK_0` @ ~180
  alpha. This separates the panel from the busy 3D world the way a blur would, cheaply.
- **Top sheen line.** A 1px horizontal line at the panel's top inner edge, `STEEL_400` @ 0.18
  — the "light catching the glass pane" cue (matches DINOForge's glass-pill highlight).
- **Inner-glow ring.** 1.5px-inset 1px stroke, `STEEL_400` @ 0.25 (chrome) or the accent @
  0.35 when the panel is **focused/active**. This is the only place accent enters a resting
  panel — and only on focus.
- **Layered translucency.** Stack at most 2 glass levels visually; deeper nesting uses
  `GRAPHITE_800` insets (opaque wells), not more translucency, to avoid mud.

---

## 3. Hologram recipe (the Star Wars projection layer)

This is the signature. Apply **only** to the holo tier (§6 map: inspector, minimap, overlay
legends, alert toasts, the boot/loading projection). All values are concrete so the
implementer can paint it with egui `Painter` primitives + a per-frame `time` uniform.

### 3.1 Holo panel base

- **Fill:** `HOLO_DEEP` `#0E3A4A` @ **0.22 alpha** — barely-there projected volume (you see the
  world faintly through it). NOT an opaque panel.
- **Border:** 1px `HOLO_CORE` @ 0.8, with a 3px outer `HOLO_GLOW` blur halo (additive).
- **Corner brackets, not full frame:** draw `⌐ ¬ L ⌐`-style L-brackets at the four corners
  (12px legs, 1px `HOLO_CORE`) instead of a closed rectangle — the "targeting/blueprint"
  read. Edges between brackets are implied by the scanlines.

### 3.2 Scanlines

- **Spacing:** horizontal lines every **3px** (1px line + 2px gap).
- **Opacity:** `HOLO_GLOW` @ **0.10**. Plus one brighter "sweep" line at `HOLO_CORE` @ 0.35
  that travels top→bottom over **2.2s**, looping (the "active scan" cue).
- **Texture, not noise:** keep them perfectly horizontal and regular — irregular = grunge,
  regular = hologram.

### 3.3 Glow / bloom

- Projected **text and line work** get a `HOLO_GLOW` glow: 4px blur radius, intensity ~0.6.
- In-world holo (if projected over the 3D view) rides the existing bloom pass; in pure-egui
  HUD, fake it with a 2-pass draw: wide soft stroke (`HOLO_GLOW`@0.4, 3px) under a crisp
  core stroke (`HOLO_CORE`, 1px).

### 3.4 Chromatic aberration

- Offset the holo content into **two ghost copies**: `HOLO_ABERR_R` shifted **+1px x / +0.5px
  y**, `HOLO_ABERR_B` shifted **−1px x / −0.5px y**, each @ 0.35 alpha, with the crisp
  `HOLO_CORE` copy on top. Apply to **text/wireframe only**, not the fill — keeps it legible.
- Aberration offset **grows to ±2px during a flicker** (see below), then settles — sells the
  "projector destabilizing" feel on state changes.

### 3.5 Flicker / jitter

- **Idle flicker:** multiply whole-holo opacity by `0.92 + 0.08 * noise(t)` where the
  brightness dips happen ~every **1.5–4s** (randomized), each dip lasting ~**80ms**. Subtle —
  never a strobe.
- **Spawn flicker:** when a holo panel opens, run a **220ms** intro: opacity ramps
  `0 → 1.1 → 1.0`, aberration `±2px → ±1px`, plus 2–3 fast brightness stutters — the Leia
  "projector switching on" moment.
- **Vertical jitter:** the whole projection shifts **±0.5px vertically** on a slow 0.7Hz sine
  — barely perceptible "unstable beam."

### 3.6 Projection tint & content style

- All holo text/numbers: `HOLO_CORE`, monospace, slightly **letter-spaced (+0.5px)**.
- Data viz inside holo (bars, sparklines, the minimap terrain) is **wireframe/blueprint**:
  thin `HOLO_CORE` outlines, `HOLO_GLOW`@0.15 fills, no opaque blocks. Think Death Star plans.
- A faint **`HOLO_GLOW` radial gradient** from panel center → edges (center brighter) sells
  "light emitted from a projector base."

---

## 4. Typography (Geist-style, monospace numerics)

Geist discipline: a clean neutral sans for labels/prose, a **monospace for every number,
unit, coordinate, ID, and tabular value**. Mono numerics are a huge part of the "precision
console" read and directly counter the generic feel.

**Fonts:** ship **Geist Sans** + **Geist Mono** (OFL, free, self-hosted — register via
`egui` `FontDefinitions`). Fallback: Inter + JetBrains Mono. Never rely on the egui default
proportional font for numerics.

### 4.1 Type scale (sizes in px; tight, Geist-like)

| Style | Font | Size | Weight | Tracking | Use |
|-------|------|------|--------|----------|-----|
| `Display` | Sans | 22 | 600 | −0.2 | screen titles, era name |
| `Heading` | Sans | 16 | 600 | 0 | panel headers |
| `Body` | Sans | 13.5 | 400 | 0 | prose, descriptions |
| `Label` | Sans | 11.5 | 500 | **+0.6** | field labels, tab text — **UPPERCASE** |
| `Caption` | Sans | 10.5 | 400 | +0.3 | hints, units-as-words |
| `Numeric` | **Mono** | 14 | 500 | 0 | **all stat values, counts, deltas** |
| `Numeric SM` | **Mono** | 11.5 | 500 | 0 | chip values, table cells |
| `Code/Coord` | **Mono** | 12 | 400 | 0 | coordinates, seeds, IDs, debug |

**Rules:**
- **Tabular numbers** (`tnum`) on all mono so columns align and counters don't jitter.
- Labels are **UPPERCASE + tracked** (`Label` style) — the Xbox/Geist "system label" look.
- Headings/labels: `TEXT_HI`/`TEXT_MID`. Values: `TEXT_HI` (or `AMBER`/`NEON`/`DANGER` when
  semantically hot). Units (`px`, `pop`, `t`): always `TEXT_LOW`, mono, after a hair space.

### 4.2 Spacing (generous, Geist restraint)

- `item_spacing`: `8 × 8`. `button_padding`: `12 × 7`. `window_margin`: `14`.
- Section rhythm via **hairlines** (`STEEL_300`@0.4) + 8px breathing room, not boxes.
- Minimum 12px gutter between a panel edge and its content. Do not crowd.

---

## 5. Structural component language (Xbox-2001 beveled "blade")

The chrome kit. Every component is **flat graphite glass + a 2-tone bevel + 1px hairline**,
with neon entering **only on active/focus**. This is the retro-tech blade structure.

### 5.1 Panel (E0/E1/E2 per §2)

`GRAPHITE_900` fill → bevel edges (`STEEL_400` T/L, `INK_1` B/R) → 1px `STEEL_300` border →
top sheen line → shadow. **Header strip:** a `GRAPHITE_800` band with an `UPPERCASE Label`
title left-aligned, a 1px `NEON_DIM` underline beneath the whole strip, and a single 6px
`NEON` "live dot" at the strip's left edge **only if the panel shows live data**.

### 5.2 Blade button

The signature widget. A button is a small extruded blade:

- **Rest:** `GRAPHITE_700` fill, bevel (T/L `STEEL_400`@0.5, B/R `INK_1`), 1px `STEEL_300`
  border, `Label` text `TEXT_MID`. **No accent.**
- **Hover:** fill → `GRAPHITE_600`, text → `TEXT_HI`, bevel highlight intensifies; a 1px
  `NEON`@0.5 line appears on the **bottom edge only** (the blade "lights its leading edge").
- **Active/pressed:** fill → `GRAPHITE_500`, bevel **inverts** (shadow now T/L — pressed-in),
  full 1px `NEON` border + 3px `NEON`@0.3 outer glow, text `TEXT_HI`.
- **Primary/confirm variant:** same but accent = `AMBER` instead of `NEON`.

### 5.3 Tab strip (Xbox blade menu)

Horizontal blades that read like the Xbox dashboard ribbon:

- Each tab: `Label` text. Inactive: `TEXT_LOW`, no fill. Active: `TEXT_HI` + a **2px `NEON`
  underline** with a 4px `NEON`@0.4 glow + a faint `GRAPHITE_700` raised blade behind it.
- The active blade sits **2px taller** than inactive ones (the extruded-selection cue).
- A continuous 1px `STEEL_300` hairline runs under the whole strip; the active underline
  overrides it in neon.

### 5.4 Chip (stat pill)

`GRAPHITE_700` fill, `RADIUS_SM`, 1px `STEEL_300` border, subtle bevel. Layout:
`[glyph]  LABEL  value` — glyph in semantic color, `LABEL` in `TEXT_LOW` uppercase, **`value`
in Mono `TEXT_HI`**. The chip itself never glows; only its glyph carries color. Delta chips
append a mono `▲`/`▼` in `OK`/`DANGER`.

### 5.5 Slider

- **Track:** `GRAPHITE_800` inset well, 1px inner `INK_1` shadow (recessed).
- **Filled portion:** `NEON_DIM` → `NEON` left-to-right gradient, 2px tall, with a 2px
  `NEON`@0.4 glow.
- **Handle:** a small `GRAPHITE_600` blade with full bevel + 1px `NEON` border; on drag, a
  floating Mono value tooltip (`HOLO`-styled) tracks the handle.
- **Tick marks:** `STEEL_200`, 1px, at quartiles.

### 5.6 Toggle / segmented control

Segmented blades sharing one bevel housing. Selected segment = pressed-in (inverted bevel) +
`NEON` bottom edge. Unselected = raised `GRAPHITE_700`. Boolean toggle = same, two segments.

### 5.7 Inset well / list row

`GRAPHITE_800` fill, recessed bevel (shadow T/L), 1px `INK_1` inner border. Selected row gets
a 2px `NEON` **left edge bar** (not a fill) + `GRAPHITE_700` fill bump. Hover = `STEEL_300`
left edge bar.

### 5.8 Scrollbar

Thin (6px), `GRAPHITE_700` thumb, `STEEL_300` on hover, no track fill. Never neon.

---

## 6. Per-HUD-element application map

| HUD element | Tier | Recipe |
|-------------|------|--------|
| **Top resource bar** | CHROME E0 | Docked graphite blade across top. Chips (§5.4) for pop/treasury/era/date. Treasury value = `AMBER` Mono; others `TEXT_HI` Mono. One `NEON` live-dot at far left = sim running. Pause = dot goes `WARN`. **No background tint, no accent fill.** |
| **Category toolbar** | CHROME E0 | Vertical/horizontal strip of **blade buttons** (§5.2) with material/tool glyphs. Active tool = pressed-in inverted bevel + full `NEON` edge + glow. Flyout drawers open as **E1 floating** panels. This is the Cities-Skylines toolbar in Xbox-blade dress. |
| **Inspector** | **HOLO** | The hero holo panel (§3). Selected entity rendered as a **wireframe/blueprint readout**: holo-cyan line work, Mono stat rows, corner brackets, scanlines, idle flicker, aberration on text. Opening it plays the 220ms spawn-flicker. This is the Death-Star-plans moment. |
| **Minimap** | **HOLO** | Projected terrain as **blueprint contours** (thin `HOLO_CORE` lines, `HOLO_GLOW`@0.15 fills), 3px scanlines, slow scan-sweep, corner brackets. Viewport rect = 1px `HOLO_CORE` bracket. Pings = brief `HOLO_CORE` flare. Reads like a tactical hologram, not a Google-map inset. |
| **Overlay legends** (heat/political/resource maps) | **HOLO** | Floating holo card: gradient swatch as a thin wireframe bar, Mono range labels, scanlines. Legend for an overlay = a small projected instrument. |
| **Notifications / toasts** | mixed | Routine info = CHROME E1 slab, `TEXT_HI`, semantic glyph, auto-dismiss. **Critical alerts** (war, disaster) = **HOLO** with `DANGER`-tinted aberration + a sharper flicker + a single `DANGER` edge pulse. Severity literally changes the medium (slab → projection). |
| **Menus / dialogs** (settings, save/load, diplomacy) | CHROME E2 | Modal graphite glass over `INK_0` scrim. Blade buttons, tab strips, sliders per §5. Geist type, generous spacing. Confirm = `AMBER` primary blade; destructive = `DANGER`. Calm and precise — the "system" surface. |
| **Loading / boot** | **HOLO** | Full Star-Wars-projector moment: logo + worldgen progress as a holo wireframe globe building up (scanlines sweeping, brackets, flicker, aberration), Mono progress `XX.X%` + phase label. The Xbox boot-sequence confidence, in holo. |
| **Tooltips** | CHROME E1 (light) | Small graphite slab, 1px `STEEL_300`, `Caption`/`Numeric SM`. Value-bearing tooltips (slider drag, hover stat) get a 1px `NEON` left edge. |
| **Context/radial menus** | CHROME E1 | Blade buttons radiating; hovered = `NEON` bottom edge. |

**Density discipline:** holo is *expensive attention*. At most **2 holo surfaces** visible at
once in steady state (inspector + minimap). Everything else is calm chrome. This concentration
is what makes the holo read as special.

---

## 7. Motion & transitions

Restrained, mechanical, confident — Xbox-blade precision, not bouncy.

| Interaction | Motion | Timing / curve |
|-------------|--------|----------------|
| Blade button hover | fill + bevel + edge-line fade in | 90ms ease-out |
| Blade press | bevel invert + neon glow snap | 60ms, near-instant (tactile) |
| Tab switch | active underline **slides** to new tab; raised blade rises 2px | 160ms ease-in-out |
| Flyout open (E1) | slide 8px in from anchor + fade | 140ms ease-out + shadow grows |
| Modal open (E2) | scrim fades + panel scales `0.98 → 1.0` + fade | 180ms ease-out |
| Holo panel spawn | the 220ms projector flicker (§3.5) | custom |
| Holo idle | continuous flicker/jitter/scan-sweep (§3.2/3.5) | looping |
| Live value change | Mono digit **flash** to accent then settle to `TEXT_HI` | 250ms color-fade |
| Alert arrival | single `DANGER` edge pulse + holo aberration spike | 300ms, once |

**Rules:** never animate layout-jarring scale on chrome (only modals, subtly). Neon glow
**pulses** only on truly-live elements (sim-running dot, active alert) at a slow ~0.4Hz —
everything else is steady. No spinners; use a holo scan-sweep or a Mono determinate `%`.

---

## 8. Mockups

### 8.1 Top bar + active toolbar (CHROME) with inspector (HOLO)

```
┌─[● LIVE]──────────────────────────────────────────────────────────── CHROME E0 ─┐
│ ◆ POP 12.4K   ⛁ TREASURY 8.2K   ⌖ ERA  IRON   ◷ YEAR  −420   ☼ 14:32           │   ← graphite glass, Mono values,
└──────────────────────────────────────────────────────────────────────────────────┘     TREASURY in amber, rest TEXT_HI
                                                                ╔════════════════════╗
 ┌─CHROME E0────┐                                               ║ ⌐              ¬   ║  HOLO inspector
 │ [TERRAIN]▾   │   ← inactive blade, TEXT_LOW                  ║   SETTLEMENT       ║  · cyan wireframe
 │ ▛WATER ▟ ◀── │   ← ACTIVE blade: pressed-in, NEON edge+glow  ║   ░░░▟▙░░░  (wire)  ║  · 3px scanlines
 │ [FOREST]     │                                               ║   POP    1 240     ║  · Mono rows
 │ [SPAWN]      │                                               ║   FOOD   +12.4 /t  ║  · corner brackets ⌐ ¬ L
 │ [ERASE]      │                                               ║   MOOD   ▲ 0.78    ║  · flicker + R/B aberration
 └──────────────┘                                               ║ L              ⌐   ║    on the text
   neon ONLY on the active blade edge                           ╚════════════════════╝
```

### 8.2 Minimap (HOLO) — tactical projection, bottom-right

```
        ╔═══════════════════════╗
        ║ ⌐                 ¬  ║   translucent HOLO_DEEP @0.22 fill
        ║   ◜‾‾◝   blueprint    ║   terrain = thin HOLO_CORE contour lines
        ║  ◟ ▞▞ ◞  contours    ║   3px horizontal scanlines @0.10
        ║   ╰──╯   + scan ▔▔▔▔ ║ ← bright scan-sweep line travels top→bottom (2.2s)
        ║  ┌╌╌┐  viewport rect  ║   viewport = 1px HOLO_CORE corner-bracket box
        ║ L         (bracket) ⌐ ║   ping = brief HOLO_CORE flare
        ╚═══════════════════════╝   whole panel jitters ±0.5px vertically (0.7Hz)
```

### 8.3 Settings modal (CHROME E2) — blade kit on scrim

```
░░░░░░░░░░ INK_0 scrim @180 over blurred-feeling world ░░░░░░░░░░
        ┌──────────────────────────────────────────────┐  E2 graphite glass
        │ SETTINGS                                  [✕] │  Display title, bevel + sheen
        │ ┌[GRAPHICS]─[AUDIO]──[GAMEPLAY]──[MODS]──┐    │  tab strip: GRAPHICS active =
        │ │‾‾‾‾‾‾‾‾‾                                │    │    NEON underline + glow + raised
        │ │ RESOLUTION      ▸ 2560×1440  (Mono)     │    │
        │ │ RENDER SCALE    ▟▆▆▆▆▆▆▆━━━━━○  1.0     │    │  slider: NEON_DIM→NEON fill,
        │ │ BLOOM           [ OFF ][▟ ON ▙]          │    │    blade handle, Mono readout
        │ │ HOLO INTENSITY  ▟▆▆▆▆━━━━━━━━○  0.6      │    │  segmented toggle: ON pressed-in
        │ │                                          │    │
        │ │                        [ CANCEL ] [ ▟APPLY▙ ] │  APPLY = AMBER primary blade
        │ └──────────────────────────────────────────┘    │  CANCEL = neutral blade
        └──────────────────────────────────────────────┘
```

---

## 9. Implementer handoff — re-theming `ui_theme.rs` + the HUD

Concrete mapping so an engineer can execute without further design input. (Token names → the
egui constants they replace.)

1. **Swap the palette block** (`ui_theme.rs` ll. 25–61): replace the cyan/gold + 4-step ramp
   with §1's 10-step graphite ramp, `NEON`/`AMBER`/`HOLO_*`/semantic tokens. Rename
   `ACCENT`→`NEON`, `GOLD`→`AMBER`; keep names that other modules import as **re-export
   aliases** to avoid a churny rename across the HUD (`pub const ACCENT: … = NEON;`).
2. **`apply_widget_visuals`:** encode the blade bevel — set `inactive/hovered/active.bg_fill`
   to `GRAPHITE_700/600/500`; move accent OFF `bg_fill` (the current `ACCENT.gamma_multiply`
   fills are the generic culprit) and onto **strokes only** (`active.bg_stroke = 1px NEON`,
   `hovered` gets a bottom `NEON`@0.5 edge). egui can't do per-side bevel natively → add a
   small `blade_frame(painter, rect, pressed)` helper that paints the 2-tone T-L / B-R edges.
3. **Fonts:** register Geist Sans + Geist Mono in `FontDefinitions`; rewire `apply_type_scale`
   to §4.1 (add a distinct `Numeric` mono style; route all value labels through it). Enable
   tabular figures.
4. **Frames:** extend `panel_frame`/`accent_frame` into `frame_e0/e1/e2` (§2) adding the scrim
   helper, top-sheen line, and bevel. Keep `accent_frame` as `frame_e1` with accent inner-glow
   on focus only.
5. **New holo module** `ui_holo.rs`: a `holo_panel(ui, rect, time, content)` painter
   implementing §3 (scanlines, sweep, brackets, flicker, aberration, glow). Inspector,
   minimap, overlay legends, alert toasts, loading call into it. Needs a `time: f32` from the
   app clock for animation. This is the only net-new surface; everything else is re-skinning.
6. **`chip`:** relayout to `[glyph] LABEL value(Mono)`; glyph carries the only color; drop the
   full-color text.
7. **Keep** `panel_shadow`, `inner_glow`, `hairline`, `compact`, the tests — extend the
   depth-ramp test to assert the 10-step graphite ordering and that no accent is used as a
   widget `bg_fill`.

**Verification (per project vision-verify rule):** after re-theme, screenshot and read the
pixels. Pass = a calm graphite console where neon touches only active edges (eyeball the ≤8%
budget), TREASURY is the lone amber value, and the inspector + minimap clearly read as glowing
cyan holograms distinct from the chrome. Telemetry alone is not "done."

---

## 10. The five signature moves (what kills the generic feel)

1. **Two-tier surface** — calm graphite Console chrome (≈92%) vs. glowing cyan Holo
   projection (≈8%). Strict separation; never blend the two palettes.
2. **Neon as edge, never fill** — accent lives only on active/focused **edges + glow** with a
   hard ≤8%-area budget. Inverting the old fill-everywhere cyan/gold is the core fix.
3. **Xbox-2001 2-tone bevel** — every blade gets a `STEEL_400` top-left highlight + `INK_1`
   bottom-right shadow, and **inverts on press**. Extruded tech chrome, not flat rectangles.
4. **Geist mono numerics** — every value/count/coord in Geist Mono with tabular figures +
   uppercase tracked labels. Precision-console typography, not default game UI text.
5. **Star Wars hologram instruments** — the inspector and minimap are genuine projections:
   scanlines, scan-sweep, corner brackets, idle flicker, chromatic aberration, blueprint
   wireframe. Concentrated, intense, unmistakably *designed*.
