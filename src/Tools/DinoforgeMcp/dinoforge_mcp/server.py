"""
DINOForge MCP Server — FastMCP 3.x

Architecture:
  MCP Client (Claude) → FastMCP server
    ├─ game_* tools  → GameControlCli (C#) → named pipe → BepInEx GameBridgeServer
    ├─ asset_* tools → PackCompiler CLI (C#) → asset pipeline
    ├─ catalog_*     → direct JSON parse of Addressables catalog
    └─ log_*         → direct read of BepInEx/dinoforge_debug.log

The C# McpServer (src/Tools/McpServer) handles the same game bridge tools via
the ModelContextProtocol NuGet. This Python server is the preferred one for
non-game-bridge tasks (asset pipeline, catalog inspection, log analysis) and
wraps game bridge commands via the lightweight GameControlCli binary.
"""
from __future__ import annotations

import argparse
import asyncio
import base64
import json
import logging
import os
import subprocess
import tempfile
import threading
from pathlib import Path
from typing import Any

from dotenv import load_dotenv
from fastmcp import FastMCP, Context
from pydantic import BaseModel, Field
from starlette.responses import JSONResponse
from starlette.requests import Request

load_dotenv()
logging.basicConfig(level=logging.DEBUG if os.getenv("DINOFORGE_MCP_DEBUG") else logging.WARNING)
logger = logging.getLogger("dinoforge_mcp")

# HMR event — set when game reloads to clear cached pack state
_reload_event = threading.Event()

# ---------------------------------------------------------------------------
# Paths
# ---------------------------------------------------------------------------

_HERE = Path(__file__).resolve().parent
REPO_ROOT = (_HERE / "../../../../").resolve()

GAME_DIR = Path(os.getenv(
    "DINO_GAME_DIR",
    r"G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option"
))
GAME_EXE = GAME_DIR / "Diplomacy is Not an Option.exe"
BEPINEX_DIR = GAME_DIR / "BepInEx"
DEBUG_LOG = BEPINEX_DIR / "dinoforge_debug.log"
CATALOG_JSON = GAME_DIR / r"Diplomacy is Not an Option_Data\StreamingAssets\aa\catalog.json"

GAME_CONTROL_PROJ = REPO_ROOT / "src/Tools/GameControlCli/GameControlCli.csproj"
PACK_COMPILER_PROJ = REPO_ROOT / "src/Tools/PackCompiler/DINOForge.Tools.PackCompiler.csproj"
ASSET_CLI_PROJ = REPO_ROOT / "src/Tools/Cli/DINOForge.Tools.Cli.csproj"
PACKS_DIR = REPO_ROOT / "packs"

# Dedicated DINOForge Virtual Display Driver (Nefarius/MTT VDD)
_VDD_INDEX_FILE = REPO_ROOT / ".dinoforge_vdd_index"

# Test instance path config file
_TEST_INSTANCE_PATH_FILE = REPO_ROOT / ".dino_test_instance_path"

# ---------------------------------------------------------------------------
# GameControlCli client (thin wrapper — avoids dotnet run cold-start overhead
# by using --no-build; caller should run `dotnet build` once before first use)
# ---------------------------------------------------------------------------

def _run_game_cli(*args: str, timeout: int = 20, json_output: bool = True) -> dict[str, Any]:
    """Invoke GameControlCli synchronously and return parsed JSON."""
    cmd = [
        "dotnet", "run",
        "--project", str(GAME_CONTROL_PROJ),
        "--no-build",
        "-c", "Release",
        "--",
        *args,
        "--format=json",
    ]
    try:
        r = subprocess.run(cmd, capture_output=True, text=True, timeout=timeout, cwd=REPO_ROOT)
        if r.returncode != 0:
            return {"success": False, "error": r.stderr.strip() or r.stdout.strip()}
        if not json_output:
            return {"success": True, "raw": r.stdout.strip()}
        try:
            return json.loads(r.stdout) if r.stdout.strip() else {"success": True}
        except json.JSONDecodeError:
            return {"success": True, "raw": r.stdout.strip()}
    except subprocess.TimeoutExpired:
        return {"success": False, "error": f"GameControlCli timed out after {timeout}s"}
    except Exception as e:
        return {"success": False, "error": str(e)}


def _run_pack_compiler(*args: str, timeout: int = 60) -> dict[str, Any]:
    """Invoke PackCompiler CLI."""
    cmd = ["dotnet", "run", "--project", str(PACK_COMPILER_PROJ), "--no-build", "--", *args]
    try:
        r = subprocess.run(cmd, capture_output=True, text=True, timeout=timeout, cwd=REPO_ROOT)
        return {"success": r.returncode == 0, "output": r.stdout.strip(), "error": r.stderr.strip()}
    except subprocess.TimeoutExpired:
        return {"success": False, "error": f"PackCompiler timed out after {timeout}s"}
    except Exception as e:
        return {"success": False, "error": str(e)}


# ---------------------------------------------------------------------------
# VDD (Virtual Display Driver) Support
# ---------------------------------------------------------------------------

def _get_vdd_index() -> int | None:
    """Read the dedicated DINOForge VDD monitor index from config file."""
    try:
        return int(_VDD_INDEX_FILE.read_text().strip())
    except Exception:
        return None


def _get_test_instance_path() -> str:
    """
    Read the test instance path from .dino_test_instance_path config file.
    Falls back to default path if file doesn't exist or is invalid.
    Validates that the directory exists.
    """
    default_path = r"G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option_TEST"

    # Try to read from config file
    if _TEST_INSTANCE_PATH_FILE.exists():
        try:
            config_path = _TEST_INSTANCE_PATH_FILE.read_text().strip()
            if config_path:
                path_obj = Path(config_path)
                if path_obj.exists() and path_obj.is_dir():
                    logger.info(f"Using test instance path from {_TEST_INSTANCE_PATH_FILE}: {config_path}")
                    return config_path
                else:
                    logger.warning(f"Config path doesn't exist or is not a directory: {config_path}. Using default.")
        except Exception as e:
            logger.warning(f"Error reading {_TEST_INSTANCE_PATH_FILE}: {e}. Using default.")
    else:
        logger.info(f"Config file not found at {_TEST_INSTANCE_PATH_FILE}. Using default path.")

    return default_path


