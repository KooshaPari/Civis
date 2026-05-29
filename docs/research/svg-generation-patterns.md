# SVG Generation Patterns for DINOForge Art Tooling

This note is a practical guide for using LLM-generated SVG as the front end of a real art pipeline.

## 1. What LLM-generated SVG is good at

LLM-generated SVG is strongest when the asset is fundamentally geometric and can be expressed with a small number of clean primitives:

- Logos, emblems, sigils, crests
- UI frames, panel borders, corner ornaments
- HUD icons and markers
- Flat-background decorations, badges, glyphs, and simple silhouettes
- Overlay elements that need to scale cleanly at many sizes

It is weak for assets that depend on high-frequency texture, physically believable lighting, painterly detail, or complex organic forms:

- Photoreal characters and environments
- Fur, foliage, cloth micro-detail, smoke, fire, dust
- Hand-painted shading and texture breakup
- Complex translucent materials and irregular brushwork

For those weak cases, use licensed source art, photobash assets, or dedicated generative image tools, then bring the result into the same pipeline as raster source material.

## 2. Patterns for clean, scalable SVG

The most reliable SVGs have a narrow coordinate system, reusable definitions, and a small number of visually meaningful paths.

### ViewBox discipline

Use a fixed artboard and keep the drawing inside it.

- Prefer a canonical `viewBox` like `0 0 256 256` or `0 0 512 512`
- Keep all drawing coordinates inside that box unless intentional bleed is needed
- Avoid dependency on implicit pixel size; `width` and `height` should be presentation hints, not the source of truth
- Use `preserveAspectRatio="xMidYMid meet"` for most icons and emblems

This gives the rasterizer a stable basis for generating multiple output sizes without distortion.

### Primitive structure

Use a small number of high-value shapes:

- `path` for outlines, panels, and complex silhouettes
- `rect`, `circle`, `ellipse`, `line`, and `polyline` for simple geometry
- `path` segments with rounded joins/caps where the look benefits from cleaner edges

Prefer fewer shapes over many tiny fragments. LLMs often over-segment geometry; the pipeline should normalize that into a minimal shape set when possible.

### Reusable `defs`

Put repeated visual language into `<defs>`:

- Gradients for fills and strokes
- Masks and clip paths for cutouts
- Symbols for repeatable motifs
- Filters only when the effect is essential and raster fallback is acceptable

Use IDs that are stable and descriptive so downstream tools can reference them safely.

Example pattern:

```svg
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 256 256">
  <defs>
    <linearGradient id="frameFill" x1="0" y1="0" x2="0" y2="1">
      <stop offset="0%" stop-color="#2d3440"/>
      <stop offset="100%" stop-color="#11161d"/>
    </linearGradient>
    <linearGradient id="frameStroke" x1="0" y1="0" x2="1" y2="1">
      <stop offset="0%" stop-color="#6c7a8c"/>
      <stop offset="100%" stop-color="#2f3b4d"/>
    </linearGradient>
  </defs>
  <path fill="url(#frameFill)" stroke="url(#frameStroke)" d="..." />
</svg>
```

### Styling discipline

Keep styles explicit and local to the SVG.

- Prefer direct attributes for critical fills/strokes
- Use `style` sparingly and only when it reduces repetition
- Avoid external font dependencies
- Avoid embedded raster images unless you intentionally want a hybrid asset

### Geometry discipline

For logos/emblems/icons:

- Snap straight edges and corners to whole units where possible
- Keep stroke widths consistent across related motifs
- Avoid tiny overlaps and near-zero length segments
- Use rounded joins/caps deliberately rather than by accident
- Test the SVG at the smallest intended raster size to catch fragile details

## 3. Converting SVG to game-ready PNG

The pipeline should treat SVG as source art and PNG as build output.

### Inkscape

Inkscape is useful when you want a faithful render with explicit export bounds and deterministic page-based output.

Typical use:

- Set the artboard with the SVG `viewBox`
- Export the page area or the selected object bounds
- Produce one PNG per target size

Good fit:

- UI art
- Emblems
- Assets that need a human-checked export boundary

### resvg

resvg is useful for fast, headless SVG rasterization in build pipelines.

Good fit:

- Batch exports
- CI-friendly rendering
- Multiple size outputs from the same source

Use it when the SVG is intentionally within the supported feature set and you want a fast renderer for predictable geometry-heavy assets.

### ImageMagick

ImageMagick is best treated as a wrapper or post-step when you need resizing, compositing, trimming, or batch manipulation around the SVG render.

Good fit:

- Converting an SVG render into multiple derived PNG sizes
- Trimming or padding after render
- Combining render output with other bitmaps

Practical rule:

