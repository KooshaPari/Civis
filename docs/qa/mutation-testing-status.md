# Mutation Testing Status

## Current State

Stryker.NET is blocked before mutant discovery. The configured mutation smoke path does not reach test execution, so there is no mutation score to report yet.

## Latest Pinning Attempt

- Requested pin: `dotnet-stryker` `3.13.1`
- Smoke result: `dotnet stryker --version` did not complete cleanly in the smoke window after the pin attempt
- Tooling blocker: `dotnet tool update -g dotnet-stryker --version 3.13.1` refused to downgrade the existing global install from `4.14.2`

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

1. Remove the existing global `dotnet-stryker` install and reinstall `3.13.1` explicitly, then rerun `dotnet stryker --version`.
2. If the downgrade still cannot be made to stick, treat Stryker 4.14.2 as unsupported in this workspace.
3. Fall back to a manual mutation-seed workflow for Bridge coverage evidence, using targeted hand-edited mutant seeds plus the existing Bridge tests to prove kill behavior without Stryker-hosted discovery.

## Conclusion

Do not spend another long Stryker session on the current setup. The current evidence is sufficient to mark this as a toolchain compatibility blocker, not a Bridge code defect.

## Status

**BLOCKED**: the requested `3.13.1` pin could not be applied over the existing global `4.14.2` install, so the version smoke could not be validated as loaded in this environment.

## Verdict (2026-06-10): BLOCKED — with concrete fix path

**Status: BLOCKED.** Stryker 4.14.2 (global) cannot load `System.Runtime 8.0.0.0`/`EqtTrace` to discover tests in the net8.0 `BridgeMutation` project, and a downgrade to 3.13.1 is refused by `dotnet tool update -g` (won't downgrade an existing global install).

**To unblock (needs one env change, not yet applied to avoid disrupting the swarm-shared toolchain):**
1. `dotnet tool uninstall -g dotnet-stryker` then `dotnet tool install -g dotnet-stryker --version 3.13.1` (clean downgrade), OR
2. Add a **local** tool manifest pinning dotnet-stryker 3.13.1 in `src/Tests/BridgeMutation/` (`dotnet new tool-manifest` + `dotnet tool install dotnet-stryker --version 3.13.1`) so the version is repo-scoped and doesn't touch the global install, then `dotnet stryker` from that dir.

**Autograder impact:** the Tier-2 Mature "mutation ≥85%" criterion stays `pass:false` with this documented rationale. Path (2) is the recommended fix — repo-local, swarm-safe, no global toolchain mutation.
