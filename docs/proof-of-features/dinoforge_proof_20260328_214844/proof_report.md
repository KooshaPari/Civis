# DINOForge Feature Proof — 2026-03-28

## Summary

This bundle captures confirmed working evidence for three DINOForge Runtime features after the Harmony UiGrid patch fix.

## Features Proven

### 1. Mods Button (main menu)

- **Status**: CONFIRMED WORKING
- **Evidence**: `validate_mods.png`, `raw_mods.mp4`, `mods_feature.mp4`
- **Method**: gdigrab screenshot + 8s clip of main menu showing "MODS" label
- **Fix applied**: `NativeMenuInjector.EnforceModsButtonText()` — re-asserts "Mods" every 10 frames to combat UiGrid reverting the label via internal text path (not interceptable via TMP setter Harmony patch on Mono)
- **Log confirmation**: `EnforceModsButtonText: re-set reverted label to 'Mods'`

### 2. F9 Debug Overlay

- **Status**: CONFIRMED WORKING
- **Evidence**: `validate_f9.png`, `raw_f9.mp4`, `f9_feature.mp4`
- **Method**: Win32 keybd_event → F9 → screenshot + 8s clip
- **Log confirmation**: `F9 pressed (transition detected)` + `RuntimeDriver F9 pressed (via KeyInputSystem)`

### 3. F10 Mod Menu

- **Status**: CONFIRMED WORKING
- **Evidence**: `validate_f10.png`, `raw_f10.mp4`, `f10_feature.mp4`
- **Method**: Win32 keybd_event → F10 → screenshot + 8s clip
- **Log confirmation**: `F10 pressed (transition detected)` + `RuntimeDriver F10 pressed (via KeyInputSystem)`

## Rendered Videos

| File | Size | Status |
|------|------|--------|
| `mods_feature.mp4` | 976 KB | OK |
| `f9_feature.mp4` | 984 KB | OK |
| `f10_feature.mp4` | 960 KB | OK |
| `dinoforge_reel.mp4` | 3.2 MB | OK |

## Technical Notes

- Raw clips captured as WebM/VP8 (gdigrab → H.264 → transcode to VP8 for Remotion/Chromium compatibility)
- TTS generated via edge-tts (5 files, ~30-40 KB each)
- Remotion render: 30fps, 10s per feature (300 frames), 38s for full reel (1140 frames)

## Harmony Patch Investigation

UiGrid restores button text via an internal path that bypasses TMP_Text setters and SetText(string,bool). The `PatchAll` approach failed with "Patching exception in method virtual void TMPro.TMP_Text::set_text" on Mono. Manual `Apply()` with `SetText(string,bool)` patch was applied but never fired — UiGrid uses a different code path. Final solution: periodic text re-enforcement in `NativeMenuInjector.Update()` at 6x/second (every 10 frames).
