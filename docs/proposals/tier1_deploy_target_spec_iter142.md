# Tier 1 MockSteamworks Deploy Target Spec

Status: SPEC ONLY - no project file changes in this document.

## Context

`src/Runtime/DINOForge.Runtime.csproj` already uses `DeployToGame` as the deployment template:
it defaults `DeployToGame` to `false`, redirects runtime output to `$(BepInExDir)\plugins`
when `GameInstalled=true` and `DeployToGame=true`, and runs `AfterTargets="Build"` copy
targets with `SkipUnchangedFiles="true"`.

`src/Tools/MockSteamworksNet/MockSteamworksNet.csproj` builds `MockSteamworksNet.dll`
as a BepInEx plugin when the game install is available.

## Proposed Target

Target name: `DeployMockSteamworksNet`.

Trigger condition: run after `Build` only when:

- `$(GameInstalled)` is `true`
- `$(DeployToGame)` is `true`
- `$(DeployMockSteamworks)` is `true`
- `$(BepInExDir)` exists

Gate flag: `-p:DeployMockSteamworks=true`.

This keeps mock Steam behavior opt-in even for normal deploy builds:

```powershell
dotnet build src/Tools/MockSteamworksNet/MockSteamworksNet.csproj -p:DeployToGame=true -p:DeployMockSteamworks=true
```

## Copy Contract

Copy source:

- `$(TargetPath)`, expected to resolve to the built `MockSteamworksNet.dll`

Copy destination:

- `$(BepInExDir)\plugins\`
- Equivalently, `{GameInstallPath}\BepInEx\plugins\MockSteamworksNet.dll`

No other files should be copied by this target. Steamworks.NET and BepInEx references remain resolved
from NuGet/game install according to the existing project references.

## Idempotency

- The target creates `$(BepInExDir)\plugins` only if it is missing.
- `Copy` uses `SkipUnchangedFiles="true"` so repeated builds do not rewrite an identical DLL.
- The destination filename is deterministic: `MockSteamworksNet.dll`.
- With `DeployMockSteamworks=false` or omitted, the target is inert and produces no game install changes.
- If `GameInstalled=false`, the target is inert and does not create partial plugin output.

## XML Snippet

Add this to `src/Tools/MockSteamworksNet/MockSteamworksNet.csproj`, following the same `AfterTargets="Build"`
pattern used by the runtime deploy targets:

```xml
<PropertyGroup>
  <DeployMockSteamworks Condition="'$(DeployMockSteamworks)' == ''">false</DeployMockSteamworks>
</PropertyGroup>

<Target Name="DeployMockSteamworksNet"
        AfterTargets="Build"
        Condition="'$(GameInstalled)' == 'true'
                   and '$(DeployToGame)' == 'true'
                   and '$(DeployMockSteamworks)' == 'true'
                   and Exists('$(BepInExDir)')">
  <PropertyGroup>
    <MockSteamworksPluginDir>$(BepInExDir)\plugins</MockSteamworksPluginDir>
  </PropertyGroup>

  <MakeDir Directories="$(MockSteamworksPluginDir)"
           Condition="!Exists('$(MockSteamworksPluginDir)')" />

  <Copy SourceFiles="$(TargetPath)"
        DestinationFiles="$(MockSteamworksPluginDir)\MockSteamworksNet.dll"
        SkipUnchangedFiles="true"
        Condition="Exists('$(TargetPath)')" />

  <Message Text="Deployed MockSteamworksNet.dll to $(MockSteamworksPluginDir)"
           Importance="high"
           Condition="Exists('$(TargetPath)')" />
</Target>
```
