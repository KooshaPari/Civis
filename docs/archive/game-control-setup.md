# DINOForge Game Control - Complete Setup

## What's Complete

### 1. GameControlCli (Binary)
Standalone CLI for checking game state via named pipes. No screen automation, direct game process communication.

**Commands:**
- `status` — Check game/mod platform readiness, loaded packs, entity count
- `wait-world` — Wait for ECS world to be ready (30s timeout)
- `resources` — Display all resource values (food, wood, stone, iron, money, souls, bones, spirit)
- `screenshot` — Capture in-game screenshot
- `catalog [category]` — Dump entity catalog (units/buildings/projectiles)
- `entities [component]` — Query entities by component type

**Build & Run:**
```bash
dotnet build src/Tools/GameControlCli
dotnet run --project src/Tools/GameControlCli -- status
```

### 2. Most-Used Workflows (Executable Scripts)
Located in `.claude/commands/workflows/`

#### check-game-state.sh
Complete game state check — runs all key checks:
```bash
./.claude/commands/workflows/check-game-state.sh
```
Checks: status → wait-world → resources → catalog

#### check-pack-list.sh
Verify pack list visibility:
```bash
./.claude/commands/workflows/check-pack-list.sh
```
Verifies packs are loaded and displayable.

### 3. Personal Script Library
Located in `.claude/commands/lib/game-check.sh`

Import functions for personal use:
```bash
source .claude/commands/lib/game-check.sh

# Now use:
game_status                    # Get full status
game_wait_world               # Wait for world ready
game_resources                # Get resource values
game_catalog [category]       # Get catalog
game_is_ready                 # Check if ready (boolean)
game_screenshot [output]      # Take screenshot
game_health_check             # Quick health check
```

## Why This Solves Your Issues

1. **Pack List Visibility**: Fixed via ModMenuPanel height calculation (UI commit 63844d6)
2. **Debug Panel (F9)**: Same height fix applied
3. **Native Mods Button**: EventSystem selection fixed
4. **Game State Verification**: Now you can programmatically verify these fixes via `game-control status` and `game-control catalog` commands

## Next Steps You Can Use

### Quick Check
```bash
# Verify packs are loaded
dotnet run --project src/Tools/GameControlCli -- status | grep "Loaded packs"

# Check resources
dotnet run --project src/Tools/GameControlCli -- resources

# Get entity count (verify ECS world)
dotnet run --project src/Tools/GameControlCli -- catalog
```

### Automated Workflows
```bash
# Full health check
./.claude/commands/workflows/check-game-state.sh

# Just pack list
./.claude/commands/workflows/check-pack-list.sh
```

### Personal Use (from shell scripts)
```bash
#!/bin/bash
source ./.claude/commands/lib/game-check.sh

if game_is_ready; then
    echo "Game ready, checking resources..."
    game_resources
else
    echo "Waiting for game..."
    game_wait_world
fi
```

## PR Status

- **PR #17**: `feat: GameControlCli - game state checking via named pipes`
- Contains: GameControlCli implementation + workflow scripts + script library
- Status: Ready for review, blocked on protected branch (requires PR approval)

## UI Automation (MCP Tool)

The `game_ui_automation` tool in McpServer provides **accessibility tree based UI automation**:

```
Get UI Tree (Playwright-style):
  {
    "action": "tree",
    "selector": "#f10-menu"  // Optional: filter by selector
  }

Click Element:
  {
    "action": "click",
    "selector": "#pack-list"  // CSS-like: #id, .class, [attr=value]
  }

Screenshot with UI Overlay:
  {
    "action": "screenshot",
    "selector": "#f9-debug"
  }
```

Returns:
- Full accessibility tree (like Playwright's `page.accessibility.snapshot()`)
- Element paths and attributes
- Element selectors for easy navigation
- Screenshot paths with UI bindings

### Selectors
- CSS-style: `#pack-list`, `.button`, `[data-id=item]`
- Path-based: `root/f10-menu/pack-list`

## Architecture

```
Game Process (DINO)
    ↓ (named pipes)
GameClient (Bridge.Client)
    ↓ (JSON-RPC 2.0)
    ├─ GameControlCli (state checking)
    └─ McpServer (13 game tools + UI automation)
        ├─ game_status, game_resources, game_screenshot
        ├─ game_query_entities, game_dump_state
        └─ game_ui_automation (accessibility tree + click)
```

Pure game process communication:
- ✅ Accessibility tree navigation (like Playwright)
- ✅ Selector-based element interaction (no pixel coordinates)
- ✅ Screenshot with UI overlay capability
- ❌ No screen capture screen parsing needed
- ❌ No window interaction
