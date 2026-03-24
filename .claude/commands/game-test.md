# game-test

Run automated game tests via the DINOForge MCP bridge. Tests entity queries, stat overrides, pack loading, and visual swap results.

**Usage**: `/game-test [--suite <all|swaps|units|stats|packs>] [--screenshot]`

**Arguments**: $ARGUMENTS

## Purpose

Uses the MCP bridge tools to verify mod behavior in a running game session. Requires the game to be running with the DINOForge Runtime plugin loaded.

## Test Suites

### `packs` — Pack loading verification
- Call `game_status` → verify `packsLoaded > 0`
- Check `warfare-starwars` pack is in loaded packs list
- Report any packs that failed to load

### `units` — Entity query verification
- Call `game_query_entities` with `Components.Unit` type
- Verify entity count > 0 (confirms `IncludePrefab` works)
- Call `game_query_entities` with `Components.MeleeUnit`
- Report entity counts per type

### `stats` — Stat modifier verification
- Call `game_get_stat` on a unit entity for `health`
- Call `game_apply_override` with a test stat change (+10% health)
- Call `game_get_stat` again and verify the change was applied
- Revert the override

### `swaps` — Asset swap verification
- Read `BepInEx/dinoforge_debug.log` tail (last 100 lines)
- Count `swap complete` entries
- Count `live swap pending` entries
- Report `TrySwapRenderMeshFromBundle: swapped N/M` results
- Fail if 0 swaps complete after 600+ frames

### `all` (default) — Run all suites

## Steps

1. Check MCP bridge connection: if not connected, report "game not running — use /launch-game first"
2. Run selected test suites sequentially
3. If `--screenshot`, call `game_screenshot` after tests and include image path in report
4. Output a test report table: Suite | Pass/Fail | Details
5. Exit code: 0 if all pass, 1 if any fail

## Pass/Fail Criteria

| Test | Pass |
|------|------|
| Pack loading | ≥1 pack loaded |
| Entity query | entity count > 0 |
| Stat override | value changes after apply |
| Asset swaps | ≥1 `swap complete` in log |
