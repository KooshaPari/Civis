# Mutation Testing Status

## Current State

Stryker.NET is blocked before mutant discovery. The configured mutation smoke path does not reach test execution, so there is no mutation score to report yet.

## Configured Stryker Files

- `stryker-smoke-config.json` is the checked-in Bridge smoke config used for the recent Bridge mutation attempts.
- `stryker-config.json` is the repo-root config, but it is stale for the installed Stryker version and uses the older root test-project shape.
- `StrykerConfig.json` is also present, but the Bridge mutation work used the smoke config above.

## Exact Blocking Error

Stryker 4.14.2 aborts during hosted discovery with the following error path:

```text
Could not load file or assembly 'System.Runtime, Version=8.0.0.0'
Could not load type 'Microsoft.VisualStudio.TestPlatform.ObjectModel.EqtTrace'
Project 'C:\Users\koosh\Dino\src\Tests\BridgeMutation\DINOForge.Tests.BridgeMutation.csproj' did not report any test.
```

The MTP path also failed before discovery:

```text
Failed to start test server for C:\Users\koosh\Dino\src\Tests\BridgeMutation\bin\Release\net8.0\DINOForge.Tests.BridgeMutation.dll
```

## What Was Tried

- Ran Stryker against the Bridge smoke project with the checked-in smoke config.
- Re-ran the same target with both `vstest` and `mtp` runner modes.
- Built the Bridge smoke test project directly with `dotnet test` to verify the project itself is healthy.
- Ran direct `dotnet vstest /ListTests` against the built `net8.0` BridgeMutation assembly.
- Added and used a focused `BridgeMutation` harness so Stryker would not need to traverse unrelated runtime areas.
- Tried a temporary harness-only workaround that pinned older test SDK packages and retargeted the harness to `net6.0`; direct test runs still worked, but Stryker discovery still failed, so that workaround was reverted.

## Root Cause

The blocker is the Stryker runner/toolchain path, not the Bridge mutation target itself.

- The Bridge smoke and Bridge mutation harnesses are `net8.0` test projects and pass under direct `dotnet test`.
- Stryker 4.14.2 is the component failing before discovery.
- The failure happens inside Stryker-hosted test discovery, where the bundled VSTest host cannot load `System.Runtime 8.0` and `EqtTrace`.
- The MTP host also fails to start, which makes this look like a Stryker/test-platform compatibility problem rather than a test code regression.

## Concrete Next Attempt

Attempt one of these, in this order:

1. Pin `dotnet-stryker` to a newer version and rerun with `dotnet-test` runner mode if available.
2. If that still fails, keep the Bridge mutation config pinned to `net8.0` and treat Stryker 4.14.2 as unsupported in this workspace.
3. Fall back to a different mutation tool or a non-Stryker mutation workflow for Bridge coverage evidence.

## Conclusion

Do not spend another long Stryker session on the current setup. The current evidence is sufficient to mark this as a toolchain compatibility blocker, not a Bridge code defect.
