# UnityExplorer

UnityExplorer is a runtime hierarchy inspector and C# REPL for BepInEx-based Unity mods.

It is useful during DINOForge development because it lets you inspect live UI canvases, browse GameObjects and components, and inspect ECS-adjacent runtime state without restarting the game. That makes it especially helpful when tuning menu UI such as `MainMenuThemer` work or when tracing entity/component relationships at runtime.

## Installation

Install it through the DINOForge development tools workflow:

```powershell
dinoforge dev-tools install
```

This is an optional dependency. It is not required for normal mod development or for shipping gameplay content.

## Usage

- Press `F7` to toggle UnityExplorer in-game.
- Use the hierarchy tree to locate objects by name.
- Inspect attached components and serialized fields.
- Use the REPL to evaluate C# against live runtime objects.

## Updating the Pinned Version

The install scripts pin UnityExplorer to a specific release and verify the download via SHA256 before extracting. To update to a newer version:

1. Find the new release on https://github.com/sinai-dev/UnityExplorer/releases
2. Download `UnityExplorer.BepInEx5.Mono.zip` for that release
3. Compute its SHA256:
   - PowerShell: `(Get-FileHash .\UnityExplorer.BepInEx5.Mono.zip -Algorithm SHA256).Hash`
   - Bash: `sha256sum UnityExplorer.BepInEx5.Mono.zip`
4. Update `$PinnedVersion`, `$PinnedUrl`, and `$ExpectedSha256` in `scripts/install-dev-tools.ps1`
5. Update `PINNED_VERSION`, `PINNED_URL`, and `EXPECTED_SHA256` in `scripts/install-dev-tools.sh`

## Common Development Uses

- Inspect UI canvases while iterating on menu layout and theming.
- Verify which GameObjects and components are active in the current scene.
- Probe runtime state when diagnosing ECS integration and bridge behavior.

