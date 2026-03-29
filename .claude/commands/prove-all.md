# /prove-all

Run the complete DINOForge evidence pipeline: game capture → TTS → Remotion renders →
Playwright Desktop Companion recording → VHS CLI recordings → bundle into docs/proof/ → commit.

This command produces a `docs/proof/latest/` directory with:
- VLM-confirmed game screenshots (Mods button, F9 overlay, F10 menu)
- Raw + rendered MP4 clips for each feature
- Playwright video of Desktop Companion UI flows
- VHS recordings of CLI tool usage
- `proof_report.md` with pass/fail for every check
- JSON test results summary
- The `docs/proof/index.md` page updated with embedded videos

After running, visiting the docs site shows full visual proof without needing to open the game.

## Prerequisites

- ffmpeg at `C:\program files\imagemagick-7.1.0-q16-hdri\ffmpeg.exe`
- Python 3 + edge-tts: `pip install edge-tts`
- Node.js 18+ with Remotion deps installed: `cd scripts/video && npm install`
- Game at `G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option\`
- VHS (charm): `winget install charmbracelet.vhs` or `choco install vhs`
- Playwright: `cd scripts/companion-playwright && npm install` (if Desktop Companion test exists)

---

## Orchestration

Run ALL steps from repo root. Each step must succeed before proceeding to the next.

---

### Phase 1: Game capture + interleaved VLM validation

Run the capture script, then **immediately validate each feature before moving to the next**.
The game stays running throughout. Each VLM screenshot is taken with the game in the correct UI state.

```powershell
powershell -ExecutionPolicy Bypass -File scripts/game/capture-feature-clips.ps1
```

The script handles: kill → launch → boot detection (Stage A+B) → record all three clips.
After the script exits (game still running), perform the interleaved VLM sequence below.

#### Phase 1a. Validate Mods clip (game still on main menu)
1. Call `game_analyze_screen` — game is on main menu with Mods button visible
2. Check VLM response for "Mods" text in button/menu area
3. Save screenshot to `$env:TEMP\DINOForge\capture\validate_mods.png`
4. **On failure**: press Escape to ensure main menu focus, wait 3s, retry `game_analyze_screen` once
5. **On second failure**: delete `raw_mods.mp4`, write `confirmed: false`, stop with exit 1
6. Write `{ "feature": "mods", "confirmed": true, "vlm_response": "...", "timestamp": "..." }`

#### Phase 1b. Validate F9 clip (re-open overlay for fresh screenshot)
1. Call `game_input` to press F9 — reopens the debug overlay
2. Wait 2s for overlay to render
3. Call `game_analyze_screen`
4. Check for debug panel / overlay with entity counts or runtime stats
5. Save to `$env:TEMP\DINOForge\capture\validate_f9.png`
6. **On failure**: re-press F9, wait 3s, retry once
7. **On second failure**: delete `raw_f9.mp4`, write `confirmed: false`, stop with exit 1
8. Write `{ "feature": "f9", "confirmed": true, "vlm_response": "...", "timestamp": "..." }`
9. Press F9 again to close overlay before next step

#### Phase 1c. Validate F10 clip (re-open mod menu for fresh screenshot)
1. Call `game_input` to press F10 — opens the mod menu
2. Wait 2s for menu to render
3. Call `game_analyze_screen`
4. Check for mod menu / pack browser panel
5. Save to `$env:TEMP\DINOForge\capture\validate_f10.png`
6. **On failure**: press F10 to close, wait 1s, re-press F10, wait 2s, retry once
7. **On second failure**: delete `raw_f10.mp4`, write `confirmed: false`, stop with exit 1
8. Write `{ "feature": "f10", "confirmed": true, "vlm_response": "...", "timestamp": "..." }`

**Temporal consistency**: The game window is open and in the correct UI state when `game_analyze_screen` is called for each feature. The screenshot is a live GDI capture — not extracted from the clip file. This guarantees the VLM sees the actual rendered UI, regardless of ffmpeg compression.

Write `$env:TEMP\DINOForge\capture\validate_report.json`:
```json
[
  { "feature": "mods", "confirmed": true,  "vlm_response": "Mods button visible in upper-left menu area", "timestamp": "..." },
  { "feature": "f9",   "confirmed": true,  "vlm_response": "Debug overlay panel visible with entity count stats", "timestamp": "..." },
  { "feature": "f10",  "confirmed": true,  "vlm_response": "Mod menu panel open showing pack browser", "timestamp": "..." }
]
```

If any `confirmed` is `false`: exit 1, do not proceed to Phase 2.

---

### Phase 2: Neural TTS

```powershell
python scripts/video/generate_tts.py `
  --spec scripts/video/vo_spec.json `
  --out "$env:TEMP\DINOForge\tts"