async def _launch_on_vdd(exe_path: str, width: int = 1920, height: int = 1080) -> dict:
    """Launch game on dedicated DINOForge Virtual Display Driver (not user's Parsec VDD)."""
    idx = _get_vdd_index()
    if idx is None:
        return {"success": False, "error": "DINOForge VDD not configured — run scripts/setup-vdd.ps1"}
    try:
        await asyncio.to_thread(
            subprocess.Popen,
            [exe_path, "-monitor", str(idx), "-screen-width", str(width),
             "-screen-height", str(height), "-popupwindow"],
            cwd=str(Path(exe_path).parent)
        )
        return {"success": True, "message": f"Launched on VDD monitor {idx} ({width}x{height})"}
    except Exception as e:
        return {"success": False, "error": str(e)}


async def _launch_hidden(exe_path: str, desktop_name: str = "DINOForge_Agent") -> dict:
    """Launch game on a hidden Win32 desktop using CreateDesktop with -popupwindow flag."""
    # Write script to a temp file to avoid -Command here-string parsing issues
    # (here-strings via -Command can fail with "Invalid argument" on some PowerShell versions)
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
    try:
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
            return {"success": True, "pid": pid, "desktop": desktop_name, "hidden": True}
        return {"success": False, "error": stdout or result.stderr}
    finally:
        try:
            script_path.unlink(missing_ok=True)
        except Exception:
            pass


# ---------------------------------------------------------------------------
# FastMCP server
# ---------------------------------------------------------------------------

mcp = FastMCP(
    "dinoforge",
    instructions=(
        "DINOForge unified MCP server. "
        "game_* tools: live game state via named pipe bridge (GameControlCli). "
        "asset_* / pack_*: asset pipeline and pack management (PackCompiler). "
        "catalog_*: direct Addressables catalog inspection. "
        "log_*: BepInEx debug log analysis."
    ),
)

# ===========================================================================
# GAME BRIDGE TOOLS  (via GameControlCli → named pipe → BepInEx plugin)
# ===========================================================================

@mcp.tool()
async def game_status(ctx: Context) -> dict:
    """Get game connection status, world readiness, entity count, and loaded packs."""
    return _run_game_cli("status")


@mcp.tool()
async def game_wait_world(ctx: Context, timeout_seconds: int = 60) -> dict:
    """Wait until the ECS game world is ready (up to timeout_seconds)."""
    return _run_game_cli("wait-world", timeout=timeout_seconds + 5)


@mcp.tool()
async def game_wait_for_world(ctx: Context, timeout_seconds: int = 60) -> dict:
    """Wait until the ECS game world is ready (up to timeout_seconds). Alias for game_wait_world."""
    return _run_game_cli("wait-world", timeout=timeout_seconds + 5)


@mcp.tool()
async def game_resources(ctx: Context) -> dict:
    """Get current in-game resources (gold, wood, food, etc.)."""
    return _run_game_cli("resources")


@mcp.tool()
async def game_get_resources(ctx: Context) -> dict:
    """Get current in-game resources (gold, wood, food, etc.). Alias for game_resources."""
    return _run_game_cli("resources")


@mcp.tool()
async def game_screenshot(ctx: Context, output_path: str | None = None) -> dict:
    """
    Capture a screenshot of the game window.

    Args:
        output_path: Optional file path to save the PNG. Defaults to a temp path.
    """
    args = ["screenshot"]
    if output_path:
        args += ["--output", output_path]
    return _run_game_cli(*args)


@mcp.tool()
async def game_query_entities(ctx: Context, component_type: str = "") -> dict:
    """
    Query ECS entities by component type.

    Args:
        component_type: Full ECS component type name, e.g. 'Components.Unit',
                        'Components.BuildingBase', 'Unity.Rendering.RenderMesh'.
                        Empty string returns all entities.
    """
    return _run_game_cli("entities", component_type)


@mcp.tool()
async def game_ui_tree(ctx: Context, selector: str | None = None) -> dict:
    """
    Snapshot the live Unity UI hierarchy (Playwright-style DOM).

    Args:
        selector: Optional CSS-like selector to filter results.
    """
    args = ["ui-tree"]
    if selector:
        args.append(selector)
    return _run_game_cli(*args)


@mcp.tool()
async def game_click_button(ctx: Context, button_name: str) -> dict:
    """
    Click a named Unity UI button.

    Args:
        button_name: Unity UI button name (e.g. 'DINOForge_ModsButton', 'PlayButton').
    """
    return _run_game_cli("click-button", button_name)


@mcp.tool()
async def game_load_scene(ctx: Context, scene_name: str) -> dict:
    """
    Load a game scene by name. Available: level0–level9 and others.

    Args:
        scene_name: Scene name or index.
    """
    return _run_game_cli("load-scene", scene_name)


@mcp.tool()
async def game_start(ctx: Context) -> dict:
    """Trigger game world load via ECS singleton (bypasses the main menu)."""
    return _run_game_cli("start-game")


@mcp.tool()
async def game_dismiss(ctx: Context) -> dict:
    """Dismiss a 'Press Any Key to Continue' loading screen."""
    return _run_game_cli("dismiss")


@mcp.tool()
async def game_catalog(ctx: Context, category: str | None = None) -> dict:
    """
    Dump the game's content catalog (units, buildings, projectiles).

    Args:
        category: Optional filter: 'units', 'buildings', 'projectiles'.
    """
    args = ["catalog"]
    if category:
        args.append(category)
    return _run_game_cli(*args)


