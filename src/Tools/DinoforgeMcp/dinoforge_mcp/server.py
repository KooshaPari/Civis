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

import asyncio
import base64
import json
import logging
import os
import subprocess
from pathlib import Path
from typing import Any

from dotenv import load_dotenv
from fastmcp import FastMCP, Context
from pydantic import BaseModel, Field

load_dotenv()
logging.basicConfig(level=logging.DEBUG if os.getenv("DINOFORGE_MCP_DEBUG") else logging.WARNING)
logger = logging.getLogger("dinoforge_mcp")

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
        "--",
        *args,
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


async def _launch_hidden(exe_path: str, desktop_name: str = "DINOForge_Agent") -> dict:
    """Launch game on a hidden Win32 desktop using CreateDesktop."""
    ps_script = r"""
param($ExePath, $DesktopName)
Add-Type -AssemblyName System.Drawing
Add-Type @"
using System;
using System.Runtime.InteropServices;
public class Win32Desktop {
    [DllImport("user32.dll")] public static extern IntPtr CreateDesktop(string lpszDesktop, IntPtr lpszDevice, IntPtr pDevmode, int dwFlags, uint dwDesiredAccess, IntPtr lpsa);
    [DllImport("user32.dll")] public static extern bool CloseDesktop(IntPtr hDesktop);
    [DllImport("kernel32.dll")] public static extern bool CreateProcess(string lpAppName, string lpCmdLine, IntPtr lpPA, IntPtr lpTA, bool bInherit, uint dwFlags, IntPtr lpEnv, string lpCurDir, ref STARTUPINFO lpSI, out PROCESS_INFORMATION lpPI);
    [StructLayout(LayoutKind.Sequential, CharSet=CharSet.Auto)] public struct STARTUPINFO { public int cb; public string lpReserved; public string lpDesktop; public string lpTitle; public int dwX, dwY, dwXSize, dwYSize, dwXCountChars, dwYCountChars, dwFillAttribute, dwFlags; public short wShowWindow, cbReserved2; public IntPtr lpReserved2, hStdInput, hStdOutput, hStdError; }
    [StructLayout(LayoutKind.Sequential)] public struct PROCESS_INFORMATION { public IntPtr hProcess, hThread; public int dwProcessId, dwThreadId; }
}
"@
$desktop = [Win32Desktop]::CreateDesktop($DesktopName, [IntPtr]::Zero, [IntPtr]::Zero, 0, 0x01FF, [IntPtr]::Zero)
if ($desktop -eq [IntPtr]::Zero) { Write-Output "ERROR: CreateDesktop failed"; exit 1 }
$si = New-Object Win32Desktop+STARTUPINFO
$si.cb = [System.Runtime.InteropServices.Marshal]::SizeOf($si)
$si.lpDesktop = $DesktopName
$si.dwFlags = 0x00000001
$si.wShowWindow = 0
$pi = New-Object Win32Desktop+PROCESS_INFORMATION
$exeDir = Split-Path $ExePath -Parent
$ok = [Win32Desktop]::CreateProcess($ExePath, $null, [IntPtr]::Zero, [IntPtr]::Zero, $false, 0x00000010, [IntPtr]::Zero, $exeDir, [ref]$si, [ref]$pi)
if ($ok) { Write-Output "PID:$($pi.dwProcessId)" } else { Write-Output "ERROR: CreateProcess failed" }
"""
    result = await asyncio.to_thread(
        subprocess.run,
        ["powershell", "-ExecutionPolicy", "Bypass", "-Command", ps_script, "-ExePath", exe_path, "-DesktopName", desktop_name],
        capture_output=True, text=True, timeout=30
    )
    stdout = result.stdout.strip()
    if stdout.startswith("PID:"):
        pid = int(stdout[4:])
        return {"success": True, "pid": pid, "desktop": desktop_name, "hidden": True}
    return {"success": False, "error": stdout or result.stderr}


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
async def game_resources(ctx: Context) -> dict:
    """Get current in-game resources (gold, wood, food, etc.)."""
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
        hidden: If True, launch on an invisible Win32 desktop (CreateDesktop).
    """
    if not GAME_EXE.exists():
        return {"success": False, "error": f"Game exe not found: {GAME_EXE}"}
    try:
        if hidden:
            return await _launch_hidden(str(GAME_EXE), "DINOForge_Agent")
        subprocess.Popen([str(GAME_EXE)], cwd=str(GAME_DIR))
        return {"success": True, "message": f"Launched: {GAME_EXE}. Use game_wait_world to wait for ECS world."}
    except Exception as e:
        return {"success": False, "error": str(e)}


@mcp.tool()
async def game_launch_test(ctx: Context, hidden: bool = False) -> dict:
    """
    Launch the TEST instance of DINO (second concurrent instance for testing).
    Uses G:\\SteamLibrary\\steamapps\\common\\Diplomacy is Not an Option_TEST\\.
    Kill existing test instances first if needed.

    Args:
        hidden: If True, launch on an invisible Win32 desktop (CreateDesktop).
    """
    test_dir = r"G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option_TEST"
    test_exe = Path(test_dir) / "Diplomacy is Not an Option.exe"
    if not test_exe.exists():
        return {"success": False, "error": f"Test game exe not found: {test_exe}"}
    try:
        if hidden:
            return await _launch_hidden(str(test_exe), "DINOForge_Agent_Test")
        subprocess.Popen([str(test_exe)], cwd=test_dir)
        return {"success": True, "message": f"Launched TEST instance: {test_exe}. Use game_wait_world to wait for ECS world."}
    except Exception as e:
        return {"success": False, "error": str(e)}


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
    mcp.run()


if __name__ == "__main__":
    main()
