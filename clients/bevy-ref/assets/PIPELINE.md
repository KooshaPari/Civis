# Civis Bevy asset pipeline

End-to-end art path from DCC tools into `clients/bevy-ref/assets/`. Bevy loads
**PNG**, **glTF/GLB**, **OGG**, and **KTX2** — not SVG or native `.blend`/`.fbx`.

## Blender → glTF

1. Model in Blender (meters, Y-up export as glTF convention).
2. Apply transforms; triangulate only where needed for game LOD.
3. Export **glTF Separate** or **GLB** to `assets/models/<kit>/`.
4. Name meshes/materials with stable prefixes (`bld_`, `chr_`, `prop_`).
5. Validate in `civ-bevy-ref` with `GltfModelsPlugin` / scene spawn.

## Adobe / Figma → SVG → PNG

1. Author UI vectors in Figma or Illustrator; export SVG to `assets/ui/`.
2. Rasterise for Bevy (see `assets/ui/README.md` and `Tools/asset-pipeline`).
3. Commit **both** SVG (source) and PNG (runtime) when PNG is wired in code.
4. Main-menu wiring checks for `ui/logo.png`, `ui/wordmark.png`, `ui/title-bg.png`.

```powershell
# From repo root
pwsh Tools/asset-pipeline/Convert-SvgToPng.ps1 -SourceDir clients/bevy-ref/assets/ui
```

## Substance → PBR

1. Author materials in Substance Painter/Designer.
2. Export **ORM** (occlusion-roughness-metallic) + base-color (+ normal) as PNG.
3. Place under `assets/textures/<biome>/` per `assets/textures/README.md`.
4. Reference in `materials` / `pbr_materials` Rust modules.

## Autodesk FBX → glTF

1. Import FBX into Blender (or use Autodesk FBX Converter → OBJ interim).
2. Clean scale/axis; re-export glTF as above.
3. Do **not** commit FBX to runtime bundles unless needed for source archive.

## Audio

- **OGG Vorbis** for SFX and music under `assets/audio/`.
- See `assets/audio/README.md` for cue naming.

## Credits

- List third-party assets in `assets/CREDITS.md` (author, license, URL).
- In-engine UI kit SVGs: Civis art direction (see `assets/ui/README.md` palette).

## CI / agent checklist

| Step | Command / path |
|------|----------------|
| SVG batch PNG | `just asset-pipeline` or `Convert-SvgToPng.ps1` |
| glTF sanity | open in Bevy standalone with `models` feature |
| Missing PNG | main menu falls back to text "Civis" until rasterised |
