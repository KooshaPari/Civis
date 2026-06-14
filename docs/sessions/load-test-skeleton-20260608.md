# Load Test Skeleton - 2026-06-08

## Scope

Added the smallest Bridge load-test skeleton under `src/Tests/Load`:

- New test project: `src/Tests/Load/DINOForge.Tests.Load.csproj`
- New skeleton test: `src/Tests/Load/BridgeLoadSkeletonTests.cs`
- Solution wiring: `src/DINOForge.sln`

The test is intentionally narrow:

- It uses a local named-pipe responder instead of the live game.
- It opens one `GameClient` and issues parallel `PingAsync()` calls in a small loop.
- It asserts all calls complete and that the mocked ping response is returned.

## Validation

Targeted build:

```powershell
dotnet build src/Tests/Load/DINOForge.Tests.Load.csproj
```

Result: succeeded with 0 warnings and 0 errors on the final pass.

## Notes

- This is a load-test skeleton only, not a perf benchmark.
- I did not duplicate the tick14 perf baseline doc.
