# Test Frameworks Status - 2026-06-10

Verified against `docs/proof/autograder-scorecard.json` with one build only.

| framework | files-exist | builds | notes |
|---|---|---|---|
| BDD | yes | not individually built | `src/Tests/BDD/DINOForge.Tests.BDD.csproj` exists, with feature files under `src/Tests/BDD/Features/*.feature` and generated step files beside them. |
| Chaos | yes | not individually built | `src/Tests/Chaos/DINOForge.Tests.Chaos.csproj` exists, with test files under `src/Tests/Chaos/*.cs`. |
| Perf | yes | not individually built | `src/Tests/Perf/DINOForge.Tests.Perf.csproj` exists, with benchmark code under `src/Tests/Perf/*.cs`. |
| Load | yes | yes | `src/Tests/Load/DINOForge.Tests.Load.csproj` exists, with `src/Tests/Load/BridgeLoadSkeletonTests.cs`; this was the single build run and it succeeded. |

Build result:

- `dotnet build src/Tests/Load/DINOForge.Tests.Load.csproj`
- Result: succeeded with warnings, 0 errors
- Scope: one build only, as requested
