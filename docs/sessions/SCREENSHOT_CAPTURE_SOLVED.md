# Screenshot Capture — SOLVED via ScreenRecorderLib

**Date**: 2026-03-25
**Status**: RESOLVED — do not continue PS1 script iteration

## What Was Built (Session 8835fde1)

`src/Tools/McpServer/Tools/GameCaptureHelper.cs` implements non-intrusive headless GPU capture:

- **Primary**: ScreenRecorderLib 6.6.0 — Windows.Graphics.Capture (WGC) API
  - Captures per-window DirectX GPU content without focus
  - No z-order issues (bypasses Steam window, overlays, etc.)
  - Works on fullscreen exclusive DX11 — PrintWindow cannot
  - API: `Recorder.CreateRecorder(options)` → `recorder.TakeSnapshot(path)` + `OnSnapshotSaved` event

- **Fallback**: ffmpeg gdigrab — GDI desktop capture (blank for GPU content, but safe fallback)

## Why PS1 PrintWindow/ddagrab Approaches Fail

| Approach | Problem |
|---|---|
| `PrintWindow(hwnd, hdc, 0)` | GDI composite only — Unity DirectX GPU content = blank |
| ffmpeg ddagrab | Requires ffmpeg 5.0+; machine has 4.2.3 (ImageMagick) |
| ffmpeg gdigrab | Captures desktop GDI, not GPU — always blank for DINO |
| `BitBlt` | Same as PrintWindow — GDI only |
| Steam window in front | WGC captures per-window, not z-order dependent |

## How to Use

The `game_screenshot` MCP tool calls `GameCaptureHelper.CaptureAsync()` automatically.
From Claude Code, just call: `game_screenshot` tool — no PS1 scripts needed.

## Build
```bash
dotnet build src/Tools/McpServer/DINOForge.Tools.McpServer.csproj -c Release
```
Requires x64 platform (ScreenRecorderLib requirement) — already configured in .csproj and .sln.

## PS1 Scripts
All PS1 capture scripts have been moved to `scripts/game/desktop/` for reference.
They are no longer the active solution — use the MCP tool instead.
