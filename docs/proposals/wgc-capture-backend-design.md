# WGC Capture Backend for DINOForge MCP — Design (#537)

**Status**: DRAFT — design + scaffold (no implementation yet)
**Date**: 2026-05-19
**Owner**: agent-driven
**Related**: #536 (autonomous game-launch screenshot pipeline), iter-143 #529

## Problem

The current `game_screenshot` MCP tool routes through `GameControlCli "screenshot"` →
named-pipe RPC → `Unity ScreenCapture.CaptureScreenshot` (GPU back-buffer). This works
for healthy gameplay but has **two failure modes that motivate #537**:

1. **DXGI exclusive fullscreen**: GDI / `PrintWindow` / `BitBlt` / `CopyFromScreen`
   return solid-black frames because the DWM-composited surface is bypassed.
2. **Hung game process**: the named pipe never responds. Tier 1-5 fallbacks
   (GDI / PrintWindow / bare-cua-via-pipe / `game-control screenshot` /
   `CopyFromScreen`) all depend on either the pipe or a desktop-window GDI surface
   and silently produce empty PNGs.

A **WGC (Windows.Graphics.Capture)** backend captures the DXGI back-buffer
**directly**, independent of game responsiveness, and is the standard Microsoft
solution for DirectX/game windows on Windows 10 1903+.

## Survey

### Rust (`windows-rs` crate) — preferred

`C:\Users\koosh\playcua_ci_test\native\Cargo.toml` (bare-cua, already a DINOForge
dependency) ALREADY declares **every WGC feature flag we need**:

- `Graphics_Capture`, `Graphics_DirectX_Direct3D11`
- `Win32_System_WinRT_Direct3D11`, `Win32_System_WinRT_Graphics_Capture`
- `Win32_Graphics_Direct3D11`, `Win32_Graphics_Dxgi`, `Win32_Graphics_Dxgi_Common`

And `native/src/adapters/windows/wgc.rs` (307 lines) **already implements**:

- `WgcCapture::capture_window(title)` with `FindWindowW` → `IGraphicsCaptureItemInterop::CreateForWindow`
- `Direct3D11CaptureFramePool::CreateFreeThreaded` + `FrameArrived` callback
- D3D11 staging texture `Map` for CPU readback
- BGRA→RGBA swap + PNG encode via `image` crate
- Auto-fallback to `xcap` if `FindWindowW` fails or any step errors

`app/mod.rs::build_capture()` already wires `WgcCapture` as the **primary** Windows
backend, with `XcapCapture` as fallback. The JSON-RPC `screenshot` method routes
`window_title` → `capture.capture_window(...)` → WgcCapture.

**Status**: Already compiled into the shipped binary at
`C:\Users\koosh\playcua_ci_test\target\release\bare-cua-native.exe`.

### Python — secondary option

| Package      | Status                          | WGC API access                       |
| ------------ | ------------------------------- | ------------------------------------ |
| `winsdk`     | Maintained; PyPI; pure projection over WinRT IDL | Full — `winsdk.windows.graphics.capture` |
| `windows-capture` (`@y-koz`) | High-level wrapper over WGC; PyPI | `WindowsCapture(window_name=...)` |
| `pywinrt`    | Microsoft's official; broader   | Full but verbose                    |
| `winrt-runtime` (Python 3.10+) | Microsoft's base; required by winsdk | Indirect    |

None of these are currently in `pyproject.toml`. Closest pre-existing path is
`ctypes` calls in `isolation_layer.py` — but **WGC requires apartment-threaded COM +
WinRT activation + D3D11 device handle marshalling**, which is doable in pure
ctypes but ~600+ LoC vs. ~30 lines via `windows-capture`.

## Two Options

### Option A — Rust (use existing bare-cua WGC) **[RECOMMENDED]**

**What changes**:
1. Add a new MCP tool `game_screenshot_wgc(window_title, output_path)` in
   `server.py` that **bypasses** `GameControlCli` and invokes `bare-cua-native`
   directly via `isolation_layer.PlayCUABackend.capture_window(title)`.
2. Update existing `game_screenshot` to **try WGC first** (via the isolation
   layer) before falling back to `GameControlCli`.

**Pros**:
- **ZERO new dependencies** — `windows-rs` + WGC code is already shipping.
- **Already isolated** — runs in a separate process (bare-cua subprocess), so a
  WGC failure / crash never takes down the MCP server.
- **Battle-tested**: `wgc.rs` is in production use by playCUA / bare-cua.
- **Cross-deployment** — same code path works in Docker / hidden desktop / VDD.

**Cons**:
- Adds subprocess round-trip latency (~10-30 ms). Acceptable for screenshot use.
- Requires `bare-cua-native.exe` to be present (already required by Tier-2
  HiddenDesktopBackend + PlayCUABackend).

**Effort**: **~50 LoC Python** — wire a new tool + extend fallback chain.

### Option B — Python in-process (`winsdk`)

**What changes**:
1. Add `winsdk>=1.0.0b10` to `[project.optional-dependencies] capture` in
   `pyproject.toml`.
2. New file `dinoforge_mcp/capture_wgc.py` with `~120 lines` reproducing the
   `wgc.rs` flow (CreateForWindow → FramePool → staging texture → PNG).
3. Wire into `game_screenshot` as a new fallback tier.

**Pros**:
- In-process — no subprocess fork.
- Smaller marginal cost when MCP server is already running.

**Cons**:
- **Duplicates Rust implementation** — two WGC code paths to maintain (CLAUDE.md
  "wrap, don't handroll").
- **Apartment-threading hazard**: WGC requires `RoInitialize(SINGLE_THREADED)`
  on the calling thread. FastMCP uses asyncio + thread-pool executors; pinning a
  worker thread to STA is non-trivial.
