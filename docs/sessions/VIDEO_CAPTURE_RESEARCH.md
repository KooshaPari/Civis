# Video Capture Options for Windows 11 - Diplomacy is Not an Option

## System Information
- **OS**: Windows 11 Pro (Build 28020)
- **GPU**: NVIDIA GeForce RTX 3090 Ti
- **Driver Version**: 32.0.15.9186
- **Game**: Diplomacy is Not an Option (Unity 2021.3, DirectX 11)
- **Game Status**: Running (PID 484960)

## Installed Tools & Capabilities

### 1. OBS Studio 27.1.3 (64-bit) ✓ INSTALLED
**Location**: `C:\Program Files\obs-studio\bin\64bit\obs64.exe`

**Built-in Capture Sources**:
- **Game Capture** (D3D11/DXGI hook) - Fast, requires running game
- **Display Capture** (GDI-based) - Slower, no coordinate dependency
- **Window Capture** (GDI) - Medium speed
- **Video Capture Device** - Hardware camera capture

**Key Plugins**:
- `obs-ffmpeg.dll` - FFmpeg encoding backend
- `win-capture.dll` - Windows capture hooks (Game Capture, Window Capture)
- `win-dshow.dll` - DirectShow device enumeration
- `win-mf.dll` - Windows Media Foundation support

**Headless Capability**:
- Can be launched with `--scene`, `--startrecording`, `--minimize-to-tray` flags
- Supports command-line recording start/stop
- Does NOT have true headless mode in v27.1.3 (requires UI thread)
- Recommended: Pre-configure scene + launch via script

---

### 2. ffmpeg 4.2.3 ✓ INSTALLED
**Location**: `C:\Program Files\ImageMagick-7.1.0-Q16-HDRI\ffmpeg.exe`

**Key Build Options**:
```
--enable-nvenc --enable-nvdec --enable-d3d11va --enable-cuvid --enable-dxva2
```

**Hardware Acceleration**:
- `cuda` - NVIDIA CUDA
- `d3d11va` - DirectX 11 Video Acceleration (modern)
- `dxva2` - DirectX Video Acceleration (legacy)
- `qsv` - Intel Quick Sync Video

**NVIDIA Encoding**:
- `h264_nvenc` - H.264 encoder via NVIDIA NVENC
- `hevc_nvenc` / `nvenc_hevc` - H.265 HEVC encoder

**Input Formats**:
- `gdigrab` - GDI API Windows frame grabber (CPU-based, slower)
- **MISSING**: `dxgicap` / `d3d11grab` (would be faster but not in this build)

---

### 3. NVIDIA GPU Level Features ✓ AVAILABLE

**NvFBC DLLs Present**:
- `C:\Windows\System32\NvFBC64.dll` (2.3 MB)
- `C:\Windows\System32\nvEncodeAPI64.dll` (1.0 MB)
- `C:\Windows\System32\nvml.dll` (1.3 MB) - NVIDIA Management Library

**NvFBC Capabilities**:
- Zero-latency GPU-level frame capture
- Works even if game minimized (framebuffer exists)
- Minimal CPU impact
- **Limitation**: Not directly exposed via ffmpeg; requires custom C# wrapper

---

### 4. Xbox Game Overlay ✓ INSTALLED

- `Microsoft.XboxGameOverlay` v1.54.4001.0
- `Microsoft.XboxGamingOverlay` v7.325.10021.0
- Uses: Windows.Graphics.Capture API (HWND-specific, modern, Win10 1903+)
- **Limitation**: Overlay-only, cannot be used for standalone headless capture

---

### 5. Windows.Graphics.Capture API ✓ AVAILABLE
- **OS Build**: 28020 (supports Win10 1903+)
- Modern HWND-specific capture API
- Available via WinRT interop in .NET/PowerShell
- **Used by**: Xbox Game Bar, modern capture tools

---

### 6. Python 3.11.9 ✓ INSTALLED
**Path**: `C:\Users\koosh\AppData\Local\Programs\Python\Python311\python.exe`

Potential libraries:
- `pywin32` - Win32 API access (drive OBS, find HWND)
- `opencv-python` + `numpy` - Frame processing
- `pillow` - Fallback screenshot capture

---

### 7. .NET 11.0.100 ✓ INSTALLED
- Can use WinRT interop for Windows.Graphics.Capture
- Can build C# wrapper for NvFBC if needed

---

## Evaluation: Top 5 Approaches

### 🥇 Option 1: OBS + Game Capture + PowerShell Script (RECOMMENDED)

**Approach**:
1. Create OBS profile with Game Capture source for the game window
2. Use PowerShell script to:
   - Launch OBS with scene configured
   - Start recording via command-line or WM_COMMAND
   - Close on completion

**Pros**:
- ✓ D3D11/DXGI hook directly from game rendering
- ✓ Hardware-accelerated via NVIDIA NVENC
- ✓ Zero latency, best quality
- ✓ OBS 27.1.3 already installed
- ✓ Fast (target: 60 fps @ 1440p = real-time)

