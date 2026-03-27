# SPEC: Prove-Features Video Pipeline v2

**Status**: Approved
**Version**: 2.0
**Date**: 2026-03-27
**Replaces**: SPEC-006 / `scripts/game/prove-features-video.ps1`
**Scope**: `/prove-features` command — full replacement of broken v1 pipeline

---

## Problem Statement

The v1 pipeline produced invalid proof artifacts:
1. **Wrong window captured** — gdigrab used `-i desktop` (foreground capture), recording the Claude Code terminal instead of the game
2. **Robotic TTS** — edge-tts argument-splitting bug caused silent fallback to SAPI (90s-era voice)
3. **No validation** — screenshots and video frames were never inspected; a blank or wrong capture counted as "proof"
4. **No animation quality** — static drawtext overlays, no spring physics, no per-feature scenes

---

## Architecture

Three independent phases, each with a single responsibility:

```
Phase 1: Capture       Phase 2: Voice         Phase 3: Compose
─────────────────      ──────────────────      ─────────────────────────
PowerShell             Python                  Remotion (React → MP4)
  │                      │                       │
  ├─ launch game          ├─ vo_spec.json          ├─ CalloutBox (spring)
  ├─ bootstrap wait       ├─ edge-tts per line     ├─ FeatureScene (3×)
  ├─ gdigrab by title     └─ output MP3s           ├─ DINOForgeReel (38s)
  ├─ SendInput F9/F10                              └─ render → MP4
  └─ VLM validate each
     frame before saving
```

---

## Phase 1: Capture (`scripts/game/capture-feature-clips.ps1`)

### Window targeting
```powershell
# CORRECT — title-based, not desktop
ffmpeg -f gdigrab -framerate 30 -i "title=Diplomacy is Not an Option" ...
# NOT: -i desktop (captures foreground — wrong)
```

### Capture sequence
Three separate clips, one per feature:

| Clip | Action | Duration |
|---|---|---|
| `raw_mods.mp4` | Record main menu with Mods button visible | 6s |
| `raw_f9.mp4` | Press F9 → wait 2s → record overlay | 8s |
| `raw_f10.mp4` | Press F10 → wait 2s → record menu | 8s |

### Key injection
Win32 `SendInput` via inline C# `Add-Type` — works without window focus. NOT `SendKeys` (requires focus, unreliable).

### VLM validation gate

Validation uses a **live screenshot of the game window**, not a frame extracted from the clip. The sequence is:

1. Start `ffmpeg` recording (background process)
2. Inject key press (F9 or F10) via `SendInput`
3. Wait 2s for UI to render
4. Call `game_analyze_screen` MCP tool — this takes a fresh GDI screenshot of the game window at that moment
5. Check VLM response for expected element
6. Wait for `ffmpeg` to finish recording (the clip captures the full event including the UI state)

The game window remains open and in the correct UI state during steps 4–6, so the live screenshot and the clip frames are temporally consistent.

| Clip | Key injected | Expected in VLM screenshot |
|---|---|---|
| `raw_mods.mp4` | *(none — main menu)* | "Mods" text in button/menu area |
| `raw_f9.mp4` | F9 | Debug panel / overlay with DINOForge stats |
| `raw_f10.mp4` | F10 | Mod menu / pack browser panel |

**On validation failure**: retry once (re-inject key, wait 3s, re-screenshot). On second failure: save the failed screenshot + write `{ confirmed: false, error: "<vlm_response>" }` to `validate_report.json`, exit non-zero. The clip file for that feature is deleted. Do NOT include unvalidated clips in the evidence bundle.

### Bootstrap detection
- Boot Stage A: poll `dinoforge_debug.log` for `"Awake completed"` (30s timeout)
- Boot Stage B: poll `LogOutput.log` with `FileShare.ReadWrite` for `"MODS BUTTON INJECTION FULLY SUCCESSFUL"` (720s timeout)

### Outputs
```
$env:TEMP\DINOForge\capture\
  raw_mods.mp4
  raw_f9.mp4
  raw_f10.mp4
  validate_mods.png
  validate_f9.png
  validate_f10.png
  validate_report.json    ← { feature, vlm_response, confirmed, timestamp }
```

---

## Phase 2: Voice (`scripts/video/generate_tts.py`)

### Why a Python file instead of CLI args
The v1 bug: PowerShell `ArgumentList` with array splits text on spaces. Fix: read text from a JSON file, write to temp file, no shell arg quoting issues.

### Interface
All paths are relative to the repo root. Run from repo root:
```bash
python scripts/video/generate_tts.py --spec scripts/video/vo_spec.json --out $env:TEMP\DINOForge\tts\
```