@mcp.tool()
async def game_launch(ctx: Context, hidden: bool = False) -> dict:
    """
    Launch Diplomacy is Not an Option directly (bypasses Steam — safe to run
    alongside an existing session for testing).

    Args:
        hidden: If True, launch on the dedicated DINOForge VDD first, fallback to CreateDesktop.
    """
    if not GAME_EXE.exists():
        return {"success": False, "error": f"Game exe not found: {GAME_EXE}"}
    try:
        if hidden:
            # Try VDD first (dedicated DINOForge virtual display, not user's Parsec VDD)
            vdd_result = await _launch_on_vdd(str(GAME_EXE))
            if vdd_result["success"]:
                return vdd_result
            # Fall back to CreateDesktop if VDD not configured
            return await _launch_hidden(str(GAME_EXE), "DINOForge_Agent")
        subprocess.Popen([str(GAME_EXE)], cwd=str(GAME_DIR))
        return {"success": True, "message": f"Launched: {GAME_EXE}. Use game_wait_world to wait for ECS world."}
    except Exception as e:
        return {"success": False, "error": str(e)}


@mcp.tool()
async def game_launch_test(ctx: Context, hidden: bool = True) -> dict:
    """
    Launch the TEST instance of DINO (second concurrent instance for testing).
    Reads path from .dino_test_instance_path config file (if present) or uses default.
    Kill existing test instances first if needed.

    Args:
        hidden: If True (default), launch on an invisible Win32 desktop (CreateDesktop). Set to False for visible window.
    """
    test_dir = _get_test_instance_path()
    test_exe = Path(test_dir) / "Diplomacy is Not an Option.exe"
    if not test_exe.exists():
        error_msg = (
            f"Test game exe not found: {test_exe}. "
            f"Check that the test instance is installed at the path specified in {_TEST_INSTANCE_PATH_FILE} "
            f"or at the default location."
        )
        return {"success": False, "error": error_msg}
    try:
        if hidden:
            return await _launch_hidden(str(test_exe), "DINOForge_Agent_Test")
        subprocess.Popen([str(test_exe)], cwd=test_dir)
        return {"success": True, "message": f"Launched TEST instance: {test_exe}. Use game_wait_world to wait for ECS world."}
    except Exception as e:
        error_msg = (
            f"Failed to launch test instance: {str(e)}. "
            f"If the path is wrong, update {_TEST_INSTANCE_PATH_FILE} or reinstall the test instance."
        )
        return {"success": False, "error": error_msg}


@mcp.tool()
async def game_launch_vdd(ctx: Context, width: int = 1920, height: int = 1080) -> dict:
    """
    Launch game on dedicated DINOForge virtual display (not user's personal VDD).
    Requires .dinoforge_vdd_index to be configured in repo root.

    Args:
        width: Virtual display width (default 1920).
        height: Virtual display height (default 1080).
    """
    if not GAME_EXE.exists():
        return {"success": False, "error": f"Game exe not found: {GAME_EXE}"}
    return await _launch_on_vdd(str(GAME_EXE), width, height)


# ===========================================================================
# ASSET PIPELINE TOOLS  (via PackCompiler CLI)
# ===========================================================================

@mcp.tool()
async def asset_validate(ctx: Context, pack: str) -> dict:
    """
    Validate assets in a pack against the asset_pipeline.yaml schema.

    Args:
        pack: Pack name (e.g. 'warfare-starwars').
    """
    return _run_pack_compiler("assets", "validate", f"packs/{pack}")


@mcp.tool()
async def asset_import(ctx: Context, pack: str) -> dict:
    """
    Import (download + convert) source assets for a pack.

    Args:
        pack: Pack name.
    """
    return _run_pack_compiler("assets", "import", f"packs/{pack}")


@mcp.tool()
async def asset_optimize(ctx: Context, pack: str) -> dict:
    """
    Generate LOD variants for all assets in a pack.

    Args:
        pack: Pack name.
    """
    return _run_pack_compiler("assets", "optimize", f"packs/{pack}")


@mcp.tool()
async def asset_build(ctx: Context, pack: str) -> dict:
    """
    Run the full asset pipeline (validate → import → optimize → generate → build).

    Args:
        pack: Pack name.
    """
    return _run_pack_compiler("assets", "build", f"packs/{pack}")


@mcp.tool()
async def pack_validate(ctx: Context, pack: str) -> dict:
    """
    Validate a mod pack (YAML schemas, references, completeness).

    Args:
        pack: Pack name or path.
    """
    return _run_pack_compiler("validate", f"packs/{pack}")


@mcp.tool()
async def pack_build(ctx: Context, pack: str) -> dict:
    """
    Compile and package a mod pack.

    Args:
        pack: Pack name.
    """
    return _run_pack_compiler("build", f"packs/{pack}")


@mcp.tool()
async def pack_list(ctx: Context) -> dict:
    """List all available packs in the repository."""
    try:
        packs = [
            {"id": p.name, "path": str(p)}
            for p in PACKS_DIR.iterdir()
            if p.is_dir() and (p / "pack.yaml").exists()
        ]
        return {"success": True, "packs": packs, "count": len(packs)}
    except Exception as e:
        return {"success": False, "error": str(e)}


# ===========================================================================
# BRIDGE-ONLY TOOLS  (JSON-output GameControlCli commands)
# ===========================================================================

@mcp.tool()
async def game_get_stat(ctx: Context, sdk_path: str, entity_index: int | None = None) -> dict:
    """
    Read a stat value from ECS entities by SDK model path.

    Args:
        sdk_path: Dot-separated SDK path (e.g. 'unit.stats.hp').
        entity_index: Optional specific entity index.
    """
    args = ["get-stat", sdk_path]
    if entity_index is not None:
        args.append(str(entity_index))
    return _run_game_cli(*args)


