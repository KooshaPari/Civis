# Mutation Bridge Run - 2026-06-08

## Scope

- Targeted the Bridge Protocol surface as the tightest practical Bridge mutation target:
  - project under test: `src/Bridge/Protocol/DINOForge.Bridge.Protocol.csproj`
  - test project: `src/Tests/BridgeSmoke/DINOForge.Tests.BridgeSmoke.csproj`
  - mutation target: `src/Bridge/Protocol/CanonicalJson.cs`
- Reused the repo root `stryker-config.json` values, but had to translate them into a temporary config because the checked-in file uses legacy keys that Stryker 4.14.2 rejects.

## Commands

```powershell
# Validation that the Bridge smoke build itself is healthy when shared compilation is disabled
dotnet build src\Tests\BridgeSmoke\DINOForge.Tests.BridgeSmoke.csproj -c Debug --property:Platform=AnyCPU -p:UseSharedCompilation=false

# Stryker run 1: Bridge Protocol file target, vstest runner
dotnet stryker --config-file $env:TEMP\stryker-bridge-20260608.json --mutate "src/Bridge/Protocol/CanonicalJson.cs" --output mutation-report-bridge-canonical --skip-version-check

# Stryker run 2: same target, Microsoft Test Platform runner
dotnet stryker --config-file $env:TEMP\stryker-bridge-20260608.json --mutate "src/Bridge/Protocol/CanonicalJson.cs" --output mutation-report-bridge-canonical --skip-version-check --test-runner mtp
```

## Timing

- `dotnet build` sanity check: `00:00:05.23`
- Stryker vstest attempt: `00:00:16.45`
- Stryker mtp attempt: `00:00:14.27`
- Bridge-wide Stryker attempt was also started and then terminated after `00:19:58.51` because the build phase was not progressing usefully.

## Mutation Score

- Not produced.
- Stryker never reached mutant execution because test discovery failed for the Bridge smoke project.

## Mutants Not Killed

- `0` mutants executed.
- `0` surviving mutants reported.
- `0` killed mutants reported by Stryker.

## Blockers

- `dotnet-stryker` 4.14.2 cannot discover tests from `src/Tests/BridgeSmoke/DINOForge.Tests.BridgeSmoke.csproj` in this workspace, even though plain `dotnet test` and `dotnet test --list-tests` both work.
- The Stryker error path reports that the test project did not report any tests, so the run aborts before mutation starts.
- The Bridge smoke build itself is not the blocker. It succeeds quickly when `UseSharedCompilation=false` is applied; that setting was only needed to avoid a long build stall during Stryker's build phase.

## Notes

- The Bridge smoke test project contains 6 MSTest cases in `CanonicalJsonSmokeTests.cs`.
- The temporary build workaround was not kept in the repo after verification.
- The existing `stryker-config.json` is still stale for the installed Stryker version and needs cleanup if this mutation workflow is going to be automated.