### `vo_spec.json`
```json
[
  { "id": "intro",  "voice": "en-US-AriaNeural", "text": "DINOForge mod platform. Feature demonstration." },
  { "id": "mods",   "voice": "en-US-AriaNeural", "text": "The Mods button was automatically injected into the native main menu in under ten seconds." },
  { "id": "f9",     "voice": "en-US-AriaNeural", "text": "Pressing F9 opens the debug overlay panel, showing live entity counts and runtime stats." },
  { "id": "f10",    "voice": "en-US-AriaNeural", "text": "Pressing F10 opens the mod menu, where packs can be browsed and toggled." },
  { "id": "outro",  "voice": "en-US-AriaNeural", "text": "All three features confirmed working. DINOForge is ready for testing." }
]
```

### Outputs
`$env:TEMP\DINOForge\tts\intro.mp3`, `mods.mp3`, `f9.mp3`, `f10.mp3`, `outro.mp3`

---

## Phase 3: Composition (`scripts/video/` — Remotion project)

### Project structure
```
scripts/video/
  package.json
  remotion.config.ts
  src/
    index.ts
    types.ts
    components/
      CalloutBox.tsx      ← spring-physics scale-in, color-coded
      CaptionBar.tsx      ← permanent bottom strip
      FeatureScene.tsx    ← video + callout + caption + audio
      TitleCard.tsx       ← full-frame text for intro/outro
    compositions/
      ModsButtonFeature.tsx
      F9OverlayFeature.tsx
      F10MenuFeature.tsx
      DINOForgeReel.tsx   ← compilation (38s)
```

### `<CalloutBox>` — spring physics
```tsx
const frame = useCurrentFrame();
const fps = useVideoConfig().fps;
const scale = spring({ frame, fps, config: { damping: 12, stiffness: 180 } });
// scale: 0→1 with natural overshoot (~1.08) then settle at 1.0
// visual: box pops in from 0%, briefly overshoots, settles — not mechanical
```

### Color system
| Feature | Color | Hex |
|---|---|---|
| Mods button | Green | `#34d399` |
| F9 overlay | Yellow | `#fbbf24` |
| F10 menu | Blue | `#60a5fa` |
| Neutral / title | White | `#f8fafc` |

### Compositions

**Individual feature clips** (10s each, 30fps = 300 frames):

Each `FeatureScene` receives a 6–8s raw clip. The clip plays from frame 0 and **freeze-frames on its last frame** for the remaining duration (handled by `<OffthreadVideo lastImageDefaultProps>`). This pads shorter clips to 10s without looping or errors.

- `ModsButtonFeature` — raw_mods.mp4 (6s, freeze last 4s), green `#34d399` callout "Mods Button Injected", mods.mp3 VO
- `F9OverlayFeature` — raw_f9.mp4 (8s, freeze last 2s), yellow `#fbbf24` callout "[ F9 ] Debug Overlay", f9.mp3 VO
- `F10MenuFeature` — raw_f10.mp4 (8s, freeze last 2s), blue `#60a5fa` callout "[ F10 ] Mod Menu", f10.mp3 VO

**Compilation reel** (`DINOForgeReel`, 38s total, 30fps = 1140 frames):

```
0–3s    TitleCard: "DINOForge Mod Platform" — animated text reveal on dark bg
        Audio: intro.mp3 via <Audio src={introMp3} />
3–13s   ModsButtonFeature <Sequence> (green #34d399 callout)
13–23s  F9OverlayFeature <Sequence> (yellow #fbbf24 callout)
23–33s  F10MenuFeature <Sequence> (blue #60a5fa callout)
33–38s  TitleCard outro: all 3 callouts visible simultaneously + "All features confirmed"
        Audio: outro.mp3 via <Audio src={outroMp3} />
```

**TitleCard audio binding**: `TitleCard` accepts an optional `audioSrc?: string` prop. Rendered via `<Audio src={audioSrc} />` when provided.

### Render commands
All render commands run from `scripts/video/` (CWD). Output lands in `scripts/video/out/` via `--output` flag:

```bash
cd scripts/video
npx remotion render src/index.ts ModsButtonFeature --output out/mods_feature.mp4
npx remotion render src/index.ts F9OverlayFeature  --output out/f9_feature.mp4
npx remotion render src/index.ts F10MenuFeature    --output out/f10_feature.mp4
npx remotion render src/index.ts DINOForgeReel     --output out/dinoforge_reel.mp4
```

**Error handling**: if `npx remotion render` exits non-zero, the orchestrator logs the error, skips the copy step for that composition, and sets `render_failed: true` in `validate_report.json`. It does not exit — remaining renders continue. The final `proof_report.md` lists which renders succeeded and which failed.

### Output specs
- Codec: H.264 baseline, `yuv420p`
- Audio: AAC 128k
- Resolution: 1280×800 (match game window)
- FPS: 30
- `faststart` flag: yes

---

## Evidence Bundle

Output to `docs/proof-of-features/<YYYYMMDD_HHmmss>/`:

```
<timestamp>/
  raw_mods.mp4            raw game clip — unedited
  raw_f9.mp4
  raw_f10.mp4
  validate_mods.png       VLM-confirmed screenshot
  validate_f9.png
  validate_f10.png
  validate_report.json    { feature, vlm_response, confirmed: true, timestamp }
  mods_feature.mp4        Remotion-rendered individual clip
  f9_feature.mp4
  f10_feature.mp4
  dinoforge_reel.mp4      full compilation reel
  proof_report.md         human-readable summary linking all artifacts
```