@mcp.tool()
async def game_apply_override(
    ctx: Context,
    sdk_path: str,
    value: float,
    mode: str | None = None,
    filter_component: str | None = None,
) -> dict:
    """
    Apply a stat override to matching ECS entities.

    Args:
        sdk_path: SDK model path (e.g. 'unit.stats.hp').
        value: The numeric value to apply.
        mode: 'override' (default), 'add', or 'multiply'.
        filter_component: Optional ECS component type to narrow affected entities.
    """
    args = ["apply-override", sdk_path, str(value)]
    if mode:
        args.append(mode)
    if filter_component:
        args.append(filter_component)
    return _run_game_cli(*args)


@mcp.tool()
async def game_get_component_map(ctx: Context, sdk_path: str | None = None) -> dict:
    """
    Return SDK-to-ECS component type mappings.

    Args:
        sdk_path: Optional filter — omit to return all 30+ mappings.
    """
    args = ["get-component-map"]
    if sdk_path:
        args.append(sdk_path)
    return _run_game_cli(*args)


@mcp.tool()
async def game_reload_packs(ctx: Context, path: str | None = None) -> dict:
    """
    Hot-reload content packs from disk without restarting the game.

    Args:
        path: Optional packs directory path override.
    """
    args = ["reload-packs"]
    if path:
        args.append(path)
    return _run_game_cli(*args)


@mcp.tool()
async def game_verify_mod(ctx: Context, pack_path: str) -> dict:
    """
    End-to-end mod verification: load a pack into the running game, verify entity changes.

    Args:
        pack_path: Path to the pack directory or manifest file.
    """
    return _run_game_cli("verify-mod", pack_path)


@mcp.tool()
async def game_dump_state(ctx: Context, category: str | None = None) -> dict:
    """
    Dump ECS game state as structured JSON.

    Args:
        category: 'unit', 'building', 'projectile', or omit for all.
    """
    args = ["dump-state"]
    if category:
        args.append(category)
    return _run_game_cli(*args)


@mcp.tool()
async def game_input(
    ctx: Context,
    key: str | None = None,
    mouse_x: int | None = None,
    mouse_y: int | None = None,
    click: bool = False,
) -> dict:
    """
    Inject keyboard or mouse input to the game without requiring window focus (Win32 SendInput).

    Args:
        key: Virtual key name (e.g. 'F1', 'Space', 'Escape').
        mouse_x: Mouse X coordinate (screen absolute).
        mouse_y: Mouse Y coordinate (screen absolute).
        click: If True, send a left mouse click at (mouse_x, mouse_y).
    """
    try:
        import ctypes
    except ImportError:
        return {"success": False, "error": "ctypes not available."}

    SendInput = ctypes.windll.user32.SendInput
    KEYEVENTF_KEYUP = 0x0002
    MOUSEEVENTF_LEFTDOWN = 0x0002
    MOUSEEVENTF_LEFTUP = 0x0004
    MOUSEEVENTF_MOVE = 0x0001

    VK_CODES = {
        "escape": 0x1B, "esc": 0x1B,
        "space": 0x20, "enter": 0x0D, "return": 0x0D,
        "tab": 0x09, "left": 0x25, "up": 0x26, "right": 0x27, "down": 0x28,
        "f1": 0x70, "f2": 0x71, "f3": 0x72, "f4": 0x73,
        "f5": 0x74, "f6": 0x75, "f7": 0x76, "f8": 0x77,
        "f9": 0x78, "f10": 0x79, "f11": 0x7A, "f12": 0x7B,
        "a": 0x41, "b": 0x42, "c": 0x43, "d": 0x44, "e": 0x45,
        "f": 0x46, "g": 0x47, "h": 0x48, "i": 0x49, "j": 0x4A,
        "k": 0x4B, "l": 0x4C, "m": 0x4D, "n": 0x4E, "o": 0x4F,
        "p": 0x50, "q": 0x51, "r": 0x52, "s": 0x53, "t": 0x54,
        "u": 0x55, "v": 0x56, "w": 0x57, "x": 0x58, "y": 0x59, "z": 0x5A,
        "0": 0x30, "1": 0x31, "2": 0x32, "3": 0x33, "4": 0x34,
        "5": 0x35, "6": 0x36, "7": 0x37, "8": 0x38, "9": 0x39,
    }

    class _KEYBDINPUT(ctypes.Structure):
        _fields_ = [("wVk", ctypes.c_ushort), ("wScan", ctypes.c_ushort),
                     ("dwFlags", ctypes.c_uint), ("time", ctypes.c_uint),
                     ("dwExtraInfo", ctypes.c_void_p)]

    class _MOUSEINPUT(ctypes.Structure):
        _fields_ = [("dx", ctypes.c_long), ("dy", ctypes.c_long),
                     ("mouseData", ctypes.c_uint), ("dwFlags", ctypes.c_uint),
                     ("time", ctypes.c_uint), ("dwExtraInfo", ctypes.c_void_p)]

    class _INPUT_UNION(ctypes.Union):
        _fields_ = [("ki", _KEYBDINPUT), ("mi", _MOUSEINPUT)]

    class _INPUT(ctypes.Structure):
        _fields_ = [("type", ctypes.c_uint), ("data", _INPUT_UNION)]

    def _make_input(type_val: int, union_val) -> _INPUT:
        inp = _INPUT()
        inp.type = type_val
        ctypes.memmove(ctypes.addressof(inp.data), ctypes.addressof(union_val), ctypes.sizeof(union_val))
        return inp

    def _send_key(vk: int) -> None:
        ki_down = _KEYBDINPUT(vk, 0, 0, 0, None)
        ki_up = _KEYBDINPUT(vk, 0, KEYEVENTF_KEYUP, 0, None)
        arr = (_INPUT * 2)(*[_make_input(1, ki_down), _make_input(1, ki_up)])
        SendInput(2, arr, ctypes.sizeof(_INPUT))

    def _send_mouse_move(x: int, y: int) -> None:
        mi = _MOUSEINPUT(x, y, 0, MOUSEEVENTF_MOVE, 0, None)
        arr = (_INPUT * 1)(*[_make_input(0, mi)])
        SendInput(1, arr, ctypes.sizeof(_INPUT))

    def _send_click(x: int, y: int) -> None:
        _send_mouse_move(x, y)
        for flags in (MOUSEEVENTF_LEFTDOWN, MOUSEEVENTF_LEFTUP):
            mi = _MOUSEINPUT(0, 0, 0, flags, 0, None)
            arr = (_INPUT * 1)(*[_make_input(0, mi)])
            SendInput(1, arr, ctypes.sizeof(_INPUT))

    try:
        if key:
            vk = VK_CODES.get(key.lower())
            if vk is None:
                return {"success": False, "error": f"Unknown key: {key}"}
            _send_key(vk)
        if mouse_x is not None and mouse_y is not None and click:
            _send_click(mouse_x, mouse_y)
        elif mouse_x is not None and mouse_y is not None:
            _send_mouse_move(mouse_x, mouse_y)
        return {"success": True, "message": "Input injected."}
    except Exception as e:
        return {"success": False, "error": str(e)}


