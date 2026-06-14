# Mutation Smoke Run - 2026-06-08

## Scope

- Retried mutation smoke after `stryker-config.json` existed.
- Narrowed the target to a tiny bridge-protocol smoke slice:
  - project under test: `src/Bridge/Protocol/DINOForge.Bridge.Protocol.csproj`
  - smoke test project: `src/Tests/BridgeSmoke/DINOForge.Tests.BridgeSmoke.csproj`
  - mutate target: `src/Bridge/Protocol/CanonicalJson.cs`

## Commands

```powershell
dotnet tool install -g dotnet-stryker
dotnet test src\Tests\BridgeSmoke\DINOForge.Tests.BridgeSmoke.csproj --verbosity minimal
dotnet test src\Tests\BridgeSmoke\DINOForge.Tests.BridgeSmoke.csproj --no-restore --verbosity minimal
dotnet stryker --config-file stryker-smoke-config.json
```

## Mutation Score

- Not produced.
- Stryker aborted before running any mutants because test discovery failed.

## Blockers

- `dotnet-stryker` 4.14.2 failed to discover tests from the smoke project even though `dotnet test` passed locally.
- The same discovery failure happened with multiple harness attempts:
  - analyzer xUnit project
  - CLI xUnit project
  - bridge smoke project with xUnit
  - bridge smoke project with MSTest
- Final Stryker error was:

```text
No test result reported. Make sure your test project contains test and is compatible with VsTest.
Project 'C:\Users\koosh\Dino\src\Tests\BridgeSmoke\DINOForge.Tests.BridgeSmoke.csproj' did not report any test.
```

## Notes

- The smoke test project itself is healthy:
  - `dotnet test src\Tests\BridgeSmoke\DINOForge.Tests.BridgeSmoke.csproj --verbosity minimal` passed
  - total: 6 tests passed, 0 failed
- The main repo still has unrelated modified files in the working tree; this smoke run did not touch them.
