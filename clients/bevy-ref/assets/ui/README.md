# UI Assets — `clients/bevy-ref/assets/ui/`

Vector source art for the Civis main-menu and loading screens.  
Bevy cannot load SVG directly; all files must be rasterised to PNG before use (see below).

---

## Colour Palette

| Role       | Value              | Usage                                      |
|------------|--------------------|--------------------------------------------|
| Cyan       | `rgb(80, 200, 240)` / `#50C8F0` | Rings, borders, glow accents, stars       |
| Gold       | `#E8B84B`          | Sun, flags, compass needle N, decorative marks |
| Dark navy  | `#04091a`          | Sky top, background base                  |
| Deep green | `#1e4a30`          | Land-mass fill on globe / scene terrain   |
| Off-white  | `#e8eef5`          | Primary wordmark text                     |

---

## Asset Inventory

### Root-level SVGs

| File | viewBox | Purpose |
|------|---------|---------|
| `logo.svg` | `0 0 800 300` | **CIVIS wordmark / emblem** — terraced globe with sun, bold serif lettering, cyan + gold accents on dark navy. Used as title logo overlay. |
| `title-bg.svg` | `0 0 2560 1440` | **Main-menu background** — full-resolution painterly-vector scene: deep-blue→warm-horizon sky, stars, moon, distant mountain ridge (right-heavy), rolling mid hills, settlement/tower silhouette (right-center), foreground ground plane. Center-left kept open for menu buttons. |
| `loading-bg.svg` | `0 0 2560 1440` | **Loading screen background** — dimmer (~40% lower contrast) variant of the title scene. A clear horizontal band at y≈840–920 is reserved for the progress bar track and tip text. Bevy code draws the actual bar; the SVG contains a faint guide track only. |
| `loading-spinner.svg` | `0 0 120 120` | **Loading spinner emblem** — compass/world-ring, radially symmetric so it looks good at any rotation angle. Outer cyan arc (gradient), inner gold arc, compass needle (gold N / cyan S), mini globe in the center with latitude/longitude lines. Bevy rotates this via a `Transform` system. |

### Sub-directories

#### `tool-icons/` — 64×64 viewBox, monochrome-ish, cyan + gold on dark navy

| File | Description |
|------|-------------|
| `select.svg` | Arrow cursor — pointer tool |
| `inspect.svg` | Magnifier with cyan lens and gold handle |
| `spawn-life.svg` | Cell-bud sprout with nucleus dot — life is emergent |
| `spawn-structure.svg` | Isometric block with gold placement cross |
| `terraform.svg` | Terrain wave with raise/lower arrows |
| `spawn-material.svg` | Flask pouring particle stream |
| `disaster.svg` | Meteor trail + impact burst |
| `diplomacy.svg` | Three linked nodes, cyan + gold arcs |
| `policy.svg` | Scroll with gear overlay |
| `time-play.svg` | Clock face + play triangle |
| `time-pause.svg` | Clock face + pause bars |

#### `faction-crests/` — 128×128 viewBox, accent-color sigils, transparent bg

| File | Color | Sigil name | Design motif |
|------|-------|-----------|--------------|
| `crest-red.svg` | `#c0392b` (crimson) | Pyre Sigil | Tall diamond totem, crossbar, wing spikes, eye |
| `crest-blue.svg` | `#2980b9` (ocean blue) | Tide Glyph | Concentric rings, wave arcs, trident central mark |
| `crest-green.svg` | `#27ae60` (forest green) | Root Sigil | Branching fractal tree with leaf nodes and roots |
| `crest-gold.svg` | `#E8B84B` (gold) | Solar Compass | 8-point sun-wheel, compass needle diamond, hub |

#### `material-icons/` — 48×48 viewBox, voxel material swatches

| File | Material type | Visual metaphor |
|------|--------------|-----------------|
| `water.svg` | Liquid | Blue teardrop with interior wave line |
| `sand.svg` | Powder | Hourglass with falling granule stream |
| `dirt.svg` | Solid (organic) | Isometric cross-section block with grass tuft |
| `stone.svg` | Solid | Faceted irregular boulder with crack lines |
| `lava.svg` | Liquid (hot) | Molten pool with dark crust cracks + splash droplet |
| `gas.svg` | Gas/steam | Three rising wispy plume strokes |
| `ice.svg` | Solid (frozen) | Hexagonal snowflake crystal form |
| `ore.svg` | Solid (mineral) | Rough chunk with cyan glowing vein + gold speckle |

#### `hud/` — UI element SVGs, 9-slice friendly where noted