@mcp.tool()
async def game_ui_automation(ctx: Context, action: str, target: str | None = None) -> dict:
    """
    Automate game UI interactions (click, hover, type, screenshot).

    Args:
        action: Action to perform: 'click', 'hover', 'type', 'snapshot'.
        target: Target button/element name or selector.
    """
    args = ["ui-automation", action]
    if target:
        args.append(target)
    return _run_game_cli(*args)


@mcp.tool()
async def game_analyze_screen(ctx: Context, screenshot_path: str | None = None) -> dict:
    """
    Capture a screenshot and detect UI elements via OmniParser (health bars, unit portraits,
    buttons, faction indicators).

    Args:
        screenshot_path: Optional path to existing screenshot (takes new one if omitted).
    """
    args = ["analyze-screen"]
    if screenshot_path:
        args += ["--input", screenshot_path]
    return _run_game_cli(*args)


def _analyze_screenshot_cv(screenshot_path: str | None) -> dict:
    """
    Analyze a screenshot using OpenCV to detect UI regions.
    Returns health bars, buttons, portraits, and faction color patches.
    """
    try:
        import cv2
        import numpy as np
    except ImportError:
        return {"success": False, "error": "opencv-python-headless not installed"}

    temp_dir = Path(os.getenv("TEMP", "/tmp")) / "DINOForge"
    if screenshot_path is None:
        screenshot_path = str(temp_dir / "screenshot.png")
        cap_result = _run_game_cli("screenshot", screenshot_path)
        if not cap_result.get("success"):
            return {"success": False, "error": f"Failed to capture screenshot: {cap_result}"}

    if not Path(screenshot_path).exists():
        return {"success": False, "error": f"Screenshot not found: {screenshot_path}"}

    img = cv2.imread(screenshot_path)
    if img is None:
        return {"success": False, "error": f"Failed to read image: {screenshot_path}"}

    h, w = img.shape[:2]
    elements: list[dict] = []

    # Convert to different color spaces for analysis
    gray = cv2.cvtColor(img, cv2.COLOR_BGR2GRAY)
    hsv = cv2.cvtColor(img, cv2.COLOR_BGR2HSV)

    # Detect health bars: narrow horizontal rectangles, typically green/red/yellow
    # Green health bar range in HSV
    green_lower = np.array([35, 50, 50])
    green_upper = np.array([85, 255, 255])
    green_mask = cv2.inRange(hsv, green_lower, green_upper)
    # Find contours (horizontal bar-shaped)
    contours, _ = cv2.findContours(green_mask, cv2.RETR_EXTERNAL, cv2.CHAIN_APPROX_SIMPLE)
    for cnt in contours:
        x, y, bw, bh = cv2.boundingRect(cnt)
        aspect = bw / max(bh, 1)
        if aspect > 3 and 5 < bw < w * 0.4 and 2 < bh < 20:  # Long thin bar = health bar
            elements.append({
                "type": "health_bar",
                "x": int(x), "y": int(y),
                "width": int(bw), "height": int(bh),
                "color": "green",
                "confidence": min(1.0, bw / 200),
            })

    # Red/amber bar for enemy health
    red_lower1 = np.array([0, 70, 50]); red_upper1 = np.array([10, 255, 255])
    red_lower2 = np.array([170, 70, 50]); red_upper2 = np.array([180, 255, 255])
    red_mask = cv2.inRange(hsv, red_lower1, red_upper1) | cv2.inRange(hsv, red_lower2, red_upper2)
    contours2, _ = cv2.findContours(red_mask, cv2.RETR_EXTERNAL, cv2.CHAIN_APPROX_SIMPLE)
    for cnt in contours2:
        x, y, bw, bh = cv2.boundingRect(cnt)
        aspect = bw / max(bh, 1)
        if aspect > 3 and 5 < bw < w * 0.4 and 2 < bh < 20:
            elements.append({
                "type": "health_bar",
                "x": int(x), "y": int(y),
                "width": int(bw), "height": int(bh),
                "color": "red",
                "confidence": min(1.0, bw / 200),
            })

    # Detect faction color patches: large uniform color regions (top corners = faction banners)
    faction_colors = [
        ("republic_blue",   ( 70,  80, 180), (110, 160, 255), 0.8),
        ("cis_gold",        ( 15, 100, 150), ( 35, 200, 255), 0.8),
        ("enemy_red",       (  0, 100, 100), ( 10, 255, 255), 0.8),
        ("neutral_gray",    ( 80,   0,  80), (120,  50, 180), 0.5),
    ]
    for name, bgr_low, bgr_high, conf in faction_colors:
        low = np.array(bgr_low); upper = np.array(bgr_high)
        mask = cv2.inRange(img, low, upper)
        cnts, _ = cv2.findContours(mask, cv2.RETR_EXTERNAL, cv2.CHAIN_APPROX_SIMPLE)
        for cnt in cnts:
            x, y, bw, bh = cv2.boundingRect(cnt)
            if bw > 30 and bh > 30:  # Faction banner minimum size
                elements.append({
                    "type": "faction_patch",
                    "faction": name,
                    "x": int(x), "y": int(y),
                    "width": int(bw), "height": int(bh),
                    "confidence": conf,
                })

    # Detect portrait-like regions: square-ish shapes in the bottom-left area (unit portrait zone)
    portrait_zone = img[int(h * 0.7):, :int(w * 0.3)]
    gray_portrait = cv2.cvtColor(portrait_zone, cv2.COLOR_BGR2GRAY)
    _, thresh = cv2.threshold(gray_portrait, 0, 255, cv2.THRESH_BINARY + cv2.THRESH_OTSU)
    cnts3, _ = cv2.findContours(thresh, cv2.RETR_EXTERNAL, cv2.CHAIN_APPROX_SIMPLE)
    offset_y = int(h * 0.7)
    for cnt in cnts3:
        x, y, bw, bh = cv2.boundingRect(cnt)
        aspect = min(bw, bh) / max(bw, bh, 1)
        area = bw * bh
        if aspect > 0.7 and 400 < area < 50000:  # Square-ish, portrait-sized
            elements.append({
                "type": "unit_portrait",
                "x": int(x), "y": int(y + offset_y),
                "width": int(bw), "height": int(bh),
                "confidence": 0.6,
            })

    # Button-like regions: light/white rectangles near bottom of screen
    button_zone = img[int(h * 0.85):, :]
    button_gray = cv2.cvtColor(button_zone, cv2.COLOR_BGR2GRAY)
    _, button_thresh = cv2.threshold(button_gray, 200, 255, cv2.THRESH_BINARY)
    cnts4, _ = cv2.findContours(button_thresh, cv2.RETR_EXTERNAL, cv2.CHAIN_APPROX_SIMPLE)
    button_offset_y = int(h * 0.85)
    for cnt in cnts4:
        x, y, bw, bh = cv2.boundingRect(cnt)
        if bw > 60 and 10 < bh < 60 and bw / max(bh, 1) > 2:
            elements.append({
                "type": "button",
                "x": int(x), "y": int(y + button_offset_y),
                "width": int(bw), "height": int(bh),
                "confidence": 0.7,
            })

    return {
        "success": True,
        "screenshot": screenshot_path,
        "resolution": {"width": w, "height": h},
        "elements_detected": len(elements),
        "elements": elements,
        "method": "opencv_color_contour",
    }


