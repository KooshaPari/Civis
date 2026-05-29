# OSS / Free Headless Art Tooling for Windows

This note focuses on CLI/scriptable, fully OSS or free tools that are practical in a Windows automation pipeline for DINOForge art production.

Scope:
- SVG authoring and rasterization
- raster compositing and batch conversion
- scripted 2D painting and batch processing
- headless 3D rendering and GLB workflows
- image-sequence and loading-video generation

## Quick Take

If the goal is a fully OSS, Adobe-free pipeline that works well from scripts on Windows, the strongest stack is:

1. `Inkscape` for SVG authoring and SVG-to-PNG export
2. `resvg` or `rsvg-convert` for deterministic SVG rasterization in batch jobs
3. `ImageMagick` for compositing, masking, cropping, resizing, color transforms, and sprite-sheet assembly
4. `Blender --background` for 3D renders, GLB import/export, and texture baking
5. `ffmpeg` for loading screens, animated previews, and image-sequence to video encoding
6. `GIMP` for scripted raster cleanup when you need more than ImageMagick but less than a full paint workflow

`Krita` is the best OSS choice when the job needs artist-friendly brush workflows plus scripting, but its headless/batch story is weaker than ImageMagick/GIMP/Blender.

## Tool-by-Tool Notes

### Inkscape

Best for:
- logos
- UI icons
- 9-slice panel source art
- SVG atlases
- any vector-first asset that later needs PNG output

Headless invocation:
```powershell
inkscape input.svg --export-type=png --export-filename=output.png
inkscape input.svg --export-type=png --export-area-page --export-width=2048 --export-filename=out.png
inkscape input.svg --actions="select-all;object-to-path;export-filename:out.png;export-do"
```

Strengths:
- Excellent SVG editor with reliable CLI export
- Good fit for iconography, UI frames, and scalable symbols
- Can be used in batch jobs to convert curated SVG sources into fixed-size PNGs
- Supports more than simple export; action strings can automate common edit/export steps

What it suits:
- Logos: excellent
- Backgrounds: good for vector backgrounds or layered decorative elements
- Sprite sheets: useful for vector source assets, not for pixel art assembly
- 9-slice panels: excellent for source creation, borders, corners, and scalable frames
- Loading videos: indirect, as a source stage rather than final video tool

Limitations:
- Not the best choice for photoreal raster manipulation
- SVG feature support is good, but complex filter behavior can differ across renderers

### ImageMagick

Best for:
- batch conversion
- compositing
- alpha/matte operations
- resize/crop/trim
- sprite-sheet layout
- variant generation

Headless invocation:
```powershell
magick input.png -resize 1024x1024 output.png
magick a.png b.png -gravity center -composite out.png
magick montage frame*.png -tile 8x -geometry 128x128+2+2 spritesheet.png
magick input.png -trim +repage output.png
```

Strengths:
- Very strong batch automation surface
- Huge operator set for compositing and conversion
- Good glue tool for pipeline steps between SVG, raster, and video
- Easy to script from PowerShell, CMD, or CI

What it suits:
- Logos: good for batch variants and packaging
- Backgrounds: good for palette/format transforms and compositing
- Sprite sheets: excellent for sheet assembly and frame packing
- 9-slice panels: excellent for border slicing, padding, and output variants
- Loading videos: good for still-frame prep before ffmpeg

Limitations:
- Not a painting tool
- Complex node-like workflows can become hard to read if not wrapped in scripts

### Blender

Best for:
- 3D renders
- turntable shots
- terrain/prop renders
- GLB import/export and cleanup
- texture baking
- procedural scene generation

Headless invocation:
```powershell
blender --background scene.blend --render-output //renders/frame_#### --render-frame 1
blender --background --python bake_textures.py -- scene.blend
blender --background --python export_glb.py -- input.glb output.glb
```

Strengths:
- Best fully OSS option here for 3D asset rendering
- Strong scriptability through Python
- Can render consistent image sequences for videos and in-game previews
- Supports geometry/texture workflows that 2D tools cannot cover

