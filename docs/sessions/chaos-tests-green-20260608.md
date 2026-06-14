# Chaos Tests Green - 2026-06-08

Scope:

- Fixed the unrelated Chaos failures around asset bundle swap fallback handling and pack load warning logging.
- Kept changes scoped to `src/SDK/Assets/AssetService.cs`, `src/SDK/ContentLoader.cs`, `src/SDK/ContentRegistrationService.cs`, and `src/Runtime/Bridge/AssetSwapSystem.cs`.

Changes:

- `AssetService` now opens bundle files inside `try/finally` scopes in the read paths that were leaking handles on corrupt bundles.
- `AssetSwapSystem.ApplySwap` now catches bundle extraction failures, logs a warning, and continues with the entity-swap fallback path.
- `ContentLoader` now logs warning-level fallbacks when discovery or content loading faults occur, and it guards pack discovery / patch-phase file collection with exception handling.
- `ContentRegistrationService` now emits warning logs when file reads or deserialize/register steps fail.

Validation:

- `dotnet build src/SDK/DINOForge.SDK.csproj -c Release` succeeded.
- `dotnet build src/Runtime/DINOForge.Runtime.csproj -c Release` succeeded.
- `dotnet test .\\src\\Tests\\Chaos\\bin\\Release\\net8.0\\DINOForge.Tests.Chaos.dll --no-build` succeeded: `Passed: 5, Failed: 0, Skipped: 0`.

Notes:

- The requested project path `src/Tests/Chaos/DINOForge.Tests.Chaos.csproj` was not resolvable in this workspace at validation time, so the compiled Chaos test assembly under `src/Tests/Chaos/bin/Release/net8.0/` was used for the final test run.
