r"""
Isolation Layer — Abstraction over playCUA and Win32 for game automation

Provides a unified interface to:
- Screenshot capture (GPU-accelerated, hidden desktop, fallback)
- Input injection (keyboard, mouse, scroll — focus-agnostic)
- Window enumeration and focus management
- Process launch/kill/status
- Image analysis (perceptual diff, hashing)

Architecture:
  3-tier fallback strategy:
    Tier 1: DINOForge Virtual Display Driver (WDDM/IDD) — future, best performance
    Tier 2: playCUA stdio JSON-RPC (bare-cua-native binary) — GPU WGC on Windows
    Tier 3: Win32 CreateDesktop + direct ctypes calls — compatibility fallback

  Each IsolationContext selects a backend; tools transparently use it.

Implementation notes:
  - playCUA binary: C:\Users\koosh\playcua_ci_test\target\release\bare-cua-native.exe
  - JSON-RPC protocol: stdin/stdout NDJSON
  - All methods are async-ready (return dicts compatible with FastMCP)
"""

import asyncio
import base64
import ctypes
import json
import logging
import os
import subprocess
import tempfile
import threading
from abc import ABC, abstractmethod
from dataclasses import dataclass
from enum import Enum
from pathlib import Path
from typing import Any, List, Optional

logger = logging.getLogger(__name__)


# ---------------------------------------------------------------------------
# Data models
# ---------------------------------------------------------------------------

@dataclass
class Frame:
    """Screenshot frame data."""
    data: bytes       # Raw PNG/JPEG bytes
    width: int        # Image width in pixels
    height: int       # Image height in pixels


@dataclass
class WindowInfo:
    """Window information."""
    hwnd: int         # Windows handle (or identifier on other platforms)
    title: str        # Window title
    process_id: int   # Process ID
    visible: bool     # Whether window is visible


# ---------------------------------------------------------------------------
# Backend abstraction
# ---------------------------------------------------------------------------

class IsolationBackend(ABC):
    """Abstract base class for isolation backends."""

    @abstractmethod
    async def capture_window(self, title: str) -> Frame:
        """Capture a screenshot of a window by title."""
        pass

    @abstractmethod
    async def capture_display(self, monitor: int = 0) -> Frame:
        """Capture a screenshot of a display/monitor."""
        pass

    @abstractmethod
    async def inject_key(self, key: str, duration: float = 0.05) -> bool:
        """Inject a keyboard key press."""
        pass

    @abstractmethod
    async def type_text(self, text: str) -> bool:
        """Type text character-by-character."""
        pass

    @abstractmethod
    async def mouse_click(self, x: int, y: int, button: str = "left") -> bool:
        """Click mouse at screen coordinates."""
        pass

    @abstractmethod
    async def mouse_scroll(self, x: int, y: int, delta: int) -> bool:
        """Scroll mouse wheel at screen coordinates."""
        pass

    @abstractmethod
    async def list_windows(self) -> List[WindowInfo]:
        """List all visible windows."""
        pass

    @abstractmethod
    async def focus_window(self, title: str) -> bool:
        """Focus a window by title."""
        pass

    @abstractmethod
    async def launch_process(self, exe: str, args: Optional[List[str]] = None, cwd: Optional[str] = None) -> int:
        """Launch a process and return PID."""
        pass


# ---------------------------------------------------------------------------
# HiddenDesktop backend (Windows only, Win32 CreateDesktop)
# ---------------------------------------------------------------------------

