# DINOForge Proof of Features

Automated evidence bundles from feature validation runs using the v2 pipeline.

## Bundle Structure

Each run of `/prove-features` creates a timestamped directory `YYYYMMDD_HHmmss/`:

```
<timestamp>/
  raw_mods.mp4            Raw 6s game clip — main menu with Mods button visible
  raw_f9.mp4              Raw 8s game clip — F9 debug overlay triggered
  raw_f10.mp4             Raw 8s game clip — F10 mod menu triggered
  validate_mods.png       VLM-confirmed screenshot (live GDI at time of Mods display)
  validate_f9.png         VLM-confirmed screenshot (live GDI at time of F9 display)
  validate_f10.png        VLM-confirmed screenshot (live GDI at time of F10 display)
  validate_report.json    Machine-readable: [{ feature, vlm_response, confirmed, timestamp }]
  mods_feature.mp4        Remotion-rendered 10s clip with spring-physics callout (green)
  f9_feature.mp4          Remotion-rendered 10s clip with spring-physics callout (yellow)
  f10_feature.mp4         Remotion-rendered 10s clip with spring-physics callout (blue)
  dinoforge_reel.mp4      38s compilation reel — primary deliverable
  proof_report.md         Human-readable summary with validation table and render status
```

## Pipeline (v2)

### Phase 1: Capture (`scripts/game/capture-feature-clips.ps1`)
- Game launched fresh, all instances killed first
- Boot wait: polls `dinoforge_debug.log` for `"Awake completed"` (Stage A, 30s)
- Menu wait: polls `LogOutput.log` for `"MODS BUTTON INJECTION FULLY SUCCESSFUL"` (Stage B, 720s)
- Capture: `ffmpeg -f gdigrab -i "title=Diplomacy is Not an Option"` — NOT `-i desktop`
- Key injection: Win32 `SendInput` via C# `Add-Type` — works without window focus
- Resolution: all clips normalized to 1280×800 via `-vf scale=1280:800`

### VLM Validation Gate (orchestrated in Claude session)
- After each clip: `game_analyze_screen` MCP takes a **live GDI screenshot** of the game window
- Game remains running and in correct UI state during validation
- Retry once on failure; second failure exits non-zero and deletes the unvalidated clip
- Results written to `validate_report.json` with `confirmed: true/false`

### Phase 2: TTS (`scripts/video/generate_tts.py`)
- Reads `scripts/video/vo_spec.json` (5 line items)
- Generates 5 MP3s via `edge-tts` en-US-AriaNeural (Microsoft Edge neural voice — free)
- File-based approach avoids PowerShell `ArgumentList` arg-splitting bug from v1

### Phase 3: Composition (`scripts/video/` — Remotion project)
- `CalloutBox`: spring-physics scale-in (`damping: 12, stiffness: 180`) — natural overshoot
- `FeatureScene`: raw clip + TTS audio + callout + caption bar, with freeze-frame padding
- Individual clips: 300 frames (10s) each — 6-8s source + freeze-frame remainder
- Compilation reel: 1140 frames (38s): 3s intro + 10s×3 features + 5s outro
- Output: H.264 baseline, yuv420p, AAC 128k, `faststart`

## Color Coding

| Feature | Color | Hex |
|---|---|---|
| Mods Button | Green | `#34d399` |
| F9 Debug Overlay | Yellow | `#fbbf24` |
| F10 Mod Menu | Blue | `#60a5fa` |

## Run

```powershell
# From repo root:
/prove-features
```

## Legacy Files

Files in this directory without a timestamp subfolder are from the v1 pipeline (pre-2026-03-27)
and are kept for reference. The v1 pipeline had three known defects:
1. gdigrab used `-i desktop` — captured the Claude Code terminal, not the game
2. edge-tts arg-splitting via PowerShell `ArgumentList` fell back to SAPI (90s-era voice)
3. No VLM validation — blank or wrong captures counted as proof
