---
description: Run an automated game test task using TITAN-inspired coverage-driven agent loop
---

# game-test-task

Runs an automated game test using a coverage-driven agent loop inspired by the TITAN paper.

**Usage**: `/game-test-task [--scenario SCENARIO] [--task DESCRIPTION] [--max-actions N]`

## What This Does

Implements the TITAN testing framework (state abstraction, coverage memory, stuck detection, reflection) on top of DINOForge MCP tools.

1. **Load coverage memory** from `docs/sessions/coverage_memory.json`
2. **Take initial screenshot** via game automation
3. **Analyze screen state** to identify UI elements
4. **Abstract state** using DINO state config from `docs/sessions/dino_state_abstraction.yaml`
5. **Main loop** (max 50 actions):
   - Check coverage memory — avoid recently-failed (state, action) pairs
   - Determine next action based on task description + current state
   - Execute action via game input
   - Take screenshot and compare to previous
   - If state unchanged for 5+ consecutive actions → trigger reflection
   - Update coverage memory with (state, action, outcome)
   - Check if task is complete → exit loop if so
6. **Save final coverage memory** back to `docs/sessions/coverage_memory.json`
7. **Report results**: task outcome, actions taken, coverage achieved

## Built-in Test Scenarios

Run a predefined scenario with:

```bash
/game-test-task --scenario smoke
/game-test-task --scenario unit_spawn
/game-test-task --scenario modern_warfare
/game-test-task --scenario starwars
/game-test-task --scenario debug_overlay
/game-test-task --scenario pause_menu
/game-test-task --scenario stress
```

### Scenario Descriptions

- **smoke**: Basic game launch + screenshot + mod menu toggle
- **unit_spawn**: Spawn units and verify visual asset swaps
- **modern_warfare**: Modern warfare pack units test
- **starwars**: Star Wars Clone Wars pack units test
- **debug_overlay**: Toggle debug overlay and verify entity counts
- **pause_menu**: Navigate pause menu with arrow keys
- **stress**: Extended gameplay stress test (15 screenshots)

## Custom Task

Run a custom task with:

```bash
/game-test-task --task "Verify game launches and mod menu opens" --max-actions 50
```

## Stuck Detection + Reflection

When stuck (5 actions without state change):

1. Capture current screenshot
2. Analyze UI state
3. Issue structured reflection prompt
4. Execute the chosen alternative action
5. Continue loop

## Coverage Memory Format

**File**: `docs/sessions/coverage_memory.json`

Fields in coverage entries:
- `state_hash` — Abstract state token (e.g., `menu=gameplay_entities=medium_health=healthy`)
- `action` — Action name (from action_bundles in dino_state_abstraction.yaml)
- `outcome` — One of: `success`, `failed`, `unknown`
- `times_tried` — How many times this pair has been attempted
- `first_tried`, `last_tried` — ISO timestamps
- `notes` — Optional notes about the outcome

## DINO State Abstraction

Reads `docs/sessions/dino_state_abstraction.yaml` which maps raw game state to symbolic tokens.

State dimensions:
- `menu_state`: main_menu, gameplay, pause_menu, loading
- `entity_count`: low (< 50), medium (50-500), high (> 500)
- `health`: critical, damaged, healthy (entity health ratio)

## Action Bundles

Actions available depend on current menu_state:

| Menu State | Available Actions |
|------------|-------------------|
| main_menu | press_enter, press_escape |
| gameplay | press_f10, press_f9, press_escape, arrow_up/down/left/right |
| pause_menu | press_escape, press_enter, arrow_up/down |
| loading | press_escape |

## Implementation Files

- **GameTestRunner**: `scripts/game_test_runner.py` — Main test executor
- **Scenarios**: `scripts/game_test_scenarios.py` — Predefined test scenarios
- **State abstraction**: `docs/sessions/dino_state_abstraction.yaml` — State config
- **Coverage memory**: `docs/sessions/coverage_memory.json` — Persistent coverage data
- **Results**: `docs/test-results/` — Test results (JSON)

## See Also

- `/game-coverage` — View coverage memory statistics
- `/game-test` — Run existing test suites (packs, units, stats, swaps)
- `/launch-game` — Launch game instance
- `/check-game` — Check game status and debug log
