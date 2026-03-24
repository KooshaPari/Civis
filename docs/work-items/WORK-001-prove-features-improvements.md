# WORK-001: /prove-features Improvements

**Priority**: High
**Status**: In Progress
**Created**: 2026-03-24
**Owner**: Agent (unassigned)
**Spec Reference**: `docs/specs/SPEC-003-prove-features-skill.md`
**Skill File**: `.claude/commands/prove-features.md`

---

## Description

The `/prove-features` command produces an autonomous video proof of DINOForge features. Two bugs currently prevent production-quality output, and two additional improvements are specified for reliability and maintainability:

1. **Window capture bug**: `gdigrab -i desktop` records the full desktop. If any other window (Discord, terminal, another game) is in the foreground during recording, it appears in the proof video instead of the DINO game. This makes the video proof invalid.

2. **SAPI voice quality**: The TTS implementation uses Windows SAPI (David/Zira voices), which produce audibly robotic speech. Any externally-shared proof video with SAPI audio signals low-quality tooling. Neural alternatives are free, require no API key, and are trivially installable.

3. **Missing exit code validation**: The skill does not check `$LASTEXITCODE` after ffmpeg invocations. A failed encode silently proceeds to `Start-Process $outFile` on a nonexistent file.

4. **No filter pre-validation**: Long `drawtext` filter chains can fail for subtle reasons (unescaped characters, missing font, unsupported expression syntax). A pre-validation pass against a synthetic source would surface these errors before the real encoding run.

---

## Tasks

### Bug Fixes

- [ ] **Fix window capture — capture DINO by window title, not desktop**

  Change the gdigrab input from `-i desktop` to `-i "title=Diplomacy is Not an Option"`.

  In `.claude/commands/prove-features.md`, Step 5, replace:
  ```powershell
  -f gdigrab -framerate 30 -i desktop -t 28
  ```
  with:
  ```powershell
  -f gdigrab -framerate 30 -i "title=Diplomacy is Not an Option" -t 28
  ```

  Add a coordinate-based fallback if the title form fails (e.g., game window title differs): use `GetWindowRect` on the process HWND + `-offset_x`, `-offset_y`, `-video_size` gdigrab parameters.

  Acceptance: recording works correctly when a terminal window is in the foreground during the record phase.

- [ ] **Replace Windows SAPI with higher-quality TTS**

  The target is `edge-tts` (Microsoft Edge neural voices, free, no API key, Python package). Install: `pip install edge-tts`. Invocation: `python -m edge_tts --voice en-US-AriaNeural --text "..." --write-media out.mp3`.

  Update `.claude/commands/prove-features.md`, Step 4:
  - Replace `Speak-ToFile` (SAPI) calls with `New-EdgeTtsVoiceover` wrapper
  - Detect Python 3.11 at known path; fail with actionable error if not found
  - Retain the existing SAPI implementation as `New-SapiVoiceover` for use only as explicit fallback when edge-tts fails (network error, Python missing)
  - Convert edge-tts MP3 output to WAV via ffmpeg before the concat stage (gdigrab pipeline expects WAV for concat demuxer compatibility)

  Acceptance: generated voiceover is audibly neural-quality (no robotic cadence), confirmed by manual spot-check.

### Reliability Improvements

- [ ] **Test neural TTS options headlessly**

  Validate that `edge-tts` and `kokoro-onnx` both work in a headless PowerShell session (no interactive terminal, no browser). Run each in a `Start-Process -NoNewWindow` context to confirm they do not spawn GUI dialogs or require stdin.

  Document results in a comment block at the top of the TTS section of `.claude/commands/prove-features.md`:
  - edge-tts: headless compatible yes/no
  - kokoro-onnx: headless compatible yes/no
  - Which is set as primary and why

- [ ] **Add exit code validation after each ffmpeg call**

  After every `& $ffmpeg ...` call, add:
  ```powershell
  if ($LASTEXITCODE -ne 0) {
      Write-Error "ffmpeg failed at [stage name] (exit code $LASTEXITCODE)"
      exit 1
  }
  ```

  Affected stages: raw capture (post-record wait), voice concatenation, final encode.

