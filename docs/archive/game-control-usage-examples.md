# DINOForge Game Control - Usage Examples

## CLI Usage (GameControlCli)

### Check Game State
```bash
# Get full status with packs
dotnet run --project src/Tools/GameControlCli -- status

# Check resources
dotnet run --project src/Tools/GameControlCli -- resources

# Get entity catalog
dotnet run --project src/Tools/GameControlCli -- catalog

# Take screenshot
dotnet run --project src/Tools/GameControlCli -- screenshot output.png

# Wait for world ready
dotnet run --project src/Tools/GameControlCli -- wait-world
```

### Run Workflow Scripts
```bash
# Full health check
./.claude/commands/workflows/check-game-state.sh

# Verify pack list
./.claude/commands/workflows/check-pack-list.sh
```

### Use Script Library
```bash
#!/bin/bash
source ./.claude/commands/lib/game-check.sh

# Check if game is ready
if game_is_ready; then
    game_resources
    game_screenshot "status_$(date +%s).png"
else
    echo "Waiting for game..."
    game_wait_world
fi
```

## MCP Tool Usage (via Claude Code)

### Get UI Accessibility Tree
Query the game's UI structure (like Playwright's accessibility tree):

```
Call: game_ui_automation
Parameters:
  action: "tree"
  selector: "#f10-menu"
```

Response:
```json
{
  "success": true,
  "tree": {
    "id": "root",
    "type": "root",
    "label": "Game UI",
    "children": [
      {
        "id": "f10-menu",
        "type": "menu",
        "label": "F10 Pack List Menu",
        "children": [
          {
            "id": "pack-list",
            "type": "list",
            "label": "Pack List Sidebar",
            "path": "root/f10-menu/pack-list"
          }
        ]
      }
    ]
  }
}
```

### Click UI Element
Interact with UI using selectors (no pixel coordinates):

```
Call: game_ui_automation
Parameters:
  action: "click"
  selector: "#pack-list"
```

Response:
```json
{
  "success": true,
  "selector": "#pack-list",
  "action": "Clicking F10 pack list (opens menu)",
  "path": "root/elements/pack-list"
}
```

### Capture Screenshot with UI Info
```
Call: game_ui_automation
Parameters:
  action: "screenshot"
  selector: "#f9-debug"
```

Response:
```json
{
  "success": true,
  "path": "screenshot_1234567890.png",
  "message": "Screenshot captured with UI automation",
  "selector": "#f9-debug"
}
```

## Common Selectors

### Pack List Menu (F10)
```
#f10-menu          - F10 menu root
#pack-list         - Pack list sidebar
#pack-details      - Pack details panel
```

### Debug Panel (F9)
```
#f9-debug          - F9 debug panel
#debug-info        - Debug information display
```

### Native Menu
```
#native-menu       - Native game menu root
#options-btn       - Options button
#mods-btn          - Mods button
```

## Use Case: Verify Pack List Fix

### Before Fix (Pack list invisible)
```bash
# Via MCP (Claude Code):
game_ui_automation:
  action: "tree"
  selector: "#pack-list"

# Returns: pack-list element with visible: false
```

### After Fix (Pack list visible)
```bash
# Via MCP (Claude Code):
game_ui_automation:
  action: "tree"
  selector: "#pack-list"

# Returns: pack-list element with visible: true
# + full tree of pack entries
```

## Use Case: Automated Testing

### Test Pack List Visibility
```python
# Pseudo-code for testing
def test_pack_list_visible():
    tree = game_ui_automation(action="tree", selector="#pack-list")
    assert tree['success']
    assert tree['tree']['visible'] == True
    assert len(tree['tree']['children']) > 0
    print(f"✓ Pack list visible with {len(tree['tree']['children'])} items")

test_pack_list_visible()
```

### Test Resources Update
```python
def test_resources():
    resources = game_status()
    assert resources['success']
    initial_food = resources['food']

    # Do something in game...

    resources_after = game_status()
    if resources_after['food'] != initial_food:
        print(f"✓ Food changed from {initial_food} to {resources_after['food']}")
    else:
        print("✗ Resources did not update")

test_resources()
```

## Use Case: Integration with Claude Code

### Check Game State in Agent Workflow
```
# Claude Code can now:
1. Take screenshot via game_ui_automation
2. Get accessibility tree
3. Click elements by selector
4. Verify UI state without screen parsing

# Example workflow:
- game_ui_automation(tree) → get UI structure
- Check if pack list is visible
- game_ui_automation(click, "#pack-list") → open pack list
- Take screenshot to visually verify
- game_status() → verify game data matches UI
```

## Architecture Decision: Why Accessibility Trees?

Similar to **Playwright MCP** approach:
- ✅ **No pixel coordinates** - Use selectors instead
- ✅ **No OCR/vision parsing** - Tree structure is explicit
- ✅ **No screenshot analysis** - Pure game process communication
- ✅ **Scalable selectors** - Works at any resolution
- ✅ **Reliable automation** - UI structure is canonical source of truth

## Next Steps

1. **Expand accessibility tree** - Add more UI elements to the tree structure (currently stub)
2. **Real element binding** - Connect tree to actual game UI RectTransform elements
3. **Interactive elements** - Add support for input fields, sliders, checkboxes
4. **Event callbacks** - Hook UI element state changes to tree updates
5. **Screenshot overlays** - Highlight UI elements on captured screenshots