@mcp.tool()
async def game_analyze_screen(ctx: Context, screenshot_path: str | None = None) -> dict:
    """
    Capture a screenshot and detect UI elements via OpenCV color/contour analysis
    (health bars, unit portraits, buttons, faction indicators).
    Falls back to VLM (transformers + vit-base) if available.

    Args:
        screenshot_path: Optional path to existing screenshot (takes new one if omitted).
    """
    return _analyze_screenshot_cv(screenshot_path)


@mcp.tool()
async def game_wait_and_screenshot(
    ctx: Context,
    timeout_seconds: int = 30,
    interval_seconds: float = 1.0,
    change_threshold: float = 0.05,
) -> dict:
    """
    Poll for a visual change in the game window, then capture a screenshot.

    Args:
        timeout_seconds: Max time to wait for visual change.
        interval_seconds: How often to check for change (seconds).
        change_threshold: Minimum pixel-change fraction to count as a change (0.0–1.0).
    """
    args = [
        "wait-and-screenshot",
        "--timeout", str(timeout_seconds),
        "--interval", str(interval_seconds),
        "--threshold", str(change_threshold),
    ]
    return _run_game_cli(*args)


@mcp.tool()
async def game_navigate_to(ctx: Context, state: str) -> dict:
    """
    Navigate to a game state via input sequences.

    Args:
        state: Target state — 'main_menu', 'gameplay', or 'pause_menu'.
    """
    # Validate state
    if state not in ("main_menu", "gameplay", "pause_menu"):
        return {
            "success": False,
            "error": f"Unknown state '{state}'. Valid values: 'main_menu', 'gameplay', 'pause_menu'."
        }

    # Check current state via status
    status_result = _run_game_cli("status")
    if not status_result.get("Running"):
        return {"success": False, "error": "Game is not running."}

    # Parse entity count from the raw/text response
    raw = status_result.get("raw", "")
    entity_count = 0
    for line in raw.split("\n"):
        if "Entity count:" in line:
            try:
                entity_count = int(line.split("Entity count:")[1].strip())
            except (ValueError, IndexError):
                pass

    if state == "main_menu":
        # Can't programmatically return to main menu without keyboard input.
        return {
            "success": False,
            "error": (
                "Returning to main_menu requires keyboard input (ESC or menu navigation) "
                "which is not yet implemented in the bridge. "
                "Use game_launch() to restart the game."
            )
        }

    if state == "pause_menu":
        # From gameplay: send ESC twice — requires keyboard input.
        return {
            "success": False,
            "error": (
                "pause_menu requires keyboard input (ESC key) which is not yet available. "
                "Use game_input(key='Escape') once keyboard input is implemented."
            )
        }

    # state == "gameplay"
    if entity_count > 50000:
        return {
            "success": True,
            "message": "Already at gameplay state.",
            "entityCount": entity_count
        }

    # Load AUTOSAVE_1 to enter gameplay
    save_result = _run_game_cli("load-save", "AUTOSAVE_1")
    if not save_result.get("success") and "already" not in save_result.get("error", "").lower():
        return save_result

    # Dismiss loading screen if present
    dismiss_result = _run_game_cli("dismiss")
    return {
        "success": True,
        "message": "Navigated to gameplay.",
        "loadResult": save_result,
        "dismissResult": dismiss_result,
        "entityCount": entity_count
    }


# ===========================================================================
# ADDRESSABLES CATALOG TOOLS  (direct JSON inspection — no CLI needed)
# ===========================================================================