| File | viewBox | 9-slice guides | Purpose |
|------|---------|---------------|---------|
| `panel-frame.svg` | `0 0 256 160` | x=20/236, y=20/140 | Rounded glass panel, r=12, cyan border + sheen |
| `button.svg` | `0 0 120 40` | x=16/104, y=12/28 | Normal button state |
| `button-hover.svg` | `0 0 120 40` | x=16/104, y=12/28 | Hover/active button state, gold bottom accent |
| `progress-bar-track.svg` | `0 0 240 20` | x=10/230 | Empty progress bar track with tick marks |
| `progress-bar-fill.svg` | `0 0 240 20` | x=10/230 | Filled bar overlay (clip to % width at runtime) |
| `chip-bg.svg` | `0 0 80 28` | x=14/66, y=8/20 | Pill-shaped info chip / resource badge |
| `resource-population.svg` | `0 0 32 32` | — | Three stylized figures (front+back pair) |
| `resource-energy.svg` | `0 0 32 32` | — | Gold lightning bolt |
| `resource-food.svg` | `0 0 32 32` | — | Wheat stalk with grain ears |
| `resource-clock.svg` | `0 0 32 32` | — | Clock face with cyan/gold hands |

---

## SVG → PNG Rasterisation

Bevy's asset loader does not support SVG. Convert to PNG before including in Bevy bundles.

### Recommended tools

- **resvg** (Rust, fast, spec-compliant): `resvg input.svg output.png`
- **rsvg-convert** (librsvg, widely available): `rsvg-convert -o output.png input.svg`

### Asset-pipeline script

The project's `Tools/asset-pipeline` scripts handle batch conversion automatically:

```powershell
# From repo root — converts all ui/**/*.svg to PNG at 1:1 pixel resolution
just asset-pipeline
# or directly:
pwsh Tools/asset-pipeline/Convert-SvgToPng.ps1 -SourceDir clients/bevy-ref/assets/ui
```

### Recommended output resolutions

| Asset | PNG output size | Notes |
|-------|----------------|-------|
| `logo.svg` | `800×300` | Or `1600×600` for HiDPI |
| `title-bg.svg` | `2560×1440` | Match viewBox 1:1 |
| `loading-bg.svg` | `2560×1440` | Match viewBox 1:1 |
| `loading-spinner.svg` | `120×120` (or `240×240` HiDPI) | Keep power-of-two if possible |
| `faction-crests/*.svg` | `256×256` or `512×512` | Square, transparent bg — 2× for HiDPI |
| `tool-icons/*.svg` | `64×64` or `128×128` | Square, transparent bg — 2× for HiDPI |
| `material-icons/*.svg` | `48×48` or `96×96` | Square, transparent bg |
| `hud/panel-frame.svg` | `256×160` | 9-slice; export at 1:1, slice in Bevy `ImageScaleMode::Sliced` |
| `hud/button.svg` | `120×40` | 9-slice corners at x=16/104, y=12/28 |
| `hud/button-hover.svg` | `120×40` | Same 9-slice as button.svg |
| `hud/progress-bar-track.svg` | `240×20` | 9-slice: x=10/230 |
| `hud/progress-bar-fill.svg` | `240×20` | 9-slice: x=10/230; clip to % width at runtime |
| `hud/chip-bg.svg` | `80×28` | 9-slice: x=14/66, y=8/20 |
| `hud/resource-*.svg` | `32×32` or `64×64` | Square, transparent bg |

### 9-slice usage in Bevy

```rust
// panel-frame example — import as ImageScaleMode::Sliced
commands.spawn((
    ImageBundle {
        image: asset_server.load("ui/hud/panel-frame.png").into(),
        style: Style { width: Val::Px(400.0), height: Val::Px(200.0), ..default() },
        ..default()
    },
    ImageScaleMode::Sliced(TextureSlicer {
        border: BorderRect::rectangle(20.0, 20.0),
        center_scale_mode: SliceScaleMode::Stretch,
        ..default()
    }),
));
```

### Batch rasterisation with resvg

```powershell
# Install resvg (Rust): cargo install resvg
# Single file
resvg input.svg output.png --width 128 --height 128

# Batch — all tool icons at 128x128
Get-ChildItem clients/bevy-ref/assets/ui/tool-icons/*.svg | ForEach-Object {
    $out = $_.FullName -replace '\.svg$', '.png'
    resvg $_.FullName $out --width 128 --height 128
}

# Batch — all material icons at 96x96
Get-ChildItem clients/bevy-ref/assets/ui/material-icons/*.svg | ForEach-Object {
    $out = $_.FullName -replace '\.svg$', '.png'
    resvg $_.FullName $out --width 96 --height 96
}

# Batch — faction crests at 512x512
Get-ChildItem clients/bevy-ref/assets/ui/faction-crests/*.svg | ForEach-Object {
    $out = $_.FullName -replace '\.svg$', '.png'
    resvg $_.FullName $out --width 512 --height 512
}
```

### Bevy usage pattern

```rust
// Load pre-rasterised PNG
let logo = asset_server.load("ui/logo.png");
let spinner = asset_server.load("ui/loading-spinner.png");
```

---

## Design Notes

- All backgrounds are intentionally dark (navy/near-black) so UI text is legible at full brightness.
- The title-bg leaves the **left ~38% of the canvas** (x < 960) less busy to accommodate main-menu buttons.
- The loading-bg progress-bar band sits at **y ≈ 840–920** (60% down the 1440-height canvas).
- The spinner is rotation-agnostic: no CSS animation is embedded; Bevy drives rotation via `Transform::rotate_z`.
- Gold (`#E8B84B`) is used sparingly as a highlight/accent colour; cyan (`#50C8F0`) carries the primary brand identity.
