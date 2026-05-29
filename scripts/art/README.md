# SVG Rasterization

Use these scripts to rasterize art sources from `packs/<mod>/assets/svg/` into `packs/<mod>/assets/ui/`.

## Inputs and outputs

- Input: `packs/<mod>/assets/svg/`
- Output: `packs/<mod>/assets/ui/`
- Supported sources: recursive `*.svg`
- Output format: `*.png`
- Transparency is preserved by the selected rasterizer

## Tool priority

The scripts detect a rasterizer in this order:

1. Inkscape
2. `resvg`
3. `rsvg-convert`
4. ImageMagick `magick`

If none are available, the scripts print install hints for `winget`, `choco`, and `apt`.

## PowerShell

```powershell
scripts\art\rasterize-svg.ps1 packs\<mod>\assets\svg packs\<mod>\assets\ui 16,32,48,256
```

## Bash

```bash
scripts/art/rasterize-svg.sh packs/<mod>/assets/svg packs/<mod>/assets/ui 16,32,48,256
```

## Output naming

- Single size: `icon.png`
- Multiple sizes: `icon-16.png`, `icon-32.png`, `icon-48.png`, `icon-256.png`

## Notes

- The scripts recurse through subdirectories and mirror the source folder structure under the output directory.
- For icon families, use the same size list across a whole batch so the outputs stay deterministic.
