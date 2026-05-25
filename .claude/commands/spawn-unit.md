# spawn-unit

Test unit spawning by requesting a pack unit spawn in-game.

**Usage**: `/spawn-unit <pack-id:unit-id> [x] [z] [--enemy]`

**Arguments**: $ARGUMENTS

## Purpose

This command helps test that the unit spawner is working. It:

1. Reads the pack definition to verify the unit exists
2. Checks the unit class maps to a vanilla archetype via VanillaArchetypeMapper
3. Explains what will happen at runtime (what entity will be cloned, what stats applied)
4. Outputs the equivalent C# call: `PackUnitSpawner.RequestSpawnStatic("pack-id:unit-id", x, z, isEnemy)`
5. Reminds to check `BepInEx/dinoforge_debug.log` after launching the game

This is useful for validating that pack unit definitions are properly recognized by the runtime.
