# DINOForge MCP Server

FastMCP 3.0 server for DINOForge game automation.

## Features

### Tools (39+)

`server.py` exports nearly 40 FastMCP tools covering:

- Game launch/state/input
- Screenshot + screen analysis
- ECS/entity query and stat tooling
- Mod pack and asset tooling
- Catalog/debug utilities
- Runtime reload and HMR signaling

### Resources

- `game://status`
- `game://resources`
- `game://packs`
- `game://ui-tree`
- `game://entities`

### Prompts

Debug/testing prompts are registered directly in the MCP runtime and kept in sync with tool surfaces.

## FastMCP 3.0 Features

- Native OpenTelemetry tracing
- Background tasks via ctx
- Response size limiting
- Pydantic models for validation
- Async/await throughout
- Rich tool descriptions

## Installation

```bash
pip install fastmcp pydantic
```

## FastMCP Runtime (HTTP/SSE)

The server defaults to HTTP/SSE when `--http --port 8765 --host 127.0.0.1` is supplied. This is the recommended
mode for live-reload and long-lived MCP clients because the process stays running while game DLLs are rebuilt.

```bash
python -m dinoforge_mcp.server --http --port 8765 --host 127.0.0.1
```

## Claude Code Integration

Recommended CC config (URL transport):

```json
{
  "mcpServers": {
    "dinoforge": {
      "url": "http://127.0.0.1:8765"
    }
  }
}
```

For quick local startup from the repo, use the managed launcher:

```powershell
./scripts/start-mcp.ps1 -Detached
```

Add `-Watch` for companion hot-reload signaling.
Set `DINOFORGE_MCP_WATCH=1` if you'd rather keep watcher enabled automatically.

If you want MCP to stay resident across tool sessions, install a managed launcher:

- **Windows**: use `scripts/services/windows/register-mcp-task.ps1 -Install`
- **Linux**: use `scripts/services/systemd/dinoforge-mcp.service`
- **macOS**: use `scripts/services/launchd/com.dinoforge.mcp.plist`

## Usage

```bash
# Run standalone
python -m dinoforge_mcp.server --http --port 8765 --host 127.0.0.1

# Run with default foreground settings
python -m dinoforge_mcp.server --http

# Or use the included MCP transport config as a separate file
cp .claude/mcp-servers.json ~/.claude/mcp-servers.json
``` 

## Architecture

```
Claude Code → FastMCP → CLI → Named Pipe → Game
```

## MCP Tools (39 total)

### Game Bridge (15 tools)
- `game_launch` — Launch primary game instance (32-bit or 64-bit)
- `game_launch_test` — Launch isolated TEST instance on hidden desktop
- `game_launch_vdd` — Launch on dedicated virtual display (IDD driver, future)
- `game_status` — Get running state, entity count, loaded packs
- `game_resources` — Read current resources (gold, lumber, etc.)
- `game_query_entities` — Query ECS entities by component type
- `game_get_stat` — Read a stat value on an entity
- `game_apply_override` — Apply a stat override via ComponentModifier
- `game_screenshot` — Capture game window screenshot (GPU backbuffer)
- `game_analyze_screen` — Screenshot + visual analysis (pHash + CLIP)
- `game_input` — Inject keyboard/mouse input (Win32 SendInput)
- `game_click_button` — Click UI button by coordinates
- `game_ui_automation` — Automated menu/HUD navigation
- `game_navigate_to` — Navigate to game state (main_menu/gameplay/pause_menu)
- `game_wait_and_screenshot` — Poll for visual change, then screenshot

### Asset Pipeline (4 tools)
- `asset_validate` — Validate asset_pipeline.yaml schema
- `asset_import` — Import GLB/FBX files to JSON mesh data
- `asset_optimize` — Generate LOD variants (decimation)
- `asset_build` — Full pipeline: import → optimize → generate → bundle

### Pack Management (3 tools)
- `pack_validate` — Validate pack.yaml and schema compliance
- `pack_build` — Compile pack: validate → assets → bundle
- `pack_list` — List all available packs

### Game Data (2 tools)
- `catalog_keys` — List all Addressables keys in catalog
- `catalog_bundles` — List all asset bundles by size

### Diagnostics & Reload (10 tools)
- `log_tail` — Read last N lines of BepInEx/dinoforge_debug.log
- `game_dump_state` — Trigger entity dump to file
- `game_get_component_map` — Get ComponentMap (30+ vanilla mappings)
- `game_reload_packs` — Hot-reload packs without restarting
- `game_verify_mod` — Verify DINOForge mod is loaded
- `game_wait_for_world` — Wait until ECS world is ready
- `swap_status` — Report entity swap phases and counts
- `notify_hmr` — Signal HMR reload event
- Plus 2 additional diagnostic tools

## Testing

```bash
pip install -e ".[dev]"
pytest tests/ -v
```

Expected: 186 tests passing in ~3 seconds

Test coverage:
- Asset/pack tools (31 tests)
- Error handling (34 tests)
- Game bridge (45 tests)
- Game launch (40 tests)
- Log analysis (36 tests)

## Visual Analysis (CLIP + pHash)

The `game_analyze_screen` tool uses three-tier image analysis:

1. **pHash** (perceptual hash): ~31ms, matches golden reference screenshots
2. **CLIP** (zero-shot classification): ~1.3s cached, identifies UI elements
3. **OpenCV** (fallback): ~53ms, color/contour analysis for non-ML paths

Used for automated visual validation without game restart.

## Requirements

- Python 3.10+
- FastMCP 3.0+
- .NET SDK
- DINO game with DINOForge mod
- PIL/Pillow (for screenshots)
- Optional: CLIP model for advanced vision tasks
