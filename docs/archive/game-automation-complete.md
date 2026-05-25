# Game Automation Complete - Full Stack

You now have **three levels** of game automation:

## Level 1: State Checking (GameControlCli)

Direct game process communication for state verification.

```bash
dotnet run --project src/Tools/GameControlCli -- status
dotnet run --project src/Tools/GameControlCli -- resources
dotnet run --project src/Tools/GameControlCli -- catalog
```

**No UI interaction, pure game state via named pipes.**

## Level 2: UI Automation (MCP Tool)

Accessibility tree based UI navigation - like **Playwright MCP**.

```
game_ui_automation:
  action: "tree"           # Get UI structure
  selector: "#pack-list"   # CSS-like selectors

game_ui_automation:
  action: "click"          # Interact with UI
  selector: "#f10-menu"

game_ui_automation:
  action: "screenshot"     # Capture with UI info
```

**Returns accessibility tree (not pixel coordinates).**

## Level 3: Personal Workflows (Scripts)

Reusable bash/shell scripts for recurring tasks.

```bash
# Most-used workflows
./.claude/commands/workflows/check-game-state.sh
./.claude/commands/workflows/check-pack-list.sh

# Personal library
source ./.claude/commands/lib/game-check.sh
game_status
game_resources
game_is_ready
```

---

## Architecture

```
Game Process (DINO)
    │
    ├─ Named Pipes (JSON-RPC)
    │   ├─ GameControlCli (state checking)
    │   │   └─ status, resources, catalog, entities, screenshot
    │   │
    │   └─ McpServer (14 tools via MCP)
    │       ├─ Game tools (status, screenshot, etc.)
    │       └─ game_ui_automation (accessibility tree)
    │
    └─ CLI/Scripts
        ├─ GameControlCli binary (dotnet run)
        ├─ Workflows (.claude/commands/workflows/)
        └─ Script library (.claude/commands/lib/)
```

## Why This Approach?

### ✅ No Screen Capture Needed
- Accessibility tree provides UI structure explicitly
- No OCR, no vision parsing, no pixel analysis
- Works at any resolution

### ✅ No Window Automation
- Direct game process communication
- No window focus issues
- No external tool dependencies (pyautogui, AppKit, etc.)

### ✅ Accessibility Tree (Playwright-Style)
```json
{
  "id": "f10-menu",
  "type": "menu",
  "label": "F10 Pack List Menu",
  "children": [
    {
      "id": "pack-list",
      "type": "list",
      "label": "Pack List Sidebar",
      "path": "root/f10-menu/pack-list",
      "visible": true
    }
  ]
}
```

### ✅ Selector-Based Navigation
- `#id` - by element ID
- `.class` - by class
- `[attr=value]` - by attribute
- `path` - by tree path

---

## Real-World Use Cases

### Check Pack List Fix
```
Before (broken):
  game_ui_automation(tree, "#pack-list")
  → visible: false

After (fixed):
  game_ui_automation(tree, "#pack-list")
  → visible: true
  → children: [pack0, pack1, pack2, ...]
```

### Automated Testing
```python
def test_ui_state():
    tree = game_ui_automation(action="tree")
    assert tree['f10-menu']['visible']
    assert tree['f9-debug']['visible']
    assert len(tree['f10-menu']['pack-list']['children']) > 0
```

### Integration with Claude Code
Claude can now:
1. Get accessibility tree
2. Click elements via selectors
3. Take screenshots with UI overlays
4. All without pixel coordinates

---

## Files & Structure

### GameControlCli
- `src/Tools/GameControlCli/` - Binary (6 commands)
- Commands: status, wait-world, resources, screenshot, catalog, entities

### MCP Tools (McpServer)
- `src/Tools/McpServer/Tools/GameUIAutomationTool.cs` - Accessibility tree tool
- Registered as `game_ui_automation` in MCP server
- Actions: tree, click, screenshot

### Workflows (Most-Used)
- `.claude/commands/workflows/check-game-state.sh` - Full health check
- `.claude/commands/workflows/check-pack-list.sh` - Pack verification

### Script Library (Personal)
- `.claude/commands/lib/game-check.sh` - Reusable functions
- `source` this file to use: game_status(), game_resources(), etc.

### Documentation
- `GAME_CONTROL_SETUP.md` - Quick start guide
- `GAME_CONTROL_USAGE_EXAMPLES.md` - Comprehensive examples
- `GAME_AUTOMATION_COMPLETE.md` - This file

---

## Next Steps for Expansion

### Short Term
1. ✅ **Accessibility tree** - Already built (GameUIAutomationTool)
2. ✅ **Selector-based clicks** - Already built
3. ✅ **Screenshot integration** - Already built
4. **Expand tree** - Add more UI elements (buttons, inputs, etc.)

### Medium Term
1. **Real element binding** - Connect tree to actual RectTransform components
2. **Interactive elements** - Input fields, sliders, checkboxes
3. **Event callbacks** - Listen for UI state changes
4. **Screenshot overlays** - Highlight elements on captured images

### Long Term
1. **Record/playback** - Record UI interactions and replay them
2. **Visual assertions** - "Element at position X looks correct"
3. **Performance monitoring** - Track UI load times
4. **Accessibility audits** - Verify UI meets accessibility standards

---

## PR Status

**PR #17: feat/game-control-cli** - Waiting for merge approval

Includes:
- GameControlCli implementation (6 commands)
- Game UI automation tool (accessibility tree)
- Workflow scripts (2 most-used workflows)
- Script library (7 reusable functions)
- Complete documentation (3 guides + examples)

All builds successfully (0 errors, 7 warnings in Assetctl only).

---

## How Claude Code Uses This

```
User: "Check if pack list is visible"

Claude Code:
1. Calls game_ui_automation(tree, "#pack-list")
2. Gets accessibility tree with visibility status
3. Reports: "Pack list is visible with 3 packs loaded"

User: "Click the first pack"

Claude Code:
1. Calls game_ui_automation(click, "root/f10-menu/pack-list/item[0]")
2. Interacts with UI element
3. Calls game_ui_automation(screenshot)
4. Verifies action took effect
```

No pixel coordinates. No OCR. Pure accessibility tree navigation.
