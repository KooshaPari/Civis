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

### Sub-directories (prior art, created by previous agent)

| Directory | Contents |
|-----------|----------|
| `faction-crests/` | Per-faction SVG crest art (populated separately). |
| `tool-icons/` | Toolbar / action icon SVGs (populated separately). |

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
| Faction crests | `256×256` or `512×512` | Square, transparent bg |
| Tool icons | `64×64` or `128×128` | Square, transparent bg |

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
