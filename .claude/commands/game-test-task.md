---
description: Run an automated game test task using TITAN-inspired coverage-driven agent loop
---

# game-test-task

Runs an automated game test using a coverage-driven agent loop inspired by the TITAN paper.

**Usage**: `/game-test-task <task_description>`

**Arguments**: $ARGUMENTS

## What This Does

Implements the TITAN testing framework (state abstraction, coverage memory, stuck detection, reflection) on top of DINOForge MCP tools.

1. **Load coverage memory** from `docs/sessions/coverage_memory.json`
2. **Take initial screenshot** via `game_screenshot`
3. **Analyze screen state** via `game_analyze_screen` to identify UI elements
4. **Abstract state** using DINO state config from `docs/sessions/dino_state_abstraction.yaml`
5. **Main loop** (max 50 actions):
   - Check coverage memory — avoid recently-failed (state, action) pairs
   - Determine next action based on task description + current state
   - Execute action via `game_input` or `game_navigate_to`
   - Take screenshot and compare to previous (via `game_wait_and_screenshot`)
   - If state unchanged for 5 consecutive actions → trigger reflection
   - Update coverage memory with (state, action, outcome)
   - Check if task is complete → exit loop if so
6. **Save final coverage memory** back to `docs/sessions/coverage_memory.json`
7. **Report results**: task outcome, actions taken, any bugs/issues discovered, coverage achieved

## Stuck Detection + Reflection

When stuck (5 actions without state change):

1. Capture current screenshot via `game_screenshot`
2. Analyze with `game_analyze_screen` to detect UI state
3. Issue structured reflection prompt
4. Execute the chosen alternative action
5. Continue loop

## Coverage Memory Format

**File**: `docs/sessions/coverage_memory.json`

Fields in coverage entries:
- `state_hash` — Abstract state token (concatenated from dino_state_abstraction.yaml)
- `action` — Action name (from action_bundles in dino_state_abstraction.yaml)
- `outcome` — One of: `success`, `failed`, `unknown`
- `times_tried` — How many times this pair has been attempted
- `notes` — Optional notes about the outcome

## DINO State Abstraction

Reads `docs/sessions/dino_state_abstraction.yaml` which maps raw game state to symbolic tokens.

Default state dimensions if file missing:
- wave_active: entity_count threshold
- menu_state: screenshot UI detection
- resource_level: resource entity aggregate
- health_state: min entity health ratio

## Action Bundles

Actions available depend on current menu_state. Check `dino_state_abstraction.yaml` for the complete list.

**Main Menu**: press_enter, press_escape
**Gameplay**: press_f10, press_f9, press_escape, screenshot
**Pause Menu**: press_escape, press_enter

## Implementation Notes for Claude

When implementing this command:

1. **Start simple**: Cover main_menu → gameplay → pause_menu flow first
2. **Use game_analyze_screen heavily**: It's your primary sensor for state
3. **Log every (state, action, outcome) triple**: This is the coverage memory
4. **Reflection is key**: When stuck, always ask the LLM why before continuing
5. **Save coverage memory after every action**: Enables session continuity
6. **Never retry the same (state, action) pair twice without reflection**
7. **Prefer game_wait_and_screenshot** over raw game_screenshot when polling for change

## See Also

- `/game-coverage` — View coverage memory statistics
- `/game-test` — Run existing test suites (packs, units, stats, swaps)
- `/launch-game` — Launch game instance
- `/check-game` — Check game status and debug log
