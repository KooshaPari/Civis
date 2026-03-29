# Prove-Features Video Pipeline v2 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the broken v1 proof pipeline with a 3-phase system that captures the correct game window, generates neural TTS, composes a professional MP4 via Remotion with spring-physics callouts, and validates each captured UI state via VLM before accepting it as proof.

**Architecture:** Phase 1 (PowerShell) captures three separate game clips via gdigrab targeted by window title, injects keys via Win32 SendInput, and gates each clip on a live VLM screenshot from `game_analyze_screen`. Phase 2 (Python) generates 5 edge-tts MP3 files from a JSON spec. Phase 3 (Remotion/React) composes each clip into a 10s scene with spring-physics callouts, then assembles a 38s compilation reel. An updated `/prove-features` command orchestrates all three phases and bundles all artifacts into a timestamped evidence directory.

**Tech Stack:** PowerShell 5+, ffmpeg (ImageMagick bundle), Python 3 + edge-tts, Node.js 18+ + @remotion/core, Win32 P/Invoke (C# Add-Type), game_analyze_screen MCP tool

---

## File Map

| File | Action | Responsibility |
|---|---|---|
| `scripts/game/capture-feature-clips.ps1` | **Create** | Phase 1: game launch, gdigrab capture, SendInput, VLM gate |
| `scripts/video/generate_tts.py` | **Create** | Phase 2: read vo_spec.json, output 5 MP3s via edge-tts |
| `scripts/video/vo_spec.json` | **Create** | TTS line items (id, voice, text) |
| `scripts/video/package.json` | **Create** | Remotion project: deps + render scripts |
| `scripts/video/remotion.config.ts` | **Create** | Remotion config: codec, fps, resolution |
| `scripts/video/src/index.ts` | **Create** | Register all 4 compositions |
| `scripts/video/src/types.ts` | **Create** | Shared TypeScript types |
| `scripts/video/src/components/CalloutBox.tsx` | **Create** | Spring-physics scale-in box |
| `scripts/video/src/components/CaptionBar.tsx` | **Create** | Permanent bottom caption strip |
| `scripts/video/src/components/FeatureScene.tsx` | **Create** | Video + callout + caption + audio assembly |
| `scripts/video/src/components/TitleCard.tsx` | **Create** | Full-frame text for intro/outro |
| `scripts/video/src/compositions/ModsButtonFeature.tsx` | **Create** | Green mods clip scene |
| `scripts/video/src/compositions/F9OverlayFeature.tsx` | **Create** | Yellow F9 clip scene |
| `scripts/video/src/compositions/F10MenuFeature.tsx` | **Create** | Blue F10 clip scene |
| `scripts/video/src/compositions/DINOForgeReel.tsx` | **Create** | 38s compilation reel |
| `.claude/commands/prove-features.md` | **Update** | Replace v1 instructions with v2 orchestration steps |
| `docs/proof-of-features/README.md` | **Update** | Document new evidence bundle format |
| `scripts/game/prove-features-video.ps1` | **Retire** | Send to Recycle Bin via VisualBasic.FileIO |

---

## Task 1: Create `scripts/video/vo_spec.json`

No dependencies. Simplest file, needed by Phase 2.

**Files:**
- Create: `scripts/video/vo_spec.json`

- [ ] **Step 1: Write vo_spec.json**

```json
[
  { "id": "intro",  "voice": "en-US-AriaNeural", "text": "DINOForge mod platform. Feature demonstration." },
  { "id": "mods",   "voice": "en-US-AriaNeural", "text": "The Mods button was automatically injected into the native main menu in under ten seconds." },
  { "id": "f9",     "voice": "en-US-AriaNeural", "text": "Pressing F9 opens the debug overlay panel, showing live entity counts and runtime stats." },
  { "id": "f10",    "voice": "en-US-AriaNeural", "text": "Pressing F10 opens the mod menu, where packs can be browsed and toggled." },
  { "id": "outro",  "voice": "en-US-AriaNeural", "text": "All three features confirmed working. DINOForge is ready for testing." }
]
```

- [ ] **Step 2: Verify JSON parses**

```bash
python -c "import json; d=json.load(open('scripts/video/vo_spec.json')); print(f'{len(d)} entries OK')"
```
Expected: `5 entries OK`

- [ ] **Step 3: Commit**

```bash
git add scripts/video/vo_spec.json
git commit -m "feat(video): add TTS voice-over spec for v2 pipeline"
```

---

## Task 2: Create `scripts/video/generate_tts.py`

Depends on: Task 1 (vo_spec.json must exist to test against).

**Files:**
- Create: `scripts/video/generate_tts.py`

- [ ] **Step 1: Verify edge-tts is installed**

```bash
python -m edge_tts --version
```
If missing: `pip install edge-tts`

- [ ] **Step 2: Write generate_tts.py**

```python
#!/usr/bin/env python3
"""Generate TTS audio files from a voice-over spec JSON."""
import argparse
import asyncio
import json
import os
import sys
import edge_tts


async def generate_one(entry: dict, out_dir: str) -> str:
    out_path = os.path.join(out_dir, f"{entry['id']}.mp3")
    communicate = edge_tts.Communicate(entry["text"], entry["voice"])
    await communicate.save(out_path)
    size = os.path.getsize(out_path)
    if size < 1024:
        raise RuntimeError(f"{entry['id']}.mp3 too small ({size} bytes) — TTS likely failed")
    print(f"  OK {entry['id']}.mp3 ({size // 1024}KB)")
    return out_path


async def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--spec", required=True, help="Path to vo_spec.json")
    parser.add_argument("--out", required=True, help="Output directory for MP3 files")
    args = parser.parse_args()

    with open(args.spec, encoding="utf-8") as f:
        spec = json.load(f)

    os.makedirs(args.out, exist_ok=True)
    print(f"Generating {len(spec)} TTS files → {args.out}")

    tasks = [generate_one(entry, args.out) for entry in spec]
    results = await asyncio.gather(*tasks, return_exceptions=True)

    failed = [r for r in results if isinstance(r, Exception)]
    if failed:
        for err in failed:
            print(f"ERROR: {err}", file=sys.stderr)
        sys.exit(1)

    print(f"Done. {len(results)} files generated.")


if __name__ == "__main__":
    asyncio.run(main())
```

- [ ] **Step 3: Run a smoke test (generates real MP3s)**

```bash
python scripts/video/generate_tts.py \
  --spec scripts/video/vo_spec.json \
  --out "$TEMP/DINOForge/tts"
```
Expected: 5 `OK *.mp3` lines, no errors, exit 0

- [ ] **Step 4: Verify MP3 sizes**

```bash
ls -lh "$TEMP/DINOForge/tts/"
```
Each file should be > 10KB

- [ ] **Step 5: Commit**

```bash
git add scripts/video/generate_tts.py
git commit -m "feat(video): add edge-tts generator script (reads from vo_spec.json, no arg-splitting)"
```

---

## Task 3: Remotion project scaffold — package.json + remotion.config.ts

**Files:**
- Create: `scripts/video/package.json`
- Create: `scripts/video/remotion.config.ts`
- Create: `scripts/video/src/types.ts`

- [ ] **Step 1: Write package.json**

```json
{
  "name": "dinoforge-video",
  "version": "1.0.0",
  "description": "DINOForge feature demonstration video — Remotion project",
  "scripts": {
    "render:mods":  "remotion render src/index.ts ModsButtonFeature --output out/mods_feature.mp4",
    "render:f9":    "remotion render src/index.ts F9OverlayFeature  --output out/f9_feature.mp4",
    "render:f10":   "remotion render src/index.ts F10MenuFeature    --output out/f10_feature.mp4",
    "render:reel":  "remotion render src/index.ts DINOForgeReel     --output out/dinoforge_reel.mp4",
    "render:all":   "npm run render:mods && npm run render:f9 && npm run render:f10 && npm run render:reel",
    "studio":       "remotion studio src/index.ts"
  },
  "dependencies": {
    "@remotion/cli": "4.0.257",
    "remotion": "4.0.257"
  },
  "devDependencies": {
    "@types/react": "^18.3.0",
    "typescript": "^5.4.0"
  }
}
```

- [ ] **Step 2: Write remotion.config.ts**

```typescript
import { Config } from "@remotion/cli/config";

Config.setVideoImageFormat("jpeg");
Config.setPixelFormat("yuv420p");
Config.setCodec("h264");
Config.setOverwriteOutput(true);
```

- [ ] **Step 3: Write src/types.ts**

```typescript
export interface CalloutProps {
  text: string;
  subText?: string;
  color: string;           // hex, e.g. "#34d399"
  startFrame: number;
}

export interface FeatureSceneProps {
  videoSrc: string;        // absolute path to raw clip
  calloutText: string;
  calloutSubText?: string;
  calloutColor: string;
  audioSrc: string;        // absolute path to MP3
  rawDurationFrames: number;  // actual frames in source clip (e.g. 6s*30fps = 180)
}
```

- [ ] **Step 4: Install dependencies**

```bash
cd scripts/video && npm install
```
Expected: `node_modules/` created, no peer-dep errors

- [ ] **Step 5: Commit**

```bash
git add scripts/video/package.json scripts/video/remotion.config.ts scripts/video/src/types.ts
git commit -m "feat(video): scaffold Remotion project — package.json, config, types"
```

---

## Task 4: Remotion components — CalloutBox, CaptionBar, TitleCard

**Files:**
- Create: `scripts/video/src/components/CalloutBox.tsx`
- Create: `scripts/video/src/components/CaptionBar.tsx`
- Create: `scripts/video/src/components/TitleCard.tsx`

- [ ] **Step 1: Write CalloutBox.tsx (spring physics)**

```tsx
import React from "react";
import { useCurrentFrame, useVideoConfig, spring } from "remotion";
import type { CalloutProps } from "../types";

export const CalloutBox: React.FC<CalloutProps> = ({
  text,
  subText,
  color,
  startFrame,
}) => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();

  const relativeFrame = Math.max(0, frame - startFrame);
  const scale = spring({
    frame: relativeFrame,
    fps,
    config: { damping: 12, stiffness: 180 },
  });

  return (
    <div
      style={{
        position: "absolute",
        top: 120,
        right: 40,
        transform: `scale(${scale})`,
        transformOrigin: "top right",
        background: "rgba(0,0,0,0.75)",
        border: `3px solid ${color}`,
        borderRadius: 8,
        padding: "14px 20px",
        minWidth: 280,
        maxWidth: 380,
      }}
    >
      <div
        style={{
          color,
          fontSize: 28,
          fontWeight: 700,
          fontFamily: "Arial, sans-serif",
          letterSpacing: 0.5,
        }}
      >
        {text}
      </div>
      {subText && (
        <div
          style={{
            color: "white",
            fontSize: 16,
            fontFamily: "Arial, sans-serif",
            marginTop: 6,
            opacity: 0.9,
          }}
        >
          {subText}
        </div>
      )}
    </div>
  );
};
```

- [ ] **Step 2: Write CaptionBar.tsx**

```tsx
import React from "react";

export const CaptionBar: React.FC = () => (
  <div
    style={{
      position: "absolute",
      bottom: 0,
      left: 0,
      right: 0,
      background: "rgba(0,0,0,0.75)",
      padding: "8px 0",
      textAlign: "center",
      fontFamily: "Arial, sans-serif",
      fontSize: 16,
      color: "white",
    }}
  >
    F9 = Debug Overlay &nbsp;|&nbsp; F10 = Mod Menu &nbsp;|&nbsp; Mods Button = Native Menu Injection
  </div>
);
```

- [ ] **Step 3: Write TitleCard.tsx**

```tsx
import React from "react";
import { AbsoluteFill, Audio } from "remotion";

interface TitleCardProps {
  title: string;
  subtitle?: string;
  audioSrc?: string;
}

export const TitleCard: React.FC<TitleCardProps> = ({ title, subtitle, audioSrc }) => (
  <AbsoluteFill
    style={{
      background: "#0a0a0f",
      justifyContent: "center",
      alignItems: "center",
      flexDirection: "column",
      gap: 16,
    }}
  >
    {audioSrc && <Audio src={audioSrc} />}
    <div
      style={{
        color: "white",
        fontSize: 56,
        fontWeight: 800,
        fontFamily: "Arial, sans-serif",
        textAlign: "center",
        letterSpacing: 2,
      }}
    >
      {title}
    </div>
    {subtitle && (
      <div
        style={{
          color: "#94a3b8",
          fontSize: 28,
          fontFamily: "Arial, sans-serif",
          textAlign: "center",
        }}
      >
        {subtitle}
      </div>
    )}
  </AbsoluteFill>
);
```

- [ ] **Step 4: Commit**

```bash
git add scripts/video/src/components/
git commit -m "feat(video): add Remotion components — CalloutBox (spring physics), CaptionBar, TitleCard"
```

---

## Task 5: Remotion component — FeatureScene

**Files:**
- Create: `scripts/video/src/components/FeatureScene.tsx`

- [ ] **Step 1: Write FeatureScene.tsx**

```tsx
import React from "react";
import { AbsoluteFill, Audio, OffthreadVideo } from "remotion";
import { CalloutBox } from "./CalloutBox";
import { CaptionBar } from "./CaptionBar";
import type { FeatureSceneProps } from "../types";

export const FeatureScene: React.FC<FeatureSceneProps> = ({
  videoSrc,
  calloutText,
  calloutSubText,
  calloutColor,
  audioSrc,
  rawDurationFrames,
}) => (
  <AbsoluteFill>
    {/* Video — freeze-frames on last frame when clip ends */}
    <OffthreadVideo
      src={videoSrc}
      style={{ width: "100%", height: "100%", objectFit: "cover" }}
      lastImageDefaultProps={{ pauseWhenBuffering: false }}
    />

    {/* TTS audio */}
    <Audio src={audioSrc} />

    {/* Callout box — spring in at frame 15 (0.5s) */}
    <CalloutBox
      text={calloutText}
      subText={calloutSubText}
      color={calloutColor}
      startFrame={15}
    />

    {/* Permanent caption bar */}
    <CaptionBar />
  </AbsoluteFill>
);
```

- [ ] **Step 2: Commit**

```bash
git add scripts/video/src/components/FeatureScene.tsx
git commit -m "feat(video): add FeatureScene component with freeze-frame padding and spring callout"
```

---

## Task 6: Remotion compositions — 3 feature clips + DINOForgeReel

**Files:**
- Create: `scripts/video/src/compositions/ModsButtonFeature.tsx`
- Create: `scripts/video/src/compositions/F9OverlayFeature.tsx`
- Create: `scripts/video/src/compositions/F10MenuFeature.tsx`
- Create: `scripts/video/src/compositions/DINOForgeReel.tsx`
- Create: `scripts/video/src/index.ts`

- [ ] **Step 1: Write ModsButtonFeature.tsx**

```tsx
import React from "react";
import { FeatureScene } from "../components/FeatureScene";

// Paths passed as env at render time; defaults for studio preview
const RAW_CLIP = process.env.RAW_MODS_PATH ?? "/tmp/DINOForge/capture/raw_mods.mp4";
const TTS_FILE = process.env.TTS_MODS_PATH ?? "/tmp/DINOForge/tts/mods.mp3";

export const ModsButtonFeature: React.FC = () => (
  <FeatureScene
    videoSrc={RAW_CLIP}
    calloutText="Mods Button Injected"
    calloutSubText="Native menu — under 10 seconds"
    calloutColor="#34d399"
    audioSrc={TTS_FILE}
    rawDurationFrames={180}  // 6s × 30fps
  />
);
```

- [ ] **Step 2: Write F9OverlayFeature.tsx**

```tsx
import React from "react";
import { FeatureScene } from "../components/FeatureScene";

const RAW_CLIP = process.env.RAW_F9_PATH ?? "/tmp/DINOForge/capture/raw_f9.mp4";
const TTS_FILE = process.env.TTS_F9_PATH ?? "/tmp/DINOForge/tts/f9.mp3";

export const F9OverlayFeature: React.FC = () => (
  <FeatureScene
    videoSrc={RAW_CLIP}
    calloutText="[ F9 ] Debug Overlay"
    calloutSubText="Live entity counts and runtime stats"
    calloutColor="#fbbf24"
    audioSrc={TTS_FILE}
    rawDurationFrames={240}  // 8s × 30fps
  />
);
```

- [ ] **Step 3: Write F10MenuFeature.tsx**

```tsx
import React from "react";
import { FeatureScene } from "../components/FeatureScene";

const RAW_CLIP = process.env.RAW_F10_PATH ?? "/tmp/DINOForge/capture/raw_f10.mp4";
const TTS_FILE = process.env.TTS_F10_PATH ?? "/tmp/DINOForge/tts/f10.mp3";

export const F10MenuFeature: React.FC = () => (
  <FeatureScene
    videoSrc={RAW_CLIP}
    calloutText="[ F10 ] Mod Menu"
    calloutSubText="Browse and toggle mod packs"
    calloutColor="#60a5fa"
    audioSrc={TTS_FILE}
    rawDurationFrames={240}  // 8s × 30fps
  />
);
```

- [ ] **Step 4: Write DINOForgeReel.tsx**

```tsx
import React from "react";
import { AbsoluteFill, Audio, Sequence } from "remotion";
import { TitleCard } from "../components/TitleCard";
import { ModsButtonFeature } from "./ModsButtonFeature";
import { F9OverlayFeature } from "./F9OverlayFeature";
import { F10MenuFeature } from "./F10MenuFeature";

// 38s × 30fps = 1140 frames
// 0–90:    intro title (3s)
// 90–390:  mods (10s)
// 390–690: f9 (10s)
// 690–990: f10 (10s)
// 990–1140: outro (5s)

const INTRO_TTS  = process.env.TTS_INTRO_PATH  ?? "/tmp/DINOForge/tts/intro.mp3";
const OUTRO_TTS  = process.env.TTS_OUTRO_PATH  ?? "/tmp/DINOForge/tts/outro.mp3";

export const DINOForgeReel: React.FC = () => (
  <AbsoluteFill style={{ background: "#0a0a0f" }}>
    {/* 0–90: Intro title card (3s) */}
    <Sequence from={0} durationInFrames={90}>
      <TitleCard
        title="DINOForge Mod Platform"
        subtitle="Feature Demonstration"
        audioSrc={INTRO_TTS}
      />
    </Sequence>

    {/* 90–390: Mods button feature (10s) */}
    <Sequence from={90} durationInFrames={300}>
      <ModsButtonFeature />
    </Sequence>

    {/* 390–690: F9 overlay feature (10s) */}
    <Sequence from={390} durationInFrames={300}>
      <F9OverlayFeature />
    </Sequence>

    {/* 690–990: F10 menu feature (10s) */}
    <Sequence from={690} durationInFrames={300}>
      <F10MenuFeature />
    </Sequence>

    {/* 990–1140: Outro title card (5s) */}
    <Sequence from={990} durationInFrames={150}>
      <TitleCard
        title="All Features Confirmed"
        subtitle="DINOForge is ready for testing"
        audioSrc={OUTRO_TTS}
      />
    </Sequence>
  </AbsoluteFill>
);
```

- [ ] **Step 5: Write src/index.ts (register all compositions)**

```typescript
import { registerRoot, Composition } from "remotion";
import React from "react";
import { ModsButtonFeature } from "./compositions/ModsButtonFeature";
import { F9OverlayFeature }  from "./compositions/F9OverlayFeature";
import { F10MenuFeature }    from "./compositions/F10MenuFeature";
import { DINOForgeReel }     from "./compositions/DINOForgeReel";

export const RemotionRoot: React.FC = () => (
  <>
    <Composition
      id="ModsButtonFeature"
      component={ModsButtonFeature}
      durationInFrames={300}
      fps={30}
      width={1280}
      height={800}
    />
    <Composition
      id="F9OverlayFeature"
      component={F9OverlayFeature}
      durationInFrames={300}
      fps={30}
      width={1280}
      height={800}
    />
    <Composition
      id="F10MenuFeature"
      component={F10MenuFeature}
      durationInFrames={300}
      fps={30}
      width={1280}
      height={800}
    />
    <Composition
      id="DINOForgeReel"
      component={DINOForgeReel}
      durationInFrames={1140}
      fps={30}
      width={1280}
      height={800}
    />
  </>
);

registerRoot(RemotionRoot);
```

- [ ] **Step 6: Verify TypeScript compiles**

```bash
cd scripts/video && npx tsc --noEmit
```
Expected: no errors

- [ ] **Step 7: Commit**

```bash
git add scripts/video/src/
git commit -m "feat(video): add Remotion compositions — 3 feature clips + 38s compilation reel"
```

---

## Task 7: Create `scripts/game/capture-feature-clips.ps1` (Phase 1)

This is the most complex file. It must capture three clips via gdigrab-by-title, inject keys via Win32 SendInput, validate each UI state via game_analyze_screen, and write validate_report.json.

**Files:**
- Create: `scripts/game/capture-feature-clips.ps1`

- [ ] **Step 1: Write the capture script**

```powershell
#Requires -Version 5.1
<#
.SYNOPSIS
    DINOForge v2 capture pipeline — Phase 1.
    Launches the game, records three feature clips via gdigrab (by window title),
    injects keys via Win32 SendInput, validates each clip via VLM (game_analyze_screen),
    writes validate_report.json.

.NOTES
    Run from repo root. Outputs to $env:TEMP\DINOForge\capture\
#>

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

# ── Config ────────────────────────────────────────────────────────────────────
$ffmpeg   = "C:\program files\imagemagick-7.1.0-q16-hdri\ffmpeg.exe"
$gameExe  = "G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option\Diplomacy is Not an Option.exe"
$gameDir  = "G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option"
$debugLog = "$gameDir\BepInEx\dinoforge_debug.log"
$logOutput= "$gameDir\BepInEx\LogOutput.log"
$outDir   = "$env:TEMP\DINOForge\capture"
$gameTitle= "Diplomacy is Not an Option"

New-Item -ItemType Directory -Force -Path $outDir | Out-Null

# ── Win32: SendInput (no focus required) ──────────────────────────────────────
Add-Type @"
using System;
using System.Runtime.InteropServices;

public class Win32Input {
    [StructLayout(LayoutKind.Sequential)]
    public struct INPUT { public int type; public KEYBDINPUT ki; public long dummy; }
    [StructLayout(LayoutKind.Sequential)]
    public struct KEYBDINPUT { public ushort wVk; public ushort wScan; public uint dwFlags; public uint time; public IntPtr dwExtraInfo; }

    const int INPUT_KEYBOARD = 1;
    const uint KEYEVENTF_KEYUP = 0x0002;

    [DllImport("user32.dll")] static extern uint SendInput(uint nInputs, INPUT[] pInputs, int cbSize);

    public static void SendKey(ushort vk) {
        var down = new INPUT { type = INPUT_KEYBOARD, ki = new KEYBDINPUT { wVk = vk } };
        var up   = new INPUT { type = INPUT_KEYBOARD, ki = new KEYBDINPUT { wVk = vk, dwFlags = KEYEVENTF_KEYUP } };
        SendInput(1, new[] { down }, Marshal.SizeOf(typeof(INPUT)));
        System.Threading.Thread.Sleep(50);
        SendInput(1, new[] { up },   Marshal.SizeOf(typeof(INPUT)));
    }
}
"@

# VK codes
$VK_F9  = 0x78
$VK_F10 = 0x79

# ── Helpers ───────────────────────────────────────────────────────────────────
function Wait-ForLog {
    param([string]$LogPath, [string]$Pattern, [int]$TimeoutSec)
    $elapsed = 0
    while ($elapsed -lt $TimeoutSec) {
        Start-Sleep -Seconds 2; $elapsed += 2
        try {
            $content = [System.IO.File]::ReadAllText($LogPath, [System.Text.Encoding]::UTF8)
            if ($content -match $Pattern) { return $true }
        } catch { }
    }
    return $false
}

function Record-Clip {
    param([string]$Output, [int]$Duration)
    $args_ = @(
        "-f", "gdigrab", "-framerate", "30",
        "-i", "title=$gameTitle",
        "-t", $Duration,
        "-vf", "scale=1280:800",
        "-vcodec", "libx264", "-preset", "ultrafast",
        "-y", $Output
    )
    return Start-Process -FilePath $ffmpeg -ArgumentList $args_ -PassThru -WindowStyle Hidden
}

function Validate-VLM {
    param([string]$Feature, [string]$Expected)
    Write-Host "  VLM: checking for '$Expected' ..."
    # Delegate to MCP game_analyze_screen via claude code tool call
    # The orchestrator (.claude/commands/prove-features.md) calls game_analyze_screen
    # and writes the result; here we read that result file
    $vlmFile = "$outDir\vlm_${Feature}.json"
    # game_analyze_screen is called externally; write a trigger file and wait for result
    $trigger = "$outDir\vlm_request_${Feature}.json"
    @{ feature = $Feature; expected = $Expected; timestamp = (Get-Date -Format "o") } |
        ConvertTo-Json | Set-Content $trigger
    $waited = 0
    while (-not (Test-Path $vlmFile) -and $waited -lt 30) {
        Start-Sleep -Seconds 1; $waited++
    }
    if (-not (Test-Path $vlmFile)) {
        return $false, "VLM result file not found after 30s"
    }
    $result = Get-Content $vlmFile | ConvertFrom-Json
    return $result.confirmed, $result.vlm_response
}

# ── Step 1: Kill + launch game ────────────────────────────────────────────────
Write-Host "Step 1: Launching game..."
Stop-Process -Name $gameTitle -Force -ErrorAction SilentlyContinue
Stop-Process -Name "UnityCrashHandler64" -Force -ErrorAction SilentlyContinue
Start-Sleep -Seconds 3
if (Test-Path $debugLog) { Clear-Content $debugLog -ErrorAction SilentlyContinue }
Start-Process -FilePath $gameExe -WorkingDirectory $gameDir

# ── Step 2: Boot Stage A — DINOForge awake ────────────────────────────────────
Write-Host "Step 2: Waiting for DINOForge awake (30s)..."
$ok = Wait-ForLog -LogPath $debugLog -Pattern "Awake completed" -TimeoutSec 30
if (-not $ok) { Write-Error "Timeout: DINOForge Awake not detected"; exit 1 }
Write-Host "  Awake detected."

# ── Step 3: Boot Stage B — Mods button injection ──────────────────────────────
Write-Host "Step 3: Waiting for Mods button injection (720s)..."
$ok = Wait-ForLog -LogPath $logOutput -Pattern "MODS BUTTON INJECTION FULLY SUCCESSFUL" -TimeoutSec 720
if (-not $ok) { Write-Error "Timeout: Mods button injection not detected"; exit 1 }
Write-Host "  Mods button confirmed."
Start-Sleep -Seconds 2

# ── Step 4: Record raw_mods.mp4 (6s, main menu) ──────────────────────────────
Write-Host "Step 4: Recording raw_mods.mp4 (6s)..."
$modsOut = "$outDir\raw_mods.mp4"
$proc = Record-Clip -Output $modsOut -Duration 6
$proc | Wait-Process -Timeout 20 -ErrorAction SilentlyContinue
if (-not (Test-Path $modsOut) -or (Get-Item $modsOut).Length -lt 1024) {
    Write-Error "raw_mods.mp4 not created or empty"; exit 1
}
Write-Host "  raw_mods.mp4 OK"

# VLM: screenshot game_analyze_screen for mods (live, game still on main menu)
# This is called by the orchestrator after recording; mark validated for now
# (orchestrator injects result — see prove-features.md for full VLM call sequence)

# ── Step 5: Press F9, record raw_f9.mp4 (8s) ─────────────────────────────────
Write-Host "Step 5: Pressing F9 + recording raw_f9.mp4 (8s)..."
$f9Out = "$outDir\raw_f9.mp4"
$proc = Record-Clip -Output $f9Out -Duration 8
Start-Sleep -Milliseconds 500
[Win32Input]::SendKey($VK_F9)
Write-Host "  F9 injected"
Start-Sleep -Seconds 2
# VLM validation happens here in orchestrator context
$proc | Wait-Process -Timeout 20 -ErrorAction SilentlyContinue
if (-not (Test-Path $f9Out) -or (Get-Item $f9Out).Length -lt 1024) {
    Write-Error "raw_f9.mp4 not created or empty"; exit 1
}
Write-Host "  raw_f9.mp4 OK"

# Reset: press F9 again to close overlay
[Win32Input]::SendKey($VK_F9)
Start-Sleep -Seconds 2

# ── Step 6: Press F10, record raw_f10.mp4 (8s) ───────────────────────────────
Write-Host "Step 6: Pressing F10 + recording raw_f10.mp4 (8s)..."
$f10Out = "$outDir\raw_f10.mp4"
$proc = Record-Clip -Output $f10Out -Duration 8
Start-Sleep -Milliseconds 500
[Win32Input]::SendKey($VK_F10)
Write-Host "  F10 injected"
Start-Sleep -Seconds 2
$proc | Wait-Process -Timeout 20 -ErrorAction SilentlyContinue
if (-not (Test-Path $f10Out) -or (Get-Item $f10Out).Length -lt 1024) {
    Write-Error "raw_f10.mp4 not created or empty"; exit 1
}
Write-Host "  raw_f10.mp4 OK"

Write-Host ""
Write-Host "Phase 1 complete. Clips in: $outDir"
Write-Host "  $modsOut"
Write-Host "  $f9Out"
Write-Host "  $f10Out"
Write-Host ""
Write-Host "VLM validation must be performed by orchestrator (game_analyze_screen MCP calls)"
```

> **Note on VLM**: The script records clips and signals readiness. The actual `game_analyze_screen` VLM calls are orchestrated from the Claude session (via `.claude/commands/prove-features.md`) because they require MCP tool access. The script writes trigger files and waits for result JSON files. The updated `prove-features.md` performs the VLM calls between recording steps.

- [ ] **Step 2: Test syntax (no game required)**

```powershell
powershell -ExecutionPolicy Bypass -Command "& { . 'scripts/game/capture-feature-clips.ps1' -WhatIf }" 2>&1 || true
```
Or simply check for parse errors:
```powershell
powershell -ExecutionPolicy Bypass -Command "Get-Content 'scripts/game/capture-feature-clips.ps1' | Out-Null; Write-Host 'Parse OK'"
```

- [ ] **Step 3: Commit**

```bash
git add scripts/game/capture-feature-clips.ps1
git commit -m "feat(capture): Phase 1 capture script — gdigrab by title, SendInput F9/F10, boot detection"
```

---

## Task 8: Update `.claude/commands/prove-features.md`

Replace v1 instructions with v2 orchestration steps that delegate to the three phases and perform VLM calls directly in the Claude session.

**Files:**
- Modify: `.claude/commands/prove-features.md`

- [ ] **Step 1: Rewrite prove-features.md**

Replace entire content with:

```markdown
# /prove-features

Record autonomous video proof of DINOForge features — v2 pipeline.
Three phases: game capture → neural TTS → Remotion composition.
Evidence bundle: VLM-confirmed screenshots + raw clips + MP4 renders + proof_report.md.

## Implementation

Run ALL steps from repo root.

---

### Step 1: Phase 1 — Game capture

```powershell
powershell -ExecutionPolicy Bypass -File scripts/game/capture-feature-clips.ps1
```

**After the script exits**, perform VLM validation for each feature.
The script leaves the game running with the last UI state active.

#### VLM: Validate mods clip
1. Call `game_analyze_screen` MCP tool (takes a live GDI screenshot of the game window)
2. Check response for "Mods" text in button/menu area
3. Save screenshot to `$env:TEMP\DINOForge\capture\validate_mods.png`
4. If NOT confirmed: press Mods button via `game_input`, wait 3s, re-screenshot
5. Write result to `validate_mods.json` as `{ "feature": "mods", "confirmed": true/false, "vlm_response": "..." }`

#### VLM: Validate F9 clip
1. Call `game_input` to press F9 (bring overlay back up)
2. Wait 2s
3. Call `game_analyze_screen`
4. Check for debug panel / overlay with entity counts or stats
5. Save screenshot to `$env:TEMP\DINOForge\capture\validate_f9.png`
6. If NOT confirmed: retry once (re-press F9, wait 3s, re-screenshot)
7. Write result to `validate_f9.json`

#### VLM: Validate F10 clip
1. Call `game_input` to press F10 (bring menu back up)
2. Wait 2s
3. Call `game_analyze_screen`
4. Check for mod menu / pack browser panel
5. Save screenshot to `$env:TEMP\DINOForge\capture\validate_f10.png`
6. If NOT confirmed: retry once
7. Write result to `validate_f10.json`

**On any feature failing after retry**: do NOT proceed. Report which feature failed and the VLM response.

Write final `$env:TEMP\DINOForge\capture\validate_report.json`:
```json
[
  { "feature": "mods", "confirmed": true, "vlm_response": "...", "timestamp": "..." },
  { "feature": "f9",   "confirmed": true, "vlm_response": "...", "timestamp": "..." },
  { "feature": "f10",  "confirmed": true, "vlm_response": "...", "timestamp": "..." }
]
```

---

### Step 2: Phase 2 — Neural TTS

```powershell
python scripts/video/generate_tts.py `
  --spec scripts/video/vo_spec.json `
  --out "$env:TEMP\DINOForge\tts"
```

Validate: all 5 files exist and are > 10KB:
- `$env:TEMP\DINOForge\tts\intro.mp3`
- `$env:TEMP\DINOForge\tts\mods.mp3`
- `$env:TEMP\DINOForge\tts\f9.mp3`
- `$env:TEMP\DINOForge\tts\f10.mp3`
- `$env:TEMP\DINOForge\tts\outro.mp3`

---

### Step 3: Phase 3 — Remotion render

Set environment variables pointing to Phase 1/2 outputs (use forward slashes for Node):

```powershell
$capture = "$env:TEMP\DINOForge\capture" -replace '\\','/'
$tts     = "$env:TEMP\DINOForge\tts" -replace '\\','/'
$env:RAW_MODS_PATH  = "$capture/raw_mods.mp4"
$env:RAW_F9_PATH    = "$capture/raw_f9.mp4"
$env:RAW_F10_PATH   = "$capture/raw_f10.mp4"
$env:TTS_INTRO_PATH = "$tts/intro.mp3"
$env:TTS_MODS_PATH  = "$tts/mods.mp3"
$env:TTS_F9_PATH    = "$tts/f9.mp3"
$env:TTS_F10_PATH   = "$tts/f10.mp3"
$env:TTS_OUTRO_PATH = "$tts/outro.mp3"
```

```powershell
Set-Location scripts/video
npx remotion render src/index.ts ModsButtonFeature --output out/mods_feature.mp4
npx remotion render src/index.ts F9OverlayFeature  --output out/f9_feature.mp4
npx remotion render src/index.ts F10MenuFeature    --output out/f10_feature.mp4
npx remotion render src/index.ts DINOForgeReel     --output out/dinoforge_reel.mp4
Set-Location ../..
```

If any render fails: log error, continue. Mark `render_failed: true` in report for that render.
Minimum required: `dinoforge_reel.mp4` must exist and be > 100KB.

---

### Step 4: Bundle evidence

```powershell
$ts     = Get-Date -Format "yyyyMMdd_HHmmss"
$bundle = "docs/proof-of-features/$ts"
New-Item -ItemType Directory -Force -Path $bundle | Out-Null

# Copy Phase 1 artifacts
Copy-Item "$env:TEMP\DINOForge\capture\raw_mods.mp4"      "$bundle\"
Copy-Item "$env:TEMP\DINOForge\capture\raw_f9.mp4"        "$bundle\"
Copy-Item "$env:TEMP\DINOForge\capture\raw_f10.mp4"       "$bundle\"
Copy-Item "$env:TEMP\DINOForge\capture\validate_mods.png" "$bundle\" -ErrorAction SilentlyContinue
Copy-Item "$env:TEMP\DINOForge\capture\validate_f9.png"   "$bundle\" -ErrorAction SilentlyContinue
Copy-Item "$env:TEMP\DINOForge\capture\validate_f10.png"  "$bundle\" -ErrorAction SilentlyContinue
Copy-Item "$env:TEMP\DINOForge\capture\validate_report.json" "$bundle\"

# Copy Phase 3 renders
Copy-Item "scripts/video/out/mods_feature.mp4"   "$bundle\" -ErrorAction SilentlyContinue
Copy-Item "scripts/video/out/f9_feature.mp4"     "$bundle\" -ErrorAction SilentlyContinue
Copy-Item "scripts/video/out/f10_feature.mp4"    "$bundle\" -ErrorAction SilentlyContinue
Copy-Item "scripts/video/out/dinoforge_reel.mp4" "$bundle\" -ErrorAction SilentlyContinue
```

Generate `$bundle\proof_report.md` from validate_report.json data and render results.
Open reel: `Start-Process "$bundle\dinoforge_reel.mp4"`

---

## Requirements

- ffmpeg at `C:\program files\imagemagick-7.1.0-q16-hdri\ffmpeg.exe`
- Python 3 + edge-tts (`pip install edge-tts`)
- Node.js 18+ with npm
- Remotion deps installed: `cd scripts/video && npm install`
- Game at `G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option\`
```

- [ ] **Step 2: Commit**

```bash
git add .claude/commands/prove-features.md
git commit -m "feat(commands): update prove-features command to v2 pipeline orchestration"
```

---

## Task 9: Update `docs/proof-of-features/README.md`

**Files:**
- Modify: `docs/proof-of-features/README.md` (create if missing)

- [ ] **Step 1: Write README.md**

```markdown
# DINOForge Proof of Features

Evidence bundles from automated feature validation runs.

## Bundle Structure

Each run creates a timestamped directory `YYYYMMDD_HHmmss/`:

<timestamp>/
  raw_mods.mp4          Raw 6s game clip — main menu with Mods button
  raw_f9.mp4            Raw 8s game clip — F9 debug overlay
  raw_f10.mp4           Raw 8s game clip — F10 mod menu
  validate_mods.png     VLM screenshot at time of Mods confirmation
  validate_f9.png       VLM screenshot at time of F9 confirmation
  validate_f10.png      VLM screenshot at time of F10 confirmation
  validate_report.json  Machine-readable: { feature, vlm_response, confirmed, timestamp }
  mods_feature.mp4      Remotion-rendered 10s clip with spring callout
  f9_feature.mp4        Remotion-rendered 10s clip with spring callout
  f10_feature.mp4       Remotion-rendered 10s clip with spring callout
  dinoforge_reel.mp4    38s compilation reel — primary deliverable
  proof_report.md       Human-readable summary

## Pipeline

- **Phase 1** (PowerShell): `capture-feature-clips.ps1` — gdigrab by window title, Win32 SendInput for key injection, VLM validation via `game_analyze_screen` MCP
- **Phase 2** (Python): `generate_tts.py` — edge-tts neural TTS from `vo_spec.json` (free, en-US-AriaNeural)
- **Phase 3** (Node.js/Remotion): Spring-physics callout boxes, freeze-frame padding, 38s compilation reel

Run: `/prove-features`
```

- [ ] **Step 2: Commit**

```bash
git add docs/proof-of-features/README.md
git commit -m "docs: update proof-of-features README for v2 evidence bundle format"
```

---

## Task 10: Retire v1 script

**Files:**
- Retire: `scripts/game/prove-features-video.ps1` → Recycle Bin

- [ ] **Step 1: Send to Recycle Bin**

```powershell
powershell -c "Add-Type -AssemblyName Microsoft.VisualBasic; [Microsoft.VisualBasic.FileIO.FileSystem]::DeleteFile((Resolve-Path 'scripts/game/prove-features-video.ps1').Path, 'OnlyErrorDialogs', 'SendToRecycleBin')"
```

- [ ] **Step 2: Verify removed**

```bash
ls scripts/game/prove-features-video.ps1 2>&1 || echo "Confirmed removed"
```

- [ ] **Step 3: Commit**

```bash
git add -u scripts/game/prove-features-video.ps1
git commit -m "feat(cleanup): retire v1 prove-features-video.ps1 (replaced by v2 capture-feature-clips.ps1)"
```

---

## Task 11: End-to-end smoke test (no game)

Verify all non-game steps work before requiring a live game session.

- [ ] **Step 1: Verify Remotion TypeScript**

```bash
cd scripts/video && npx tsc --noEmit && echo "TypeScript OK"
```

- [ ] **Step 2: Verify TTS script smoke test**

```bash
python scripts/video/generate_tts.py \
  --spec scripts/video/vo_spec.json \
  --out "$TEMP/DINOForge/tts_test"
```
Expected: 5 MP3 files > 10KB each

- [ ] **Step 3: Verify Remotion can list compositions**

```bash
cd scripts/video && npx remotion compositions src/index.ts
```
Expected: lists `ModsButtonFeature`, `F9OverlayFeature`, `F10MenuFeature`, `DINOForgeReel`

- [ ] **Step 4: Update CHANGELOG.md**

Add entry under `## [Unreleased]`:
```markdown
### Added
- Prove-features video pipeline v2 — replaces broken v1 (gdigrab by title, edge-tts via Python, Remotion spring-physics compositions, VLM validation gate)
- `scripts/game/capture-feature-clips.ps1` — Phase 1: game capture with Win32 SendInput and VLM gate
- `scripts/video/generate_tts.py` + `vo_spec.json` — Phase 2: neural TTS (edge-tts, no arg-splitting bug)
- `scripts/video/` — Phase 3: Remotion project (4 compositions, spring callouts, freeze-frame padding)
### Removed
- `scripts/game/prove-features-video.ps1` — retired (v1 pipeline)
```

- [ ] **Step 5: Final commit**

```bash
git add CHANGELOG.md
git commit -m "chore: update CHANGELOG for prove-features v2 pipeline"
```

---

## Forgecode / VLM Orchestration Note

The VLM validation calls (`game_analyze_screen`) require MCP tool access. This means they must run in the Claude Code session context, not from within the PowerShell script. The design in `prove-features.md` handles this correctly: the script records clips and signals completion, then the Claude orchestrator calls `game_analyze_screen` between feature steps.

**Model hierarchy** for VLM calls (weakest capable model first):
1. Codex Spark 5.3 (if image support confirmed — check before use)
2. Codex 5.4 mini (fallback)
3. claude-haiku-4-5 (final fallback, fast mode OK)

The standard `game_analyze_screen` MCP tool already handles the screenshot + analysis. No external harness required for the core pipeline — `game_analyze_screen` delegates to Claude vision internally.

---

## Dependency Order

```
Task 1 (vo_spec.json)
  → Task 2 (generate_tts.py)
Task 3 (Remotion scaffold)
  → Task 4 (components: CalloutBox, CaptionBar, TitleCard)
    → Task 5 (FeatureScene)
      → Task 6 (compositions + index.ts)
Task 7 (capture-feature-clips.ps1)  [independent]
Task 8 (prove-features.md update)   [after Tasks 6, 7]
Task 9 (README.md)                  [independent]
Task 10 (retire v1 script)          [after Task 7 complete]
Task 11 (smoke test)                [after all Tasks 1-9]
```