class HiddenDesktopBackend(IsolationBackend):
    """
    Win32 CreateDesktop backend for hidden game launches.
    Uses ctypes for direct Win32 API calls.
    """

    def __init__(self):
        self.game_cli_proj = None  # Will be set during initialization
        self.game_dir = None

    async def capture_window(self, title: str) -> Frame:
        """Capture window screenshot via GameControlCli (calls Win32 internally)."""
        try:
            # This delegates to the existing GameControlCli "screenshot" command
            # which handles WGC/BitBlt/fallback internally
            return Frame(data=b"", width=0, height=0)  # Placeholder — actual impl in server.py
        except Exception as e:
            logger.error(f"Capture window failed: {e}")
            raise

    async def capture_display(self, monitor: int = 0) -> Frame:
        """Capture display screenshot."""
        try:
            return Frame(data=b"", width=0, height=0)  # Placeholder
        except Exception as e:
            logger.error(f"Capture display failed: {e}")
            raise

    async def inject_key(self, key: str, duration: float = 0.05) -> bool:
        """Inject a keyboard key via Win32 SendInput."""
        try:
            return await self._send_key(key.lower(), duration)
        except Exception as e:
            logger.error(f"Key injection failed: {e}")
            return False

    async def type_text(self, text: str) -> bool:
        """Type text character by character via Win32 SendInput."""
        try:
            for char in text:
                await self._send_char(char)
                await asyncio.sleep(0.05)
            return True
        except Exception as e:
            logger.error(f"Text typing failed: {e}")
            return False

    async def mouse_click(self, x: int, y: int, button: str = "left") -> bool:
        """Click mouse at coordinates via Win32 SendInput."""
        try:
            return await self._send_click(x, y, button)
        except Exception as e:
            logger.error(f"Mouse click failed: {e}")
            return False

    async def mouse_scroll(self, x: int, y: int, delta: int) -> bool:
        """Scroll mouse wheel via Win32 SendInput."""
        try:
            return await self._send_scroll(x, y, delta)
        except Exception as e:
            logger.error(f"Mouse scroll failed: {e}")
            return False

    async def list_windows(self) -> List[WindowInfo]:
        """List windows (Win32 EnumWindowsEx via ctypes)."""
        try:
            # Stub: would require EnumWindowsEx callback
            return []
        except Exception as e:
            logger.error(f"List windows failed: {e}")
            return []

    async def focus_window(self, title: str) -> bool:
        """Focus window by title via Win32 SetForegroundWindow."""
        try:
            return await self._focus_window(title)
        except Exception as e:
            logger.error(f"Focus window failed: {e}")
            return False

    async def launch_process(self, exe: str, args: Optional[List[str]] = None, cwd: Optional[str] = None) -> int:
        """Launch process on hidden desktop via Win32 CreateDesktop."""
        try:
            return await self._launch_hidden(exe, args, cwd)
        except Exception as e:
            logger.error(f"Launch process failed: {e}")
            raise

    # -----------------------------------------------------------------------
    # Win32 helpers (extracted from server.py)
    # -----------------------------------------------------------------------

    async def _send_key(self, key: str, duration: float) -> bool:
        """Send key press via Win32 SendInput."""
        try:
            vk_codes = {
                "escape": 0x1B, "esc": 0x1B,
                "space": 0x20, "enter": 0x0D, "return": 0x0D,
                "tab": 0x09, "left": 0x25, "up": 0x26, "right": 0x27, "down": 0x28,
                "f1": 0x70, "f2": 0x71, "f3": 0x72, "f4": 0x73,
                "f5": 0x74, "f6": 0x75, "f7": 0x76, "f8": 0x77,
                "f9": 0x78, "f10": 0x79, "f11": 0x7A, "f12": 0x7B,
            }

            vk = vk_codes.get(key.lower())
            if vk is None:
                logger.warning(f"Unknown key: {key}")
                return False

            SendInput = ctypes.windll.user32.SendInput
            KEYEVENTF_KEYUP = 0x0002

            class KEYBDINPUT(ctypes.Structure):
                _fields_ = [("wVk", ctypes.c_ushort), ("wScan", ctypes.c_ushort),
                             ("dwFlags", ctypes.c_uint), ("time", ctypes.c_uint),
                             ("dwExtraInfo", ctypes.c_void_p)]

            class INPUT_UNION(ctypes.Union):
                _fields_ = [("ki", KEYBDINPUT)]

            class INPUT(ctypes.Structure):
                _fields_ = [("type", ctypes.c_uint), ("data", INPUT_UNION)]

            # Key down
            ki_down = KEYBDINPUT(vk, 0, 0, 0, None)
            inp_down = INPUT()
            inp_down.type = 1
            inp_down.data.ki = ki_down

            # Key up
            ki_up = KEYBDINPUT(vk, 0, KEYEVENTF_KEYUP, 0, None)
            inp_up = INPUT()
            inp_up.type = 1
            inp_up.data.ki = ki_up

            arr = (INPUT * 2)(inp_down, inp_up)
            SendInput(2, arr, ctypes.sizeof(INPUT))

            await asyncio.sleep(duration)
            return True
        except Exception as e:
            logger.error(f"_send_key failed: {e}")
            return False

    async def _send_char(self, char: str) -> bool:
        """Send a single character via Win32 (simplified)."""
        # For now, just try to send as key if it's alphanumeric
        if char.isalnum():
            return await self._send_key(char.lower(), 0.05)
        return True

    async def _send_click(self, x: int, y: int, button: str) -> bool:
        """Send mouse click via Win32 SendInput."""
        try:
            SendInput = ctypes.windll.user32.SendInput
            MOUSEEVENTF_LEFTDOWN = 0x0002
            MOUSEEVENTF_LEFTUP = 0x0004
            MOUSEEVENTF_MOVE = 0x0001

            class MOUSEINPUT(ctypes.Structure):
                _fields_ = [("dx", ctypes.c_long), ("dy", ctypes.c_long),
                             ("mouseData", ctypes.c_uint), ("dwFlags", ctypes.c_uint),
                             ("time", ctypes.c_uint), ("dwExtraInfo", ctypes.c_void_p)]

            class INPUT_UNION(ctypes.Union):
                _fields_ = [("mi", MOUSEINPUT)]

            class INPUT(ctypes.Structure):
                _fields_ = [("type", ctypes.c_uint), ("data", INPUT_UNION)]

            # Move to coordinates
            mi_move = MOUSEINPUT(x, y, 0, MOUSEEVENTF_MOVE, 0, None)
            inp_move = INPUT()
            inp_move.type = 0
            inp_move.data.mi = mi_move
            SendInput(1, ctypes.byref(inp_move), ctypes.sizeof(INPUT))

            # Click down
            mi_down = MOUSEINPUT(0, 0, 0, MOUSEEVENTF_LEFTDOWN, 0, None)
            inp_down = INPUT()
            inp_down.type = 0
            inp_down.data.mi = mi_down
            SendInput(1, ctypes.byref(inp_down), ctypes.sizeof(INPUT))

            # Click up
            mi_up = MOUSEINPUT(0, 0, 0, MOUSEEVENTF_LEFTUP, 0, None)
            inp_up = INPUT()
            inp_up.type = 0
            inp_up.data.mi = mi_up
            SendInput(1, ctypes.byref(inp_up), ctypes.sizeof(INPUT))

            return True
        except Exception as e:
            logger.error(f"_send_click failed: {e}")
            return False

    async def _send_scroll(self, x: int, y: int, delta: int) -> bool:
        """Send mouse scroll via Win32 SendInput."""
        try:
            SendInput = ctypes.windll.user32.SendInput
            MOUSEEVENTF_WHEEL = 0x0800

            class MOUSEINPUT(ctypes.Structure):
                _fields_ = [("dx", ctypes.c_long), ("dy", ctypes.c_long),
                             ("mouseData", ctypes.c_uint), ("dwFlags", ctypes.c_uint),
                             ("time", ctypes.c_uint), ("dwExtraInfo", ctypes.c_void_p)]

            class INPUT_UNION(ctypes.Union):
                _fields_ = [("mi", MOUSEINPUT)]

            class INPUT(ctypes.Structure):
                _fields_ = [("type", ctypes.c_uint), ("data", INPUT_UNION)]

            mi = MOUSEINPUT(x, y, delta, MOUSEEVENTF_WHEEL, 0, None)
            inp = INPUT()
            inp.type = 0
            inp.data.mi = mi
            SendInput(1, ctypes.byref(inp), ctypes.sizeof(INPUT))

            return True
        except Exception as e:
            logger.error(f"_send_scroll failed: {e}")
            return False

    async def _focus_window(self, title: str) -> bool:
        """Focus window by title."""
        try:
            FindWindowW = ctypes.windll.user32.FindWindowW
            SetForegroundWindow = ctypes.windll.user32.SetForegroundWindow

            hwnd = FindWindowW(None, title)
            if hwnd == 0:
                logger.warning(f"Window not found: {title}")
                return False

            SetForegroundWindow(hwnd)
            return True
        except Exception as e:
            logger.error(f"_focus_window failed: {e}")
            return False

    async def _launch_hidden(self, exe_path: str, args: Optional[List[str]] = None, cwd: Optional[str] = None) -> int:
        """Launch process on hidden Win32 desktop (via PowerShell script)."""
        try:
            desktop_name = "DINOForge_Agent"
            script_path = Path(tempfile.gettempdir()) / f"dinoforge_launch_{os.getpid()}.ps1"

            script_content = f'''\
param($ExePath, $DesktopName)
Add-Type @"
using System;
using System.Diagnostics;
using System.Runtime.InteropServices;
public class Win32Desktop {{
    [DllImport("user32.dll")] public static extern IntPtr CreateDesktop(string lpszDesktop, IntPtr lpszDevice, IntPtr pDevmode, int dwFlags, uint dwDesiredAccess, IntPtr lpsa);
    [DllImport("user32.dll")] public static extern bool CloseDesktop(IntPtr hDesktop);
    [DllImport("kernel32.dll")] public static extern bool CreateProcess(string lpAppName, string lpCmdLine, IntPtr lpPA, IntPtr lpTA, bool bInherit, uint dwCreationFlags, IntPtr lpEnv, string lpCurDir, ref STARTUPINFO lpSI, out PROCESS_INFORMATION lpPI);
    [StructLayout(LayoutKind.Sequential, CharSet=CharSet.Auto)] public struct STARTUPINFO {{ public int cb; public string lpReserved; public string lpDesktop; public string lpTitle; public int dwX, dwY, dwXSize, dwYSize, dwXCountChars, dwYCountChars, dwFillAttribute, dwFlags; public short wShowWindow, cbReserved2; public IntPtr lpReserved2, hStdInput, hStdOutput, hStdError; }}
    [StructLayout(LayoutKind.Sequential)] public struct PROCESS_INFORMATION {{ public IntPtr hProcess, hThread; public int dwProcessId, dwThreadId; }}
}}
"@
$DESKTOP_ALL = [uint32]0x000F01FF
$CREATE_NO_WINDOW = [uint32]0x08000000
$CREATE_DEFAULT_ERROR_MODE = [uint32]0x04000000
$desktop = [Win32Desktop]::CreateDesktop($DesktopName, [IntPtr]::Zero, [IntPtr]::Zero, 0, $DESKTOP_ALL, [IntPtr]::Zero)
if ($desktop -eq [IntPtr]::Zero) {{ Write-Output "ERROR:CreateDesktop"; exit 1 }}
$si = New-Object Win32Desktop+STARTUPINFO
$si.cb = [System.Runtime.InteropServices.Marshal]::SizeOf($si)
$si.lpDesktop = $DesktopName
$si.dwFlags = 0x00000001
$si.wShowWindow = 0
$pi = New-Object Win32Desktop+PROCESS_INFORMATION
$exeDir = Split-Path $ExePath -Parent
$cmdLine = $ExePath + " -popupwindow"
$creationFlags = $CREATE_NO_WINDOW -bor $CREATE_DEFAULT_ERROR_MODE -bor [uint32]0x00000010
$ok = [Win32Desktop]::CreateProcess($ExePath, $cmdLine, [IntPtr]::Zero, [IntPtr]::Zero, $false, $creationFlags, [IntPtr]::Zero, $exeDir, [ref]$si, [ref]$pi)
if (!$ok) {{ Write-Output "ERROR:CreateProcess"; exit 1 }}
$scriptBlock = {{
    param($desktopHandle, $processId)
    try {{
        $proc = [System.Diagnostics.Process]::GetProcessById($processId)
        $proc.WaitForExit()
    }} finally {{
        [Win32Desktop]::CloseDesktop($desktopHandle) | Out-Null
    }}
}}
Start-Job -ScriptBlock $scriptBlock -ArgumentList $desktop, $pi.dwProcessId | Out-Null
Write-Output "PID:$($pi.dwProcessId)"
'''

            script_path.write_text(script_content, encoding="utf-8-sig")
            result = await asyncio.to_thread(
                subprocess.run,
                ["powershell", "-ExecutionPolicy", "Bypass", "-File", str(script_path),
                 "-ExePath", exe_path, "-DesktopName", desktop_name],
                capture_output=True, text=True, timeout=30
            )

            stdout = result.stdout.strip()
            if stdout.startswith("PID:"):
                pid = int(stdout[4:])
                logger.info(f"Launched hidden process PID={pid}")
                return pid

            logger.error(f"Launch failed: {stdout or result.stderr}")
            raise RuntimeError(stdout or result.stderr)
        except Exception as e:
            logger.error(f"_launch_hidden failed: {e}")
            raise
        finally:
            try:
                script_path.unlink(missing_ok=True)
            except Exception:
                pass


