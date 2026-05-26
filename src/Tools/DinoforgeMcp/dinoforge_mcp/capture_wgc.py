"""
WGC (Windows.Graphics.Capture) backend for DINOForge MCP screenshot pipeline (#537).

DESIGN: docs/proposals/wgc-capture-backend-design.md

This module is a THIN PYTHON ADAPTER over bare-cua-native's WGC implementation
(`C:\\Users\\koosh\\playcua_ci_test\\native\\src\\adapters\\windows\\wgc.rs`). It does
NOT contain its own WGC code — bare-cua already wraps `windows-rs` correctly,
and CLAUDE.md "wrap, don't handroll" applies.

bare-cua is a pure stdio JSON-RPC 2.0 server (no CLI flags — `--help` causes
it to start the daemon and exit on EOF). We invoke it via `PlayCUAClient`
already defined in `isolation_layer.py`, calling the `screenshot` method with
`{"window_title": <title>}`, which routes to `WgcCapture.capture_window()` on
Windows targets per `playcua_ci_test/native/src/app/mod.rs::build_capture()`.

The wider routing strategy:

    game_screenshot (server.py)
        |
        +-- Tier 1: capture_wgc.capture_window_via_wgc()        # this module
        |       \\-> isolation_layer.PlayCUABackend.capture_window()
        |             \\-> bare-cua-native JSON-RPC "screenshot"
        |                   \\-> WgcCapture (Rust) -> WGC framepool -> PNG
        |
        +-- Tier 2: _run_game_cli("screenshot")                 # named-pipe
        |
        +-- Tier 3: PrintWindow via ctypes (HiddenDesktopBackend)
"""

from __future__ import annotations

import asyncio
import logging
import os
import time
from dataclasses import dataclass, field
from pathlib import Path
from typing import Optional

logger = logging.getLogger(__name__)

# Default window title for DINO. Tooling that supports other games should pass
# explicit titles.
DEFAULT_DINO_WINDOW_TITLE = "Diplomacy is Not an Option"

# Hard cap so a hung capture cannot lock the MCP request. The Rust side already
# enforces 5s on the FrameArrived channel; this gives the subprocess RPC layer
# a generous overhead.
WGC_CAPTURE_TIMEOUT_SECONDS = 8.0

# Default bare-cua-native location. PlayCUABackend's default points at a
# `native/target/release/...` nested path that does NOT actually exist on this
# machine — the real binary lives directly under `target/release/`. We resolve
# via DF_BARE_CUA_PATH env var first, then this default. Keep this in sync with
# isolation_layer.PlayCUABackend.__init__ when reconciled (separate task).
_DEFAULT_BARE_CUA_PATH = r"C:\Users\koosh\playcua_ci_test\target\release\bare-cua-native.exe"


@dataclass
class WgcCaptureResult:
    """Result of a single WGC capture attempt."""

    success: bool
    path: Optional[Path] = None
    width: int = 0
    height: int = 0
    elapsed_ms: float = 0.0
    backend: str = "wgc"
    error: Optional[str] = None

    def to_dict(self) -> dict:
        """Serialize for MCP tool return."""
        return {
            "success": self.success,
            "path": str(self.path) if self.path else None,
            "width": self.width,
            "height": self.height,
            "backend": self.backend,
            "elapsed_ms": round(self.elapsed_ms, 2),
            "error": self.error,
        }


def _resolve_bare_cua_path() -> Optional[Path]:
    """
    Resolve bare-cua-native.exe location. Returns Path if found, None otherwise.

    Resolution order:
      1. DF_BARE_CUA_PATH env var (operator override)
      2. BARE_CUA_NATIVE_PATH env var (legacy name from design doc)
      3. Default location at top-of-file
    """
    for env_name in ("DF_BARE_CUA_PATH", "BARE_CUA_NATIVE_PATH"):
        v = os.environ.get(env_name)
        if v:
            p = Path(v)
            if p.exists():
                return p
            logger.warning(
                "%s=%s set but binary not found at that path; falling through",
                env_name,
                v,
            )

    default = Path(_DEFAULT_BARE_CUA_PATH)
    if default.exists():
        return default
    return None


def check_wgc_available() -> bool:
    """
    Probe whether the WGC capture path is reachable end-to-end.

    Returns True iff:
      1. The bare-cua-native binary is on disk at the expected path
         (or DF_BARE_CUA_PATH / BARE_CUA_NATIVE_PATH env var is set and points
         at a valid exe).

    Note: we deliberately do NOT spawn the subprocess here — that would force
    JIT cold-start on every health check. Capture itself will surface deeper
    failures (Windows < 1903, COM init failure, etc.) via the error string.
    """
    return _resolve_bare_cua_path() is not None


