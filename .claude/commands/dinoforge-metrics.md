# dinoforge-metrics

Pull current runtime metrics from the running game via the MCP bridge.

**Usage**: `/dinoforge-metrics [--category <category>] [--format <json|table>]`

**Arguments**:
- `--category`: Filter metrics (memory, gc, entities, systems, packs, all). Default: all
- `--format`: Output format (json, table). Default: table

## Purpose

Displays telemetry from the running DINOForge Runtime: memory usage, GC stats, entity counts, system counts, pack load counts, frame time, etc.

## Steps

1. **Check game is running**:
   - If no game process found, exit with error

2. **Connect to RPC bridge** via MCP `game_get_stat`:
   ```
   GET /game/metrics
   ```

3. **Parse and display**:
   - **Memory**: Heap size, Mono reserved, managed GC pressure
   - **GC**: Last collection duration, collection frequency
   - **Entities**: Total entity count by archetype, prefab vs live
   - **Systems**: Count of running systems, grouped by system group
   - **Packs**: Count of loaded packs, count of registered registries
   - **Frame time**: Last frame ms, FPS average (last 60 frames)

4. **Format output**:
   - Table: aligned columns with category grouping
   - JSON: hierarchical structure suitable for scripting

## Use When

- Profiling DINOForge overhead (memory footprint, GC pressure)
- Verifying packs are loaded (pack count should match expected)
- Debugging performance: is entity count growing unbounded?
- Understanding system load: how many systems are active?
- Before/after comparisons to validate an optimization

## Example Output (table)

```
Memory (MB)
  Heap Size:        512
  Mono Reserved:    1024
  Managed GC Heap:  245

GC Stats
  Last Collection:  12ms
  Collections/Sec:  0.8

Entities
  Total:            45776
  Prefab:           890
  Live:             44886
  
Systems
  Active:           23
  In Initialization: 0
  In Simulation:    18

Packs
  Loaded:           4
  Registries:       12

Frame Time
  Last Frame:       5.2ms
  FPS (avg):        143
```

## Time

~2 seconds (MCP call + parsing).