**Cons**:
- ✗ Not fully headless (UI thread ~100ms overhead)
- ✗ Requires pre-configured scene
- ~ Recording control is timestamp-based

**Quality Score**: 9.5/10

**Command Example**:
```powershell
& 'C:\Program Files\obs-studio\bin\64bit\obs64.exe' `
  --scene "GameCapture" `
  --startrecording `
  --profile "default" `
  --collection "default"
```

---

### 🥈 Option 2: ffmpeg gdigrab + NVIDIA NVENC (Fallback)

**Approach**:
```bash
ffmpeg -f gdigrab -framerate 60 \
  -offset_x 100 -offset_y 50 -video_size 2560x1440 \
  -i desktop \
  -c:v h264_nvenc -preset fast -b:v 8000k \
  output.mp4
```

**Pros**:
- ✓ No setup required (ffmpeg already installed)
- ✓ Fully headless and scriptable
- ✓ Hardware encoding via NVENC (minimal CPU)
- ✓ Can specify exact region via coordinates

**Cons**:
- ✗ `gdigrab` is GDI-based (slower than D3D hook)
- ✗ Requires calculating game window coordinates (fragile)
- ✗ Slower performance (~30-40 fps @ 1440p)
- ✗ No direct game buffer integration

**Quality Score**: 6.5/10

---

### 🥉 Option 3: C# WinRT Wrapper (Windows.Graphics.Capture)

**Approach**:
1. Build C# console app using WinRT interop
2. Enumerate windows, find game HWND
3. Create `GraphicsCaptureItem` from HWND
4. Process frames to MP4 via FFmpeg

**Pros**:
- ✓ Modern API (Win10 1903+)
- ✓ Direct HWND capture (no coordinate calculation)
- ✓ Modern implementation, lower CPU than gdigrab
- ✓ Can integrate into DINOForge tooling

**Cons**:
- ✗ Requires C# project + FFmpeg wrapper
- ✗ ~200-300 lines of code
- ✗ WinRT interop edge cases on older .NET
- ✓ Development: 2-3 hours

**Quality Score**: 7.5/10

---

### 📍 Option 4: NvFBC via C# P/Invoke

**Approach**:
1. Build P/Invoke wrapper to `NvFBC64.dll`
2. Capture GPU framebuffer directly
3. Pipe to NVENC or x264 encoder

**Pros**:
- ✓ Zero-latency GPU-level capture
- ✓ Works minimized (framebuffer exists)
- ✓ Minimal CPU impact
- ✓ Fastest method available

**Cons**:
- ✗ Requires NvFBC SDK (separate from driver)
- ✗ Complex P/Invoke marshaling
- ✗ Edge cases across GPU generations
- ✗ Development: 4-6 hours

**Quality Score**: 8/10 (if SDK available)

---

### 🔵 Option 5: Unity Screenshot (In-Game)

**Approach**:
Inject into DINOForge Runtime:
```csharp
ScreenCapture.CaptureScreenshot("frame.png", superSize: 2);
```

**Pros**:
- ✓ Runs inside game process
- ✓ Access to all game data

**Cons**:
- ✗ Very slow (CPU-GPU sync per frame)
- ✗ Cannot run headless
- ✗ 1-2 fps max (unsuitable for sustained recording)
- ✗ Requires ECS integration

**Quality Score**: 2/10

---

## Recommendation

| Use Case | Approach | Time | Effort |
|----------|----------|------|--------|
| **CI/Testing** | Option 1 (OBS Game Capture) | 30 min | Low |
| **Interactive Dev** | Option 2 (ffmpeg gdigrab) | 5 min | Trivial |
| **Production** | Option 3 (WinRT) or 4 (NvFBC) | 2-6 hrs | High |

---

## Command Examples

### OBS Game Capture (Headless-ish)
```powershell
# Requires pre-configured profile/scene
$obsPath = 'C:\Program Files\obs-studio\bin\64bit\obs64.exe'
& $obsPath --scene "Game Capture" --startrecording --profile default
Start-Sleep -Seconds 30  # Record for 30 seconds
# To stop: SendMessage or kill process
```

### ffmpeg Screen Region (Fully Headless)
```bash
ffmpeg \
  -f gdigrab \
  -framerate 60 \
  -offset_x 0 -offset_y 0 \
  -video_size 2560x1440 \
  -i desktop \
  -c:v h264_nvenc \
  -preset fast \
  -b:v 8000k \
  -t 30 \
  output.mp4
```

### ffmpeg Full Screen (Simplest)
```bash
ffmpeg \
  -f gdigrab \
  -framerate 60 \
  -i desktop \
  -c:v h264_nvenc \
  -preset fast \
  output.mp4
```

---

## Next Steps

1. **Test OBS capture** - Pre-configure scene, test command-line launch
2. **Test ffmpeg gdigrab** - One-liner for quick validation
3. **Choose approach** - Based on quality vs. effort trade-off
4. **Integrate into tooling** - Add to DINOForge test harness if needed
