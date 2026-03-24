# Check Game State

Check the DINOForge asset swap status by reading the debug log from the running game.

## Steps

1. Read the last 200 lines of the debug log at:
   `G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option\BepInEx\dinoforge_debug.log`

2. Look for:
   - `AssetSwapSystem: probe query created` — confirms IncludePrefab fix is working; note entity count
   - `AssetSwapSystem: processing N pending swap(s)` — confirms swaps are being attempted
   - `ApplySwap: entity swap result=True` — confirms Phase 2 (live entity swap) succeeded
   - `ApplySwap: Phase 1 skipped` — normal; catalog doesn't have unit addresses
   - `TrySwapRenderMeshFromBundle: swapped N/M entities` — shows actual swap count

3. Also read `G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option\BepInEx\LogOutput.log`
   - Look for errors or warnings from DINOForge Runtime
   - Check what scene is loaded

4. Report:
   - Whether swaps are succeeding (entity count > 0)
   - Any unexpected errors
   - What scene/state the game is in
   - The most recent batch result (`N succeeded, M failed`)