```

Validate all 5 files exist and are > 10KB:
- `$env:TEMP\DINOForge\tts\intro.mp3`
- `$env:TEMP\DINOForge\tts\mods.mp3`
- `$env:TEMP\DINOForge\tts\f9.mp3`
- `$env:TEMP\DINOForge\tts\f10.mp3`
- `$env:TEMP\DINOForge\tts\outro.mp3`

If any file is missing or < 10KB: exit 1.

---

### Phase 3: Remotion render

Set environment variables (forward slashes required for Node.js file paths):

```powershell
$cap = ($env:TEMP + "\DINOForge\capture") -replace '\\', '/'
$tts = ($env:TEMP + "\DINOForge\tts")     -replace '\\', '/'

$env:RAW_MODS_PATH  = "$cap/raw_mods.mp4"
$env:RAW_F9_PATH    = "$cap/raw_f9.mp4"
$env:RAW_F10_PATH   = "$cap/raw_f10.mp4"
$env:TTS_INTRO_PATH = "$tts/intro.mp3"
$env:TTS_MODS_PATH  = "$tts/mods.mp3"
$env:TTS_F9_PATH    = "$tts/f9.mp3"
$env:TTS_F10_PATH   = "$tts/f10.mp3"
$env:TTS_OUTRO_PATH = "$tts/outro.mp3"
```

```powershell
Push-Location scripts/video
npx remotion render src/index.tsx ModsButtonFeature --output out/mods_feature.mp4
npx remotion render src/index.tsx F9OverlayFeature  --output out/f9_feature.mp4
npx remotion render src/index.tsx F10MenuFeature    --output out/f10_feature.mp4
npx remotion render src/index.tsx DINOForgeReel     --output out/dinoforge_reel.mp4
Pop-Location
```

Error handling:
- If a render exits non-zero: log the error, continue remaining renders
- Mark failed renders as `render_failed: true` in the bundle report
- Minimum requirement: `scripts/video/out/dinoforge_reel.mp4` must exist and be > 100KB

---

### Phase 4: Playwright — Desktop Companion recording (optional)

If `scripts/companion-playwright/` exists with tests:

```powershell
Push-Location scripts/companion-playwright
npx playwright test --reporter=html --video=on 2>&1
if ($LASTEXITCODE -eq 0) {
    $ts = Get-Date -Format "yyyyMMdd_HHmmss"
    Get-ChildItem test-results -Filter "*.webm" -Recurse | ForEach-Object {
        Copy-Item $_.FullName "../../docs/proof/latest/companion_ui_$ts.webm"
    }
}
Pop-Location
```

**On failure or missing directory**: log warning, continue to Phase 5.

---

### Phase 5: VHS — CLI recordings

If `scripts/vhs/` tape files exist:

```powershell
if (Test-Path scripts/vhs) {
    Get-ChildItem scripts/vhs -Filter "*.tape" | ForEach-Object {
        $name = $_.BaseName
        $output = "docs/proof/latest/${name}.gif"
        vhs scripts/vhs/$($_.Name) -o $output
    }
}
```

**On VHS not found or no tape files**: log warning, continue.

---

### Phase 6: Bundle evidence

```powershell
$ts     = Get-Date -Format "yyyyMMdd_HHmmss"
$bundle = "docs/proof/latest"
New-Item -ItemType Directory -Force -Path $bundle | Out-Null

