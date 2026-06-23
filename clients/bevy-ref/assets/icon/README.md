# Civis app / window icon

Branding mark for the Civis desktop client, authored in the **"Console Holo"**
design language (see [`docs/design/ui-design-language.md`](../../../../docs/design/ui-design-language.md)
and [`docs/research/art-direction.md`](../../../../docs/research/art-direction.md)).

## The mark

An **emergent voxel-world stack**: a 3-tier isometric ziggurat of voxels
(civilization rising from the substrate) on a graphite console plate, with:

- **Graphite base** — `GRAPHITE`/`INK` ramp plate + Xbox-2001 2-tone bevel.
- **Electric-green neon edge** (`#3DF07A`) — the structural edges of the voxel
  stack; the signature, used as line/edge only.
- **Holo-cyan accent** (`#7FE9FF`) — a projected orbit/scan ring + a rising apex
  spark (the "emergence / live" cue).
- **Corner registration ticks** — the Civis console motif.

The mark is authored to survive 16px (bold neon silhouette) and reward
inspection at 256px (bevel, holo ring, glow).

## Files

| File | Purpose |
|------|---------|
| `civis-icon.svg` | Master source (256 viewBox), full detail. |
| `civis-icon-small.svg` | 16–32px-optimized source (thicker edges, no fine detail). |
| `icon-{16,24,32,48,64,128,256}.png` | Rasterized sizes (small src ≤32, master ≥48). |
| `civis.ico` | Multi-size Windows icon (16/24/32/48/64/128/256). |

## Wiring

- **Window icon (runtime):** `src/window_icon.rs` embeds `icon-64.png` and sets
  it on the primary winit window at startup (`WindowIconPlugin`, added in
  `src/bin/standalone.rs`).
- **`.exe` icon (Windows release):** `build.rs` stamps `civis.ico` onto the
  executable via `winresource`, behind the `exe-icon` cargo feature:
  `cargo build -p civ-bevy-ref --features "bevy,egui,exe-icon" --release`.
- **Docs favicon:** copied to `docs/.vitepress/public/` and wired in
  `docs/.vitepress/config.ts` `head`.

## License

All SVGs here (and the wordmark / splash / tool / material icons under
`../ui/`) are **hand-authored for Civis and are project-owned** — no third-party
or AI-generated raster assets are vendored. Regenerate PNGs with
`pwsh Tools/rasterize-svg.ps1`.