---

## Orchestration (`/prove-features` command)

Updated `.claude/commands/prove-features.md` delegates to:

```
All steps run from the repo root unless noted.

Step 1: powershell -ExecutionPolicy Bypass -File scripts/game/capture-feature-clips.ps1
        → validates: raw_mods.mp4, raw_f9.mp4, raw_f10.mp4 exist in $env:TEMP\DINOForge\capture\
                     validate_report.json exists, all entries confirmed=true
        → on any failure: exit 1, do not proceed

Step 2: python scripts/video/generate_tts.py --spec scripts/video/vo_spec.json --out $env:TEMP\DINOForge\tts\
        → validates: intro.mp3, mods.mp3, f9.mp3, f10.mp3, outro.mp3 exist, each > 10KB
        → on any failure: exit 1, do not proceed

Step 3: cd scripts/video
        npx remotion render src/index.ts ModsButtonFeature --output out/mods_feature.mp4
        npx remotion render src/index.ts F9OverlayFeature  --output out/f9_feature.mp4
        npx remotion render src/index.ts F10MenuFeature    --output out/f10_feature.mp4
        npx remotion render src/index.ts DINOForgeReel     --output out/dinoforge_reel.mp4
        → each render: if exit non-zero, log error, mark render_failed=true in validate_report.json,
          continue remaining renders (partial output is acceptable)
        → validates: at least dinoforge_reel.mp4 exists and > 100KB (reel is the primary deliverable)

Step 4: $ts = Get-Date -Format "yyyyMMdd_HHmmss"
        $bundle = "docs/proof-of-features/$ts"
        Copy all artifacts (raw clips, validate PNGs, validate_report.json, rendered MP4s) to $bundle\
        Generate $bundle\proof_report.md (see template below)
        Start-Process "$bundle\dinoforge_reel.mp4"
```

---

## `proof_report.md` Template

Generated by the orchestrator. Fields populated from `validate_report.json` and render results:

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
- Window capture: gdigrab title="Diplomacy is Not an Option"
- VLM validation: game_analyze_screen MCP (live screenshot at time of feature display)
- TTS: edge-tts en-US-AriaNeural (free, neural)
- Composition: Remotion v4, spring-physics callouts
```

---

## Capture Resolution Normalization

Phase 1 `ffmpeg` commands include `-vf scale=1280:800` to normalize all clips to match Phase 3 output spec. This ensures consistent compositing regardless of the actual game window size:

```powershell
ffmpeg -f gdigrab -framerate 30 -i "title=Diplomacy is Not an Option" `
  -t <duration> -vf scale=1280:800 -vcodec libx264 -preset ultrafast <output.mp4>
```

---

## Files Changed

| File | Action |
|---|---|
| `scripts/game/capture-feature-clips.ps1` | **New** — replaces prove-features-video.ps1 |
| `scripts/video/package.json` | **New** — Remotion project |
| `scripts/video/remotion.config.ts` | **New** |
| `scripts/video/src/index.ts` | **New** |
| `scripts/video/src/types.ts` | **New** |
| `scripts/video/src/components/CalloutBox.tsx` | **New** |
| `scripts/video/src/components/CaptionBar.tsx` | **New** |
| `scripts/video/src/components/FeatureScene.tsx` | **New** |
| `scripts/video/src/components/TitleCard.tsx` | **New** |
| `scripts/video/src/compositions/ModsButtonFeature.tsx` | **New** |
| `scripts/video/src/compositions/F9OverlayFeature.tsx` | **New** |
| `scripts/video/src/compositions/F10MenuFeature.tsx` | **New** |
| `scripts/video/src/compositions/DINOForgeReel.tsx` | **New** |
| `scripts/video/generate_tts.py` | **New** |
| `scripts/video/vo_spec.json` | **New** |
| `.claude/commands/prove-features.md` | **Updated** — new orchestration steps |
| `docs/proof-of-features/README.md` | **Updated** — new evidence bundle format |
| `scripts/game/prove-features-video.ps1` | **Retired** — send to Windows Recycle Bin via `[Microsoft.VisualBasic.FileIO.FileSystem]::DeleteFile(...)` per File Deletion Protocol |

---

## Non-Goals

- No paid TTS (ElevenLabs, Azure, etc.) — edge-tts `en-US-AriaNeural` only
- No OBS — gdigrab title-based capture only
- No headless game capture — window always visible during recording
- No audio mixing/ducking between VO and in-game audio — VO track only
- No Linux/macOS support — Windows gdigrab + Win32 SendInput only
- No multiple-take selection — first validated take is accepted
- No CI gating in this phase — `validate_report.json` is machine-readable for future CI use
- No asset-swap / mod gameplay proof — this phase covers UI features (Mods button, F9, F10) only
