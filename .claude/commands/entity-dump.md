# entity-dump

Dump and analyze ECS entities from the running game. Shows archetypes, component counts, and unit/building breakdowns.

**Usage**: `/entity-dump [--filter <type>] [--live] [--from-file <path>]`

**Arguments**: $ARGUMENTS

## Purpose

Reads the entity dump output from the game (written by DumpSystem) or queries live via MCP bridge. Used for debugging asset swap targets, verifying unit archetype mappings, and understanding the ECS world state.

## Steps

1. **Find dump file**:
   - Look in `G:\SteamLibrary\...\BepInEx\` for `dinoforge_entity_dump_*.json` files
   - Use the most recent one if multiple exist
   - If `--from-file` is provided, use that path instead

2. **If `--live`** and MCP bridge is connected:
   - Call `game_dump_state` to trigger a fresh dump
   - Wait 2s for file to be written
   - Read the new dump file

3. **Parse and analyze**:
   - Count total entities
   - Group by archetype (by component set)
   - Find all `Components.Unit` entities → report count
   - Find all `Components.BuildingBase` entities → report count
   - Find all `Unity.Rendering.RenderMesh` entities → report count
   - If `--filter <type>` provided, show only entities with that component

4. **Report**:
   ```
   Total entities: N
   Units (Components.Unit): N
     └─ MeleeUnit: N
     └─ RangeUnit: N
     └─ CavalryUnit: N
   Buildings (Components.BuildingBase): N
   RenderMesh entities: N  ← target for asset swaps
   ```

5. **Cross-reference with asset swaps**:
   - List which swap addresses have a `vanilla_mapping` that resolves to a found archetype
   - Flag any swap addresses where the target entity type has 0 entities

## Notes

- Entity dump is written by `DumpSystem` after a 300-frame delay (configurable)
- RenderMesh entity count must be > 0 for asset swaps to work
- Use this to verify `IncludePrefab` is working (pre-fix: RenderMesh count = 0)