What it suits:
- Logos: only if the logo is part of a 3D treatment
- Backgrounds: excellent for rendered scene backdrops
- Sprite sheets: useful for render-to-sheet pipelines, especially 3D props or VFX frames
- 9-slice panels: not a primary fit
- Loading videos: excellent for animated loops or cinematic loading screens

Limitations:
- Steeper setup cost than 2D tools
- Headless rendering requires more scene discipline and scripted pipeline management

### GIMP

Best for:
- scripted raster cleanup
- batch touch-up
- layer-based image fixes
- format conversion when you need layer semantics

Headless invocation:
```powershell
gimp -i -b '(gimp-quit 0)'
gimp -i -b '(python-fu-my-script RUN-NONINTERACTIVE "in.png" "out.png")' -b '(gimp-quit 0)'
gimp -i --batch-interpreter=plug-in-script-fu-eval -b '(script-fu-some-procedure "in.png" "out.png")' -b '(gimp-quit 0)'
```

Strengths:
- Better than ImageMagick when the job needs layer-aware editing logic
- Script-Fu and Python-based batch workflows are available
- Useful for one-off or repeatable raster cleanup that is awkward in pure CLI image operators

What it suits:
- Logos: good for raster cleanup and export variants
- Backgrounds: good for paint-over cleanup and layered corrections
- Sprite sheets: okay for hand-edited source frames, not ideal as the packing engine
- 9-slice panels: good for raster touch-up and border corrections
- Loading videos: weak as a video tool, but useful for generating source frames

Limitations:
- Batch automation is older and less ergonomic than ImageMagick or Blender Python
- GIMP is not a substitute for video encoding or 3D rendering

### ffmpeg

Best for:
- loading videos
- looping previews
- image sequences to video
- GIF/MP4/WebM export
- audio muxing for preview assets

Headless invocation:
```powershell
ffmpeg -framerate 30 -i frame_%04d.png -c:v libx264 -pix_fmt yuv420p loading.mp4
ffmpeg -loop 1 -i splash.png -t 5 -c:v libx264 -pix_fmt yuv420p splash.mp4
ffmpeg -i input.webm -vf "scale=1920:-2" output.mp4
```

Strengths:
- The standard OSS command-line video tool
- Excellent for converting still assets into videos and animated previews
- Works well as the final packaging stage after ImageMagick or Blender output

What it suits:
- Logos: not a creation tool, but good for intro/outro clips
- Backgrounds: good for animated background loops
- Sprite sheets: not a direct fit, but can preview sheet animation as video
- 9-slice panels: not a direct fit
- Loading videos: excellent

Limitations:
- Not an image editor
- Source quality depends on the frame pipeline feeding it

### Krita

Best for:
- painted textures
- concept art
- hand-drawn backgrounds
- raster touch-up with artist-friendly brushes
- scripted export pipelines when you want painting plus automation

Headless / scripting reality:
- Krita supports Python scripting and plugin-based automation
- It is strong as an interactive painting environment, but it is not as CLI-native as ImageMagick, ffmpeg, or Blender
- In practice, Krita is best used for scripted export/processing or as a human-in-the-loop paint stage, not as the core batch engine

Representative invocation patterns:
```powershell
krita.exe
krita.exe --nosplash --template
```

Strengths:
- Excellent brush engine and paint workflow
- Very good for texture painting and hand-authored art
- Python extensibility makes repetitive export tasks feasible

What it suits:
- Logos: only if hand-painted or texture-styled
- Backgrounds: excellent for painted backgrounds
- Sprite sheets: okay for hand-painted frame sources
- 9-slice panels: good for stylized painted UI panels
- Loading videos: weak directly, but can generate frame assets used elsewhere

Limitations:
- Less mature as a headless batch processor than the other tools in this list
- Not the first choice for fully automated pipeline orchestration

### resvg / rsvg-convert

Best for:
- deterministic SVG rasterization
- build pipelines that need consistent SVG-to-PNG conversion
- icon and UI export jobs

Headless invocation:
```powershell
resvg input.svg output.png
rsvg-convert -w 2048 -h 2048 input.svg -o output.png
rsvg-convert --format=png input.svg > output.png
```

