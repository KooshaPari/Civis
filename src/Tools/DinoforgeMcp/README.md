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

## Requirements

- Python 3.10+
- FastMCP 3.0+
- .NET SDK
- DINO game with DINOForge mod