- [ ] **Add filter pre-validation pass**

  Before the real encode, validate the filter string against a synthetic 1-second black video:
  ```powershell
  & $ffmpeg -vf $filters -f lavfi -i "color=black:s=1920x1080:d=1" -f null - 2>&1 | Out-Null
  if ($LASTEXITCODE -ne 0) {
      Write-Error "drawtext filter validation failed. Check font path and filter syntax."
      exit 1
  }
  ```

  This catches font path errors, unescaped special characters, and unsupported expression syntax before committing to the full 28s encode.

### Documentation

- [ ] **Update `.claude/commands/prove-features.md` skill**

  After all code changes are complete, update the skill file to reflect:
  - Corrected gdigrab invocation (title= form + coordinate fallback)
  - edge-tts as primary TTS with SAPI fallback
  - Exit code checks at each ffmpeg stage
  - Filter pre-validation step
  - Updated Requirements section (add Python 3.11, edge-tts as deps)
  - Updated TTS section (replace SAPI function with edge-tts wrapper)

- [ ] **Update `SPEC-003-prove-features-skill.md` status table**

  Mark resolved items as "Fixed" in the Status table at the bottom of the spec.

---

## Acceptance Criteria

1. **Window capture**: Recording a 28-second session with a terminal window in the foreground produces video showing only the DINO game, not the terminal.

2. **TTS quality**: Generated voiceover audio is audibly neural (smooth prosody, natural cadence). A human listener cannot distinguish it from a professional recording in a casual listening context.

3. **TTS fallback**: If Python is not installed or edge-tts raises a network error, the skill falls back to SAPI and logs a warning (does not crash).

4. **Error propagation**: If any ffmpeg stage fails, the script prints a clear error message identifying which stage failed and exits with code 1. It does not proceed to open a nonexistent output file.

5. **Filter safety**: A deliberately broken filter string (e.g., missing closing quote) is caught by the pre-validation pass and reported before the real encode begins.

6. **Skill file updated**: `.claude/commands/prove-features.md` reflects all changes. No remnants of the old SAPI-only TTS path remain as the primary code path.

7. **End-to-end smoke test**: The full pipeline runs without error and produces a valid `.mp4` file of at least 5MB that opens in Windows Media Player.

---

## Notes

### TTS Engine Recommendation Summary

Three headless-compatible options exist, ranked by suitability:

| Engine | Quality | Offline | API Key | Complexity |
|--------|---------|---------|---------|------------|
| edge-tts | Excellent (neural) | No — streams from Edge API | None | `pip install edge-tts` |
| Kokoro-82M (onnxruntime) | Good (near-neural) | Yes (after ~90MB model download) | None | `pip install kokoro-onnx soundfile` |
| Azure TTS | Excellent (neural) | No — always requires REST | Yes (Azure account) | REST call, no Python needed |
| Windows SAPI | Poor (robotic) | Yes | None | Built-in, no install |

**Selected approach**: edge-tts primary, Kokoro-82M offline fallback, SAPI last resort. Azure TTS is not adopted due to the API key requirement adding friction for agent-driven use.

### gdigrab title= Behavior Notes

The `gdigrab` device in ffmpeg supports two input specifiers:
- `-i desktop` — captures the entire virtual desktop (all monitors)
- `-i "title=<window title>"` — captures the client area of the window with that exact title

The `title=` form calls `FindWindow(NULL, title)` internally. The match is exact and case-sensitive on some ffmpeg builds. If the game appends resolution or API info to the title bar (e.g., "Diplomacy is Not an Option - Direct3D 11"), the match fails and ffmpeg exits with `Could not find window with title`.

Mitigation: Test the exact title at the time of recording using `Get-Process | Select-Object MainWindowTitle` and compare. If the title is dynamic, use the HWND coordinate fallback path.

### Skill File Scope

This work item only modifies the skill file (`.claude/commands/prove-features.md`) and its spec (`docs/specs/SPEC-003-prove-features-skill.md`). No C# source files, MSBuild targets, or pack definitions are affected.

---

## Related

- `docs/specs/SPEC-003-prove-features-skill.md` — Full technical specification
- `docs/specs/SPEC-prove-features-video-pipeline.md` — Prior informal spec (superseded by SPEC-003)
- `.claude/commands/prove-features.md` — Skill implementation file to be updated
- `docs/specs/SPEC-duplicate-instance-bypass.md` — Related: game launch reliability
