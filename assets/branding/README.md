# Civis Branding Assets

On-brand vector art for Civis, authored as hand-coded SVG (source of truth) and rasterized to PNG/ICO. Follows the **Keycap Palette**: teal `#7ebab5` + midnight `#090a0c`, glassmorphic, "holocron command deck" aesthetic with an emergence-network motif.

| Asset | Source | Raster | Use |
|-------|--------|--------|-----|
| Mascot / emblem | `mascot.svg` | `mascot.png` (512²), `civis.ico` | app icon, loading emblem, watermark |
| Wordmark | `wordmark.svg` | `wordmark.png` (800×200) | title / header lockup |
| Menu backdrop | `menu-backdrop.svg` | `menu-backdrop.png` (1920×1080) | main-menu background |

The mascot is a geometric world-tree / phoenix formed from interconnected hex nodes and connective lines (emergence network), teal linework with a glassmorphic glow over a midnight glass disc.

## Regenerate rasters from SVG source

```sh
RSVG=/c/iverilog/gtkwave/bin/rsvg-convert
MAGICK="/c/program files/imagemagick-7.1.0-q16-hdri/magick"
"$RSVG" -w 512  -h 512  mascot.svg        -o mascot.png
"$RSVG" -w 800  -h 200  wordmark.svg      -o wordmark.png
"$RSVG" -w 1920 -h 1080 menu-backdrop.svg -o menu-backdrop.png
"$MAGICK" mascot.png -define icon:auto-resize=256,128,64,48,32,16 civis.ico
```

SVG is the source of truth — edit the `.svg`, then regenerate. Branding is **AI-coded, not image-generated** (deterministic, versionable, editable).
