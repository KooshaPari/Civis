---
description: Start and manage the DINOForge development harness (MCP + optional HMR watcher).
---

# dev-harness

Manage the long-lived DINOForge MCP harness used by CC game automation.
CC session hooks attempt automatic startup; this command is for manual control and status.

**Usage**: `/dev-harness [--watch]`

**Arguments**: $ARGUMENTS

## What This Does

Runs `./scripts/start-mcp.ps1` in managed mode and prints the current lifecycle state.

- `--watch`: starts companion `scripts/game/hot-reload.ps1 -Watch` so runtime reloads notify MCP automatically.
- `stop`: stop running MCP + watcher.
- `status`: show PID + listener state.
- `service`: manage a persistent service wrapper (`Install|Status|Start|Stop|Uninstall`).

## Default Actions

1. If MCP is not running:
   - start it with `./scripts/start-mcp.ps1 -Detached`
   - start hot-reload watcher if `--watch` is supplied
2. If MCP is already running:
   - print PID and endpoint details
3. Verify port listener is active and report URL:
   - `http://127.0.0.1:8765/messages`

## Usage Examples

```powershell
# Start MCP only
/dev-harness

# Start MCP + hot-reload companion
/dev-harness --watch

# Check status
./scripts/start-mcp.ps1 -Action status

# Stop
./scripts/start-mcp.ps1 -Action stop
```

CC hooks attempt automatic MCP startup; use `/dev-harness` only when you want explicit control (force start, status, stop, restart, watcher mode).

For service-managed deployment, run:

```powershell
pwsh -File scripts/services/mcp-service.ps1 -Action Install
pwsh -File scripts/services/mcp-service.ps1 -Action Start
pwsh -File scripts/services/mcp-service.ps1 -Action Stop
pwsh -File scripts/services/mcp-service.ps1 -Action Uninstall
```
