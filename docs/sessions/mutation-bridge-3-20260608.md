# Mutation Bridge 3 - 2026-06-08

## Scope

- Re-ran Bridge-scoped mutation testing against `src/Bridge/Protocol/DINOForge.Bridge.Protocol.csproj`.
- Added a dedicated protocol-only mutation harness so Stryker would not need to build the unrelated Runtime subtree:
  - `src/Tests/BridgeMutation/DINOForge.Tests.BridgeMutation.csproj`
  - `src/Tests/BridgeMutation/CanonicalJsonMutationTests.cs`
- Kept the existing Bridge test coverage intent intact with targeted cases for:
  - canonical object ordering
  - nested container ordering
  - string scalar roots
  - floating-point and decimal scalar formatting
  - negative zero normalization
  - non-finite rejection
  - BigInteger canonicalization
  - unsafe nested numeric rejection

## Files Changed

- `src/Tests/BridgeMutation/DINOForge.Tests.BridgeMutation.csproj`
  - Added a new xUnit test project that references `DINOForge.Bridge.Protocol` directly.
- `src/Tests/BridgeMutation/CanonicalJsonMutationTests.cs`
  - Added 10 focused CanonicalJson tests aimed at high-cost mutation survivors.
- `src/DINOForge.sln`
  - Added the new BridgeMutation test project to the solution.
- `src/Tests/DINOForge.Tests.csproj`
  - Added a direct project reference to `DINOForge.Bridge.Protocol` while investigating the mutation-host resolution issue.

## Validation

Targeted harness validation:

```powershell
dotnet test src/Tests/BridgeMutation/DINOForge.Tests.BridgeMutation.csproj --configuration Release --verbosity minimal
```

Result:

- Passed: 10
- Failed: 0
- Skipped: 0

## Stryker Attempts

Command family used:

```powershell
dotnet stryker --config-file $env:TEMP\stryker-bridge-mutation-*.json --output $env:TEMP\stryker-bridge-mutation-*-out --skip-version-check [--diag]
```

Test runners tried:

- `vstest`
- `mtp`

## Mutation Score

- Not produced.
- Stryker never reached mutation execution because test discovery failed in both runner modes.

## Score Delta

- Unavailable.
- There is no baseline-vs-current Bridge mutation score to compare because this run did not emit a score.

## Un-Killed Summary

- Mutants executed: `0`
- Killed: `0`
- Survived: `0`
- Timed out: `0`

## Blockers

- `vstest` failed inside Stryker's test host with a .NET runtime load error while discovering `DINOForge.Tests.BridgeMutation.dll`:
  - `System.Runtime, Version=8.0.0.0` could not be loaded in the Stryker-hosted `vstest.console.exe`.
- `mtp` also failed to start the test server for the same test assembly.
- Plain `dotnet test` on the same harness passed, so the blocker is specific to Stryker's runner integration in this workspace.

## Notes

- The dedicated BridgeMutation harness is intentionally protocol-only so the mutation target stays isolated from the unrelated Runtime build failures already present in the tree.
- The added tests are small and deterministic; once the Stryker host issue is cleared, this harness should be the right place to capture the actual mutation score and any surviving mutants.
