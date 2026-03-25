---
description: Show current game test coverage memory statistics
---

# game-coverage

Shows statistics from the TITAN coverage memory for game testing.

**Usage**: `/game-coverage [--reset]`

## What This Does

Reads `docs/sessions/coverage_memory.json` and computes:

- **Total entries**: How many (state, action) pairs have been explored
- **Success rate**: Percentage of pairs that succeeded vs failed
- **Most-visited states**: Top 5 states with most actions tried
- **Most-successful actions**: Actions with highest success rate
- **Recently stuck states**: States where we hit the 5-action stuck threshold
- **Coverage gaps**: Unexplored high-value state-action combinations
- **Session timeline**: When each major state was first discovered

## Output Format

```
═════════════════════════════════════════════════════════════════
                    GAME COVERAGE MEMORY
═════════════════════════════════════════════════════════════════

Total Entries: 42 (state, action) pairs
Last Updated: 2026-03-25T14:30:00Z
Session Duration: 127 actions across 3 sessions

SUCCESS RATE: 78% (33 succeeded, 9 failed)

TOP STATES BY EXPLORATION:
  1. gameplay_wave_active_health_healthy    — 12 actions, 100% success
  2. main_menu                               — 8 actions, 75% success
  3. pause_menu                              — 6 actions, 83% success
  4. gameplay_wave_active_health_damaged     — 5 actions, 60% success
  5. gameplay_wave_inactive_health_healthy   — 4 actions, 100% success

MOST SUCCESSFUL ACTIONS:
  1. press_f10 (mod menu)         — 8 attempts, 100% success
  2. screenshot                   — 12 attempts, 100% success
  3. press_escape (pause)         — 7 attempts, 86% success

RECENTLY STUCK (last 5 sessions):
  - State: pause_menu, at 2026-03-25T12:15:00Z
    Actions tried: press_enter, press_escape, press_enter, press_escape, press_enter
    Resolution: Cleared on 6th action (press_f10 to open menu)

COVERAGE GAPS (unexplored combinations):
  Suggested high-value pairs to try:
  1. gameplay_wave_inactive + press_f9 (debug toggle)
  2. pause_menu + press_f10 (mod menu from pause)
  3. health_critical + screenshot (observe critical state)

═════════════════════════════════════════════════════════════════
```

## Options

`--reset`: Clear the entire coverage memory and start fresh.
Use with caution — this removes all historical test data.

## See Also

- `/game-test-task <task>` — Run an automated game test
- `/game-test` — Run existing test suites (packs, units, stats, swaps)
