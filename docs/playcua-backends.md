# Isolation Layer & playCUA Backend Selection

The DINOForge MCP uses an abstraction layer (`isolation_layer.py`) to support multiple backends for game capture, input injection, and window management. This enables platform abstraction (Windows/Linux/macOS), cross-deployment flexibility (hidden desktop, Docker/K8s-ready), and graceful fallback.

## Backend Implementations

1. **HiddenDesktopBackend** (Windows-only, Tier 2 — current stable)
   - Win32 `CreateDesktopW()` for isolated desktop creation; used by `game_launch(hidden=True)`.
   - Pros: battle-tested, no external deps. Cons: Windows-only.

2. **PlayCUABackend** (Cross-platform, Tier 1 preferred)
   - Routes to playCUA JSON-RPC server (port 9000). Provides screenshot capture, input injection, window enumeration, process management, image analysis.
   - Pros: cross-platform, abstracted, Docker/K8s compatible. Cons: requires playCUA binary or `cargo run`.
   - Platform adapters: Windows (WGC, SendInput, EnumWindowsEx), Linux (X11, uinput, EWMH), macOS (CoreGraphics, Quartz, NSWorkspace).

3. **DockerBackend** (Stub, Tier 1 future) — headless/CI game automation in containers. Planned v0.20+.

## Backend Selection

```python
context = IsolationContext.get('auto')          # tries playCUA, falls back to HiddenDesktop
context = IsolationContext.get('hidden_desktop') # Windows only
context = IsolationContext.get('playcua')        # cross-platform
```

## Starting playCUA Server

```bash
# From cargo:
cd C:\Users\koosh\playcua_ci_test && cargo run -- --listen 127.0.0.1:9000
# Compiled binary:
./playcua.exe --listen 127.0.0.1:9000
# Via script:
./scripts/start-playcua.ps1
```

playCUA module surface: **analysis** (image diff, BLAKE3), **input** (kb/mouse injection), **window** (enumeration, focus, hidden desktop), **process** (launch/kill/status lifecycle), **capture** (WGC/X11/CG screenshots). See `docs/playCUA_integration_audit.md` and `docs/playcua_phase3_5_spec.md`.

## Display Isolation Tier Chain

- Tier 1 (future): VDD driver-based isolation (dedicated DINOForge IDD/WDDM virtual display — headless launch without Parsec VDD or CreateDesktop)
- Tier 2 (current): Win32 CreateDesktop (HiddenDesktopBackend)
- Tier 3: playCUA (cross-platform)
- Tier 4 (future): Docker/Kubernetes (DockerBackend)