@mcp.tool()
async def catalog_keys(ctx: Context, filter_term: str = "") -> dict:
    """
    List Addressables catalog keys (asset addresses used in the game).

    Args:
        filter_term: Optional substring filter on keys.
    """
    if not CATALOG_JSON.exists():
        return {"success": False, "error": f"Catalog not found: {CATALOG_JSON}"}
    try:
        with open(CATALOG_JSON, encoding="utf-8") as f:
            cat = json.load(f)
        ids: list[str] = cat.get("m_InternalIds", [])
        non_bundle = [s for s in ids if not s.startswith("{") and not s.endswith(".bundle")]
        if filter_term:
            non_bundle = [s for s in non_bundle if filter_term.lower() in s.lower()]
        return {"success": True, "keys": non_bundle[:200], "total": len(non_bundle)}
    except Exception as e:
        return {"success": False, "error": str(e)}


@mcp.tool()
async def catalog_bundles(ctx: Context) -> dict:
    """List all AssetBundle files registered in the Addressables catalog."""
    if not CATALOG_JSON.exists():
        return {"success": False, "error": f"Catalog not found: {CATALOG_JSON}"}
    try:
        with open(CATALOG_JSON, encoding="utf-8") as f:
            cat = json.load(f)
        bundles = [
            s.replace("{UnityEngine.AddressableAssets.Addressables.RuntimePath}", "")
            for s in cat.get("m_InternalIds", [])
            if s.endswith(".bundle")
        ]
        return {"success": True, "bundles": bundles, "count": len(bundles)}
    except Exception as e:
        return {"success": False, "error": str(e)}


# ===========================================================================
# DEBUG LOG TOOLS  (direct file read — instant, no CLI)
# ===========================================================================

@mcp.tool()
async def log_tail(ctx: Context, lines: int = 100) -> dict:
    """
    Read the last N lines from the DINOForge debug log.

    Args:
        lines: Number of lines to return (default 100).
    """
    if not DEBUG_LOG.exists():
        return {"success": False, "error": f"Debug log not found: {DEBUG_LOG}"}
    try:
        with open(DEBUG_LOG, encoding="utf-8", errors="replace") as f:
            all_lines = f.readlines()
        tail = all_lines[-lines:]
        return {"success": True, "lines": [l.rstrip() for l in tail], "total_lines": len(all_lines)}
    except Exception as e:
        return {"success": False, "error": str(e)}


@mcp.tool()
async def log_swap_status(ctx: Context) -> dict:
    """
    Parse the debug log and summarise asset swap results for the latest game session.
    Returns swap success count, pending count, entity counts, and any exceptions.
    """
    if not DEBUG_LOG.exists():
        return {"success": False, "error": f"Debug log not found: {DEBUG_LOG}"}
    try:
        with open(DEBUG_LOG, encoding="utf-8", errors="replace") as f:
            content = f.read()

        lines = content.splitlines()
        # Find the last OnCreate (start of latest session)
        session_start = 0
        for i, line in enumerate(lines):
            if "AssetSwapSystem.OnCreate" in line:
                session_start = i

        session_lines = lines[session_start:]
        completed = sum(1 for l in session_lines if "swap complete" in l)
        pending = sum(1 for l in session_lines if "live swap pending" in l)
        exceptions = [l for l in session_lines if "swap exception" in l]
        entity_lines = [l for l in session_lines if "swapped " in l and "/"]
        render_line = next((l for l in session_lines if "RenderMesh entities present" in l), None)
        probe_line = next((l for l in session_lines if "probe query created" in l), None)

        return {
            "success": True,
            "session_start_line": session_start,
            "swaps_complete": completed,
            "swaps_pending": pending,
            "exceptions": exceptions,
            "entity_swap_lines": entity_lines,
            "render_mesh_entities_present": render_line is not None,
            "probe_query_line": probe_line,
        }
    except Exception as e:
        return {"success": False, "error": str(e)}


@mcp.tool()
async def log_bepinex(ctx: Context, lines: int = 50) -> dict:
    """
    Read the last N lines from the BepInEx LogOutput.log.

    Args:
        lines: Number of lines to return.
    """
    bepinex_log = BEPINEX_DIR / "LogOutput.log"
    if not bepinex_log.exists():
        return {"success": False, "error": f"BepInEx log not found: {bepinex_log}"}
    try:
        with open(bepinex_log, encoding="utf-8", errors="replace") as f:
            all_lines = f.readlines()
        tail = all_lines[-lines:]
        return {"success": True, "lines": [l.rstrip() for l in tail]}
    except Exception as e:
        return {"success": False, "error": str(e)}


@mcp.tool()
async def log_debug_log(ctx: Context, lines: int = 500) -> dict:
    """
    Read the full DINOForge debug log (all entries, not just tail).
    Use this for deep analysis of swap exceptions, ECS world state, and pack loading.

    Args:
        lines: Maximum lines to return (default 500, use 0 for all).
    """
    if not DEBUG_LOG.exists():
        return {"success": False, "error": f"Debug log not found: {DEBUG_LOG}"}
    try:
        with open(DEBUG_LOG, encoding="utf-8", errors="replace") as f:
            all_lines = f.readlines()
        tail = all_lines[-lines:] if lines > 0 else all_lines
        return {
            "success": True,
            "path": str(DEBUG_LOG),
            "total_lines": len(all_lines),
            "returned_lines": len(tail),
            "lines": [l.rstrip() for l in tail],
        }
    except Exception as e:
        return {"success": False, "error": str(e)}


