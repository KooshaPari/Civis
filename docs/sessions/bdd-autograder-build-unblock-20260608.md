# BDD/Autograder Build Unblock - 2026-06-08

## Goal

Unblock the `src/Tests/DINOForge.Tests.csproj` Release build after coverage tooling started surfacing generated-file compile errors from the nested BDD and Autograder test outputs.

## Root Cause

The parent test project was compiling source files from nested `obj` directories under `src/Tests/BDD` and `src/Tests/Autograder`.

That caused the Release build to pick up generated files such as:

- `src\Tests\BDD\obj\Release\net8.0\xUnit.AssemblyHooks.DINOForge_Tests_BDD.cs`
- `src\Tests\Autograder\obj\Release\net11.0\.NETCoreApp,Version=v11.0.AssemblyAttributes.cs`

Those generated files produced duplicate xUnit assembly types/attributes and duplicate `TargetFrameworkAttribute` errors.

There was also a separate C# access-modifier regression in `src/Runtime`: a set of ECS `OnCreate` / `OnUpdate` / `OnDestroy` overrides were declared `public override` even though the base members are `protected`, which broke the runtime project compile under Release.

## Fix

1. In `src/Tests/DINOForge.Tests.csproj`, excluded generated outputs from the nested test sub-projects:
   - `BDD\obj\**`
   - `Autograder\obj\**`

2. In `src/Runtime`, changed the ECS lifecycle overrides from `public override` to `protected override` in the affected systems:
   - `Aviation/AerialMovementSystem.cs`
   - `Aviation/AerialSpawnSystem.cs`
   - `Aviation/AerialTargetingSystem.cs`
   - `Bridge/AssetSwapSystem.cs`
   - `Bridge/BuildMenuInjectionSystem.cs`
   - `Bridge/BuildingDestructionVFXSystem.cs`
   - `Bridge/FactionSystem.cs`
   - `Bridge/KeyInputSystem.cs`
   - `Bridge/PackUnitSpawner.cs`
   - `Bridge/ProjectileMeshSwapSystem.cs`
   - `Bridge/ProjectileVFXSystem.cs`
   - `Bridge/StatModifierSystem.cs`
   - `Bridge/UnitDeathVFXSystem.cs`
   - `Bridge/WaveInjector.cs`
   - `DumpSystem.cs`

## Validation

Ran:

`dotnet test src/Tests/DINOForge.Tests.csproj -c Release --no-restore -v minimal`

Result:

- The generated-file compile errors from `BDD\obj` and `Autograder\obj` were gone.
- The test run completed successfully through build and test execution.
- Remaining failures were unrelated snapshot mismatches:
  - `DINOForge.Tests.Snapshots.PackDisplayInfo_Snapshot.StarWarsPack_DisplayInfo_MatchesSnapshot`
  - `DINOForge.Tests.Snapshots.PackManifest_Schema_Snapshot.PackManifestSchema_MatchesSnapshot`

## Notes

The fix was kept narrow to avoid editing the generated files themselves. The parent test project now ignores the nested generated outputs that caused the duplicate assembly/type definitions during coverage-oriented Release builds.