# ---------------------------------------------------------------------------
# playCUA JSON-RPC 2.0 client
# ---------------------------------------------------------------------------

class PlayCUAClient:
    """
    JSON-RPC 2.0 client for bare-cua-native binary.
    Spawns the binary as a subprocess and communicates via stdin/stdout NDJSON.
    """

    def __init__(self, binary_path: str):
        self.binary_path = binary_path
        self.process: Optional[subprocess.Popen] = None
        self.pending_responses: dict[int, asyncio.Future] = {}
        self._reader_task: Optional[asyncio.Task] = None
        self._lock = asyncio.Lock()
        self._request_id_counter = 0

    async def start(self) -> None:
        """Start the bare-cua-native binary and reader loop."""
        if self.process is not None:
            return

        logger.info(f"Starting playCUA binary: {self.binary_path}")
        try:
            self.process = await asyncio.to_thread(
                subprocess.Popen,
                [self.binary_path],
                stdin=subprocess.PIPE,
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                text=True,
                bufsize=1,
            )
        except Exception as e:
            logger.error(f"Failed to start playCUA binary: {e}")
            raise

        self._reader_task = asyncio.create_task(self._read_responses())

    async def stop(self) -> None:
        """Stop the binary and clean up."""
        if self.process is None:
            return

        logger.info("Stopping playCUA binary")
        try:
            self.process.stdin.close()
            self.process.wait(timeout=5)
        except Exception as e:
            logger.warning(f"Error stopping playCUA: {e}")
            if self.process.poll() is None:
                self.process.terminate()
                self.process.wait(timeout=2)

        self.process = None
        if self._reader_task:
            self._reader_task.cancel()
            try:
                await self._reader_task
            except asyncio.CancelledError:
                pass

    async def _read_responses(self) -> None:
        """Background task: read NDJSON responses from stdout."""
        try:
            loop = asyncio.get_event_loop()
            while self.process and self.process.stdout and not self.process.stdout.closed:
                line = await asyncio.to_thread(self.process.stdout.readline)
                if not line:
                    break

                try:
                    response = json.loads(line)
                    request_id = response.get("id")
                    if request_id in self.pending_responses:
                        future = self.pending_responses.pop(request_id)
                        loop.call_soon_threadsafe(future.set_result, response)
                except json.JSONDecodeError as e:
                    logger.error(f"Invalid JSON from playCUA: {line} — {e}")
        except Exception as e:
            logger.error(f"Reader loop error: {e}")

    async def call(self, method: str, params: dict[str, Any]) -> dict[str, Any]:
        """Call a playCUA JSON-RPC method and wait for response."""
        if self.process is None:
            raise RuntimeError("playCUA not running — call await client.start() first")

        async with self._lock:
            self._request_id_counter += 1
            request_id = self._request_id_counter

            request = {
                "jsonrpc": "2.0",
                "id": request_id,
                "method": method,
                "params": params,
            }

            future: asyncio.Future = asyncio.Future()
            self.pending_responses[request_id] = future

            try:
                await asyncio.to_thread(
                    lambda: self.process.stdin.write(json.dumps(request) + "\n")
                )
                await asyncio.to_thread(lambda: self.process.stdin.flush())
            except Exception as e:
                self.pending_responses.pop(request_id, None)
                raise RuntimeError(f"Failed to send playCUA request: {e}")

        try:
            response = await asyncio.wait_for(future, timeout=30.0)
        except asyncio.TimeoutError:
            self.pending_responses.pop(request_id, None)
            raise asyncio.TimeoutError(f"playCUA did not respond to {method} within 30s")

        return response