@mcp.tool()
async def log_packs_loaded(ctx: Context) -> dict:
    """
    Extract pack loading summary from the debug log (PacksLoader.OnAfterDeserialize output).
    Returns a list of loaded packs with their versions and any load errors.
    """
    if not DEBUG_LOG.exists():
        return {"success": False, "error": f"Debug log not found: {DEBUG_LOG}"}
    try:
        with open(DEBUG_LOG, encoding="utf-8", errors="replace") as f:
            content = f.read()

        packs: list[dict] = []
        for line in content.splitlines():
            if any(tag in line for tag in ("PacksLoader", "Pack loaded", "Pack load error",
                                            "warfare-starwars", "warfare-modern", "warfare-guerrilla",
                                            "economy-balanced", "example-balance")):
                ts = line[:23] if len(line) >= 23 else ""
                msg = line[24:].strip() if len(line) > 24 else line
                packs.append({"timestamp": ts, "line": msg.strip()})

        return {
            "success": True,
            "total_entries": len(packs),
            "entries": packs,
        }
    except Exception as e:
        return {"success": False, "error": str(e)}


# ===========================================================================
# RESOURCES  (live data readable without tool calls)
# ===========================================================================

@mcp.resource("game://status")
async def status_resource() -> str:
    return json.dumps(_run_game_cli("status"), indent=2)


@mcp.resource("log://debug")
async def debug_log_resource() -> str:
    result = await log_tail(None, lines=200)  # type: ignore[arg-type]
    return "\n".join(result.get("lines", [result.get("error", "")]))


@mcp.resource("catalog://bundles")
async def catalog_resource() -> str:
    result = await catalog_bundles(None)  # type: ignore[arg-type]
    return json.dumps(result, indent=2)


# ===========================================================================
# HEALTH CHECK ENDPOINT
# ===========================================================================

@mcp.custom_route("/health", methods=["GET"])
async def health_check(request: Request):
    """Health check endpoint for service monitoring and startup verification."""
    return JSONResponse({"status": "ok", "server": "dinoforge-mcp", "version": "0.13.0"})


# ===========================================================================
# HMR (HOT MODULE RELOAD) ENDPOINT
# ===========================================================================

@mcp.custom_route("/hmr", methods=["POST"])
async def hmr_route(request: Request):
    """
    HTTP endpoint for hot-module-reload notifications.
    Called by scripts/game/hot-reload.ps1 after deploying a new Runtime DLL.
    Clears internal caches and signals pack change watchers.
    """
    _reload_event.set()
    _reload_event.clear()
    return JSONResponse({"success": True, "message": "HMR event triggered — pack caches cleared"})


@mcp.tool()
async def notify_hmr(ctx: Context) -> dict:
    """
    Notify the MCP server that a hot-reload event has occurred (e.g. DLL deployed,
    packs reloaded). This clears internal caches and signals pack change.

    Typically called via POST http://127.0.0.1:8765/hmr after Runtime rebuild.
    """
    _reload_event.set()
    _reload_event.clear()
    return {"success": True, "message": "HMR event triggered — pack caches cleared"}


# ===========================================================================
# PROMPTS
# ===========================================================================

@mcp.prompt()
def debug_asset_swap(issue: str = "swaps not visible") -> str:
    return f"""Diagnose DINOForge asset swap issue: {issue}

Steps:
1. log_swap_status → check swaps_complete, render_mesh_entities_present
2. If render_mesh_entities_present=False → IncludePrefab fix not deployed, rebuild Runtime DLL
3. If swaps_complete=0 → check entity_swap_lines for "swapped 0/N entities"
4. game_query_entities("Unity.Rendering.RenderMesh") → verify entity count > 0
5. game_screenshot → visual confirmation
6. catalog_keys("") → verify asset addresses are NOT in catalog (normal for unit swaps)

Key facts:
- ALL DINO entities are ECS Prefab entities — EntityQueryOptions.IncludePrefab is mandatory
- Phase 1 (catalog disk patch) will always skip unit/building swaps — this is normal
- Phase 2 (live RenderMesh entity swap) is the primary mechanism
- 600-frame delay before swaps fire (~10s at 60fps)"""


@mcp.prompt()
def asset_pipeline_workflow(pack: str = "warfare-starwars") -> str:
    return f"""Asset pipeline workflow for pack: {pack}

1. pack_validate("{pack}") → verify YAML is valid
2. asset_validate("{pack}") → verify asset_pipeline.yaml
3. asset_import("{pack}") → download/convert source assets
4. asset_optimize("{pack}") → generate LOD variants
5. asset_build("{pack}") → full pipeline
6. game_launch → start test instance
7. game_wait_world → wait for ECS world
8. log_swap_status → verify swaps fired
9. game_screenshot → visual confirmation"""


# ===========================================================================
# Entry point
# ===========================================================================

def main() -> None:
    parser = argparse.ArgumentParser(
        description="DINOForge MCP Server (FastMCP 3.1.1)",
        epilog="Examples:\n  python -m dinoforge_mcp.server                    # stdio (for MCP client)\n  python -m dinoforge_mcp.server --http --port 8765  # HTTP/SSE (persistent server)",
        formatter_class=argparse.RawDescriptionHelpFormatter,
    )
    parser.add_argument(
        "--http",
        action="store_true",
        help="Run as HTTP/SSE server instead of stdio (allows hot-reload without restart)",
    )
    parser.add_argument(
        "--port",
        type=int,
        default=8765,
        help="HTTP server port (default: 8765, ignored if --http not set)",
    )
    parser.add_argument(
        "--host",
        type=str,
        default="127.0.0.1",
        help="HTTP server host (default: 127.0.0.1, ignored if --http not set)",
    )
    args, remaining = parser.parse_known_args()

    if args.http:
        # HTTP/SSE mode: Uvicorn ASGI server with persistent process
        logger.info(f"Starting DINOForge MCP in HTTP mode at {args.host}:{args.port}")
        logger.info(f"  JSON-RPC endpoint: http://{args.host}:{args.port}")
        mcp.run(
            transport="http",
            host=args.host,
            port=args.port,
        )
    else:
        # stdio mode: default for direct MCP client (Claude, other tools)
        logger.info("Starting DINOForge MCP in stdio mode (for MCP client)")
        mcp.run()


if __name__ == "__main__":
    main()