- **Heavier deps**: `winsdk` pulls `winrt-runtime` (~30 MB compiled wheels).
- **More LoC + tests** to write from scratch.

**Effort**: **~200 LoC Python + tests** — larger surface, novel code, COM hazards.

## Recommendation: **Option A**

Rationale (in priority order):
1. **CLAUDE.md "Wrap, don't handroll"** is the governing rule — bare-cua already
   has a working WGC adapter; writing a second one violates the doctrine.
2. **Already shipping** — `bare-cua-native.exe` is on disk, compiled, and the
   `screenshot` JSON-RPC method already routes to `WgcCapture` first via
   `app/mod.rs::build_capture()`.
3. Effort is **~10x smaller** (50 LoC vs 200 LoC + dependencies).
4. Isolation — subprocess boundary protects MCP from WGC/D3D11 crashes.

## API Contract

```python
# New tool added to dinoforge_mcp/server.py
@mcp.tool()
async def game_screenshot_wgc(
    ctx: Context,
    window_title: str = "Diplomacy is Not an Option",
    output_path: str | None = None,
    timeout_seconds: float = 5.0,
) -> dict:
    """
    Capture the game window directly via Windows.Graphics.Capture (WGC).
    Works on hung/unresponsive Unity processes and DXGI exclusive fullscreen.
    Bypasses the GameBridge named pipe entirely.

    Returns:
        {
            "success": bool,
            "path": str | None,    # path to PNG file on disk
            "width": int,
            "height": int,
            "backend": "wgc",
            "elapsed_ms": float,
            "error": str | None,   # populated on failure
        }
    """
```

**Routing inside `game_screenshot` (modified)**:

```
1. Try bare-cua WGC (window_title="Diplomacy is Not an Option")
   - 1s timeout: if `FindWindowW` returns null OR FramePool times out, fall through
2. Fall back to GameControlCli (current behavior)
3. Final fallback: PrintWindow via ctypes (HiddenDesktopBackend)
```

**Error semantics**:

| Failure                              | Returned error                                          |
| ------------------------------------ | ------------------------------------------------------- |
| Window not found                     | `{"error": "wgc: window not found: <title>"}`         |
| WGC unavailable (older Windows)      | `{"error": "wgc: graphics capture unavailable"}`      |
| D3D11 device creation failed         | `{"error": "wgc: d3d11 device init failed: <msg>"}`   |
| Frame timeout                        | `{"error": "wgc: frame timeout after 5s"}`            |
| bare-cua binary missing              | `{"error": "wgc: bare-cua-native not found at <path>"}` |

## Test Plan

1. **Healthy DXGI fullscreen** — launch DINO, switch to fullscreen (alt+enter),
   call `game_screenshot_wgc`. Expect non-black PNG with width/height matching
   monitor.
2. **Hung process** — `Suspend-Process` on the game; call `game_screenshot_wgc`.
   Expect successful capture (WGC reads GPU surface independent of CPU).
3. **Black-frame validator** — `cv2.mean(img) < 5.0` ⇒ regression. Existing
   `bare_cua_diagnostic.ps1` already has this check; reuse it.
4. **Window-not-found** — kill DINO, call tool. Expect
   `success=False, error="wgc: window not found"` in <1s.
5. **Vs. GDI baseline** — capture same scene through `PrintWindow` and WGC; pHash
   the two; WGC should match the rendered scene, GDI should be near-black.
6. **CI gate** — add to `scripts/proof/preflight-runbook.ps1` so the headless
   automation path uses WGC by default.

## Prerequisites

- Windows 10 1903+ (build 18362+) — WGC API requirement. CI runners on `windows-2022`
  satisfy.
- `bare-cua-native.exe` present at `C:\Users\koosh\playcua_ci_test\target\release\`
  (or env-resolved path via `BARE_CUA_NATIVE_PATH`).
- No COM/apartment changes needed in MCP server (subprocess boundary handles it).

## Effort Estimate

| Phase                                      | LoC | Wall-clock |
| ------------------------------------------ | --- | ---------- |
| `capture_wgc.py` scaffold (this round)     | 50  | done       |
| Wire into `game_screenshot` fallback chain | 30  | 30 min     |
| Add new `game_screenshot_wgc` MCP tool     | 40  | 30 min     |
| Tests (3 happy + 3 sad paths)              | 120 | 1 h        |
| Doc updates (CLAUDE.md MCP table)          | 10  | 15 min     |
| **Total**                                  | 250 | ~2.5 h     |

## Blockers Identified

1. **bare-cua path is hard-coded** in `isolation_layer.py:566` to
   `C:\Users\koosh\playcua_ci_test\target\release\bare-cua-native.exe`. CI
   runners and other contributors will not have it. Need a resolver (env-var +
   repo-relative fallback) — already partially exists via
   `NativeDepResolver` (`native_dep_resolver.py`).
2. **PlayCUAClient lifecycle**: current code spawns one bare-cua subprocess per
   `IsolationContext`; if the binary crashes mid-capture, recovery semantics are
   not well-specified. Acceptable for v1; harden in #536.
3. **Unicode window-title round-trip**: `FindWindowW` requires UTF-16; the
   bare-cua JSON-RPC currently passes UTF-8 strings. Verified working in
   `wgc.rs:90-92` (HSTRING::from(&str) auto-converts), so no action needed.

## Decision Log

| Date       | Decision                                                |
| ---------- | ------------------------------------------------------- |
| 2026-05-19 | Adopt Option A (bare-cua WGC) per CLAUDE.md wrap rule. |
| 2026-05-19 | Scaffold `capture_wgc.py` as thin Python adapter only. |
| 2026-05-19 | Defer in-process winsdk path; revisit if subprocess overhead > 100 ms p95. |