# ---------------------------------------------------------------------------
# PlayCUA backend
# ---------------------------------------------------------------------------

class PlayCUABackend(IsolationBackend):
    """playCUA JSON-RPC backend via stdio NDJSON."""

    def __init__(self, binary_path: Optional[str] = None):
        if binary_path is None:
            binary_path = r"C:\Users\koosh\playcua_ci_test\target\release\bare-cua-native.exe"
        self.binary_path = binary_path
        self.client: Optional[PlayCUAClient] = None

    async def _ensure_client(self) -> PlayCUAClient:
        """Ensure client is started."""
        if self.client is None:
            self.client = PlayCUAClient(self.binary_path)
            await self.client.start()
        return self.client

    async def capture_window(self, title: str) -> Frame:
        """Capture window screenshot via playCUA."""
        try:
            client = await self._ensure_client()
            response = await client.call("screenshot", {"window_title": title})

            if "error" in response:
                raise RuntimeError(response["error"].get("message", "Unknown error"))

            result = response.get("result", {})
            data_b64 = result.get("data")
            width = result.get("width", 0)
            height = result.get("height", 0)

            if not data_b64:
                raise RuntimeError("No image data in response")

            data = base64.b64decode(data_b64)
            return Frame(data=data, width=width, height=height)
        except Exception as e:
            logger.error(f"PlayCUA capture_window failed: {e}")
            raise

    async def capture_display(self, monitor: int = 0) -> Frame:
        """Capture display screenshot via playCUA."""
        try:
            client = await self._ensure_client()
            response = await client.call("screenshot", {"monitor": monitor})

            if "error" in response:
                raise RuntimeError(response["error"].get("message", "Unknown error"))

            result = response.get("result", {})
            data_b64 = result.get("data")
            width = result.get("width", 0)
            height = result.get("height", 0)

            if not data_b64:
                raise RuntimeError("No image data in response")

            data = base64.b64decode(data_b64)
            return Frame(data=data, width=width, height=height)
        except Exception as e:
            logger.error(f"PlayCUA capture_display failed: {e}")
            raise

    async def inject_key(self, key: str, duration: float = 0.05) -> bool:
        """Inject key via playCUA."""
        try:
            client = await self._ensure_client()
            response = await client.call("input.key", {"key": key.lower(), "action": "press"})

            if "error" in response:
                logger.error(f"Key injection failed: {response['error']}")
                return False

            return True
        except Exception as e:
            logger.error(f"PlayCUA inject_key failed: {e}")
            return False

    async def type_text(self, text: str) -> bool:
        """Type text via playCUA."""
        try:
            client = await self._ensure_client()
            response = await client.call("input.type", {"text": text})

            if "error" in response:
                logger.error(f"Text typing failed: {response['error']}")
                return False

            return True
        except Exception as e:
            logger.error(f"PlayCUA type_text failed: {e}")
            return False

    async def mouse_click(self, x: int, y: int, button: str = "left") -> bool:
        """Click mouse via playCUA."""
        try:
            client = await self._ensure_client()
            response = await client.call("input.click", {
                "x": x, "y": y, "button": button, "action": "click"
            })

            if "error" in response:
                logger.error(f"Mouse click failed: {response['error']}")
                return False

            return True
        except Exception as e:
            logger.error(f"PlayCUA mouse_click failed: {e}")
            return False

    async def mouse_scroll(self, x: int, y: int, delta: int) -> bool:
        """Scroll mouse via playCUA."""
        try:
            client = await self._ensure_client()
            response = await client.call("input.scroll", {
                "x": x, "y": y, "delta": delta
            })

            if "error" in response:
                logger.error(f"Mouse scroll failed: {response['error']}")
                return False

            return True
        except Exception as e:
            logger.error(f"PlayCUA mouse_scroll failed: {e}")
            return False

    async def list_windows(self) -> List[WindowInfo]:
        """List windows via playCUA."""
        try:
            client = await self._ensure_client()
            response = await client.call("windows.list", {})

            if "error" in response:
                logger.error(f"List windows failed: {response['error']}")
                return []

            result = response.get("result", [])
            windows = []
            for w in result:
                windows.append(WindowInfo(
                    hwnd=w.get("hwnd", 0),
                    title=w.get("title", ""),
                    process_id=w.get("process_id", 0),
                    visible=w.get("visible", True)
                ))
            return windows
        except Exception as e:
            logger.error(f"PlayCUA list_windows failed: {e}")
            return []

    async def focus_window(self, title: str) -> bool:
        """Focus window via playCUA."""
        try:
            client = await self._ensure_client()
            response = await client.call("windows.focus", {"window_title": title})

            if "error" in response:
                logger.error(f"Focus window failed: {response['error']}")
                return False

            return True
        except Exception as e:
            logger.error(f"PlayCUA focus_window failed: {e}")
            return False

    async def launch_process(self, exe: str, args: Optional[List[str]] = None, cwd: Optional[str] = None) -> int:
        """Launch process via playCUA."""
        try:
            client = await self._ensure_client()
            params = {"exe": exe}
            if args:
                params["args"] = args
            if cwd:
                params["cwd"] = cwd

            response = await client.call("process.launch", params)

            if "error" in response:
                raise RuntimeError(response["error"].get("message", "Unknown error"))

            result = response.get("result", {})
            pid = result.get("pid")
            if pid is None:
                raise RuntimeError("No PID in response")

            return pid
        except Exception as e:
            logger.error(f"PlayCUA launch_process failed: {e}")
            raise


