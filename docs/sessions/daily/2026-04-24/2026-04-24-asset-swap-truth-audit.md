# AssetSwapSystem Truth Audit — 2026-04-24

## Verdict: REAL implementation, BROKEN INPUT DATA, FAKE TESTS

The AssetSwapSystem is working code that has never been observed working in production because the input data it consumes is mismatched and the unit-test surface that "validates" it uses substitute `FakeAssetBundle` / `FakeSwappableEntity` types instead of real bundles.

This is the exact failure pattern the user is calling out: features have implementations, the implementations even pass tests, and yet at runtime they fail 100% of the time because the verification harness never exercised the real input.

## Findings

### Core swap logic — REAL
- `AssetSwapSystem.ApplySwap` (lines 165–224): real — calls `assetService.ExtractAsset()`, `assetService.ReplaceAsset()`, writes patched bundles to `BepInEx/dinoforge_patched_bundles/`.
- `AssetSwapSystem.TrySwapRenderMeshFromBundle` (lines 233–418): real — loads bundles, extracts `Mesh` / `Material` from prefab, mutates ECS entity `RenderMesh` shared component via reflection (`meshField.SetValue()` at line 374, `materialField.SetValue()` at line 387).

### AssetSwapRegistry population — REAL but conditional
- `ContentLoader.RegisterAssetSwaps` (lines 295–334): iterates `RegistryManager.Units.All`, registers entries with `visual_asset` field.
- Gates on `File.Exists(bundlePath)` (line 309) — registry is empty when bundles aren't built. (Star Wars pack ships 18 real bundles + 12 stub bundles per prior audit; the 18 do pass this gate.)

### AddressablesCatalog parsing — REAL with heuristic fallback
- `Load()` (lines 46–111): reads `catalog.json`, parses `m_InternalIds`, decodes Base64 entry data.
- `ParseEntryData()` (lines 119–171): real binary parse of the 28-byte Addressables entry format.
- Fallback (lines 95–107): if parse fails, assigns all assets to first bundle — heuristic, not guaranteed correct.
- `ResolveBundlePath()` (181–198): real placeholder substitution.

### Smoking gun: bundle asset names do not match `visual_asset`

`AssetSwapTests.cs` lines 28–29 (verbatim, in source):

> Root cause of the current 0/36 swap failure: bundle names don't match the visual_asset YAML field.

`AssetSwapTests.cs` line 99 (verbatim):

> Currently 0/36 succeed (all 36 swaps failing)

### Execution trace at runtime
1. ✅ `ContentLoader.RegisterAssetSwaps` runs, registers 36 entries in `AssetSwapRegistry`.
2. ✅ `AssetSwapSystem.OnUpdate` waits 600 frames, calls `ApplySwap`.
3. ✅ `ApplySwap` calls `LoadBundle(modBundlePath)` — succeeds.
4. ✅ Calls `bundle.LoadAsset<Mesh>("sw-clone-trooper")` (the `visual_asset` key).
5. ❌ Returns null — the bundle contains a mesh named `CloneTrooperMesh` (or whatever the FBX asset name was), not `sw-clone-trooper`.
6. ❌ Falls back to `bundle.LoadAsset<GameObject>` — also returns null.
7. ❌ Logs "no Mesh/Material named X in bundle" and returns false.
8. ❌ `MarkFailed`, fail count incremented.
9. ❌ After 3 retries, swap permanently skipped.

**Result: 0 of 36 Star Wars unit visuals have ever rendered in the live game.**

### Tests that "pass" — fake by design

`src/Tests/Integration/Tests/AssetSwapTests.cs` and adjacent fixtures use:
- `FakeAssetBundle` — substitute that returns the requested asset by name regardless of bundle contents.
- `FakeAssetSwapSystem` — substitute that records swap calls without actually loading bundles.
- `FakeSwappableEntity` — substitute ECS entity that doesn't require Unity ECS world.

These tests pass at green every CI run. They confirm the *control flow* of swap calls. They do not exercise the *data path* (bundle name vs `visual_asset` key match) where the real failure lives.

`TRACEABILITY_VERIFICATION_20260420.md` cites tests like `AssetSwapTests.EntitySwap_ReplacesVanillaWithCustom` as evidence of asset-swap "verified." Those tests exclusively use the Fake* types.

## Honest fix

Two layers, both required:

1. **Data layer**: either rename each `visual_asset` key in pack YAML to match the real asset name inside the corresponding bundle (manual sync), OR change `TrySwapRenderMeshFromBundle` to use `bundle.LoadAllAssets()` and find by type rather than by name (already a fallback at lines around 280, but not the first try). The right fix is probably "iterate `LoadAllAssets`, pick first `Mesh`/`Material`" rather than name-keyed lookup, since bundle names are an Addressables internal detail Pack authors shouldn't need to know.

2. **Test layer**: replace `FakeAssetBundle` etc. with a real-bundle integration test using one of the 18 real Star Wars bundles. If the test can't run on Linux CI (Unity bundle loading needs Windows + DirectX adapters), gate it behind a `WINDOWS_GAME_AVAILABLE` env var and explicitly fail CI when that variable isn't set on the machine running asset-swap-related changes — rather than silently skipping.

3. **Telemetry**: when `AssetSwapSystem` fails a swap, the failure should surface in `dinoforge_debug.log` AND in the `game_status` MCP tool's response, so an agent doing `prove-features` can detect "we shipped 18 bundles and 0 of them ever rendered" without having to read source code.

## TRUTH_TABLE.md entry

| AssetSwapSystem (in-game render of mod bundles) | ❌ STUB (functional code, broken data path) | Implementation real (loads bundles, mutates ECS entities). 0 of 36 Star Wars unit swaps render at runtime because `visual_asset` YAML key doesn't match the asset name inside the bundle. Tests pass because they use `FakeAssetBundle`. Fix: change lookup from name-keyed to type-keyed `LoadAllAssets()` first-match, and replace fake tests with real-bundle integration tests. |

## Replay command (pending implementation)

Once fixed, the proof should be: launch the game with warfare-starwars enabled, screenshot a clone trooper, run `game_analyze_screen --external-judge --golden vanilla_militia` and confirm Kimi judge says the rendered unit does NOT match the vanilla militia (i.e. swap took effect). Save the receipt to `docs/proof/judge-receipts/`.