def _default_temp_path() -> Path:
    """Return a per-call PNG path under %TEMP%/DINOForge/wgc/."""
    import tempfile

    base = Path(tempfile.gettempdir()) / "DINOForge" / "wgc"
    base.mkdir(parents=True, exist_ok=True)
    ts = int(time.time() * 1000)
    return base / f"wgc_capture_{ts}.png"


async def capture_window_via_wgc(
    window_title: str = DEFAULT_DINO_WINDOW_TITLE,
    output_path: Optional[Path] = None,
    timeout_seconds: float = WGC_CAPTURE_TIMEOUT_SECONDS,
) -> WgcCaptureResult:
    """
    Capture the named window via WGC (Windows.Graphics.Capture).

    Implementation: delegates to bare-cua-native subprocess via
    isolation_layer.PlayCUABackend. WGC is bare-cua's primary capture path on
    Windows when the `windows` crate features are enabled (they are — see
    playcua_ci_test/native/Cargo.toml lines 33-50).

    Args:
        window_title: Window title to find via FindWindowW (UTF-16 internally).
                      Defaults to DINO. Case-sensitive, must match exactly.
        output_path: Where to save the PNG. If None, a temp path under
                     %TEMP%/DINOForge/wgc/ is created.
        timeout_seconds: Hard cap on the entire capture round-trip.

    Returns:
        WgcCaptureResult with success/path/dimensions/timing/error.
    """
    t0 = time.perf_counter()

    bin_path = _resolve_bare_cua_path()
    if bin_path is None:
        return WgcCaptureResult(
            success=False,
            error=(
                "wgc: bare-cua-native not found at "
                f"{_DEFAULT_BARE_CUA_PATH} (set DF_BARE_CUA_PATH to override)"
            ),
            elapsed_ms=(time.perf_counter() - t0) * 1000,
        )

    # Late import to avoid a circular dependency at module load time.
    # isolation_layer pulls in fastmcp/ctypes etc.; capture_wgc must be cheap
    # to import (server.py reads it at top-level).
    try:
        from .isolation_layer import PlayCUABackend
    except Exception as e:  # noqa: BLE001
        return WgcCaptureResult(
            success=False,
            error=f"wgc: isolation_layer import failed: {type(e).__name__}: {e}",
            elapsed_ms=(time.perf_counter() - t0) * 1000,
        )

    backend = PlayCUABackend(binary_path=str(bin_path))

    try:
        frame = await asyncio.wait_for(
            backend.capture_window(window_title),
            timeout=timeout_seconds,
        )
    except asyncio.TimeoutError:
        return WgcCaptureResult(
            success=False,
            error=f"wgc: frame timeout after {timeout_seconds}s",
            elapsed_ms=(time.perf_counter() - t0) * 1000,
        )
    except FileNotFoundError as e:
        return WgcCaptureResult(
            success=False,
            error=f"wgc: bare-cua-native missing: {e}",
            elapsed_ms=(time.perf_counter() - t0) * 1000,
        )
    except Exception as e:  # noqa: BLE001
        # bare-cua surfaces window-not-found / WGC-unavailable / D3D failures
        # as RuntimeError("...") from PlayCUABackend.capture_window. Normalize
        # the prefix so callers (and tests) can grep `wgc:`.
        msg = str(e) or type(e).__name__
        if not msg.lower().startswith("wgc"):
            msg = f"wgc: {msg}"
        return WgcCaptureResult(
            success=False,
            error=msg,
            elapsed_ms=(time.perf_counter() - t0) * 1000,
        )
    finally:
        # PlayCUAClient is held inside `backend.client`. We didn't go through
        # the IsolationContextManager singleton (intentionally — keep WGC
        # captures isolated from the main isolation backend), so we own
        # teardown. Best-effort stop; ignore failures.
        try:
            if backend.client is not None:
                await backend.client.stop()
        except Exception as e:  # noqa: BLE001
            logger.debug("wgc: backend.client.stop() suppressed: %s", e)

    # Persist PNG to disk.
    path = Path(output_path) if output_path else _default_temp_path()
    try:
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(frame.data)
    except Exception as e:  # noqa: BLE001
        return WgcCaptureResult(
            success=False,
            error=f"wgc: failed to write PNG to {path}: {e}",
            elapsed_ms=(time.perf_counter() - t0) * 1000,
        )

    return WgcCaptureResult(
        success=True,
        path=path,
        width=frame.width,
        height=frame.height,
        elapsed_ms=(time.perf_counter() - t0) * 1000,
    )


__all__ = [
    "DEFAULT_DINO_WINDOW_TITLE",
    "WGC_CAPTURE_TIMEOUT_SECONDS",
    "WgcCaptureResult",
    "capture_window_via_wgc",
    "check_wgc_available",
]
