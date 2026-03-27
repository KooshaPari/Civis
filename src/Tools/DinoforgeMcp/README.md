# DINOForge MCP Server

FastMCP 3.0 server for DINOForge game automation.

## Features

### Tools (21)

**Game State (3)**
- `game_status` - Check if game is running
- `game_resources` - Get resources (gold, wood, food)
- `game_packs` - Get loaded mod packs

**Entity Management (2)**
- `query_entities` - Query ECS entities by component
- `spawn_unit` - Spawn unit at position

**UI Automation (5)**
- `ui_tree` - Capture UI hierarchy
- `ui_query` - Query UI elements
- `ui_click` - Click UI element
- `ui_wait` - Wait for UI state
- `ui_expect` - Assert UI condition

**Mod Operations (2)**
- `apply_override` - Apply stat override
- `reload_packs` - Hot reload mods

**Screenshots (1)**
- `take_screenshot` - Capture game screenshot

**Asset Pipeline (5)**
- `asset_list` - List pack assets
- `asset_validate` - Validate assets
- `asset_import` - Import assets
- `asset_optimize` - Optimize/generate LOD
- `asset_build` - Full pipeline build

**Pack Operations (2)**
- `pack_validate` - Validate pack YAML
- `pack_build` - Build pack

**Debug (3)**
- `dump_world` - Dump world state
- `dump_entities` - Dump entities
- `component_map` - Get ECS mapping

### Resources (5)
- `game://status` - Game status
- `game://resources` - Resources
- `game://packs` - Packs
- `game://ui-tree` - UI hierarchy
- `game://entities` - Entities

### Prompts (4)
- `debug_prompt` - Debug workflow
- `testing_prompt` - UI testing workflow
- `modding_prompt` - Mod development
- `asset_pipeline_prompt` - Asset pipeline

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

## Usage

```bash
# Run standalone
python -m dinoforge_mcp.server --http --port 8765 --host 127.0.0.1

# Run with default foreground settings
python -m dinoforge_mcp.server --http

# Or use the included config
cp .claude/mcp-servers.json ~/.claude/settings.json
``` 

## Architecture

```
Claude Code → FastMCP → CLI → Named Pipe → Game
```

## Requirements

- Python 3.10+
- FastMCP 3.0+
- .NET SDK
- DINO game with DINOForge mod
