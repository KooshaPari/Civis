# Mutation Testing Status

## Decision

Keep Stryker.NET, but only against a dedicated `net8.0` Bridge mutation harness.

I evaluated the available mutation tools that are surfaced for this workspace:

- `dotnet-stryker-netx` exists, but its own package description says it is a .NET 10 port of Stryker.NET 4.14.1. That makes it a poor fit for this repo's mixed `net8.0` / `net11.0` test surface.
- `faultify` / `faultify.cli` are available as tools, but I could not find current compatibility evidence for this repo's test stack, and they are not already integrated here.

Given that, the least risky path is to keep the existing Stryker-based Bridge harness and keep the mutation scope pinned to `net8.0`.

## Chosen Path

Use the dedicated Bridge mutation project:

- `src/Tests/BridgeMutation/DINOForge.Tests.BridgeMutation.csproj`
- target project: `src/Bridge/Protocol/DINOForge.Bridge.Protocol.csproj`
- target framework: `net8.0`
- mutation scope: `src/Bridge/Protocol/CanonicalJson.cs`

## Exact Command And Config

Run from the repo root:

```powershell
dotnet stryker --config-file stryker-bridge-net8.json --configuration Release --skip-version-check
```

Temporary config content:

```json
{
  "stryker-config": {
    "project": "src/Bridge/Protocol/DINOForge.Bridge.Protocol.csproj",
    "test-projects": [
      "src/Tests/BridgeMutation/DINOForge.Tests.BridgeMutation.csproj"
    ],
    "target-framework": "net8.0",
    "test-runner": "vstest",
    "reporters": [
      "json",
      "progress"
    ],
    "thresholds": {
      "high": 0,
      "low": 0,
      "break": 0
    },
    "mutation-level": "Standard",
    "since": {
      "enabled": false
    },
    "report-file-name": "mutation-bridge-net8",
    "ignore-mutations": [
      "String",
      "Linq"
    ],
    "mutate": [
      "src/Bridge/Protocol/CanonicalJson.cs"
    ]
  }
}
```

## Smoke Check

Help-only smoke passed:

```powershell
dotnet stryker --help
```

That confirms the installed CLI exposes the needed `--project`, `--test-project`, `--target-framework`, and `--test-runner` options without starting a mutation run.

## Viability

**BLOCKED**.

Reason:

- The chosen path is structurally correct for this repo because the mutation target and test harness are both `net8.0`.
- The current Stryker 4.14.2 host still fails during test discovery in this workspace, before mutant execution starts.
- The installed non-Stryker alternatives do not provide a better supported path for this repo's current `net8.0` / `net11.0` test mix.

So the actionable status is: the config is ready, but the workspace is blocked on the Stryker/test-platform integration layer, not on the Bridge protocol code itself.
