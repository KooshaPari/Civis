# Extended Feature Evaluation Pipeline (Pipeline A)

## Overview

The `/eval-game-features` command extends the existing `/prove-features` pipeline to validate **5 additional runtime capabilities** using MCP tools and VLM screenshot analysis.

**Core 3 features** (validated by `/prove-features`):
- Mods button visible
- F9 debug overlay
- F10 mod menu

**Extended 5 features** (validated by `/eval-game-features`):
- **A-2**: Stat override (HP=999 via `game_apply_override`)
- **A-3**: Hot reload (YAML change picked up without restart via `game_reload_packs`)
- **A-4**: Economy pack loaded + resources queryable
- **A-5**: Scenario pack loaded + scenario system active
- **A-6**: Asset swap (clone trooper visual model visible)

## File Structure

### Command Definition
- **`.claude/commands/eval-game-features.md`** — Complete orchestration guide for Claude
  - 6 main steps (game bridge check, A-2, A-3, A-4, A-5, A-6)
  - Explicit MCP tool calls with parameter names and types
  - VLM prompts for each feature screenshot
  - JSON report structure and bundling instructions
  - Error handling policy (continue on individual failures, aggregate results)

### Helper Script
- **`scripts/game/eval-game-features.ps1`** — Pre-flight checks
  - Verifies MCP server health on `http://127.0.0.1:8765`
  - Builds test projects to ensure no compilation errors
  - Prepares output directories (`$env:TEMP\DINOForge\capture` and `docs/proof-of-features/`)
  - Provides instructions for users

## MCP Tools Used

| Tool | Feature | Purpose |
|------|---------|---------|
| `game_status` | Baseline | Verify game is running, entity count > 0 |
| `game_apply_override` | A-2 | Apply stat override (HP=999) |
| `game_get_stat` | A-2 | Query stat value to verify override applied |
| `game_input` | A-2 | Press F9 to open debug overlay for screenshot |
| `game_screenshot` | All | Capture current game window (GDI, no compression) |
| `game_analyze_screen` | All | VLM analysis of screenshot with custom prompt |
| `game_reload_packs` | A-3 | Trigger hot reload (writes HMR signal file) |
| `log_tail` | A-3, A-5 | Tail BepInEx log to find reload/scenario evidence |
| `game_get_resources` | A-4 | Query current economy resources (gold, food, etc.) |
| `game_dump_state` | A-6 | Dump unit entities to find clone trooper models |
| `log_swap_status` | A-6 | Query asset swap completion count |

## Evaluation Flow

### Pre-requisites
1. Game is running with DINOForge Runtime loaded
2. MCP server is running on `http://127.0.0.1:8765`
3. (Optional) Run `/prove-features` first to ensure stable boot

### Execution
1. **Step 1**: Call `game_status` — verify game is ready (running, entity count > 0)
2. **Feature A-2**: Apply override, query stat, screenshot F9 overlay, VLM validate
3. **Feature A-3**: Trigger reload, tail logs for "reload" keyword, screenshot, VLM validate game stability
4. **Feature A-4**: Check economy pack loaded, query resources, screenshot, VLM validate resource HUD
5. **Feature A-5**: Check scenario pack loaded, tail logs for "ScenarioRunner", screenshot, VLM validate gameplay
6. **Feature A-6**: Check swap status, dump entities for clone troopers, screenshot, VLM detect Star Wars units
7. **Bundling**: Aggregate all results into `validate_extended_report.json`, copy artifacts to `docs/proof-of-features/extended_<timestamp>/`

### Output
- **`validate_extended_report.json`** — Structured results: stat values, pack loaded flags, VLM confirmations, timestamps
- **5 screenshots** — One per feature (PNG, GDI-captured, live game window)
- **`EXTENDED_EVAL_REPORT.md`** — Human-readable summary with feature validation table

## Error Handling

- **If game not running**: STOP and ask user to `/launch-game` or `/prove-features`
- **If MCP server unreachable**: STOP and provide instructions to start MCP server
- **If individual feature fails**: Log failure, but CONTINUE to next feature (do not stop pipeline)
- **Overall result**: If >= 3/5 features confirmed, mark as "PASS"; if < 3, mark as "PARTIAL PASS"

## VLM Model Selection

For `game_analyze_screen` calls:
1. **Claude Opus** (if available and budget allows)
2. **Claude Sonnet 4** (preferred balance of capability and cost)
3. **Claude Haiku** (fallback, fast mode OK)

Each VLM call receives:
- Live GDI screenshot (no compression, captured while game is running)
- Custom prompt asking about specific visual feature (HP=999, resource HUD, clone trooper, etc.)
- Returns text analysis (what it sees in the screenshot)

## Integration with Existing Commands

- **`/prove-features`** — Validates core 3 features (video-based, Remotion renders)
- **`/eval-game-features`** — Validates extended 5 features (MCP-based, screenshot-based)
- **`/eval-all`** — Could orchestrate both `/prove-features` and `/eval-game-features` in sequence

## Future Extensions

Potential additions to the extended pipeline:
- **A-7**: Mod compatibility check (pack conflict detection via registry)
- **A-8**: Performance profiling (entity count, frame time via game telemetry)
- **A-9**: Save/load mechanics (game state persistence)
- **A-10**: Network multiplayer (if applicable)

## References

- Command file: `.claude/commands/eval-game-features.md`
- Helper script: `scripts/game/eval-game-features.ps1`
- MCP tool catalog: `CLAUDE.md` (Game Automation section)
- Existing pipeline: `.claude/commands/prove-features.md`
