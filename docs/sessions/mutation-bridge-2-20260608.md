# Mutation Bridge 2 - 2026-06-08

## Scope

- Re-ran Stryker against the Bridge protocol surface after the tick12/tick15 bridge tests landed.
- Mutation target:
  - `src/Bridge/Protocol/CanonicalJson.cs`
- Test harness used for the run attempt:
  - xUnit bridge mutation harness under `src/Tests/BridgeMutation/`

## Commands

```powershell
# Validate the small bridge mutation harness first
dotnet test src/Tests/BridgeMutation/DINOForge.Tests.BridgeMutation.csproj -c Release --no-restore --verbosity minimal

# Stryker attempts
dotnet stryker --config-file $env:TEMP\stryker-bridge-mutation.json --configuration Release --skip-version-check
dotnet stryker --config-file $env:TEMP\stryker-bridge-mutation.json --configuration Release --skip-version-check --test-runner mtp
```

## Mutation Score

- Not produced.
- Stryker completed analysis, but the run never reached mutant execution because test discovery failed.

## Mutation Delta

- N/A.
- No prior score could be compared because this run did not emit a mutation score.

## Un-Killed Summary

- Mutants executed: `0`
- Killed: `0`
- Survived: `0`
- Timed out: `0`

## Blocker

- `dotnet-stryker` 4.14.2 aborted in test discovery for both the MSTest bridge smoke harness and the xUnit bridge mutation harness.
- The same `Project '... did not report any test'` failure appeared under both `vstest` and `mtp`.
- Plain `dotnet test` on the same harnesses passed, so the blocker is specific to Stryker's discovery path in this workspace.

## Notes

- I did not add a permanent targeted test because no mutants were actually executed, so there was no survivor to target.
- Temporary build scaffolding used during the investigation was removed after the failed Stryker attempts.
