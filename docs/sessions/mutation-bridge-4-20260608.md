# Mutation Bridge 4 - 2026-06-08

## Scope

- Re-ran Bridge-scoped Stryker after the tick12/tick15/tick16/tick17 Bridge tests.
- Mutation target:
  - `src/Bridge/Protocol/CanonicalJson.cs`
- Test harness:
  - `src/Tests/BridgeMutation/DINOForge.Tests.BridgeMutation.csproj`
  - `src/Tests/BridgeMutation/CanonicalJsonMutationTests.cs`

## Commands

```powershell
# Validate the focused mutation harness.
dotnet test src/Tests/BridgeMutation/DINOForge.Tests.BridgeMutation.csproj --configuration Release --verbosity minimal

# Stryker vstest attempt, scoped to Bridge Protocol / CanonicalJson.
dotnet stryker --config-file $env:TEMP\stryker-bridge-mutation-4.json --configuration Release --output mutation-report-bridge-4 --skip-version-check

# Stryker MTP attempt with the same scoped config.
dotnet stryker --config-file $env:TEMP\stryker-bridge-mutation-4.json --configuration Release --output mutation-report-bridge-4-mtp --skip-version-check --test-runner mtp

# Diagnostic run to capture the hosted VSTest failure.
dotnet stryker --config-file $env:TEMP\stryker-bridge-mutation-4.json --configuration Release --output mutation-report-bridge-4-diag --skip-version-check --verbosity trace --log-to-file --diag

# Direct VSTest discovery control check.
dotnet vstest src/Tests/BridgeMutation/bin/Release/net8.0/DINOForge.Tests.BridgeMutation.dll /ListTests
```

## Validation

- `dotnet test` passed:
  - Passed: `10`
  - Failed: `0`
  - Skipped: `0`
- Direct `dotnet vstest /ListTests` discovered the same BridgeMutation tests from the built `net8.0` assembly.

## Mutation Score

- Not produced.
- Stryker did not execute mutants because test discovery failed before the initial test run.

## Score Delta

- Previous Bridge mutation score: not produced in `mutation-bridge-3-20260608.md`.
- Current Bridge mutation score: not produced.
- Delta: unavailable.

## Un-Killed Summary

- Mutants executed: `0`
- Killed: `0`
- Survived: `0`
- Timed out: `0`

## Targeted Test Work

- No targeted survivor-killing tests were added in this pass.
- Reason: there was no Stryker survivor report. Adding tests without executed mutants would be guesswork and would not prove that the highest-cost survivor was killed.

## Blocker

- `dotnet-stryker` 4.14.2 still aborts during hosted test discovery for the BridgeMutation harness.
- The diagnostic `vstest` run inside Stryker deploys a bundled `net462` VSTest host and fails before discovery:

```text
Could not load file or assembly 'System.Runtime, Version=8.0.0.0'
Could not load type 'Microsoft.VisualStudio.TestPlatform.ObjectModel.EqtTrace'
Project 'C:\Users\koosh\Dino\src\Tests\BridgeMutation\DINOForge.Tests.BridgeMutation.csproj' did not report any test.
```

- The MTP runner also failed before discovery:

```text
Failed to start test server for C:\Users\koosh\Dino\src\Tests\BridgeMutation\bin\Release\net8.0\DINOForge.Tests.BridgeMutation.dll
```

## Notes

- I also tested a temporary harness-only workaround that pinned older test SDK packages and retargeted the harness to `net6.0`; direct `dotnet test` and direct `dotnet vstest /ListTests` still worked, but Stryker's hosted discovery still failed. That workaround was reverted.
- The remaining fix is likely in the Stryker/test-platform integration path, not in the BridgeMutation tests themselves.