- Render with SVG-aware tooling first
- Use ImageMagick for downstream processing, not as the only source of truth for SVG interpretation

### Suggested export matrix

For UI assets, export several sizes from the same SVG source:

- `1x` reference size for inspection
- `2x` for standard UI
- `4x` for high-DPI or future reuse

If the asset is a frame or panel, also export the exact size required by the game UI layout so the atlas or slice system can use it directly.

## 4. Building 9-slice UI panel sprites from SVG

9-slice panels should be authored with layout intent, not as a generic image.

### Authoring pattern

Design the SVG around three regions:

- Corners that never stretch
- Edges that stretch only in one axis
- Center area that can scale freely

That means the art should reserve clear non-stretch zones around all four corners and a stable border band.

### Export pattern

Generate at least one canonical PNG for the panel, then derive slice metadata from that source.

Recommended workflow:

1. Author the panel in SVG with a known artboard
2. Rasterize the full panel to PNG
3. Define the 9-slice borders in a sidecar JSON/YAML manifest
4. Feed the PNG plus slice metadata into the UI importer or runtime pack step

### Slicing guidance

- Keep borders thick enough to survive downscaling
- Avoid decorative micro-details near the slice boundaries
- Make the center region visually neutral if it will stretch a lot
- Put high-contrast ornamentation in corners and fixed edges

If the game pipeline supports native 9-slice metadata, keep that metadata adjacent to the SVG source so the art and layout intent stay synchronized.

## 5. Sprite atlases

SVG is not usually the atlas format itself; it is the source for atlas entries.

### When to atlas

Use atlases when you have:

- Many small UI icons
- Numerous badge variants
- State changes that reuse a common visual style
- Performance-sensitive UI that should minimize texture switches

### Atlas generation flow

1. Rasterize each SVG source into a fixed-size PNG
2. Pack the PNGs into a sprite atlas
3. Generate a metadata file with UV rectangles, pivot points, and any nine-slice or trimming data
4. Consume the atlas and metadata in the UI/runtime loader

### Atlas hygiene

- Keep source SVG names stable
- Avoid translucent padding unless the packer expects it
- Use consistent export resolution for a family of icons
- Record the original source and intended display size in metadata

## 6. Proposed DINOForge workflow

The intended workflow should be simple and reproducible:

1. The LLM emits SVG into `packs/<mod>/assets/svg/`
2. A build step validates the SVG for basic hygiene and rasterizes it into `packs/<mod>/assets/ui/`
3. The build step generates any sidecar metadata for 9-slice regions, atlas UVs, or icon naming
4. The pack compiler consumes the rasterized outputs, not the raw SVG, for game deployment

Recommended directory responsibilities:

- `packs/<mod>/assets/svg/` - source SVG authored by the LLM or edited by humans
- `packs/<mod>/assets/ui/` - generated PNGs and derived UI metadata
- `packs/<mod>/assets/ui/atlases/` - packed atlas textures and atlas manifests
- `packs/<mod>/assets/ui/slices/` - optional 9-slice metadata if not embedded elsewhere

### Build step responsibilities

The build step should:

- Validate that every SVG has a `viewBox`
- Normalize or reject obviously unsafe or malformed XML
- Rasterize to the required output sizes
- Generate deterministic filenames
- Preserve source-to-output traceability in metadata

### Minimal implementation contract

The orchestrator can generate SVG directly, but the pipeline should not depend on perfect SVG authoring from the model.

The build step should therefore be defensive:

- Reject missing `viewBox`
- Reject external references unless explicitly allowed
- Warn on filters, embedded rasters, or unsupported features
- Produce fallbacks for assets that fail rasterization

## 7. Recommended operating model

Use LLM-SVG as the first draft for:

- Symbols
- Insignia
- Frame language
- Icon families
- Simple HUD graphics

Use human review or a subsequent vector cleanup pass for assets that ship broadly or appear in multiple UI surfaces.

Use licensed source material or dedicated generative image tools for:

- Character portraits
- Painted scenes
- Organic textures
- Anything that should read as illustrative rather than diagrammatic

## 8. Practical summary

The pipeline works best when SVG is treated as structured source code:

- Clear `viewBox`
- Simple, reusable geometry
- Stable `defs`
- Deterministic raster export
- Explicit 9-slice metadata
- Atlas generation from raster outputs

That approach makes LLM-generated SVG useful as a scalable front end to a real game-art build chain instead of a novelty output.

## References

- Inkscape command-line export docs: https://wiki.inkscape.org/wiki/Using_the_Command_Line
- resvg project: https://github.com/RazrFalcon/resvg
- ImageMagick SVG input/raster conversion docs: https://imagemagick.org/script/formats.php
- MDN SVG reference: https://developer.mozilla.org/en-US/docs/Web/SVG