Strengths:
- Fast SVG rasterization
- Good for production batch conversion
- `resvg` is attractive when you want a portable CLI with no external system-library dependency chain
- `rsvg-convert` is a stable GNOME tooling choice for scripted SVG rasterization

What it suits:
- Logos: excellent
- Backgrounds: good for vector backgrounds and SVG-derived UI art
- Sprite sheets: useful as a source rasterizer, not a packer
- 9-slice panels: excellent for SVG source conversion
- Loading videos: indirect, as a frame source

Limitations:
- Not an editor
- SVG animation/interactivity is not the goal; raster output is

## Recommended Fully OSS Pipeline

If I were standardizing a DINOForge art pipeline with no Adobe dependency, I would use this stack:

1. `Inkscape` for source SVG creation and reusable UI/vector pieces
2. `resvg` for fast, reproducible SVG rasterization in build scripts
3. `ImageMagick` for batch transforms, trims, masks, composites, and sprite-sheet packing
4. `Krita` for painted textures and any artist-driven raster work
5. `Blender` for 3D assets, lighting, turntables, renders, and texture baking
6. `ffmpeg` for final animation/video assembly

Why this is the best SOTA OSS mix:
- It covers all core asset classes without duplicating responsibility
- Each tool is used where it is strongest
- The pipeline is scriptable end-to-end on Windows
- It keeps SVG, raster, 3D, and video responsibilities separate instead of forcing one tool to do everything
- `resvg` plus `ImageMagick` is especially strong for deterministic build outputs

Practical pipeline split:
- Source art: Inkscape or Krita
- Raster conversion: resvg or rsvg-convert
- Composition and variants: ImageMagick
- 3D/animated renders: Blender
- Final preview videos: ffmpeg

For most DINOForge needs:
- UI icons, logos, and 9-slice panels should start in SVG and end through `resvg` or Inkscape export
- Sprite sheets should usually be assembled with ImageMagick after the source frames are prepared
- Loading animations and preview clips should be produced by Blender or still-frame pipelines plus ffmpeg

## Suggested Use Map

| Task | Best Tool | Secondary Tool |
|---|---|---|
| Logos | Inkscape | resvg, ImageMagick |
| Backgrounds | Krita | Blender, ImageMagick |
| Sprite sheets | ImageMagick | Krita, ffmpeg preview |
| 9-slice panels | Inkscape | resvg, ImageMagick |
| Loading videos | Blender | ffmpeg |
| SVG rasterization | resvg / rsvg-convert | Inkscape |
| Batch compositing | ImageMagick | GIMP |

## Notes for Windows Automation

- Prefer explicit absolute paths in scripts so tool discovery is stable across CI and local shells.
- Prefer `resvg` when you want the simplest portable SVG-to-PNG path.
- Use `Inkscape` when the SVG itself needs editor logic or a richer export action sequence.
- Use `ImageMagick` for everything that looks like "take N raster files and derive M variants".
- Use `Blender` only when the asset actually needs 3D or rendering logic.

## Sources

- Inkscape manual and CLI docs: https://inkscape.org/doc/inkscape-man.html
- Inkscape documentation portal: https://inkscape.org/learn/
- ImageMagick command-line tools: https://imagemagick.org/script/command-line-tools.php
- ImageMagick usage overview: https://imagemagick.org/script/usage.php
- Blender manual, command line: https://docs.blender.org/manual/en/latest/advanced/command_line/index.html
- Blender manual, render options: https://docs.blender.org/manual/en/latest/advanced/command_line/render.html
- GIMP documentation portal: https://docs.gimp.org/
- GIMP batch mode reference: https://docs.gimp.org/en/gimp-batch-mode.html
- Krita manual: https://docs.krita.org/
- Krita scripting docs: https://docs.krita.org/en/reference_manual/scripting.html
- ffmpeg documentation: https://ffmpeg.org/documentation.html
- ffmpeg formats/filters docs: https://ffmpeg.org/ffmpeg.html
- resvg project: https://github.com/linebender/resvg
- librsvg / rsvg-convert docs: https://gnome.pages.gitlab.gnome.org/librsvg/
- librsvg product overview: https://gnome.pages.gitlab.gnome.org/librsvg/devel-docs/product.html