# ---------------------------------------------------------------------------
# Isolation context (singleton with auto-detection)
# ---------------------------------------------------------------------------

class IsolationContextManager:
    """Singleton context manager for isolation backends."""

    def __init__(self):
        self._backend: Optional[IsolationBackend] = None
        self._lock = threading.Lock()

    def get(self, backend_name: str = "auto") -> IsolationBackend:
        """
        Get or create isolation backend.

        Args:
            backend_name: "auto" (try playCUA, fallback to HiddenDesktop),
                         "playcua", or "hidden_desktop"

        Returns:
            IsolationBackend instance
        """
        if backend_name == "auto":
            return self._auto_select()
        elif backend_name == "playcua":
            return PlayCUABackend()
        elif backend_name == "hidden_desktop":
            return HiddenDesktopBackend()
        else:
            logger.warning(f"Unknown backend: {backend_name}, using auto-detection")
            return self._auto_select()

    def _auto_select(self) -> IsolationBackend:
        """Auto-detect: try playCUA, fallback to HiddenDesktop."""
        with self._lock:
            if self._backend is None:
                # Try playCUA first
                try:
                    backend = PlayCUABackend()
                    logger.info("Using PlayCUABackend (auto-detected)")
                    self._backend = backend
                except Exception as e:
                    logger.warning(f"playCUA not available: {e}, falling back to HiddenDesktop")
                    backend = HiddenDesktopBackend()
                    logger.info("Using HiddenDesktopBackend (fallback)")
                    self._backend = backend
            return self._backend


# Global singleton instance
_isolation_context_manager = IsolationContextManager()


def get_isolation_context(backend: str = "auto") -> IsolationBackend:
    """Get isolation backend (singleton with auto-detection)."""
    return _isolation_context_manager.get(backend)


def set_isolation_context(backend: IsolationBackend) -> None:
    """Manually set isolation backend."""
    with _isolation_context_manager._lock:
        _isolation_context_manager._backend = backend
