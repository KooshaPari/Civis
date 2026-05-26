# DINOForge UI Assets — Kenney CC0 Sprites

This directory holds Kenney.nl CC0 public-domain UI sprite packs used by `UiAssets.cs`
to skin the DINOForge UGUI overlay at runtime.

All packs are released under **Creative Commons CC0** (public domain, no attribution required).
See https://kenney.nl/assets for full details.

## Required Packs

Download each ZIP and **extract its contents into the matching subdirectory**:

| Pack | Subdirectory | Download URL | Files |
|------|-------------|--------------|-------|
| UI Pack (v2) | `kenney/ui-pack/` | https://kenney.nl/media/pages/assets/ui-pack/af874291da-1718203990/kenney_ui-pack.zip | 430 |
| UI Pack - Sci-Fi / Space Expansion | `kenney/ui-pack-sci-fi/` | https://kenney.nl/media/pages/assets/ui-pack-sci-fi/d83f166279-1724181109/kenney_ui-pack-space-expansion.zip | 130 |
| Fantasy UI Borders | `kenney/fantasy-ui-borders/` | https://kenney.nl/media/pages/assets/fantasy-ui-borders/458f216b46-1701602367/kenney_fantasy-ui-borders.zip | 140 |
| UI Pack - Adventure | `kenney/ui-pack-adventure/` | https://kenney.nl/media/pages/assets/ui-pack-adventure/a9a6ec7d59-1723597274/kenney_ui-pack-adventure.zip | 130 |

## Quick Setup (PowerShell)

```powershell
$packs = @(
    @{ url = "https://kenney.nl/media/pages/assets/ui-pack/af874291da-1718203990/kenney_ui-pack.zip";                           dest = "kenney/ui-pack" },
    @{ url = "https://kenney.nl/media/pages/assets/ui-pack-sci-fi/d83f166279-1724181109/kenney_ui-pack-space-expansion.zip";    dest = "kenney/ui-pack-sci-fi" },
    @{ url = "https://kenney.nl/media/pages/assets/fantasy-ui-borders/458f216b46-1701602367/kenney_fantasy-ui-borders.zip";     dest = "kenney/fantasy-ui-borders" },
    @{ url = "https://kenney.nl/media/pages/assets/ui-pack-adventure/a9a6ec7d59-1723597274/kenney_ui-pack-adventure.zip";       dest = "kenney/ui-pack-adventure" }
)
foreach ($p in $packs) {
    $zip = "$env:TEMP\kenney_temp.zip"
    Invoke-WebRequest $p.url -OutFile $zip
    Expand-Archive $zip -DestinationPath $p.dest -Force
    Remove-Item $zip
}
```

## Quick Setup (Bash/curl)

```bash
BASE="src/Runtime/UI/Assets"
declare -A PACKS=(
  ["kenney/ui-pack"]="https://kenney.nl/media/pages/assets/ui-pack/af874291da-1718203990/kenney_ui-pack.zip"
  ["kenney/ui-pack-sci-fi"]="https://kenney.nl/media/pages/assets/ui-pack-sci-fi/d83f166279-1724181109/kenney_ui-pack-space-expansion.zip"
  ["kenney/fantasy-ui-borders"]="https://kenney.nl/media/pages/assets/fantasy-ui-borders/458f216b46-1701602367/kenney_fantasy-ui-borders.zip"
  ["kenney/ui-pack-adventure"]="https://kenney.nl/media/pages/assets/ui-pack-adventure/a9a6ec7d59-1723597274/kenney_ui-pack-adventure.zip"
)
for dir in "${!PACKS[@]}"; do
  curl -L "${PACKS[$dir]}" -o /tmp/kenney_temp.zip
  unzip -o /tmp/kenney_temp.zip -d "$BASE/$dir"
done
```

## Deployment

At build time (when `GameInstalled=true`), an MSBuild target copies all files from
`kenney/*/` into `$(BepInExDir)/plugins/dinoforge-ui-assets/` alongside the DLL.

At runtime, `UiAssets.Initialize(pluginDir)` points the loader at this deployed directory.
Sprites are loaded on demand; if a file is missing the system falls back silently to flat colors.

## Expected Sprite Names used by UiAssets.cs

The loader looks for these paths relative to the pack root.
Exact file names depend on the Kenney pack version — inspect `UiAssets.MissingFiles` in-game
(logged at startup) to see which sprites failed to load.

### ui-pack/
- `PNG/panel_background.png` — 9-sliceable panel background
- `PNG/button_rectangleFlat.png` — button normal state
- `PNG/button_rectangleDepressed.png` — button pressed state
- `PNG/checkbox_unchecked.png` / `checkbox_checked.png`
- `PNG/slide_horizontal_color_n.png` — scrollbar track
- `PNG/slide_hangle.png` — scrollbar handle

### ui-pack-sci-fi/
- `PNG/panel_metalCorner.png` — dark sci-fi panel corner
- `PNG/panel_screws.png` — panel with screw corners

### fantasy-ui-borders/
- `PNG/panel_ornate.png` — decorative bordered panel

### ui-pack-adventure/
- `PNG/panel_brownCornersSmall.png` — compact panel

## License

All Kenney assets are CC0 (https://creativecommons.org/publicdomain/zero/1.0/).
No attribution is required, but Kenney appreciates a mention.