# Phase 1: raw clips + validation artifacts
@("raw_mods.mp4","raw_f9.mp4","raw_f10.mp4",
  "validate_mods.png","validate_f9.png","validate_f10.png",
  "validate_report.json") | ForEach-Object {
    $src = "$env:TEMP\DINOForge\capture\$_"
    if (Test-Path $src) { Copy-Item $src "$bundle\" -Force }
}

# Phase 3: rendered videos
@("mods_feature.mp4","f9_feature.mp4","f10_feature.mp4","dinoforge_reel.mp4") | ForEach-Object {
    $src = "scripts/video/out/$_"
    if (Test-Path $src) { Copy-Item $src "$bundle\" -Force }
}

# Create bundle metadata
$metadata = @{
    "timestamp" = $ts
    "generated_at" = (Get-Date -Format "o")
    "pipeline_version" = "2.0"
}
$metadata | ConvertTo-Json | Out-File "$bundle/bundle_metadata.json"
```

Generate `docs/proof/latest/proof_report.md` from `validate_report.json` data and render results using the template:

```markdown
# DINOForge Feature Proof — <timestamp>

## Validation Results
| Feature | VLM Confirmed | Screenshot | Raw Clip |
|---|---|---|---|
| Mods Button | ✓ / ✗ | validate_mods.png | raw_mods.mp4 |
| F9 Debug Overlay | ✓ / ✗ | validate_f9.png | raw_f9.mp4 |
| F10 Mod Menu | ✓ / ✗ | validate_f10.png | raw_f10.mp4 |

## Rendered Videos
| Output | Status | File |
|---|---|---|
| Mods feature clip | OK / FAILED | mods_feature.mp4 |
| F9 feature clip | OK / FAILED | f9_feature.mp4 |
| F10 feature clip | OK / FAILED | f10_feature.mp4 |
| Compilation reel | OK / FAILED | dinoforge_reel.mp4 |

## Methodology
- Window capture: gdigrab title="Diplomacy is Not an Option" (NOT desktop)
- Key injection: Win32 SendInput (no window focus required)
- VLM validation: game_analyze_screen MCP (live GDI screenshot while UI is active)
- TTS: edge-tts en-US-AriaNeural (free neural, Microsoft Edge voice)
- Composition: Remotion v4, spring-physics callout boxes
- Desktop Companion UI: Playwright integration tests with video capture
- CLI recordings: VHS charm terminal session recording
```

---

### Phase 7: Update docs/proof/index.md

Update the `docs/proof/index.md` VitePress page with the latest bundle metadata and video embeds.
The page auto-detects the latest timestamp from `docs/proof/latest/bundle_metadata.json`.

---

### Phase 8: Git commit

```powershell
Push-Location C:\Users\koosh\Dino
git add docs/proof/ scripts/vhs/
$ts = Get-Date -Format "yyyyMMdd_HHmmss"
git commit -m "proof(all): auto-generated evidence bundle $ts"
Pop-Location
```

On commit failure (nothing to commit): warn and continue — the bundle exists on disk even if git is clean.

---

## VLM Model Selection

For `game_analyze_screen` VLM calls, use the weakest capable model (fastest + lowest cost):
1. Codex Spark 5.3 (if image input supported — preferred)
2. Codex 5.4 mini (fallback)
3. claude-haiku-4-5-20251001 (final fallback, fast mode OK)

`game_analyze_screen` handles the actual screenshot capture; the model only needs to describe what it sees.

---

## Troubleshooting

**Phase 1 fails**: Game capture script returned non-zero. Check:
- Game is installed at `G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option\`
- DINOForge DLL is deployed and mod loads (F10 opens menu, no error overlay)
- ffmpeg can capture window (test with `game_screenshot` MCP tool)

**Phase 2 fails**: TTS generation failed. Check:
- Python 3 is installed: `python --version`
- edge-tts is installed: `pip show edge-tts`
- `scripts/video/vo_spec.json` exists and is valid JSON

**Phase 3 fails**: Remotion render crashed. Check:
- Node.js 18+: `node --version`
- Remotion CLI installed: `cd scripts/video && npm list remotion`
- All input files (raw clips, TTS MP3s) exist and are valid

**Phase 8 fails**: Git commit failed. Check:
- `.git/` exists
- No other commits in progress
- Disk space available

---

## Success Criteria

✓ All phases completed without stopping
✓ `docs/proof/latest/` directory contains at least:
  - `raw_mods.mp4`, `raw_f9.mp4`, `raw_f10.mp4` (raw clips)
  - `validate_*.png` (3 screenshots)
  - `validate_report.json` with all `confirmed: true`
  - At least 1 rendered MP4 (e.g., `dinoforge_reel.mp4`)
  - `proof_report.md` with pass/fail summary
  - `bundle_metadata.json` with timestamp
✓ `docs/proof/index.md` page updated and renders without 404s
✓ Git commit created with evidence bundle
✓ Docs site (when built) displays all videos inline without broken links
